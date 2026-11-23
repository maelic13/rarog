# RAR-S137 — D.6.7's review on the compact kernel: how often its searches reach the ply cap, replaying its own games at their node counts times 1, 10/3 and 20 — REGISTERED 2026-11-22, before any count is read

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-22, before any replay count was
  read. The read is the agent's (deterministic, no games).
- **Kind:** a zero-game screen, PLAN D.6.7's review on the new kernel.
- Disposition: open — read 2026-11-22: 0.511% of searches reach the cap at 20 times the games' node counts (none at 1 or 10/3), all in endings at iteration 16 or deeper; D.6.7 stays open for the maintainer's decision on the form.
- **Baseline:** `d6-review` at `b90e983`, the engine source of `9507abd`
  (behaviour of `master`'s `94d7f72`), `bench 13` 3,390,842 / EBF 2.317.

## The question

The kernel ends a line at ply 127 (`MAX_PLY` 128). It handles that node as
the donor does: a draw by rule first, a draw in check, otherwise the raw
evaluation. The classic kernel's defect, a static evaluation in check or at
a draw, is gone. What is left is how often the cap truncates a line.

The kernel holds a line to eight times its iteration's depth before
quiescence (`LINE_BOUND`, chosen at zero games and held by no fit). From
iteration 16, eight times the depth is at least 128, so the cap binds before
the bound. The donor's cap is 240. RAR-S122's retry trigger, frozen on the
classic kernel: a read at a rated time control that finds more than 0.1% of
searches reaching the cap.

## Instrument

- **Binary.** `analysis/artifacts/d67-review-2026-11-22/rarog-s137.exe`
  (SHA-256 `54f998d9…`), a release build of the clean tree at `b90e983`,
  `id name Rarog dev-20261122-b90e9837`, `bench 13` 3,390,842 / EBF 2.317
  on the copy itself.
- **The cap's reach, from the engine's own output.** Negamax and quiescence
  set seldepth to ply + 1 before the cap test, so a search reached the cap
  exactly when its largest reported seldepth is 128. No patch.
- **Script.** `analysis/artifacts/d67-review-2026-11-22/replay_cap.py`
  (SHA-256 `18ddfb21…`), frozen with this registration.
- **Positions.** RAR-M91's games, the merged kernel's own
  (`dev_test_frontier.pgn`, 8,800 games with the kernel). 60 games are
  sampled with seed 20261122. One engine process a game, Hash 128 as the
  games had, one thread, the game's moves played in order. At each of the
  kernel's moves, `go nodes` at the move's recorded count (`n=`) times k.
- **States.** k = 1 is `3+0.03` itself; k = 10/3 and k = 20 stand in for
  `10+0.1` and `60+0.6` on the same positions and table history, as in
  RAR-S121.

## Wire

1. `bench 13` reads 3,390,842 on the measured copy. Passed before
   registration.
2. The counter is live: RAR-S108's race position,
   `8/2p2P1P/2n2P2/1k6/p7/6N1/P5K1/8 b - - 0 44`, reports seldepth 128 at
   `go depth 22`. Passed before registration (`replay_cap.py wire`).

## Prediction (frozen)

- X_1, the share of searches reaching the cap at k = 1: 0 (probability
  0.8). At `3+0.03` the kernel's mean depth is 15.0 (RAR-M91), so the line
  bound, 120 plies at depth 15, binds before the cap.
- X_10/3 at or under 0.1% (probability 0.65).
- X_20 above 0.1% (probability 0.55): there most iterations reach 16 or
  more, where the cap binds first.
- Most likely failure of the prediction: the cap is reached at k = 10/3 in
  more searches than predicted, because the kernel's lines run longer than
  the classic's did under its two-times bound.

## Rule (frozen)

- If X_10/3 and X_20 are both at or under 0.1%, D.6.7 closes `NO_CHANGE`:
  the cap does not reach a rated time control's searches often enough to
  read.
- If either is above 0.1%, the defect has an instance on the kernel. D.6.7
  stays open and the review goes to the maintainer with the measured
  exposure and the forms, before any change. The forms are the cap raised
  to the donor's 240 with the band constants PLAN lists, or the line bound
  tied to the cap.
- Diagnostics beside the verdict, not part of it: the share of searches
  at iteration 16 or deeper, and the number of games with a cap search.

## Exposure, stated

Seen before registration: the measured copy's `bench 13`, the wire, and
two hand runs on the race position and on bench position 15
(`8/8/p1p5/1p5p/1P5P/P1P5/8/K1k5 w`), where the cap is reached at depth 17
and depth 18 had not finished after 60 seconds. Unexposed: every replay count.

## Result

Read 2026-11-22, once, after registration. 60 of the kernel's 8,800 games,
4,107 searches at each scale, every replay exit 0; a watchdog sampling each
engine's CPU time every 20 seconds flagged no stalled process.

| k | Searches at the cap | Games with one | At iteration 16 or deeper | Median depth | Largest seldepth |
|---|---|---|---|---|---|
| 1 | 0 (0.000%) | 0 | 18.3% | 13 | 123 |
| 10/3 | 0 (0.000%) | 0 | 53.3% | 16 | 108 |
| 20 | 21 (0.511%) | 14 | 80.8% | 21 | 128 |

**Verdict by the frozen rule:** X_20 is 0.511%, above 0.1%, so the defect
has an instance on the kernel. D.6.7 stays open and goes to the maintainer
with this exposure and the forms, before any change.

**Where the cap is reached.** All 21 searches are endings, 4 to 12 men,
none in check at the root, at iterations 16 to 99: 13 at 16 to 25, where the
line bound (eight times the depth) lets a line run past the cap, and 8 at 37
or deeper, 4 of them at 64 or more, where twice the iteration reaches the
cap by itself. Below iteration 16 the bound holds every line under the cap,
which is why no search reaches it at k = 1 or 10/3.

**Calibration.** All three predictions held: X_1 = 0, X_10/3 at or under
0.1%, X_20 above 0.1%. The failure mode named, more reach at k = 10/3, did
not occur: the largest seldepth there was 108.

**Beside the verdict.** Bench position 15, `8/8/p1p5/1p5p/1P5P/P1P5/8/K1k5 w`,
reaches the cap at depth 17 (11.6M nodes, 3.3 s), and depth 18 had not
finished after 60 seconds: a fixed-depth search can sink time in such an
ending once its lines meet the cap. That is the clock's question, D.6.20's.

**Artifacts** (`analysis/artifacts/d67-review-2026-11-22/`, ignored):
`replay_x1.jsonl` (`8ea39245…`), `replay_x10_3.jsonl` (`effec9cd…`),
`replay_x20.jsonl` (`d3b4d12b…`), their `.log` files, `run.sh`,
`run_status.txt`, `watchdog.ps1` and `watchdog.log`.
