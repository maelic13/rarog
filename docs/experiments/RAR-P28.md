# RAR-P28 — B.7.2 candidates 1 and 2, pooled-PGO NPS, and the re-profile, COMPLETE 2026-10-02

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**B.7.2 candidates 1 and 2, pooled-PGO NPS, and the re-profile, COMPLETE 2026-10-02.** Four PGO pext builds per arm: `dda5112` (base), `5fe42cf` (candidate 1, the quiet-stage early exit), `0d95763` (candidate 2, the in-place scored list), all bench 11,171,726. `nps_multibuild.ps1`, 20 interleaved cycles, 3 repeats, `bench 13`, idle host (3.8% CPU), one overnight run (`analysis/artifacts/b72-nps-c1/overnight.ps1`). Re-profile of `0d95763` (SHA-256 `8b820e47…a084c`) by RAR-P27's protocol. **Predictions, frozen in RAR-P27:** +4% to +9% and +1% to +3%.

## Result / disposition

**Candidate 1 +8.85%** (95% bootstrap CI +8.17% .. +9.48%). **Candidate 2 +6.93%** over candidate 1 (CI +6.38% .. +7.68%). Both accepted against the +0.5% floor. Re-profile: 231,374 samples against 256,177 for the same nodes (−9.7%); move ordering −6.50% and `vcruntime` copies −4.23% of the old total, no other region beyond 0.72%; `pick_next` 6.26% remains. **Calibration:** candidate 1 hit, at the upper end. Candidate 2 missed in magnitude (×2–3), an instrument miss: the caller table credited the copies with 2.92% where the re-profile shows 4.23% removed, and `bench` gains more than the cohorts.

## Conditional lesson and retry trigger

NPS only, no Elo; behaviour-neutral work closes on deterministic and performance qualification. Candidate 3 (the scan layout) qualifies on its share and waits on a local-speedup falsifier (B.7.2.9).

## Source

PLAN B.7.2; `analysis/b72_profile_2026-10-01.md`; `analysis/artifacts/b72-nps-c1/`, `analysis/artifacts/b72-reprofile/`
