# Time forfeits at `3+0.03` — diagnosis and repair (PLAN A.3.3, RAR-R11)

## The defect

Rarog forfeits on time about once per thousand games at `3+0.03`, 1T, fourteen
concurrent games, on every recent SPRT and on both arms: RAR-E16's STC run
(game 114 of 743, the candidate), RAR-E15 (1 of 1,951), RAR-E09's runs (4 of
7,389 and 1 of 1,501). RAR-M14 had measured the floor at 0.08–0.17% and found
that games end having spent 97–99% of their clock by the engine's own
accounting.

## The seven forfeited games, reconstructed

For each forfeited game the loser's clock was rebuilt from the PGN move
comments (engine-reported time per move, base 3 s, increment 30 ms), and the
engine's own hard ceiling for the stalled move computed from
`time_manager.rs`: `min(0.8097 × time − 10, time − 20)` ms.

| Game | Remaining before the stalled move | Engine maximum | Engine-reported last time | Reading |
|---|---:|---:|---:|---|
| RAR-E16 g130 | 181 ms | 137 ms | 21 ms | inside budget; silent for > 180 ms |
| RAR-E15 g145 | 225 | 172 | 253 | reported time past the ceiling by 80 ms |
| RAR-E09 g25 | 91 | 64 | 163 | past by 99 |
| RAR-E09 g367 | 148 | 110 | 183 | past by 73 |
| RAR-E09 g4555 | 55 | 35 | 85 | past by 50 |
| RAR-E09 g5578 | 212 | 162 | 323 | past by 161 |
| RAR-E09b g117 | 527 | 417 | 43 | inside budget; silent for > 500 ms |

The reported time is stamped when an `info` line is printed at the end of a
completed iteration. A completed iteration stamped 80–160 ms past the hard
ceiling means the search ran past the ceiling without its every-2048-node
clock check firing, which cannot happen while the search is executing. It can
happen only if the process was **stalled** (descheduled, or blocked in I/O)
and a small TT-driven iteration then completed inside one check window. The
two "silent" cases are the same stall with nothing left to print. The seven
are one population of 50–500 ms stalls under a saturated host.

## What was and was not reproduced

`tools/results/time-forfeit-20260909/tm_probe.py` and `tm_load.py` drive the
fixed-feature PEXT build at bullet clocks over UHO positions and compare wall
time (`go` to `bestmove`) with the engine's reported time and its own maximum.

| Condition | Searches | Worst wall overrun past the maximum | Moves past clock + 20 ms |
|---|---:|---:|---:|
| Idle host, 1 process, clocks 60–400 ms | 1,000 | 1.2 ms | 0 |
| Idle host, 14 processes in parallel | 5,600 | 2.0 ms | 0 |

CPU contention alone does not reproduce the forfeit on this host: the hard
stop is precise to a millisecond. A run with the driver sleeping 20 ms per
line read made every move overrun, but that measured the **driver's own
per-line lag** (it read the thirteen `info` lines sequentially before
reaching `bestmove`), not engine blocking; it is recorded here as the
instrument artifact it was, and it is why the first repair below was reverted.

## What the donors do

- Stockfish: `limits.startTime = now()` in `uci.cpp` while parsing `go`,
  annotated "the search starts as early as possible"; every depth printed
  with a blocking `sync_cout`; `Move Overhead` default 10 ms.
- Reckless: `TimeManager::new` (its `start_time`) at `go` parse on the UCI
  thread; every completed depth printed with `println!`; the hard bound
  checked every 2,048 nodes, as in Rarog.
- Rarog before this leaf: the clock started in `reset_search_state` on the
  **engine thread**, after the command was queued from the UCI thread and the
  engine thread woke up, and after configuration invalidation. Any latency
  in that hand-off under a loaded host was invisible to the budget and came
  straight off the harness's 20 ms margin.

Neither donor throttles output. Rarog's per-depth output volume was not a
difference from them.

## The repair

1. `d93f808` throttled `info` lines to one per 250 ms. **Reverted at
   `e3430d9`**: it made Rarog behave unlike every other engine on the basis
   of an artifact, and the donors show the volume is not the difference.
2. **`79d3974`: the clock starts when `go` is parsed.** `SearchLimits.issued`
   is stamped on the UCI thread in `set_search_parameters` and reset per
   `go`; `reset_search_state` uses it as the clock origin, falling back to
   `Instant::now()` for tests and bench. Wake-up and setup latency now count
   against the engine's own budget, as the harness counts them. Test
   `search_clock_starts_when_go_was_parsed`: a `movetime 200` search issued
   300 ms ago returns at once with a legal move from a completed iteration and
   reports elapsed from the parse instant.

What this closes and what it does not: the part of a stall that happens
between the harness writing `go` and the search starting is now inside the
budget. A stall that lands mid-search or after `bestmove` is written is not
fixable by any engine; the donors are exposed to it equally, and the
operational answer is the harness's own margin (`Move Overhead`, fastchess
`timemargin`), whose sweep RAR-M14 already sized. The low-time reserve
(`2 × MoveOverhead` at 1T, plus 30 ms above one thread) was deliberately not
changed: a 30–40 ms reserve rescues at most one of the seven reconstructed
stalls at a cost on every last move, and stays with D.1.

## Verification

`cargo fmt --check`, `cargo clippy --all-features --all-targets -D warnings`,
`cargo test` debug and release (567 tests) clean on `79d3974`. Fresh
no-feature builds: PEXT native and magic both **7,601,220 / EBF 2.474**;
bench does not use a `go` clock, so the change is bench-invisible by
construction. The registered RAR-R11 run decides the forfeit rate.

## Evidence

`tools/results/time-forfeit-20260909/` (ignored): probe scripts and logs,
verification logs, bench outputs of both builds. The seven forfeited games
are in the SPRT PGNs named above under `tools/results/`.
