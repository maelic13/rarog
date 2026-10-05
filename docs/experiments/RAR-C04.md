# RAR-C04 — Aspiration mate-score re-search could fail to terminate

Indexed under *7. Correctness and protocol lessons* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment or failure mode

Aspiration mate-score re-search could fail to terminate; capture cutoffs could train quiet correction.

## Disposition

Fixed and regression-covered.

## Conditional lesson / coverage

Rare control-flow and attribution faults can contaminate root/time/history together. Retain deterministic tests before strength gates.

## Source

`CHANGELOG.md` 2.3.1
