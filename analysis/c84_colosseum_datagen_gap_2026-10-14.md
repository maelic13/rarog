# Datagen on Colosseum: the gap check (C.8.4 step 1), 2026-10-14

PLAN C.8.4 step 1 asks what the pinned Colosseum CLI lacks for the datagen
contract `tools/datagen.ps1` meets today through fastchess. Checked against
`colosseum-cli 0.2.0` (`5A4ECEDB…`, source `ca05dfa`), from its `--help`, its
source at that revision and short runs on an idle host (7% CPU, no engine or
harness process). The engine throughout is C.8.1's datagen engine
`rarog-c8head-pext-pgo.exe` (`E73E3230…`). Evidence:
`analysis/artifacts/c84/gap/` (ignored storage).

## Result

**No functional gap.** `colosseum-cli tournament run` with two copies of one
engine, `--games-per-pair 1` and `--cycles N` meets every item. A fixed `match`
does not: it always plays a colour-reversed pair on each opening
(`match_runner.rs`, book index = pair index), and `extract_parallel.py` already
discards the second game of a pair as a replay, so a match would halve the
yield.

| Contract item | Colosseum 0.2.0 | Checked by |
|---|---|---|
| Both sides at fixed nodes (8,000) | `--nodes 8000` (tournament), `--a-nodes`/`--b-nodes` (match) | smoke PGN `TimeControl "white nodes/8000; black nodes/8000"`; `n=` ≤ 8,000 per move |
| No adjudication | off unless `--draw-adjudication`/`--resign-adjudication` | resolved config `"draw":null,"resign":null` |
| Seeded random order from a start index, no wrap | `--book-order random --seed S --book-start I` (zero-based); a range past the book is refused unless `--book-wrap` | runs from start 0 (6 games) and start 3 (3 games) play the same positions at indices 3–5 |
| One game per opening | `tournament run --games-per-pair 1 --cycles N`; one opening per encounter | 6 games on indices 0–5, 900 games on 900 distinct openings |
| The same engine on both sides | accepted (`--engine X --engine X --label A --label B`) | the smokes |
| 200-move cap | `--max-moves 200`, full moves, as fastchess's `-maxmoves` | the longest of `hce-v5`'s 595,240 games is 392 plies; a cap counted in plies would stack games at 200 |
| Oversubscribed concurrency (30) | accepted with placement off | dry run at 595,240 games and concurrency 30; 900 games played at 30 |
| Hash 16, Threads 1 | `--option Hash=16 --option Threads=1` | resolved config |
| PGN and run record | `games.pgn`, `games.jsonl`, `run-record.json`, `resolved-config.json` per run directory | the smokes |

## Port items (Rarog side, not Colosseum requests)

1. **Score comments.** Colosseum writes `{s=175 d=6 t=10ms h=1ms n=7456}`, a
   versioned format (`colosseum-move-comment/3`; mate `s=#n`; mover's point of
   view, as fastchess). `extract.py`'s `COMMENT_CP` reads only fastchess's
   `+1.75/6 0.010s`. The score is used only for `--blend` below 1; the parser
   must read both formats.
2. **The frozen opening sequence.** Every regeneration since `hce-v3` replays
   the same starts: `phase_book_v1.epd`, fastchess seed 10403, from start 1
   (`analysis/eval_programme_2026-10-05.md`, section 9). Colosseum's shuffle
   is its own, so seed 10403 there is a different sequence. fastchess's
   `Round` tag is the opening's offset in its shuffled order: across the
   `b9head`, `e08head` and `c8head` corpora, the `c8head` pilot and a fresh
   900-game run, every shared round has the same start FEN (0 mismatches over
   612,747 rounds). So the port writes that sequence once as an ordered EPD
   and has Colosseum play it in sequential order. The longest corpus recovers
   612,747 starts; a regeneration needing more must first extend the sequence.
3. **Start index** is zero-based in Colosseum and one-based in `datagen.ps1`.
4. **Manifest and guards** stay the wrapper's: the schema
   `extract_parallel.py` checks, `-ExpectSha256`, `-ExpectFingerprint`, the
   book audit, refusing an existing output.

## Throughput (measured; a request only if the maintainer wants one)

Same 900-game workload (seed-10403 openings in each harness's own order,
8,000 nodes, Hash 16, concurrency 30, `phase_book_v1.epd`), alternated three
times:

| Harness | Wall seconds | Games/min | Mean plies |
|---|---|---|---|
| fastchess 1.8.0 | 25.1, 25.2, 25.1 | ≈ 2,150 | 79.5 |
| Colosseum 0.2.0 | 34.1, 34.4, 33.9 | ≈ 1,585 | 79.1 |

Colosseum takes about 36% more wall time for the same work; each harness
reproduced its own games exactly. Colosseum charges 8.52 ms a move, 2.14 ms of
it its recorded overhead (`h=`), against fastchess's 6.06 ms; at concurrency
14 the overhead is still 1.49 ms on a 3.53 ms search. Part of `h` is the
engine's whole-millisecond `info time`, so the wall time is the evidence and
`h` only locates it: per-move latency at very short searches. At `hce-v5`'s
595,240 games the difference is about 4.6 h against 6.3 h. Nothing in the
contract is lost, so it does not block the port.

## Port and equivalence (steps 2 and 3), 2026-10-14

**Port** (`8bd33d2`). `datagen.ps1 -Runner colosseum` (now the default;
`-Runner fastchess` is unchanged and prints the C.8.1 pilot's exact command)
runs `tournament run` as above on `-OpeningSequence`, the book's lines in
fastchess's seed order, read sequentially from `-Start − 1`.
`tools/texel/opening_sequence.py` writes that sequence and a sidecar binding it
to the book's SHA-256, the seed and its source PGNs. From the `b9head`,
`e08head` and `c8head` corpora and both pilots it recovered **612,747** starts
(`phase_book_v1-s10403.order.epd`, `F0CF3E07…`; all five sources agree; every
start is a book line). Before the first game the wrapper refuses a sequence
whose sidecar names another book or seed or another file, a segment past the
recovered sequence, and the legacy adjudication profiles. After the run it
reads the runner's records, not its console: completed, every game scored,
zero engine faults (`Assert-ColosseumRunFaults`, made safe under strict mode
in the same commit), the resolved openings range, order, wrap, node limit,
adjudication, cap and engine. It then checks the PGN's starts against the
sequence's lines from `-Start` to the end: one game per opening and none
outside. A segment whose games finished before the wrapper did is verified
without replaying, because Colosseum refuses to resume a completed run. The
manifest (`rarog-colosseum-datagen-v1`) keeps the label contract
`extract_parallel.py` checks and records the sequence and the runner.
`extract.py` reads Colosseum's score comments. Checked by 25 unit tests, the
fastchess command's identity, seven refused setups plus a tampered sequence,
and the pilot below.

**Equivalence pilot.** 3,000 games, C.8.1's engine (`E73E3230…`, bench
14,197,472 verified in the run), `-Start 1`, concurrency 30, through the new
path (`analysis/artifacts/c84/equiv/`), against the fastchess pilot
`selfplay-c8head-n8000-s1-g3000.pgn`:

- Opening set identical: 3,000 starts in both, none unshared.
- 2,994 games identical move for move; 6 differ, all at the end; results equal
  in all 3,000.
- The preflight (`--preflight-games 3000 --target-train 3500000`) recommends
  **595,240** games for both. Its counts differ only by those 6 games (24
  skipped against 30, 6 more raw and unique positions).

**The 6 games are a Colosseum rules defect.** In each, the start position
occurs a third time at ply 8: fastchess ends the game there (FIDE 9.2, the
initial position counts), and Colosseum plays one more ply and draws on the
next repetition. At `ca05dfa`, `crates/colosseum-engine/src/runner.rs` enters a
position in its repetition table after each book move (line 747) and each
engine move (line 1025), never the setup position, so from an EPD start the
start position's threefold needs a fourth occurrence. Every Colosseum game is
affected in principle. In RAR-E60's gate-1 and `10+0.1` games (8,232) no start
position recurs three times, because the UHO starts are unbalanced
middlegames; in the datagen pilot 6 of 3,000 do, all drawn either way
(`start_threefold.py`). The request to the maintainer is to count the setup
position. Once Colosseum is re-pinned with that fix, the pilot is re-run, and
step 3's move-for-move criterion is expected to hold in full.

## Colosseum 0.3.0 verified before release, 2026-10-14

Branch `rarog-cli-0.3.0` of the Colosseum repository: six commits on
`cli-v0.2.0`, HEAD `af712d9`, tree clean. Its exe (`colosseum-cli 0.3.0`,
`AC56265A…`) is current with that source: cargo rebuilt nothing. Its own
record, `docs/architecture/cli-0.3.0-record.md`, finds the main cost was
`tournament` starting two fresh engine processes for every game. Step 1 above
was wrong to assume the two harnesses both keep their engines. Checked here
independently (`analysis/artifacts/c84/v030/`):

- **Tests:** `cargo test -p colosseum-cli -p colosseum-engine -p colosseum-uci`
  passes, 410 tests, 0 failed. These include the four threefold tests and the
  two tournament engine-keeping tests.
- **Equivalence:** the 3,000-game pilot, with exactly `datagen.ps1`'s
  command, is identical to the fastchess pilot in 3,000 of 3,000 games
  (238,348 plies, 0 move or result mismatches). The 6 start-position
  threefolds now end at the third occurrence. The resolved-configuration
  fields `datagen.ps1` checks are present.
- **Speed** (`harness_speed.ps1`, identical games): 18.2 / 18.0 / 17.8 s
  against fastchess's 24.9 / 24.8 / 24.9 s. The median is 28% below
  fastchess's, against a target of at most 2% above. Plies are equal
  (71,523).
- **Gate wrapper:** `colosseum.ps1`, run on the 0.3.0 exe through a test pin,
  passes an SPRT dry run (pin, policy) and a 280-game `3+0.03` match: no
  faults, and the recount agrees.
- **Adoption item:** a tournament now keeps its engines per slot by default,
  and `tools/colosseum/gauntlet.toml` unsets the option. Under 0.3.0 a pool
  gauntlet would therefore switch silently to kept engines, a condition
  RAR-M64's pool ratings were not measured under. `colosseum.ps1` does not
  check the field. At the re-pin, `gauntlet.toml` sets
  `engine-processes = "per-game"` unless the maintainer decides otherwise.
  0.3.0 accepts the option, and 0.2.0 did not.
- The release asset will be a CI build with its own SHA-256. At adoption it
  gets the `--version`, pilot and speed checks again before the pin moves.
