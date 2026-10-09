# RAR-S71 — 4.6.2 SearchCore rewrite, registered before games at `43d5174`

Indexed under *3. Search and selectivity › Closed 4.6 follow-ups* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and reconstruction

**4.6.2 SearchCore rewrite, registered before games at `43d5174`.** Steps 13 and 16 were rebuilt together behind `search_core`; arm A default 1 fingerprint **3,479,169 / EBF 2.343**, arm B default 0 exact RAR-S70 **6,977,070 / EBF 2.466**. Registered `[-5,5]` nElo, cap 30,000, `3+0.03`, 1T, Hash 64, paired UHO, concurrency 14, strength-v2. Candidate/base executable SHA-256: `2DAE50778BCEB3BF89A19D1DB47BF6D14B4D78BE1664F3E568B5984626C4D4AB` / `B4DDEE13E06866B4BAF3F8D9941A44161AA09F568099867776832B58CA86979C`. Reconstruct without a branch in a disposable worktree by applying the inverse of revert `c5e451d` to the accepted head; the revert contains the complete 1,772-line removal, including `src/search/core.rs`. Build both arms from one tree by changing only `search_core` default 0 -> 1 and require the fingerprints above.

## Result / disposition

**Stopped manually before a boundary; rejected as the development route and reverted.** `tools/pgn_result.ps1` reconstructs 356 complete pairs / **712 games**, 181-204-327, score 48.596%, **-9.76 +/- 17.70 Elo**, LOS 13.76%, pentanomial `[20,101,134,81,20]`; two partial rounds excluded. Commit `c5e451d` restores exact RAR-S70 and passed debug/release tests, fmt, all-feature/all-target clippy and the accepted fingerprint.

## Conditional lesson

The zero-game evidence was unusually strong—EBF 2.466 -> 2.343 and WAC 182/300 versus 167/300 on 54% fewer nodes—and still did not predict games. The rewrite changed several co-adapted policies while retaining constants fitted to the old search, so a loss could not attribute structure versus fit. Do not repeat a wholesale search rewrite; isolate one producer/consumer contract and gate it.

## Artifact evidence

Local PGN/log/manifest SHA-256: `C67CB4E175EA8A2F1F12501FE545F222A230D62384C884F223E06C538DAE299B`, `37ACD07750F7600E0EC755B17FB21D111D17BA21A0B5B7492413467135F3DE4E`, `5429F1CEF20CF709EC370E2D4C72E1460AF218400984D74C7F586397C42F61F9`; opening seed `596286585`; book SHA-256 `7A7F6470615A69C6CF23D565417701D38732876F480AF90D67B42ABADE35644A`
