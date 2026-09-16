# UCI `info` line conformance review — findings for PLAN B.2.5 and D.3

**Status: implementer's review, 2026-09-16, recorded as the evidence behind
PLAN B.2.5 (items 1–5 and 7) and D.3's score-normalisation research card
(item 6).** Probed binary: `tools/test_engines/rarog-b24a-core-pext-pgo.exe`
(the B.2.4a head, `b2core` arm, bench 4,706,910 / EBF 2.391) at `df6308e`.
References read locally: Stockfish `D:/code/stockfish`, Reckless
`D:/code/Reckless`. Line references are to that head.

## What is already correct

Example line, single-PV:

    info depth 10 seldepth 19 score cp 32 nodes 25417 nps 1337736 hashfull 0 tbhits 0 time 19 pv d2d4 d7d5 ...

- Field order is Stockfish's exactly (`uci.cpp:654` `on_update_full`):
  `depth seldepth [multipv] score [bound] nodes nps hashfull tbhits time pv`,
  `pv` last. No unknown tokens.
- `hashfull` is present and live: on a 1 MB table it climbed 509 → 760 → 910
  → 993 → 1000 and saturated. `src/tt.rs:929 hashfull_of` samples 334
  clusters (about 1,002 entries, Stockfish's sample size) and counts occupied
  slots of the current age.
- `tbhits` is present, aggregated over the pool (`reported_tb_hits`,
  `src/search/mod.rs:1450`). No Syzygy tables on this host, so the wiring is
  verified but not a live count.
- Mate scores, `bestmove ... ponder ...`, and per-line nodes/nps/time in
  MultiPV match the references.
- Single-threaded, the last printed info line always matches the played
  move: an aborted iteration never installs a new bestmove
  (`src/search/mod.rs:755-789`). Measured 0/24 mismatches at Threads 1.

## Defects, ranked

1. **SMP: the final info line often contradicts `bestmove`.** The pool votes
   on the result (`src/search/threads.rs:432 select_parallel_result`), but
   only the main thread emits info and nothing is printed after the vote, so
   the winning helper's line is never reported. Measured: 4 of 24 searches at
   Threads 8 (movetime 100/300/1000 over 8 positions); 0 of 24 at Threads 1.
   Example: last line `... pv b1c3 d7d5 ...` then `bestmove e2e4`.
   Stockfish's fix, end of `start_searching`: `if (!uciPvSent || bestThread
   != this) main_manager()->output_pv(*bestThread, ...)`. The move played is
   correct; only the report is stale. Affects GUIs, analysis logs and any
   PGN-annotating harness.
2. **A mated or stalemated root prints no info line at all.**
   `src/search/mod.rs:583 no_legal_moves_result` returns straight out; the
   UCI layer prints only `bestmove 0000` (`src/engine.rs:416`; the null move
   renders "0000" at `src/board/moves.rs:226`). Stockfish: `info depth 0
   score mate 0` then `bestmove (none)` (`search.cpp:207-211`, `uci.cpp:650
   on_update_no_moves`). Reckless: `info depth 0 score mate 0` in check,
   `info depth 0 score cp 0` otherwise (`src/thread.rs:377
   print_uci_no_move`).
3. **`nps` is 1000x too low whenever a line is printed inside the first
   millisecond.** `(nodes as u128 * 1000).checked_div(elapsed_ms).unwrap_or(
   nodes as u128)` at `src/search/mod.rs:1487` (single-PV) and its twin in
   `send_multipv_info` (`src/search/mod.rs:1273`): on a zero divisor it falls
   back to `nodes` instead of clamping elapsed to 1 ms as Stockfish does
   (`time = std::max(TimePoint(1), ...)`, `search.cpp:2343`). Measured,
   reproducible: `info depth 1 seldepth 1 score cp 143 nodes 49 nps 49 ...
   time 0`. Happens on the first iterations of essentially every move at STC.
4. **No bound reporting in single-PV mode, and no output inside an
   iteration.** Aspiration fail-high and fail-low `continue` silently
   (`src/search/mod.rs:745-753`); the strings " lowerbound"/" upperbound"
   exist only at `src/search/mod.rs:1281-1282`, for the interrupted line in
   MultiPV mode. Rarog emits exactly one line per completed iteration, so a
   long iteration sends the GUI nothing. Stockfish emits a mid-iteration PV
   and `currmove`/`currmovenumber` past 10M nodes (`NODES_LIMIT_OUTPUT`,
   `search.cpp:71`, used at `:415`, `:495`, `:1147`). Reckless reports bounds
   per root move (`src/thread.rs:321-357`) and prints on STOPPED as well
   (`src/search.rs:222-230`).
5. **`multipv 1` omitted in single-PV mode.** Legal per spec, but both
   references always emit it, so Rarog presents two line shapes to parsers.
   `send_info_line` has no multipv token (`src/search/mod.rs:1495`).
6. **Score scale is raw internal units.** `format_score`
   (`src/search/mod.rs:325`) prints `cp <raw>`; startpos depth 1 reports
   `cp 143`. Stockfish normalises (`UCIEngine::to_cp`, `uci.cpp:585`),
   Reckless has `normalize_to_cp`. Also: `TB_WIN_SCORE = MATE_SCORE −
   2·MAX_PLY = 31744` (`src/search/mod.rs:72`) is below `format_score`'s mate
   cutoff `MATE_SCORE − MAX_PLY = 31872`, so a Syzygy win prints as
   `cp 31744`. Stockfish prints `cp 20000 − plies` (`uci.cpp:566`), Reckless
   about ±19999. Ordering is preserved in all three; only the magnitude is
   Rarog's alone. Normalisation needs a fitted win-rate model: its own piece
   of research, not a mechanical fix.
7. **`seldepth` conventions.** Rarog reports the 0-based maximum ply
   (`src/search/node.rs:244`, `:1556`; `src/search/core/node.rs:357`,
   `:1752`), one less than Stockfish's `ss->ply + 1`; Reckless matches Rarog.
   Rarog resets once per search (`src/search/mod.rs:532`) while both
   references reset per iteration (Reckless `src/search.rs:87`), so shallow
   depths inherit a deeper earlier maximum. Cosmetic, but it makes seldepth
   non-comparable with Stockfish logs.

Absent but optional, and skipped by Reckless too: `currmove`/`currmovenumber`,
`wdl` (`UCI_ShowWDL`), `cpuload`, `currline`, `refutation`. No Chess960
support anywhere in `src/`, so no `UCI_Chess960` and no castling-format
question. Advertised options: `src/search_options.rs:162-171`.

## Verification the fixes require

- Items 1–5 and 7 are output-only and must be behaviour-neutral: reproduce
  4,706,910 / EBF 2.391 on the `b2core` arm and 7,601,220 / EBF 2.474 on
  the off arm. The bench prints no info lines, so an identical bench does not
  by itself prove the change neutral; targeted protocol tests are required.
  There is an existing format assertion at `src/search/mod.rs:1587` and an
  `InfoSink` recorder harness at `:1567` to build on.
- Engine changes need debug and release tests, `cargo fmt --check`, and
  `cargo clippy --all-features --all-targets` at zero warnings.
- Item 6 changes displayed evaluations only, but needs its own research
  card (win-rate model fit) before implementation.
- Nothing here is a playing-strength change, so no SPRT.
