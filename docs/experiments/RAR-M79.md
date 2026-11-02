# RAR-M79 — Where the games' four-thread speed goes: short matches of Rarog against itself, one thread against four, with large pages on and off, one game at a time, and the B.9 head — REGISTERED 2026-11-02, before any game; READ 2026-11-02: ABSENT, and one game at a time costs the four-thread side

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
  Basilisk tune) and waited, as designed; it was stopped there.

## Preparation, 2026-11-02 at 16:23, before any game

- Host idle (2.1% CPU, no engine, harness or build process).
- The runner's self-test (`selftest.log`): the wait, the stream and the stop
  on a failing step all held.
- **Dry run** (`dryrun.log`, `<leg>.log.dryrun`): all four legs resolve as
  designed: `10000+100`, margin 20 ms, Hash 256, one thread against four,
  the UHO book in random order with seed 81, no adjudication; slots CPUs
  2–9, 16–23 and 24–31 (`solo`: 2–9 alone); `off` passes `LargePages=false`
  to both sides; `b9` runs the B.9 binary. A first dry run was refused at the
  `b9` leg by a Basilisk process that appeared for a few seconds; the second
  passed whole.
- **The switch is live** (`wire_largepages.log`): after `setoption name
  LargePages value false` the head reports `Large pages: disabled`, and `on`
  with the default; the node count is the same either way.

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

## Result — 2026-11-02, appended after exposure

Run by the maintainer with `run.ps1` on 2026-11-02 from 16:51 to 18:02.
Every leg completed with no fault and no time loss, and no idle waiver; the
host was 0.2% busy at `on`'s start, 6.8% at `off`'s, 1.1% at `b9`'s and 1.1%
at `solo`'s. The `b9` leg was stopped after 50 games and resumed at 17:34 at
its seed (`resuming: 50 of 100 games complete`, host 5.3% busy); the other
legs ran in one piece. Colosseum's manifests:

- `on`: 100 games, pentanomial 4, 30, 15, 1, 0, `games.pgn` `E587C474…`;
- `off`: 100 games, 2, 28, 18, 2, 0, `CB33DC28…`;
- `b9`: 100 games, 3, 31, 14, 2, 0, `AAF69DE9…`;
- `solo`: 40 games, 2, 8, 7, 3, 0, `CD353C4A…`.

The reading (`harness_read.log`, `dc919baa…`):

| Leg | 1T NPS | 4T NPS | R | R on charged time | Depth 4T − 1T |
|---|---:|---:|---:|---:|---:|
| `on` | 2.74M | 9.41M | **3.44** | 3.17 | −1.82 |
| `off` | 2.72M | 9.60M | 3.53 | 3.26 | −1.35 |
| `b9` | 2.15M | 7.07M | 3.28 | 2.90 | +0.30 |
| `solo` | 2.89M | 7.77M | 2.69 | 2.48 | −2.23 |
| RAR-M78's leg (anchor) | 2.79M | 6.91M | 2.48 | 2.28 | −1.87 |

- ln(R_off / R_on) +0.027, ln(R_solo / R_on) −0.247, ln(R_b9 / R_on)
  −0.047: no condition lifts R by 15%.
- **Outcome line:** `ABSENT: the head's four threads run at 3.4 times one
  thread or more in games; no condition lifts R by 15%`. R_on is 0.04 over
  the line.
- No Elo is read, as registered.

**Read after the result, from the same files, not registered readings:**

- **The binary does not carry the anchor's gap.** RAR-M78 played
  `ebbbc917…`, the d2 head before `e057d84`; this leg plays the head with
  it, which [RAR-P52](RAR-P52.md) read 2.4% faster at four threads. That is
  ln 0.024 of the ln 0.33 between 2.48 and 3.44.
- **One game at a time costs the four-thread side**
  (`solo_same_openings.py`, `solo_same_openings.log`). From the same 20
  openings, `on`'s 40 games ran the four-thread side at 10.15M and the
  one-thread side at 2.73M (R 3.71); `solo`'s ran them at 7.77M and 2.89M
  (R 2.69). With its game alone on the host the one-thread side is 6%
  faster and the four-thread side 23% slower. The registered tags looked for
  a lift only, so this is not a registered reading. RAR-R31's replay, one
  process searching move after move alone in the slot, kept 3.78; in the
  `solo` games the two engines take turns, so each four-thread search
  starts after the opponent's move with the rest of the host idle.

## Disposition

- **ABSENT, by the rule:** RAR-M78's 2.48 was its sessions' host state, not
  a fixed condition of the games. Its threads leg is recorded as confounded
  (RAR-M78, *Later reads*), and Rarog's four-thread scaling is measured again
  before D.2 reads anything from its effective speedup. What that state was
  is not known: RAR-M78's two sessions (2026-10-30 and 2026-11-01) both ran
  low and flat ([RAR-R31](RAR-R31.md)'s `pgn_drift.log`), and the same
  configuration today does not.
- **Large pages carry no loss:** off reads 2.7% higher in R, far from 15%.
  D.8 is designed as written and the Windows default stays.
- **B.9 scales no better in games here:** 3.28 against the head's 3.44;
  RAR-M74's 3.72 for B.9 was another day's host.
- **H7 changes shape.** How much speed four threads keep in games depends on
  the host's state, by up to a third: three games at once 3.44 (3.71 on
  `solo`'s openings), one game alone 2.69, RAR-M78's sessions 2.48. Reckless's
  3.44 in RAR-M78 was another session's host, so "Rarog sensitive, Reckless
  not" is no longer established. One game alone is the nearest of these to a
  rating list that plays one game a machine, so the solo loss is the next
  read: one game at a time against three, Rarog and Reckless on one host
  state, registered before any game.

## Calibration

- REPRODUCED was predicted 0.80 and ABSENT 0.08: a miss in the instrument.
  The prediction took RAR-M78's anchor as a property of the configuration;
  its own reasoning named "unless the host has changed" and priced it at
  0.08.
- NEIGHBOURS was predicted to lift R (0.30); one game at a time lowered it
  by 28% on the same openings (ln −0.32): a miss in sign. The prediction
  assumed other games only take shared resources from the four-thread side;
  alone on the host it keeps less of its speed than beside two other games.
- LARGE PAGES (0.40) and B.9 SCALES (0.50) did not fire: LARGE PAGES matches
  the more likely reading, B.9 SCALES a coin flip.
- Levels: R_on 2.5 (80% 2.2 to 3.0) read 3.44, outside; R_off 2.8 (2.3 to
  3.6) read 3.53, inside; R_solo 2.7 (2.3 to 3.5) read 2.69, inside but by
  the opposite mechanism to the one predicted; R_b9 3.0 (2.4 to 3.7) read
  3.28, inside.

## Audit 2026-11-02

- The `off` leg ran in a session where the four-thread speed loss was absent
  (R_on 3.44), so it shows that large pages cost nothing in the healthy
  state; it cannot clear them for the slow one, which no leg of this run
  reproduced at three games at a time. "Large pages carry no loss" above is
  read in that light, and D.8's order paragraph is corrected.
- The depth column (4T − 1T) averages mate-score moves in, where the head's
  four-thread side reports 7 to 10 plies and its one-thread side 24 to 28,
  while B.9 reports 41 to 45 on both sides. Over the other moves: `on` −0.25,
  `off` −0.15, `b9` −0.51, `solo` −0.93 (`analysis/d2_audit_2026-11-02.md`).
  At R 3.44 the 0.25 is RAR-R29's X 0.87 plus the four-thread side's shorter
  own time; in `solo` it is about twice what R 2.69 predicts.
