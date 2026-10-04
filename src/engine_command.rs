use std::collections::VecDeque;
use std::sync::{
    Arc, Condvar, Mutex,
    atomic::{AtomicBool, AtomicU64, Ordering},
    mpsc::Sender,
};

use crate::search_options::{EngineOptions, SearchOptions};

/// Signals from the protocol thread to the search. Every command that starts a
/// search takes a new epoch, and `stop` and `ponderhit` are scoped to the latest
/// one: they reach that search even when they arrive before the engine thread
/// starts it, and never reach a later one.
#[derive(Default)]
pub struct EngineControl {
    quit: AtomicBool,
    /// Searches with an epoch below this one are stopped.
    stopped_below: AtomicU64,
    /// The epoch a pending `ponderhit` belongs to; 0 when none is pending,
    /// since issued epochs start at 1.
    ponderhit: AtomicU64,
    searching: AtomicBool,
    epoch: AtomicU64,
}

impl EngineControl {
    /// Stop the latest search, started or still queued, without replacing it:
    /// a queued `go` still runs and answers with its `bestmove`.
    pub(crate) fn request_stop(&self) -> u64 {
        let epoch = self.current_epoch();
        self.stopped_below.fetch_max(epoch + 1, Ordering::AcqRel);
        epoch
    }

    pub(crate) fn request_quit(&self) -> u64 {
        let epoch = self.next_epoch();
        self.quit.store(true, Ordering::Release);
        epoch
    }

    /// Convert the latest search, started or still queued.
    pub(crate) fn request_ponderhit(&self) {
        self.ponderhit
            .store(self.current_epoch(), Ordering::Release);
    }

    /// Take the epoch for a new search command and stop every earlier search.
    pub(crate) fn start_replacing_search(&self) -> u64 {
        let epoch = self.next_epoch();
        self.searching.store(true, Ordering::Release);
        self.stopped_below.fetch_max(epoch, Ordering::AcqRel);
        epoch
    }

    /// False when a later command replaced this one before it started. Clears
    /// no signal: one that arrived since the command was issued belongs to it.
    pub(crate) fn prepare_search(&self, epoch: u64) -> bool {
        if epoch != 0 && self.current_epoch() != epoch {
            return false;
        }
        self.searching.store(true, Ordering::Release);
        true
    }

    pub(crate) fn finish_search_if_current(&self, epoch: u64) {
        if epoch == 0 || self.current_epoch() == epoch {
            self.searching.store(false, Ordering::Release);
        }
    }

    pub(crate) fn current_epoch(&self) -> u64 {
        self.epoch.load(Ordering::Acquire)
    }

    pub(crate) fn is_searching(&self) -> bool {
        self.searching.load(Ordering::Acquire)
    }

    /// Epoch 0 is a search outside the protocol's numbering, as in unit tests:
    /// any stop reaches it and no `ponderhit` converts it.
    pub(crate) fn poll_search(&self, epoch: u64) -> SearchControl {
        if self.quit.load(Ordering::Acquire) {
            SearchControl::Quit
        } else if epoch < self.stopped_below.load(Ordering::Acquire) {
            SearchControl::Stop
        } else if epoch != 0
            && self
                .ponderhit
                .compare_exchange(epoch, 0, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
        {
            SearchControl::PonderHit
        } else {
            SearchControl::None
        }
    }

    fn next_epoch(&self) -> u64 {
        self.epoch.fetch_add(1, Ordering::AcqRel) + 1
    }
}

pub(crate) enum SearchControl {
    None,
    Stop,
    Quit,
    PonderHit,
}

#[derive(Clone, Default)]
pub struct EngineCommandQueue {
    inner: Arc<QueueInner>,
}

#[derive(Default)]
struct QueueInner {
    commands: Mutex<VecDeque<EngineCommand>>,
    available: Condvar,
}

impl EngineCommandQueue {
    pub fn push(&self, command: EngineCommand) {
        {
            let mut commands = self.inner.commands.lock().expect("command queue poisoned");
            commands.push_back(command);
        }
        self.inner.available.notify_one();
    }

    pub(crate) fn push_priority(&self, command: EngineCommand) {
        {
            let mut commands = self.inner.commands.lock().expect("command queue poisoned");
            commands.push_front(command);
        }
        self.inner.available.notify_one();
    }

    pub(crate) fn wait_pop(&self) -> EngineCommand {
        let mut commands = self.inner.commands.lock().expect("command queue poisoned");
        loop {
            if let Some(command) = commands.pop_front() {
                return command;
            }
            commands = self
                .inner
                .available
                .wait(commands)
                .expect("command queue poisoned");
        }
    }
}

/// One unit of work for the engine thread, in queue order.
pub enum EngineCommand {
    Go {
        options: SearchOptions,
        epoch: u64,
    },
    Stop {
        epoch: u64,
    },
    Quit {
        epoch: u64,
    },
    /// Search the bench suite `repeats` times at a fixed depth.
    Bench {
        depth: u16,
        repeats: u16,
        options: SearchOptions,
        epoch: u64,
    },
    /// Search the WAC suite at a fixed depth.
    Wac {
        depth: u16,
        options: SearchOptions,
        epoch: u64,
    },
    Configure(EngineOptions),
    ClearHash,
    NewGame,
    /// The protocol thread has already raised the ponderhit flag; the queued
    /// command keeps the engine's view of the queue in UCI order.
    PonderHit,
    /// Answered once every earlier command has run.
    Ready(Sender<()>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_preparation_rejects_replaced_epochs() {
        let control = EngineControl::default();
        let replaced_epoch = control.start_replacing_search();
        let current_epoch = control.start_replacing_search();

        assert!(!control.prepare_search(replaced_epoch));
        assert!(matches!(
            control.poll_search(replaced_epoch),
            SearchControl::Stop
        ));
        control.finish_search_if_current(replaced_epoch);
        assert!(control.is_searching());

        assert!(control.prepare_search(current_epoch));
        assert!(matches!(
            control.poll_search(current_epoch),
            SearchControl::None
        ));
        control.finish_search_if_current(current_epoch);
        assert!(!control.is_searching());
    }

    #[test]
    fn stop_before_the_search_starts_stops_it_without_replacing_it() {
        let control = EngineControl::default();
        let epoch = control.start_replacing_search();
        let stop_epoch = control.request_stop();

        assert_eq!(stop_epoch, epoch);
        assert!(control.prepare_search(epoch));
        assert!(matches!(control.poll_search(epoch), SearchControl::Stop));
        control.finish_search_if_current(stop_epoch);
        assert!(!control.is_searching());

        let next_epoch = control.start_replacing_search();
        assert!(control.prepare_search(next_epoch));
        assert!(matches!(
            control.poll_search(next_epoch),
            SearchControl::None
        ));
    }

    #[test]
    fn ponderhit_before_the_search_starts_converts_that_search_once() {
        let control = EngineControl::default();
        let epoch = control.start_replacing_search();
        control.request_ponderhit();

        assert!(control.prepare_search(epoch));
        assert!(matches!(
            control.poll_search(epoch),
            SearchControl::PonderHit
        ));
        assert!(matches!(control.poll_search(epoch), SearchControl::None));
    }

    #[test]
    fn ponderhit_never_reaches_a_later_search() {
        let control = EngineControl::default();
        control.request_ponderhit();
        let first_epoch = control.start_replacing_search();
        assert!(matches!(
            control.poll_search(first_epoch),
            SearchControl::None
        ));

        control.request_ponderhit();
        let second_epoch = control.start_replacing_search();
        assert!(control.prepare_search(second_epoch));
        assert!(matches!(
            control.poll_search(second_epoch),
            SearchControl::None
        ));
    }
}
