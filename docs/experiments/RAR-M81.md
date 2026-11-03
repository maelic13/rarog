# RAR-M81 — Four-thread scaling measured again: RAR-M78's threads legs for Rarog and Reckless on one night, with a passive host sampler and a per-leg validity check — REGISTERED 2026-11-02, before any game; READ 2026-11-03: UNRESOLVED, Rarog's leg in its own slow state

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

## Preparation, 2026-11-02, before any game

- **Scripts** (`analysis/artifacts/d2-night/`, SHA-256):

  | Script | SHA-256 | Role |
  |---|---|---|
  | `run.ps1` | `bd8ac1f9…` | the night driver |
  | `leg.ps1` | `388c89dc…` | one leg through `tools/colosseum.ps1` |
  | `sampler.ps1` | `03b6e9d3…` | the host sampler |
  | `m81_read.py` | `1f40d6aa…` | the reading |
  | `sampler_smoke.ps1`, `sampler_smoke.py` | `0a6af48f…`, `bc8e9e68…` | the sampler's smoke |

  `run.ps1` checks both engines, RAR-M80's three scripts and the three
  above against their hashes before anything runs, then starts the sampler
  and waits for its first CSV row (a dead wire stops the night before any
  game), dry-runs this card's legs, and runs RAR-M80's `run.ps1`, the
  Rarog leg, the Reckless leg and the reading, each after the harness's idle
  wait, its output appended to `<step>.log` beside it. A step that exits 0
  (a leg only with its `result.json`) leaves `<step>.done` and is skipped on
  a rerun; a half-played leg resumes at its seed.
- **Seeds.** The Design table lists "101, 102, 103" on both rows, the form of
  RAR-M78's column, which gave one seed per engine (Rarog 61, Reckless 62,
  Stockfish 63). Read the same way: Rarog's leg plays seed 101, Reckless's
  102, and 103 is Stockfish's if its leg is played again (the split-verdict
  branch). A Colosseum match takes one seed; the legs play different
  openings, as RAR-M78's did.
- **Outputs.** Run directories `tools/results/night-rarog-threads` and
  `night-reckless-threads`; the sampler's CSVs and `sampler.log` in
  `tools/results/d2-night-sampler-<stamp>/`, beside the manifests.
- **The sampler as built.** `typeperf` expands an instance wildcard once, when
  it starts, and drops a pattern with no instance without a word: started
  before the engine, `\Thread(rarog*)` yields no column. So the four
  processor counters run in one collector for the night (`cpu-NNN.csv`), and
  the thread counters (`% Processor Time`, with `ID Process` and `ID Thread`
  to tell the six same-named engine processes apart) in a collector restarted
  into a new segment (`threads-NNN.csv`) whenever the engines' thread set
  changes, checked every 2 s. Colosseum keeps engine processes per slot
  (`engine_processes: per-slot`), so that happens at a leg's start and end.
  `sampler.log` times every change. The sampler stops on a stop file, or by
  itself when the driver is gone.
- **Sampler smoke** (`sampler_smoke.log`,
  `tools/results/d2-night-smoke-20261102_222448/`): every counter resolves,
  the processor ones over 34 instances (32 logical CPUs and two totals), the
  thread ones over both engines' threads (8 OS threads for Rarog at
  Threads 4, 9 for Reckless). A four-thread search pinned to slot 0's CPUs
  2–9, as Colosseum places it, moves the slot from 6.9% to 54–55% and the
  process's thread columns to 399%, for both engines, at 4,000 and 2,000 ms;
  the thread set does not change between searches. Cost with both
  collectors running: 1.52 CPU seconds in 60.1 s, 0.079% of the machine.
  Under it the harness's idle check sees no busy process and a host 1.66%
  busy. Three earlier smoke runs failed on the smoke's own faults (two
  script bugs, then a one-second sample straddling a 2-second search's end,
  34.9% against 35%); the check now counts only samples whose whole second
  lies inside the search, its threshold unchanged.
- **Self-test** (`selftest.log`): the idle wait; live output; a finished step
  skipped on a rerun; a failing step (exit 3) stops the sequence and leaves
  no marker; a leg that exits 0 without `result.json` is not marked done;
  the sampler starts, writes, stops and leaves no `typeperf` behind. An
  uncaught failure exits 1.
- **Dry run** (`dryrun.log`, exit 0): hashes; the sampler live and stopped;
  RAR-M80's four legs through its own `run.ps1 -DryRun`; this card's two
  legs. Their resolved configurations, read field by field from the dry-run
  JSON: 1,000 games, seeds 101 and 102, three at a time, four cores a slot
  (CPUs 2–9, 16–23, 24–31), `10+0.1` with margin 20 on both sides, Hash 256,
  Threads 1 and 4, Reckless `MoveOverhead` 10 on both sides, the UHO book in
  random order, no adjudication; identical to RAR-M78's Reckless leg but for
  the seed. Runner `colosseum-cli 0.5.0` (`8fa63cc6…`, executable
  `81EA6CBA…`).
- **Reading self-test** (`m81_read_selftest.log`, exit 0). Games are read in
  completion order from `games.jsonl`, each game's text checked against its
  recorded SHA-256. On RAR-M78: R 2.48, 3.46, 3.47; D 0.72 [0.58, 0.86], 1.20
  [1.03, 1.38], 1.31; without the validity rule DEFICIENT with upper bounds
  −0.26 and −0.36, as recorded; under this card's rule UNRESOLVED (Rarog's
  leg SLOW STATE). Non-mate depth equals the runner's own non-mate figure to
  the move for Rarog and Reckless (Rarog 16.87 over 60,037 and 15.84 over
  59,438); on Stockfish's resumed leg the 0.4.0 runner counts fewer
  mate-score moves than the PGN holds, cause unread, and Stockfish's depth is
  not read here. On RAR-M79: R and move counts equal `harness_read.log`'s
  for all four legs. The judgement reaches every outcome. Printed outcome
  lines: `DEFICIENT`, `NOT DEFICIENT`, `UNRESOLVED (split)`,
  `UNRESOLVED (SLOW STATE: <legs>)`, and `NOT JUDGED (<legs> over 0.5% time
  losses)` for the time-loss rule, which the card states as "reported as
  such and not judged".
- **Exposure found in preparation; nothing registered changes.** Read by
  fifths, RAR-M79's legs, played in one sequence from 16:51 to 18:02 on
  2026-11-02, move between two speeds inside a leg: `on` 3.94, 3.50, 4.04,
  3.06, 2.72; `off` 2.84, 2.82, 4.05, 4.03, 4.09; `b9` 4.09, 3.92, 2.78,
  2.72, 2.86; `solo` 2.58 to 2.81 throughout (20 games a fifth, `solo` 8).
  RAR-M78's legs are flat over hours (Rarog 2.47 to 2.49, Reckless 3.44 to
  3.47). The healthy state reads near 4.0 and the slow one near 2.8, in
  episodes from minutes to hours, and this card's rule marks every RAR-M79
  leg SLOW STATE, where the registration took their whole-leg 3.44 to 3.53
  as healthy. The thresholds and the frozen prediction (a SLOW STATE leg
  0.30) stand; this is recorded before any game so that the calibration
  reads it. On this evidence UNRESOLVED is likelier than its frozen 0.25,
  and the reading prints each fifth's clock window so that a slow fifth's
  mechanism is read from the sampler's CSVs.
- **Expected runtime:** RAR-M80 about 80 minutes; RAR-M78's Rarog and
  Reckless threads legs took 2 h 41 min and 3 h 02 min; with the idle waits
  and dry runs about 7 to 7.5 hours.

## Result — 2026-11-03, appended after exposure

Run by the maintainer with `analysis/artifacts/d2-night/run.ps1` from 23:09
on 2026-11-02 to 06:15 on 2026-11-03, ending `ALL DONE`. Both legs
completed with no fault and no time loss; the host was 0.1% and 0.2% busy at
their starts, and the logical CPUs outside the three slots averaged 1.3% and
1.2% across them (`foreign_load.py` `2798dc36…`, `foreign_load.log`), so
nothing else ran. Colosseum's manifests:

- `rarog-threads`: 1,000 games, pentanomial 24, 224, 217, 35, 0, arm A
  −83.9 ± 11.2 Elo, `games.pgn` `54B75420…`;
- `reckless-threads`: 1,000 games, 3, 318, 163, 16, 0, −110.6 ± 9.4,
  `46C56905…`.

The reading (`m81_read.py`, `m81_read.log` `e695a3c7…`):

| Leg | R, whole leg | R by fifth, in order | Validity |
|---|---:|---|---|
| Rarog | 2.65 | 2.64, 2.64, 2.63, 2.66, 2.66 | SLOW STATE |
| Reckless | 3.46 | 3.44, 3.46, 3.47, 3.44, 3.48 | valid |

| Engine | G2 (Elo) | G4 (Elo) | D, 95% | X |
|---|---|---|---|---|
| Rarog | +101.8 ± 11.5 | +83.9 ± 11.2 | 0.82 [0.68, 0.97] | 1.77 |
| Reckless | +86.2 ± 9.8 | +110.6 ± 9.4 | 1.28 [1.10, 1.47] | 2.43 |
| Stockfish (RAR-M78's leg) | +86.9 ± 9.9 | +114.1 ± 9.3 | 1.31 [1.13, 1.50] | 2.48 |

Rarog's D minus Reckless's is −0.46 ± 0.23 (upper bound −0.23), minus
Stockfish's −0.49 ± 0.23 (−0.26). **Outcome: UNRESOLVED (SLOW STATE:
`rarog-threads`).** Replication: Reckless's G4 +110.6 against RAR-M78's
+103.7, +6.9 ± 13.2. Also read: non-mate depth, Rarog 16.81 at one thread
and 16.07 at four (−0.74), Reckless 17.44 and 18.19 (+0.75); the
four-thread side's moves held over the margin, Rarog 17,849 of 64,234,
Reckless 260 of 72,025.

**The sampler** (`tools/results/d2-night-sampler-20261102_230936/`; read by
`sampler_read.py` `97b57387…` and `thread_read.py` `3b212f67…`, logs beside
them):

- No logical CPU of any slot was parked (mean Parking Status 0.000), and `%
  of Maximum Frequency` read 100 throughout, in both legs and RAR-M80's.
- The four-thread processes' summed thread CPU, over their busy one-second
  samples: median 225–227% and p90 317–326% for both engines (a one-second
  sample mixes the two sides' turns). Rarog's threads are scheduled as much
  as Reckless's; each produces about a third less than Rarog's single
  thread (7.37 M nodes a second over four against 2.78 M, where Reckless's
  four make 0.86 of its one each).
- Load on both SMT siblings of a core, `2·Σ min / Σ` over busy samples by
  slot: Rarog 0.82, 0.76, 0.75; Reckless 0.67, 0.70, 0.59. Samples with a
  core's two siblings both above 60%: Rarog 57.7%, 29.4%, 30.0%; Reckless
  45.1%, 25.0%, 19.4%.
- The thread collector restarted 81 times as idle threads came and went,
  leaving short gaps; the CPU collector ran unbroken (25,333 samples).

## Disposition

- **The slow state is Rarog's, not the host's.** In the same hours, on the
  same quiet host and in the same slots, Reckless's four threads kept 3.44 to
  3.48 times one thread's speed in every fifth, Rarog's 2.63 to 2.66, flat
  for 2 h 40 and in all four of RAR-M80's legs before (9.5 hours in all;
  RAR-M78's leg read 2.48, flat, beside Reckless's 3.46). The audit's
  host-state reading, on which this registration's question was framed, is
  corrected here and in PLAN D.2.
- **In that state four threads are worth 0.82 doublings of Rarog's time**,
  against Reckless's 1.28 and Stockfish's 1.31: deficient by RAR-M78's Elo
  rule, both upper bounds below zero. The registered rule does not judge a
  slow leg, so the outcome is UNRESOLVED; G4 follows R as predicted (RAR-M78's
  +73.7 at R 2.48 to +83.9 at 2.65: 0.066 doublings at 101.8 Elo a doubling
  and `e057d84`'s 2.4% give about +10), so the deficit is the slow state's.
- **The mechanism** is not core parking, frequency or scheduling: Rarog's
  threads run as long as Reckless's and each does a third less work than
  Rarog's own single thread. Rarog's load sits more often on both siblings
  of a core; one-second samples cannot say whether that is the cause or a
  symptom.
- **Decision for the maintainer.** The registered UNRESOLVED branch plays the
  slow leg again on another night. I recommend against that now: two
  1,000-game legs and four short ones over two nights read Rarog slow
  throughout, so a replay most likely repeats 2.6 and costs a night. The
  cheaper discriminating read is zero-game, about an hour on an idle host:
  the slot rebuilt outside the harness (an idle one-thread process and a
  four-thread one confined to the same eight logical CPUs, three slots at
  once), the four-thread search's NPS with its process confined to one
  logical CPU per physical core against both siblings, Reckless the control.
  One per core restoring R near 3.8 (RAR-R31's alone) names Windows'
  placement of Rarog's threads; no lift points at per-thread throughput under
  co-location (memory, caches, the shared table). It needs the host free of
  Basilisk's tune.

## Calibration

- NOT DEFICIENT was predicted 0.60 and UNRESOLVED 0.25; UNRESOLVED read. A
  miss in mechanism: the prediction took the slow state for the host's and
  expected a healthy night; it is Rarog's own and lasted all night.
- Levels: Rarog's G4 +122 (80% +105 to +138) read +83.9, below; its D 1.20
  (1.03 to 1.36) read 0.82, below; its R 3.4 (3.2 to 3.6) read 2.65, below.
  Reckless's G4 +104 (+90 to +118) read +110.6, D 1.20 read 1.28, R 3.45
  (3.3 to 3.6) read 3.46: inside. Rarog's misses all follow from R; the Elo
  for an R held.
- A SLOW STATE leg (0.30) appeared. Given one, the sampler naming a mechanism
  (0.6): in part; it excludes parking, frequency and scheduling and does not
  show the remaining candidate.

## Superseded 2026-11-03

- "The slow state is Rarog's, not the host's" is corrected: it was the
  scheduler's packing of each four-thread side inside the slot's
  eight-logical-CPU mask, Rarog's harder than Reckless's (the audit, section
  10; RAR-P54 for the per-sibling loss, the same for both engines). RAR-M82,
  the two legs again with one logical CPU per physical core, read Rarog's
  four threads at 4.03 times one and Reckless's at 4.45, so the D of 0.82
  above was measured in the packed state and Reckless's 1.28 in a lesser one;
  neither is a clean reference. The comparison is re-registered sibling-free.
