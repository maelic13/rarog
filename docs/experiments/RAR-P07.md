# RAR-P07 — `origin/arm_fix` wrapped TT clusters in 128-byte Apple-oriented blocks

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

`origin/arm_fix` wrapped TT clusters in 128-byte Apple-oriented blocks.

## Result / disposition

**Unverified when written; now CLOSED and rejected — see RAR-P16,** which measured it on an M4 (-0.12% median, 4/12 paired wins, inside the noise floor) and showed the allocator already returns 128 B-aligned TT bases, so the wrapper is a no-op. No ARM timing existed here and existing cluster alignment already prevents the claimed boundary straddle.

## Conditional lesson and retry trigger

Alignment folklore is not evidence. Compare equal-capacity layouts on actual target topology before retaining a wrapper.

## Source

recipe in RAR-P16 — branch `arm_fix` deleted, do not cite it
