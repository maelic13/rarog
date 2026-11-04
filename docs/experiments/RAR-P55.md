# RAR-P55 — The `sse42` tier against `base`: pooled PGO NPS on the 5950X, indicative of what older CPUs gain — READ 2026-11-04: +8.06%

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** 2026-11-04, PLAN E.3.2's last check; the agent ran it
  with the maintainer's go-ahead on an idle host.
- **Kind:** performance qualification of a new build tier. The engine
  source is unchanged between the arms; both bench the development
  fingerprint, 14,749,043.
- **Question.** How much faster is the new `sse42` tier (x86-64-v2:
  SSE4.2, SSSE3, POPCNT) than `base` (x86-64)? The 5950X runs both, so the
  read measures the code generation, not an older CPU: it indicates what a
  Nehalem-to-Ivy-Bridge or Bulldozer-class CPU gains, and does not measure
  one.
- **Registration.** None beyond PLAN E.3.2, which named the read as
  indicative and set no accept rule: the tier ships either way, for the CPUs
  it serves. No prediction was frozen, so there is no calibration.

## Arms

Two pools of four PGO builds by `tools/nps_build_pool.ps1`, from `8a1c022`,
clean (engine inputs those of `ef8d60e`), MSVC, `rustc 1.99.1`. Location:
`analysis/artifacts/e32-sse42-nps/`.

| Pool | Recipe | First hashes |
|---|---|---|
| `base` | `cargo xtask build --tier base` | `B0FAEA64`, `D18BAE68`, `49E261EC`, `F240F811` |
| `sse42` | `cargo xtask build --tier sse42` | `92D794F7`, `666C6CFC`, `FE220A5F`, `6F76F823` |

## Read

`python tools/nps_read.py --base …/base --cand …/sse42 --out …/read`, step 1
(two cycles of `bench 13 3`). Host idle at 1.2% CPU, quiet after 65 s; 1.4%
and 1.1% before the cycles, 1.0% after; no disturbed cycle. The first
attempt stopped before measuring: `nps_read.py` read only
`manifest-pext.txt`, fixed in `145ab14` (one `manifest-<tier>.txt` per pool).

| Arm | Per-build medians (n/s) | Level |
|---|---|---|
| `base` | 2,697,584, 2,696,352, 2,695,126, 2,700,548 | 2,697,402 |
| `sse42` | 2,914,256, 2,926,688, 2,901,185, 2,916,875 | 2,914,751 |

**`sse42` over `base`: +8.06%, 95% t-interval +7.57% to +8.55%** (t 2.447,
df 6), accepted by step 1. The `sse42` build carries 237 hardware `popcnt`
where `base` uses software popcount (`cargo xtask verify-isa`).

## Reading it

- It is code generation on a Zen 3, not a pre-Haswell CPU: the gain on the
  CPUs the tier exists for may differ.
- It is larger than the `avx2`-over-`base` +4.59% of RAR-P20, read on an
  older engine (2.3-era) in another read. The two are not comparable, so this
  says nothing about `sse42` against `avx2`; E.3.3's tier ladder reads every
  tier of the current engine in one design.
- Not Elo: NPS has no exchange rate to strength here beyond RAR-M20's rough
  constant, and the tier changes no search.
