# Rarog development plan

Rarog is a UCI chess engine in Rust (HCE eval, PVS/negamax search, PGO builds).
Sibling projects share methodology and data: **Basilisk** (C++, `D:/code/basilisk`),
**Hydra** (Python, `D:/code/hydra`); position corpus **Beast** (`A:\Chess\Beast\data`).

**Pruned 2026-07-11** (pre-prune history: commit `d9e0d85` and earlier). This
document keeps: the development process, the release procedure, the version
record, the lessons that must not be re-learned, and the forward plan.

---

## S1. Current state

**2.3.1 patch release is PREPARED.** It restores PGO for the Windows ARM64
release asset by selecting the toolchain's LLD linker for that target. Version,
CHANGELOG and README are updated; the engine search is unchanged, so bench 13
remains 5,173,540. The remaining release steps are pushing `master`, waiting
for its CI gate, tagging `v2.3.1`, and publishing the GitHub release.

Everything after 9.8 belongs to the 2.4.0 cycle (Phase 10: root model,
aspiration/TM, the 10.2.5 search capstone, and the 10.4.6 SPSA re-fit under the
corrected schedule).

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
  weather-factory `state.json` mid-run.

### Testing methodology (the gates)

| Gate | When | Rule |
|---|---|---|
| **Bench fingerprint** | behaviour-preserving refactors | `bench 13` node total must be **identical**; any change means search/eval changed |
| **SPRT** `[0,3]` (`elo0=0 elo1=3`) | every strength claim | **The only verdict.** `tools/sprt.ps1 -Elo1 3` at `tc=3+0.03` vs the current accepted head |
| **SPRT** `[-3,0]` | non-inferiority / simplification | `-Elo0 -3 -Elo1 0`; H1 supports non-regression, H0 supports a meaningful loss. A single `[-3,+3]` SPRT is not an equivalence test. |
| **Fixed null calibration** | after harness/runner changes | byte-identical engines, 30k games; the complete 95% nElo CI must fit inside `[-5,+5]` (`-Mode calibrate`) |
| **LTC confirm** | TC-sensitive features (TM), phase boundaries | `-TC "10+0.1"` |
| **SPSA** | tuning constant groups | weather-factory via `tools/spsa.ps1 -ConfigGroup <g> -EngineSuffix <s>`; **SPSA finds candidates, SPRT decides**; bake → PGO build → SPRT |
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
  unaffected. Run `setup_tools.ps1` before the next SPSA so weather-factory gets
  the verified machine-specific affinity patch.
- **Test binaries:** `tools/build_test.ps1 -Suffix <s>` → `rarog-<s>-pext-pgo.exe`
  (SPRT/gauntlet), `-Tune` → `rarog-<s>-tune.exe` (SPSA only, exposes UCI knobs),
  `-Native` (local-only znver3). PGO trains on the internal `bench` (SF-style).
- **Texel fits are minutes** — the model runs them freely; games are the user's.

### Guiding principles (hard-won)

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

**Opponent ladder** (gauntlets, `tc=10+0.1`): Rarog 2.0.2 + 2.2.0 (own history),
Basilisk 1.5.x/1.8.0 (sibling), **Critter 1.6a** (~3150–3200, the engine to
beat), SF `UCI_Elo`-capped 2700→2800→3000 (keep Rarog scoring 30–70 %), one
independent mid HCE (Lambergar/Peacekeeper/Igel).

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
  verified. External gauntlet: **+240 over 2.1.0, ~3000 CCRL** (~75 % of
  self-play gain transferred).

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
   not another global cycle. The next large eval lever is NNUE (Phase 12), not
   more Texel cycles over the existing feature function.
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
  item carries its own tuning — the fair class), the 2.3.0 release, then
  root/TM/speed/menu and 2.4.0.
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
| **Phase 8** | Search wave — 8.1 ✅ (+22.13); 8.1b ❌; 8.2 ✅ (+30.75); 8.6 ❌ (−7.78); 8.7 ❌ (−7.29); 8.10 ❌ (≈−5.4); 8.4 ✅ (+6.01, bundles 8.12); 8.11 ❌ (−5.96); 8.12 ✅ (+0.98% NPS); 8.13 ✅ (+102.78 @4T, 1T-neutral) → **8.5 (SPSA running) is the LAST open item; 8.9 capstone MOVED to 10.2.5 for the 2.4.0 cycle (user decision 2026-07-25)** | — |
| **Phase 9** | Reproducible builds + CI + shipped PGO; clean-code P1 (9.0a) + P2 (9.0b) | — |
| **9.7.5** | SMP quality wave II — threading follow-ups from 8.13 (added 2026-07-25; deployment is now 1T **and** 4T) | — |
| **9.8** | External boundary gauntlet (you) | **▶ RELEASE 2.3.0** — correctness + search wave |
| **Phase 10** | Root model, aspiration+TM modernization, **10.2.5 search capstone (moved from 8.9 — schedule EARLY)**, profile-guided speed (**10.3 ✅ +20.31**), ⏭ menu | — |
| **10.5** | External boundary gauntlet (you) | **▶ RELEASE 2.4.0** — root / speed |
| **━━ NNUE CUTOFF ━━** | Everything above survives NNUE; **no standalone HCE-eval strength before here** | |
| **Phase 11** | NNUE infra prep — per-ply StateInfo, dirty-piece, accumulator scaffolding, frozen corpus + clean-code P3 (11.0 structure era) | — |
| **Phase 12** | NNUE program — contract → data → king buckets → material path → scaling → search re-SPSA | **▶ RELEASE 2.5.0** — first shipped net (+later net bumps) |
| **Phase 13** | Contingent HCE deepening — **only if the NNUE program fails/stalls**; every NNUE-subsumed eval idea lives here | — |

**Revived (rejected → rescheduled) features, all numbered:** **7.2 SEE**
(−8.49 standalone → `config_see` re-tune bundle, in gate now); **10.2a
aspiration** (7.0a −4.52 standalone → modern shape + retuned delta); **7.5 TM**
falling-eval fix (escalates to 10.2's `tm` re-SPSA on H0). **Correctly not
revived:** 6.1 distillation / 6.2 refit (lesson 1, off-policy/no-headroom),
7.1b–d draw rework (genuine heuristic loss, lesson 14), 7.3 rule-50 TT key
(net-negative at our adjudication → Phase 14). **7.4c OCB scope moved to
Phase 13** (2026-07-16): it is HCE eval *strength*, which NNUE subsumes — not
pre-NNUE-durable, so it no longer sits in Phase 7.

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
freeze held. New **9.0** (unsafe-surface cleanup) inserted *ahead* of 9.1
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



### Phase 10 — Root model, speed pass + ⏭ opportunity menu (EV +5–15) — Opus 4.8 medium (speed); Sonnet 5 medium (root/menu)

- **10.0 Search-accuracy decomposition — MEASURE BEFORE BUILDING. Added
  2026-07-28. Runs FIRST; its result redirects 10.2.5 and 10.4.6.**

  **Why this exists.** A paired static-eval comparison against Basilisk 1.9.1
  over 8,000 quiet labelled positions (both engines answering identical FENs,
  each with its own fitted scale constant K — Rarog 0.658, Basilisk 0.694)
  returned a difference of **−0.0003 ± 0.0012**, with the sign flipping between
  a 4,000- and an 8,000-position sample. **The two evaluations predict game
  outcome equally well.** Speed is also equal (2.2M NPS both). Yet Rarog is
  ≈43 Elo weaker in the Colosseum pool. The deficit is therefore in **search
  accuracy** — how well the search converts nodes into decisions.

  ⚠ **Framing (user, 2026-07-28): Basilisk is a measuring instrument here, not
  the target. The goal is the strongest engine we can build, not parity with a
  sibling.** Basilisk is uniquely useful for this one question because it is a
  same-eval, same-speed reference, which isolates the variable. A search-accuracy
  fix gains against *every* opponent — and unlike an eval fix it **survives the
  NNUE transition**, since Phase 12 replaces the evaluation and keeps the search.

  **The converging evidence, and why it points at OVER-pruning.** Rarog reports
  **14.6 nominal depth vs Basilisk's 12.7 at identical NPS with equal eval
  quality** — it reaches a bigger depth number on a thinner tree and plays
  worse. Every Phase 8 attempt to prune *harder* was rejected: 8.6 cutoffCnt
  −7.78 (the candidate searched 16% more aggressively), 8.7 do-deeper −7.29
  (fewer nodes *and* worse moves), 8.11 fail-soft −5.96. Four independent facts
  pointing the same way. **Nothing currently in Phase 10 tests the opposite
  direction**, and both 10.2.5 and 10.4.6 are designed to make the search *more*
  selective — possibly the wrong way. This step exists so we find out for ~one
  match instead of a cycle.

  - **(a) Internal search-quality readout — no games.** `lmr_applied` /
    `lmr_research` already gives a direct over-reduction ratio. Add the one
    missing standard metric: **first-move cutoff rate** (one counter at the
    beta-cutoff site). Healthy engines sit ~90%+; materially below implicates
    move ORDERING rather than pruning depth. ⚠ Absolute measurement only —
    Basilisk's internal counters are off-limits (its tree is read-only), so this
    cannot be a head-to-head.
  - **(b) Fixed-nodes match vs Basilisk 1.9.1.** Removes speed and time
    management entirely. If Rarog still reads ≈−43 at equal nodes, the gap is
    *pure search quality*; if it shrinks materially, TM is implicated and 10.2
    rises in priority. Note the weak prior against TM: Colosseum time/move is
    80 ms for Rarog vs 78/79 ms for the Basilisks, so gross allocation already
    matches.
  - **(c) Over-pruning probe — the decisive cheap test.** Scale reductions and
    futility margins down by a fixed factor; one binary, one `[0,3]` gate, no
    tuning. **If Rarog gains from pruning LESS, the diagnosis is confirmed** and
    10.2.5/10.4.6 must be re-aimed before either is built. If it loses, the
    over-aggression hypothesis dies for the price of one match.


  **✅ THE 2×2 IS MEASURED (2026-07-28/29) — and it sharpens the target.**
  Rarog 2.3.0 minus Basilisk 1.9.1, pool ratings, one anchor:

  | | 3+0.03 | 10+0.1 |
  |---|--:|--:|
  | **1T** | −55 ± 21 | **−38 ± 27** |
  | **4T** | −32 ± 50 | **+34 ± 24** |

  Decomposed: more TIME is worth +17 at 1T and +66 at 4T; more THREADS is worth
  +23 at STC and **+72 at LTC**. **Threads dominate.** The single-thread deficit
  is ~−38…−55 at BOTH time controls, i.e. essentially TC-INDEPENDENT, and it is
  the SMP advantage — 8.13's work — that flips the sign at 4T.

  **This kills one hypothesis and sharpens the other.** The "width-for-depth
  trade that only pays once there is depth to cash it" story predicted a
  TC-DEPENDENT deficit; there isn't one. What survives is the plainer version:
  **a mis-tuned selectivity surface costing a roughly constant fraction at any
  depth.** Operationally that is good news — it means 10.4.6's re-fit, run at
  3+0.03, should transfer to long TC, and (c)'s probe reads cleanly at either.

  ⚠ **Matchup caveat.** At both time controls the DIRECT head-to-head is worse
  for Rarog than the pool rating: 1T 10+0.1 reads −73 ± 52 head-to-head vs −38
  in the pool; 4T 10+0.1 reads −13 ± 46 vs +34. Basilisk has a ~35–45 Elo
  matchup-specific edge on top of its general strength — unsurprising for two
  ports of the same engine whose evals measured equally good, since similar
  evals produce similar plans and the better-tuned search wins the specific
  battles. Judge general strength by the pool rating (that is the goal); expect
  a direct match to read worse.

  **Pre-registered consequence.** (c) positive → 10.4.6's selectivity re-fit
  becomes the cycle's headline and 10.2.5 is re-scoped toward *accuracy* rather
  than extra selectivity. (c) negative and (b) shrinks the gap → 10.2's TM work
  leads instead. (c) negative and (b) flat → the deficit is move ordering or
  implementation, and (a) says which.

- **10.1 Persistent `RootMove` records** (search §6): per root move —
  `score, previous_score, average_score, mean_squared_score, pv, nodes,
  seldepth, fail_highs, fail_lows, last_best_depth`. Pure bookkeeping first:
  **bench-identical, no games** — the substrate for 10.2, the Phase-14 SMP
  diversity work, better interrupted-iteration fallback, and MultiPV later.
  Today root state is a bare `Vec<Move>` plus the current best result
  (`search.rs:172`). EV 0 direct; enabler.
- **10.2 Aspiration + time-management consumers** (absorbs old 8.6; needs
  10.1): (a) **aspiration modernization** — running-average centre,
  magnitude-scaled asymmetric delta growth, fail-high depth-reduced
  re-search (Reckless search.rs:98-167). **This replaces the whole widening
  loop and retires the 7.0b hang guard** — the new rule must terminate *by
  construction* (the `aspiration_terminates_on_sudden_mate_scores` test in
  `tests/wac.rs` pins the invariant), and `AspirationDelta` + the growth
  constants are SPSA'd together with the new shape before its gate (lesson
  13: the un-retuned SF shape alone measured −4.52). One `[0,3]`, EV +1–4.
  (a′) **TM escalation slot:** if 7.5 H0'd standalone, its `falling_eval`
  fix re-enters here bundled with the `tm`-group re-SPSA — joint verdict,
  final.
  (b)
  **root-informed TM** — root variance / effort distribution / stability age
  terms on top of the 7.5-corrected `falling_eval`; `[0,3]` **+ LTC
  confirm** (TC-sensitive), EV +1–4. **Cross-engine validation (Basilisk,
  2026-07-16):** Basilisk's Phase-5 TM SPSA *washed and was reverted* under
  its old root model (its "SPSA-at-maturity 0-for-2" list, alongside Rarog's
  30 h LMR null) and it deferred the TM re-tune to *its* 10.7 — after a
  root-state model supplies the same variance/instability/effort inputs.
  Independent confirmation that a bare `tm`-group SPSA is a wash pre-root-model
  and only becomes a lever here, on top of 10.1's `RootMove` records. (Note:
  Basilisk's *gaining* TM changes were robustness, not SPSA — `clock-at-go`
  +2.95 and the forfeit fix — both of which Rarog already has: `move_overhead`
  covers the `go`→clock-start and bestmove→GUI latency, and the Phase-2.9.1
  `2·overhead` reserve restored zero forfeits.)
- **10.2.5 Unified prospective LMR depth — the search capstone. ⏭ MOVED HERE
  FROM 8.9 (user decision 2026-07-25) so 2.3.0 can ship on 8.5 alone.**
  High risk / high reward, EV +3–10. **⚠ Despite the number, schedule this
  EARLY in the 2.4.0 cycle** — numbers are frozen and do not imply order
  (§S6), and a weeks-long item with a real chance of rejection needs runway,
  not the slot before a release boundary.
  Compute one confidence-adjusted `lmr_depth` per move (base table + cut-node
  pressure + weak/absent TT evidence + bad-capture SEE + correction magnitude
  − PV/TT-PV − history strength − forcing evidence) and drive LMP, futility,
  SEE pruning *and* the actual reduction from it; allow zero reduction for
  strong late moves (today clamped ≥ 1).
  **Absorbs 8.3:** the stored-PV-bit graded adjustments are the "weak/absent
  TT evidence − PV/TT-PV" inputs here; re-measure `tt_pv_veto` on the
  pre-capstone head to weight them.
  **Entry condition was MET in Phase 8** (≥2 of 8.2–8.7 passed: 8.2 +30.75
  and 8.4 +6.01), so nothing gates it but machine time and 8.5's tuner.
  ⚠ **Basilisk design input (2026-07-20):** its persistent-TT-PV prototype
  measured **+51% nodes through the LMR route with no good operating point** —
  the stored bit pays only via **pruning conservatism** (relax futility/LMP on
  tt_pv nodes), NOT via reduction adjustments. Weight the tt_pv inputs toward
  the pruning-side consumers.
  ⚠ **Prior from this project's own record:** four of the five preceding
  search-mechanism items were rejected (8.6 −7.78, 8.7 −7.29, 8.10 ≈−5.4,
  8.11 −5.96), and 8.6's specific failure mode is the one to fear here — a
  self-play-tuned candidate that searched 16% more aggressively won its SPSA
  and then lost the gate against the more accurate baseline. Gate against the
  accepted head, never against a sibling of the tuning run.
- **10.3 Profile-guided speed pass** — **now has a MEASURED budget and a
  named suspect (2026-07-22).** A three-way NPS decomposition (same engine,
  bench 5,480,624 in all three, 6 interleaved rounds, idle pinned machine)
  isolated where the Phase-9 clean-code program went:

  | build | mean NPS |
  |---|---|
  | pre-refactor source + rustc 1.97.0 | 2,983,608 |
  | pre-refactor source + rustc **1.97.1** | 2,987,306 |
  | **post-refactor** source + rustc 1.97.1 | 2,892,818 |

  **Compiler: +0.12% (nil). Refactors: −3.16%** (all six rounds negative).
  So the 9.x clean-code work cost **~3.2% NPS ≈ 2.2 Elo** — recovering it is
  10.3's first, concrete objective, ahead of any speculative hot-path work.

  **Prime suspect: 9.0a(iv).** `cont_history` was ALREADY four `Vec<i16>`
  fields before the refactor (`cont_history_1/2/4/6`), so heap indirection is
  not the change. What changed is that **12 unrolled, statically-known
  accesses became loops over `[Vec<i16>; 4]` driven by `CONT_PLY_BACK`** — the
  compiler lost compile-time knowledge of *which* table and *which* offset, in
  the hottest path in the engine (history update + move ordering run at every
  node). Restore the compile-time knowledge (unroll via macro/const-generic
  over the slots) while keeping the table-driven source shape. Secondary
  suspects: the 9.0 unsafe→safe index conversions (each adds a bounds check
  that was individually "NPS-neutral" at ±3% measurement noise).

  **Methodological lesson (record as such): bench-identity does NOT imply
  NPS-identity.** Every 9.x step was verified bench-identical and spot-checked
  "NPS neutral", but a ~0.5%-per-step regression is invisible at the ±3% noise
  of a single best-of-N comparison, and eight such steps compound to 3.2%. A
  refactor program needs ONE end-to-end NPS measurement against its own
  starting point, not per-step spot checks.

  **✅ COMPLETE AND ACCEPTED 2026-07-22: +20.31 ± 7.13 Elo, nElo +33.06,
  LOS 100%, H1 on `[-3,0]` (3,460 games @ 3+0.03).** New head `p103-gate`.
  Collective speed **+10.35%** (3,003,789 → 3,314,560 NPS pext-PGO, CI
  10.10…10.65, two PGO builds per arm). Target ≥2.7M native met. Every item
  was bench-identical (5,480,624 throughout), so the gate isolated execution
  speed from behaviour — the cleanest speed→Elo datapoint the project owns.
  Per-item results and the rewritten NPS-measurement protocol live in the
  dev guide's 10.3 block; the seven landed items were cont_history boxing
  (+1.15%), per-node CheckInfo (+2.75%), check-hinted `make_move` (+1.08%),
  MovePicker single-buffer collapse (+2.04%), pin sharing (≲1%),
  `has_pseudo_capture` split by call site (+1.5%), and the small sweeps
  (+1.18%, essentially all of it the `pick_next` scan). Rejected: qsearch
  make_move hint (−0.79%). Closed no-change: insufficient-material early
  exit (whole prize ≤0.23%). Startup: generic build 192 ms → 19 ms.

  **REVISED CONSTANT — speed→Elo at STC is ≈ +2 Elo per 1% NPS, not 0.7.**
  This item was graded with "+10% speed ≈ +7 Elo LTC"; the gate returned
  +20.31, about 3× that. Use ≈2 Elo/1% NPS when grading speed work at
  3+0.03. Keep the old figure for LTC until measured there — deeper searches
  plausibly gain less per extra node — so do not transfer this constant to
  10+0.1 unverified. Consequence for planning: **speed work is materially
  under-valued in this plan's EV columns**, and it has now outperformed the
  search-mechanism items it was queued behind (8.6 −7.78, 8.7 −7.29).

  **Methodological lesson #2 (the NPS instrument itself can lie).** Two
  estimators — ABBA rounds and a neighbour-sandwich — each read −0.2…−0.4% on
  a SELF PAIR (the same .exe in both arms), and had already produced two
  confident false rejections before the self pair caught them. Cause: bench
  NPS is left-skewed, so any estimator weighting the arms unequally against
  the slow tail manufactures a bias. Rules now: validate on a self pair
  first; compare arm-level median and best-of; pool ≥2 PGO builds per arm
  (two builds of identical source differ ~0.36%); treat non-PGO as a cheap
  deterministic screen that OVERSTATES the shipped gain (8d: +6.35% non-PGO
  vs +1.18% PGO). Tools: `tools/nps_ab.ps1`, `tools/nps_multibuild.ps1`.
- **10.4 ⏭ all-skippable menu** (each `[0,3]`, strict EV gate): threat-aware
  history (Reckless `[from_threatened][to_threatened]` shape; Fable 5 high /
  alt Opus 4.8 high; **after 8.4** — search §4: update coverage before new
  context dimensions), multi-cut/singular port (poor codex-port record),
  razoring `depth≤1`, LMR tt-move-is-capture (**absorbed into 8.6's SPSA
  2026-07-20** as `lmr_tt_capture`), bad-noisy futility, qsearch SEE
  from `(alpha−eval)`, contempt (gauntlet-only), selective extensions,
  ProbCut-margin SPSA **+ TT-veto/capture-history context** (search §9 — an
  adequate TT upper bound should veto the attempt; the raw SF-formula port
  already lost 24.5, so context, not copying), **upcoming-repetition cuckoo
  cutoff** (⚠ needs repetition-state plumbing that 7.1 tried and lost —
  lessons 14/15 prior is negative; menu long-shot only. **2026-07-20:
  Basilisk implemented it fully and SPRT'd it twice — −4.58 at 10+0.1 and
  −1.64 at UHO 3+0.03, with the +16% node cost always paid — a second
  engine's game-tested rejection; discuss before ever starting**),
  **qsearch quiet
  checks at qply 0** (demoted 8.8, 2026-07-15: after captures fail, quiets
  with `gives_check && see_ge(0)`, cap 4–6; one stricter-gate retry on H0;
  search §8 warns it is not public-engine consensus and tree cost is high
  — menu-grade EV +1–4. 2026-07-20: Basilisk closed its version as **SKIP**
  — its SPSA baked the qsearch-check cap at 0, and current SF searches no
  quiet checks in qsearch), **NMP verification region**
  (search §9: `nmp_min_ply`-style suppression through the verified subtree —
  today only the verification root disables null, descendants re-enable it;
  test endgames separately; EV +0–2), **IIR node-type awareness** (search
  §9: current rule also reduces PV nodes with no TT move; instrument via 7.6
  first), **continuation-correction true pairs** (search §5: replace the
  384-entry `(prev piece,to)` key — `search.rs:2342` — with
  `(prev piece,to)→(cur piece,to)` at offsets 2/4; **after 8.5**),
  **fail-soft bound shaping** (search §8/§11: retain lower estimates at
  prune exits — e.g. the qsearch big-delta `return alpha` at
  `search.rs:1698`; plus an isolated test of qsearch stand-pat TT stores —
  SF's 2026 progression gained by *not* storing them. **2026-07-20: the
  qsearch fail-soft half promoted to 8.11**; the stand-pat TT-store A/B
  stays here), **`lmr_shallow_tt`
  polarity A/B** (one-flag gate; doc fixed in 7.6). **Dropped outright:**
  codex `tt.rs` overhaul, "mobility-area refinement", broad quiet qchecks
  beyond 8.8's narrow gate, deepest-thread SMP selection (the existing
  score/depth voting is already stronger — search §7).
- **10.4.3 Texel re-fit of the HCE eval (added 2026-07-27, user request).**
  The eval was staged-fit in Phase 4 (+42.5 … +65.0, every stage H1) and the
  feature function is unchanged since — but the engine generating the labels
  has gained roughly +80–100 self-play Elo (8.1/8.2/8.4/10.3/8.13), and
  better search means outcomes correlate better with true eval: the same
  fit on stronger-labelled data is a real, cheap lever. The pipeline is
  complete (`tools/texel-tuner`: K-fit/Adam/groups/L2-to-prior/holdout;
  fits are minutes — games are only spent on the gate).
  - (a) Build the dataset from post-8.13 self-play PGNs (SPRT + SPSA
        archives are hundreds of thousands of games at 3+0.03; filter to
        quiet positions, drop book plies, dedupe by key), re-fit K, run the
        full-scalar fit with L2-to-prior, bake via `tools/texel/bake_params.py`,
        verify with `--verify`, one `[0,3]` gate.
  - (b) **Exactly ONE unconditional run — a second run is pre-registered as
        CONDITIONAL on 10.2.5 landing.** Rationale (user asked to be
        challenged on "more runs"): iterating Texel on the *same* engine
        re-draws the same sample — labels don't improve, so run 2 converges
        to run 1's fit and the night is better spent on 10.4.6. The one
        thing that makes a second run worth it is a materially different
        label generator, and the only such event before 2.4.0 is the
        capstone. So: if 10.2.5 is ACCEPTED, regenerate data post-capstone
        and re-fit once (this is also why the wave's search re-fits sit
        AFTER the conditional re-fit — they must tune against the final
        eval); if 10.2.5 is rejected, one run was the right number.
- **10.4.6 SPSA re-fit under the fixed schedule (added 2026-07-27, user
  request; REVISED same day to minimize tune count — SPSAs are the
  most expensive thing this project runs).** Every existing fit was
  annealed ~8× too fast (the schedule bug, fixed `a0fbc9f`): each tune
  spent ~19% of its intended adaptation budget. Accepted bakes all won
  real SPRTs and stand; this step collects the unrealized upside at
  **minimum cost: ONE mandatory tune night + one gate**, exploiting the
  core SPSA property that per-iteration cost is 2 evaluations *regardless
  of dimension* — merging groups is free, and where knobs interact
  (corr scales multiply the very margins the pruning group sets), tuning
  them together is not just cheaper but more correct than sequential
  single-group fits.
  **⚠ Sizing was MODELLED before committing machine nights** — see
  `tools/spsa_convergence_model.py`, which runs the shipped schedule against
  a calibrated noise model (32-game mini-match, std(w−l)=4.2, logistic
  slope 0.092/Elo, measured 28.57 s/iteration). Five findings drive the
  design, and two of them correct things I previously asserted:
  1. **Dimension is ~free.** p=6 and p=26 converge at nearly identical
     rates — Spall's result, confirmed on our own schedule. Merging groups
     costs nothing and captures cross-knob interaction. **One tune, not
     three, is both cheaper AND more correct.**
  2. **Iterations dominate, and our history was far too short.** At
     1,000–2,500 iterations a tune barely beats its own seed — and *every*
     Rarog tune ever run sat in that range (8.5's 3,673 was the longest).
     ~5,000 recovers ~70% of the available gain, ~10,000 ~85%. This is a
     second, independent reason past fits underperformed, on top of the
     schedule bug.
  3. **There is an absolute noise floor, independent of the starting
     point.** At p=18 a run lands in the same band whether seeded 1.0 or
     0.25 steps off the optimum. **So re-tuning knobs already inside the
     floor strictly HURTS — it scatters them.** Whether our current values
     are inside it is genuinely unknown, which is what (a)'s kill-checkpoint
     and the `[0,3]` gate exist to handle. (The per-knob "bake filter" once
     named here is RETRACTED — see (a): it breaks the joint fit, and the tail
     mean already suppresses per-knob wander.) This floor is a property of
     the NOISE, so the recalibrated `a=0.1` lowers it — the sweep reads RMSE
     0.32 at a=0.1 vs 0.78 at a=1.0 — but cannot remove it.
  4. **Curvature below ~0.5 Elo per full step is unfittable** at 32
     games/iteration; such knobs wander forever and baking their wander
     ships noise. 8.5's corr bundle measured +1.4 ± 4.9 *in total* across
     8 knobs, so those knobs are probably in this class.
  5. Games-per-iteration is ~neutral at fixed game budget (16…128 all land
     within noise), so 32 stays.
  - (a) **THE tune — one combined "selectivity" run, 26 knobs:**
        `config_pruning` (14, already including `EvalPruneTtMinDepth`) +
        `config_see` (6, overlapping on `SeePruningCoeff`/`Max`) +
        `config_corr` (8; the guard stays OUT — separately-gated discrete,
        audit class 5 enforces it), **with 8.11's fail-soft qsearch
        re-applied first** — the strongest could-have-been-saved candidate:
        its −5.96 was mechanically traced to this exact group having been
        fitted against fail-hard's inflated bounds (+14.4% nodes through
        `eval_for_pruning`). **Target 5,000 iterations ≈ 160,000 games ≈ 40
        h ≈ 2 nights** (28.57 s/iter measured); extend toward 10,000 only
        if the trajectory is still moving and the machine is free.
        **Correction to the earlier draft of this item: `DeeperMargin` does
        NOT exist** — 8.7's revert deleted the knob and `config_deeper`, so
        giving it "one seat" would mean re-implementing do-deeper first.
        Do-deeper stays rejected and out of scope here.
        **Multi-session operation (hardened 2026-07-27 — a 40 h tune always
        spans sessions, and all three of these were broken):** the log now
        APPENDS on resume (it truncated; 8.5 lost 1,086 of its 3,670
        iterations, and the trajectory is precisely what the tail-mean bake
        and the per-knob filter read); `main.py` now STOPS ITSELF at
        `$env:RAROG_MAX_ITERS` (it was `while True:`, so the target lived
        only in the operator's head); and `spsa.ps1` prints
        iteration/percent/ETA on every resume. Both patches are re-applied
        idempotently by `setup_tools.ps1` and enforced by launch-time marker
        checks, like the affinity patch. **⚠ `A` is frozen at first launch** —
        `main.py` restores `spsa_params` from `state.json`, so re-passing
        `-Iterations` on a resume is ignored (the script now says so out
        loud). Set the target correctly on the FIRST launch:
        `./tools/spsa.ps1 -ConfigGroup <g> -Iterations 5000`, then resume with
        `-LaunchOnly -Iterations 5000`.
        **Kill-checkpoint at ~1,500 iterations (free, built in):** seed two
        knobs a full step off their baked values. The fixed schedule must
        visibly walk them back. If they wander instead, the tuner lacks
        resolving power at this noise level — **stop and debug before
        spending night two**, because finding 3 says the rest of the run
        cannot then help either.
        **Bake ALL tail means — no per-knob filter. ⚠ RETRACTED 2026-07-27
        (user challenge, and they were right).** An earlier draft of this
        step said to bake a knob only if it "moved meaningfully and showed a
        consistent direction", keeping the seed otherwise. That is unsound:
        SPSA estimates a **joint** optimum, and the knobs in this group
        interact by construction (the corr scales multiply the very margins
        the pruning knobs set — that interaction is *why* the groups were
        merged). Reverting a subset yields a point the tuner never
        evaluated, and if a kept knob's fitted value was conditional on a
        reverted one, the survivors are no longer justified by the run.
        **The tail mean already IS the filter**, which is what makes the
        extra rule not just wrong but redundant: a knob wandering on noise
        around its seed has a tail mean ≈ its seed automatically, so the
        filter can only ever act on knobs that genuinely moved — exactly the
        ones it must not touch. Bake the whole vector; let the gate judge
        the whole vector. If it fails, decompose *then* — that is what
        8.5's guard-off arm did, and it is a diagnostic on a rejection, not
        a bake-time heuristic. Then one `[0,3]` gate vs the pre-wave head.
        If the gate loses WITH fail-soft, re-gate once at the fitted values
        without fail-soft (two binaries, no new tune) — that closes 8.11
        permanently either way.
  - (b) **`config_lmr` family — ONLY if the 10.2.5 capstone is REJECTED**
        (if it lands, its own SPSA has just fitted the LMR surface and
        this night does not exist). Optional zero-tune rider either way:
        `cutoffCnt` at hand-picked SF-shaped values, one flag-style gate,
        no family re-tune around it (8.6's trap was self-play aggression
        drift, which the schedule fix does not address).
  - (c) **`config_history`/`config_histcov` — CUT by default.** 8.4's fit
        is the most recent, won its gate, and history knobs interact least
        with the selectivity group. Pre-registered trigger to resurrect it
        as one night: (a)'s fit moved ≥3 knobs by >20% of range AND gated
        ≥ +5 — i.e., only if the frozen schedule demonstrably left big
        money in a *recently-fitted* group's neighbours.
  - **Explicitly NOT retried:** 8.10 mop-up gating (eval-semantics failure,
    no SPSA in the loop) and any 8.6-style LMR-family SPSA bundle (the
    trap is self-play reward hacking, not the anneal). **Not re-tuned:**
    `tm`/aspiration (10.2's own bundled SPSA covers them), anything the
    capstone fits itself.
  - **Total SPSA budget before 2.4.0:** (a) is **2 nights** (5,000 iters,
    ~40 h) not one — the modelling says a single night at ~1,500–2,500
    iterations is inside the "barely beats its seed" regime and would be
    wasted. Worst case adds (b) 2 nights + (c) 2 nights = **2–6 nights**,
    plus the tunes intrinsic to 10.2/10.2.5 which exist regardless. Best
    case: 2 nights, one gate. **Every future tune in this project inherits
    the 5,000-iteration floor** — a 1,000-iteration SPSA is not a cheap
    tune, it is a null result with a bake attached.
  - **⚠ Prior, from the cross-engine review 2026-07-27 — keep this step
    SMALL and resist growing it.** Two independent lines of evidence say
    re-tuning already-tuned constants is low-EV:
    1. **Reckless has ZERO active tunables.** Its `parameters.rs` carries
       the SPSA macro scaffolding (`define!`, `set_parameter`,
       `print_options` behind a `spsa` feature) but the macro is **never
       invoked** — 34 lines of dormant infrastructure, no declared knobs,
       nothing for a tuner to move. Its search constants are hardcoded,
       and its recent accepted work is mechanism and *simplification*
       (aspiration-window shrink on stable searches, NMP TT-bound fix,
       "simplify away checking for quiet moves in SEE pruning"). A top
       Rust engine is gaining strength with no SPSA campaign at all.
       (Caveat: it is NNUE, so its EVAL is learned rather than
       hand-tuned — but its SEARCH constants are hardcoded too, and that
       is the part we tune.)
    2. **Our own ledger says the same.** Accepted Elo has come
       overwhelmingly from mechanism and speed — 8.13 +102.78 @4T, 8.2a
       +30.75, 8.1 +22.13, 10.3 +20.31, plus +4.56% NPS ≈ +9 — while the
       SPSA-led items net near zero or negative: 8.4 +6.01 (and that
       bundle carried a mechanism change), 8.5 +1.43 (rejected), 8.6
       −7.78, 8.7 −7.29. **24 archived tuner runs** for that.
    The resolution is not "SPSA is useless" — tuning *untuned* constants
    paid well early (Phases 2–5, and Phase 4's Texel fits at +42.5…+65.0).
    It is that **the marginal value of RE-tuning an already-fitted group
    is small**, exactly what a noise floor predicts. So 10.4.6 keeps its
    kill-checkpoint, and the queue ahead of it — 10.2.5's mechanism,
    10.2's aspiration/TM, 10.4's menu, and any speed work — should be
    preferred whenever machine time is contested.
  - **⚠ Honest EV note.** Finding 3 means 10.4.6's value is genuinely
    uncertain, not merely unmeasured: if the current values already sit
    inside the noise floor, the best possible outcome is a wash and the
    likely one is a small regression the gate then rejects — 2 nights spent
    to learn "we were already there". The schedule bug makes it *plausible*
    they are outside the floor (tunes froze near their seeds rather than
    converging), but that is an argument, not evidence. The 1,500-iteration
    walk-back checkpoint is what converts it into evidence for ~13 h of
    machine time, and it is the reason this item is scheduled LAST, after
    every higher-EV 2.4.0 item has had the machine.
- **⚙ PHASE 10 EXECUTION ORDER — REVISED 2026-07-28 after the eval-parity
  measurement (numbers are frozen and do NOT imply order, §S6):** **10.0
  decomposition FIRST** (cheap, and it redirects the two biggest items) → 10.1
  (bookkeeping, no games) → **10.4.6 re-fit PROMOTED** (where a mis-tuned
  selectivity surface would live) → **10.2.5 capstone**, re-scoped by 10.0's
  result → **10.2** aspiration/TM, priority conditional on 10.0(b) → 10.4 menu
  picks → **10.4.3 Texel re-fit DEMOTED** (eval measured at parity, so it is
  cheap insurance rather than the lever) → 10.5 gauntlet + release.
- **10.6 Guarantee at least one `info` line before every `bestmove` (added
  2026-07-29; deferred out of the 2.3.0 fix deliberately — it is an engine
  behaviour change and the release was mid-flight).**
  `info depth` is emitted only when an iteration COMPLETES, so a search whose
  budget expires mid-iteration returns `bestmove` having reported nothing at
  all. `threaded_go_nodes_returns_bestmove_and_reports_nodes` hit exactly this
  on a windows release runner at `go nodes 4096` with Threads=4 (the shared
  budget is consumed ~4x faster), and the test was fixed by raising the budget
  rather than changing the engine.
  **Why it may be worth doing:** it is legal UCI but unhelpful — GUIs that
  render a search line get nothing, and node/depth telemetry silently vanishes
  for short searches. Stockfish always emits at least one line.
  **Shape:** on the stop path, if no iteration has been reported yet, emit the
  current partial best (depth, score, nodes, pv) before `bestmove`.
  **Gate:** bench-identical by construction (output only, no search change), so
  it needs no SPRT — a fingerprint check plus a UCI-protocol test that a
  1-node search still produces one `info` line.
- **10.5 Gauntlet + release (2.4.0), then Phase 11 and the Phase-12 NNUE
  program.** The decision here is architecture/compute sizing, not whether
  to avoid NNUE: the large HCE fallback is considered only after a real
  Phase-12 prototype has failed or stalled.

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
| `tools/texel/datagen.ps1` | self-play datagen (node-limited; concurrency 24 OK) |
| `tools/texel/extract.py` | PGN → `FEN;target`; `--balance-phase`; 6.2.0 adds quiet-filter + blend |
| `tools/texel/sample_fens.py` | Beast `positions.txt` (read-only!) → EPD book |
| `rarog-texel --tune <group> <train> <holdout> [out] [--epochs N --lr X --l2 X --max-positions N --from-cp --fix-k K]` | Texel fit; `--verify` reconstruction; `--buckets` per-bucket loss; `--tune-kingsafety` nonlinear KS |
| `tools/texel/bake_params.py <dump>` | bake a full dump into `src/eval.rs`; verify by bench-match (tune-binary-on-dump == baked build) |
| `tools/texel/data/beast_seed.epd` | diverse 100k-opening EPD book for datagen |
| `tools/books/UHO_Lichess_4852_v1.epd` | SPRT/SPSA/gauntlet opening book (adopted 2026-07-17, same day as Basilisk) — the SF/OpenBench-standard Unbalanced Human Openings: 2,632,036 positions, 3–4 moves deep, curated to the +0.48–0.52 White-edge band, played from both colours per pair (symmetric ⇒ unbiased but decisive). Replaces the balanced 4-move PGNs, which cost twice over: SuperGM's 2,668 lines were exhausted by any run > 5,336 games (7.2b recycled 23% of pairs → optimistic error bars), and balanced openings kept the draw rate at 56% (43% dead pairs). UHO cuts draws to ~35–45% ⇒ SPRTs resolve in substantially fewer games. **Two earlier same-day judgments corrected within hours:** (i) "book size is the issue, draw rate is healthy" — reuse was the *acute* flaw, but decisiveness was the larger standing tax; (ii) "UHO only at a phase boundary" — wrong, since every SPRT/SPSA is a self-contained A-vs-B, only *cross-run* draw-rate/Elo magnitudes lose comparability, verdicts don't. weather-factory takes the EPD natively (format from extension), so tune→confirm stays unified (principle #7). Caveats: absolute draw rates / logistic Elo not comparable to pre-UHO runs; gauntlets for CCRL-comparable estimates should use `-Book tools/books/IM_4mvs.pgn` (balanced, 11,172 unique lines, the audited fallback) |
| `wac [depth]` (engine command, like `bench`) | WAC-300 tactical suite; deterministic solved count at fixed depth (default 10). Regression telltale for Phase-8 selectivity work; floor test in `tests/wac.rs` |
| `D:/code/net_trainer` | Phase-12 NNUE training stack (bullet, CUDA GPU): `tools/datagen.py` / `extract_nnue.py` → `net-trainer convert/shuffle/train` → `quantised.bin` |
| `D:/code/net_trainer/docs/nnue_format.md` + `models/test/` | the net consumer contract + integer-exact conformance vectors (12.1's acceptance gate); reference impls in `examples/` |
| `D:/code/hydra/tools/texel/data/sf_*.csv` | SF-60k cp labels (2M; rejected for Rarog — lesson 1) |
| `analysis/{infra,search,hce}_analysis.md` | Codex 5.6 audit (2026-07-13, at `ff21dc1`); basis of Phases 7–14. `search_analysis.md` verified line-by-line at head + fully merged 2026-07-14 (→ 7.5/7.6, 8.2–8.9, 10.1/10.2/10.4, Phase-14 SMP). `hce_analysis.md` merged same day after live re-verification (→ 7.4, 8.5c, 9.6, 11.1, Phase-12 ladder, Phase 13) — its §7/§9 fitted-value tables quote the rejected 6.2.2 refit, and two consequence claims are disproven; see lesson 12 |

**Milestones:** M1 SF-capped-2600 ✅ · M2 Basilisk 1.5.0 ✅ (2.2.0 gauntlet) ·
M3 ≈ 3150+ (Critter 1.6a) — the multi-cycle grind target.

**NNUE boundary rule:** never let the search know how the eval works; if a
pruning condition needs eval internals explained, it's a boundary violation.










