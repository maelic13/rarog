use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Output, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use rarog::board::Board;

/// Budget for waiting on engine output, scaled for the build profile.
///
/// A debug build searches roughly an order of magnitude slower than release,
/// and a CI runner is slower again than a dev box — so budgets calibrated
/// against `cargo test --release` locally are not safe in the CI debug matrix.
/// `long_endgame_search_does_not_overflow_engine_thread_stack` is the one that
/// caught us: `go movetime 100` with a flat 2 s budget passed in release for
/// months and failed the first CI debug run with `seen: []`, having produced no
/// output at all inside the window.
///
/// **This never weakens an assertion.** Every call site guarded by this helper
/// asserts that the engine eventually *does* something — emits `bestmove`,
/// reports `info depth` — never that it does so quickly. The two places that
/// genuinely assert timing are deliberately NOT scaled and say so at their call
/// site: `assert_no_line_containing` (where a longer window is stricter), and
/// the two ponderhit budgets that must stay below the ~1 s a restarted
/// `movetime` clock would take, since that is the bug they exist to detect.
fn wait(secs: u64) -> Duration {
    let scale = if cfg!(debug_assertions) { 8 } else { 1 };
    Duration::from_secs(secs * scale)
}

fn run_rarog(input: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rarog"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Rarog binary should start");

    child
        .stdin
        .as_mut()
        .expect("stdin should be piped")
        .write_all(input.as_bytes())
        .expect("test input should be written");
    drop(child.stdin.take());

    child.wait_with_output().expect("Rarog should exit")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

struct UciSession {
    child: Child,
    stdin: ChildStdin,
    stdout_rx: Receiver<String>,
}

impl UciSession {
    fn start() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_rarog"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Rarog binary should start");

        let stdin = child.stdin.take().expect("stdin should be piped");
        let stdout = child.stdout.take().expect("stdout should be piped");
        let (stdout_tx, stdout_rx) = mpsc::channel();

        thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if stdout_tx.send(line).is_err() {
                    break;
                }
            }
        });

        Self {
            child,
            stdin,
            stdout_rx,
        }
    }

    fn send(&mut self, command: &str) {
        writeln!(self.stdin, "{command}").expect("command should be written");
        self.stdin.flush().expect("command should be flushed");
    }

    fn expect_line_containing(&self, needle: &str, timeout: Duration) -> String {
        self.collect_until_line_containing(needle, timeout)
            .pop()
            .expect("matching line should be present")
    }

    fn collect_until_line_containing(&self, needle: &str, timeout: Duration) -> Vec<String> {
        let deadline = Instant::now() + timeout;
        let mut seen = Vec::new();
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                panic!("timed out waiting for `{needle}`; seen: {seen:?}");
            }
            match self.stdout_rx.recv_timeout(remaining) {
                Ok(line) if line.contains(needle) => {
                    seen.push(line);
                    return seen;
                }
                Ok(line) => seen.push(line),
                Err(err) => panic!("timed out waiting for `{needle}` ({err}); seen: {seen:?}"),
            }
        }
    }

    fn assert_no_line_containing(&self, needle: &str, duration: Duration) {
        let deadline = Instant::now() + duration;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return;
            }
            match self
                .stdout_rx
                .recv_timeout(remaining.min(Duration::from_millis(25)))
            {
                Ok(line) if line.contains(needle) => {
                    panic!("unexpected `{needle}` line before release command: {line}");
                }
                Ok(_) => {}
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    panic!("Rarog stdout closed while checking for absence of `{needle}`");
                }
            }
        }
    }

    fn quit(mut self) {
        self.send("quit");
        self.stdin.flush().expect("quit should be flushed");
        assert!(
            self.child
                .wait()
                .expect("Rarog process should be waitable")
                .success(),
            "Rarog should exit successfully"
        );
    }
}

impl Drop for UciSession {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn uci_advertises_ponder_and_core_options() {
    let output = run_rarog("uci\nquit\n");

    assert!(output.status.success(), "status: {:?}", output.status);
    let out = stdout(&output);
    assert!(out.contains("id name Rarog"));
    assert!(out.contains("option name Ponder type check default false"));
    assert!(out.contains("option name Hash type spin default 64 min 1 max 33554432"));
    assert!(out.contains("uciok"));
}

#[test]
fn completed_ponder_search_waits_for_ponderhit_before_bestmove() {
    let mut session = UciSession::start();
    session.send("uci");
    session.expect_line_containing("uciok", wait(15));
    session.send("position startpos moves e2e4");
    session.send("go ponder depth 1");

    session.expect_line_containing("info depth 1", wait(2));
    session.assert_no_line_containing("bestmove", Duration::from_millis(200));

    session.send("ponderhit");
    session.expect_line_containing("bestmove", wait(2));
    session.quit();
}

#[test]
fn completed_ponder_search_waits_for_stop_before_bestmove() {
    let mut session = UciSession::start();
    session.send("uci");
    session.expect_line_containing("uciok", wait(15));
    session.send("position startpos moves e2e4");
    session.send("go ponder depth 1");

    session.expect_line_containing("info depth 1", wait(2));
    session.assert_no_line_containing("bestmove", Duration::from_millis(200));

    session.send("stop");
    session.expect_line_containing("bestmove", wait(2));
    session.quit();
}

#[test]
fn ponderhit_after_spent_movetime_does_not_restart_search_clock() {
    let mut session = UciSession::start();
    session.send("uci");
    session.expect_line_containing("uciok", wait(15));
    session.send("position startpos moves e2e4");
    session.send("go ponder movetime 1000");

    thread::sleep(Duration::from_millis(1300));
    session.send("ponderhit");

    session.expect_line_containing("bestmove", Duration::from_millis(750));
    session.quit();
}

#[test]
fn movetime_100_completes_at_least_one_depth() {
    let mut session = UciSession::start();
    session.send("uci");
    session.expect_line_containing("uciok", wait(15));
    session.send("position startpos");
    session.send("go movetime 100");

    let lines = session.collect_until_line_containing("bestmove", wait(2));

    assert!(
        lines.iter().any(|line| line.starts_with("info depth 1 ")),
        "short movetime search should complete depth 1 before bestmove: {lines:?}"
    );
    assert!(
        lines
            .last()
            .is_some_and(|line| line.starts_with("bestmove ")),
        "search should finish with bestmove: {lines:?}"
    );
    session.quit();
}

#[test]
fn long_endgame_search_does_not_overflow_engine_thread_stack() {
    let mut session = UciSession::start();
    session.send("uci");
    session.expect_line_containing("uciok", wait(15));
    session.send("position fen 8/2Pq4/5K2/7k/8/6Q1/8/8 b - - 0 80");
    session.send("go movetime 100");

    let lines = session.collect_until_line_containing("bestmove", wait(2));

    assert!(
        lines.iter().any(|line| line.starts_with("info depth ")),
        "long queen endgame should search and return normally: {lines:?}"
    );
    session.quit();
}

#[test]
fn threaded_go_nodes_returns_bestmove_and_reports_nodes() {
    let mut session = UciSession::start();
    session.send("uci");
    session.expect_line_containing("uciok", wait(15));
    session.send("setoption name Threads value 4");
    session.send("isready");
    session.expect_line_containing("readyok", wait(5));
    session.send("position startpos");
    // Budget deliberately far above one iteration's cost. `info depth` is only
    // emitted when an iteration COMPLETES, and the node limit is shared across
    // threads — so at Threads=4 a 4096-node budget was consumed in ~1024 nodes
    // per thread, which is around what depth 1 costs from the start position.
    // Whether the iteration finished first was then a scheduling race, and it
    // lost on a windows release runner: the search returned `bestmove` having
    // emitted no `info` line at all. The assertion below is about node-limited
    // search WORKING under threads, not about the budget being tight, so the
    // fix is to stop racing. (Whether the engine should guarantee at least one
    // `info` line before every `bestmove` is a real question — filed as 10.6.)
    session.send("go nodes 200000");

    let lines = session.collect_until_line_containing("bestmove", wait(5));
    assert!(
        lines
            .iter()
            .any(|line| line.starts_with("info depth")
                && parse_uci_u64_field(line, "nodes").is_some()),
        "threaded node search should emit at least one info line with nodes: {lines:?}"
    );
    assert!(
        lines
            .last()
            .is_some_and(|line| line.starts_with("bestmove ")),
        "search should finish with bestmove: {lines:?}"
    );
    session.quit();
}

#[test]
fn threaded_infinite_search_stops_cleanly() {
    let mut session = UciSession::start();
    session.send("uci");
    session.expect_line_containing("uciok", wait(15));
    session.send("setoption name Threads value 4");
    session.send("isready");
    session.expect_line_containing("readyok", wait(5));
    session.send("position startpos");
    session.send("go infinite");

    session.expect_line_containing("info depth 1", wait(5));
    session.send("stop");
    session.expect_line_containing("bestmove", wait(5));
    session.quit();
}

#[test]
fn threaded_ponderhit_after_spent_movetime_does_not_restart_search_clock() {
    let mut session = UciSession::start();
    session.send("uci");
    session.expect_line_containing("uciok", wait(15));
    session.send("setoption name Threads value 4");
    session.send("isready");
    session.expect_line_containing("readyok", wait(5));
    session.send("position startpos moves e2e4");
    session.send("go ponder movetime 1000");

    thread::sleep(Duration::from_millis(1300));
    session.send("ponderhit");

    session.expect_line_containing("bestmove", Duration::from_millis(750));
    session.quit();
}

#[test]
fn emitted_pvs_are_legal_for_tournament_positions_with_threads() {
    let fens = [
        "2k5/pp3pp1/5n2/2P5/bPP2P2/P3K3/6Pp/3Q1B1R w - - 0 23",
        "2k5/pp3pp1/5n2/2P5/1PP2P2/P2BK1p1/2b3PP/7R w - - 2 24",
        "2k5/pp3pp1/5n2/2P5/1PP2P2/P2BK1p1/6PP/3b3R b - - 1 23",
    ];
    let mut session = UciSession::start();
    session.send("uci");
    session.expect_line_containing("uciok", wait(15));

    for threads in [1, 8] {
        session.send(&format!("setoption name Threads value {threads}"));
        session.send("isready");
        session.expect_line_containing("readyok", wait(5));

        for fen in fens {
            session.send(&format!("position fen {fen}"));
            session.send("go depth 4");

            let lines = session.collect_until_line_containing("bestmove", wait(10));
            assert_uci_pv_lines_are_legal(fen, &lines);
        }
    }

    session.quit();
}

#[test]
fn unknown_command_and_option_print_explicit_diagnostics() {
    let output = run_rarog("setoption name Not A Real Option value 1\nunknownthing\nquit\n");

    assert!(output.status.success(), "status: {:?}", output.status);
    let out = stdout(&output);
    assert!(out.contains("No such option: Not A Real Option"));
    assert!(out.contains("Unknown command: 'unknownthing'. Type help for more information."));
}

#[test]
fn go_perft_runs_synchronously_before_following_quit() {
    let output = run_rarog("go perft 1\nquit\n");

    assert!(output.status.success(), "status: {:?}", output.status);
    let out = stdout(&output);
    assert!(out.contains("Nodes searched: 20"));
    assert!(!out.contains("bestmove"), "{out}");
}

#[test]
fn position_accepts_uppercase_move_text_before_perft() {
    let output = run_rarog("position startpos moves E2E4\ngo perft 2\nquit\n");

    assert!(output.status.success(), "status: {:?}", output.status);
    let out = stdout(&output);
    assert!(out.contains("Nodes searched: 600"), "{out}");
}

#[test]
fn invalid_position_fen_is_a_critical_exit() {
    let output = run_rarog("position fen invalid\n");

    assert_eq!(output.status.code(), Some(1), "status: {:?}", output.status);
    let out = stdout(&output);
    assert!(out.contains(
        "info string CRITICAL ERROR: Command `position fen invalid` failed. Reason: Invalid FEN."
    ));
}

/// The last `info` line that carries a PV, and the `bestmove` after it.
fn last_pv_and_bestmove(lines: &[String]) -> (String, String) {
    let pv_move = lines
        .iter()
        .rev()
        .filter(|line| line.starts_with("info depth") && line.contains(" pv "))
        .find_map(|line| {
            line.split(" pv ")
                .nth(1)
                .and_then(|pv| pv.split_whitespace().next())
                .map(str::to_owned)
        })
        .unwrap_or_else(|| panic!("a search should report at least one PV: {lines:?}"));
    let best = lines
        .iter()
        .rev()
        .find(|line| line.starts_with("bestmove "))
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or_else(|| panic!("a search should end with bestmove: {lines:?}"))
        .to_owned();
    (pv_move, best)
}

/// A parallel search picks its move by vote across threads, but only the main
/// thread prints `info`. When the vote chose a helper's move, `bestmove` named
/// a move no printed line mentioned — 4 of 24 searches in the 2026-09-16
/// review, and the report in GitHub issue #1. The pool now prints the winner's
/// own line.
///
/// **A guard, not a proof.** Whether a given search's vote disagrees with the
/// main thread is a scheduling matter, so this cannot fail deterministically
/// on a broken build; it runs at the thread count and time control where the
/// review saw disagreement most often. The decision itself is pinned
/// deterministically by `only_a_helpers_line_is_reported_after_the_vote`.
#[test]
fn threaded_bestmove_is_the_move_the_last_info_line_reports() {
    let mut session = UciSession::start();
    session.send("uci");
    session.expect_line_containing("uciok", wait(15));
    session.send("setoption name Threads value 8");
    session.send("isready");
    session.expect_line_containing("readyok", wait(5));
    for fen in [
        "position startpos",
        "position fen r1bqkbnr/pppp1ppp/2n5/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 3 3",
        "position fen r2q1rk1/pp1bbppp/2np1n2/4p3/2B1P3/2NP1N2/PPP2PPP/R1BQ1RK1 w - - 0 9",
        "position fen 2rq1rk1/pb1nbppp/1p2pn2/2pp4/2PP4/1PN1PN2/PB2BPPP/R2Q1RK1 w - - 0 11",
        "position fen r1b1k2r/ppppqppp/2n2n2/2b5/2B1P3/2N2N2/PPPP1PPP/R1BQ1RK1 w kq - 6 6",
        "position fen rnbq1rk1/ppp1ppbp/3p1np1/8/2PPP3/2N2N2/PP2BPPP/R1BQK2R b KQ - 0 6",
    ] {
        session.send("ucinewgame");
        session.send(fen);
        session.send("isready");
        session.expect_line_containing("readyok", wait(5));
        session.send("go movetime 300");
        let lines = session.collect_until_line_containing("bestmove", wait(20));
        let (pv_move, best) = last_pv_and_bestmove(&lines);
        assert_eq!(
            pv_move, best,
            "the last info line must describe the move played ({fen}): {lines:?}"
        );
    }
    session.quit();
}

/// The engine's own `time` for a `go depth 4` from a lone-pawn KPK position,
/// read from the last `info depth` line.
fn kpk_search_ms(session: &mut UciSession) -> u64 {
    session.send("ucinewgame");
    session.send("position fen 8/8/8/4k3/8/3PK3/8/8 w - - 0 1");
    session.send("isready");
    session.expect_line_containing("readyok", wait(5));
    session.send("go depth 4");
    let lines = session.collect_until_line_containing("bestmove", wait(5));
    lines
        .iter()
        .rev()
        .filter(|line| line.starts_with("info depth"))
        .find_map(|line| parse_uci_u64_field(line, "time"))
        .unwrap_or_else(|| panic!("KPK search should report its time: {lines:?}"))
}

/// A fresh process's first search that reaches KPK must cost what the same
/// search costs warm. The bitbase was built on first probe, inside that search
/// and on its clock (about 34 ms in release), which lost games on time late in
/// fast games whenever a harness started a fresh engine for every game.
///
/// A genuine timing assertion, deliberately NOT scaled by `wait`: the margin
/// sits far above a depth-4 KPK search's noise in either profile and below
/// the cost of building the table in either profile.
#[test]
fn first_kpk_search_in_a_fresh_process_costs_what_a_warm_one_does() {
    let mut session = UciSession::start();
    session.send("uci");
    session.expect_line_containing("uciok", wait(15));
    let cold = kpk_search_ms(&mut session);
    let warm = kpk_search_ms(&mut session);
    assert!(
        cold <= warm + 15,
        "first KPK search took {cold} ms against {warm} ms warm: a table is being built inside the search"
    );
    session.quit();
}

fn parse_uci_u64_field(line: &str, field: &str) -> Option<u64> {
    let mut parts = line.split_whitespace();
    while let Some(part) = parts.next() {
        if part == field {
            return parts.next().and_then(|value| value.parse().ok());
        }
    }
    None
}

fn assert_uci_pv_lines_are_legal(root_fen: &str, lines: &[String]) {
    let mut saw_pv = false;
    for line in lines {
        let parts = line.split_whitespace().collect::<Vec<_>>();
        let Some(pv_index) = parts.iter().position(|part| *part == "pv") else {
            continue;
        };
        saw_pv = true;
        let mut board = Board::from_fen(root_fen).unwrap_or_else(|err| panic!("{root_fen}: {err}"));
        for mv_text in &parts[pv_index + 1..] {
            let mv = board.parse_move(mv_text).unwrap_or_else(|| {
                panic!(
                    "illegal PV move `{mv_text}` in `{}` from line `{line}`",
                    board.to_fen()
                )
            });
            board.make_move(mv);
        }
    }
    assert!(saw_pv, "search should emit at least one PV line: {lines:?}");
}
