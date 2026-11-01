# RAR-R29 — Time-to-depth at four threads against one: the B.9 head, the d2 head and RAR-P52's candidate, on RAR-R10's paired harness — REGISTERED 2026-11-01, before the read

Indexed under *4. Root search, time management and SMP* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-01 by the agent; the maintainer runs
  it (about 25 minutes).
- **Leaf:** PLAN D.2, the four-thread card's read 2 (engine review input,
  item 3; DRIVER-1).
- **Question.** Q, the shared-search efficiency of RAR-P49 and RAR-P50, is an
  NPS ratio, and NPS mixes node cost with node mix. Does the d2 head's main
  thread reach a fixed depth with four threads as much faster than with one
  as B.9's did? And how much faster at all: in RAR-M78's games the
  four-thread side reported 1.9 plies less depth than the one-thread side at
  2.28× its nodes per second.
- **Kind:** zero-game diagnostic. It decides which of the card's hypotheses
  stay in scope, not strength.

## Design

- `analysis/artifacts/d2-ttd/ttd_read.py` (`41d5c74d…`, ignored), RAR-R10's
  harness:
  - one engine process per (binary, position), Threads 1 and Threads 4
    alternating inside it, the order swapped every pair, `ucinewgame` before
    every search;
  - pinned to CPUs 2, 4, 6, 8 (one logical CPU per physical core of CCD0),
    Hash 256, `go depth D`;
  - eight positions, RAR-P46's two and six from `bench`'s suite, eight pairs
    each, the three binaries' order rotating by position.
- **D per position** (`calibrate.log`, node counts only): the first depth at
  which the d2 head's one-thread search passes 4 M nodes, 20, 20, 22, 21, 20,
  20, 22 and 22. B.9 searches the same depths.
- **Time-to-depth** of a search is the `time` of its last `info depth D`
  line. Only the main thread prints, so this is the main thread's completed
  iteration D; the nodes on that line are all threads'.
- **Arms:**

  | Arm | Binary | SHA-256 |
  |---|---|---|
  | `b9` | `rarog-bis-24aefb4-pext-pgo.exe` (B.9 head, rustc 1.99.1, RAR-P50) | `2e4e2820…` |
  | `ctl` | `rarog-d2ctl-pext-pgo.exe` (d2 head `84f978b`, RAR-P52's control) | `a0a28509…` |
  | `dyn` | `rarog-d2dyn-pext-pgo.exe` (`e057d84`, RAR-P52's candidate) | `a545b6b8…` |

## Readings and judgement

- Per pair r = ln(T1 / T4); per binary and position the median r; per
  binary the mean over positions, **X = exp(mean)**, the four-thread
  time-to-depth speedup. A 90% bootstrap over pairs within positions is
  printed beside each difference.
- **Registered comparison** (PLAN D.2, item 3), d = ln X_ctl − ln X_b9:
  - **SAME** when |d| < ln 1.05: Q's loss is node mix, not waste;
  - **WASTE** when d ≤ ln 0.94: the loss is real waste, and the card's
    backend and store items stay in scope;
  - **HEAD BETTER** when d ≥ ln 1.06; **UNRESOLVED** in between.
- **Also read:** the levels X_b9 and X_ctl, and ln X_dyn − ln X_ctl, which
  RAR-P52's close uses.
- Invalid when any pair lacks a depth-D line; an interrupted read is rerun
  whole.

## Exposure before registration

- The calibration printed one-thread node counts by depth for the d2 head
  only. No four-thread search was read.
- One smoke pair per binary and position at D − 6 (`smoke.log`): 24 of 24
  pairs with a depth line. The smoke prints no reading; its JSON was not
  opened.

## Prediction, frozen before the read

| Outcome | Probability |
|---|---|
| SAME | 0.40 |
| WASTE | 0.40 |
| UNRESOLVED | 0.10 |
| HEAD BETTER | 0.10 |

- **Levels:** X_ctl 1.2 (80% interval 0.9 to 1.6), X_b9 1.3 (1.0 to 1.7).
  Probability that X_ctl is under 1, four threads reaching the depth more
  slowly than one: 0.25.
- **The candidate:** ln X_dyn − ln X_ctl +0.03 to +0.10 if RAR-P52's
  mechanism holds, as its Q.
- **Reasoning.** The review's reading (Q falls when nodes get cheaper, with
  nothing wasted) predicts SAME; RAR-P50's per-thread loss of 6–7% carried
  into depth predicts WASTE, and the games' depth deficit leans that way.
  The games cannot decide it: their depth mixes the clock, the soft-stop
  vote and the table carried across moves, which `go depth` removes.

## Decision and stop rule

- **SAME:** the head's four-thread loss in Q is node mix; the shared
  backend (RAR-P53) and the byte-identical stores leave the card unless
  RAR-P53 prices the backend on its own.
- **WASTE:** they stay in scope, in RAR-P53's cost order.
- **X_ctl under 1.2** (whatever the comparison): the helpers do not speed
  the main thread to depth, and the card's next read is what they do to it
  (helper depth, table use, root diversity, the RAR-R03 bundle's members).
- **X_ctl at 1.8 or above:** the main thread's depth is not the deficit,
  and the games' depth loss is the clock's (RAR-R30) or the vote's.
- One read; nothing in the positions, depths, pairs or arms changes after it
  starts.

## How to run

From PowerShell 7 on an idle host, nothing else running, about 25 minutes:

```powershell
python D:\code\rarog\analysis\artifacts\d2-ttd\ttd_read.py D:\code\rarog\analysis\artifacts\d2-ttd\ttd_read.json 2>&1 | Tee-Object -FilePath D:\code\rarog\analysis\artifacts\d2-ttd\ttd_read.log
```

If it refuses with `HOST NOT IDLE`, run it again. The summary ends with an
`OUTCOME:` line.
