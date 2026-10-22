# D.1.2 — time management: what the 2.6.0 head does with its clock, what the donors do, and what the clock cluster should be

Research for PLAN D.1.2 (the audit, the bound model, the won-ending stalls
and the forfeit margin), recorded as RAR-R14. No games were played; every
number below is read from games already in `tools/results/` or computed from
the formulas. Instruments and raw readings:
`analysis/artifacts/d12-tm-research-2026-10-22/` (ignored storage):
`tm_profile.py` and `tm_profile.txt` (the head's clock use in six runs),
`tm_formula_compare.py` and `tm_formula_compare.txt` (the three budget
formulas side by side), `tm_first_iteration_probe.py` and its `.txt`.

**Decision: `READY_FOR_IMPLEMENTATION`** for one clock-only cluster (PLAN
D.1.3), fitted once (D.1.4) and gated once (D.1.5). The cluster keeps
Rarog's Stockfish-shaped budget and adds four bounded mechanisms the
evidence supports: a sub-second horizon contraction, a stability factor, a
proved-mate stop and a root fail-high stop past the soft target; the effort
term's band becomes a coordinate so the fit decides between the two donors'
shapes. The forfeit reserve stays. `bench 13` is untouched by construction:
everything lives behind a clock.

## Decision needed

PLAN D.1 asks three things: tick the ADR-0065 checklist against Rarog's
clock; decide the bound model (may a clock search end inside an iteration
after a fail-high, or on a proved mate); size the forfeit margin against the
OS holds the gates recorded. The maintainer's question on 2026-10-22 adds
the fourth: compare the clock with Reckless and current Stockfish and say
whether to improve ours or move to a donor's shape.

## Known evidence

### The code (engine source `9fb525e`, `src/search/time.rs`, `src/search/mod.rs`)

- Budget: Stockfish's 2024 sudden-death shape transcribed in 2.3.0: horizon
  50 moves at every clock, `timeLeft = time + inc·49 − overhead·52`,
  `optScale` from `log10(timeLeft)` and the game ply, `maxScale` up to 6.873,
  `maximum = min(0.8097·time − overhead, maxScale·optimum)`, then Rarog's
  own reserve `maximum ≤ time − 2·overhead` (−30 ms more above one thread).
  Movetime spends its whole budget; a DTZ-ranked root plays `maximum =
  optimum` (B.5.2). The clock starts when `go` is parsed (RAR-R11).
- Soft stop, between completed exact iterations only:
  `optimum × TmOptScale × falling_eval(0.572–1.708) × instability(1.10 +
  2.29·n, n halved per iteration) × effort(0.924 → 0.71 as the best move's
  share of the iteration rises from 0.79 to 1.0)`, capped at the maximum.
  The seven coordinates are 2.3.0's and were never fitted on the B or C
  search (`analysis/b5_research_2026-09-30.md`, the inventory; C.10 kept
  them as clock-only coordinates a fixed-depth tune cannot read).
- Hard stop: polled every 2,048 nodes against the maximum; a started
  iteration always completes (RAR-R13).
- Threads: each thread's own best-move changes feed its own target
  (RAR-R05 rejected the pool's); the soft stop is a latched majority vote,
  so helpers can extend the main thread, never past the maximum.
- No ponder credit, no stability-over-iterations term, no mate stop, no
  in-iteration stop, no low-clock horizon rule.

### The head in games (`tm_profile.txt`; the C head, which is the 2.6.0 release)

| Run (ledger) | Control | Games | Spent / optimum, median · p90 · p99 | Past the optimum, share of head time | Moves ≥ 3× optimum | Clock floor, median | Games touching < 100 ms |
|---|---|---:|---|---:|---:|---:|---:|
| `c11-gauntlet` (RAR-M73) | `3+0.03` 1T | 2,400 | 0.82 · 2.06 · 4.14 | 27.9% | 3.4% | 273 ms | 525 (22%) |
| `pool2-rr` (RAR-M75) | `3+0.03` 1T | 2,400 | 0.85 · 2.26 · 4.53 | 31.1% | 4.3% | 120 ms | 992 (41%) |
| `rel260-stc` (RAR-M76) | `3+0.03` 1T | 2,000 | 0.83 · 2.04 · 4.06 | 27.0% | 3.2% | 248 ms | 457 (23%) |
| `rel260-ltc` (RAR-M76) | `10+0.1` 1T | 1,000 | 0.78 · 1.85 · 3.77 | 20.9% | 3.6% | 455 ms | 0 (203 under 200 ms) |
| `rel260-4t` (RAR-M76) | `3+0.03` 4T | 1,000 | 0.88 · 1.94 · 3.72 | 25.3% | 2.4% | 142 ms | 24 (571 under 200 ms) |
| `c11-gauntlet-4t` (RAR-M74) | `3+0.03` 4T | 2,400 | 0.91 · 2.06 · 3.85 | 28.2% | 2.9% | 154 ms | 111 (1,318 under 200 ms) |

The clock is rebuilt per move from the harness-charged `t=`; the optimum and
maximum are the transcription of `time.rs` at that clock, ply and thread
count. A clock floor is the lowest clock a side holds after one of its own
moves.

**Shape.** At `3+0.03` the head spends 44–47% of its game time in the first
forty plies after the book and reaches the last fifth of the game with
about 200 ms. In `pool2-rr` 21% of its moves (33,544 of 158,737) are played
with under 200 ms on the clock and 4.7% with under 100 ms. The engines in
that tournament split into two policies: Stockfish 11 never goes under
200 ms (floor p1 200, median 640), Dragon 1 never under 790 and Komodo 13.3
never under 650 (both spend only 3.4–3.6 s of a game's 4.5–4.6 s and leave
1.2–1.6 s unused at the end whatever the result); Houdini 3 and 6.03 go as
low as the head (floor p1 58–66 ms). The 2.5.0 release's floors are lower
still (median 93 ms in `rel260-stc`, 54% of games under 100 ms).

**Why the floor is where it is.** The budget assumes fifty moves to go at
every clock, so at 500 ms it still counts 1,470 ms of future increments
and plans an optimum of 60 ms; a game whose moves spend one optimum each
drifts to the clock where the optimum equals the increment, 155 ms for this
formula (`tm_formula_compare.txt`, part D). Current Stockfish contracts the
horizon below one second (`mtg = time_ms / 20`); with only that change the
same drift settles at 400 ms, and at Stockfish's own constants at 520 ms.
Reckless, whose bounds are linear in the main clock less a 100 ms overhead,
settles at 510 ms. At `10+0.1` the floor is a non-issue (515 ms either way).

**Holds.** `h=` is the harness's charge beyond the engine's own reported
time: the OS hold a move suffered outside the engine's clock. At one thread
it is 1 ms at the median and the 99.9th percentile is 87–112 ms; holds of
80 ms or more hit 0.11–0.15% of moves at `3+0.03` and 0.77% in the daytime
`rel260-ltc` run (its 99.9th percentile 509 ms, maximum 2.1 s). At four
threads the median is 3 ms and the 90th percentile 21–25 ms, because the
move is printed only once every helper has stopped: 0.4% of moves wait
80 ms or more. Stockfish 11, Dragon and Komodo show `h=` of 0–4 ms
throughout, Critter and Basilisk holds like the head's: the holds are the
host's, not the engine's.

**Forfeits.** In every run since 2026-10-10 with the 20 ms harness margin:
3 time losses in 219,180 games at `3+0.03` one thread (RAR-E29 round 3068,
RAR-E32 rounds 1777 and 1855), 1 in 6,000 at `10+0.1` (RAR-E42 round 875),
0 in 1,000 at four threads; 0 in the 15,600 gauntlet games charged the
desktop's 2,000 ms margin. Each of the four came on a short search (2–81 ms
of engine time) whose process the OS held for 69–129 ms with 70–163 ms on
the clock, in a game already lost on the board. The minimum reserve
(`2·overhead` = 20 ms at one thread) cannot cover a hold longer than the
clock; only not holding such a clock can. At this rate the forfeits are
worth under 0.01 Elo here; their cost is robustness on another harness.

**Won-ending stalls (RAR-R13 re-read on the C head).** Won-position moves at
≥ 0.9 × the maximum: 0.30–0.44% at `3+0.03`, 1.44% at `10+0.1`; mate-score
moves there: 0.24–0.53% and 3.40% (154 moves in 1,000 games, 71 with rec2's
deep-mate signature). Every one was in a game the head won, except three
`c11-gauntlet` draws. Games not won after three consecutive won own moves:
14 (gauntlet), 20 (pool: 18 draws, 2 losses against the stronger field), 7,
3, 4, 18; of those with a won-position move at ≥ 3 × the optimum: 8, 4, 1,
0, 1, 6. The reading of RAR-R13 stands: real, growing with the control, no
measured cost at these controls.

**First iteration.** `go wtime 1 btime 1` on six positions, sixty searches:
every move played came from a completed iteration of depth 4–6, because the
first clock poll is at node 2,048 (`tm_first_iteration_probe.txt`). The
fallback to `legal_moves[0]` is unreached in practice and unguarded in code.

### The donors

- **Reckless 0.10.0-dev** (`D:/code/Reckless`, head of 2026-09-09; `src/time.rs`
  last changed 2026-06-18, the multipliers from its 200k-game SMP SPSA of
  2026-06-06): soft `= (0.0594 − 0.0492·e^(−0.0386·fullmove)) × (main −
  100 ms) + 0.75·inc − 15 ms`, hard `= 0.7281 × (main − 100 ms) + 0.75·inc −
  15 ms`; the soft bound is scaled between iterations by `node fraction
  (3.1838 − 2.6554·f, floor 0.546; f = the best move's share of all nodes of
  the search) × pv stability (1.2881 − 0.0440·k, floor 0.716) × eval
  stability (1.2664 − 0.0416·k, floor 0.864) × score trend (0.7426 +
  0.0480·(previous move's score − score), 0.721–1.403) × best-move changes
  (1.15 + ln(1 + 0.2526·n))`. The hard bound is never checked before depth
  one completes. No mate stop in clock mode; a root fail-high re-search is
  one ply shallower. Threads vote the soft stop at 65% and may withdraw a
  vote.
- **Stockfish master 2026-09-30** (`49ea5ded`; the `sf_19` tag of 2026-09-05
  carries the same lines): the budget shape Rarog transcribed plus three
  additions since: `originalTimeAdjust = 0.3272·log10(timeLeft) − 0.4141`
  fixed from the first move's `timeLeft` (0.763 at `3+0.03`, 0.946 at
  `10+0.1`, 1.205 at `60+0.6`, 1.534 at 40/15), the sub-second horizon
  contraction `mtg = time_ms / 20`, and (2026-09-09) `optScale × (1 + 0.9 ×
  min(timeAdvantage, 0))` when behind on the clock. Between iterations:
  `fallingEval (0.576–1.728, from the previous average and the score four
  iterations back) × reduction ((1.468 + previous move's) / (2.284 ×
  timeReduction), timeReduction 0.629–1.544 from the iterations since the
  best move last changed) × instability (1.077 + 2.229·n / threads) ×
  effort (0.838 down to 0.714 only once the best move holds 90–105% of all
  nodes)`; `increaseDepth = false` once half the target is spent, which
  makes the following iterations shallower; a completed iteration stops the
  search at `mate_in(3)` or `mated_in(2)`; a single legal move is capped at
  500 ms; ponder adds a quarter of the optimum. The main thread alone
  decides.
- **Manta ADR-0065** (the checklist): distinct optimum and immutable maximum;
  maximum rooted at `go` receipt; ponder credit consumed once at 75%;
  optimum adjusted only by completed exact root iterations through
  stability, best-move change, score trend and effort concentration as
  bounded factors; stable concentrated effort spends less; no preferred
  stop below completed depth four; helpers contribute at most a normalised
  best-move-change count.

### The checklist, ticked

| ADR-0065 item | Rarog | Note |
|---|---|---|
| Distinct optimum and immutable maximum | present | the maximum is only ever lowered (DTZ root) |
| Maximum rooted at `go` receipt, never extended | present | RAR-R11; the ponder path suspends the hard stop and keeps the origin |
| Ponder credit once, discounted | absent | no credit at all; ponder is off in CCRL and in Colosseum |
| Optimum adjusted only by completed exact iterations | present | the four factors above, between iterations |
| …from stability | **absent** | both donors carry an iterations-since-change discount (×0.65 Stockfish, ×0.72 × 0.86 Reckless); Rarog has only the halving change count |
| …best-move change, score trend | present | instability, falling_eval |
| …effort concentration, bounded | present, Stockfish's shape | a cut only when the best move holds ≥ 79% of the iteration; Stockfish's fires at ≥ 90% of the search; Reckless's is a continuous ×0.55–2.5 |
| Easy root spends less | present in form | inert on 91% of bench iterations (RAR-S47), as Stockfish's is by design |
| No preferred stop below a minimum depth | absent | no soft guard (neither donor has one); the hard stop has no completed-depth guard (Reckless has) |
| Helpers: normalised change count only | different | own-thread signals, a majority vote that can extend the main thread; D.2 owns it |

## Unknowns

- The Elo of the spend shape itself: the front-loaded profile is shared with
  Stockfish 11 and Houdini 6; the strongest engines in the pool spread it
  and under-spend, but their strength is not the clock's. Only games read
  it.
- Whether the multipliers fitted for the 2.3 search are near a fit for this
  one: no zero-game instrument reads a clock-only coordinate.
- The sign of the sub-second contraction at `3+0.03`, where 21% of the
  moves are affected and the gain of never flagging is near zero here.

## Hypotheses

- **H1 (leading):** the clock leaves Elo in three places — multipliers never
  fitted on this search, no stability discount, and cascades that run an
  iteration to four times its target — worth a few Elo together at STC and
  more at longer controls; a fitted cluster with the four mechanisms reads
  positive under `[0,3]`.
- **H2:** the formula's time shape is near-optimal for this search and the
  fit moves little; the mechanisms redistribute time without changing
  strength (0 to +2), and the cluster ends at the gate's cap.
- **H3:** the sub-second contraction costs more at `3+0.03` than the stalls
  and the stability term gain, because late-game moves lose budget on a
  control where the increment is a third of the median move; the cluster
  reads negative while its parts would not.
- **H4 (Reckless's shape):** a continuous node-fraction term that *extends*
  time on dispersed effort beats the Stockfish-style cut. Decided by the
  fit, not by a port: the effort band's floor and endpoints are
  coordinates with ranges that reach both shapes.

## Interaction map

- Producers: the root loop's completed-iteration events (score, best move,
  effort share, iterations since the best move changed), `go`'s clocks and
  ply, the thread count.
- Stored state: `RuntimeLimits` (optimum, maximum, the horizon), the soft
  target's running inputs, the stop-vote latch.
- Consumers: the between-iteration stop, the in-tree hard poll, the
  tablebase PV extension's time box (`tb_extension_latest_start_ms` reads
  the hard ceiling and is unchanged), the single-legal-move shortcut, the
  parallel vote.
- Shared signals: the aspiration loop sets the iteration boundaries the
  soft stop reads; a root fail-high stop reads the window's bound (RAR-S89,
  RAR-S17: the loop itself is not changed). B.5's retry trigger (mid-
  iteration stops above 10%) is not reopened: the fail-high stop ends at a
  window boundary with a move whose bound exceeds the previous best. A
  proved-mate stop interacts with the tablebase root (a DTZ root already
  caps the maximum at the optimum) and with the ply cap (rec2's growth is a
  search property, out of scope). The horizon contraction changes the
  maximum too, through `maxScale × optimum`, so late-game hard limits
  fall with it.
- Threads: every mechanism acts in the main thread's target; helpers keep
  their vote. The 4T direction read in D.1.5 reads the interaction; D.2
  owns the vote.

## Cheapest discriminating tests

1. **Done, zero games:** the game profile above (the floor, the holds, the
   forfeit arithmetic, the stall shares); the formula comparison (the
   horizon is the floor's cause). Result: the margin question is a horizon
   question; the stalls are real and cost nothing measurable; the shape
   question has no zero-game answer.
2. **Wire check before the fit (D.1.4, cheap):** `TmOptScale` at an absurd
   value (0.3) in a 200-game match must cut the head's time per game by
   about 60% in `tm_profile.py`'s reading; the same for each new
   coordinate at its range ends (the horizon's, the stability floor's).
   Proves the harness reports the clock's behaviour, not that the engine
   responds.
3. **Activation of the stability and effort signals (D.1.3's instrumentation):**
   per-iteration log of the iterations-since-change count, the effort
   share and each factor on 300 book positions at `3000+30` (B.5's
   replay); a signal that sits at one end on more than 90% of iterations
   is reported before the fit, and its coordinate is dropped from the
   tune (RAR-S47's lesson: an inert factor tunes as a constant).
4. **The fit (D.1.4) and the gate (D.1.5):** the only readers of the Elo
   question.

## Prospective prediction (frozen here; re-frozen with the fitted values at D.1.5's registration)

- Gate, `3+0.03` one thread, `[0,3]` nElo, cap 20,000 pairs: **+3 ± 4 Elo**;
  H1 0.45, cap 0.40, H0 0.15. At RAR-M10's drift a true +4 nElo accepts in
  about 47,000 games, beyond the cap: the cap is the likely outcome unless
  the cluster is worth +5 or more.
- `10+0.1` direction read, 1,000 games: the same sign as STC with a larger
  point estimate (the stalls and the iteration growth are worth more there).
- 4T direction read, 1,000 games: within ±10 Elo of zero; a loss beyond that
  names the vote's interaction for D.2.
- Probe (`tm_rec_probe.py`, both clocks): no rec move at the hard maximum;
  rec2 under 2 s; rec1 and rec3 within about 1.5 × their soft target.
- `tm_profile.py` on the gate's games: the candidate's clock floor median
  above 350 ms at `3+0.03` and games touching 100 ms under 3%; won and mate
  shares at ≥ 0.9 × the maximum below 0.2%; forfeits 0 in the candidate arm
  against the head arm's rate (1 to 3 expected in 40,000 games).
- Most likely failure: H3, the contraction's late-game cost at `3+0.03`
  outweighing the rest, read as a negative STC estimate with a positive
  LTC one.

## Falsifiers and stop conditions

- A wire check that does not move time use stops the fit (instrument).
- A stability or effort signal inert on more than 90% of iterations drops
  its coordinate (activation).
- The fit's movement stop rule (PLAN rule 7c): a first block that moves no
  coordinate a full step ends the fit; no second block.
- The gate: H0 returns the cluster to research with the point estimate; the
  cap bakes nothing (RAR-S89's rule) and records "unresolved"; the fallback
  below is the only registered follow-up.
- The probe or the profile reading failing their predictions refutes the
  stall mechanism's repair as implemented, independent of Elo.

## Decision

**`READY_FOR_IMPLEMENTATION`.** The cluster:

1. **Horizon contraction below one second:** `mtg = min(50, ⌊clock_s ×
   TmHorizonPerSec⌋)`, seed 50 (Stockfish's `time_ms / 20`), range 20–100,
   `mtg ≥ 1`. Owner of D.1.2.2: the reserve stays at `2·overhead` (+30 ms
   above one thread) and the harness margin at 20 ms, with the arithmetic
   above recorded as the reason.
2. **Stability factor:** `max(TmStabFloor, TmStabBase − TmStabSlope × k)`,
   `k` the completed iterations since the best move last changed; seeds
   1.2881 / 0.0440 / 0.716 (Reckless's; Stockfish's timeReduction is the
   same mechanism in another parametrisation).
3. **Effort band as coordinates:** the factor interpolates from
   `TmEffortHigh` to `TmEffortLow` as the best move's node share rises from
   `TmEffortFloor` to 1.0, the share taken over the whole search so far (both
   donors) instead of the iteration; seeds 0.924 / 0.71 / 0.79, ranges
   0.6–2.0 / 0.4–1.0 / 0.0–0.95, so the fit can reach Reckless's extending
   shape or keep Stockfish's cut.
4. **Proved-mate stop:** a clock search (not analysis, ponder, movetime,
   depth, nodes or `searchmoves`) ends after a completed iteration whose
   root score is a mate for the side to move with the same best move and
   distance for three consecutive completed iterations, or whose score is a
   loss in at most two plies (nothing to find). Owner of rec2.
5. **Root fail-high stop past the soft target:** when a root window fails
   high (`score ≥ beta`) with the elapsed time at or past the current soft
   target, the search ends and plays the move that failed high, reporting
   its lower bound; its bound exceeds the previous best's exact score, so
   no best move is discarded. Owner of rec1 and rec3.
6. **Contract guard:** no hard stop before the first iteration completes
   (Reckless's `completed_depth == 0`), with a test; no behaviour change
   measured, a guard on the unreached fallback.

Not in the cluster, with the trigger that reopens each: Stockfish's
`originalTimeAdjust` (a control-dependent scale fitted to Stockfish's
search; trigger: D.1.5's LTC read opposite in sign to STC), the
time-disadvantage scale (no evidence here; trigger: a game record showing
losses on the clock against an opponent ahead on time), ponder credit
(trigger: a maintainer decision to play ponder-on events), eval stability
(Reckless only; `falling_eval` already reads the trend), Stockfish's
`increaseDepth` (the fail-high stop addresses the measured cases; trigger:
`tm_profile.py` on the gate games still showing ≥ 3% of moves at ≥ 3 × the
optimum), the aspiration loop (RAR-S89, RAR-S17), the thread vote (D.2).

**Fallback, registered now:** if D.1.5's SPRT reaches its cap or H0, the
robustness pieces alone — mechanisms 1, 4, 5 and 6 at the unfitted seeds,
without 2, 3 or the fit — are offered to the maintainer under AGENTS'
repair case 2 (a pre-registered 2,000-game read with a harm rule at
−13 nElo), since each is expected near zero in Elo and bought for
correctness; the maintainer decides whether to run it.

## Implementation handoff (D.1.3)

- **Goal:** the six mechanisms above in `src/search/time.rs` and the root
  loop of `src/search/mod.rs`, clock-only, with the new coordinates in
  `SearchParams` (`TmHorizonPerSec`, `TmStabBase`, `TmStabSlope`,
  `TmStabFloor`, `TmEffortFloor`) beside the seven existing ones.
- **Semantics fixed above;** the effort share's accumulator is the best
  move's nodes over the search so far, kept per thread; the stability
  count resets to zero when the completed best move differs from the
  previous completed one.
- **Invariants:** `bench 13` unchanged (13,987,223 / EBF 2.565); `bestmove`
  is the move the last `info` line names; a fail-high stop's line carries
  `lowerbound`; a mate stop plays the mate line's move; the maximum is never
  raised; the DTZ root's `maximum = optimum` and the tablebase extension's
  box are unchanged; nothing per node changes (the hard poll stays at 2,048
  nodes; no allocation, lock or `Arc`).
- **Instrumentation:** `diag` counters for the mate stop, the fail-high
  stop and the first-iteration guard; the per-iteration log of test 3,
  behind `diag`, printed by the B.5 replay driver.
- **Deterministic tests:** the first-iteration guard (a budget expired
  before the search starts still returns a depth-1 move); the mate stop on
  rec2-like positions at a fixed node budget (the stop fires after three
  stable mate iterations and not under `go infinite`); the fail-high stop
  on a constructed window (fires only past the soft target, plays the
  fail-high move); the horizon (`compute_runtime_limits` at 500 ms gives
  the contracted optimum, at 1,000 ms the old one); a late-`issued`
  injected delay at a 150 ms clock leaving the reserve intact (D.1.2.2's
  test); SMP inertness at one thread.
- **Cheap qualification:** fingerprint; the timed probe on an idle host
  (maintainer-run, about eight minutes); the wire check (200 games, about
  two minutes).
- **Maintainer-owned:** the fit and the gate (D.1.4, D.1.5; hours below).
- **Non-goals:** the aspiration loop, the vote, ponder, nodestime,
  movestogo's branch, the harness margin.

## Costs (maintainer's hours, at the measured rates)

| Job | Games | Rate | Hours |
|---|---:|---:|---:|
| SPSA block, `3+0.03`, 15 slots | 60,000 | about 6,000/h | 10 |
| SPRT `[0,3]` to the cap | 40,000 | 5,700/h | 7 (a true +6 nElo accepts near 26,000 games, 4.6 h) |
| `10+0.1` direction read | 1,000 | 1,660/h | 0.6 |
| 4T direction read | 1,000 | 1,210/h | 0.8 |
| Timed probe, idle host | — | — | 0.15 |

**Evidence layer.** Counts, ratios and formula values; none is Elo.
