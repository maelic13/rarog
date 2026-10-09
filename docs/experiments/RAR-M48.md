# RAR-M48 — A.8.4 pooled-PGO NPS baseline of the 2.4.0 release head, 2026-09-11

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**A.8.4 pooled-PGO NPS baseline of the 2.4.0 release head, 2026-09-11.** RAR-M41 protocol, `tools/nps_build_pool.ps1` then `tools/nps_multibuild.ps1 -Cycles 10 -Repeats 3`. **Two independent three-binary `pext` PGO pools** built at `5513573` on a clean tree, `tools/results/nps-baseline-240-20260911{,-poolB}/`; all six distinct by SHA-256, every copy verified at **7,601,220**, manifests archived beside them. Run as a **true 3v3 null** (pool A against pool B, same source) rather than the registered single three-build pool. **Expansion declared before the numbers were collected**, per the measurement rules, for a stated reason: a single pool yields a median but never establishes what that median is worth, and this is the figure every speed claim in B and C is quoted against.

## Result / disposition

**Baseline: pooled median 3,189,100 n/s (pool A) and 3,190,438 (pool B) - about 3.19 M n/s. Instrument resolution: null delta +0.04%, 95% bootstrap [-0.23%, +0.15%]**, straddling zero, so the instrument carries no arm-level offset and resolves to roughly **+/-0.2%** at three builds per arm. Pooled best-of 3,208,619 and 3,203,211. Per-build medians span **0.42%** (3,182,430 to 3,195,804), which reproduces the ~0.4% per-binary PGO offset RAR-P17 and RAR-P18 document. Host idle throughout: 1.4% mean, 9% max over 60 s before, 1-7% after.

## Conditional lesson and retry trigger

**The first attempt at this measurement was discarded, and why matters more than the number.** It read 3,168,504 n/s with its three builds spanning **2.5%** - six times the documented per-binary offset. The host then sampled **12-16% mean CPU with spikes to 41%** against the 12% rejection threshold RAR-M43/M44 operate under, with **no user-space process accountable**, consistent with the application-control layer scanning ~40 MB of freshly written executables. It was recorded as held rather than published. **The re-measurement proves the diagnosis rather than assuming it:** pool A's `pext-2.exe`, the same bytes, read 3,109,522 contaminated and **3,182,430** idle - a **2.3%** swing on an unchanged binary, larger than most effects this instrument is used to accept. **An anomalous spread is a host symptom before it is a build finding.** **Do not read 3.19 M as a regression against the 3.22 M in the checkpoint tables.** That figure is a **best-of**; this run's best-of is 3,208,619, which agrees with it to 0.4%. The tables now carry the median with the best-of beside it, because a median is the honest central figure and mixing the two across checkpoints would manufacture a phantom 1% regression at the next comparison.

## Source

PLAN A.8.4; RAR-M41; RAR-M36; RAR-P17; RAR-P18; RAR-M43; RAR-M44; `tools/results/nps-baseline-240-20260911{,-poolB}/`
