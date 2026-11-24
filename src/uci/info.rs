//! The `info` lines a search reports, written from the search's record of
//! them: the score's UCI form, the bound, and the line built in one place.

use crate::score::{MATE_BOUND, MATE_SCORE, TB_VALUE, TB_WIN_SCORE};
use crate::search::{InfoRecord, InfoReport, ScoreBound};

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
    if score >= MATE_BOUND {
        format!("mate {}", (MATE_SCORE - score + 1) / 2)
    } else if score <= -MATE_BOUND {
        format!("mate -{}", (MATE_SCORE + score + 1) / 2)
    } else if score.abs() >= TB_WIN_SCORE {
        let cp = TB_DISPLAY_CP - (TB_VALUE - score.abs());
        format!("cp {}", score.signum() * cp)
    } else {
        format!("cp {}", ordinary_cp(score, material))
    }
}

#[cfg(test)]
mod tests;
