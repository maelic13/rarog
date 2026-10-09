# RAR-M14 — Time-forfeit floor at concurrency 14, Threads 1, `3+0.03`, measured from RAR-E06's 3,915-game PGN

Indexed under *2. Measurement, harness and tuning* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**Time-forfeit floor at concurrency 14, Threads 1, `3+0.03`, measured from RAR-E06's 3,915-game PGN.** Every game ends having spent **97-99% of its whole clock** (base 3s plus 0.03s per move); the five longest games, 367-494 plies, sit at 97.5-98.7% and do not forfeit. The three that did forfeit were 90/98/121 plies -- **shorter** than the 131-ply median -- and one of them flagged while its own reported move times summed to only **94.5%** of budget.

## Result / disposition

**Observation.** The forfeits are not clock mismanagement and not long-game exhaustion. They are the gap between engine-reported thinking time and harness-measured wall time, against an aggregate slack of about 2% of a ~4.9s budget -- roughly 100ms for a whole game. A single descheduling event of that size, with all 14 physical cores running engines and fastchess contending for the same silicon, is a forfeit. Rate 0.077%, consistent with 0.135% and 0.172% in two identical-binary null pairs.

## Conditional lesson and retry trigger

`Move Overhead` defaults to 10ms and `time_manager.rs` reserves `2*overhead` only below ~520ms of clock; the `smp_reserve` of 30ms is gated on `threads > 1`, so a single-threaded engine under a saturated runner gets no equivalent protection. Its comment records `0/3,460 at Threads=1`, which no longer holds at this concurrency. Retry trigger: measure forfeit rate against `Move Overhead` on a null pair before changing any default -- a TM change alters playing behavior and needs its own gate. Diagnosis owner **4.2b**; every repair owner **4.12a**.

## Source

`tools/results/sprt_HCERefit_vs_HCEBase_20260901_072106.pgn`; `src/time_manager.rs`
