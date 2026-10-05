# RAR-S65 — SUPERSEDED by B.2, never run (B.1, 2026-09-14): the switch or parameter this arm needed was removed as inert …

Indexed under *3. Search and selectivity › Search-accuracy decomposition* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**SUPERSEDED by B.2, never run (B.1, 2026-09-14): the switch or parameter this arm needed was removed as inert; its question is answered inside B.2's cluster and SPSA or not at all, no retry trigger.** **Audit finding 3 — bound how far a killer can travel. REGISTERED, NOT YET RUN.** `killers[ply]` is cleared once per search, so within a search a ply's killers persist across every sibling subtree reaching that depth and a node can inherit killers from a positionally unrelated subtree. The candidate clears the GRANDCHILD's slot on node entry, bounding the distance. Arm A `rarog-45killer` `1e2a30d`, bench **6,556,136 / EBF 2.443**. Arm B `rarog-45head` `1155ec3`, bench **7,467,143 / EBF 2.477** — the accepted head after Cluster A closed. Final-PGO both, `3+0.03`, 1T, 64 MB, paired UHO, RAR-M13 adjudication. **Registered bounds `[0,10]` nElo, cap 20,000, fixed before any games.**

## Result / disposition

**STOPPED at 12,522 games, not resolved, NOT promoted.** Elo **+2.80 ± 3.90**, nElo **+4.38 ± 6.09**, LLR −0.65 of ±2.94, drifting toward H0. Stopped deliberately by the operator once the projection was clear; recorded as a STOP, not a result, following RAR-S54's precedent. RAR-M10 predicted the drift throughout (−0.66 against −0.65 observed at 10,680). ⚠ **The bracket was wrong, and that is the finding.** `[0,10]` does not merely resolve slowly against a true +4.4 — it drives it to **H0 in ~35k games**. This gate was registered at bounds configured to reject what it was measuring, as were RAR-S61 and RAR-S64. Fishtest uses `[0,2]` STC / `[0,1]` LTC, narrow and anchored at zero; `[0,3]` accepts a true +4 in ~47k games and is RAR-M10's fitted regime. **The +2.80 point estimate is therefore not evidence the mechanism works, and it is not evidence it fails either — the instrument was pointed wrongly.** Rejected on the registered rule; re-testable at `[0,3]` if it is ever worth an overnight run.

## Conditional lesson

**The bench signal is the strongest this cluster produced, and that is explicitly not why it is being gated.** First-move cutoff moves **88.04% → 88.70%**, +0.66 points, where RAR-S59's rejected candidate moved it 0.05 and RAR-S64's adopted-then-dead mechanism moved it 0.14. ⚠ Two cautions carried from this cluster's failures. `cut/node` is **flat** (0.0853 → 0.0856), the exact signature RAR-S59 used to unmask a disguised selectivity increase — so the tree shrinking 12.2% is not self-evidently an ordering gain. And it makes the engine **more** selective, the one direction RAR-S53/S54/S55 and 4.7 all contradict. RAR-S64 settled how much bench proxies are worth here: a mechanism adopted on a clean proxy measured exactly zero in games. `[0,10]` chosen from RAR-M10 rather than habit — it resolved RAR-S64 in 8,088 games where `[3,10]` had spent 16,000 without moving.

## Source

`tools/test_engines/rarog-45{head,killer}-pext-pgo.exe`; **Arm A recipe** (branch deleted): set `killer_clear_grandchild = 1` in `params.rs`; rebuild and confirm `bench 13` = **6,556,136 / EBF 2.443**. `analysis/code_audit_2026_08_19.md`; RAR-S59; RAR-S64; RAR-M10
