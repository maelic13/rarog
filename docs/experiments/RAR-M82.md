# RAR-M82 — The sibling-free slot: RAR-M81's threads legs with one logical CPU per physical core — REGISTERED 2026-11-03, before any game

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

- **Result:** appended after the read.
