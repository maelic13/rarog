# RAR-M69 — C.8.5 label audit of the `hce-v6` source games: clean wins the cycle-1 head does not win at 8,000 nodes

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-14, before any `hce-v6` game exists, the 3,000-game pilot included, which is not read for this. The generation is the maintainer's. The audit is agent-run: zero games, tablebase probes of the recorded games.
- **Baseline SHA / candidate SHA / dirty-diff hash:** branch `c7` at this registration's commit. No candidate: this grades labels, not an engine.
- **Binary / compiler / PGO identity:**
  - The label generator is the accepted cycle-1 head, `tools/test_engines/rarog-c8cycle1-pext-pgo.exe` (SHA-256 `76CD1F3C…5DA7`, `bench 13` 14,291,377, built clean at `bfd008a`; the engine source is unchanged since).
  - `datagen.ps1 -Runner colosseum -ExpectSha256 -ExpectFingerprint` verifies it before the first game and records it in the run manifest. The runner is the pinned `colosseum-cli 0.3.0` (`381EB7A5…`), whose games equal fastchess's at fixed nodes (PLAN C.8.4).
  - Tool: `tools/diag/datagen_label_audit.py`, as RAR-M67 and RAR-M68 ran it. Syzygy 3–6 men at `D:/chess/tablebases/syzygy3456`.
- **Research question:** what share of first tablebase clean wins do the generating games not win? The comparison is RAR-M68's 6.74% for `hce-v5`: the C.8.1 head, the same book, starts, node budget and `datagen-v2`. Does the cycle-1 refit, which is the same evaluation surface refitted and the search frozen, convert its won endings at least as well?
- **Hypothesis / proposed mechanism:** conversion at 8,000 nodes is mostly the search's (RAR-M67), and RAR-M68 found the evaluation programme left it level (6.86% to 6.74%). The cycle-1 refit moves no term into or out of the evaluation, so the rate stays level.
- **Competing hypotheses:**
  1. The refit's +11 Elo at `3+0.03` (RAR-E60) is partly better endgame play, and conversion improves measurably.
  2. The refit moved the magnitudes in won endings that pruning relies on (RAR-E60's magnitude shift reached +2.8 cp up to 600), and conversion slips.
  3. A stronger engine reaches different endings from the same starts, so the rate moves without any change in conversion skill.
- **Interacting mechanisms / consumers:** the labels above six men, which the Syzygy relabel cannot correct; C.8.6's refit is fitted on them.
- **PRE-REGISTERED PREDICTION (freeze before exposure):**
  - Games reaching a first clean win: 40% to 46% (RAR-M68 42.63%), probability 0.7.
  - Clean wins not won, of clean wins: **6.6%**, 80% band 4.5% to 9%. Probability that it is below RAR-M68's 6.74%: 0.55.
  - The largest failing families are still rook and pawn endings: KRPP-KR and KRP-KRP among the five largest by count, probability 0.75.
  - Expected Elo: none; this grades labels.
- **Falsification criteria:** none for an engine; the measurement is descriptive. A rate above 9% supports competing hypothesis 2.
- **Registered rule and use (frozen):** one audit over the complete source PGN, run once after generation and before the corpus is fitted.
  - A conditional rate above **10%** holds C.8.6's fit until the maintainer decides: labels clearly worse than `hce-v5`'s would confound the cycle's gate.
  - Otherwise the result is recorded with the corpus.
- **Full conditions / provenance:** `python tools/diag/datagen_label_audit.py --pgn tools/texel/data/selfplay-c8cycle1-n8000-s1-g<N>.pgn --syzygy D:/chess/tablebases/syzygy3456 --max-men 6 --workers 30 --output tools/results/label-audit-hce-v6/report.json`, where `<N>` is the game count the pilot's preflight recommends (`extract.py --preflight-games 3000 --target-train 3500000`). First clean win per game, cursed wins excluded, as RAR-M22, RAR-M67 and RAR-M68.
- **Result:** pending; the generation has not run.
