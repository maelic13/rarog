# RAR-E49 — KRP-KB's DTZ-progress report: a 100-position floor or a slower engine?

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-12, before any number was computed; agent-run (zero games), on the maintainer's request of 2026-10-12 to give the unowned report an owner.
- **Baseline SHA / candidate SHA / dirty-diff hash:** the current head, engine `38cbb02` (**14,197,472 / EBF 2.557**), as `analysis/artifacts/c523/rarog-head.exe` (SHA-256 `b34990d3…`). Comparison arms: RAR-E25's pre-C.3 head at 400 positions, already measured (`analysis/artifacts/c3-screens/endgame/truth400-head.json`: KRP-KB DTZ progress 0.3032 over 6,274 checked moves, conversion 384 of 389, win preservation 0.9987, cohort `8dfc47db…`); and the 2.4.0 release, `D:/chess/engines/rarog/rarog-v2.4.0-windows-pext-native-pgo.exe`, as the nearest available build to the one that set the floors (4.11.2, `tools/results/truth-v2-head/`, built from a pre-2.4.0 head whose exact binary was not kept). Floors: `tools/diag/endgame_floors.json`, KRP-KB DTZ progress 0.3280 over 1,942 moves (conversion 90 of 96).
- **Research question:** the floors read reports KRP-KB's DTZ progress below its floor at every head measured since the floors were set: −2.1 SE (pre-C.3 head, RAR-E25), −2.7 SE (RAR-E25's candidate), −2.6 SE (current head, RAR-E48). Is the floor a high draw of a 100-position sample, as KBP-K's was (RAR-E48), or has the engine become slower in this family since 2.4.0?
- **Hypothesis / proposed mechanism:** the floor is a sampling artefact. Moves within one game are correlated, so 1,942 moves from 100 games carry less information than their binomial error suggests, and a 100-position floor can sit above what any engine of that period reads at 400 positions. The truth tool already marks DTZ progress as no measure of technique in this pawn-move family (`dtz_progress_is_technique: false`).
- **Competing hypotheses:** (1) *a real slowdown:* an evaluation change since 2.4.0 (C.3's king terms, C.6's passer terms, the endgame scale) makes the engine prefer zeroing-neutral moves in KRP-KB, which would show as 2.4.0 reading higher than the current head on the same 400 positions. (2) *a conversion cost behind it:* the slower progress also loses wins at the 100-ply limit, which would show as lower conversion or win preservation than the pre-C.3 head.
- **Interacting mechanisms / consumers:** the rook-and-pawn evaluation, C.6's passer terms and square rule, the endgame scale and the mop-up; the floors file, which every later endgame screen reads.
- **PRE-REGISTERED PREDICTION (freeze before exposure):**
  1. The 400-position KRP-KB cohort's first 100 positions hash to the floors' KRP-KB digest `d09190f0…`, and its 400-position digest equals RAR-E25's `8dfc47db…` (probability 0.95).
  2. The current head reads KRP-KB DTZ progress between 0.28 and 0.31 at 400 positions (0.7), with conversion of at least 380 of 389 (0.7).
  3. 2.4.0 reads between 0.29 and 0.32 at 400 positions, within 2 SE of the current head (0.65).
  4. The rule below re-baselines the floor (0.6).
- **Falsification criteria:** void if the head arm does not bench 14,197,472, or if any arm's KRP-KB cohort digest differs from the others'.
- **Cheapest prior falsifier:** the prefix check (seconds).
- **Registered order (frozen):** into `analysis/artifacts/krpkb/`:
  1. `check_prefix.py` (KRP-KB's version of RAR-E48's): the first 100 of the generated 400 positions against the floors' digest, and the 400 against `8dfc47db…`.
  2. `endgame_truth.py --families KRP-KB --positions 400` at RAR-E25's settings (60,000 nodes, 100 plies, seed 6200600, hash 16, 14 workers), once with the current head (`truth400-head.json`) and once with 2.4.0 (`truth400-240.json`), on an idle host.
  - Nothing is added after the numbers are seen.
- **Rule (frozen):** SE is the two-sample binomial difference over checked moves (`endgame_floors.py`'s `se_diff`; optimistic, moves being correlated). **The floor is re-baselined at 400 positions**, to the current head's 400-position reading, in its own commit, when all three hold: (a) the prefix check passes; (b) the current head's 400-position conversion and win preservation are not below the pre-C.3 head's by more than 2 SE; (c) 2.4.0's 400-position DTZ progress is not above the current head's by more than 2 SE. If (c) fails, the engine has slowed in KRP-KB since 2.4.0 on the same positions: no re-baseline, and the diagnosis goes to the maintainer. If (b) fails, the conversion loss is a defect held under C.5.8 with this read as its evidence.
- **Use, frozen:** a re-baseline removes the report and closes this item; anything else keeps it reported with an owner named here.
- **Result:** pending.
