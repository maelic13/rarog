/// Declares every tunable search parameter **once**: one line generates the
/// struct field, its `Default` value, the tune build's UCI `option` string and
/// the setter's clamp range, so the four cannot disagree and a bake changes
/// one number.
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
        /// tune builds. Defaults are the current accepted values; the trailing
        /// comment on each declaration gives its real value. To re-tune, copy
        /// the weather-factory configs from
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
            /// it is the invariant that keeps the four generated forms in step.
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
    /// root window, the side that fails opens to ±INF. A fail high still
    /// raises alpha and a fail low lowers beta, so an unstable score can fail
    /// on the other side again; every fail high widens the half-width, so both
    /// sides reach ±INF after a bounded number of searches, mate scores
    /// included. A score that stays consistent ends within `asp_max_fails + 2`.
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

    // ── The clock's soft limit ──────────────────────────────────────────────
    // Between iterations a clock search stops once its time passes the soft
    // bound times five factors (`time::soft_stop`), the donor's form and
    // seeds, tuned there with its search. Stored in ten-thousandths so each
    // seed reconstructs exactly (`x / 10000.0`). The budget's own shape is a
    // constant in `time.rs`, not a coordinate. The clock acts only in clock
    // play, never in the depth-limited `bench`.
    /// The node-share factor at no share: `base - slope * share`.
    tm_nodes_base = 31_838, "TmNodesBase", 15000..=50000;  // 3.1838
    /// The node-share factor's fall per unit of the best move's share.
    tm_nodes_slope = 26_554, "TmNodesSlope", 10000..=40000;  // 2.6554
    /// The node-share factor's floor.
    tm_nodes_floor = 5_460, "TmNodesFloor", 2000..=9000;  // 0.546
    /// The score-trend factor with the score where the previous search left
    /// it: `base + slope * fall`, within the two clamps below.
    tm_trend_base = 7_426, "TmTrendBase", 4000..=12000;  // 0.7426
    /// The score-trend factor's rise per unit the score fell from the
    /// previous search: the donor's 0.0480 a unit of its evaluation, in the
    /// kernel's units (1024/2536 of the donor's).
    tm_trend_slope = 1_189, "TmTrendSlope", 0..=3000;  // 0.1189
    /// The score-trend factor's lower clamp.
    tm_trend_low = 7_214, "TmTrendLow", 4000..=10000;  // 0.7214
    /// The score-trend factor's upper clamp.
    tm_trend_high = 14_031, "TmTrendHigh", 10000..=25000;  // 1.4031
    /// The best-move stability factor with the move just changed: `base -
    /// slope * iterations held`, never below the floor.
    tm_pv_base = 12_881, "TmPvBase", 8000..=20000;  // 1.2881
    /// Its fall per iteration the best move has held.
    tm_pv_slope = 440, "TmPvSlope", 0..=2000;  // 0.0440
    /// Its floor.
    tm_pv_floor = 7_160, "TmPvFloor", 4000..=10000;  // 0.716
    /// The score stability factor with the score just moved: `base - slope *
    /// iterations held near the root's average`, never below the floor.
    tm_eval_base = 12_664, "TmEvalBase", 8000..=20000;  // 1.2664
    /// Its fall per iteration the score has held.
    tm_eval_slope = 416, "TmEvalSlope", 0..=2000;  // 0.0416
    /// Its floor.
    tm_eval_floor = 8_642, "TmEvalFloor", 5000..=10000;  // 0.8642
    /// The best-move change factor with no change in the iteration: `base +
    /// ln(1 + scale * changes)`.
    tm_changes_base = 11_500, "TmChangesBase", 7000..=18000;  // 1.15
    /// The change count's scale inside the log.
    tm_changes_scale = 2_526, "TmChangesScale", 0..=8000;  // 0.2526
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
        assert!(p.tm_nodes_floor < p.tm_nodes_base);
        assert!(p.tm_trend_low < p.tm_trend_base && p.tm_trend_base < p.tm_trend_high);
        assert!(p.tm_pv_floor <= p.tm_pv_base);
        assert!(p.tm_eval_floor <= p.tm_eval_base);
        assert!(p.tm_changes_base > 0 && p.tm_changes_scale >= 0);
    }
}
