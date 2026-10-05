# RAR-M19 — Audit of the SEE / move-ordering piece-value scale, 2026-09-05

Indexed under *2. Measurement, harness and tuning* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**Audit of the SEE / move-ordering piece-value scale, 2026-09-05.** Prompted by the `cross-engine-board-v1` benchmark's threshold-SEE column not being comparable with Basilisk or Manta. Zero games; the evidence is the source, `git log -S`, and the two peer implementations.

## Result / disposition

**`piece_value()` has not moved since the initial commit while the evaluator was refit four times underneath it.** Three vectors sit on consecutive lines of `src/eval.rs`: `MG_VAL` = 88/394/418/537/1131 and `EG_VAL` = 123/239/290/486/930, both Texel-fitted inside the 1,218-slot surface, and `PIECE_VALUES` = 100/320/330/500/900/`MATE_SCORE`, traced by `git log -S` to `d3f58a2` "Version 1.0.0" (2026-05-22) and never tuned since. RAR-E05, RAR-E06, RAR-E08 and RAR-E12 each moved the evaluator's material and left it alone. Measured blast radius in `src/search.rs`: **10 executable `see_ge` / `see_ge_quiet_aware` sites plus 7 direct `piece_value` uses** in MVV-LVA scores, promotion ordering bonuses and the qsearch delta margin `stand_pat + piece_value(Queen) + 200 < alpha` -- a margin sized on a 900-cp queen while the evaluator's queen is 1131 mg.

## Conditional lesson and retry trigger

**Operating rule 7 already required this audit and it was never run:** after an HCE changes, cp-valued search consumers are audited and, if justified, fitted separately. The peers show the two coherent designs and Rarog has neither -- **Manta** parameterises SEE (`see.PieceValues` as a comptime argument), injects the contract's 100/300/300/500/900/20000 into its benchmark and passes its own fitted `mg_val` = 84/323/364/514/1085 in production; **Basilisk** hardcodes a dedicated `SEE_VALUES` table in `board.cpp` that already equals the contract, so its bench needs no injection. Rarog reuses a legacy evaluation constant and can do neither. **This is not a regression**: every accepted SPRT was played with these values, so current strength already includes them; the open question is whether the coupling costs Elo. Owner **4.15.3** (zero-game audit), **4.15.4** (give the values an owner and a tunable surface, gate it), **4.15.5** (restore the benchmark column), with the vector joining **4.16**'s SPSA surface if 4.15.4 exposes it.

## Source

`src/eval.rs:30-33`; `benches/board.rs` header; `D:/code/manta/src/eval/hce.zig:125`; `D:/code/basilisk/src/board.cpp:1604`; PLAN 4.15.3-4.15.5; record: `analysis/ledger_records_2026-09-14.md`, RAR-M19 (Measurement, harness and tuning); record: `analysis/ledger_records_2026-09-14.md`, RAR-M19 (Measurement, harness and tuning, part 2)
