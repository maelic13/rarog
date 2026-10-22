# RAR-P39 — C.11 pooled NPS, the C head against the B.9 head — REGISTERED 2026-10-21, before any read; READ 2026-10-21: +28.92%

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**C.11 pooled NPS, the C head against the B.9 head — REGISTERED 2026-10-21, before any read.** PLAN C.11's "pooled NPS": what Phase C, as shipped, did to bench speed. **Base pool** `analysis/artifacts/b9-checkpoint/pool`: RAR-P35's four pext PGO builds of `bae803f` (engine source `24aefb4`, bench **11,171,726**, rustc 1.98.1). **Candidate pool** `analysis/artifacts/toolchain-1.99/pool-1.99.1-r2`: RAR-P37's four pext PGO builds of `fb71b2a` (engine source `9fb525e`, unchanged since, bench **13,987,223**, rustc 1.99.1). Each pool is read at its own fingerprint from its manifest. The two arms differ in source and compiler: this is the C head as it ships against the B.9 head as it shipped, and RAR-P37's −1.63% for the compiler alone is the known part of the difference. **Read:** `python tools/nps_read.py --base analysis/artifacts/b9-checkpoint/pool --cand analysis/artifacts/toolchain-1.99/pool-1.99.1-r2 --out analysis/artifacts/c11-checkpoint/nps-read --label "RAR-P39"`, step 1, as the first step of `analysis/artifacts/c11-checkpoint/run_all.ps1`, on a host the read's guard calls idle. A measurement, not an acceptance: the tool's accept or close line does not apply; a disturbed read is repeated once by the driver, then reported unread.

**Prediction, frozen 2026-10-21 before any read:** **+28%**, 80% band **[+18%, +38%]**. Basis: C.12's pass read +22.47% over the C.13 head at one fingerprint; the evaluation units between B.9 and C.12 cost per node (C.4.3 gained +2.57% by removing the x-ray loop, the refit cycles −0.48% and −0.92%, C.2's and C.3's costs unread); large pages +2.48% and C.12's third pass +0.41% came after; the compiler −1.63%. RAR-P35's base level (2,326,903 nps on 2026-10-03) against the C head pools' 2.95–3.11 M in today's reads says about +30%, but host levels move by several percent between days. Evidence layer: bench NPS, not Elo; bench NPS is not in-game NPS (RAR-M73 reads that).

## Result / disposition

**Read 2026-10-21 21:02–21:08 local** by `run_all.ps1`'s first step (`analysis/artifacts/c11-checkpoint/nps-read/{raw.json,read.txt}`, `step-1-nps-1.log`; the read's guard 0.0% at the start, CPU 2.0% and 3.1% before the cycles, 1.4% after): per-build medians base 2,345,034 / 2,345,295 / 2,343,561 / 2,340,757, candidate 3,018,399 / 3,019,372 / 3,021,978 / 3,025,584; levels **2,343,662 against 3,021,333**; **+28.92%**, 95% **+28.72% .. +29.11%** (t 2.447, df 6); no disturbed cycle. The tool's line "accepted by step 1" is its acceptance verdict and does not apply to a measurement. The base pool read 0.7% above its RAR-P35 level (2,326,903), inside the day-to-day drift the basis named. In the games the bench figure carried at one thread (RAR-M73: +27.8% over the B.9 head) and not at four (RAR-M74: −10.3%), which this single-thread bench cannot see.

**Prediction calibration, appended after exposure.** Predicted +28% in [+18%, +38%]: **a hit**, 0.9 above the centre; the chain of recorded speed changes and the levels both pointed here.

## Conditional lesson and retry trigger

**Phase C, as shipped, is 29% faster in bench than the B.9 head**, the compiler's −1.6% included; C.12's speed pass carried it. **Retry trigger:** E.1's attribution checkpoint; a bench read at four threads belongs to Phase D (RAR-M74's finding).

## Source

PLAN C.11; RAR-P35; RAR-P37; C.12's speed pass (`analysis/c12_speed_pass_2026-10-17.md`)
