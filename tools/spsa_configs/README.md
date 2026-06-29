# SPSA tuning with weather-factory + fastchess

fastchess does **not** have a built-in SPSA tuner. The community-standard tuner
is **weather-factory** (https://github.com/jnlt3/weather-factory), a small
Python driver that perturbs UCI options and runs mini-matches via fastchess.
This folder holds ready-made weather-factory config files for Rarog.

## One-time setup

Run the repo-local setup helper if the tool folders are missing:

```powershell
./tools/setup_tools.ps1
```

This keeps helper tools inside the Rarog repo:

| Tool | Repo-local path |
|---|---|
| fastchess | `tools\bin\fastchess.exe` |
| weather-factory | `tools\weather-factory\` |
| opening book | `tools\books\SuperGM_4mvs.pgn` |
| test engines | `tools\test_engines\` |

`tools\setup_spsa.ps1` populates `tools\weather-factory\tuner\` for each run.

## Per-run setup

Run `setup_spsa.ps1` for the group you are tuning. It writes the three config
files into the weather-factory root (next to `main.py`):
   - `cutechess.json`             (runner settings — same for every group)
   - `spsa.json`                  (SPSA hyper-params; `A = iterations / 10`)
   - `config_<group>.json` → rename to `config.json` (the parameter set)

Example:

```powershell
./tools/setup_spsa.ps1 -ConfigGroup lmr -EngineSuffix p25-lmr -Iterations 5000
```

## Run

```powershell
cd tools\weather-factory
python main.py        # progress + tuned values written to its own state files
```

weather-factory writes the running parameter values to its state file every
`save_rate` games; stop it any time with Ctrl-C.

## CRITICAL: SPSA finds candidates, SPRT decides

SPSA optimizes a noisy objective and **over-fits**. The tuned values are only a
*candidate*. Always finish by:

1. Baking the tuned values in as the new UCI-option defaults (or passing them
   explicitly), then building a fresh `pext --pgo` binary with `tools\build_test.ps1`.
2. Running `tools\sprt.ps1` (default `tc=3+0.03` — the **same** TC this SPSA
   uses, so the optimum transfers) of the tuned binary vs the pre-tuning head.
   **Keep the tuned values only if SPRT accepts H1.** For a phase-boundary or a
   TC-suspect feature, also confirm at LTC (`-TC "10+0.1"`).

## Settings rationale

| Setting | Value | Why |
|---|---|---|
| Runner | fastchess (`use_fastchess: true`) | less overhead than cutechess-cli |
| `tc` | `3` → 3+0.03 s | Clock + 1% increment (Stockfish convention), ~depth 16. **Unified with `sprt.ps1`** so the SPSA optimum transfers to the confirming SPRT with no condition gap (2026-06-17 change — the old `tc=1` SPSA / `st=0.1` SPRT split manufactured transfer failures). |
| `hash` | 64 | matches deployment |
| `threads` | 15 | concurrency = physical cores (16) − 1 |
| `games` | 32 | per iteration; multiple of 2 and ≈ 2×threads for a stable gradient |
| `A` (spsa.json) | iterations / 10 | **must update per run** (see step 4 above) |
| `a`, `c`, `alpha`, `gamma` | defaults | do not change (weather-factory guidance) |
| per-param `step` | see tables below | sized to cause a ~2–3 Elo swing per weather-factory guidance |

## Parameter groups (tune one group at a time)

Tune **one config file per run**. Do not combine groups into one run —
the gradient becomes too noisy with many parameters at once.

### config_lmr.json — LMR weighted terms (in 1024ths)

Current values are the Phase 2.5.1 clock-TC SPSA candidate in
`SearchParams::default()`. They are baked for SPRT gating, not accepted until
the `[0,3]` primary gate passes.

| UCI option name  | Default | Range       | Step | Source in search.rs |
|------------------|---------|-------------|------|---------------------|
| `LmrTtPvAdj`     | 887     | [0, 2048]   | 80   | LMR reduction for PV / TT-PV nodes (stored positive; subtracted) |
| `LmrExactBound`  | 109     | [0, 2048]   | 80   | Reduction when TT bound is Exact |
| `LmrShallowTt`   | 656     | [0, 2048]   | 80   | Reduction when TT entry depth < depth-1 |
| `LmrCutNode`     | 780     | [0, 2048]   | 80   | Extra reduction at cut nodes |
| `LmrTableBase`   | 646     | [512, 1024] | 50   | Additive base in the LMR table formula |
| `LmrTableDiv`    | 2335    | [1536, 3072]| 50   | Logarithm divisor in the LMR table formula |
| `LmrHistDiv`     | 8395    | [4096, 16384]| 300 | History divisor in the per-move reduction adjustment |

Historical note: the default-equivalent seeds were
`1024 / 0 / 1024 / 1024 / 768 / 2304 / 8192`. The Phase 1 four-param candidate
(`914 / 136 / 1073 / 834`) was rejected after the `[0,3]` SPRT remained
inconclusive at ~58k games (`nElo ~+1.7`, LLR ~0.34). The Phase 2.4
fixed-movetime SPSA candidate was
`1110 / 98 / 880 / 1138 / 738 / 2334 / 8268`; it failed the old gate and was
replaced by the Phase 2.5.1 clock-TC candidate above.

### config_pruning.json — Pruning / margin constants

Current values are the accepted Phase 1 Group B defaults in
`SearchParams::default()`, with `FutilityNotImproving` / `LmpNotImproving`
widened to `[0,120]` for the Phase 5 post-eval retune (both were pinned near
their old `[0,60]` ceiling).

| UCI option name        | Default | Range        | Step | Source in search.rs |
|------------------------|---------|--------------|------|---------------------|
| `FutilityBase`         | 86      | [30, 150]    | 10   | `:1003`  `(base + not_improving·coeff) · depth` |
| `FutilityNotImproving` | 49      | [0, 120]     | 10   | `:1003`  not-improving coefficient |
| `RazoringCoeff`        | 191     | [60, 300]    | 20   | `:1007`  `coeff · depth` |
| `NullMoveDepthCoeff`   | 15      | [4, 30]      | 4    | `:1012`  depth-scaled null-move margin |
| `NullMoveImprovingBonus` | 25    | [0, 60]      | 8    | `:1012`  improving bonus |
| `LmpBase`              | 115     | [40, 180]    | 14   | `:1182`  LMP margin base |
| `LmpNotImproving`      | 57      | [0, 120]     | 10   | `:1182`  not-improving coefficient |
| `QuietHistPruneCoeff`  | 4419    | [1000, 8000] | 400  | `:1186`  quiet-history pruning coefficient |
| `SeePruningCoeff`      | 81      | [30, 160]    | 12   | `:1195`  SEE pruning coefficient |
| `SeePruningMax`        | 811     | [200, 1600]  | 80   | `:1195`  SEE pruning floor magnitude |
| `AspirationDelta`      | 31      | [10, 60]     | 6    | `:615`   initial aspiration half-window (cp) |
| `SingularBetaMult`     | 4       | [1, 6]       | 1    | `:1215`  `tt_score - mult·depth` |
| `LmpCountBase`         | 2       | [1, 10]      | 1    | `:2394`  base in `base + 2·d²/3` |

Each parameter name **must** match a UCI `spin` option exposed in
`src/search_options.rs` (Phase 1 work). Until those options exist, weather-factory
has nothing to set — wire up the UCI options first.

### config_futility.json — Per-move quiet futility

Current values are the accepted Phase 2.7 defaults in `SearchParams::default()`.
These margins are centipawn-scaled, so they are retuned again in Phase 4 after
the Phase 3 eval refit.

| UCI option name | Default | Range | Step | Source in search.rs |
|-----------------|---------|-------|------|---------------------|
| `FpBase`        | 184     | [0, 400] | 20 | Per-move quiet futility base margin |
| `FpCoeff`       | 117     | [0, 300] | 15 | Per-depth quiet futility coefficient |

### config_probcut.json — ProbCut margin (Phase 5)

Rarog's live ProbCut (the flat-margin form: `probcut_beta = beta + margin`,
`search.rs:1108`) was hardcoded at `180` until Phase 5 exposed it as a UCI
option for the post-eval SPSA wave. An earlier, more elaborate improving-aware
3-parameter port (separate base/depth/improving-bonus margins) was tried in
Phase 2 and dropped after SPRT H0 (`-24.5 +/- 8.5 Elo`) — that design is not
revived here; only the simple flat margin that shipped through Phase 4 is
tunable.

| UCI option name | Default | Range | Step | Source in search.rs |
|-----------------|---------|-------|------|---------------------|
| `ProbCutMargin` | 180 | [60, 400] | 20 | `:1108`  `probcut_beta = beta + margin` |

### config_tm.json — Time-management dynamic multipliers (Phase 5.1 TM group)

The clock-mode between-iteration soft-stop scales `optimum_ms` by
`falling_eval × best_move_instab × effort_factor` (the SF-style block in
`search.rs::search_root`). These are the 2.2 SF-seeded constants, now exposed as
their own SPSA group — clock play is the test/deployment target, so they are
exercised directly at `tc=3+0.03`. **Values are stored ×10000** (so `9240`
means `0.924`); the engine divides by 10000, which reconstructs the float seeds
bit-exactly, so the defaults are behaviour-identical. **TM affects only clock
play — it never moves the depth-limited `bench` fingerprint**, so there is no
bench gate for this group; SPRT at `tc=3+0.03` (plus an LTC `10+0.1` confirm,
since TM is depth/clock-sensitive) is the authority.

| UCI option name | Default | Range | Step | Meaning |
|-----------------|---------|-------|------|---------|
| `TmOptScale`     | 10000 | [5000, 20000]  | 500  | Overall ×multiplier on `optimum_ms` (10000 = ×1.0) — the highest-leverage knob |
| `TmFallBase`     | 1187  | [0, 5000]      | 150  | falling-eval base (0.1187) |
| `TmFallSlope`    | 221   | [0, 1000]      | 40   | falling-eval slope on `(prev_avg − score)` (0.0221) |
| `TmInstabBase`   | 11000 | [8000, 16000]  | 400  | best-move-instability base (1.10) |
| `TmInstabSlope`  | 22900 | [0, 50000]     | 2000 | best-move-instability slope on `tot_best_move_changes` (2.29) |
| `TmEffortHigh`   | 9240  | [6000, 12000]  | 300  | effort factor at low effort, interp t=0 (0.924) |
| `TmEffortLow`    | 7100  | [4000, 10000]  | 300  | effort factor at high effort, interp t=1 (0.71) |

### config_lazymargin.json — Lazy-eval margin (Phase 5.1b)

The lazy-eval cutoff (`eval.rs`; skip the expensive positional block when the
material + PST + pawn score already exceeds the margin). Accepted at `600` at
the seeded-0 head (+4.4 Elo, Phase 3.16). **Do the safety check first:** Phase 4
grew the positional weights, so the margin that guaranteed "no skipped term can
flip the sign" may now be too tight. **Widen it (e.g. 600 → 900/1200) and
confirm a non-regression SPRT `[-3,3]` at the post-Phase-4 eval scale** before
tuning for NPS. Only then run this SPSA. (Lazy is disabled under `--features
texel`; the mop-up runs on both eval paths, so mating is margin-independent.)

| UCI option name | Default | Range | Step | Source |
|-----------------|---------|-------|------|--------|
| `LazyMargin` | 600 | [200, 2000] | 80 | `eval.rs` lazy cutoff; pushed to the evaluator each search start |

### Futility-direction A/B (`FutilityImprovingDir` — Phase 5.1, relocated 2.5.2)

Not an SPSA config — a **discrete A/B**, folded into the futility-group work.
Rarog's reverse-futility margin (`search.rs:1041`) shrinks when `improving`
(prunes *more*); `FutilityImprovingDir` flips which side of the flag the
`FutilityNotImproving` coefficient is added to:

- `0` (default) — coefficient added when **not** improving (current / SF-RFP).
- `1` — coefficient added when **improving** (the conventional "larger margin
  when improving" direction).
- no-modulation — set `FutilityNotImproving 0` (works at either setting).

Run the A/B by setting the option per engine in fastchess (no separate binary
needed), each gated `[-3,3]` vs the current head, e.g.:

```
-engine cmd=rarog-tune.exe name=dir1 option.FutilityImprovingDir=1
-engine cmd=rarog-tune.exe name=dir0 option.FutilityImprovingDir=0
```

Keep whichever direction wins; if neither beats `0`, keep the default. The
coefficient *magnitude* (`FutilityNotImproving`) is still tuned in
`config_pruning.json`.
