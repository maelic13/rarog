# RAR-R10 — Phase-4.9c-i: the powered sizing RAR-R09 said was owed

Indexed under *4. Root search, time management and SMP* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Phase-4.9c-i: the powered sizing RAR-R09 said was owed. Redesigned harness — ONE engine process per cell (no process-start variance), arms ALTERNATING inside it, `ucinewgame` before every rep, 40 pairs x 2 positions per thread count at `go depth 18`, analysed per-pair rather than as independent medians. Bootstrap CIs (20k) and a permutation test against an n=80 null run under the identical protocol.

## Result / disposition

**The harness now works and the answer is NO BENEFIT WHERE IT COUNTS.** The null validates the design: median per-pair ratio **exactly 1.0000**, 39/80 wins, p=0.91 — against the 21.6% swing the n=6 cross-process attempt produced. Against that baseline: **4T time 1.067, nodes 1.28/0.97; 16T time 1.065, nodes 1.00/0.95 — no benefit, if anything harm.** 8T is the only positive cell: time **0.868** (95% CI 0.766–1.013, permutation p=0.168, NOT significant) and nodes **0.845** (CI 0.740–0.944, p=**0.0493**, borderline). Sign tests are null in every cell (46/80 at 8T, p=0.22).

## Conditional lesson and retry trigger

**The effect has the wrong SHAPE across thread counts.** Depth diversity should deepen monotonically — more threads on one iteration means more to gain from spreading them — so 16T should be strongest. Nodes ratios run **1.046 → 0.845 → 0.989** at 4/8/16T, a V rather than a trend, with 16T's CI [0.872, 1.190] straddling 1.0. The lone signal is the middle cell, significant on the quieter metric only and borderline there (p=0.049 uncorrected over six cells and two metrics). ⚠ A first draft argued instead from 'the gate runs at 4T and 4T shows nothing' — the WRONG criterion, since the objective is good scaling with threads rather than a merely workable 4T, so a real 8T or 16T gain would have counted regardless of where a gate is defined. The non-monotone shape refutes the mechanism under either objective; the gate-condition argument would not have. `SmpIterationSkip` therefore stays INERT and no gate is spent — the mechanistic case from RAR-R08 remains true and unconverted. **The harness lesson is the durable part:** pairing WITHIN one process with interleaved arms turned an unmeasurable quantity into one whose null centres on 1.0000, and the null is what licensed reading the arm at all. Nodes-to-depth is also the better metric here — same direction as time, roughly half the spread.

## Source

`src/params.rs`; Plan 4.9c-i
