//! `setoption` and the option list `uci` advertises, from one table: each
//! option's name, type, default and range are written once, and both the
//! advertised line and the range a value is held to come from it.

use std::ops::RangeInclusive;
use std::str::FromStr;

#[cfg(feature = "tune")]
use crate::eval::{FAMILY_MASKS, MaskedFamily};
use crate::infra::to_i32;
use crate::search::options::{EngineOptions, MAX_MULTI_PV, MAX_THREADS};
#[cfg(feature = "tune")]
use crate::search::params::{CoreParams, ProofParams, QuietParams, SearchParams};

/// What a `setoption` did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptionUpdate {
    /// The option took the value; the engine thread must be reconfigured.
    Applied,
    /// The `Clear Hash` button: nothing to store, one action to run.
    ClearHash,
    /// A recognised option whose value was not taken; the setting is
    /// unchanged and the notice says why.
    Rejected { notice: String },
    /// Not an option this engine has, by the name the GUI sent.
    Unknown { name: String },
}

/// Holds `value` to `range` and stores it, at its field's own type; false when
/// the value does not read at that type.
type SpinApply = fn(&mut EngineOptions, &str, &RangeInclusive<i32>) -> bool;

enum Kind {
    Spin {
        default: i32,
        range: RangeInclusive<i32>,
        apply: SpinApply,
    },
    Check {
        default: bool,
        apply: fn(&mut EngineOptions, bool),
    },
    /// A string, advertised with the `<empty>` placeholder as its default.
    Text {
        apply: fn(&mut EngineOptions, String),
    },
    Button,
}

struct UciOption {
    name: &'static str,
    kind: Kind,
}

fn spin(
    name: &'static str,
    default: i32,
    range: RangeInclusive<i32>,
    apply: SpinApply,
) -> UciOption {
    UciOption {
        name,
        kind: Kind::Spin {
            default,
            range,
            apply,
        },
    }
}

fn check(name: &'static str, default: bool, apply: fn(&mut EngineOptions, bool)) -> UciOption {
    UciOption {
        name,
        kind: Kind::Check { default, apply },
    }
}

/// Every option, in the order `uci` lists them.
fn table() -> Vec<UciOption> {
    #[allow(unused_mut, reason = "LargePages is pushed only on Windows")]
    let mut options = vec![
        spin("Hash", 64, 1..=33_554_432, |engine, value, range| {
            store(&mut engine.hash_mb, clamped(value, range))
        }),
        UciOption {
            name: "Clear Hash",
            kind: Kind::Button,
        },
        check("Ponder", false, |engine, on| engine.ponder = on),
        // Fractional milliseconds are taken; a value outside the range is
        // refused, not held to it.
        spin(
            "Move Overhead",
            10,
            0..=5000,
            |engine, value, range| match value.parse::<f64>() {
                Ok(ms)
                    if ms.is_finite()
                        && (f64::from(*range.start())..=f64::from(*range.end())).contains(&ms) =>
                {
                    engine.move_overhead = ms;
                    true
                }
                _ => false,
            },
        ),
        spin(
            "Threads",
            1,
            1..=to_i32(MAX_THREADS),
            |engine, value, range| store(&mut engine.threads, clamped(value, range)),
        ),
        spin(
            "MultiPV",
            1,
            1..=to_i32(MAX_MULTI_PV),
            |engine, value, range| store(&mut engine.multi_pv, clamped(value, range)),
        ),
        UciOption {
            name: "SyzygyPath",
            kind: Kind::Text {
                apply: |engine, path| engine.syzygy.path = path,
            },
        },
        spin("SyzygyProbeDepth", 1, 1..=100, |engine, value, range| {
            store(&mut engine.syzygy.probe_depth, clamped(value, range))
        }),
        spin("SyzygyProbeLimit", 7, 0..=7, |engine, value, range| {
            store(&mut engine.syzygy.probe_limit, clamped(value, range))
        }),
        check("Syzygy50MoveRule", true, |engine, on| {
            engine.syzygy.fifty_move_rule = on;
        }),
    ];
    // Only Windows has the page allocator the option switches.
    #[cfg(windows)]
    options.push(check("LargePages", true, |engine, on| {
        engine.large_pages = on;
    }));
    options
}

impl UciOption {
    fn advertised(&self) -> String {
        let name = self.name;
        match &self.kind {
            Kind::Spin { default, range, .. } => format!(
                "option name {name} type spin default {default} min {} max {}",
                range.start(),
                range.end()
            ),
            Kind::Check { default, .. } => {
                format!("option name {name} type check default {default}")
            }
            Kind::Text { .. } => format!("option name {name} type string default <empty>"),
            Kind::Button => format!("option name {name} type button"),
        }
    }
}

/// `value` read as `T` and held to `range`; `None` when it does not read as
/// `T`, so a negative value for an unsigned field is refused, not raised.
fn clamped<T>(value: &str, range: &RangeInclusive<i32>) -> Option<T>
where
    T: FromStr + TryInto<i128> + TryFrom<i128>,
{
    let wide: i128 = value.parse::<T>().ok()?.try_into().ok()?;
    T::try_from(wide.clamp(i128::from(*range.start()), i128::from(*range.end()))).ok()
}

fn store<T>(field: &mut T, value: Option<T>) -> bool {
    match value {
        Some(value) => {
            *field = value;
            true
        }
        None => false,
    }
}

/// The `option` lines `uci` prints.
pub fn advertised() -> Vec<String> {
    table()
        .iter()
        .map(UciOption::advertised)
        .chain(tune_option_lines())
        .collect()
}

/// The value of a string option, with the `<empty>` placeholder read as the
/// empty string. A GUI echoes back the default the engine advertised, and
/// CuteChess sends this one literally (GitHub issue maelic13/rarog#1): taken
/// at face value it made `SyzygyPath` search a folder called `<empty>` and
/// report "loaded no usable tablebases" at every game. Stockfish maps the same
/// token to an empty string in `ucioption.cpp`.
fn string_option_value(raw: &str) -> String {
    if raw.trim() == "<empty>" {
        String::new()
    } else {
        raw.to_string()
    }
}

/// Apply one `setoption` to `engine`. An invalid value keeps the previous
/// setting and says so; names compare without regard to case.
pub fn apply(engine: &mut EngineOptions, args: &[String]) -> OptionUpdate {
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

    let name = name_parts.join(" ");
    let value_raw = string_option_value(&value_parts.join(" "));
    let value = value_raw.to_lowercase();

    let options = table();
    let Some(option) = options
        .iter()
        .find(|option| option.name.eq_ignore_ascii_case(&name))
    else {
        return apply_tunable(engine, &name, &value);
    };
    let taken = match &option.kind {
        Kind::Spin { range, apply, .. } => apply(engine, &value, range),
        Kind::Check { apply, .. } => match value.as_str() {
            "true" => {
                apply(engine, true);
                true
            }
            "false" => {
                apply(engine, false);
                true
            }
            _ => false,
        },
        Kind::Text { apply } => {
            apply(engine, value_raw);
            true
        }
        Kind::Button => return OptionUpdate::ClearHash,
    };
    if taken {
        OptionUpdate::Applied
    } else {
        OptionUpdate::Rejected {
            notice: format!("Invalid {} value.", option.name),
        }
    }
}

/// The tuning build's options: generated from the single `search_params!`
/// declaration in params.rs, so the strings cannot drift from the defaults
/// and clamps. A production build lists none, so a GUI's list stays short.
#[cfg(feature = "tune")]
fn tune_option_lines() -> Vec<String> {
    let mut lines = SearchParams::uci_option_strings();
    lines.extend(CoreParams::uci_option_strings());
    lines.extend(ProofParams::uci_option_strings());
    lines.extend(QuietParams::uci_option_strings());
    lines.extend(
        [MaskedFamily::KingDanger, MaskedFamily::KingShelter].map(|family| {
            format!(
                "option name {} type check default false",
                family.option_name()
            )
        }),
    );
    lines
}

#[cfg(not(feature = "tune"))]
fn tune_option_lines() -> Vec<String> {
    Vec::new()
}

/// A name the table does not hold: a tuning option in the tuning build,
/// otherwise unknown.
#[cfg(feature = "tune")]
fn apply_tunable(engine: &mut EngineOptions, name: &str, value: &str) -> OptionUpdate {
    let lower = name.to_lowercase();
    match lower.as_str() {
        "kingdangermask" => return set_family_mask(MaskedFamily::KingDanger, value),
        "kingsheltermask" => return set_family_mask(MaskedFamily::KingShelter, value),
        _ => {}
    }
    if engine.search_params.set_uci_option(&lower, value)
        || engine.core_params.set_uci_option(&lower, value)
        || engine.proof_params.set_uci_option(&lower, value)
        || engine.quiet_params.set_uci_option(&lower, value)
    {
        return OptionUpdate::Applied;
    }
    OptionUpdate::Unknown {
        name: name.to_string(),
    }
}

#[cfg(not(feature = "tune"))]
fn apply_tunable(_engine: &mut EngineOptions, name: &str, _value: &str) -> OptionUpdate {
    OptionUpdate::Unknown {
        name: name.to_string(),
    }
}

/// `KingDangerMask` or `KingShelterMask`: an attribution mask, which the
/// first evaluation fixes for the life of the process.
#[cfg(feature = "tune")]
fn set_family_mask(family: MaskedFamily, value: &str) -> OptionUpdate {
    let name = family.option_name();
    let on = match value {
        "true" => true,
        "false" => false,
        _ => {
            return OptionUpdate::Rejected {
                notice: format!("Invalid {name} value."),
            };
        }
    };
    if FAMILY_MASKS.set(family, on).is_err() {
        return OptionUpdate::Rejected {
            notice: format!(
                "{name} is fixed once a position has been evaluated; restart the engine to change it."
            ),
        };
    }
    OptionUpdate::Applied
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The table reproduces the list the engine advertised before it existed,
    /// line for line.
    #[test]
    fn the_table_advertises_the_engine_s_option_list() {
        let mut expected = vec![
            "option name Hash type spin default 64 min 1 max 33554432",
            "option name Clear Hash type button",
            "option name Ponder type check default false",
            "option name Move Overhead type spin default 10 min 0 max 5000",
            "option name Threads type spin default 1 min 1 max 1024",
            "option name MultiPV type spin default 1 min 1 max 256",
            "option name SyzygyPath type string default <empty>",
            "option name SyzygyProbeDepth type spin default 1 min 1 max 100",
            "option name SyzygyProbeLimit type spin default 7 min 0 max 7",
            "option name Syzygy50MoveRule type check default true",
        ];
        if cfg!(windows) {
            expected.push("option name LargePages type check default true");
        }
        let advertised: Vec<String> = table().iter().map(UciOption::advertised).collect();
        assert_eq!(advertised, expected);
    }

    /// Each advertised default is the engine's default: setting it changes
    /// nothing.
    #[test]
    fn every_advertised_default_is_the_engine_s_default() {
        let mut engine = EngineOptions::default();
        for option in table() {
            let value = match option.kind {
                Kind::Spin { default, .. } => default.to_string(),
                Kind::Check { default, .. } => default.to_string(),
                Kind::Text { .. } => String::from("<empty>"),
                Kind::Button => continue,
            };
            let args: Vec<String> = ["name", option.name, "value", &value]
                .iter()
                .map(ToString::to_string)
                .collect();
            assert_eq!(
                apply(&mut engine, &args),
                OptionUpdate::Applied,
                "{}",
                option.name
            );
        }
        let default = EngineOptions::default();
        assert_eq!(engine.hash_mb, default.hash_mb);
        assert_eq!(engine.ponder, default.ponder);
        assert_eq!(engine.move_overhead, default.move_overhead);
        assert_eq!(engine.threads, default.threads);
        assert_eq!(engine.multi_pv, default.multi_pv);
        assert_eq!(engine.large_pages, default.large_pages);
        assert!(engine.syzygy == default.syzygy);
    }
}
