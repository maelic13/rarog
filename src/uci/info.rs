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
    format!(
        "info depth {} seldepth {} multipv {} score {}{} nodes {} nps {} hashfull {} tbhits {} \
         time {} pv {}",
        record.depth,
        record.seldepth,
        record.multipv,
        format_score(record.score),
        bound,
        record.nodes,
        nps(record.nodes, record.elapsed_ms),
        record.hashfull,
        record.tb_hits,
        record.elapsed_ms,
        pv
    )
}

/// Nodes per second for an `info` line. The elapsed time is floored at one
/// millisecond, as Stockfish floors it: dividing by a zero elapsed time made
/// the first lines of every search report `nps` a thousand times too low.
fn nps(nodes: u64, elapsed_ms: u128) -> u128 {
    u128::from(nodes) * 1000 / elapsed_ms.max(1)
}

pub(crate) fn format_score(score: i32) -> String {
    if score >= MATE_SCORE - infra::to_i32(MAX_PLY) {
        format!("mate {}", (MATE_SCORE - score + 1) / 2)
    } else if score <= -MATE_SCORE + infra::to_i32(MAX_PLY) {
        format!("mate -{}", (MATE_SCORE + score + 1) / 2)
    } else if score.abs() >= TB_WIN_SCORE {
        let cp = TB_DISPLAY_CP - (TB_VALUE - score.abs());
        format!("cp {}", score.signum() * cp)
    } else {
        format!("cp {}", score.clamp(-MAX_ORDINARY_CP, MAX_ORDINARY_CP))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A line printed inside the first millisecond used to divide by zero and
    /// fall back to the raw node count: `nodes 49 nps 49` instead of 49,000.
    #[test]
    fn nps_floors_the_elapsed_time_at_one_millisecond() {
        assert_eq!(nps(49, 0), 49_000);
        assert_eq!(nps(49, 1), 49_000);
        assert_eq!(nps(2_000, 4), 500_000);
        assert_eq!(nps(0, 0), 0);
    }

    /// An ordinary score at or above the tablebase band's display printed raw,
    /// so the rule-50-downgraded mate read `cp 31743`, above every tablebase
    /// win.
    #[test]
    fn ordinary_scores_print_below_every_tablebase_win() {
        assert_eq!(format_score(143), "cp 143");
        assert_eq!(format_score(19_872), "cp 19872");
        assert_eq!(format_score(19_873), "cp 19872");
        assert_eq!(format_score(TB_WIN_SCORE - 1), "cp 19872");
        assert_eq!(format_score(-(TB_WIN_SCORE - 1)), "cp -19872");
        assert_eq!(format_score(TB_WIN_SCORE), "cp 19873");
        assert_eq!(format_score(TB_VALUE), "cp 20000");
        assert_eq!(format_score(-TB_WIN_SCORE), "cp -19873");
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
