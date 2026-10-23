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
