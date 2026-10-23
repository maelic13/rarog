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
