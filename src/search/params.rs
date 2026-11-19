/// Declares every tunable search parameter **once**: one line generates the
/// struct field, its `Default` value, the tune build's UCI `option` string and
/// the setter's clamp range, so the four cannot disagree and a bake changes
/// one number. Written separately, twelve UCI declarations once advertised
/// stale defaults.
///
/// Syntax: `struct Name, checks_module; field = default, "UciName", min..=max;`
/// — doc comments and plain `//` section comments pass through as normal.
macro_rules! search_params {
    (
        struct $name:ident, $checks:ident;
        $(
            $(#[$meta:meta])*
            $field:ident = $default:literal, $uci:literal, $min:literal ..= $max:literal;
        )+
    ) => {
        /// Tunable search parameters — every field is a UCI `spin` option in
        /// tune builds. Defaults are the current accepted integration-head
        /// values; the trailing comment on each declaration records its bake
        /// history. To re-tune, copy the weather-factory configs from
        /// `tools/spsa_configs/` and run `./tools/spsa.ps1` (see that
        /// directory's README).
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct $name {
            $( $(#[$meta])* pub $field: i32, )+
        }

        impl $name {
            /// The compiled-in values.
            pub const DEFAULT: Self = Self { $( $field: $default, )+ };
        }

        impl Default for $name {
            fn default() -> Self {
                Self::DEFAULT
            }
        }

        impl $name {
            /// UCI `spin` declarations for every tunable, generated from the
            /// same literals as the defaults and clamps — they cannot disagree.
            /// Tune builds only (production must not advertise these).
            #[cfg(feature = "tune")]
            pub fn uci_option_strings() -> Vec<String> {
                vec![$(
                    format!(
                        "option name {} type spin default {} min {} max {}",
                        $uci, $default, $min, $max
                    ),
                )+]
            }

            /// Applies `setoption name <UciName> value <v>` (name matched
            /// case-insensitively), clamping to the declared range. Returns
            /// `false` if the name is not a tunable, so the caller can fall
            /// through to the engine options.
            #[cfg(feature = "tune")]
            pub fn set_uci_option(&mut self, name: &str, value: &str) -> bool {
                $(
                    if name.eq_ignore_ascii_case($uci) {
                        if let Ok(v) = value.parse::<i32>() {
                            self.$field = v.clamp($min, $max);
                        }
                        return true;
                    }
                )+
                false
            }
        }

        #[cfg(test)]
        impl $name {
            /// Every parameter's UCI name, declared range and field, for the
            /// tests that visit each one.
            pub(crate) const FIELDS: &'static [(&'static str, i32, i32, fn(&mut Self) -> &mut i32)] = &[
                $( ($uci, $min, $max, |p: &mut Self| &mut p.$field), )+
            ];
        }

        #[cfg(test)]
        mod $checks {
            use super::*;

            /// Every default must sit inside its own declared range. Cheap, but
            /// it is the invariant the four-way duplication used to break.
            #[test]
            fn defaults_are_within_declared_ranges() {
                let p = $name::default();
                $(
                    assert!(
                        ($min..=$max).contains(&p.$field),
                        "{} default {} is outside [{}, {}]",
                        $uci, p.$field, $min, $max
                    );
                )+
            }

            /// Ranges must be non-empty and ordered.
            #[test]
            fn declared_ranges_are_sane() {
                $( assert!($min < $max, "{} has min >= max", $uci); )+
            }
        }
    };
}

// The single source of truth for every tunable: `field = default, "UciName",
// min..=max;`. Struct field, Default value, UCI option string and setter clamp
// are all generated from these lines — see the `search_params!` docs above.
search_params! {
    struct SearchParams, generated_param_checks;

    /// ⛔ TERMINATION BY CONSTRUCTION. After this many failed searches of one
    /// root window, a side that fails opens to ±INF unconditionally, so it
    /// cannot fail again; a window is searched at most `asp_max_fails + 2`
    /// times whatever the score's magnitude, mate scores included.
    ///
    /// A lower value bounds the re-searches of a runaway iteration, as engines
    /// that open fully after a few fails do; that is the direction the tune
    /// explores.
    asp_max_fails = 20, "AspMaxFails", 1..=32;

    /// Lazy-eval margin: if the tapered material + PST + pawn score already
    /// exceeds it, the positional block is skipped. Pushed into the evaluator
    /// at every search start; the default must equal `eval::LAZY_MARGIN`, the
    /// evaluator's own seed and the margin `texel` fits use.
    lazy_margin = 475, "LazyMargin", 100..=2000;

    // ── Time-management dynamic multipliers ──────────────────────────────────
    // The clock-mode between-iteration soft-stop scales `optimum_ms` by
    // falling-eval × best-move-instability × effort (`search_root` soft-stop block);
    // seeded from Stockfish and exposed for the time-management tune.
    // Stored in ten-thousandths so the float defaults reconstruct bit-exactly
    // (`x / 10000.0` is correctly-rounded, identical to the original literal).
    // TM affects only clock play, never the depth-limited `bench` fingerprint.
    /// Overall multiplier on `optimum_ms` (10000 = ×1.0). The single
    /// highest-leverage TM knob; lets SPSA scale base time allocation. The
    /// range reaches 0.3, the value a harness wire check sets.
    tm_opt_scale = 10_000, "TmOptScale", 2000..=20000;  // ×1.0
    /// Falling-eval base term. Seed 1187 (0.1187).
    tm_fall_base = 1_187, "TmFallBase", 0..=5000;  // 0.1187
    /// Falling-eval slope on `(prev_avg_score - score)`. Seed 221 (0.0221).
    tm_fall_slope = 221, "TmFallSlope", 0..=1000;  // 0.0221
    /// Falling-eval slope on the fall across the move pair: this thread's
    /// previous search's root score in the game less this iteration's. Both
    /// donors read the previous move's score; the slope above reads only this
    /// search. Seed 204 (0.0204, Stockfish's raw cross-move slope; its score
    /// unit is not this engine's, so the fit sets the weight).
    tm_fall_prev_slope = 204, "TmFallPrevSlope", 0..=1000;  // 0.0204
    /// Best-move-instability base. Seed 11000 (1.10).
    tm_instab_base = 11_000, "TmInstabBase", 8000..=16000;  // 1.10
    /// Best-move-instability slope on `tot_best_move_changes`. Seed 22900 (2.29).
    tm_instab_slope = 22_900, "TmInstabSlope", 0..=50000;  // 2.29
    /// Effort factor at or below `TmEffortFloor` of the search's nodes on the
    /// best move. Seed 9240 (0.924); the range reaches an extending shape.
    tm_effort_high = 9_240, "TmEffortHigh", 6000..=20000;  // 0.924
    /// Effort factor when the best move took all of the search's nodes. Seed
    /// 7100 (0.71).
    tm_effort_low = 7_100, "TmEffortLow", 4000..=10000;  // 0.71
    /// Best-move node share at which the effort factor starts to fall from
    /// `TmEffortHigh` toward `TmEffortLow`. Seed 7900 (0.79).
    tm_effort_floor = 7_900, "TmEffortFloor", 0..=9500;  // 0.79
    /// Moves to go assumed per second of clock below one second: the
    /// sudden-death horizon is `min(50, clock_s * this)`, at least one move,
    /// so a short clock stops counting increments it will not live to
    /// receive. Plain moves per second. Seed 50.
    tm_horizon_per_sec = 50, "TmHorizonPerSec", 20..=100;
    /// Stability factor with the best move just changed. Seed 12881 (1.2881).
    tm_stab_base = 12_881, "TmStabBase", 8000..=20000;  // 1.2881
    /// Stability factor's fall per completed iteration the best move has
    /// held. Seed 440 (0.0440).
    tm_stab_slope = 440, "TmStabSlope", 0..=2000;  // 0.0440
    /// Stability factor's floor. Seed 7160 (0.716).
    tm_stab_floor = 7_160, "TmStabFloor", 4000..=10000;  // 0.716
    /// The multiple of the soft target past which a root fail-high ends the
    /// search on the move that failed high, in ten-thousandths: twice the
    /// target. A won ending's score can climb with every re-search, and
    /// without the stop such a move ran to the hard maximum. At twice the
    /// target it fires on well under 1% of searches. At the target itself
    /// (10000) it ended ordinary single fail-highs and cost about 15 Elo at
    /// `3+0.03`. 10000000, a thousand targets, lies past every maximum and
    /// switches it off.
    tm_fail_high_target = 20_000, "TmFailHighTarget", 10000..=10000000;  // 2.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_params_defaults_are_sane() {
        let p = SearchParams::default();
        assert!(p.lazy_margin > 0);
        assert!(p.tm_opt_scale > 0);
        assert!(p.tm_fall_base > 0);
        assert!(p.tm_fall_slope > 0);
        assert!(p.tm_instab_base > 0);
        assert!(p.tm_instab_slope > 0);
        assert!(p.tm_effort_high > 0);
        assert!(p.tm_effort_low > 0);
        assert!(p.tm_effort_floor < 10_000);
        assert!(p.tm_horizon_per_sec > 0);
        assert!(p.tm_stab_floor <= p.tm_stab_base);
    }
}
