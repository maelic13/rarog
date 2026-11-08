//! Search, evaluation and endgame behaviour that Basilisk's suite pins, read
//! against Rarog and kept as regression anchors; each test names the Basilisk
//! test it follows. Two are `#[ignore]`d: they fail today and are the
//! acceptance tests of the repairs that own them.
//!
//! Every test holds one process-wide lock, because the Syzygy tables are global
//! to the process: a test that loads them must not run beside a search that
//! expects none, and it unloads them before it releases the lock.

use std::sync::{Mutex, MutexGuard};

use rarog::bench::BENCH_FENS;
use rarog::board::{Board, CastlingRights, Color, Move, Piece, Square};
use rarog::eval::{Evaluator, MATE_SCORE};
use rarog::search::options::SearchOptions;
use rarog::search::{SearchEvent, SearchResult, Searcher};
use rarog::syzygy::{self, Wdl};
use rarog::wac::wac_positions;

const ENDGAMES_EPD: &str = include_str!("endgames.epd");

/// Basilisk's `MAX_PLY` band: a score at or above `MATE_SCORE - 256` is a mate.
const MATE_BAND: i32 = MATE_SCORE - 256;

static SERIAL: Mutex<()> = Mutex::new(());

fn serial() -> MutexGuard<'static, ()> {
    SERIAL
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn board(fen: &str) -> Board {
    Board::from_fen(fen).unwrap_or_else(|e| panic!("bad FEN {fen}: {e}"))
}

fn search_with(searcher: &mut Searcher, fen: &str, depth: Option<u32>, nodes: u64) -> SearchResult {
    search_board(searcher, board(fen), depth, nodes)
}

fn search_board(
    searcher: &mut Searcher,
    root: Board,
    depth: Option<u32>,
    nodes: u64,
) -> SearchResult {
    let mut options = SearchOptions {
        board: root.clone(),
        ..SearchOptions::default()
    };
    options.limits.depth = depth;
    options.limits.nodes = nodes;
    options.engine.threads = 1;
    searcher.search(root, &options, false, || SearchEvent::None)
}

/// One search from a fresh searcher (a fresh 64 MB table).
fn search(fen: &str, depth: Option<u32>, nodes: u64) -> SearchResult {
    search_with(&mut Searcher::default(), fen, depth, nodes)
}

fn is_legal(fen: &str, mv: Move) -> bool {
    !mv.is_null()
        && board(fen)
            .generate_legal_moves()
            .iter()
            .any(|&legal| legal.same_uci_move(mv))
}

fn mate_in_from_score(score: i32) -> i32 {
    (MATE_SCORE - score.abs() + 1) / 2
}

fn summary(r: &SearchResult) -> String {
    format!(
        "bestmove {} score {} depth {} nodes {} pv {:?}",
        r.bestmove,
        r.score,
        r.depth,
        r.nodes,
        r.pv.iter().map(ToString::to_string).collect::<Vec<_>>()
    )
}

/// Static evaluation from White's side (Rarog's is side-to-move relative).
fn eval_white(fen: &str) -> i32 {
    let b = board(fen);
    let stm = Evaluator::default().evaluate(&b);
    if b.side_to_move() == Color::White {
        stm
    } else {
        -stm
    }
}

// ------------------------------------------------- test_search.cpp ----

/// `shortest_mate_not_first_mate`: KQK must not stop on a longer visible mate.
/// Basilisk: depth 18, 64 MB; the depth is reached, the move is legal, the
/// score is a forced mate shorter than the stuck-on-8 pathology.
#[test]
fn shortest_mate_not_first_mate() {
    let _g = serial();
    let fen = "4K3/2Q5/6k1/8/8/8/8/8 w - - 0 1";
    let r = search(fen, Some(18), 0);
    let s = summary(&r);
    assert_eq!(r.depth, 18, "depth reached: {s}");
    assert!(is_legal(fen, r.bestmove), "legal: {s}");
    assert!(r.score >= MATE_BAND, "a forced mate score: {s}");
    assert!(
        mate_in_from_score(r.score) < 8,
        "shorter than mate in 8: {s}"
    );
}

/// `rook_promotions` 1: a rook only where the queen would stalemate (depth 8).
#[test]
fn rook_promotion_where_the_queen_stalemates() {
    let _g = serial();
    let r = search("8/5P1k/5K2/8/8/8/8/8 w - - 0 1", Some(8), 0);
    assert_eq!(r.bestmove.to_string(), "f7f8r", "{}", summary(&r));
}

/// `rook_promotions` 2: the rook line only transposes into the queen line,
/// and a tie at 0 must not pick the rook (200,000 nodes, no depth cap).
#[test]
fn queen_promotion_where_the_rook_only_transposes() {
    let _g = serial();
    let r = search("3r4/P7/2K1p3/R7/5p2/6kP/8/8 w - - 2 66", None, 200_000);
    let s = summary(&r);
    assert_eq!(r.bestmove.to_string(), "a7a8q", "{s}");
    assert!(r.score > 0, "{s}");
}

/// `rook_promotions` 3: the queen though the rook was searched first
/// (100,000 nodes).
#[test]
fn queen_promotion_though_the_rook_was_searched_first() {
    let _g = serial();
    let r = search(
        "2k2rr1/6P1/2P5/7p/1B2p3/4P3/5K2/8 w - - 0 46",
        None,
        100_000,
    );
    assert_eq!(r.bestmove.to_string(), "g7f8q", "{}", summary(&r));
}

/// `rook_promotions` 4: Saavedra, c8=R wins where c8=Q draws (100,000 nodes).
#[test]
fn saavedra_rook_promotion_wins() {
    let _g = serial();
    let r = search("8/2P5/8/8/3r4/8/2K5/k7 w - - 0 6", None, 100_000);
    let s = summary(&r);
    assert_eq!(r.bestmove.to_string(), "c7c8r", "{s}");
    assert!(r.score >= MATE_SCORE - 40, "{s}");
}

/// `tournament_infraction_positions`: legal best moves in two final positions
/// of rules-infraction games (depths 10 and 6, as Basilisk).
#[test]
fn tournament_infraction_positions() {
    let _g = serial();
    for (fen, depth) in [
        ("8/6K1/8/8/8/p7/P7/1k6 b - - 4 71", 10),
        ("8/8/8/K3R3/3Q4/8/6p1/2k4q w - - 26 91", 6),
    ] {
        let r = search(fen, Some(depth), 0);
        assert!(is_legal(fen, r.bestmove), "{fen}: {}", summary(&r));
    }
}

/// `rule50_mate_found` 1: mate in one with the clock at 99 is a mate.
#[test]
fn rule50_mate_in_one_at_clock_99() {
    let _g = serial();
    let r = search("7k/5Q2/5K2/8/8/8/8/8 w - - 99 1", Some(4), 0);
    let s = summary(&r);
    assert!(r.score >= MATE_SCORE - 3, "{s}");
    assert_eq!(mate_in_from_score(r.score), 1, "{s}");
}

/// `rule50_mate_found` 2: a quiet position at clock 100 scores a draw.
#[test]
fn rule50_quiet_position_at_clock_100_is_a_draw() {
    let _g = serial();
    let r = search("7k/8/5K2/8/8/8/8/6Q1 b - - 100 1", Some(4), 0);
    assert_eq!(r.score, 0, "{}", summary(&r));
}

// --------------------------------------------- packaged FEN legality ----

/// Basilisk's strict `try_set_fen(fen, true)` plus the castling and en-passant
/// consistency the brief asks for. Basilisk fails on: field syntax, back-rank
/// pawns, king count, more than 8 pawns, promoted pieces beyond missing pawns,
/// an en-passant square whose target or origin is occupied or whose pushed
/// pawn is missing, the side not to move in check, and the side to move in
/// check from more than two pieces. It silently DROPS inconsistent castling
/// rights rather than failing; here an inconsistent right fails, and a right
/// Rarog's parser dropped or added is reported too.
fn strict_check(fen: &str) -> Result<(), String> {
    let b = Board::from_fen(fen).map_err(|e| format!("rejected by from_fen: {e}"))?;
    let fields: Vec<&str> = fen.split_whitespace().collect();
    if fields.len() < 4 {
        return Err("fewer than four fields".to_string());
    }
    for color in [Color::White, Color::Black] {
        if b.pieces(color, Piece::King).count() != 1 {
            return Err(format!("{color:?} king count"));
        }
        let pawns = b.pieces(color, Piece::Pawn).count();
        if pawns > 8 {
            return Err(format!("{color:?} has {pawns} pawns"));
        }
        let extra = |piece, base: u32| b.pieces(color, piece).count().saturating_sub(base);
        let promoted = extra(Piece::Knight, 2)
            + extra(Piece::Bishop, 2)
            + extra(Piece::Rook, 2)
            + extra(Piece::Queen, 1);
        if promoted > 8 - pawns {
            return Err(format!("{color:?} has too many promoted pieces"));
        }
    }
    let pawns = b.pieces(Color::White, Piece::Pawn).0 | b.pieces(Color::Black, Piece::Pawn).0;
    if pawns & 0xFF00_0000_0000_00FF != 0 {
        return Err("pawn on rank 1 or 8".to_string());
    }
    let us = b.side_to_move();
    let them = !us;
    let occ = b.occupied();
    if b.attackers_to_color(b.king_sq(them), occ, us).any() {
        return Err("side not to move is in check".to_string());
    }
    let checkers = b.attackers_to_color(b.king_sq(us), occ, them).count();
    if checkers > 2 {
        return Err(format!("side to move in check from {checkers} pieces"));
    }
    let at = |sq: u8| b.piece_at(Square(sq));
    let mut text_rights = Vec::new();
    for c in fields[2].chars().filter(|&c| c != '-') {
        let (color, king, rook, flag) = match c {
            'K' => (Color::White, 4, 7, CastlingRights::WHITE_KINGSIDE),
            'Q' => (Color::White, 4, 0, CastlingRights::WHITE_QUEENSIDE),
            'k' => (Color::Black, 60, 63, CastlingRights::BLACK_KINGSIDE),
            'q' => (Color::Black, 60, 56, CastlingRights::BLACK_QUEENSIDE),
            other => return Err(format!("castling character {other}")),
        };
        if at(king) != Some((color, Piece::King)) || at(rook) != Some((color, Piece::Rook)) {
            return Err(format!("castling right {c} without king and rook at home"));
        }
        text_rights.push(flag);
    }
    for flag in [
        CastlingRights::WHITE_KINGSIDE,
        CastlingRights::WHITE_QUEENSIDE,
        CastlingRights::BLACK_KINGSIDE,
        CastlingRights::BLACK_QUEENSIDE,
    ] {
        if b.castling().has(flag) != text_rights.contains(&flag) {
            return Err(format!(
                "parsed castling rights differ from `{}`",
                fields[2]
            ));
        }
    }
    if fields[3] != "-" {
        let ep = fields[3].as_bytes();
        if ep.len() != 2 || !(b'a'..=b'h').contains(&ep[0]) || !(b'1'..=b'8').contains(&ep[1]) {
            return Err(format!("en-passant square {}", fields[3]));
        }
        let (file, rank) = (ep[0] - b'a', ep[1] - b'1');
        let expected_rank = if us == Color::White { 5 } else { 2 };
        if rank != expected_rank {
            return Err(format!("en-passant rank {}", fields[3]));
        }
        let target = rank * 8 + file;
        let (pushed, origin) = if us == Color::White {
            (target - 8, target + 8)
        } else {
            (target + 8, target - 8)
        };
        if at(target).is_some() || at(origin).is_some() || at(pushed) != Some((them, Piece::Pawn)) {
            return Err(format!("en-passant square {} inconsistent", fields[3]));
        }
    }
    Ok(())
}

fn epd_fens(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let head = line.split(';').next().unwrap_or("");
            let fields: Vec<&str> = head.split_whitespace().collect();
            // Six fields when the counters are numbers, else EPD's four.
            if fields.len() >= 6
                && fields[4].parse::<u32>().is_ok()
                && fields[5].parse::<u32>().is_ok()
            {
                fields[..6].join(" ")
            } else {
                format!("{} 0 1", fields[..4.min(fields.len())].join(" "))
            }
        })
        .collect()
}

fn sweep(label: &str, fens: &[String], failures: &mut Vec<String>) -> usize {
    for fen in fens {
        if let Err(e) = strict_check(fen) {
            failures.push(format!("{label}: {fen} ({e})"));
        }
    }
    fens.len()
}

/// `packaged_fens_strictly_legal`: the bench suite, WAC and every EPD under
/// `tests/` (only `endgames.epd`).
#[test]
fn packaged_fens_strictly_legal() {
    let _g = serial();
    let mut failures = Vec::new();
    let bench: Vec<String> = BENCH_FENS.iter().map(|f| (*f).to_string()).collect();
    let wac: Vec<String> = wac_positions().into_iter().map(|p| p.fen).collect();
    let endgames = epd_fens(ENDGAMES_EPD);
    let counts = [
        sweep("bench", &bench, &mut failures),
        sweep("wac", &wac, &mut failures),
        sweep("endgames.epd", &endgames, &mut failures),
    ];
    eprintln!(
        "checked bench {} wac {} endgames.epd {}",
        counts[0], counts[1], counts[2]
    );
    assert_eq!(counts[0], 40);
    assert_eq!(counts[1], 300);
    assert!(counts[2] >= 90, "endgames.epd rows: {}", counts[2]);
    assert!(failures.is_empty(), "{failures:#?}");
}

/// The same sweep over the small frozen EPDs under `tools/books` and
/// `tools/diag` (not shipped in the crate, but tracked and consumed).
#[test]
fn tracked_tool_epds_strictly_legal() {
    let _g = serial();
    let mut failures = Vec::new();
    let mut checked = 0;
    for rel in [
        "tools/books/endgame_cohort_v1.epd",
        "tools/books/endgame_minor_v1.epd",
        "tools/books/ocb_cohort_v1.epd",
        "tools/diag/board_search_profile_v1.epd",
        "tools/diag/phase4_suite_v1.epd",
    ] {
        let path = format!("{}/{rel}", env!("CARGO_MANIFEST_DIR"));
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
        checked += sweep(rel, &epd_fens(&text), &mut failures);
    }
    eprintln!("checked {checked} tool EPD rows");
    assert!(checked > 9_000, "{checked}");
    assert!(
        failures.is_empty(),
        "{} failures: {failures:#?}",
        failures.len()
    );
}

// --------------------------------------------------- test_eval.cpp ----

const Q: &str = "1B3Q2/8/8/2k2K2/8/8/8/8 b - - 0 77";
const R: &str = "1B3R2/8/8/2k2K2/8/8/8/8 b - - 0 77";
const N: &str = "1B3N2/8/8/2k2K2/8/8/8/8 b - - 0 77";
const BI: &str = "1B3B2/8/8/2k2K2/8/8/8/8 b - - 0 77";

/// `bare_king_material_is_monotone` 1: from KBP-K, Q > R > N and R > B.
#[test]
#[ignore = "D.6.25: two same-coloured bishops outscore a queen against a bare king"]
fn bare_king_promotion_order() {
    let _g = serial();
    let (q, r, n, bi) = (eval_white(Q), eval_white(R), eval_white(N), eval_white(BI));
    assert!(q > r && r > n && r > bi, "Q {q} R {r} N {n} B {bi}");
}

/// Not a Basilisk test: whether the promotion order above reaches the search.
/// BI's bishops (b8, f8) share a colour, a draw. From KBP-K the search must
/// promote to a queen or rook at every depth 1-12, never to a minor.
#[test]
fn bare_king_promotion_choice_in_search() {
    let _g = serial();
    eprintln!(
        "same-colour KBB-K draws (endgames.epd 62, 63): {} {}; opposite-colour 1B4B1: {}",
        eval_white("2B5/8/2B5/8/3k2K1/8/8/8 w - - 0 1"),
        eval_white("8/1k1K4/1B6/8/8/8/5B2/8 w - - 0 1"),
        eval_white("1B4B1/8/8/2k2K2/8/8/8/8 b - - 0 77"),
    );
    let fen = "1B6/5P2/8/2k2K2/8/8/8/8 w - - 0 77";
    let mut bad = Vec::new();
    for depth in 1..=12 {
        let r = search(fen, Some(depth), 0);
        eprintln!("KBP-K depth {depth}: {}", summary(&r));
        if !["f7f8q", "f7f8r"].contains(&r.bestmove.to_string().as_str()) {
            bad.push(format!("depth {depth}: {}", summary(&r)));
        }
    }
    assert!(bad.is_empty(), "{bad:#?}");
}

/// `bare_king_material_is_monotone` 2: shedding a pawn never raises the score.
#[test]
fn bare_king_shedding_a_pawn_never_raises_the_score() {
    let _g = serial();
    let with_n = eval_white("1B6/5P2/8/2k2K2/8/8/N7/8 w - - 0 77");
    let without_n = eval_white("1B6/8/8/2k2K2/8/8/N7/8 w - - 0 77");
    let with_r = eval_white("1B6/5P2/8/2k2K2/8/8/8/R7 w - - 0 77");
    let without_r = eval_white("1B6/8/8/2k2K2/8/8/8/R7 w - - 0 77");
    assert!(
        with_n >= without_n && with_r >= without_r,
        "KBNP {with_n} vs KBN {without_n}; KBRP {with_r} vs KBR {without_r}"
    );
}

// ----------------------------------------------- test_endgames.cpp ----

/// `near_mate_recognition`: mates within the bound at Basilisk's MATE_DEPTH
/// 18, including the KQK stalemate trap (Qb7# mates, Qb6 stalemates).
#[test]
fn near_mate_recognition() {
    let _g = serial();
    let cases: [(&str, i32, &str); 5] = [
        ("k7/7Q/2K5/8/8/8/8/8 w - - 0 1", 2, "KQK: mate-in-1 (Qb7#)"),
        ("k7/8/2K5/8/8/8/8/1Q6 w - - 0 1", 2, "KQK stalemate trap"),
        ("k7/2K5/8/8/8/8/8/7R w - - 0 1", 2, "KRK: mate-in-1"),
        ("7k/8/6K1/8/8/8/8/R7 w - - 0 1", 2, "KRK: mate-in-1 (Ra8#)"),
        (
            "k7/2K5/8/3N4/4B3/8/8/8 w - - 0 1",
            12,
            "KBNK: mate within a few moves",
        ),
    ];
    let mut failures = Vec::new();
    for (fen, max_plies, note) in cases {
        let r = search(fen, Some(18), 0);
        if r.score < MATE_SCORE - max_plies {
            failures.push(format!(
                "[{note}] {fen} bound {max_plies} plies: {}",
                summary(&r)
            ));
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

/// `kbnk_corner_preference`: the bare king near the bishop's corner scores
/// better for White, for both bishop colours (Basilisk's positions).
#[test]
fn kbnk_corner_preference() {
    let _g = serial();
    let dark_right = eval_white("8/8/8/8/4K3/5N2/3B4/k7 b - - 0 1");
    let dark_wrong = eval_white("k7/8/8/8/4K3/5N2/3B4/8 b - - 0 1");
    let light_right = eval_white("k7/8/8/8/4K3/3B1N2/8/8 b - - 0 1");
    let light_wrong = eval_white("8/8/8/8/4K3/3B1N2/8/k7 b - - 0 1");
    assert!(
        dark_right > dark_wrong && light_right > light_wrong,
        "dark bishop: a1 {dark_right} vs a8 {dark_wrong}; \
         light bishop: a8 {light_right} vs a1 {light_wrong}"
    );
}

/// `kbbk_drive_gradient`: edge beats centre, corner beats edge, the attacking
/// king's approach is rewarded.
#[test]
fn kbbk_drive_gradient() {
    let _g = serial();
    let centre = eval_white("8/8/8/3k1B2/5B2/4K3/8/8 b - - 0 1");
    let edge = eval_white("8/8/8/5B2/5B2/4K3/8/4k3 b - - 0 1");
    let corner = eval_white("8/8/8/5B2/5B2/2K5/8/k7 b - - 0 1");
    let side = eval_white("8/8/8/5B2/5B2/k1K5/8/8 b - - 0 1");
    let near_king = eval_white("k7/8/2K5/5B2/5B2/8/8/8 b - - 0 1");
    let far_king = eval_white("k7/8/8/5B2/5B2/8/8/6K1 b - - 0 1");
    let mut failures = Vec::new();
    if edge <= centre {
        failures.push(format!("edge {edge} <= centre {centre}"));
    }
    if corner <= side {
        failures.push(format!("corner {corner} <= side {side}"));
    }
    if near_king <= far_king {
        failures.push(format!("near king {near_king} <= far king {far_king}"));
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

/// Plays `fen` for ten plies, one searcher for both sides, `nodes` per move
/// and no depth cap; the ply where White lost a minor, if any.
fn kbnk_playout(fen: &str, nodes: u64) -> Result<(), String> {
    let mut b = board(fen);
    let mut searcher = Searcher::default();
    searcher.new_game();
    let mut line = Vec::new();
    for ply in 0..10 {
        if b.generate_legal_moves().is_empty() {
            break;
        }
        let r = search_board(&mut searcher, b.clone(), None, nodes);
        if r.bestmove.is_null() {
            break;
        }
        line.push(r.bestmove.to_string());
        b.make_move(r.bestmove);
        if b.pieces(Color::White, Piece::Bishop).count() != 1
            || b.pieces(Color::White, Piece::Knight).count() != 1
        {
            return Err(format!(
                "minor lost at ply {ply}: {line:?} -> {}",
                b.to_fen()
            ));
        }
    }
    Ok(())
}

/// `kbnk_keeps_both_minors`: the BAS-E41 anchor at the cohort's 60,000 nodes;
/// the board carries the game's history, as Basilisk's does.
#[test]
fn kbnk_keeps_both_minors() {
    let _g = serial();
    let fen = "8/8/8/2K5/8/3k4/8/N1B5 w - - 0 1";
    let at_60k = kbnk_playout(fen, 60_000);
    // Information only: the neighbouring budgets Rarog's own KBNK anchor uses.
    for nodes in [40_000, 50_000, 80_000, 120_000] {
        eprintln!(
            "KBNK keeps both minors at {nodes}: {:?}",
            kbnk_playout(fen, nodes)
        );
    }
    assert!(at_60k.is_ok(), "{at_60k:?}");
}

/// `pawnless_scale_truth_table`, second half: KQ against two minors is served
/// as the queen's win at depth 1 (no tables loaded, as in Basilisk).
#[test]
fn kq_against_two_minors_is_the_queens_win_at_depth_1() {
    let _g = serial();
    let mut failures = Vec::new();
    for fen in [
        "8/8/3nb3/4k3/8/8/8/K6Q b - - 1 1",
        "8/8/3nn3/4k3/8/8/8/K6Q b - - 1 1",
        "8/8/3bb3/4k3/8/8/8/K6Q b - - 1 1",
    ] {
        let r = search(fen, Some(1), 0);
        if -r.score <= 150 {
            failures.push(format!(
                "{fen}: White's score {} ({})",
                -r.score,
                summary(&r)
            ));
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

/// `pawnless_scale_truth_table`, first half, against Syzygy instead of
/// Basilisk's internal scale factor (Rarog exposes none). None of these classes
/// is in the committed fixture, so this reads `RAROG_SYZYGY_PATH`. A Syzygy win
/// must evaluate as a clear win (>= 150 cp for the winner, the Basilisk bound
/// between a scaled and an unscaled score); a Syzygy draw must not be claimed
/// as one (|eval| < 150), and a Basilisk scale-0 class must read within 25 cp.
#[test]
#[ignore = "D.6.26: KR-KB, KRB-KR and KRN-KR draws read as wins; needs RAROG_SYZYGY_PATH with 4- and 5-man tables"]
fn pawnless_scale_truth_table() {
    let _g = serial();
    let path = std::env::var("RAROG_SYZYGY_PATH").expect("RAROG_SYZYGY_PATH");
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        assert!(syzygy::initialize(&path) >= 5, "5-man tables at {path}");
        // KQ-KNN is mostly the queen's win but this placement is a draw, so a
        // per-position check of it would claim more than an unscaled class.
        let rows: [(&str, i32, &str); 8] = [
            ("4k3/8/8/2bn4/8/8/8/3QK3 b - - 0 1", 64, "KQ-KBN unscaled"),
            ("4k3/8/8/2bb4/8/8/8/3QK3 b - - 0 1", 64, "KQ-KBB unscaled"),
            ("4k3/8/8/2r5/8/8/8/3QK3 b - - 0 1", 64, "KQ-KR unscaled"),
            ("4k3/8/8/2b5/8/8/8/3RK3 b - - 0 1", 4, "KR-KB near draw, 4"),
            ("4k3/8/8/2n5/8/8/8/3BK3 b - - 0 1", 0, "KB-KN dead draw"),
            (
                "4k3/8/8/2r5/8/8/8/2BRK3 b - - 0 1",
                14,
                "KRB-KR near draw, 14",
            ),
            (
                "4k3/8/8/2r5/8/8/8/2NRK3 b - - 0 1",
                14,
                "KRN-KR near draw, 14",
            ),
            (
                "4k3/8/8/2rb4/8/8/8/3QK3 b - - 0 1",
                14,
                "KQ-KRB near draw, 14",
            ),
        ];
        let mut failures = Vec::new();
        for (fen, scale, note) in rows {
            let b = board(fen);
            let wdl = syzygy::probe_wdl(&b, true).unwrap_or_else(|| panic!("probe {fen}"));
            // Black is to move: a black loss is White's win.
            let white = match wdl {
                Wdl::Loss => 1,
                Wdl::Win => -1,
                _ => 0,
            };
            let e = eval_white(fen);
            eprintln!("{note}: Syzygy {wdl:?} (Black to move), eval {e} (White)");
            let ok = match white {
                1 => e >= 150,
                -1 => e <= -150,
                _ if scale == 0 => e.abs() <= 25,
                _ => e.abs() < 150,
            };
            if !ok {
                failures.push(format!("[{note}] {fen}: Syzygy {wdl:?}, eval {e}"));
            }
        }
        assert!(failures.is_empty(), "{failures:#?}");
    }));
    syzygy::initialize("");
    if let Err(panic) = outcome {
        std::panic::resume_unwind(panic);
    }
}
