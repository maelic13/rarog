# RAR-P32 — The pooled-PGO NPS read, shortened — study COMPLETE 2026-10-02

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**The pooled-PGO NPS read, shortened — study COMPLETE 2026-10-02** (the maintainer found twenty cycles, about 43 minutes an arm, too long). Sixteen existing pext PGO builds in four source groups (the c2 base pool, A's clean pool, A's scratch-tree pool as a null pair for it, I's pool), six interleaved cycles of `bench 13 3`, every run archived (`tools/results/nps-method-20261002/`). No prediction was frozen here; the stated expectation was that twenty cycles do not limit precision.

## Result / disposition

On the three quiet cycles: noise between readings 0.27% for best of 3 (0.34% best of 2, 0.65% one run, 1.32% median of 3; the three runs of a reading differ by 3.3% on average); builds of one source 0.13%. With four builds an arm a delta's 95% interval is ±0.36% at 3 cycles, ±0.30% at 6, ±0.25% at 20. Load arrived in cycle 4 and moved whole arms by 2% to 4% in two cycles; over all six the null pair read +0.30% by means and **−0.05% by per-build medians**. **Adopted (PROCESS):** four builds an arm, six cycles of `bench 13 3`, the difference of the arms' means of per-build medians with a t-interval on the per-build medians, a cycle more than 1% off its arm's median counted as disturbed and the read repeated at three or more; about 14 minutes. **Made two steps by the maintainer the same day:** two cycles first (about 6 minutes, ±0.41%), accepted at +0.9% or more and closed at +0.1% or less; only in between, four more cycles and the +0.5% rule.

## Conditional lesson and retry trigger

The builds set the limit, not the cycles, and the reduction (best of 3) matters more than either. `nps_multibuild.ps1`'s bootstrap resamples readings and is not the uncertainty. One session read A at +2.1% and I at +0.8% over A on its quiet cycles against RAR-P31's +1.69% and +0.37%: about two standard errors, unexplained, both prefetches; RAR-P31's verdicts stand. The tool must archive runs and report the new estimate before the first use (B.8).

## Source

`analysis/nps_method_study_2026-10-02.md`
