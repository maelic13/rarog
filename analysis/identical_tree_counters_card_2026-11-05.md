# D.4.6 card: an identical-tree counter read

Date: 2026-11-05. Leaf: PLAN D.4.6 (`R2`, then `I1`). Source:
`analysis/triumviratus_review_2026-11-03.md` TM-7. Registered before any
counter is read.

## Decision needed

Can a hardware or kernel counter read the speed of a change that keeps the
tree below the pooled read's limit (four PGO builds an arm resolve about 1%,
eight about 0.5%; PROCESS, *NPS*), and with what tool on this host? Users:
the speed checks of D.5 (items 6, 9, 10), D.7 and D.8, and H.4.

## Known evidence

- **The pooled read's limit is the builds, not the runs.** Two PGO builds
  of one source differ by up to 0.36%; twenty cycles reach only ±0.25%
  (RAR-P32, `analysis/nps_method_study_2026-10-02.md`). Basilisk's BAS-P20
  read −0.58% with identical hot code, gone with 64-byte function
  alignment: layout.
- **Triumviratus's ±0.1% null is run noise only.** Its control is a
  byte-identical copy of the base, which shares the base's layout; any real
  change moves code and so moves layout. Its sibling-pair form is unusable
  here (RAR-P54: either engine keeps 0.63–0.65 of a thread beside its
  sibling).
- **One plain binary's processes vary by up to 6%** (RAR-P38): some
  placement is per process, not per build.
- **The engine is memory-latency bound since C.12**: cuts in instruction
  count off the latency chain read flat in NPS, so instructions retired
  are not speed here.
- **This host** (Ryzen 9 5950X, Windows 11 Pro): ETW exposes TotalCycles,
  InstructionRetired, BranchMispredictions and 14 other sources, six at a
  time (`xperf -pmcsources`, 2026-11-05). Starting a kernel session needs
  an elevated shell, which the agent's shell is not
  (`tools/profile_etw.ps1` is maintainer-run for that reason). AMD uProf is
  not installed. `QueryThreadCycleTime` needs no privilege and reads any
  thread of the user's own processes: the time-stamp counter's ticks while
  the thread was scheduled, so it removes preemption and start-up but not
  clock-frequency changes.

## Hypotheses

- **H1, layout.** The spread between PGO builds of one source shows in any
  cycle count as it does in NPS. Then no counter beats the build limit with
  fewer builds; it can only shorten each build's reading.
- **H2, run noise.** Within one build, a scheduled-cycle count per node
  varies less than wall-clock NPS, because it drops preemption and the
  harness's own time.
- **H3, instructions.** Instructions retired per node are nearly identical
  across PGO builds of one source and across runs, so they resolve a
  change's instruction cost finely; for a memory-bound engine that is a
  diagnostic, never an accept.

## Interaction map

The read touches nothing in the engine: bench at fixed depth, one thread,
the node count asserted equal to the fingerprint on every run, so every
reading counts the same tree. A counter tool lives in `tools/`; an
in-engine counter would be a behaviour-neutral engine change and is not
proposed. Shared hazards: antivirus scans of fresh executables (RAR-M48),
the session's own load (`nps-measurement` note), frequency boost per core.

## Cheapest discriminating test

**T1, unprivileged, now (about 3 minutes, one core).** The four pext PGO
builds of `e5fceb5` in `analysis/artifacts/e33-universal-nps/pext/`
(fingerprint 14,749,043, hashes checked against their manifest). Each
process pinned to logical CPU 6; one `bench 8 1` per build first (the
antivirus scan); then five rounds, each running every build once in a
rotated order, `bench 13 1` on a quiet host (under 10% load for 30 s
before the first run, by `GetSystemTimes`). Per run: nodes (must equal the
fingerprint), bench time and NPS, and the scheduled cycles of each thread
of the process across the bench (`QueryThreadCycleTime` before and after;
the busiest thread is the search), as cycles per node. Read, for both NPS
and cycles per node: the within-build coefficient of variation, and the
spread (largest minus smallest) of the per-build medians. Host load during
each run is recorded; a run above 10% is repeated once.

**T2, maintainer-run, only if T1 leaves it open.** The same schedule under
an ETW session counting TotalCycles, InstructionRetired and
BranchMispredictions per thread at each context switch, from an elevated
shell; its tool is the `I1` step.

## Prospective prediction (frozen before T1 runs)

| # | Prediction | Confidence |
|---|---|---|
| P1 | Within a build, the coefficient of variation of cycles per node is between 0.5 and 1.0 times that of NPS (H2 holds weakly: an idle pinned core has little preemption to remove) | 60% |
| P2 | The spread of the per-build medians of cycles per node is at least 0.2% (H1: layout shows in the counter) | 65% |
| P3 | No prediction on instructions retired until T2 | — |

## Falsifiers and stop conditions

- **The counter wins outright** if the per-build medians of cycles per
  node spread by at most 0.1% and each build's runs vary by at most 0.1%:
  then one build an arm resolves well below 1%, H1 is refuted, and the
  `I1` step builds the unprivileged tool and validates it against the
  pooled read on a planted cost of known size.
- **The build limit stands** if the spread is at least 0.3%: cycles cannot
  beat the pooled builds. T2 then asks only H3, whether instructions retired
  are fine enough to be worth a diagnostic beside the pooled read.
- **In between** (0.1–0.3%): T2 decides, with core cycles beside the
  scheduled count.
- **Stop:** if neither counter beats the pooled builds, D.4.6 closes
  `NO_CHANGE`. The pooled read stays the speed check of D.5, D.7, D.8 and
  H.4, and layout is a separate lever (function alignment, BAS-P20), not
  this leaf's.

## Decision

Pending T1.
