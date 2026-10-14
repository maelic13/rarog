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
