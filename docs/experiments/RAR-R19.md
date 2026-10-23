# RAR-R19 — D.1.6.1: the root fail-high cascade in the search; the depth-reduced re-search as a prototype, its zero-game reads and the gate's registration — REGISTERED 2026-10-23, before any read

Indexed under *4. Root search, time management and SMP* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-23, before any read of the prototype.
  The zero-game reads, the untimed traces and the timed probe are the agent's
  (an idle host for the probe and the replay); the gate (D.1.6.3) and the
  `10+0.1` read are the maintainer's.
- **Kind:** PLAN D.1.6.1, research with a prototype behind a tune-only
  switch. Nothing here accepts a playing change; the gate does.
- **Baseline:** `d1` at `808211e`, engine inputs the 2.6.0 release's search
  and evaluation plus D.1.3's clock cluster at its seeds with the fail-high
  stop off (`a0f9755`); fingerprint **13,987,223 / EBF 2.565**, re-read on a
  fresh no-feature release build of that source on 2026-10-23.
- **Binaries:** the head `tools/test_engines/rarog-d15head-pext-pgo.exe`
  (`468ed7b`, the 2.6.0 engine, `273FEDF4…`); the prototype a `diag,tune`
  release build of the switch commit (named at *The prototype* below).

## The question

Rarog searches every root window of an iteration at the iteration's depth.
When the root fails high, the window widens on the stale centre and the
re-search costs another full iteration; a rising score fails high again on
the widened window, and the move runs into the hard maximum (RAR-R13's
rec1 and rec3). D.1.3's clock stop capped the symptom and cost about 15 Elo
at `3+0.03` (RAR-R18), which says the re-searches are worth their time.
**What ends the cascade in the search, cheaply, without discarding what the
re-searches find?**

## What the traces say (2026-10-23, the 2.6.0 engine, zero games)

RAR-R13's method on the `diag,tune` build of the head (`go infinite`, one
thread, Hash 64, a fresh process, stopped by wall budget; node counts exact,
`trace_rec_d16.py`, read by `cascade_read.py`):

| Position | Iteration | Windows | Nodes | Completed |
|---|---|---|---:|---|
| rec1 | depth 18 | 8 fail-highs (811, 826, 851, 892, 960, 1072, 1257, 1562), then exact 1845 | 17.0 M (the exact window 10.8 M) | 6.5 s |
| rec1 | depth 19 | fail-low 1833, then 7 fail-highs (1837 … 2303), each costlier than the last (3.2 → 20.3 M) | 75.3 M | not in 40 s (94.4 M nodes) |
| rec3 | depth 20 | fail-high, fail-low, 2 fail-highs, exact 831 | 4.4 M | 2.5 s |
| rec3 | depth 21 | 5 fail-highs (843, 858, 883, 924, 992) | 18.5 M | not in 20 s (26.6 M nodes) |

- **Every root fail-high returns exactly its beta** (every bounded line in
  both traces, as in RAR-R13's on the B.9 head). The root is fail-hard in
  effect, so a fail-high carries no information past "at least beta", and
  the number of re-searches to bracket a rising score is set by the window
  schedule alone: deltas 12, 27, 52, 93, 161, 273, 458, 763, 1,266, 2,097
  (C.10's `AspGrowthHighPct` 165, `AspGrowthAdd` 8), eight fail-highs for a
  rise of about 1,000 cp. Re-centring on the failing score (both donors;
  RAR-S17 −4.52) would save at most one re-search here, since the failing
  score is beta.
- **Each re-search costs more than the last:** proving a higher bound at the
  same depth searches more of the tree (rec1 depth 19: 3.2, 6.0, 8.6, 9.2,
  11.5, 14.3, 20.3 M). Fail-high windows are 85% of rec1's search nodes
  (80.2 of 94.4 M) and 82% of rec3's (21.8 of 26.6 M).
- The 2.6.0 engine's rec1 cascade sits at depth 18–19 with scores 800–2,300
  (the B.9 head's was at depth 32 around 534): the evaluation changed the
  position's scores and depths, and the mechanism reproduces.
- In games (RAR-R14's profile of the 2.6.0 engine), won and mate moves at
  ≥ 0.9 × the maximum are 0.3–0.4% at `3+0.03` and 1.4% at `10+0.1`; the
  replay of first-move searches at `3000+30` shows 29% of searches with a
  fail-high past the soft target (RAR-R18). The cascade is rare; single
  fail-highs are common and are the re-searches worth their time.

## Hypotheses

- **H_R (leading): the cost is the re-search's depth, not its count.** A
  window searched after k fail-highs at depth − k (Stockfish's
  `rootDepth − failedHighCnt`, Reckless's `depth − reduction`) costs a
  fraction of a full iteration, the iteration completes, the clock's soft
  stop reads it, and the next iteration starts at full depth with the TT
  carrying the shallower bounds. The device changes every root fail-high,
  not only cascades, so its Elo at `3+0.03` is a gate question.
- **H_S (the schedule): the cascade is a property of the aspiration
  dynamics,** and a shallower re-search masks it: the fail-hard root and the
  geometric schedule set how many windows a rise needs, and that number is
  unchanged. True as stated, but the schedule is refuted territory (RAR-S89
  one retry −18.3 ± 6.9; RAR-S17 re-centring −4.52), its constants were
  fitted by C.10 around full-depth re-searches (RAR-S104, RAR-S105), and
  the traces show the count is not the cost: the last re-search of a
  cascade costs more than all the others together. H_S predicts the device
  lowers each window's cost but leaves k windows; it is not a competing
  fix, it is the reason the gate is symmetric.
- **H_I (information loss):** a shallower re-search finds less, so the
  iteration's exact result is k plies shallower than its nominal depth,
  the next iteration pays to recover it, and the clock's instability and
  effort factors read a nominal depth that was not searched. B.5 read this
  as canary shifts (`reduce`, 5 lost on WAC at fixed nodes; RAR-S89's
  packet) on a form that also re-centred. If H_I dominates, the gate reads
  negative despite the cascades ending; it is what `[-5,5]` must exclude.

## Interactions

- **The aspiration constants** (`AspirationDelta` 12, `AspGrowthPct` 164,
  `AspGrowthHighPct` 165, `AspGrowthAdd` 8, `AspMaxFails` 20) were fitted by
  C.10 with full-depth re-searches. A cheaper re-search changes what growth
  trades off; this registration does not refit them. Retry trigger below.
- **The TT** carries the shallower windows' bounds and PV into the next
  iteration at `depth − k`; a deeper probe at the next iteration treats
  them as shallow entries (ordinary). The exact window of a reduced
  iteration stores its root result at `depth − k`.
- **LMR and history at the root:** `root_delta` (the window's width, read by
  the PV reduction term) is unchanged; `root_depth` is set per window, so
  the singular-extension budget reads the depth actually searched. Root
  move records sample the iteration once whatever depth their last window
  was searched at (the prototype corrects the comparison).
- **The clock's factors** read iteration completion: a completed iteration
  at nominal depth with a reduced exact window feeds `falling_eval`,
  instability and effort as before; the soft stop fires between iterations,
  which is the mechanism by which the cascade ends on the clock.
- **The fail-high stop** (`TmFailHighTarget`, off) is the control arm of the
  probe and goes when the fix lands. The proved-mate stop (rec2) is
  untouched.
- **Threads:** each thread's own root loop carries its own count; the pool
  publishes a root move's score at the depth it was searched. MultiPV lines
  (analysis only) keep full-depth re-searches in the prototype; D.1.6.2
  decides whether the landed device applies there (recommendation: yes, the
  same `Aspiration` serves both loops).

## Invariants

- `bench 13` **moves** when the device is on: a deliberate change,
  re-declared when it lands. At the default (0) the fingerprint is
  **13,987,223 / EBF 2.565**, checked on the switch commit.
- `bestmove` is the move the last `info` line names; a reduced iteration's
  last line is exact and carries the nominal depth (both donors print
  `rootDepth`).
- No per-node allocation, lock or `Arc`: the device is one subtraction in
  the root loop (`tests/allocation_guard.rs` covers the search).
- Every root record samples each completed iteration once (test).

## The prototype

`AspFailHighReduce`, a tune-only spin in `SearchParams`, 0..=4, **default
0 = today**: the plies taken off a root window search for each fail-high so
far in the iteration, the window that finally holds included, floor depth 1;
a fail-low takes nothing off and the count resets each iteration (Reckless's
form inside an iteration; Stockfish also carries a recovery into the next
iterations, not prototyped). The iteration counts as its nominal depth. A
`diag` counter `asp_reduced_plies` sums the plies cut. Tests: the depth
arithmetic; a fail-high candidate at depth 10 with the reduction on
completes at depth 10, its last line exact, `bestmove` that line's move, and
every root record with ten samples.

## The reads (frozen before any ran)

1. **Bench and counter differential** (`tools/diag/bench_counters.py` on the
   `diag,tune` build, `AspFailHighReduce=1` against the default, stride 1):
   nodes, `root_iterations`, `asp_fail_high`, `asp_fail_low`,
   `asp_search_nodes`, `asp_fail_high_nodes`, `asp_reduced_plies`. The
   oracle differential (`phase4_differential.py`) is not run: the oracle
   does not carry the switch and the question is Rarog against Rarog.
2. **Untimed traces** of rec1 and rec3 on the prototype at 1 (RAR-R13's
   method, `trace_rec_d16.py`), read by `cascade_read.py` beside the head's.
3. **Timed probe, idle host** (`tm_rec_probe_d16.py`): rec1 and rec3 at
   `60000+600` and `180000+2000`, the head (the 2.6.0 engine, no clock
   stop) and the prototype at 1 with `TmFailHighTarget` left off.
4. **Fail-high rate and cost per iteration under a clock, idle host:** B.5's
   300 positions at `3000+30` on the prototype at 0 and 1
   (`tm_activation.py`'s replay with the aspiration counters added to its
   sum), the mean completed depth and the mean spend.

**Predictions:**

1. Bench 13 at 1: **0.80 ×** the head's nodes (band 0.65–0.92); B.5's
   `reduce` read 0.56 × on the B.9 head with re-centring on top, and C.10's
   faster growth already fails less. `asp_fail_high` within ±15% of the
   head's (the schedule sets the count, 0.7); `asp_fail_high_nodes` down by
   40–70%; `root_iterations` unchanged (fixed depth).
2. rec1: depth 19 completes within 30 M nodes from the start of the search
   (head: not in 94 M), and the trace reaches depth 21 or more in 40 s (0.8).
   rec3: depth 21 completes within 15 M nodes (head: not in 26.6 M), depth
   23 or more in 20 s (0.8). Fail-high windows under half the search nodes
   in both (0.8).
3. Probe: the head reaches ≥ 0.9 × the maximum on rec1 at both clocks (0.8)
   and runs rec3 past 2 × its optimum at `60000+600` (0.7); the prototype
   plays rec1 and rec3 under the maximum at both clocks (0.9) and within
   2.5 × the optimum (0.7). The last line of a prototype move is exact at
   its nominal depth (0.8).
4. Replay at 1 against 0: mean completed depth +0.2 to +0.6 ply nominal;
   `asp_fail_high` per iteration within ±20%; fail-high nodes' share of the
   search halved; mean spend within ±5% (the clock, not the tree, sets it).

**Falsifiers and stop rule:** a bench differential that does not cut
`asp_fail_high_nodes` by 25% or more, or a trace in which rec1's depth 19
still does not complete within the head's 94 M nodes, refutes the device as
a cascade fix and the leaf ends `NO_CHANGE` with the traces. A probe move at
the maximum with the device on fails the probe and sends the leaf back to
research. A bench cut past 0.5 × says the device removes far more than
cascades and the registration below widens its prior, not its bracket.

## The gate, registered here (D.1.6.3)

- **Arms:** the device at its seed (1 ply per fail-high) baked as the
  default on a throwaway arm from `d1`'s head, a clean PGO build versioned
  `2.7.0-dev+d16fix`, bench and counters recorded at the bake, against the
  2.6.0 engine `rarog-d15head-pext-pgo.exe` (`468ed7b`, `273FEDF4…`,
  13,987,223). **No clock stop in either arm** (`TmFailHighTarget` is off by
  default in both; the head has no such option). The candidate carries
  D.1.3's six clock pieces at their seeds and the head none: a confound
  shared with every D.1 read since RAR-R17, and the reason the clock pieces
  are read again on the gated engine in D.1.5.
- **SPRT:** Colosseum `sprt-repair` (`[-5,5]` nElo, pentanomial,
  `model=normalized`, α = β = 0.05; AGENTS' repair case 3: shared search
  signals change), `3+0.03`, 20 ms margin, Hash 64, one thread, 14 slots,
  the UHO book in random order, no adjudication, **cap 20,000 pairs**,
  `tools/results/rar-r19-gate`. About 12,000 games expected (RAR-S62),
  about two to seven hours.
- **Then:** the `10+0.1` direction read, `match-fixed-ltc`, 1,000 games,
  `tools/results/rar-r19-ltc` (about 0.6 hours); the probe with no clock
  stop on the gate's PGO binaries (`tm_rec_probe_d16.py`).
- **Dispositions:** H1 lands the device (harm near −5 excluded), re-declares
  the fingerprint and deletes `TmFailHighTarget`; the probe (rec1 and rec3
  under the maximum at both clocks) ticks D.1.2.1's cascade half. H0 refutes
  it and the fallback is the clock stop at twice the target behind its spin,
  read as RAR-R18 read it. The cap records "unresolved"; nothing is baked
  and the maintainer decides between a second cap and the fallback.
- **Prediction, frozen:** **−2 ± 6 Elo** at `3+0.03` (80% band −9 to +5);
  H1 0.35, cap 0.40, H0 0.25. The device cheapens re-searches that RAR-R18
  showed are worth their time, and the C.10 constants were fitted without
  it (H_I); the donors carry it under their own fits. `10+0.1`: the same
  sign as STC or better by a few Elo (the cascades are worth more there),
  P(point estimate above STC's) 0.6. Most likely miss: magnitude, from the
  unfitted constants.
- **Retry triggers:** a refit of the five aspiration coordinates around the
  device (a registered C.10-style block) only if the gate reads H0 with the
  probe passing, or the next search tune (F.6). Stockfish's recovery into
  the next iterations only if the gate passes and the probe shows the next
  iteration repaying the cascade.

**Evidence layer:** node counts, window counts and milliseconds here; Elo
only from the gate.

## Results (2026-10-23, zero games)

The switch is engine `027a604` (`AspFailHighReduce`, default 0; fmt, clippy at
zero warnings, debug and release tests all green, `bench 13` at the default
13,987,223 / EBF 2.565 on a fresh no-feature release build; the production
build lists no such option, the `tune` build does). The reads ran on its
`diag,tune` build (`target/diag`), in the registered order, the probe and
the replay on an idle host (2.6% busy, no engine, harness or build process).
Raw outputs: `analysis/artifacts/d16-cascade-2026-10-23/` (ignored):
`bench_counters_reduce{0,1}.txt`, `trace_head_rec{1,3}.txt`,
`trace_proto1_rec{1,3}.txt` with `cascade_read.py`, `tm_rec_probe_d16.txt`,
`replay_d16_reduce{0,1}.json`, `reads.log`, `verify.log`.

**1. Bench and counter differential** (`bench_counters.py`, stride 1):

| Counter | 0 (today) | 1 | Change |
|---|---:|---:|---|
| nodes | 13,987,223 | 8,510,754 | **0.608 ×** |
| `root_iterations` | 520 | 520 | — |
| `asp_fail_high` | 487 | 496 | +1.8% |
| `asp_fail_low` | 440 | 377 | −14% |
| `asp_fail_high_nodes` | 5,368,190 | 2,924,913 | **−45.5%** |
| `asp_fail_low_nodes` | 3,292,831 | 2,867,061 | −13% |
| exact windows (search − failed) | 5,312,968 | 2,704,701 | **−49%** |
| `asp_reduced_plies` | 0 | 976 | 1.97 a fail-high |

- Prediction 1: the fail-high count held (+1.8%, inside ±15%) and the
  fail-high nodes fell inside the band (−45.5% in 40–70%); **the bench
  missed low** (0.61 × against 0.80, band 0.65–0.92). Half the saving is
  not the cascades: the exact window after k fail-highs is searched k plies
  shallower and costs half what it did. That is H_I made visible at fixed
  depth, and the reason the gate is symmetric.

**2. Untimed traces at 1** (`cascade_read.py`; the head's beside them):

| Position | Head (0) | Prototype (1) |
|---|---|---|
| rec1, depth 19 | 7 fail-highs, 75 M nodes, **not completed** in 94 M (40 s) | completes at about 1.0 M nodes cumulative (375 ms) |
| rec1, deeper | — | depth 20: 7 fail-highs, 4.7 M (exact window 2.5 M at depth 13); depth 21: 7 fail-highs, 41 M (exact 14.4 M at depth 14), complete at 13.8 s; depth 22: first window 27 M at full depth, 4 fail-highs, 80 M, not completed at 40 s (127 M) |
| rec3, depth 21 | 5 fail-highs, 18.5 M, **not completed** in 26.6 M (20 s) | completes at 8.2 M cumulative (2.1 s) |
| rec3, deeper | — | depth 22: 4 fail-highs, complete near 19.7 M; depth 23: 4 fail-highs, not completed at 20 s (72.9 M) |

- Prediction 2: rec1's depth 19 under 30 M (held, by a factor of thirty);
  depth 21 or more in 40 s (held, 22 reached); rec3's depth 21 under 15 M
  (held); depth 23 or more in 20 s (**missed**, 22); fail-high windows under
  half the search nodes (**missed**: 86% and 83%, against the head's 85% and
  82%). Under `go infinite` the search runs on to the next cascade at the
  next depth, so the share is the positions', not the device's. What the
  device changes is that every iteration completes.
- **The next iteration repays part of the saving.** A reduced iteration's
  exact window is k plies shallower, and the next iteration's first window
  starts at full depth with a TT that holds only the shallower result: rec1's
  depth-22 first window costs 27 M, against 2.6–3.8 M for depth 21's
  windows. Stockfish's `failHighRecovery` (the next iterations also
  shallower, recovering by two plies an iteration) exists for this; it is
  the registered follow-up, not part of the device under gate.
- The count of fail-highs in a cascade is unchanged by the device (7, 7, 4,
  4): H_S holds as stated and is not the cost.

**3. Timed probe, idle host** (`tm_rec_probe_d16.py`; the 2.6.0 engine
`rarog-d15head-pext-pgo.exe` against the prototype at 1 on the `diag,tune`
build, no clock stop on either):

| Clock | Position | Head | Prototype at 1 |
|---|---|---|---|
| `60000+600` | rec1 (optimum 4.7 s, maximum 32.3 s) | 4.26 s (0.91 × optimum), depth 18, cp 1845 | 12.69 s (2.70 ×), depth 21, cp 2144 |
| `60000+600` | rec3 (optimum 4.4 s, maximum 30.3 s) | **30.33 s, the hard maximum** (6.87 ×), depth 21 lowerbound | 2.13 s (0.48 ×), depth 21, cp 769 |
| `180000+2000` | rec1 (optimum 15.2 s, maximum 104.4 s) | 38.09 s (2.51 ×), depth 19, cp 2882 | 12.53 s (0.83 ×), depth 21, cp 2144 |
| `180000+2000` | rec3 (optimum 14.3 s, maximum 97.9 s) | **97.95 s, the hard maximum** (6.87 ×), depth 21 lowerbound | 6.77 s (0.48 ×), depth 22, cp 903 |

- Prediction 3: the head at ≥ 0.9 × the maximum on rec1 at both clocks
  (**missed** at both: on the 2.6.0 engine rec1's depth-18 cascade completes
  at 4.3 s, just past the `60000+600` target, and depth 19's at 38 s, under
  the long clock's maximum); the head past 2 × rec3's optimum at `60000+600`
  (held, and at the maximum at both clocks: rec3 is the 2.6.0 engine's
  stall, where the B.9 head's was rec1). The prototype under the maximum at
  both clocks (held, all four); within 2.5 × the optimum (**missed** once:
  rec1 at `60000+600` 2.70 ×, where the head's luck ends the move at 0.91 ×);
  the last line exact at its nominal depth (held, all four). The prototype's
  rec1 search is the same at both clocks (47.3 M nodes, depth 21): the soft
  stop fires after the iteration that crosses the target, so the overrun is
  that one iteration's length.

**4. Replay under a clock, idle host** (`replay_d16.py` on `tm_activation.py`'s
driver; B.5's 300 positions at `3000+30`, first-move searches):

| | 0 | 1 |
|---|---:|---:|
| mean spend | 97.9 ms | 104.6 ms (+6.9%) |
| mean completed depth (nominal) | 11.61 | 12.28 (+0.68) |
| root iterations | 3,482 | 3,686 |
| fail-highs an iteration | 0.605 | 0.614 (+1.6%) |
| fail-lows an iteration | 0.514 | 0.469 |
| fail-high windows' share of root-window nodes | 0.279 | 0.323 |
| plies cut | 0 | 3,174 |
| searches ended by the hard stop inside a fail-high window | 0 | 2 |

- Prediction 4: the fail-high rate held (inside ±20%); the depth gain
  overshot slightly (+0.68 against +0.2 to +0.6, nominal plies on reduced
  exact windows); the share did **not** halve (it rose); the spend moved
  +6.9% against ±5%: the soft stop fires between completed iterations, and
  the iteration that crosses the target is a different one.

**Falsifiers:** none fired. Fail-high nodes fell 45% (rule: 25% or more);
rec1's depth 19 completes in 1 M nodes (rule: within the head's 94 M); no
prototype move reached the maximum. The bench cut, 0.61 ×, is above the
0.5 × line, so the prior is not widened.

## The arm (built 2026-10-23, after the reads)

- Throwaway branch `d16-arm` at `2d013c8` on `027a604`, in a worktree at
  `D:/code/rarog-d16arm`: `AspFailHighReduce` 1 as the default, version
  `2.7.0-dev+d16fix`. Recipe
  `analysis/arm_patches/2d013c8-rar-r19-d16gate.patch` (SHA-256
  `2c743b42…f097f1`); applied to `027a604` through a temporary index it
  reproduces the arm's tree `2ff4b241…`.
- `tools/test_engines/rarog-d16fix-pext-pgo.exe`, SHA-256
  `F90F1735E29577BF60E091A6B6E581AAD3A654E6085BFBDF604DD6E282A1A477`,
  `cargo xtask build --arch pext --pgo` by `tools/build_test.ps1 -SourceRoot`,
  `rustc 1.99.1`, clean tree; **bench 8,510,754**, equal to the `diag,tune`
  build with the option set. The head is `rarog-d15head-pext-pgo.exe`
  (`468ed7b`, `273FEDF4…`, 13,987,223), the same binary RAR-R16–R18 used.
- `run_r19.ps1 -Read gate|ltc|probe` beside this entry runs the three reads,
  `-ExpectBench 8510754, 13987223` pinning both arms.

## Calibration and disposition

- Four of the sub-predictions missed: the bench cut was larger than
  predicted, the head's stall is rec3 rather than rec1 on this evaluation,
  one prototype move overran 2.5 × its optimum, and the fail-high share and
  the spend moved where the registration said they would not. Every miss is
  in magnitude or in which position stalls; none is in mechanism or sign.
  The mechanism reading (iterations complete, the clock reads them) held in
  every trace and every probe row.
- **The cost the gate must weigh, read without games:** at fixed depth the
  exact window after fail-highs is k plies shallower and the bench loses 39%
  of its nodes, half of it there; under a clock the first move gains 0.7
  nominal ply for 7% more time, nominal because the plies were not all
  searched. Nothing here converts to Elo (RAR-S89's lesson: depth and nodes
  pointed the wrong way at −18.3). The frozen gate prediction stands
  unchanged at −2 ± 6; after these reads the agent's own lean is a little
  lower, since the fixed-depth loss is larger than the registration assumed,
  and that lean is recorded here and not written into the prediction.
- **Disposition as first written, before the probe on the PGO arm:
  `READY_FOR_IMPLEMENTATION`** (superseded below, kept as written). The
  device ends both cascades on the 2.6.0 engine where the head runs rec3 to
  the hard maximum at both clocks, its default-0 switch is bench-neutral,
  and the gate is registered symmetric with the fallback named. Semantics
  for D.1.6.2: one ply a fail-high, floor 1, reset each iteration, the exact
  window included, the nominal depth reported and sampled; MultiPV the same.

## The probe on the PGO arm (2026-10-23, after the arm was built): the stop rule fires

`run_r19.ps1 -Read probe`, idle host (3.8% busy, no engine, harness or build
process), `rarog-d16fix-pext-pgo.exe` against `rarog-d15head-pext-pgo.exe`,
no clock stop on either (`tm_rec_probe_d16_gate.txt`):

| Clock | Position | Head (2.6.0 engine) | Arm (`d16fix`) |
|---|---|---|---|
| `60000+600` | rec1 | 4.41 s (0.94 × optimum), depth 18 | 11.26 s (2.40 ×), depth 21 |
| `60000+600` | rec3 | **30.33 s, the hard maximum** (6.87 ×) | 5.31 s (1.20 ×), depth 22 |
| `180000+2000` | rec1 | 39.05 s (2.57 ×), depth 19 | 10.74 s (0.71 ×), depth 21 |
| `180000+2000` | rec3 | **97.95 s, the hard maximum** (6.87 ×), depth 21 lowerbound | **97.95 s, the hard maximum** (6.87 ×), depth 23 completed, 109.6 M nodes |

- **The arm runs rec3 into the hard maximum at `180000+2000`.** The
  `diag,tune` build at the same setting ended that move at 6.8 s after
  depth 22; the PGO build is about 1.3 × faster, completed depth 22 under
  the soft target, went on to depth 23 (completed) and depth 24, whose
  cascade did not complete in the 90 s left to the maximum. Which iteration
  crosses the target is set by speed; the probe's PASS on the slower build
  was that luck, as the head's 0.94 × on rec1 at `60000+600` is.
- **Why the device does not end this stall.** Three readings from the
  traces, each verified on rec1 and rec3: (1) the number of windows in a
  cascade is unchanged (the schedule sets it; 7, 7, 4, 4); (2) in these
  endings one ply of depth saves little per window (rec1 depth 22's windows
  27.1 → 20.7 → 17.6 → 14.6 M nodes at depths 22, 21, 20, 19: about 20% a
  ply, not the bench's 2.5 ×, because the tree here is extensions, checks
  and the TT, not width); (3) the next iteration's first window starts at
  full depth over a TT that holds only the shallower result and is the
  most expensive window of all. So the device moves the stall two or three
  nominal plies deeper and makes it rarer at `60000+600`, but at
  `180000+2000` an iteration begun just under the target can still cost
  more than the 90 s to the maximum. **H_S holds more than the registration
  allowed:** the cascade's cost is the schedule times the per-window cost,
  and in the stall positions the per-ply saving is small.
- **The frozen stop rule fires:** "a probe move at the maximum with the
  device on fails the probe and sends the leaf back to research." Prediction
  3's "the prototype under the maximum at both clocks (0.9)" is **missed**
  on the arm.
- The arm's rec1 and the `60000+600` rec3 rows are large improvements
  (39 → 10.7 s, 30.3 → 5.3 s); one row is worse but under the maximum (rec1
  at `60000+600`, 4.4 → 11.3 s). Three of four stalls end; one does not.

## Disposition: `MORE_RESEARCH` (supersedes the paragraph above)

- **D.1.6.1 stays in research.** The depth-reduced re-search alone is not a
  complete fix of the cascade stall on the 2.6.0 engine: it is the half of
  both donors' device that lives in the search, and the other half is
  clock-coupled in both of them (Stockfish stops deepening once half the
  target is spent and carries a recovery into the next iterations; Reckless
  re-centres on the failing score, which here saves at most one window, and
  relies on its soft bound). The maintainer asked for the fix in the search
  and not a clock cap; the evidence says the search half ends three of four
  probed stalls and not the fourth.
- **The gate (D.1.6.3) stays registered and is held, not started:** its
  registration reads the arm as the cascade fix, and the probe says it is
  not one. Running it would measure the device's Elo at `3+0.03` (a question
  of its own: the bench loses 39% of its nodes, half in shallower exact
  windows), and the maintainer may choose to run it as that read; the agent
  recommends against it until the device is complete, since a `[-5,5]`
  verdict on a partial device would then have to be repeated.
- **Next read, registered here as the cheapest discriminator (zero games,
  node-indexed, not speed-dependent):** the cost of the most expensive
  iteration of rec1 and rec3 within a 200 M-node budget (`go nodes`, one
  thread), for the head, the device at 1, and the device at 1 with
  Stockfish's recovery (the next iterations shallower by
  `(fail-highs + 1) / 2 + 2`, decaying by two an iteration), as a second
  tune-only coordinate. The stall ends on a clock when no iteration costs
  more than the maximum less the target, about 100 M nodes at the PGO
  speed in these positions. Prediction: the device alone leaves an iteration
  over 100 M in both positions (as the traces show); with the recovery the
  worst iteration falls under 60 M in rec3 and under 100 M in rec1 (0.5);
  if it does not, the stall is the schedule's and the search-only fix is
  refuted for these positions, which returns the question to the
  maintainer with the clock-coupled half named. The recovery is a
  broadening of the mechanism and is implemented only on the maintainer's
  word.
- Retry triggers as registered; the fallback (the clock stop at twice the
  target, `TmFailHighTarget` 20000, probe PASS in RAR-R18) stands.

## Maintainer decision 2026-10-23: complete the device, then gate it

The gate is not run on the partial device. The research continues toward a
fix that works, several connected mechanisms if need be, each read
node-indexed and the whole gated once.

## The recovery read (2026-10-23, `go nodes 200000000`, one thread, Hash 64)

`AspFailHighRecover`, the second tune-only switch (engine commit
`bce79c6`, off by default): after an iteration with k fail-highs the next iterations
start `(k + 1) / 2 + 2` plies short, two less each iteration (Stockfish's
`failHighRecovery`). Verified on the final state: fmt, clippy at zero
warnings, debug and release tests green, bench at the defaults 13,987,223 /
EBF 2.565. Traces `trace_nodes_*.txt` beside this entry, read by
`cascade_read.py`; a window still running when the node budget ends prints
no line, so its cost is the budget less the last line's nodes.

| Arm | rec3: costliest iteration | rec1: costliest iteration |
|---|---|---|
| head | depth 21: **> 192 M**, not completed (6 fail-highs printed, the seventh window 133 M and running) | depth 19: **135 M**, completed (9 fail-highs, exact window 35 M); depth 20 running at 46 M |
| reduce 1 | depth 23: 84 M completed (5 fail-highs, exact 34 M); depth 24: **> 90 M**, its first full-depth window still running | depth 21: 41 M completed; depth 22: **> 153 M**, not completed (8 fail-highs of 14–27 M each) |
| reduce 1 + recovery | depth 25: 28 M completed; depth 26: **> 162 M**, not completed (one fail-high, then a window of 136 M running) | depths 22–28 cheap (0.5–12 M, exact windows at reduced depth); depth 29: **> 152 M**, not completed (7 fail-highs, then a window of 129 M running) |

- **Prediction (the device alone leaves an iteration over 100 M in both;
  with the recovery the worst under 60 M in rec3 and under 100 M in rec1,
  0.5): the first half held, the second failed.** The recovery lets the
  nominal depth inflate cheaply (rec1 reaches depth 28 in 28 M nodes with
  exact windows at reduced depth) and the cascade returns at the next
  rising score, at the same cost. In these endings the cost of proving a
  rising bound at an effective depth near 20 is 10–40 M nodes a window,
  seven or eight windows a cascade, and no depth device changes the window
  count or the per-window cost by more than about 20% a ply.
- **What this says about the donors.** Stockfish does not stall on these
  positions (RAR-R16's probe: 9.3 s and 43.8 s on rec1, 5.7 s and 19.3 s on
  rec3, two to three times Rarog's optimum, never its maximum) because its
  third element stops deepening once half the target is spent: the next
  iteration repeats the effective depth, the table answers its first window
  at once, and the soft stop fires. That element reads the clock. It cannot
  bound an iteration begun under the target either; Stockfish's iterations
  are cheaper on these positions than Rarog's, which is a search question
  beyond the root loop.
- **The remaining search-side lever is the schedule itself:** a cascade of
  eight shrinking windows against one open window. The exact window after
  rec1's depth-19 cascade cost 35 M where the cascade cost 100 M; opening
  the fail-high side after a few consecutive failures would cost a few
  small windows and one open one. `AspMaxFails` is already a coordinate
  (both sides; RAR-S89 gated 1 at −18.3 Elo and swept 2, 4 and 8 at zero
  games, never gating them), so this read needs no code.

**Third read, registered before it ran:** `AspMaxFails` 3 and 4, alone and
with `AspFailHighReduce` 1, rec1 and rec3 at 200 M nodes. Prediction: at 3
the costliest iteration falls under 60 M in rec1 (head 135 M) and under
50 M in rec3 (0.6); with the reduction as well under 40 M in both (0.5); at
4 between those and the head's. Falsifier: a costliest iteration still over
100 M at 3 refutes the schedule as the lever too, and the search-only fix
of these stalls is refuted on this engine, leaving the clock-coupled
elements (Stockfish's depth freeze past half the target, or RAR-R18's stop
at twice the target) as the only measured devices.

**Third read, result (traces `trace_nodes_maxfails*` and
`trace_nodes_reduce1maxfails*`):** the falsifier fires in every arm. Once
the fail-high side opens fully, the one open window costs more than the
cascade it replaces:

| Arm | rec3 | rec1 |
|---|---|---|
| `AspMaxFails` 3 | depth 20: three fail-highs (2–3 M each), then the open window **> 182 M**, running at the budget | depths 17–32 exact at 904 for 0.1–3.9 M each; depth 33: three fail-highs, then the open window **> 168 M** |
| 3 with the reduction | depth 21 completes at 17.8 M; depth 22's open window **> 168 M** | depths 18–28 exact at 743; depth 29's open window **> 188 M** |
| `AspMaxFails` 4 | depth 18: four fail-highs, then the exact window **80.8 M** (completed); depth 19's **> 104 M** running | depth 25's open window **> 184 M** |
| 4 with the reduction | depth 22 completes at 35.6 M; the next window **> 149 M** | depth 21: exact window at 3013 **69 M**; depth 22: a 46 M window, then **> 36 M** running |

The schedule is not the lever either: proving the position's rising score
at an effective depth near 20 costs 70–190 M nodes however the windows are
cut, so **the search-only fix of these stalls is refuted on this engine's
root loop** (three devices, node-indexed, both positions).

## Where the cost is: the score drifts with depth and the lines grow to the cap

Depth for depth, one thread, Hash 64, fresh processes (`depthcost.log`,
`rarog_rec*_d*.txt`; Stockfish 19 at `D:/chess/engines/stockfish/`):

| Position, depth | Stockfish 19: nodes, score, seldepth | Rarog (the head): nodes, score, seldepth | Ratio |
|---|---|---|---:|
| rec3, 16 | 0.13 M, cp 484, 29 | 1.15 M, cp 706, 41 | 8.8 × |
| rec3, 18 | 0.28 M, cp 506, 39 | 2.01 M, cp 727, 45 | 7.3 × |
| rec3, 20 | 1.04 M, cp 573, 50 | 8.04 M, cp 831, 61 | 7.7 × |
| rec1, 16 | 0.24 M, cp 520, 31 | 0.16 M, cp 706, 39 | 0.7 × |
| rec1, 18 | 0.33 M, cp 541, 26 | 19.06 M, cp 1845, 89 | **57 ×** |
| rec1, 20 | 0.72 M, cp 524, 39 | not completed in 200 M (depth 19 cost 135 M, cp 2882) | > 280 × |

- **Stockfish's root score is flat across these depths** (484–573, 520–541)
  and its windows hold; it reads the decisive value (cp 9165) only from
  depth 31 on, in one step. **Rarog's root score rises with every ply**
  (rec3 706 → 727 → 831 → 903 → 1028 → 1149; rec1 799 → 1845 → 2882), so
  every iteration fails high and must prove a higher bound than the last,
  and each proof is a tree whose lines reach four to five times the nominal
  depth (seldepth 89 at depth 18, 127 at depth 19, the ply cap) where
  Stockfish's reach 1.4 ×. The cascade is the aspiration loop following a
  score that drifts with depth through a tree that grows to the cap. The
  root loop is where the symptom shows, not where the cost is.
- What drives the drift and the growth is the next question, and it is a
  search and evaluation question for won rook endings (B.5's "no bound on
  deep-iteration line growth", recorded then without a measured cost; the
  evaluation's scaling of a winning material edge with depth; extensions
  and quiescence checks in rook endings). The diag dump at depth 18 on rec1
  shows an 82% TT hit rate and about 1.6% of nodes truncating an extension
  against the line budget, against 1.05% on bench; the sampled counters
  are not read further here (ratios need stride 1, `bench_counters.py`'s
  method).

## Disposition: `MORE_RESEARCH`, the question re-scoped

- **Refuted on this engine, node-indexed:** every root-loop device read
  here (depth-reduced re-searches, Stockfish's recovery, the schedule
  opening after three or four failures) leaves an iteration of 90–190 M
  nodes in rec1 and rec3, so none ends the long-control stall by itself.
  The reduction alone still ends three of four probed stalls and reads
  0.61 × on bench; that is a strength question, not a fix, and the gate on
  it stays held.
- **The cause is upstream of the root loop:** a root score that rises with
  depth in won endings, proved through lines that grow to the ply cap, at
  7–57 × Stockfish's nodes for the same depth. The proper fix is the
  research leaf that explains and bounds that, with its own cheap
  discriminators: the static evaluation along the PV against the search
  score at each depth (drift from the evaluation's scaling or from the
  search finding the conversion), the extension and quiescence composition
  of these trees at stride 1, and Stockfish's classical evaluation inside
  Rarog's search on the same positions (RAR-O07's instrument) to separate
  the evaluation's share.
- **For the clock, meanwhile:** the only measured devices that end every
  probed stall are clock-coupled: RAR-R18's fail-high stop at twice the
  target (probe PASS, +12.2 ± 8.8 against the stop at the target, 0 ± 13
  against no stop) and Stockfish's depth freeze past half the target
  (unread here). The research recommends the maintainer reconsider the
  stop at twice the target as the robustness piece in D.1.5's closing read
  while the search cause is researched; a `[-5,5]` gate on any root-loop
  device as "the fix" is not recommended.
- The switches stay on `d1` at their defaults (`AspFailHighReduce` 0,
  `AspFailHighRecover` 0, `TmFailHighTarget` off), bench-neutral, as the
  instruments of the next leaf; the arm `2d013c8` and its binary stay for
  the held gate.

**Evidence layer:** node counts and window counts throughout; Elo only in
the RAR-R18 figures quoted.

## Maintainer decision 2026-10-23: the stop stays off; D.1.6.1 continues on the re-scoped question

## Where the cost is (2026-10-23, zero games, node-indexed)

**1. The same evaluation in Stockfish's search does not drift and does not
grow.** The oracle `hybrid/stockfish/src/stockfish.exe` (Stockfish's search
over Rarog's frozen HCE, `Use Rarog HCE` on; RAR-O07's instrument) and
Stockfish 19, `go depth 26`, one thread, Hash 64 (`drift.log`,
`drift_*_rec*.txt`):

| Engine, position | Score by depth | Nodes to depth 20 / 26 | seldepth at 26 |
|---|---|---:|---:|
| Oracle (Rarog's eval), rec1 | 700 (d1) → 877–888 flat from d12 to d25 → 1110 at d26 (finds e7e8q) | 0.57 M / 3.07 M | 39 |
| Oracle (Rarog's eval), rec3 | 416 → 601 (d20) → 718 (d26), about 15 cp a ply | 0.93 M / 4.73 M | 41 |
| Stockfish 19, rec1 | 520–654, flat | 0.72 M / 3.32 M | 37 |
| Stockfish 19, rec3 | 434–621, flat | 1.04 M / 2.31 M | 37 |
| Rarog, rec1 | 799 (d17) → 1845 (d18) → 2882 (d19) | 19.06 M to depth 18; depth 20 not in 200 M | 89 at 18, 127 at 19 |
| Rarog, rec3 | 706 (d16) → 831 (d20) → 1149 (d23) | 8.04 M to depth 20 | 61 at 20 |

With Rarog's own evaluation, Stockfish's search reaches depth 26 on rec1 in
3.1 M nodes with a score that moves 11 cp over fourteen depths; Rarog spends
19 M on depth 18 and its score doubles. **The drift and the cost are the
search, not the evaluation.** Rarog's depth-18 PV on rec1 is 21 plies long
and ends in `e7e8q`: the search finds the conversion, correctly, through
lines far longer than the nominal depth.

**2. Where the lines grow.** `reduced_depth` searches a reduced move at
`clamp(new_depth − r/1024, 1, new_depth + 2) + 2·PV`: a negative reduction
takes a child up to two plies ABOVE its nominal depth, and a PV node adds
two more, outside the line budget that caps singular and LDSE extensions at
the iteration's depth (Reckless has the same expression; Stockfish's PV term
is one ply). The reduction's negative terms are the PV node, the window,
improvement, the correction history, quiet and noisy history, the TT-PV
flags, the critical-ply distance and a checking move. Three new exact diag
counters (`lmr_deeper`, `lmr_deeper_plies`, `lmr_deeper_pv`; engine
`d428c00`, diag only) read how often a reduced move is searched deeper
than `new_depth` (stride 1, `bench_counters_lmr.txt`, `lmr_rec*_d*.txt`):

| Search | reduced moves | searched deeper | share | plies added | at PV nodes |
|---|---:|---:|---:|---:|---:|
| bench 13 | 4,216,025 | 443,080 | 10.5% | 762,867 | 103,666 |
| rec1 depth 16 | 498,217 | 36,129 | 7.3% | 57,572 | 6,471 |
| rec1 depth 18 (the cascade iteration) | 5,191,212 | 1,628,623 | **31.4%** | 2,721,289 | 16,633 |
| rec3 depth 18 | 587,981 | 55,098 | 9.4% | 89,069 | 6,322 |
| rec3 depth 20 (the cascade iteration) | 2,377,670 | 340,129 | 14.3% | 552,914 | 14,753 |

In the iteration that cascades, a third of all reduced moves are searched
deeper than their nominal depth, three times bench's share, and almost all
of them at non-PV nodes: the growth is the negative reduction, not the PV
bonus.

**3. Which terms.** Each coefficient is a tune option, so each term was
zeroed in turn on the `diag,tune` build, rec1 `go depth 18` and rec3
`go depth 20` (`lmr_terms.log`, `lmr_terms_*.txt`; stride 1):

| Term zeroed | rec1 d18: nodes, seldepth, score, deeper share | rec3 d20: nodes, seldepth, score, deeper share |
|---|---|---|
| none (the head) | 19.06 M, 89, 1845, 31.4% | 8.04 M, 61, 831, 14.3% |
| `CoreLmrCorrection` | **4.07 M, 39, 1184, 8.9%** | **6.32 M, 48, 808, 8.3%** |
| `CoreLmrGivesCheck` | **2,023 M, 127, 1662, 72.4%** (572 s) | stopped after 30 M nodes, unfinished |
| `CoreLmrQuietHistory` | 6.73 M, 43, 1213, 4.0% | 23.9 M, 49, 953, 5.8% |
| `CoreLmrPv` and `CoreLmrPvWindow` | 2.13 M, 42, 829, 9.2% | 21.8 M, 90, 968, 21.4% |
| `CoreLmrImprovement` | 3.95 M, 47, 830, 12.0% | stopped after 11 minutes, unfinished |

- **The correction term is the one term whose removal shrinks both trees,
  bounds both selective depths and flattens both scores** (rec1's depth-18
  score 1184 instead of 1845): `r −= 3330 · |correction| / 1024` is worth
  one to three plies when the correction history is large, and it is large
  exactly where the evaluation is far from the search's results, which is
  these endings. The deeper searches then find more, the correction grows,
  and the reductions go further negative: a feedback loop through the
  correction history.
- **Every other term is also live at the ply scale:** removing the check
  term takes rec1's depth 18 to two billion nodes, removing the PV terms
  takes rec3's to 21.8 M with selective depth 90. The reduction formula in
  these positions sits where a one-ply change in one term changes the tree
  by three to a hundred times. None of this is Elo; C.10 fitted these terms
  on a `3+0.03` tune and they are right for games, not for these endings.

**4. The prototype for a zero-game read, registered before it ran:**
`CoreLmrDeeperCap` (tune-only, 0..=4, default 4 = today): the plies a
reduced move may be searched above `new_depth`, the PV node's two included.
Reads: bench 13 counters at 4, 2 and 0 (4 must reproduce 13,987,223); rec1
`go depth 18` and rec3 `go depth 20` at each; the 200 M-node traces at 0 and
2 for the costliest iteration. **Prediction:** at 4 the fingerprint is
exact; at 0 bench falls to 0.75–0.95 × (0.6), rec1's depth 18 under 6 M
nodes with selective depth under 50 (0.7), rec3's depth 20 under 6 M (0.6),
rec1's depth-18 score under 1500 (0.6); in the traces at 0 the costliest
iteration is under 60 M in both positions (0.5), at 2 between 0 and the
head. **Falsifier:** an iteration over 100 M at 0 in either position means
the deeper searches are not the growth's cause and the loop runs through
another path (the correction history's own feedback through pruning and the
TT), which is then the next read.

**5. The cap, read** (engine commit `ddbe1f2`, default 4; fmt, clippy at
zero warnings, debug and release tests green, bench at the defaults
13,987,223 / EBF 2.565; `cap_reads.log`,
`bench_counters_cap*.txt`, `cap*_rec*_d*.txt`, `trace_nodes_cap*_rec*.txt`):

| | cap 4 (today) | cap 2 | cap 0 |
|---|---|---|---|
| bench 13 | 13,987,223 (exact) | 10,993,003 (0.786 ×) | **6,639,253 (0.475 ×)** |
| reduced moves searched deeper, bench | 443,080 | 272,391 | 0 |
| rec1 depth 18: nodes, seldepth, score | 19.06 M, 89, 1845 | 2.67 M, 51, 948 | 3.22 M, 40, 1228 |
| rec3 depth 20: nodes, seldepth, score | 8.04 M, 61, 831 | 3.49 M, 49, 695 | 1.53 M, 39, 605 |
| rec3, 200 M nodes: costliest iteration | depth 21 > 192 M | depth 21 **> 196 M** (six fail-highs, the seventh window 179 M and running) | depth 27 **100.4 M** completed (score 1237 → 1486, the exact window 42 M); depth 28 running |
| rec1, 200 M nodes: costliest iteration | depth 19, 135 M | depth 23, 67.6 M; depth 24 **> 128 M** | depth 25, 75.3 M (score 2980 → 6434, decisive); depth 26 > 62 M running, scores 7353 → 9437 |

- Prediction 4: the fingerprint at 4 held; at 0 the fixed-depth trees
  shrank as predicted (rec1 3.2 M and selective depth 40; rec3 1.5 M; the
  score 1228 under 1500) **but bench fell to 0.475 ×, far past the band**:
  the deeper searches are half of Rarog's tree at depth 13, which C.10
  fitted around them. In the traces at 0 the costliest iteration is 100.4 M
  in rec3 (the rule said under 60 M; 100 M is the falsifier's line) and
  75 M in rec1, with the explosion moved seven nominal plies deeper and
  arriving when the score approaches the decisive value; at 2 rec3's depth
  21 is as bad as the head's. **The falsifier fires:** the deeper searches
  amplify the growth (three to six times at fixed depth) and are not its
  whole cause.
- What remains at 0 is the cost of proving the win itself: rec1's scores
  run 2980 → 6434 → 9437 at depths 25–26 (Stockfish 19 reads 9165 there),
  and each window of that resolution costs 3–28 M nodes. The oracle,
  Stockfish's search over Rarog's evaluation, reads the same conversion
  (1110 at depth 26) for 3 M nodes in all.

## Disposition: `MORE_RESEARCH`, the question answered one level down

- **Answered here, node-indexed:** (1) the cascade's cost and the score
  drift are Rarog's search, not its evaluation (the oracle with the same
  evaluation: flat score, 7–60 × fewer nodes); (2) in the cascade iteration
  a third of all reduced moves are searched deeper than their nominal
  depth, outside the line budget, and the correction-history term is the
  one whose removal shrinks both trees, bounds both selective depths and
  flattens both scores, with every other term live at the ply scale;
  (3) capping those deeper searches shrinks the fixed-depth trees three to
  six times and halves bench, and delays but does not remove the explosion,
  which returns when the search proves the win.
- **Not answered:** why Rarog's search proves a won rook ending at 7–60 ×
  the oracle's nodes with the same evaluation. That is a search-efficiency
  question over the whole selectivity core in one class of position, and it
  is where the stall's cause lies. Its instrument exists: the comparable
  counters of `phase4_differential.py` between Rarog and the oracle, run on
  these positions at fixed depth, which locate the mechanism (pruning,
  reductions, extensions, quiescence, TT use) whose counts diverge. That is
  the next read, and a change there is a strength change for a cluster with
  its own gate, not a D.1 time-management piece.
- **The switches stay on `d1` at their defaults** (`AspFailHighReduce` 0,
  `AspFailHighRecover` 0, `CoreLmrDeeperCap` 4, `TmFailHighTarget` off),
  all bench-neutral, as instruments. The held gate on the reduction arm is
  not run; the arm `2d013c8` and its binary stay for the record.
- **Recommendation:** D.1.6 cannot close on a root-loop or reduction-cap
  device: none ends the stall, and the cap is a halving of the tree that
  only a search gate could judge. The research recommends (a) the
  differential read above as D.1.6.1's next step, R3; (b) if it points, as
  the term reads suggest, at the correction history's feedback into
  reductions, a bounded cluster on that feedback (the correction term's
  coefficient and clamp, the deeper-search ceiling) gated `[-5,5]` with the
  probe as its correctness read; (c) for the clock, the maintainer's
  decision stands: the stop stays off, and D.1.5's closing read runs
  without a cascade device, so RAR-R13's long-control stalls remain in
  2.7.0 until the search fix lands.

**Evidence layer:** node counts, selective depths and scores at fixed depth;
nothing here is Elo. The one Elo-adjacent fact is bench 0.475 × at cap 0,
which says the deeper searches are half of the tuned search at depth 13.

## Maintainer decision 2026-10-23: continue to the root cause

## The equal-result comparison (2026-10-23, zero games)

`phase4_differential.py` pointed at rec1 and rec3 (`differential_d16.py`, the
tool unchanged; `differential_*.txt`): Rarog's tree is 3.4 × the oracle's on
the standard suite at depth 12 and on these endings at depth 14, **20.6 × at
depth 18** (rec1 alone: 5.9 × at 17, 45.7 × at 18). Per node, between rec1's
depth 17 and 18, Rarog's null-move verifications rise 95 × raw (norm 0.05 →
0.32), its in-check nodes 15 × (norm 1.0 → 1.8), its reductions per node
halve (`reduction_depth_sum` norm 0.96 → 0.45), and its exact stores and
exact TT cuts fall to a third (norm 0.45 → 0.14, 0.28 → 0.08): the blow-up
iteration is bound-proving through checks with the reductions gone negative.

**The oracle to depth 34** (`oracle_deep_rec*.txt`), beside Rarog, same
evaluation:

| Score reached | Oracle: depth, cumulative nodes, that iteration | Rarog (head): depth, cumulative nodes, that iteration |
|---|---|---|
| rec1 ≈ 1850 | depth 30, 37.1 M, 13.7 M | depth 18, 19.1 M, 17.0 M |
| rec1 ≈ 2880 | depth 32, 100.2 M, 29.9 M | depth 19, 154 M, 135 M |
| rec1 4207 | depth 34, 172.8 M, 34.2 M | not reached in 200 M |
| rec3 ≈ 1010 | depth 31, 44.5 M, 15.6 M | depth 21 lowerbound 1104, not completed in 200 M (cap 0: 1486 at depth 27, about 170 M, that iteration 100 M) |

- **The total cost of proving the win is the same order in both searches**
  (rec1 to 2880: 100 M against 154 M; rec3 the oracle is about four times
  cheaper). **The iteration cost is not.** The oracle's iterations grow by
  about 1.3 × a ply at these depths (largest 38 M, selective depth 41–48 at
  depth 34); Rarog's jump 26 × from depth 17 to 18 and 8 × from 18 to 19,
  with lines at ply 89 and 127. Rarog's nominal depth 18 is the oracle's
  depth 30 in the PV region.
- **This is the stall's cause.** A clock can only act between iterations,
  and the iteration that resolves the win is, in Rarog, one enormous step
  of effective depth; in the oracle it is eight ordinary ones, so its soft
  stop ends the move within one modest iteration (Stockfish 19's 2.5–2.9 ×
  optimum overruns in RAR-R16's probe). The depth inflation comes from
  searching reduced moves above their nominal depth outside the line
  budget: the budget's own comment states that no line grows deeper than
  twice the iteration plus quiescence, and the negative reductions bypass
  it (selective depth 89 at depth 18 against a budget of 36 plus 16).

**The device, registered before its read:** `CoreLmrDeeperBudget`
(tune-only switch, 0 = today): the plies a reduced move is searched above
`new_depth` draw on the line's extension budget like a singular extension,
so the invariant holds; a line at its budget searches the move at
`new_depth`. Unlike the cap, it leaves the deeper searches alone until a
line has spent its budget, so bench should move little. **Prediction:**
bench 13 at 1 within 0.90–1.00 × (0.6); rec1 depth 18 selective depth at
most 52 (0.7) and under 8 M nodes (0.6); rec3 depth 20 under 5 M (0.6); in
the 200 M-node traces the costliest iteration under 60 M in both positions
(0.5) and no iteration more than four times its predecessor among the last
five completed (0.5). **Falsifier:** a costliest iteration still over 100 M
means the inflation runs through another path (singular extensions, the
in-check quiescence, which has no ply cap), and the next read is the
per-node accounting of `ply + depth` against the root depth.

**The budget switch, read** (engine commit `0366884`; fmt, clippy at zero
warnings, debug and release tests green, bench at the defaults 13,987,223 /
EBF 2.565; `budget_reads.log`, `budget1_rec*_d*.txt`,
`trace_nodes_budget1_rec*.txt`):

| | head | budget on |
|---|---|---|
| bench 13 | 13,987,223 | 9,802,148 (0.70 ×); reduced moves still searched deeper 188,696 of 3,129,972 |
| rec1 depth 18: nodes, seldepth, score | 19.06 M, 89, 1845 | **2.58 M, 33, 876** (the oracle: 0.37 M, 23, 879) |
| rec3 depth 20: nodes, seldepth, score | 8.04 M, 61, 831 | 3.18 M, 42, 688 |
| rec3, 200 M nodes | depth 21 > 192 M | depth 20 0.24 M; depth 21 16.2 M (7 windows); depth 22 57.6 M; depth 23 **> 123 M**, not completed; selective depth 51 |
| rec1, 200 M nodes | depth 19 135 M | depth 19 0.15 M; depth 20 **37.1 M** (9 fail-highs, 888 → 2818); depth 21 73.2 M (11 fail-highs, 2830 → 6284); depth 22 > 87 M; selective depth 52 |

- **The invariant holds and the depth-for-depth gap closes:** with the
  budget on, Rarog's depth 18 on rec1 reads the oracle's score (876 against
  879) at 7 × its nodes instead of 51 ×, and no line passes twice the depth
  plus quiescence (selective depth 33 and 52 against 89 and 127).
  Predictions 2–4 held; bench missed (0.70 × against 0.90–1.00: the budget
  bites at depth 13 too).
- **The falsifier fires all the same.** The costliest iteration is still
  over 100 M in rec3, and the cost still jumps 68 × (rec3, depth 20 → 21)
  and 245 × (rec1, depth 19 → 20) at the iteration where the win first
  resolves. With the lines bounded, that iteration is nine to eleven
  fail-high windows walking the schedule from 888 to 2818 and from 2830 to
  6284, each a bound-proof at near-full depth of 1–8 M nodes, then an exact
  window of 24 M. **The lumpiness is the score jump itself times the window
  walk, not the line growth.** The oracle climbs the same score range over
  eight iterations with one or two windows each because its root is
  fail-soft and its window re-centres on the failing score; Rarog's root
  returns exactly beta on every fail-high (every trace in this entry), so
  its window can only walk the schedule, and re-centring (RAR-S17, B.5's
  `recentre`) had no information to use when it was read.
- **Why the root returns beta: the interpolations are not it.** The
  candidate was the fail-high interpolation toward beta (quiescence
  `QsCutoffLerp` 575 and `QsStandPatLerp` 594, `CoreRfpLerp` 586,
  `CoreProbcutLerp` 253, `CoreSingMulticutLerp` 379 of 1024). With the two
  quiescence coefficients at 0 (`trace_nodes_lerp0_rec*.txt`, 60 M nodes)
  and with all five at 0 (`trace_nodes_alllerp0_rec1.txt`, 40 M) every root
  fail-high still returns exactly the schedule's beta (969 + 12, 27, 52, 93,
  161, 273, 458, 763). A `go depth 18` on rec1 with the quiescence pair at 0
  did not finish in five minutes and was stopped. The TT cutoff returns the
  stored score, the null-move cutoff `max(score, beta)`, the move loop and
  the fail-low return the fail-soft best score; none clamps to the window
  by inspection, so where the value becomes exactly beta is the next
  instrument: a diag histogram of `score − beta` at the root fail-high and
  of `alpha − best_score` at the ply-1 fail-low by its source (TT cut,
  null move, reverse futility, the move loop).

## Disposition: `MORE_RESEARCH`, the cause chain established, the device not yet

- **Established, node-indexed, same evaluation on both sides:** the stall is
  the iteration in which Rarog first resolves the win. Its cost is lumpy
  (26–245 × the previous iteration) where the oracle's is smooth (1.3 × a
  ply), for two reasons now separated by instrument: (1) lines grow past
  the budget through negative reductions, which the budget switch removes
  (selective depth 127 → 52, the oracle's depth-18 score reproduced) without
  removing the lump; (2) the root is fail-hard in effect and the window
  walks a geometric schedule, so a 2,000 cp rise costs nine to eleven
  near-full-depth bound-proofs in one iteration where the oracle pays one or
  two per iteration over eight. Neither the root loop's depth devices nor
  the schedule's opening changes (2), as the reads above showed; a fail-soft
  root with a re-centring window would, if the fail-hardness is where the
  candidate says.
- **Instruments on `d1`, all off or at today's value, bench-neutral:**
  `AspFailHighReduce` 0 (`027a604`), `AspFailHighRecover` 0 (`bce79c6`),
  the deeper-search counters (`d428c00`), `CoreLmrDeeperCap` 4
  (`ddbe1f2`), `CoreLmrDeeperBudget` 0 (`0366884`).
- **Next reads, in order:** (a) where the fail-high value becomes exactly
  beta: the overshoot histogram above (a diag change), since the five
  interpolations are cleared; (b) once the root can carry a fail-soft value,
  a tune-only switch for re-centring the fail-high side on the failing
  score (the donors' form) read at 200 M nodes for the windows per
  iteration and the costliest iteration, with the budget switch on and off;
  (c) the budget switch alone
  as a candidate for its own gate, since it restores a stated invariant,
  closes the depth-for-depth gap to the oracle and costs 30% of bench: a
  strength question `[-5,5]` cannot be skipped.
- **For the clock, nothing changes:** the stop stays off by the
  maintainer's decision; D.1.5's closing read runs without a cascade device.

**Evidence layer:** node counts, selective depths, window counts and scores
at fixed depth and at fixed node budgets; nothing here is Elo.

## Maintainer decision 2026-10-23: the fail-hard instrument, then re-centring with and without the budget

## Where the value becomes exactly beta (2026-10-23, zero games)

Exact diag counters (engine commit named below): at the root, fail-highs
whose score is exactly beta and the overshoot of the rest; at plies 1 and 2,
every value that left its window, by the return path that produced it (TT
cutoff, reverse futility, null move, ProbCut, multi-cut, a move-loop cutoff,
the loop's end) and whether it sat exactly on the edge it crossed
(`edge_rec*_d*.txt`, `bench_counters_edge.txt`, stride 1):

| | rec1 depth 18 | rec3 depth 20 | bench 13 |
|---|---|---|---|
| root fail-highs exactly at beta | **31 of 31** | **30 of 30** | **451 of 487** (93%) |
| ply 1, loop's end (the root move's fail-low): at alpha / below | 82 / 47 | 52 / 21 | 1,647 / 2,424 |
| ply 2, move-loop cutoff: at beta / above | 1,076 / 517 | 767 / 425 | 7,896 / 8,235 |
| ply 2, loop's end: at / off | 631 / 474 | 320 / 304 | 9,241 / 11,161 |
| ply 2, TT cutoff: at / off | 64 / 475 | 31 / 369 | 687 / 9,427 |

The root is fail-hard on the whole suite, not only here: 93% of bench's
root fail-highs return exactly beta. The TT cutoffs and the null move return
fail-soft values; the move loop's cutoffs and ends do not, two-thirds of
them exactly on the edge. The source is the leaf: **quiescence returns and
stores `alpha` when nothing raises it** (`node.rs`, the tail of
`quiescence`), so every fail-low leaf value is the window's edge, every
parent's cutoff is exactly beta, and the edge propagates to the root. Both
donors return the best value found (the stand pat or the best capture). The
legacy engine read fail-soft quiescence alone at **−5.96** (8.11, 2026-07,
`docs/archive/PLAN-legacy-2026-07-29.md`), traced to pruning fitted against
the fail-hard bounds, and re-centring alone at **−4.52** (RAR-S17) on a root
that returned beta: the two halves of the donors' mechanism were each read
without the other, and neither can work alone: re-centring on a value equal
to beta is the schedule, and fail-soft values that no window uses are only
different bounds in the table.

**The read, registered before it ran.** Two tune-only switches, both 0 =
today: `QsFailSoft` (quiescence returns and stores its best value) and
`AspRecentre` (a failed window re-centres on the failing score, both sides,
the donors' form; Rarog's growth and fail-low midpoint kept). Arms at 200 M
nodes on rec1 and rec3, read by `cascade_read.py` for the windows per
iteration and the costliest iteration, plus bench 13 and its counters for
each arm: fail-soft alone; re-centring alone; both; both with
`CoreLmrDeeperBudget`. **Predictions:** fail-soft alone cuts the root
fail-highs exactly at beta from 93% to under 30% on bench (0.7) and leaves
the window count of a cascade unchanged (0.8), bench within 0.85–1.15 ×
(0.6); re-centring alone changes neither the window count nor the costliest
iteration (0.8); **both together** bring the resolving iteration's windows
from nine to eleven down to four or fewer (0.6) and the costliest iteration
under 60 M in rec1 (0.5) and rec3 (0.4); with the budget as well under 40 M
in both (0.4); bench for the pair 0.7–1.2 × (0.6). **Falsifier:** the pair
still walking seven or more windows in the resolving iteration means the
fail-soft overshoot at the root is too small to steer the window, and the
lever is elsewhere (the schedule's growth under fail-soft values, which is
then a fit, not a device).

**The read, result** (engine commit `10f1b22`; fmt, clippy at zero
warnings, debug and release tests green, bench at the defaults 13,987,223 /
EBF 2.565; `pair_reads.log`, `bench_counters_{failsoft,recentre,pair,pairbudget}.txt`,
`trace_nodes_{failsoft,recentre,pair,pairbudget}_rec*.txt`):

| Arm | bench 13 | root fail-highs at beta, bench | rec1, 200 M: the resolving iteration | rec3, 200 M: costliest iteration |
|---|---|---|---|---|
| head | 13,987,223 | 451 of 487 (93%) | depth 19, 9 windows, 135 M | depth 21 > 192 M |
| fail-soft | 20,976,489 (**1.50 ×**) | 319 of 566 (56%) | depth 21, 9 windows (1035 → 2289), > 190 M | depth 18 > 200 M, not completed |
| re-centring | 10,332,915 (0.74 ×) | 413 of 439 (94%) | depth 20, 8 windows (861 → 2682), 100 M; depth 21 > 98 M | depth 20, 5 windows, 41 M; depth 21 > 151 M |
| both | 18,438,572 (1.32 ×) | 251 of 470 (53%) | depth 28, **8 windows** (732 → 2559), > 192 M | depth 18 > 194 M |
| both with the line budget | 14,309,316 (1.02 ×) | 241 of 467 (52%) | depth 26, **9 windows** (1260 → 4353), > 186 M | depth 22, 6 windows (733 → 1271), 122 M |

- **The falsifier fires.** With fail-soft quiescence and re-centring
  together the resolving iteration still walks eight or nine windows, and
  the windows' scores are the schedule's steps added to a failing score that
  sits on or just above the old beta (732 → 759 → 811 → 904 …): the root's
  fail-soft overshoot is too small to steer the window. The overshoot that
  does appear (bench: 44% of root fail-highs off beta, mean about 140 cp)
  does not reach the cascade iterations, whose bounds come through
  zero-window searches and the table.
- **Calibration.** Fail-soft alone: the at-beta share fell to 56%, not under
  30% (missed), the window count held (as predicted) and bench rose 50%,
  outside 0.85–1.15 (missed: the legacy +14.4% nodes understated it).
  Re-centring alone: predicted to change nothing; it cut bench 26% and
  shortened rec3's depth-20 cascade to five windows (missed), because
  re-centring on a value equal to beta adds the new delta to the old one,
  a faster schedule, which is what RAR-S17's −4.52 measured. The pair:
  predicted four windows or fewer (0.6), read eight to nine; predicted
  under 60 M, read over 180 M. The pair with the budget: predicted under
  40 M, read over 120 M.
- **What remains after every device:** the score itself jumps 1,500–3,000
  cp within one nominal ply when the win resolves (rec1: 1248 at depth 25,
  4353 at depth 26 with the budget on; 720 at depth 27, 2559 at depth 28
  without), where the oracle's score climbs the same range over eight
  iterations (888 → 1110 → 1345 → 1531 → 1752 → 1866 → 2112 → 2871 → 3093
  → 4207, depths 25–34). No window schedule walks a 3,000 cp jump cheaply;
  the oracle never has to. Rarog's effective depth still advances several
  plies a nominal ply even inside the budget (selective depth 52 at depth
  22 against the oracle's 41 at 34): singular, double and triple extensions
  and the deeper reduced searches fill the budget, which allows every line
  twice the iteration's depth.

**Next read, registered before it ran:** the budget's size as a coordinate,
`CoreExtensionBudgetEighths` (tune-only, 8 = today: a line's extensions may
sum to the root depth; 4 = half of it), read with `CoreLmrDeeperBudget` on
so that every depth addition is charged. Arms 8, 4 and 2 at 200 M nodes on
rec1 and rec3, bench for each. **Prediction:** at 4 the largest score jump
between consecutive completed iterations of rec1 falls under 1,500 cp (head
with the budget: 3,100) and the costliest iteration under 60 M in both
positions (0.5); bench at 4 within 0.80–1.00 × (0.6); at 2 the trees are
smaller again and the win is found later. **Falsifier:** a costliest
iteration still over 100 M at 4 and 2 means the lump is not the extension
budget's size either, and the remaining candidate is the schedule's growth
under a bounded tree, a fit over the five aspiration coordinates with the
budget on, which is C.10's territory and a tune, not a device.

**The budget's size, read** (engine commit `7b4a9e2`; fmt, clippy at zero
warnings, debug and release tests green, bench at the defaults 13,987,223 /
EBF 2.565; `bench_counters_eighths*.txt`, `trace_nodes_eighths*_rec*.txt`;
the line budget on in every arm):

| Eighths | bench 13 | rec3, 200 M nodes | rec1, 200 M nodes |
|---|---|---|---|
| 8 (today's size) | 9,802,148 (0.70 ×) | depth 23 > 123 M (above) | depth 20 37 M, 21 73 M, 22 > 87 M (above) |
| 4 | 6,984,801 (0.50 ×) | scores 830 → 831 → 992 → 1004 → 1164 over depths 26–31, largest step 161 cp; iterations 0.7, 3.5, 5.8, 48, 19, **86 M**, all completed | 1274 flat from depth 19 to 30 (0.2–2 M each); depth 31: **13 windows, 1286 → 7000+, > 186 M**, not completed |
| 2 | 4,658,737 (0.33 ×) | scores 749 → 1181 over depths 27–34, largest step 128 cp; iterations up to 68 M, all completed | 1096 flat from depth 22 to 34; depth 35: **12 windows, 1108 → 6822, > 190 M** |

- **rec3 behaves like the oracle at half the budget:** the score climbs in
  steps of about 130 cp and every iteration completes, the largest 86 M.
  Prediction 4's rec3 half held in kind and missed in size (86 against
  under 60 M).
- **rec1 does not.** With the lines bounded to one and a half or one and a
  quarter times the iteration, the win is found later (depth 31, depth 35)
  and still all at once: a flat 1274 for twelve iterations, then 7000 in
  one, through thirteen windows, over 180 M nodes. **The falsifier fires
  for rec1:** the lump is not the budget's size either. In this ending the
  conversion (e5–e6–e7–e8 with the rook and bishop covering) is a single
  forced line; the first iteration deep enough to see its end sees a
  decisive score, and the window must walk from 1274 to 7000 whatever
  bounds the lines. The oracle climbs the same ending in nine steps because
  its search sees the conversion gradually, a property of its pruning and
  extensions that no single Rarog coordinate reproduced here.
- Bench at 4 is 0.50 × today's tree and at 2 0.33 × (prediction 0.80–1.00
  missed): the budget's size is a large strength coordinate, not a repair.

## Disposition: `MORE_RESEARCH`; the fail-hardness explained, the lump located, no device yet

- **Established this round, node-indexed:** (1) the root is fail-hard in
  effect on every position (93% of bench's root fail-highs return exactly
  beta) and the source is quiescence returning `alpha` on a fail-low; (2)
  fail-soft quiescence and re-centring, the two halves of the donors'
  mechanism, do not shorten the cascade together (eight to nine windows,
  bench 1.32 ×): the root's fail-soft overshoot does not reach the
  resolving iteration; (3) charging the deeper reduced searches to the line
  budget restores its invariant and reproduces the oracle's depth-18 score,
  at 0.70 × bench; (4) a smaller budget smooths rec3's score path but not
  rec1's, whose win resolves in one iteration whatever bounds the lines.
- **What the stall is, in one sentence:** in a won ending whose conversion
  is one long forced line, the first iteration deep enough to see its end
  jumps the score by thousands of centipawns, and Rarog's window walks that
  jump in eight to thirteen near-full-depth bound-proofs, each 1–60 M
  nodes; the oracle reaches the same scores over eight iterations of
  gradual increase and one or two windows each.
- **Refuted as the device, node-indexed:** depth-reduced re-searches,
  Stockfish's recovery, the schedule's early opening, the cap on deeper
  searches, the line budget alone, fail-soft quiescence alone, re-centring
  alone, the pair, the pair with the budget, the budget's size. Each is on
  `d1` as a tune-only switch at its neutral default.
- **Not refuted, not read:** (a) the growth of the fail-high side of the
  schedule under a bounded tree, which a fit over `AspGrowthHighPct`,
  `AspGrowthAdd` and `AspirationDelta` with `CoreLmrDeeperBudget` on would
  read, a tune rather than a device (RAR-S89's one-retry rejection bounds
  one end of it); (b) why the oracle's score path is gradual on rec1 where
  Rarog's is a step: the per-ply differential at depths 26–34 on the
  oracle's own iterations against Rarog's at 19–20, an instrument the tool
  gives directly; (c) the clock-coupled half of the donors' device,
  Stockfish's depth freeze past half the target, which the maintainer's
  decision keeps out for now.
- **Recommendation:** I recommend against landing any of the switches read
  here as the cascade fix. Two of them are strength candidates in their own
  right and only games can judge them: `CoreLmrDeeperBudget` (an invariant
  restored, bench 0.70 ×) and the fail-hard quiescence tail (the donors
  are fail-soft; bench 1.50 ×, the legacy −5.96 read under pruning fitted
  to fail-hard bounds), each a `[-5,5]` gate against the 2.6.0 engine with
  the C.10 constants unfitted, and the research recommends the budget first
  since it restores a stated contract. For the stall itself the search-side
  reads are exhausted at the single-device level; what remains is a fit
  (a) and the instrument (b), and the clock-coupled half (c).

**Evidence layer:** node counts, selective depths, window counts and scores
at fixed depth and at fixed node budgets; nothing here is Elo.
