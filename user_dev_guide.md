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

> "Bench 13 result: **4,713,975 nodes** ✓"  (matches baseline — safe)
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
| **Phase 2 time-management feature** | Port it (only useful with real clocks, not our fixed-TC testing) or skip? | Plan recommends skip unless you switch to tc-based testing |

When you hit one of these, just answer the question in plain English.
The model will proceed accordingly.

---

## Prerequisites — complete these before Phase 1

Phase 0 tooling is written but not yet verified. Do these once:

- [ ] **Download fastchess** → `D:\chess\fastchess\fastchess.exe`
      https://github.com/Disservin/fastchess/releases
- [ ] **Run the calibration test** (takes ~15–30 min):
      ```powershell
      ./tools/sprt.ps1 `
          -EngineA "D:\chess\engines\rarog-v2.1.0-windows-pext-pgo-codex-work.exe" `
          -EngineB "D:\chess\engines\rarog-v2.0.2-windows-pext-pgo.exe" `
          -NameA "CW" -NameB "2.0.2"
      ```
      **Expected: H0 accepted.** These two engines are behaviour-identical.
      If it returns H1, stop and report — the harness needs investigation
      before any further results can be trusted.
- [ ] **Report the calibration result** to the model and say "Phase 0 complete,
      implement next step."

Weather-factory (for SPSA) is only needed from Phase 1 step 4 onward.
Setup instructions: `tools/spsa_configs/README.md`.

---

## Quick command reference

```powershell
# Build a named pext-PGO test binary into D:\chess\engines\test_engines\
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
cd D:\chess\weather-factory
python main.py

# Bench fingerprint check — run the release binary directly in a terminal:
#   .\target\release\rarog.exe
#   bench 13
#   quit
# Expected baseline: "Nodes searched  : 4713975"
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
- [x] fastchess installed at `D:\chess\fastchess\fastchess.exe`
- [x] Calibration test passed (H0 accepted: codex-work ≈ 2.0.2, ~10k games)

### Phase 1 — SPSA-tune existing constants
- [x] `SearchParams` struct + `src/params.rs` — commit `2b39f24`
- [x] 13 constants exposed as UCI spin options (`src/search_options.rs`)
- [x] **SPRT gate #1 (default-equivalence)** — bench 13 = **4,713,975** ✓, SPRT ~2.4k games score 49.57% LLR=-0.68 ✓ (refactor is behavior-safe)
- [x] SPSA group B (pruning/margin constants) tuned — commit `fae334a`
      (2271 iters / 72672 games; biggest movers: FutilityImproving 20→51,
      LmpImproving 25→53, SingularBetaMult 2→4, LmpBase 90→115)
- [x] **SPRT group B confirmation** — **H1 accepted** after 19,458 games. nElo +6.17 ± 4.88, LOS 99.34%.
- [x] Gate tunable options behind `--features tune` — commit `2fe6cc4`
- [x] SPSA group A unblocked: 1024ths LMR port (default-equivalent, commit `d1f60be`).
      6,478 games, score 50.98% — clearly safe, SPRT run aborted by unrelated OS restart.
      Proceeding on evidence (no regression visible; slightly positive).
- [ ] **SPSA group A** — run weather-factory with `config_lmr.json` and `--features tune` binary
- [ ] **SPRT group A confirmation** (`elo0=0 elo1=5`, `st=0.1`)

> Every SPSA group earns its own SPRT. The groups tune different behavior and
> may transfer differently to `st=0.1`. Never skip a group's SPRT or roll two
> groups into one confirmation test.

### Phase 2 — Port search features
- [~] `improvements` branch: check-aware ordering — **H0 discarded** (~11k games, LLR flat −0.5 to −0.8; use `[-3,3]` bounds for small features next time)
- [ ] ProbCut
- [ ] Extended correction history
- [ ] Multi-cut / singular refinements
- [ ] TT-cutoff / fail-low-parent history
- [ ] (Optional) Time management

### Phase 3 — Port eval terms
- [ ] `EvalParams` struct + `tune.rs` ported (all new terms disabled)
- [ ] Texel dataset built from Results.pgn / Results2.pgn
- [ ] Mobility tuned + SPRT confirmed
- [ ] King safety tuned + SPRT confirmed
- [ ] Passed pawns tuned + SPRT confirmed
- [ ] Bishop pair / rook terms / knight outposts tuned + SPRT confirmed
- [ ] Pawn threats / tempo tuned + SPRT confirmed
- [ ] Global Texel re-pass + final SPRT

### NNUE readiness (NOT a scheduled phase — guardrails only)
- [ ] Not planned, not scheduled. The only action is to keep the eval boundary
      clean *throughout Phases 1–3* (PLAN.md §10 + the eval-boundary ground rule
      below). No NNUE tasks to track.

### Release gates (after each phase)
- [ ] Little Blitzer gauntlet vs 2.0.2, Stockfish 18-2500, Basilisk 1.4.9
- [ ] CHANGELOG updated
- [ ] Version bumped, PGO asset rebuilt with `pext` + `avx2`

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
  reject it. Full guardrails in PLAN.md §10 — note NNUE is **not scheduled**;
  this is the only thing to keep in mind for it during normal HCE work.
