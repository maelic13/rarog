use super::*;
use crate::search::options::EngineOptions;
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
fn only_quit_ends_the_engine_loop() {
    let (mut engine, _commands, control) = engine_fixture();

    let stop = EngineCommand::Stop {
        epoch: control.request_stop(),
    };
    assert_eq!(engine.handle(stop), SearchExit::Stop);

    let options = EngineOptions {
        hash_mb: 1,
        ..EngineOptions::default()
    };
    assert_eq!(
        engine.handle(EngineCommand::Configure {
            options,
            hash: crate::search::options::HashRequest::Sent,
            syzygy_path: crate::search::options::SyzygyPathRequest::NotSent,
        }),
        SearchExit::Stop
    );
    assert_eq!(engine.handle(EngineCommand::ClearHash), SearchExit::Stop);
    assert_eq!(engine.handle(EngineCommand::NewGame), SearchExit::Stop);

    let quit = EngineCommand::Quit {
        epoch: control.request_quit(),
    };
    assert_eq!(engine.handle(quit), SearchExit::Quit);
}

#[test]
fn search_converts_queued_stop_command_into_search_stop() {
    let (mut engine, _commands, control) = engine_fixture();
    control.request_stop();
    let mut options = SearchOptions::default();
    options.limits.depth = Some(99);

    let result = engine.search(&options, false, 0);

    assert_eq!(result.exit, SearchExit::Stop);
    assert!(result.nodes >= 512, "nodes: {}", result.nodes);
    assert!(result.depth < 99);
}

#[test]
fn search_converts_queued_quit_command_into_search_quit() {
    let (mut engine, _commands, control) = engine_fixture();
    control.request_quit();
    let mut options = SearchOptions::default();
    options.limits.depth = Some(99);

    let result = engine.search(&options, false, 0);

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
    let epoch = control.start_replacing_search();
    let mut options = SearchOptions::default();
    options.limits.ponder = true;
    let (done_tx, done_rx) = mpsc::channel();

    thread::Builder::new()
        .stack_size(crate::infra::THREAD_STACK_SIZE)
        .spawn(move || {
            let exit = engine.wait_until_bestmove_allowed(&options, epoch, false);
            done_tx.send(exit).expect("wait result should be sent");
        })
        .expect("spawn wait thread");

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

    thread::Builder::new()
        .stack_size(crate::infra::THREAD_STACK_SIZE)
        .spawn(move || {
            let exit = engine.wait_until_bestmove_allowed(&options, 0, false);
            done_tx.send(exit).expect("wait result should be sent");
        })
        .expect("spawn wait thread");

    assert!(done_rx.recv_timeout(Duration::from_millis(50)).is_err());
    control.request_stop();
    assert_eq!(
        done_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("stop should release the wait"),
        SearchExit::Stop
    );
}
