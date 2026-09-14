use crate::board::{Board, Move};
#[cfg(feature = "b2core")]
use crate::search::params::CoreParams;
use crate::search::params::SearchParams;

pub(crate) const MAX_THREADS: usize = 1024;

#[derive(Clone, PartialEq, Eq)]
pub struct SyzygyOptions {
    pub path: String,
    pub probe_depth: i32,
    pub probe_limit: usize,
    pub fifty_move_rule: bool,
}

impl Default for SyzygyOptions {
    fn default() -> Self {
        Self {
            path: String::new(),
            probe_depth: 1,
            probe_limit: 7,
            fifty_move_rule: true,
        }
    }
}

#[derive(Clone)]
pub struct EngineOptions {
    pub move_overhead: f64,
    pub hash_mb: usize,
    pub ponder: bool,
    pub threads: usize,
    pub syzygy: SyzygyOptions,
    pub search_params: SearchParams,
    /// The selectivity core's coordinates.
    #[cfg(feature = "b2core")]
    pub core_params: CoreParams,
}

impl Default for EngineOptions {
    fn default() -> Self {
        Self {
            move_overhead: 10.0,
            hash_mb: 64,
            ponder: false,
            threads: 1,
            syzygy: SyzygyOptions::default(),
            search_params: SearchParams::default(),
            #[cfg(feature = "b2core")]
            core_params: CoreParams::default(),
        }
    }
}

#[derive(Clone, Default)]
pub struct SearchLimits {
    pub move_time: usize,
    pub white_time: usize,
    pub white_increment: usize,
    pub black_time: usize,
    pub black_increment: usize,
    /// Fixed-depth limit from `go depth N` / `go mate N`. `None` = no depth
    /// limit (the search runs to the internal MAX_DEPTH ceiling).
    pub depth: Option<u32>,
    pub movestogo: usize,
    pub nodes: u64,
    pub infinite: bool,
    pub ponder: bool,
    pub search_moves: Vec<Move>,
    /// The instant the `go` command was parsed on the UCI thread.
    ///
    /// The harness charges the clock from the moment it writes `go`, so the
    /// search budget starts there too, as Stockfish's `limits.startTime` and
    /// Reckless's parse-time `TimeManager` do. Stamping on the engine thread
    /// instead would take any hand-off latency under a loaded host straight
    /// off the harness margin. `None` (tests, bench) means the search stamps
    /// its own start.
    pub(crate) issued: Option<std::time::Instant>,
}

/// What a `setoption` changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionUpdate {
    /// An engine option; the engine thread must be reconfigured.
    Engine,
    /// The `Clear Hash` button: nothing to store, one action to run.
    ClearHash,
    /// Not an option this engine has. A notice has been printed.
    Unknown,
}

/// What a `go` asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoRequest {
    Search,
    /// `go perft N`: count leaf nodes on the protocol thread; no search runs.
    Perft(u32),
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

#[derive(Clone, Default)]
pub struct SearchOptions {
    pub board: Board,
    pub engine: EngineOptions,
    pub limits: SearchLimits,
}

impl SearchOptions {
    pub fn get_uci_options() -> Vec<String> {
        // `mut` is needed when compiled with --features tune (the extend
        // below). Stays `allow`, not `expect`: the lint fires under default
        // features and does NOT under `tune`, so an expectation would be
        // unfulfilled in one of the two configurations whichever way it is
        // written. This is the only suppression in the crate with that shape.
        #[allow(unused_mut)]
        let mut opts = vec![
            String::from("option name Hash type spin default 64 min 1 max 33554432"),
            String::from("option name Clear Hash type button"),
            String::from("option name Ponder type check default false"),
            String::from("option name Move Overhead type spin default 10 min 0 max 5000"),
            format!("option name Threads type spin default 1 min 1 max {MAX_THREADS}"),
            String::from("option name SyzygyPath type string default <empty>"),
            String::from("option name SyzygyProbeDepth type spin default 1 min 1 max 100"),
            String::from("option name SyzygyProbeLimit type spin default 7 min 0 max 7"),
            String::from("option name Syzygy50MoveRule type check default true"),
        ];
        // Tunable search parameters — only exposed when compiled with --features tune.
        // weather-factory sets these via UCI setoption; production builds omit them
        // so they don't pollute the option list shown to GUIs.
        // Generated from the single `search_params!` declaration in
        // params.rs, so the strings cannot drift from the defaults and clamps.
        #[cfg(feature = "tune")]
        opts.extend(SearchParams::uci_option_strings());
        #[cfg(all(feature = "tune", feature = "b2core"))]
        opts.extend(CoreParams::uci_option_strings());
        opts
    }

    pub fn reset(&mut self) {
        self.board = Board::default();
        self.limits = SearchLimits::default();
    }

    pub fn set_position(&mut self, args: &[String]) -> Result<(), String> {
        if args.is_empty() {
            return Ok(());
        }

        let mut board = if args[0] == "startpos" {
            Board::default()
        } else if args[0] == "fen" {
            let fen_parts: Vec<&str> = args[1..]
                .iter()
                .take_while(|part| part.as_str() != "moves")
                .map(String::as_str)
                .collect();
            let fen = fen_parts.join(" ");
            match Board::from_fen(&fen) {
                Ok(board) => board,
                Err(_) => {
                    return Err(String::from("Invalid FEN."));
                }
            }
        } else {
            return Ok(());
        };

        let moves_start_index = args
            .iter()
            .position(|part| part == "moves")
            .map_or(args.len(), |index| index + 1);

        for move_text in &args[moves_start_index..] {
            if Move::from_uci(move_text).is_none() {
                return Err(format!("Illegal move: {move_text}"));
            }
            if !board.play_uci(move_text) {
                return Err(format!("Illegal move: {move_text}"));
            }
        }

        self.board = board;
        Ok(())
    }

    /// Parse the arguments of `go` into fresh limits. Only the first
    /// occurrence of a keyword counts, and values are applied in a fixed order,
    /// so `mate` overrides `depth` wherever either appears.
    pub fn set_search_parameters(&mut self, args: &[String]) -> GoRequest {
        self.limits = SearchLimits {
            issued: Some(std::time::Instant::now()),
            ..SearchLimits::default()
        };

        let mut first = [None; GoKeyword::COUNT];
        for (index, token) in args.iter().enumerate() {
            if let Some(keyword) = GoKeyword::parse(token) {
                first[keyword as usize].get_or_insert(index);
            }
        }
        let at = |keyword: GoKeyword| first[keyword as usize];

        self.limits.ponder = at(GoKeyword::Ponder).is_some();
        self.limits.infinite = at(GoKeyword::Infinite).is_some();
        if let Some(index) = at(GoKeyword::MoveTime) {
            self.limits.move_time = Self::parse_or_notice(args, index, "movetime");
        }
        if let Some(index) = at(GoKeyword::WTime) {
            self.limits.white_time = Self::parse_or_notice(args, index, "wtime");
        }
        if let Some(index) = at(GoKeyword::WInc) {
            self.limits.white_increment = Self::parse_or_notice(args, index, "winc");
        }
        if let Some(index) = at(GoKeyword::BTime) {
            self.limits.black_time = Self::parse_or_notice(args, index, "btime");
        }
        if let Some(index) = at(GoKeyword::BInc) {
            self.limits.black_increment = Self::parse_or_notice(args, index, "binc");
        }
        if let Some(index) = at(GoKeyword::Depth) {
            // An unparseable depth searches 2 plies, not the generic zero.
            let parsed = args
                .get(index + 1)
                .and_then(|value| value.parse::<u32>().ok())
                .unwrap_or_else(|| {
                    crate::info_string!("Invalid depth value.");
                    2
                });
            self.limits.depth = Some(parsed.max(1));
        }
        if let Some(index) = at(GoKeyword::Mate) {
            let mate: usize = Self::parse_or_notice(args, index, "mate");
            if mate > 0 {
                // Mate in N -> search 2N-1 plies.
                let plies = mate.saturating_mul(2).saturating_sub(1);
                self.limits.depth = Some(u32::try_from(plies).unwrap_or(u32::MAX).max(1));
            }
        }
        if let Some(index) = at(GoKeyword::MovesToGo) {
            self.limits.movestogo = Self::parse_or_notice(args, index, "movestogo");
        }
        if let Some(index) = at(GoKeyword::Nodes) {
            self.limits.nodes = Self::parse_or_notice(args, index, "nodes");
        }
        let mut request = GoRequest::Search;
        if let Some(index) = at(GoKeyword::Perft) {
            let depth: u32 = Self::parse_or_notice(args, index, "perft");
            if depth > 0 {
                request = GoRequest::Perft(depth);
            }
        }
        if let Some(index) = at(GoKeyword::SearchMoves) {
            for token in args.iter().skip(index + 1) {
                if GoKeyword::parse(token).is_some() {
                    break;
                }
                if let Some(mv) = Move::from_uci(token) {
                    self.limits.search_moves.push(mv);
                } else {
                    crate::info_string!("Invalid searchmoves move: {token}");
                    break;
                }
            }
        }
        request
    }

    /// Apply one `setoption`. An invalid value keeps the previous setting,
    /// with a notice, and still counts as a recognised option.
    pub fn set_option(&mut self, args: &[String]) -> OptionUpdate {
        let mut index = 0;
        if index < args.len() {
            index += 1; // Consume the leading "name" token unconditionally.
        }

        let mut name_parts = Vec::new();
        while index < args.len() && args[index] != "value" {
            name_parts.push(args[index].as_str());
            index += 1;
        }

        let mut value_parts = Vec::new();
        if index < args.len() && args[index] == "value" {
            index += 1;
            while index < args.len() {
                value_parts.push(args[index].as_str());
                index += 1;
            }
        }

        let option_name_raw = name_parts.join(" ");
        let option_name = option_name_raw.to_lowercase();
        let value_raw = value_parts.join(" ");
        let value = value_raw.to_lowercase();

        match option_name.as_str() {
            "hash" => {
                if let Ok(hash_mb) = value.parse::<usize>() {
                    self.engine.hash_mb = hash_mb.clamp(1, 33_554_432);
                } else {
                    crate::info_string!("Invalid Hash value.");
                }
                OptionUpdate::Engine
            }
            "clear hash" => OptionUpdate::ClearHash,
            "ponder" => match value.as_str() {
                "true" => {
                    self.engine.ponder = true;
                    OptionUpdate::Engine
                }
                "false" => {
                    self.engine.ponder = false;
                    OptionUpdate::Engine
                }
                _ => {
                    crate::info_string!("Invalid Ponder value.");
                    OptionUpdate::Engine
                }
            },
            "move overhead" => {
                if let Ok(move_overhead) = value.parse::<f64>()
                    && move_overhead.is_finite()
                    && (0.0..=5000.0).contains(&move_overhead)
                {
                    self.engine.move_overhead = move_overhead;
                } else {
                    crate::info_string!("Invalid Move Overhead value.");
                }
                OptionUpdate::Engine
            }
            "threads" => {
                if let Ok(threads) = value.parse::<usize>() {
                    self.engine.threads = threads.clamp(1, MAX_THREADS);
                } else {
                    crate::info_string!("Invalid Threads value.");
                }
                OptionUpdate::Engine
            }
            "syzygypath" => {
                self.engine.syzygy.path = value_raw;
                OptionUpdate::Engine
            }
            "syzygyprobedepth" => {
                if let Ok(depth) = value.parse::<i32>() {
                    self.engine.syzygy.probe_depth = depth.clamp(1, 100);
                } else {
                    crate::info_string!("Invalid SyzygyProbeDepth value.");
                }
                OptionUpdate::Engine
            }
            "syzygyprobelimit" => {
                if let Ok(limit) = value.parse::<usize>() {
                    self.engine.syzygy.probe_limit = limit.clamp(0, 7);
                } else {
                    crate::info_string!("Invalid SyzygyProbeLimit value.");
                }
                OptionUpdate::Engine
            }
            "syzygy50moverule" => match value.as_str() {
                "true" => {
                    self.engine.syzygy.fifty_move_rule = true;
                    OptionUpdate::Engine
                }
                "false" => {
                    self.engine.syzygy.fifty_move_rule = false;
                    OptionUpdate::Engine
                }
                _ => {
                    crate::info_string!("Invalid Syzygy50MoveRule value.");
                    OptionUpdate::Engine
                }
            },
            // Tunable search parameters — only active when compiled with --features tune.
            _ => {
                // Tunables are matched by the generated
                // `SearchParams::set_uci_option` (one declaration per param in
                // params.rs) instead of ~47 hand-written arms.
                #[cfg(feature = "tune")]
                if self
                    .engine
                    .search_params
                    .set_uci_option(&option_name, &value)
                {
                    return OptionUpdate::Engine;
                }
                #[cfg(all(feature = "tune", feature = "b2core"))]
                if self.engine.core_params.set_uci_option(&option_name, &value) {
                    return OptionUpdate::Engine;
                }
                crate::info_string!("No such option: {option_name_raw}");
                OptionUpdate::Unknown
            }
        }
    }

    /// The value after `args[index]`, or zero with a notice when it is missing
    /// or does not parse.
    fn parse_or_notice<T: std::str::FromStr + Default>(
        args: &[String],
        index: usize,
        name: &str,
    ) -> T {
        match args.get(index + 1).and_then(|value| value.parse().ok()) {
            Some(value) => value,
            None => {
                crate::info_string!("Invalid {name} value.");
                T::default()
            }
        }
    }
}
