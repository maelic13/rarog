# RAR-M17 — Adjudication dropped as the harness default, 2026-09-01, by maintainer decision on RAR-M16

Indexed under *2. Measurement, harness and tuning* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**Adjudication dropped as the harness default, 2026-09-01, by maintainer decision on RAR-M16.** `sprt.ps1` and `gauntlet.ps1` now run with no draw or resign adjudication unless `-Adjudicate` is passed; `-NoAdjudication` is retained and still truthful, so every recipe already recorded here reproduces verbatim. Extended the same day to **every** instrument: `datagen.ps1` moved to a new `datagen-v2` profile with no adjudication, and `setup_tools.ps1` strips both the resign and the draw line from weather-factory's `cutechess.py` (`RAROG_ADJUDICATION_PATCH_V4`), with `spsa.ps1` refusing to start a tune without it while still exempting a resume. `datagen-v1` is retained unedited so `hce-v2`'s manifests keep meaning what they said.

## Result / disposition

**Instrument change, no games.** Verified in both directions rather than by reading the flag: a bare invocation produced **0 adjudicated terminations in 30 games** and a manifest reading `adjudication: none`, while `-Adjudicate` produced **30 of 30** and the `strength-v2` label. Passing both switches is refused.

## Conditional lesson and retry trigger

**Results before and after 2026-09-01 used different instruments and the size of the discontinuity is UNKNOWN.** This is unlike RAR-M13, whose one-sided-to-two-sided change was backed by a 69,350-game replay showing 0.20% of triggers and no changed result; nothing equivalent has been replayed for adjudication-versus-none, and RAR-O01 vs RAR-O02 suggests it can be large when evaluators differ. Do not difference a pre-change Elo against a post-change one without saying so. Retry trigger: the whole default is worth revisiting once 4.9a closes -- adjudication's loss scales with how badly the engine converts, so it costs a 99% converter far less than a 52% one.

## Source

`tools/sprt.ps1`; `tools/gauntlet.ps1`; RAR-M13; RAR-M15; RAR-M16; RAR-O01/O02
