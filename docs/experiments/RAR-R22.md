# RAR-R22 — D.1.6.3: the gate of the fitted cascade cluster, on an arm rebuilt with RAR-R21's block-1 values — REGISTERED 2026-10-24, before any game

Indexed under *4. Root search, time management and SMP* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-24 before any game; the gate and the
  `10+0.1` read are the maintainer's commands, the probe and the 200-game
  wire check the agent's.
- **Kind:** PLAN D.1.6.3's gate, in the form RAR-R20 registered and
  RAR-R21's Stage 3 deferred to a rebuilt arm. RAR-R20's own gate on the
  unfitted seeds (`6fd195e`) was never run; this supersedes it.
- **Hypothesis:** the cascade cluster (RAR-R19's device: a known-win offset
  for bare-king endings, a score-scaled first aspiration window, deeper
  reduced searches charged to the line budget, a deepening freeze past part
  of the soft target), with its constants fitted by RAR-R21's block 1 and
  the clock's scale refitted with it, is not weaker than the 2.6.0 engine at
  `3+0.03`, and is stronger at `10+0.1` where the stalls and the won
  endings it repairs are worth more.
- **Baseline:** the 2.6.0 engine, `rarog-d15head-pext-pgo.exe` (`468ed7b`,
  bench 13,987,223), as in RAR-R20.
- **Candidate:** the throwaway branch `d16-gate2` at **`584c02d`** on `d1`
  (`f12661d`; engine source identical to `2f17467`): the fifteen tuned
  coordinates at their rounded block-1 values and `CoreLmrDeeperBudget` 1
  as the search defaults, version `2.7.0-dev+d16gate2`. Recipe
  `analysis/arm_patches/584c02d-rar-r22-d16gate2.patch` (SHA-256
  `391c3010…570850`, 30 changed lines), baked by `bake_d16gate2.py` in the
  artifact directory. Binary `tools/test_engines/rarog-d16gate2-pext-pgo.exe`,
  PGO, SHA-256 `F6547014EA1C271F…7359C3E9`, **bench 9,013,917**, sidecar
  from a clean tree. The values:

  | Coordinate | 2.6.0 | Arm |
  |---|---|---|
  | AspirationDelta | 12 | 12 |
  | AspGrowthPct / AspGrowthHighPct / AspGrowthAdd | 164 / 165 / 8 | 178 / 168 / 10 |
  | AspDeltaSquareDiv | off (0) | 14707 |
  | MopupKnownWin | off (0) | 5826 |
  | TmDepthFreeze | off | 5781 (×0.0001 of the soft target) |
  | TmOptScale | 10000 | 10801 |
  | CoreLmrDeeperBudget | 0 | 1 |
  | CoreLmrPv / PvWindow / Correction | 399 / 306 / 3330 | 401 / 279 / 3357 |
  | CoreLmrGivesCheck / Improvement / TtPv / QuietHistory | 1030 / 333 / 179 / 316 | 840 / 282 / 155 / 347 |

- **What the gate can see and cannot.** The equal-speed unfitted cluster
  read −6.9 ± 8.8 Elo over 2,000 games (RAR-R21 Stage 1); the fit's three
  movers spend the time the freeze left on the clock and narrow the first
  window at large scores (RAR-R21 Stage 2); the arm's in-game NPS reads
  lower than the head's by composition, not speed (RAR-R20). A PGO build
  pair carries ±1.5% NPS, about ±3 Elo at this clock. The `[-5,5]` bracket
  excludes harm near −5 and accepts a cluster near even; it does not
  measure the won-ending repair itself, which the probe and the `10+0.1`
  read carry.

## The gate (frozen)

- **SPRT:** Colosseum `sprt-repair` (`[-5,5]` nElo, pentanomial,
  `model=normalized`, α = β = 0.05; AGENTS' repair case 3), `3+0.03`, 20 ms
  margin, Hash 64, one thread, the wrapper's slot count, the UHO book in
  random order, no adjudication, **cap 20,000 pairs**, seed 20261061, run
  directory `rar-r22-gate`; then `match-fixed-ltc` 1,000 games at `10+0.1`,
  seed 20261062, `rar-r22-ltc`. Both arms play with their own clocks and
  no option set. `run_r22.ps1 -Read gate|ltc|wire|probe` beside this entry;
  each dry-ran clean before registration.
- **Before the gate (agent-run, idle host):** the probe
  (`tm_rec_probe_d16.py`, rec1 and rec3 at `60000+600` and `180000+2000`,
  no clock stop) on the arm; a 200-game wire check at `3+0.03` (seed
  20261063, `rar-r22-wire`) that the arm plays whole games without fault
  and whose `search:` lines show its time use. Neither accepts anything.
- **Dispositions:** H1 lands the cluster on `d1` as the defaults at these
  values, re-declares the fingerprint (9,013,917 at the arm; the landed
  engine's own bench is recorded then), deletes `TmFailHighTarget` and its
  test, and writes the user-facing changelog entry; D.1.2.1's cascade half
  ticks on the probe. H0 refutes the cluster at the fitted values: the
  research resumes with RAR-R21's part-by-part reads (2,000 games each,
  registered then) and the fifth-part option (a fail-high stop at twice the
  target) as the candidate. The cap records "unresolved"; the `10+0.1` read
  and the probe inform the research and accept nothing.

## PRE-REGISTERED PREDICTION (frozen before exposure)

- **STC:** **+1 ± 8 Elo** at `3+0.03` (80% band −7 to +9): the −6.9
  unfitted at equal speed, plus about +4 from the time the fit spends, plus
  the window's narrowing of unknown sign, with the PGO pair's ±3. H1 0.40,
  cap 0.35, H0 0.25.
- **`10+0.1`:** the point estimate above STC's by a few Elo, P 0.65.
- **Probe:** seven of eight rows within the optimum; rec3 at `180000+2000`
  still at the maximum through one window at depth 24, as the PGO arm of
  RAR-R20 (P 0.6); the later freeze does not change that row because the
  freeze acts between iterations and the window is one iteration.
- **Wire check:** 0 faults; the arm's time a move within ±3% of the head's
  (the fit spent the freeze's saving); in-game NPS below the head's by
  2–5% (composition).
- **Most likely miss:** the STC sign, from the LMR terms the fit barely
  moved sitting at values C.10 chose for a different tree.

## Before the gate: the probe and the wire check (2026-10-24, agent-run, idle host)

**Probe, `tm_rec_probe_d16.py`, the arm alone, no clock stop
(`tm_rec_probe_r22.txt`): PASS, eight of eight rows within the optimum's
order.**

| Clock | Position | Time | × optimum | Depth | Score |
|---|---|---|---|---|---|
| `60000+600` | rec1 | 6,839 ms | 1.46 | 16 | +4797 |
| `60000+600` | rec3 | 2,678 ms | 0.61 | 23 | +819 |
| `180000+2000` | rec1 | 8,175 ms | 0.54 | 19 | +5338 |
| `180000+2000` | rec3 | 8,242 ms | 0.58 | 24 | +1095 |

The row RAR-R20's arm left at the maximum (rec3 at `180000+2000`, 97,949
ms, 255 M nodes, depth 24 as a lower bound) now ends at 0.58 × optimum with
depth 24 complete and an exact score: the fitted window (divisor 14707) and
the later freeze close the one window that cost a whole clock. **The
probe prediction missed in the good direction** (seven of eight was
predicted at 0.6; the mechanism claim that the freeze could not touch a
one-iteration window was wrong: the narrower first window at a large score
fails high less often than Stockfish's constant gave, and the iteration
ends at the optimum).

**Wire check, 200 games at `3+0.03`, seed 20261063, `rar-r22-wire`
(manifest `colosseum_match_rar-r22-wire_20261024_131546.manifest.txt`,
host 13.5% busy at launch):** 0 faults, 0 time losses; the runner's line
**−29.6 ± 26.2 Elo** (38/107/55, pentanomial 2, 32, 48, 17, 1). The arm's
time a move 67.6 ms against the head's 70.6 (−4.2%), its in-game NPS
2,336,492 against 2,461,940 (−5.1%), nodes a move −9.2%, mean completed
depth 14.34 against 15.16 (the head's cascades inflate its depth and its
node count in won positions). Calibration of the wire predictions: faults
held; NPS held (−5.1%, the composition band); time a move missed the ±3%
band at −4.2%, so the fit spent back most of the freeze's 5.3% but not all.
The Elo line decides nothing at ±26 and is recorded as read; the gate's
prediction stays as frozen above.

## Results (2026-10-24, maintainer-run, idle host)

**The gate accepted H0.** `rar-r22-gate`, manifest
`colosseum_sprt_rar-r22-gate_20261024_132142.manifest.txt`, host 3.9% busy
at launch, 0 faults, 0 time losses; the runner's closing line:

> Elo −7.4 ± 7.5 | nElo −13.0 ± 13.0 | W/D/L 650/1395/709 | Ptnml 39, 338,
> 673, 297, 30 | LLR −2.95 in [−2.94, 2.94] (accept H0) | 1,377 official
> pairs, 2,754 games.

The arm's time a move 66.4 ms against the head's 70.3 (−5.5%), in-game NPS
2,338,422 against 2,458,212 (−4.9%, composition), mean completed depth
14.23 against 15.23.

**The `10+0.1` read** (`rar-r22-ltc`, 1,000 games, seed 20261062, manifest
`colosseum_match_rar-r22-ltc_20261024_135016.manifest.txt`, host 4.7%, 0
faults; a first start at 13:21 local was interrupted after two seconds
while the gate ran and holds no game): **+2.1 ± 11.6 Elo** (233/540/227,
pentanomial 5, 119, 248, 121, 7). Time a move 221.3 ms against 232.4
(−4.8%), NPS −3.4%.

**Calibration.** STC was predicted +1 ± 8 (band −7 to +9) and read −7.4 ±
7.5: a miss in sign at the band's edge; H0 carried 0.25. The `10+0.1` point
estimate above STC's (0.65) **held**, by 9.5 Elo, consistent with the
mechanism being worth more where the won endings and the stalls weigh
more. The wire check's −29.6 ± 26.2 pointed the same way as the gate and
was rightly given no weight on its own.

## Disposition: H0, the cluster returns to research

- **What is established:** the cluster ends every probed stall (eight of
  eight rows within the optimum at the fitted values, the first arm to do
  so), plays without fault at both clocks, and reads even or better at
  `10+0.1`. The known-win offset with its window reads even at equal speed
  at every size (RAR-R20), and the fit did not move the offset or the LMR
  terms (RAR-R21).
- **What is refuted:** the cluster as a whole at the fitted values at
  `3+0.03` against the 2.6.0 engine; harm near −5 nElo is not excluded, so
  nothing lands. `TmFailHighTarget` and the unused instruments stay until a
  cluster lands.
- **Where the cost most likely is**, from the reads in hand: the arm still
  spends 5.5% less a move at STC after the fit (the freeze), and parts 1–3
  at equal time read −9.7 ± 8.9 at equal speed (RAR-R21 Stage 1), so both
  the freeze and the search half are candidates; the evaluation half is
  not (RAR-R20's decompositions). The 200-game decompositions cannot
  separate them; 2,000-game reads can.
- **The next research (RAR-R23, to register before any game):**
  1. zero games: the probe on the known-win offset with the fitted window
     alone (`MopupKnownWin` 5826, `AspDeltaSquareDiv` 14707, no budget, no
     freeze) on the tune build, to learn whether the evaluation half alone
     ends the stalls at the fitted divisor;
  2. 2,000-game equal-speed reads on one tune build, each part against the
     defaults: the offset with its window; the budget alone; the freeze
     alone at the fitted 5781 with the fitted scale; the fitted vector
     without the freeze; each with a frozen prediction;
  3. the rule, frozen then: a part within noise of even that passes the
     probe joins the next cluster; a part that costs is dropped, or for the
     freeze replaced by the fail-high stop at twice the target (RAR-R18's
     reading, the D.1.6 fallback) and read again;
  4. the gate re-registered on the surviving cluster, in this entry's form.
