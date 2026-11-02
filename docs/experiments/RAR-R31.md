# RAR-R31 — The games' four-thread depth deficit, replayed: RAR-M78's games searched again position by position, alone and under the games' conditions one at a time — REGISTERED 2026-11-02, before the read

Indexed under *4. Root search, time management and SMP* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-02 by the agent; the maintainer runs
  it (about 70 minutes).
- **Leaf:** PLAN D.2, the four-thread card's read 5, by
  [RAR-R29](RAR-R29.md)'s rule for X under 1.2.
- **Question.** In RAR-M78's games the four-thread side reported 1.87 plies
  less depth than the one-thread side (2.05 on moves without a hold), but
  RAR-R29's time-to-depth read accounts for about 0.15 plies at equal time.
  Is the deficit the engine's own, under its clock with the table kept, or
  does it need the games' conditions? And which condition carries it: the
  table kept across moves, the clock, the threads' placement on the slot's
  cores, or the neighbouring games?
- **Kind:** zero-game diagnostic. It decides where the card looks next;
  nothing about strength follows.

## Read from the games before registration (file reads only)

`analysis/artifacts/d2-gamecond/pgn_nps.py` (`852eb922…`, `pgn_nps.log`) and
`pgn_slots.py` (`6d8a1bf8…`, `pgn_slots.log`) over the threads legs'
`games.pgn` and `games.jsonl`:

- **In play, four threads run at 2.28 times one thread's NPS** (6.29 M
  against 2.76 M, as RAR-M78's manifest), and the ratio is the same at every
  search length: 2.26 under 100 ms, 2.30 at 500 to 1,000 ms, 2.39 over a
  second. Alone, pinned one thread a core, the same binary's ratio is 3.57
  (RAR-R29, eight positions) to 3.66 (RAR-P52's positions). A steady loss,
  not a start-up cost.
- **Not the cache domain.** Slot 0 has CCD0 to itself, slots 1 and 2 share
  CCD1, and Rarog's four-thread NPS is the same in all three (6.26, 6.41,
  6.20 M). Reckless keeps 3.44 under the same placement.
- **Depth by search length** (four threads against one): 13.31 against 17.52
  under 100 ms, 15.00 against 16.35 at 200 to 300 ms, 17.48 against 19.09
  over a second. The largest gap is in the shortest searches, where one
  thread reaches 17.5 plies in about 50 ms from its kept table.
- **The slot.** Colosseum gave each game four physical cores as eight
  logical CPUs (slot 0: CPUs 2–9), the affinity enforced on both engines,
  three games at once; RAR-R29 and RAR-P52 pinned one logical CPU per
  physical core and ran alone.

## Design

- `analysis/artifacts/d2-gamecond/replay_read.py` (`9dc43860…`, ignored).
- **Games:** RAR-M78's Rarog threads leg (`tools/results/selfscale-rarog-threads/games.pgn`),
  game numbers 1 to 12, six openings with both colours. Every engine move of
  both sides is replayed (the book's eight plies are in the start FEN).
- **Replay.** For each game, cell and side, one engine process searches that
  side's positions in order, `ucinewgame` once at the start, so its table and
  histories carry from its own earlier searches as in the game. Each search
  is `position fen <start> moves <the game's moves so far>` and `go wtime W
  btime B winc 100 binc 100`, with both clocks as the game recorded them
  before that move (from 10,000 ms, less each move's charged `t` plus the
  increment). The game's own moves are played whatever the replay chooses.
  Both sides' positions are replayed in every cell, so every one-thread and
  four-thread cell searches the same positions with the same clocks.
- **Arms:** `dyn`, the d2 head with `e057d84` (`rarog-d2dyn-pext-pgo.exe`,
  `a545b6b8…`), and `b9`, the B.9 head (`rarog-bis-24aefb4-pext-pgo.exe`,
  `2e4e2820…`). The games ran `ebbbc917…`, the d2 head before `e057d84`. Each
  process keeps its binary's defaults but Hash and Threads (`LargePages` on
  for `dyn`, as in the games; B.9 predates the option).
- **Cells:**

  | Cell | Arm | Threads | CPUs | Mode | Hash |
  |---|---|---:|---|---|---:|
  | `dyn-1T-slot` | dyn | 1 | slot 0 (2–9) | clock | 256 |
  | `dyn-4T-slot` | dyn | 4 | slot 0 | clock | 256 |
  | `dyn-4T-cores` | dyn | 4 | 2, 4, 6, 8 | clock | 256 |
  | `dyn-4T-x3` | dyn | 4 | slots 0, 1, 2 at once, three processes | clock | 256 |
  | `dyn-1T-cleared`, `dyn-4T-cleared` | dyn | 1, 4 | slot 0 | clock, `Clear Hash` before every search | 256 |
  | `dyn-1T-movetime`, `dyn-4T-movetime` | dyn | 1, 4 | slot 0 | `go movetime` the game's own `t` for the move | 256 |
  | `dyn-4T-h1024` | dyn | 4 | slot 0 | clock | 1024 |
  | `b9-1T-slot`, `b9-4T-slot`, `b9-4T-x3` | b9 | 1, 4, 4 | as `dyn`'s | clock | 256 |

  The cells run game by game, their order rotated each game. About 1,290
  searches a cell (3,870 in the x3 cells).
- **Per search:** the depth of the last `info` line before `bestmove` (what
  the games' `d=` records), its nodes and time; the main thread's deepest
  exact line before any closing line; and whether the search ended on a
  **closing line**. Before `bestmove` the engine prints the chosen result's
  line when no earlier line named its move: a helper's move chosen by the
  vote, or a move that failed high and was never confirmed. It is the closing
  line when the last line names a different first move than the line before.

## Readings and judgement

Depth differences are paired by position, a bootstrap over games gives 90%
intervals; NPS is the cell's summed nodes over summed time of the last lines.

- **Primary, Δ_clock** = depth(`dyn-4T-slot`) − depth(`dyn-1T-slot`):
  - **REPRODUCED** at −1.0 ply or less;
  - **PARTLY REPRODUCED** from −1.0 to −0.5;
  - **NOT REPRODUCED** above −0.5.
- **Secondary readings,** each named in the outcome line when it fires:
  - **CARRIED TABLE:** Δ_clock − Δ_cleared at −0.5 or less (the kept table
    widens the deficit); **CARRIED TABLE HELPS** at +0.5 or more.
  - **TIME MANAGEMENT:** |Δ_clock − Δ_movetime| at 0.5 or more (the clock's
    decisions, the soft-stop vote among them, differ from the game's time).
  - **PLACEMENT:** NPS(`dyn-4T-cores`) / NPS(`dyn-4T-slot`) at 1.10 or more.
  - **LOAD:** NPS(`dyn-4T-x3`) / NPS(`dyn-4T-slot`) at 0.90 or less.
  - **H6 TABLE SIZE:** depth(`dyn-4T-h1024`) − depth(`dyn-4T-slot`) at
    +0.3 or more.
  - **H5 ACTIVE:** the share of searches ending on a closing line, four
    threads less one, at 5 points or more.
- **Also printed:** the exact-line Δ; B.9's Δ_clock; depth x3 less alone;
  NPS 4T / 1T in the slot for both arms; B.9's load ratio; and the
  **fidelity**: the replay's depth less the games' `d=` on each side's own
  moves (`dyn-4T-slot` on the four-thread side's, `dyn-1T-slot` on the
  one-thread side's), with the games' own depths on these twelve games.
- Each search without an info line is counted and left out; over 2% makes
  the read invalid. An interrupted read is rerun whole.

## Exposure before registration

- The games' files above, read for this registration.
- The judgement code's self-test on synthetic records
  (`judge_selftest.py`, `judge_selftest.log`). It printed the games' own
  depths on these twelve games: **16.41 for four threads, 17.60 for one,
  −1.19**, smaller than the leg's −1.87.
- One smoke game, four plies a side in every cell (`smoke.log`): 128 of 128
  searches with a line, masks and hashes held; it prints no reading.
- The wire of `Clear Hash` (`wire.log`): nodes to depth 10 on a fresh table,
  a kept one, after `Clear Hash`, and kept again are 38,360, 17,092, 36,664
  and 2,310 for `dyn` (41,468, 12,350, 37,338, 11,396 for B.9).

## Prediction, frozen before the read

| Primary outcome | Probability |
|---|---|
| NOT REPRODUCED | 0.40 |
| PARTLY REPRODUCED | 0.35 |
| REPRODUCED | 0.25 |

| Secondary reading | Probability it fires |
|---|---|
| CARRIED TABLE (widens) | 0.45 |
| CARRIED TABLE HELPS | 0.05 |
| TIME MANAGEMENT | 0.25 |
| PLACEMENT | 0.20 |
| LOAD | 0.30 |
| H6 TABLE SIZE | 0.05 |
| H5 ACTIVE | 0.50 |

- **Levels:** Δ_clock −0.5 (80% interval −1.3 to −0.1); NPS 4T / 1T in the
  slot 3.2 for `dyn` (2.6 to 3.6) and 3.4 for B.9 (2.8 to 3.8); B.9's
  Δ_clock 0.2 to 0.5 plies nearer zero than `dyn`'s; fidelity on the
  four-thread side's moves +0.3 to +1.0 plies (the replay alone deeper than
  the games), on the one-thread side's −0.3 to +0.3.
- **Reasoning.** RAR-R29 puts the engine's own cost at about 0.15 plies at
  equal time from a cleared table; with the slot's placement and its NPS
  somewhat below the pinned 3.6, and the shortest searches' gap from the
  kept table, the replay should show some of the games' deficit but not all
  of it. The games' NPS of 2.28 at every length points at the games'
  conditions for the rest; the two conditions this read rebuilds (the slot
  and three processes) are the obvious ones, and neither is certain to be
  the one that matters.

## Decision and stop rule

- **REPRODUCED or PARTLY:** the deficit is at least partly the engine's own
  under its clock. The secondary readings name the mechanism to pursue:
  CARRIED TABLE and [RAR-R32](RAR-R32.md)'s members point at what the
  helpers write and how the main thread uses it (H3); TIME MANAGEMENT at the
  soft-stop vote (H4), which then goes to games sooner; H5 ACTIVE registers a
  read of the vote's choices.
- **NOT REPRODUCED with the four-thread fidelity at +0.5 or more and the
  one-thread fidelity within ±0.3:** the games' conditions, not the engine
  alone, cost the four-thread side. If LOAD or PLACEMENT fired, that
  condition is the next read's subject; if neither did, the next read
  rebuilds the harness itself (a Colosseum match with the replay's
  instrument). No engine change is in scope until the condition is located,
  and RAR-M78's result is recorded as measured under those conditions.
- **PLACEMENT:** a candidate behaviour-neutral repair (the engine placing
  its own threads), which needs research before any change.
- **LOAD:** the engine's memory footprint per thread enters the card.
- **H6 TABLE SIZE:** Hash at four threads enters the card; otherwise H6 is
  refuted.
- One read; nothing in the games, cells, clocks or thresholds changes after
  it starts.

## How to run

From PowerShell 7 on an idle host, nothing else running, about 70 minutes:

```powershell
python D:\code\rarog\analysis\artifacts\d2-gamecond\replay_read.py D:\code\rarog\analysis\artifacts\d2-gamecond\replay_read.json 2>&1 | Tee-Object -FilePath D:\code\rarog\analysis\artifacts\d2-gamecond\replay_read.log
```

If it refuses with `HOST NOT IDLE`, run it again. The summary ends with an
`OUTCOME:` line.
