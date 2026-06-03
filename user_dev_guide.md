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

# SPRT — smaller feature (tighter bound)
./tools/sprt.ps1 -EngineA <new.exe> -EngineB <head.exe> -NameA "X" -NameB "Head" -Elo1 3

# SPRT — non-regression / simplification check
./tools/sprt.ps1 -Mode simplify -EngineA <clean.exe> -EngineB <head.exe> -NameA "Clean" -NameB "Head"

# SPSA tuning — see tools/spsa_configs/README.md for full setup
cd D:\chess\weather-factory
python main.py

# Bench fingerprint check (run from repo root, PowerShell)
echo "bench 13`nquit" | .\target\release\rarog.exe

# Inspect what a branch added
git log --oneline 5a8ce52..v2.1.0-codex
git diff 5a8ce52 v2.1.0-codex -- src/search.rs
```

---

## Phase progress tracker

Update this as each step is completed.

### Phase 0 — Harness ✅ (code done, verification pending)
- [x] fastchess SPRT script (`tools/sprt.ps1`)
- [x] pext-PGO build script (`tools/build_test.ps1`)
- [x] weather-factory configs (`tools/spsa_configs/`)
- [ ] fastchess installed at `D:\chess\fastchess\fastchess.exe`
- [ ] Calibration test passed (H0 accepted: codex-work ≈ 2.0.2)

### Phase 1 — SPSA-tune existing constants
- [ ] `SearchParams` struct + `src/params.rs` ported
- [ ] Constants exposed as UCI spin options
- [ ] Default-equivalence verified (bench 13 unchanged)
- [ ] SPSA group A: LMR terms tuned
- [ ] SPSA group B: pruning/margin constants tuned
- [ ] SPRT confirmation of tuned set vs codex-work head

### Phase 2 — Port search features
- [ ] `improvements` branch: check-aware ordering + SEE pruning (harness shakeout)
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

## Ground rules (keep the model honest)

- **Never accept a change without a bench-13 check first.** The model should
  always report the new fingerprint for real changes, or confirm it is
  unchanged for refactors.
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
