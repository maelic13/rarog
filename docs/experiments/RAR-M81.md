# RAR-M81 — Four-thread scaling measured again: RAR-M78's threads legs for Rarog and Reckless on one night, with a passive host sampler and a per-leg validity check — REGISTERED 2026-11-02, before any game

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-02 by the agent after the D.2 audit
  (`analysis/d2_audit_2026-11-02.md`); the maintainer runs it (one night,
  about 6.5 hours after [RAR-M80](RAR-M80.md)'s 80 minutes in the same
  command).
- **Leaf:** PLAN D.2, the four-thread card's decision read.
- **Question.** [RAR-M78](RAR-M78.md) read Rarog deficient at four threads
  (effective speedup 1.65 against Reckless 2.30 and Stockfish 2.48), but its
  Rarog threads leg ran in a host state where the four-thread side kept 2.48
  times one thread's speed against 3.44 in the same configuration a day
  later ([RAR-M79](RAR-M79.md)), so the verdict is not established. Measured
  on a healthy host, is Rarog's four-thread gain, in its own doublings of
  time, below Reckless's and Stockfish's?
- **Kind:** fixed-length matches with intervals; they decide no gate and no
  default. Their outcome decides whether D.2 has a four-thread target.

## Design

- **Legs**, each a 1,000-game `match` of the engine against itself, one
  thread (arm A) against four (arm B), `10+0.1`, Hash 256 on both sides,
  `match-fixed-ltc.toml` (the UHO book, random order, no adjudication),
  three games at a time, every slot four physical cores
  (`--cores-per-game 4`), Colosseum CLI 0.5.0 (pinned), exactly RAR-M78's
  threads legs but for the runner's version, the seeds and Rarog's binary:

  | Leg | Engine | Binary | Seeds |
  |---|---|---|---|
  | `rarog-threads` | the d2 head with `e057d84` | `tools/test_engines/rarog-d2dyn-pext-pgo.exe` (`a545b6b8…`, the binary of RAR-M79, RAR-M80 and RAR-R31) | 101, 102, 103 |
  | `reckless-threads` | Reckless 0.9.0, `MoveOverhead` 10 | `D:/chess/engines/reckless-windows-avx2.exe` (`b74ead56…`) | 101, 102, 103 |

  Rarog's leg plays first. RAR-M78 played `ebbbc917…`, the head before
  `e057d84`, which [RAR-P52](RAR-P52.md) read 2.4% slower at four threads;
  the head is what ships, so it is what is measured, and the 2.4% is noted
  against the prediction.
- **The odds legs are reused.** RAR-M78's `20+0.2` against `10+0.1` legs ran
  on one thread, which the host state never touched (one-thread NPS 2.79 M
  there against 2.74 M in RAR-M79), so G2 is RAR-M78's: Rarog +101.8 ± 11.5,
  Reckless +86.2 ± 9.8. Stockfish's threads leg (+114.1 ± 9.3, R 3.47) is
  reused as RAR-M78 recorded it, with its host-load note.
- **The host sampler,** passive, started by the driver before the first game
  and stopped after the last: `typeperf` at one-second intervals over
  `\Processor Information(*)\% Processor Time`, `Parking Status`,
  `% of Maximum Frequency` and `Processor Frequency`, and
  `\Thread(rarog*)\% Processor Time` and `\Thread(reckless*)\% Processor
  Time`, to a CSV beside the run's manifests. Its smoke proves the wire: a
  short four-thread search moves the slot's utilisation and thread columns.
  It changes nothing the engines see.
- **Driver:** `analysis/artifacts/d2-night/run.ps1` (prepared by the next
  session; its hash, the sampler script's hash, the self-test and the dry run
  are appended under *Preparation* before any game). It runs RAR-M80's
  `run.ps1` first, then these two legs, each after the harness's idle wait;
  a finished leg is skipped on a rerun and an interrupted one resumes at its
  seed; it ends with `ALL DONE`.

## Readings and judgement

- **Validity, per leg, read before any Elo:** the per-side in-game NPS by
  RAR-M79's rule (the sum of each move's `n` over the sum of its `t − h`)
  and **R = NPS(4T) / NPS(1T)**, over the whole leg and over each fifth of
  its games in order. The leg is **SLOW STATE** when any fifth reads R
  under 3.0 for Rarog or under 3.2 for Reckless (RAR-M78's valid legs read
  3.46 and 3.47; RAR-M79's healthy legs 3.44 to 3.53). A SLOW STATE leg's
  Elo is recorded and not compared; its fifths' windows in the sampler's
  CSV are read for the mechanism (threads on sibling logical CPUs, parked
  cores, frequency), and the leg is played again on another night.
- **Elo:** G4 = arm B over arm A per leg, D = G4 / G2, X = 2^D, the delta
  method, as RAR-M78.
- **Outcome**, on valid legs only:
  - **DEFICIENT:** Rarog's D below both Reckless's and Stockfish's, both
    differences' 95% upper bounds below zero (RAR-M78's rule);
  - **NOT DEFICIENT:** both upper bounds above zero;
  - **UNRESOLVED:** one above, one below, or a leg SLOW STATE.
- **Also read, not judged:** the per-side depth over non-mate moves
  (`s=` not a mate), never the runner's all-moves mean (the audit's trap);
  the four-thread side's holds; Reckless's G4 against RAR-M78's +103.7
  (a replication read).

## Exposure before registration

- RAR-M78's six legs and RAR-M79's four, read in the audit.
- No game of this design has been played; no dry run yet.

## Prediction, frozen before any game

| Outcome | Probability |
|---|---|
| NOT DEFICIENT | 0.60 |
| UNRESOLVED | 0.25 |
| DEFICIENT | 0.15 |

- **Levels.** Rarog G4 +122 (80% interval +105 to +138), D 1.20 (1.03 to
  1.36), X 2.3; Reckless G4 +104 (+90 to +118), D 1.20, X 2.3. Rarog's leg
  R 3.4 (3.2 to 3.6), Reckless's 3.45 (3.3 to 3.6). A SLOW STATE leg
  appears: 0.30 (one of RAR-M78's six legs did, and RAR-M79's `solo`).
  Given a slow fifth, the sampler shows a mechanism (sibling packing,
  parking or frequency): 0.6.
- **Reasoning.** The confounded leg read +73.7 at R 2.48; at R 3.44 the
  four-thread side's own search is 0.47 doublings faster at the odds leg's
  101.8 Elo a doubling, which adds about +48 if the helpers' contribution is
  unchanged, and `e057d84` adds a little. That puts Rarog level with
  Reckless. DEFICIENT needs G4 at about +85 or below, which would mean the
  helpers' width is worth far less than the main thread's speed, consistent
  with [RAR-R29](RAR-R29.md) and [RAR-R33](RAR-R33.md) (the helpers buy no
  depth) if their width is worth little; the audit gives that 0.15.

## Decision and stop rule

- **NOT DEFICIENT:** D.2's four-thread target closes `NO_CHANGE`; the
  host-state finding and the validity check go to E.2's four-thread arm and
  every rated four-thread read; scaling beyond four threads stays with G.1.
- **DEFICIENT:** the card's first engine question is RAR-R33's next read,
  the depth and bound of the main thread's hits against what its node
  needs, with helper depth staggering and the shared histories (Basilisk
  cross-check H27) as the competing mechanisms; one candidate for the leaf's
  4T SPRT `[0,5]`.
- **UNRESOLVED:** a SLOW STATE leg is played again with the sampler's
  mechanism read in hand; a split verdict adds Stockfish's threads leg on a
  valid night before anything else.
- Two legs, played to completion; an interrupted leg resumes at its seed; a
  leg abandoned for an engine fault is reported, not replaced. No leg's
  clock, threads, Hash, book, seeds or game count changes after any game,
  and the validity thresholds do not move after a reading.
- **Time losses:** a leg with over 0.5% losses on time is reported as such
  and not judged.

## How to run

From PowerShell 7 on an idle host, overnight, after RAR-M80 in the same
command (about 8 hours in all):

```powershell
pwsh -File D:\code\rarog\analysis\artifacts\d2-night\run.ps1
```

It waits for an idle host before each step, resumes after an interruption
with the same command, and ends with `ALL DONE`.
