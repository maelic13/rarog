# Colosseum CLI: requests for the next CLI release (2026-11-13)

For the Colosseum maintainer. The maintainer implements these for the next CLI
release, after the GUI and the CLI are in separate repositories (maintainer
decision 2026-11-13: Colosseum CLI stays Rarog's main harness). Each request is
proved from artifacts, not inferred. Evidence: `docs/experiments/RAR-M88.md`
and `analysis/artifacts/harness-speed-2026-11-13/` (ignored storage).

Priority: 1 (tuning speed) is the important one; 2 is fidelity; 3 is coverage.

## 1. SPSA without an iteration barrier (priority)

**Observed.**

- `spsa` runs one iteration at a time. `colosseum-application/src/spsa.rs`
  prepares an iteration (`prepare_iteration`), plays its mini-match and commits
  it (`commit_iteration`) before preparing the next.
- RAR-M88's two Colosseum tunes ran 15 slots and 30 games an iteration. In
  every one of their 32 iteration boundaries, the next iteration's first game
  started at or after the previous iteration's last game ended (gap 0.00 s,
  `decompose.txt`).
- An iteration therefore lasts as long as its slowest slot, and slots that
  finish early sit idle.
  - Slot fill in the tunes was 80.4% and 78.4%, against 98.2–98.4% in the
    matches on the same slots, engine and clock.
  - The engines' share of slot time was 78.3% against 97.1%.

**Gain available.** If tunes filled the slots as the matches do, a tune would
play about 1.24× the games per hour (98.3 / 79.4). That is about 4,864 to
about 6,000 games an hour on this host. A 6,000-iteration tune at 30 games an
iteration (180,000 games) would take about 30 hours instead of 37. The cause
is the barrier, not the per-game cost: the gap between iterations is zero, so
the loss is idle slots inside each iteration.

**Request.** Keep every slot busy across iteration boundaries. Fishtest's
answer is one perturbation per game pair, its result applied as it arrives;
another is starting the next iteration's pairs on free slots while the
current one finishes. The design is the Colosseum maintainer's. Whatever it
is must keep:

- **Exact replay.** The journal records, for each committed unit, which
  perturbation it played and the centre it was played at, and the order in
  which updates were applied. `tools/spsa_read.py` rebuilds every
  perturbation sign from the master seed (ChaCha12, stream
  `colosseum-rng-v1`, iteration-major, knob order) and checks every recorded
  centre against the runner's update; that check must remain possible.
- **Resume.** A stopped run resumes from its own directory, as now.
- **The stated schedule.** The gain schedule (`a`, `c`, `A`, `alpha`,
  `--r-end`) is stated per update. If the unit becomes a pair, the run file
  says how the schedule is scaled, so a tune can be compared with the
  30-game-iteration tunes before it.
- **A synchronous mode.** The current behaviour stays available as an option,
  for a cross-check against it.

**On the Rarog side (not Colosseum's).** Rarog's SPSA rule and reader are
written in 30-game iterations:

- PLAN rule 7c's checkpoints (iteration 2,000, then every 1,000);
- `spsa_read.py`'s `elo_per_d(games_per_iteration)`, even and odd cross-fitting,
  and 500-iteration segments.

Adopting a barrier-free mode means restating those in games or in the new unit
and re-running `tools/diag/test_spsa_read.py`. That is a Rarog change, made
when the release lands.

**Check.**

- On RAR-M88's tune (b23core surface, 15 slots, `3+0.03`, 510 games, an idle
  host), slot fill above 95% and games per hour at least 1.15× RAR-M88's 4,864.
- `spsa_read.py`, adapted, reproduces every recorded centre.
- A synchronous run is unchanged from 0.5.0's.

## 2. Keep an EPD opening's move counters

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

**Note for Rarog.** This changes what every Colosseum run plays from this
book. When Rarog pins the release, its development fingerprint is unaffected
(bench does not use the book), but gate conditions change. The re-pin's runner
checks (as RAR-M70 did for 0.3.0) should note it.

## 3. Release builds for Linux on ARM and Intel macOS

**Observed.** Colosseum CLI 0.5.0's release assets cover Windows x64 and
arm64, Linux x64 and macOS arm64. fastchess 1.8.3 also ships Linux arm64
(and RISC-V) and macOS x64.

**Request.** Add `aarch64-unknown-linux-gnu` and `x86_64-apple-darwin` to the
release matrix, if those platforms are wanted. Rust cross-builds make this
mostly CI work.

**Check.** Each new asset runs `colosseum-cli --version` and a short `match`
on its platform.
