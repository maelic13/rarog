# Measuring

A module of `AGENTS.md`, as binding as the core. Read it in full when any
line below applies; when unsure, read it.

- Scope: producing or reading a number: the binary, the host, the harness, the counters.
- Kind: performance qualification; any step whose result is a measured number.
- Paths: `tools/`, `analysis/`, `docs/experiments/`, `docs/EXPERIMENTS.md`.
- Before: running, or preparing for the maintainer, a bench, NPS read, profile, match, gauntlet, SPRT, tune or datagen; building a binary that will be measured; summing, differencing or quoting a counter, timing or size.

## What to measure

- Get the information a use needs and no more. Before registering or
  running a measurement, name what it is for: a decision, an analysis or
  understanding a question. A read that feeds nothing is prepared or
  left out, not run, and runs when a use is named; a decision is never
  taken without the information it needs either (maintainer decision
  2026-11-14, RAR-O09).
- Define the smallest sufficient measurement before launching it, inside the
  leaf's registered scope; never shrink a registered run after seeing results.
  Reuse the harness's runner, parser and archive format.

## Measurement

- Colosseum CLI is the main harness for gates, fixed matches, tunes, null pairs,
  gauntlets and datagen; `tools/colosseum.ps1` and `tools/datagen.ps1` drive it
  and carry every guard. fastchess and weather-factory (`tools/sprt.ps1`,
  `tools/spsa.ps1`, `tools/gauntlet.ps1`, `datagen.ps1 -Runner fastchess`)
  stay installed and working as backup only, the second opinion. They retire
  only on a maintainer decision recorded here (kept on review 2026-10-04 and
  again 2026-10-14). PROCESS's *Harness* section names the cross-check
  triggers.
- Never measure a `--all-features` binary: it enables `texel`, which bypasses
  the eval and pawn caches. If a number you are not changing changes, check
  the binary.
- Measure only on an idle host: check CPU use and running engine or harness
  processes first, and if the machine is busy stop and ask rather than measure
  (RAR-M48's first pool was discarded for this). Keep builds, profiling and
  unrelated load off a match host while it plays. `colosseum.ps1` refuses a
  busy host; a waiver is recorded in the run's manifest, so it cannot be
  forgotten.
- Rebuild before measuring, with the exact feature set: `cargo test`,
  `clippy` and `bench` leave their own `target/release/rarog.exe`. For a
  multi-run study, build once, verify the fingerprint, hash and archive that
  executable, and measure the copy; rebuild only when source, features,
  toolchain or build settings change. Keep test builds separate.
- A `diag` build's `bench` dumps counters once per position (40 lines per
  name for `bench 13`); sum them with `tools/diag/bench_counters.py`, never a
  hand-rolled parser. The search's rule counts are the kernel's rate probes
  (`--features probes`), read by `tools/oracle-hybrids/tree_stats.py`;
  `tools/diag/phase4_differential.py` reads node counters that only builds
  before the compact kernel emit.
- Before differencing two counters, confirm they are in the same unit (per
  node versus per move produced RAR-S25). A passing invariant does not prove
  comparability.
- A counter from another engine or another tree (nodes, hits per thousand
  nodes, rule rates) is at most a prior for a prediction, never evidence
  about Rarog: the evidence is Rarog's own before and after. A timing or
  size that was not measured is marked unmeasured, never stated as a
  number.
- A binary entered in a rated pool is a tagged release or a development
  build of a committed tree, whose `id name` names its commit
  (`dev-20261104-209b555a`; a dirty tree's `-m` is refused for testing); a
  feature arm is named in its file and sidecar, and its ledger row names the
  fingerprint. The harness binary is pinned the same
  way: `tools/colosseum/colosseum.pin.json` names its revision and SHA-256, and
  a runner that is not the pinned one is refused, not substituted.
- The current fingerprint is declared once, in GUIDE's checkpoint;
  `check_guide.py` fails when a rule file's "currently" value or PLAN's
  checkpoint row disagrees with it.
- Prove a harness wire is live before trusting a null from it: set an absurd
  value and require the numbers to move. Proving the engine responds is not
  proving the instrument reports it.
