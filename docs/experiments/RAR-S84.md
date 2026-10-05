# RAR-S84 — B.3.4, the cluster 2 gate — REGISTERED 2026-09-27, before any game; binaries built and hashed

Indexed under *3. Search and selectivity › Registered, open* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**B.3.4, the cluster 2 gate — REGISTERED 2026-09-27, before any game; binaries built and hashed.** Candidate: the `b3proof` arm with RAR-S83's block-2 theta baked into every one of the 36 coordinates, the three shared LMR bases included (`CoreLmrQuiet` 1932, `CoreLmrNoisy` 830, `CoreLmrCutNode` 2071; corrected 2026-09-27 from a mistyped 727: theta, the gate binary and `f53ca7d` all carry 830, whose commit message repeats the typo), on the throwaway branch `b33-gate` (`11e7145`, worktree `D:/code/rarog-b33gate`, tree clean; the diff is preserved as `analysis/arm_patches/11e7145-b33-gate-proofparams.patch` since B.10, 2026-10-04; recipe: `tools/results/b33m-spsa-2/tuned-options.txt` applied to `src/search/params.rs` at `dev` `f5d16d8`+ by the bake script, nothing else). **Binary:** `tools/test_engines/rarog-b33gate-pext-pgo.exe`, sha256 **`69A568E1B11CA62476217B32EB3734244D660A2BE0EA3F9F2DE012CB4533D1C3`**, `cargo xtask build --arch pext --pgo --features b3proof`, bench **12,897,901**, equal to `rarog-b33m-tune.exe` with the 36 values set by option. **Baseline:** the accepted head `rarog-b27all-core-pext-pgo.exe` (sha256 `C27E2053…`, 7,435,006, `52c46df`). **Gate:** Colosseum `sprt-default`, `[0,3]` nElo, alpha = beta = 0.05, `3+0.03`, 20 ms margin, Hash 64, Threads 1, 14 slots, UHO random, no adjudication, **cap 20,000 pairs**, seed 384, `tools/results/b33-gate`; runner `cli-v0.2.0` pinned; dry run at policy. H1 accepts cluster 2 as a unit and flips `b3proof` on by default with the theta baked on `dev` (the three LMR bases then become the head's, since the head becomes this search); H0 or the cap rejects the cluster as a unit (PLAN rule 6): nothing is baked and the bit sweep says what B.6 may carry. **Direction read beside it, registered here:** 2,000 games `match-fixed`, the same two binaries, seed 385, `tools/results/b33-gate-read`, never an acceptance; **prediction +50 ± 12 Elo** (RAR-S83's, unchanged). **Gate prediction:** H1 (0.9), in under 3,000 pairs (0.7). Stop rule: bounds, cap, book and adjudication never change; an unresolved cap is a rejection. Maintainer-run: the read first (22 min), then the SPRT (a true +50 resolves in about an hour).

## Result / disposition

**Played 2026-09-27** (both on an idle host, 0 faults, recounts equal, hashes and fingerprints as registered): direction read **+48.1 ± 9.5 Elo** (+78.0 ± 15.2 nElo; 650-975-375, [20, 150, 430, 335, 65]); **SPRT H1 accepted at 804 pairs**, 1,608 games, **+50.5 ± 10.9 Elo** (+79.5 ± 17.0 nElo; 538-764-306, [14, 135, 320, 275, 60]), LLR +2.95. **Cluster 2 accepted as a unit**: block-2 theta baked into all 36 coordinates on `dev` and `b3proof` made a default feature in `f53ca7d`; the default build now reads **12,897,901 / EBF 2.523** (equal to the gate binary's bench), the legacy `--no-default-features` search 7,601,220 / EBF 2.474 unchanged, and the previous head's search (`--no-default-features --features b2core`) reads 5,640,715 / EBF 2.411 with the three shared LMR bases at their fitted values.

## Conditional lesson

**Calibration:** read predicted +50 ± 12, measured +48.1: hit. Gate H1 (0.9) and under 3,000 pairs (0.7): both hit, at 804. The one prediction that failed on this cluster was the first head read after block 1 (+20 ± 12 against +44), whose lesson, that B.2.7's and B.3's mechanisms do not overlap, is in RAR-S82.

## Source

PLAN B.3.4, rule 6; RAR-S83; RAR-S82; RAR-S79
