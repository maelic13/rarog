//! External legal-exchange truth and explicit production policy boundaries.
//! Regenerate with tools/diag/see_contract_oracle.py (python-chess).
//! Historical debt rows have active acceptance tests after the 4.11b.5 repair.

use rarog::board::{Board, CROSS_ENGINE_SEE_VALUES, PRODUCTION_SEE_VALUES, SeeValues};

const FIXTURES: &str = include_str!("data/see-contract-v1.tsv");
const REPAIR_FIXTURES: &str = include_str!("data/see-repair-v1.tsv");

fn fixture_lines() -> impl Iterator<Item = &'static str> {
    FIXTURES
        .lines()
        .chain(REPAIR_FIXTURES.lines())
        .filter(|line| !line.starts_with('#'))
}

#[test]
fn report_see_contract_observations() {
    for line in fixture_lines() {
        let f: Vec<_> = line.split('|').collect();
        assert_eq!(f.len(), 6);
        let board = Board::from_fen(f[2]).expect("independent FEN is legal");
        let mv = board.parse_move(f[3]).expect("independent move is legal");
        let truth: i32 = f[4].parse().unwrap();
        let thresholds = [truth - 1, truth, truth + 1, 0];
        let ordinary = thresholds.map(|t| board.see_ge(mv, t));
        let quiet_aware = thresholds.map(|t| board.see_ge_quiet_aware(mv, t));
        println!(
            "{}|{}|tree={truth}|see={}|thresholds={thresholds:?}|ge={ordinary:?}|quiet={quiet_aware:?}",
            f[0],
            f[1],
            board.see(mv)
        );
    }
}

fn verify(selected_debt: Option<&str>) {
    let mut checked = 0;
    for line in fixture_lines() {
        let f: Vec<_> = line.split('|').collect();
        assert_eq!(f.len(), 6);
        if selected_debt.is_some_and(|name| name != f[0]) {
            continue;
        }
        checked += 1;
        let board = Board::from_fen(f[2]).expect("valid fixture");
        let original = board.to_fen();
        let mv = board.parse_move(f[3]).expect("legal fixture move");
        let truth: i32 = f[4].parse().unwrap();
        let immediate: i32 = f[5].parse().unwrap();
        let full_expected = if f[1].starts_with("policy-") {
            immediate
        } else {
            truth
        };
        assert_eq!(board.see(mv), full_expected, "{} full SEE", f[0]);
        for threshold in [
            truth - 1,
            truth,
            truth + 1,
            -301,
            -300,
            -299,
            0,
            1,
            immediate,
            immediate + 1,
        ] {
            assert_eq!(
                board.see_ge(mv, threshold),
                full_expected >= threshold,
                "{} threshold {threshold}",
                f[0]
            );
            let aware_expected = if f[1] == "policy-quiet" {
                truth
            } else {
                full_expected
            };
            assert_eq!(
                board.see_ge_quiet_aware(mv, threshold),
                aware_expected >= threshold,
                "{} quiet-aware {threshold}",
                f[0]
            );
        }
        assert_eq!(board.to_fen(), original, "{} changed board", f[0]);
        board
            .check_consistency()
            .expect("SEE must leave keys/state consistent");
    }
    assert_eq!(checked, if selected_debt.is_some() { 1 } else { 41 });
}

#[test]
fn independent_exchange_and_explicit_shortcut_contracts() {
    verify(None);
}

#[test]
fn repaired_king_exchange_repair() {
    verify(Some("king-after-pawn"));
}

#[test]
fn repaired_created_pin_repair() {
    verify(Some("pin-created"));
}

#[test]
fn repaired_recapture_promotion_repair() {
    verify(Some("promotion-recapture"));
}

fn cross_engine_truth(name: &str) -> i32 {
    let name = name.strip_prefix("mirror-").unwrap_or(name);
    match name {
        "free-pawn" | "pinned-pawn" | "pin-created" | "promoted-piece-recaptured" => 100,
        "defended-pawn" => -400,
        "king-after-pawn" | "pin-created-later" => -300,
        "legal-king-recapture" | "ep-opens-rook" | "castle-king" | "castle-queen" => 0,
        "defended-king-destination" => 900,
        "pin-released" | "skip-pinned-choose-rook" => -200,
        "xray" => 500,
        "quiet-hanging" => -500,
        "quiet-promotion-hanging" => -100,
        "quiet-underpromotion" => 200,
        "capture-promotion" => 1300,
        "capture-underpromotion" => 700,
        "promotion-recapture" => -800,
        "quiet-allows-promotion" => -1300,
        "initial-king-capture" => 300,
        _ => panic!("missing independent cross-engine value for {name}"),
    }
}

#[test]
fn production_and_cross_engine_injection_match_independent_contracts() {
    assert_eq!(
        PRODUCTION_SEE_VALUES.as_array(),
        [100, 320, 330, 500, 900, 20_000]
    );
    assert_eq!(
        CROSS_ENGINE_SEE_VALUES.as_array(),
        [100, 300, 300, 500, 900, 20_000]
    );
    let mut checked = 0;
    for line in fixture_lines() {
        let f: Vec<_> = line.split('|').collect();
        let board = Board::from_fen(f[2]).expect("valid fixture");
        let mv = board.parse_move(f[3]).expect("legal fixture move");
        let production = board.see(mv);
        assert_eq!(
            production,
            board.see_with_values(mv, PRODUCTION_SEE_VALUES),
            "{} production injection",
            f[0]
        );
        let truth = cross_engine_truth(f[0]);
        let full = match f[1] {
            "policy-quiet" | "policy-castle" => 0,
            "policy-promotion" if f[0].contains("underpromotion") => 200,
            "policy-promotion" => 800,
            _ => truth,
        };
        assert_eq!(
            board.see_with_values(mv, CROSS_ENGINE_SEE_VALUES),
            full,
            "{} cross full",
            f[0]
        );
        for threshold in [
            truth - 1,
            truth,
            truth + 1,
            -301,
            -300,
            0,
            1,
            full,
            full + 1,
        ] {
            assert_eq!(
                board.see_ge_with_values(mv, threshold, CROSS_ENGINE_SEE_VALUES),
                full >= threshold,
                "{} cross threshold {threshold}",
                f[0]
            );
            let aware = if f[1] == "policy-quiet" { truth } else { full };
            assert_eq!(
                board.see_ge_quiet_aware_with_values(mv, threshold, CROSS_ENGINE_SEE_VALUES),
                aware >= threshold,
                "{} cross quiet threshold {threshold}",
                f[0]
            );
        }
        checked += 1;
    }
    assert_eq!(checked, 41);
}

#[test]
fn absurd_injected_rook_value_changes_a_known_verdict() {
    let board = Board::from_fen("4k3/8/2p5/3p4/8/8/3R4/4K3 w - - 0 1").unwrap();
    let mv = board.parse_move("d2d5").unwrap();
    assert!(!board.see_ge(mv, 0));
    let absurd = SeeValues::new(100, 300, 300, 1, 900, 20_000);
    assert!(board.see_ge_with_values(mv, 0, absurd));
}

#[test]
fn threshold_parity_on_deterministic_legal_walks() {
    // A parity check complements external truth; it does not define truth.
    let mut checked = 0;
    for seed in [17u64, 83, 211, 997] {
        let mut rng = seed;
        let mut board = Board::from_fen(rarog::board::STARTING_FEN).unwrap();
        for _ in 0..128 {
            let moves = board.generate_legal_moves();
            if moves.is_empty() {
                break;
            }
            for mv in moves.iter().filter(|mv| mv.is_capture()) {
                let value = board.see(*mv);
                for threshold in [value - 1, value, value + 1, -300, 0, 100] {
                    assert_eq!(
                        board.see_ge(*mv, threshold),
                        value >= threshold,
                        "{} {mv} at {threshold}, full={value}",
                        board.to_fen()
                    );
                    assert_eq!(board.see_ge_quiet_aware(*mv, threshold), value >= threshold);
                }
                checked += 1;
            }
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            let index = usize::try_from(rng % u64::try_from(moves.len()).unwrap()).unwrap();
            board.make_move(moves[index]);
        }
    }
    assert!(checked >= 1000, "insufficient capture coverage: {checked}");
    println!("SEE parity: {checked} legal captures across four deterministic walks");
}
