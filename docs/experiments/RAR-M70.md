# RAR-M70 — runner checks after the Colosseum 0.3.0 re-pin: the backup-path cross-check and the null pair

Indexed under *2. Measurement, harness and tuning* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-14, before any game. The runs are the maintainer's; the reads are the agent's.
- **Why:** PROCESS's *Harness* triggers fired on 2026-10-14. The runner changed (`colosseum-cli 0.2.0` → `0.3.0`, `c5f11b1`), and so did the Colosseum version. Fixed-node games are already shown identical to fastchess's (PLAN C.8.4). Timed games are not: 0.3.0 writes engine input from a dedicated thread, which changes when a command reaches an engine. These checks judge the instrument before the next timed gate (RAR-E61's gate 1). They decide nothing about any candidate.
- **Binary / runner identity:**
  - Colosseum `colosseum-cli 0.3.0` (`381EB7A5…`, `cb12cd3`).
  - fastchess alpha 1.8.0 (`8444E739…`), through `tools/sprt.ps1`.
  - The arms are RAR-E60's gate arms: `rarog-c8cycle1-pext-pgo.exe` (`76CD1F3C…`, bench 14,291,377) and `rarog-c8head-pext-pgo.exe` (`E73E3230…`, bench 14,197,472).
- **Conditions shared by both runners:** `3+0.03`, one thread, Hash 64, the UHO book in random order, no adjudication.
- **(1) The cross-check** (RAR-M61's method): a fixed 2,000-game match on the same arms under each runner.
  - Colosseum: `./tools/colosseum.ps1 -Mode match -EngineA tools/test_engines/rarog-c8cycle1-pext-pgo.exe -EngineB tools/test_engines/rarog-c8head-pext-pgo.exe -NameA c8cycle1 -NameB c8head -Games 2000 -Seed 7001 -ExpectBench 14291377,14197472 -Dir tools/results/rar-m70-xcheck-colosseum`. The dry run passed 2026-10-14 (`colosseum_match_rar-m70-xcheck-colosseum_20261014_215255.dry-run.json`): idle host, the pinned runner, both fingerprints, policy in every checked field.
  - fastchess: `./tools/sprt.ps1 -Mode fixed -EngineA tools/test_engines/rarog-c8cycle1-pext-pgo.exe -EngineB tools/test_engines/rarog-c8head-pext-pgo.exe -NameA c8cycle1 -NameB c8head -Games 2000 -Seed 7001`. The wrapper has no dry run; its guards refuse a busy host, a provenance mismatch and an unpinned runner before the first game.
  - **Rule (frozen):** with each Elo estimate's standard error taken as its 95% half-width divided by 1.96, the runners **agree** when the difference of the estimates is within 1.96 × √(se_C² + se_f²). If they agree, gates proceed on 0.3.0. If they disagree, no gate runs until the maintainer decides. The reads are not pooled with each other or with any gate.
- **(2) The null pair** (RAR-M03): `./tools/colosseum.ps1 -Mode calibrate -EngineA tools/test_engines/rarog-c8cycle1-pext-pgo.exe -EngineB tools/test_engines/rarog-c8cycle1-pext-pgo.exe -NameA a -NameB b -Seed 7002 -ExpectBench 14291377,14291377 -Dir tools/results/rar-m70-null`. The run file's 30,000 games. The dry run passed 2026-10-14 (`colosseum_calibrate_rar-m70-null_20261014_215258.dry-run.json`).
  - **Rule (frozen):** the runner's own calibration verdict. A pass lets gates proceed. A fail, or an inconclusive result, stops gates until the maintainer decides.
- **PRE-REGISTERED PREDICTION (freeze before exposure):**
  - Cross-check: both runners read the arms near RAR-E60's gate, each about **+8 Elo** with a 95% half-width near ±13. RAR-E60's +11.0 is a first-crossing estimate and overstates. The runners agree by the rule (0.9).
  - Null pair: pass (0.9).
- **Cost:** the two cross-check matches take about 20 minutes each, and the null pair about 5 to 5½ hours, at RAR-E60's gate rate of about 5,700 games an hour.
- **Result:** pending.
