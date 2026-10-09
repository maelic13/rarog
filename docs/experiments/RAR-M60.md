# RAR-M60 — Colosseum CLI against fastchess on the same gate — the SPRT half of the 2026-09-17/18 harness parity …

Indexed under *2. Measurement, harness and tuning* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**Colosseum CLI against fastchess on the same gate — the SPRT half of the 2026-09-17/18 harness parity, recorded 2026-09-21 from its artifacts, zero new games.** Arms: `rarog-b24a-core-pext-pgo.exe` against `rarog-b24a-base-pext-pgo.exe`, the pair B.2.4a gated. Conditions on both instruments: `[0,10]` nElo, alpha = beta = 0.05, `model=normalized`, `3+0.03` with a 20 ms margin, Hash 64, one thread, UHO_Lichess_4852_v1 in random order, no adjudication, concurrency 14 on pinned physical cores. fastchess ran the gate on 2026-09-16 (`sprt_b24a-core_vs_b24a-base_20260916_075137`); Colosseum replayed it on 2026-09-17/18 with its own seeds (`colosseum-sprt-b24a-3`, seed 660717961; `colosseum-verify-sprt-3`, seed 26). Every number below is recounted from the PGNs (`tools/pgn_result.ps1`, `tools/diag/colosseum_recount.py`), not copied from a console line.

## Result / disposition

**All three accepted H1, and on the closest-sized pair the two instruments agree to within a tenth of an nElo.** fastchess: 216 complete pairs / 432 games, Ptnml [5, 35, 77, 73, 26], **+65.09 +/- 23.26 Elo, +94.00 +/- 32.76 nElo**. Colosseum `verify-sprt-3`: 220 pairs / 440 games, Ptnml [10, 29, 77, 72, 32], **+69.61 +/- 24.67 Elo, +94.08 +/- 32.46 nElo**, one time forfeit in 440 games (0.23%, under the 0.5% ceiling), wall 355 s. Colosseum `sprt-b24a-3`: 400 pairs / 800 games, Ptnml [22, 66, 159, 111, 42], +37.05 +/- 17.54 Elo, +51.26 +/- 24.08 nElo, no fault, wall 620 s. The recount reproduces each run's own pentanomial exactly wherever the record carries one. **Observation, not an acceptance:** it says the instruments agree at this resolution and nothing about the candidate, which B.2.4a already gated.

## Conditional lesson and retry trigger

**A stopped SPRT is the wrong instrument for a parity claim**: its point estimate is selected at the boundary, so it is biased upward by construction, and two seeds of the same true effect differ here by more (37.05 against 69.61) than either differs from fastchess. Read parity from the fixed match instead (RAR-M61). Repeat this comparison on the triggers in PROCESS's *Harness* section: a runner, scheduler or topology change, a surprising result, or a re-pin of `colosseum.pin.json`.

## Source

PLAN B.2.6.2; RAR-S73 (B.2.4a); `tools/results/sprt_b24a-core_vs_b24a-base_20260916_075137.{pgn,log,manifest.txt}`, `tools/results/colosseum-sprt-b24a-3/`, `tools/results/colosseum-verify-sprt-3/`
