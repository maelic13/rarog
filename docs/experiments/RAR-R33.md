# RAR-R33 — Does the helpers' work reach the main thread: its own table probes and hits at one thread and at four, to the same depth — REGISTERED 2026-11-02, before the read; READ 2026-11-02: HITS WITHOUT SAVINGS

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

## Result — 2026-11-02, appended after exposure

Run by `run.ps1` after RAR-M79's legs on 2026-11-02, ending at 18:06, in
263 s; host 0.2% busy before and 0.8% after; 48 of 48 pairs with both
counters and a depth line (`mainhit_read.log`; `mainhit_read.json`
`1c309922…`). The one-thread searches repeat exactly: each position's six
one-thread searches give the same probes and hits. The four-thread ones
vary, from 0.55 to 2.86 times the one-thread probes in single pairs.

| Position | Depth | Main probes 1T | Main probes 4T | M | Main hit rate 1T | 4T | All nodes 4T / 1T |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 0 | 20 | 3,109,364 | 3,438,598 | 1.11 | 0.518 | 0.587 | 4.34 |
| 1 | 20 | 3,010,529 | 2,950,279 | 0.98 | 0.359 | 0.414 | 3.81 |
| 2 | 22 | 2,798,565 | 2,990,558 | 1.07 | 0.550 | 0.556 | 4.28 |
| 3 | 21 | 3,049,247 | 5,710,774 | 1.87 | 0.414 | 0.421 | 7.53 |
| 4 | 20 | 3,056,139 | 2,863,326 | 0.94 | 0.414 | 0.497 | 3.78 |
| 5 | 20 | 2,897,428 | 3,375,642 | 1.17 | 0.426 | 0.510 | 4.67 |
| 6 | 22 | 3,056,243 | 2,855,172 | 0.93 | 0.477 | 0.539 | 3.77 |
| 7 | 22 | 4,148,099 | 3,852,052 | 0.93 | 0.637 | 0.720 | 3.64 |

(Medians over the six pairs.)

- M **1.094** (90% 0.975 to 1.209); main hit rate four threads less one
  **+5.61 points** (90% +5.09 to +5.96).
- **Outcome line:** `HITS WITHOUT SAVINGS: the main thread hits more but its
  tree does not shrink`.

## Disposition

- **By the rule:** the helpers' entries reach the main thread, whose hit
  rate rises at every position (+0.6 to +8.4 points), but its tree does not
  shrink: it probes 9% more at four threads, and 87% more at position 3. The
  next H3 read is the depth and bound of the main thread's hits against what
  its node needs.
- Consistent with [RAR-R29](RAR-R29.md)'s node accounting (four threads at
  4.09 times one thread's nodes): here 3.64 to 7.53 by position.
- Not a change. H3 waits beside H7, which [RAR-M79](RAR-M79.md) reshaped:
  the in-game speed loss is the host's state, and one game alone costs the
  four-thread side most.

## Calibration

- HITS WITHOUT SAVINGS was predicted 0.50: a hit.
- M 1.0 (80% 0.85 to 1.15) read 1.094, inside. Δh +3 points (−1 to +8) read
  +5.6, inside and above the point: the helpers' entries reach the main
  thread more than predicted and still do not cut its tree. Position 3's
  1.87 widens the interval on M; the prediction did not foresee one position
  where four threads search a far larger tree.
