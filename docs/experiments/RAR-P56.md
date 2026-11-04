# RAR-P56 — The universal x86-64 binary against the single-tier pext build: the speed gate of PLAN E.3.3 — REGISTERED 2026-11-04, before the read; READ 2026-11-04: PASS, +0.45%

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-04 by the agent; the maintainer asked
  for E.3.3's implementation and measurements in this session, and the read
  runs only on a host the maintainer has said is free.
- **Kind:** performance qualification of a build form. The engine source is
  the same in both arms, and every tier benches the development
  fingerprint, 14,749,043.
- **Question.** Is the universal binary, whose `pext` tier this host runs,
  as fast as the single-tier `pext` build every measurement and today's
  release use?

## What differs between the arms

The universal binary's `pext` tier is the same source, tier flags and PGO
recipe as the single-tier build, with two differences (PLAN E.3.3,
`analysis/universal_binary_2026-09.md` section 11):
- every function aligned to 64 bytes (`-C llvm-args=-align-all-functions=6`)
  in both PGO phases, so its profile is its own;
- its post-LTO object linked beside three other tiers and a dispatcher, so
  its functions sit at other addresses.

Its hot code is the object rustc produced for that tier's own PGO build;
relinked alone such an object gives the build's text byte for byte
(section 11, check 1).

## Arms

Two pools of four builds by `tools/nps_build_pool.ps1`, from one clean
commit, MSVC, `rustc 1.99.1`, in `analysis/artifacts/e33-universal-nps/`:
- `pext`: `cargo xtask build --tier pext`, the base arm;
- `universal`: `cargo xtask build --universal`, the candidate.

## Rule, fixed before the read

`python tools/nps_read.py --no-regression`, the harness's rule unchanged:
step 1 (two cycles) passes at a delta of −0.1% or better and fails at −0.9%
or worse; between, `--extend` adds four cycles and the read passes above
−0.5%. Disturbed reads repeat by the harness's own rule. A pass makes the
universal binary the x64 release asset (maintainer decision 2026-11-04); a
fail keeps the per-tier assets and, by the maintainer's instruction, reverts
the universal binary's integration.

## Prediction

Delta **−0.1%**, 90% interval −0.6% to +0.4%; P(pass) 0.75. Basis:
Basilisk's same design read −0.29% [−0.73%, +0.16%] over six cycles once its
functions were aligned (BAS-P21), and its assembly route +0.04% (BAS-P24);
its unaligned relink read −0.58% (BAS-P20). Rarog's link differs (rustc's
fat LTO, MSVC's `link.exe` rather than lld), so the transfer is a prior, not
evidence.

## If it fails

Diagnostics only, not a rescue: the `pext` tier's own aligned build against
the single-tier build separates the alignment from the relink, and the
universal binary against that aligned build reads the placement alone.
Neither changes the verdict.

## Result (2026-11-04)

Pools from `e5fceb5`, clean, in `analysis/artifacts/e33-universal-nps/`:

| Pool | First hashes |
|---|---|
| `pext` | `EF3EDF18`, `26E28271`, `D70E6F3A`, `DC63EFDF` |
| `universal` | `4AEF6C1F`, `D91DEB8F`, `690D5C9C`, `D1BEFBE6` |

Host idle at 1.7% CPU, quiet after 65 s; 1.0% and 1.2% before the cycles,
1.3% after; no disturbed cycle.

| Arm | Per-build medians (n/s) | Level |
|---|---|---|
| `pext` | 2,938,107, 2,947,758, 2,936,952, 2,938,118 | 2,940,234 |
| `universal` | 2,947,205, 2,956,346, 2,953,648, 2,956,320 | 2,953,380 |

**Universal over pext: +0.45%, 95% t-interval +0.17% to +0.72%** (t 2.447,
df 6): **PASS**, regression excluded by step 1. By the registered rule the
universal binary becomes the x64 release asset.

**Calibration.** Predicted −0.1% [−0.6%, +0.4%]: a miss in sign and
magnitude, small, just above the interval's top. The prior was Basilisk's,
whose aligned relink read slightly below its asset; here the aligned form
reads slightly faster. The likely difference is that the 64-byte alignment
itself helps this build (Rarog's single-tier functions sit at 16 bytes), but
the registered diagnostics that would separate alignment from placement
were for a failure and were not run, so that is a reading, not a finding.
