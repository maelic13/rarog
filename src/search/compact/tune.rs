//! The compact kernel's seeds as tuning options: every seed scalar but five
//! is a UCI `spin` option in the `tune` build, its range set by one rule so a
//! fit's surface is the kernel's whole table.

use super::rules::{SEED_FIELDS, Seeds};

/// Seeds no fit can move: the thread spreads multiply the thread index, zero
/// with one thread; a pawn has no cheaper attacker, so its escape never
/// fires; the king's escape is 0 at its natural floor, where a perturbation
/// would clamp; and the pawn's exchange value is the unit of the other
/// four and of every exchange margin, which move together with it without
/// changing a decision.
const EXCLUDED: [&str; 5] = [
    "lmr_spread_thread",
    "fds_spread_thread",
    "pick_escape[0]",
    "pick_escape[5]",
    "exchange_value[0]",
];

/// Seeds whose range is declared rather than the rule's, each for a value
/// the rule's range would not reach: ProbCut's exchange scale spans the
/// unconverted 1,024 and the donor's 2,536.
const DECLARED: [(&str, (i32, i32)); 1] = [("probcut_see_scale", (512, 4_096))];

/// A seed's option name: `K` and the seed's name in Pascal case, an array
/// entry's index appended (`pick_cont[2]` is `KPickCont2`).
fn option_name(seed: &str) -> String {
    let mut name = String::from("K");
    for part in seed.split(['_', '[', ']']).filter(|part| !part.is_empty()) {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            name.extend(first.to_uppercase());
            name.push_str(chars.as_str());
        }
    }
    name
}

/// A seed's range: half to twice its value, the other way round for a
/// negative one, or four units either side of a value under eight, never
/// crossing to the other sign.
fn range(value: i32) -> (i32, i32) {
    if value.abs() >= 8 {
        if value > 0 {
            (value / 2, 2 * value)
        } else {
            (2 * value, value / 2)
        }
    } else if value > 0 {
        ((value - 4).max(1), value + 4)
    } else if value < 0 {
        (value - 4, (value + 4).min(-1))
    } else {
        (value - 4, value + 4)
    }
}

/// Each tunable seed: its option name, field access and range.
fn tunables() -> impl Iterator<Item = (String, fn(&mut Seeds) -> &mut i32, i32, i32)> {
    SEED_FIELDS
        .iter()
        .filter(|(seed, _)| !EXCLUDED.contains(seed))
        .map(|&(seed, field)| {
            let mut seeds = Seeds::FITTED;
            let (min, max) = DECLARED
                .iter()
                .find(|(declared, _)| *declared == seed)
                .map_or_else(|| range(*field(&mut seeds)), |&(_, declared)| declared);
            (option_name(seed), field, min, max)
        })
}

/// The `option` lines a tuning build lists, the kernel's seeds as defaults.
pub(in crate::search) fn uci_option_strings() -> Vec<String> {
    tunables()
        .map(|(name, field, min, max)| {
            let mut seeds = Seeds::FITTED;
            let default = *field(&mut seeds);
            format!("option name {name} type spin default {default} min {min} max {max}")
        })
        .collect()
}

/// Applies `setoption name <name> value <v>` to `seeds` (the name matched
/// case-insensitively), clamped to the seed's range. False when the name is
/// no tunable seed, so the caller tries the other options.
pub(in crate::search) fn set_uci_option(seeds: &mut Seeds, name: &str, value: &str) -> bool {
    let Some((_, field, min, max)) =
        tunables().find(|(option, ..)| option.eq_ignore_ascii_case(name))
    else {
        return false;
    };
    if let Ok(value) = value.trim().parse::<i32>() {
        *field(seeds) = value.clamp(min, max);
    }
    true
}

#[cfg(test)]
mod tests;
