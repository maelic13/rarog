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
| opening book | `tools\books\UHO_Lichess_4852_v1.epd` |
| test engines | `tools\test_engines\` |

`tools\spsa.ps1` populates `tools\weather-factory\tuner\` and launches the run.

## Setup + run — one command

`spsa.ps1` (merged setup + launch) writes the three config files into the
weather-factory root (next to `main.py`) and then launches the tuner:
   - `cutechess.json`             (runner settings — same for every group)
   - `spsa.json`                  (SPSA hyper-params; `A = iterations / 10`)
   - `config_<group>.json` → copied to `config.json` (the parameter set)

```powershell
./tools/spsa.ps1 -ConfigGroup lmr -EngineSuffix p25-lmr -Iterations 5000
```

It runs from the repo root (no cd), pipes the console through `watch.ps1`
(per-game noise → the log, only param/report blocks on screen), and returns
you to root on Ctrl-C. `-Resume` continues an interrupted run; `-SetupOnly` /
`-LaunchOnly` split the two phases. weather-factory writes the running
parameter values to `tuner\state.json` every `save_rate` games.

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
| `threads` | auto | concurrency = detected physical cores − 2; explicit physical-core affinity is injected by `setup_tools.ps1` |
| `games` | 32 | per iteration; multiple of 2 and ≈ 2×threads for a stable gradient |
| `A` (spsa.json) | iterations / 10 | **must update per run** (see step 4 above) |
| `a`, `c`, `alpha`, `gamma` | defaults | do not change (weather-factory guidance) |
| per-param `step` | see tables below | sized to cause a ~2–3 Elo swing per weather-factory guidance |

## Coverage audit — run this before any tune

```powershell
./tools/audit_spsa_coverage.ps1
```

9.0a's `search_params!` macro made the four in-source copies of a tunable
impossible to desync, but **these config files sit outside that macro and drift
silently**. Drift is expensive in a specific way: a group whose seeds trail the
baked defaults spends its early, largest gain steps just walking back to where
the engine already was. First run of the audit (2026-07-25) found **15 real
drifted seeds** — `config_history.json` still held pre-8.4 `Hist*` values,
`config_lmr.json` held the REJECTED 8.6-era LMR values, and
`config_pruning.json` held the pre-9.0a `SeePruningCoeff` 83 / `SeePruningMax`
804. All are now synced to `params.rs`.

**Expected audit output** (not problems):

- **`corr` reports 4 off-default seeds.** Deliberate: the three `Corr*Scale`
  knobs start at an interior 128 rather than their code default of 0 so SPSA
  can explore both directions, and `CorrGuardCapture` is pinned ON. See the
  `config_corr.json` section below.
- **Three tunables are in NO group**, all correctly: `HistNoAging` (a `[0,1]`
  A/B; 8.1b was REJECTED at −6.6, do not retry), `FutilityImprovingDir` (a
  discrete A/B, documented at the end of this file), and
  `EvalPruneTtMinDepth` — which IS in `config_pruning.json` as of 2026-07-25,
  so if the audit lists it, the config was reverted.
- **8 names appear in two groups** (`Hist*` in `histcov`+`history`, the two
  SEE params in `pruning`+`see`). Not an error, but re-tuning one group can
  undo the other's fit — check which group last produced the accepted values.

⚠ **`params.rs` defaults are the single source of truth for seeds.** The
`Default` columns in the per-group tables below are documentation and MAY LAG;
trust the audit script, not the tables. Ranges and steps are only defined here,
so those columns are authoritative.

## Parameter groups

Tune **one config file per run** — but note that a config file may itself be a
MERGE of several groups.

⛔ **The old rule here — "do not combine groups into one run, the gradient
becomes too noisy with many parameters at once" — was WRONG and is retracted
(2026-07-30).** It was never measured, and
`tools/spsa_convergence_model.py` refuted it against our own shipped schedule:
**p=6 and p=26 converge at nearly identical rates.** That is Spall's result —
SPSA costs 2 evaluations per iteration *regardless of dimension*, so dimension
is close to free. What actually dominates is ITERATION COUNT (see below), and
splitting one 26-knob problem into three 8-knob runs triples the nights while
throwing away every cross-knob interaction. Where knobs interact by
construction — the `Corr*Scale` values multiply the very margins the pruning
knobs set — tuning them together is not merely cheaper but *more correct*.

Two rules that DO hold, and that the retracted one was standing in for:

- **5,000 iterations is the floor for any tune.** At 1,000–2,500 a run barely
  beats its own seed, and every Rarog tune before 10.4.6 sat in that range
  (8.5's 3,673 was the longest). A 1,000-iteration SPSA is not a cheap tune, it
  is a null result with a bake attached.
- **Run the coverage audit first** (above). Class 5 and 6 are hard errors: a
  pinned knob or one whose perturbation rounds to zero stops being *measured*
  while still being *updated*, i.e. it random-walks and drags the fit with it.

### config_selectivity.json — 10.4.6(a) THE selectivity re-fit (28 knobs)

The merged group: `config_pruning` (14) + the four non-overlapping
`config_see` knobs + `config_corr` (8) + `config_futility` (2). One tune
replaces four. `CorrGuardCapture` is deliberately EXCLUDED — it is a discrete
A/B knob, and pinning it inside a tune is precisely what cost 8.5 its gate
(the guard silently discarded 59.7% of correction training, and 117k games
fitted eight knobs to a crippled signal).

**Why it exists: 10.0(c) measured this exact surface as mis-tuned.** A blind,
untuned, uniform 15% shift of these constants toward *less* selectivity beat the
fitted values by **+4.06 ± 3.71 Elo (LOS 98.42%, 14,196 games)**. Values sitting
inside the SPSA noise floor cannot be beaten by a blind shift, so this group is
demonstrably outside it — which is what converts 10.4.6 from a speculative
re-tune into the cycle's headline item.

⚠ **The seeds are DELIBERATELY not the baked defaults, and the coverage audit
will report 8 "drifted seeds" for this file. That is intended, not drift.**
Eight knobs are seeded at 10.0(c)'s probe values (a measured +4.06 better than
the defaults) so the run starts from the best point we know rather than from one
we have just measured as worse: `FutilityNotImproving` 48, `RazoringCoeff` 222,
`LmpNotImproving` 72, `QuietHistPruneCoeff` 5829, `SeePruningCoeff` 59,
`SeePruningMax` 999, `FpBase` 212, `FpCoeff` 135. If the tune ends up going
nowhere, its final theta should remain near the seeded probe values and the
gate should read ≈+4 — i.e. the floor is a known gain rather than a known zero.

**Completed 2026-08-01:** 5,000 iterations / 160,000 games. A same-binary
tail-15%-mean-vs-final-theta comparison was stopped as a wash after 1,800 games
(tail +1.19 ± 16.05 nElo, LLR −0.01, zero anomalies). Rarog therefore follows
the conventional SPSA rule: bake the complete `Final parameters` theta printed
by the tuner. Do not create an averaging window after seeing a trajectory; if
iterate averaging is ever wanted, define and validate that estimator before
the run. The exact 28-value vector and rail analysis are recorded in `PLAN.md`.
The baked source and tune binary at theta bench-match exactly at **6,477,102**
nodes (+21.7% over the fail-soft tuning substrate), pending the strength gate.

📌 **`FutilityBase` (60) and `LmpBase` (88) are the KILL-CHECKPOINT and are
held at the accepted-head values on purpose** — one full `step` below the
probe direction the other eight start from. They are the two highest-traffic
margins in the group (RFP cuts 21.9% of interior nodes; LMP discards more moves
than there are interior nodes). By ~1,500 iterations the fixed schedule must
visibly walk them UP toward ~69 and ~101. **If they wander instead, STOP and
debug before spending night two** — the rest of the run cannot help either, and
this is the one direction in the whole group whose sign is backed by four
independent measurements (8.6 −7.78, 8.7 −7.29, 8.11 −5.96 and 10.0(c) +4.06),
so a tuner that cannot find it lacks resolving power at this noise level.

⚠ `EvalPruneTtMinDepth` seeds at 0, on its MIN rail (one-sided gradient — the
audit warns about this). Accepted here because 0 *is* today's behaviour and the
only interesting direction is up: it decides whether a depth-0 qsearch bound may
override the static eval for an RFP decision at depth 8. It is the knob that
carries 8.11's honest-bounds question, since the re-applied fail-soft covers the
prune exits only and leaves the tail's stored bound alone. The three
`Corr*Scale` knobs seed on their min rail for the same reason (8.5 closed them
neutral; any activation is upward).

⚠ Expect low resolving power on the `Corr*` block: 8.5 measured that bundle at
+1.4 ± 4.9 *in total* across 8 knobs, which is inside the "curvature below
~0.5 Elo per full step is unfittable at 32 games/iteration" class. They are
included because dimension is free and they multiply the margins, not because
they are expected to move.

### config_corr.json — Phase 8.5 correction semantics + margins + blend

The 8.5 bundle: (a) a capture guard on correction updates, (b) three
|correction|-scaled margin/reduction knobs, (c) five correction-blend source
weights. All are exposed in `src/params.rs` seeded NEUTRAL, so the tune binary
is bench-identical to the 8.4 head at defaults (verified 5,173,540).

**`CorrGuardCapture` is PINNED ON in this config** (`value/min/max = 1`): the
guard is a semantic hypothesis that must be *active* while SPSA fits (b)/(c) —
"the guard is what makes (b)'s signal honest" (PLAN 8.5). weather-factory
perturbs it, but both arms clamp back to 1, so it stays on and contributes no
gradient. The SPSA therefore tunes the 8 continuous knobs below.

The (b) scales start at an interior `128` (not their `0` code default) so SPSA
explores both directions; if the true optimum is 0 they drift back down and bake
to off. The (c) weights start at their neutral `128`.

| UCI option name | Start | Code default | Range | Step | Source in search.rs |
|---|---|---|---|---|---|
| `CorrGuardCapture` | 1 (pinned) | 0 | [1,1] | 1 | skip correction update on a capture cutoff / best move |
| `CorrRfpScale`   | 128 | 0 | [0,512] | 50 | `+ |corr|·s/128` on the reverse-futility margin |
| `CorrFutScale`   | 128 | 0 | [0,512] | 50 | `+ |corr|·s/128` on the quiet-futility margin |
| `CorrLmrScale`   | 128 | 0 | [0,512] | 50 | `− |corr|·s/128` on the LMR reduction (1024ths) |
| `CorrWeightPawn` | 128 | 128 | [0,384] | 30 | pawn-key correction weight (128 = old ×1) |
| `CorrWeightMinor`| 128 | 128 | [0,384] | 30 | minor-key correction weight |
| `CorrWeightOwnNp`| 128 | 128 | [0,384] | 30 | own non-pawn correction weight |
| `CorrWeightTheirNp`| 128 | 128 | [0,384] | 30 | opponent non-pawn correction weight |
| `CorrWeightCont` | 128 | 128 | [0,384] | 30 | continuation correction weight (applied after its inherent `/2`) |

**Pre-registered decomposition (PLAN 8.5):** gate the full bundle `[0,3]` vs the
8.4 head; on H0, retry **(b) margins-only, guard dropped** once before
abandoning.

### config_lmr.json — LMR weighted terms (in 1024ths)

Current values are the **Phase 2.5.1** clock-TC SPSA values in
`SearchParams::default()`. The **Phase 5.1 re-tune was REJECTED** (SPRT H0,
−2.58 ± 3.02 Elo, 21,850 games): LMR lives in depth/move-index space and is
~eval-scale-independent, so the Phase-4 rescale did not move its optimum —
re-tuning it was low expected value and it confirmed flat/slightly-worse.
Reverted; **do not re-tune LMR again unless the LMR *structure* changes.**

| UCI option name  | Default | Range       | Step | Source in search.rs |
|------------------|---------|-------------|------|---------------------|
| `LmrTtPvAdj`     | 887     | [0, 2048]   | 80   | LMR reduction for PV / TT-PV nodes (stored positive; subtracted) |
| `LmrExactBound`  | 109     | [0, 2048]   | 80   | Reduction when TT bound is Exact |
| `LmrShallowTt`   | 656     | [0, 2048]   | 80   | Extra reduction on late moves (`tt_move` present && searched≥4) — name is legacy/misleading (see note) |
| `LmrCutNode`     | 780     | [0, 2048]   | 80   | Extra reduction at cut nodes |
| `LmrTableBase`   | 646     | [512, 1024] | 50   | Additive base in the LMR table formula |
| `LmrTableDiv`    | 2335    | [1536, 3072]| 50   | Logarithm divisor in the LMR table formula |
| `LmrHistDiv`     | 8395    | [4096, 16384]| 300 | History divisor in the per-move reduction adjustment |

> Note: `LmrShallowTt`'s param doc claims "absent TT / no tt_move", but the live
> condition (`search.rs:1393`) fires when a `tt_move` is **present** on a late
> move — a name/comment discrepancy flagged during the Phase 5.1 review. The tune
> is valid (SPSA optimizes the live code); the rename/doc-fix is a separate cleanup.

Historical note: the default-equivalent seeds were
`1024 / 0 / 1024 / 1024 / 768 / 2304 / 8192`. The Phase 1 four-param candidate
(`914 / 136 / 1073 / 834`) was rejected after the `[0,3]` SPRT remained
inconclusive at ~58k games (`nElo ~+1.7`, LLR ~0.34). The Phase 2.4
fixed-movetime SPSA candidate was
`1110 / 98 / 880 / 1138 / 738 / 2334 / 8268`; it failed the old gate and was
replaced by the Phase 2.5.1 clock-TC candidate above.

### config_pruning.json — Pruning / margin constants

Defaults are the **Phase 5.1 pruning SPSA candidate** (tc=3+0.03, 2,482 iters /
79,424 games at the post-Phase-4 eval scale), baked into `SearchParams::default()`
pending the confirming `[0,3]` SPRT. `FutilityNotImproving` / `LmpNotImproving`
ceilings were widened `[0,60]→[0,120]` for this retune. `SingularBetaMult`
**pinned at its `[1,6]` ceiling** and stayed there: the config was widened `6→8`
in the repo, but weather-factory resumed from its own state and kept the `[1,6]`
range, so `6` was **never actually tested against `7–8`**. It is baked at `6`
(the tuner's best estimate, conservative) and flagged as an **open micro-item** —
re-poke it with the `[1,8]` range loaded fresh (run `spsa.ps1` without
`-Resume`, or delete `tuner/state.json` first).

| UCI option name        | Default | Range        | Step | Source in search.rs |
|------------------------|---------|--------------|------|---------------------|
| `FutilityBase`         | 60      | [30, 150]    | 10   | `:1003`  `(base + not_improving·coeff) · depth` |
| `FutilityNotImproving` | 42      | [0, 120]     | 10   | `:1003`  not-improving coefficient |
| `RazoringCoeff`        | 193     | [60, 300]    | 20   | `:1007`  `coeff · depth` |
| `NullMoveDepthCoeff`   | 10      | [4, 30]      | 4    | `:1012`  depth-scaled null-move margin |
| `NullMoveImprovingBonus` | 32    | [0, 60]      | 8    | `:1012`  improving bonus |
| `LmpBase`              | 88      | [40, 180]    | 14   | `:1182`  LMP margin base |
| `LmpNotImproving`      | 63      | [0, 120]     | 10   | `:1182`  not-improving coefficient |
| `QuietHistPruneCoeff`  | 5069    | [1000, 8000] | 400  | `:1186`  quiet-history pruning coefficient |
| `SeePruningCoeff`      | 83      | [30, 160]    | 12   | `:1195`  SEE pruning coefficient |
| `SeePruningMax`        | 804     | [200, 1600]  | 80   | `:1195`  SEE pruning floor magnitude |
| `AspirationDelta`      | 30      | [10, 60]     | 6    | `:615`   initial aspiration half-window (cp) |
| `SingularBetaMult`     | 6       | [1, 8]       | 1    | `:1215`  `tt_score - mult·depth` (pinned at old [1,6] ceiling; [1,8] not yet tested — re-poke) |
| `LmpCountBase`         | 2       | [1, 10]      | 1    | `:2394`  base in `base + 2·d²/3` |
| `EvalPruneTtMinDepth`  | 0       | [0, 8]       | 1    | minimum TT entry depth before the TT score may replace the static eval in `eval_for_pruning` (added 2026-07-25) |

**On `EvalPruneTtMinDepth`** — it belongs in THIS group rather than in a
standalone A/B, and the reason is the 8.6 lesson. negamax hands off to qsearch
at `depth <= 0`, so it only ever stores at depth ≥ 1 and depth-0 entries are
exactly the qsearch ones; letting one decide an RFP cut at depth 8 is hard to
defend, which is the principled case for the value **1**. But the bench sweep
(2026-07-25) shows higher values simply prune harder — MinDepth 3 → −3.5%
nodes, 6 → −5.3%, 8 → −10.0% — and **fewer nodes is not evidence of strength**:
8.6 tuned a candidate that searched 16% more aggressively, won its self-play
SPSA, then lost −7.78 to the accurate baseline. So this knob must be fitted
JOINTLY with the margins that compensate for it, never A/B'd alone.
⚠ Caveat: SPSA handles small-integer knobs poorly — with step 1, the
perturbation `step · c_t` is ~0.4 and rounds to zero much of the time. The same
shape already bit `SingularBetaMult`, which pinned at its ceiling. Expect weak
gradient here and read the result accordingly.

Each parameter name **must** match a UCI `spin` option exposed in
`src/search_options.rs` (Phase 1 work). Until those options exist, weather-factory
has nothing to set — wire up the UCI options first.

### config_futility.json — Per-move quiet futility

Current values are the **Phase 2.7** values in `SearchParams::default()`. The
**Phase 5.1 re-tune was a wash** (candidate `157 / 126`; SPRT +0.37 ± 4.33 over
10,216 games — no change) and was reverted: the pruning group already re-tuned
the neighbouring cp-margins (RFP `FutilityBase/NotImproving` + LMP
`LmpBase/NotImproving`), leaving this narrower per-move lever with little to add.

| UCI option name | Default | Range | Step | Source in search.rs |
|-----------------|---------|-------|------|---------------------|
| `FpBase`        | 184     | [0, 400] | 20 | Per-move quiet futility base margin (`:1247`) |
| `FpCoeff`       | 117     | [0, 300] | 15 | Per-depth quiet futility coefficient (`:1247`) |

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
exercised directly at `tc=3+0.03`. **Values are stored ×10000** (so `8247`
means `0.8247`); the engine divides by 10000. **TM affects only clock play — it
never moves the depth-limited `bench` fingerprint** (an unchanged bench after
baking is *correct*, not a bug), so the gate is SPRT at `tc=3+0.03` **plus an
LTC `10+0.1` confirm and a time-forfeit check**, since TM is depth/clock-sensitive.

Current values are the **2.2 SF seeds**. The **Phase 5.1 re-tune was a wash**
(candidate `10776/1560/231/10364/23667/8247/7770`, 2,739 iters / 87,648 games;
SPRT +0.20 ± 4.16 over 10,540 games, **0 forfeits**) → reverted. Third search
null (LMR, futility, TM) — the search-constant tuning is exhausted; pruning's
+12.07 was the whole Phase-5 search win.

| UCI option name | Default | Range | Step | Meaning |
|-----------------|---------|-------|------|---------|
| `TmOptScale`     | 10000 | [5000, 20000]  | 500  | Overall ×multiplier on `optimum_ms` (10000 = ×1.0) — highest-leverage knob |
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
confirm a non-inferiority SPRT `[-3,0]` at the post-Phase-4 eval scale** before
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
needed), each gated `[-3,0]` vs the current head, e.g.:

```
-engine cmd=rarog-tune.exe name=dir1 option.FutilityImprovingDir=1
-engine cmd=rarog-tune.exe name=dir0 option.FutilityImprovingDir=0
```

Keep whichever direction wins; if neither beats `0`, keep the default. The
coefficient *magnitude* (`FutilityNotImproving`) is still tuned in
`config_pruning.json`.
