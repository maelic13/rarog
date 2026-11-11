# RAR-S120 — D.6.6's screen: what generating captures after the TT move could save, and how often it would reorder them

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-11, before any of its unexposed counts
  or cycles was read. The read is the agent's: no games, and short timed runs
  on an idle host, checked first.
- **Kind:** a zero-game readiness screen for a speed change that also
  changes play.
- Disposition: no-change — read 2026-11-11: lazy capture generation could save at most 0.82% of
  search cycles game-like (0.93% on the bench), under the frozen 1%; it would reorder the top
  capture at 0.11% of nodes. D.6.6 closes `NO_CHANGE`.
- **Baseline:** `d6` at `f091bc2`, engine behaviour of `ee0b3e4`, bench
  13,038,236 / EBF 2.540.

## The question

`fill_noisy` generates and scores a node's captures before the TT move is
searched. When the TT move cuts, that work is never used. The donor generates
captures only after the TT move fails to cut (Reckless `aec5cb2`,
`movepick.rs`). C.12's speed review put such nodes near 1.3M of 3.67M loop
nodes and called this the one lead worth several percent. It was never
measured.

## The four questions (AGENTS)

- **Mechanism.** A node whose TT move cuts skips capture generation and
  scoring.
- **Interactions.**
  - Where the TT move fails, the captures are scored after its subtree.
    The noisy history the scores read has moved by then, so the capture
    order can change. That makes this a playing change, and the reason for
    a harm read.
  - The pinned set that quiet generation reuses would be computed at the
    same point. It is unchanged.
  - Nothing else reads the capture list before the first move.
  - At an exclusion node the TT move is skipped, not searched, so nothing
    changes there.
- **Invariants.** The instrument changes no node count. `bench 13` must read
  13,038,236 with reorder counting off and on; both were seen before
  registration.
- **Falsifier.** If capture generation at nodes that cut before any capture
  is looked at is under 1% of search cycles, nothing is left to buy. A gain
  under 1% in a non-PGO build cannot be told from the speed read's floor once
  the tree also changes.

## Instrument

- **Patch and binary.** `analysis/arm_patches/d66-rar-s120-screen.patch`
  (SHA-256 `e8deadd8…`, adds `src/search/s120.rs` and the `s120` feature) on
  `f091bc2`. Its source tree is `2b5fd7f5…`. It is built with
  `--features s120` only, release, no PGO, as
  `analysis/artifacts/d66-screen-2026-11-11/rarog-s120.exe` (`943bb7ac…`).
- **What it records,** at every non-root node, with the time-stamp counter:
  - `fill_noisy`'s cycles;
  - those same cycles, booked as unused when the node cuts with no capture
    yet looked at;
  - the search's own cycles, from `go` to the dump;
  - a 256-pair calibration of the counter's own cost, the minimum, which is
    subtracted once per fill and twice per fill from the total.
- **The reorder count,** only with `RAROG_S120_ORDER=1`, which adds work and
  is never timed. At the first look at the captures after a TT move was
  handed over, it rescores them as the lazy form would. It counts whether
  any score changed, and whether the top capture changed among nodes with
  two or more.
- **Script.** `analysis/artifacts/d66-screen-2026-11-11/read_s120.py`
  (SHA-256 `3cf8478f…`, frozen with this registration).
- **Runs,** in this order, each started on an idle host and with nothing
  else running:
  1. `bench`: three timed runs of `bench 13`.
  2. `replay time`: RAR-S117's 60 games, the baseline's moves, `go nodes` at
     the recorded counts, one worker, timed.
  3. `replay order`: the same games, six workers, counts only.

## Wire

- Every timed bench run reads 13,038,236 and dumps 40 times.
- Both replays equal RAR-S117's arm 0 in every search's move, depth and
  nodes at each depth.
- Every search dumps once.

## Exposure, stated

Seen: the two bench node counts and that `bench 13` dumps 40 times.
Unexposed: every cycle and every reorder count.

## Decision (frozen)

S is the unused cycles over the search cycles, both corrected for the
counter's cost, on the timed replay.
- **Void** if a wire fails, or if S on the three bench runs spreads over
  0.5 points.
- **`NO_CHANGE`** if S is under 1%.
- **`READY_FOR_IMPLEMENTATION`** (as `I2`) if S is 1% or more.
  - S is an upper bound: the lazy form adds a stage step and saves nothing
    where the TT move fails. The implementation therefore freezes its
    prediction (pooled-PGO NPS +x%, with x at most S) before its speed read.
  - The speed read is the maintainer's: pooled PGO on an idle host.
  - Then comes case 2's 2,000-game harm read, as PLAN names it. Its
    registration states this read's reorder rate as the playing change's
    exposure.

**Reported, not deciding:** S on the bench runs; the fill share of search
cycles; the share of fills booked unused; cycles per fill; the reorder rates.

## PRE-REGISTERED PREDICTION (frozen before exposure)

1. The wire passes. 0.9.
2. Nodes cutting before any capture are 30% to 50% of fills, game-like.
   0.6.
3. The fill share of search cycles is 3% to 8%. 0.5.
4. S is 1.0% to 3.0% on the replay. 0.5. S under 1%: 0.35.
5. Bench S lies within 1 point of the replay's. 0.6.
6. Any capture score changes at 30% to 70% of first looks after a TT move.
   0.5. The top capture changes at 3% to 15% of those with two or more.
   0.5.
7. `READY_FOR_IMPLEMENTATION`. 0.6.
8. **Most likely failure mode:** the counter's own cost is of the same order
   as a cheap fill. The correction then decides the answer near 1%, so S is
   read with its raw value beside it.

## Result: the read (2026-11-11)

Read by `read_s120.py` (SHA-256 `3cf8478f…`, unchanged since registration) on
`rarog-s120.exe` (`943bb7ac…`). The host was idle before the runs: CPU at 0%
to 7%, with no engine, harness or build process. Logs and data are beside the
script: `run.log`, `report.log`, `bench_time_{0,1,2}.log`, `bench_time.json`,
`replay_time.json`, `replay_order.json` and `overhead.log`.

**Wire: passed.**
- The three timed bench runs read 13,038,236 and dumped 40 times each.
- Both replays equal RAR-S117's arm 0 in all 3,670 searches.
- Every search dumped once.

| | bench 13, three runs | game-like replay, one worker |
|---|---:|---:|
| **S, cycles lazy generation could save** | **0.921%, 0.934%, 0.936%** | **0.820%** (raw 0.825%) |
| capture generation and scoring, of search cycles | 4.39% to 4.46% | 3.95% |
| fills at nodes that cut before any capture | 21.10% | 22.04% of 189.5M |
| counter ticks per fill | 171 | 167 |
| the counter's own pair cost | 1 tick | 1 tick |

| reorder count, game-like replay | |
|---|---:|
| first looks at the captures after a TT move | 31.7M |
| any capture's score changed by the TT move's subtree | 2.70% |
| the top capture changed, of 18.4M with two or more | 0.11% |

**Decision, by the frozen rule: `NO_CHANGE`.** S is 0.820% on the replay,
under 1%. The bench runs agree to 0.015 points, well inside the 0.5 limit.

**Checked after the read, not deciding.** The instrument does not deflate S
by slowing the search: alternating `bench 13` runs of a build without
`s120` and the screen build read 2.593M, 2.605M, 2.595M nps against
2.598M, 2.612M, 2.621M (`overhead.log`). Its cost is inside the run-to-run
noise, so the denominator is honest.

**What the data say.**
- **C.12's "several percent" was an overestimate.** Generating and scoring
  captures costs about 4% of search cycles in all. Only 22% of the fills are
  never used, so lazy generation can save at most 0.8% to 0.9%, before its
  own stage step. By the measured 2 Elo per 1% NPS at `3+0.03`, that bounds
  the change at under 2 Elo. That is below what the speed read's floor and a
  case-2 read can bank.
- **The playing change is negligible.** The top capture changes at 0.11% of
  the nodes where order could matter. The lazy form would be close to
  behaviour-neutral, but there is too little to buy.

**Retry trigger.** Capture generation or scoring becomes materially dearer,
by a scorer change or the network's staging, or a profile of a later head
(H.4's speed work) puts capture generation at 8% or more of search cycles.
Then re-read S with this instrument, and reopen only at 1% or more.

## Calibration (appended after the read)

1. The wire passes: hit.
2. Nodes cutting before any capture, 30% to 50%: missed in magnitude (22%).
   C.12's 1.3M of 3.67M counted TT-move cuts among loop nodes; the
   denominator here is every fill, including the nodes with no TT move.
3. Fill share 3% to 8%: hit (3.95%).
4. S 1.0% to 3.0%: missed (0.82%). The 0.35 put on S under 1% was the better
   call.
5. Bench S within 1 point of the replay's: hit (0.93% against 0.82%).
6. Any score changed, 30% to 70%: missed in magnitude (2.7%). Top capture
   changed, 3% to 15%: missed (0.11%). The assumption was that the TT
   move's subtree writes the noisy-history entries this node reads. It
   rarely does: its updates are mostly to other piece, square and victim
   keys.
7. `READY_FOR_IMPLEMENTATION` 0.6: missed.
8. The counter's cost: the failure mode did not occur. One tick a pair, with
   raw and corrected S within 0.005 points.
