# RAR-S21 — Phase-4.1 diagnostic `bench 13`, 1T, deterministic sampled interaction map on the retained 6,502,902-node …

Indexed under *3. Search and selectivity › Rejected, neutral or deferred* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Phase-4.1 diagnostic `bench 13`, 1T, deterministic sampled interaction map on the retained 6,502,902-node baseline.

## Result / disposition

**Observation:** first-move cutoff 88.17%; LMR re-search 1.38%; sampled best move first 81.44%; TT sample hit 63.05% with 275 usable cuts and 113 contradictions; qsearch stores stand-pat/qmove/tail exact/tail upper 913/240/6/514; NMP verification pass/fail 533/7 with 83 nested attempts; pruning overlap 0.47%; 145,372 of 283,590 correction updates were capture-attributed.

## Conditional lesson and retry trigger

Under this corpus, ordering, depth-0 TT authority, nested NMP verification and correction attribution deserve priority; the low observed pruning overlap gives little evidence that simple LMP/futility deduplication is a major prize. These counters are diagnostic priorities, not Elo estimates, and require game/TC validation.

## Source

`tools/diag_search_quality.ps1`; Plan 4.1–4.6
