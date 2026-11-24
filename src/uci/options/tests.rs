use super::*;

/// The table advertises the engine's option list, line for line.
#[test]
fn the_table_advertises_the_engine_s_option_list() {
    let hash = format!(
        "option name Hash type spin default 64 min 1 max {}",
        hash_limit_mb(physical_memory::total_bytes())
    );
    let mut expected = vec![
        hash.as_str(),
        "option name Clear Hash type button",
        "option name Ponder type check default false",
        "option name Move Overhead type spin default 10 min 0 max 5000",
        "option name Threads type spin default 1 min 1 max 1024",
        "option name MultiPV type spin default 1 min 1 max 256",
        "option name UCI_ShowWDL type check default false",
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

/// The limit is the machine's memory in whole MiB, held to the protocol's
/// largest, and the protocol's largest where the memory is unknown.
#[test]
fn the_hash_limit_is_the_machine_s_memory() {
    assert_eq!(hash_limit_mb(Some(32 << 30)), 32_768);
    assert_eq!(hash_limit_mb(Some((32 << 30) + (1 << 19))), 32_768);
    assert_eq!(hash_limit_mb(None), MAX_HASH_MB);
    assert_eq!(hash_limit_mb(Some(u64::MAX)), MAX_HASH_MB);
    assert_eq!(hash_limit_mb(Some(1 << 10)), 1);
}

/// Above the limit a `Hash` is refused with a notice that says why and the
/// setting is kept; at the limit it is taken; below 1 it is held to 1.
#[test]
fn a_hash_above_the_machine_s_memory_is_refused_not_held_to_it() {
    let range = 1..=32_768;
    let mut engine = EngineOptions::default();
    match apply_hash(&mut engine, "32769", &range) {
        Err(Refusal::TooLarge(notice)) => assert_eq!(
            notice,
            "Hash value 32769 is above the 32768 MiB this machine can hold; the setting is unchanged."
        ),
        _ => panic!("32769 MiB on a 32 GiB machine must be refused"),
    }
    assert_eq!(engine.hash_mb, 64);
    assert!(matches!(
        apply_hash(&mut engine, "99999999999999999999", &range),
        Err(Refusal::Unreadable)
    ));
    assert!(apply_hash(&mut engine, "32768", &range).is_ok());
    assert_eq!(engine.hash_mb, 32_768);
    assert!(apply_hash(&mut engine, "0", &range).is_ok());
    assert_eq!(engine.hash_mb, 1);
    assert!(matches!(
        apply_hash(&mut engine, "-5", &range),
        Err(Refusal::Unreadable)
    ));
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
        let want = if option.name == "Hash" {
            OptionUpdate::HashApplied
        } else {
            OptionUpdate::Applied
        };
        assert_eq!(apply(&mut engine, &args), want, "{}", option.name);
    }
    let default = EngineOptions::default();
    assert_eq!(engine.hash_mb, default.hash_mb);
    assert_eq!(engine.ponder, default.ponder);
    assert_eq!(engine.move_overhead, default.move_overhead);
    assert_eq!(engine.threads, default.threads);
    assert_eq!(engine.multi_pv, default.multi_pv);
    assert_eq!(engine.show_wdl, default.show_wdl);
    assert_eq!(engine.large_pages, default.large_pages);
    assert!(engine.syzygy == default.syzygy);
}

/// `UCI_ShowWDL` reaches the engine's setting, and its case does not
/// matter, as for every option.
#[test]
fn showing_the_rates_is_set_by_its_option() {
    let mut engine = EngineOptions::default();
    let set = |engine: &mut EngineOptions, value: &str| {
        let args: Vec<String> = ["name", "uci_showwdl", "value", value]
            .iter()
            .map(ToString::to_string)
            .collect();
        apply(engine, &args)
    };
    assert_eq!(set(&mut engine, "true"), OptionUpdate::Applied);
    assert!(engine.show_wdl);
    assert_eq!(set(&mut engine, "false"), OptionUpdate::Applied);
    assert!(!engine.show_wdl);
}
