# RAR-P52 — The PGO profile's coverage of the multi-thread kernel: the poll as a trait object, read on Q at four threads — REGISTERED 2026-11-01, before the read

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-01 by the agent; the maintainer runs
  it (about 15 minutes).
- **Leaf:** PLAN D.2, the four-thread card's read 1 (engine review input,
  item 1; `analysis/engine_review_2026-11-01.md` §3.3, RUST-1 / DRIVER-3 /
  DRIVER-4).
- **Question:** every thread of a `Threads > 1` search runs a copy of the
  kernel the PGO profile never trained. Does one trained copy for every
  thread raise the four-thread shared-search efficiency Q on RAR-P50's
  instrument, with the one-thread cell unchanged?
- **Kind:** performance qualification of a behaviour-neutral engine change.
  It decides whether the change stays and whether the untrained copies held
  part of the four-thread loss. Nothing about strength follows.

## The change and the arms

- **Candidate, `e057d84`** on `d2`: `search_impl`, `search_parallel` and
  `run_worker_job` take the poll as `&mut dyn FnMut() -> SearchEvent`. The
  main thread's and the helpers' polls are coerced to it, so `negamax`,
  `quiescence`, `check_stop` and the root search exist once per node type.
- **The profile, checked** (`analysis/artifacts/d2-pgo/kernel_copies.py`, `913d4567…`,
  over `llvm-profdata show` of each build's own profile, `kernel_copies.log`):

  | Build | Kernel instantiations | All-zero profile | Over a closure type |
  |---|---:|---:|---:|
  | Control | 17 | 11 (6 main-thread, 5 helper) | 17 |
  | Candidate | 6 | 0 | 0 (all over `dyn FnMut`) |

  The candidate's counts equal the control's serial copy's (`negamax<NonPv>`
  593,841,256 in both), so the training is the same and every thread now runs
  the trained copy. The control reproduces the review's replica exactly.
- **What the cure leaves untrained.** The table's shared-form branches sit
  inside the same kernel (`TtStorage::Shared`), and the one-thread training
  never builds a shared table. Every four-thread probe and store still runs
  those branches untrained in both arms. A Threads-2 training run (the
  review's cure (b)) would also cover them; it is not run here (one cure at a
  time). [RAR-P53](RAR-P53.md)'s two builds price the cold branches.
- **Builds** (`analysis/artifacts/d2-pgo/build.sh`, `build.log`): `cargo
  xtask build --arch pext --pgo`, rustc 1.99.1, both on 2026-11-01.

  | Arm | Binary | Source | SHA-256 | `bench 13` |
  |---|---|---|---|---|
  | `ctl` | `tools/test_engines/rarog-d2ctl-pext-pgo.exe` | `84f978b` (engine as `2b7bca9`), detached worktree | `a0a28509…` | 13,987,223 / 2.565 |
  | `dyn` | `tools/test_engines/rarog-d2dyn-pext-pgo.exe` | `e057d84` | `a545b6b8…` | 13,987,223 / 2.565 |

  - The control is a fresh build, not RAR-M78's `ebbbc917…`: two PGO builds of
    one source differ (a first control build read `61ed5686…`), so both arms
    come from one session and one compiler. One build an arm resolves about
    ±0.4% (PROCESS, NPS); the prediction is ten times that.
  - The candidate is 1,667,072 bytes against 1,851,392.
- **Checks on `e057d84`:** `cargo fmt --check` 0; `cargo clippy
  --all-features --all-targets` 0, no warnings; `cargo test` debug and
  release: all pass except `tests/uci_process.rs`, where 1 to 5 of its 28
  tests time out with no engine output in a full run. They pass 28 of 28
  alone in both profiles, and the unchanged head fails the same way in a full
  release run (4 of 28), so this predates the change; it is recorded in PLAN
  D.2 and owned by D.3.

## Instrument

- `analysis/artifacts/d2-pgo/q_read.py` (`433c1530…`, ignored): RAR-P50's `split_read3.py`
  unchanged but for the arms and the judgement.
  - `go movetime 10000`, every info line kept, nodes interpolated at exactly
    5,000 ms; each position read on its own.
  - Fresh processes, Hash 256, RAR-P46's two positions, one logical CPU per
    thread on its own physical core of CCD0, masks read back, hashes checked.
  - Cells `1T` (CPU 2), `4x1T` (CPUs 2, 4, 6, 8, together) and `4T` (CPUs 2,
    4, 6, 8). Six rounds, the two arms alternating first, a 10 s settle
    before each arm's block.
- **Readings,** per position: L = ln(Q_dyn / Q_ctl), and the 1T, 4x1T and
  4T medians' log ratios. The 4T log ratio is the four-thread NPS read the
  close needs.
- **Outcome** on the mean of the two positions' L:
  - **KILLED** under 0.02;
  - **BELOW BAND** 0.02 to 0.03;
  - **CONFIRMED IN BAND** 0.03 to 0.10;
  - **CONFIRMED ABOVE BAND** over 0.10;
  - with **1T CELL MOVED** added when the 1T cell moves more than 1% at
    either position.
  - Invalid with over 5% of readings missing; an interrupted read is rerun
    whole.

## Exposure before registration

- The build logs and `bench 13`'s node counts and EBF (its NPS line was in
  the output files and was not read). No four-thread search of either arm
  was read.
- One smoke round of the instrument (`smoke.log`): 24 of 24 readings, masks
  and hashes held. The smoke prints no reading, and its JSON was not opened.

## Prediction, frozen before the read

The review froze the band (PLAN D.2, item 1): Q rises by 0.03 to 0.10 in
ln, the 1T cell within 1%; under 0.02 kills it.

| Outcome | Probability |
|---|---|
| CONFIRMED IN BAND | 0.45 |
| CONFIRMED ABOVE BAND | 0.20 |
| BELOW BAND | 0.10 |
| KILLED | 0.25 |
| 1T CELL MOVED (added to any of the above) | 0.15 |

- **Why in band.** The untrained `negamax<NonPv>` is 6,680 assembly lines
  against 15,568 for the trained one: the inlining the speed pass was tuned
  under is missing on every thread at 4T.
- **Why it could die.** B.9 had the same closure-generic kernel and scaled
  well (Q 1.05), so the untrained copies alone may cost both heads alike; and
  the cold shared-table branches stay cold in both arms.
- **The 1T cell** runs the serial copy in both arms; it moves only by build
  luck or by the layout change, both expected under 1%.

## Decision and stop rule

- **CONFIRMED** (in or above band, 1T within 1%): the change stays. It is
  behaviour-neutral and closes on this read's 4T NPS and
  [RAR-R29](RAR-R29.md)'s time-to-depth for the `dyn` arm (its ln X no lower
  than the control's by more than 0.02). It improves every shipped
  multi-thread build and the four-thread arm of every read since RAR-P46.
- **BELOW BAND or KILLED:** the hypothesis does not hold the four-thread
  loss. The change stays only as a simplification, when no cell is slower by
  more than 1% at either position; otherwise it is reverted.
- **1T CELL MOVED** over 1% in the slower direction: reverted whatever Q
  says, and the layout question returns to research.
- **Cure (b)**, a Threads-2 run in the training, is registered next only if
  RAR-P53 shows the cold shared-table branches cost at least 2% (R(pgo) /
  R(nopgo) ≤ 0.98).
- One read; nothing in the arms, cells, rounds or window changes after it
  starts.

## How to run

From PowerShell 7 on an idle host, nothing else running, about 15 minutes:

```powershell
python D:\code\rarog\analysis\artifacts\d2-pgo\q_read.py D:\code\rarog\analysis\artifacts\d2-pgo\q_read.json 2>&1 | Tee-Object -FilePath D:\code\rarog\analysis\artifacts\d2-pgo\q_read.log
```

If it refuses with `HOST NOT IDLE`, run it again. The summary ends with an
`OUTCOME:` line.
