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

1. **Update `A` in `spsa.json`** to `planned_iterations / 10`.
   This is weather-factory's only required change per run.
   Example: planning 10 000 iterations → set `"A": 1000`.
   The other fields (`a`, `c`, `alpha`, `gamma`) should stay at their defaults.

2. **Run `setup_spsa.ps1`** for the group you are tuning. It writes the three
   config files into the weather-factory root (next to `main.py`):
   - `cutechess.json`             (runner settings — same for every group)
   - `spsa.json`                  (SPSA hyper-params — updated per step 4)
   - `config_<group>.json` → rename to `config.json` (the parameter set)

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
2. Running `tools\sprt.ps1` (st=0.1, the deployment condition) of the tuned
   binary vs the pre-tuning head. **Keep the tuned values only if SPRT accepts H1.**

## Settings rationale

| Setting | Value | Why |
|---|---|---|
| Runner | fastchess (`use_fastchess: true`) | less overhead than cutechess-cli |
| `tc` | `1` → 1+0.01 s | Near the fast-blitz regime of the 100 ms/move deployment; avoids bullet timing jitter. The final st=0.1 SPRT bridges the small gap. |
| `hash` | 64 | matches deployment |
| `threads` | 15 | concurrency = physical cores (16) − 1 |
| `games` | 32 | per iteration; multiple of 2 and ≈ 2×threads for a stable gradient |
| `A` (spsa.json) | iterations / 10 | **must update per run** (see step 4 above) |
| `a`, `c`, `alpha`, `gamma` | defaults | do not change (weather-factory guidance) |
| per-param `step` | see tables below | sized to cause a ~2–3 Elo swing per weather-factory guidance |

## Parameter groups (tune one group at a time)

Tune **one config file per run**. Do not combine both groups into one run —
the gradient becomes too noisy with many parameters at once.

### config_lmr.json — LMR weighted terms (in 1024ths)

Defaults are the default-equivalent 1024ths LMR values. The Phase 1 group A
SPSA candidate (`914 / 136 / 1073 / 834`) was rejected after the `[0,3]` SPRT
remained inconclusive at ~58k games (`nElo ~+1.7`, LLR ~0.34).

| UCI option name  | Default | Range       | Step | Source in search.rs |
|------------------|---------|-------------|------|---------------------|
| `LmrTtPvAdj`     | 1024    | [0, 2048]   | 80   | LMR reduction for PV / TT-PV nodes (stored positive; subtracted) |
| `LmrExactBound`  | 0       | [0, 2048]   | 80   | Reduction when TT bound is Exact |
| `LmrShallowTt`   | 1024    | [0, 2048]   | 80   | Reduction when TT entry depth < depth−1 |
| `LmrCutNode`     | 1024    | [0, 2048]   | 80   | Extra reduction at cut nodes |

### config_pruning.json — Pruning / margin constants

All defaults from `src/search.rs`.

| UCI option name        | Default | Range        | Step | Source in search.rs |
|------------------------|---------|--------------|------|---------------------|
| `FutilityBase`         | 70      | [30, 150]    | 10   | `:1003`  `(70 + 20·not_improving) · depth` |
| `FutilityImproving`    | 20      | [0, 60]      | 8    | `:1003`  the `20` coefficient |
| `RazoringCoeff`        | 150     | [60, 300]    | 20   | `:1007`  `150 · depth` |
| `NullMoveDepthCoeff`   | 12      | [4, 30]      | 4    | `:1012`  `12 · depth` |
| `NullMoveImprovingBonus` | 24    | [0, 60]      | 8    | `:1012`  `24 · improving_i` |
| `LmpBase`              | 90      | [40, 180]    | 14   | `:1182`  `(90 + 25·not_improving) · depth` — base |
| `LmpImproving`         | 25      | [0, 60]      | 8    | `:1182`  the `25` coefficient |
| `QuietHistPruneCoeff`  | 4000    | [1000, 8000] | 400  | `:1186`  `−4000 · depth` (stored positive) |
| `SeePruningCoeff`      | 80      | [30, 160]    | 12   | `:1195`  `−80 · depth` (tune the magnitude) |
| `SeePruningMax`        | 800     | [200, 1600]  | 80   | `:1195`  `.max(−800)` floor magnitude |
| `AspirationDelta`      | 25      | [10, 60]     | 6    | `:615`   initial aspiration half-window (cp) |
| `SingularBetaMult`     | 2       | [1, 6]       | 1    | `:1215`  `tt_score − 2·depth` |
| `LmpCountBase`         | 4       | [1, 10]      | 1    | `:2394`  `base = 4 + 2·d²/3` — tune the 4 |

Each parameter name **must** match a UCI `spin` option exposed in
`src/search_options.rs` (Phase 1 work). Until those options exist, weather-factory
has nothing to set — wire up the UCI options first.
