//! Test-data integrity canary (adopted 2026-07-16 after the Basilisk canary
//! post-mortem: its gate had 4 *illegal* endgame positions — side-not-to-move
//! in check — that "passed by coincidence" and made their eval/playout
//! meaningless; and Rarog itself once shipped an illegal 9-pawn bench position).
//!
//! `Board::from_fen` already rejects illegal positions (side-not-to-move in
//! check, adjacent kings, bad pawn counts), and every position set is loaded
//! through it. This test makes that invariant **explicit and consolidated**: a
//! single gate that sweeps every packaged FEN — the `bench` fingerprint suite,
//! the WAC tactical suite, and the endgame regression EPD — so a future data
//! edit that slips in an illegal position fails here with a clear message,
//! naming the source and the FEN, instead of surfacing as a confusing
//! incidental panic in whichever test happens to touch it first.

use rarog::bench::BENCH_FENS;
use rarog::board::Board;
use rarog::wac::wac_positions;

const ENDGAMES_EPD: &str = include_str!("endgames.epd");

/// Extract the FEN from an endgame EPD line (`<FEN> ; <verdict> ...`).
fn endgame_fens() -> Vec<String> {
    ENDGAMES_EPD
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| l.split_once(';').map(|(fen, _)| fen.trim().to_string()))
        .collect()
}

#[test]
fn every_packaged_position_is_legal() {
    let mut checked = 0usize;

    let mut check = |source: &str, fen: &str| {
        checked += 1;
        if let Err(e) = Board::from_fen(fen) {
            panic!("illegal packaged position in {source}: `{fen}` — {e}");
        }
    };

    for fen in BENCH_FENS {
        check("bench (src/bench.rs)", fen);
    }
    for pos in wac_positions() {
        check(&format!("WAC {}", pos.id), &pos.fen);
    }
    for fen in endgame_fens() {
        check("endgames.epd", &fen);
    }

    // Every row of every source, so a parser change that drops some of them
    // fails: 40 bench, 300 WAC and endgames.epd's 91. Adding rows updates the
    // count in the same commit.
    assert_eq!(
        checked, 431,
        "canary swept {checked} positions, not every row of its sources"
    );
}
