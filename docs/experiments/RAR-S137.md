# RAR-S137 — D.6.7's review on the compact kernel: how often its searches reach the ply cap, replaying its own games at their node counts times 1, 10/3 and 20 — REGISTERED 2026-11-22, before any count is read

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-22, before any replay count was
  read. The read is the agent's (deterministic, no games).
- **Kind:** a zero-game screen, PLAN D.6.7's review on the new kernel.
- Disposition: open — registered 2026-11-22, no count read yet.
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

Not read yet.
