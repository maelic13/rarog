# RAR-S134 — D.7.10: the pre-fit read, the repaired kernel with Reckless's time manager against the classic kernel, 2,000 games at `3+0.03` and 2,000 at `10+0.1` — READ 2026-11-20: +63.8 and +65.0 Elo, the fit at `3+0.03`

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-20, before any game. Both runs are
  the maintainer's commands.
- **Kind:** PLAN D.7.10, a diagnostic and not a gate (D.7.13 gates the
  cluster). Its three uses are frozen in PLAN and copied below.
- **What it compares:** the whole of what D.7 has built so far against
  what it replaces:
  - the compact kernel, with its first fit baked (RAR-S128), D.7.8's
    repairs and speed pass, and D.7.9.1's time manager (RAR-R34);
  - against the classic kernel, with the hybrid clock it had.

  Until now this difference was only a chain of separate reads.

## Arms and runs (frozen)

Both are pext PGO builds by `tools/build_test.ps1` from clean trees,
rustc 1.100.0:

| Arm | File | Commit | `id name` | Bench | SHA-256 |
| --- | --- | --- | --- | --- | --- |
| `d710kern` | `rarog-d791cand-pext-pgo.exe` | `38e947c` | `dev-20261120-38e947c6` | 3,320,355 / EBF 2.317 | `ff55f198bf9a8a668a898ed3f6cd08ade220c9d697aaf5ed58f493316fcfec66` |
| `d7classic` | `rarog-d7classic-pext-pgo.exe` | `29f32fb` | `dev-20261119-29f32fbb` | 9,067,595 / EBF 2.479 | `781797e99f40d5b118032c9d110e1cb2d291817015b068ecf5130994893926f4` |

- `d710kern` is RAR-R34's candidate, reused rather than rebuilt. Since
  `38e947c` the engine source has changed by comment lines only
  (`e26ace6`, checked mechanically: zero non-comment lines).
- `d7classic` is the baseline archived by D.7.7 (PLAN, its SHA-256 there).

Colosseum `match` through `tools/colosseum.ps1`: Hash 64, one thread, 14
slots, the UHO book in random order, no adjudication, a 20 ms margin.

1. **`3+0.03`:** 2,000 games, seed 20261129, `tools/results/rar-s134`.
   Dry run `colosseum_match_rar-s134_20261120_135625.dry-run.json`.
2. **`10+0.1`:** 2,000 games (`match-fixed-ltc.toml`), seed 20261130,
   `tools/results/rar-s134-ltc`. Dry run
   `colosseum_match_rar-s134-ltc_20261120_135629.dry-run.json`.

**Cost:** about 25 and 80 minutes on an idle host, about 1 hour 45
minutes in all.

**One command runs both:**
`analysis/artifacts/d710-prefit-2026-11-20/run_s134.ps1` (SHA-256
`2bde9600fd94b25c…`).
- It retries a start only while the harness refuses a busy host before
  any game.
- Run again, it resumes an interrupted read from its checkpoint and skips
  a completed one.
- Its `-DryRun` path ran through both reads on 2026-11-20.
- It is run from the machine's own terminal.

## Readings and dispositions (frozen; PLAN D.7.10)

- **Validity:** every game ends in a rules result. A run with time losses
  or engine faults in more than 0.5% of its games is void. Each arm's time
  losses are reported.
- **Readings:** each manifest's Elo and nElo with their 95% intervals,
  pentanomial and search lines, with the clock profile
  (`analysis/artifacts/d79-clock-2026-11-20/clock_screen.py`) beside them.
- **Use 1:** the first direct measurement of the new kernel against the
  old. It is reported as read and decides nothing by itself.
- **Use 2, the fit's time control:**
  - If the `10+0.1` Elo is at least half the `3+0.03` Elo, by point
    estimates, D.7.11 runs at `3+0.03` with rule 7c's shape unchanged.
  - If it is less, D.7.11 runs at `10+0.1` with a ceiling of 3,000
    iterations, its registration giving the hours.
  - The ratio of the two Elo point estimates decides; their intervals
    are reported beside it.
- **Use 3:** a `3+0.03` interval wholly below zero stops D.7 for the
  maintainer before any fit.
- Nothing changes after a game.

## PRE-REGISTERED PREDICTION (frozen before any game)

**Basis.** The chain of reads that this match replaces:
- RAR-S126: the unfitted kernel against the classic, about −6 Elo;
- RAR-S128: the first fit, +56.8 over the donor's seeds;
- D.7.8's kept repairs: about −1 Elo together;
- D.7.8.12's speed pass: +2.71% NPS, about +5 Elo at PROCESS's 2 Elo
  per 1%;
- RAR-R34: the time manager, +30.0 at `3+0.03` and +18.6 at `10+0.1`.

That sums to about +85 at `3+0.03`. Chained self-play reads usually
overstate the direct difference, so the centre is shrunk.

1. **`3+0.03`:** +65 Elo, 80% band +40 to +90. P(interval wholly below
   zero) 0.01.
2. **`10+0.1`:** +45 Elo, 80% band +15 to +75. Lower, because the first
   fit was read at `3+0.03` only and the time manager gained less at
   `10+0.1`.
3. **Use 2:** the `10+0.1` Elo at least half the `3+0.03` Elo, so the fit
   runs at `3+0.03`: 0.70.
4. **Faults:** no time loss by either arm in either run, 0.85.

- Most likely outcome: a large gain at both controls, smaller at
  `10+0.1`, and the fit at `3+0.03`.
- The failure that would surprise: a `10+0.1` Elo under half the
  `3+0.03` one. That would say the first fit, made at `3+0.03`, does not
  carry to a longer control.

## Results (read 2026-11-20)

Both runs completed, 2,000 scored games each, every game a rules result,
no fault, no time loss by either arm in either run (each manifest's
`run_status`, `pentanomial` and `time_losses` lines; the pentanomial
recount of `games.pgn` agrees).

| Control | Manifest | Elo | nElo | Pentanomial |
| --- | --- | --- | --- | --- |
| `3+0.03` | `colosseum_match_rar-s134_20261120_135946` | **+63.8 ± 9.1** | +109.2 ± 15.2 | 15, 107, 444, 368, 66 |
| `10+0.1` | `colosseum_match_rar-s134-ltc_20261120_142106` | **+65.0 ± 8.3** | +121.8 ± 15.2 | 4, 115, 423, 423, 35 |

Search lines (each `results.txt`), `d710kern` against `d7classic`:
- `3+0.03`: depth 16.31 against 14.43 mean; 132,467 against 127,649
  nodes a move; 1.96M against 1.94M nodes a second.
- `10+0.1`: depth 20.15 against 18.16 mean; 440,122 against 442,661
  nodes a move; 1.97M against 2.03M nodes a second.

Clock profile (`clock_screen.py`, outputs `clock_s134_stc.txt` and
`clock_s134_ltc.txt` in `analysis/artifacts/d710-prefit-2026-11-20/`):
- Spend of the budget, pooled: 0.896 against 0.872 at `3+0.03`, 0.913
  against 0.894 at `10+0.1`. The first twenty moves take 0.762 and 0.753
  of the base against 0.721 and 0.676.
- At `3+0.03` the kernel's clock goes lower: 31.3% of its sides fall
  under 100 ms at some move, against none of the classic's, the floor's
  median 258 ms against 344. No time was lost. At `10+0.1` neither arm's
  sides fall under 100 ms.
- The harness's hold stays small: the per-game maximum's p99 is 5.5 ms
  at `3+0.03` and 13.7 ms at `10+0.1`.

### Uses

1. **The new kernel against the old:** +63.8 ± 9.1 Elo at `3+0.03` and
   +65.0 ± 8.3 at `10+0.1`, the kernel and the time manager together.
   Not a gate: D.7.13 gates the cluster.
2. **The fit's control:** the `10+0.1` Elo is 1.02 times the `3+0.03`
   Elo, above one half, so D.7.11 runs at `3+0.03` with rule 7c's shape
   unchanged.
3. The `3+0.03` interval is wholly above zero, so D.7 goes on.

### Calibration

- `3+0.03`, predicted +65 [+40, +90]: +63.8, a hit.
- `10+0.1`, predicted +45 [+15, +75]: +65.0, inside the band but a miss
  in magnitude of +20. The assumption that failed: that the first fit,
  read at `3+0.03` only, and the time manager, which gained less at
  `10+0.1` in RAR-R34, would leave less at the longer control. The
  kernel's gain holds there whole.
- Use 2 at 0.70 and no time loss at 0.85: both happened.
- Sizing, a miss in the instrument's scope: the `10+0.1` read took 2,000
  games because RAR-R34's did, not from its use. Use 2 reads point
  estimates against a threshold of half the `3+0.03` Elo; at the
  predicted values about 1,000 games (±11.6 Elo) would have mis-set the
  control about 2% of the time, and by 1,260 games the running estimate
  stood six standard errors above the threshold. A read's game count is sized
  from its decision rule at registration.

Evidence layer: fixed reads inside D.7's cluster, Elo at each control;
nothing here accepts strength.
