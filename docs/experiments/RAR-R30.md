# RAR-R30 — The four-thread holds: where Rarog spends the time between its last `info` line and `bestmove`, from RAR-M78's games and a timed read at one and four threads — REGISTERED 2026-11-01, before the timed read; READ 2026-11-02

Indexed under *4. Root search, time management and SMP* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-01 by the agent; the games half is
  read, the timed half is the maintainer's (about a minute).
- **Leaf:** PLAN D.2, the four-thread card's read 3 (DRIVER-14, DRIVER-15).
- **Question.** RAR-M78 found 28% of Rarog's four-thread moves held over the
  20 ms margin (p99 175 ms), against 2% at one thread and under 1% for
  Reckless and Stockfish. Where does that time go: the join of the helpers,
  the vote and the output path, or the search itself?
- **Kind:** zero-game diagnostic. A defect in the stop path would be
  repaired under D.1's clock rules; anything else enters the card.

## What `h=` is

- `h=` is Colosseum's per-move `overhead`: the time from the engine's last
  `info` line to its `bestmove`, plus that line's own lag of about 1 ms. In
  all 2,000 game sides of RAR-M78's Rarog threads leg, the side's largest
  `h=` equals the game record's `overhead_ns` within 1 ms, where
  `last_info_to_bestmove_ns` alone misses 257
  (`analysis/artifacts/d2-holds/h_definition_check.py`, `47a76494…`, and its
  `.log`).
- **It is inside the clock.** The charged time `t=` runs from the `go` write
  to the `bestmove` arrival (`go-write-to-bestmove-arrival`) and contains
  `h=`: no move has `t` more than 1 ms below `h` (307 do by exactly 1 ms,
  rounding; 299 of them are `t=0 h=1`). RAR-M78's "held outside its own
  clock" is corrected here: the hold is charged thinking time that produced
  no reported iteration, not time the budget never sees, and no forfeit risk.

## The games, read 2026-11-01 (file reads only)

`analysis/artifacts/d2-holds/pgn_holds.py` (`8a9830b3…`) over the six legs' `games.pgn`
(`pgn_holds.log`; the move counts, mean depths and percentiles reproduce the
manifests' lines exactly):

| Side | Moves | Time held / time charged | `h` p50 / p90 / p99 ms | Over 20 ms | Depth, `h` ≤ 20 | Depth, `h` > 20 |
|---|---:|---:|---|---:|---:|---:|
| Rarog 1T | 64,392 | 0.012 | 1 / 1 / 53 | 2.0% | 17.30 | 16.36 |
| Rarog 4T | 64,486 | **0.090** | 4 / 56 / 175 | 28.2% | **15.25** | 15.83 |
| Reckless 1T / 4T | 72,263 / 72,395 | 0.005 / 0.005 | 1 / 1–2 / 2–4 | 0.7% / 0.4% | 20.04 / 21.35 | — |
| Stockfish 1T / 4T | 80,362 / 80,541 | 0.001 / 0.001 | 0 / 1 / 1 | 0.0% | 26.37 / 29.13 | — |

- At four threads 9.0% of Rarog's charged time follows its last reported
  iteration, against 1.2% at one thread.
- **The holds do not explain the depth deficit.** On moves held 20 ms or less
  the four-thread side still reports 2.05 plies less than the one-thread side
  (15.25 against 17.30) at a similar time (194 against 220 ms).

## The code's account

- At one thread the root loop breaks when an iteration ends past the soft
  target. In a pool the soft stop is a vote: a thread past its own target
  casts one vote and keeps searching, and the pool stops at a strict
  majority, three of four (`mod.rs`, the soft-stop block; `shared.rs`,
  `votes_needed`). The main thread defers too, so it starts an iteration it
  will not finish, and nothing is reported until `bestmove`.
- After the stop, the join waits for every helper (`threads.rs`, the
  result loop), and the vote and the result line follow. The helpers check
  the stop flag every 128 nodes, so the join should cost little.

## The timed read

- `analysis/artifacts/d2-holds/hold_read.py` (`4199fc9a…`, ignored) on `rarog-d2ctl-pext-pgo.exe`
  (`a0a28509…`), one process per cell pinned to CPUs 2, 4, 6, 8, Hash 256,
  `ucinewgame` once, `bench`'s 16 curated positions twice over as in a game:
  - `movetime-1T`, `movetime-4T`: `go movetime 250`, so the overshoot past
    250 ms is the stop, join, vote and output path alone (`go movetime`
    skips the soft stop);
  - `clock-1T`, `clock-4T`: `go wtime 6000 btime 6000 winc 100 binc 100`,
    the engine's own time management as in games.
- Per search: t and the lag from the last info line to `bestmove`, read on
  arrival.
- **Outcome:**
  - **NOT REPRODUCED:** `clock-4T` lag p90 under 20 ms;
  - **JOIN:** `movetime-4T` overshoot p90 at 20 ms or more (with `clock-1T`
    under 20 ms);
  - **VOTE:** reproduced, the stop path fast, `clock-1T` under 20 ms: the
    hold is search after the main thread's last completed iteration, at four
    threads only;
  - **OTHER:** `clock-1T` lag p90 also at 20 ms or more.
- **Exposure:** a smoke of two positions a cell (`smoke.log`, 8 of 8
  searches with an info line); it prints no reading and its JSON was not
  opened.

## Prediction, frozen before the timed read

| Outcome | Probability |
|---|---|
| VOTE | 0.70 |
| NOT REPRODUCED | 0.15 |
| OTHER | 0.10 |
| JOIN | 0.05 |

- `clock-4T` lag over time 0.05 to 0.15 (the games' 0.090); `movetime-4T`
  overshoot p90 under 5 ms.

## Decision and stop rule

- **VOTE:** the hold is the soft-stop vote's extension of the main thread,
  a designed behaviour (RAR-R04 measured the 2T first-vote form at −15.85
  Elo), not a clock-side defect. It enters the card as the hypothesis that
  the deferred main iteration wastes about 9% of the four-thread time; it is
  priced only by games, under the leaf's 4T gate, after the zero-game reads.
- **JOIN:** a clock-side defect, repaired under D.1's rules (DRIVER-15's vote
  on the results received by a deadline).
- **NOT REPRODUCED:** the games' holds need the harness's conditions
  (opponent processes, unpinned threads); recorded, no further read until a
  hypothesis names the condition.
- One read; nothing changes after it starts.

## How to run

From PowerShell 7 on an idle host, nothing else running, about one minute:

```powershell
python D:\code\rarog\analysis\artifacts\d2-holds\hold_read.py D:\code\rarog\analysis\artifacts\d2-holds\hold_read.json 2>&1 | Tee-Object -FilePath D:\code\rarog\analysis\artifacts\d2-holds\hold_read.log
```

If it refuses with `HOST NOT IDLE`, run it again. The summary ends with an
`OUTCOME:` line.

## Result — 2026-11-02, appended after exposure

Run on 2026-11-02 at 00:09, host busy 0.3% before and 0.1% after; 128
searches, 32 a cell (`hold_read.log`; `hold_read.json` `c6ca6e01…`).

| Cell | t mean ms | Lag p50 / p90 / p99 ms | Lag / t | Over 20 ms | Overshoot p90 ms |
|---|---:|---|---:|---:|---:|
| `movetime-1T` | 235 | 13.9 / 76.5 / 130.9 | 0.101 | 0.34 | 0.5 |
| `movetime-4T` | 235 | 32.5 / 60.6 / 77.2 | 0.127 | 0.66 | 1.0 |
| `clock-1T` | 292 | 0.0 / 0.0 / 625.4 | 0.072 | 0.06 | — |
| `clock-4T` | 314 | 6.8 / 53.3 / 117.0 | 0.054 | 0.25 | — |

- **VOTE:** `clock-4T` lag p90 53 ms, the stop path fast (`movetime-4T`
  overshoot p90 1.0 ms), `clock-1T` lag p90 0.
- `clock-1T`'s lag over time is one search: the second pass of position 14
  ran to 1,203 ms and answered 625 ms after its last info line (the first
  pass held 49 ms at the same time), an unfinished last iteration under the
  limit. Without it one thread holds 0.6% of its time, against 5.4% for four.
- Under `go movetime` both thread counts end inside an unfinished
  iteration by design, so those cells show only that the stop is fast.

## Disposition

- The hold is the soft-stop vote's extension of the main thread, a designed
  behaviour and not a clock-side defect: no D.1 repair. H4 stays in the card
  as 5 to 9% of four-thread time spent in an iteration never reported,
  priced only by games under the leaf's 4T gate, after the zero-game reads.

## Calibration

- VOTE, predicted 0.70: a hit. `clock-4T` lag over time 0.054, inside the
  predicted 0.05 to 0.15 at its low edge and below the games' 0.090 (16
  positions, a fast clock, no opponent). `movetime-4T` overshoot p90 1.0 ms,
  under the predicted 5 ms.

## Audit 2026-11-02

- The table's depth columns (15.25 against 17.30 on moves held 20 ms or
  less) average mate-score moves in, where the four-thread side reports 7 to
  10 plies and the one-thread side 23 to 27; over the other moves the
  four-thread side of RAR-M78's leg is 1.05 plies shallower, of which the
  leg's R 2.48 and this read's hold explain most. "The holds do not explain
  the depth deficit" compared against the contaminated figure; the VOTE
  reading and the hold accounting stand (`analysis/d2_audit_2026-11-02.md`).
