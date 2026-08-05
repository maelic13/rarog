# Rarog development plan

Rarog is a UCI chess engine in Rust (HCE eval, PVS/negamax search, PGO builds).
Sibling projects share methodology and data: **Basilisk** (C++, `D:/code/basilisk`),
**Hydra** (Python, `D:/code/hydra`); position corpus **Beast** (`A:\Chess\Beast\data`).

**Pruned 2026-07-11** (pre-prune history: commit `d9e0d85` and earlier). This
document keeps: the development process, the release procedure, the version
record, the lessons that must not be re-learned, and the forward plan.

---

## S1. Current state

**2.3.1 is RELEASED and the 2.4.0 cycle is OPEN.** `master`/`v2.3.1` sit at
`a5fd288`; `development` carries the Phase-10 work. Phases 7, 8 and 9 are
complete — the 9.8 boundary gauntlet
validated the cycle (+76 / +78 Elo at 1T, +194 at 4T over 2.2.0) and 2.3.1
restored PGO for the Windows ARM64 asset. Bench 13 = **5,173,540**, geomean
EBF **2.406**. The search is unchanged since 2.3.0, so that fingerprint covers
both releases.

**The clean accepted development baseline** is the post-`RootMove`,
zero-reduction-LMR, selectivity-SPSA and accepted Texel-refresh head: bench
**6,502,902**, geomean EBF **2.449**, gate binary
`rarog-p1043-base-pext-pgo.exe`. The evaluator is now frozen: the accepted
refresh remains in the baseline, but Phase 10 contains **no further HCE work**
because NNUE will replace it. Released 2.3.1 remains the release comparator;
its clean binary is `rarog-p100-base-pext-pgo.exe`, bench 5,173,540.

**Two live measurements are explicitly provisional (2026-08-05).** The active
Rating Tournament uses `Rarog 2.4.0-dev` built from `development` plus interim
constants copied from the not-yet-finished aspiration SPSA. At 5,643 / 36,400
games (~806 per engine), it reads 2955 versus 2946 for 2.3.1: only **+9 pool
Elo**, despite the earlier selectivity re-SPSA's roughly +10 logistic-Elo
expectation. That is compatible with a small gain, external-transfer loss, or
ordinary noise; it proves none of them yet. Interim SPSA values are not an
accepted vector and this mixed binary is not a release candidate. Finish and
archive both jobs, then resume from one clean, reproducible baseline.

Phase 10 has therefore been rebuilt as an evidence-coherent search program.
Its target is no longer “reach Basilisk”: Basilisk remains a same-eval measuring
instrument. The 2.4.0 release gate is to beat the complete Rybka library,
Critter 1.6a, Houdini 2.0c and Fritz 16 under registered equal-resource
conditions, before the NNUE line opens.

## S2. The development process

### The rhythm

```text
User  -> "Implement the next step of the plan."
Model -> Reads PLAN.md + GUIDE.md, implements, verifies locally
         (build, bench, cargo test, fmt), updates BOTH documents, commits,
         and hands the user EXACT commands to run.
User  -> Runs the long jobs (SPSA / SPRT / gauntlet / datagen) and pastes
         the result. The user is the only one who runs games.
Model -> Acts on the verdict: bake or revert, record in both documents, commit.
```

- **Commits:** imperative subject + detailed body; release bumps are exactly
  `Version X.Y.Z`. Never add co-author trailers. Commit docs together with the
  change they describe.
- **Both documents stay in sync** after every step: PLAN carries the record and
  rationale; the guide carries only the user's forward view.
- **SPSA results:** wait for the user to paste final values; never read
  weather-factory `state.json` mid-run. Bake the **final theta**, i.e. the
  `Final parameters` values printed by the tuner, as one whole vector. Do not
  substitute an ad-hoc tail average: final theta is SPSA's actual accumulated
  estimator, while an averaging window is a different estimator that must be
  designed and validated before the run if we ever want to use it.

### Testing methodology (the gates)

| Gate | When | Rule |
|---|---|---|
| **Bench fingerprint** | behaviour-preserving refactors | `bench 13` node total must be **identical**; any change means search/eval changed |
| **SPRT** `[0,3]` (`elo0=0 elo1=3`) | every strength claim | **The only verdict.** `tools/sprt.ps1 -Elo1 3` at `tc=3+0.03` vs the current accepted head |
| **SPRT** `[-3,0]` | non-inferiority / simplification | `-Elo0 -3 -Elo1 0`; H1 supports non-regression, H0 supports a meaningful loss. A single `[-3,+3]` SPRT is not an equivalence test. |
| **Fixed null calibration** | after harness/runner changes | byte-identical engines, 30k games; the complete 95% nElo CI must fit inside `[-5,+5]` (`-Mode calibrate`) |
| **LTC confirm** | TC-sensitive features (TM), phase boundaries | `-TC "10+0.1"` |
| **SPSA** | tuning constant groups | weather-factory via `tools/spsa.ps1 -ConfigGroup <g> -EngineSuffix <s>`; **SPSA finds candidates, SPRT decides**; final values → required adjudication calibration when the profile is under review → bake → PGO build → SPRT |
| **External gauntlet** | phase boundaries | §11-ladder at `tc=10+0.1`; self-play over-states (Phase 4: +316 staged → +240 real) |
| **CPU affinity** | EVERY clock match (SPRT/SPSA/gauntlet) | Explicit one-logical-CPU-per-physical-core affinity is mandatory. fastchess must be >=1.7.0 on Windows; never rely on its alternating-CPU auto-topology. |

- **Unified TC:** SPSA and the primary SPRT both run `tc=3+0.03` (~depth 16) so
  optima transfer. Oversubscribing logical cores drops NPS and distorts depth —
  never for clock-TC testing (node-limited datagen may use 24).
- **Affinity + concurrency (corrected 2026-07-21).** The original +9.34 ± 8.20
  null result justified investigation but did not prove a persistent +9 Elo
  offset; the old `[-3,+3]` follow-up also had zero expected LLR drift at true
  0. Concrete defects did exist: fastchess before 1.7.0 failed to apply Windows
  process affinity, while 1.8.0 auto-topology guesses SMT siblings from logical
  CPU order. The repo now detects physical cores via the OS, passes explicit CPU
  IDs, leaves two cores free by default, fails affinity warnings, and validates
  with a fixed 30k-game `-Mode calibrate` equivalence CI. Historical borderline
  decisions remain borderline on their reported uncertainty; large verdicts are
  unaffected. The affinity patch remains required, but do **not** prepare a
  fresh SPSA with an old checkout of `setup_tools.ps1`: `strength-v1` has now
  replaced its forced `600/3 twosided` patch with calibrated 600/3 one-sided.
- **Test binaries:** `tools/build_test.ps1 -Suffix <s>` → `rarog-<s>-pext-pgo.exe`
  (SPRT/gauntlet), `-Tune` → `rarog-<s>-tune.exe` (SPSA only, exposes UCI knobs),
  `-Native` (local-only znver3). PGO trains on the internal `bench` (SF-style).
- **Texel fits are minutes** — the model runs them freely; games are the user's.
- **Preserve tuned-off features across the NNUE transition.** A parameter at
  its off rail is evidence about this HCE surface, not proof that the mechanism
  is useless. Keep the implementation, UCI option, and SPSA configuration until
  NNUE is integrated and the post-NNUE tune has had a chance to reactivate it.
  Removal requires a separate explicit decision after that re-tune.

### Guiding principles (hard-won)

**Strength-first escalation rule (project mandate, 2026-08-01).** The objective
is the strongest chess engine we can build, not preserving today's constraints
or finding increasingly elaborate ways around them. Whenever the model finds
something sub-optimal, wrong, weak, badly implemented, or limited by a
constraint that may be removable, it MUST surface it to the user promptly. Do
not silently accommodate it, freeze it into the plan, reduce scope around it,
or build a workaround first. State: (1) the evidence and whether this is a
demonstrated defect or a hypothesis, (2) the likely strength/quality upside,
(3) implementation and validation cost/risk, and (4) the best direct fix plus
credible alternatives. Then stop for a joint keep/fix/defer decision before
expanding the work. Existing EV gates and SPRT discipline still decide what we
spend and ship; they must never be used as a reason not to disclose a real
improvement opportunity. Treat mutable constraints as engineering targets,
not laws.

1. **SPRT is the only verdict.** Holdout MSE, bench nodes, EBF, NPS — all are
   diagnostics. A −4.9 % holdout fit lost 17 Elo (§S4, SF-distillation).
2. **EV-gate the compute** (principle #10, after the 30 h LMR null). Before any
   multi-hour run, state which of the group's *inputs changed* since its last
   tune. No >10 h runs on EV < +3 without user sign-off. Don't re-tune
   eval-scale-independent groups after an eval change.
3. **Review SPSA output before baking** (principle #9): value pinned at a bound
   → widen once and resume; driven to no-op → inspect the implementation;
   verify each value against the live code it feeds.
4. **Fit the eval once, then tune search once.** cp-denominated search margins
   (futility/razoring/LMP/ProbCut/lazy) are re-tuned only after an eval rescale;
   one big *joint* margin tune captures the gain — the leftover per-group
   re-tunes were nulls.
5. **Behaviour-preserving steps need no games** — gate on bench identity + tests.
   New eval structure is **seeded inert** (weights 0) so it lands bench-identical
   and the next refit activates it.
6. **One candidate at a time**, always vs the current accepted head. No
   statistical fishing (don't loop SPRTs on tweaks until one passes).
7. **Revert cleanly on H0.** Keep the widened SPSA ranges and any
   infrastructure; only the baked values go back.
8. **Clean, idiomatic Rust only (2026-07-19, user decree).** No new
   violations — real (unsafe, UB-adjacent, silent-truncation casts) or
   spirit-level (sentinel values where `Option` belongs, positional scalar
   soups, suppressed lints without written justification). Standing rules:
   clippy stays at **zero warnings**; every `#[allow]` and every `unsafe`
   carries an in-code justification (a `SAFETY:`/`KEEP-UNSAFE:` note, with
   measurements where the justification is performance — see 9.0's precedent:
   27→19 unsafe sites, each measured or proven); new params/options must not
   widen existing duplication; prefer making invalid states unrepresentable
   (the `Square::index & 63` / `Option<u32>` depth patterns) over guarding
   them. Refactors gate per principle #5: bench-identical + suite, NPS
   best-of-N where the hot path is touched, SPRT only where bench can't see
   the change.

### Bench harness semantics (redesigned 2026-07-01)

`bench [depth] [repeats]`, 40 positions (identical to Basilisk/Hydra), depth 13
default, single-threaded, PGO training workload.

- **Node total = change-detector fingerprint ONLY.** Never compare magnitude
  across param changes (±1 threshold changes swing it several %; the old
  16-position suite swung 15 % on sub-1-Elo deltas). Identical = behaviour-preserving.
- **Geomean EBF = the selectivity metric** (robust to outliers; head 2.548;
  SF ≈ 1.8–2.0). Track it per accepted search change.
- **Speed = best-of-N NPS** (`bench 13 5`): single-run NPS has ~43 % machine
  noise; compare binaries only by best-of-N.
- FEN legality is guarded four ways (parser pawn-count + side-not-to-move-in-check
  validation in `from_fen`, per-suite legality unit tests, the consolidated
  `canary_integrity` sweep of *all* packaged FENs, and the PGO-log check) after
  an illegal 9-pawn position hid in the suite for weeks.

### Canary / first-gate policy (Basilisk post-mortem, 2026-07-16)

Basilisk's endgame canary failed two ways and blocked good work: (1) 4
**illegal** positions passed by coincidence; (2) a **single-position full
conversion at fixed depth** was a *hard gate* — a search-**shape trajectory**
that over-fired on benign eval/search/TT changes while the eval still saw the
win. Rarog audited clean on (1) — `from_fen` rejects illegal positions and every
suite loads through it — and the rule from (2) is now doctrine:

- **Gate correctness, never search shape.** A hard gate may assert *what the
  engine concludes* (finds the mate, recognises the win, eval is symmetric,
  reconstruction is exact, position is legal) but not *the path it took* (exact
  depth reached, exact node count as a magnitude, exact conversion ply). The
  fingerprint (`bench` node total) is the one exception, and only as a *sameness*
  check ("identical ⇒ behaviour-preserving"), never a magnitude.
- **Floors, not exact counts** for anything path-dependent (WAC solved count →
  `FLOOR`, tolerant of drift; conversion → `>= total−1`, tolerant of one fragile
  trajectory). Print the exact figure as a **diagnostic**, gate on the floor.
- Rarog canaries (all correctness-form): `canary_integrity` (every packaged FEN
  legal), `near_mate_recognition_canary` (KQK/KRK/KBNK find mate + a KQK
  stalemate trap — mates, not stalemates), `eval_invariants` colour symmetry,
  `endgames` draw/win recognition + KBNK mate, the texel `--verify`
  reconstruction, and the WAC floor. Fixed 2026-07-16: the
  `search_continues_to_resolve_shorter_mate` `depth == 18` shape assertion →
  `depth >= 9` + the (kept) mate-distance correctness checks.

---

### SPSA harness — calibration and multi-session operation

Relocated here 2026-07-28 from Phase 8, because it governs EVERY
future tune rather than the wave it was found during.


**ADJUDICATION CALIBRATION (corrected 2026-08-01; mandatory before the
10.4.6(a) gate).** weather-factory ships `-resign movecount=3 score=400`
one-sided, exactly the setting used by Reckless OpenBench for both SPSA and
ordinary strength tests. Stockfish fishtest also uses one-sided `movecount=3`,
at `score=600`. Stockfish raised 400→600 in 2022 specifically because its new
NNUE architecture inflated reported evaluations by about 50%; the threshold is
therefore engine-scale calibration, not a universal constant. Both projects
kept one-sided resignation.

One-sided is meaningful evidence: the side about to lose must itself report
at most -400 for three consecutive searches. Two-sided adds the winner's
correlated opinion; it protects against transient disagreement, but cannot
protect against a shared fortress/endgame blind spot. In the current SPSA PGN,
only 69.3% of the existing one-sided-400 adjudications also satisfied
two-sided-400 at the same ply. The remainder are games that would continue,
not demonstrated result errors. The previous claim that the missing
`twosided` flag was the worse problem, and the unsupported donated-worker
rationale, are **retracted**.

**Calibration completed 2026-08-02.** We replayed the score stream of 69,350
Rarog games completed under the stricter `600/3 twosided` rule. Fastchess PGN
scores are from the moving engine's perspective; the losing-side test is
therefore `score <= -threshold` for three consecutive searches by that side.

| candidate | triggers | changed final result | decisive reversal | mean plies saved |
|---|---:|---:|---:|---:|
| 400/3 one-sided | 36,946 | 1,533 (4.15%) | 80 | 24.5 |
| 500/3 one-sided | 35,860 | 435 (1.21%) | 17 | 10.9 |
| 600/3 one-sided | 35,486 | 74 (0.21%) | 3 | 1.6 |

All three apparent 600 reversals were later **time forfeits**, not chess-result
reversals; the other 71 games eventually drew. Thus 400 is demonstrably too
aggressive for Rarog's evaluation scale, while 600/3 one-sided is accepted as
the shared **`strength-v1` profile** for SPSA, SPRT, and gauntlets. It is
centralised in `tools/harness_common.ps1`; setup patches weather-factory from
either its upstream 400 rule or the obsolete V1 two-sided patch. Datagen keeps
its separate stricter **training-label profile** because one incorrect game
result labels many positions.

**NOT aligned to fishtest, deliberately — the draw rule.** Ours is
`movenumber=40 movecount=8 score=10` against fishtest's `movenumber=34
movecount=8 score=20`: later, and with a tighter score window, i.e. strictly
more conservative on both axes. It already agrees between `sprt.ps1` and the
tuner, so the discrepancy the fix targets does not exist there, and moving it
would shift the verdict instrument and break comparability with the entire
existing ledger for no correctness gain.

The obsolete 2026-07-30 `RAROG_ADJUDICATION_PATCH_V1` prepared `600/3
twosided`. `setup_tools.ps1` now replaces it with
`RAROG_ADJUDICATION_PATCH_V2`, 600/3 one-sided. This does not retroactively
change the completed 10.4.6(a), which correctly stayed under its original
400/3 one-sided rule. Never change termination rules inside a running SPSA.

**HARNESS DEBT — SPSA `A` is in the wrong units (found 2026-07-23, during
8.4's first night).** `spsa.ps1` writes `A = Iterations / 10`, i.e. in
ITERATIONS, but weather-factory's `t` counts GAMES (`spsa.py`:
`self.t += cutechess.games`, 32/iteration) and both schedule terms read it —
`a_t = a/(t+A)^0.601`, `c_t = c/t^0.102`. So a 6000-iteration run gets A=600
where the intended "A ≈ 10% of the run" is ~19,200: the gain decays ~32×
faster than the design implies. Measured on 8.4 at iteration 1244, `a_t` was
already at **8.2%** of its initial value (intended damping: 51%), and the
whole remaining 1244→6000 stretch only moves it 8.2% → 3.2%.

Consequences, in order of importance: (1) **judge SPSA stopping on the gain
curve, never on "% of planned iterations"** — the late budget is nearly
worthless as configured; (2) every SPSA in this project so far (7.1 history,
7.2 see, 8.6 lmr, 8.7 deeper, …) annealed far faster than intended, which is
a plausible contributor to the "converged SPSA, then H0" pattern in 8.6/8.7 —
an under-annealed *late* phase means those fits may be noisier than assumed;
(3) fixing it is a one-line change (`A = Iterations * games_per_iter / 10`)
but it CHANGES EVERY FUTURE FIT, so it must not land mid-8.4 — schedule it
after 8.4/8.5 close, and re-fit nothing retroactively without a gate. Not
worth re-running 8.4 for: the values are still fitted, just fitted under a
faster anneal.

**🔴 AND THE FIX ITSELF SHIPPED BROKEN — found 2026-07-30, before its first
use.** `spsa.ps1` wrote `"A": 0.0965450005455993` where it needed `"A": 500`,
because **PowerShell variable names are CASE-INSENSITIVE and `$A`/`$a` are one
variable**: `$A = Iterations/10` (500) was silently clobbered by
`$a = REnd · (A+N)^α / N^(2γ)` (0.0965) on the very next line, three lines below
the comment block explaining why the damping matters. `a` itself was right (it
read `$A` before the assignment landed); only the emitted `A` was wrong — and
`a_t = a/(t+A)^0.601` with A ≈ 0 is **no damping at all**, i.e. exactly the
defect this whole section exists to remove.
It was caught by a `-SetupOnly` dry run before 10.4.6(a), which is the FIRST
tune the end-state parameterization would ever have driven, **so no fit is
contaminated** — the fix was authored 2026-07-27 with no tune running and
nothing has tuned since. A 40-hour run would have annealed wrongly and looked
entirely normal doing it, because the schedule leaves no trace in the output.
Renamed to `$dampingA`/`$gainA`, and — the part that matters — `spsa.json` is now
**read back and asserted** (A equals 10 % of the horizon, A > 0, a matches), with
the launch printing `Verified: A = 500 (10% of horizon), a = 0.09655`. The
read-back hit the same footgun one layer down: `ConvertFrom-Json` refuses keys
differing only in case, so it uses `-AsHashtable` with **bracket** indexing —
property access on a hashtable is case-insensitive and would silently return the
wrong one of the two.
**Lesson, and it is the general one: a derived constant that leaves no runtime
trace must be asserted at the point it is written.** Two independent
correctness passes over this schedule (the units fix, then the fishtest
re-parameterization) both reviewed the *reasoning* and neither looked at the
*written file*.

**✅ FIXED 2026-07-27 (8.5's SPSA complete; no tune running).** The landed
fix converts `t` to iteration units inside `spsa.py::step` (`it = t /
games`), keeping `t`/`state.json` in games so old states resume correctly,
and keeping `A = Iterations/10` in `spsa.ps1` — which is now dimensionally
right. Quantified (and cross-checked against the 8.4 measurement above —
old code reproduces the 8.2%-at-iter-1244 reading exactly):

| iteration | a_t old | a_t new | ratio |
|--:|--:|--:|--:|
| 1 | 0.0207 | 0.0214 | ×1.0 |
| 100 | 0.0071 | 0.0195 | ×2.8 |
| 600 | 0.0026 | 0.0141 | ×5.4 |
| 3673 | 0.0009 | 0.0066 | ×7.4 |

Total adaptation budget over a 3,673-iteration run: **7.3 old vs 38.2 new
(×5.3)**.

**⚠ CORRECTION 2026-07-27, same day — the claim "`a=1.0` stays calibrated"
was WRONG, and shipping the schedule fix alone would have made tunes
WORSE.** For `k ≫ A/32` the old games-fed decay was simply the correct
*shape* times a constant: `(32k)^-0.601 = 32^-0.601 · k^-0.601 = 0.126 ·
k^-0.601`. So `a=1.0` under the old schedule behaved like **a ≈ 0.126**
does under the fixed one. Restoring the shape without restoring the
magnitude multiplies every step by ~8.

Quantified with a simulation **validated against the real 8.5 trajectory**
— it reproduces the observed mean tail wander of 0.42 steps to within 0.02
(model 0.44), so its noise/signal scale is right:

| schedule | a | RMSE from optimum | tail wander |
|---|--:|--:|--:|
| broken, 3,670 it (what 8.5 ran) | 1.0 | 0.53 | 0.44 steps |
| **fixed, 5,000 it — as first shipped** | **1.0** | **0.78** | **2.12 steps** |
| fixed, 5,000 it | 0.2 | 0.38 | 0.48 steps |
| **fixed, 5,000 it — adopted** | **0.1** | **0.32** | **0.24 steps** |

`a=1.0` on the fixed schedule is *worse than the bug it replaced* (0.78 vs
0.53). Swept 0.05…1.5; the minimum is at a≈0.1 and the curve is flat
between 0.1 and 0.2. **`spsa.ps1` now writes `a: 0.1`.**

So the schedule fix's real content is the **shape, not the magnitude**:
`A` now damps the first ~10% of the run as designed (it was effectively
`A/32 ≈ 19`, i.e. no damping at all), and the planned horizon now controls
the anneal instead of being decorative. The ×5.3 "budget" figure is
arithmetically right but was the wrong thing to want — more gain is only
better up to the point where noise-driven wander dominates, and ×5.3 was
far past it.

Practical readings that change: (a) "resume for another night" was
near-worthless under the old schedule (the 8.4 and 8.5 second nights moved
almost nothing — a_t was at 3–8% of initial); it is meaningful again;
(b) late-run wander bands were partly *frozen-schedule* artifacts — a third
cause besides flat-optimum and disconnected-knob for a non-monotone
trajectory; (c) **`a` and the schedule are one calibration** — never change
either without re-deriving the other against a trajectory-validated model.

**✅ STRUCTURAL FIX 2026-07-27: adopt fishtest's end-state parameterization,
so this bug class cannot recur.** Stockfish's fishtest never hand-picks
`a`. Each parameter carries `c_end` (perturbation at the END of the run)
and `r_end` (learning rate at the end), and the schedule constants are
*back-solved* from them and the planned horizon `N`:

```
c     = c_end * N^gamma            ->  c_t == c_end exactly at t = N
a_end = r_end * c_end^2
a     = a_end * (A + N)^alpha      ->  a_t == a_end exactly at t = N
A     = 0.1 * N                    (same as ours)
alpha = 0.602, gamma = 0.101       (Spall; ours 0.601/0.102 — same)
```

Because both constants are derived FROM the horizon, changing the planned
iteration count cannot silently change the end behaviour, and `a` can never
be left stale when the schedule shape changes — precisely the two failures
of this morning. `spsa.ps1` now takes `-REnd` (default 0.0031) and derives
`a = r_end · (A+N)^alpha / N^(2γ)`; verified invariant, r_end holds at
0.0033 across N = 1,000 / 2,500 / 5,000 / 10,000 while `a` moves
0.051 → 0.127.

⚠ **HOW MUCH OF THAT IS ACTUALLY IMPLEMENTED — corrected 2026-08-04, after the
description above misled a reader (me) into a wrong conclusion about a knob's
perturbation.** Only the `a` half is back-solved. **`c` is hardcoded to 1.0 in
`spsa.json` and no knob declares a `c_end`.** What a config's `step` field
actually holds is the **INITIAL** perturbation multiplier, not the final one:

```text
c_t   = c / it^gamma          with c = 1.0, gamma = 0.102
c_t(1)      = 1.0000          <- `step` is the perturbation HERE
c_t(5000)   = 1/5000^0.102 = 0.4195
perturbation(knob, it) = step × c_t(it)
```

So a knob's perturbation **falls to ~42 % of its `step` by iteration 5,000**.
The system is still internally consistent — substituting `c_end = N^-γ` into
`a = r_end · c_end² · (A+N)^α` gives exactly the shipped
`a = r_end · (A+N)^α / N^(2γ)` — so the maths is right and every completed tune
is valid. What is wrong is the *name*: `step` is not `c_end`, and reading it as
`c_end` produces the wrong answer about whether a knob stays measurable.

**The rule that falls out, and it is the practically important one:** the engine
receives `round(value)`, so a knob stops being measured once its perturbation
drops below half a unit. For an integer knob:

```text
step × c_t(N) ≥ 0.5   ⇒   step ≥ 0.5 / 0.4195 ≈ 1.19   ⇒   step ≥ 2
```

**Any integer knob with `step = 1` dies mid-run** — precisely at
`it > 2^(1/γ) = 2^9.804 ≈ 894`, i.e. 82 % of a 5,000-iteration run is spent
feeding both arms the same value while the knob keeps being *updated* by the
other knobs' gradient. That is audit class 6, and the 894 it reports is this
formula, not a heuristic. **The audit models the real schedule; this section's
prose did not. When they disagree, the audit is right.**

*(Adopting the real per-knob `c_end` form — each knob declaring its end
perturbation and `c` being back-solved per knob — would make `step` mean what
it says and remove the ≥2 rule. It is a genuine improvement and not urgent:
the audit already catches every case the current form gets wrong.)*

**Independent corroboration of the recalibration.** Converting our settings
into fishtest's units, `a=0.1` at N=5000 is r_end ≈ 0.0031 — the same order
as fishtest's 0.002 default — while the `a=1.0` shipped this morning is
r_end ≈ 0.031, roughly **15× hotter than fishtest has ever defaulted to**.
The trajectory-validated simulation and the strongest engine's production
tuner agree, from completely independent directions, that a=1.0 was far too
hot.

**Where we still differ from fishtest, deliberately:** fishtest updates
every **2 games** (one pair); we use 32. That is not a defect to copy —
fishtest is a distributed system with persistent workers and ~zero
per-iteration cost, whereas we relaunch fastchess every iteration for a measured
~4 s fixed overhead on top of ~0.77 s/game. At 2 games/iteration our
overhead would be 72% of wall-clock. A fixed-wall-clock sweep of 8/16/24/
32/48 games per iteration came back **within simulation noise**, so 32
stays — but the reason is our process model, not disagreement with Spall. **Basilisk's weather-factory has the identical bug**
(byte-same `spsa.py` schedule, A=500 in iteration units) — relay the same
one-line fix. Shared bug ⇒ it does NOT explain the Rarog–Basilisk gap.
Every accepted bake stays valid (each won a real SPRT); the cost was
unrealized upside, so re-tunes of the big groups under the fixed schedule
are a post-2.3.0 opportunity, gated as always.



### Documentation audiences (hard rule)

Every document in this repo belongs to exactly one audience, and mixing them is
a defect. Written down 2026-07-29 after the 2.3.0 README was found carrying
per-version history back to 2.0.0, a "release-preparation checks for 2.1.0"
command block, and a forty-item test inventory — all of it maintainer notes
sitting in the first thing a new user reads.

| Document | Audience | Contains |
|---|---|---|
| `README.md` | **users** | what the engine is, how to get it, how to run it, how to build it |
| `CHANGELOG.md` | **users** | what changed in each release, in their terms |
| GitHub release notes | **users** | the same, for one release |
| `PLAN.md` | maintainer | why, evidence, rejected experiments, method, forward plan |
| `GUIDE.md` | maintainer | operational state, what to run next |

**User-facing documents describe the CURRENT state and what it means for
someone using the engine.** They do not carry project history, roadmap, Elo
methodology (SPRT, nElo, self-play, gate brackets), internal type or function
names, phase/item numbers, or notes addressed to the maintainer. If a sentence
only makes sense to someone who has read PLAN.md, it belongs in PLAN.md.

Two specific traps, both hit for real:
- **Machine-specific claims.** A README example commented `# fastest build for
  this machine` is true for whoever wrote it and false for every reader.
- **Documentation that outlives its subject.** The README claimed local builds
  used `target-cpu=native` for a full release cycle after that stopped being
  true. When behaviour changes, grep the user-facing docs for claims about it.

Release notes are generated FROM the `CHANGELOG.md` entry, so the two can never
disagree — and the CHANGELOG is edited to the user standard first.

## S3. Release procedure

Release at phase boundaries that change playing strength (patch releases for
standalone fixes). CI builds and attaches all platform binaries when a GitHub
release is published — releasing is documentation + tagging. **From Phase 9.3
on, CI ships PGO binaries and smoke-tests the exact artifacts** (uci handshake
+ bench) — shipped binaries must match the configuration we SPRT locally.

Model does 1–6; **the user runs the boundary gauntlet and publishes (7–8)**:

1. **`cargo test` (DEBUG) *and* `cargo test --release`**, plus
   `cargo fmt --check` and `cargo clippy` clean; `bench 13` matches S1.
   ⚠ **Debug is not optional and release alone is not sufficient** — CI runs a
   debug × release matrix, a debug build searches ~an order of magnitude
   slower, and a CI runner is slower again. 2.3.0's first CI run failed on
   exactly this: `long_endgame_search_does_not_overflow_engine_thread_stack`
   had a flat 2 s budget for `go movetime 100`, which passed release locally
   for months and produced `seen: []` in the CI debug matrix. The debug suite
   costs ~30 s; running only `--release` is how that reached master.
1a. **Build flavours are two independent choices (reworked 2.3.0).**
   `cargo xtask build --arch {base|avx2|pext|arm64} [--native]`. `--arch` picks
   the ISA contract — which source path compiles (`--cfg rarog_pext`) and which
   features are required. `--native` only swaps the portable `target-cpu`
   baseline for the host's own, and is LOCAL ONLY; distributed assets never set
   it, and any binary that does is marked `-native` in its filename.
   ⚠ Before this, `native` was an *arch* that hardcoded the PEXT path, so a
   pre-BMI2 x86_64 host could not get a native build at all — asking for one
   emitted `_pext_u64` against a `target-cpu` that did not enable BMI2, i.e. an
   illegal-instruction binary. `--arch pext --native` now refuses up front if
   the host reports no BMI2; `--arch native` survives as a deprecated alias.
   **Gate binaries stay portable** (`build_test.ps1` runs `--arch pext --pgo`,
   no `--native`) so what we SPRT matches the shipped asset.
1a2. **Never put `rustflags` in `.cargo/config.toml`.** It applies to BUILD
   SCRIPTS and proc macros, which run on the host, and CI caches `target/` —
   so a build script compiled with `target-cpu=native` on one runner CPU is
   restored onto a different runner and dies with STATUS_ILLEGAL_INSTRUCTION.
   That failed `bench windows-x86-64` during the 2.3.0 release after passing
   on identical code one run earlier; **the intermittency is the tell**. The
   line dated from 1.0.0 and was redundant — xtask and build_test.ps1 both set
   explicit flags that override it. Removed in 2.3.0; ask for native
   explicitly via `cargo xtask build --arch native`.
1b. **Optional but cheap: dispatch CI against `development` before porting.**
   `ci.yml` carries `workflow_dispatch`, so the full matrix can be run on the
   release candidate rather than discovering failures after the master push.
2. Bump `Cargo.toml` version.
3. `CHANGELOG.md` entry at top (Added/Changed/Fixed/Removed), written to the
   **user standard above** — no SPRT/nElo/self-play vocabulary, no phase
   numbers, no internal symbol names.
4. Check `README.md` against the same standard, and specifically grep it for
   any claim about behaviour this release changed.
5. Rebuild release assets locally (`cargo xtask build --arch pext`, `--arch avx2`),
   `bench 13` each, clean `uci` handshake with no tune-only options visible.
6. Commit `Version X.Y.Z`.
7. User: run the external boundary gauntlet and paste the result; do not tag a
   phase release until transfer is acceptable.
8. User: `git push origin master`, wait for the master CI run to pass, then
   `git tag vX.Y.Z && git push origin vX.Y.Z`; publish the GitHub release and
   let the release workflow build and smoke-test its assets.

**Opponent ladder** (gauntlets, primary `tc=3+0.03`, confirm `10+0.1`): own
release history; Basilisk current release; every installed Rybka (at minimum
3/4.1/4/5/6); Critter 1.6a; Houdini 1.5a and 2.0c; Fritz 16; plus a capped
Stockfish control kept inside a useful 30–70% scoring range. Phase 10.12 is a
direct target gate, not a rating-list inference.

### S3a. Rating baseline — the measured ladder (2026-08-04)

**Live 2026-08-05 snapshot — provisional, 5,643 / 36,400 games.** This is the
most relevant current external signal, but not a verdict: each engine has only
~806 games, the pool ratings are mutually fitted, and `2.4.0-dev` includes
interim values from an unfinished aspiration SPSA.

| engine | live rating | gap over `2.4.0-dev` |
|---|--:|--:|
| Houdini 1.5a | 3207 | +252 |
| Critter 1.6a | 3181 | +226 |
| Rybka 6 / 5 / 4 | 3162 / 3138 / 3102 | +207 / +183 / +147 |
| Rybka 4.1 | 3082 | +127 |
| Basilisk 1.9.3 | 3009 | +54 |
| **Rarog 2.4.0-dev** | **2955** | — |
| **Rarog 2.3.1** | **2946** | −9 |
| Rybka 3 | 2923 | −32 |

Houdini 2.0c and Fritz 16 are not in this screenshot and must be added to the
registered target cohort. The current ordering already shows why the old
“Basilisk then NNUE” plan is too small: even the best optimistic reading leaves
roughly 150–250 pool Elo to the named pre-NNUE targets.

⛔ **The old "~3000 CCRL for 2.2.0" figure is RETRACTED. It was never measured**
— it was a hand estimate that then got chained into later reasoning as though it
were data, which is exactly the error lesson 12 warns about. Replaced by a real
anchored measurement: a 3+0.03 tournament over the full engine library, anchored
on Rybka 4's archived CCRL Blitz rating.

| engine | rating | vs Rarog 2.3.0 |
|---|--:|--:|
| Stockfish dev / 18 | 3780 / 3775 | +828 |
| Shredder 13 | 3271 | +319 |
| **Houdini 1.5a** | **3186** | **+234** |
| **Critter 1.6a** | **3176** | **+224** |
| Rybka 4.1 / 4 | 3108 / 3102 | +150 |
| **Basilisk 1.9.2** | **3005** | **+53** |
| Basilisk 1.9.1 / 1.9.0 | 3002 / 3001 | +50 |
| **Rarog 2.3.0** | **2952** | — |
| Rybka 3 | 2934 | −18 |
| Basilisk 1.8.0 | 2904 | −48 |
| Rarog 2.2.0 | 2882 | −70 |
| Rarog 2.1.0 | 2700 | −252 |
| Rarog 2.0.2 | 2625 | −327 |

This is a self-consistent internal list at one TC with one anchor; it is **not**
CCRL-comparable in absolute terms, and absolute values will shift if the pool or
anchor changes. It is exactly right for tracking progress and sizing deficits,
which is what it is used for. The 2.2.0 → 2.3.0 delta (+70) is consistent with
the 9.8 gauntlet's independently measured +76 ± 21 / +78 ± 28.

Historical HCE ceilings remain useful context, but they are **not a Phase-10
lever**. By user decision on 2026-08-05 the evaluator is frozen until NNUE; no
HCE feature, Texel refresh, scale refit or HCE-specific rescue is allowed into
the pre-NNUE search program. The implication that survives is methodological:
closing a 200+ Elo external gap requires many independently measured search
improvements and periodic target-engine transfer checks, not one global tune.

---

## S4. Version record — what was done, what it was worth

### 2.0.x (baseline)
Pre-plan engine: PVS/negamax + TT, staged move ordering, PeSTO-seeded HCE,
PGO build pipeline.

### 2.1.0 — Phases 0–2.9 (harness + search repairs + robustness)
- **Phase 0 harness:** repo-local fastchess + weather-factory, `sprt.ps1`,
  `spsa.ps1`, `build_test.ps1`; unified clock TC (3+0.03) after the old
  fixed-movetime gate manufactured false negatives.
- **Phase 1/2/2.5 search tuning:** pruning-margin SPSA group B (+6.2 nElo);
  SF-style time management (+81 at the old harness); qsearch TT-bound stand-pat
  (+6.5); per-move quiet futility (+8.0); LMR redo at clock TC (+4).
  *Rejected on the way:* codex ProbCut port (−24.5), no-aging history (−12.4,
  precondition = bonus/malus split), double-extension cap (−1.7), do-deeper
  (TC artifact, parked).
- **Phase 2.9 robustness:** time-safety valve (28 forfeits → 0; clock path
  reserves `2*MoveOverhead`; movetime path uses the full budget), native-build
  option, struct shrink, `gives_check` clone removal; close-SPRT +2.0.

### 2.2.0 — Phases 3–4 (the eval program; +316 staged self-play, +240 real)
- **Phase 3 (bench-identical build-out):** attack-map substrate; `EvalParams`
  + runtime tables + tune-time loader; **texel trace + Rust tuner**
  (`tools/texel-tuner`, K-fit/Adam/groups/L2-to-prior/bucketed holdout/
  feature-support diagnostics/nonlinear KS coordinate-descent path); 2.19M
  self-play dataset; king-safety v2 (danger funnel), threats v2, per-count
  mobility, pawn/passer detail, SF imbalance, small terms, endgame framework
  (KPK bitbase, KBNK drive, scale factors) + permanent EPD suite; eval-cache
  purity fix; **lazy eval (+4.4)**. Eval reached the full SF11-class feature list.
- **Phase 4 (staged Texel fit, every stage H1):** king-safety **+42.5**,
  threats **+45.2**, mobility **+24.1** (early-stopped at the clean-bucket
  boundary), scalars **+85.2**, imbalance **+26.7** (deliberate small OCB bucket
  regression — OCB drawishness is scaling, not material), material+PST **+27.6**,
  global polish **+65.0**. Baked via `tools/texel/bake_params.py`, bench-match
  verified. External gauntlet: **+240 over 2.1.0** (~75 % of self-play gain
  transferred). ⛔ The "~3000 CCRL" once written here was a hand estimate, never
  a measurement; 2.2.0 measures **2882** on the anchored ladder (§S3a).

### 2.3.0 — Phases 7–9 (correctness + search wave + build program)

**RELEASED 2026-07-28** after the 9.8 boundary gauntlet.
Bench 13 = **5,173,540**, geomean EBF **2.406**.

- **Phase 7 correctness ≈ +4 Elo**, but the value was removing the bug class
  that made later measurements untrustworthy.
- **Phase 8 search wave ≈ +60 Elo at 1T** (8.1 +22.13, 8.2a +30.75, 8.4 +6.01)
  **plus +102.78 at 4T** (8.13 SMP rework). Half the wave H0'd and was reverted.
- **Phase 9 build program −2…−3 Elo** (measured twice), bought the lint wall,
  pinned toolchain, CI, PGO release binaries and two real bugs. 9.7.5 returned
  net zero Elo but **+1.0…+1.6% 1T NPS** and the finding that SMP coordination
  is saturated while time allocation is the live lever.
- **Speed +11%** across the span (10.3's +10.35% plus 9.7.5's ~+1%).

### 2.3.1 — Windows ARM64 PGO patch

- Forces the pinned toolchain's `rust-lld` for `aarch64-pc-windows-msvc`,
  working around rust-lang/rust#156675 so instrumentation profiles merge.
- Restores PGO for the ninth release asset; measured locally at +8% NPS on
  Windows ARM64, with unchanged search behaviour and bench fingerprint.

### 2.3.0 development cycle — Phases 5–6
- **Phase 5 (search-constant wave):** pruning group (13 params, joint)
  **+12.07 H1** — kept. LMR re-tune −2.6 H0; futility re-tune ~0; TM re-tune ~0
  (0 forfeits) — all reverted; the joint pruning tune had captured the entire
  re-tune gain. Gauntlet + release skipped (user call; validated at the Phase-6
  boundary); version bumped untagged.
- **Phase 6.1 SF-distillation bootstrap: REJECTED, −17.11** (see lessons).
  Reverted; tuner `--from-cp`/`--fix-k`/misread-guard kept.
- **Phase 6.2 on-policy self-play refresh: REJECTED, −1.28 ± 2.79** (LLR −2.27,
  26.8k games). 500k self-play games (25k nodes, `beast_seed.epd`) → 2.18M
  unique; pure-WDL beat blended-0.7 on the shared holdout (0.075438 vs
  0.075484); `--tune all` + KS rider baked cleanly (recon 10k/10k, 160 tests,
  bench 17.6M). Lost the `[0,3]` SPRT vs the pruning head → reverted (bench back
  to 13,541,282). 6.2.0/6.2.1 pipeline + 11 inert eval params kept. Confirms
  lesson 1: even *on-policy* refresh adds ~nothing to a clean base.

---

## S5. Lessons that must not be re-learned

15. **A verified-correct search change can still lose 8–12 Elo; classify the
    failure before reacting (2026-07-15).** Three consecutive Phase-7 candidates
    — 7.0a aspiration recenter (−4.52), 7.1 draw/repetition rework
    (−7.21 / −11.91), 7.2 SEE pin awareness (−8.49) — were each verified
    *correct* (7.2: an independent legal-oracle differential showed the pin fix
    *reduced* SEE↔legal mismatches 215→200 and introduced none) and each still
    H0'd at the bound. But they are **two different failures**, told apart by
    one test — *does the change feed an SPSA-tuned constant?*
    - **De-tuning victim** (feeds a tuned constant): old constants are now
      wrong for the improved primitive; a re-SPSA of those constants *recovers*
      it, up to the change's true value. **7.0a** (aspiration delta + pruning
      group), **7.2 SEE** (`see_pruning_coeff/max` + qsearch SEE thresholds).
      Revisit only bundled with that re-SPSA — a standalone `[-3,0]` vs the
      tuned head is rigged to fail. Expected ceiling is ~neutral for a *small*
      accuracy fix, so a *dedicated* SPSA to recover it is low-EV (principle 2);
      fold it into a re-SPSA being run anyway (a Phase-8 wave / Phase-12.7).
    - **Genuine heuristic loss** (feeds *no* tunable): the loss is real and
      permanent; no SPSA recovers it. **7.1 draws** — the aggressive
      twofold-is-a-draw is simply stronger than the root-aware/FIDE variant,
      and nothing tunes that. Stays reverted; it is *resolved*, not pending.

    Cross-cutting facts: (i) all three are *search-quality* changes, not
    data-integrity (search-internal scoring, never a recorded label or an
    illegal move — "correctness prevents pollution" does not apply, SPRT is the
    direct verdict); (ii) the free wins are the bench-identical fixes that
    de-tune nothing (7.0b hang guard, 7.1a mate precedence — both kept); (iii)
    only fold a change into a *shared* gate once it is proven correct
    (oracle/differential) — an unverified change could be a genuine loss masking
    a real gain, which is the one thing bundling must never hide.
1. **SF-distillation post-mortem (−17.11, 2026-07-11).** Off-policy
   distillation gain is *inversely proportional to prior label quality*:
   Hydra +57 (pathological saturated-WDL priors), Basilisk +6.75 (skewed
   weak-head WDL), **Rarog −17 (clean staged on-policy Phase-4 fit — nothing to
   fix; the pull dragged the eval away from what wins Rarog's games).** The fit
   was textbook (holdout −4.9 %, material pinned via free-K + `--l2 1e-6`,
   9/10 buckets improved) and still lost 17 Elo. **One-shot, never re-attempt.
   Off-policy MSE is not a strength signal.** *Corollary (Phase 6.2, −1.28):*
   the **on-policy** version fails too — self-play WDL data that improves the
   holdout (0.07598→0.07544) still SPRT'd flat/negative on a clean Phase-4 base.
   Both label sources confirm: a well-fit HCE eval has no **unchanged
   representation** refit headroom left. The narrow 7.4b fit is allowed only
   because 7.4a corrects activations and adds an inert phalanx feature; it is
   not another global cycle.

   ⚠ **SHARPENED 2026-08-04 by 10.4.3, which PASSED at +11.56 ± 5.19 Elo and is
   the counterexample this lesson needed.** The rule as written above is too
   broad, and I applied the broad version to predict this gate would fail. What
   6.1 and 6.2 actually demonstrate is that **wholesale re-derivation** of a
   well-fitted eval fails — both replaced the fitted vector with a fresh global
   optimum on new labels and discarded the accumulated correctness of the
   staged Phase-4 fit. 10.4.3 did something different: anchored by `--l2 1e-6`
   to the *current* values, it moved **57 of 1,204 parameters, nearly all by a
   single centipawn**, and left everything else exactly where Phase 4 put it.
   A narrow, prior-anchored refresh is not the same operation as a refit, and it
   pays.

   **The corrected rule:** what has no headroom is *re-deriving* the eval on
   unchanged representation. What does have headroom is *refreshing* it against
   materially better labels, anchored to the existing values. The condition is
   the label generator: 10.4.3's data came from a head ~+25 nElo stronger than
   the previous fit's, and that converted into +11.56 Elo of eval.

   ⚠ **And the diagnostic reasoning that produced the wrong prediction is worth
   recording, because it will recur.** The forecast rested on two numbers, and
   both were the wrong statistic:
   - *"Holdout moved only −0.31%, less than 6.2's −0.71% which lost."* Holdout
     movement is not comparable **across fits of different kinds**. 6.2's larger
     movement came from replacing the eval; a small movement from a refinement
     is a different quantity, not a smaller amount of the same one.
   - *"Only 57 of 1,204 parameters moved, by ~1 cp."* Counting parameters is not
     effect size. A 1 cp change to `eg_val` pawn, `tempo`, or the first two
     king-safety table entries applies to nearly every position evaluated. The
     right measure is the change in the eval's **output distribution**, which
     nobody computed.
   - The signal that *was* there and got under-weighted: the per-bucket table.
     Rook endings −1.51 % and pawn endings −0.89 % against a −0.31 % global
     move is not a uniform improvement, it is a **targeted endgame** one — and
     endgame accuracy converts to Elo efficiently because small errors there
     decide games outright rather than shifting a middlegame plan.

   The next large eval lever is still NNUE (Phase 12), but "more Texel cycles
   are pointless" is now false as stated: a refresh is justified **whenever the
   label generator has materially improved**, which is a testable trigger rather
   than a blanket prohibition (see 10.4.3(c)).
2. **The 30 h LMR null → EV-gate.** LMR lives in depth/move-index space —
   eval rescales don't move its optimum. Re-tuning it after Phase 4 was
   negative-EV compute. Futility/TM nulls confirmed: after one joint margin
   tune, per-group re-tunes are dead.
3. **WDL-label saturation (Hydra).** SF *WDL-expectation* labels saturate
   (26.6 % at 0/1) → no magnitude gradient → material inflation (Q 900→1308).
   Raw cp + scale anchor fixes it. Basilisk's direct SF-WDL fit: −16.6.
4. **MSE under-values nonlinear/dynamic terms (Hydra).** KS fit moved MSE ~1 %
   but SPRT'd +19.6; MSE made scale/winnable *worse*. Nonlinear groups need the
   re-eval (finite-difference / coordinate-descent) path and the SPRT verdict.
5. **Datagen book diversity (Basilisk).** Fixed-node self-play from a tiny
   book collapses: 200k games from `SuperGM_4mvs.pgn` → **31,880 unique**
   positions; from `beast_seed.epd` → 1.73M. Always seed datagen from the
   diverse EPD book.
6. **Bench is a fingerprint, not a metric** (§S2). The 15 %-swing panic and the
   51 %-swing artifact both dissolved under analysis; the +12-Elo candidate had
   a "worse-looking" node count.
7. **Contempt cancels in self-play** — validate by gauntlet only.
8. **SPSA resume does not reload the repo config** — a widened range must be
   loaded fresh (delete/archive `tuner/state.json`), or the pin persists
   (`SingularBetaMult` stayed clipped at 6).
9. **⏹ RETIRED 2026-07-25 — Little Blitzer is out of scope** (user decision:
   it is old and Colosseum supersedes all of its functionality). The lesson it
   taught survives in general form and is what to keep: **a harness can
   mis-measure time in ways the engine cannot see, so validate any forfeit fix
   in the harness you actually compete in, not only the one you develop in.**
   Colosseum is now that harness, alongside fastchess for SPRT/SPSA. Historical
   note for reading old entries: LB mis-measured time for some engines via a
   GUI-side clock, which is why several 2.1.x/2.2.x forfeit investigations quote
   LB numbers that fastchess could not reproduce.
10. **Shallow-depth eval reads run hot after refits** (Basilisk: +122@d5 →
    +50@d12); sanity-test bounds may need widening, not the eval reverting.
11. **Green self-consistency suites hide correlated omissions (Codex 5.6
    audit, 2026-07-13).** 160 tests passed while four P0/P1 bugs shipped:
    rule-50 draw overriding checkmate, null moves advancing the halfmove
    clock, repetition that is neither root- nor null-aware, and SEE counting
    absolutely pinned recapturers — the SEE test compared `see_ge()` to
    `see()`, two implementations sharing the same bug. Verification needs
    independent oracles (slow legal-exchange SEE, external perft) and
    randomized state-reconstruction walks, not more mirror tests. All four
    bugs were independently reproduced at head before planning the fixes.
12. **Audits describe the revision they read — re-verify every claim at the
    served head (hce-audit merge, 2026-07-14).** `hce_analysis.md` was cut at
    `ff21dc1`, *inside* the later-reverted 6.2.2 refit: its "negative
    whole-path passer coefficients" (§9) and several king-safety numbers
    describe values that no longer exist at head (the three path params are
    0/inert; the safety table starts at 29, not 51; `space_piece_mg` is 0).
    Two of its consequence claims also failed live verification: restricted
    squares are *immune* to the `attacked2` pawn defect (`pawn_attacks[ti]`
    already grants strong protection at `eval.rs:1736`), and in-check nodes
    are already excluded from correction updates (`static_eval ==
    VALUE_NONE`). The five headline defects all verified live. Claims enter
    the plan only after re-verification at head, with stale numbers corrected
    and Elo priors tempered accordingly.
14. **The whole draw/repetition rework was anti-Elo; only the free
    mate-precedence fix survived (7.1, 2026-07-15).** Two `[-3,3]` SPRTs:
    the full bundle (rule-50 mate precedence + null clock + cross-null
    repetition fence + root-aware repetition) lost **−7.21 ± 6.03** (5,396
    games); removing the root-aware part left the null-clock + fence fixes at
    **−11.91 ± 7.67** (3,210 games). The two runs are statistically
    consistent with a single ~−9 regression that lives in the null-clock /
    repetition-scan changes, *not* in the piece I first blamed (my bench-based
    decomposition was blind to it — bench positions carry no game history, so
    only games see these effects). **Two ways I was misled:** (i) reasoning
    that standard "FIDE/SF-correct" fixes must be Elo-neutral — they aren't
    when they perturb the common-case repetition/null path to fix
    astronomically-rare cases; (ii) attributing a multi-part regression from a
    bench delta that can't observe the responsible behavior. Reverted all of
    (b)/(c)/(d) to the p5 head; kept only (a) `is_rule50_draw()`
    (bench-identical, fires ~never, ships without an SPRT). Lessons: a
    repetition-rule change is a *strength candidate*, not a correctness
    repair; "the audit called it a bug" is not the verdict (lesson 1); and
    decompose a failed multi-part change with **games**, not a fingerprint
    that is blind to the mechanism.
13. **Correctness fixes must be minimal-diff — window/search dynamics are
    tuned-in (7.0, 2026-07-15).** The aspiration hang had two available
    fixes: the SF-standard "re-center the widened window on the failing
    score" (changes every fail-high/low re-search) and a narrow termination
    guard (full-open only on mate-magnitude scores or delta saturation).
    The SF-style variant lost **−4.52 ± 4.80 (H0 at the bound, 8,604
    games)** — `AspirationDelta` and the whole pruning group were SPSA'd
    around the old re-centering dynamics, so "modernizing" the mechanism
    silently un-tuned them. The narrow guard is bench-identical
    (13,541,282) and fixes the hang with no games needed. When a bug fix
    can be written as "old behavior + escape hatch in the broken case
    only", write it that way; a mechanism change is a *strength candidate*
    and competes on `[0,3]` terms, not as a repair.

---

## S6. Forward plan

Reworked 2026-07-14 around the Codex 5.6 audit (`analysis/*.md`; every
demonstrated bug independently reproduced at head before planning). Second
pass same day: `search_analysis.md` fully mined — its findings were verified
line-by-line at head (all confirmed, incl. a live repro of the rule-50/mate
defect and two additional bugs: the TM `falling_eval` ordering and the
`lmr_shallow_tt` doc/condition mismatch) and merged below. Third pass same
day: `hce_analysis.md` mined with every claim re-verified at the
*post-revert* head (lesson 12) — the five headline eval defects all
confirmed live, the stale fitted-value tables (cut inside the rejected
6.2.2 refit) corrected before planning, and two of its consequence claims
disproven. **Fourth pass (2026-07-15, after lesson 15):** three consecutive
verified-correct standalone fixes lost 8–12 Elo each by de-tuning the
SPSA-tuned search, so the correctness phase was rebuilt around one rule —
**a fix ships either free (bench-identical / dead-rare path) or bundled with
the re-tune of the constants it feeds; a standalone gate against the tuned
head is rigged and is no longer run.**

The macro-order (user mandate, 2026-07-15):

- **A. Correctness program — Phase 7.** Every remaining *demonstrated
  defect*, packaged as lesson-15-compliant bundles. Queue within the phase:
  **7.6 (diagnostics, no games) → 7.4 (HCE fix+refit bundle) → 7.2 (SEE
  bundle) → 7.5 (TM fix) → done.** 7.3 verified as net-negative-EV and
  parked (→ Phase 14).
- **B. Strength program — Phases 8 → 9 → 10.** Mechanism+SPSA waves (each
  item carries its own tuning — the fair class), the 2.3.0 release, then the
  evidence-coherent search/target-ladder program and 2.4.0. Phase 10 freezes
  HCE and must beat the registered legacy targets before it closes.
- **C. NNUE program — Phases 11 → 12.** Phase 11 *is* the NNUE infra
  preparation (frozen measurement corpus, StateInfo/dirty-piece/accumulator
  scaffolding), immediately followed by the Phase-12 net_trainer program.
- **D. Contingent HCE deepening — Phase 13, standalone and last.** Entered
  only if the NNUE program fails its gates or stalls; may never run.
  Phase 14 stays parked.

Models given as *driver; implementer* (Fable 5 alternates: Opus 4.8). NNUE
is the primary evaluation-strength path.

### Roadmap: pre-NNUE program, releases, and the NNUE cutoff (2026-07-16)

The entire pre-NNUE program is deliberately **NNUE-durable** — correctness,
search, speed, infrastructure. NNUE swaps the *eval function* only, so none of
this is invalidated by it; the one thing NNUE subsumes is HCE eval-feature
*strength* work, which is therefore deferred to the contingent Phase 13 (may
never run). Linear order and the exact release / cutoff points:

| Step | What | Release / cutoff |
|---|---|---|
| **Phase 7** | Correctness bugs — 7.6 ✅, 7.4 ✅, 7.2 SEE ✅, 7.5 TM ✅ — **COMPLETE** | — |
| **Phase 8** | Search wave — accepted/rejected arms and their measured results are preserved below and in git history — **COMPLETE** | — |
| **Phase 9** | Reproducible builds + CI + shipped PGO; clean-code P1 (9.0a) + P2 (9.0b) — **COMPLETE** | — |
| **9.7.5** | SMP quality wave II — threading follow-ups from 8.13 (added 2026-07-25; deployment is now 1T **and** 4T) — ✅ **net zero Elo, +1.0…+1.6% 1T NPS** | — |
| **9.8** | External boundary gauntlet ✅ — +76 / +78 at 1T, +194 at 4T over 2.2.0 | ✅ **RELEASED 2.3.0** (+ 2.3.1 ARM64 PGO patch) |
| **Phase 10** | Evidence provenance → selective-search repairs → learning/selectivity/root/SMP symbiosis → one final tune; evaluator frozen — **▶ CURRENT** | — |
| **10.12** | Hard 1T/4T target ladder: every Rybka in the library, Critter 1.6a, Houdini 2.0c, Fritz 16; cumulative regression/release matrix | **▶ RELEASE 2.4.0** |
| **━━ NNUE CUTOFF ━━** | Everything above survives NNUE; **no standalone HCE-eval strength before here** | |
| **Phase 11** | NNUE infra prep — per-ply StateInfo, dirty-piece, accumulator scaffolding, frozen corpus + clean-code P3 (11.0 structure era) | — |
| **Phase 12** | NNUE program — contract → data → king buckets → material path → scaling → search re-SPSA | **▶ RELEASE 2.5.0** — first shipped net (+later net bumps) |
| **Phase 13** | Contingent HCE deepening — **only if the NNUE program fails/stalls**; every NNUE-subsumed eval idea lives here | — |

**Supersession note (2026-08-05).** The old Phase-10 menu and rescheduling
paragraphs are historical, not a forward queue. The current 10.0–10.12 program
below absorbs aspiration, TM, qsearch, selectivity and speed work only through
their new evidence/symbiosis contracts. HCE items remain frozen; draw rework
and rule-50 TT work remain rejected/parked unless new diagnostics justify a
separate proposal.

**Renumbering map:** old 6.2/6.3 are closed (→ §S4). The history bonus/malus
split implemented as "Phase 7.1" (commits `fe5810a`/`e0362b8`) is now
**8.1**; pre-audit 7.2–7.6 became first-pass 8.2–8.6; old Phase 8 (speed +
menu) → Phase 10; old Phase 9 (NNUE) → Phase 12. Search-audit merge
(2026-07-14 second pass): first-pass 8.2 (correction margins) folded into
new **8.5**; 8.3→**8.6**; 8.4→**8.7**; 8.5→**8.8**; 8.6 (aspiration)→
**10.2**; first-pass 10.1→**10.3**; 10.2→**10.4**; 10.3→**10.5**. New items:
  7.4/7.5, 8.2/8.3/8.4/8.9, 10.1, SMP cluster (parked). HCE-audit merge
(2026-07-14 third pass): new **7.4** (HCE semantics/refit/OCB), the previous
7.4/7.5 became **7.5/7.6**, 8.5 gains the
correction-blend weight exposure, new **9.6** (eval-measurement lite; old
9.6/9.7 → **9.7/9.8**), new **11.1** (frozen diagnostic corpus; old
11.1–11.4 → **11.2–11.5**), Phase-12 staging ladder, new contingent
**Phase 13 — HCE deepening fallback after a failed/stalled NNUE program**;
old Phase 13 (parked + SMP) → **Phase 14**. Fourth pass (2026-07-15):
item *numbers* frozen (history references them); only order/status changed —
7.3 → **parked (Phase 14)**; 7.4a+7.4b → **one gated bundle**; 7.5 escalates
to 10.2b on H0; 8.8 → **10.4 menu**; Phase-7 queue reordered
**7.6 → 7.4 → 7.2 → 7.5**. Fifth pass (2026-07-19): **no renumbering** — the
  freeze held. **Phase-10 numbers were explicitly unfrozen and rewritten by
  user authorization on 2026-08-05; git history preserves the superseded
  ledger.** New **9.0** (unsafe-surface cleanup) inserted *ahead* of 9.1
using the `.0` convention 7.0 already established, precisely so 9.1–9.8 keep
the numbers commits and history cite. 8.2 split into gated sub-items
**(a) ✅ / (b) ⏸ deferred to 8.9 / (c)** rather than new top-level numbers.

### Phase 7 — Correctness program — ✅ COMPLETE

**Worth ≈ +4 Elo; the point was banked correctness**, which removed the class
of bugs that made later measurements untrustworthy.
- ✅ 7.0 aspiration hang (minimal guard; invariant pinned by a `tests/wac.rs`
  test that 10.2(a) must keep satisfying when it replaces the widening loop).
- ❌ **7.1 draw-semantics rework −7.2 / −11.9, reverted. Do not retry**; the
  repetition-state plumbing it needed is why 10.4's cuckoo item carries a
  negative prior.
- ✅ 7.2 SEE pin-awareness bundle **+1.47** (a more accurate SEE de-tuned its
  consumers first — the origin of lesson 15).
- ⏸ 7.3 rule-50 TT key **parked → Phase 14** (its prereqs were rejected).
- ✅ 7.4 HCE semantics bundle **flat −0.31** over 8,900 games, accepted as
  correctness. ✅ 7.5 TM falling-eval **+2.85 LTC**. ✅ 7.6 diag counters.

### Phase 8 — Search wave — ✅ COMPLETE

**Worth ≈ +60 Elo at 1T plus +102.78 at 4T.** Half the wave H0'd, as the EV
line predicted. Accepted: 8.1 history bonus/malus split **+22.13**; 8.2(a)
unconditional in-check extension **+30.75**; 8.4 history bundle **+6.01**;
8.12 index hoist **+0.98% NPS**; 8.13 SMP rework **+102.78 @4T**.

**Rejections, with the reason each must not be retried as-is:**
- ❌ **8.1b no-aging −6.6.**
- ❌ **8.6 cutoffCnt + LMR re-tune −7.78.** The self-play over-aggression trap:
  the tuned candidate searched 16% more aggressively, won its own SPSA, and
  lost to the more *accurate* baseline. **Gate against the accepted head, never
  against a sibling of the tuning run.** Basilisk never validated cutoffCnt
  either; this is the ecosystem's only real test of it and it is negative.
- ❌ **8.7 do-deeper −7.29.** Searched FEWER nodes and still lost, so not a
  speed/accuracy trade — the distortion hurts move quality. Harness ruled out
  by a null pair (−0.81 ± 9.09). `DeeperMargin` gets one seat in 10.4.6's tune;
  do-deeper re-enters only if the re-fit pulls it decisively off its rail.
- ❌ **8.10 mop-up gating ≈−5.4.** Semantically right, but the ungated geometry
  was load-bearing for our eval (lesson 15, 4th instance). The term disappears
  in the NNUE era; do not retry.
- ❌ **8.11 fail-soft qsearch −5.96.** Mechanically traced to the pruning group
  having been SPSA-fitted against fail-hard's inflated bounds (+14.4% nodes via
  `eval_for_pruning`). `EvalPruneTtMinDepth` was built for exactly this
  coupling; **the retry lives in 10.4.6(a)** and closes 8.11 either way.
- ⬛ **8.5 correction-history: wash (+1.43), knobs reverted to neutral, code
  parked.** Its SPSA pinned `CorrGuardCapture=1`, and that guard discards
  **59.7%** of all correction training — so 117k games fitted eight knobs to a
  crippled signal and the gate lost **−55.98**. The group has never had a valid
  tune; it gets one in 10.4.6(a). See §S5 for the lesson.

⚠ **8.13's +102.78 was measured vs the ORIGINAL SMP (variant 0) as a
five-change bundle and was never decomposed.** Do not treat membership in it
as evidence that any one component (root rotation, stop voting, pooled root
scores) is individually positive — that inference is what cost 8.5 its gate.

### Phase 9 — Reproducible builds, CI, shipped PGO, clean code — ✅ COMPLETE (2.3.0 RELEASED)

**9.0–9.7 (no games): cost ≈ −2 to −3 Elo, and that is a measured fact, not an
estimate** — its `[-3,3]` gate read +1.20 ± 3.84 but sat inside the ~+4–5
candidate-slot inflation the null pairs later exposed, and 10.3's decomposition
independently put the refactors at −3.2% NPS ≈ −2.2 Elo. Two methods agreeing
on ≈ −2 to −3. It bought the lint wall, a pinned toolchain, CI, PGO release
builds, `Board::check_consistency`, build manifests, and the NNUE-ready
structure — plus two real bugs. **"Equivalence-or-better" was wrong.**
⚠ **Methodological lesson (the reason this is still written down):
bench-identity does NOT imply NPS-identity.** Every 9.x step was verified
bench-identical and spot-checked "NPS neutral", but a ~0.5%-per-step regression
is invisible at the ±3% noise of a single comparison, and eight compounded to
3.2%. **A refactor program needs ONE end-to-end NPS measurement against its own
starting point, not per-step spot checks.**

**9.7.5 SMP quality wave II — ✅ COMPLETE 2026-07-28, net Elo ZERO.**
Accepted: (a) duplicated-compute audit **+0.80% NPS**, (g) `unpack_root_score`
cleanup. Closed by measurement: (b) diagnostics, (c) node-to-quality, (d)
helper TT policy, (h) 16T efficiency. Rejected and reverted: **(f) symmetric
stop vote at 2T −15.85**, **(j) pool-view instability TM −5.54**, (m) ordering
jitter. Wash and reverted: **(l) helper history blending −0.52**. Closed
without code: (i) depth-weighted votes. Unresolved by design: (e) root rotation
(switch deleted, rotation kept).

**What the wave actually bought — this is the part that steers future work:**
1. **SMP coordination and diversification are SATURATED.** The shared TT
   couples the threads regardless (main's hit rate rises 53.0% → 64.9% from 1T
   to 16T), so rotation, jitter and history-sharing are *substitutes*, not
   additive contributors. (c) closed on 28,362 games at −0.81 ± 2.55; (l)
   washed because helper tables are largely REDUNDANT with main's. **Do not
   mine this area again** — including the obvious escalation of (l) to
   `cont_history`/`pawn_history`, which inherits the same prior at multi-MB cost.
2. **TIME ALLOCATION is the live lever, worth ~16 Elo.** (f) proved it by
   getting the sign wrong: stopping on the first of two votes takes `min` of the
   expiry times, a downward-biased estimator. The soft target is a heuristic
   ESTIMATE, not a budget — `maximum_ms` is the real constraint. **This points
   the next SMP investment at 10.2's root-informed TM.** The cheap probe is one
   `[0,3]` at Threads=4 with the stop threshold raised to 4-of-4.
3. **16T's deficit is not ours.** L3-resident hash 78% eff vs DRAM 75% (memory
   ≈3%), clock flat (turbo ≈0%), thread placement ≈6%. The TT is exonerated —
   cache-line aligned, prefetched at 4 sites, node counting batched at 128.
   ⛔ Thread/core pinning is OUT OF SCOPE outside SPSA/SPRT (user decision,
   permanent); the placement figure is diagnostic evidence only.
4. **Two latent bugs found:** `diag::reset()`/`dump()` sat in `search_root`,
   which helpers also call — so every multi-thread diag number this engine had
   ever printed was junk; and `--features diag` had not compiled since 8.12.


**9.8 boundary gauntlet — ✅ COMPLETE 2026-07-28/29, and it validated the
cycle.** Four Colosseum conditions, ~19,000 games, one anchor (Rybka 4 = 3102)
with every other engine free. **Zero time forfeits in every condition**, which
discharges the LTC 4T sanity item owed since 8.13.

| condition | 2.3.0 − 2.2.0 |
|---|--:|
| 1T 3+0.03 (10,402 games) | **+76 ± 21** |
| 1T 10+0.1 | **+78 ± 28** |
| 4T 10+0.1 (4,468 games) | **+194 ± 24** |

Self-play predicted ~+60 at 1T; the gauntlet returned +76 and +78 in two
independent conditions. **The gains transfer** — which is the whole reason 9.8
exists, given Phase 6.1 (−17.11) is the standing proof self-play can be
confidently wrong, and Rarog had had no external validation since 2.2.0. The
4T figure decomposes as ~+76 (1T work) + ~+103 (8.13's SMP rework, which only
applies at 4T) = ~+179 against +194 measured, so **8.13's +102.78 is externally
confirmed too**.

**Cross-engine reference point (Basilisk 1.9.1, a same-eval same-speed sibling
— see 10.0 for why that matters):** −55 ± 21 at 1T 3+0.03, −38 ± 27 at 1T
10+0.1, −32 ± 50 at 4T 3+0.03, **+34 ± 24 at 4T 10+0.1**. Also confirmed
Basilisk 1.9.1 − 1.9.0 = −6 ± 21, i.e. that C++23 cleanup release was
strength-neutral as its author expected.

Per-item detail is in git history (commits `cfb75ba`…`d11fbe9`).

**1T NPS CONFIRMATION — ✅ DISCHARGED 2026-07-28. Net +1.0…+1.6%, no
regression.** This was owed since 9.7.5 opened, and it exists because of 9.0's
lesson: per-step "NPS neutral" spot checks are invisible to a ~0.5%-per-step
regression, and eight of them compounded to −3.2%. **A change program needs ONE
end-to-end measurement against its own starting point.** This is that
measurement for 9.7.5.

Current HEAD vs `24c6b9c` (the last commit before any 9.7.5 code), built in a
clean worktree so both trees were pristine. **Both bench 5,173,540**, so 9.7.5
changed no 1T search behaviour at all and this isolates execution speed —
the same clean property that made 10.3's gate the project's best speed→Elo
datapoint.

| pass | cycles | median Δ | 95% bootstrap CI |
|---|--:|--:|---|
| self-pair (identical source, 2 builds) | 6 | +0.10% | −3.32 … +3.57 |
| first | 24 | **+0.99%** | −0.12 … +1.88 |
| confirmation (independent) | 40 | **+1.56%** | **+0.90 … +2.13** |

2 pooled PGO builds per arm, `bench 13` × 3 repeats, interleaved with
alternating order per cycle. **Every pre-9.7.5 build sat below every HEAD build
in both passes (8/8 separations)** — stronger evidence than the pooled CI alone,
which is conservative because it ignores that structure. Absolute NPS drifted
up between passes (base 3.155M → 3.185M), which is why only within-pass
interleaved comparison is trusted; the two passes agree on sign and roughly on
magnitude.

At the 10.3 constant (≈2 Elo per 1% NPS at STC) this is **≈ +2 to +3 Elo at
1T**. It is consistent with 9.7.5(a)'s claimed +0.80% from the four hoists,
plus a little more that is most likely codegen from the (e) field removal and
the (f) `votes_needed` extraction — both behaviour-identical, so no other
explanation is needed.



### Phase 10 — Evidence-coherent search program and target ladder (→ 2.4.0)

**Objective.** Turn the accepted post-2.3.1 search into a coherent decision
system, then prove that the cumulative result beats **every Rybka**, **Critter
1.6a**, **Houdini 2.0c**, and **Fritz 16** under the registered rating
conditions. Basilisk is now only the first rung. Passing Basilisk, or a
self-play SPRT by itself, cannot close this phase.

**Scope freeze.** Phase 10 may change search, move ordering, root control,
time management, SMP coordination and search-facing state. It does **not** add,
retune or deepen HCE terms. The accepted HCE refresh remains part of the
baseline, but evaluator development is frozen because NNUE replaces it in
Phase 12. Search changes must remain evaluator-agnostic where practical.

**Reality check (2026-08-05).** The current Rating Tournament is informative,
not a gate: `Rarog 2.4.0-dev` contains the accepted development head plus an
interim vector from an unfinished aspiration SPSA. At 5,643/36,400 games it is
2955 versus 2946 for 2.3.1, a provisional +9 pool Elo. That is compatible with
the expected ≈10 Elo improvement and with zero; do not attribute it until the
run finishes. The same snapshot leaves gaps of +54 to Basilisk 1.9.3, +127 to
Rybka 4.1, +147 to Rybka 4, +183 to Rybka 5, +207 to Rybka 6, +226 to Critter
1.6a and +252 to Houdini 1.5a. Houdini 2.0c and Fritz 16 are absent and must be
added to the release ladder.

Closing a 150–250 Elo external gap with search alone is an aggressive target;
there is no honest additive-Elo forecast that guarantees it. The rationale for
this order is that the largest plausible upside is multiplicative: better TT
evidence changes NMP/ProbCut/IIR/singularity decisions; better attribution then
makes history/correction and selectivity fit meaningful; root confidence lets
the accepted search spend time on the resulting uncertainty. If the cumulative
checkpoint is still below a target, Phase 10 stays open rather than relabeling
partial progress as 2.4.0.

#### Banked Phase-10 baseline

| Work already accepted | Result / consequence |
|---|---|
| Search-accuracy decomposition | Evaluation and speed did not explain the Basilisk gap; Rarog was too selective for the quality of the resulting decisions. |
| 10.4.6 search refit | +15.33 ±7.34 nElo; accepted broader tree. |
| 10.2.5(a) zero-reduction LMR | +9.13 ±5.45 nElo; accepted. |
| 10.1 `RootMove` records | Infrastructure retained, but its signals are not yet jointly consumed by aspiration and TM. |
| 10.4.3 Texel refresh | +11.56 ±5.19 Elo; accepted baseline, now frozen with all other HCE work. |
| Clean accepted baseline | Fingerprint **6,502,902 / EBF 2.449**; `rarog-p1043-base-pext-pgo.exe`. |

These results are real, but they are not additive proof of the present dev
binary's rating. The current tournament and unfinished SPSA are the first
cumulative check after the larger refit.

#### What the cross-engine review changes

Stockfish and Reckless are **design references, not code or constant donors**.
For every borrowed idea, first state the invariant it protects, implement the
smallest Rarog-native mechanism, and tune/test locally. Current-source review
was pinned to Stockfish `762dd1da9a5db458180b2c5db6c53dc40ec61e1a` and
Reckless `d6603046e76d66edd43622ded23458da1af50c68`; re-check their current code
when an item starts.

Permanent review links: [Stockfish `search.cpp`](https://github.com/official-stockfish/Stockfish/blob/762dd1da9a5db458180b2c5db6c53dc40ec61e1a/src/search.cpp),
[Reckless `search.rs`](https://github.com/codedeliveryservice/Reckless/blob/d6603046e76d66edd43622ded23458da1af50c68/src/search.rs),
[Reckless `history.rs`](https://github.com/codedeliveryservice/Reckless/blob/d6603046e76d66edd43622ded23458da1af50c68/src/history.rs),
[Reckless `time.rs`](https://github.com/codedeliveryservice/Reckless/blob/d6603046e76d66edd43622ded23458da1af50c68/src/time.rs),
Stockfish's [stand-pat TT fix](https://github.com/official-stockfish/Stockfish/commit/bb4eb04a),
[PV-IIR fix](https://github.com/official-stockfish/Stockfish/commit/e20ef7ed)
and [TT-bound mismatch penalty](https://github.com/official-stockfish/Stockfish/commit/319d61ef),
plus the historical TalkChess discussions of
[null-move/TT provenance](https://talkchess.com/viewtopic.php?t=33679) and
[`lmrDepth`](https://talkchess.com/viewtopic.php?t=63521). Forum material is a
hypothesis source only; current code and Rarog measurements decide.

| Area | Rarog now | Useful reference pattern | Phase-10 consequence |
|---|---|---|---|
| TT meaning | A depth/bound/PV entry has no source provenance. Depth-0 qsearch and reduced or speculative results can look authoritative to later pruning/extension consumers. | Both mature engines constrain consumers by node state, bound, depth and search context; Stockfish has separately fixed stand-pat TT pollution and PV IIR behaviour. | Add explicit evidence/provenance and consumer contracts before more tuning. |
| Qsearch → main search | Qsearch may store a pruning eval as a bound; `EvalPruneTtMinDepth=0` lets depth-0 data influence RFP, razor, NMP and ProbCut through depth 8. | Stockfish's stand-pat path avoids manufacturing a TT bound when there was no searched move. | Stop laundering static/pruning estimates into searched TT evidence. |
| ProbCut → singular/TT | ProbCut stores a margin-normalized score at `depth-3`; singular extension can accept lower/exact TT evidence at `depth-3`. | Reckless stores the searched ProbCut score and chooses its depth dynamically. | Tag speculative cutoff evidence; never let it trigger singularity or masquerade as an exact search. |
| NMP | Verification only disables null at its root; descendants can null again. There is no subtree suppression, cut-node gate, potential-singularity guard, raw-eval gate or decisive-score guard. | Stockfish/Reckless use subtree `nmpMinPly`, node-role/eval guards, and stronger verification discipline. | Correct the verification contract before tuning null margins. |
| IIR / singularity | IIR can reduce PV nodes with no TT move; singular starts at depth 4 and can double-extend from weak TT evidence. | Stockfish fixed PV IIR and now applies IIR more selectively; both references tightly couple singular tests to trustworthy TT evidence. | Make node role and provenance explicit; instrument extension debt. |
| Global `tt_pv` veto | RFP, razor, NMP and ProbCut are all disabled when `tt_pv` is set, even though those mechanisms have different safety requirements. | Mature engines gate each mechanism on its own node/evidence contract. | Replace the global veto with per-mechanism predicates. |
| LMR/selectivity | Reduction affects search depth, but later futility and SEE decisions do not consistently use the prospective reduced depth; quiet checks bypass several mechanisms via a fixed 32,000 bonus. The prior LMR re-search rate was only ≈1.8%. | Stockfish uses `lmrDepth`, history/stat score and reduction feedback across later decisions. | Build one prospective-depth pipeline and calibrate checks inside it. |
| In-check search | Documentation says late evasions are reducible, while the active LMR condition still contains `!in_check`. | Both references treat evasions by evidence and ordering rather than a blanket label. | Resolve the code/document mismatch and test safe late-evasion reduction. |
| Histories/correction | Continuation correction is only the previous `(piece,to)` table; the capture-update guard is tuned off, and prior diagnostics found ≈52.8% of correction updates capture-caused. | Stockfish/Reckless use true continuation pairs; both separate quiet/noisy context more carefully. Reckless also indexes threat context and halfmove-clock buckets. | Repair attribution before increasing history authority. |
| Root control | `RootMove` records average, mean-square, PV, nodes and fail counts, but aspiration/TM barely consume those per-move signals. | Both references use root variance/stability; Stockfish also uses effort, score trends, best-move age and pooled best-move changes. | Make aspiration, TM and SMP share one root-confidence model. |
| Aborted search | Fallback behaviour can expose stale or unstable root information near time limits, including decisive scores. | Reckless has explicit aborted-mate/loss handling; mature root loops preserve last completed evidence. | Define completed-iteration ownership and legal fallback invariants. |
| Parallel search | Shared TT already couples workers; prior diversification work was saturated. | Stockfish pools root instability across threads rather than treating workers as independent votes. | Spend SMP effort on pooled confidence and timing, not more random diversity. |

#### Cross-feature invariants

These are hard design constraints, not optional polish:

1. **A score is not evidence by itself.** Every consumer must know whether it
   came from a full search, qsearch move, stand pat, ProbCut, null move,
   singular verification, reduced search or an aborted iteration.
2. **Depth is prospective.** LMR, futility, SEE pruning, extensions and history
   updates must reason about the depth that the move will actually receive,
   not unrelated pre-reduction depths.
3. **Attribution precedes adaptation.** Correction/history updates are allowed
   only when the searched result can reasonably be attributed to that feature
   context; captures, null moves, speculative cutoffs and aborted work cannot
   silently train quiet-position tables.
4. **Root confidence is shared.** Aspiration width, time allocation, fallback
   and SMP instability consume the same completed-iteration statistics.
5. **A mechanism change precedes its constant fit.** No broad search SPSA may
   compensate for evidence bugs or disconnected consumers. Refit only after
   the mechanisms are stable.
6. **One upstream change, all downstream consumers audited.** Any TT, history,
   depth or root-stat change must list every reader and prove its contract.

#### Execution and evidence policy

- One implementation bundle at a time against the then-current clean accepted
  head. First use deterministic tests, fixed-position shadow diagnostics and
  fingerprint/NPS checks; only strength-test a bundle with a credible signal.
- Register hypotheses before games. Default mechanism gate: `[0,3]` nElo with
  `strength-v1`; risky or broad bundles use `[-3,3]`. Promote only on H1 unless
  the item explicitly has a correctness-only acceptance rule.
- A correctness fix may be banked bench-identical without games. If it changes
  play, it needs a gate; do not hide it inside an unrelated tune.
- Shadow modes never affect ordering, pruning, TT writes, timing or RNG. They
  report counterfactual decisions and overlaps so interactions can be measured
  before activation.
- Record 1T STC first, then 1T LTC and 4T for timing/SMP/root changes. Run the
  external ladder only at cumulative checkpoints; do not optimize for a single
  opponent.
- While the current tournament/SPSA occupies 14 cores, run no bench, NPS, PGO,
  SPRT or other game job. Source/document work and non-timing unit tests only.

#### 10.0 — Close and freeze the two live experiments

1. Let the 36,400-game Rating Tournament finish unchanged. Archive its engine
   binaries, manifests, opening set, TC/thread/hash settings, full PGN and
   final standings. Report paired/opponent deltas and uncertainty; do not use
   the live ordinal Elo column as an acceptance test.
2. Let the aspiration SPSA reach its registered 5,000 iterations unchanged.
   Archive `state.json`, seeds, trajectory, final theta and tuner logs. The
   interim vector inside `2.4.0-dev` is not eligible for acceptance.
3. Apply the predeclared estimator to the finished SPSA, bake once, rebuild a
   clean PGO candidate, and gate it against
   `rarog-p1043-base-pext-pgo.exe`. If it fails, restore the clean baseline;
   retain only neutral instrumentation.
4. Reconcile the tournament attribution only after the aspiration result is
   known. The output is a baseline memo, not a new mechanism.

**Exit:** no live/interim constants remain in source; clean manifest and
fingerprint are recorded; the accepted head is reproducible.

#### 10.1 — Diagnostic substrate and interaction map

Implement bench-neutral counters and deterministic trace sampling for:

- TT hits by depth, bound, PV bit and producing path; consumer matrix for
  qsearch, RFP, razor, NMP, ProbCut, IIR and singular extension;
- stand-pat vs searched-qmove cutoffs, qsearch TT-return rate, delta/SEE/futility
  overlap, check/evasion classes and qsearch share (currently ≈40% of nodes);
- in-check-node share (prior sample ≈9.6%), legal-evasion count/rank and the
  exact effect of the still-active `!in_check` LMR exclusion;
- NMP attempts/cutoffs/verifications, nested nulls inside verification, cut-node
  status, raw vs TT-adjusted eval and later contradiction rate;
- IIR at PV/non-PV/all/cut nodes; singular candidates by provenance, single vs
  double extensions, extension debt and immediate fail-low/reduction fallout;
- move stage, prospective depth, LMP/futility/SEE/LMR reason mask, quiet-check
  subtype, evasion rank, LMR re-search and result reversal;
- history/correction update source, capture involvement, context bucket,
  saturation and prediction residual before/after adjustment;
- root score mean/variance, best-move age, node effort, fail direction/count,
  completed depth, fallback source, worker best-move changes and time usage.

Add shadow predicates for the proposed 10.2–10.8 contracts and emit an
interaction report: pairwise overlap, contradictory decisions, unique node
share and outcome correlation. Add unit/property tests for score
normalization, mate distance, TT replacement/aging, null verification nesting,
legal root fallback and counter overflow.

**Exit:** identical bench and best move with diagnostics off; bounded overhead
with diagnostics on; one representative corpus report checked into the
experiment record. This substrate stays through NNUE unless measured costly.

#### 10.2 — Evidence model and TT consumer contracts

Introduce internal `OutcomeKind`, `NodeEvidence` and `MoveEvidence` concepts.
They need not all be stored in TT: keep transient metadata in the stack/search
result, then store only the smallest provenance needed by later TT consumers.
Prototype two encodings:

- **Compact:** reuse spare `flag_age` bits for a small provenance class while
  preserving the current 10-byte entry and cluster density.
- **Explicit:** widen/repack the entry only if the compact scheme cannot express
  the necessary safety contract and measured cache cost is acceptable.

Minimum distinctions: full-width searched result, reduced result later
verified, qsearch searched move, qsearch stand pat, ProbCut/speculative cutoff,
null cutoff, and incomplete/aborted result (which must not enter TT as normal
evidence). Audit replacement, aging, serialization assumptions and every
reader. Define a capability table: which provenance/bound/depth combinations
may adjust static eval, cause an immediate TT cutoff, seed NMP, satisfy ProbCut,
suppress IIR or authorize singular extension.

Treat an inexact TT bound that contradicts the current window as negative
evidence. Shadow-test a small confidence/depth or replacement-priority penalty
(the useful idea behind Stockfish's mismatch penalty), including repeated-hit,
mate-score and shared-TT race cases; do not mutate entries merely because the
reference engine does.

Prefer recomputation or refusal over pretending weak evidence is exact. If the
compact encoding loses strength through reduced TT utility, compare against a
stack-only version before considering a larger entry.

**Gate:** correctness tests + identical disabled-path bench; then `[0,3]` for
the activated consumer contract. Keep instrumentation even if storage encoding
is rejected.

#### 10.3 — Qsearch, ProbCut and TT evidence hygiene

Build on 10.2:

1. Separate raw static eval, TT-adjusted eval, stand pat and searched scores.
   On a no-TT-hit stand-pat cutoff, do not manufacture a searched lower bound.
2. A qsearch TT entry may influence main search only through the capability
   table. In particular, depth-0 stand-pat/pruning estimates cannot drive
   depth-8 RFP/razor/NMP/ProbCut as if searched.
3. Store the actual searched ProbCut score with explicit speculative
   provenance. Tune/derive storage depth only after measuring contradiction by
   parent depth; never normalize the score merely to resemble the parent beta.
4. Singular extension requires compatible full-search evidence. ProbCut,
   null-cutoff, stand-pat and incomplete evidence are forbidden even when
   depth/bound numerically match.
5. Audit fail-soft mate/TB score conversion and draw contempt/context before
   accepting or rejecting any TT result.

Use counterfactual replay to measure removed cutoffs, re-search cost and later
score contradiction. Gate subchanges separately unless all are individually
bench-neutral correctness repairs.

**Exit:** no speculative or stand-pat result can silently acquire full-search
authority; qsearch/main-search and ProbCut/singular interaction tests pass.

#### 10.4 — NMP, IIR and singular-extension cooperation

Rebuild the trio around node/evidence contracts:

- **NMP:** add subtree-scoped verification suppression (`nmpMinPly`-style or a
  Rarog equivalent), `cutNode`/node-role handling, raw-eval and improving
  evidence, non-decisive score guards, material/zugzwang safety and a
  potential-singularity guard. Measure TT-bound-adjusted versus raw null
  windows; do not copy either reference blindly. Verification must not contain
  a nested null unless an explicitly tested policy permits it.
- **IIR:** never reduce a PV-following node merely because its TT move is
  missing. Compare eligible non-PV/all-node depths and use measured TT miss
  quality. Treat IIR depth as debt visible to later pruning/extensions.
- **Singular extension:** raise or dynamically derive its depth threshold from
  evidence quality; separate single/double extension conditions, cap cumulative
  extension debt, and feed extension/reduction outcomes back into move ordering
  only after a full-width result.
- Replace the blanket `!tt_pv` forward-pruning veto with separately named RFP,
  razor, NMP and ProbCut eligibility predicates. A stored PV bit is a hint, not
  a universal safety proof.

Test NMP, IIR and singular changes first in isolation, then as a registered
joint bundle because their interaction may reverse individual results. Required
shadow report: nodes saved/spent, verification rate, tactical/mate errors,
zugzwang suite, extension debt and unique/overlap cutoffs.

**Gate:** each credible arm `[0,3]`; joint bundle `[-3,3]`. Keep the joint
bundle only if it beats the best accepted isolated composition.

#### 10.5 — Correction and history attribution

Repair the learning signals before tuning their weight:

1. Replace the one-step continuation-correction surrogate with true contextual
   pairs (test useful offsets such as 2 and 4 plies) while controlling table
   size/cache pressure.
2. Restore a semantic guard against capture-caused quiet correction updates;
   compare strict exclusion, scaled credit and a separate noisy residual. The
   old tuned default of zero does not override the attribution bug.
3. Split or index histories only when diagnostics show residual value:
   quiet/noisy, threat-from/threat-to context, check/evasion class and
   halfmove-clock bucket are candidates from Reckless, not a mandated bundle.
4. Centralize bonus/malus application and saturation. Update only from
   completed, attributable full-width searches; speculative/aborted paths do
   not train tables.
5. Make correction confidence available to pruning and aspiration, but do not
   allow the same residual to amplify both static eval and reduction without a
   double-counting audit.

Run ablations for each context and for their memory cost. Gate prediction
quality on a held-out fixed corpus before games; strength-test only contexts
that improve residual calibration without destructive saturation.

**Gate:** `[0,3]` per compact bundle; a larger table also needs interleaved NPS
and hash-hit non-regression evidence.

#### 10.6 — Unified prospective-depth selectivity

Create one per-move `MoveEvidence` record after ordering and before pruning. It
contains move stage, quiet/noisy/check/evasion class, history/stat score, SEE,
current and prospective (`lmrDepth`) depth, reduction/extension debt and reason
bits. Then make LMP, futility, SEE pruning, LMR and re-search consume it in a
defined order.

- Replace disconnected thresholds with monotone relationships over prospective
  depth. A move must not be pruned at a depth where the same evidence would
  earn a less severe reduction at a lower depth.
- Calibrate the accepted zero-reduction floor inside this pipeline; do not
  silently undo 10.2.5(a).
- Add stat-score/prior-reduction feedback only from attributable full searches.
- Replace the universal quiet-check bonus/bypass with check subtypes: forcing
  contact/discovered checks, safe ordinary checks and losing/SEE-negative
  checks. Measure whether checks need exemption, reduced pruning or merely an
  ordering bonus.
- Resolve late evasions explicitly. Permit reduction/pruning only for ordered,
  non-forcing, safe late evasions with preserved mate legality; remove either
  the stale plan claim or the stale `!in_check` code.
- Track pruning overlap so several weak predicates cannot accidentally combine
  into an unmeasured cliff.

Use a factorial or staged ablation, not one mega-patch: prospective-depth
plumbing; check taxonomy; evasions; then feedback. Validate tactical, mate,
quiet and zugzwang suites plus node-quality diagnostics.

**Gate:** `[0,3]` for each mechanism; final combined surface `[-3,3]`. Reject
any speed win that reduces decision quality at fixed nodes without a clock-time
strength pass.

#### 10.7 — Qsearch as a first-class search

After TT hygiene is correct, optimize the ≈40% qsearch share without changing
HCE:

- stage TT move, good captures/promotions, forcing checks and bad captures with
  separate quiet/noisy histories where evidence supports them;
- test qsearch SEE history and threat context, using main-search history only
  when attribution matches;
- derive delta/futility/SEE pruning from raw eval, material gain, promotion and
  check/evasion state; audit overlaps and mate-distance safety;
- give in-check qsearch a complete legal-evasion contract: no stand pat, no
  false mate from pruned evasions, and measured late-evasion handling;
- distinguish searched-move bounds from stand pat in TT and correction
  updates; do not train quiet correction from capture-only resolution;
- profile hot paths only after the behaviour bundle is accepted. Preserve
  semantics in any movegen/SEE/cache optimization.

Candidate arms: TT hygiene alone (from 10.3), qsearch move ordering/history,
check/evasion selectivity, and a combined arm. Compare fixed-node decision
quality as well as clock-time Elo so a lower node count is not mistaken for
better search.

**Gate:** `[0,3]` per arm, `[-3,3]` combined; interleaved NPS protocol for any
hot-path structural change.

#### 10.8 — One root-confidence model: aspiration, TM, fallback and SMP

Finish the work that `RootMove` made possible. At the end of every completed
iteration, derive a root-confidence snapshot from per-move mean and mean-square
score, score gap/variance, best-move age, node effort share, PV continuity,
fail-high/low history and completed depth. Persist no partial iteration as the
new authoritative root result.

- **Aspiration:** initial width and asymmetric growth may use measured
  volatility, but cap re-searches and open the window safely after repeated
  failures. Treat mate/TB scores separately. The ongoing SPSA decides only the
  legacy shape; it cannot replace this mechanism test.
- **TM:** combine stability, score trend, confidence gap, effort and available
  increment. Avoid counting correlated signals twice. Hard/soft limits and
  ponder/overhead behaviour retain explicit safety margins.
- **Fallback:** on abort, return the best legal move from the last completed
  iteration. Never publish an uncompleted mate/win/loss score; handle depth-0,
  only-move and TT/PV corruption cases deterministically.
- **SMP:** pool best-move changes/instability across workers for timing, while a
  designated completed root result owns the move. Do not spend another wave on
  random diversification; shared TT already supplies coupling.
- **Feedback:** TM may grant time because confidence is low, but aspiration
  width must not then widen from the same signal without measuring the joint
  cost. Log the causal components.

Test aspiration-only, TM-only, fallback-only correctness and the joint model.
Use fixed-time replay traces, forced timeouts and multi-thread determinism/legal
move tests before games.

**Gate:** aspiration `[0,3]`; TM/root joint model `[-3,3]` at 1T STC plus 1T
LTC; mandatory 4T LTC sanity with zero forfeits/regressions before acceptance.

#### 10.9 — Parallel and throughput pass

Only after 10.2–10.8 stabilize:

1. Profile accepted 1T and 4T builds with PEXT and the shipped PGO workflow.
   Separate node savings from instructions-per-node, TT contention, allocator
   work and timer overhead.
2. Audit TT cluster layout after provenance changes, replacement contention,
   false sharing, atomic ordering and prefetch. Preserve correctness on every
   supported architecture.
3. Batch/hoist only invariant work proven hot: move metadata, attack refresh,
   history addresses, time checks and root aggregation. Behaviour-changing
   speedups return to their owning mechanism rather than entering a speed
   bundle.
4. Measure scaling at 1/2/4/8 threads and at tournament hash sizes. Report NPS,
   effective depth, time-to-depth, hashfull/replacement, root stability and Elo.
5. Stop diversification experiments unless 10.8 diagnostics reveal a new,
   specific independent-work failure.

**Gate:** behaviour-identical items use interleaved repeated builds and the NPS
protocol; any changed fingerprint/best move needs `[0,3]`. Mandatory 4T
strength/forfeit gate for shared-state changes.

#### 10.10 — Final joint search refit

This is the only broad parameter fit in the rewritten phase. Start only when
10.2–10.9 mechanisms and accepted composition are frozen.

- Generate parameters and bounds from the live code; remove dead/tuned-off
  knobs from the active group but preserve their implementation through NNUE.
- Fit coupled groups in dependency order: evidence/selectivity; history and
  correction; qsearch; NMP/ProbCut/IIR/singular; root aspiration/TM. Use a final
  small joint polish only if cross-group residuals justify it.
- Include interaction coordinates or staged revisits for known symbioses:
  TT provenance×singularity, correction×RFP/NMP, LMR depth×futility/SEE,
  qsearch history×TT storage, and root variance×aspiration×TM.
- Use held-out seeds/openings and compare final theta with a predeclared tail
  estimator only if that estimator was registered before the run. No post-hoc
  tail selection.
- Bake once, `cargo fmt`, clean PGO build, manifest/fingerprint, then a real
  `[0,3]` gate against the pre-fit accepted head. A tune is not accepted because
  its training objective improved.

No HCE coordinate participates. If the fit exposes an evaluator residual,
record it for NNUE feature/training design rather than reopening HCE work.

#### 10.11 — Cumulative checkpoint and regression matrix

Before targeting named engines, prove the accepted composition is more than a
collection of self-play wins:

| Check | Required result |
|---|---|
| Reproducibility | Clean source, pinned toolchain, PGO manifest, UCI defaults and fingerprint reproduce on a second build. |
| Correctness | Perft/unit/property/tactical/mate/zugzwang/TB suites pass; no illegal/stale fallback; no decisive-score corruption. |
| 1T strength | Cumulative head beats the 10.0 frozen baseline at STC and confirms at LTC. |
| 4T transfer | Positive/non-regressive result, healthy scaling and zero time forfeits at release conditions. |
| Ablation | Remove each major accepted subsystem once; confirm the joint gain is not carried by a single hidden regression or tune compensation. |
| Telemetry | No provenance violation, pathological extension debt, history saturation, aspiration loop or timer overrun in sampled games. |

If cumulative strength is materially below the accepted individual evidence,
stop and run interaction ablations. Do not proceed by adding more constants.

#### 10.12 — Hard target ladder and 2.4.0 release

Run a locked, seeded, adjudication-audited ladder with the release candidate
and at least: Basilisk 1.9.3, Rybka 3/4.1/4/5/6, Critter 1.6a, Houdini 2.0c and
Fritz 16. Houdini 1.5a remains a continuity anchor. Record exact engine
versions, UCI settings, hash/threads, openings, TC, concurrency, tablebases and
PGNs. Use enough games for paired confidence intervals; report head-to-head and
pool estimates, not only ranks.

**Hard strength gate:** for every target, the paired head-to-head logistic-Elo
lower bound must exceed zero under the registered primary 1T condition; use a
Holm-adjusted 95% family-wise result across the target cohort so one lucky arm
cannot release the engine. Confirm the result at 1T LTC and run a 4T LTC
transfer/forfeit matrix. If licensing or platform prevents a target from
running, Phase 10 remains open until an equivalent user-approved direct test
is available; a rating-list inference is not a substitute.

Then complete release hygiene: clean tree/build manifest, default UCI audit,
bench fingerprint, CI and regression suites, Windows binaries, README/UCI
documentation, user-facing changelog/release notes without internal experiment
history, version `2.4.0`, tag and archive the full tournament evidence.

**Release rule:** 2.4.0 ships only after the hard target gate passes. A large
gain that still merely catches Basilisk is progress, not completion. Phase 11
starts after this release boundary; NNUE work does not become an escape hatch
for an unfinished Phase 10.

### Phase 11 — NNUE runway: measurement + state rework (EV ~0 direct; unblocks Phase 12) — Fable 5 medium/high; 11.1: Sonnet 5 medium pipeline

**This phase IS the NNUE infra preparation (macro-section C opens here)** —
the audit-§8 runway plus the hce-§16 measurement substrate, kept as its own
phase so board surgery never mixes with strength patches. **11.1** has no
engine footprint and should be pulled forward into Phase 8–10 SPRT downtime;
the remaining state work (11.2–11.5) starts once Phase 10 closes (or
earlier, via the Phase-8 wave stop-rule) and directly feeds Phase 12. Its
residual report chooses NNUE feature experiments and, only if NNUE later
fails, selects Phase-13 fallback items. Gates: bench fingerprint
identical per step, best-of-N NPS watch, one batch `[-3,0]` only if NPS
moves.

0. **11.0 Clean-code package P3 — "structure era" (audit 2026-07-19).**
   Deliberately scheduled HERE, not in Phase 9: it rearranges exactly the
   code the Phase-8 wave is still editing, and this phase's charter is
   "board surgery never mixes with strength patches" — 11.2 (StateInfo) and
   11.4 (accumulator stack) already do the adjacent surgery, so the three
   moves compose into one era:
   - **(i) Extract a `history` module:** all quiet/capture/continuation/
     correction tables + their update rules behind a `HistoryTables` struct
     (search.rs is 3,135 lines with a 746-line negamax and a 48-field
     Searcher — top Rust engines (Viridithas/Velvet/Black Marlin) keep
     ordering state in its own module). Pure code motion, bench-identical.
   - **(ii) Per-ply `PlyContext` array-of-structs** (stack_moves,
     stack_pieces, killers, cutoff_cnt, pv_len, …) — the search-side twin of
     11.2's StateInfo, and the exact shape 11.4's accumulator stack wants.
     Bench-identical + best-of-N NPS watch (cache-layout change).
   - **(iii) Split a `rarog-core` workspace crate** (board, movegen,
     attacks, zobrist, perft): texel-tuner (already a member) and the
     Phase-12 datagen tools depend on core without the engine; search
     changes stop rebuilding movegen; fuzz targets isolate. Fat LTO keeps
     codegen identical — no perf cost expected, NPS-verified anyway.
   - **(iv) `TimeBudget` type retiring the `optimum_ms`/`maximum_ms`
     `f64::INFINITY` sentinels** (the depth-Option pattern finished). This is
     the package's ONE SPRT-class change (bench cannot see TM): `[-3,0]`
     non-inferiority, riding the same gate slot as the phase's NPS batch check.
   - **(v) `Move`/`Square` private fields + `const fn` constructors**
     (deferred here from 9.0a(vii), 2026-07-19). `Square(pub u8)` and
     `Move(pub u16)` let any caller build an out-of-range value; 9.0's
     `Square::index() & 63` already makes *reads* total, so this is the
     constructor half of the same invariant — the type, not the call site,
     should be the boundary. Deferred out of Phase 9 deliberately: it touches
     **56 construction sites**, nearly all in hot movegen/board arithmetic
     (`Square(ep_sq.0 - 8)` and friends), for what is now a *conceptual*
     rather than a soundness gain. 11.0 already rewrites this code for
     `PlyContext`/`rarog-core`, so the churn is absorbed into a diff that has
     to happen anyway. Gate: bench-identical (it is a pure constructor swap).
   Explicitly NOT planned, with reasons on record: negamax context-struct
   (aliasing risk in the hottest recursion — accepted crate-wide in
   Cargo.toml's lint wall, user decision 2026-07-19), MovePicker box
   (measured −10% class), zero-dependency stance stays (auditability).
1. **11.1 Frozen diagnostic corpus + residual harness** (hce audit §16; no
   games): deep external teacher cp/WDL labels (not Rarog-adjudicated — the
   datagen loop is self-referential, hce §14) + Syzygy WDL/DTZ cohorts,
   by-game train/validation/test separation (extends 9.6a), exact
   material-signature / phase / king-danger / passer-cohort labels, paired
   counterfactual positions per intended feature, and per-candidate
   reports: residual by cohort, full-vs-lazy deltas, raw-vs-corrected HCE,
   HCE vs qsearch/depth-N, activation counts/covariance. Diagnostic and
   experiment-*selection* only — SPRT remains the verdict (lesson 1; the
   −17.11 distillation proves static loss alone misleads). Doubles as the
   Phase-12 stage-gate metric source and the Phase-13 selector.
2. **11.2 Per-ply `StateInfo`:** consolidate keys / castling / EP / rule50 /
   `plies_from_null` / checkers / captured piece (today scattered across
   `Board` fields + `UnmakeInfo`) and add `blockers_for_king[2]`,
   `pinners[2]`, `check_squares[piece]`, repetition distance/status.
   Retro-feeds 7.2 (cached pins make SEE masks ~free) and 10.3's geometry
   sharing.
3. **11.3 Dirty-piece delta contract:** `removed/added/(color,piece,square)`
   + king-moved/bucket-changed flags for every move type incl. castling, EP,
   promotion, and null; randomized make/unmake walks compare incremental
   state against a full refresh every ply.

   **Design resolved 2026-07-22 — adopt Reckless's `BoardObserver` shape**
   (read from their master `45ea6a9` after the 4-engine perft comparison,
   `analysis/board_perft_compare.md`, surfaced that Reckless/SF boards EMIT
   their changes while ours mutates silently):
   - `trait BoardObserver`, THREE events: `on_piece_change(piece, sq, add)`
     (add/remove), `on_piece_move(piece, from, to)` (quiet relocation),
     `on_piece_mutate(old, new, sq)` (capture = victim→attacker on the
     to-square; promotion = pawn→promo). Emitted at the exact mutation
     points — castling fires rook-remove + king-move + rook-add, EP fires
     the `to^8` pawn-remove. `make_move<T: BoardObserver>` is GENERIC, so
     the null observer (perft/tests/datagen) monomorphizes to zero code —
     verify with `tools/perft_compare.py` (suite must be unchanged) plus
     bench bit-identity.
   - **Two delta channels with different timing needs:** a COMPACT pre-make
     push (mv, moving piece, captured) into a MAX_PLY stack entry feeds the
     PST accumulators and is reconstructable later; the DURING-make observer
     events feed threat features that need the board mid-transition and
     cannot be reconstructed post-hoc (their net has 66,864 threat inputs).
     If our net (12.5) grows relation/threat inputs, the observer channel is
     what makes them incrementable.
   - **Laziness at evaluate(), per POV:** walk the stack back to the last
     accurate entry, replay deltas forward for the needed perspective only;
     king crossing an input-bucket boundary ⇒ cached full refresh (finny
     tables — 11.4's cache slot). `pop()` is `index -= 1`: undo is free.
     **Null moves emit nothing and don't push** — the top entry stays
     accurate.
   - **Why:** most nodes never reach evaluate() (pruned/TT-served/cut), so
     make pays delta recording only, unmake pays zero, and the board stays
     eval-agnostic. This bookkeeping is why Reckless's perft is 34% slower
     than ours while its per-node search eval is nearly free — the design
     trades board-microbench speed for search-eval speed, which is the
     right trade once a net exists.
   - **Deliberately NOT pulled into Phase 8** (considered and reverted
     2026-07-22): HCE gains nothing — 8.12(a)'s three scalars (mg, eg,
     phase) are cheapest updated eagerly inline in `make_move`, no trait
     needed — so the plumbing waits for its real consumer. When 8.12(a)
     lands, its inline update sites double as the future emission-point
     map: every place it touches is a place 11.3 will emit.
4. **11.4 Per-thread evaluator state:** accumulator stack + quantized
   inference scratch, scaffolding only — the accumulator lives with the
   search worker, **not** inside the copyable `Board`; HCE keeps running
   through `Evaluator::eval()` untouched. (Stage-A chess768 nets never
   refresh; reserve the king-bucket refresh-cache slot but build it in 12.3,
   where net_trainer v2 defines the bucket layout.)
5. **11.5 Threat-map hooks (optional, audit §8.4):** reserve the dirty-threat
   interface so threat inputs can land in Phase 12 without another
   make/unmake rewrite.

### Phase 12 — NNUE: primary evaluation program via net_trainer (EV +150–350 initially; required for top-tier aspirations) — Fable 5 high (alt Opus 4.8 high) engine integration + architecture revs; Sonnet 5 medium data pipeline

Start after Phase 11. A competitive NNUE is necessary, not sufficient, for
top-level strength: data quality/scale, incremental inference speed, search
recalibration and repeated self-play cycles matter as much as the layer sizes.
Keep `Evaluator::eval()` as the only search↔eval boundary so HCE remains a
known-good fallback and search never depends on evaluator internals.

**Training stack: `net_trainer` (`D:/code/net_trainer`)** — the existing
engine-agnostic, bullet-based pipeline (datagen → extract → convert/shuffle →
GPU train → `quantised.bin`). Rarog's side of the work is *implementing the
consumer contract* (`docs/nnue_format.md`), not building a trainer: the v1
architecture is **chess768 → (H×2, perspective, SCReLU) → 1×8 material output
buckets** (QA=255, QB=64, SCALE=400), H=1024 default; correctness is gated by
integer-exact **conformance vectors** (`models/test/`, reference C++/Rust
implementations in `examples/`). The documented upgrade path is bullet's
progression: v1 (output buckets, **no king buckets — accumulators never
refresh**) → v2 mirrored king-bucket inputs → v3 multi-layer/pairwise-mul.
Training needs an NVIDIA GPU + CUDA (`trainer/` has no CPU backend); data
tools run anywhere. Do not fork the format: architecture changes go through
net_trainer (trainer + format doc + new conformance net together).

1. **12.1 Contract bring-up (stage A; not a final net).** Implement
   `net_trainer/docs/nnue_format.md` in Rarog: chess768 inputs, two
   perspectives, SCReLU, 8 material output buckets
   (`bucket = (popcount(occ) − 2) / 4`), quantized SIMD inference. Start from
   `examples/rust`. **The acceptance gate is the conformance vectors,
   integer-exact** — that replaces any custom header/versioning scheme (the
   file is bullet's raw `quantised.bin`; H is published with the net and
   recoverable from file size; embed the net file's hash in engine
   identification/manifests for provenance). Remaining correctness gates on
   Rarog's side: incremental accumulator vs full-refresh equality on
   randomized make/unmake walks (castling, EP, promotion, null — no king
   refresh exists in v1), malformed/truncated-net rejection, and clean HCE
   fallback. Hard best-of-N NPS gate before games; optimize update paths
   before adding capacity if the baseline is too slow.
1a. **12.1a Seed-book design from the measured yield matrix (input: 10.4.3(a2)).**
   Before generating a 30–60M-position corpus, solve the seed allocation
   instead of assuming a phase-balanced book yields a phase-balanced harvest —
   10.4.3(a2) measured that it does not, and at Texel scale the mismatch already
   costs ~3.6× more games than the smallest quota requires. At NNUE scale that
   is a 2–3× multiplier on the single most expensive data step in the project.
   Inputs: the 5×5 yield matrix from the pilot; the target per-phase counts; and
   an explicit **floor of independently seeded games per phase**, because
   positions harvested from one game are correlated and "fewest games" is the
   wrong objective on its own. Re-measure the matrix if the label generator
   changes materially — a stronger engine plays longer games and traverses
   differently. Note the bucket *count* (3 vs 5 phases) is not the lever: it
   changes reporting granularity, not the traversal asymmetry that makes
   opening positions scarce.
2. **12.2 Data through net_trainer's pipeline.** Use `tools/datagen.py` +
   `extract_nnue.py` + `convert`/`shuffle` — do **not** grow `tools/texel`
   into a second trainer (Rarog's fastchess datagen may substitute for
   `datagen.py` since the PGN format is the same; either way seed from a
   diverse EPD book — `beast_seed.epd` / `sample_fens.py`, lesson 5). The
   label recipe is built in: `target = (1−λ)·sigmoid(score/SCALE) + λ·result`
   via `--wdl` — select λ on validation (note 6.2's pure-WDL win for *HCE*
   does not transfer automatically; a higher-capacity student can use cp
   signal a linear eval could not). What Rarog's repo adds *around* the
   pipeline: by-game/trajectory splits, dedup, the frozen test from 11.1, and
   a dataset manifest (source engine/net SHA, search budget, book, λ, RNG
   seed, net_trainer commit). Include on-policy Rarog positions, hard
   loss/fortress/conversion cohorts and tablebase-supervised endgames; do not
   train primarily on positions adjudicated early by the same eval. Report
   once on the frozen test, but let SPRT decide.
3. **12.3 King-conditioned net (stage B = net_trainer v2; minimum serious
   architecture).** Mirrored king-bucket inputs — bullet progression stage 3
   and net_trainer's documented next step (trainer change ~20 lines; the
   contract gains a bucket-layout table + a new conformance net, so Rarog's
   port is verifiable the same afternoon). Engine side is where the cost
   lives: accumulator refresh on king-bucket/mirror changes with cached
   refresh tables (11.4's cache slot; 11.3's dirty-piece contract gains the
   king-moved/bucket-changed flags here). Compare bucket counts by quantized
   NPS, frozen-cohort residuals and `[0,3]` SPRT against the accepted stage-A
   net. Stage A may ship only as an implementation milestone; do not declare
   the NNUE program complete without testing king conditioning.
4. **12.4 Material specialization — largely ships in stage A.** The 8
   material output buckets are already in net_trainer v1 (near-zero engine
   cost), supplying the conditional phase model the HCE lacks and subsuming
   fallback item 13.6. The residual stage-C item is a **direct PSQT/skip
   path** — add it only if frozen-cohort residuals still show
   material-linear error after stages A/B, as a net_trainer architecture
   change (trainer + format + conformance net), one `[0,3]`.
5. **12.5 Residual-driven relation inputs (stages D/E).** Add exact threat
   pairs after 11.5 only if threat/king cohorts remain a major residual; then
   test pawn-pair inputs for chains, levers and rams. These go beyond
   net_trainer's documented v2/v3 path — each family is a full architecture
   rev (trainer + format doc + conformance net), not an engine-side patch.
   Memory bandwidth and update frequency are first-class costs:
   quantize/compress sparse feature weights and accept a feature family only
   on net Elo after NPS. Do not add both families at once, and do not
   hand-copy another engine's final shape. (net_trainer's v3 —
   multi-layer/pairwise-mul — competes with these for the same slot; pick by
   measured v2 residuals, per its architecture doc.)
6. **12.6 Data/size scaling flywheel.** net_trainer's design regime is
   **30–60M unique positions for a serious net** (10–20 sampled
   positions/game — i.e. roughly the 6.2 datagen run's scale, 3–6M games);
   use tiny sets only for plumbing. Capacity must match data (its
   architecture doc's first principle): grow `--hidden` and step v1→v2 as
   the corpus grows, not before. Then scale accepted recipes toward
   hundreds-of-millions+ as compute permits. Generate fresh on-policy data
   with each clearly stronger net, mix it with stable teacher/tablebase
   data, retrain, and stop only when both SPRT and frozen-cohort improvement
   flatten. Data-scale comparisons keep architecture/recipe fixed;
   architecture comparisons keep the data snapshot fixed. If local compute
   is the limiter, distributed/donated training becomes part of this phase
   rather than a reason to retreat to HCE.
7. **12.7 Search recalibration after the net stabilizes.** cp margins do
   **not** transfer across evaluators, while Phase-8 structural mechanisms do.
   Refit correction source weights, run one joint cp-margin SPSA (RFP/null/
   futility/ProbCut/LMR/lazy as applicable), reconsider/remove HCE lazy eval,
   and re-run narrowly H0'd structural candidates where the new uncertainty
   distribution changes their premise. Use 7.6 diagnostics and 8.5 correction
   measurements as the before/after record.
8. **12.8 Acceptance and release gate.** Every accepted stage passes
   conformance-vector correctness, NPS, frozen residual cohorts, STC SPRT,
   LTC confirmation and 1/2/4/8-thread validation; a phase release also runs
   the external ladder. Keep the best HCE and every accepted net reproducibly
   trainable: archive the 12.2 manifest + net_trainer commit alongside each
   accepted `quantised.bin`. Phase 13
   is entered only if stage B/C plus at least one inference optimization and
   one meaningful data-scale retry fail to produce a viable net, or if the
   program later stalls well below its target despite stages D/E. A weak
   stage-A bring-up is not evidence that NNUE failed.

### Phase 13 — HCE deepening fallback (**contingent**; enter only after Phase 12 fails its NPS/strength gates or stalls after serious architecture/data retries) — Fable 5 medium/high

**Standalone final section (user mandate 2026-07-15): entered only after
Phase 12's verdict, never interleaved before it — may never run.**
Everything here is representational work that a king-conditioned,
threat-aware NNUE subsumes — sunk cost the day a net ships, which is why it
sits behind a demonstrated NNUE attempt rather than before it (hce audit
§5–§12/§17C, all re-verified at head; Elo priors tempered per lesson 12).
Non-additive: realistic HCE package **+10–30 self-play** if the top items land (the audit's 20–60
prior, discounted by the twice-proven refit ceiling, lesson 1). Selection
discipline: 11.1 cohort residuals pick the order; zero or sign-flipped
fitted weights trigger activation/covariance analysis first, never a direct
chess conclusion (hce §14). Every item is a fix/structure + refit bundle
with one gate (lesson 15 applies to eval exactly as to search — weights are
fitted around current activations).

1. **13.1 King-safety semantic rework** (needs the 7.4 bundle) — the largest remaining
   classical family (the Phase-4 KS fit alone was +42.5): activation
   instrumentation by queen presence/phase, legal vs geometric safe checks,
   blocked/unblocked/lever-supported storms, current-vs-reachable castling
   shelter, defender overload/pin inputs only where cheap, joint danger-input
   fits (`--tune-kingsafety` re-eval path, lesson 4). At head the weak-ring /
   flank / missing-shelter / storm / shelter-storm inputs are **all 0** —
   zeroed inputs are unidentified, not disproven (mixed activations cancel in
   a coordinate fit). Several SPSA/SPRT cycles. EV +5–20.
2. **13.2 Winnability / material-specific scaling** — replace the sign-only
   initiative (`eval.rs:2531` can only push a nonzero EG score *away* from
   zero; fitted weight 2): residual tables by exact material signature
   first, Syzygy WDL/DTZ as direct evidence, sign-preserving non-amplifying
   scalers only, drawn/won/cursed cohorts validated separately. Gate: one
   refit bundle + `[0,3]` per scaler family; cohort-validated before games.
   EV +3–10.
3. **13.3 Passer/pawn conditionality** — blocker ownership/type and
   rear-line openness for both rook-behind terms, connected-passer
   semantics, candidate-passer exchange conditioning, a short-horizon race
   diagnostic instead of more static path terms. (The audit's "negative
   whole-path coefficients" were the rejected refit's — at head all three
   path params are 0/inert; treat any future sign anomaly as aliasing,
   lesson 12.) Gate: structure + refit bundle, one `[0,3]`. EV +2–8.
4. **13.4 Threat conditionality** — SEE-safe pawn pushes (today "no enemy
   pawn attacks the push square" only, `eval.rs:1702`), restricted mobility
   counted per affected piece rather than board-global, pin/overload
   relations only where cheaply available; do not hand-write a threat net
   one scalar at a time. Gate: structure + refit bundle, one `[0,3]`;
   NPS-check first (threat recomputation is hot-path). EV +2–8.
5. **13.5 Broad positional repairs** — queen infiltration on the full enemy
   attack map (today pawn-attacks only at 47/73 MG/EG, `eval.rs:1786`),
   bad-bishop blocked/central-pawn conditioning, space usability (all three
   space weights are 0 at head — the representation, not the scale, is the
   problem), conditioned rook-7th (fitted to 0 as a bare rank test). One
   grouped refit + one gate per sub-bundle. EV +2–6.
6. **13.6 Material/phase specialization** — bucketed coefficients /
   king-bucketed PSTs / queen-presence gates. Worst time-to-Elo on the
   list: it hand-builds Phase-12 stage C. Only if NNUE is abandoned
   outright. Gate: bucketed refit + `[0,3]` per bucketing family. EV +3–10.
7. **13.7 Lazy-margin conditioning** — only if 9.6b's dual-eval data shows a
   material sign-flip cohort; margin by non-pawn material / king danger,
   SPSA + one gate. EV 0–3.
8. **13.8 OCB material-scope refinement** (moved from 7.4c, 2026-07-16) — the
   opposite-coloured-bishop scaler fires with queens/rooks/knights present
   (`eval.rs:2831`); a small material hierarchy (strong scaling for pure OCB,
   milder with extra minors, ~none with majors) with
   non-amplification/sign/pure-OCB/+minor/+major tests. Cheap and
   high-confidence, so it is the natural *first* Phase-13 item if this phase
   is ever entered — but it is NNUE-subsumed eval, hence here not pre-NNUE.
   `[0,3]`, EV 0–3.

### Phase 14 — Parked / later scaling (enter on demand)

Deliberately deferred (audit §13 Phase E): large-page/NUMA-aware TT and
high-thread scaling (measure scaling first), shared-TT atomic packing
revisit (only after 9.4 + a thread-scaling profile), AVX-512/VNNI kernels +
runtime dispatch and real ARM64 CPU targeting (NNUE-era), Chess960 castling
metadata + FRC regression coverage, the full match-manifest schema (9.7
lite first), stratified micro-bench workloads (audit §11.2), and
OpenBench/distributed testing once typical accepted patches are +1–3 Elo.

**Parked 7.3 — rule-50-bucketed TT search key** (moved here 2026-07-15):
probe/store/prefetch with `hash ^ RULE50_KEY[halfmove / 8]`; repetition
keeps the raw key; all four prefetch sites migrate. `score_from_tt()`
already gates mates; this fixes ordinary bounds shared across materially
different clocks (SF/Plenty/Reckless all do it). Parked because its
prerequisites (7.1b/c null-clock + fence) were SPRT-rejected, both
draw-adjacent reworks lost 7–12 Elo, and our harness's move-40 draw
adjudication means test games almost never reach high clocks — benefit
invisible at our gates, de-tuning risk not. **Re-entry triggers:** LTC-era
primary testing, an adjudication-policy change, or the 12.7 post-NNUE
re-tune.

**SMP cluster** (search §7; parked here because the 1-thread primary gate
cannot see any of it — enters together with the scaling-measurement item
and 2/4/8-thread gauntlet infrastructure; needs 10.1 `RootMove` records):

- **Keep the vote merge.** Rarog's score/depth-weighted voting
  (`search.rs:2559`) is already stronger than deepest-thread selection — the
  Basilisk "weak merge" criticism does not transfer.
- **Helper first-root-move diversity:** root score rotation today cannot
  displace the TT move — `MovePicker` emits a legal TT move before the
  diversified scores (`search.rs:237`), so all threads open identically.
  Make selected helpers' first root move override TT precedence
  deliberately, on a deterministic depth/thread schedule.
- **Helper TT write policy measured, not intuited:** the Exact 3 / Lower 5 /
  Upper 7 depth filter (`search.rs:2361`) reduces pollution but may delay
  useful cross-thread information; measure at 2/4/8 threads.
- **Whole-tree ordering jitter for helpers** (SaberTooth, 2026-07-14 review):
  a per-thread seed adds a tiny hash-derived perturbation (`0..64`) to the
  ordering score of *quiet moves only*, sized below every tier gap so no
  capture/killer/TT move is ever demoted. Diversifies helpers throughout the
  tree, not just at the root, provably ordering-safely and ~free. Pairs with:
- **Staggered helper start depths** (ditto): helpers begin iterative
  deepening at depth 1/2 alternating so they desynchronize immediately
  instead of racing the main thread through identical shallow iterations.
- **Overlap diagnostics** (7.6 extension): % of root/depth-2 nodes visited
  by multiple threads, unique TT stores, helper cutoff contribution,
  speedup/Elo at 2/4/8 threads. EV +5–15 at 4+ threads, 0 at 1 thread.

---

## S7. Reference

| Tool / path | Purpose |
|---|---|
| `tools/sprt.ps1 -EngineA <exe> -EngineB <exe> -NameA -NameB -Elo1 3 [-TC "10+0.1"]` | SPRT (fastchess); default `tc=3+0.03`, hash 64, threads 1, physical cores minus two, explicit affinity, UHO book |
| `tools/spsa.ps1 -ConfigGroup <g> -EngineSuffix <s> [-Iterations N] [-Resume] [-SetupOnly] [-LaunchOnly]` | weather-factory SPSA (setup + launch, one command; groups in `tools/spsa_configs/`, +README) |
| `tools/build_test.ps1 -Suffix <s> [-Tune|-Native]` | test binaries → `tools/test_engines/` |
| `cargo xtask build --arch pext\|avx2\|native --pgo` | release/deploy builds (PGO trains on `bench`) |
| `tools/datagen.ps1 -Suffix <s> -Rounds <N> -Start <I> -Seed <S> [-SetupOnly]` | deterministic fixed-node self-play segment; auto-concurrency leaves two physical cores, explicit oversubscription such as 24 remains deterministic |
| `tools/texel/extract.py <PGNs...> [--preflight-games 20000]` | PGN → exact phase-balanced `FEN;target`; quiet filter and optional `--blend` |
| `tools/texel/sample_fens.py` | Beast `positions.txt` (read-only!) → EPD book |
| `rarog-texel --tune <group> <train> <holdout> [out] [--epochs N --lr X --l2 X --max-positions N --from-cp --fix-k K]` | Texel fit; `--verify` reconstruction; `--buckets` per-bucket loss; `--tune-kingsafety` nonlinear KS |
| `tools/texel/bake_params.py <dump>` | bake a full dump into `src/eval.rs`. ⚠ **Always follow with `cargo fmt`** — the writer emits one long line per PST and `cargo fmt --check` fails until it runs (found 2026-08-04, after a gate binary had already been built from a would-be-CI-red tree). Then verify by bench-match (tune-binary-on-dump == baked build) |
| `tools/texel/data/beast_seed.epd` | phase-balanced 750k-start EPD book for datagen; generated 2026-08-03, 150k per phase, SHA-256 `B91C756A…B2C7F` (gitignored; regenerate with `sample_fens.py`) |
| `tools/books/UHO_Lichess_4852_v1.epd` | SPRT/SPSA/gauntlet opening book (adopted 2026-07-17, same day as Basilisk) — the SF/OpenBench-standard Unbalanced Human Openings: 2,632,036 positions, 3–4 moves deep, curated to the +0.48–0.52 White-edge band, played from both colours per pair (symmetric ⇒ unbiased but decisive). Replaces the balanced 4-move PGNs, which cost twice over: SuperGM's 2,668 lines were exhausted by any run > 5,336 games (7.2b recycled 23% of pairs → optimistic error bars), and balanced openings kept the draw rate at 56% (43% dead pairs). UHO cuts draws to ~35–45% ⇒ SPRTs resolve in substantially fewer games. **Two earlier same-day judgments corrected within hours:** (i) "book size is the issue, draw rate is healthy" — reuse was the *acute* flaw, but decisiveness was the larger standing tax; (ii) "UHO only at a phase boundary" — wrong, since every SPRT/SPSA is a self-contained A-vs-B, only *cross-run* draw-rate/Elo magnitudes lose comparability, verdicts don't. weather-factory takes the EPD natively (format from extension), so tune→confirm stays unified (principle #7). Caveats: absolute draw rates / logistic Elo not comparable to pre-UHO runs; gauntlets for CCRL-comparable estimates should use `-Book tools/books/IM_4mvs.pgn` (balanced, 11,172 unique lines, the audited fallback) |
| `tools/diag_search_quality.ps1 [-Csv <path>]` | 10.0(a) search-quality readout: first-move cutoff rate + LMR over-reduction over `bench 13`, aggregated from the per-position diag dumps. Needs a `cargo build --release --features diag` binary. ⚠ `bench` is queued asynchronously, so a piped `bench …; quit` tears the engine down before the suite runs and prints only the banner — the script drives a live process |
| `wac [depth]` (engine command, like `bench`) | WAC-300 tactical suite; deterministic solved count at fixed depth (default 10). Regression telltale for Phase-8 selectivity work; floor test in `tests/wac.rs` |
| `D:/code/net_trainer` | Phase-12 NNUE training stack (bullet, CUDA GPU): `tools/datagen.py` / `extract_nnue.py` → `net-trainer convert/shuffle/train` → `quantised.bin` |
| `D:/code/net_trainer/docs/nnue_format.md` + `models/test/` | the net consumer contract + integer-exact conformance vectors (12.1's acceptance gate); reference impls in `examples/` |
| `D:/code/hydra/tools/texel/data/sf_*.csv` | SF-60k cp labels (2M; rejected for Rarog — lesson 1) |
| `analysis/{infra,search,hce}_analysis.md` | Codex 5.6 audit (2026-07-13, at `ff21dc1`); basis of Phases 7–14. `search_analysis.md` verified line-by-line at head + fully merged 2026-07-14 (→ 7.5/7.6, 8.2–8.9, 10.1/10.2/10.4, Phase-14 SMP). `hce_analysis.md` merged same day after live re-verification (→ 7.4, 8.5c, 9.6, 11.1, Phase-12 ladder, Phase 13) — its §7/§9 fitted-value tables quote the rejected 6.2.2 refit, and two consequence claims are disproven; see lesson 12 |

**Milestones** (§S3a/Phase 10.12): M1 SF-capped-2600 ✅ · M2 Basilisk 1.5.0 ✅
· **M3 = beat current Basilisk** · **M4 = beat every installed Rybka** ·
**M5 = beat Critter 1.6a, Houdini 2.0c and Fritz 16 with simultaneous paired
confidence**. M3–M5 are rungs inside one Phase-10 release gate, not separate
release boundaries; 2.4.0 waits for M5.

**NNUE boundary rule:** never let the search know how the eval works; if a
pruning condition needs eval internals explained, it's a boundary violation.










