# RAR-S55 — Phase-4 step 4.2 — first differential reading

Indexed under *3. Search and selectivity › Search-oracle observations* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**Phase-4 step 4.2 — first differential reading.** Versioned suite `phase4_suite_v1.epd`, 50 positions across five cohorts drawn from repo sources, fixed depth 8, 1 thread, both engines instrumented to the same counter contract. Rarog at `RAROG_DIAG_SAMPLE_STRIDE=1` (exact); oracle = `hybrid-diag` `de568b3`. Counters normalised by the node ratio (Rarog searches 1.861x the oracle's nodes at equal depth), so `norm` is firings per node searched and 1.00 means "in line with tree size".

## Result / disposition

**Observation.** All three spec invariants pass on both engines, so the join is trustworthy. **Corrected 2026-08-12 — see the note below this table.** Largest normalised divergences: `q_tt_cut` 4.25x, `singular_attempt` 3.21x, `singular_multicut` 2.98x, `probcut_attempt` 2.33x, `move_seen_quiet` 2.11x — and in the other direction `see_prune` **0.18x**, `nmp_cut` **0.22x**, `asp_fail_low` 0.28x, `root_best_changes` 0.31x, `quiet_futility_prune` 0.44x. `nmp_attempt` is 0.94x, so Rarog attempts null move as often and converts it 4.5x less. First-move cutoff rate is **higher in Rarog in every cohort** (86.7–93.2% vs 83.5–87.0%).

## Conditional lesson

Rarog orders *better* than the reference and still loses ~196 Elo to it, which is a fourth independent arrival at the same place as RAR-S52 (ordering is not the defect), RAR-S53 (2.5 plies of depth it cannot use) and RAR-S54 (blind de-selectivity gained +4.06). The selectivity profile differs in shape: five times less SEE pruning (a scope difference — Rarog prunes captures by SEE, the reference prunes quiets), and null move that fires as often and pays far less. This is cluster-selection evidence for **4.7**, and it says the cluster's subject is the *shape* of the selectivity surface rather than its constants. It is not an Elo estimate and no individual counter is credited with anything.

## Source

`analysis/phase4_differential_v1_depth8.txt`; `tools/diag/phase4_differential.py`; PLAN 4.2; record: `analysis/ledger_records_2026-09-14.md`, RAR-S55 (Search and selectivity)
