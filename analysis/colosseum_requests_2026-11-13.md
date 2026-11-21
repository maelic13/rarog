# Colosseum CLI: requests for the next CLI release (2026-11-13)

For the Colosseum maintainer. The maintainer implements these for the next CLI
release, after the GUI and the CLI are in separate repositories (maintainer
decision 2026-11-13: Colosseum CLI stays Rarog's main harness). Each request is
proved from artifacts, not inferred. Evidence: `docs/experiments/RAR-M88.md`
and `analysis/artifacts/harness-speed-2026-11-13/` (ignored storage).

Priority.

- **First:** 1, tuning speed, and 4, moves-per-period clocks, the only
  experiment Colosseum cannot run at all.
- **Then the measurement guards:** 5 and 6.
- **Then fidelity and coverage:** 2 and 3.
- **The rest are conveniences:** 7 to 10.
- **For the next oracle read (RAR-O09) on Colosseum:** 2, which its clock
  legs need, then 11 to 13, which let its reads be checked from the run's
  own files. Nothing else in the CLI blocks it: 0.5.0's `match` already
  takes a node budget, a move time and a clock for each side. The rest is
  on Rarog's side (its wrapper, PLAN D.7.1.1).

Items 4 to 13 were checked against the pinned 0.5.0 (`81EA6CBA…`): its
`--help`, its source and its run artifacts.

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
- **On a match between different engines** (added for RAR-O09). The two
  time managers do not lose the same share. Reckless's soft bound reads
  the move number (`src/time.rs:35`, `0.0594 - 0.0492 * exp(-0.0386 *
  fullmove)`): at `3.380+0.034` it is 51 ms at move 1 against 93 ms at
  move 9, before its multiplier, where Rarog loses 15%. A clock match
  between them on
  Colosseum therefore does not replicate RAR-O08's fastchess match, and
  the clock-use share RAR-O09 reads would be measured on reset counters.

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

## 4. Moves-per-period time controls

**Observed.**

- `match --help` offers per side only `--*-movetime-ms`, `--*-base-ms`,
  `--*-increment-ms`, `--*-nodes` and `--*-depth`.
- Neither `colosseum-core/src/time.rs` nor `colosseum-uci/src/position.rs`
  mentions `movestogo`. The `go` line is `go wtime btime winc binc` only.

**Why it matters.**

- PLAN F.10's gate is a CCRL 40/15 or Blitz list rating, and 40/15 is 40
  moves per period, repeating.
- Rarog's time manager has a separate `movestogo` branch
  (`src/search/time.rs`), which no Colosseum game can reach.
- fastchess plays these controls (`tc=40/900`). This is the one experiment
  family Rarog cannot run through Colosseum at all.

**Request.** Per side, a time control of N moves in T, repeating, with an
optional increment.

- The engine receives `movestogo` (the moves left in the period).
- The clock is refilled at each period boundary.
- The PGN `TimeControl` tag is written in the standard form (`40/900`).
- `match`, `sprt`, `tournament` and `gauntlet` all accept it.

**Check.**

- A smoke at `10/5` (10 moves in 5 s) logs `movestogo 10` on each side's first
  move, `1` on the tenth, and the refilled clock on the eleventh.
- A Rarog game at `40/15`-shaped conditions shows its explicit-`movestogo`
  branch active, read from the engine's own debug output.

## 5. Placement for engines with more than one thread

**Observed.**

- With `--placement auto` and `--cores-per-game 4`, a game's slot is both SMT
  siblings of each of its cores. Windows then packs four busy threads onto two
  or three cores' siblings.
- Rarog's 4T/1T NPS ratio read 2.5–2.7 in that slot (RAR-M74, RAR-P46,
  RAR-M78 to RAR-M81)
  against 4.03 with an explicit one-logical-CPU-per-core list (RAR-M82, on
  0.5.0). Reckless was packed the same way.
- Rarog's wrapper (`tools/colosseum.ps1`) refuses a sibling in an explicit
  list but cannot stop `auto` with Threads above 1.

**Request.** When any engine in a slot runs more than one thread, automatic
placement gives the slot one logical CPU per physical core and no SMT sibling.
The other way is to refuse `auto` there with a message naming the explicit
form. The resolved configuration says which policy applied.

**Check.** A 4T `match` with `--placement auto` resolves to one logical CPU per
core, and Rarog's 4T/1T NPS ratio reads near 4 in it, as RAR-M82's did.

## 6. Host load recorded during the run

**Observed.**

- The run record's `host` block holds static facts only: OS, architecture,
  logical CPUs and capabilities (RAR-M88's `run-record.json`).
- Rarog's idle check runs once, before the start. Nothing records whether
  something else ran on the game CPUs during play.
- That is what voided RAR-M48's first pool. The four-thread reads needed an
  external `typeperf` sampler (RAR-M81, RAR-M82).

**Request.** While a run plays, sample each logical CPU's busy time outside the
run's own engine processes, a few times a second, and record a summary in the
run record: per game CPU, the share of seconds above a threshold. Then:

- flag a run in `results.txt` whose game CPUs were disturbed beyond a stated
  limit;
- optionally, pause between games while the host is busy.

**Check.**

- A run with a CPU-bound process started on one game CPU partway through is
  flagged, and names that CPU.
- An undisturbed run is not flagged.

## 7. Clock left on every move

**Observed.**

- Colosseum's PGN move comments carry `s`, `d`, `t`, `h` and `n`, but not the
  clock left.
- `games.jsonl` holds clock summaries per game, not per move.
- fastchess writes `tl=` on every move. RAR-M88 had to reconstruct Colosseum's
  clock from the time control and `t=`.

**Request.** Add the mover's clock after the move to each move comment (or to
`games.jsonl`), in milliseconds.

**Check.** On a smoke game, each move's clock equals the previous one for that
side, minus `t`, plus the increment.

## 8. The conversion read in `results.txt`

**Observed.** Since 0.5.0, `results.txt` carries colour, terminations, faults,
time losses and per-side search. It does not carry the conversion read that
Rarog's gauntlet registrations use (RAR-M73):
`tools/diag/conversion_audit.py` counts the draws and losses after a side has
held at least a minor piece's material advantage for 12 plies, with a lone
minor excluded, per 1,000 games.

**Request.** Report that count per engine in `results.txt`, with the threshold
and persistence stated. Rarog keeps its PGN-based tool as the harness-neutral
check.

**Check.** On a recorded gauntlet PGN, Colosseum's count equals
`conversion_audit.py`'s.

## 9. The SPSA checkpoint verdict in `spsa status`

**Observed.**

- PLAN rule 7c reads a tune at iteration 2,000 and every 1,000 after, by a
  per-coordinate gradient test.
- `tools/spsa_read.py`, shared with Basilisk, does that from the run directory.
- Someone must stop the run at each checkpoint, read it, and resume or end it.

**Request.**

- `spsa status` prints the checkpoint verdict: quiet, live, a centre pushed
  outward at a bound, or faults.
- The run file can register checkpoints, so the run stops itself at a quiet
  checkpoint and continues on a live one.
- The statistics must match `spsa_read.py` exactly, which keeps its synthetic
  tests (`tools/diag/test_spsa_read.py`) as the reference.

**Check.**

- On a recorded tune, `spsa status` agrees with `spsa_read.py` at every
  checkpoint.
- A registered run stops itself at the first quiet checkpoint.

## 10. Wait for an idle host before starting

**Observed.** Rarog's wrapper refuses a start when the host is above 15% CPU
over a 2-second window (`Assert-HarnessHostIdle`, `tools/harness_common.ps1`).
On 2026-10-24 the Claude desktop app's own bursts refused six starts in a row,
on a host a 12-second read showed at 0.2%. Long runs since start through
retry wrappers.

**Request.**

- An option to wait for an idle host before the first game, with a stated
  window, threshold and deadline.
- The run record records the reading it started on.

**Check.**

- A run started beside a busy process waits, then starts within a few seconds
  of that process ending.
- It gives up at the deadline with a clear message.

## 11. `winc` and `binc` carry the mover's own increment

**Observed.** `colosseum-engine/src/runner.rs`, `move_limits` (lines
1369 to 1386 at the pin) builds the clock command as

```
GoLimits::Clock { wtime: clocks.white, btime: clocks.black, winc: inc, binc: inc }
```

with `inc` the increment of the side to move's own control. With a clock per
side (`--a-increment-ms 100 --b-increment-ms 113`) the engine to move is told
its opponent has its own increment. The remaining times are right.

**Effect.**

- **None for Rarog or Reckless.** Both read only their own side's increment
  (`src/search/time.rs:149-152`; Reckless `src/uci.rs`).
- **An engine that reads the opponent's clock** is misinformed in every
  time-odds game. Equal clocks are unaffected.

**Request.** `winc` from White's control and `binc` from Black's.

**Check.** A game at 10000+100 against 11270+113 sends `winc 100 binc 113`,
or the reverse by colour, on every `go`.

## 12. A fixed match's score in `results.txt` and the run record

**Observed.**

- A `match` run's `results.txt` holds wins, draws and losses by colour,
  terminations, faults, time losses and the per-side search figures, but no
  Elo, nElo or pentanomial line (`tools/results/rar-s124-stc/results.txt`).
- Its `run-record.json` holds `official_sample.pentanomial` as five zeros:
  `colosseum-cli/src/composition/match_command.rs:380` writes
  `pentanomial: [0; 5]`. RAR-S124's run record shows 2,000 scored games,
  1,000 completed pairs and `[0, 0, 0, 0, 0]`.
- The counts exist: `checkpoint.json` holds `[24, 252, 474, 221, 29]` for
  that run, and the run record's progress fields hold `Ptnml` and
  `nElo -6.2 +/- 15.2` as strings. Rarog's wrapper recounts the PGN to get
  them.

**Request.**

- Fill `official_sample.pentanomial` from the paired sample.
- Add the pentanomial, Elo and nElo with their intervals to `results.txt`,
  as an `sprt` run reports them.

**Check.** On a recorded match the run record's five counts equal a recount
of the PGN by pair, and `results.txt` carries the figures the progress output
printed.

## 13. Node-budget compliance in the summary

**Observed.**

- In a fixed-node game the harness sends `go nodes N` and nothing else, with
  a 600-second deadline (`runner.rs`, `FIXED_SEARCH_DEADLINE`). It cannot
  count nodes; the PGN's `n=` is the engine's last `info` line
  (`colosseum-uci/src/process.rs:655-657`).
- `results.txt` and the telemetry give each side's mean and median reported
  nodes only. An engine that overshoots or ignores its budget does not show:
  in `tools/results/smoke-cli050-match` (20,000 nodes) the largest `n=` was
  19,994 and the mean 13,815; in `c03-lazy-nodes` (150,000 nodes, CLI 0.2.0)
  3,868 of 239,963 moves reported more than the budget, the largest 150,006.
- RAR-O09's first match gives each side its own budget, and its validity
  rule is "no move reports more than its side's budget plus one".

**Request.** In a run with a node budget, report for each side the largest
reported node count and the number of moves whose reported count exceeds
that side's budget, in `results.txt` and the telemetry.

**Check.**

- On `smoke-cli050-match` the summary reads a largest count of 19,994 and no
  move over budget.
- An engine started with a budget it ignores is named, with its count of
  moves over budget.

**Not requested.** Limits finer than a millisecond: a clock of 3.380+0.034
against 3+0.03 gives 1.127 on the base and 1.133 on the increment, a
difference too small to matter here.

## For the GUI, after the split

The GUI's database (`%APPDATA%/Colosseum/data/colosseum.sqlite`) has an empty
`engines` table. Engine names, versions and paths are only in
`tournament_engines.engine_config_json` (`$.meta.name`, `$.meta.version`,
`$.path`), so a join on `engines` returns nothing.
