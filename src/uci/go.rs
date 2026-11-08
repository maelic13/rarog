//! `go`: one search's limits, or a `perft` count.

use crate::board::UciMove;
use crate::search::options::SearchLimits;

/// What a `go` asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoRequest {
    Search,
    /// `go perft N`: count leaf nodes on the protocol thread; no search runs.
    Perft(u32),
}

/// A parsed `go`: what it asks for, its limits, and a notice for each value
/// that could not be read, in the order the values are applied.
pub struct Go {
    pub request: GoRequest,
    pub limits: SearchLimits,
    pub notices: Vec<String>,
}

/// The keywords of `go`. A `searchmoves` list ends at the next one.
#[derive(Clone, Copy)]
enum GoKeyword {
    SearchMoves,
    Ponder,
    WTime,
    BTime,
    WInc,
    BInc,
    MovesToGo,
    Depth,
    Nodes,
    Perft,
    Mate,
    MoveTime,
    Infinite,
}

impl GoKeyword {
    const COUNT: usize = Self::Infinite as usize + 1;

    fn parse(token: &str) -> Option<Self> {
        Some(match token {
            "searchmoves" => Self::SearchMoves,
            "ponder" => Self::Ponder,
            "wtime" => Self::WTime,
            "btime" => Self::BTime,
            "winc" => Self::WInc,
            "binc" => Self::BInc,
            "movestogo" => Self::MovesToGo,
            "depth" => Self::Depth,
            "nodes" => Self::Nodes,
            "perft" => Self::Perft,
            "mate" => Self::Mate,
            "movetime" => Self::MoveTime,
            "infinite" => Self::Infinite,
            _ => return None,
        })
    }
}

/// Parse the arguments of `go` into fresh limits. Only the first occurrence of
/// a keyword counts, and values are applied in a fixed order, so `mate`
/// overrides `depth` wherever either appears.
pub fn parse(args: &[String]) -> Go {
    let mut limits = SearchLimits {
        issued: Some(std::time::Instant::now()),
        ..SearchLimits::default()
    };
    let mut notices = Vec::new();

    let mut first = [None; GoKeyword::COUNT];
    for (index, token) in args.iter().enumerate() {
        if let Some(keyword) = GoKeyword::parse(token) {
            first[keyword as usize].get_or_insert(index);
        }
    }
    let at = |keyword: GoKeyword| first[keyword as usize];

    limits.ponder = at(GoKeyword::Ponder).is_some();
    limits.infinite = at(GoKeyword::Infinite).is_some();
    if let Some(index) = at(GoKeyword::MoveTime) {
        limits.move_time = parse_or_notice(args, index, "movetime", &mut notices);
    }
    if let Some(index) = at(GoKeyword::WTime) {
        limits.white_time = Some(parse_clock(args, index, "wtime", &mut notices));
    }
    if let Some(index) = at(GoKeyword::WInc) {
        limits.white_increment = parse_or_notice(args, index, "winc", &mut notices);
    }
    if let Some(index) = at(GoKeyword::BTime) {
        limits.black_time = Some(parse_clock(args, index, "btime", &mut notices));
    }
    if let Some(index) = at(GoKeyword::BInc) {
        limits.black_increment = parse_or_notice(args, index, "binc", &mut notices);
    }
    if let Some(index) = at(GoKeyword::Depth) {
        // An unparseable depth searches 2 plies, not the generic zero.
        let parsed = args
            .get(index + 1)
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or_else(|| {
                notices.push(String::from("Invalid depth value."));
                2
            });
        limits.depth = Some(parsed.max(1));
    }
    if let Some(index) = at(GoKeyword::Mate) {
        let mate: usize = parse_or_notice(args, index, "mate", &mut notices);
        if mate > 0 {
            limits.mate = Some(u32::try_from(mate).unwrap_or(u32::MAX));
        }
    }
    if let Some(index) = at(GoKeyword::MovesToGo) {
        limits.movestogo = parse_or_notice(args, index, "movestogo", &mut notices);
    }
    if let Some(index) = at(GoKeyword::Nodes) {
        limits.nodes = parse_or_notice(args, index, "nodes", &mut notices);
    }
    let mut request = GoRequest::Search;
    if let Some(index) = at(GoKeyword::Perft) {
        let depth: u32 = parse_or_notice(args, index, "perft", &mut notices);
        if depth > 0 {
            request = GoRequest::Perft(depth);
        }
    }
    if let Some(index) = at(GoKeyword::SearchMoves) {
        for token in args.iter().skip(index + 1) {
            if GoKeyword::parse(token).is_some() {
                break;
            }
            if let Some(mv) = UciMove::parse(token) {
                limits.search_moves.push(mv);
            } else {
                notices.push(format!("Invalid searchmoves move: {token}"));
                break;
            }
        }
    }
    Go {
        request,
        limits,
        notices,
    }
}

/// A side's clock: the value after `args[index]` in milliseconds, at least 1.
/// A GUI past its flag sends a negative clock, and a zero or unreadable one
/// must not read as no clock, which searches without limit; each moves at once
/// instead, as the donors do.
fn parse_clock(args: &[String], index: usize, name: &str, notices: &mut Vec<String>) -> usize {
    let parsed = args
        .get(index + 1)
        .and_then(|value| value.parse::<i64>().ok());
    if parsed.is_none() {
        notices.push(format!("Invalid {name} value."));
    }
    usize::try_from(parsed.unwrap_or(0).max(1)).unwrap_or(usize::MAX)
}

/// The value after `args[index]`, or zero with a notice when it is missing or
/// does not parse.
fn parse_or_notice<T: std::str::FromStr + Default>(
    args: &[String],
    index: usize,
    name: &str,
    notices: &mut Vec<String>,
) -> T {
    match args.get(index + 1).and_then(|value| value.parse().ok()) {
        Some(value) => value,
        None => {
            notices.push(format!("Invalid {name} value."));
            T::default()
        }
    }
}
