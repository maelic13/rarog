# RAR-P27 — B.7.2 whole-search profile, COMPLETE 2026-10-01

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**B.7.2 whole-search profile, COMPLETE 2026-10-01.** Registered in PLAN B.7.2 before any sample (`ab4835a`). The shipped `pext` PGO build at `b45bf6f` with symbols (bench 11,171,726, SHA-256 `1230d569…0d5aea`); ETW over the five frozen cohorts, 600,000 nodes, 5 repeats, 8 kHz; maintainer capture on an idle host. Read with `--scheme search` (`8e9d77e`, corrected `a6fa72c`) and `--scheme board`.

## Result / disposition

256,177 samples, all engine samples resolved. Move ordering 25.4% (`pick_next` 13.4%, `quiet_score` 6.5%), evaluation 21.2%, search node 12.0%, TT 8.2%, board work 18.2%, `vcruntime` 5.1%. A counter build over `bench` puts 60% of the picker's scanned elements after `skip_quiets` with no survivor left. Six 4,104-byte `memcpy` calls per node copy the scored list (2.9%). Two exact candidates are ready (early exit, ceiling about +8.7%; in-place list, about +3.0%). **Calibration:** ordering 29.2% with history, against 6–14%; evaluation 21.2%, against 30–45%. Mechanism misses: the scan after a skip and the per-node copies were not foreseen. The check-query split was an instrument miss, corrected.

## Conditional lesson and retry trigger

NPS only, no Elo. Re-profile after candidates 1 and 2 before deciding the scan layout. Evaluation speed waits for Phase C's rewrite.

## Source

PLAN B.7.2; `analysis/b72_profile_2026-10-01.md`; `analysis/artifacts/b72-profile-2026-10-01/`
