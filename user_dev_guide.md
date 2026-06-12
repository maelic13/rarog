# Rarog Development Workflow Guide

How to drive the improvement plan with Claude (or any AI model) and know
exactly what to say, what to run, and when a decision is yours to make.
Read alongside `PLAN.md`, which contains all the technical details.

---

## The basic rhythm

Every step is a ping-pong between you and the model:

```
You  →  "Implement next step of the plan."
Model→  Writes code, verifies build, tells you exactly what to run.
You  →  Run the command, come back with the result.
Model→  Acts on the result. Either commits and moves on, or flags a decision.
```

Most iterations cost you **one message**. You say the result; the model
handles everything else.

---

## How to start a session

Opening message when continuing work:

> "Implement next step of the plan."

The model will read `PLAN.md`, check the current branch state, and know
exactly where we left off. You do not need to re-explain context.

If you want a specific phase or feature instead of "next":

> "Implement Phase 1 step 2 — expose the search constants as UCI options."
> "Implement the ProbCut port from Phase 2."

---

## After the model writes code — your turn

The model will end its response with an explicit instruction like:

> **"Build and test:**
> `./tools/build_test.ps1 -Suffix feat-probcut`
> `./tools/sprt.ps1 -EngineA ... -EngineB ... -NameA "ProbCut" -NameB "Head"`"

Run those commands. Then come back with one of these reports:

### SPRT result

> "SPRT result: **H1 accepted** after 1,840 games."

or

> "SPRT result: **H0 accepted** after 2,210 games."

That is all the model needs. It will commit (H1) or discard (H0) and move on.

### SPSA result

After weather-factory finishes (or you stop it at a reasonable point):

> "SPSA done. Tuned values:
> FutilityBase=62, RazoringCoeff=138, NullMoveDepthCoeff=10,
> AspirationDelta=21  (ran ~8000 iterations)"

The model will bake those values in as new defaults, build a test binary,
and give you the SPRT command to confirm them.

### Bench fingerprint check

When the model asks you to verify a refactor didn't change behaviour:

> "Bench 13 result: **5,318,762 nodes** ✓"  (matches Phase 1 final baseline — safe)
> "Bench 13 result: **4,891,203 nodes**"      (changed — expected for real features)

---

## Decision points — when the model will stop and ask you

These are moments where the choice is genuinely yours:

| Situation | The question | Typical answers |
|---|---|---|
| **Calibration test returns H1** | Something is wrong with the harness — investigate or re-check the released binaries? | Investigate (re-run, check binary paths) |
| **SPRT returns H0** | Discard and move on, or try a second SPSA pass first? | Usually: discard and move on |
| **SPSA converges to an outlier** (e.g. a constant at its range boundary) | Accept the outlier, widen the range and re-run, or discard? | Widen and re-run if it looks plausible; discard if it feels wrong |
| **Feature touches risky code** (tt.rs, zobrist.rs) | Proceed with caution or skip this feature? | Your call based on appetite |
| **End of a phase** | Run the Little Blitzer gauntlet before moving on? | Yes, always recommended |
| **Time forfeits after the TM rewrite (2.2)** | Enable the overhead safety valve, or investigate the specific GUI? | Enable the valve (`min(MoveOverhead, movetime/10)`), re-test |

When you hit one of these, just answer the question in plain English.
The model will proceed accordingly.

---

## Prerequisites — fresh checkout only

Phase 0 is already complete for the current workspace. On a fresh checkout,
do these once before running new SPSA/SPRT work:

- [x] **Install helper tools locally** → `./tools/setup_tools.ps1`
      (creates `tools\bin\fastchess.exe` and `tools\weather-factory\`)
- [x] **Run the calibration test** (takes ~15–30 min):
      ```powershell
      ./tools/sprt.ps1 `
          -EngineA "tools\test_engines\rarog-v2.1.0-windows-pext-pgo-codex-work.exe" `
          -EngineB "tools\test_engines\rarog-v2.0.2-windows-pext-pgo.exe" `
          -NameA "CW" -NameB "2.0.2"
      ```
      **Expected: H0 accepted.** These two engines are behaviour-identical.
      If it returns H1, stop and report — the harness needs investigation
      before any further results can be trusted.
- [x] **Report the calibration result** to the model and say "Phase 0 complete,
      implement next step."

Weather-factory (for SPSA) is only needed from Phase 1 step 4 onward.
Setup instructions: `tools/spsa_configs/README.md`.

---

## Quick command reference

```powershell
# Build a named pext-PGO test binary into tools\test_engines\
./tools/build_test.ps1 -Suffix <name>

# SPRT — test a gain (default H0=0, H1=5)
./tools/sprt.ps1 -EngineA <new.exe> -EngineB <head.exe> -NameA "X" -NameB "Head"

# SPRT — smaller feature (tighter bound, faster conclusion)
./tools/sprt.ps1 -EngineA <new.exe> -EngineB <head.exe> -NameA "X" -NameB "Head" -Elo1 3

# SPRT — non-regression / simplification check
./tools/sprt.ps1 -Mode simplify -EngineA <clean.exe> -EngineB <head.exe> -NameA "Clean" -NameB "Head"

# SPRT — default-equivalence / refactor check (symmetric bounds, accepts H0 in ~1-3k games)
# Use this INSTEAD of [0,5] when verifying a pure refactor with identical bench fingerprint.
# With [0,5] and truly identical engines, H0 can take 10,000+ games to formally accept.
./tools/sprt.ps1 -EngineA <refactor.exe> -EngineB <head.exe> -NameA "Refactor" -NameB "Head" `
    -Elo0 -3 -Elo1 3

# SPSA tuning — see tools/spsa_configs/README.md for full setup
cd tools\weather-factory
python main.py

# Bench fingerprint check — run the release binary directly in a terminal:
#   .\target\release\rarog.exe
#   bench 13
#   quit
# Expected baseline: "Nodes searched  : 5318762" (Phase 1 final, after LMR-1024ths port)
# (PowerShell piping is unreliable for this; type the commands interactively)

# Inspect what a branch added
git log --oneline 5a8ce52..v2.1.0-codex
git diff 5a8ce52 v2.1.0-codex -- src/search.rs
```

---

## Phase progress tracker

Update this as each step is completed.

### Phase 0 — Harness ✅
- [x] fastchess SPRT script (`tools/sprt.ps1`)
- [x] pext-PGO build script (`tools/build_test.ps1`)
- [x] weather-factory configs (`tools/spsa_configs/`)
- [x] fastchess installed at `tools\bin\fastchess.exe`
- [x] Calibration test passed (H0 accepted: codex-work ≈ 2.0.2, ~10k games)

### Phase 1 — SPSA-tune existing constants
- [x] `SearchParams` struct + `src/params.rs` — commit `2b39f24`
- [x] 13 constants exposed as UCI spin options (`src/search_options.rs`)
- [x] **SPRT gate #1 (default-equivalence)** — bench 13 = **4,713,975** ✓ (pre-LMR port), SPRT ~2.4k games score 49.57% LLR=-0.68 ✓ (refactor is behavior-safe)
- [x] SPSA group B (pruning/margin constants) tuned — commit `fae334a`
      (2271 iters / 72672 games; biggest movers: FutilityImproving 20→51,
      LmpImproving 25→53, SingularBetaMult 2→4, LmpBase 90→115)
- [x] **SPRT group B confirmation** — **H1 accepted** after 19,458 games. nElo +6.17 ± 4.88, LOS 99.34%.
- [x] Gate tunable options behind `--features tune` — commit `2fe6cc4`
- [x] SPSA group A unblocked: 1024ths LMR port (default-equivalent, commit `d1f60be`).
      6,478 games, score 50.98% — clearly safe, SPRT skipped (unrelated OS restart + no regression).
- [x] Group B pass-2 re-tune (weather-factory resumed old state.json by mistake):
      2552 iters / 81664 games from already-tuned start; small refinements baked in — commit `c121892`.
      SPRT skipped: changes are tiny continuation of already-confirmed Group B values.
- [x] `setup_spsa.ps1` bug fixed: now deletes `tuner/state.json` before each setup.
- [x] **SPSA group A** (LMR: LmrTtPvAdj, LmrExactBound, LmrShallowTt, LmrCutNode)
      tuned — 3565 iters / 114080 games. Candidate values:
      LmrTtPvAdj=914, LmrExactBound=136, LmrShallowTt=1073, LmrCutNode=834.
- [x] **SPRT group A confirmation** — **rejected / inconclusive**. `[0,5]`
      first run: ~54k games, nElo +3.32 ± 2.92, LLR +1.86. `[0,3]`
      rerun: ~58k games, nElo +1.7 ± 2.8, LLR ~+0.34. Timeouts were balanced
      (13 Phase1Final / 15 Phase1LMR) and treated as harness jitter, not an
      engine-specific issue. Reverted LMR values to default-equivalent
      `1024 / 0 / 1024 / 1024`.
- [x] **Phase 1 complete** — accepted Group B pruning/margin tune only.

> Every SPSA group earns its own SPRT. The groups tune different behavior and
> may transfer differently to `st=0.1`. Never skip a group's SPRT or roll two
> groups into one confirmation test.

### Phase 2 — Repairs & proven tuning
(re-scoped 2026-06-10 after the cross-engine measurements — see PLAN.md §5.0
finding 10; item numbers = PLAN.md §5 sub-sections. Codex ports and the speed
pass moved to Phase 4: eval fitting comes first.)
- [~] `improvements` branch: check-aware ordering — **H0 discarded** (~11k games, LLR flat −0.5 to −0.8; use `[-3,3]` bounds for small features next time)
- [x] 2.1 ProbCut — **dropped**. SPSA tuned to 165/1/31 (base/depth/improving);
      SPRT [0,3]: **H0**, -24.5 ± 8.5 Elo after 3380 games. The codex-branch
      implementation (cut-node gating, SEE threshold, verification search)
      was -25 Elo vs the original flat `beta+180` code already in master.
      Reverted to original. Commit `426e6e8`.
- [x] 2.2 Stockfish-style time management rewrite — **confirmed**. Commit
      `72f1c54`. Bench 13 = 5,318,762 (unchanged from Phase 1 final).
      (i) `[0,5]` st=0.1 — **H1**, +81.2 ± 19.5 Elo (nElo +106.6), 762 games.
          The broken `max(movetime-10ms, 1ms)` was capping to depth 1 at st=0.1.
      (ii) `[-5,0]` simplify — **H1** (no regression; also gains in clock mode),
          +72.6 ± 18.8 Elo (nElo +98.3), 762 games.
      Zero time forfeits in both runs.
- [ ] 2.3 History maintenance per SF/Reckless: delete per-search halving,
      persist across searches, reset only `low_ply_history` per search.
      Bench changes; SPRT `[0,3]`. Fallback if H0: keep halving but only
      every 2nd search.
- [ ] 2.4 LMR formula coefficients exposed (`LmrTableBase`=768,
      `LmrTableDiv`=2304, `LmrHistDiv`=8192) + SPSA group A **redo** with all
      7 LMR params + SPRT. (Basilisk's identical re-tune passed +15.6 Elo;
      Rarog's first attempt lacked exactly these three knobs.)
- [ ] 2.5 Per-move quiet futility pruning, depth ≤ 8 (seed `FpBase=180`,
      `FpCoeff=128`) + SPSA + SPRT.
- [ ] 2.6 LMR do-deeper/do-shallower re-search (seeds 64 / 8, deeper margin
      includes `+2*reduction`) + SPSA + SPRT.
- Extended correction history — **removed: already in baseline** (verified
  against `master`; porting it would be a no-op)
- Codex time management — **superseded** by 2.2 (SF-style rewrite)

### Phase 3 — Texel-tune the eval (gradient-trace pipeline)
(linear-trace gradient tuner with Adam, NOT coordinate descent — full
self-contained specs including the loss function, trace design, and dataset
recipe are in PLAN.md §6.)
- [ ] 3.0 `EvalParams` struct + param registry over the **existing** eval
      weights (bench-identical; release bench wall-time within ~3%)
- [ ] 3.1 Loader (`RAROG_EVAL_FILE`) + `dumpeval` round-trip
      (`--features tune` only)
- [ ] 3.2 Trace instrumentation (`texel` feature; bypass BOTH eval caches) +
      tuner binary (K-fit, Adam, group masks, L2-to-PeSTO for PSTs;
      acceptance: reconstruction == evaluate() exactly on 10k positions)
- [ ] 3.3 Dataset: node-limited self-play datagen (~60k games, nodes=8000)
      + extraction filters + holdout by game; ≥1.5M train positions
- [ ] 3.4a Material tuned + SPRT (pipeline proof — debug, don't proceed, if
      it fails)
- [ ] 3.4b Scalars (non-KS, non-PST) tuned + SPRT
- [ ] 3.4c King safety block tuned + SPRT
- [ ] 3.4d PSTs + material refit (L2 toward PeSTO) + SPRT
- [ ] 3.4e Global polish + SPRT (stop here regardless)
- [ ] 3.5 New eval terms ported one at a time (claude-branch terms, then the
      structural upgrades in PLAN.md §6: attack maps, threats package, king
      safety v2, per-count mobility), each trace-instrumented + retuned + SPRT
- [ ] 3.6 External gauntlet + CHANGELOG + PGO assets

### Phase 4 — Search-efficiency wave (EBF gap) + consolidation
(track EBF with the protocol in PLAN.md §7: three fixed positions,
`go movetime 1000`, EBF ≈ nodes^(1/depth), averaged; re-measure after each
accepted item; target ~2.2 → ~1.9. Trend metric only — SPRT decides.)
- [ ] Second SPSA wave over all search constants (pruning, LMR, Phase-2
      additions) at the post-Phase-3 head — eval scale changed, margins must
      re-fit; SPRT each group
- [ ] Search-wave items (PLAN.md §7 step 2): history bonus/malus formula
      (seeds 170/90/1700, 180/100/1500); qsearch TT-bound stand-pat;
      LMR tt-move-is-capture; qsearch quiet checks (cap 4–6, SEE ≥ 0);
      double-extension cap (default 8, non-regression gate); razoring
      depth ≤ 1 experiment
- [ ] Codex ports (moved from Phase 2): multi-cut / singular refinements;
      threat-aware history (prefer Reckless's threat-indexed shape if the
      codex port fails); TT-cutoff / fail-low-parent history;
      (optional) `tt.rs` overhaul
- [ ] Speed pass: profile first, then apply the micro-opt list in PLAN.md §7
      step 4; bench-identity gate per change, target ≥2.9 M NPS
- [ ] Reckless-derived menu (PLAN.md §7 step 5): aspiration-window
      modernization; correction-magnitude-aware margins; hindsight
      reductions; cutoff-count LMR term; bad-noisy futility; qsearch SEE
      threshold from `(alpha − eval)`
- [ ] External gauntlet + release; extend opponent ladder (SF capped 2800+)
      as milestones fall

### NNUE readiness (NOT a scheduled phase — guardrails only)
- [ ] Not planned, not scheduled. The only action is to keep the eval boundary
      clean *throughout Phases 1–4* (PLAN.md §11 + the eval-boundary ground rule
      below). No NNUE tasks to track.

### Release gates (after each phase)
- [x] Little Blitzer gauntlet vs 2.0.2, Stockfish 18-2500, Basilisk 1.4.9
      (in-progress external gauntlet showed Rarog 2.1.0 ahead of 2.0.2 by
      about +11.6 Elo after ~2065 games/player)
- [x] CHANGELOG updated
- [x] PGO release assets rebuilt with `pext` + `avx2`
      (`target/dist/rarog-v2.1.0-windows-pext-pgo.exe`,
      `target/dist/rarog-v2.1.0-windows-avx2-pgo.exe`; bench 13 =
      5,318,762 nodes)

---

## What makes a good result report

**Minimal (always sufficient):**
> "H1 accepted after 2,100 games."

**Helpful extras (paste if available):**
> "H1 accepted after 2,100 games. Score 53.1%. LLR crossed +2.94."

**For SPSA:**
> "Stopped at 6,000 iterations. Final values:
> FutilityBase=65  FutilityImproving=17  RazoringCoeff=144
> NullMoveDepthCoeff=11  NullMoveImprovingBonus=26
> LmpBase=88  SeePruningCoeff=74  AspirationDelta=22  SingularBetaMult=2"

**If something looks wrong:**
> "fastchess exited immediately with: [paste error message]"
> "Bench 13 returned 0 nodes — engine crashed on startup."

The model will diagnose and fix. You don't need to understand the error.

---

## Why search constants are UCI options (and when to remove them)

weather-factory (the SPSA driver) has no interface to the engine other than UCI.
To perturb `FutilityBase`, it sends `setoption name FutilityBase value 65` before
each mini-match. There is no other mechanism — UCI options are required.

This is standard practice: Stockfish, Ethereal, and most modern engines expose
constants during development behind a compile-time flag (e.g. `--features tune`),
then strip them from release builds. The v2.1.0-claude branch already scaffolded
this with `tune.rs` and `--features tune`.

**Current state:** options are always-on (no feature flag yet). Fine for development.
**Before any public release:** gate them behind `--features tune` so production
binaries don't pollute the UCI option list shown to GUIs.

---

## Ground rules (keep the model honest)

- **Never accept a change without a bench-13 check first.** The model should
  always report the new fingerprint for real changes, or confirm it is
  unchanged for refactors. For pure refactors (bench fingerprint unchanged),
  the bench result alone is sufficient proof of equivalence — a full SPRT
  H0 acceptance is not required. If you do run SPRT on a refactor, use
  `-Elo0 -3 -Elo1 3` (symmetric bounds), not `[0, 5]`, to get a verdict in
  ~1–3k games rather than 10k+.
- **Never skip the SPRT gate.** If the model says "this is clearly good, let's
  skip the test" — refuse. The whole point of this plan is that we learned
  "clearly good" changes can still lose Elo.
- **SPSA proposes; SPRT decides.** Tuned values from weather-factory are
  *candidates*. The st=0.1 SPRT result is the final word.
- **One feature per commit.** If the model bundles two features, ask it to
  split them.
- **Run the Little Blitzer gauntlet at the end of each phase**, not just
  after individual features. Self-play can over-fit; external opponents catch
  it.
- **Keep the eval boundary clean (NNUE door open).** The search must only reach
  eval through `Evaluator::eval()` — a single function taking a board and
  returning a side-to-move score. Never call eval helpers, piece values, PST
  lookups, or mobility counts directly from `search.rs`; never let pruning
  margins depend on eval internals. This costs nothing now (it is just clean
  design) and makes a future HCE→NNUE switch a localized replacement rather than
  a surgical rewrite. If the model proposes code that crosses this boundary,
  reject it. Full guardrails in PLAN.md §11 — note NNUE is **not scheduled**;
  this is the only thing to keep in mind for it during normal HCE work.
