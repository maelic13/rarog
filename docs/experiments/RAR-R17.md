# RAR-R17 — D.1.5's decomposition: the clock cluster at its seeds against the head, and the horizon's late-game cost, two 2,000-game reads at `3+0.03`

Indexed under *4. Root search, time management and SMP* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-23, before any game, while RAR-R16's
  SPRT is paused at 1,782 of 20,000 pairs (unresolved, +0.2 ± 6.6 Elo). The
  reads are the maintainer's commands; they do not touch the SPRT, which
  resumes afterwards with its own command.
- **Kind:** two direction reads that decompose what RAR-R16 gates,
  following `analysis/d15_mechanism_review_2026-10-23.md`. They decide
  nothing about the gate; they direct the research D.1 does next.
- **Why now:** the maintainer would rather improve the mechanisms than wait
  on a gate trending even. The review names the horizon's late-game cut
  (RAR-R14's H3) as the first suspect.
- **The baseline is the 2.6.0 engine.** `rarog-d15head-pext-pgo.exe` is
  `468ed7b`, which differs from the `v2.6.0` tag (`0127e58`) only in its
  version string and a reworded comment in `src/cpu_advice.rs`, checked
  with `git diff v2.6.0 468ed7b`. It carries none of D.1's mechanisms.

## The reads (frozen)

Both are Colosseum `match` at `3+0.03`, Hash 64, one thread, 14 slots, the
UHO book in random order, no adjudication. They are run by
`analysis/artifacts/d15-clock-gate-2026-10-23/run_r17.ps1 -Read
seeds|horizon`. Both dry runs resolved to policy on an idle host on
2026-10-23.

1. **`seeds`: the cluster at its seeds against the head.**
   - `rarog-d15seeds-pext-pgo.exe` (SHA-256 `DBC6615E…`, `b96a9c1`, all
     seven mechanisms at their seeds) against `rarog-d15head-pext-pgo.exe`
     (`273FEDF4…`, `468ed7b`); both PGO, bench 13,987,223.
   - **2,000 games**, seed 20261035, `tools/results/rar-r17-seeds`.
   - With RAR-R15's fit-against-seeds wire match (even over 200 games), it
     splits RAR-R16's total into mechanisms and fit.
2. **`horizon`: the horizon at 100 against 53, everything else at theta.**
   - `-CategoricalTuneBuild` on `rarog-d13cluster7-tune.exe` (`a5efb11c…`,
     `2b9831f`, bench 13,987,223). Both arms carry block 1's thirteen values
     from `tools/results/d14-spsa-block1/tuned-options.txt`, read by the
     script.
   - Arm A sets `TmHorizonPerSec=100` (contraction only below 0.5 s); arm B
     keeps theta's 53.
   - **2,000 games**, seed 20261036, `tools/results/rar-r17-horizon`.
   - The manifest's commands confirm the arms differ in that one option.

**Readings (frozen):**
- Each read's Elo with its 95% interval, from the manifest, and its time a
  move by ply band and clock floor: `tm_profile_d15.py` for `seeds`, and
  `tm_wire_read.py` with its relabelling for `horizon`.
- **What each result directs:**
  - **`horizon` at +5 Elo or more with its lower bound above zero:** the
    horizon is re-scoped before any fallback or new gate. Its contraction
    is confined to the clocks where forfeits occur, which needs a wider
    range for `TmHorizonPerSec` or a different form, registered then.
  - **`seeds` at −5 or less with its upper bound below zero:** the
    mechanisms cost at their seeds. The research splits them further, the
    fail-high stop first, which needs a switch.
  - **Both inside ±5:** neither read locates a cost. RAR-R16's SPRT
    resumes, and its own dispositions stand.
- **Maintainer requirement, 2026-10-23, for any fallback:** "no harm" must
  be shown at longer time controls too. RAR-R16's `10+0.1` read (−6.3 ±
  11.7 Elo) does not show it. A fallback registration therefore carries a
  `10+0.1` harm read beside the `3+0.03` one.

## PRE-REGISTERED PREDICTION (frozen before any game)

1. **`seeds`:** −2 ± 6 Elo; P(point estimate below zero) 0.65.
2. **`horizon`:** arm A (100) **+4 ± 6 Elo** over arm B (53); P(above zero)
   0.65. Arm A's time a move in plies 80–119 higher by at least 15%, 0.8.
   Its clock-floor median lower than arm B's, 0.9.
3. **No fault or time loss in either read**, 0.85 (arm A of `horizon`
   keeps less reserve).

- Most likely outcome: `horizon` favours 100 by a few Elo inside its
  interval, and `seeds` reads near zero. Then the horizon is the lead but
  not proven.
- The failure that would surprise: `horizon` favours 53. That would refute
  H3 at `3+0.03` and point the cost, if any, elsewhere.

## Results (played 2026-10-23, maintainer-run)

Both reads ran on an idle host (5.1% and 0.4% busy, no waiver), with 0
faults, 0 time losses and a recount that agreed.

**1. `seeds`: −14.9 ± 8.8 Elo** (−25.8 ± 15.2 nElo; pentanomial 33, 266,
473, 210, 18). Manifest
`colosseum_match_rar-r17-seeds_20261023_131031.manifest.txt` (`1ccdb886…`);
profile `profile_rar-r17-seeds.txt` (`3d44a680…`).
- **The frozen rule fires:** −5 or less, with the upper bound (−6.1) below
  zero. The mechanisms cost at their seeds.
- **How: the seeds arm leaves clock unused.**
  - It spends 65.0 ms a move against the head's 70.2 (−7.4%), and less at
    every stage, the opening included: plies 0–19 94 against 99 ms, where
    the horizon does not act (clock above a second).
  - It holds more clock from ply 40: 606 against 276 ms at plies 100–119.
  - It ends games with about 1.0 s in wins and draws, against the head's
    0.33 and 0.75 s; in losses 322 against 232 ms.
  - At this host's rate of about 2 Elo per 1% of thinking time at `3+0.03`
    (three earlier measurements), −7.4% is about −15 Elo, the read's point
    estimate. The decision quality of what it plays is not shown to differ.
- **The most likely source, not yet isolated:** the fail-high stop. It ends
  searches inside the iteration that overran the soft target, which cuts
  the overrun that is 21–31% of the head's time (RAR-R14). No term spends
  that time elsewhere.
  - The stability factor's decay toward 0.716 on long-held moves is the
    second candidate.
  - The horizon cannot cause the opening deficit.
- The clock floor: median 627 against 187 ms; games under 100 ms 0 against
  618 of 2,000.

**2. `horizon`: −0.9 ± 8.0 Elo** for arm A (100) against arm B (53)
(−1.6 ± 15.2 nElo; pentanomial 19, 203, 563, 194, 21). Manifest
`colosseum_match_rar-r17-horizon_20261023_133050.manifest.txt`
(`2dc3a6f3…`); profile `profile_rar-r17-horizon.txt` (`08d00bb0…`).
- **Inside ±5: the horizon's late-game cost is not shown.**
- Time a move is the same late in both arms (plies 80–119: 38 and 28
  against 38 and 29 ms). Arm A spends slightly more at plies 40–79 (87 and
  57 against 83 and 52), then plays the late game with less clock (335
  against 431 ms at plies 80–99).
- **The forfeit protection is large and free at this resolution:** 443 of
  2,000 arm-A games went under 100 ms of clock, against 0 at 53. The clock
  floor's median was 217 against 331 ms.

**Calibration:**
- 1 (`seeds` −2 ± 6, P(below zero) 0.65): the sign held, but the magnitude
  missed, at −14.9. The prediction did not price the unused clock; the
  review looked at the fitted candidate, which the fit had already moved
  back toward the head's spending.
- 2 (`horizon` +4 ± 6, P(above zero) 0.65): missed, at −0.9.
  - Arm A spending 15% more in plies 80–119 (0.8): missed, equal.
  - Its clock floor lower (0.9): held.
  - RAR-R14's H3, the contraction's late-game cost at `3+0.03`, is not
    supported here.
- 3 (no fault, 0.85): held.

**What it directs (frozen rules):**
- The horizon is **not** re-scoped. It is roughly Elo-neutral at `3+0.03`
  and removes the sub-100 ms games.
- The mechanisms cost at their seeds, so the research splits them, **the
  fail-high stop first, which needs a switch.**
- Read with RAR-R15's fit-against-seeds match (even over 200 games, which
  is ±35) and RAR-R16's paused SPRT (+0.2 ± 6.6 at 1,782 pairs), the fit
  appears to have bought back about what the seeds lost, mostly by
  spending more.
- **The registered fallback** (mechanisms 1, 4, 5 and 6 at their seeds)
  carries the fail-high stop at its seeds. On this read it would likely
  fail its −13 nElo harm rule. It should not run in its registered form
  before the fail-high stop's own read.
