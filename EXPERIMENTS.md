# Rarog experiment ledger

This is the indexed maintainer record of measured experiments and the lessons
that may inform later work. It is not a roadmap: [`PLAN.md`](PLAN.md) owns what
will be done and in what order. [`CHANGELOG.md`](CHANGELOG.md) remains the
user-facing release record.

Every lesson below is conditional. A result describes one engine state, test
protocol, time control, compiler and machine population; it does not establish
a universal chess-programming rule. An experiment from Basilisk is only a
prior for Rarog and never bypasses Rarog's own gates.

## Contents

- [1. How to use this ledger](#1-how-to-use-this-ledger)
  - [Result and evidence vocabulary](#result-and-evidence-vocabulary)
  - [Recording contract](#recording-contract)
- [2. Measurement, harness and tuning](#2-measurement-harness-and-tuning)
- [3. Search and selectivity](#3-search-and-selectivity)
  - [Accepted or retained](#accepted-or-retained)
  - [Rejected, neutral or deferred](#rejected-neutral-or-deferred)
- [4. Root search, time management and SMP](#4-root-search-time-management-and-smp)
- [5. Evaluation and data experiments](#5-evaluation-and-data-experiments)
- [6. Throughput, build and platforms](#6-throughput-build-and-platforms)
- [7. Correctness and protocol lessons](#7-correctness-and-protocol-lessons)
- [8. Cross-engine evidence imported from Basilisk](#8-cross-engine-evidence-imported-from-basilisk)
- [9. Open retry map](#9-open-retry-map)
- [10. Template for a new experiment](#10-template-for-a-new-experiment)

## 1. How to use this ledger

Search the contents by subsystem before proposing a mechanism, tune or retry.
Use the stable IDs in commit messages and `PLAN.md` when a prior result changes
a future decision. Do not copy the tables into `PLAN.md`.

### Result and evidence vocabulary

| Term | Meaning in this document |
|---|---|
| **Accepted** | Passed the registered gate and entered an accepted baseline. |
| **Retained** | Kept for correctness, infrastructure or structural value; any Elo figure may be unresolved. |
| **Rejected** | Failed its registered gate or had a clear adverse measurement and was reverted. |
| **Neutral/inconclusive** | Evidence did not distinguish a useful effect at the tested resolution. |
| **Observation** | Diagnostic evidence, not an acceptance verdict. |
| **Imported prior** | Evidence from Basilisk; useful for ordering or designing a Rarog test, never for accepting it. |

Unless a row says otherwise, historical strength tests used paired games at
fast time control. Results before the pinned-harness repair of 2026-07-21 may
carry scheduler-placement bias. Fast-TC deltas are non-additive and may
compress or reverse at longer TC.

### Recording contract

For every experiment that reaches a verdict, update this file in the same
commit that accepts, reverts or closes it. Record:

1. baseline and candidate source SHAs, dirty-diff hash if applicable;
2. hypothesis and interactions expected to move;
3. binary/compiler/PGO, book, TC, threads, hash, concurrency, affinity and
   adjudication profile;
4. registered gate, games, W-D-L, estimate/CI and LLR where available;
5. diagnostics separately from the verdict;
6. disposition, conditional lesson and an objective retry trigger.

Use cautious language: “under these conditions this suggests …”, not “feature
X is good/bad”. If conditions or artifacts are unknown, say so.

## 2. Measurement, harness and tuning

| ID | Experiment and conditions | Result / disposition | Conditional lesson and retry trigger | Source |
|---|---|---|---|---|
| RAR-M01 | Early fixed-`movetime` gates were compared with the deployed clock path at `3+0.03`. | Fixed movetime manufactured false negatives; SPRT/SPSA moved to a unified clock TC. | A test TC must exercise the same time-management semantics as deployment. Fixed movetime remains useful only for deterministic diagnostics. | legacy plan at `757e9a3^` |
| RAR-M02 | Historical unpinned fastchess runs were audited under explicit physical-core placement on the Ryzen 9 5950X. | Real affinity/topology defects were found; the original +9.34 ± 8.20 null did not itself prove a fixed +9 Elo offset. | On this Windows host, small unpinned results may be biased. Re-audit a borderline old verdict only if it affects a current decision. | legacy plan at `757e9a3^` |
| RAR-M03 | Identical-binary null testing after harness changes. | The old symmetric `[-3,+3]` setup had zero expected LLR drift at equality; current policy is fixed-N 30k at 1T, requiring the full 95% nElo CI inside ±5. | Equivalence needs a calibration design, not an ordinary gain SPRT. Repeat after runner, scheduler, topology or adjudication changes. | `PLAN.md` §2 |
| RAR-M04 | Opening-book migration to paired UHO games. | Retained because it increased decisive-game signal and aligned SPRT, SPSA and gauntlet conditions. | Raw Elo from old and UHO protocols is not directly comparable without a bridge test. | legacy plan at `757e9a3^` |
| RAR-M05 | SPSA schedule audit: iteration/game units, PowerShell `$A`/`$a` collision and integer perturbation resolution. | Several schedule defects were repaired; old runs annealed faster than intended. Accepted bakes remain accepted because independent SPRTs passed. | A plausible SPSA trajectory is not proof of a correct schedule. Assert every emitted derived constant and inspect coordinate observability over the full horizon. | legacy plan at `757e9a3^` |
| RAR-M06 | Resignation threshold replay against 69,350 historical games. | `400/3` one-sided was too aggressive for Rarog's scale; `600/3` one-sided became `strength-v1`. | Adjudication scores are engine-scale dependent. Recalibrate after a material score-scale change such as NNUE integration. | legacy plan at `757e9a3^`; `PLAN.md` §2 |
| RAR-M07 | Staged self-play gains were checked in an external engine cohort. | The 2.2 cycle's roughly +316 staged result transferred as about +240 over 2.1.0. The 2.3 boundary measured +76/+78 at 1T and +194 at 4T over 2.2.0. | Self-play gives direction under these conditions, not an additive external forecast. Phase boundaries need direct prior-release and target-engine checks. | `CHANGELOG.md` 2.2.0, 2.3.0; legacy plan |
| RAR-M08 | The 36,400-game 2026-08-05 rating tournament used a 2.4-dev binary with interim values from an unfinished aspiration SPSA. | **Closed observation:** the last supplied checkpoint was 8,626 games, +11 pool Elo over 2.3.1 and 39 below Basilisk 1.9.3. It was not completed or accepted as a gate. | Under these conditions a mixed unfinished binary located a possible gap but established neither component value nor a new baseline. Do not extrapolate the partial pool ratings. | `PLAN.md` §1, 4.0 |
| RAR-M09 | Phase-4.1 normal versus diagnostic release builds on the Ryzen 9 5950X, non-PGO, `bench 13` plus four depth-10 positions. | **Retained infrastructure:** both builds matched 6,502,902 nodes / EBF 2.449; the four probes matched nodes, scores, PVs and best/ponder moves. Paired best-of-three NPS was 2,585,646 versus 2,301,097 (11.0% diagnostic cost including legacy exact atomics). | Under these conditions, the sampled observers did not alter the tree and their offline cost was bounded. This does not prove equivalence on every ISA/thread count or make counter movement an Elo proxy; repeat the gate after diagnostic control-flow changes. | `src/diag.rs`; `tools/diag_search_quality.ps1`; Plan 4.1 |

## 3. Search and selectivity

### Accepted or retained

| ID | Experiment and conditions | Result / disposition | Conditional lesson | Source |
|---|---|---|---|---|
| RAR-S01 | Pruning/margin SPSA Group B in the early search state. | **Accepted, +6.17 ± 4.88 nElo** after 19,458 SPRT games. | Joint fitting helped an under-tuned early parameter group. It does not imply repeated retunes of a mature group have similar value. | `CHANGELOG.md` 2.1.0 |
| RAR-S02 | Qsearch TT-bound stand-pat refinement. | **Accepted, about +6.5 Elo.** | Tighter TT evidence helped qsearch in that state, but Plan 4.2–4.3 must distinguish searched bounds from stand-pat estimates before expanding reuse. | `CHANGELOG.md` 2.1.0 |
| RAR-S03 | Per-move quiet futility pruning. | **Accepted, +7.98 ± 4.42 Elo** in the early baseline. | Move-local selectivity paid under that history/eval scale; thresholds require revalidation after prospective-depth unification. | `CHANGELOG.md` 2.1.0 |
| RAR-S04 | Joint pruning-family re-tune after the 2.2 HCE cycle. | **Accepted, +12.07 Elo;** separate LMR (−2.6), futility (~0) and TM (~0) retunes were reverted. | In that state, the joint pruning group captured the available retune value. Repeating adjacent fits without changed inputs had low value. | legacy plan at `757e9a3^` |
| RAR-S05 | Split history bonus/malus semantics and consumers. | **Accepted, +22.13 Elo.** | Under that history stack, separating positive and negative evidence materially improved learning. Preserve attribution and consumer normalization in Plan 4.5. | legacy plan at `757e9a3^` |
| RAR-S06 | Unconditional in-check extension in the then-current search. | **Accepted, +30.75 Elo.** | The extension helped this Rarog state; Basilisk's −10.17 result shows the verdict is not portable across fitted consumers. Plan 4.4/4.6 may change the tradeoff and must remeasure it. | legacy plan at `757e9a3^` |
| RAR-S07 | Broader history mechanism/tuning bundle. | **Accepted, +6.01 Elo.** | A coherent bundle produced a smaller additional gain after RAR-S05. Its members remain independently ablatable in future fits. | legacy plan at `757e9a3^` |
| RAR-S08 | Broad selectivity refit after the search-accuracy decomposition. | **Accepted, +15.33 ± 7.34 nElo;** broader tree, current 2.4 baseline. | In that baseline, Rarog appeared too selective for its decision quality. This result motivates better evidence, not indiscriminate tree growth. | `PLAN.md` 4 banked baseline; legacy plan |
| RAR-S09 | Zero-reduction LMR floor. | **Accepted, +9.13 ± 5.45 nElo;** current 2.4 baseline. | Some nominally reduced moves benefited from a zero-reduction outcome in that model. Plan 4.6 must preserve the accepted floor while changing shared depth semantics. | `PLAN.md` 4 banked baseline; legacy plan |
| RAR-S10 | Persistent `RootMove` records for mean, mean-square, PV, nodes and fail state. | **Retained infrastructure; isolated Elo unresolved.** | Collecting evidence does not help until aspiration, TM and fallback consume one coherent completed snapshot. | `PLAN.md` 4.7 |

### Rejected, neutral or deferred

| ID | Experiment and conditions | Result / disposition | Conditional lesson and retry trigger | Source |
|---|---|---|---|---|
| RAR-S11 | Direct port of a more Stockfish-like ProbCut formula into the older search. | **Rejected, −24.5 ± 8.5 Elo; reverted.** | Under that evaluator/search, copying a formula without matching TT provenance, history and revalidation harmed strength. Plan 4.2–4.4 tests evidence hygiene rather than “more ProbCut”. | `CHANGELOG.md` 2.1.0; `analysis/search_analysis.md` |
| RAR-S12 | Removed/altered history aging before and after the bonus/malus split. | **Rejected twice:** about −12.4 in the early state and −6.6 in the later wave. | In both tested stacks, unaged evidence was harmful. Reopen only if table ownership/normalization changes enough to invalidate both conditions. | `CHANGELOG.md` 2.1.0; legacy plan |
| RAR-S13 | `cutoffCnt` plus full LMR-family SPSA. | **Rejected, −7.78 ± 8.00.** Candidate searched about 16% more aggressively and won its tuning self-play before losing to the accepted head. | A tuner can select a sibling-local optimum. Future Plan-4.6/4.10 coordinates must gate against the accepted head and receive post-fit ablations. | legacy plan at `757e9a3^` |
| RAR-S14 | Post-LMR “do deeper” mechanism. | **Rejected, −7.29 Elo;** searched fewer nodes and lost; null pair −0.81 ± 9.09. | The tested eligibility/depth response hurt move quality rather than merely spending more time. Retry only if unified prospective depth pulls its coordinate off the neutral rail. | legacy plan at `757e9a3^` |
| RAR-S15 | Fail-soft qsearch against constants fitted around fail-hard bounds. | **Rejected, −5.96 Elo; reverted.** | A mechanically cleaner primitive can de-tune its pruning consumers. Plan 4.2–4.3 may retry the semantics only with explicit provenance and joint refit. | legacy plan at `757e9a3^` |
| RAR-S16 | Correction-history tune with `CorrGuardCapture=1`. | Aggregate mechanism washed at +1.43; the guarded tune discarded 59.7% of training and lost −55.98. Knobs returned neutral. | The run measured a crippled signal, not correction history's full value. Plan 4.5 must fix attribution/coverage before the single Plan-4.10 fit. | legacy plan at `757e9a3^` |
| RAR-S17 | Aspiration re-centering, verified mechanically against the tuned head. | **Rejected, −4.52 Elo.** | The change may have de-tuned aspiration/pruning consumers. Retry only through the completed root-confidence model and its own Plan-4.7 gate. | legacy plan at `757e9a3^` |
| RAR-S18 | Full FIDE-like draw/repetition bundle, then a reduced null-clock/fence variant. | **Rejected, −7.21 ± 6.03 and −11.91 ± 7.67;** only free mate precedence remained. | In that state, aggressive twofold handling was stronger even though the alternative was semantically cleaner. Keep legal correctness separate from optional search-draw policy. | legacy plan at `757e9a3^` |
| RAR-S19 | SEE pin-awareness verified against an independent legal-exchange oracle. | **Standalone rejected, −8.49 Elo;** mismatches improved 215→200. | Correcting a primitive can lose after its SEE thresholds are tuned around old behavior. Retry only inside a fit already justified by Plan 4.6, not via a dedicated low-EV tune. | legacy plan at `757e9a3^` |
| RAR-S20 | Half-run aspiration SPSA snapshot `ba3170b` (`15/148/149/9/20/8/0`) versus clean `p1043-base`; `[0,+3]`, `3+0.03`, 1T, 64 MB, paired UHO. | **Rejected by acceptance rule after manual stop:** 13,000 games, W-D-L 3,261-6,378-3,361, −2.67 ± 3.83 Elo / −4.16 ± 5.97 nElo, LLR −1.83 (bounds ±2.94). It did not formally hit H0, but did not accept H1; candidate bench was 7,047,226 versus 6,502,902. | In this incomplete fit, narrowing the initial window widened the tree without demonstrated strength. Do not resume or tail-select it; revisit aspiration only through the completed root-confidence model and consolidated Plan-4.10 fit. | snapshot `ba3170b`; Plan 4.0 |
| RAR-S21 | Phase-4.1 diagnostic `bench 13`, 1T, deterministic sampled interaction map on the retained 6,502,902-node baseline. | **Observation:** first-move cutoff 88.17%; LMR re-search 1.38%; sampled best move first 81.44%; TT sample hit 63.05% with 275 usable cuts and 113 contradictions; qsearch stores stand-pat/qmove/tail exact/tail upper 913/240/6/514; NMP verification pass/fail 533/7 with 83 nested attempts; pruning overlap 0.47%; 145,372 of 283,590 correction updates were capture-attributed. | Under this corpus, ordering, depth-0 TT authority, nested NMP verification and correction attribution deserve priority; the low observed pruning overlap gives little evidence that simple LMP/futility deduplication is a major prize. These counters are diagnostic priorities, not Elo estimates, and require game/TC validation. | `tools/diag_search_quality.ps1`; Plan 4.1–4.6 |
| RAR-S22 | Phase-4.2 opening static audit of the TT producer/consumer graph at `f35bc09`, plus a re-run of RAR-S21's reading on a freshly built diag binary. | **Observation.** The reading reproduced RAR-S21 digit-for-digit (fingerprint 6,502,902, EBF 2.449), so the sampled map is stable across a rebuild. Static findings: `TtEntry.flag_age` is **fully allocated** — 5 bits age (`0xF8`), 1 bit `is_pv`, 2 bits bound — so Plan 4.2's assumed "spare `flag_age` capacity" does not exist. 7 store sites and 13 read sites were enumerated. Sampled store mix: main 803 (7 exact / 508 lower / 288 upper), qsearch 1,673, ProbCut 14 — i.e. **67% of sampled stores are depth-0 qsearch entries and 37% are bare stand-pat**. `singular_probcut_depth_match` was 32 of 101 sampled singular attempts, meaning a third of singular decisions read an entry at exactly ProbCut's `depth-3` + `Lower` signature, which cannot be attributed to a producer without provenance. `EvalPruneTtMinDepth` is seeded 0, so those depth-0 entries can refine pruning at any depth. | Under this state the shortage is attribution, not counting: the coincidence rate is measurable while the producer is not, which is the argument for typed provenance rather than for tightening a depth threshold blind. Also recorded: `bench` shares one table across its 40 positions and ages it by 8 per position, wrapping after 31, so **any change to the age field's width is bench-visible and is a behaviour change, not a free refactor** — the cheapest 1-bit provenance slot (age 5→4 bits) therefore needs a strength gate, not a fingerprint check. Retry/extend when 4.3–4.4 need a persisted producer class. | `src/tt.rs`; `src/search.rs`; `src/evidence.rs`; Plan 4.2 |

## 4. Root search, time management and SMP

| ID | Experiment and conditions | Result / disposition | Conditional lesson and retry trigger | Source |
|---|---|---|---|---|
| RAR-R01 | Early Stockfish-style clock management on the old harness. | **Accepted, reported +81 Elo,** but the harness/protocol predates current calibration. | The direction and zero-forfeit improvement were useful; the magnitude is not a current prior. Revalidate only changed clock behavior under Plan-2 gates. | legacy plan; `CHANGELOG.md` 2.1.0 |
| RAR-R02 | Clock safety reserved `2*MoveOverhead`; fixed movetime used its full budget. | **Retained:** 28 fast-TC forfeits fell to zero; cumulative close gate +2.02 ± 3.62 Elo. | Under those time controls, safety dominated nominal think-time recovery. Preserve zero-forfeit gates after root/TM changes. | `CHANGELOG.md` 2.1.0 |
| RAR-R03 | Five-change Lazy-SMP/root-result bundle versus the original 4T implementation. | **Accepted, +102.78 ± 16.38 at 4T;** externally consistent with the 2.3 boundary. | The deployed bundle was strongly better, but no individual member inherits that value because it was never decomposed. | legacy plan; `CHANGELOG.md` 2.3.0 |
| RAR-R04 | Symmetric early stop vote at 2T. | **Rejected, −15.85 Elo.** | Taking the first of two estimated expiry votes biased time downward in this design. Plan 4.7 treats soft time as confidence, while maximum time remains the hard budget. | legacy plan at `757e9a3^` |
| RAR-R05 | Pool-view instability TM. | **Rejected, −5.54 Elo.** | Raw helper instability was not a useful direct time multiplier in that state. Plan 4.7 may pool it only as one input to completed-root confidence. | legacy plan at `757e9a3^` |
| RAR-R06 | Helper-history blending and additional ordering jitter. | **Neutral/rejected:** blending −0.52; jitter reverted. Shared TT hit rate already rose strongly with thread count. | In that state, TT coupling made generic diversification/history sharing largely redundant. Reopen only with measured independent-work failure, as required by Plan 4.9. | legacy plan; `analysis/smp_analysis.md` |

## 5. Evaluation and data experiments

New HCE strength work is frozen. These rows remain relevant to NNUE data,
teacher and measurement design; they do not authorize Plan-9 HCE work unless
NNUE is explicitly abandoned.

| ID | Experiment and conditions | Result / disposition | Conditional lesson and retry trigger | Source |
|---|---|---|---|---|
| RAR-E01 | Staged Texel fit over 2.19M self-play positions: king safety, threats, mobility, scalars, imbalance, material/PST and polish. | Every stage accepted; about +316 staged self-play and +240 externally over 2.1.0. | Sequential fitting worked strongly on that corpus, but only about 75% of the staged gain transferred to the external cohort. | `CHANGELOG.md` 2.2.0; legacy plan |
| RAR-E02 | Lazy HCE shortcut after the evaluator expansion. | **Accepted, about +4.4 Elo.** | The shortcut paid under that HCE distribution, but the safety margin is representation/scale dependent and is not a NNUE constant. | legacy plan at `757e9a3^` |
| RAR-E03 | Stockfish-at-60k off-policy distillation with material scale pinned. | **Rejected, −17.11 Elo,** despite 4.9% lower holdout loss and 9/10 improved buckets. | For this well-fitted HCE/corpus, lower teacher-fit loss did not predict play. Basilisk's +6.75 opposite result reinforces that transfer is engine-state dependent. | legacy plan; `analysis/hce_analysis.md` |
| RAR-E04 | 500k-game on-policy refresh yielding 2.18M unique positions; pure WDL beat blended labels on the shared holdout. | **Rejected, −1.28 ± 2.79 over 26.8k games;** pipeline and inert parameters retained. | Even on-policy lower validation loss did not improve this unchanged representation. Retry only after representation/policy changes and with a frozen external holdout. | legacy plan at `757e9a3^` |
| RAR-E05 | Narrow L2-anchored refresh from a stronger label generator, moving 57/1,204 parameters mostly by 1 cp. | **Accepted, +11.56 ± 5.19 Elo;** frozen in the 2.4 baseline. | A narrow anchored refresh differed materially from wholesale re-derivation. This does not reopen general HCE work before NNUE. | `PLAN.md` banked baseline; legacy plan |

## 6. Throughput, build and platforms

| ID | Experiment and conditions | Result / disposition | Conditional lesson and retry trigger | Source |
|---|---|---|---|---|
| RAR-P01 | Phase-9 clean-code/build program, each step bench-identical and spot-checked. | End-to-end result was about −3.2% NPS, inferred around −2 to −3 Elo; infrastructure retained. | On this host, several sub-noise regressions compounded. Every refactor program needs one pooled end-to-end NPS comparison against its starting point. | legacy plan at `757e9a3^` |
| RAR-P02 | Phase-10.3 bench-identical hot-path wave with two PGO builds/arm. | **Accepted, +10.35% NPS and +20.31 ± 7.13 Elo at `3+0.03`.** | In this x64 PEXT/fast-TC condition, speed converted near 2 Elo per 1% NPS. Do not project that constant to LTC, NNUE or another ISA without measurement. | legacy plan at `b75666e^` |
| RAR-P03 | Post-SMP duplicate-compute/index-hoist cleanup. | **Retained, +0.99% then +1.56% median NPS** in independent pooled passes; search fingerprint unchanged. | Small speed gains became credible only through clean worktrees, pooled builds, self-pair calibration and interleaving. | legacy plan at `757e9a3^` |
| RAR-P04 | Board-layer perft comparison with Basilisk. | Rarog's board layer was not the main source of the remaining search-strength gap. | A faster primitive benchmark does not identify search quality. Revisit only if a profile attributes material deployed time there. | `analysis/board_perft_compare.md` |
| RAR-P05 | Pawn-cache enlargement from the profile audit. | A 128× larger table gained about 1.1 hit-rate points but lost 4.5% NPS. | Under that workload, lookup/memory cost dominated the small hit-rate gain. Any future cache size change needs both profile and strength evidence. | `analysis/speed_profile_8_12c.md` |
| RAR-P06 | `origin/arm_fix` added AArch64 `PRFM PLDL1KEEP` and hoisted two HCE `LazyLock` accesses. | **Unverified on ARM;** branch reported +2.51% on x64 for a combined patch, so causality is not isolated. | Reimplement only the architecture-specific prefetch on current development and require target-native A/B. HCE-only hoists stay frozen. | `PLAN.md` 4.8; branch `origin/arm_fix` |
| RAR-P07 | `origin/arm_fix` wrapped TT clusters in 128-byte Apple-oriented blocks. | **Unverified:** no ARM timing and existing cluster alignment already prevents the claimed boundary straddle. | Alignment folklore is not evidence. Compare equal-capacity layouts on actual target topology before retaining a wrapper. | `PLAN.md` 4.8; branch `origin/arm_fix` |
| RAR-P08 | Windows ARM64 PGO with pinned Rust used `rust-lld` to work around profile-link failure. | **Retained in 2.3.1:** about +8% NPS locally, unchanged bench/search behavior. | Toolchain workarounds are versioned debt. Re-test on each pinned compiler bump and keep behavior/performance claims separate. | `CHANGELOG.md` 2.3.1 |

## 7. Correctness and protocol lessons

| ID | Experiment or failure mode | Disposition | Conditional lesson / coverage | Source |
|---|---|---|---|---|
| RAR-C01 | Self-consistency tests compared implementations sharing the same rule-50, repetition and SEE omissions. | Independent legal-exchange and external perft oracles added/required. | Green correlated tests do not establish correctness. Important invariants need an implementation-independent oracle. | legacy plan; `analysis/infra_analysis.md` |
| RAR-C02 | Rule-50 draw could override mate; null moves changed the halfmove clock; repetition lacked root/null awareness. | Free mate-precedence fix retained; optional draw-policy bundle followed RAR-S18's strength verdict. | Separate legal terminal precedence from heuristic repetition policy; they can have different acceptance criteria. | legacy plan; `analysis/search_analysis.md` |
| RAR-C03 | Multi-thread diagnostics reset/dumped inside helper-called root search and the diagnostic build had stopped compiling. | Fixed; previous multi-thread counter history was declared unreliable. | Telemetry must have one owner and a build/runtime canary before it can guide search decisions. | `CHANGELOG.md` 2.3.1; legacy plan |
| RAR-C04 | Aspiration mate-score re-search could fail to terminate; capture cutoffs could train quiet correction. | Fixed and regression-covered. | Rare control-flow and attribution faults can contaminate root/time/history together. Retain deterministic tests before strength gates. | `CHANGELOG.md` 2.3.1 |

## 8. Cross-engine evidence imported from Basilisk

These are ideas, warnings or ordering priors already incorporated where useful
in Rarog's forward plan. No additional roadmap item is created merely by
listing them here.

| ID | Basilisk evidence | Possible Rarog implication | Existing PLAN coverage |
|---|---|---|---|
| RAR-X01 | TT-bound-aware pruning evaluation gained +7.18 Elo while preserving raw/corrected eval for learning. | Strong prior for typed result evidence and producer/consumer capability separation, not for copying its TT layout. | 4.2–4.4 |
| RAR-X02 | Check-extension removal lost −10.17 ± 6.52 in Basilisk while Rarog's extension had gained +30.75. | Confirms that extensions and their LMR/pruning consumers co-adapt. Rework only inside prospective-depth/refit gates. | 4.4, 4.6, 4.10 |
| RAR-X03 | Stockfish distillation gained +6.75 in Basilisk but lost −17.11 in Rarog. | Teacher/corpus/representation fit dominates transfer; a sibling success cannot reopen RAR-E03 unchanged. | 5.0, 6.0–6.2, 7.0–7.2 |
| RAR-X04 | A 6-ply continuation-history channel lost −7.70 in Basilisk. | Wider history distance can duplicate existing signals. Rarog should prove unique held-out attribution before adding contexts. | 4.5 |
| RAR-X05 | Exact/PV reward-only history and surprise scaling jointly reverified at +3.06 ± 4.35. | Result kind and confidence can be useful training inputs when sibling maluses are not misapplied. | 4.2, 4.5 |
| RAR-X06 | Root instability TM reverified at +6.46 ± 4.12, while Rarog's raw pool-view variant lost −5.54. | Instability may help only when derived from a completed authoritative root snapshot and bounded with other confidence signals. | 4.7 |
| RAR-X07 | Basilisk's +4.34% NPS wave measured +8.69 ± 6.63 Elo at STC; some cached-check/pin optimizations that helped Rarog were negative there. | Speed-to-Elo direction is corroborated near this TC, but individual hot-path optimizations are profile- and language-specific. | 4.8, 4.9 |
| RAR-X08 | Basilisk's `arm_fix` independently tried unmeasured TT over-alignment; its shipped build has clearer ISA-tier documentation but also a PEXT documentation/flag mismatch. | Corroborates the need to measure topology and audit the executable asset contract, not the proposed wrapper. | 4.8 |
| RAR-X09 | Basilisk's SMP safety bundle gained +30.42 ± 8.77 at 4T, smaller than Rarog's five-change +102.78 bundle. | The different gains suggest different baseline defects; compare ownership/clock/TT interactions without assigning Elo to individual components. | 4.7, 4.9, 8.0 |

The cross-review found no additional high-value Basilisk item missing from the
current Rarog plan. Items above are already covered, contradicted by local
evidence, or deliberately postponed to the NNUE/scaling phases.

## 9. Open retry map

| Prior IDs | Retry condition | PLAN destination |
|---|---|---|
| RAR-S11, RAR-S15, RAR-X01 | Typed producer/consumer evidence and qsearch/ProbCut provenance exist; retry arms are independently ablatable. | 4.2, 4.3, 4.10 |
| RAR-S13, RAR-S14, RAR-S19, RAR-X02 | Unified pre-move `MoveEvidence` and prospective depth exist; the single joint fit includes affected consumers. | 4.4, 4.6, 4.10 |
| RAR-S16, RAR-X04, RAR-X05 | Diagnostics prove attributable coverage/unique signal before adding or fitting histories. | 4.1, 4.5, 4.10 |
| RAR-S17, RAR-R04, RAR-R05, RAR-X06 | Completed root-confidence snapshot exists and avoids double-counting. | 4.7, 4.10 |
| RAR-P06, RAR-P07, RAR-X08 | Cross-build runner and physical non-x64 hardware are available; test one isolated layout/prefetch arm at a time. | 4.8 |
| RAR-P01–RAR-P05, RAR-X07 | A new profile identifies a material deployed hotspot; use pooled same-target PGO A/B. | 4.9 |
| RAR-E03, RAR-E04, RAR-X03 | NNUE data/teacher experiment with changed representation and frozen external holdout—not another HCE refit. | 5.0–7.2 |

Anything not meeting its trigger stays closed. A retry is a new experiment with
a new ID and manifest; it does not overwrite the historical row.

## 10. Template for a new experiment

```markdown
### RAR-<area><number> — <short name>

- Date / owner:
- Baseline SHA / candidate SHA / dirty-diff hash:
- Hypothesis and interacting consumers:
- Registered gate and stop rule:
- Build: compiler, flags, PGO manifest, binary hashes:
- Games: book/hash, TC, threads, hash, concurrency, affinity, adjudication:
- Result: games, W-D-L, Elo/nElo and CI, LLR:
- Diagnostics: nodes, EBF, NPS, depth, counters, suites (not the verdict):
- Disposition: accepted / retained / rejected / neutral / observation:
- Conditional lesson:
- Retry trigger or `closed`:
- Artifacts / commits:
```
