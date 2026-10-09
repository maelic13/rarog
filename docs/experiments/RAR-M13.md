# RAR-M13 — Adjudication unified on 600/3 two-sided, 2026-08-18

Indexed under *2. Measurement, harness and tuning* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**Adjudication unified on 600/3 two-sided, 2026-08-18**, by maintainer decision on consistency rather than on new measurement. `strength-v1` (600/3 one-sided) becomes `strength-v2` (600/3 two-sided); `datagen-v1` is unchanged and now carries identical values. Draw rule untouched at `movenumber=40 movecount=8 score=10`. Implemented in `tools/harness_common.ps1` and read from there by `sprt.ps1`, `spsa.ps1`, `setup_tools.ps1` and `datagen.ps1`, so the two profiles cannot drift apart again; the weather-factory patch marker moves V2 -> V3 and its guards were re-pointed.

## Result / disposition

**Instrument change, no games.** Two-sided requires both engines to agree before a game is called, so this is the conservative direction: fewer adjudications, more games played out, marginally more wall time. RAR-M06's replay over 69,350 games is what makes it cheap — one-sided and two-sided 600/3 differed on **0.20%** of triggers (71 of 35,486) and on **no** final chess result.

## Conditional lesson and retry trigger

A verdict instrument should be one rule, not two justified rules, once the measured difference between them is 0.20% and never changes a result. The 0.20% is also the exact size of the discontinuity between ledger rows: **strength results recorded before 2026-08-18 were adjudicated one-sided.** Do not re-derive a pre-2026-08-18 Elo from post-change games without noting it. Recalibrate the whole rule after any material score-scale change, as RAR-M06 already requires.

## Source

`tools/harness_common.ps1`; RAR-M06
