# RAR-M93 — The `d9` head against Basilisk's rebuilt evaluation: the "Frontier HCE" Colosseum GUI round robin, six engines, 9,331 of 15,000 games at `3+0.03`, stopped — OBSERVED 2026-11-26: 3,502.3, 76.5 below Basilisk's development build

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** started 2026-11-26 14:40 UTC by the maintainer in the
  Colosseum GUI (v1.2.1), tournament "Frontier HCE"
  (`4db7de7e-7da8-4496-8816-5416c6d6368e`), on the match host, and stopped
  at 9,331 of its 15,000 games. Recorded by the agent from the GUI's
  database, opened read-only.
- **Why:** the maintainer's read of where Rarog's development head stands
  against Basilisk's latest build. That build rebuilt its evaluation in
  classical Stockfish's form and refitted it on its own games (Basilisk
  `ff65589` and `41d952f`, after `v1.11.0`).
- **Kind:** an observation. Nothing was registered and no prediction was
  frozen, so it accepts and rejects nothing. It is a partial run: every
  pairing has the same 311 complete colour-reversed pairs, so the reads
  below are balanced, but they are not the 15,000 games the tournament
  was set up for. That the host was idle is not established.
- Disposition: observation — the `d9` head rates 3,502.3 [3,492.6,
  3,511.4] on RAR-M89's scale, unchanged since D.7 (RAR-M90 3,508.1,
  RAR-M91 3,500.6); Basilisk's build rates 3,578.8 [3,569.5, 3,587.7], and
  head to head the `d9` head is −77.2 [−96.4, −58.4]; no fault.

## The field

| Engine | Binary (`D:\chess\engines\`) | SHA-256 | Note |
|---|---|---|---|
| Rarog dev-20261126-0231c3ed | `rarog\rarog-dev-20261126-0231c3ed-windows-x64-pext.exe` | `9d520ad7…71f1` | the maintainer's build of `d9` at `0231c3e`; `bench 13` 3,694,298 / EBF 2.324 on this file (`dev_bench.out`), the fingerprint of `master` at `70ac02e` |
| Basilisk dev-20261126-41d952f2 | `basilisk\basilisk-dev-20261126-41d952f2-windows-x64-pext.exe` | `c65ee8ce…e813` | Basilisk at `41d952f`, "Rebuild the evaluation" |
| Stockfish 16 HCE | `stockfish-16-x64-bmi2-classical-pgo.exe` | `041396a4…5442` | RAR-M89's file and options |
| Stockfish 11 | `stockfish\stockfish-11-x64-bmi2.exe` | `271b62dc…5b02` | RAR-M89's file and options |
| Rarog 2.7.0 | `rarog\rarog-2.7.0-windows-x64-pext.exe` | `9dad2c99…df57` | the release |
| Basilisk 1.11.0 | `basilisk\basilisk-1.11.0-windows-x64-pext.exe` | `f6ca0a44…d756` | the release; RAR-M89's B.11.1 head (RAR-P59) |

`d9` differs from `master` in D.9.1's option and loading repairs only. They
leave every default and the fingerprint as they were, and tablebases were
off, so the `d9` head plays as `master` does.

## The run

Round robin, set for 500 cycles of two colour-reversed games (15,000
games); stopped with 9,331 finished and 5,669 pending. `3+0.03`, one
thread, Hash 128, 14 games at a time, no ponder, no adjudication, no move
cap, no tablebases; UHO `UHO_Lichess_4852_v1.epd` in random order, eight
plies, seed 202611261540. One finished game has no colour-reversed partner
and is left out; 4,665 pairs remain, 311 a pairing. Every game ended by
the rules: 6,085 mates, 2,134 repetitions, 791 insufficient material, 233
fifty-move draws, 88 stalemates. The GUI's forfeit column is empty for
every engine.

## Ratings

Bradley–Terry on game scores, fitted on RAR-M89's scale: the four engines
RAR-M89 rated keep their RAR-M89 mean, 3,499.4, and every engine is free.
95% intervals from 200 resamples of colour-reversed pairs within each
pairing (`rating.py`, RAR-M91's method). The GUI's own column (3,502 and
3,578 for the two development builds, written back from a start of 3,550,
which its Δ column shows as −48 and +28) agrees to within 1.

| # | Engine | Rating | Score | RAR-M89 |
|---|---|---|---|---|
| 1 | Stockfish 16 HCE | 3,599.0 [3,591.9, 3,606.7] | 64.0% | 3,604.7 |
| 2 | Basilisk dev-20261126-41d952f2 | 3,578.8 [3,569.5, 3,587.7] | 60.8% | |
| 3 | Stockfish 11 | 3,550.1 [3,542.8, 3,555.7] | 56.1% | 3,542.4 |
| 4 | **Rarog dev-20261126-0231c3ed** | **3,502.3** [3,492.6, 3,511.4] | 48.2% | |
| 5 | Rarog 2.7.0 | 3,438.5 [3,429.9, 3,444.8] | 37.7% | 3,444.6 |
| 6 | Basilisk 1.11.0 | 3,409.9 [3,400.9, 3,417.4] | 33.3% | 3,405.7 |

The four anchored engines move by −6.1 to +7.7 against RAR-M89, inside the
scale's spread between two fields that RAR-M91 measured (about ten). With
four anchors in six engines, this scale is less firmly tied than RAR-M91's
nine in twelve.

## Head to head

311 colour-reversed pairs each; Elo with the 95% pentanomial interval.

| Pairing | W-D-L | Elo | Earlier read |
|---|---|---|---|
| `d9` head – Stockfish 16 HCE | 116-214-292 | −101.1 [−121.0, −81.8] | −103.3 (RAR-M91) |
| `d9` head – Basilisk dev-20261126-41d952f2 | 132-222-268 | −77.2 [−96.4, −58.4] | |
| `d9` head – Stockfish 11 | 161-219-242 | −45.5 [−64.6, −26.7] | −46.3 (RAR-M91) |
| `d9` head – Rarog 2.7.0 | 235-269-118 | +66.1 [+48.2, +84.5] | +79.1 (RAR-M91) |
| `d9` head – Basilisk 1.11.0 | 297-190-135 | +92.6 [+72.6, +113.2] | +86.4 (RAR-M91) |
| Basilisk's build – Stockfish 16 HCE | 163-251-208 | −25.2 [−42.9, −7.6] | |
| Basilisk's build – Stockfish 11 | 224-224-174 | +28.0 [+8.5, +47.7] | |
| Basilisk's build – Basilisk 1.11.0 | 351-204-67 | +171.3 [+151.1, +192.6] | |

## What it shows

- **Rarog has not moved since D.7.** The `d9` head rates 3,502.3 against
  the merged kernel's 3,500.6 (RAR-M91) and the adopted kernel's 3,508.1
  (RAR-M90), and reads −101.1 head to head with Stockfish 16 HCE against
  −103.3 and −100.6. D.8's simplifications and D.9's repairs were meant to
  be neutral, and this read finds no change.
- **Basilisk's rebuilt evaluation moved it about +170.** It reads +171.3
  over 1.11.0 head to head, and its commits report +85.2 and +85.3 at
  `3+0.03` for the refit and the rebuild. It now stands 20 below
  Stockfish 16 HCE in the fit and −25.2 head to head, and 76.5 above the
  `d9` head in the fit and +77.2 head to head. The search did not change
  between its two builds here: about 1.44M to 1.49M nodes a second and
  depth 18.3 to 18.6 in this table's means.
- **Rarog's own measured lever is the same kind.** RAR-O13 (E.4.0.1) read
  Stockfish 16's classical evaluation inside Rarog's kernel at +68.8 ±
  10.4 on the clock and +80.3 ± 10.8 at equal nodes. E.4.4 is the
  rebuild in `sf_11`'s form that E.4.0's decision accepted. That is
  Rarog's evidence; Basilisk's result is a prior, not evidence for Rarog.
- **Robustness:** no forfeit, crash or time loss by either Rarog in its
  3,110 or 3,111 games.

Speed and depth by engine, each engine's mean of per-game values over its
finished games. These are leads, not measurements: every engine counts
nodes and reports depth its own way, and Colosseum's depth includes moves
with a mate score.

| Engine | NPS | Depth | Move ms |
|---|---|---|---|
| Stockfish 11 | 2.006M | 21.04 | 73.0 |
| Stockfish 16 HCE | 1.951M | 24.36 | 73.6 |
| Basilisk 1.11.0 | 1.493M | 18.64 | 76.1 |
| Basilisk dev-20261126-41d952f2 | 1.440M | 18.29 | 74.6 |
| **Rarog dev-20261126-0231c3ed** | **1.400M** | **14.73** | 73.4 |
| Rarog 2.7.0 | 1.350M | 12.64 | 73.6 |

**Artifacts:** `analysis/artifacts/frontier-hce-2026-11-26/` (ignored):
`frontier_hce.pgn`, the 9,331 finished games exported from the database
(52,525,937 bytes, `664bd855…91ec`); `tournament_config.json`
(`728d114c…7e63`); `rating.py` (binaries, the fit, the bootstrap, the
head-to-head reads and the speed table, `7e6bc4e3…0975`) and its output
`rating.out` (`880a8dac…da65`); `dev_bench.out`, the Rarog build's bench
(`8e304982…5c87`); `standings.png`, the GUI's standings
(`d72f8b07…52bc`).

**Retry trigger or `closed`:** closed as an observation. If the
tournament is resumed and finished, `rating.py` reads it again unchanged,
and this record takes the final figures as an addendum.
