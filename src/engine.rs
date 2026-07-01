use std::io::{self, Write};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::bench::BENCH_FENS;
use crate::board::Board;
use crate::engine_command::{EngineCommand, EngineCommandQueue, EngineControl, SearchControl};
use crate::search::{SearchEvent, SearchExit, SearchResult, Searcher};
use crate::search_options::SearchOptions;

pub struct Engine {
    commands: EngineCommandQueue,
    control: Arc<EngineControl>,
    searcher: Searcher,
}

impl Engine {
    pub fn new(commands: EngineCommandQueue, control: Arc<EngineControl>) -> Engine {
        Engine {
            commands,
            control,
            searcher: Searcher::default(),
        }
    }

    pub fn start(&mut self) {
        loop {
            let command = self.commands.wait_pop();

            if self.handle_control_command(&command) {
                break;
            }
            if command.configure.is_some()
                || command.new_game
                || command.ponderhit
                || command.ready.is_some()
            {
                continue;
            }
            if command.stop {
                continue;
            }
            if let Some(depth) = command.bench_depth {
                if self.run_bench(
                    depth,
                    command.bench_repeats,
                    &command.search_options,
                    command.epoch,
                ) == SearchExit::Quit
                {
                    break;
                }
                continue;
            }

            if command.epoch != 0 && self.control.current_epoch() != command.epoch {
                continue;
            }
            if !self.control.prepare_search(command.epoch) {
                continue;
            }
            let result = self.search(command.search_options.clone(), true, command.epoch);
            let delayed_exit = if result.exit == SearchExit::Quit {
                SearchExit::Quit
            } else {
                self.wait_until_bestmove_allowed(
                    &command.search_options,
                    command.epoch,
                    result.ponderhit,
                )
            };
            self.control.finish_search_if_current(command.epoch);
            print_bestmove(&result);
            if result.exit == SearchExit::Quit || delayed_exit == SearchExit::Quit {
                break;
            }
        }
    }

    fn handle_control_command(&mut self, command: &EngineCommand) -> bool {
        if let Some(options) = &command.configure {
            self.searcher.configure(options);
        }
        if command.new_game {
            self.searcher.new_game();
        }
        if command.stop && (command.epoch == 0 || self.control.current_epoch() == command.epoch) {
            self.control.finish_search_if_current(command.epoch);
        }
        if let Some(ready) = &command.ready {
            let _ = ready.send(());
        }
        command.quit
    }

    fn search(&mut self, options: SearchOptions, emit_info: bool, epoch: u64) -> SearchResult {
        let control = Arc::clone(&self.control);
        self.searcher.search(
            options.position.board.clone(),
            &options,
            emit_info,
            || match control.poll_search() {
                SearchControl::Quit => SearchEvent::Quit,
                SearchControl::Stop if epoch == 0 || control.current_epoch() != epoch => {
                    SearchEvent::Stop
                }
                SearchControl::Stop => SearchEvent::Stop,
                SearchControl::PonderHit => SearchEvent::PonderHit,
                SearchControl::None => SearchEvent::None,
            },
        )
    }

    fn wait_until_bestmove_allowed(
        &self,
        options: &SearchOptions,
        epoch: u64,
        ponderhit_seen: bool,
    ) -> SearchExit {
        let waiting_on_ponder = options.limits.ponder && !ponderhit_seen;
        if !waiting_on_ponder && !options.limits.infinite {
            return SearchExit::Stop;
        }

        loop {
            match self.control.poll_search() {
                SearchControl::Quit => return SearchExit::Quit,
                SearchControl::Stop | SearchControl::PonderHit => return SearchExit::Stop,
                SearchControl::None => thread::sleep(Duration::from_millis(1)),
            }

            if epoch != 0 && self.control.current_epoch() != epoch {
                return SearchExit::Stop;
            }
        }
    }

    fn run_bench(
        &mut self,
        depth: u16,
        repeats: u16,
        base_options: &SearchOptions,
        epoch: u64,
    ) -> SearchExit {
        if epoch != 0 && self.control.current_epoch() != epoch {
            return SearchExit::Stop;
        }
        if !self.control.prepare_search(epoch) {
            return SearchExit::Stop;
        }
        let repeats = repeats.max(1);
        // Per-position detail is only printed for a single run; multi-run mode
        // (best-of-N NPS) prints one compact line per repeat instead.
        let detailed = repeats == 1;
        // Fingerprint / EBF / concentration are deterministic across repeats, so
        // they are captured from the first run; one NPS sample is kept per run.
        let mut fingerprint_nodes = 0u64;
        let mut geomean_ebf = 0f64;
        let mut median_nodes = 0u64;
        let mut max_nodes = 0u64;
        let mut total_ms_first = 0u128;
        let mut nps_samples: Vec<u128> = Vec::with_capacity(repeats as usize);

        println!();
        for repeat in 0..repeats {
            // Clean, identical starting state each repeat (TT + histories + eval
            // caches) so every run is the same deterministic workload — the only
            // NPS variation is machine noise — and the fingerprint is independent
            // of any prior engine state.
            self.searcher.new_game();
            let mut total_nodes = 0u64;
            let mut total_ms = 0u128;
            let mut per_position_nodes: Vec<u64> = Vec::with_capacity(BENCH_FENS.len());
            let mut ln_ebf_sum = 0f64;
            let mut ebf_count = 0usize;

            for (index, fen) in BENCH_FENS.iter().enumerate() {
                if epoch != 0 && self.control.current_epoch() != epoch {
                    self.control.finish_search_if_current(epoch);
                    return SearchExit::Stop;
                }
                let board = match Board::from_fen(fen) {
                    Ok(board) => board,
                    Err(err) => {
                        println!(
                            "info string bench position {} failed to parse: {}",
                            index + 1,
                            err
                        );
                        self.control.finish_search_if_current(epoch);
                        return SearchExit::Stop;
                    }
                };
                let mut options = SearchOptions::default();
                options.position.board = board;
                options.limits.depth = depth as f64;
                options.engine = base_options.engine.clone();

                let result = self.search(options, false, epoch);
                total_nodes += result.nodes;
                total_ms += result.elapsed_ms;
                per_position_nodes.push(result.nodes);
                // Per-position effective branching factor: nodes^(1/depth). Skip
                // positions solved before depth 1 (mates / trivial draws) so they
                // don't distort the geometric mean.
                let ebf = if result.depth >= 1 && result.nodes >= 1 {
                    let ebf = (result.nodes as f64).powf(1.0 / result.depth as f64);
                    ln_ebf_sum += ebf.ln();
                    ebf_count += 1;
                    ebf
                } else {
                    0.0
                };

                if detailed {
                    let nps = if result.elapsed_ms > 0 {
                        result.nodes as u128 * 1000 / result.elapsed_ms
                    } else {
                        result.nodes as u128
                    };
                    println!(
                        "bench {}/{}  depth {}  score {}  nodes {}  ebf {:.2}  time {}ms  nps {}",
                        index + 1,
                        BENCH_FENS.len(),
                        result.depth,
                        result.score,
                        result.nodes,
                        ebf,
                        result.elapsed_ms,
                        nps
                    );
                    flush_stdout();
                }

                if result.exit == SearchExit::Quit {
                    self.control.finish_search_if_current(epoch);
                    return SearchExit::Quit;
                }
                if epoch != 0 && self.control.current_epoch() != epoch {
                    self.control.finish_search_if_current(epoch);
                    return SearchExit::Stop;
                }
            }

            let run_nps = if total_ms > 0 {
                total_nodes as u128 * 1000 / total_ms
            } else {
                total_nodes as u128
            };
            nps_samples.push(run_nps);

            if repeat == 0 {
                fingerprint_nodes = total_nodes;
                total_ms_first = total_ms;
                geomean_ebf = if ebf_count > 0 {
                    (ln_ebf_sum / ebf_count as f64).exp()
                } else {
                    0.0
                };
                let mut sorted = per_position_nodes.clone();
                sorted.sort_unstable();
                median_nodes = sorted.get(sorted.len() / 2).copied().unwrap_or(0);
                max_nodes = per_position_nodes.iter().copied().max().unwrap_or(0);
            }
            if !detailed {
                println!(
                    "run {}/{}  nodes {}  time {}ms  nps {}",
                    repeat + 1,
                    repeats,
                    total_nodes,
                    total_ms,
                    run_nps
                );
                flush_stdout();
            }
        }

        // Robust diagnostics so the node total is read as a fingerprint, not a
        // strength/speed proxy (it is hypersensitive and non-monotonic to tiny
        // threshold changes — see PLAN.md §9). Geomean EBF is the selectivity
        // trend; median + top-share expose how concentrated the total is.
        let top_share = if fingerprint_nodes > 0 {
            max_nodes as f64 * 100.0 / fingerprint_nodes as f64
        } else {
            0.0
        };
        nps_samples.sort_unstable();
        let best_nps = nps_samples.last().copied().unwrap_or(0);
        let min_nps = nps_samples.first().copied().unwrap_or(0);
        let median_nps = nps_samples.get(nps_samples.len() / 2).copied().unwrap_or(0);

        println!(
            "\n=========================\n\
             Nodes searched  : {}\n\
             Geomean EBF     : {:.3}\n\
             Median nodes    : {}\n\
             Top-pos share   : {:.1}%  ({} nodes)",
            fingerprint_nodes, geomean_ebf, median_nodes, top_share, max_nodes
        );
        // Keep a line beginning "Nodes/second" for the single-run case — the PGO
        // training harness (xtask) waits for it as the completion marker.
        if repeats == 1 {
            println!(
                "Total time (ms) : {}\nNodes/second    : {}",
                total_ms_first, best_nps
            );
        } else {
            println!(
                "Nodes/second    : {}   (best of {}; median {}, min {})",
                best_nps, repeats, median_nps, min_nps
            );
        }
        flush_stdout();

        self.control.finish_search_if_current(epoch);
        SearchExit::Stop
    }
}

fn print_bestmove(result: &SearchResult) {
    if result.pondermove.is_null() {
        println!("bestmove {}", result.bestmove);
    } else {
        println!("bestmove {} ponder {}", result.bestmove, result.pondermove);
    }
    flush_stdout();
}

fn flush_stdout() {
    io::stdout().flush().expect("stdout flush failed");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Duration;

    fn engine_fixture() -> (Engine, EngineCommandQueue, Arc<EngineControl>) {
        let commands = EngineCommandQueue::default();
        let control = Arc::new(EngineControl::default());
        (
            Engine::new(commands.clone(), Arc::clone(&control)),
            commands,
            control,
        )
    }

    #[test]
    fn handle_control_command_returns_true_only_for_quit() {
        let (mut engine, _commands, control) = engine_fixture();

        assert!(!engine.handle_control_command(&EngineCommand::stop(control.request_stop())));

        let mut options = SearchOptions::default();
        options.engine.hash_mb = 1;
        options.engine.clear_hash = true;
        assert!(!engine.handle_control_command(&EngineCommand::configure(options)));
        assert!(!engine.handle_control_command(&EngineCommand::new_game()));
        assert!(!engine.handle_control_command(&EngineCommand::ponderhit()));

        assert!(engine.handle_control_command(&EngineCommand::quit(control.request_quit())));
    }

    #[test]
    fn search_converts_queued_stop_command_into_search_stop() {
        let (mut engine, _commands, control) = engine_fixture();
        control.request_stop();
        let mut options = SearchOptions::default();
        options.limits.depth = 99.0;

        let result = engine.search(options, false, 0);

        assert_eq!(result.exit, SearchExit::Stop);
        assert!(result.nodes >= 512, "nodes: {}", result.nodes);
        assert!(result.depth < 99);
    }

    #[test]
    fn search_converts_queued_quit_command_into_search_quit() {
        let (mut engine, _commands, control) = engine_fixture();
        control.request_quit();
        let mut options = SearchOptions::default();
        options.limits.depth = 99.0;

        let result = engine.search(options, false, 0);

        assert_eq!(result.exit, SearchExit::Quit);
        assert!(result.nodes >= 512, "nodes: {}", result.nodes);
        assert!(result.depth < 99);
    }

    #[test]
    fn bestmove_wait_releases_infinite_search_on_stop() {
        let (engine, _commands, control) = engine_fixture();
        let mut options = SearchOptions::default();
        options.limits.infinite = true;

        control.request_stop();

        assert_eq!(
            engine.wait_until_bestmove_allowed(&options, 0, false),
            SearchExit::Stop
        );
    }

    #[test]
    fn bestmove_wait_releases_infinite_search_on_quit() {
        let (engine, _commands, control) = engine_fixture();
        let mut options = SearchOptions::default();
        options.limits.infinite = true;

        control.request_quit();

        assert_eq!(
            engine.wait_until_bestmove_allowed(&options, 0, false),
            SearchExit::Quit
        );
    }

    #[test]
    fn bestmove_wait_does_not_rewait_after_ponderhit_seen_by_search() {
        let (engine, _commands, _control) = engine_fixture();
        let mut options = SearchOptions::default();
        options.limits.ponder = true;

        assert_eq!(
            engine.wait_until_bestmove_allowed(&options, 0, true),
            SearchExit::Stop
        );
    }

    #[test]
    fn bestmove_wait_blocks_ponder_search_until_ponderhit() {
        let (engine, _commands, control) = engine_fixture();
        let mut options = SearchOptions::default();
        options.limits.ponder = true;
        let (done_tx, done_rx) = mpsc::channel();

        thread::spawn(move || {
            let exit = engine.wait_until_bestmove_allowed(&options, 0, false);
            done_tx.send(exit).expect("wait result should be sent");
        });

        assert!(done_rx.recv_timeout(Duration::from_millis(50)).is_err());
        control.request_ponderhit();
        assert_eq!(
            done_rx
                .recv_timeout(Duration::from_secs(1))
                .expect("ponderhit should release the wait"),
            SearchExit::Stop
        );
    }

    #[test]
    fn bestmove_wait_blocks_infinite_search_until_stop() {
        let (engine, _commands, control) = engine_fixture();
        let mut options = SearchOptions::default();
        options.limits.infinite = true;
        let (done_tx, done_rx) = mpsc::channel();

        thread::spawn(move || {
            let exit = engine.wait_until_bestmove_allowed(&options, 0, false);
            done_tx.send(exit).expect("wait result should be sent");
        });

        assert!(done_rx.recv_timeout(Duration::from_millis(50)).is_err());
        control.request_stop();
        assert_eq!(
            done_rx
                .recv_timeout(Duration::from_secs(1))
                .expect("stop should release the wait"),
            SearchExit::Stop
        );
    }
}
