# RAR-R31 — The games' four-thread depth deficit, replayed: RAR-M78's games searched again position by position, alone and under the games' conditions one at a time — REGISTERED 2026-11-02, before the read; READ 2026-11-02

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

## Result — 2026-11-02, appended after exposure

Run by the maintainer on 2026-11-02 from about 10:00 to 11:05, host busy 0.8%
before and 0.3% after; 20,656 searches, none without an info line
(`replay_read.log`; `replay_read.json` `98543862…`).

| Cell | Depth (last line) | Main thread's exact depth | NPS | Wall ms | Closing lines (depth over the exact) |
|---|---:|---:|---:|---:|---|
| `dyn-1T-slot` | 17.76 | 17.71 | 2.94M | 240 | 0.016 (+0.86) |
| `dyn-4T-slot` | 17.84 | 17.56 | 11.11M | 238 | 0.072 (+0.59) |
| `dyn-4T-cores` | 17.79 | 17.49 | 11.24M | 235 | 0.071 (+0.63) |
| `dyn-4T-x3` | 17.59 | 17.30 | 10.50M | 231 | 0.073 (+0.71) |
| `dyn-1T-cleared` | 18.28 | 18.24 | 2.88M | 255 | 0.016 (+0.81) |
| `dyn-4T-cleared` | 18.19 | 17.85 | 11.12M | 247 | 0.081 (+1.12) |
| `dyn-1T-movetime` | 18.29 | 17.86 | 2.97M | 236 | 0.043 (+0.51) |
| `dyn-4T-movetime` | 18.47 | 18.06 | 11.12M | 237 | 0.103 (+0.77) |
| `dyn-4T-h1024` | 17.71 | 17.41 | 11.32M | 230 | 0.070 (+0.70) |
| `b9-1T-slot` | 18.85 | 18.82 | 2.42M | 246 | 0.014 (+0.94) |
| `b9-4T-slot` | 18.68 | 18.49 | 9.23M | 255 | 0.086 (+0.71) |
| `b9-4T-x3` | 18.58 | 18.38 | 8.62M | 256 | 0.091 (+0.75) |

- **Δ_clock +0.08** (90% −0.13 to +0.33): **NOT REPRODUCED.** Alone, under
  the games' own clocks with the table kept, four threads report the same
  depth as one. The exact-line Δ is −0.15 (−0.36 to +0.11), B.9's Δ −0.17
  (−0.47 to +0.23).
- Cleared table −0.09, the game's own time +0.19, three at once less alone
  −0.25 plies (−0.35 to −0.17), Hash 1024 less 256 −0.13 (−0.23 to 0.00).
- **NPS four threads over one, alone in the slot: 3.78 for `dyn`, 3.81 for
  B.9**, against the games' 2.28. One thread a core +1.2%; three at once
  −5.7% (B.9 −6.9%).
- **Fidelity:** on the four-thread side's own moves the replay reports
  **+1.38 plies** (90% +1.15 to +1.59) more than the games did; on the
  one-thread side's +0.14 (−0.05 to +0.31).
- Closing lines: 7.2% of four-thread searches against 1.6% at one thread,
  **+5.6 points: H5 ACTIVE**, the closing result 0.6 plies deeper than the
  main thread's own exact line on average.
- **Outcome line:** `NOT REPRODUCED: under half a ply less alone; H5 ACTIVE`.

**Read after the result, from the same files, not registered readings:**

- On the same moves the games ran slower than the replay
  (`same_moves_nps.log`): the four-thread side at **6.21 M against the
  replay's 11.13 M** (56%; about 61% once the 9% held after the last line is
  allowed for), the one-thread side at 2.67 M against 2.94 M (91%). The
  games' binary (`ebbbc917…`, the d2 head before `e057d84`) is about 2.4%
  slower at four threads than `dyn` ([RAR-P52](RAR-P52.md)), which explains
  little of it.
- The games' four-thread NPS is flat over the match (`pgn_drift.py`,
  `pgn_drift.log`): 6.28 to 6.32 M in each fifth of every slot's sequence. It
  is present from the first games, not built up over the engine process's
  life.

## Disposition

- **The games' depth deficit is not the engine's own.** Alone, with the
  games' positions, clocks and kept tables, four threads report the same
  depth as one at 3.78 times its NPS. In the games the four-thread side ran
  at about 60% of that speed and reported 1.38 plies less on the same moves,
  while the one-thread side lost about 9% and 0.14 plies.
- **By the rule** (NOT REPRODUCED, fidelity +1.38 on the four-thread side,
  +0.14 on the one-thread side, neither LOAD nor PLACEMENT fired): the games'
  conditions, not the engine alone, cost the four-thread side. The next read
  rebuilds the harness. No engine change is in scope until the condition is
  located, and [RAR-M78](RAR-M78.md)'s result is recorded as measured under
  it.
- **H7, the condition,** is now the card's lead. The replay rebuilt the
  slot's placement and three processes at once, and neither costs more than
  6%. What it did not rebuild: the games' binary, the opponent's process
  sharing the slot, the long-lived engine processes Colosseum keeps per slot,
  and Colosseum's own way of starting and pinning them. Reckless kept 3.44
  under the same harness, so whatever the condition is, Rarog is sensitive to
  it and Reckless is not. Whether CCRL's testers would meet it is part of the
  question.
- **H6 is refuted:** Hash 1024 reads 0.13 plies less, not more.
- **H5 is active** but held: a vote that chooses a helper's move in about 6%
  of four-thread searches matters only once the speed loss is located.
- **H3 stands as a fact about this Lazy SMP, not as the deficit's cause:**
  four threads search 3.78 times the nodes for the same depth, so whatever
  they add is width, which only games price.

## Calibration

- NOT REPRODUCED, predicted 0.40: a hit. The level missed: Δ_clock +0.08,
  above the 80% interval (−1.3 to −0.1). The prediction expected part of the
  deficit to be the engine's own; none of it is.
- NPS four over one, predicted 3.2 (2.6 to 3.6) for `dyn` and 3.4 (2.8 to
  3.8) for B.9: 3.78 and 3.81, above and at the top. The slot's eight logical
  CPUs cost nothing measurable (PLACEMENT, given 0.20, did not fire).
- Fidelity on the four-thread side, predicted +0.3 to +1.0: +1.38, a miss in
  magnitude; on the one-thread side, predicted −0.3 to +0.3: +0.14, a hit.
- B.9 predicted 0.2 to 0.5 plies nearer zero than `dyn`: −0.17 against
  +0.08, a miss; both are zero within their intervals.
- Secondary readings: CARRIED TABLE (0.45) did not fire, a miss in mechanism:
  the shortest searches' gap in the games came with the condition, not the
  kept table. LOAD (0.30), TIME MANAGEMENT (0.25), H6 (0.05) did not fire; H5
  (0.50) did.

## Audit 2026-11-02

- The question's 1.87 and 2.05 plies average mate-score moves in (1.05 over
  the other moves), so the replay was asked to reproduce a figure that was
  largely a reporting artefact. Its registered readings stand.
- The fidelity on the four-thread side's own moves, +1.38, is +1.33 over the
  non-mate moves (every record joined to its move comment, 0 mismatches in
  depth or nodes; `analysis/artifacts/d2-audit/replay_fidelity_nonmate.log`):
  in RAR-M78's slow state the games' four-thread side was about 0.7 plies
  shallower than its 60% speed alone explains. That points at a state that
  does not slow every thread alike, a clue for the sampler D.2 now runs
  beside every four-thread leg (`analysis/d2_audit_2026-11-02.md`).
