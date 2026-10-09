# Basilisk review of 2026-10-06: four suspicions checked in Rarog

Basilisk shares Rarog's origins, so its review's findings were checked here
as suspicions, measured on the frozen head (engine source `ee02ed1`, the
binary `tools/test_engines/rarog-b9head-pext-pgo.exe`, `aac92114…`) on
2026-10-06. Probes and raw outputs are in
`analysis/artifacts/basilisk-review-2026-10-06/` (ignored storage).

| Item | Verdict | Severity | Owner |
|---|---|---|---|
| 1. Hard time maximum late in a game | The maximum is the donor's and not a defect by itself; the iterations that run into it are: a stalled re-search (rec1), no stop after a found mate (rec2), and a clock-independent stop not yet explained (rec3) | Medium | D.1.1, pulled forward (maintainer, 2026-10-06) |
| 2. Known-win evaluations against the decisive band | No defect | None | — (one stale guard fixed in `0827ab9`) |
| 3. Repetition after a stored result (maelic13/manta#4) | No defect | None | — (regression test added in `0827ab9`) |
| 4. `bench` nps at 0 ms | Defect, cosmetic | Low | Fixed in `0827ab9` |

## 1. Hard time maximum late in a game

**The formula.** `src/search/time.rs` is Stockfish's sudden-death branch:
`max_scale` sits at its 6.873 cap at every ply, so the maximum is 6.87 times
the optimum, and the optimum grows with ply. The transcription
(`tm_table.py`, after `tools/results/b52-20261001/scripts/tm_limits.py`)
reproduces the engine: at rec1 (ply 170, `60000+600`) optimum 4,700 ms,
maximum 32,304 ms, and Basilisk spent 32,305 ms.

| Clock | Ply 0 | Ply 100 | Ply 170 | Ply 200 | First ply above half the clock |
|---|---|---|---|---|---|
| `60000+600` | 11,250 ms (18.8%) | 26,939 (44.9%) | 32,304 (53.8%) | 34,240 (57.1%) | 138 |
| `180000+2000` | 35,671 (19.8%) | 86,883 (48.3%) | 104,396 (58.0%) | 110,715 (61.5%) | 112 |

**In real games it rarely binds.** Over the head's side of RAR-E19's 2,000
games at `3+0.03` (the clock rebuilt per move from each move's reported
time), a move used 90% of its maximum or more in 0.31% of moves at plies
0–39, 0.41% at 40–79, 0.61% at 80–119, 0.06% at 120–159 and never later;
the largest single-move shares (about 75% of the clock) all came with about
400 ms left, where the `0.8097 × clock` term binds, not the ply term. The
games had no time fault.

**Rarog against Stockfish 19** (`tm_rec_probe.py`, no tablebases, Threads 1,
Hash 64, a fresh process per search; run by the maintainer on an idle host,
2.5% CPU):

| Clock | Position | Rarog | Stockfish 19 | Rarog's last line | Stockfish's last line |
|---|---|---|---|---|---|
| `60000+600` | rec1, ply 170 | **32,305 ms** (maximum 32,304) | 9,405 | depth 32 `cp 1302 lowerbound` | depth 31 `cp 9165` |
| | rec2, ply 160 | **31,620** (maximum 31,620) | 1,434 | depth 56 `mate 5` | depth 245 `mate 5` |
| | rec3, ply 142 | 10,441 (optimum 4,413) | 5,679 | depth 21 `cp 951` | depth 36 `cp 698` |
| `180000+2000` | rec1 | **104,397** (maximum 104,396) | 43,788 | depth 32 `cp 1302 lowerbound` | depth 32 `cp 9165` |
| | rec2 | **102,162** (maximum 102,162) | 2,711 | depth 56 `mate 5` | depth 245 `mate 5` |
| | rec3 | 10,424 (optimum 14,251) | 19,231 | depth 21 `cp 951` | depth 41 `cp 753` |

rec1 `7r/5R2/8/2k1PB2/8/4K3/8/8 w - - 0 86`, rec2
`8/P4k2/8/1N6/1P2B1K1/8/8/8 w - - 7 81`, rec3
`1r6/R7/6k1/8/8/5PP1/6K1/8 w - - 6 72` (B.5.2's record).

**Reading.** Stockfish has the same maximum and stays well below it, so the
maximum is not the defect. Three behaviours of Rarog's iterations are:

1. **rec1, a stalled re-search:** the aspiration cascade B.5.2 recorded,
   now seen at both clocks; the move ends only at the hard maximum, at the
   same `cp 1302 lowerbound` and depth 32 whatever the clock.
2. **rec2, no stop after a found mate:** `mate 5` is on the board, yet
   Rarog searches to the maximum at both clocks while its iterations stall
   near depth 56; Stockfish reaches depth 245 and stops in 1.4 and 2.7 s.
   Whether this is the same cascade or a missing stop rule is open.
3. **rec3, a clock-independent stop:** 10.4 s and depth 21 at both clocks,
   above the optimum at one and below it at the other, so the clock did not
   end that search. Not explained.

The cost is up to 52–58% of the remaining clock on one move in a won
ending. Frequency in games is the open question that sizes the gate.

## 2. Known-win evaluations against the decisive band

The decisive band starts at `TB_WIN_SCORE` = 31,744 (`MATE_SCORE` 32,000
less `2 × MAX_PLY`); a tablebase root win is `TB_VALUE` = 31,871. The mop-up
adds at most `MOPUP_MAX` = 2,800, and the largest material an evaluation can
see is about 15,000. Every static evaluation the search uses passes through
`corrected_eval_parts`, which clamps it to ±31,743 (`src/search/correction.rs`,
with a unit test): the main search and the quiescence stand pat both
convert the raw evaluation there before any use, and the raw value only
fills the TT's raw-evaluation slot (16-bit, saturating). The review's KBNK
position, `go depth 12` without `SyzygyPath`, printed `cp 329` to `cp 1056`
over 53 lines and ended `cp 1056`, `bestmove c5d5`; Basilisk printed
`cp 22048`. No evaluation can enter the band, so no consumer (pruning
guards, correction admission, the PV extension, the display) can mistake one
for a tablebase result.

The mop-up's `const` assertion guarded only the mate band (31,872); a second
assertion against the decisive band was added in `0827ab9`.

## 3. Repetition after a stored result

Every non-root node scores a repeated position as a draw
(`can_declare_draw_in_search`: twofold against the game history, rule 50,
insufficient material) before its TT probe and cutoff
(`src/search/node.rs`, the draw check near line 502 against the probe near
565 and the cutoff near 615); quiescence does the same (near 2127, before
its probe). The root never takes a TT cutoff: the cutoff requires a non-PV
node and the root node type is PV.

Measured (`repetition_probe.py`, one process, MultiPV over every root move,
depth 14): KQ v K with `d1d2` scoring a winning `cp 14552` from the fresh
root; after the game shuffles back (`d1d2 h8g8 d2d1 g8h8`), `d1d2` from the
repeated root scores `cp 0` while the best line stays `mate 5`. The
regression test `a_stored_win_never_answers_a_position_repeating_the_game_history`
(`tests/draw_semantics.rs`, `0827ab9`) stores a deep win for the position
after `d1d2` and requires `d1d2` from the repeated root to score 0; with
both repetition checks disabled in a throwaway worktree it fails, reading
31,973.

## 4. `bench` nps at 0 ms

The `info` lines floor the elapsed time at 1 ms (`45779ee`), but `bench`
computed its per-position and per-run nps itself and fell back to the node
count at 0 ms: `bench 7/40 … nodes 57 … time 0ms nps 57` on the current
engine. Both lines now floor at 1 ms (`0827ab9`; the same position prints
`nps 57000`). `Nodes/second`, `tools/nps_read.py` and the other tools read
run totals or never divide by a zero time; `xtask` prints no nps.

## Decisions (maintainer, 2026-10-06)

- Items 2–4 are taken care of in `0827ab9`: behaviour-neutral, `bench 13`
  11,171,726 / EBF 2.512, debug 375 and release 376 tests.
- Item 1 becomes D.1.1, pulled forward ahead of Phase C's first gates: the
  mechanism of each behaviour above, its frequency in games, then a frozen
  fix and its gate; Phase C continues on the resulting head.
