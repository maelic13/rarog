# Board-implementation speed: Rarog vs Basilisk vs Reckless vs Stockfish

**Date:** 2026-07-22 · **Tool:** `tools/perft_compare.py` (re-runnable) ·
**Machine:** the usual Zen 3 box, idle, single process at a time.

## What was measured and why it is comparable

Perft = move generation + make/unmake only — no search, no eval, no TT. It is
the purest available measurement of "the board implementation" in real use.

Comparability was verified in source, not assumed:

- **All four bulk-count at depth 1** (return `movelist.len()` instead of
  making leaf moves): Rarog `movegen.rs::perft`, Basilisk `engine.cpp::perft`,
  Reckless `tools/perft.rs::perft_internal`, Stockfish `perft.h` (leaf trick
  at depth 2 — same semantics).
- **All four generate fully legal moves** in this path and use make/unmake
  (no copy-make).
- **Reckless runs with `NullBoardObserver`** — its NNUE accumulator wiring is
  disconnected, so it pays board cost only, like the others.
- **Timing is external and identical for all engines**: wall clock from
  writing the go-command to reading the result line, on a live process (no
  startup cost included). Pipe latency ~ms against multi-second runs.
- **Every node count was checked against the reference perft values, every
  round, all four engines: all matched.** A speed number with a wrong node
  count would be meaningless; none was.

Binaries: Rarog `p103head-a-pext-pgo` (accepted head), Basilisk
`v1.9.0-pext-pgo` (latest release), Reckless `0.10.0-dev-45ea6a9f` (built
2026-07-22 from their master, release profile), Stockfish 18 official
`bmi2` build.

## Results (best of 3 rounds per cell, Mnps = million perft nodes/sec)

| Position | depth | nodes | Rarog | Basilisk | Reckless | Stockfish |
|---|---|---|---|---|---|---|
| P1 startpos (opening) | 6 | 119.1M | 300.6 | **318.4** | 164.2 | 174.7 |
| P2 kiwipete (tactical mg) | 5 | 193.7M | **375.4** | 364.6 | 247.0 | 215.9 |
| P3 rook endgame (EP pins) | 7 | 178.6M | 204.0 | **237.6** | 130.0 | 113.6 |
| P4 promo storm | 6 | 706.0M | 333.7 | **352.9** | 230.8 | 179.7 |
| P5 promo+checks | 5 | 89.9M | 322.7 | **354.4** | 227.4 | 201.7 |
| P6 quiet middlegame | 5 | 164.1M | 342.6 | **349.2** | 234.0 | 216.2 |
| **SUITE (weighted)** | | 1.45G | **311.4** | **331.3** | 206.2 | 175.2 |

Relative to Rarog: Basilisk **+6.4%**, Reckless **−34%**, Stockfish **−44%**.

## Reading the numbers honestly

1. **There is no board-implementation deficit.** Rarog's board is ~1.5× faster
   than Reckless's and ~1.8× faster than Stockfish's at raw legal-movegen +
   make/unmake, and within 6% of Basilisk — which shares its bitboard design
   DNA. The 1-thread NPS lead Rarog now enjoys (3.31M vs Basilisk's ~2.9M
   search NPS) is NOT being dragged by the board layer.

2. **The SF/Reckless numbers are not evidence their boards are "bad".** Both
   carry per-move bookkeeping that pays off elsewhere (SF: StateInfo chain +
   NNUE dirty-piece plumbing; Reckless: observer hook). They optimized for
   search throughput, not perft. The correct conclusion is directional only:
   nothing in Rarog's board needs a philosophy change.

3. **The one analyzable gap: Basilisk leads most where boards are sparse.**
   Per-position deficit vs Basilisk: P3 rook endgame **−14.1%**, P5 −8.9%,
   P1 −5.6%, P4 −5.4%, P6 −1.9%, P2 **+3.0%** (Rarog wins the densest
   position). A deficit that grows as positions get sparser = a fixed
   per-node/per-make overhead that dense positions amortize. Checked and
   ruled out: both engines fully recompute `checkers` per make in this path
   (`board.cpp:678` vs `make_move_inner`), so that is not it. Remaining
   candidates, in 8.12(c) profile-pass scope:
   - `compute_pinned` runs per `gen_moves` call in Rarog's perft path
     (the 10.3(5) pin sharing helps the *staged search* path, not plain
     `generate_legal_movelist`);
   - per-call `MoveList` (256-slot `MaybeUninit`) setup vs Basilisk's;
   - undo-info width (Rarog saves 4 keys + checkers + castling per make).

4. **Perft slightly understates Rarog's search-condition board speed**: in
   search, 10.3(3) passes the gives-check hint into `make_move` (checkers
   stored as `EMPTY` for the ~95% of moves that don't check), and 10.3(5/7)
   share the pinned set across stages. Plain perft uses neither. If anything,
   the search-path board gap vs Basilisk is smaller than the −6.4% here.

## Build on this

- Re-run any time with `python tools/perft_compare.py` (engines and
  positions are data at the top of the file; totals are cross-checked
  automatically, so a movegen regression fails loudly).
- The P3-style sparse-position deficit is the only thread worth pulling, and
  it belongs in **8.12(c)** (profile-first): confirm with a flamegraph over a
  perft P3 run before touching anything.
- If 8.12(a) (incremental accumulators) lands, re-run this suite: it adds
  per-make work, and this suite is the cheapest way to see its board-side
  cost in isolation.
