# D.6.29 screen, 2026-11-13: the stalemates quiescence scores as its stand pat

PLAN D.6.29 (`analysis/basilisk_check_2026-11-13.md` §1). Out of check with no
capture to search, quiescence never asks whether the side to move has a legal
move, so a stalemate reads as its stand pat. Basilisk's rebuilt quiescence
tests the narrow case, the side to move left with king and pawns.

## The instrument

Three diag-only counters in the quiescence entry (`src/diag.rs`,
`src/search/node.rs`; nothing changes in a production build): `q_kp_nodes`,
quiescence nodes out of check whose side to move has only king and pawns;
`q_kp_stalemate`, those with no legal move; `q_kp_stalemate_misread`, those of
them that reach the stand pat with it 100 cp or more from the draw. The
`diag` build's `bench 13` reads 9,077,016, the release's fingerprint, so the
counters do not move the search.

## Registered before the replay (frozen 2026-11-13)

Seen before this was written: `bench 13` only (below).

- **Sample:** the release head's side of the first 200 games of RAR-M87's
  head-to-head leg (`tools/results/rar-m87-h2h/games.pgn`), each game in one
  process (Hash 64, one thread, `ucinewgame` once), every head move searched
  with `go nodes` at the count the game recorded, moves followed as played.
  Driver `analysis/artifacts/d629-screen-2026-11-13/replay_d629.py`.
- **The quantity:** the share of replayed searches with at least one misread
  stalemate (`q_kp_stalemate_misread` > 0). It bounds from above the
  searches a repair could change; whether the root move would change is not
  read.
- **Stop rule:** under 1% of searches → `NO_CHANGE`: a repair that reaches
  fewer than one search in a hundred cannot move a gate, and its harm read
  would be noise. At 1% or more → a repair card, gated by what it moves.
- **Prediction:** 0.3% of searches, 80% band [0.03%, 2%]; P(`NO_CHANGE`) 0.8.
  Basis: bench finds 97 stalemates in 2.48M quiescence nodes (0.004%), and
  king-and-pawns positions are commoner in games than in bench, by an unknown
  factor.

## Results

`bench 13`, stride 1: 120,733 king-and-pawns quiescence nodes out of check in
2,482,373 quiescence nodes; 97 stalemates; 87 of them misread by 100 cp or
more.

**The replay** (`replay.json` SHA-256 `5f1a4379…`, driver `bccb9e53…`):
200 games, 12,520 searches, 419,837,709 quiescence nodes. King-and-pawns
nodes out of check are 4.27% of them (17,928,024); 59,053 are stalemates
(0.33% of those, against bench's 0.08%), and 41,790 of those reach the
stand pat 100 cp or more from the draw. **2,403 searches, 19.2%, hold at
least one misread stalemate**, in 164 of the 200 games; a search that
holds one holds 4 at the median and 37 at the 90th percentile.

**Decision by the frozen rule: 19.2% is above 1%, so D.6.29 becomes a
repair card** (PLAN D.6.29, `READY_FOR_IMPLEMENTATION`).

**Calibration.** Predicted 0.3% [0.03%, 2%]; the miss is in magnitude,
about sixty-fold, and in instrument: the prediction scaled bench's
per-node rate, but the registered quantity is the share of searches with
at least one, and a search of some 33,000 quiescence nodes in a pawn
ending reaches one easily even at 0.33% of its king-and-pawns nodes. The
assumption that failed was reading a per-node rate as a per-search share.

**What this does not show.** Whether any root move or score changes: a
misread stalemate inside a refuted line changes nothing at the root, and
the main search scores a stalemate correctly one ply up whenever the
capture is searched there. Evidence layer: node counts, which choose and
do not accept.
