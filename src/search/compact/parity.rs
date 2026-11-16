//! Layer A of the parity instrument: each of the compact kernel's rules
//! against the donor's own decisions on the same inputs.
//!
//! The donor's tracing twin writes one line per decision it takes at a
//! sample of nodes, `<kind> fire=<0|1> <name>=<int> ...`, its inputs and
//! outputs as named integers. The test reads every trace file in the
//! directory `RAROG_DONOR_TRACE` names, calls the rule for each record of
//! its kind and requires the donor's output exactly. A trace is the donor's
//! behaviour as numbers, so it stays in ignored storage and this test is a
//! local check, not CI's.
//!
//! Each plant changes one seed; it must fail the kind that reads it and no
//! other, which proves both that the kind's check reads its rule and that
//! no other kind's check is blind to a change it should not see.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::PathBuf;

use super::rules::{Seeds, WrapperInput, wrapped_eval};

/// A kind's records must come from at least this many positions, and
/// number at least this many, for its zero mismatches to count.
const MIN_POSITIONS: usize = 200;
const MIN_RECORDS: usize = 2_000;

/// The kinds the rules cover so far, each held to zero mismatches.
const COVERED: &[&str] = &["wrapper"];

/// One decision record.
struct Record<'a> {
    kind: &'a str,
    fields: Vec<(&'a str, i64)>,
}

impl Record<'_> {
    fn parse(line: &str) -> Option<Record<'_>> {
        let mut words = line.split_ascii_whitespace();
        let kind = words.next()?;
        let fields = words
            .map(|word| {
                let (name, value) = word.split_once('=')?;
                Some((name, value.parse().ok()?))
            })
            .collect::<Option<Vec<_>>>()?;
        Some(Record { kind, fields })
    }

    fn int(&self, name: &str) -> i32 {
        let value = self
            .fields
            .iter()
            .find(|(field, _)| *field == name)
            .map(|&(_, value)| value)
            .unwrap_or_else(|| panic!("a {} record has no {name}", self.kind));
        i32::try_from(value).unwrap_or_else(|_| panic!("{name}={value} is out of range"))
    }
}

/// Whether the rule reproduces the record; `None` for a kind no rule covers.
fn reproduces(s: &Seeds, r: &Record<'_>) -> Option<bool> {
    let matched = match r.kind {
        "wrapper" => {
            let input = WrapperInput {
                raw: r.int("raw"),
                material: r.int("material"),
                optimism: r.int("optimism"),
                fifty: r.int("fifty"),
                correction: r.int("correction"),
            };
            wrapped_eval(s, &input) == r.int("eval")
        }
        _ => return None,
    };
    Some(matched)
}

/// One planted seed: the kind it must fail and the change.
struct Plant {
    kind: &'static str,
    change: fn(&mut Seeds),
}

const PLANTS: &[Plant] = &[Plant {
    kind: "wrapper",
    change: |s| s.wrap_raw_base += 512,
}];

#[derive(Default)]
struct Tally {
    records: usize,
    positions: usize,
    mismatches: usize,
    /// The first few records the rule did not reproduce.
    examples: Vec<String>,
}

#[test]
#[ignore = "reads the donor's trace from ignored storage, named by RAROG_DONOR_TRACE"]
fn the_rules_reproduce_the_donors_decisions() {
    let dir = PathBuf::from(
        std::env::var_os("RAROG_DONOR_TRACE").expect("RAROG_DONOR_TRACE names the trace"),
    );
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("the trace directory is readable")
        .map(|entry| entry.expect("a readable entry").path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("trace_") && name.ends_with(".txt"))
        })
        .collect();
    files.sort();
    assert!(
        !files.is_empty(),
        "no trace_*.txt files in {}",
        dir.display()
    );

    let base = Seeds::DONOR;
    let planted: Vec<Seeds> = PLANTS
        .iter()
        .map(|plant| {
            let mut seeds = Seeds::DONOR;
            (plant.change)(&mut seeds);
            assert_ne!(
                seeds,
                Seeds::DONOR,
                "the {} plant changes a seed",
                plant.kind
            );
            seeds
        })
        .collect();
    let mut tallies: BTreeMap<String, Tally> = BTreeMap::new();
    // Mismatches under each plant, by kind.
    let mut plant_tallies: Vec<BTreeMap<String, usize>> = vec![BTreeMap::new(); PLANTS.len()];
    let mut unparsed = 0usize;

    for file in &files {
        let text = std::fs::read_to_string(file).expect("a readable trace file");
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        for line in text.lines().filter(|line| !line.is_empty()) {
            let Some(record) = Record::parse(line) else {
                unparsed += 1;
                continue;
            };
            let Some(matched) = reproduces(&base, &record) else {
                continue;
            };
            let tally = tallies.entry(record.kind.to_string()).or_default();
            tally.records += 1;
            if seen.insert(record.kind) {
                tally.positions += 1;
            }
            if !matched {
                tally.mismatches += 1;
                if tally.examples.len() < 5 {
                    tally.examples.push(line.to_string());
                }
            }
            for (seeds, counts) in planted.iter().zip(plant_tallies.iter_mut()) {
                if reproduces(seeds, &record) == Some(false) {
                    *counts.entry(record.kind.to_string()).or_default() += 1;
                }
            }
        }
    }

    let mut report = String::new();
    let _ = writeln!(report, "{} trace files from {}", files.len(), dir.display());
    for (kind, tally) in &tallies {
        let _ = writeln!(
            report,
            "{kind:16} records {:>8} positions {:>4} mismatches {}",
            tally.records, tally.positions, tally.mismatches
        );
        for example in &tally.examples {
            let _ = writeln!(report, "    {example}");
        }
    }
    for (plant, counts) in PLANTS.iter().zip(&plant_tallies) {
        let _ = writeln!(
            report,
            "plant {}: mismatches by kind {counts:?}",
            plant.kind
        );
    }
    println!("{report}");

    assert_eq!(unparsed, 0, "every trace line parses\n{report}");
    for kind in COVERED {
        let tally = tallies
            .get(*kind)
            .unwrap_or_else(|| panic!("no {kind} records\n{report}"));
        assert!(
            tally.records >= MIN_RECORDS && tally.positions >= MIN_POSITIONS,
            "{kind}: {} records from {} positions is below the floor\n{report}",
            tally.records,
            tally.positions
        );
        assert_eq!(tally.mismatches, 0, "{kind} mismatches the donor\n{report}");
    }
    assert_eq!(
        tallies.len(),
        COVERED.len(),
        "a rule covers a kind the list does not name\n{report}"
    );
    for (plant, counts) in PLANTS.iter().zip(&plant_tallies) {
        assert!(
            counts.get(plant.kind).is_some_and(|&n| n > 0),
            "the {} plant fails nothing\n{report}",
            plant.kind
        );
        assert_eq!(
            counts.len(),
            1,
            "the {} plant fails another kind too\n{report}",
            plant.kind
        );
    }
}
