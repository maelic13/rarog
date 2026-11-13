# Adopting Basilisk's SPSA method: one run read at checkpoints (2026-11-13)

**Status:** adopted 2026-11-13 by maintainer decision, before any tune under
it. PLAN rule 7c and PROCESS *SPSA go/no-go procedure* hold the rule; this note
records what was ported, how it was checked, and what Rarog's own recorded
tunes say under it. Zero games.

## What changed

Rule 7c ran a tune as a chain of 2,000-iteration blocks, each restarted from
the last block's rounded centres, stopped when fewer than three coordinates
moved a full step, at most three blocks (2026-09-25, from RAR-S75 and RAR-S78;
amended 2026-09-26 and 2026-10-20, RAR-S104). Basilisk's B.5.6 (its research
packet `analysis/b56_spsa_method_2026-11-12.md`) replaced that rule there, and
its B.5.8 built the tooling. The maintainer signed off both in Basilisk and
asked for the same system here. The rule now reads:

- one run to a registered ceiling (6,000 iterations unless argued), at twice
  the step where the seed has the room and the gain ratio `r_end` 0.00155;
- checkpoints at iteration 2,000 and every 1,000 after, each reading the last
  1,000 iterations by a per-coordinate gradient test on the tune's own games,
  `z = Σ δ·D / √(Σ D²)`, whose null is exact because the tuner drew the signs;
- quiet (no |z| past the familywise 5% line and `Σ z² − n` under
  `2·√(2n)`) stops the tune with theta the checkpoint's rounded centres; live
  runs the next 1,000; a centre at a bound pushed outward past the line stops
  for a new range; faults stop for the maintainer;
- the fit's gate is chosen by the tune's own gain read before any gate game: a
  fixed 6,000 games passing when its 95% interval excludes 0 where the read is
  +10 Elo or more, otherwise the `[0,3]` SPRT.

## The port

- `tools/spsa_read.py`: Basilisk's reader, unchanged but for its schema names.
  It regenerates the runner's perturbation signs from the run's master seed
  (ChaCha12 as Colosseum derives it) and checks every recorded centre against
  the runner's update before it reads, so a misread journal or seed fails.
  Both repositories pin the same runner, Colosseum CLI 0.5.0
  (`81EA6CBA…`).
- `tools/diag/test_spsa_read.py`: Basilisk's synthetic tests unchanged (the
  runner's golden vectors, the statistics, every verdict on simulated runs, the
  record, faults); its recorded-block tests replaced by Rarog's ten finished
  tunes below, which run where `tools/results` holds them.
- `tools/spsa_config_to_colosseum.py --rule7c --ceiling N [--out DIR]`: the
  graded perturbation (two steps where the seed has the room, otherwise one to
  two, `c_end` rounded down so no first arm plays clamped), the ceiling in the
  header and `r-end = 0.00155` in the run file. The block form `--iterations`
  stays only so the historical groups' committed files still check; all twelve
  do (`tools/diag/test_spsa_config_to_colosseum.py`).
- `tools/colosseum.ps1 -Mode spsa -Ceiling N`: every guard it had (idle host,
  pinned runner, sidecar provenance and fingerprint, the tune build advertising
  the surface, the resolved policy, placement), now playing a segment at a time
  with `--stop-after-iteration` and applying the read at each checkpoint. The
  rule 7c files it reads must reproduce from the JSON where they lie.
  `-Iterations`, `-TotalGames` and `-SeedFrom` are gone; `-AllowShortRun` lifts
  the production shape for an instrument smoke only. Basilisk runs the loop in
  a separate `spsa_colosseum.ps1`; here it stays inside the one wrapper so the
  policy checks are not duplicated.
- `tools/spsa_theta_audit.py --at K`: the bake's audit reads the checkpoint
  where a tune stopped, through the reader's loader (a stopped tune has no
  `result.json`); a full step stays the registered step.
- Retired: `tools/spsa_block_rule.py` and its test.
- `tools/diag/test_colosseum_guards.ps1`: 70/70 on 2026-11-13. The `-SeedFrom`
  cases went with the blocks; new cases refuse a tune without a ceiling, off
  the production shape, on 14 slots, on files converted for another ceiling, on
  the block-form files, and `-Ceiling` in a gate; a 6,000-iteration tune and a
  short smoke shape resolve.

**Not yet run:** a live smoke of the checkpoint loop (a short tune with
`-AllowShortRun` through a real pause, read and resume). The host was busy with
another session's match when the port finished, so it waits for an idle host.

## Rarog's recorded tunes under the new reader

Every recorded move of all ten finished Colosseum tunes reproduces from the
regenerated signs (worst error 5e-13 to 2e-12), so the reader's sign derivation
and journal parsing hold for Rarog's runs. Read at each 1,000 with the
production checkpoints (window 1,000, first 2,000); the full output is
`analysis/artifacts/spsa-adoption-reads.txt` (ignored), the 2,000 reads are
pinned in the tests.

| Run | Ledger | n | At 2,000 (max \|z\| / line; excess SD) | Later |
|---|---|---:|---|---|
| `b27all-spsa` | RAR-S75 | 82 | STOP_QUIET 2.57 / 3.42; +1.4 | quiet to 5,000; its gain read falls to −10.3 |
| `b33m-spsa-1` | | 36 | STOP_QUIET 2.22 / 3.19; −1.2 | |
| `b33m-spsa-2` | | 36 | STOP_QUIET 2.02 / 3.19; +0.3 | |
| `b33r-spsa-1` | | 36 | FAULT 3.70 / 3.19; +0.8 | the run recorded faults; `CoreSingularMargin` live at 1,000 (z 5.90) |
| `b43-spsa-1` | | 14 | STOP_QUIET 2.79 / 2.91; +1.4 | |
| `c10-long` | RAR-S104 | 82 | CONTINUE 3.50 / 3.42 (`QsDeltaMargin`); +1.9 | quiet at 3,000: stops 1,000 iterations before its 4,000 |
| `d14-spsa-block1` | RAR-R15 | 13 | STOP_QUIET 2.28 / 2.88; +0.4 | live at 1,000 (`TmOptScale` z 5.34) |
| `rar-e24-block` | RAR-E24 | 28 | STOP_QUIET 2.69 / 3.12; +0.6 | live at 1,000 |
| `rar-e25-block` | RAR-E25 | 28 | STOP_CEILING 2.90 / 3.12; +2.2 | live by the excess at its 2,000 ceiling |
| `rar-r21-fit` | RAR-R21 | 15 | STOP_QUIET 2.49 / 2.93; +0.8 | |

**The one tension, and why the shape changed.** Rule 7c's blocks rested on
RAR-S75 and RAR-S78: restarting RAR-S75's 82 coordinates from its endpoint
gained +13.1 ± 5.4. The gradient test reads RAR-S75 quiet from 2,000 on. Its
own floor says why the two do not contradict: in that run's shape (one step,
`r_end` 0.0031) at 82 coordinates, a quiet read at 2,000 rules out no more
than a forecast of about 11 Elo over the next 2,000 iterations (two SD; the
forecast itself +7.8 ± 5.6). A quiet read in the old shape was too coarse to
see what the restart found. That is the reason the rule doubles the
perturbation and halves the ratio: each gradient's signal doubles at the same
move per result, and Basilisk's packet puts the floor at 2,000 under 5 Elo for
every surface size in its record (4.5 at 49 coordinates, against 14.9 for a
restarted block). Rarog's largest surface, 82 coordinates, lies beyond that
record. **Watch item:** every read states its floor, so the first tune
under the rule shows what a quiet stop rules out at Rarog's surface sizes. A
floor well above Basilisk's 5 Elo would be evidence to revisit the shape by a
new registration, never by extending a stopped tune.

## Sources

Basilisk B.5.6 and B.5.8 (its PLAN, PROCESS *SPSA go/no-go*, and
`analysis/b56_spsa_method_2026-11-12.md`, both errata); Rarog PLAN rule 7c as
it read before 2026-11-13 (`docs/PLAN.md` history), RAR-S75, RAR-S78, RAR-S104.
