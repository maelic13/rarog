# D.1.5 — the clock cluster's mechanisms reviewed while its SPRT runs: where the candidate's time goes, what overlaps, and whether Reckless's shape fits

**Correction, 2026-10-23, after RAR-R17's reads.** Section 1 named the
horizon's late-game cut (H3) as the first suspect. **RAR-R17 does not
support it:** the horizon at 100 against 53 reads −0.9 ± 8.0 Elo, with the
same late time a move in both arms, and the 53 arm has 0 games under 100 ms
against 443.

What does read is the cluster at its seeds against the head: **−14.9 ± 8.8
Elo**, with 7.4% less time a move at every stage, the opening included, and
about 0.3–0.7 s of clock left unused at the end of draws and wins. The
likeliest source is the fail-high stop, which removes the head's
in-iteration overruns without spending the time elsewhere; that is not yet
isolated.

Section 1's profile figures stand as measured. Its causal reading is
superseded, and section 3's plan is advanced to its fail-high step. The text
below is the review as first written.

**Status:** a review, written 2026-10-23 during RAR-R16's SPRT, from source
at `bed16d1` and from results already on disk. Nothing was run on the match
host. It changes no registered rule, reading or prediction: the SPRT decides
the gate. Everything here is input to the research D.1 returns to on H0 or
the cap (RAR-R16's dispositions).

**Evidence layer:** time profiles, formula values and counts, not Elo. The
two direction reads (RAR-R16) are the only Elo figures cited, and they
decide nothing.

## 1. Where the candidate's time goes

`tm_profile_d15.py` prints each arm's time a move by ply band
(`profile_rar-r16-4t.txt`, `profile_rar-r16-ltc.txt` in
`analysis/artifacts/d15-clock-gate-2026-10-23/`). The fitted candidate
against the head:

| Plies | 4T `3+0.03`: candidate / head ms a move | clock before | `10+0.1`: candidate / head ms | clock before |
|---|---|---|---|---|
| 0–19 | 117 / 100 | 2,615 / 2,702 | 366 / 318 | 8,888 / 9,091 |
| 20–39 | 109 / 100 | 1,772 / 1,990 | 364 / 339 | 6,165 / 6,777 |
| 40–59 | 82 / 86 | 1,090 / 1,341 | 302 / 301 | 3,751 / 4,521 |
| 60–79 | 56 / 70 | 672 / 847 | 217 / 248 | 1,998 / 2,675 |
| 80–99 | 40 / 54 | 464 / 486 | 145 / 186 | 1,079 / 1,356 |
| 100–119 | 31 / 40 | 376 / 270 | 109 / 127 | 696 / 701 |

- **The candidate is more front-loaded than the head**, which was already
  front-loaded (RAR-R14: 44–47% of the game's time in the first forty
  plies).
  - Plies 0–39 take 51.8% of its time at four threads (head 44.7%) and
    49.1% at `10+0.1` (head 43.8%).
  - Plies 60–119 take 23.7% and 24.9% (head 29.6% and 29.4%).
- **From ply 60 it spends 12–28% less a move.** At `3+0.03`, by ply 100 it
  does so while holding more clock than the head (376 against 270 ms).
- **Two mechanisms produce this:**
  - **The fit's scale** (`TmOptScale` +17%, with `TmStabBase` and
    `TmEffortHigh` up) spends more early. The wire check against the seeds
    reads ×1.21 a move in plies 0–19 and +5% over the whole game.
  - **The horizon** halves the budget in the second half of a `3+0.03`
    game. From the transcription, with the fitted 53 moves a second against
    none:

    | Clock | 1,090 ms | 672 ms | 464 ms | 376 ms | 340 ms |
    |---|---|---|---|---|---|
    | Optimum, with / without the horizon | ×1.00 | ×0.81 | ×0.62 | ×0.52 | ×0.49 |

  - The measured cut (−20 to −28% a move) is smaller than the formula's,
    because near 40 ms a move the time spent stops following the optimum
    (RAR-R15's horizon wire check).
- **This is RAR-R14's H3,** the packet's named "most likely failure": the
  contraction's late-game cost at `3+0.03` outweighing the rest.
  - The horizon was bought for forfeit protection, worth under 0.01 Elo
    here (RAR-R14: four time losses in 226,180 games). On the four-thread
    read it lifts the clock floor's median from 151 to 353 ms and leaves
    0 of 1,000 games under 100 ms, against 39.
  - That reserve is clock the head plays with and the candidate does not.
- **A caution against blaming the fit.** SPSA compares arms against each
  other with the horizon present on both sides. Its preference for more
  early spending is a real head-to-head result between those arms. It does
  not see the horizon's cost, which both arms carry.
  - The fit against the seeds scored even over 200 games (pentanomial 3,
    26, 42, 26, 3; about ±35 Elo).
  - So if the direction reads' lean (about −8 ± 8 Elo over 2,000 games) is
    real, the first suspect is the cluster against the head (the horizon
    above all), not the tune.

## 2. Overlaps and inconsistencies in the mechanisms

None is a coding defect: the implementation does what PLAN D.1.3 and the
review in RAR-R15 say. These are design questions for the research.

1. **Three factors respond to one event, a change of best move.**
   - Stockfish's instability (`1.12 + 2.24·n`, n halved each iteration) is
     about ×3 just after a change.
   - Reckless's pv stability (`1.313 − 0.030·k`) is at its maximum at
     k = 0.
   - The effort share is low for a move that just became best, so the
     effort factor sits at its high end.
   - They multiply. Each donor carries only its own form, calibrated in its
     own budget: Stockfish instability with `timeReduction`, Reckless pv
     stability with a mild `1.15 + ln(1 + 0.25·n)` change term. Rarog has
     Stockfish's change term at full strength **and** Reckless's stability
     on top.
   - The fit did not resolve the overlap: `TmInstabSlope` −0.16 step,
     `TmStabBase` +0.33, `TmEffortHigh` +0.58.
2. **Three near-collinear scales.** `TmOptScale`; `TmEffortHigh`, which is
   the effort factor on 87.5% of iterations; and the falling-eval floor
   0.572, the factor on 66.9%. Each multiplies the whole target on most
   iterations.
   - SPSA spends resolution walking the ridge between them, and a shape
     change that needs two of them to move together (Reckless's extender:
     `TmEffortHigh` up, `TmOptScale` down) has almost no first-order
     signal.
   - The fit is a poor instrument for RAR-R14's H4, the shape question. It
     moved both scale coordinates up together.
3. **Falling eval's slopes are Stockfish's per-unit values applied per
   Rarog centipawn.** Stockfish's internal score unit is not a centipawn,
   so the term's sensitivity differs from the donor's by that unit ratio,
   not measured here. It leaves the clamp only for falls of about 19 cp or
   more (fitted: `(0.572 − 0.1345) / 0.0237`).
   - The cross-move seed (0.0204) carries the same caveat, which RAR-R15
     recorded; the fit left both near their seeds.
4. **`TmStabFloor` never binds.** At the seeds it binds from k = 13, on
   under 0.6% of iterations; at theta (`1.3132 − 0.0302·k`) from k = 21.
   Block 1 moved it −0.22 step, noise.
5. **The fail-high stop is the one mechanism neither donor has.** It ends
   29% of first-move searches (the activation read) on a lower bound.
   - Reckless re-searches one ply shallower instead. Stockfish shallows the
     next iterations once half the target is spent (`increaseDepth`).
   - D.1.2.1 argued it discards no better move, and the probe shows it
     ending rec1 and rec3 under their optimum. Its Elo has never been read
     on its own.
6. **Small edges, recorded and not acted on:**
   - A fail-high stop leaves a lower bound as the next move's cross-move
     reference, so the next fall reads slightly small.
   - A ponder search that is stopped on a miss stores a score for a
     position that did not occur.
   - The mate stop stays off after a `ponderhit` (the flag is set at `go`).
   - Ponder is off in CCRL and Colosseum.
7. **The four-thread read fired a frozen reading.** −10.4 ± 11.6 is past
   the −10 at which RAR-R16 says the vote's interaction is named for D.2.
   The interval includes zero.

## 3. Should D.1 try Reckless's shape if the SPRT is negative?

**Not as a port, and not before the cluster is decomposed.** The reasons,
from the evidence above:

- **What a negative SPRT would most likely mean is a distribution problem:**
  too much time early and too little late, with a reserve held unused.
  Reckless's budget is a different distribution:
  - its fraction of the remaining clock grows with the game, from about
    1.2% at move 1 to about 5.5% at move 60 (`0.0594 −
    0.0492·e^(−0.0386·fullmove)`), so it is not front-loaded;
  - **it holds more reserve at bullet than our candidate:** its clock
    settles near 510 ms at `3+0.03` (RAR-R14's formula comparison), against
    the candidate's median floor of 353 ms at four threads.
  - On the axis that looks costly here, the late reserve, Reckless's shape
    would move the wrong way. On the opening axis it would move the right
    way.
- **Reckless's multipliers were fitted** (a 200,000-game SPSA, 2026-06-06)
  jointly with its own base, its search and its conditions. Transplanted,
  they need a refit like ours, and a donor's shape is not evidence
  (AGENTS).
- **One part has data behind the "our search shape" argument: the
  node-fraction multiplier.** Rarog's search is Reckless-derived, and the
  activation read's best-move shares (median 0.40, p90 0.82) put Reckless's
  continuous factor (`3.18 − 2.66·f`) between about ×2.1 and ×1.0. It would
  be active on every iteration, where ours sits at its high end on 87.5%.
  That is the H4 question, and item 2 says our fit could not answer it.

**The cheapest discriminating path,** to register if the SPRT ends H0 or at
the cap, and before any new code:

1. **The cluster at its seeds against the head:** the PGO builds already on
   disk (`rarog-d15seeds` against `rarog-d15head`), 2,000 games at
   `3+0.03`, about 25 minutes. Does the cluster cost before the fit?
2. **The horizon's late-game cost:** on the thirteen-coordinate tune build
   with theta set, `TmHorizonPerSec` 100 (contraction only below 0.5 s)
   against 53, a 2,000-game categorical read, about 25 minutes. H3 predicts
   the 100 arm stronger at `3+0.03`.
3. **Only if (1) or (2) shows a distribution cost:** a research card on the
   allocation's shape across the game, using the per-ply profile as the
   instrument. RAR-R14 already has profiles of Stockfish 11, Dragon,
   Komodo and the Houdinis; Reckless can be added as a profiled opponent.
   - Then one arm per hypothesis: a flattened Stockfish curve, or
     Reckless's base with its node-fraction factor and our stops, each
     fitted.
   - D.1.2.2's forfeit protection can be confined to where forfeits happen
     (clocks under about 300 ms) instead of every clock under a second.
4. **The fail-high stop's own read**, if (1) shows a cost the horizon does
   not explain. It needs a switch, which is code; only then.

**Not recommended:** a third SPSA block, a wider fit with search
coordinates, or a Reckless port before (1) and (2) have read. None of them
separates the hypotheses, and each costs hours.
