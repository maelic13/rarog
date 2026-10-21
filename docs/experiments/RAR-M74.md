# RAR-M74 — C.11 pool gauntlet of the C head at 4T, RAR-M66's field and conditions

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-21, before any game. The maintainer's overnight run, the fourth step of `analysis/artifacts/c11-checkpoint/run_all.ps1`, after RAR-M73. PLAN's measurement table lists the pool score at 1T and 4T for C.11; this is the 4T half.
- **Kind:** an observation on a held scale, never a gate.
- **Arm:** RAR-M73's, `tools/test_engines/rarog-c11head-pext-pgo.exe` (`9750D3DE…`, `Rarog 2.6.0-dev+c11head`, **13,987,223 / EBF 2.565**, rustc 1.99.1; recipe `analysis/arm_patches/a0146a3-rar-m73-c11head.patch`).
- **Field and conditions: RAR-M66's, unchanged** except the head: `tools/colosseum.ps1 -Mode gauntlet -Seed 43 -Hash 512 -Threads 4 -Concurrency 3 -MarginMs 2000` through `analysis/artifacts/c11-checkpoint/gauntlet_4t.ps1` (RAR-M66's driver with the head, its label and the run directory `tools/results/c11-gauntlet-4t` changed); the RAR-M64 field at RAR-M57's held ratings (Houdini 3 3287, Critter 1.6a 3192, Fritz 16 3173, Rybka 4.1 3111, Basilisk 1.10.0 2993, Rarog 2.4.0 3001); the head's prior 3287 as in RAR-M66; four threads each under each engine's own option, Hash 512, `3+0.03`, margin 2,000 ms, three games at a time with `--cores-per-game 4`, UHO random, no adjudication, 400 games per pair, **2,400 games**, about 2 h 10 min. Dry run 2026-10-21 23:12, exit 0, at policy: the three slots own CPUs 2–9, 16–23 and 24–31 (four physical cores each, enforced), as RAR-M66's; `dry-run.json` SHA-256 **`A5C30C88CBDCA64F807A78C03AA5463EFAF21D60E0BF3EA3B4C1846E89E534B8`**, which the run's manifest must repeat. The runner is Colosseum 0.3.0 (RAR-M66 ran on an earlier version; RAR-M70's cross-check).
- **Reading:** per-opponent W-D-L, score and Elo with interval (the primary reading, since the held ratings are 1T ratings and the head's rating here is a performance on that scale); the head's performance against RAR-M66's **3,340.0 ± 16.7**; faults and time losses per engine; E.2 at 4T; in-game NPS and depth; the 4T-minus-1T shift per pair against RAR-M73.
- **Fault rule (RAR-M66's):** an opponent's faults (Houdini 3 crashed at launch 3 times in RAR-M66) are read when they are under 1% of the games, each scored as the harness scored it and reported, with the pair also given without them; any fault of the C head voids the run. `colosseum.ps1`'s zero-tolerance clause does not separate the two and exits 1 after a completed run; the driver then continues and the faults are read here against this rule.
- **PRE-REGISTERED PREDICTION (frozen 2026-10-21, before any game):**
  - *Performance:* **3,440**, 80% band **[3,385, 3,495]**: RAR-M73's predicted 3,385 plus RAR-M66's 1T-to-4T shift for the B.9 head (+53.5).
  - *Houdini 3:* **+210**, band [+150, +270] (RAR-M66 +113.3). *Rarog 2.4.0:* about **+420**, band [+350, +490] (RAR-M66 +322.7).
  - *E.2 at 4T:* all four targets pass, probability 0.97.
  - Most likely failure mode: as RAR-M73's, plus the C head's SMP behaviour, which Phase C did not touch and Phase D owns.
- **Stop rule:** one run of 2,400 games, no early stop, no extension; a run voided by a fault of the C head is repeated, not read.
- **Use:** GUIDE's checkpoint row "Pool position, `3+0.03` 4T" takes this reading beside RAR-M66's; E.2's 4T targets are re-read from it. It changes no default and accepts nothing.
- **Result:** not played.
- **Artifacts / commits:** this registration; `analysis/artifacts/c11-checkpoint/` (driver, dry run).
