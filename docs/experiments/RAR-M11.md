# RAR-M11 — Phase-4 consolidated SPSA go/no-go review

Indexed under *2. Measurement, harness and tuning* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Phase-4 consolidated SPSA go/no-go review. Proposed surface: 30 coordinates, three fixed architecture switches, 10,000 iterations × 32 games (320,000 games), about 79 hours, complete final theta. Setup and resume mechanics were validated; no games started.

## Result / disposition

**Canceled before launch and removed in 2.3.2.** The architecture bundle had failed to accumulate, its best node-saving pair had no material Elo prior, and the three proposed fixed switches were unaccepted. A valid schedule could at most refine a low-prior HCE surface immediately before NNUE.

## Conditional lesson and retry trigger

Perform the expected-value review before investing in schedule optimization. SPSA is a local constant optimizer, not the missing mechanism for a 50–100 Elo target; clean setup, sunk preparation and a larger coordinate count do not create that prior. The next broad fit is post-NNUE and must select its coordinates/horizon from new activation and curvature evidence.

## Source

`PLAN.md` §2–3; `tools/spsa_convergence_model.py`; `tools/spsa_configs/README.md`
