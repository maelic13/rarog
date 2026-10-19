# RAR-M68 — C.8.1 label audit of the `hce-v5` source games: clean wins the accepted head does not win at 8,000 nodes

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-13, before any `hce-v5` game exists beyond the 3,000-game pilot, which is not read for this; the generation is the maintainer's, the audit is agent-run (zero games, tablebase probes of the recorded games).
- **Baseline SHA / candidate SHA / dirty-diff hash:** branch `c7` at this registration's commit. No candidate: this grades labels, not an engine.
- **Binary / compiler / PGO identity:**
  - The label generator is the accepted head, `tools/test_engines/rarog-c8head-pext-pgo.exe` (SHA-256 `E73E3230…C1C7`, `bench 13` 14,197,472, built at `c72b41c`, whose engine source equals `master`'s `52cc6de`).
  - `datagen.ps1 -ExpectSha256 -ExpectFingerprint` verifies it before the first game and records it in the run manifest.
  - Tool: `tools/diag/datagen_label_audit.py`, as RAR-M67 ran it. Syzygy 3–6 men at `D:/chess/tablebases/syzygy3456`.
- **Research question:** the share of first tablebase clean wins that the generating games do not win, against RAR-M67's 6.86% for the `hce-v4` source (the B.9 head, same book, same 8,000 nodes, same `datagen-v2`). Does the evaluation programme's head (king safety, threats, the opposite-bishop rule, passers, the KNN-KP terms; search frozen) convert its won endings at least as well at the same node budget?
- **Hypothesis / proposed mechanism:** conversion at 8,000 nodes is mostly the search's (RAR-M67: the B.9 search tripled it). The evaluation changes since then act on few endings, the KNN-KP terms and C.6's passer terms among them, so the rate is level or a little lower.
- **Competing hypotheses:** (1) C.6's passer and square-rule terms improve pawn-ending conversion measurably; (2) the refits since `hce-v4` moved magnitudes the search's pruning relies on in won endings, and conversion slips; (3) a different game population (another engine's choices from the same starts) reaches different endings, so the rate moves without any conversion skill changing.
- **Interacting mechanisms / consumers:** the labels above six men that the Syzygy relabel cannot correct; C.8.2's refit is fitted on them.
- **PRE-REGISTERED PREDICTION (freeze before exposure):**
  - Games reaching a first clean win: 40% to 46% (RAR-M67 42.32%), probability 0.7.
  - Clean wins not won, of clean wins: **6.5%**, 80% band 4.5% to 9%; probability that it is below RAR-M67's 6.86%: 0.55.
  - The largest failing families are still rook and pawn endings (KRPP-KR, KRP-KRP and KPP-KPP among the five largest by count), probability 0.75.
  - Expected Elo: none; this grades labels.
- **Falsification criteria:** none for an engine; the measurement is descriptive. Competing (2) is supported if the rate is above 9%.
- **Registered rule and use (frozen):** one audit over the complete source PGN, run once after generation and before the corpus is fitted. A conditional rate above **10%** (labels clearly worse than `hce-v4`'s) holds C.8.2's fit until the maintainer decides, since a label contract worse than the accepted corpus's would confound the cycle's gate. Otherwise the result is recorded with the corpus.
- **Full conditions / provenance:** `python tools/diag/datagen_label_audit.py --pgn tools/texel/data/selfplay-c8head-n8000-s1-g595240.pgn --syzygy D:/chess/tablebases/syzygy3456 --max-men 6 --workers 30 --output tools/results/label-audit-hce-v5/report.json`; first clean win per game, cursed wins excluded, as RAR-M22 and RAR-M67.
- **Result (run 2026-10-13 19:18–19:21, once, after generation and before any fit; `tools/results/label-audit-hce-v5/report.json`, `34f590b4…`):**
  - *Source:* 595,240 games (`selfplay-c8head-n8000-s1-g595240.pgn`, `7A0EA868…`, generated 2026-10-13 13:11 to 16:13 UTC at concurrency 30, engine verified in the run at `E73E3230…` and bench 14,197,472, clean tree): 223,533 wins for White, 181,662 for Black, 190,045 draws.
  - *Audit:* **253,752 games reach a first clean win (42.63%; `hce-v4` 42.32%), and 17,101 of them are not won: 6.74% of clean wins** (`hce-v4` 6.86%), 2.87% of all games (`hce-v4` 2.90%).
  - *Largest failing families by count:* KRPP-KR 2,031 of 16,093 (12.62%), KRP-KRP 1,150 of 7,270 (15.82%), KRNP-KR 564 (14.80%), KRP-KBP 553 (15.56%), KBP-KPP 550 (11.27%); KPP-KPP 496 of 9,174 (5.41%) is eighth.
  - *The corpus agrees independently:* the Syzygy relabel of `hce-v5` changed 22,706 train labels (0.65% of rows; `hce-v4` 0.63%), 0 probe failures.
- **Disposition:** observation. The accepted head converts its clean wins at 8,000 nodes as the B.9 head did (6.74% against 6.86% not won), so the label contract is unchanged in quality. By the frozen use (hold above 10%), **C.8.2's fit is not held**.
- **PREDICTION CALIBRATION (appended after exposure):**
  - Original prediction (not rewritten): the items above.
  - Observed: games reaching a clean win 42.63%, inside 40–46% (hit). Not won 6.74%, inside the 4.5–9% band and below 6.86% (hit, probability 0.55). Failing families: KRPP-KR and KRP-KRP among the five largest, KPP-KPP not (it is eighth, behind three rook-and-minor and minor-and-pawn families): **partial miss**.
  - Which assumption failed: none in the rate; the evaluation programme left conversion at this node budget where the search had put it, which is the hypothesis. The family ranking moved toward rook-and-minor endings, which the 2.4.0-era ranking did not weigh.
- **Retry trigger or `closed`:** closed on its record.
- **Artifacts:** `tools/results/label-audit-hce-v5/` (ignored storage); the pilot, preflight, publication, relabel and contract logs in `analysis/artifacts/c81/`.
