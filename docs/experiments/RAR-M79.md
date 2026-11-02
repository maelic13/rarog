# RAR-M79 — Where the games' four-thread speed goes: short matches of Rarog against itself, one thread against four, with large pages on and off, one game at a time, and the B.9 head — REGISTERED 2026-11-02, before any game

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-02 by the agent; the maintainer runs
  it (about 75 minutes).
- **Leaf:** PLAN D.2, the four-thread card's H7.
- **Question.** In RAR-M78's games Rarog's four-thread side ran at 2.48
  times its one-thread side's speed. Replayed alone on the same positions
  ([RAR-R31](RAR-R31.md)) it runs at 3.78, and the slot's placement and three
  searching processes at once cost under 6%. Which condition of the games
  takes the rest? Rarog's distinctive process-level feature is its large
  pages: since `6ade0e6` (2026-10-19) every allocation of 512 KB or more,
  the table and each search thread's correction, continuation and pawn
  tables, is locked in 2 MB pages. Reckless allocates ordinarily on Windows.
  B.9, before large pages, kept 3.72 in games ([RAR-M74](RAR-M74.md)).
- **Kind:** an in-game speed measurement. The games are played only for
  their move comments: no Elo is read, and no gate.

## Why RAR-M77 does not answer it

[RAR-M77](RAR-M77.md) played large pages on against off and found on +2.7%
faster. Each of its games had one process of each kind in the same slot, and
three games ran at once, so a cost that large pages impose on every process
on the host, through the memory they lock and the other processes' locked
pages, would have hit both arms alike. Here each leg runs every process one
way.

## Design

- Legs (`analysis/artifacts/d2-harness/leg.ps1`, `0374ce41…`), each Rarog
  against itself, one thread (arm A) against four (arm B), `10+0.1`, Hash
  256 on both sides, `match-fixed-ltc.toml` (the UHO book, random order, no
  adjudication), seed 81 in every leg so the openings repeat, every slot four
  physical cores (`--cores-per-game 4`), Colosseum CLI 0.5.0 (pinned):

  | Leg | Binary | Large pages | Games at a time | Games |
  |---|---|---|---:|---:|
  | `on` | the head with `e057d84` (`rarog-d2dyn-pext-pgo.exe`, `a545b6b8…`) | on (the default) | 3 | 100 |
  | `off` | the same | `LargePages=false` on both sides | 3 | 100 |
  | `b9` | the B.9 head (`rarog-bis-24aefb4-pext-pgo.exe`, `2e4e2820…`) | none (predates them) | 3 | 100 |
  | `solo` | the head | on | 1 | 40 |

- `analysis/artifacts/d2-harness/run.ps1` (`66bc5d95…`) runs them in that
  order, each after the harness's idle wait, and then [RAR-R33](RAR-R33.md);
  a finished leg is skipped on a rerun.
- **Reading** (`harness_read.py`, `e7064420…`): per leg and side, NPS = the
  sum of each move's `n` over the sum of its `t − h` (the engine's own time to
  its last info line; moves with `t ≤ h` left out), and **R = NPS(4T) /
  NPS(1T)**. Also printed: NPS over charged time, mean depths.

## Judgement

- **Primary, R_on:**
  - **REPRODUCED** at 2.9 or less;
  - **ABSENT** at 3.4 or more;
  - **PARTLY** in between.
- **Conditions,** each named when ln(R_leg / R_on) is at least ln 1.15:
  - **LARGE PAGES** (`off`);
  - **NEIGHBOURS** (`solo`);
  - **B.9 SCALES** (`b9`).

## Exposure before registration

- RAR-M78's threads leg read with the same script (`anchor.log`): **R 2.48**
  (2.28 on charged time), depth four threads less one −1.87. RAR-R31's replay
  alone: 3.78.
- The runner's self-test on 2026-11-02 met a busy host (a maintainer's
  Basilisk tune) and waited, as designed; it was stopped there. The dry run
  of the four legs is made before the read, when the host is free, and
  appended here.

## Prediction, frozen before any game

| Reading | Probability |
|---|---|
| REPRODUCED | 0.80 |
| PARTLY | 0.12 |
| ABSENT | 0.08 |
| LARGE PAGES fires | 0.40 |
| NEIGHBOURS fires | 0.30 |
| B.9 SCALES fires | 0.50 |

- **Levels:** R_on 2.5 (80% interval 2.2 to 3.0); R_off 2.8 (2.3 to 3.6);
  R_solo 2.7 (2.3 to 3.5); R_b9 3.0 (2.4 to 3.7).
- **Reasoning.** RAR-M78's anchor makes REPRODUCED likely unless the host
  has changed. Large pages are the one process-level feature added since
  B.9 that Reckless lacks, and they lock per-thread tables as well as the
  shared one; but RAR-M77 found them faster in games, and RAR-R31's three
  processes with large pages lost only 6%, so they are a lead, not a likely
  cause. The opponent's process sharing the slot and Colosseum's own way of
  starting the engines are untested here; a REPRODUCED with no condition
  firing points at them.

## Decision and stop rule

- **REPRODUCED with LARGE PAGES:** large pages carry the loss. D.2 registers
  the repair: which allocations take large pages, or the default, read by
  this leg's in-game NPS; and [D.8](../PLAN.md) (Linux huge pages) is
  designed in that light, since transparent huge pages lock nothing.
- **REPRODUCED with NEIGHBOURS:** concurrent games cost Rarog's four-thread
  side more than Reckless's; the memory each process and thread uses enters
  the card.
- **REPRODUCED with B.9 SCALES alone:** something else between B.9 and the
  head; an in-game bisect over RAR-P50's revisions follows.
- **REPRODUCED with none:** the next legs change the opponent (a
  non-Rarog engine sharing the slot) and how the engines are started.
- **ABSENT:** RAR-M78's condition was the host's at the time; its threads
  leg is recorded as confounded, and its scaling is measured again.
- One read; nothing in the legs, games, seed or thresholds changes after the
  first game.

## How to run

From PowerShell 7 on an idle host, about 80 minutes with RAR-R33:

```powershell
pwsh -File D:\code\rarog\analysis\artifacts\d2-harness\run.ps1
```

It waits for an idle host before each step, resumes after an interruption,
and ends with `ALL DONE`.
