# RAR-S136 — D.7.13: the gate, the fitted, repaired kernel against the classic kernel by SPRT `[0,3]` at `3+0.03`, with 2,000 games at `10+0.1` and 2,000 at four threads beside it — READ 2026-11-21: H1 at 520 pairs, +67.0 Elo at `10+0.1` and +73.3 at four threads

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-21, before any game. All three runs
  are the maintainer's command.
- **Kind:** PLAN D.7.13, the one strength gate of the D.7 cluster
  (maintainer decisions 2026-11-16 and 2026-11-19: the new kernel must
  gain). Its uses are PLAN's, copied below.
- **What it compares:** everything D.7 built against what it replaces.
  - `d713kern`: the compact kernel; D.7.8's repairs and
    speed pass; Reckless's time manager (D.7.9.1); the first fit baked
    (RAR-S128) and the finishing fit baked (RAR-S135, D.7.11.8); the score
    printed on its fitted scale (D.7.12, output only).
  - `d7classic`: the classic kernel with its hybrid clock, archived by
    D.7.7.

## Arms (frozen)

Both are pext PGO builds by `tools/build_test.ps1` from clean trees,
rustc 1.100.0:

| Arm | File | Commit | `id name` | Bench | SHA-256 |
| --- | --- | --- | --- | --- | --- |
| `d713kern` | `rarog-d713kern-pext-pgo.exe` | `eb84cce` | `dev-20261121-eb84cce0` | 3,390,842 / EBF 2.317 | `e2ad12685bc8f5c42254827d95fbffdf485180f83a73ffcfd6f443406d48ac99` |
| `d7classic` | `rarog-d7classic-pext-pgo.exe` | `29f32fb` | `dev-20261119-29f32fbb` | 9,067,595 / EBF 2.479 | `781797e99f40d5b118032c9d110e1cb2d291817015b068ecf5130994893926f4` |

`d713kern` was built on 2026-11-21 from `eb84cce` with the tree clean
(sidecar `git_dirty: false`). Its `bench 13` was read again outside the
build script, 3,390,842 / EBF 2.317, the fingerprint D.7.11.8 recorded.
`d7classic` benched 9,067,595 / EBF 2.479 the same day. `d713kern` lists
`UCI_ShowWDL` and no tuning option.

## The gate (frozen)

- **Runner:** Colosseum CLI 0.5.0, the pinned build, through
  `tools/colosseum.ps1 -Mode sprt -Bracket default`.
- **Test:** `sprt-default.toml`: `[0,3]` nElo, pentanomial,
  `model=normalized`, α = β = 0.05, LLR bounds ±2.944.
- **Conditions:** `3+0.03`, margin 20 ms, Hash 64, one thread, 14 slots,
  the UHO book in random order, no adjudication.
- **Cap: 20,000 pairs** (40,000 games), the cap of the earlier `[0,3]`
  gates. By RAR-M10's drift (8.3e-6 × 3 × (nElo − 1.5) a game), it
  resolves a true gain of about +4.5 nElo or more. An unresolved stop at
  the cap is read as no H1.
- **Games at the expected value** (RAR-M10, extrapolated far outside the
  ±6 nElo it was fitted on, so a rough guide only):
  - at D.7.10's reading (+109 nElo), about 550 pairs to H1;
  - at that reading plus the finishing fit's +13 (RAR-S135), about 520;
  - roughly 15 minutes at about 5,000 games an hour. The cap would be
    about 8 hours.
- **Seed** 2026113601, `tools/results/rar-s136-gate`.
- Bounds, cap, book and adjudication never change after a game is seen.

## The reads beside it (frozen; they decide nothing alone)

Colosseum `match` through the same wrapper, same arms:

1. **`10+0.1`:** 2,000 games (`match-fixed-ltc.toml`), seed 2026113602,
   `tools/results/rar-s136-ltc`; about 80 minutes. It is also the clock's
   check (PLAN D.7.11, item 1: the clock's coordinates were not fitted at
   `3+0.03`): each arm's time losses and the clock profile
   (`analysis/artifacts/d79-clock-2026-11-20/clock_screen.py`) are reported.
2. **Four threads:** 2,000 games at `3+0.03`, `Threads 4`, 3 slots of 4
   cores on the explicit placement list
   `2,4,6,8,16,18,20,22,24,26,28,30` (one logical CPU per physical core,
   RAR-M82), seed 2026113603, `tools/results/rar-s136-4t`; about 2 hours.
   Its validity check is each side's time losses and that each slot ran on
   its listed cores.

The seeds occur in no earlier registration, driver or run directory
(checked 2026-11-21).

## Validity (frozen)

- Every game ends in a rules result.
- The wrapper refuses a run whose faults or time losses pass its ceiling
  (0.5% of the games); such a run is void and is reported, not read.
- Each arm's time losses are reported for every run.

## Uses (frozen; PLAN D.7.13)

- **H1:** opens D.7.14, adoption by the maintainer's decision.
- **H0, or the cap without H1:** opens D.7.15. The new kernel is discarded
  and the classic kernel restored, each later step by its own
  registration.
- The two reads beside it are reported with their intervals. They decide
  nothing alone. The `10+0.1` read is also the clock's check.

## The command (dry-run 2026-11-21)

```
pwsh -NoProfile -File D:\code\rarog\analysis\artifacts\d713-gate-2026-11-21\run_s136.ps1
```

`run_s136.ps1` (ignored, in `analysis/artifacts/d713-gate-2026-11-21/`),
SHA-256 `83fcb6df015ab06489a02b719893d4e53bd5cee780e276304d69856061e50f6a`.
It:
- checks both binaries' SHA-256 first;
- runs the gate, then the `10+0.1` read, then the four-thread read;
- retries a start once a minute, only while the harness refuses a busy
  host before any game;
- skips a finished run, and resumes an interrupted one when run again;
- logs each manifest's exit code, time losses and faults.

Its `-DryRun` path ran all three through the wrapper's dry run on
2026-11-21 with every guard passed (manifests
`colosseum_sprt_rar-s136-gate_20261121_132619`,
`colosseum_match_rar-s136-ltc_20261121_132623` and
`colosseum_match_rar-s136-4t_20261121_132626`). The resolved gate reads
`[0,3]`, `normalized`, α = β = 0.05, cap 20,000 pairs. The four-thread
read has the explicit list at 4 cores a game. No run directory was
created.

**Cost:** about 15 minutes for the gate at the expected value (8 hours
at its cap), 80 minutes for `10+0.1` and 2 hours for four threads: about
3 hours 35 minutes in all on an idle host.

## PRE-REGISTERED PREDICTION (frozen before any game)

**Basis.**
- RAR-S134 (D.7.10), the same comparison before the finishing fit: +63.8 ±
  9.1 Elo (nElo +109.2) at `3+0.03`, +65.0 ± 8.3 at `10+0.1`.
- RAR-S135's read at the stop: the finishing fit +7.8 ± 9.1 Elo over its
  start, measured on the tune build.
- D.7.12 changes output only.
- No earlier read of the new kernel against the classic at four threads
  exists. RAR-R34 played its time manager 200 games at four threads
  without a time loss.

1. **The gate:** H1, 0.97; H0, 0.01; the cap without H1, 0.02. Its Elo
   estimate at the stop: +70, 80% band +50 to +95 (an early stop
   overstates an effect this size). H1 within 300 to 900 pairs: 0.75.
2. **`10+0.1`:** +68 Elo, 80% band +50 to +85.
3. **Four threads:** +55 Elo, 80% band +25 to +85. This one is wide: the
   kernel's narrow windows and the time manager have no four-thread read
   against the classic.
4. **Faults:** no time loss by `d713kern` in any of the three runs, 0.75.

- Most likely outcome: H1 in well under a thousand pairs, and gains of the
  same size at `10+0.1` and four threads.
- The failure that would surprise: a four-thread read far below the
  one-thread gain. That would point at the shared table or the clock under
  threads, which no D.7 read has measured.

## Results (read 2026-11-21)

All three runs completed, every game a rules result, no fault and no time
loss by either arm (each manifest's `run_status`, `pentanomial`, `faults`
and `time_losses` lines; the pentanomial recount of `games.pgn` agrees).
The host read 4.1%, 1.7% and 1.2% busy at the three starts; no waiver.

| Run | Manifest | Games | Elo | nElo | Pentanomial |
| --- | --- | --- | --- | --- | --- |
| Gate, `3+0.03` | `colosseum_sprt_rar-s136-gate_20261121_135534` | 1,040 | +79.9 ± 12.3 | +142.2 ± 21.1 | 1, 52, 217, 211, 39 |
| `10+0.1` | `colosseum_match_rar-s136-ltc_20261121_140723` | 2,000 | **+67.0 ± 8.1** | +128.3 ± 15.2 | 6, 98, 437, 427, 32 |
| Four threads | `colosseum_match_rar-s136-4t_20261121_152211` | 2,000 | **+73.3 ± 8.1** | +141.6 ± 15.2 | 2, 95, 426, 439, 38 |

- **The gate: H1 accepted** at 520 pairs, LLR 2.947 against 2.944
  ("the gain is at least 3.00 normalized Elo"). Its Elo is read at a
  boundary stop and overstates the effect; the two fixed reads do not.

Search lines (each `results.txt`), `d713kern` against `d7classic`:
- `3+0.03`: depth 16.05 against 14.40 mean; 124,327 against 120,472
  nodes a move; 1.83M against 1.82M nodes a second.
- `10+0.1`: depth 19.74 against 18.13; 422,684 against 426,899 nodes a
  move; 1.91M against 1.97M nodes a second.
- Four threads: depth 17.60 against 15.43; 543,122 against 512,472 nodes
  a move; 8.26M against 8.00M nodes a second.

Four-thread placement: the manifest and the resolved configuration hold
the explicit list, 4 CPUs a game, and the journal's three slots played
669, 663 and 668 games. Each side's four-thread rate is 4.51 (`d713kern`)
and 4.39 (`d7classic`) times its one-thread rate in the gate, against
2.65 when Windows packs a game's threads on SMT siblings and 4.03 on one
logical CPU a core (RAR-M82), so no slot was packed. The rest of the
ratio is likely three games sharing the memory instead of fourteen.

Clock profile at `10+0.1` (`clock_screen.py`, output `clock_ltc.log` in
`analysis/artifacts/d713-gate-2026-11-21/`):
- Spend of the budget, pooled: 0.920 against 0.901. The first twenty
  moves take 0.737 of the base against 0.678.
- The kernel's clock goes lower: its floor's median is 332 ms against
  504, and 45.4% of its sides fall under 200 ms at some move against
  34.4%; neither arm's sides fall under 100 ms. Of its undecided moves,
  8.8% are played under 500 ms against 3.7%.
- The process's hold, the charged time less its CPU time, is the same for
  both: the per-game maximum's p99 is 24.0 ms against 23.3. The
  manifest's `harness_hold_ms`, larger for the kernel (p999 1,005 ms
  against 341), is the charged time less the time in the last `info`
  line, so it counts the interrupted iteration the kernel's driver
  discards (PROCESS, *Harness*); it is not a hold.

### Uses

1. **H1: D.7.14 opens**, adoption by the maintainer's decision. D.7.15
   does not open.
2. **The reads beside it:** +67.0 ± 8.1 Elo at `10+0.1` and +73.3 ± 8.1
   at four threads, both wholly above zero. They decide nothing alone.
3. **The clock's check** (PLAN D.7.11, item 1's retry trigger): the
   `10+0.1` read is 12.9 Elo below the gate's estimate, inside their
   combined intervals (±20.4), and above RAR-S134's fixed `3+0.03` read
   (+63.8 ± 9.1), so the trigger's first clause does not fire. Its second
   clause, a factor at one end on more than 90% of iterations, needs the
   time manager's factors, which the game records do not carry; it is not
   read here. The clock stays with H.3. Its lower clock at `10+0.1` lost
   no game on time.

### Calibration

- Gate: H1 (0.97) at 520 pairs, inside 300 to 900 pairs (0.75); +79.9
  at the stop against +70, inside the band of +50 to +95.
- `10+0.1`: +67.0 against +68, a hit.
- Four threads: +73.3 against +55 (band +25 to +85), inside the band but
  a miss in magnitude of +18. The assumption that failed: that the
  kernel's narrow windows and the time manager, with no four-thread read
  against the classic, would keep less of the one-thread gain under
  threads. The gain holds whole at four threads, and the surprising
  failure did not happen.
- No time loss by `d713kern` (0.75): none in any run.
- The `10+0.1` read played its registered 2,000 games; when the maintainer
  asked during it whether 1,000 would do, the registered length was kept
  (2026-11-21).

Evidence layer: the SPRT on the gain bracket at `3+0.03` is the one
acceptance; the fixed reads are Elo at `10+0.1` and at four threads; the
clock profile is not Elo.
