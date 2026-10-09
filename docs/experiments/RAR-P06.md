# RAR-P06 — `origin/arm_fix` added AArch64 `PRFM PLDL1KEEP` and hoisted two HCE `LazyLock` accesses

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

`origin/arm_fix` added AArch64 `PRFM PLDL1KEEP` and hoisted two HCE `LazyLock` accesses.

## Result / disposition

**Unverified when written; now CLOSED — both halves have had their target-native A/B.** The prefetch was isolated, ported and ACCEPTED at +1.42% (RAR-P10, RAR-P11). The HCE `LazyLock` hoists were unfrozen and measured in RAR-P16: **+0.12% median, 5/12 paired wins, inside the noise floor** — merged for consistency with the third hoist site dev already had, with no speed claim. The branch's combined +2.51% x64 figure was never reproduced as such.

## Conditional lesson and retry trigger

Causality was correctly refused until each half was measured alone; splitting a combined patch is what let the real +1.42% be told apart from a null.

## Source

`src/tt.rs`, `src/eval.rs` on dev; recipe in RAR-P16 — branch `arm_fix` deleted, do not cite it
