# RAR-M67 — C.2 label audit of the `hce-v4` source games: clean wins the B.9 head does not win at 8,000 nodes

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-06, before any `hce-v4` game exists; the generation is the maintainer's, the audit is agent-run (zero games, tablebase probes of the recorded games).
- **Baseline SHA / candidate SHA / dirty-diff hash:** `dev` at this registration's commit. No candidate: this grades labels, not an engine.
- **Binary / compiler / PGO identity:** the label generator is the Phase C frozen head `tools/test_engines/rarog-b9head-pext-pgo.exe` (SHA-256 `AAC92114…01EE`, `bench 13` 11,171,726, built at `24aefb4`), verified by `datagen.ps1 -ExpectSha256 -ExpectFingerprint` before the first game and recorded in the run manifest. Tool: `tools/diag/datagen_label_audit.py`, whose code is RAR-M22's (only its docstring changed since, in `bd1c1a5`; its tests pass at this commit); Syzygy 3–6 men at `D:/chess/tablebases/syzygy3456`.
- **Research question:** the protocol (programme document, section 9) asks for the share of first tablebase clean wins that the generating games do not win, against RAR-M22's 20.33% for the `hce-v3` source (the RAR-E08 head, same book, same 8,000 nodes, same `datagen-v2`). Does the B.9 search, +272 Elo over 2.4.0 and further over the `hce-v3` labeller, convert its won endings more often at the same node budget?
- **Hypothesis / proposed mechanism:** the B.9 search spends its 8,000 nodes better (pruning, extensions, mop-up drive, recognisers), so it converts more clean wins; the failures stay concentrated in rook and pawn endings, which search strength helps least.
- **Competing hypotheses:** (1) at 8,000 nodes conversion is a node-budget limit, not a search-quality one (Basilisk: 3.1× nodes bought only 31% relative), so the rate barely moves; (2) a stronger search reaches harder clean wins more often (more endgames reached at all), holding the conditional rate up while the all-games rate rises.
- **Interacting mechanisms / consumers:** the labels above six men that the Syzygy relabel cannot correct; RAR-E24's refit is fitted on them.
- **PRE-REGISTERED PREDICTION (freeze before exposure):**
  - Games reaching a first clean win: 40% to 48% (RAR-M22 read 44.22% for `hce-v3`; same book), probability 0.7.
  - Clean wins not won, of clean wins: **15%**, 80% band 10% to 21%; probability that it is below 20.33%: 0.75.
  - The largest failing families are still rook and pawn endings (KRP-KR, KRPP-KR, KRP-KRP, KPP-KPP among the five largest by count), probability 0.8.
  - Expected Elo: none; this grades labels.
  - Most likely failure mode: competing (1), the rate within a point of 20%.
- **Falsification criteria:** the hypothesis is refuted if the conditional rate is 20.33% or more.
- **Cheapest prior falsifier:** none cheaper than the audit itself; the 3,000-game pilot is too small for a family-level rate and is not read.
- **Registered gate and stop rule:** no gate; one audit over the complete source PGN, run once after generation and before the corpus is fitted. **Use, frozen:** recorded with the corpus; a conditional rate above **25%** (labels worse than `hce-v3`'s) holds RAR-E24's fit until the maintainer decides, since a label contract worse than the one RAR-E12 fitted would confound the refit's gate.
- **Full conditions / provenance:** `python tools/diag/datagen_label_audit.py --pgn tools/texel/data/selfplay-b9head-n8000-s1-g612747.pgn --syzygy D:/chess/tablebases/syzygy3456 --max-men 6 --workers 30 --output tools/results/label-audit-hce-v4/report.json`; first clean win per game, cursed wins excluded, as RAR-M22.
- **Result (run 2026-10-07, once, after generation; `tools/results/label-audit-hce-v4/report.json`):** 612,747 games (PGN `selfplay-b9head-n8000-s1-g612747.pgn`, generated 2026-10-06 21:18 to 2026-10-07 00:27 UTC at concurrency 30, engine verified in the run at SHA-256 `AAC92114…01EE` and `bench 13` 11,171,726). **259,306 games reach a first clean win (42.32%; `hce-v3` 44.22%) and 17,799 of them are not won: 6.86% of clean wins** (`hce-v3`: 20.33%), 2.90% of all games (`hce-v3`: 8.99%). Largest failing families by count: KRPP-KR 2,164 of 16,631 (13.01%), KRP-KRP 1,400 of 7,830 (17.88%), KPP-KPP 586 of 9,437 (6.21%), KRP-KBP 580 of 3,727 (15.56%), KBPP-KB 565 of 4,859 (11.63%); KRP-KR is not among the eight largest. The corpus agrees independently: the Syzygy relabel of `hce-v4` changed 22,083 train labels (0.63% of rows; `hce-v3` 113,046, 3.23%), 0 probe failures.
- **Disposition:** observation. The B.9 head converts its clean wins at 8,000 nodes about three times as often as the `hce-v3` labeller did; by the frozen use (hold above 25%) RAR-E24's fit is not held.
- **PREDICTION CALIBRATION (appended after exposure):**
  - Original prediction (not rewritten): the items above.
  - Observed: games reaching a clean win 42.32%, inside 40–48% (hit). Clean wins not won 6.86% against a point of 15% and an 80% band of 10% to 21%: **below the band** (direction hit, probability 0.75 for under 20.33%; magnitude miss). Failing families: KRPP-KR, KRP-KRP and KPP-KPP are among the five largest, KRP-KR is not (three of the four named).
  - Mechanism supported? The hypothesis (search quality, not node budget, limits conversion at 8,000 nodes) is supported more strongly than predicted; competing (1), a node-budget limit, is refuted for this search.
  - Missed interaction or instrument limit: I anchored on Basilisk's node-scaling figure (3.1× nodes for 31% relative), which measures a different lever; the B.9 changes (mop-up drive, recognisers, pruning) act on exactly the endings that failed.
  - Confidence: under-confident on the size of the improvement.
- **Retry trigger or `closed`:** closed on its record.
- **Artifacts / commits:** `tools/results/label-audit-hce-v4/` (ignored storage).
