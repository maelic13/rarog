use super::*;

use crate::board::{Color, Piece, Square};

fn args(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|part| (*part).to_string()).collect()
}

fn protocol_fixture() -> (UciProtocol, EngineCommandQueue) {
    let commands = EngineCommandQueue::default();
    let control = Arc::new(EngineControl::default());
    (UciProtocol::new(commands.clone(), control), commands)
}

#[test]
fn handle_command_reports_what_it_did() {
    let (mut protocol, _commands) = protocol_fixture();
    assert_eq!(protocol.handle_command("uci"), CommandOutcome::Handled);
    // Not `isready`: it blocks on a reply from the engine thread, and this
    // fixture deliberately has none.
    // Blank and whitespace-only lines are not errors; the interactive loop
    // skips them and argv must agree.
    assert_eq!(protocol.handle_command(""), CommandOutcome::Handled);
    assert_eq!(protocol.handle_command("   "), CommandOutcome::Handled);
    assert_eq!(
        protocol.handle_command("notacommand"),
        CommandOutcome::Unknown
    );
    assert_eq!(protocol.handle_command("quit"), CommandOutcome::Quit);
}

#[test]
fn run_once_queues_the_eof_style_quit_not_the_interactive_one() {
    // `quit` from the keyboard uses `push_priority`, which jumps ahead of
    // a dispatched `bench` and cuts it short; EOF uses an ordinary push
    // that the bench completes before. `run_once` must use the latter, or
    // `rarog bench 13` benches nothing.
    let (mut protocol, commands) = protocol_fixture();
    assert_eq!(protocol.run_once("bench 1"), CommandOutcome::Handled);

    // `bench` enqueues a stop ahead of the bench itself.
    assert!(matches!(commands.wait_pop(), EngineCommand::Stop { .. }));
    assert!(
        matches!(commands.wait_pop(), EngineCommand::Bench { depth: 1, .. }),
        "the bench must be queued"
    );
    assert!(
        matches!(commands.wait_pop(), EngineCommand::Quit { epoch: 0 }),
        "epoch 0 is the EOF-style quit; the interactive one carries the \
             control's epoch, is pushed with priority, and pre-empts the bench"
    );
}

#[test]
fn help_is_recognised_under_every_alias() {
    let (mut protocol, _commands) = protocol_fixture();
    // `unknown_command` tells users to "Type help", so every alias must be
    // a command.
    for alias in ["help", "--help", "-h", "license", "--license"] {
        assert_eq!(
            protocol.handle_command(alias),
            CommandOutcome::Handled,
            "`{alias}` must be recognised"
        );
    }
}

#[test]
fn help_text_orients_rather_than_lists_the_protocol() {
    let text = help_text();
    // The four things orientation has to cover.
    assert!(text.contains("chess engine"), "says what it is");
    assert!(
        text.contains("General Public License"),
        "states the licence"
    );
    assert!(text.contains("Universal Chess Interface"), "names UCI");
    assert!(text.contains("github.com"), "points somewhere further");
    // Plus the one thing a shell user cannot learn elsewhere.
    assert!(text.contains("rarog bench 13"), "shows the argument form");
    // And NOT a second copy of the UCI specification.
    for protocol_command in ["isready", "ucinewgame", "ponderhit", "setoption"] {
        assert!(
            !text.contains(protocol_command),
            "help must not restate the UCI spec, found `{protocol_command}`"
        );
    }
}

#[test]
fn run_once_on_quit_does_not_queue_a_second_quit() {
    let (mut protocol, commands) = protocol_fixture();
    assert_eq!(protocol.run_once("quit"), CommandOutcome::Quit);
    assert!(matches!(commands.wait_pop(), EngineCommand::Quit { .. }));
}

#[test]
fn go_sends_current_position_with_search_parameters() {
    let (mut protocol, commands) = protocol_fixture();

    assert_eq!(
        protocol.handle_command("position startpos moves e2e4"),
        CommandOutcome::Handled
    );
    protocol.go(&args(&["depth", "3", "nodes", "123"]));

    let EngineCommand::Go { options, epoch } = commands.wait_pop() else {
        panic!("go must queue a search");
    };
    assert!(epoch > 0);
    assert_eq!(options.limits.depth, Some(3));
    assert_eq!(options.limits.nodes, 123);
    assert_eq!(options.board.side_to_move(), Color::Black);
    assert_eq!(
        options.board.piece_at(Square::E4),
        Some((Color::White, Piece::Pawn))
    );
}

#[test]
fn setoption_sends_configure_and_clear_hash_sends_its_own_command() {
    let (mut protocol, commands) = protocol_fixture();
    let configured = |command: EngineCommand| match command {
        EngineCommand::Configure {
            options,
            hash,
            syzygy_path,
        } => (options, hash, syzygy_path),
        _ => panic!("setoption must configure the engine"),
    };

    protocol.set_option(&args(&["name", "Hash", "value", "8"]));
    let (options, hash, syzygy_path) = configured(commands.wait_pop());
    assert_eq!(
        (options.hash_mb, hash, syzygy_path),
        (8, HashRequest::Sent, SyzygyPathRequest::NotSent)
    );
    // The same value again is still a request: the engine thread tries it.
    protocol.set_option(&args(&["name", "Hash", "value", "8"]));
    assert_eq!(configured(commands.wait_pop()).1, HashRequest::Sent);

    protocol.set_option(&args(&["name", "Clear", "Hash"]));
    assert!(matches!(commands.wait_pop(), EngineCommand::ClearHash));

    protocol.set_option(&args(&[
        "name",
        "SyzygyPath",
        "value",
        "D:\\TB",
        "MixedCase",
    ]));
    let (options, hash, syzygy_path) = configured(commands.wait_pop());
    assert_eq!(options.syzygy.path, "D:\\TB MixedCase");
    assert_eq!(
        (hash, syzygy_path),
        (HashRequest::NotSent, SyzygyPathRequest::Sent)
    );

    protocol.set_option(&args(&["name", "Ponder", "value", "true"]));
    let (options, hash, syzygy_path) = configured(commands.wait_pop());
    assert!(options.ponder);
    assert_eq!(
        (hash, syzygy_path),
        (HashRequest::NotSent, SyzygyPathRequest::NotSent)
    );
}

#[test]
fn bench_sends_stop_before_bench_command() {
    let (mut protocol, commands) = protocol_fixture();

    protocol.bench(&args(&["5"]));

    let EngineCommand::Stop { epoch: stop_epoch } = commands.wait_pop() else {
        panic!("bench must stop the current search first");
    };
    let EngineCommand::Bench {
        depth,
        options,
        epoch,
        ..
    } = commands.wait_pop()
    else {
        panic!("bench must queue the bench");
    };
    assert_eq!(depth, 5);
    assert_eq!(epoch, stop_epoch);
    assert_eq!(options.engine.threads, 1);
}

#[test]
fn newgame_resets_position_and_sends_marker_command() {
    let (mut protocol, commands) = protocol_fixture();

    assert_eq!(
        protocol.handle_command("position startpos moves e2e4"),
        CommandOutcome::Handled
    );
    assert_eq!(
        protocol.search_options.board.piece_at(Square::E4),
        Some((Color::White, Piece::Pawn))
    );

    protocol.new_game();

    assert!(matches!(commands.wait_pop(), EngineCommand::NewGame));
    assert_eq!(
        protocol.search_options.board.piece_at(Square::E2),
        Some((Color::White, Piece::Pawn))
    );
    assert_eq!(protocol.search_options.board.piece_at(Square::E4), None);
}

#[test]
fn control_commands_are_forwarded_to_engine_thread() {
    let (mut protocol, commands) = protocol_fixture();

    protocol.stop();
    protocol.quit();

    assert!(matches!(commands.wait_pop(), EngineCommand::Quit { .. }));
    assert!(matches!(commands.wait_pop(), EngineCommand::Stop { .. }));
}
