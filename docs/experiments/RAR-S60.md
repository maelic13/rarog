# RAR-S60 — Phase-4 steps 4.5.3/4.5.4 — the six deferred per-ply fields, disposed with zero games

Indexed under *3. Search and selectivity › Search-accuracy decomposition* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**Phase-4 steps 4.5.3/4.5.4 — the six deferred per-ply fields, disposed with zero games.** 4.5.1 deliberately landed no speculative state and handed six fields to the sub-items owning their consumers. Each was built behind a default-off switch where it had one, verified to reproduce `bench 13` 7,467,143 in the off position, and measured on the 40-position bench corpus at stride 1 using RAR-S59's method — cutoff RATE **and** cutoffs-per-node, because in this engine a history/reduction change is never only an ordering change.

## Result / disposition

**Two adopted, four rejected.** ADOPTED — *continuation key*: derived in `push_move`, and it exposed the ProbCut piece desync. *Prior reduction*: reduce 512/1024 ply less when the parent move was itself reduced. Cutoffs per node rise **faster** than nodes (+4.5% against +1.6%) and first-move cutoff improves **88.04% → 88.18%** — the mirror image of RAR-S59's rejected candidate, and the signature of a real gain rather than a disguised selectivity change. At 1024 the effect is larger still (+7.5% cutoffs/node, 88.27%) but costs +6.1% nodes; 512 is the conservative categorical default. `bench 13` 7,467,143 → **7,587,235**. REJECTED — *cutoff count*: **completely inert**, byte-identical output at 512 and 1024, because a beta cutoff BREAKS Rarog's move loop, so a per-visit count is 0 or 1 by construction. *Statistical score* and *TT/PV evidence*: already exist as `quiet_hist` and `tt_pv`, threaded into the reduction; both are node-LOCAL, read at the ply that computes them, so per-ply storage would add lifetime nothing consumes. *Previous-PV following*: structurally redundant — the previous iteration's PV move is stored in the TT and already emitted first by `Stage::TtMove`.

## Conditional lesson

**The deferred-field list was a hypothesis, and measuring it was worth more than implementing it.** Four of six were wrong for this engine: one inert by a control-flow difference, two already present under other names, one duplicated by existing ordering. Had 4.5.1 landed all six as the plan listed them, the cluster would carry four unused fields, one of them a mechanism that cannot fire. ⚠ **Correction, same day:** the cutoff-count rejection above is of the IMPLEMENTATION, not the mechanism, and the original wording overstated it. Stockfish's `cutoffCnt` semantics are knowable, not guesswork: it resets `(ss+2)->cutoffCnt = 0` on node entry — zeroing the GRANDCHILD's counter, not its own — so a ply slot accumulates cutoffs across every sibling visit between resets, which is how it exceeds 3. It is a recency signal for "this ply has been cutting a lot lately", not a per-node count. The per-visit reset used here is what made it 0-or-1 and inert. The mechanism remains unpursued for a different and better reason: its consumer is `if ((ss-1)->cutoffCnt > 3) r++`, a selectivity INCREASE, which is the direction RAR-S53/S54/S55 and 4.7's +15.56 all contraindicate. Handed to 4.10, which owns second-pass selectivity, with the counter-point recorded: it is a CONDITIONAL increase aimed at plies that demonstrably cut often, which is not the same as a blanket one. Switches built, measured and **removed** — 4.5.5 inherits no dormant knobs.

## Source

`src/search.rs` `lmr_reduction_units`, `NodeContext`; `tools/diag/bench_counters.py`; RAR-S59; PLAN 4.5.3/4.5.4
