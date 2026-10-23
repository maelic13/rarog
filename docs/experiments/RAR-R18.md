# RAR-R18 — D.1.5's research: why the clock cluster does not gain, read as time against decisions; the fail-high stop and the factors, three 2,000-game reads at `3+0.03` and a probe at twice the target

Indexed under *4. Root search, time management and SMP* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-23, before any game. The reads and
  the probe are the maintainer's commands; the zero-game work below is the
  agent's. RAR-R16's SPRT stays paused and is not touched.
- **Kind:** PLAN D.1.5 returned to research after RAR-R17: the mechanism
  question, not a gate. Nothing here accepts anything.
- **The question:** does any of D.1.3's seven mechanisms change the quality
  of the engine's decisions at `3+0.03`, or do they only move time, so that
  the cluster's Elo is the time it leaves unused?

## What is known (zero games, 2026-10-23)

**Every read so far sits on the time rule.** At `3+0.03` this engine
converts thinking time to Elo at about 145 Elo a doubling, 2 Elo per 1%
(RAR-S98, RAR-S99: bench-identical speed passes). Each of D.1.5's results
against that rule, with each arm's time a move from the runner's `search:`
lines or the profile:

| Read | Time a move, candidate against head | Time rule | Read | Deviation |
|---|---|---|---|---|
| RAR-R17 seeds against the head | 65.0 / 70.2 ms, −7.4% | −14.8 | −14.9 ± 8.8 | −0.1 (0.0 SE) |
| RAR-R16 fit against the head, `3+0.03` 1T: the paused SPRT's 4,690 games (`profile_rar-r16-gate-paused.txt`, `07d55d18…`) | 69 / 71, −2.5% | −5.0 | +0.6 ± 5.8 (the runner's last progress line, 2,345 pairs) | +5.6 (1.9 SE) |
| RAR-R16 fit, `10+0.1` | 230.4 / 232.3, −0.8% | −1.6 | −6.3 ± 11.7 | −4.7 (0.8 SE) |
| RAR-R16 fit, four threads | 67.4 / 69.1, −2.5% | −5.0 | −10.4 ± 11.6 | −5.4 (0.9 SE) |

Pooled, the deviation from the time rule is **+1.6 ± 4.2 Elo (95%)**. The
cluster, at its seeds or fitted, changes decision quality by somewhere
between −3 and +6 Elo at these controls, and moves time. The seeds' −15 is
the 7.4% they do not spend; the fit's +17% scale bought that back by
front-loading (118 against 100 ms a move in plies 0–19, 30 against 39 by
plies 100–119, wins ended with 759 ms unused against 323) and reads +0.6.
The rule is an STC constant; at `10+0.1` it is an assumption, so that row
is less certain than its interval shows.

**Where the seeds' time goes, from the replay** (`tools/diag/tm_activation.py`
at `a43b3a5` driving a `diag,tune` build of the same source, `9fb1eb51…`;
B.5's 300 positions at `3000+30`, first-move searches, idle host;
`r18-replay-*.txt` and `.json` beside this entry):

| Setting | Searches the stop ended | Mean spend | The stops' elapsed over the last target |
|---|---|---|---|
| the seeds, stop at the target (`7a1caff3…`) | 87 of 300 (29%) | 88.0 ms | p50 1.18, p90 1.59; 11.5% ≥ 1.5×, 2.3% ≥ 2×, none ≥ 3× |
| stop at 1.5× (`e2811791…`) | 18 | 95.9 | p50 1.65 |
| stop at 2× (`02e71823…`) | 2 | 97.4 | — |
| stop at 3× (`90701b1a…`) | 0 | 96.4 | — |
| stop off (`f09dfec5…`) | 0 | 97.1 | — |
| stop off, stability neutral, cross-move 0 (`5ee457ca…`) | 0 | 86.9 | — |
| stop at the target, factors neutral (`9a8bdd8d…`) | 95 | 80.4 | p50 1.19 |

- **The stop is not a cascade stop.** It fires on 29% of searches at a
  median 1.18 × the target and never past 3×: single fail-highs just past
  the target, where the head would have re-searched once. It removes about
  10% of the spend. At twice the target it fires on under 1% and the spend
  is the no-stop level, so that threshold is time-neutral by construction.
- **The stability factor adds about 12% of spend at the seeds** (its mean
  is 1.206 over iterations, 1.085 at the last completed one); the cross-move
  term is zero in a first-move replay. The two roughly cancel here (88.0
  against 86.9), while the games read the seeds 5% under the head in plies
  0–19 and 7.4% over the game. That gap is the stop's share of game moves,
  which a first-move replay cannot read; the `stopoff` read measures it.
- Mean factors at the seeds, all iterations / last completed: falling eval
  0.783 / 0.610, instability 2.80 / 1.57, stability 1.21 / 1.09, effort
  0.91 / 0.88. The ending target is about 0.9 of the optimum.

**Hypotheses.**
- **H_T (time alone):** each mechanism's Elo is the time it adds or
  removes at 2 Elo per 1%; no mechanism changes decision quality beyond
  ±10 Elo. The reads above favour it.
- **H_D (decisions):** the stop plays an unconfirmed move where the search
  was finding something and costs more than its time; or the stability ×
  instability × effort product, three factors on one event (a best-move
  change), misallocates. The pooled deviation bounds the whole cluster near
  zero, but one mechanism's cost could hide behind another's gain.
- **H_C (correctness only):** the stops' value is the long-control stalls
  (the probe: rec1 32 s → 4 s), unreadable in Elo at these controls; zero at
  `3+0.03`.

**Interactions.** The stop reads the aspiration window: it fires at a
fail-high boundary, and the window's growth sets how long a cascade runs
(unchanged; RAR-S89, RAR-S17). Its threshold is the soft target, which the
factors scale: with the factors neutral the target is lower and the stop
fires more (95 against 87 searches in the replay), so the factors' read
runs with the stop off on both arms. In a pool only the main thread stops;
a four-thread read waits for the kept set (D.2 owns the vote).

## The switch (engine `f3549ff`, behaviour-neutral by default)

`TmFailHighTarget`, a tune-only spin: the multiple of the soft target past
which a root fail-high ends the search; 10000 is the registered mechanism
and the default, so play is unchanged; 10000000, a thousand targets, lies
past every maximum (at most 6.873 optima against a target of at least
0.0146 of one at the ranges' ends), so the stop never fires. Verification
on the final state, exit status read directly: `cargo fmt --check` 0,
`cargo clippy --all-features --all-targets` 0 at zero warnings, `cargo test
--no-fail-fast` 0 and `cargo test --release --no-fail-fast` 0 (827 passed),
`bench 13` on a fresh no-feature release build **13,987,223 / EBF 2.565**.
The fail-high test also checks that at a thousand targets the same late
clock completes the fail-high's iteration. The reader (`a43b3a5`) reads
each search's spend and the stops' elapsed over the target.

The build: `tools/test_engines/rarog-d18switch-tune.exe` (SHA-256
`33C61D7DDA8A9E94…`, `pext-tune`, built clean in a worktree at
`D:/code/rarog-d18` from `a43b3a5`, whose engine inputs are `f3549ff`'s;
bench 13,987,223; fourteen `Tm*` spins). Its defaults are D.1.3's seeds with
the stop at the target.

## The reads (frozen)

Three categorical matches on that build, each **2,000 games** at `3+0.03`,
Hash 64, one thread, 14 slots, the UHO book in random order, no
adjudication, run by `run_r18.ps1` (`a0c6216c…`) `-Read
stopoff|cascade|factorsoff`, each resumable by the same command, about
20–25 minutes each. All three were dry-run to policy on 2026-10-23
(`colosseum_match_rar-r18-stopoff_20261023_142939`, `…cascade…142944`,
`…factorsoff…142949`), the run directories unused:

1. **`stopoff`:** arm A `TmFailHighTarget=10000000` (the stop never fires)
   against the seeds. Seed 20261037, `tools/results/rar-r18-stopoff`.
2. **`cascade`:** arm A `TmFailHighTarget=20000` (the stop at twice the
   target) against the seeds. Seed 20261038, `tools/results/rar-r18-cascade`.
3. **`factorsoff`:** arm A `TmStabBase=10000 TmStabSlope=0 TmStabFloor=10000
   TmFallPrevSlope=0` (the stability factor neutral, the cross-move slope
   zero) against the seeds' factors, **the stop off on both arms**. Seed
   20261039, `tools/results/rar-r18-factorsoff`. The effort band's
   whole-search share has no option and stays on both arms.

**And the probe,** `tm_rec_probe_d18.py` (`41aac5d3…`): rec1–rec3 at
`60000+600` and `180000+2000` on the same build, the stop at the target and
at twice it, a fresh process per search, idle host, about eight minutes.
Verdict on the 2× arm, D.1.2.1's acceptance: no rec move at the maximum,
rec2 under 2 s.

**Readings (frozen):**
- Each read's Elo with its interval from the manifest; each arm's time a
  move over the game and in plies 0–19 from `tm_wire_read.py` and
  `wire_resolution.py` (the reading lands beside the script as
  `<dir>.txt`), its clock floor and its clock at the end of games.
- **The time rule:** Δtime is arm A's time a move over the game against arm
  B's; the rule's Elo is 145 × log2(1 + Δtime), 2 Elo per 1% for small
  changes.
- **The decision effect** is the read less the rule's Elo. Its SE is the
  read's SE with 2 Elo for the time measurement added in quadrature, about
  4.8 at 2,000 games. **A mechanism changes decision quality if the
  decision effect is at least 10 Elo in magnitude** (about two SE);
  otherwise its effect is time alone, and a smaller decision effect is not
  resolved by these reads.

**Dispositions (frozen):**
- `stopoff` and `cascade` both time alone, and `cascade`'s arm A within
  ±3% of the seeds' time: the stop stays at **twice the target**, a cascade
  device that costs no time at `3+0.03`, in the kept set.
- `stopoff` with a decision effect of +10 or more (removing the stop gains
  more than its time): the stop at the target costs decisions; it stays only
  in `cascade`'s form, and only if `cascade`'s decision effect is under +10.
- `stopoff` with a decision effect of −10 or less (the stop at the target
  decides better than the time it saves): the stop stays at the target and
  its time is respent through the scale, which is the fitted arm; RAR-R16's
  SPRT then resumes as that arm's gate.
- `factorsoff` with a decision effect of −10 or less (the factors buy more
  than their time): the stability factor and the cross-move slope stay,
  with their time respent through the scale, which needs a fit. Otherwise
  they are **removed**: a mechanism added on donor evidence that neither
  the fit nor a read shows to decide better is not kept on an ungated
  branch.
- The probe failing on the 2× arm: the threshold is lowered toward 1.5×
  and the probe re-run before `cascade`'s read is used.
- **After the reads, the kept set** (the horizon, the mate stop, the
  first-iteration guard, the stop at the threshold the reads chose, and the
  factors only if they earned it) is registered as one repair read under
  AGENTS' case 2 against the 2.6.0 head: 2,000 games at `3+0.03` and 2,000
  at `10+0.1` (the maintainer's requirement of 2026-10-23), each with the
  harm rule at −13 nElo, before anything lands. The horizon and the mate
  stop carry no `10+0.1` harm read of their own yet; that read is it.

## PRE-REGISTERED PREDICTION (frozen before any game)

1. **`stopoff`:** arm A spends **+8%** a move over the game (band +4 to
   +14: the replay's +10% on first moves, and the games' 5% gap in plies
   0–19 that the replay does not show). Time rule +16 (band +8 to +27).
   **Read +16 ± 9.** Decision effect within ±10: 0.65. Arm A ahead: 0.95.
2. **`cascade`:** arm A spends +7% (band +3 to +13), nearly all of
   `stopoff`'s gain, since 97.7% of the stops fire under 2×. Time rule +14.
   **Read +14 ± 9.** Decision effect within ±10: 0.65.
3. **`factorsoff`:** arm A spends **−10%** (band −15 to −5: the stability
   factor's +12% in the replay, damped as the factor checks were, less the
   cross-move's small addition). Time rule −20 (band −30 to −10). **Read
   −20 ± 9.** Decision effect within ±10: 0.6; the factors buying more
   than their time (−10 or less): 0.15.
4. **Probe:** PASS on the 2× arm, 0.85; rec1 and rec3 under 2.5 × their
   optimum at both clocks, 0.7.
5. **No fault or time loss** in the three reads, 0.9.
- Most likely outcome: all three on the time rule. Then the stop goes to
  twice the target, the factors go, and the kept set reads as a repair,
  expected within ±5 of the head at both controls.
- The failure that would surprise: `factorsoff` far below its rule (the
  factors deciding better), which would reopen the fit.

**Evidence layer:** Elo from the three reads; counts, milliseconds and
formula values elsewhere. The time rule is a measured STC constant with its
own uncertainty (three passes, about ±0.3 Elo per 1%), which the bands
absorb.

## Recommendations recorded with the registration

- **RAR-R16's SPRT:** record it unresolved at the runner's last progress
  line (2,345 pairs, +0.6 ± 5.8 Elo, LLR −0.06) rather than spend its
  remaining six hours. Its arm, the fit over a stop at the target with the
  factors, will not be what lands whatever these reads say, and its STC
  result is already bounded. The maintainer decides.
- **Reckless's shape:** not now. Every read sits on the time rule, the
  late-reserve axis read neutral (RAR-R17), and a distribution change has
  no instrument short of games; a donor's shape is not evidence.
- **A respending budget:** not needed at twice the target, where the stop
  saves no time at `3+0.03`; at long controls the time a cascade stop saves
  is in the remaining clock the budget already reads.

## Results (played 2026-10-23, maintainer-run, the probe too)

Every run was on an idle host (0.0–2.0% busy, no waiver), with 0 faults, 0
time losses and a recount that agreed. The runner's final progress line
gives each read's Elo. The manifests (`589a09fa…`, `47f6fbcb…`,
`68f92fe6…`), the per-arm readings beside this entry
(`rar-r18-stopoff.txt` `58d313ab…`, `rar-r18-cascade.txt` `11f87972…`,
`rar-r18-factorsoff.txt` `204a58d4…`) and the probe (`tm_rec_probe_d18.txt`,
`80f2428b…`) are the evidence.

| Read | Elo, arm A less arm B | nElo | Pentanomial | Time a game, A / B | Plies 0–19, A / B | Clock left in wins, A / B |
|---|---|---|---|---|---|---|
| `stopoff` | **+15.5 ± 9.3** | +25.3 ± 15.2 | 25, 212, 459, 257, 47 | 4.21 / 4.09 s (+2.9%) | 110.0 / 97.1 ms (+13.3%) | 730 / 1,010 ms |
| `cascade` (2×) | **+12.2 ± 8.8** | +21.1 ± 15.2 | 17, 217, 477, 257, 32 | 4.18 / 4.08 s (+2.5%) | 108.2 / 98.2 ms (+10.2%) | 790 / 983 ms |
| `factorsoff` | **−6.9 ± 8.7** | −12.2 ± 15.2 | 30, 238, 493, 220, 19 | 4.17 / 4.18 s (−0.2%) | 100.6 / 111.3 ms (−9.6%) | 738 / 716 ms |

**Probe, the 2× arm: PASS.** No rec move at the maximum; rec2 in 5 ms
(mate 6, depth 10) at both clocks; rec1 4.78 s (1.02 × optimum) at
`60000+600` and 18.9 s (1.24 ×) at `180000+2000`; rec3 1.96 s (0.44 ×)
and 15.1 s (1.06 ×). At the target the same four took 4.80, 9.81, 1.97 and
15.1 s. The head took 32.3, 104.4, 10.4 and 10.4 s (RAR-R13).

**Against the frozen rules.**
- `stopoff`: time rule +6.0 for +2.9%; decision effect **+9.5** (1.8 SE of
  5.1), under 10, so "time alone" by the rule's letter, at its edge.
- `cascade`: rule +5.1; decision effect **+7.1** (1.5 SE); arm A's time
  within ±3% of the seeds' (+2.5%).
- `factorsoff`: rule −0.5; decision effect **−6.4** (1.3 SE); the case in
  which the factors buy more than their time (−10 or less) did not occur.
- **Dispositions, as frozen:** the stop moves to **twice the target**; the
  stability factor and the cross-move slope are **removed** on this
  evidence; the kept set (the horizon, the mate stop, the guard, the stop at
  2×) goes to one repair read at `3+0.03` and `10+0.1`.

**Calibration.**
1. `stopoff`: the Elo held (+15.5 against +16 ± 9) and the decision effect
   stayed within ±10 (0.65); **the time missed** (+2.9% against +8%, band
   +4 to +14).
2. `cascade`: the Elo held (+12.2 against +14 ± 9); the time missed (+2.5%
   against +7%, band +3 to +13).
3. `factorsoff`: the time missed badly (−0.2% against −10%, band −15 to
   −5) and the Elo missed (−6.9 against −20 ± 9); the decision effect
   stayed within ±10 (0.6); the factors' −10 case (0.15) did not occur.
4. Probe PASS (0.85) and rec1 and rec3 under 2.5 × optimum (0.7): held.
5. No fault (0.9): held.
- **The miss is in the instrument.** The registration read time over the
  whole game, and between two arms of one clock the whole-game time is
  nearly conserved: what an arm saves early it holds and spends later, and
  only the remainder at the end differs. `stopoff` spends 13% more in plies
  0–19 and 3% more over the game; `factorsoff` spends 10% less in plies
  0–19 and the same over the game. The 2-Elo-per-1% rule was measured on
  uniform speed changes; applied to a redistribution it reads near zero
  and attributes the whole read to decisions. Its "time alone" verdicts
  here therefore say little; what the reads say directly is below.
- The seeds-against-head read is not a redistribution: there the cluster
  leaves clock unused against the head (1.0 s in wins against 0.33), which
  is the mate stop's and the horizon's nature, so that −7.4% is real.

**What the reads say, read directly.**
- **The stop at the target costs about 15 Elo at `3+0.03`** (+15.5 ± 9.3
  for removing it). It ends ordinary fail-highs, and the time it saves is
  left on the clock: the seeds end wins with 1,010 ms and draws with 934,
  without the stop 730 and 786. At twice the target the cost is gone within
  the read's resolution (+12.2 ± 8.8 recovered; off against 2× is +3.3 ±
  12.8) and the long-control cascades still end: rec1 at `180000+2000` in
  18.9 s against the head's 104 s.
- **The factors front-load at no whole-game cost** (+10.6% in plies 0–19,
  the same time a game) and read **+6.9 ± 8.7** for keeping them: positive
  in sign at 1.5 SE, in the direction the fit moved (`TmStabSlope` −1.1
  steps, less decay) and the direction both donors carry. Insufficient
  evidence by the frozen rule, not a refutation.
- **Arithmetic for the kept set against the 2.6.0 head**, additive over the
  reads: the other pieces (the horizon, the mate stop, the guard, the
  effort share) come to −6.3 ± 15.5; the kept set with the stop at 2× and
  the factors removed to **−9.6 ± 15.2**, with them kept **−2.7 ± 12.4**.
  A 2,000-game repair read at the −13 nElo harm rule passes a true −9.6
  with probability about 0.3 and a true −2.7 with about 0.85.

**What it directs.**
- The stop to twice the target: settled by the frozen rule, and by the
  reads directly.
- RAR-R16's SPRT: its arm carries the stop at the target; recorded
  unresolved at 2,345 pairs on the maintainer's word.
- **The factors' place in the repair arm is a maintainer decision.** The
  research's verdict under its frozen rule is "removed": insufficient
  evidence that they decide better than their time. The reads' direct
  reading is +6.9 ± 8.7, and a kept set without them is predicted to fail
  its own harm read more often than not. The repair read's harm rule at
  both controls is the test either way. Retry trigger if removed: the next
  clock fit, when the search changes (F.6), re-reads a stability discount
  seeded from the donors.
