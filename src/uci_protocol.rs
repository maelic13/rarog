use std::io::{self, Write};
use std::process;
use std::sync::{Arc, mpsc};

use crate::bench::DEFAULT_BENCH_DEPTH;
use crate::engine_command::{EngineCommand, EngineCommandQueue, EngineControl};
use crate::infra::capitalize_first_letter;
use crate::search_options::SearchOptions;
use crate::wac::DEFAULT_WAC_DEPTH;

/// What one command line did, so a caller knows whether to keep reading and
/// what to exit with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandOutcome {
    /// Recognised and dispatched.
    Handled,
    /// `quit`: stop reading input.
    Quit,
    /// Not a command. A message has already been printed.
    Unknown,
}

pub struct UciProtocol {
    search_options: SearchOptions,
    commands: EngineCommandQueue,
    control: Arc<EngineControl>,
}

impl UciProtocol {
    pub fn new(commands: EngineCommandQueue, control: Arc<EngineControl>) -> UciProtocol {
        UciProtocol {
            search_options: SearchOptions::default(),
            commands,
            control,
        }
    }

    pub fn uci_loop(&mut self) {
        loop {
            let mut input = String::new();
            // 9.0a: a read error means stdin is gone (pipe closed / GUI
            // exited) — treated exactly like EOF, which the next branch
            // already handles, rather than panicking.
            let bytes_read = io::stdin().read_line(&mut input).unwrap_or_default();
            if bytes_read == 0 {
                self.commands.push(EngineCommand::quit(0));
                break;
            }
            if self.handle_command(input.trim()) == CommandOutcome::Quit {
                break;
            }
        }
    }

    /// Run one command line and shut down, as `rarog bench 13` does.
    ///
    /// A.4.5. The shutdown deliberately mirrors **stdin EOF** and not the
    /// interactive `quit`, and the difference is not cosmetic: `quit` calls
    /// `control.request_quit()` and `push_priority`, which jumps the queue and
    /// cuts short work already dispatched to the engine thread, while EOF
    /// pushes an ordinary FIFO `quit` that a running `bench` completes ahead
    /// of. That asymmetry is exactly why feeding `bench 13\nquit` on stdin
    /// benches nothing while `bench 13` alone works.
    pub fn run_once(&mut self, command_line: &str) -> CommandOutcome {
        let outcome = self.handle_command(command_line);
        if outcome != CommandOutcome::Quit {
            self.commands.push(EngineCommand::quit(0));
        }
        outcome
    }

    /// Dispatch one command line, exactly as the interactive loop does.
    pub fn handle_command(&mut self, command_line: &str) -> CommandOutcome {
        let input: Vec<String> = command_line
            .split_whitespace()
            .map(str::to_string)
            .collect();
        if input.is_empty() {
            return CommandOutcome::Handled;
        }
        let command: &str = &input[0];
        let args: &[String] = &input[1..];

        match command {
            "uci" => self.uci(),
            "isready" => self.is_ready(),
            "go" => self.go(args),
            "stop" => self.stop(),
            "setoption" => self.set_option(args),
            "ucinewgame" => self.new_game(),
            "position" => self.position_with_command(args, command_line),
            "bench" => self.bench(args),
            "wac" => self.wac(args),
            #[cfg(feature = "tune")]
            "dumpeval" => self.dump_eval(),
            "ponderhit" => self.ponderhit(),
            "help" | "--help" | "-h" | "license" | "--license" => self.help(),
            "quit" => {
                self.quit();
                return CommandOutcome::Quit;
            }
            _ => {
                self.unknown_command(command_line);
                return CommandOutcome::Unknown;
            }
        }
        CommandOutcome::Handled
    }

    fn uci(&self) {
        println!(
            "id name {} {}",
            capitalize_first_letter(env!("CARGO_PKG_NAME")),
            env!("CARGO_PKG_VERSION")
        );
        println!("id author {}", env!("CARGO_PKG_AUTHORS").replace(':', ", "));
        for option in SearchOptions::get_uci_options() {
            println!("{option}");
        }
        println!("uciok");
        flush_stdout();
    }

    fn is_ready(&self) {
        if !self.control.is_searching() {
            let (ready_tx, ready_rx) = mpsc::channel();
            self.commands.push(EngineCommand::ready(ready_tx));
            let _ = ready_rx.recv();
        }
        println!("readyok");
        flush_stdout();
    }

    fn quit(&self) {
        let epoch = self.control.request_quit();
        self.commands.push_priority(EngineCommand::quit(epoch));
    }

    fn go(&mut self, args: &[String]) {
        self.search_options.set_search_parameters(args);
        if self.search_options.limits.perft > 0 {
            self.run_perft(self.search_options.limits.perft);
            return;
        }

        let epoch = self.control.start_replacing_search();
        self.commands
            .push(EngineCommand::go(self.search_options.clone(), epoch));
    }

    fn stop(&mut self) {
        let epoch = self.control.request_stop();
        self.commands.push(EngineCommand::stop(epoch));
    }

    fn set_option(&mut self, args: &[String]) {
        self.wait_for_search_finished();
        if self.search_options.set_option(args) {
            self.commands
                .push(EngineCommand::configure(self.search_options.clone()));
            self.search_options.engine.clear_hash = false;
        }
    }

    fn new_game(&mut self) {
        self.search_options.reset();
        self.commands.push(EngineCommand::new_game());
    }

    fn position_with_command(&mut self, args: &[String], full_command: &str) {
        if let Err(message) = self.search_options.set_position(args) {
            terminate_on_critical_error(full_command, &message);
        }
    }

    fn bench(&mut self, args: &[String]) {
        let depth = args
            .first()
            .and_then(|depth| depth.parse::<u16>().ok())
            .unwrap_or(DEFAULT_BENCH_DEPTH);
        // Optional second arg: repeat the whole suite N times for a best-of-N
        // NPS read (`bench <depth> <repeats>`). The fingerprint is identical
        // every repeat; only NPS varies (machine noise). Default 1.
        let repeats = args
            .get(1)
            .and_then(|r| r.parse::<u16>().ok())
            .unwrap_or(1)
            .max(1);

        let epoch = self.control.start_replacing_search();
        self.commands.push(EngineCommand::stop(epoch));
        self.commands.push(EngineCommand::bench(
            depth,
            repeats,
            self.search_options.clone(),
            epoch,
        ));
    }

    /// `wac [depth]` — run the WAC tactical suite at a fixed depth (default
    /// 10) and report the solved count. A diagnostic like `bench`, not a gate.
    fn wac(&mut self, args: &[String]) {
        let depth = args
            .first()
            .and_then(|depth| depth.parse::<u16>().ok())
            .unwrap_or(DEFAULT_WAC_DEPTH);

        let epoch = self.control.start_replacing_search();
        self.commands.push(EngineCommand::stop(epoch));
        self.commands.push(EngineCommand::wac(
            depth,
            self.search_options.clone(),
            epoch,
        ));
    }

    #[cfg(feature = "tune")]
    fn dump_eval(&self) {
        let params = crate::eval::EvalParams::load_from_env();
        print!("{}", params.dump());
        flush_stdout();
    }

    fn ponderhit(&mut self) {
        self.control.request_ponderhit();
        self.commands.push(EngineCommand::ponderhit());
    }

    fn run_perft(&self, depth: u32) {
        let mut board = self.search_options.position.board.clone();
        let nodes = board.perft(depth);
        println!("\nNodes searched: {nodes}\n");
        flush_stdout();
    }

    fn wait_for_search_finished(&self) {
        if !self.control.is_searching() {
            return;
        }
        let (ready_tx, ready_rx) = mpsc::channel();
        self.commands.push(EngineCommand::ready(ready_tx));
        let _ = ready_rx.recv();
    }

    /// `help` — orientation, not a manual.
    ///
    /// Deliberately short: what the engine is, its licence, that it is normally
    /// driven by a GUI, and where to read more. A full command reference here
    /// would be a second copy of the UCI specification, going stale against
    /// `README` and the protocol both.
    ///
    /// What it does list is the handful of commands a person types by hand,
    /// and the fact that they work as arguments — someone who reached this text
    /// by typing `rarog help` at a shell is exactly who needs telling.
    fn help(&self) {
        println!("{}", help_text());
        flush_stdout();
    }

    fn unknown_command(&self, command_line: &str) {
        if command_line.is_empty() || command_line.starts_with('#') {
            return;
        }
        println!("Unknown command: '{command_line}'. Type help for more information.");
        flush_stdout();
    }
}

/// The text `help` prints.
///
/// Separated from the printing so its content is testable without capturing
/// stdout, and `const` so it costs nothing when unused.
const fn help_text() -> &'static str {
    concat!(
        "\n",
        "Rarog is a chess engine for playing and analysing chess.\n",
        "It is free software, licensed under the GNU General Public License v3 or later.\n",
        "\n",
        "Rarog speaks the Universal Chess Interface (UCI) protocol and is normally used\n",
        "from a chess GUI rather than typed at directly. Any UCI-compatible GUI will do.\n",
        "\n",
        "Beyond the UCI commands a GUI sends, these are useful by hand:\n",
        "\n",
        "  bench [depth] [repeats]   Search a fixed suite of positions. The node count is\n",
        "                            identical on every platform, so it tells you a build\n",
        "                            is correct; the speed tells you how fast this machine\n",
        "                            is. Defaults to depth 13.\n",
        "  wac [depth]               Run the WAC tactical suite and report how many it\n",
        "                            solved. A diagnostic, not a rating.\n",
        "  help                      This text.\n",
        "  quit                      Exit.\n",
        "\n",
        "These work as command-line arguments too, so `rarog bench 13` runs one command\n",
        "and exits. An unrecognised argument exits with status 2.\n",
        "\n",
        "For more, see ",
        env!("CARGO_PKG_REPOSITORY"),
        "#readme\n",
    )
}

fn flush_stdout() {
    // 9.0a: a failed flush means the GUI closed the pipe — a normal way for a
    // UCI session to end, not a bug. Panicking here aborted the process
    // (release sets `panic = "abort"`), turning an ordinary disconnect into a
    // crash; the write is simply dropped instead.
    let _ = io::stdout().flush();
}

fn terminate_on_critical_error(full_command: &str, message: &str) -> ! {
    crate::info_string!("CRITICAL ERROR: Command `{full_command}` failed. Reason: {message}");
    flush_stdout();
    process::exit(1);
}

#[cfg(test)]
mod tests {
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
        // has always skipped them and argv must agree.
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
        // A.4.5's load-bearing detail. `quit` from the keyboard uses
        // `push_priority`, which jumps ahead of a dispatched `bench` and cuts
        // it short; EOF uses an ordinary push that the bench completes before.
        // `run_once` must use the latter, or `rarog bench 13` benches nothing -
        // which is the exact bug this leaf exists to fix.
        let (mut protocol, commands) = protocol_fixture();
        assert_eq!(protocol.run_once("bench 1"), CommandOutcome::Handled);

        // `bench` enqueues a stop ahead of the bench itself.
        let stop = commands.wait_pop();
        assert!(stop.stop && !stop.quit && stop.bench_depth.is_none());

        let bench = commands.wait_pop();
        assert_eq!(bench.bench_depth, Some(1), "the bench must be queued");
        assert!(!bench.quit, "the bench command must not carry quit");

        let quit = commands.wait_pop();
        assert!(quit.quit, "a quit must follow the bench");
        assert_eq!(
            quit.epoch, 0,
            "epoch 0 is the EOF-style quit; the interactive one carries the \
             control's epoch, is pushed with priority, and pre-empts the bench"
        );
    }

    #[test]
    fn help_is_recognised_under_every_alias() {
        let (mut protocol, _commands) = protocol_fixture();
        // `unknown_command` has always told users to "Type help"; before this
        // existed, doing so answered "Unknown command: 'help'".
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
        assert!(commands.wait_pop().quit);
    }

    #[test]
    fn go_sends_current_position_with_search_parameters() {
        let (mut protocol, commands) = protocol_fixture();

        protocol.position_with_command(
            &args(&["startpos", "moves", "e2e4"]),
            "position startpos moves e2e4",
        );
        protocol.go(&args(&["depth", "3", "nodes", "123"]));

        let command = commands.wait_pop();
        assert!(!command.stop);
        assert!(!command.quit);
        assert!(command.epoch > 0);
        assert_eq!(command.search_options.limits.depth, Some(3));
        assert_eq!(command.search_options.limits.nodes, 123);
        assert_eq!(
            command.search_options.position.board.side_to_move(),
            Color::Black
        );
        assert_eq!(
            command.search_options.position.board.piece_at(Square::E4),
            Some((Color::White, Piece::Pawn))
        );
    }

    #[test]
    fn setoption_sends_configure_and_clears_clear_hash_button_state() {
        let (mut protocol, commands) = protocol_fixture();

        protocol.set_option(&args(&["name", "Hash", "value", "8"]));
        let hash_command = commands.wait_pop();
        assert_eq!(
            hash_command
                .configure
                .expect("hash command must configure engine")
                .engine
                .hash_mb,
            8
        );

        protocol.set_option(&args(&["name", "Clear", "Hash"]));
        let clear_command = commands.wait_pop();
        assert!(
            clear_command
                .configure
                .expect("clear hash command must configure engine")
                .engine
                .clear_hash
        );
        assert!(!protocol.search_options.engine.clear_hash);

        protocol.set_option(&args(&[
            "name",
            "SyzygyPath",
            "value",
            "D:\\TB",
            "MixedCase",
        ]));
        let syzygy_command = commands.wait_pop();
        assert_eq!(
            syzygy_command
                .configure
                .expect("syzygy path command must configure engine")
                .engine
                .syzygy
                .path,
            "D:\\TB MixedCase"
        );

        protocol.set_option(&args(&["name", "Ponder", "value", "true"]));
        let ponder_command = commands.wait_pop();
        assert!(
            ponder_command
                .configure
                .expect("ponder command must configure engine")
                .engine
                .ponder
        );
    }

    #[test]
    fn bench_sends_stop_before_bench_command() {
        let (mut protocol, commands) = protocol_fixture();

        protocol.bench(&args(&["5"]));

        let stop = commands.wait_pop();
        assert!(stop.stop);
        assert!(stop.bench_depth.is_none());

        let bench = commands.wait_pop();
        assert!(!bench.stop);
        assert_eq!(bench.bench_depth, Some(5));
        assert_eq!(bench.epoch, stop.epoch);
        assert_eq!(bench.search_options.engine.threads, 1);
    }

    #[test]
    fn newgame_resets_position_and_sends_marker_command() {
        let (mut protocol, commands) = protocol_fixture();

        protocol.position_with_command(
            &args(&["startpos", "moves", "e2e4"]),
            "position startpos moves e2e4",
        );
        assert_eq!(
            protocol.search_options.position.board.piece_at(Square::E4),
            Some((Color::White, Piece::Pawn))
        );

        protocol.new_game();

        let command = commands.wait_pop();
        assert!(command.new_game);
        assert_eq!(
            protocol.search_options.position.board.piece_at(Square::E2),
            Some((Color::White, Piece::Pawn))
        );
        assert_eq!(
            protocol.search_options.position.board.piece_at(Square::E4),
            None
        );
    }

    #[test]
    fn control_commands_are_forwarded_to_engine_thread() {
        let (mut protocol, commands) = protocol_fixture();

        protocol.stop();
        protocol.ponderhit();
        protocol.quit();

        let quit = commands.wait_pop();
        assert!(quit.stop);
        assert!(quit.quit);
        assert!(commands.wait_pop().stop);
        assert!(commands.wait_pop().ponderhit);
    }
}
