# RAR-M88 — Harness speed at each runner's best setup: Colosseum CLI 0.5.0 against fastchess 1.8.3 and against weather-factory

Indexed under *2. Measurement, harness and tuning* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- Disposition: observation — a throughput read, not a strength read: matches the same speed in steady
  state; tunes Colosseum 1.22x weather-factory. No engine change.

## Experiment and conditions

**A throughput read, not a strength read: is Colosseum CLI faster than, the same as, or slower than fastchess for fixed matches and weather-factory for SPSA, each at its best setup? Maintainer question 2026-11-13, for the decision on whether to keep developing Colosseum CLI. Predictions frozen after a 30-game smoke and before the timed series; played 2026-11-13 19:40–20:29 UTC.** Runners: Colosseum CLI 0.5.0 (the pinned exe, `81EA6CBA…`); fastchess 1.8.3 from the GitHub release zip (zip `e4c66cce…` matching GitHub's digest, exe `A7AA6762…`); weather-factory from `tools/weather-factory` copied and driving fastchess 1.8.3. Eight runs alternating the runners (fastchess, Colosseum, fastchess, Colosseum, then weather-factory, Colosseum, weather-factory, Colosseum), each behind an idle-host wait: no engine, runner, compiler or Python process, and under 15% CPU over 5 s. Shared conditions: 15 slots each pinned to one logical CPU on 2, 4, …, 30; `3+0.03` with a 20 ms margin; Hash 64; one thread; no adjudication; engines kept between games.

- **Matches.** 500 games per run of `rarog-d68base-pext-pgo.exe` (`67889E9E…`) against itself, the UHO_Lichess_4852_v1 book in sequential order from its first opening.
  - fastchess: `-use-affinity` with an explicit CPU list (without one it uses CPU 0 too, and with Threads above 1 it turns affinity off), `-autosaveinterval 0`, no engine log, engines kept alive by default. Nothing else in 1.8.3 bears on throughput: `-events`, `-wait 0` and the ping timeouts were checked.
  - Colosseum: `match --engine-processes per-slot --placement 2,…,30`.
- **Tunes.** The b23core surface (82 coordinates) on `rarog-b23core-tune.exe` (`25467C63…`), 17 iterations × 30 games per run, the shape RAR-M62 measured.
  - Colosseum: `spsa` with `tools/spsa_configs/colosseum/b23core.run.toml`.
  - weather-factory: patched in its copy to pass `timemargin=20` (the shipped copy passes no margin) and to skip its per-iteration graph. The graph was timed on its own at 0.33–0.48 s an iteration and is added back for the tool as shipped.
- **Scoring.**
  - Games per hour from a stopwatch around the runner's whole process.
  - The engines' share of slot time: the sum of every move's recorded think time over 15 × wall.
  - A split of wall time into start-up, games and tail, taken from each runner's own per-game timestamps.
  - Tunes only: the gap between one iteration's last game and the next iteration's first.

Driver, scripts and outputs: `analysis/artifacts/harness-speed-2026-11-13/` (`run_speed.ps1`, `setup_speed.py`, `analyse_speed.py`, `decompose_speed.py`, `clock_use.py`, `depth_band.py`, `time_wf_graph.py`; `analysis.txt`, `decompose.txt`, `clock_use.txt`, `depth_band.txt`; `runs/`).

**Predictions, frozen before the timed series:**
- Matches, Colosseum over fastchess in games per hour: 1.02, in a band of 0.97 to 1.06.
- Tunes, Colosseum over weather-factory: 1.04, in a band of 1.00 to 1.10.
- The engines' share of slot time at least 85% in both matches.
- At most two time losses per run.

## Result / disposition

**Matches: the same speed in steady state. Tunes: Colosseum 1.22× weather-factory (about 1.24× against weather-factory as shipped).**

- **Validity.** All eight runs exited 0, and the host was idle before each (0.5–1.8% CPU, no listed process). Every planned game was played and every move carried its time. There were no time losses and no faults. The four matches played the identical 250 openings, each with both colours.
- **Matches.**

  | | fastchess 1.8.3 | Colosseum 0.5.0 | Colosseum / fastchess |
  |---|---|---|---|
  | Games per hour (per run) | 5,988 (6,038, 5,938) | 6,260 (6,214, 6,305) | 1.045 |
  | Engines' share of slot time | 95.8% | 97.1% | 1.014 |
  | Slot fill with process exit excluded | 98.4%, 98.3% | 98.2%, 98.4% | 1.00 |
  | From the last game's end to process exit | 3.9 s, 3.8 s | 0.2 s, 0.2 s | |

  fastchess's game timestamps are whole seconds, so its fill and tail carry about ±1 s. Its only measured deficit is the exit: 1.3% of a 500-game run, negligible in an SPRT.
- **The rest of the match gap is not harness speed.**
  - Colosseum completes each EPD opening with `0 1` move counters (`colosseum-engine/src/openings.rs`, `epd_fen_into`). fastchess keeps the counters the book carries, mostly `0 3` to `0 7`, some with a non-zero halfmove clock.
  - Rarog's time manager scales its optimum with game ply (`src/search/time.rs`, `(ply + 3.23)^0.469`). On the same position and clock it should therefore allot about 1.20× on move 1 at fullmove 4, and 1.25× at fullmove 5.
  - Measured on ply 1 under fastchess: 121.7 ms against 102.9 ms (1.18×), and depth 12.57 against 12.21.
  - The engines therefore spend about 88% of the clock per game under fastchess against about 86% under Colosseum. That is worth about 3 points of games per hour.
  - fastchess's own `tl=` exceeds a plain clock reconstruction by exactly 30 ms on every move. The cause was not traced: either one extra increment at the start or a different way of reporting the clock.
- **Tunes.**

  | | weather-factory (graph off) | Colosseum `spsa` | Colosseum / weather-factory |
  |---|---|---|---|
  | Games per hour (per run) | 3,986 (3,991, 3,982) | 4,864 (4,893, 4,836) | 1.220 |
  | Seconds per iteration | 27.06, 27.12 | 22.07, 22.33 | |
  | Gap between iterations | 4.56 s, 4.81 s | 0.00 s | |
  | Iteration span, first start to last end | 22.47 s, 22.29 s | 22.04 s, 22.31 s | |

  - The whole gap lies between iterations; the iteration spans agree within 1%.
  - weather-factory starts one fastchess process per iteration. Each iteration therefore pays fastchess's exit (as in the matches), the Python step, a new fastchess and 30 engine starts.
  - As shipped, the graph adds 0.33–0.48 s an iteration on top.
- **Calibration.**
  - **Matches: inside the band, wrong mechanism.** 1.045 is inside 0.97–1.06, but the harness-only part is 1.014, and all of it is fastchess's exit. The other 3 points came from the opening counters, which the prediction did not consider: a miss in mechanism.
  - **Tunes: a miss in magnitude.** 1.22 against 1.04. The failed assumption was a per-iteration cost of 0.5–1 s for weather-factory; fastchess alone takes about 4 s to exit after its last game.
  - **Engine share and time losses held.** The share was 95.8–97.1% against a floor of 85%, and there were no time losses against an allowance of two.

## Conditional lesson and retry trigger

- **Fixed matches and SPRTs.** fastchess 1.8.3 at its best setup is as fast as Colosseum CLI. Match throughput is not a reason to prefer either runner.
- **SPSA on one host.** Colosseum CLI is about 20% faster than weather-factory, because it has no process boundary between iterations. Within an iteration both are bounded by the same iteration barrier: about 22 s for 30 games on 15 slots.
- **Opening counters.** A throughput comparison between runners must hold the opening's move counters equal, or normalise for the engine's clock use. Comparisons of games per hour across runners that complete EPD counters differently are confounded.
- **Gates.** Earlier gates are unaffected: both arms of a gate get the same completed openings.
- **The counter difference.** Colosseum resets the book's counters, which fastchess and the book's other users do not. It is recorded for the Colosseum maintainer in `analysis/colosseum_requests_2026-11-13.md`.
- **Retry trigger.** Measure again when fastchess changes its shutdown, when Colosseum's `spsa` loses its iteration barrier, when either runner changes how it completes openings, or before retiring either harness.

## Source

PROCESS, *Harness*; RAR-M61 (the 2026-09 match parity read, 14 slots); RAR-M62 (the 15 × 30 tune shape); RAR-S75 (weather-factory's 14 × 32 throughput); `analysis/artifacts/harness-speed-2026-11-13/`.
