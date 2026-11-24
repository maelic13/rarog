use super::*;

#[test]
fn every_seed_but_the_five_is_an_option_inside_its_range() {
    let lines = uci_option_strings();
    assert_eq!(SEED_FIELDS.len(), 251);
    assert_eq!(lines.len(), 246);
    for (name, field, min, max) in tunables() {
        let mut seeds = Seeds::FITTED;
        let value = *field(&mut seeds);
        assert!(
            min < max && (min..=max).contains(&value),
            "{name}: {value} in [{min}, {max}]"
        );
        assert!(value == 0 || (min.signum() == value.signum() && max.signum() == value.signum()));
    }
    let mut names: Vec<String> = tunables().map(|(name, ..)| name.to_lowercase()).collect();
    names.sort();
    names.dedup();
    assert_eq!(names.len(), 246, "option names are unique");
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
        "option name KRfpSquare16 type spin default 8812 min 4406 max 17624"
    )));
    assert!(lines.contains(&String::from(
        "option name KProbcutSeeScale type spin default 2630 min 512 max 4096"
    )));
    let mut seeds = Seeds::FITTED;
    assert!(set_uci_option(&mut seeds, "KProbcutSeeScale", "1024"));
    assert_eq!(
        seeds.probcut_see_scale, 1_024,
        "the unconverted bar is reachable"
    );
}

#[test]
fn each_new_option_reaches_its_rule() {
    use super::super::exchange_values_of;
    use super::super::rules::{q_see_threshold, singular_margin};

    let lines = uci_option_strings();
    for line in [
        "option name KPawnHistoryMax type spin default 8401 min 4200 max 16802",
        "option name KSingularPvPerPly1024 type spin default 403 min 201 max 806",
        "option name KQSeeCorrectionScale type spin default 2559 min 1279 max 5118",
        "option name KExchangeValue1 type spin default 396 min 198 max 792",
        "option name KExchangeValue2 type spin default 447 min 223 max 894",
        "option name KExchangeValue3 type spin default 658 min 329 max 1316",
        "option name KExchangeValue4 type spin default 1213 min 606 max 2426",
    ] {
        assert!(lines.contains(&String::from(line)), "{line}");
    }
    assert!(
        !lines.iter().any(|line| line.contains(" KExchangeValue0 ")),
        "the pawn's value is the unit"
    );
    let base = Seeds::FITTED;

    let mut seeds = Seeds::FITTED;
    assert!(set_uci_option(&mut seeds, "KPawnHistoryMax", "20000"));
    assert_eq!(seeds.pawn_history_max, 16_802, "clamped inside an i16");

    let mut seeds = Seeds::FITTED;
    assert!(set_uci_option(&mut seeds, "KSingularPvPerPly1024", "600"));
    assert_eq!(
        singular_margin(&seeds, 10, false, false),
        singular_margin(&base, 10, false, false),
        "off the stored-PV case the coefficient is not read"
    );
    assert!(singular_margin(&seeds, 10, false, true) > singular_margin(&base, 10, false, true));

    let mut seeds = Seeds::FITTED;
    assert!(set_uci_option(&mut seeds, "KQSeeCorrectionScale", "1300"));
    assert!(q_see_threshold(&seeds, 0, 0, 20, 0) > q_see_threshold(&base, 0, 0, 20, 0));

    for index in 1..5 {
        let mut seeds = Seeds::FITTED;
        assert!(set_uci_option(
            &mut seeds,
            &format!("KExchangeValue{index}"),
            "700"
        ));
        let mut expected = exchange_values_of(&base).as_array();
        expected[index] = 700;
        assert_eq!(exchange_values_of(&seeds).as_array(), expected);
    }
}

#[test]
fn a_set_option_reaches_its_seed_clamped_and_nothing_else() {
    let mut seeds = Seeds::DONOR;
    assert!(set_uci_option(&mut seeds, "krfpsquare16", "8000"));
    assert_eq!(seeds.rfp_square_16, 8_000);
    assert!(set_uci_option(&mut seeds, "KRfpSquare16", "1"));
    assert_eq!(seeds.rfp_square_16, 4_406, "clamped to the range");
    assert!(set_uci_option(&mut seeds, "KPickCont2", "1200"));
    assert_eq!(seeds.pick_cont[2], 1_200);
    assert!(!set_uci_option(&mut seeds, "KLmrSpreadThread", "1"));
    assert!(!set_uci_option(&mut seeds, "Hash", "16"));
    let mut expected = Seeds::DONOR;
    expected.rfp_square_16 = 4_406;
    expected.pick_cont[2] = 1_200;
    assert_eq!(seeds, expected);
}
