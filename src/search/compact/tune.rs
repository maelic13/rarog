//! The compact kernel's seeds as tuning options: every seed scalar but five
//! is a UCI `spin` option in the `tune` build, its range set by one rule so a
//! fit's surface is the kernel's whole table.

use super::rules::{SEED_FIELDS, Seeds};

/// Seeds no fit can move: the thread spreads multiply the thread index, zero
/// with one thread; a pawn has no cheaper attacker, so its escape never
/// fires; the king's escape is 0 at its natural floor, where a perturbation
/// would clamp; and the correction's scale into exchange units is the unit
/// conversion itself, a coefficient the donor's form does not have.
const EXCLUDED: [&str; 5] = [
    "lmr_spread_thread",
    "fds_spread_thread",
    "pick_escape[0]",
    "pick_escape[5]",
    "q_see_correction_scale",
];

/// Seeds whose range is declared rather than the rule's, each for a value
/// the rule's range would not reach: ProbCut's exchange scale spans the
/// unconverted 1,024 and the donor's 2,536 the kernel plays.
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
mod tests {
    use super::*;

    #[test]
    fn every_seed_but_the_five_is_an_option_inside_its_range() {
        let lines = uci_option_strings();
        assert_eq!(SEED_FIELDS.len(), 247);
        assert_eq!(lines.len(), 242);
        for (name, field, min, max) in tunables() {
            let mut seeds = Seeds::FITTED;
            let value = *field(&mut seeds);
            assert!(
                min < max && (min..=max).contains(&value),
                "{name}: {value} in [{min}, {max}]"
            );
            assert!(
                value == 0 || (min.signum() == value.signum() && max.signum() == value.signum())
            );
        }
        let mut names: Vec<String> = tunables().map(|(name, ..)| name.to_lowercase()).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), 242, "option names are unique");
        for seed in EXCLUDED {
            assert!(
                SEED_FIELDS.iter().any(|(name, _)| *name == seed),
                "{seed} is a seed"
            );
            assert!(
                !lines
                    .iter()
                    .any(|line| line.contains(&format!(" {} ", option_name(seed))))
            );
        }
    }

    #[test]
    fn names_and_ranges_follow_the_registered_rule() {
        assert_eq!(option_name("rfp_square_16"), "KRfpSquare16");
        assert_eq!(option_name("pick_cont[2]"), "KPickCont2");
        assert_eq!(range(7_365), (3_682, 14_730));
        assert_eq!(range(-241), (-482, -120));
        assert_eq!(range(5), (1, 9));
        assert_eq!(range(2), (1, 6));
        assert_eq!(range(-3), (-7, -1));
        let lines = uci_option_strings();
        assert!(lines.contains(&String::from(
            "option name KRfpSquare16 type spin default 8218 min 4109 max 16436"
        )));
        assert!(lines.contains(&String::from(
            "option name KProbcutSeeScale type spin default 2536 min 512 max 4096"
        )));
        let mut seeds = Seeds::FITTED;
        assert!(set_uci_option(&mut seeds, "KProbcutSeeScale", "1024"));
        assert_eq!(
            seeds.probcut_see_scale, 1_024,
            "the unconverted bar is reachable"
        );
    }

    #[test]
    fn a_set_option_reaches_its_seed_clamped_and_nothing_else() {
        let mut seeds = Seeds::DONOR;
        assert!(set_uci_option(&mut seeds, "krfpsquare16", "8000"));
        assert_eq!(seeds.rfp_square_16, 8_000);
        assert!(set_uci_option(&mut seeds, "KRfpSquare16", "1"));
        assert_eq!(seeds.rfp_square_16, 4_109, "clamped to the range");
        assert!(set_uci_option(&mut seeds, "KPickCont2", "1200"));
        assert_eq!(seeds.pick_cont[2], 1_200);
        assert!(!set_uci_option(&mut seeds, "KLmrSpreadThread", "1"));
        assert!(!set_uci_option(&mut seeds, "Hash", "16"));
        let mut expected = Seeds::DONOR;
        expected.rfp_square_16 = 4_109;
        expected.pick_cont[2] = 1_200;
        assert_eq!(seeds, expected);
    }
}
