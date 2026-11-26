# RAR-P62 — D.10's Linux read: `LargePages` on against off in one binary, transparent huge pages in WSL2, one thread by the two-step read and a four-thread cell — REGISTERED 2026-11-26, before the read

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-26 by the agent, before any timed
  run; the read is the maintainer's (timed, on an idle host).
- Disposition: open — registered 2026-11-26; the read is the maintainer's.
- **Kind:** performance qualification of a behaviour-neutral change
  (PLAN D.10). Evidence layer: NPS, not Elo.
- **Change:** D.10, `cc08bc3` on `d9`. On Linux each page allocation (512 KB
  or more: the transposition table, the correction, continuation and pawn
  tables, the pawn cache) is rounded up to whole 2 MB pages, aligned to one,
  and advised with `madvise(MADV_HUGEPAGE)` while `LargePages` is on.
  Nothing is locked and no privilege is needed. The tree is unchanged:
  `bench 13` 3,694,298 / EBF 2.324 with the option on and off, on Windows
  and on Linux.
- **Host:** WSL2 on the match host (Ryzen 9 5950X, 32 logical CPUs; WSL
  sees 62 GB), Ubuntu 24.04, kernel 6.18.58.1-microsoft-standard-WSL2,
  `enabled` `always [madvise] never`, `defrag` `[madvise]`, when prepared.
  The command records the mode again at the read.
- **Binary and pool:** `cargo xtask build --tier pext` (PGO) in a clean WSL
  clone at `cc08bc3`: ``rarog-dev-20261126-cc08bc38-linux-x64-pext``, SHA-256 ``A1FCB854B16CE560F0F1E35E1A59A0E0B1FEDF953399364BDE2BE1FAFF5F2635``, `id name` `Rarog dev-20261126-cc08bc38`;
  a second build of the same clone is byte-identical. The pool is four copies, `~/d10-nps/pool/pext-1` to `pext-4`,
  with `manifest-pext.txt` (fingerprint 3,694,298). Copies, not builds:
  Linux PGO builds of one source are byte-identical (RAR-P57), and both arms
  run the same bytes, so a build's layout cancels; the per-copy medians carry
  process noise only.
- **The wire, checked before any timing.** `backing.py` runs one search per
  arm at Hash 256 and requires the on arm's `AnonHugePages` at 200 MB or
  more and the off arm's below a quarter of it; otherwise the command stops
  before timing anything. When prepared: on 284 MB of 289 MB resident, off
  0 MB.
- **The host measure.** `nps_read.py`'s own, which in WSL runs through
  Windows' PowerShell over the whole machine and adds the Linux side's
  engine and build processes (`1015348`). Checked live when prepared:
  idle 1.4%, eight busy loops on the Linux side 27.1% (a quarter of the 32 CPUs), and a Linux process named `rarog-probe` reported and refused (`analysis/artifacts/d10-thp-nps/wire_live.txt`).

## Read

`tools/nps_read.py` with one pool on both arms: base `LargePages=false`,
candidate `LargePages=true`, `bench 13 3`, PROCESS's step 1 and, only
between its thresholds, step 2 (`--extend`). Then the four-thread cell: the
same pool and arms with `Threads=4` on both, `--smp`, two cycles. The whole
run is `~/d10-nps/run.sh` in WSL (SHA-256 `0C911D1D…`; `backing.py`
`0A4A6BC5…`; copies in `analysis/artifacts/d10-thp-nps/`):

```
python3 /mnt/d/code/rarog/tools/nps_read.py --base pool --cand pool \
  --base-option LargePages=false --cand-option LargePages=true \
  --out <read>/1t --label "RAR-P62 1T: LargePages on (cand) against off (base), WSL"
python3 /mnt/d/code/rarog/tools/nps_read.py --extend <read>/1t     # only if step 1 is between
python3 /mnt/d/code/rarog/tools/nps_read.py --base pool --cand pool \
  --base-option LargePages=false --cand-option LargePages=true \
  --smp --base-option Threads=4 --cand-option Threads=4 \
  --out <read>/4t --label "RAR-P62 4T cell: LargePages on (cand) against off (base), WSL"
```

## Rule, fixed before the read

PROCESS's speed rule, not `--no-regression`: step 1 accepts at +0.9% or
more and closes `NO_CHANGE` at +0.1% or less; between, step 2 accepts at
+0.5% or more with the lower bound above zero. A disturbed step is repeated
once; disturbed again, WSL's timing is recorded as unable to resolve the
question (RAR-P57's precedent) and D.10's close is the maintainer's
decision.

- **Accepted:** D.10 closes `DONE`; the option stays on by default on Linux.
- **`NO_CHANGE`:** the change stays (no new `unsafe` site, no tree change,
  nothing a user sets up), and D.10 closes `NO_CHANGE` for speed in WSL;
  the retry trigger is a native Linux host.
- **Step 1 at −0.9% or below:** reported to the maintainer before anything
  else; the Linux default is theirs to decide.
- The four-thread cell decides nothing; it is recorded beside the read.

## Prediction

- **One thread: +2%**, 90% interval −0.5% to +5%. Accepted 0.5,
  `NO_CHANGE` 0.35, a loss at −0.9% or below 0.05, unresolved 0.1; a step
  disturbed at least once 0.4 (RAR-P57 saw WSL readings swing 10% across
  cycles). Basis: on Windows large pages off cost 1–4% (RAR-P47) and on read
  +2.7% in four-thread games (RAR-M77). Under WSL2 the guest's 2 MB pages
  shorten the guest's half of a nested page walk, but whether the hypervisor
  backs the guest's memory with large pages is not known here; if it does
  not, a TLB entry still maps 4 KB and the gain shrinks. That is a prior,
  not a measurement.
- **Four-thread cell: +2.5%**, 90% interval −1% to +6%.

## Result

Not read.
