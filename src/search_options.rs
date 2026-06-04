use crate::board::{Board, Move};
use crate::params::SearchParams;

pub const MAX_THREADS: usize = 1024;

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
    pub clear_hash: bool,
    pub ponder: bool,
    pub threads: usize,
    pub syzygy: SyzygyOptions,
    pub search_params: SearchParams,
}

impl Default for EngineOptions {
    fn default() -> Self {
        Self {
            move_overhead: 10.0,
            hash_mb: 64,
            clear_hash: false,
            ponder: false,
            threads: 1,
            syzygy: SyzygyOptions::default(),
            search_params: SearchParams::default(),
        }
    }
}

#[derive(Clone)]
pub struct PositionState {
    pub board: Board,
}

impl Default for PositionState {
    fn default() -> Self {
        Self {
            board: Board::default(),
        }
    }
}

#[derive(Clone)]
pub struct SearchLimits {
    pub move_time: usize,
    pub white_time: usize,
    pub white_increment: usize,
    pub black_time: usize,
    pub black_increment: usize,
    pub depth: f64,
    pub movestogo: usize,
    pub nodes: u64,
    pub perft: u32,
    pub infinite: bool,
    pub ponder: bool,
    pub search_moves: Vec<Move>,
}

impl SearchLimits {
    fn reset_temporary_parameters(&mut self) {
        self.move_time = 0;
        self.white_time = 0;
        self.white_increment = 0;
        self.black_time = 0;
        self.black_increment = 0;
        self.depth = f64::INFINITY;
        self.movestogo = 0;
        self.nodes = 0;
        self.perft = 0;
        self.infinite = false;
        self.ponder = false;
        self.search_moves.clear();
    }
}

impl Default for SearchLimits {
    fn default() -> Self {
        Self {
            move_time: 0,
            white_time: 0,
            white_increment: 0,
            black_time: 0,
            black_increment: 0,
            depth: f64::INFINITY,
            movestogo: 0,
            nodes: 0,
            perft: 0,
            infinite: false,
            ponder: false,
            search_moves: Vec::new(),
        }
    }
}

#[derive(Clone, Default)]
pub struct SearchOptions {
    pub position: PositionState,
    pub engine: EngineOptions,
    pub limits: SearchLimits,
}

impl SearchOptions {
    pub fn get_uci_options() -> Vec<String> {
        // `mut` is needed when compiled with --features tune (the extend below).
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
        #[cfg(feature = "tune")]
        opts.extend([
            String::from("option name AspirationDelta type spin default 25 min 5 max 100"),
            String::from("option name FutilityBase type spin default 70 min 20 max 200"),
            String::from("option name FutilityImproving type spin default 20 min 0 max 80"),
            String::from("option name RazoringCoeff type spin default 150 min 50 max 300"),
            String::from("option name NullMoveDepthCoeff type spin default 12 min 2 max 40"),
            String::from("option name NullMoveImprovingBonus type spin default 24 min 0 max 80"),
            String::from("option name LmpBase type spin default 90 min 30 max 200"),
            String::from("option name LmpImproving type spin default 25 min 0 max 80"),
            String::from("option name QuietHistPruneCoeff type spin default 4000 min 1000 max 10000"),
            String::from("option name SeePruningCoeff type spin default 80 min 20 max 200"),
            String::from("option name SeePruningMax type spin default 800 min 200 max 1600"),
            String::from("option name SingularBetaMult type spin default 2 min 1 max 8"),
            String::from("option name LmpCountBase type spin default 4 min 1 max 12"),
            // LMR weighted adjustments (1024ths of a ply; default 1024 = 1 ply = current behavior).
            String::from("option name LmrTtPvAdj type spin default 1024 min 0 max 2048"),
            String::from("option name LmrExactBound type spin default 0 min 0 max 2048"),
            String::from("option name LmrShallowTt type spin default 1024 min 0 max 2048"),
            String::from("option name LmrCutNode type spin default 1024 min 0 max 2048"),
        ]);
        opts
    }

    pub fn reset(&mut self) {
        self.position = PositionState::default();
        self.limits.reset_temporary_parameters();
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

        self.position.board = board;
        Ok(())
    }

    pub fn set_search_parameters(&mut self, args: &[String]) {
        self.limits.reset_temporary_parameters();

        self.limits.ponder = args.iter().any(|r| r == "ponder");

        let infinite_index = args.iter().position(|r| r == "infinite");
        if infinite_index.is_some() {
            self.limits.depth = f64::INFINITY;
            self.limits.infinite = true;
        }

        let move_time_index = args.iter().position(|r| r == "movetime");
        let white_time_index = args.iter().position(|r| r == "wtime");
        let white_increment_index = args.iter().position(|r| r == "winc");
        let black_time_index = args.iter().position(|r| r == "btime");
        let black_increment_index = args.iter().position(|r| r == "binc");
        let depth_index = args.iter().position(|r| r == "depth");
        let mate_index = args.iter().position(|r| r == "mate");
        let movestogo_index = args.iter().position(|r| r == "movestogo");
        let nodes_index = args.iter().position(|r| r == "nodes");
        let perft_index = args.iter().position(|r| r == "perft");
        let searchmoves_index = args.iter().position(|r| r == "searchmoves");

        if let Some(index) = move_time_index {
            self.limits.move_time = Self::parse_usize(args, index, "movetime");
        }

        if let Some(index) = white_time_index {
            self.limits.white_time = Self::parse_usize(args, index, "wtime");
        }
        if let Some(index) = white_increment_index {
            self.limits.white_increment = Self::parse_usize(args, index, "winc");
        }
        if let Some(index) = black_time_index {
            self.limits.black_time = Self::parse_usize(args, index, "btime");
        }
        if let Some(index) = black_increment_index {
            self.limits.black_increment = Self::parse_usize(args, index, "binc");
        }
        if let Some(index) = depth_index {
            self.limits.depth = Self::parse_f64(args, index, "depth");
        }
        if let Some(index) = mate_index {
            let mate = Self::parse_usize(args, index, "mate");
            if mate > 0 {
                self.limits.depth = mate.saturating_mul(2).saturating_sub(1) as f64;
            }
        }
        if let Some(index) = movestogo_index {
            self.limits.movestogo = Self::parse_usize(args, index, "movestogo");
        }
        if let Some(index) = nodes_index {
            self.limits.nodes = Self::parse_u64(args, index, "nodes");
        }
        if let Some(index) = perft_index {
            self.limits.perft = Self::parse_u32(args, index, "perft");
        }
        if let Some(index) = searchmoves_index {
            for token in args.iter().skip(index + 1) {
                if Self::is_go_parameter(token) {
                    break;
                }
                if let Some(mv) = Move::from_uci(token) {
                    self.limits.search_moves.push(mv);
                } else {
                    println!("info string Invalid searchmoves move: {token}");
                    break;
                }
            }
        }
    }

    pub fn set_option(&mut self, args: &[String]) -> bool {
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
                    println!("info string Invalid Hash value.");
                }
                true
            }
            "clear hash" => {
                self.engine.clear_hash = true;
                true
            }
            "ponder" => match value.as_str() {
                "true" => {
                    self.engine.ponder = true;
                    true
                }
                "false" => {
                    self.engine.ponder = false;
                    true
                }
                _ => {
                    println!("info string Invalid Ponder value.");
                    true
                }
            },
            "move overhead" => {
                if let Ok(move_overhead) = value.parse::<f64>()
                    && move_overhead.is_finite()
                    && (0.0..=5000.0).contains(&move_overhead)
                {
                    self.engine.move_overhead = move_overhead;
                } else {
                    println!("info string Invalid Move Overhead value.");
                }
                true
            }
            "threads" => {
                if let Ok(threads) = value.parse::<usize>() {
                    self.engine.threads = threads.clamp(1, MAX_THREADS);
                } else {
                    println!("info string Invalid Threads value.");
                }
                true
            }
            "syzygypath" => {
                self.engine.syzygy.path = value_raw;
                true
            }
            "syzygyprobedepth" => {
                if let Ok(depth) = value.parse::<i32>() {
                    self.engine.syzygy.probe_depth = depth.clamp(1, 100);
                } else {
                    println!("info string Invalid SyzygyProbeDepth value.");
                }
                true
            }
            "syzygyprobelimit" => {
                if let Ok(limit) = value.parse::<usize>() {
                    self.engine.syzygy.probe_limit = limit.clamp(0, 7);
                } else {
                    println!("info string Invalid SyzygyProbeLimit value.");
                }
                true
            }
            "syzygy50moverule" => match value.as_str() {
                "true" => {
                    self.engine.syzygy.fifty_move_rule = true;
                    true
                }
                "false" => {
                    self.engine.syzygy.fifty_move_rule = false;
                    true
                }
                _ => {
                    println!("info string Invalid Syzygy50MoveRule value.");
                    true
                }
            },
            // Tunable search parameters — only active when compiled with --features tune.
            #[cfg(feature = "tune")]
            "aspirationdelta" => {
                if let Ok(v) = value.parse::<i32>() {
                    self.engine.search_params.aspiration_delta = v.clamp(5, 100);
                }
                true
            }
            #[cfg(feature = "tune")]
            "futilitybase" => {
                if let Ok(v) = value.parse::<i32>() {
                    self.engine.search_params.futility_base = v.clamp(20, 200);
                }
                true
            }
            #[cfg(feature = "tune")]
            "futilityimproving" => {
                if let Ok(v) = value.parse::<i32>() {
                    self.engine.search_params.futility_improving = v.clamp(0, 80);
                }
                true
            }
            #[cfg(feature = "tune")]
            "razoringcoeff" => {
                if let Ok(v) = value.parse::<i32>() {
                    self.engine.search_params.razoring_coeff = v.clamp(50, 300);
                }
                true
            }
            #[cfg(feature = "tune")]
            "nullmovedepthcoeff" => {
                if let Ok(v) = value.parse::<i32>() {
                    self.engine.search_params.nm_depth_coeff = v.clamp(2, 40);
                }
                true
            }
            #[cfg(feature = "tune")]
            "nullmoveimprovingbonus" => {
                if let Ok(v) = value.parse::<i32>() {
                    self.engine.search_params.nm_improving_bonus = v.clamp(0, 80);
                }
                true
            }
            #[cfg(feature = "tune")]
            "lmpbase" => {
                if let Ok(v) = value.parse::<i32>() {
                    self.engine.search_params.lmp_base = v.clamp(30, 200);
                }
                true
            }
            #[cfg(feature = "tune")]
            "lmpimproving" => {
                if let Ok(v) = value.parse::<i32>() {
                    self.engine.search_params.lmp_improving = v.clamp(0, 80);
                }
                true
            }
            #[cfg(feature = "tune")]
            "quiethistprunecoeff" => {
                if let Ok(v) = value.parse::<i32>() {
                    self.engine.search_params.quiet_hist_prune_coeff = v.clamp(1_000, 10_000);
                }
                true
            }
            #[cfg(feature = "tune")]
            "seepruningcoeff" => {
                if let Ok(v) = value.parse::<i32>() {
                    self.engine.search_params.see_pruning_coeff = v.clamp(20, 200);
                }
                true
            }
            #[cfg(feature = "tune")]
            "seepruningmax" => {
                if let Ok(v) = value.parse::<i32>() {
                    self.engine.search_params.see_pruning_max = v.clamp(200, 1_600);
                }
                true
            }
            #[cfg(feature = "tune")]
            "singularbetamult" => {
                if let Ok(v) = value.parse::<i32>() {
                    self.engine.search_params.singular_beta_mult = v.clamp(1, 8);
                }
                true
            }
            #[cfg(feature = "tune")]
            "lmpcountbase" => {
                if let Ok(v) = value.parse::<i32>() {
                    self.engine.search_params.lmp_count_base = v.clamp(1, 12);
                }
                true
            }
            #[cfg(feature = "tune")]
            "lmrttpvadj" => {
                if let Ok(v) = value.parse::<i32>() {
                    self.engine.search_params.lmr_tt_pv_adj = v.clamp(0, 2_048);
                }
                true
            }
            #[cfg(feature = "tune")]
            "lmrexactbound" => {
                if let Ok(v) = value.parse::<i32>() {
                    self.engine.search_params.lmr_exact_bound = v.clamp(0, 2_048);
                }
                true
            }
            #[cfg(feature = "tune")]
            "lmrshallowtt" => {
                if let Ok(v) = value.parse::<i32>() {
                    self.engine.search_params.lmr_shallow_tt = v.clamp(0, 2_048);
                }
                true
            }
            #[cfg(feature = "tune")]
            "lmrcutnode" => {
                if let Ok(v) = value.parse::<i32>() {
                    self.engine.search_params.lmr_cut_node = v.clamp(0, 2_048);
                }
                true
            }
            _ => {
                println!("No such option: {option_name_raw}");
                false
            }
        }
    }

    fn parse_usize(args: &[String], index: usize, name: &str) -> usize {
        match args.get(index + 1).and_then(|value| value.parse().ok()) {
            Some(value) => value,
            None => {
                println!("info string Invalid {} value.", name);
                0
            }
        }
    }

    fn parse_u64(args: &[String], index: usize, name: &str) -> u64 {
        match args.get(index + 1).and_then(|value| value.parse().ok()) {
            Some(value) => value,
            None => {
                println!("info string Invalid {} value.", name);
                0
            }
        }
    }

    fn parse_f64(args: &[String], index: usize, name: &str) -> f64 {
        match args.get(index + 1).and_then(|value| value.parse().ok()) {
            Some(value) => value,
            None => {
                println!("info string Invalid {} value.", name);
                2.0
            }
        }
    }

    fn parse_u32(args: &[String], index: usize, name: &str) -> u32 {
        match args.get(index + 1).and_then(|value| value.parse().ok()) {
            Some(value) => value,
            None => {
                println!("info string Invalid {} value.", name);
                0
            }
        }
    }

    fn is_go_parameter(token: &str) -> bool {
        matches!(
            token,
            "searchmoves"
                | "ponder"
                | "wtime"
                | "btime"
                | "winc"
                | "binc"
                | "movestogo"
                | "depth"
                | "nodes"
                | "perft"
                | "mate"
                | "movetime"
                | "infinite"
        )
    }
}
