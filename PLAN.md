# Rarog development plan

This is the maintainer-facing source of truth. `GUIDE.md` is the concise
operational mirror; `README.md`, `CHANGELOG.md` and release notes are
user-facing and must not contain experiment bookkeeping.

## 1. Current state

| Item | State |
|---|---|
| Branches | `master`/`v2.3.1` at `a5fd288`; `development` carries the 2.4.0 cycle. |
| Released baseline | Rarog **2.3.1**, bench-13 **5,173,540**, EBF **2.406**. |
| Clean accepted development baseline | RootMove + zero-reduction LMR + accepted selectivity fit + frozen Texel refresh; bench **6,502,902**, EBF **2.449**, `rarog-p1043-base-pext-pgo.exe`. |
| Evaluation | Accepted HCE remains in the baseline, but all new HCE strength work is frozen. Phase 9 is the optional last HCE fallback and runs only if NNUE is abandoned. |
| Active jobs | 36,400-game Rating Tournament and registered 5,000-iteration aspiration SPSA. The tournament's `2.4.0-dev` includes interim unfinished SPSA constants. |
| Next release | **2.4.0 at Phase 4.10**, after the complete pre-NNUE search and target gate. |
| NNUE release | **2.5.0 at Phase 6.7**, followed by the Phase-7 frontier cycle. |

### Live rating evidence — provisional 2026-08-05

At 8,626/36,400 games (~1,232 per engine):

| Engine | Rating | Gap from `2.4.0-dev` |
|---|---:|---:|
| Houdini 1.5a | 3217 | +250 |
| Critter 1.6a | 3190 | +223 |
| Rybka 6 | 3178 | +211 |
| Rybka 5 | 3156 | +189 |
| Rybka 4 | 3102 | +135 |
| Rybka 4.1 | 3088 | +121 |
| Basilisk 1.9.3 | 3006 | +39 |
| **Rarog 2.4.0-dev** | **2967** | — |
| **Rarog 2.3.1** | **2956** | −11 |
| Rybka 3 | 2928 | −39 |

The +11 pool Elo over 2.3.1 is compatible with the expected small gain and
with noise; it cannot be attributed while aspiration SPSA is unfinished.
Houdini 2.0c and Fritz 16 are absent and must be added to Phase 4.10. Closing a
~190–250 Elo search-only gap is aggressive and cannot be promised. The phase
stays open if any direct target gate fails.

## 2. Development process

### Responsibilities and commits

```text
Model  -> inspect, implement, locally verify, update PLAN + GUIDE, commit.
User   -> run long SPSA/SPRT/gauntlet/datagen jobs and return final artifacts.
Model  -> accept/revert from the registered verdict, update docs, commit.
```

- Commit after every completed plan step with an imperative subject and useful
  experiment body. No co-author trailers; never push unless asked.
- Update `PLAN.md` and `GUIDE.md` together.
- Preserve unrelated user changes. A dirty binary records its diff hash and
  cannot become a release baseline.
- While the two active jobs occupy 14 physical cores: no bench, NPS, PGO,
  SPRT, datagen or competing game workload.

### Required gates

| Change | Evidence |
|---|---|
| Behaviour-neutral refactor/test/tooling | `cargo fmt --check`, `cargo test`, relevant Python/PowerShell tests, exact bench fingerprint |
| Correctness repair changing play | Deterministic regression + tactical/mate/endgame suites + strength gate unless unreachable in legal play |
| Strength mechanism | Registered `[0,3]` nElo SPRT at `3+0.03` against current accepted head; broad/risky bundles use `[-3,3]` |
| Non-inferiority/simplification | `[-3,0]`; H1 supports non-regression |
| Speed-only | Bench-identical plus pooled/interleaved PGO NPS after identical-binary self-pair |
| Root/TM/SMP | 1T STC, 1T `10+0.1`, 4T `10+0.1`, zero forfeits and recorded topology/hash |
| Harness change | 30k identical-binary calibration at 1T; 4T calibration where applicable; full 95% nElo CI inside ±5 |
| Phase boundary | Clean reproducible PGO, correctness matrix, prior-release cumulative match and external target cohort |

Use `strength-v1` adjudication and the paired UHO book. Record engine/source
SHA, compiler/PGO manifest, binary/book hashes, TC, threads, hash, concurrency,
affinity and adjudication. SPRT decides strength; node counts, WAC, static loss
and telemetry explain it.

### SPSA budget

1. Finish the already-active aspiration SPSA unchanged; it is a sunk in-flight
   experiment, not permission for more piecemeal tuning.
2. **One additional pre-NNUE search SPSA:** Phase 4.9, after the complete
   search architecture freezes; diagnostics select ≤24 coordinates.
3. **One post-NNUE search SPSA:** Phase 7.3, after the retained NNUE
   architecture/scale freezes.
4. Any further run needs explicit evidence that the prior fit could not
   identify the required parameter class. No HCE coordinate enters these runs.
5. Discrete mechanisms use A/B switches or small grids. De-tuned mechanisms
   may land inert and be tested with the joint fit plus post-fit ablations.
6. SPSA proposes; clean PGO SPRT accepts. Estimator, horizon, bounds and stop
   rule are registered before launch—no post-hoc tail choice.

## 3. Durable lessons

1. External transfer beats additive self-play Elo. Report accepted arms, never
   sum them into a rating prediction.
2. Mechanisms and consumer constants form one system; fit after architecture.
3. A smaller/deeper tree can make worse decisions. Measure best-move recall,
   contradiction and fixed-node quality, not only node savings.
4. Bounds require provenance: static eval, stand pat, qsearch moves, ProbCut,
   null, reduced and full searches do not have equal authority.
5. Bench identity proves behaviour, not speed. Pool independent PGO builds.
6. Canaries catch semantics but are not strength oracles.
7. Root aspiration, time management, fallback and SMP cannot maintain separate
   incompatible confidence models.
8. Multi-thread strength/clock safety is a separate deployment condition.
9. Tuned-off features stay implemented through the post-NNUE fit, but they do
   not justify pre-NNUE HCE work.
10. Git/version history is the archive; GUIDE is a forward overview.

## 4. Released phases

These sequential numbers replace the old roadmap. Old numbers remain
historical references in git and released changelogs only.

### Phase 1 — Foundations and robustness — CLOSED (2.0.x–2.1.0)

Built the Rust board/UCI/PVS/qsearch/TT/history/SEE/Syzygy/time/SMP stack,
testing and release infrastructure; repaired early search, legality, draw,
mate and clock behaviour.

### Phase 2 — Evaluation and search expansion — CLOSED (2.2.0)

Delivered the major HCE program and associated search/tuning pipeline. Staged
self-play showed +316 Elo and external transfer about +240. Later evidence
showed further HCE fitting was no longer the best investment.

### Phase 3 — Correctness, search wave and reproducible builds — CLOSED (2.3.0/2.3.1)

Banked correctness, history/ordering, in-check selectivity, 4T SMP, CI,
reproducible PGO and clean-code work. The boundary campaign measured +76/+78
Elo at 1T and +194 at 4T over 2.2.0 with zero forfeits. 2.3.1 restored the
Windows ARM64 PGO asset without changing search.

## 5. Phase 4 — Evidence-coherent pre-NNUE search (→ 2.4.0)

### Objective and source references

Build the strongest evaluator-agnostic search possible, then directly beat
every installed Rybka, Critter 1.6a, Houdini 2.0c and Fritz 16. Basilisk is the
first rung, not the target. No HCE feature/weight/Texel work is permitted.

Pinned design references:

- [Stockfish search.cpp](https://github.com/official-stockfish/Stockfish/blob/762dd1da9a5db458180b2c5db6c53dc40ec61e1a/src/search.cpp)
- [Reckless search.rs](https://github.com/codedeliveryservice/Reckless/blob/d6603046e76d66edd43622ded23458da1af50c68/src/search.rs)
- Stockfish [stand-pat TT repair](https://github.com/official-stockfish/Stockfish/commit/bb4eb04a), [PV-IIR repair](https://github.com/official-stockfish/Stockfish/commit/e20ef7ed), [TT mismatch penalty](https://github.com/official-stockfish/Stockfish/commit/319d61ef)
- Historical [null-move/TT provenance](https://talkchess.com/viewtopic.php?t=33679) and [`lmrDepth`](https://talkchess.com/viewtopic.php?t=63521) discussions

References provide invariants and hypotheses, never copied constants.

### Banked 2.4 baseline

| Work | Result |
|---|---|
| Search-accuracy decomposition | Evaluation/speed did not explain Basilisk gap; Rarog was too selective for resulting decision quality. |
| Broad selectivity fit | +15.33 ±7.34 nElo; accepted. |
| Zero-reduction LMR | +9.13 ±5.45 nElo; accepted. |
| Persistent `RootMove` | Bookkeeping retained, consumers incomplete. |
| Frozen Texel refresh | +11.56 ±5.19 Elo; accepted baseline, no further HCE work. |
| Clean head | 6,502,902 / EBF 2.449. |

These are individual mechanism gates, not additive proof of the live binary.

### Current audit and missing symbiosis

| Area | Rarog today | Repair |
|---|---|---|
| TT evidence | 10-byte entry stores score/raw eval/move/depth/bound/PV/age, no producer provenance. | Compact provenance/consumer capabilities while preserving density if possible. |
| Qsearch → main | Stand-pat/pruning values can become depth-0 bounds; `EvalPruneTtMinDepth=0` lets them refine pruning through depth 8. | Separate raw/corrected/stand-pat/searched evidence. |
| ProbCut → singular | Stores margin-normalized score at `depth-3`; singular accepts lower/exact at `depth-3`. | Store actual speculative result; forbid singular authority. |
| NMP | Verification disables null only at its root; descendants re-enable it. Missing subtree suppression, cut-node/potential-singularity/raw-eval/decisive guards. | Correct the verification contract before margins. |
| IIR | Can reduce PV nodes with no TT move, starting depth 4. | Restrict by node role/evidence and expose debt. |
| `tt_pv` | One inherited bit disables RFP, razor, NMP and ProbCut together. | Per-mechanism eligibility predicates. |
| Selectivity | Later pruning does not consistently use real prospective LMR depth; quiet checks receive a fixed 32,000 bonus. Prior LMR re-search ≈1.8%. | One history-aware `MoveEvidence` pipeline and forcing-check taxonomy. |
| In-check | Comment says late evasions reducible; active LMR still has `!in_check`. | Resolve mismatch with safe evidence-based evasion handling. |
| Correction/history | Continuation correction is one previous `(piece,to)` slot; capture guard defaults off; prior sample ≈52.8% capture-caused updates. | Attribution guards, true compact continuation pairs and confidence. |
| Root | Mean/mean-square/PV/nodes/fails are collected but barely consumed by aspiration/TM. | One completed root-confidence model. |
| SMP/fallback | Shared TT couples workers; generic diversification saturated. Partial/decisive fallback ownership remains weak. | Pool instability for timing; completed main/root result owns move and score. |

### Cross-feature invariants

1. Every result is typed: full, verified reduced, qsearch move, stand pat,
   ProbCut, null or incomplete.
2. Every consumer declares accepted evidence/bound/depth/node role.
3. LMP/futility/SEE/LMR share one prospective depth and monotone thresholds.
4. History/correction learn only from completed attributable searches.
5. Aspiration/TM/fallback/SMP share one completed root snapshot.
6. Joint-fit mechanisms remain independently ablatable.

### 4.0 — Close and freeze the two live experiments

Finish the 36,400-game tournament and registered 5,000-iteration aspiration
SPSA unchanged. Archive binaries/manifests/config/book/PGN and SPSA state,
seeds/trajectory/logs/final estimator. Interim constants in `2.4.0-dev` are not
accepted evidence. Bake the predeclared estimator once, build clean PGO and
prepare its registered gate against `rarog-p1043-base-pext-pgo.exe`; if it
fails, restore the clean baseline.

### 4.1 — Diagnostic substrate and interaction map

Extend deterministic sampled traces for TT producer/consumer/contradiction;
stand-pat/qmove; NMP nesting/raw-corrected-TT eval; ProbCut/singular reuse; IIR
and extension debt; move stage/prospective depth/pruning overlap/best-move
recall; correction capture/collision/saturation; and root variance/gap/effort/
fails/fallback/worker instability. Add shadow predicates for 4.2–4.7.
Diagnostics off preserves fingerprint; on preserves nodes/best moves with
bounded overhead.

### 4.2 — Result evidence and TT contract

Introduce transient `OutcomeKind`, `NodeEvidence` and `MoveEvidence`. Compare
compact persisted provenance in spare `flag_age` capacity with stack-only
metadata; widen only if both fail. Audit aging/replacement, local/shared TT,
mate/rule-50 conversion and all readers. Publish consumer capabilities for
cutoff/eval refinement/NMP/IIR/ProbCut/singularity. Shadow-test confidence/
depth penalty for inexact bounds contradicting the current window. Correctness
and bench first; `[0,3]` if active semantics change.

### 4.3 — Qsearch and ProbCut evidence hygiene

Do not manufacture searched authority from no-TT stand pat. Keep depth-0
pruning estimates out of deep main consumers; searched qmoves retain limited
qsearch capability. Store actual ProbCut result with speculative provenance
and measured depth; never authorize singularity/exact learning. Stage complete
in-check qsearch ordering and test capture/SEE history plus coherent delta/
SEE/futility after storage is correct. Gate useful arms `[0,3]`, combined
`[-3,3]`.

### 4.4 — NMP, IIR and singular cooperation

Add subtree null suppression, raw-eval/non-decisive/material and cut-node
guards, potential-singularity protection and zugzwang tests. Compare raw vs
TT-adjusted null windows; nested verification nulls are forbidden unless
proven. Keep IIR off PV-following nodes, restrict by role/TT quality and expose
debt. Singular requires compatible full-search evidence, separate single/
double rules and extension caps. Replace blanket `!tt_pv` with per-mechanism
eligibility. Test isolated `[0,3]` arms then `[-3,3]` joint.

### 4.5 — History and correction attribution

Prevent capture/speculative/null/aborted outcomes from training quiet
correction. Compare exclusion/scaled/noisy residual; implement true compact
2/4-ply continuation-correction pairs. Add threat, quiet/noisy, check/evasion
or halfmove contexts only with held-out unique signal. Centralize saturation/
aging and prevent correction double-counting across eval/pruning/reduction. No
dedicated SPSA; final weights enter 4.9.

### 4.6 — Unified prospective-depth selectivity

Create pre-move `MoveEvidence` with check/evasion/capture class, node/TT
evidence, SEE, histories, correction confidence and extension/IIR debt. Derive
LMP, futility, SEE pruning and LMR from one history-aware prospective depth;
preserve the accepted zero-reduction floor. Replace universal quiet-check
bonus/bypass with forcing/safe/losing check classes. Resolve late-evasion
`!in_check` mismatch, add attributable post-LMR feedback and track pruning
overlap/best-move recall. Keep switches ablatable for 4.9.

### 4.7 — One root-confidence model

Derive completed-iteration confidence from per-move mean/mean-square, gap, PV,
best-move age, effort, fail direction/count and depth. Use it for bounded
asymmetric aspiration and TM without double-counting. Abort returns last
completed legal evidence; incomplete mate/win/loss never becomes authoritative.
Pool worker instability for time, not result ownership. Gate aspiration
`[0,3]`; root/TM/SMP `[-3,3]` at 1T STC/LTC and 4T LTC, zero forfeits.

### 4.8 — Throughput, TT and parallel scaling

Profile accepted semantics at 1/2/4/8T: NPS, time-to-depth, TT replacement/
contention, root stability and strength. Audit TT layout after provenance;
hoist/batch proven invariants only. Regenerate PGO after final shape. Every
speed arm uses pooled/interleaved builds. Do not reopen generic worker
diversification without a specific measured independent-work failure.

### 4.9 — Single consolidated pre-NNUE search fit

Freeze architecture and generate configuration from the live parameter source.
Sensitivity/collinearity selects ≤24 coordinates spanning prospective depth/
pruning, NMP family, correction/history, qsearch and root/TM. Run one registered
5,000-iteration SPSA, one estimator and one bake; exclude HCE, dead/off and
redundant knobs. Clean PGO + fmt/tests/telemetry + `[0,3]` against pre-fit
architecture. Post-fit switch ablations must show no harmful subsystem hidden
by compensation.

### 4.10 — Cumulative target ladder and release 2.4.0

Beat the clean 4.0 baseline cumulatively at 1T STC/LTC and transfer at 4T with
zero forfeits; pass correctness/tactical/mate/zugzwang/TB/provenance/history/
extension/root telemetry. Then run paired Rarog, Basilisk 1.9.3, every
installed Rybka (minimum 3/4.1/4/5/6), Critter 1.6a, Houdini 1.5a/2.0c and
Fritz 16. Every required target needs a logistic-Elo lower bound above zero at
primary 1T, with Holm-adjusted 95% family-wise confidence; confirm 1T/4T LTC.
Rating-list inference cannot replace a missing engine. If all pass: clean ISA/
PGO/default-UCI/docs/archive, version/commit 2.4.0, no push/tag. Otherwise Phase
4 remains open.

## 6. Phase 5 — NNUE runway

### 5.0 — Frozen measurement corpus

Freeze quiet/tactical/endgame/rule-50, phase-balanced and search-disagreement
cohorts from released 2.4.0. Record teacher SHA/settings/labels/hashes and
untouched split IDs; define `net_trainer` integer contract.

### 5.1 — Per-ply state and dirty pieces

Move reversible state into a compact per-ply structure; record exact dirty
pieces for quiet, capture, EP, promotion/castling and define null. Randomized
make/unmake compares board, keys, attacks and state after unwind.

### 5.2 — Accumulator scaffolding

Add per-thread/per-ply accumulator ownership, refresh markers and debug full-
recompute seams while HCE search remains identical. No network inference yet.

### 5.3 — Trainer preflight

Pin `D:/code/net_trainer`, Bullet, Rust/CUDA/driver/GPU; verify conversion,
shuffle, deterministic splits/manifests, reference vectors and exact resume or
forbid resume. Malformed CLI/conversion loss fails loudly.

### 5.4 — Runway gate

Bench-identical state/search, fmt/tests/sanitizers/random unwind and reproducible
corpus/pilot. Create the integration branch only after it passes.

## 7. Phase 6 — Baseline NNUE and release 2.5.0

### 6.0 — Harden trainer and conformance

Strict CLI, train/validation/untouched-test splits, checkpoint selection,
manifests/hashes/seeds/resume and exact Rust/NumPy/engine integer references.

### 6.1 — Controlled data at scale

Generate 30–60M unique teacher positions with search score/WDL; A/B WDL blend,
node budget, natural finishes and behavioural disagreement mining.

### 6.2 — Baseline networks

Train documented baseline widths/buckets with ≥2 seeds; validation selects
within a run and untouched cohorts are evaluated once. Identify every net by
complete architecture/data/trainer manifest and SHA.

### 6.3 — Scalar integration

Strict net size/layout validation, embedded release net and optional validated
EvalFile. Full recompute matches references/large FEN corpus exactly and search
keeps an evaluator-agnostic boundary.

### 6.4 — Incremental accumulator and SIMD

Use Phase-5 dirty deltas per thread/ply; debug/sanitizers compare full
recompute after randomized move/unmove. Prove integer bounds; add exact
portable/shipped SIMD kernels, benchmark components/full search and rebuild PGO.

### 6.5 — Baseline architecture loop

Compare data amount, blend, width/buckets, LR and duration one variable at a
time with two-seed evidence. Diagnose contract/data/training/architecture when
a bring-up net loses.

### 6.6 — Provisional search-scale calibration

Inspect score/correction/pruning telemetry and change only gross safety margins
through isolated gates. Comprehensive search SPSA waits for Phase 7.

### 6.7 — Release 2.5.0

Default embedded NNUE beats 2.4.0 at STC/LTC, transfers at 4T, has zero
incremental/full mismatches and passes external/ISA/net-metadata gates. Update
user docs/version and commit; no push/tag.

## 8. Phase 7 — NNUE frontier and final search fit

### 7.0 — Residual and disagreement analysis

Measure phase/material/king/tactical/endgame residuals, calibration,
teacher-search disagreement and refresh cost; choose work from evidence.

### 7.1 — Data and label frontier

Scale/deduplicate data, natural finishes and hard-position mining; A/B teacher
depth, WDL blend, labelled subsets and disagreement replay with untouched sets.

### 7.2 — Architecture ladder

Test king/perspective buckets, threat/material/output inputs, width/activation
and refresh-friendly variants one axis at a time. Require two seeds, exact
conformance, NPS and SPRT; static loss alone cannot promote.

### 7.3 — Single post-NNUE search SPSA

After architecture/scale freezes, select ≤24 search/history/correction/qsearch/
root coordinates whose optima may have moved. Run the normally final search
SPSA, clean bake/PGO/SPRT/LTC/4T and ablations.

### 7.4 — Frontier release gate

Beat 2.5.0 and test contemporary Stockfish, Reckless, PlentyChess and another
independent engine with calibrated odds if necessary. Archive all manifests
and release the next strength version when the matrix passes.

## 9. Phase 8 — Scaling, platforms and product completeness

### 8.0 — High-thread/NUMA scaling

Measure 8T+ topology, first-touch/NUMA, TT/accumulator sharing, root stopping
and false sharing while preserving 1T/4T.

### 8.1 — Memory and runtime dispatch

Optimize TT/network placement, pages/prefetch and runtime ISA dispatch with
exact scalar parity and real-hardware tests.

### 8.2 — Protocol/platform completion

Add demanded product work such as Chess960 and platform/ARM64/NEON parity.

### 8.3 — Scaling release

Publish only after thread/platform matrix, clock safety, net parity and user
documentation pass.

## 10. Phase 9 — Optional HCE fallback (only if NNUE is abandoned)

This last phase may never run. Enter only after serious Phase-6/7 contract,
data and architecture retries fail and the user explicitly abandons NNUE.

### 9.0 — Failure review and scope decision

Document what failed and distinguish trainer/data/integration/compute from
evaluation capacity. Re-enable no HCE work without approval.

### 9.1 — HCE residual program

Select a few independent king/threat/endgame/complexity features from frozen
residuals, not a feature-name menu. Each structure change includes its refit.

### 9.2 — HCE fit and release

Run one evidence-selected HCE fit and the complete external matrix; preserve
NNUE branches/artifacts for later return.

## 11. Release checklist

1. Phase gate and direct prior-release match passed.
2. Clean tree; version/default UCI/bench manifest recorded.
3. fmt/tests/tactical/mate/TB and platform CI pass.
4. Fresh revision-matched PGO/ISA assets smoke-tested; manifests archived; no
   tune-only options.
5. NNUE releases record net/architecture/trainer/Bullet/data hashes and exact
   incremental/reference parity.
6. Update user-facing changelog/readme only for visible changes; release notes
   contain no internal phase history.
7. Commit locally. Do not tag or push.

## 12. Common commands

```powershell
cargo fmt --check
cargo test
.\tools\build_test.ps1 -Suffix <name>
.\tools\sprt.ps1 -EngineA <candidate> -EngineB <baseline> `
  -NameA Candidate -NameB Baseline -Elo1 3
.\tools\nps_ab.ps1 -EngineA <candidate> -EngineB <candidate>
.\tools\nps_ab.ps1 -EngineA <candidate> -EngineB <baseline> -Rounds 12
.\tools\spsa.ps1 -ConfigGroup <group> -LaunchOnly -Iterations 5000
```
