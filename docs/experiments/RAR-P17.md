# RAR-P17 — Phase-4 step 4.5.1 — typed per-ply search context, pooled-PGO NPS

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**Phase-4 step 4.5.1 — typed per-ply search context, pooled-PGO NPS.** Replaces three parallel `[_; MAX_PLY]` arrays (`stack_moves`, `stack_pieces`, `stack_static_eval`) with one `NodeContext` per ply. Pure representation change: no new fields, no behaviour. Three independent PGO builds per arm, 10 interleaved cycles, `bench 13 3` per sample, machine idle. Baseline pool includes `rarog-47c-only`, whose engine source is identical to the pre-refactor head.

## Result / disposition

**Behaviour-neutral and NPS-neutral — a clean null.** `bench 13` **6,922,439 / EBF 2.451** on all three candidate builds, exactly the accepted fingerprint. Pooled median NPS base 3,153,730 against cand 3,157,326, **delta +0.11%, 95% bootstrap CI −0.14%..+0.48%**; pooled best-of delta −0.05%. Per-build medians span 3,150,860–3,160,932 across BOTH arms, i.e. the between-build spread swamps the between-arm difference. fmt, all-feature clippy, 248/248 all-feature release and 242/242 debug all clean.

## Conditional lesson and retry trigger

**The locality argument did not pay, and that is recorded in the code rather than quietly dropped.** The change was motivated by every continuation-history lookup reading move and piece at the same ply from two arrays; merging them into one record produced no measurable speed-up, and the whole CI sits inside this machine's ±0.5% floor (RAR-P13). The step is still correct to land, because 4.5.1's purpose is the substrate 4.5.2–4.5.4 consume, not throughput — but nobody should later cite this refactor as an NPS win. ⚠ A null here is also the pass condition: PLAN 4.5.1 gates on exact fingerprint **and** pooled-PGO NPS, and 'no regression' is what a representation change owes.

## Source

`tools/nps_multibuild.ps1`; `tools/test_engines/rarog-451{base-a,base-b,ctx-a,ctx-b,ctx-c}-pext-pgo.exe`; RAR-P13; PLAN 4.5.1
