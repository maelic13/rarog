# RAR-R29 — Time-to-depth at four threads against one: the B.9 head, the d2 head and RAR-P52's candidate, on RAR-R10's paired harness — REGISTERED 2026-11-01, before the read; READ 2026-11-02

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

## Result — 2026-11-02, appended after exposure

Run from 2026-11-01 23:53 to 2026-11-02 00:08, host busy 0.5% before and
0.3% after; 192 of 192 pairs with a depth-D line (`ttd_read.log`;
`ttd_read.json` `b4e33496…`).

| Arm | X | Per position | 4T nodes to depth / 1T | 4T NPS / 1T |
|---|---:|---|---:|---:|
| `b9` | 0.962 | 0.75 1.14 1.21 0.93 1.06 0.71 1.17 0.86 | 3.95 | 3.79 |
| `ctl` | **0.871** | 0.75 0.97 1.00 0.54 1.48 0.70 0.92 0.90 | 4.09 | 3.57 |
| `dyn` | 0.897 | 0.80 1.00 0.94 0.41 1.41 0.89 0.88 1.25 | 4.09 | 3.66 |

The last two columns are geometric means over positions of per-position
medians, computed from the JSON after the read; they are not the registered
reading.

- d = ln X_ctl − ln X_b9 **−0.099** (90% bootstrap −0.235 to +0.014):
  **WASTE** by the registered rule, with an interval that reaches zero.
- ln X_dyn − ln X_ctl +0.030 (90% −0.076 to +0.138).
- Four-thread times scatter widely within a position (position 7, `ctl`:
  683 to 3,575 ms over eight pairs); one-thread times agree within 1%.

## Disposition

- **Four threads reach a fixed depth more slowly than one, in every arm.**
  They search about four times one thread's nodes to get there, so the main
  thread's own tree is no smaller than when it searches alone: the helpers'
  work buys it no depth. B.9 shares this (3.95), so it is not the C head's
  regression; the head adds a lower NPS ratio. d splits about 0.06 to the
  NPS ratio (ln 3.57/3.79) and 0.035 to the nodes.
- **By the rule for X_ctl under 1.2**, the card's next read is what the
  helpers do to the main thread: helper depth, table use, root diversity,
  the RAR-R03 bundle's members. By WASTE the store items stay in scope; the
  backend left with [RAR-P53](RAR-P53.md).
- [RAR-P52](RAR-P52.md) closes on +0.030.
- **Not Elo.** The helpers' nodes may still widen the search usefully; only
  games price that.
- **The games' depth deficit is not reproduced.** At equal time X 0.87 costs
  about 0.15 plies at EBF 2.565, against 2.05 on the games' unheld moves
  ([RAR-R30](RAR-R30.md)). This read searched depths 20 to 22 for about 2 s
  from a cleared table; the games search depths 15 to 17 in about 200 ms with
  the table kept across moves.

## Calibration

- WASTE, predicted 0.40: a hit by the rule, though the interval leaves SAME
  open.
- The levels missed. X_ctl 0.871 lies below its 80% interval (0.9 to 1.6)
  and X_b9 0.962 below its (1.0 to 1.7); four threads slower to depth than
  one had 0.25. A miss in the level's sign, in mechanism: the prediction
  assumed the helpers shorten the main thread's tree through the shared
  table, as in engines that scale, and here they do not, at B.9 or now.
- The candidate, predicted +0.03 to +0.10: +0.030, at the edge and
  unresolved.
