# RAR-M57 — Pool gauntlet of Rarog 2.5.0-dev (the fit at 3,900) — maintainer-run 2026-09-19, recorded after the event …

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**Pool gauntlet of Rarog 2.5.0-dev (the fit at 3,900) — maintainer-run 2026-09-19, recorded after the event; not pre-registered, so no prediction is scored (PLAN B.2.3.4).** Colosseum tournament `8791a27e` "Rarog 2.5.0-dev Gauntlet", 09:55–11:06 UTC: one seed against ten opponents, 600 games each, **6,000 games**, the Super Rating Tournament's exact conditions (RAR-M54: `3+0.03`, 1T, Hash 128, UHO_Lichess_4852_v1 with book seed 42, no adjudication, tablebases off, concurrency 14). **Ratings:** only Rarog-dev's rating follows the event; the opponents stay at their Super Rating Tournament ratings, so the event places one newcomer in a pool whose ratings were fitted jointly from 172,200 games. **Rarog:** `D:\chess\engines\rarog\rarog-v2.5.0-dev-fit3900-windows-pext-native-pgo.exe` (sha256 `1F73D5C9…`, `2.5.0-dev+b2core`, bench 6,199,302; the RAR-S76 values as a native PGO build).

## Result / disposition

**Rating 3191** (performance 3191; start 3100) against Rarog 2.4.0 at 3001 on the same scale. Per opponent (Rarog's score, read from the database, 600 games each): Houdini 4 (3310) 35.3%, −105; Houdini 3 (3287) 39.2%, −77; Critter 1.6a (3192) 46.1%, −27; Stockfish 5 (3221) 46.3%, −26; Houdini 1.5a (3211) 50.5%, +4; Shredder 13 (3201) 50.5%, +4; Fritz 16 (3173) 52.3%, +16; Rybka 4.1 (3111) 59.1%, +64; Rarog 2.4.0 (3001) 72.4%, +168; Basilisk 1.10.0 (2993) 74.4%, +186. Total W 2377, L 2063, D 1560; every game ended naturally (checkmate 4,440, threefold 1,040, insufficient material 331, fifty-move 163, stalemate 26), no forfeit. **Cross-checks:** against 2.4.0, +168 head-to-head and +190 on the rating scale, near the self-play sum of B.2.4a and RAR-S76 (+184). Against Rybka 4.1, +64 ± about 25 here against +97.69 ± 12.84 under fastchess at Hash 64 (RAR-M55), 1.2 σ apart. **E.2 at 1T** (≥50% against each target; 4T not measured): Fritz 16 passes, Critter 1.6a (−27) and Houdini 3 (−77) do not; Rybka 4 was not in the event. **Target amendment, after this row, 2026-09-19:** the maintainer made Rybka 4.1 the E.2 target in place of Rybka 4, and Rybka 4.1 at 59.1% passes; two of the four targets pass at 1T. A placement estimated in conversation before the event from RAR-M54's 200-game pairs put Critter about 95 below Rarog-dev; the event reads 27 above, inside those pairs' wide intervals.

## Conditional lesson and retry trigger

An observation, never a gate. It is the first pool rating of the fitted core and the reference for B.2.3.3 and B.9; a rerun after the final theta uses the same tournament settings.

## Source

RAR-M54, RAR-M55, RAR-S76; PLAN B.2.3.4, E.2
