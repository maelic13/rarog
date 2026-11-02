# RAR-R33 — Does the helpers' work reach the main thread: its own table probes and hits at one thread and at four, to the same depth — REGISTERED 2026-11-02, before the read

Indexed under *4. Root search, time management and SMP* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-02 by the agent; the maintainer runs
  it after [RAR-M79](RAR-M79.md), in the same command (about 5 minutes).
- **Leaf:** PLAN D.2, the four-thread card's H3.
- **Question.** Alone, four threads reach a depth with about four times one
  thread's nodes ([RAR-R29](RAR-R29.md)) and report the same depth as one
  under the games' clocks ([RAR-R31](RAR-R31.md)): the helpers buy the main
  thread no depth. In RAR-M78's games Reckless's four threads reported 1.3
  plies more than its one, and Stockfish's 2.8. None of the four RAR-R03
  members holds the main thread back ([RAR-R32](RAR-R32.md)). Do the helpers'
  table entries reach the main thread at all, and does its own tree shrink?
- **Kind:** zero-game diagnostic.

## Design

- **Build** (`analysis/artifacts/d2-mainhit/build.sh`, `4d374b91…`):
  `e057d84` with `--features diag`, pext, no PGO (its timing is never read),
  as `tools/test_engines/rarog-d2diag-pext.exe`. Before the read the script
  checks `bench 13` at 13,987,223 (the counters never steer) and that a
  one-thread and a four-thread search each print the two counters, and only
  then pins the binary's hash in the read script (`set_hash.py`).
- **The counters** (`src/search/node.rs`, at the table probe): every probe
  by the thread with index 0 counts `main_tt_probes`, every one that finds an
  entry `main_tt_hits`. Both are exact event counts, reset before a search
  and printed after it (after the helpers are joined, at four threads).
- `analysis/artifacts/d2-mainhit/mainhit_read.py` (ignored; its hash is
  recorded when the build pins it): RAR-R29's harness, one process per
  position, Threads 1 and 4 alternating inside it, the order swapped every
  pair, `ucinewgame` before every search, CPUs 2, 4, 6, 8, Hash 256, `go
  depth D` at RAR-R29's frozen depths, eight positions, six pairs.

## Readings and judgement

- Per position, the medians over pairs: the main thread's probes at one
  thread (P1) and at four (P4), its hit rates h1 and h4, and all threads'
  nodes at four over one.
- **M = exp(mean over positions of ln(P4 / P1))**, the main thread's own
  tree at four threads against alone; **Δh = mean of h4 − h1**. 90%
  bootstraps over pairs.
- **Outcome:**
  - **MAIN TREE SHRINKS** when M is 0.85 or less;
  - otherwise **HITS WITHOUT SAVINGS** when Δh is +2 points or more;
  - otherwise **HELPERS INVISIBLE**.
- Invalid when any search lacks either counter or its depth-D line.

## Exposure before registration

None.

## Preparation, 2026-11-02 at 16:22, before the read

- `build.sh` (`build.log`): `rarog-d2diag-pext.exe`, SHA-256 `08a7587c…`,
  from the clean worktree at `e057d84`; `bench 13` reads **13,987,223**; a
  one-thread and a four-thread search each print `main_tt_probes` and
  `main_tt_hits` once. The read script then pinned the hash
  (`mainhit_read.py`, `510ece68…` after pinning).
- Smoke (`smoke.log`): one pair a position at depth D − 6, 8 of 8 pairs with
  both counters and a depth line; it prints no reading.
- The judgement's self-test on synthetic pairs (`judge_selftest.py`,
  `judge_selftest.log`).

## Prediction, frozen before the read

| Outcome | Probability |
|---|---|
| HITS WITHOUT SAVINGS | 0.50 |
| HELPERS INVISIBLE | 0.40 |
| MAIN TREE SHRINKS | 0.10 |

- **Levels:** M 1.0 (80% interval 0.85 to 1.15); Δh +3 points (−1 to +8).
- **Reasoning.** RAR-R29's nodes (4.09 times one thread's for four threads)
  leave the main thread about one thread's tree, so a shrinking tree is
  unlikely. The helpers search the same root moves to the same depths, so
  the main thread should meet their entries; whether those entries are deep
  enough and bounded the right way to cut is the open part.

## Decision and stop rule

- **HELPERS INVISIBLE:** the helpers' entries do not reach the main thread;
  the shared table's write and replacement path is the card's next read (in
  an instrument build, the share of the main thread's hits written by a
  helper).
- **HITS WITHOUT SAVINGS:** the entries arrive but do not cut; the next read
  is the depth and bound of the main thread's hits against what its node
  needs.
- **MAIN TREE SHRINKS:** contradicts RAR-R29's node accounting; the
  instrument is checked before anything else.
- None of these is a change. H3 is pursued only beside H7, whose answer
  decides what four threads are worth in games.
- One read; nothing in the positions, depths, pairs or thresholds changes
  after it starts.

## How to run

It runs as the last step of RAR-M79's command:

```powershell
pwsh -File D:\code\rarog\analysis\artifacts\d2-harness\run.ps1
```
