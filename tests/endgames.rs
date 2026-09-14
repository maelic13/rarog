//! Phase 3.11 permanent endgame regression suite.
//!
//! Protects the scale-factor framework (insufficient-material draws) and the
//! KBNK corner-drive mate against future eval/search changes. The position
//! list lives in `tests/endgames.epd`; this harness loads it and asserts:
//! * `draw` positions evaluate to exactly 0 statically.
//! * `win` positions evaluate clearly for the side to move (the winning side),
//!   confirming a won ending is not zeroed by a draw rule.
//! * `kbnk-mate` positions are driven to checkmate within a move budget by a
//!   fixed-depth search loop.
//!
//! It also unit-tests the corner-drive *direction* (right-coloured corner is
//! scored better than the wrong-coloured one) so the term can't silently flip.
//!
//! Partial-scale endings (KRKP ≈×¼, OCB passer relaxation) do not produce a
//! clean `draw`/`win` verdict, so they are covered by the unit tests in
//! `src/eval.rs` (`endgame_311c_tests`) rather than here.

use rarog::board::{Board, Color, Move, Piece};
use rarog::eval::{Evaluator, MATE_SCORE};
use rarog::infra::THREAD_STACK_SIZE;
use rarog::search::{SearchEvent, Searcher};
use rarog::search_options::SearchOptions;

const ENDGAMES_EPD: &str = include_str!("endgames.epd");

/// Move budget (plies) for a KBNK playout. Perfect play mates in <= ~33 plies;
/// from the near-corner suite positions far fewer are needed.
/// 40 was too small for any position the anchor could actually discriminate
/// on: the head needs 45-75 plies from a centre-king start, so a 40-ply budget
/// admitted only near-corner cases -- which mate even with a broken drive.
const KBNK_MOVE_BUDGET: usize = 90;
/// Node budget per move during a KBNK playout.
///
/// PLAN 4.10.4. This was a fixed DEPTH of 10 with a fresh `Searcher` -- and so
/// a fresh, empty transposition table -- for every move of the game. Both
/// halves of that were wrong, and wrong in the direction that makes an anchor
/// pass when it should fail:
///
/// * The conversion failure this suite exists to catch was measured at 60,000
///   nodes per move with ONE table persisting across the game
///   (`tools/diag/endgame_truth.py`). A depth cap is a different budget, and
///   an empty table each move is a different search.
/// * Basilisk hit exactly this: an anchor written for a specific losing line
///   PASSED under the very vector it was written to catch, for these two
///   reasons, and only reproducing the original conditions made it fail
///   correctly (BAS-E39).
///
/// So the playout now matches the instrument: a node budget, and one searcher
/// for the whole game. 60,000 is the instrument's figure.
///
/// One budget is not enough, though. Replayed at 40k, 50k, 60k, 80k and 120k
/// nodes, the accepted engine mates `7N/8/7B/8/8/8/8/2K2k2 w` at three of the
/// five and a selectivity-core candidate misses `8/8/K3B3/8/5N2/8/7k/8 w` at
/// exactly 60k while mating at the other four: whether a single playout lands
/// inside 90 plies is chaotic in the node budget. So each position is played
/// at these five budgets, one searcher per budget, and must mate at a
/// majority. The discrimination survives: with the corner drive scaled to
/// 1/24 the three discriminator positions mate at 2, 2 and 0 of the five.
const KBNK_NODE_BUDGETS: [u64; 5] = [40_000, 50_000, 60_000, 80_000, 120_000];

struct Case {
    fen: String,
    verdict: String,
    comment: String,
}

fn parse_cases() -> Vec<Case> {
    let mut cases = Vec::new();
    for line in ENDGAMES_EPD.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (fen, tail) = line
            .split_once(';')
            .unwrap_or_else(|| panic!("malformed EPD line (no ';'): {line}"));
        let mut tail = tail.trim().splitn(2, char::is_whitespace);
        let verdict = tail.next().unwrap_or("").to_string();
        let comment = tail.next().unwrap_or("").to_string();
        cases.push(Case {
            fen: fen.trim().to_string(),
            verdict,
            comment,
        });
    }
    assert!(!cases.is_empty(), "endgames.epd produced no cases");
    cases
}

fn static_eval(fen: &str) -> i32 {
    let board = Board::from_fen(fen).unwrap_or_else(|e| panic!("bad FEN {fen}: {e}"));
    let mut evaluator = Evaluator::default();
    evaluator.evaluate(&board)
}

/// Scores that mean "forced mate found". `MATE_SCORE` minus the ply horizon is
/// the standard band; anything at or above it is a mate claim.
const MATE_THRESHOLD: i32 = MATE_SCORE - 256;

fn search_score(board: Board, depth: u32) -> i32 {
    let mut searcher = Searcher::default();
    let mut options = SearchOptions {
        board: board.clone(),
        ..SearchOptions::default()
    };
    options.limits.depth = Some(depth);
    options.engine.threads = 1;
    searcher
        .search(board, &options, false, || SearchEvent::None)
        .score
}

/// One move from a searcher that PERSISTS across the game, under a node
/// budget. See `KBNK_NODE_BUDGET` for why both properties matter.
fn search_bestmove_nodes(searcher: &mut Searcher, board: Board, nodes: u64) -> Move {
    let mut options = SearchOptions {
        board: board.clone(),
        ..SearchOptions::default()
    };
    options.limits.nodes = nodes;
    options.engine.threads = 1;
    searcher
        .search(board, &options, false, || SearchEvent::None)
        .bestmove
}

/// Plays the position out at one node budget. `true` when the bare king is
/// checkmated within the move budget; panics on a stalemate, a mated winner or
/// a null move, which are defects at any budget.
fn kbnk_playout_mates(fen: &str, comment: &str, nodes: u64) -> bool {
    let mut board = Board::from_fen(fen).unwrap_or_else(|e| panic!("bad FEN {fen}: {e}"));
    let winner = if board.pieces(Color::White, Piece::Bishop).any() {
        Color::White
    } else {
        Color::Black
    };
    // ONE searcher for the whole game, so the table carries across moves --
    // the condition the defect was measured under. `new_game` once, not once
    // per move.
    let mut searcher = Searcher::default();
    searcher.new_game();

    for _ in 0..KBNK_MOVE_BUDGET {
        if board.generate_legal_moves().is_empty() {
            assert!(
                board.is_in_check(),
                "[{comment}] expected checkmate but found stalemate at {nodes} nodes: {}",
                board.to_fen()
            );
            assert_ne!(
                board.side_to_move(),
                winner,
                "[{comment}] the winning side was mated at {nodes} nodes: {}",
                board.to_fen()
            );
            return true;
        }
        let mv = search_bestmove_nodes(&mut searcher, board.clone(), nodes);
        assert!(
            !mv.is_null(),
            "[{comment}] search returned a null move at {nodes} nodes: {}",
            board.to_fen()
        );
        board.make_move(mv);
    }
    false
}

/// Plays the position at every budget in [`KBNK_NODE_BUDGETS`], in parallel,
/// and asserts it is mated at a strict majority of them.
fn assert_kbnk_mates(fen: &str, comment: &str) {
    let mates = std::thread::scope(|scope| {
        let playouts = KBNK_NODE_BUDGETS.map(|nodes| {
            std::thread::Builder::new()
                .stack_size(THREAD_STACK_SIZE)
                .spawn_scoped(scope, move || {
                    (nodes, kbnk_playout_mates(fen, comment, nodes))
                })
                .expect("spawn a KBNK playout thread")
        });
        playouts
            .into_iter()
            .map(|playout| {
                playout
                    .join()
                    .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
            })
            .collect::<Vec<_>>()
    });
    let mated = mates.iter().filter(|(_, mated)| *mated).count();
    assert!(
        mated * 2 > KBNK_NODE_BUDGETS.len(),
        "[{comment}] KBNK mated within {KBNK_MOVE_BUDGET} plies at only {mated} of {} budgets \
         from {fen}: {mates:?}",
        KBNK_NODE_BUDGETS.len()
    );
}

#[test]
fn insufficient_material_positions_are_dead_draws() {
    for case in parse_cases().iter().filter(|c| c.verdict == "draw") {
        let eval = static_eval(&case.fen);
        assert_eq!(
            eval, 0,
            "[{}] expected static eval 0, got {eval}: {}",
            case.comment, case.fen
        );
    }
}

#[test]
fn won_positions_are_clearly_winning() {
    // Each `win` line has the winning side to move, so static eval (side-to-move
    // perspective) must be clearly positive — confirming a won ending (e.g. a
    // won KPK) is not zeroed by a draw rule like a drawn KPK is.
    let mut checked = 0;
    for case in parse_cases().iter().filter(|c| c.verdict == "win") {
        let eval = static_eval(&case.fen);
        assert!(
            eval > 80,
            "[{}] expected a clearly winning score, got {eval}: {}",
            case.comment,
            case.fen
        );
        checked += 1;
    }
    assert!(checked > 0, "no `win` cases found");
}

/// Search depth for the `tb-draw` mate veto. Deep enough to find a real forced
/// mate in these tiny trees, shallow enough to stay fast in debug.
const TB_DRAW_SEARCH_DEPTH: u32 = 12;

// WHY THERE IS NO MATE-DRIVE GRADIENT TEST HERE (4.9a.4).
//
// The 4.9a.4 defect was that the drive used pure Chebyshev distance, which is
// flat: 94% of won KBNK positions had a TIED best move, so the engine shuffled
// until the fifty-move rule and converted 19.4% of them. Every test in this
// file stayed green throughout -- the mate-in-one passed, the direction was
// correct, the recognizer was present and wired. That is a gap worth closing if
// it can be closed cheaply, and two attempts show it cannot:
//
//   1. Tie rate over `static_eval` across legal moves. PASSED on the broken
//      drive: static_eval is the WHOLE evaluation, and its other terms break
//      ties that the mop-up alone cannot. It measured something real, just not
//      the thing named.
//   2. Corner progress over 24 plies at depth 6, aggregated over the frozen
//      KBNK cases. Also PASSED on the broken drive: at short range the SEARCH
//      finds progress without needing an eval gradient, so the bar cannot be
//      set anywhere that separates 19.4% conversion from 57.1%.
//
// What actually separates them is conversion rate over ~100 positions per
// family at 60,000 nodes -- about two minutes of compute, and a statistic with
// real sampling noise rather than an absolute property. That belongs in
// `tools/diag/endgame_floors.json`, compared with a noise allowance and
// ratcheted after accepted improvements, which is exactly where it lives.
//
// The lesson generalises: a term can be present, correctly signed, and
// individually tested, and still fail to steer a search. Direction is testable
// here; ORDERING is not, and asserting the wrong one produces a green suite
// over a broken engine.

#[test]
fn syzygy_won_positions_are_not_scored_as_drawn_or_lost() {
    // HARD VETO, not a tuning target. Syzygy says the side to move wins; the
    // static score must at least have the right sign. This is deliberately
    // loose: RAR-E09 measured a won KR-K scoring +426 cornered and +487
    // centralised, and pinning a floor near those would turn a correctness test
    // into a calibration test that any refit could trip.
    let mut checked = 0;
    for case in parse_cases().iter().filter(|c| c.verdict == "tb-win") {
        let eval = static_eval(&case.fen);
        assert!(
            eval > 0,
            "[{}] Syzygy says this is won but static eval is {eval}: {}",
            case.comment,
            case.fen
        );
        checked += 1;
    }
    assert!(
        checked >= 30,
        "expected the frozen tb-win set, found {checked}"
    );
}

#[test]
fn syzygy_drawn_positions_are_never_claimed_as_forced_mate() {
    // HARD VETO. A theoretically drawn position may legitimately carry a large
    // material score -- a drawn KR-KP really is a rook up -- so the veto is on
    // the one thing that is unambiguously wrong: claiming a forced mate.
    let mut checked = 0;
    for case in parse_cases().iter().filter(|c| c.verdict == "tb-draw") {
        let board = Board::from_fen(&case.fen)
            .unwrap_or_else(|_| panic!("bad FEN in endgames.epd: {}", case.fen));
        let score = search_score(board, TB_DRAW_SEARCH_DEPTH);
        assert!(
            score.abs() < MATE_THRESHOLD,
            "[{}] Syzygy says this is drawn but search reports a mate score              ({score}): {}",
            case.comment,
            case.fen
        );
        checked += 1;
    }
    assert!(
        checked >= 25,
        "expected the frozen tb-draw set, found {checked}"
    );
}

#[test]
fn kbnk_positions_are_driven_to_mate() {
    // Thin-sample refusal (PLAN 4.10.4). Without a count, an EPD that stopped
    // producing `kbnk-mate` rows -- a rename, a parse change, a bad filter --
    // would make this test pass over an empty set. A guard that cannot fail is
    // not a guard. The sibling Syzygy vetoes already carry `checked >= 30` and
    // `checked >= 25`; this one carried nothing.
    let mut checked = 0;
    for case in parse_cases().iter().filter(|c| c.verdict == "kbnk-mate") {
        assert_kbnk_mates(&case.fen, &case.comment);
        checked += 1;
    }
    // The frozen set holds exactly ONE kbnk-mate case, which is thin for an
    // anchor guarding the family 4.9a.4 rebuilt. The guard records the real
    // number rather than an aspirational one; widening the set belongs to
    // 4.12.21, which owns KBNK. What this guard does buy is that a parse or
    // filter change cannot silently reduce it to zero.
    assert!(
        checked >= 4,
        "expected the frozen kbnk-mate set, found {checked}"
    );
}

/// The KBNK corner-drive must steer the bare king toward a corner the winning
/// bishop can actually reach (one of its own colour). Static eval is confounded
/// by the bishop's PST (the light/dark bishops sit on different squares) and by
/// the bare king's PST, so this isolates the corner term with a
/// difference-of-differences:
///
///   isolated = [E(lightB, Kcorner) - E(darkB, Kcorner)]
///            - [E(lightB, Kcentre) - E(darkB, Kcentre)]
///
/// The bishop-PST term cancels (it appears in both brackets) and the king-PST
/// term cancels (king square is identical within each bracket). What remains is
/// the corner-drive's contribution. h8 is a light-coloured corner here, so the
/// light bishop must be favoured there: `isolated` is clearly positive. If the
/// corner mapping were inverted, the sign would flip and this test would catch it.
#[test]
fn kbnk_drives_to_the_bishops_corner() {
    // White Ke4, Ng1 fixed; bishop is c1 (light) or d1 (dark). Bare king is on
    // the light corner h8, or on a central square (c5) to cancel the bishop PST.
    let light_corner = static_eval("7k/8/8/8/4K3/8/8/2B3N1 w - - 0 1"); // Bc1, Kh8
    let dark_corner = static_eval("7k/8/8/8/4K3/8/8/3B2N1 w - - 0 1"); // Bd1, Kh8
    let light_centre = static_eval("8/8/8/2k5/4K3/8/8/2B3N1 w - - 0 1"); // Bc1, Kc5
    let dark_centre = static_eval("8/8/8/2k5/4K3/8/8/3B2N1 w - - 0 1"); // Bd1, Kc5

    let isolated = (light_corner - dark_corner) - (light_centre - dark_centre);
    assert!(
        isolated > 20,
        "KBNK corner drive points the wrong way: isolated corner term = {isolated} \
         (light bishop should be favoured at the light corner h8). \
         corner(light={light_corner}, dark={dark_corner}) \
         centre(light={light_centre}, dark={dark_centre})"
    );
}
