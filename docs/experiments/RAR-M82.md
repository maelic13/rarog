# RAR-M82 — The sibling-free slot: RAR-M81's threads legs with one logical CPU per physical core — RESTORED 2026-11-03: Rarog's four threads 4.03 times one, Reckless's 4.45

- **Date / owner:** 2026-11-03, D.2.
- **Baseline SHA / binaries:** d2 head `38ef0d5` (the wrapper accepts an
  explicit game-core placement from this commit); Rarog
  `tools/test_engines/rarog-d2dyn-pext-pgo.exe` (`a545b6b8…`, RAR-M81's
  binary); Reckless 0.9.0 `D:/chess/engines/reckless-windows-avx2.exe`
  (`b74ead56…`); Colosseum CLI 0.5.0 (pinned).
- **Research question:** RAR-M81's slot gave each game four physical cores
  with both SMT siblings, and Windows packed each four-thread side on two or
  three of them (the audit, section 10). With the siblings taken out of the
  mask, one logical CPU per physical core, does Rarog's in-game four-thread
  ratio return to what it reads alone (3.6 to 3.8 on the game cores,
  RAR-R31), and does Reckless's?
- **Hypothesis:** the placement is the mechanism: with no sibling in the
  mask Rarog's R reads near 3.7, Reckless's near 3.5, and the night's
  "deficient" Elo was measured in a packed state.
- **Competing hypotheses:** the packing is not the whole loss (RAR-P54
  measures the per-core part), so Rarog returns only partly; or the mask is
  not honoured for one side (the instrument rule below catches it).
- **Interacting mechanisms:** the one-thread side runs on the same four
  cores, as in RAR-M81; the slot's cores are the night's (2–8, 16–22,
  24–30), each within one CCD, with 10, 12 and 14 free as headroom; the
  openings are the night's first 200 (RAR-M81's seeds), so the odds legs'
  G2 and RAR-M81's Elo are not read here.

## Design

- **Legs**, each a 200-game `match` of the engine against itself, one
  thread (arm A) against four (arm B), `10+0.1`, Hash 256 on both sides,
  `match-fixed-ltc.toml`, three games at a time, `--cores-per-game 4`, and
  `--placement 2,4,6,8,16,18,20,22,24,26,28,30`: RAR-M81's legs but for
  the game count and the mask. Rarog's leg first, seed 101; Reckless's,
  `MoveOverhead` 10, seed 102.
- **The host sampler** of RAR-M81 (`d2-night/sampler.ps1`, `03b6e9d3…`)
  rides from before the first game to after the last.
- **Driver:** `analysis/artifacts/d2-placement/run.ps1`, which checks both
  binaries and all four scripts against their recorded hashes
  (`leg.ps1` `33797c04…`, `placement_read.py` `fb2fabe3…`, the sampler and
  `m81_read.py` `1f40d6aa…`), dry-runs both legs, waits for the idle host
  before each leg, resumes a finished or half-played leg on a rerun, and
  ends with the reading and `ALL DONE`. Dry runs of both legs on
  2026-11-03 resolved to the policy with `placement: explicit
  (2,4,6,8,16,18,20,22,24,26,28,30), one logical CPU per physical core`
  and `slot_cpus` equal to that list.

## Readings and judgement

- **Instrument, per leg, read first:** over the leg's clock window in the
  sampler's CSV, on the seconds with at least three of a slot's four
  logical CPUs above 60% (a four-thread turn), the share on four distinct
  cores must be at least 90% and the share with any excluded sibling above
  60% at most 5% in every slot; otherwise the leg is **INSTRUMENT** (the
  mask did not take) and not read. (RAR-M81's Rarog leg reads 0% and
  89–100% on this measure.)
- **R** = NPS(4T) / NPS(1T) by RAR-M79's rule, whole leg and each fifth
  (`placement_read.py`, through `m81_read.read_leg`).
- **Outcome, Rarog:** **RESTORED** at R ≥ 3.5 over the leg and ≥ 3.3 in
  every fifth; **PARTIAL** at R in [3.0, 3.5); **NOT RESTORED** under 3.0.
  Reckless is the control, read the same way and not judged.
- **Also read, not judged:** the per-side depth over non-mate moves; the
  four-thread side's holds.

## Prediction, frozen before any game

| Outcome | Probability |
|---|---|
| RESTORED | 0.70 |
| PARTIAL | 0.20 |
| NOT RESTORED | 0.05 |
| INSTRUMENT | 0.05 |

- **Levels.** Rarog R 3.7 (3.5 to 3.85) over the leg, no fifth under 3.4;
  Reckless 3.5 (3.4 to 3.6). Four-core share 95% or more in every slot,
  sibling share under 2%.
- **Reasoning.** Alone on the game cores (one logical CPU per core, the
  harness's own list) Rarog reads 3.6 to 3.8 at four threads; the night's
  2.65 is the packed state and the mask forbids it. PARTIAL covers a
  one-thread side that now shares a core with a four-thread turn's
  thread more often, and whatever RAR-P54 finds beyond placement.

## Decision and stop rule

- **RESTORED:** RAR-M81's "deficient" verdict was measured in a packed
  state and is superseded. The self-relative comparison is re-registered
  in this slot (a 1,000-game Rarog threads leg, and Reckless's and
  Stockfish's where RAR-M78's cannot be reused), and the explicit game-core
  placement becomes the policy for every rated multi-thread read. An
  engine-side thread-placement hint stays a separate leaf, held until how
  CCRL's testers place a four-thread engine is known.
- **PARTIAL:** placement and per-core loss both matter; the re-registered
  leg still runs sibling-free, and the per-thread loss (RAR-P54) becomes an
  engine question on D.2's card (working set per thread).
- **NOT RESTORED:** placement refuted as the cause; D.2 returns to its
  engine card (RAR-R33's next read).
- Two legs, 200 games each, played to completion; no clock, threads, Hash,
  book, seed, mask or game count changes after any game, and the
  thresholds do not move after a reading. A leg with over 0.5% losses on
  time is reported and not judged.

## How to run

From PowerShell 7 on an idle host, about an hour and a quarter:

```powershell
pwsh -File D:\code\rarog\analysis\artifacts\d2-placement\run.ps1
```

It dry-runs both legs first, waits for an idle host before each, resumes
with the same command after an interruption, writes
`placement_read.log` beside itself and ends with `ALL DONE`.

## Preparation, 2026-11-03, before any game

- `run.ps1` (`3d6cf102…`, full `3d6cf10208a26ac28857734a6e79db8048b241d96059302160db75fb29b8c1af`) dry-run end to end: both
  binaries and all four scripts matched their recorded hashes, both legs
  resolved to the policy (`dryrun.log` beside it), nothing played.
- RAR-P54, read the same morning, refuted the per-core hypothesis: both
  engines keep 0.63 to 0.65 beside a sibling, so this read's RESTORED
  prediction stands on placement alone, as registered.

## Result — 2026-11-03, appended after the read

- **Run:** `run.ps1` (`3d6cf102…`, unchanged) from 10:08 to 11:15; host
  idle at 0% CPU before each leg and 0.3% at each launch
  (`host_idle_waived: False`); both manifests read `placement: explicit
  (2,4,6,8,16,18,20,22,24,26,28,30), one logical CPU per physical core`.
  Rarog's leg `colosseum_match_placement-rarog-threads_20261103_100819`
  (`results.txt` `76dbf57f…`), Reckless's
  `colosseum_match_placement-reckless-threads_20261103_103919`
  (`47ca8c9a…`), 200 scored games each, no faults, no time losses; the
  sampler `tools/results/d2-placement-sampler-20261103_100809/cpu-001.csv`
  (`135d554f…`, 10:08:11 to 11:15:51); the reading
  `analysis/artifacts/d2-placement/placement_read.log` (`07c343e6…`).
- **Instrument, both legs valid.** Four-thread turns on four distinct cores:
  Rarog 98.0 / 96.5 / 94.8% by slot, Reckless 97.4 / 95.1 / 94.1%; an
  excluded sibling busy 0.0% in every slot. The mask took.
- **R, Rarog: 4.03** over the leg (NPS 11.01 M at four threads against
  2.73 M at one), fifths 4.11, 4.04, 3.94, 4.03, 4.00: **RESTORED**. Four
  threads on four cores do four times one thread's work (1.007 a thread).
- **R, Reckless (control): 4.45**, fifths 4.36 to 4.53 (6.40 M against
  1.44 M), 1.11 a thread, above four as RAR-P54's two-core read (2.08 for
  two) already showed.
- **Observed, not judged.** The runner's lines, on the night's first 200
  openings: Rarog's four threads over one `Elo +127.0 +/- 23.7` (the night's
  packed leg +83.9 ± 11.2 over 1,000; RAR-M78's audit predicted +122 ± 12
  for a clean leg), Reckless's `+121.1 +/- 19.6` (the night +110.6 ± 9.4).
  Non-mate depth: Rarog 16.85 at one thread, 17.22 at four (+0.37; the
  night −0.74); Reckless 17.72 and 19.47 (+1.75; the night +0.75). The
  four-thread side's holds are what they were in the packed state: Rarog
  p50 5, p99 162, p999 531, max 1,686 ms, 3,393 of 12,148 moves over the
  margin (the night: p99 167, the same share); Reckless p99 3 ms. Placement
  does not move them; they stay the card's third read.

## Disposition

- **RESTORED.** The slow state was Colosseum's auto slot: with both SMT
  siblings in the mask Windows packed each four-thread side on two or three
  cores (the audit, section 10) and a thread beside its sibling keeps 0.63
  of its work for either engine (RAR-P54); with one logical CPU per core
  Rarog's four threads run at four times one. RAR-M81's "the slow state is
  Rarog's" and RAR-M78's "RAROG DEFICIENT" were measured in the packed
  state and are superseded as evidence of an engine deficit; RAR-M78's
  time-odds legs, one thread on both sides, stand.
- **Reckless's night was packed too:** 3.46 against 4.45 here, so neither
  its RAR-M78 nor its RAR-M81 four-thread leg is a clean reference, and
  Stockfish's RAR-M78 leg (R 3.47, the same slot) is not either.
- **Policy:** every rated multi-thread read names its game cores, one
  logical CPU per physical core (PROCESS, *Harness*); the wrapper accepts
  the list since `38ef0d5` and refuses a sibling or a list with no free
  core. An engine-side placement hint stays a separate leaf, held until how
  CCRL's testers place a four-thread engine is known.
- **Next:** the self-relative comparison re-registered in this slot: Rarog's,
  Reckless's and Stockfish's four-thread legs, 1,000 games each at
  `10+0.1`, about nine hours in all, with RAR-M78's three time-odds legs
  reused as G2. Its prediction is frozen in its own registration; the
  figures above are orientation, not a reading: at 200 games on the night's
  openings Rarog's D would be 1.25 ± 0.27 against Reckless's 1.40 ± 0.28, a
  difference of −0.16 ± 0.39 that resolves nothing.

## Calibration

- **Outcome:** RESTORED was predicted 0.70 and read; INSTRUMENT (0.05) did
  not occur, the sibling share 0.0% against under 2% predicted.
- **Levels, Rarog:** R 3.7 (3.5 to 3.85) read 4.03, above the range in the
  predicted direction, every fifth above 3.9 against no fifth under 3.4. A
  miss in magnitude: the level was set from RAR-R31's 3.78 alone, where
  RAR-M79's healthy stretches had already read about 4.0; four threads on
  four cores lose nothing to one.
- **Levels, Reckless:** 3.5 (3.4 to 3.6) read 4.45, far above. A miss in
  mechanism: the night's 3.46 was taken for Reckless's clean state and used
  as the control's level, but Reckless was packed too (two or three cores
  in most of its turns), and RAR-P54's two-core read of 2.08 for two
  threads, in hand before the games, already said its four threads exceed
  four. The control was read against the wrong baseline, not the wrong
  mechanism.
- **Instrument levels:** four-core share 95% or more in every slot predicted,
  read 94.1 to 98.0, two slots a little under; the measure counts seconds
  in which three logical CPUs are busy, so a turn's first and last second
  dilute it. Inside the rule's 90%.
- **Retry trigger:** `closed`; the comparison continues as its own
  registration.
