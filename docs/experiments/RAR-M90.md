# RAR-M90 — D.7.14: the pre-NNUE meter read again, the adopted kernel against RAR-M89's field, a 7,800-game Colosseum gauntlet at `3+0.03` — REGISTERED 2026-11-21, before any game

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-21, before any game; the run is the
  maintainer's command.
- **Kind:** a measurement at a checkpoint (PLAN rule 10; PLAN D.7.14: "the
  pre-NNUE meter read again"). It accepts and rejects nothing.
- **Why now:** the maintainer adopted the new kernel on 2026-11-21 after
  RAR-S136's H1. The meter (PLAN section 1) is Rarog's gap to Stockfish 16
  with `Use NNUE=false`, read head to head and in a rated field; its last
  reading, RAR-M89, is of 2.7.0, the classic kernel.

## What plays

- **The seed:** `d713kern`, the binary RAR-S136 gated:
  `tools/test_engines/rarog-d713kern-pext-pgo.exe`, `eb84cce`, `id name`
  `dev-20261121-eb84cce0`, `bench 13` 3,390,842 / EBF 2.317, SHA-256
  `e2ad12685bc8f5c42254827d95fbffdf485180f83a73ffcfd6f443406d48ac99`, at
  its defaults.
- **The field:** RAR-M89's thirteen engines, the same files (each SHA-256
  checked against RAR-M89's prefix by the driver, all thirteen matched on
  2026-11-21), each with the options RAR-M89 recorded (move overhead 10 ms
  where an engine has it; Stockfish 16 `Use NNUE=false`; Dragon 3.3
  `Use Regular Eval=true`, `OwnBook=false`; Houdini 6.03 `NUMA
  Enabled=false`; Koivisto `OwnBook=false`), and each **pinned at its
  RAR-M89 rating** (`--fixed`):

  | Engine | Pin |
  |---|---:|
  | Stockfish 16 HCE | 3,604.7 |
  | Berserk 4.7.0 HCE | 3,543.6 |
  | Stockfish 11 | 3,542.4 |
  | Ethereal 14.40 HCE | 3,466.9 |
  | Houdini 6.03 | 3,461.7 |
  | Rarog 2.7.0 | 3,444.6 |
  | Dragon 3.3 HCE | 3,423.4 |
  | Basilisk B.11.1 head | 3,405.7 |
  | Ethereal 13.07 | 3,400.8 |
  | Fire 8.3 | 3,384.6 |
  | Rarog 2.5.0 | 3,301.5 |
  | rofChade 2.3 | 3,281.4 |
  | Koivisto 4.83 | 3,196.8 |

  Booot 6.5 stays out, as it did of RAR-M89's fit.

## Conditions (frozen)

- Colosseum CLI 0.5.0, the pinned build, through `tools/colosseum.ps1
  -Mode gauntlet` (`gauntlet.toml`): one seed, 300 cycles of 2 games,
  **600 games a pair in colour-reversed pairs, 7,800 games**.
- `3+0.03`, one thread, Hash 128, 14 games at a time, harness margin
  2,000 ms (the desktop's, which RAR-M89 played under), no ponder, no
  adjudication, no tablebases. UHO `UHO_Lichess_4852_v1.epd` in random
  order, seed 2026112190 (in no earlier registration, driver or run
  directory), `tools/results/rar-m90-meter`.
- Stop rule: one fixed run to its end, no early stop, no extension. A run
  voided by the wrapper's fault ceiling (78 faults, 1%) is repeated, not
  read.
- About 1 hour 40 minutes at RAR-M89's 4,824 games an hour.

## What is read (frozen)

1. **The meter.** The kernel's rating as Colosseum fits it with the field
   pinned, with its interval; the gap is 3,604.7 less that rating. Head to
   head, Colosseum's Elo against Stockfish 16 HCE with its 95% interval.
   Every pair's W-D-L and Elo is reported.
2. **The gain over the release in the same field:** the kernel's rating
   less 2.7.0's 3,444.6, and Colosseum's head-to-head Elo against 2.7.0.
3. **Conversion:** `tools/diag/conversion_audit.py --engine "Rarog
   d713kern"` on the run's `games.pgn`, draws and losses a piece up per
   1,000 games. The same tool on BAS-M18's PGN reproduces RAR-M89's 2.7.0
   row exactly (15.4 draws and 3.2 losses, 120 and 25 of 7,800;
   `analysis/artifacts/d714-meter-2026-11-21/conversion_m89_270.log`), so
   the two compare like for like. A stronger engine reaches more piece-up
   positions, so a rate a game is not a rate a chance.
4. **Speed in games:** Colosseum's search line for the kernel (nodes a
   second, depth, time a move), beside RAR-S136's.

Validity: every game a rules result; each engine's time losses and faults
reported; an engine losing more than 1% of its games on time is listed and
the kernel's rating given with and without those games, as in RAR-M89.

Uses: (1) and (2) are GUIDE's *Pre-NNUE goal* row and the 2.8.0 release
line's strength sentence; (3) is the *Conversion* row; (4) the *Speed*
row. None decides anything.

## The command (dry-run 2026-11-21)

```
pwsh -NoProfile -File D:\code\rarog\analysis\artifacts\d714-meter-2026-11-21\run_m90.ps1
```

`run_m90.ps1` (ignored, in `analysis/artifacts/d714-meter-2026-11-21/`),
SHA-256 `9119071e0cde4b3e024257d6e1c678ed7245397a41f69a19d71c8bb90f8e507c`,
checks every binary's hash, then runs the gauntlet, retrying a start once a
minute only while the harness refuses a busy host before any game; Ctrl+C
stops it at a committed boundary and running it again resumes it. Its dry
run passed every policy check on 2026-11-21 (manifest
`colosseum_gauntlet_rar-m90-meter_20261121_175432`): 7,800 games, thirteen
pairs of 600, the thirteen pins and every option above in the resolved
plan.

## PRE-REGISTERED PREDICTION (frozen before any game)

**Basis.** RAR-S136: the kernel +67.0 ± 8.1 Elo at `10+0.1` and +73.3 ±
8.1 at four threads over `d7classic`, which is 2.7.0 with the null move no
longer counted (RAR-S125, a neutral change); RAR-S134 +63.8 ± 9.1 at
`3+0.03` before the finishing fit, which added +7.8 ± 9.1. A gain over a
sibling usually shrinks against foreign engines.

1. **Rating:** 3,505, 80% band 3,480 to 3,530; so the gap to Stockfish
   16 HCE about 100 (75 to 125).
2. **Head to head against Stockfish 16 HCE:** −105 Elo, 80% band −135 to
   −75 (2.7.0: −164.8).
3. **Against 2.7.0 head to head:** +65, 80% band +45 to +85; the field
   gain (rating less 3,444.6) +60, 80% band +35 to +85.
4. **Place:** above Ethereal 14.40 HCE and Houdini 6.03, below Berserk and
   Stockfish 11, probability 0.6.
5. **Conversion:** fewer than 2.7.0's 18.6 thrown a thousand, probability
   0.6 (RAR-S126 saw won endings converted better by the kernel).
6. **Faults:** no time loss by the kernel, 0.9.
