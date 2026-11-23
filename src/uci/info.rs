//! The `info` lines a search reports, written from the search's record of
//! them: the score's UCI form, the bound, and the line built in one place.

use crate::eval::MATE_SCORE;
use crate::infra;
use crate::search::{InfoRecord, InfoReport, MAX_PLY, ScoreBound};
use crate::tt::{TB_VALUE, TB_WIN_SCORE};

/// What a tablebase win at the root prints, in centipawns; one less per ply
/// further from the root, so a GUI reads a decided game, not a 300-pawn
/// evaluation.
const TB_DISPLAY_CP: i32 = 20_000;

/// The most an ordinary score prints: one below the lowest tablebase win, so
/// a score that is neither a mate nor a tablebase result, the rule-50
/// downgraded mate (`TB_WIN_SCORE - 1`) among them, never reads as decided.
const MAX_ORDINARY_CP: i32 = TB_DISPLAY_CP - (TB_VALUE - TB_WIN_SCORE) - 1;

/// The win-rate model an ordinary score prints through, fitted on the
/// engine's own games by `tools/wdl_fit.py`: a side scored `x` at material
/// `m` wins with probability `1 / (1 + exp((a(m) - x) / b(m)))`, loses with
/// the same at `-x`, and draws otherwise. `a` and `b` are cubics in `m / 58`,
/// from the cube down, `m` held to the fitted range. A refit of the
/// evaluation or the search refits these.
const WDL_A: [f64; 4] = [
    -145.416_906_82,
    467.735_974_4,
    -495.457_311_77,
    256.829_234_84,
];
const WDL_B: [f64; 4] = [43.967_439_66, -59.727_306_25, 26.909_525_93, 33.482_499_1];

/// The material range the model was fitted on, and its anchor.
const WDL_MATERIAL: (i32, i32) = (17, 78);
const WDL_ANCHOR: f64 = 58.0;

/// The model's `a` and `b` at a root's material.
fn win_rate_params(material: i32) -> (f64, f64) {
    let m = f64::from(material.clamp(WDL_MATERIAL.0, WDL_MATERIAL.1)) / WDL_ANCHOR;
    let cubic = |c: [f64; 4]| ((c[0] * m + c[1]) * m + c[2]) * m + c[3];
    (cubic(WDL_A), cubic(WDL_B))
}

/// An ordinary score in centipawns: `100 x / a(m)`, so +100 is the score at
/// which the side wins half its games at that material. Held below the
/// tablebase band.
fn ordinary_cp(score: i32, material: i32) -> i32 {
    let (a, _) = win_rate_params(material);
    let cp = (100.0 * f64::from(score) / a).round();
    let limit = f64::from(MAX_ORDINARY_CP);
    // The clamp keeps the value inside `i32`, so the cast is exact.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "clamped to the ordinary band first"
    )]
    let cp = cp.clamp(-limit, limit) as i32;
    cp
}

/// The win, draw and loss rates in permille for the side to move, as UCI's
/// `wdl` prints them. A mate or a tablebase result is decided.
fn wdl(score: i32, material: i32) -> [u32; 3] {
    if score.abs() >= TB_WIN_SCORE {
        return if score > 0 {
            [1000, 0, 0]
        } else {
            [0, 0, 1000]
        };
    }
    let (a, b) = win_rate_params(material);
    let rate = |x: f64| -> u32 {
        // A rate in [0, 1] times 1,000, rounded: the cast is exact.
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a permille in [0, 1000]"
        )]
        let permille = (1000.0 / (1.0 + ((a - x) / b).exp())).round() as u32;
        permille
    };
    let win = rate(f64::from(score));
    let loss = rate(-f64::from(score));
    [win, 1000 - win - loss, loss]
}

/// The line a report prints, without the trailing newline.
pub fn report_line(report: &InfoReport<'_>) -> String {
    match report {
        InfoReport::Iteration(record) => iteration_line(record),
        // Both donors report what a root with no legal move is worth; a GUI
        // otherwise has a `bestmove` and no score to show.
        InfoReport::NoLegalMove { in_check: true } => String::from("info depth 0 score mate 0"),
        InfoReport::NoLegalMove { in_check: false } => String::from("info depth 0 score cp 0"),
    }
}

fn iteration_line(record: &InfoRecord<'_>) -> String {
    let bound = match record.bound {
        ScoreBound::Exact => "",
        ScoreBound::Lower => " lowerbound",
        ScoreBound::Upper => " upperbound",
    };
    let pv = record
        .pv
        .iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>()
        .join(" ");
    let wdl = if record.show_wdl {
        let [win, draw, loss] = wdl(record.score, record.material);
        format!(" wdl {win} {draw} {loss}")
    } else {
        String::new()
    };
    format!(
        "info depth {} seldepth {} multipv {} score {}{}{} nodes {} nps {} hashfull {} tbhits {} \
         time {} pv {}",
        record.depth,
        record.seldepth,
        record.multipv,
        format_score(record.score, record.material),
        bound,
        wdl,
        record.nodes,
        nps(record.nodes, record.elapsed_ms),
        record.hashfull,
        record.tb_hits,
        record.elapsed_ms,
        pv
    )
}

/// Nodes per second for an `info` line. The elapsed time is floored at one
/// millisecond, as Stockfish floors it, so a line printed at zero elapsed time
/// still reports a rate.
fn nps(nodes: u64, elapsed_ms: u128) -> u128 {
    u128::from(nodes) * 1000 / elapsed_ms.max(1)
}

/// A score's UCI form at a root's material: a mate in moves, a tablebase
/// result in its display band, an ordinary score through the win-rate model.
pub(crate) fn format_score(score: i32, material: i32) -> String {
    if score >= MATE_SCORE - infra::to_i32(MAX_PLY) {
        format!("mate {}", (MATE_SCORE - score + 1) / 2)
    } else if score <= -MATE_SCORE + infra::to_i32(MAX_PLY) {
        format!("mate -{}", (MATE_SCORE + score + 1) / 2)
    } else if score.abs() >= TB_WIN_SCORE {
        let cp = TB_DISPLAY_CP - (TB_VALUE - score.abs());
        format!("cp {}", score.signum() * cp)
    } else {
        format!("cp {}", ordinary_cp(score, material))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A line printed inside the first millisecond reports `nps` from a
    /// one-millisecond floor: 49 nodes read 49,000, not the raw count.
    #[test]
    fn nps_floors_the_elapsed_time_at_one_millisecond() {
        assert_eq!(nps(49, 0), 49_000);
        assert_eq!(nps(49, 1), 49_000);
        assert_eq!(nps(2_000, 4), 500_000);
        assert_eq!(nps(0, 0), 0);
    }

    /// An ordinary score is held below the tablebase band's display, so the
    /// rule-50-downgraded mate never reads above a tablebase win.
    #[test]
    fn ordinary_scores_print_below_every_tablebase_win() {
        // The band's width moves with the ply cap; the display keeps every
        // tablebase win above every ordinary score, and both far above any
        // evaluation.
        let top = MAX_ORDINARY_CP;
        assert!(top > 19_000, "{top}");
        for material in [0, 17, 30, 58, 78, 100] {
            assert_eq!(
                format_score(TB_WIN_SCORE - 1, material),
                format!("cp {top}")
            );
            assert_eq!(
                format_score(-(TB_WIN_SCORE - 1), material),
                format!("cp -{top}")
            );
            assert_eq!(
                format_score(TB_WIN_SCORE, material),
                format!("cp {}", top + 1)
            );
            assert_eq!(format_score(TB_VALUE, material), "cp 20000");
            assert_eq!(
                format_score(-TB_WIN_SCORE, material),
                format!("cp -{}", top + 1)
            );
            assert_eq!(format_score(MATE_SCORE - 3, material), "mate 2");
            assert_eq!(format_score(-(MATE_SCORE - 4), material), "mate -2");
        }
    }

    /// The model is defined over every material a board can hold: `a` and
    /// `b` stay positive, so the three rates are probabilities.
    #[test]
    fn the_model_is_defined_at_every_material() {
        for material in 0..=104 {
            let (a, b) = win_rate_params(material);
            assert!(a > 0.0 && b > 0.0, "material {material}: a {a}, b {b}");
        }
        assert_eq!(
            win_rate_params(5),
            win_rate_params(17),
            "held to the fitted range"
        );
        assert_eq!(win_rate_params(90), win_rate_params(78));
    }

    /// +100 is where the side wins half its games, at any material, and the
    /// printed score never reverses the order of two scores.
    #[test]
    fn a_hundred_is_an_even_chance_of_winning_and_the_map_is_monotone() {
        for material in [17, 30, 45, 58, 78] {
            let (a, _) = win_rate_params(material);
            // The integer score nearest `a`, which the model calls an even chance.
            #[expect(clippy::cast_possible_truncation, reason = "a is a few hundred")]
            let even = a.round() as i32;
            let cp = ordinary_cp(even, material);
            assert!(
                (99..=101).contains(&cp),
                "material {material}: {even} prints {cp}"
            );
            let [win, _, _] = wdl(even, material);
            assert!((495..=505).contains(&win), "material {material}: win {win}");
            let mut last = i32::MIN;
            for score in (-3_000..=3_000).step_by(7) {
                let cp = ordinary_cp(score, material);
                assert!(
                    cp >= last,
                    "material {material}: {score} prints {cp} after {last}"
                );
                last = cp;
            }
            assert_eq!(ordinary_cp(0, material), 0);
        }
    }

    #[test]
    fn the_rates_sum_to_a_thousand_mirror_and_decide_mates() {
        for material in [17, 58, 78] {
            for score in [-900, -150, -1, 0, 1, 150, 900] {
                let [win, draw, loss] = wdl(score, material);
                assert_eq!(win + draw + loss, 1000);
                assert_eq!(wdl(-score, material), [loss, draw, win]);
            }
            assert_eq!(wdl(MATE_SCORE - 5, material), [1000, 0, 0]);
            assert_eq!(wdl(-TB_VALUE, material), [0, 0, 1000]);
        }
    }

    /// `wdl` follows the score and its bound, as Stockfish prints it, and only
    /// when asked for.
    #[test]
    fn the_rates_follow_the_score_only_when_shown() {
        let pv = [crate::board::Move::from_uci("e2e4").expect("valid move")];
        let record = |show_wdl| InfoRecord {
            depth: 9,
            seldepth: 12,
            multipv: 1,
            score: 90,
            bound: ScoreBound::Lower,
            nodes: 1_000,
            hashfull: 3,
            tb_hits: 0,
            elapsed_ms: 10,
            pv: &pv,
            material: 78,
            show_wdl,
        };
        let shown = iteration_line(&record(true));
        let [win, draw, loss] = wdl(90, 78);
        let expected = format!(
            " score {} lowerbound wdl {win} {draw} {loss} nodes ",
            format_score(90, 78)
        );
        assert!(shown.contains(&expected), "{shown}");
        let plain = iteration_line(&record(false));
        assert!(!plain.contains(" wdl "), "{plain}");
        assert!(
            plain.contains(&format!(
                " score {} lowerbound nodes ",
                format_score(90, 78)
            )),
            "{plain}"
        );
    }

    #[test]
    fn a_root_with_no_legal_move_reports_mate_or_a_draw() {
        assert_eq!(
            report_line(&InfoReport::NoLegalMove { in_check: true }),
            "info depth 0 score mate 0"
        );
        assert_eq!(
            report_line(&InfoReport::NoLegalMove { in_check: false }),
            "info depth 0 score cp 0"
        );
    }
}
