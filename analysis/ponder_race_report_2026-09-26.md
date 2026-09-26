# Ponder race report, 2026-09-26 (PLAN B.3.5)

Received from the maintainer on 2026-09-26, from another machine, verbatim
below. Filed as evidence for PLAN B.3.5, to be reproduced and fixed once
cluster 2 (B.3) closes. The two flag sites the report names exist in `dev`
today: `prepare_search` in `src/engine_command.rs` clears `ponderhit` before
the search starts, and the `go` handler in `src/engine.rs` skips a `go` whose
`prepare_search` fails without sending a `bestmove`.

```text
PROBLEM: Rarog 2.5.0-dev never answers a `go ponder` if `ponderhit` or `stop` arrives
right after it.

Setup: UCI options Threads=4 (also reproduces with Threads=1), Hash=256, Ponder=true.

Bug 1: ponderhit is lost.
If the GUI sends `ponderhit` immediately after `go ponder`, Rarog never sends a
bestmove. The search either keeps pondering with no time limit, or, once it hits its
depth cap, waits for a ponderhit it has already dropped. A GUI then forfeits it on time.
This happens whenever the opponent replies within about 1 ms (instant recaptures, book
moves, tablebase moves, moves already found in the opponent's hash). If ponderhit
arrives 100 ms after go ponder, it works.
Likely mechanism (from the 2.4.0 source on master; not checked against dev): the engine
thread clears the ponderhit flag in prepare_search (src/engine_command.rs) just before
the search starts. A ponderhit the UCI thread recorded before that point is erased.

Bug 2: early stop drops the whole go.
If the GUI sends `stop` immediately after `go ponder` (a ponder miss where the opponent
replied instantly), Rarog prints no info lines and never sends a bestmove. Further
`stop` or `ponderhit` commands get no response; `isready` still gets `readyok`. UCI
requires exactly one bestmove for every go, so the GUI waits and forfeits the engine.
A stop sent 100 ms later works.
Likely mechanism (2.4.0 source): in src/engine.rs the go handler does
`if !self.control.prepare_search(command.epoch) { continue; }` and skips the go without
sending any bestmove.

This was seen in real games. In a 120+1 Colosseum tournament (4 threads, ponder on,
Syzygy 3-4-5), Basilisk 1.10.0 lost twice on time from exactly this sequence, once in a
position where it had mate in 3. Basilisk had the same ponderhit bug and has since been
fixed. Tested with the Rarog binary: Rarog has bug 1 and also bug 2, which Basilisk
never had.

REPRODUCTION: send each block to a fresh Rarog process. Write the last two lines
back to back with no delay between them (for example in a single write to stdin).

Scenario A: won middlegame (Black has mate in 3, e.g. Qg4 Kf2 Qg1+ Kf3 Qf1#)
  uci
  setoption name Threads value 4
  setoption name Hash value 256
  setoption name Ponder value true
  isready
  position fen 6k1/8/4p1P1/2p2p1P/P2p1q2/Q7/1r2n1K1/6R1 b - - 4 55
  go ponder wtime 13537 btime 6905 winc 1000 binc 1000
  ponderhit            (bug 1; or `stop` for bug 2)
Expected: bestmove within about 1-2 s (after ponderhit), or right away (after stop).
Actual: no bestmove at all. After ponderhit Rarog keeps searching (info lines keep
coming). After stop it prints nothing.

Scenario B: tablebase endgame (Black has a lone king and is lost, KBBP vs K)
  (same setup lines)
  position fen 5B2/P7/2k5/8/8/3B2K1/8/8 b - - 0 82
  go ponder wtime 6493 btime 7249 winc 1000 binc 1000
  ponderhit            (or stop)
Expected and actual: same as A. With SyzygyPath set it still reproduces.

Scenario C: ordinary opening (so the bug doesn't depend on the position)
  (same setup lines; also reproduces with Threads=1)
  position startpos moves e2e4 e7e5 g1f3 b8c6 f1b5 a7a6
  go ponder wtime 10000 btime 10000 winc 1000 binc 1000
  ponderhit            (or stop)
Expected and actual: same as A.

Control case: in any scenario, wait about 100 ms before sending ponderhit or stop, and
Rarog answers correctly (for ponderhit, within its normal time budget).
```
