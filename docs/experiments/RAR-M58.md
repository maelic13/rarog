# RAR-M58 — Rarog 2.5.0-dev against Critter 1.6a — maintainer-run 2026-09-19, recorded after the event; not …

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**Rarog 2.5.0-dev against Critter 1.6a — maintainer-run 2026-09-19, recorded after the event; not pre-registered, so no prediction is scored (PLAN B.2.3.4).** Colosseum tournament `d6574360` "Rarog 2.5.0-dev vs. Critter 1.6a", started 11:14 UTC, scheduled for 2,000 games and **stopped by the maintainer at 1,050** (950 pending). Conditions as RAR-M57: `3+0.03`, 1T, Hash 128, UHO_Lichess_4852_v1 with book seed 42, no adjudication, tablebases off, concurrency 14, two games per opening; ratings never updated. Same Rarog binary as RAR-M57 (native, sha256 `1F73D5C9…`).

## Result / disposition

**−5.0 Elo** for Rarog, 95% interval **−22.4 to +12.4** over 525 complete opening pairs (W 376, L 391, D 283, 49.29%; pentanomial [67, 108, 181, 111, 58]). Every game ended naturally (checkmate 767, threefold 196, insufficient material 54, fifty-move 26, stalemate 7), no forfeit. With RAR-M57's 600 games against Critter (46.1%), 1,650 games read about 48.1%, about −13; both events draw openings with book seed 42, so they are not fully independent samples.

## Conditional lesson and retry trigger

Critter 1.6a is within about ±20 Elo of the fit at 3,900 and the E.2 target is not yet met against it at 1T; the rerun after the final theta decides the reading. An observation, never a gate.

## Source

RAR-M57; PLAN B.2.3.4, E.2
