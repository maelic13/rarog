//! 9.5 fuzz-lite: malformed input must be REJECTED, never panic.
//!
//! Two reasons this matters more than usual here. The release profile sets
//! `panic = "abort"`, so a panic on a malformed `position` line is not a
//! caught error — it is process death mid-game, scored as a loss. And the
//! parser is the one surface a GUI can drive with arbitrary bytes.
//!
//! The strong property asserted below is not merely "does not panic" but:
//! **any FEN that parses successfully must yield a self-consistent board.**
//! A parser that accepts garbage and builds a subtly broken position is worse
//! than one that rejects it, because everything downstream then trusts it.

use rarog::board::Board;

const VALID: [&str; 4] = [
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
    "rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3",
];

/// Hand-picked structural breakages, each targeting a different parser stage.
const MALFORMED: [&str; 22] = [
    "",
    " ",
    "not a fen at all",
    "8/8/8/8/8/8/8/8 w - - 0 1",                       // no kings
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP w KQkq - 0 1", // 7 ranks
    "rnbqkbnr/pppppppp/8/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", // 9 ranks
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNRR w KQkq - 0 1", // 9 files
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNX w KQkq - 0 1", // bad piece char
    "Pnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", // pawn on rank 8
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNp w KQkq - 0 1", // pawn on rank 1
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR x KQkq - 0 1", // bad stm
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w ZZZZ - 0 1", // bad castling
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w K-Qk - 0 1", // '-' mixed in
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq z9 0 1", // bad ep square
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq e3 0 1", // ep with no pawn
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - x 1", // bad halfmove
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 x", // bad fullmove
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 999999999999 1",
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 999999999999",
    "kkkkkkkk/8/8/8/8/8/8/KKKKKKKK w - - 0 1", // many kings
    "♔♕♖/8/8/8/8/8/8/8 w - - 0 1",             // non-ASCII
    "4k3/8/8/8/8/8/8/4K3 w KQkq - 0 1",        // castling rights with no rooks
];

/// xorshift64*, seeded so any failure reproduces.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below_len(&mut self, n: usize) -> usize {
        let n64 = u64::try_from(n).expect("lengths fit u64");
        usize::try_from(self.next() % n64).expect("value below n fits usize")
    }
}

#[test]
fn malformed_fens_are_rejected_not_panicked_on() {
    for fen in MALFORMED {
        match Board::from_fen(fen) {
            Err(_) => {}
            Ok(board) => {
                // Accepting is allowed only if the result is actually sound.
                board.check_consistency().unwrap_or_else(|e| {
                    panic!("from_fen ACCEPTED {fen:?} and produced an inconsistent board: {e}")
                });
            }
        }
    }
}

#[test]
fn mutated_fens_never_panic_and_never_yield_a_broken_board() {
    let mut rng = Rng(0x5EED_1234_ABCD_0001);
    for base in VALID {
        let bytes = base.as_bytes();
        for _ in 0..4_000 {
            let mut buf = bytes.to_vec();
            // 1-3 random single-byte edits from a chess-plausible alphabet, so
            // mutations land near the grammar rather than as obvious garbage.
            let edits = 1 + (rng.next() % 3) as usize;
            for _ in 0..edits {
                let pos = rng.below_len(buf.len());
                const ALPHABET: &[u8] = b"rnbqkpRNBQKP12345678/ wb-KQkqabcdefgh0123456789";
                buf[pos] = ALPHABET[rng.below_len(ALPHABET.len())];
            }
            let Ok(candidate) = std::str::from_utf8(&buf) else {
                continue;
            };
            if let Ok(board) = Board::from_fen(candidate) {
                board.check_consistency().unwrap_or_else(|e| {
                    panic!("from_fen ACCEPTED mutated {candidate:?} but the board is broken: {e}")
                });
            }
        }
    }
}

#[test]
fn truncations_and_extensions_never_panic() {
    for base in VALID {
        for cut in 0..base.len() {
            let _ = Board::from_fen(&base[..cut]);
            let _ = Board::from_fen(&base[cut..]);
        }
        let _ = Board::from_fen(&format!("{base} extra tokens here"));
        let _ = Board::from_fen(&"8/".repeat(64));
        let _ = Board::from_fen(&format!("{}{}", base, "9".repeat(300)));
    }
}

/// 9.5-A RESOLVED (2026-07-19): a malformed `position` command exits the
/// process with status 1 after printing an `info string CRITICAL ERROR: ...`
/// diagnostic. That is DELIBERATE and matches the reference implementation.
///
/// Measured across 11 engines before deciding (`D:/chess/engines`):
///
/// | engine                  | bad FEN            | illegal move in `moves` |
/// |-------------------------|--------------------|-------------------------|
/// | Stockfish dev-20260716  | exit 1 + diagnostic| exit 1 + diagnostic     |
/// | Stockfish 18 (release)  | CRASH (139)        | survives                |
/// | Critter 1.6a            | survives           | survives                |
/// | Fruit 2.1               | exit 1             | survives                |
/// | SaberTooth 0.2.0        | survives           | survives                |
/// | Whitespine 1.4.0        | survives           | survives                |
/// | Basilisk 1.9.0          | survives           | survives                |
/// | Hydra 1.5.0             | survives           | survives                |
/// | Beast 4.0.0             | exit 1             | exit 1                  |
/// | Lynx 1.3.3              | exit 1             | exit 1                  |
/// | Rarog (this engine)     | exit 1 + diagnostic| exit 1 + diagnostic     |
///
/// Current Stockfish prints the identical message format and exits the same
/// way on both cases, so Rarog is not merely defensible here — it matches the
/// reference engine's current design. It is also strictly better than
/// Stockfish 18's release behaviour, which segfaults silently on a bad FEN.
/// The permissive majority is not obviously right either: an engine that
/// "survives" a bad FEN keeps the PREVIOUS position and will search it, so
/// the GUI receives a legal-looking move for the wrong board — a forfeit too,
/// just slower and harder to diagnose.
///
/// So this test is INVERTED from how it was first written. It no longer
/// complains that the engine exits; it now GUARDS the intended contract:
/// exit code exactly 1, a `CRITICAL ERROR` diagnostic on stdout, and prompt
/// termination. A future change that crashes (any other exit code), hangs, or
/// dies silently instead will fail here.
mod malformed_position {
    use std::io::Write;
    use std::process::{Command, Stdio};

    /// Inputs that MUST be fatal — the position could not be established, so
    /// continuing would mean searching a stale board.
    const FATAL: [&str; 7] = [
        "position fen ",
        "position fen  ",
        "position fen not a fen at all",
        "position fen",
        "position startpos moves xxxx",
        "position startpos moves aé1",
        "position fen 8/8/8/8/8/8/8/8 w - - 0 1 moves a1a2",
    ];

    /// Inputs that MUST be tolerated — nothing was actually asked of us, so
    /// the previous position stays valid and the session continues.
    const TOLERATED: [&str; 3] = ["position", "position kentucky", "position startpos"];

    struct Run {
        code: Option<i32>,
        stdout: String,
    }

    fn feed(lines: &[&str]) -> Run {
        let mut child = Command::new(env!("CARGO_BIN_EXE_rarog"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("engine binary spawns");
        let mut input = String::from("uci\n");
        for line in lines {
            input.push_str(line);
            input.push('\n');
        }
        input.push_str("isready\nquit\n");
        child
            .stdin
            .as_mut()
            .expect("stdin is piped")
            .write_all(input.as_bytes())
            .expect("engine accepts input");
        let out = child
            .wait_with_output()
            .expect("engine terminates promptly");
        Run {
            code: out.status.code(),
            stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        }
    }

    #[test]
    fn fatal_inputs_exit_cleanly_with_a_diagnostic() {
        for line in FATAL {
            let run = feed(&[line]);
            assert_eq!(
                run.code,
                Some(1),
                "{line:?} must exit with status 1 (a crash or a survival is both wrong); \
                 got {:?}",
                run.code
            );
            assert!(
                run.stdout.contains("CRITICAL ERROR"),
                "{line:?} exited without telling the GUI why; stdout:\n{}",
                run.stdout
            );
            assert!(
                !run.stdout.contains("readyok"),
                "{line:?} answered isready after a fatal error"
            );
        }
    }

    #[test]
    fn tolerated_inputs_keep_the_session_alive() {
        for line in TOLERATED {
            let run = feed(&[line]);
            assert_eq!(
                run.code,
                Some(0),
                "{line:?} must not be fatal; got exit {:?}",
                run.code
            );
            assert!(
                run.stdout.contains("readyok"),
                "{line:?} left the engine unresponsive; stdout:\n{}",
                run.stdout
            );
        }
    }

    /// Byte soup on the `position` command must still reach a decision — exit
    /// 1 with a diagnostic, or carry on — never a crash and never a hang.
    ///
    /// RELEASE ONLY, deliberately. Each case needs its own process (a fatal
    /// one kills the session), and a debug engine spends ~5 s per start on
    /// unoptimised table init, so in debug this cost 343 s on its own. The
    /// property under test — that the SHIPPED binary cannot be crashed or
    /// wedged by a GUI — is a property of the release build anyway, and CI
    /// runs both profiles, so release coverage loses nothing real.
    #[cfg(not(debug_assertions))]
    #[test]
    fn random_position_garbage_never_crashes_or_hangs() {
        let mut rng = super::Rng(0x5EED_0000_FEED_0002);
        for _ in 0..25 {
            let len = 1 + rng.below_len(40);
            let mut line = String::from("position ");
            for _ in 0..len {
                const ALPHABET: &[u8] = b"abcdefgh12345678 fenmovsstartpo/-wbKQkq";
                line.push(ALPHABET[rng.below_len(ALPHABET.len())] as char);
            }
            let run = feed(&[&line]);
            let code = run.code.unwrap_or_else(|| {
                panic!("{line:?} killed the engine by signal, not by a clean exit")
            });
            assert!(
                code == 0 || code == 1,
                "{line:?} produced exit code {code}; only 0 (tolerated) or 1 \
                 (clean fatal) are acceptable — anything else is a crash"
            );
            if code == 1 {
                assert!(
                    run.stdout.contains("CRITICAL ERROR"),
                    "{line:?} exited 1 without a diagnostic"
                );
            }
        }
    }
}
