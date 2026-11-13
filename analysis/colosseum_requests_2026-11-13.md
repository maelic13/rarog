# Colosseum CLI: requests from RAR-M88 (2026-11-13)

For the Colosseum maintainer. The maintainer passes these on once the GUI and
the CLI are in separate repositories. Each request is proved from artifacts,
not inferred. Evidence: `docs/experiments/RAR-M88.md` and
`analysis/artifacts/harness-speed-2026-11-13/` (ignored storage).

## 1. Keep an EPD opening's move counters

**Observed.** `colosseum-engine/src/openings.rs`, `epd_fen_into`, keeps an EPD
line's first four fields and always appends `0 1`.

- `UHO_Lichess_4852_v1.epd` carries FEN counters on every line (for example
  `… w KQkq - 2 4`).
- In RAR-M88's matches, all 500 Colosseum games started from `0 1`. fastchess
  1.8.3 kept the book's own counters (`0 3` to `0 7`, and non-zero halfmove
  clocks such as `1 5`).

**Effect.**

- **On the engine.** Engines whose time manager uses the game ply see a
  different game.
  - Rarog's optimum scales with `(ply + 3.23)^0.469`.
  - On the same 250 positions and the same clock, Rarog thought 102.9 ms on
    move 1 under Colosseum against 121.7 ms under fastchess (1.18×), and
    searched 0.36 ply shallower. The formula predicts 1.20× to 1.25×.
- **On the 50-move rule.** Resetting the halfmove clock moves the 50-move
  draw.
- **On comparisons.** Games per hour across runners are confounded by about 3%
  on this book.
- **Not affected:** within one Colosseum run, both sides get the same openings,
  so gates are fair.

**Request.**

- When an EPD line has two trailing integer fields after the four position
  fields and before any opcode, use them as the halfmove clock and fullmove
  number. Otherwise default to `0 1`, as now.
- Optionally, take the EPD `hmvc` and `fmvn` opcodes when they are present.
- Record in the resolved configuration which source the counters came from.

**Check.**

- A book line `… w KQkq - 2 4` reaches the engine as `position fen … 2 4`.
- On RAR-M88's book, ply-1 think time matches fastchess's within noise.
