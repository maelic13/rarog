# RAR-S70 — Root-only LMR relief, 1536/1024 ply — REGISTERED, NOT YET RUN

Indexed under *3. Search and selectivity › Search-accuracy decomposition* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**Root-only LMR relief, 1536/1024 ply — REGISTERED, NOT YET RUN.** `lmr_reduction_units` is not passed the ply and `reducible` has no `ply == 0` term, so **the reduction formula cannot see the root**. From the third root move onward an alternative is searched at REDUCED depth and can displace the incumbent only by beating alpha *while reduced*. Measured mean root reduction **2.90 ply**. This subtracts 1536/1024 (1.5 ply) at ply 0 only; nothing else changes. Arm A `rarog-46root`, bench **6,977,070 / EBF 2.466**. Arm B `rarog-46base`, bench **7,467,143 / EBF 2.477** — the accepted head. Both arms built from ONE tree differing only in the parameter default, and the candidate's fingerprint reproduces the value measured through `--rset LmrRootRelief=1536` on the base binary exactly, so the default path and the option path agree. Final-PGO both, `3+0.03`, 1T, 64 MB, paired UHO, RAR-M13 adjudication. **Registered bounds `[0,3]` nElo, cap 80,000, fixed before any games.** Stop rule: run to an LLR boundary or the cap; **no early stop on a point estimate**, per RAR-S61.

## Result / disposition

**ACCEPTED — H1 at `[0,3]` nElo. Elo +2.33 +/- 1.85, nElo +3.58 +/- 2.85, LOS 99.30%, LLR 2.95 over 56,928 games in 9h42m. Ptnml [1222, 6843, 12030, 7070, 1299], PairsRatio 1.04, DrawRatio 42.26%.** Merged to `dev`; new accepted head fingerprint **6,977,070 / EBF 2.466**. Second accepted gain of the phase, after 4.7c's +15.56.

## Conditional lesson

**The prior is a zero-game instrument, and it is the first one in this phase that replicated.** `tools/diag/answer_compare.py` compares what the two searches RETURN, at held-constant evaluation. Final-move agreement with the oracle, relief 0 -> 1024 -> 1536: **d10 62/66/70%, d12 66/72/78%, d14 72/78/80%** — monotone in the parameter at every depth, and 1536 is the peak (2048 gives 76% at d12, 78% at d14). Root revisions move toward the oracle's at every depth (d14 1.62 -> 1.78 against 2.28), which is the mechanism doing exactly what it was built to do. And it costs nothing: **6.6% FEWER bench nodes**. ⚠ **RAR-S68 measured an UNCONDITIONAL LMR relief at −1.40 ± 6.24 — dead flat.** This is the same family and the adjacency must be stated. The distinction is population: RAR-S68 relieved every reduction in the tree, this one relieves only ply 0, which is ~0.3% of nodes and 100% of the answer. That is a real distinction and it is also exactly the kind of story that has been wrong before. ⚠ **Agreement with the oracle is a PROXY, not the objective**, on n = 50 positions. RAR-S64 is the standing warning: a mechanism with a clean bench signal measured exactly zero in games. ⚠ **Gated ALONE because the cluster does not compose.** 4.8.1's `LmrMinReducedDepth=1` scores 78% -> 70% when combined with this at d12, below either alone. Two members of one cluster interfering is why the fitted configuration is relief-only. ⚠ Two screens of this candidate were previously recorded as NULL results by a dead `--rset` wire, and one depth-replication was void because `cargo test --all-features` left a **texel** binary in place — the manifest says texel bypasses the eval caches and must not be used for strength. Both corrections are in `analysis/answer_harness_rset_correction.md`.

## Source

`tools/test_engines/rarog-46{base,root}-pext-pgo.exe`; branch `p46-root-relief`; recipe = `lmr_root_relief` default `0 -> 1536` in `src/params.rs`, one line; RAR-S68; RAR-S64; PLAN 4.6c; record: `analysis/ledger_records_2026-09-14.md`, RAR-S70 (Search and selectivity)
