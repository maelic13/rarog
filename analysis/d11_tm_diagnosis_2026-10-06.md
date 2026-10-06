# D.1.1: why the head spends its hard maximum in won endings — NO_CHANGE

Research for PLAN D.1.1 (pulled forward by the maintainer on 2026-10-06),
recorded as RAR-R13. The question comes from
`analysis/basilisk_review_2026-10-06.md`, item 1: on B.5.2's rec1 and rec2,
without tablebases, the frozen head (engine source `ee02ed1`, binary
`tools/test_engines/rarog-b9head-pext-pgo.exe`, `aac92114…01ee`,
11,171,726 / EBF 2.512) spends its hard maximum, 52–58% of the clock, where
Stockfish 19 does not, and on rec3 it stops at 10.4 s and depth 21 at both
`60000+600` and `180000+2000`.

**Verdict: `NO_CHANGE`.** Each behaviour's mechanism is established below
from per-iteration traces. In 5,400 games of the head's search, every move
that ran into the maximum in a won or mate position was played in a game the
head won, bar one draw whose long move was an ordinary low-clock overrun
rather than one of these stalls. Even if every unconverted won game with a
won-position time sink is charged to that sink, the effect is about half an
Elo, which no gate can resolve. The repair candidates all sit in territory a
gate has already rejected or that belongs to D.1.2's bound model. Nothing
lands ahead of Phase C, and Phase C's frozen head stays at engine source
`ee02ed1`.

Evidence is in `analysis/artifacts/d11-tm-diagnosis-2026-10-06/` (ignored
storage): `trace_rec.py` and its three traces, `tm_games.py` and
`tm_games.txt`, and `tm_unconverted.py` and `tm_unconverted.txt`.

## Instrument

The traces are **untimed**. Each runs `go infinite` on one thread with
Hash 64 in a fresh process, every UCI line logged and stopped after a wall
budget. On one thread the limits only stop the search, so up to a stop the
tree is the one a clock search builds, and node counts are exact. Each trace
reproduces the review's last line at both clocks: rec1 depth 32
`cp 1302 lowerbound`, rec2 depth 56 `mate 5`, rec3 depth 21 `cp 951`. Times
in the traces are wall times on an unchecked host and are read only as
approximations. Where a time matters (rec3), the review's idle-host total
anchors it. No timed run was made, so no idle host was needed.

Game frequency comes from `tm_games.py`. It rebuilds the clock per move from
`base + increment − t=`, which is the review's method, and takes the optimum
and maximum from `tm_table.py`'s exact transcription of `time.rs`. The score
`s=` (from the mover's view; `#N` marks a mate) and the depth `d=` come from
each move comment. "Won" means `s ≥ +500` cp; "mate+" means a mate for the
mover. The first version of the parser skipped comments that carry only `t=`
(Critter sometimes omits score and depth), which flipped side attribution in
the gauntlet. The script now reads every comment and refuses one without
`t=`, and the numbers below come from the fixed run. Three runs were read:

| Run | Ledger | Control | Games | Head moves |
|---|---|---|---|---|
| `tools/results/b9-gauntlet` | RAR-M64 | `3+0.03`, against six engines | 2,400 | 153,501 |
| `tools/results/b10-ltc` | RAR-M65 | `10+0.1`, against 2.4.0 | 1,000 | 58,333 |
| `tools/results/c03-lazy-read` | RAR-E19 | `3+0.03`, self-play; arm `b` (LazyMargin 600) is the head's search and played evaluation in a tune build | 2,000 | 122,411 |

## rec1: a fail-high cascade whose tenth re-search never finishes

`7r/5R2/8/2k1PB2/8/4K3/8/8 w - - 0 86`. Depths 16–31 all read exactly
`cp 534`, and depth 31 completes at 6,598,627 nodes (about 1.6 s). At
depth 32 the root fails high nine times in a row:

| Re-search | Bound (`lowerbound`) | Beta = 534 + delta | Nodes in the window |
|---|---|---|---|
| 1 | 555 | 534 + 21 | 0.95 M |
| 2 | 570 | + 36 | 0.32 M |
| 3 | 593 | + 59 | 0.27 M |
| 4 | 627 | + 93 | 0.32 M |
| 5 | 678 | + 144 | 0.33 M |
| 6 | 755 | + 221 | 0.71 M |
| 7 | 870 | + 336 | 1.37 M |
| 8 | 1,043 | + 509 | 3.90 M |
| 9 | 1,302 | + 768 | 21.66 M |
| 10 | window [513, 1,691] | + 1,157 | did not finish: about 120 M by the stop at 40 s here, and not by 104 s in the review's run |

Each fail-high returns exactly its beta. The window grows ×1.5 + 5 and stays
centred on the stale 534 (`Aspiration::fail_high`), with alpha fixed at 513.
Stockfish reads `cp 9165` here, so reaching the true value would take about
six more widenings; the loop opens fully only after `AspMaxFails` = 20
failures. From the eighth re-search on, seldepth sits at 127, the ply cap
(`MAX_PLY` 128). Each higher beta makes the search prove more, deeper, and
from the sixth window on the cost grows two- to sixfold per step. The soft stop runs only
between completed iterations (`search_root`), so after about 1.6 s nothing
but the hard maximum can end the move. The best move played is depth 31's.

## rec2: a missing stop, not rec1's cascade

`8/P4k2/8/1N6/1P2B1K1/8/8/8 w - - 7 81`. `mate 6` appears at depth 9
(5 ms) and `mate 5` at depth 17 (24 ms, 78,494 nodes); every iteration from
there to depth 56 reports the same `mate 5` and the same PV (`a7a8q f7e7 …`).
The trace has **no bound lines**: `Aspiration::new` turns the window off when
the centre is a mate score, so this is not rec1's cascade. The cost per
iteration grows with depth: about 0.2 M nodes at depth 51, 0.56 M at 54 and
1.2 M at 55 (seldepth 127, the ply cap) and 56, which completes at about
1.43 s and 7.03 M nodes. Depth 57 then does not complete, in more than 38 s
here or about 100 s in the review's run.

So rec2 is a proved mate with no rule that ends the search, followed by an
iteration that finds nothing to cut and does not finish. Stockfish has no
clock-mode stop after a mate either (its mate stop serves `go mate`); its
depth 245 is its own depth cap (`MAX_PLY` − 1), which its iterations reach
while they stay cheap. Rarog's would reach `MAX_DEPTH` − 1 = 99 if its
iterations stayed cheap; they stop being cheap from about depth 52.

## rec3: the clock's soft stop, one iteration late

`1r6/R7/6k1/8/8/5PP1/6K1/8 w - - 6 72`. Depth 20 completes at about 1.8 s
(1,970 ms here, scaled by the review's idle total of 10,441 against 11,275
here). Depth 21 then fails high six times (696, 711, 734, 768, 819 and 896,
from about 2.0 to 6.6 s here), and its exact search completes at 10.4 s on
the idle host, at 46.3 M nodes. The best move is `f3f4` from depth 15 on.
After depth 21 the soft target is
`optimum × falling_eval × instability × effort`:

- `falling_eval` sits at its 0.572 floor, because the score is rising
  (0.1187 + 0.0221 × (prior average − 951) is below the clamp).
- With the last best-move change at depth 15, `tot_best_move_changes` has
  decayed to about 0.05 at depth 20 and 0.02 at depth 21, so instability is
  1.21 and then 1.155.
- Effort is unknown from outside and lies between 0.71 and 0.924.

| Clock | Optimum | Soft target after depth 20 | After depth 21 | Depth 20 ends | Depth 21 ends |
|---|---|---|---|---|---|
| `60000+600` | 4,413 | 2,169–2,822 ms | 2,070–2,694 ms | about 1.8 s: continue | 10.4 s: stop |
| `180000+2000` | 14,251 | 7,003–9,113 ms | 6,684–8,698 ms | continue | 10.4 s: stop |

Both targets fall inside the one iteration from 1.8 to 10.4 s, so both clocks
stop at its completion. That iteration is the mechanism, not a stop that
ignores the clock. It is rec1's cascade in a mild form that does complete:
at `60000+600` the move takes about four times its soft target. At
`180000+2000` it takes less than the optimum, because a rising score sets
`falling_eval` to its floor, as the shape intends.

## How often it happens in games, and what it costs

From `tm_games.txt`, as shares of the head's moves:

| | `3+0.03` gauntlet | `10+0.1` | `3+0.03` self-play |
|---|---|---|---|
| Moves in won (≥ 500 cp) / mate+ positions | 15.0% / 7.0% | 16.5% / 9.9% | 14.3% / 5.9% |
| Won-position moves at ≥ 0.9 × maximum | 111 (0.48%) | 171 (1.77%) | 99 (0.57%) |
| Mate+ moves at ≥ 0.9 × maximum | 26 (0.24%) | 150 (2.59%) | 33 (0.46%) |
| Of those, rec2's signature (mate shown, depth ≥ 40) | 20 | 101 | 17 |
| Head's result in games with a won or mate+ move at ≥ 0.9 × maximum | 137 of 137 moves in wins | 320 of 321 in wins, one draw | 132 of 132 in wins |
| Time past the optimum in won and mate+ positions, share of all head time | 4.4% | 5.1% | 4.4% |
| Games not won after three consecutive won own moves | 9 (1 loss, 8 draws) | 1 draw | 8 draws |
| …of which with a won-position move at ≥ 3 × optimum | 3 | 1 | 3 |

Both behaviours occur, and more often at the longer control, as the
mechanism predicts: iterations get deep enough to stall. Their cost to
results is nil in these records. The one `10+0.1` draw (game 742) and every
unconverted game with a won-position sink (`tm_unconverted.txt`) show the
ordinary overrun instead: depth 11–17, 3–6 × the optimum, below the maximum,
at clocks of 250–1,800 ms. None is rec1's or rec2's deep stall. Charging all
seven such draws to their sink gives 3.5 points in 5,400 games, about 0.5 Elo
near a 70% score. That is an upper bound, not an estimate. A repair case-2
read resolves about ±7.8 nElo in standard error, and an SPRT `[0,3]` cannot
separate 0.5 Elo from zero within any budget the maintainer would spend.

The general overrun is not specific to won endings. Moves at ≥ 3 × the
optimum are 4.7% of non-won moves at `3+0.03` and 1.8% at `10+0.1`. Time past
the optimum is 31.4% and 19.7% of all head time, by design in part, since
the soft target scales the optimum by up to about 2 × and a started
iteration is always completed. That is the bound model's question and goes
to D.1.2 as input.

## Why no repair is recommended now

I recommend against implementing a repair now: the behaviours are real, but
the measured cost is nil.

1. **The cascade lives in the aspiration loop.** B.5 measured that loop's
   re-searches as useful search at this control: the one-retry repair
   (`AspMaxFails` 1) was rejected at −18.3 ± 6.9 Elo (RAR-S89), and
   re-centring on the failing score read −4.52 (RAR-S17). The donors'
   shallower fail-high re-search reaches the same tree and was not
   implemented. Changing the loop would also move `bench 13`, which breaks
   the premise of the freeze exception (clock play only, `bench 13`
   unchanged).
2. **A clock-only containment** (stop after a fail-high once past the soft
   target, playing the fail-high move) would fire on every overrunning
   iteration, not only in won endings. That makes it a time-formula change,
   which needs an SPRT `[0,3]`, and it sits in D.1.2's audit: ADR-0065 lets
   only completed exact iterations adjust the optimum.
3. **A mate-proved stop** (end a clock search once a mate for the side to
   move is exact and stable) is narrow, cheap and correct, but it buys time
   only in games already won: 150 mate+ maximum moves in 1,000 `10+0.1`
   games, all in wins. It is a feature for a measured-zero problem, and
   Stockfish plays without it.
4. **The deep-iteration growth** (seldepth at the ply cap from depth 52 in
   rec2 and in rec1's late windows) is a search property. B.5 recorded it
   as "no bound on deep-iteration line growth" with no measured cost, and
   the search is frozen.

**Owner of the fix: D.1.2** (maintainer's condition for closing,
2026-10-06). The behaviours are real and grow with the control, so the time
manager's bound model after Phase C decides on an in-iteration stop and a
mate-proved stop. Its acceptance adds two checks: a timed rec1–rec3 probe in
which no move reaches the hard maximum, and `tm_games.py` on its gate games.
PLAN D.1.2 carries the detail.

**Retry trigger.** These pull the fix forward ahead of D.1.2:

- A game record at any rated control shows the head failing to win, or
  forfeiting, after a won position, with one of these stalls preceding the
  loss of the win: a move at ≥ 0.9 × the maximum at a depth of 40 or more,
  after a mate was shown, or ending on a root lower bound.
- A record at a control of CCRL blitz (`2+1`) or longer shows such moves
  costing results.
- D.1.2's candidate changes the bound model. It then reruns `tm_games.py`
  on its own games as a diagnostic.

**Evidence layer.** Traces are node and iteration structure; the game
frequencies are counts. Neither is Elo, and the half-Elo figure is an upper
bound from counts, not a measurement of strength.

**No prediction was registered.** The leaf is diagnosis, the readings decide
against implementation and no game is played, so nothing is calibrated.
