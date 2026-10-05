# RAR-S68 — SUPERSEDED by B.2, never run (B.1, 2026-09-14): the switch or parameter this arm needed was removed as inert …

Indexed under *3. Search and selectivity › Search-accuracy decomposition* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**SUPERSEDED by B.2, never run (B.1, 2026-09-14): the switch or parameter this arm needed was removed as inert; its question is answered inside B.2's cluster and SPSA or not at all, no retry trigger.** **Unconditional LMR-reduction relief — REGISTERED, NOT YET RUN.** Subtracts a fixed **336/1024 ply (15% of the 2.19-ply mean reduction)** from every LMR reduction. The DIRECTIONAL form of what RAR-S54 and RAR-S64 measured, replacing the symmetric form RAR-S67 disproved. Arm A `rarog-47relief` `5dbeb52`, bench **6,539,063 / EBF 2.449**. Arm B `rarog-47base2` `23b21b8`, bench **7,467,143 / EBF 2.477** — the accepted head. Final-PGO both, `3+0.03`, 1T, 64 MB, paired UHO, RAR-M13 adjudication. **Registered bounds `[0,3]` nElo, cap 60,000, fixed before any games.**

## Result / disposition

*Pending.*

## Conditional lesson

**The magnitude comes from the evidence, not from the sweep, and the distinction matters after RAR-S64.** Mean reduction is 2.19 ply = 2,243/1024; RAR-S54 shifted its twelve selectivity constants by **15%**, which is 336. That the sweep's best `cut/node` (0.0890 against 0.0853 at zero) also lands at 336 is corroboration only — RAR-S64 adopted a value picked off a clean bench sweep and it measured exactly zero in games. ⚠ **Directly tests whether RAR-S54's headroom survived 4.7.** That +4.06 ± 3.71 over 14,196 games was measured against the 2.3.1 head; 4.7 has since banked +15.56 Elo of structural de-selectivity, so some or all of it may already be spent. A null here is therefore informative rather than merely disappointing: it would say the blind-shift headroom is gone. ⚠ **Not a bake candidate for the scalar itself.** PLAN is explicit that RAR-S54 licenses a structural rework and does not license shipping a uniform scalar. If this gates positive the right response is to find where the reduction is systematically too aggressive, as 4.7 did for ProbCut — not to ship 336 and call it done.

## Source

`tools/test_engines/rarog-47{base2,relief}-pext-pgo.exe`; branch `p410-lmr-relief`; RAR-S54; RAR-S64; RAR-S67; RAR-S57; record: `analysis/ledger_records_2026-09-14.md`, RAR-S68 (Search and selectivity)
