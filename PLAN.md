# Rarog development plan

This is the maintainer-facing source of truth for future work. `GUIDE.md` is
the concise operational mirror; [`EXPERIMENTS.md`](EXPERIMENTS.md) is the
indexed, conditional evidence ledger. `README.md`, `CHANGELOG.md` and release
notes are user-facing and must not contain experiment bookkeeping.

## 1. Current state

| Item | State |
|---|---|
| Branches | `master`/`v2.3.1` at `a5fd288`; `development` carries the 2.4.0 cycle. `origin/arm_fix` is a stale two-commit experiment (`0ddc8e5`, `3ee4660`) based on 2.3.1; Phase 4.8 inventories it, but it must not be merged wholesale. |
| Released baseline | Rarog **2.3.1**, bench-13 **5,173,540**, EBF **2.406**. |
| Clean accepted development baseline | RootMove + zero-reduction LMR + accepted selectivity fit + frozen Texel refresh; bench **6,502,902**, EBF **2.449**, `rarog-p1043-base-pext-pgo.exe`. |
| Evaluation | Accepted HCE remains in the baseline, but all new HCE strength work is frozen. Phase 9 is the optional last HCE fallback and runs only if NNUE is abandoned. |
| Active jobs | None. Phase 4.0 closed the incomplete rating observation and rejected the stopped `p102a-snapshot`; do not resume either job. |
| Next release | **2.4.0 at Phase 4.11**, after the complete pre-NNUE search, portability and target gate. |
| NNUE release | **2.5.0 at Phase 6.7**, followed by the Phase-7 frontier cycle. |

### Closed rating observation — last supplied checkpoint 2026-08-05

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

The +11 pool Elo over 2.3.1 was compatible with a small gain and with noise;
the mixed interim-constant binary and incomplete tournament provide no
component attribution and are not a baseline or gate. Phase 4.0 preserves this
checkpoint only as an observation.
Houdini 2.0c and Fritz 16 are absent and must be added to Phase 4.11. Closing a
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
- Before proposing or retrying a mechanism, consult `EXPERIMENTS.md` by
  subsystem. Update its verdict, conditions, lesson and retry trigger in the
  same commit that closes an experiment; keep forward sequencing only here.
- Preserve unrelated user changes. A dirty binary records its diff hash and
  cannot become a release baseline.
- No repository job currently reserves this machine. Start long jobs only when
  a later numbered step explicitly requests one.

### Required gates

| Change | Evidence |
|---|---|
| Behaviour-neutral refactor/test/tooling | `cargo fmt --check`, `cargo test`, relevant Python/PowerShell tests, exact bench fingerprint |
| Correctness repair changing play | Deterministic regression + tactical/mate/endgame suites + strength gate unless unreachable in legal play |
| Strength mechanism | Test the **baked final-PGO candidate once** against the current accepted final-PGO baseline. The default material-gain gate is `[3,10]` nElo at `3+0.03`, capped at **16,000** games; only H1 promotes. H0 or an unresolved cap parks/reverts the candidate. The cap is derived, not chosen — see the drift calibration below. |
| Small knob/probe | Keep inert or fold into its coherent subsystem/Phase 4.10 fit. Same-binary option games may diagnose causality, but are short probes and never a second release gate. |
| Broad architectural bundle | Register `[0,10]` or `[3,12]` on the final PGO binaries according to risk and expected value. Preserve ablation switches; if the bundle fails, ablate rather than pre-gating every component. |
| Non-inferiority/simplification | `[-3,0]`; H1 supports non-regression |
| Speed-only | Bench-identical plus pooled/interleaved PGO NPS after identical-binary self-pair |
| Root/TM/SMP | 1T STC, 1T `10+0.1`, 4T `10+0.1`, zero forfeits and recorded topology/hash |
| Harness change | 30k identical-binary calibration at 1T; 4T calibration where applicable; full 95% nElo CI inside ±5 |
| Phase boundary | Clean reproducible PGO, correctness matrix, prior-release cumulative match and external target cohort |

Use `strength-v1` adjudication and the paired UHO book. Record engine/source
SHA, compiler/PGO manifest, binary/book hashes, TC, threads, hash, concurrency,
affinity and adjudication. SPRT decides strength; node counts, WAC, static loss
and telemetry explain it.

#### Deriving a game budget instead of guessing one

A cap must be able to resolve the bounds it is paired with, or the gate quietly
becomes stricter than it reads. This project's own gates fit

```text
LLR drift per game  ~  8.3e-6 * (Elo1 - Elo0) * (true_nElo - midpoint)
```

calibrated within 1% on three independent runs — RAR-S31 (+5.24 nElo, `[0,3]`),
RAR-S29 (−4.95, `[0,3]`) and RAR-S27 (−2.33, `[0,3]`) — and recorded as RAR-M10.
Games to a ±2.94 boundary is then `2.94 / drift`.

For the default `[3,10]`, midpoint 6.5:

| True effect | Games to boundary |
|---:|---:|
| 12 nElo | ~9,200 (H1) |
| **10 nElo (exactly H1)** | **~14,500 (H1)** |
| 8 nElo | ~33,700 — indifference region, resolves slowly by design |
| **3 nElo (exactly H0)** | **~14,500 (H0)** |

Hence the 16,000 default: it covers a candidate sitting exactly on either
hypothesis, with headroom for the random walk. A 12,000 cap would have parked a
share of the changes the bounds exist to accept, making the effective bar about
12 nElo rather than the stated 10.

Recompute this table before adopting different bounds, and change a cap only
**prospectively** — the calibration is a design tool, never a reason to extend a
run whose games you have already seen.

### SPSA budget

1. The stopped half-run aspiration SPSA and its `p102a-snapshot` gate are
   **closed/rejected** by Phase 4.0; do not resume or tail-select them.
2. **One additional pre-NNUE search SPSA:** Phase 4.10, after the complete
   search architecture freezes; diagnostics select ≤24 coordinates.
3. **One post-NNUE search SPSA:** Phase 7.3, after the retained NNUE
   architecture/scale freezes.
4. Any further run needs explicit evidence that the prior fit could not
   identify the required parameter class. No HCE coordinate enters these runs.
5. Discrete mechanisms use A/B switches or small grids. De-tuned mechanisms
   may land inert and be tested with the joint fit plus post-fit ablations.
6. SPSA proposes; one baked final-PGO SPRT accepts the retained fit. Estimator,
   horizon, bounds and game budget are registered before launch—no post-hoc
   tail choice. A tune/non-PGO SPRT is not an additional prerequisite.

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
8. Machine time is a development budget. Do not spend a full gate proving an
   isolated 3-Elo knob while larger architectural work remains; require a
   material final-binary gain or park it for the consolidated fit.
9. Multi-thread strength/clock safety is a separate deployment condition.
10. Tuned-off features stay implemented through the post-NNUE fit, but they do
   not justify pre-NNUE HCE work.
11. Git/version history is the archive; GUIDE is a forward overview.
12. Cross-compilation is not platform validation. A production asset needs
    target-native execution, exact search agreement, an executable ISA
    contract and same-target performance evidence.

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
| TT evidence | The accepted baseline has no producer provenance. The 4.3c candidate preserves the 10-byte entry while trading one age bit for an explicit speculative bit; age wrap falls 32→16 generations while replacement cost per generation stays 4. | Final-PGO gate the narrow producer/consumer contract; do not generalize the bit into a scalar quality score. |
| Qsearch → main | Stand-pat/pruning values can become depth-0 bounds; `EvalPruneTtMinDepth=0` lets them refine pruning through depth 8. Measured: 67% of sampled stores are depth-0 qsearch and 37% are bare stand-pat. | Separate raw/corrected/stand-pat/searched evidence. |
| ProbCut → singular | The accepted baseline stores a margin-normalized score and infers authority from depth/bound shape. The 4.3c candidate stores the actual fail-high, tags it speculative and blocked 863 otherwise-eligible singular seeds in `bench 13`. | Gate the explicit contract; retain cutoff/eval/move/IIR authority unless separate evidence changes it. |
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

### 4.0 — Close and freeze the two live experiments — **COMPLETE (2026-08-05)**

The rating tournament is archived only at its last supplied 8,626/36,400-game
checkpoint; its mixed interim-constant binary remains observational. The
aspiration SPSA was stopped at iteration 2,510/5,000. Its frozen snapshot at
`ba3170b` changed `21/150/150/5/20/0/0` to
`15/148/149/9/20/8/0` and widened bench from 6,502,902 to 7,047,226 nodes.

The registered `p102a-snapshot` versus `p1043-base` `[0,+3]` gate at
`3+0.03`, 1T, 64 MB and `UHO_Lichess_4852_v1.epd` was manually stopped after
13,000 games at −2.67 ± 3.83 Elo / −4.16 ± 5.97 nElo, W-D-L
3,261-6,378-3,361, LLR −1.83 of ±2.94. This did not reach the formal H0
boundary, but it also did not accept H1; under the registered “H1 accepts,
otherwise revert” rule the snapshot is rejected. `development` already held
the restored baseline constants and still matches 6,502,902 / EBF 2.449, so no
source revert was necessary. The disposable local `strength_test` pointer was
deleted; RAR-S20 retains the exact constants, result and source hash. No p102a
binary, tuner state or result artifact existed on this machine.

### 4.1 — Diagnostic substrate and interaction map — **COMPLETE (2026-08-05)**

`--features diag` now combines the legacy exact event counters with independent,
deterministic 1/1024 samples for main search, qsearch and correction updates.
The sampled map covers TT bounds/contradictions and main/qsearch/ProbCut
producers; stand-pat versus searched qmoves; NMP eval source, verification and
descendant nesting; ProbCut/singular reuse; PV/no-TT/shallow-TT IIR and extension
debt; move stage, prospective/reduced depth, pruning overlap/check exemptions
and winning-move recall; correction residual/capture/collision/saturation; and
root gap/variance/effort/fails/fallback plus worker disagreement/spreads.
Coverage counters expose inert observation points for every 4.2–4.7 consumer.

The implementation is isolated in `src/diag.rs`, call sites compile out of the
production build, and `tools/diag_search_quality.ps1` prints both a grouped
interaction map and raw counters. Gate on this machine: normal and diagnostic
release builds both produced 6,502,902 nodes / EBF 2.449 on `bench 13`; four
independent depth-10 positions matched nodes, score, PV and best/ponder moves.
A paired best-of-three un-POGO timing check measured 2,585,646 versus
2,301,097 NPS (11.0% diagnostic cost, including the older exact atomics); this
bounds use as an offline diagnostic but is not a speed verdict. The sampler unit
test, normal/diagnostic test suites and feature-enabled lint wall pass. Record
future readings and conditional lessons in `EXPERIMENTS.md`; never infer Elo
from counter movement alone.

### 4.2 — Result evidence and TT contract — **COMPLETE (2026-08-05)**

Landed at `47f3ac6` and recorded as RAR-S23: `src/evidence.rs` defines
`OutcomeKind`, `NodeEvidence` and `MoveEvidence`; all seven store sites declare
a producer kind; all thirteen read sites go through named capability predicates
(`cutoff_score`, `refine_eval`, `refine_eval_bound_only`, `allows_singular`,
`too_shallow_to_order`, `is_exact`, `pv_line`). NMP and ProbCut consume TT
evidence only through `refine_eval`, so they gained no separate predicate —
their own eligibility split is 4.4's per-mechanism `tt_pv` work. Mate-distance
and rule-50 conversion now happen exactly once per node. A debug shape contract
plus an exact producer census (reconciled against the independent store
counters) makes a mislabelled store a test failure rather than a later depth
coincidence. Verified bench-identical at 6,502,902 / EBF 2.449 with 96 matching
depth lines against the pre-refactor binary.

Centralizing exposed one divergence worth carrying forward: the main search's
eval refinement enforces a depth floor and a `VALUE_NONE` test, the qsearch
stand-pat path enforces neither. Both are preserved as distinct named
capabilities with a test pinning the difference; unifying them is 4.3.

The registered shadow test of a confidence/depth penalty for window-
contradicting inexact bounds ran at `7815054` and is recorded as RAR-S24. It
returned the opposite of its motivating hypothesis and its result is a design
constraint on 4.3, not a pending task. Exposure is 18.59% of sampled hits
(2.4× the previously reported figure, which counted only the cutoff-eligible
subset). Score consumers are materially exposed — 31.6% of those hits moved
`eval_for_pruning` by a mean 123.7 cp, and 41 of 101 sampled singular attempts
were seeded by one. **But a contradicting entry's move was best 91.79% of the
time against 84.77% for an agreeing entry**, so contradiction improves the
move as an ordering hint while staling the score. Score staleness and move
staleness are not the same property, and one per-entry confidence scalar would
throw away real ordering evidence to fix a scoring problem.

4.2 is therefore closed. Cutoffs need no arm at all: a contradicting entry
cannot cut off, which is unit-tested rather than assumed.

Original scope for reference. Introduce transient `OutcomeKind`,
`NodeEvidence` and `MoveEvidence`. Audit
aging/replacement, local/shared TT, mate/rule-50 conversion and all readers.
Publish consumer capabilities for cutoff/eval refinement/NMP/IIR/ProbCut/
singularity. Shadow-test confidence/depth penalty for inexact bounds
contradicting the current window. Correctness and bench first; `[0,3]` if
active semantics change.

**Persistence is deferred, and the reason is measured (RAR-S22).** There is no
spare `flag_age` capacity: the byte is 5 bits age (`0xF8`), 1 bit `is_pv` and
2 bits bound — 8 of 8. The three candidate slots and their real costs are:

- **Age 5→4 bits** buys one bit. Keeping `entry_quality`'s per-generation
      penalty needs its divisor moved 2→4, which preserves the penalty exactly
      (8/2 = 16/4 = 4) but halves the wraparound horizon from 32 to 16
      generations. `bench` shares one table across its 40 positions and ages it
      once per position, so this **changes the bench fingerprint** and is a
      behaviour change requiring a `[0,3]` gate, not a neutral refactor.
- **`LocalCluster`'s 2 spare bytes** are local-only. `SharedCluster` is a
      6×10 B/64 B struct-of-arrays with a compile-time size assert and no
      padding, so this slot cannot be made backend-symmetric.
- **Widening the entry to 12 B** breaks both cluster invariants (3×10+2=32 and
      6×10=64) and costs capacity on every target.

So 4.2 lands the transient types and the centralized capability predicates at
**exactly current semantics** (bench-identical), which converts 4.3 and 4.4
from scattered condition surgery into single-predicate edits. Persist a
producer class only when 4.3/4.4 show a consumer that cannot be corrected from
stack-local evidence, and price it against the list above.

### 4.3 — Qsearch and ProbCut evidence hygiene

> **The flat depth-floor premise is closed for the main-search eval consumer
> (RAR-S27/S29).** Neither `EvalPruneTtMinDepth=2` nor `=1` met its registered
> acceptance rule, so the default remains 0. RAR-S29 reached formal H0 with LOS
> 2.66%, although its −3.18 ± 3.23 Elo interval narrowly includes zero. RAR-S30
> is an all-depth, overlapping-predicate shadow and cannot causally explain the
> games or establish estimator quality. It only shows that TT refinement often
> changes pruning direction in this bench. Arm C is retired as low priority,
> not inferred equivalent. The independent open defect is explicit ProbCut
> provenance at the singular consumer.

Do not infer a producer from depth/bound/move shape. Preserve the currently
measured depth-0 eval-refinement behaviour until the joint fit. Store the actual
ProbCut result with explicit speculative provenance and measured depth; never
authorize speculative evidence to seed singular verification. Stage complete
in-check qsearch ordering and test capture/SEE history plus coherent delta/
SEE/futility after storage is correct. Gate one coherent baked final-PGO
candidate under the material-gain policy; keep individual arms diagnostic and
ablatable.

Carried in from 4.2 (RAR-S22–S24), with the measurement that justifies each:

- **~~Separate the two eval-refinement capabilities now.~~ DEFERRED (RAR-S29).**
      The asymmetry is real and stays documented in `evidence.rs`, but neither
      tested main-search depth floor was accepted. RAR-S02 separately accepted
      the loose qsearch form at about +6.5 Elo. Keep both defaults and revisit
      only inside 4.10's joint fit, where their consumers can move with them.
- **Deny singular authority to speculative evidence.** The accepted baseline
      stores a margin-shifted ProbCut score at `depth-3`, while singular accepts
      that shape. The 4.3c candidate stores the actual result and requires
      non-speculative evidence for the seed.
- **Penalize the SCORE, never the MOVE.** RAR-S24 measured a contradicting
      entry's move as best 91.79% versus 84.77% for an agreeing entry. Any
      confidence/depth penalty applies to eval refinement and singular
      seeding; move ordering and IIR keep full nominal authority. A single
      per-entry confidence scalar is ruled out by measurement.
- **No cutoff arm.** A contradicting entry cannot produce a cutoff; this is
      unit-tested, so no gate is needed for that path.
- **Price the depth penalty from the slack histogram** (20/19/16/22/8 across
      slack 0/1/2-3/4-7/8+): P=1 blocks 23.5% of contradicting refinements,
      P=2 45.9%, P=4 64.7%, P=8 90.6%. Pick P from this, not by feel.

#### 4.3a — registered arms, landed inert at `d354d02`

Four knobs, all defaulting to pre-4.3 behaviour, verified bench-identical at
6,502,902 on normal, diag and tune builds. A `Default` assert pins the inert
values so baking one becomes a test failure. Sized in RAR-S26 at fixed depth
12 over 4 positions — node counts only, which per lesson 3 explain a gate and
never replace one.

| Arm | Knob | Candidate | Node effect | Moves | Priority |
|---|---|---|---:|---|---|
| A | `EvalPruneTtMinDepth` | 2, then 1 | −43.8% / −15.3% | unchanged on probes | **first** |
| B | `SingularTtDepthMargin` | 2 | −11.8% | differ | second |
| D | `ProbCutStoreDepthAdj` | 4 | +18.2% | differ | third |
| C | `QsRefineMinDepth` | 1 | +5.3% | differ | held |

Arm A is first because it addresses the headline defect directly — a depth-0
qsearch bound refining the eval that RFP, razor and NMP consume at any depth —
and because a 44% node reduction with unchanged probe moves suggests real
waste. Its prior SPSA retained 0 while sitting ON a rail, where a fit is least
informative, so that prior is weak rather than decisive. Arm C is held: it
costs nodes and would gut RAR-S02's accepted +6.5 Elo mechanism, so it needs a
reason beyond symmetry before spending games on it.

These arms ran under the former policy as one non-PGO `tune` binary against
itself with one option changed. That isolates the option but is now classified
as diagnostic evidence: only a baked final-PGO candidate can enter the
accepted baseline, and small standalone knobs are not owed that machine time.

#### 4.3b — arm B resolution — **PARKED, NOT BAKED (RAR-S31/S32)**

`SingularTtDepthMargin=2` reached H1 on the tune binary (+3.35 ± 2.44 Elo,
31,822 games, zero forfeits). It does not isolate ProbCut: margin 2 also rejects
legitimate full-search entries at `depth-3`, while an older/deeper ProbCut entry
can still qualify at a shallower consumer. RAR-S32 established identical bench
decisions and a similar aggregate NPS ratio across build modes; those are useful
transfer diagnostics, not proof of final-PGO playing strength.

Under the material-gain policy, another long gate for an observed ~3 Elo knob is
not a good use of the machine. Keep `SingularTtDepthMargin=3`, retain value 2 as
an inert 4.10 coordinate/ablation, and retire arm D unrun: explicit provenance
solves the producer question without sacrificing unrelated depth bands. This is
a resource-priority decision, not evidence that value 2 is neutral or harmful.

#### 4.3c — persisted provenance and real ProbCut-result handling — **IMPLEMENTED; GATE OPEN**

Implemented one explicit **speculative** bit, not a vague producer-quality bit.
It comes from TT age (5→4 bits); age stride 8→16 and replacement divisor 2→4
preserve the 4-point per-generation penalty, the 10-byte entry and both backend
densities. `OutcomeKind::is_speculative` supplies it, `NodeEvidence` decodes it,
and only singular-seed capability is denied. Cutoffs, eval refinement, move
ordering and IIR retain their existing bound/depth rules.

ProbCut now stores its actual fail-high while returning the existing
conservative adjusted value to the live caller. Local/shared round-trip,
age-wrap and consumer-contract tests pass. Release and diagnostic builds both
bench **6,595,869 / EBF 2.447**; the diagnostic census records **863** actual
speculative singular seeds blocked in `bench 13`. This is one coherent
final-PGO candidate, registered `[3,10]` nElo at `3+0.03`, 1T/64 MB/paired UHO,
maximum **16,000** games; only H1 promotes. Baseline commit is `d00e1ac` at
6,502,902 nodes.

Independently verified before launch (RAR-S33): the fingerprint reproduces on
normal, diag AND tune builds, the census reconciles, provenance round-trips on
both backends, and fmt/clippy×3/tests×3 pass. Gate binaries are
`rarog-43c-pgo.exe` (median of three clean PGO builds) versus
`rarog-d00e1ac-pgo.exe`.

Two things the verdict reader must know. The candidate needs **+1.43% more
nodes** for bench depth 13 *and* runs **−2.45% NPS** — about 4% worse
time-to-depth, a real headwind at ~2 Elo per 1% NPS. A park would therefore
reflect that cost/benefit balance, not necessarily a wrong contract. And this
step bundles the singular-rejection contract with the change from a
margin-shifted to an actual ProbCut stored score, **with no ablation switch on
the score half**, so a failure cannot be attributed between them; add one before
re-testing rather than guessing which half was responsible.

#### 4.3d — in-check qsearch ordering, after 4.3c

Stage complete in-check qsearch ordering, test capture/SEE history and make
delta/SEE/futility coherent. Deliberately sequenced last: the plan requires
storage to be correct first, and RAR-S25 shows the within-horizon producer
split is currently un-inferable, so ordering work would be building on evidence
that cannot yet be attributed.

#### 4.2 refactor speed — **CLOSED (RAR-S28)**

The typed-evidence refactor cost no measurable throughput. Three independent PGO
builds per arm, pooled and interleaved in both directions: bias-cancelled
−0.125% median, implied slot bias +0.025%, all six builds reproducing bench
6,502,902. At ~2 Elo per 1% NPS that bounds the cost near 0.25 Elo. 4.2 is now
clean on behaviour and throughput.

Two method points worth reusing. Run the pooled A/B in **both directions** and
take `(forward − reverse)/2`; it cancels the estimator's slot bias better than a
single self-pair null estimates it (the two disagreed by 0.28pp here, and the
difference was better behaved). And pool builds: cand-arm build medians spanned
0.62% with one build 0.6% below its siblings, which is larger than any effect
worth measuring at this scale.

### 4.4 — NMP, IIR and singular cooperation

Add subtree null suppression, raw-eval/non-decisive/material and cut-node
guards, potential-singularity protection and zugzwang tests. Compare raw vs
TT-adjusted null windows; nested verification nulls are forbidden unless
proven. Keep IIR off PV-following nodes, restrict by role/TT quality and expose
debt. Singular requires compatible full-search evidence, separate single/
double rules and extension caps. Replace blanket `!tt_pv` with per-mechanism
eligibility. Use cheap deterministic/short diagnostic ablations while building
the subsystem, then run one material-gain gate on the coherent final-PGO joint
candidate.

### 4.5 — History and correction attribution

Prevent capture/speculative/null/aborted outcomes from training quiet
correction. Compare exclusion/scaled/noisy residual; implement true compact
2/4-ply continuation-correction pairs. Add threat, quiet/noisy, check/evasion
or halfmove contexts only with held-out unique signal. Centralize saturation/
aging and prevent correction double-counting across eval/pruning/reduction. No
dedicated SPSA; final weights enter 4.10.

### 4.6 — Unified prospective-depth selectivity

Create pre-move `MoveEvidence` with check/evasion/capture class, node/TT
evidence, SEE, histories, correction confidence and extension/IIR debt. Derive
LMP, futility, SEE pruning and LMR from one history-aware prospective depth;
preserve the accepted zero-reduction floor. Replace universal quiet-check
bonus/bypass with forcing/safe/losing check classes. Resolve late-evasion
`!in_check` mismatch, add attributable post-LMR feedback and track pruning
overlap/best-move recall. Keep switches ablatable for 4.10.

### 4.7 — One root-confidence model

Derive completed-iteration confidence from per-move mean/mean-square, gap, PV,
best-move age, effort, fail direction/count and depth. Use it for bounded
asymmetric aspiration and TM without double-counting. Abort returns last
completed legal evidence; incomplete mate/win/loss never becomes authoritative.
Pool worker instability for time, not result ownership. Preserve aspiration,
TM and SMP ablations, but gate the coherent final-PGO root-confidence bundle;
reserve 1T LTC and 4T LTC for its phase/cumulative confirmation, zero forfeits.

### 4.8 — Cross-platform and ISA baseline (`origin/arm_fix`)

Make every shipped x86-64/ARM64 asset a release condition before the final
throughput fit. Local development currently proves only Windows x86-64 native
PEXT on a Ryzen 9 5950X. Rarog's CI and release jobs are already unusually
strong—they execute five OS/architecture cells and build target-native PGO—
but node agreement does not prove intended instructions or preserved speed.

Pinned platform references: Rust supports stable
[AArch64 inline assembly](https://doc.rust-lang.org/reference/inline-assembly.html#supported-architectures),
but its [`target-cpu`/`target-feature`](https://doc.rust-lang.org/stable/rustc/codegen-options/index.html#target-cpu)
contract must be inspected in generated artifacts. Apple requires querying
[`hw.cachelinesize`](https://developer.apple.com/documentation/apple-silicon/addressing-architectural-differences-in-your-macos-code)
rather than assuming it from `aarch64`, and GitHub currently provides native
[Linux, Windows and macOS ARM64 runners](https://docs.github.com/en/actions/reference/runners/github-hosted-runners).

#### Branch and sibling-engine disposition

| Evidence | Finding | Disposition |
|---|---|---|
| `origin/arm_fix` / `0ddc8e5` | Adds AArch64 `PRFM PLDL1KEEP`; the current development prefetch remains x86-only. Also hoists `ATTACKS` `LazyLock` access in two HCE routines. | Reimplement and target-test the prefetch on current development. Preserve the reported +2.51% x86 branch A/B as non-isolated evidence only; the HCE-only hoist is frozen with HCE and is not pre-NNUE work. |
| `origin/arm_fix` / `3ee4660` | Wraps four 32-byte local or two 64-byte shared TT clusters in 128-byte Apple blocks. It has no ARM timing evidence; the logical clusters already cannot straddle a 128-byte boundary at their existing alignments. | Target-measured hypothesis only. Do not port the wrapper until it beats flat storage with identical capacity/indexing. |
| Basilisk `67a987b` | Independently made the same unmeasured TT-wrapper assumption. | Corroborates the question, not the answer; widen the audit to hot shared atomics and actual cache topology. |
| Basilisk build/release contract | Has explicit release-tier documentation, startup feature checks and per-asset manifests; its current PEXT docs/flags also disagree. | Add one Rarog ISA-contract document/manifest and verify any startup guard is itself compiled for a safe baseline. Do not copy Basilisk's mismatch. |

Execute in this order:

1. **Freeze an executable asset contract.** Production assets are Linux/
   Windows x86-64 baseline, AVX2 and PEXT, plus Linux/Windows/macOS ARM64.
   `xtask` correctly separates ISA tier from `--native`; retain that design.
   State the complete instruction contract for each tier. AVX2 and PEXT use
   `target-cpu=x86-64-v3`, while the engine currently checks only BMI2 in PEXT
   builds and has no AVX2 preflight. Either provide a genuinely baseline-safe
   startup guard for every required feature or stop promising graceful
   rejection and make download guidance exact. Baseline artifacts must contain
   no forbidden instructions; optimized artifacts must contain the intended
   AVX2/POPCNT/BMI2/PEXT paths.
2. **Keep correctness target-native and pre-release.** Preserve the existing
   debug/release suite on x86-64 and macOS ARM64 plus bench fingerprints on
   Linux/Windows/macOS ARM64. Add UCI/perft/Syzygy smoke and artifact checks to
   all five cells, and run the production feature/PGO path before the release
   event rather than discovering failures while attaching assets. Record the
   Windows ARM64 LLD/PGO workaround as toolchain-versioned debt and retest it
   on every pinned-rustc bump.
3. **Reimplement and inspect AArch64 prefetch.** Port only the small prefetch
   helper onto current development, behind exact architecture cfg. Prove
   `PRFM PLDL1KEEP` (or the selected equivalent) appears at real/null/ProbCut/
   qsearch child sites in Linux, Windows and macOS ARM64 artifacts. Exact bench
   must remain unchanged. Keep it only after a same-runner, paired target-native
   PGO A/B; a correct hint may still be neutral or harmful.
4. **Measure topology instead of encoding Apple folklore.** Log real cache-line
   and page size, then audit TT, `SharedSearchState` (`nodes`, `tb_hits`, stop/
   vote fields and root-score publication), engine-control atomics and future
   accumulators for destructive sharing. Compare flat TT storage, over-aligned
   allocation and block wrappers without changing byte capacity, associativity,
   indexing or replacement. An Apple-specific layout needs an Apple result and
   must not regress Linux/Windows ARM64 or x86-64.
5. **Create per-target performance anchors.** For each tier, calibrate an
   identical-binary pair and interleave revision-matched baseline/candidate PGO
   on the same stable hardware/runner. Raw NPS from different GitHub runner
   types is never compared. CI timing is diagnostic; controlled within-run A/B
   decides. On the 5950X, exercise baseline and AVX2 semantics and same-tier
   regression in addition to native PEXT.
6. **Archive the stale branch.** Record each commit as ported, rejected or
   HCE-deferred. No version bump or old PLAN/GUIDE text is cherry-picked. The
   branch is evidence, not a release candidate.

Behaviour-neutral platform work uses exact bench/search agreement and
same-target NPS, never SPSA. Any move-choice change uses the normal strength
gate. Exit when current development passes the complete production matrix,
ISA manifests match emitted code, the ARM prefetch has a target verdict, TT/
false-sharing hypotheses have target evidence and `origin/arm_fix` has no
unclassified item.

### 4.9 — Throughput, TT and parallel scaling

Profile accepted semantics at 1/2/4/8T: NPS, time-to-depth, TT replacement/
contention, root stability and strength. Audit TT layout after provenance;
hoist/batch proven invariants only. Preserve the 4.8 platform/ISA matrix and
regenerate target-native PGO after final shape. Every speed arm uses pooled/
interleaved builds. Do not reopen generic worker diversification without a
specific measured independent-work failure.

#### Carried-in retry — TT pressure of the rejected 4.3a arms

RAR-S27 rejected `EvalPruneTtMinDepth=2` at 1T, where its 43.8% node reduction
bought nothing: at a clock TC the saving was immediately spent on depth, and the
deeper, worse-informed tree netted even. One hypothesis survives that verdict
and belongs here rather than in 4.3, because 1T cannot test it — **a materially
smaller tree also means less TT write pressure and less shared-table
contention, which may scale differently at 4T/8T than at 1T.**

Conditions for the retry: test at 4T and 8T with the recorded hash and topology,
against the same-thread baseline, and report TT replacement and `same_key` share
alongside strength. `store_kind_*` and `tt_store_fresh`/`same_key` already give
the write-pressure readout. Treat it as a scaling arm, not a rehabilitation of
the 1T result — a 1T-neutral, 4T-positive change is a legitimate outcome, but so
is confirming it is neutral everywhere. Do not bake anything at 1T on the
strength of a 4T reading without the 1T non-inferiority to match.

### 4.10 — Single consolidated pre-NNUE search fit

Freeze architecture and generate configuration from the live parameter source.
Sensitivity/collinearity selects ≤24 coordinates spanning prospective depth/
pruning, NMP family, correction/history, qsearch and root/TM. Run one registered
5,000-iteration SPSA, one estimator and one bake; exclude HCE, dead/off and
redundant knobs. Build the retained fit into one clean final-PGO candidate and
run the registered material-gain gate against the pre-fit final-PGO baseline.
Post-fit switch ablations must show no harmful subsystem hidden by compensation.

### 4.11 — Cumulative target ladder and release 2.4.0

Beat the clean 4.0 baseline cumulatively at 1T STC/LTC and transfer at 4T with
zero forfeits; pass correctness/tactical/mate/zugzwang/TB/provenance/history/
extension/root telemetry and the Phase-4.8 production platform/ISA matrix.
Then run paired Rarog, Basilisk 1.9.3, every
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
incremental/full mismatches and passes external/net-metadata gates. Portable
scalar inference must pass the Phase-4.8 matrix; every shipped x86 SIMD and
ARM64/NEON kernel is bit-exact to it and target-native PGO-smoked. Update user
docs/version and commit; no push/tag.

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

### 8.1 — Advanced memory and runtime dispatch

Revisit TT/network placement, large pages, topology-specific prefetch and
runtime ISA dispatch with exact scalar parity and real-hardware tests. Phase
4.8 already guarantees the baseline production asset matrix.

### 8.2 — Protocol/platform completion

Add demanded product work such as Chess960 or additional platform/tier support.
ARM64 correctness is already required by Phase 4.8 and NNUE/NEON parity by
6.4/6.7.

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
