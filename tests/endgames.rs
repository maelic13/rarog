//! Phase 3.11 permanent endgame regression suite.
//!
//! Protects the scale-factor framework (insufficient-material draws) and the
//! KBNK corner-drive mate against future eval/search changes. The position
//! list lives in `tests/endgames.epd`; this harness loads it and asserts:
//!   * `draw`      positions evaluate to exactly 0 statically,
//!   * `win`       positions evaluate clearly for the side to move (the winning
//!                 side), confirming a won ending is not zeroed by a draw rule, and
//!   * `kbnk-mate` positions are driven to checkmate within a move budget by a
//!     fixed-depth search loop.
//! It also unit-tests the corner-drive *direction* (right-coloured corner is
//! scored better than the wrong-coloured one) so the term can't silently flip.
//!
//! Partial-scale endings (KRKP ≈×¼, OCB passer relaxation) do not produce a
//! clean `draw`/`win` verdict, so they are covered by the unit tests in
//! `src/eval.rs` (`endgame_311c_tests`) rather than here.

use rarog::board::{Board, Color, Move, Piece};
use rarog::eval::Evaluator;
use rarog::search::{SearchEvent, Searcher};
use rarog::search_options::SearchOptions;

const ENDGAMES_EPD: &str = include_str!("endgames.epd");

/// Move budget (plies) for a KBNK playout. Perfect play mates in <= ~33 plies;
/// from the near-corner suite positions far fewer are needed.
const KBNK_MOVE_BUDGET: usize = 40;
/// Fixed search depth per move during a KBNK playout. KBNK trees are tiny, so
/// this stays fast even under heavy CPU load.
const KBNK_SEARCH_DEPTH: usize = 10;

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

fn search_bestmove(board: Board, depth: usize) -> Move {
    let mut searcher = Searcher::default();
    let mut options = SearchOptions::default();
    options.position.board = board.clone();
    options.limits.depth = depth as f64;
    options.engine.threads = 1;
    searcher
        .search(board, &options, false, || SearchEvent::None)
        .bestmove
}

/// Plays the position out at a fixed depth and asserts the bare king is
/// checkmated (not stalemated) within the budget, by the side holding the
/// bishop+knight.
fn assert_kbnk_mates(fen: &str, comment: &str) {
    let mut board = Board::from_fen(fen).unwrap_or_else(|e| panic!("bad FEN {fen}: {e}"));
    let winner = if board.pieces(Color::White, Piece::Bishop).any() {
        Color::White
    } else {
        Color::Black
    };

    for _ in 0..KBNK_MOVE_BUDGET {
        if board.generate_legal_moves().is_empty() {
            assert!(
                board.is_in_check(),
                "[{comment}] expected checkmate but found stalemate: {}",
                board.to_fen()
            );
            assert_ne!(
                board.side_to_move(),
                winner,
                "[{comment}] the winning side was mated: {}",
                board.to_fen()
            );
            return;
        }
        let mv = search_bestmove(board.clone(), KBNK_SEARCH_DEPTH);
        assert!(
            !mv.is_null(),
            "[{comment}] search returned a null move: {}",
            board.to_fen()
        );
        board.make_move(mv);
    }
    panic!("[{comment}] KBNK did not mate within {KBNK_MOVE_BUDGET} plies from {fen}");
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

#[test]
fn kbnk_positions_are_driven_to_mate() {
    for case in parse_cases().iter().filter(|c| c.verdict == "kbnk-mate") {
        assert_kbnk_mates(&case.fen, &case.comment);
    }
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
