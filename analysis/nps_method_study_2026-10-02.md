# The pooled-PGO NPS read, shortened — RAR-P32

**Status: study complete 2026-10-02; the revised read is in PROCESS (*Harness
notes*, the NPS paragraph).** The maintainer found the 20-cycle read (about
43 minutes an arm) too long. This measures what its length buys. Raw runs,
scripts and outputs: `tools/results/nps-method-20261002/` (git-ignored).

## What was measured

Sixteen existing pext PGO builds in four source groups, read in six
interleaved cycles of `bench 13 3`, direction alternating, every run archived
(`study.py`, `raw.json`; 288 runs, 5.07 s each, every run at fingerprint
11,171,726, every build matching its pool manifest):

- `base`: the pool before the TT prefetch (`analysis/artifacts/b72-nps-c1/c2`);
- `A`: the accepted prefetch, clean pool (`b72-audit-nps/a/pool`);
- `A-screen`: the same code from the audit's scratch tree, an independent
  pool, so `A-screen` against `A` is a null pair;
- `I`: the reverted correction-slot prefetch over A (`b72-audit-nps/i/pool`).

The host was quiet for cycles 1 to 3 (CPU 0 to 1% before each, about 2% by
`typeperf` beforehand, a browser open). Load arrived during cycle 4: CPU read
4%, 9% and 16% before cycles 4 to 6 and 14% after. That was not planned; it
split the run into a quiet half and a disturbed half.

## Findings

**1. The reduction matters more than the cycle count.** Within one reading
the three runs differ by 3.3% on average and up to 11%: single runs are slow
at random. On the quiet cycles, noise between readings of one build:

| Reading | Noise |
|---|---:|
| best of 3 (the harness's) | 0.27% |
| best of the first 2 | 0.34% |
| the first run alone | 0.65% |
| median of 3 | 1.32% |

**2. Builds of one source differ by 0.13%** (best of 3, quiet cycles; PROCESS
quoted 0.36% between two builds, from single pairs).

**3. With four builds an arm, the builds set the limit, not the cycles.**
Per-build levels have standard deviation `sqrt(0.13² + 0.27²/C)`; a delta of
two four-build pools, with a t-interval on six degrees of freedom:

| Cycles | 95% interval | Minutes |
|---:|---:|---:|
| 3 | ±0.36% | 8 |
| 4 | ±0.33% | 10 |
| 6 | ±0.30% | 14 |
| 10 | ±0.27% | 22 |
| 20 | ±0.25% | 43 |

**4. Load does not add noise, it moves whole cycles.** In cycle 4 the arms
read −2.7%, −4.2%, −1.8% and +0.5% against their own median cycle; in cycle
6 the base read +2.1%. The CPU sample before a cycle does not predict it (4%
before the worst one). Estimates over all six cycles:

| Delta | Mean of per-build means | Mean of per-build medians |
|---|---:|---:|
| A-screen over A (null pair) | +0.30% | **−0.05%** |
| A over base | +1.63% | +2.10% |
| I over A | +1.47% | +0.82% |

The per-build median over six cycles absorbed two disturbed cycles and read
the null pair at zero; the mean did not.

**5. The bootstrap interval of `nps_multibuild.ps1` is not the uncertainty.**
It resamples readings, not builds, and RAR-P31 read the same pair at
+1.56..+1.89 and +0.92..+2.56 two hours apart.

**6. An unexplained session effect.** On the quiet cycles A read +2.1% over
the base and I +0.8% over A; RAR-P31's reads were +1.69% and +0.37%. The gap
is about 0.4% in both, a little over two standard errors of the difference.
Both changes are prefetches, whose value depends on how slow memory is at
that moment, and the host state differed (a browser open here). One session
of three quiet cycles decides nothing; it says a single session's interval
understates what a change is worth elsewhere. RAR-P31's verdicts stand.

## The revised read

- Four pext PGO builds an arm from clean trees, as now.
- **Six interleaved cycles of `bench 13 3`,** best of three per reading, one
  warm-up reading per build; every run archived.
- **Estimate:** the difference between the arms' means of per-build medians.
  **Interval:** from the spread of the per-build medians, t on six degrees
  of freedom; about ±0.30% on a quiet host.
- **Disturbance guard:** a cycle in which an arm's mean is more than 1% from
  that arm's median cycle is disturbed. The medians absorb two of six; with
  three or more the read is repeated, not interpreted.
- About 14 minutes instead of 43.

**Made two steps by the maintainer the same day**, who found six cycles
still long for one comparison. Most changes are far from the floor and need
less:
- **Step 1:** two cycles, about 6 minutes, ±0.41%. Accepted at +0.9% or
  more, closed at +0.1% or less.
- **Step 2, only in between:** four more cycles, six in all, decided as
  above at +0.5% with the lower bound above 0.
- A no-regression check mirrors the thresholds around −0.5%.
- Two cycles cannot absorb a disturbed one: they must agree within 1% per
  arm, or step 1 is repeated.

B.7's three accepted changes (+8.85%, +6.93%, +1.69%) would each have been
decided by step 1; only the correction-slot prefetch (+0.37%) would have
gone to step 2.

Not chosen: more builds an arm (eight builds and three cycles reach ±0.22%
in 16 minutes, but every arm then costs eight PGO builds); four cycles (the
median absorbs only one disturbed cycle).

## Owed before the first use

`nps_multibuild.ps1` archives nothing and reports the pooled median with a
bootstrap interval. The revised read needs the tool to archive runs and
report the estimate and interval above, with a test on this study's
`raw.json` (it must reproduce the three deltas of finding 4). B.8's
before-and-after reading is the first use.

## Calibration

No prediction was frozen in the ledger; the stated expectation was that
twenty cycles do not limit precision. That held (finding 3). Not expected:
that single runs are so unequal (finding 1), that load shifts whole cycles
rather than widening the noise (finding 4), and the session effect
(finding 6).
