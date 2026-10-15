# RAR-M71 — C.8.8 label audit of the `hce-v7` source games: clean wins the cycle-2 head does not win at 8,000 nodes

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-15, before any `hce-v7` game exists, the 3,000-game pilot included, which is not read for this. The generation is the maintainer's. The audit is agent-run: zero games, tablebase probes of the recorded games.
- **Baseline SHA / candidate SHA / dirty-diff hash:** branch `c7` at this registration's commit. No candidate: this grades labels, not an engine.
- **Binary / compiler / PGO identity:**
  - The label generator is the accepted cycle-2 head, `tools/test_engines/rarog-c8cycle2-pext-pgo.exe` (SHA-256 `A3A11321…189D`, `bench 13` 14,126,183, built clean at `75c3db5`).
  - `datagen.ps1 -Runner colosseum -ExpectSha256 -ExpectFingerprint` verifies it before the first game and records it in the run manifest. The runner is the pinned `colosseum-cli 0.3.0` (`381EB7A5…`).
  - Tool: `tools/diag/datagen_label_audit.py`, as RAR-M67 to RAR-M69 ran it. Syzygy 3–6 men at `D:/chess/tablebases/syzygy3456`.
- **Research question:** what share of first tablebase clean wins do the generating games not win? The comparison is RAR-M69's 6.66% for `hce-v6`: the cycle-1 head, the same book, starts, node budget and `datagen-v2`.
- **Hypothesis / proposed mechanism:** conversion at 8,000 nodes is mostly the search's (RAR-M67). RAR-M68 and RAR-M69 found the evaluation programme and the first refit left it level (6.74% and 6.66%). The second refit moves no term either, so the rate stays level.
- **Competing hypotheses:**
  1. The second refit's gain (RAR-E61: +6.7 Elo at `3+0.03`) is partly endgame play, and conversion improves measurably.
  2. The refit moved the magnitudes in won endings that pruning relies on, and conversion slips. RAR-E61's tree read grew the endgame band +13.2%.
  3. A stronger engine reaches different endings from the same starts, so the rate moves without any change in conversion skill.
- **Interacting mechanisms / consumers:** the labels above six men, which the Syzygy relabel cannot correct; C.8.9's refit is fitted on them.
- **PRE-REGISTERED PREDICTION (freeze before exposure):**
  - Games reaching a first clean win: 40% to 46% (RAR-M69 42.70%), probability 0.7.
  - Clean wins not won, of clean wins: **6.6%**, 80% band 4.5% to 9%. Probability that it is below RAR-M69's 6.66%: 0.5.
  - KRPP-KR and KRP-KRP are among the five largest failing families by count, probability 0.8.
  - Expected Elo: none; this grades labels.
- **Falsification criteria:** none for an engine; the measurement is descriptive. A rate above 9% supports competing hypothesis 2.
- **Registered rule and use (frozen):** one audit over the complete source PGN, run once after generation and before the corpus is fitted.
  - A conditional rate above **10%** holds C.8.9's fit until the maintainer decides.
  - Otherwise the result is recorded with the corpus.
- **Full conditions / provenance:** `python tools/diag/datagen_label_audit.py --pgn tools/texel/data/selfplay-c8cycle2-n8000-s1-g<N>.pgn --syzygy D:/chess/tablebases/syzygy3456 --max-men 6 --workers 30 --output tools/results/label-audit-hce-v7/report.json`, where `<N>` is the game count the pilot's preflight recommends (`extract.py --preflight-games 3000 --target-train 3500000`). First clean win per game, cursed wins excluded.
- **Result:** pending; the generation has not run.
