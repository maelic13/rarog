# RAR-P13 — Phase-4.8f: identical-binary calibration on macOS ARM64 — the null pair PLAN item 5 requires, run AFTER the …

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Phase-4.8f: identical-binary calibration on macOS ARM64 — the null pair PLAN item 5 requires, run AFTER the prefetch was already accepted, to audit it. Same 12-round interleaved `bench` protocol as RAR-P11 with the SAME binary in both slots, first-slot ordering preserved so any first-in-round penalty would show.

## Result / disposition

**NO ORDERING ARTIFACT; the +1.42% survives.** Slot A (first) median 5,269,778 (MAD 0.28%), slot B (second) 5,278,329 (MAD 0.12%): slot bias **+0.162%** with slot B winning only **6 of 12** rounds, i.e. a coin flip, against 12/12 in the real A/B. The decisive cross-check is same-slot: slot A held the BASELINE at 5,194,011 in RAR-P11 and holds the CANDIDATE at 5,269,778 here, **+1.46% in the same slot** — reproducing the effect with only the binary changed. Conservatively subtracting the (absent) bias still leaves **+1.26%**. Machine noise floor: **MAD 0.12–0.28%**, so an effect must clear roughly ±0.5% to be resolvable here.

## Conditional lesson and retry trigger

**Calibrate the harness even when the result already looks clean.** The A/B was accepted on 12/12 with zero overlap, which is strong — but it ran baseline-first every round, and nothing in that design could distinguish a real gain from a first-in-round penalty. The null pair is what separates them, and it cost two minutes against a conclusion already banked. Also recorded: one slot-A round read 4,893,079, **−7.2% below median**, on a fanless Air. A mean-of-N estimator would have absorbed that into the answer; median/MAD with interleaving is why it did not. This is the macOS ARM64 performance anchor, and future ARM arms are read against the ±0.5% floor it establishes.

## Source

Plan 4.8f
