# Rarog development plan

Updated 2026-09-07. This is the current roadmap. Historical evidence belongs
in `EXPERIMENTS.md`; the short operator contract, current status and model
mapping belong in `GUIDE.md`.
Phase 4's open work was reordered and renumbered on 2026-09-04 after an
instrument audit; section 13 maps the old numbers to the new ones.

## 1. Current state

| Item | State |
|---|---|
| Released baseline | **2.3.2** at `f931722` on `master` |
| Accepted head | RAR-E12 + 4.9a.7 on `dev`; development `bench 13` = **7,601,220 nodes / EBF 2.474** after the behavior-neutral SEE repair. Includes the 4.9a.4 mate drive, which is bench-INVISIBLE |
| Integration state | The failed SearchCore rewrite is reverted by `c5e451d`; `d2c7788`/`e4f10ca` upgrade search evidence and `8d8f507` supplies the audited complete HCE fitting pipeline without changing accepted behavior |
| Frozen search/HCE oracle | `hybrid` at `75d0d43`, Stockfish `9587eeeb` driving the exact Rarog 2.3.2 HCE |
| Measured search deficit | **355.26 +/- 27.03 Elo at equal nodes** and **250.77 +/- 13.12 Elo at equal time**; Rarog's speed is worth a measured **104.5 Elo** |
| Accepted Phase-4 gains | ProbCut **+15.56 +/- 10.02**; root LMR relief **+2.33 +/- 1.85**; complete HCE refit **+22.04 +/- 7.51**; TB-corrected labels **+6.73 +/- 3.82**; hce-v3 refit **+11.81 +/- 5.33** |
| Active game job | none. **RAR-E15 ACCEPTED 2026-09-08** — 4.11b integrated board cluster, **+12.12 +/- 10.17 Elo**, **+18.40 nElo**, H1 at 1,950 games under `[-5,5]`. RAR-E12 accepted 2026-09-03 at +11.81 +/- 5.33 Elo; RAR-E13 withdrawn unresolved |
| Current step | **4.12.1 — adopt the endgame order; recognizer-vs-scale classification**, `RESEARCH / R3` |
| Instrument state | 4.10 repairs; v2 baselines/floors, budget transfer, label audit, mate-drive closure and conversion-claim correction are complete. Historical v1 pawn-family conversion claims are retained but superseded in place by RAR-M24 |
| HCE state | Completely refitted and accepted. The 1,218-slot surface has one whole-surface game verdict; structural gaps (4.9) are closed and endgame closure (4.12) is open |
| Next release | Conditional **2.4.0** after 4.20; baseline NNUE then targets **2.5.0** |

Phase 4 remains a bounded pre-NNUE programme because the hybrid established
large search and HCE populations worth investigating. It is not a commitment
to keep working until the oracle is matched. Each dependency-complete cluster
must earn its own continuation.

### Execution queue and explicit holds

The nominal order is unchanged. **4.11.7 was held in its original place**;
the earlier guide simply did not show the scheduling hold. Claude's handoff
predates 4.11b and is not the current roadmap: after 4.11 comes **4.11b**, then
4.12. The maintainer authorized heavy computation, and **4.11.7 is now complete**
(RAR-M21, 2026-09-06). RAR-M22 and RAR-M23 subsequently completed 4.11.8 and
4.11.9. RAR-M24 completed 4.11.10; its scheduling dependency is resolved.
Board leaves 4.11b.1–4.11b.8 are complete. RAR-M31's pin candidate was
withdrawn in `c44608a`: whole-search value remains unproven, and another
standalone timing campaign is not prioritized. Its independent oracle remains.
4.11b.9 is now **ACCEPTED** and integrated in `5c439da` (RAR-M33): fused quiet
relocation holds exact state and fingerprint parity, gains +16.33/+17.30/+19.32%
on the isolated make/unmake primitive and **+0.876%** full-search with a 95%
bootstrap interval of **+0.050% to +2.055%**, which excludes zero on a
verified-idle host. RAR-M32's earlier `NO_CHANGE` is **VOID**: it was taken
while a Manta SPRT held the host at ~50% CPU busy, and the same baseline code
re-measured 40.7% faster once idle. Its targeted per-piece-class test is in
`8a73cfd`. 4.11b.10 is now closed `NO_CHANGE` (RAR-M34): intra-node pin
sharing is already implemented and active, `compute_pinned` and `check_info`
share no subexpression because they query different king squares against
different slider colours, and sharing into SEE is forbidden by the 4.11b.5
exchange-occupancy contract rather than merely unprofitable.
4.11b.11 is now closed `NO_CHANGE` (RAR-M35): incremental attacker maintenance
was implemented in full and verified correct, but made `threshold SEE only`
**slower** in all three rounds, so its registered stage-1 screen rejected it and
the expensive full-search arm was never run. The leaf's premise was partly
wrong — the two `attackers_to_color` calls per exchange step are not duplicates,
and the second is the mandatory per-candidate king-legality test that a carried
target-attacker set cannot serve.
4.11b.12 is now closed `NO_CHANGE` (RAR-M37): the refreshed profile reads
king-square lookup at 0.502%, whose 2x-local ceiling of 0.25% is two to four
times smaller than the measured width of the instrument that would have to
accept it, so the candidate is unqualifiable however it is written.
4.11b.13 is now done (RAR-M38, `f70ac19`): search reserves `MAX_PLY` of
history headroom on the root before the hot path, worker clones inherit it
through `Board::clone`'s capacity preservation, and five contract tests pin
headroom, no mid-search reallocation, clone inheritance, exact restoration and
the `legal_move`/`is_legal` canonicalization difference. Behaviour-neutral, no
speed claim.
4.11b.14 is now closed `NO_CHANGE` (RAR-M39): no board region exceeds 6.7%,
so the leaf's gate for opening an implementation is not met, and each named
alternative loses on measured grounds — six type boards save 48 bytes that were
never binding while taxing 208 `pieces()` sites, per-ply state copying would
cost 33 KiB against the current 3 KiB, and legality is already amortized at
517:1 fast-to-full check calls.
4.11b.15 is now closed `NO_CHANGE` (RAR-M40): all four draw policies are kept
with independent dispositions and retry triggers, and the repetition-identity
separation is pinned by test.
4.11b.16 is now **QUALIFIED** (RAR-M41): the integrated board cluster measures
**+1.421%** whole-search NPS, 95% **[+0.953%, +1.764%]**, under production
pooled-PGO settings on a verified-idle host, with a null pair of same-revision
builds confirming the instrument is unbiased at **+0.222% [-0.130%, +0.630%]**.
The correctness matrix passed in full.
4.11b.17 (RAR-E15, accepted) and 4.11b.18 (RAR-M42) followed. A post-closure
review on 2026-09-09 (RAR-M44) inserted **4.11b.19**: every generator returned
its 520-byte move list through a `memcpy` that Basilisk avoids, in both the
cross-engine harness and the search. Its (a) and (b) are now **IMPLEMENTED**
(`55e228a`, `021dc98`) and deterministically qualified; the pooled-PGO run
that decides whether (b) is banked is owed to the maintainer, and (c)/(d)
follow.
It precedes 4.12.1; it is behaviour-neutral and owes no game gate.

| Leaf | State and reason | Unblock event / owner | Dependency boundary |
|---|---|---|---|
| 4.12.21 | Future evidence gap, not a skipped completed step | Obtain independent 7-man truth/verification, or record a justified scope exclusion under 4.12's closure contract | Cannot claim this family verified or close 4.12.23 with unowned missing evidence |

RAR-M22 completed 4.11.8 from hash-verified source PGNs and local 3–6-man
truth. Its raw game-result audit is an input to 4.13.1, not a row-level audit
of the already corrected `hce-v3-tb` CSVs. RAR-M23 closed 4.11.9's promotion
closure and recorded two family debts. RAR-M24 used the completed
RAR-M21/RAR-M22/RAR-M23 evidence to correct the conversion claims at 4.11.10.
Their remaining family work is owned by 4.12; the obligations were not dropped.

At each handoff review every held item: state what was done, what is still
held, its unblock event, and the next executable leaf. Resume the earliest
eligible held leaf when its prerequisite is satisfied. A conditional future
audit is not a skip: it closes only on the measured disposition its contract
requires. Keep one ID/checkbox per obligation; never move a hold out of sight
or mark missing verification done. GUIDE shows the short status, this register
holds the reason/dependency, and historical artifacts retain their original
numbers and dates.

## 2. Operating and evidence rules

`AGENTS.md` is authoritative for measurement, verification, documents and
gating. The following rules determine this roadmap's order.

1. Similarity to Stockfish is never an objective or acceptance criterion.
   Reference code identifies problems, dependencies and useful tests; Rarog
   implements its own answer and games decide.
2. Cross-engine ablation measures **marginal value inside each co-adapted
   engine**. It may rank questions, but it is not portable headroom and family
   losses must not be summed.
3. A strength candidate starts from the latest accepted head, is registered in
   `EXPERIMENTS.md` before games, and ends accepted or reverted before another
   candidate opens.
4. The normal unit is one smallest dependency-complete, locally fitted
   cluster. Node count, EBF, tactical suites, fit loss and oracle agreement
   explain or refute a candidate; only a clean final-PGO SPRT accepts it.
5. Default gain bounds are `[0,3]` nElo. Use wider, symmetric or
   simplification bounds only when the prospective prior and RAR-M10 sizing
   justify them. Never change the gate after seeing games.
6. Search SPSA is conditional on live coordinates, interaction and local
   curvature. HCE Texel fitting owns traced linear coefficients. HCE SPSA owns
   only activated nonlinear/global residue that the linear trace cannot fit.
7. Search and HCE coordinates never share a tune. After an HCE changes,
   cp-valued search consumers are audited and, if justified, fitted separately.
8. **Every strength A/B and cohort runs with adjudication off** (maintainer
   decision, 2026-09-01, on RAR-M16). Playing games out costs about 10% wall
   time; adjudication destroys 52.7% of all endgames before they are reached
   and priced at ~74 Elo as a cross-evaluator confounder. `-Adjudicate` opts
   back in and must be justified in the registration: wall time genuinely
   binding, and a change that provably cannot touch conversion or defensive
   holding. **This now covers every instrument**: `sprt.ps1`, `gauntlet.ps1`,
   `datagen.ps1` (profile `datagen-v2`) and the SPSA tuner. Label generation
   still uses a prospectively named, immutable profile -- `datagen-v1` names
   the adjudicated contract that `hce-v2` was built under and keeps meaning
   that, so old manifests stay true.
9. Two fully implemented clusters in the same track without an accepted gain
   stop implementation and force a new evidence audit.
10. Engine, tooling and documentation changes remain separate commits.
11. **State the measurement LAYER.** Theory truth, move quality, conversion and
    game strength are four different questions with four different units, and
    occurrence gates whether the first three can ever reach the fourth. Every
    report names its layer, instrument, node budget and position set. Layers are
    never aggregated -- there is no exchange rate between a truth failure and a
    conversion gain -- and when two layers disagree, that disagreement is the
    finding. Truth vetoes absolutely; conversion never establishes strength;
    bench identity is provenance and belongs to no layer.
12. **Node budget is a run condition, not a detail.** Record it beside TC,
    threads, hash, book and adjudication. Justify a screen budget against the
    DEPLOYMENT TC by measuring actual nodes/move rather than guessing, and
    bracket rather than assume: a failure that appears only at a low budget is
    PROVISIONAL. Prefer nodes to depth for cross-variant work, because equal
    depth is unequal work once an eval change shifts pruning.
13. **Split a selection cohort before selecting on it, and register which half
    decides.** Carry a runner-up into confirmation; the leader can be rejected
    there. A cohort that has produced a verdict is SPENT for selection and
    survives only as a VETO, because a safety property is not an estimate.
    Report a plateau as a plateau: "best of N" without separation from its
    neighbours is not a winner.
14. **A term's blast radius is its dispatcher condition's PROMOTION CLOSURE.**
    Promotion manufactures material, so an argument that a term cannot reach a
    family is incomplete until under-promotion is considered. Testing only the
    families the safety argument already excluded proves nothing.
15. **A guard is not verified until it FAILS on a known-bad input.** Passing on
    a good input is not evidence. This covers regression anchors, vetoes,
    floors, drift gates, harness wires and fingerprints; reproduce the original
    failing conditions exactly, and hash behaviour through an explicit field
    list rather than whole records. An interim SPRT reading is likewise not
    evidence, in either direction.

### Prospective workflow and ownership

Open nontrivial work uses this conceptual state machine:

`RESEARCH -> READY_FOR_IMPLEMENTATION -> IMPLEMENTED -> LOCAL_QUALIFIED -> GAME_GATE -> CLOSED`

States describe evidence maturity, not checkbox progress. A dependency can
hold a ready verification leaf without returning it to research. Tasks skip
irrelevant states: documentation/provenance can close after its own checks;
research can close `NO_CHANGE` or `NOT_WORTH_PURSUING`; behavior-neutral work
uses deterministic/performance qualification; playing-strength changes
normally require `GAME_GATE`.

`READY_FOR_IMPLEMENTATION` means the central research decision is no longer an
implementation choice. The mechanism, intended semantics, evidence in this
engine, known interactions and invariants, falsifier, qualification and
accept/reject rule are explicit here or in a linked analysis handoff. If a
material premise fails during implementation, preserve useful diagnostics and
return the leaf to `RESEARCH`; do not rescue it with an adjacent mechanism.

| Class | Required capability |
|---|---|
| `R3` | Frontier research for unresolved causal, chess or architecture questions |
| `R2` | Bounded but correctness-sensitive architecture/reasoning |
| `I2` | Difficult implementation with strong reasoning |
| `I1` | Well-specified implementation |
| `M` | Mechanical documentation/provenance work |
| `V` | Deterministic, performance or playing-gate verification |

PLAN owns classes; GUIDE alone maps them to current model names and thinking
modes. Research owns the question, competing hypotheses, interaction map,
prospective prediction, falsifier, stop rule and readiness verdict.
Implementation owns ordinary local code structure, refactoring, focused tests,
build/debug work and cheap qualification inside that frozen contract. It does
not redesign the experiment. Long tournaments/SPRTs, datagen, expensive tuning,
PGO campaigns and lengthy profiling are maintainer-started unless explicitly
delegated; the agent prepares and verifies the command and artifacts.

Evidence precedence remains rules 4 and 11: a coherent explanation or proxy
movement is not a verdict. Before result exposure, freeze the prediction in
`EXPERIMENTS.md`; after exposure append calibration rather than rewriting the
prediction. Research packets and implementation handoffs use the lightweight
format in `PROCESS.md` and live under `analysis/` only when PLAN would become
unwieldy.

### Subsystem audit contract: correctness, composition and cost

**Planning only, added 2026-09-05.** The audits below are required analyses,
not findings, optimization promises or permission to implement an imagined
solution. Existing studies are inputs, not proof that the current composition
is correct. Reference current local Stockfish/Reckless/Basilisk implementations
when useful; pin source revisions and compare their contracts with Rarog's
actual consumers rather than porting a feature checklist.

For each audit, before proposing implementation:

1. Inventory current files, entry points and consumers. Trace producer ->
   stored state -> consumer -> invalidation/undo/reset, including feature
   flags, thread ownership, score units, bound/evidence class and lifetime.
   Name missing, duplicated, stale, contradictory or mutually suppressing
   behavior as hypotheses until demonstrated.
2. Describe each feature's intended role and alternative implementation
   contracts. Check that the chosen form is compatible with surrounding
   features. A feature can pass an isolated test while emitting a score,
   depth, history update or cache entry its consumer interprets differently.
   Include both correctness invariants and intended heuristic interactions.
3. Profile realistic workloads and retained difficult cohorts. Separate cost
   per operation, invocation frequency, search tree changes, whole-search NPS,
   time-to-solution and strength. Use exact builds and live observations;
   record overhead and confounds immediately. A faster node that expands a
   worse tree is not automatically an improvement.
4. Test interacting pairs/clusters with bounded ablations or a factorial
   screen where justified: baseline, A, B, A+B under matched conditions.
   Explain reinforcement, cancellation and masking using mechanism counters
   and targeted cases. Do not add component Elo estimates or infer interaction
   strength from noisy point estimates. Prior rejected work needs new evidence
   and its original retry condition; no automatic SearchCore rewrite.
5. Classify each finding as correctness repair, behavior-neutral cost change,
   heuristic/composition change, fitting issue, intentional tradeoff or
   no-change. Give evidence, impact, confidence, prerequisites, risk, exact
   consumer scope and a deciding test. Preserve supported functionality and
   independently verified correctness even if fixing a defect changes bench.
6. End the analysis leaf by expanding PLAN/GUIDE with only evidence-backed,
   dependency-ordered implementation/test/fit/gate leaves. Include done
   criteria, a no-change disposition where appropriate, and a named future
   owner for deferred work. Reuse existing leaves first. Add sibling steps
   within the owning section, or a letter-suffixed section before its next
   dependent gate; keep one consistent numbering order and map changed open
   IDs. Do not create deep four-part IDs or duplicate owners. Stop after the
   analysis handoff; no implementation in that same leaf.
   If a childless audit section gains children, its first child records the
   completed audit and the parent stays open until required follow-up children
   close. For an existing audit leaf such as 4.15.1, add sibling leaves under
   its existing parent rather than nested four-part numbers. Update the parent
   title/status to describe the resulting scope; keep completed IDs stable.
7. Future implementation qualifies the smallest dependency-complete cluster.
   Behavior-neutral optimizations require targeted parity plus exact immediate
   accepted fingerprint and controlled whole-search performance. Deliberate
   heuristic changes require registered strength qualification and retention
   of correctness/functionality, not NPS alone. Correctness defects cannot be
   tuned away. Use local fitting when the changed semantics displace an
   identifiable optimum: Texel for traced HCE terms; conditional SPSA for live
   nonlinear/search interactions. Keep HCE/search tunes separate, fit relevant
   covariances, prove the fitting wire live, and register before games. A flat
   or monotone surface is a reason to skip SPSA, not an invitation to spend it.

### Audit ownership and order

| Part / boundary | Analysis owner | Why here / later integration |
|---|---|---|
| Board, legality, make/unmake, SEE mechanics | 4.11b | Existing detailed audit; accepted board before endgame changes |
| Endgame dispatch, recognizers/scales, rule-50 and generic HCE composition | 4.12.1 and family leaves | Extend current family contracts; respect budget/truth holds |
| Datagen, labels, extraction, splits and fitting data integrity | 4.13.1 / 4.13.4 | Analyse before publishing the next corpus; reuse 4.11.8 evidence |
| HCE implementation and interacting terms/caches/search score contract | 4.13a | After family/label evidence, before 4.14's mandatory consolidation fit |
| Search, qsearch, move ordering, histories, pruning/reductions/extensions | 4.15.1 | Broaden existing authority audit after accepted HCE; reuse 4.15.2–5 |
| TT, eval/pawn caches, hashing, storage and hot memory layout | 4.15a | Search consumers known; before final search fit |
| Threading, engine/UCI lifecycle, search control and tablebase integration | 4.15b | Supported behavior now; advanced scaling stays in Phase 8 |
| Diagnostics, benchmark/game harnesses, feature builds and ISA delivery | 4.15c | Instrument/build trust before final search fitting, TM and release gates |
| Time management and root-confidence consumers | 4.17.1 | Extend existing owner; recheck fit/search/SMP signals |
| Final module coverage, dead paths and cross-subsystem closure | 4.18.1 / 4.19 | Every production/tool boundary gets an owner or justified disposition |
| NNUE board state and evaluator/runtime composition | 5.2 / 5.3 / 5.6 / 6.4 | Retain RAR-M20 cost stages; do not preload NNUE runtime cost into HCE |
| Trainer/network formats, feature parity and inference scale | 5.4 / 6.0 / 6.3 | Extend existing preflights before training/inference |
| Post-NNUE search/eval retuning and interaction revalidation | 7.3 | HCE-era compatibility is not proof for a new evaluator |
| High-thread/NUMA, universal dispatch, TT/net placement and platforms | 8.0–8.2 | Reuse Phase-4 audits; new architectures only when this phase opens |

Audits produce a bounded recommendation and implementation plan, not an
unlimited mandate to keep optimizing every subsystem. Preserve existing
cluster/game-budget stop rules. 4.19 cannot close while an identified required
repair or validation has no disposition; future-only work needs an explicit
later owner and must not be quietly counted as verified current behavior.

### Minimum gates

| Change | Minimum evidence |
|---|---|
| Correctness | Independent invariant/regression test; strength gate if playing behavior changes materially |
| Behavior-neutral hot path | Exact fingerprint, debug/release tests, pooled NPS |
| Search/HCE strength | Revision-matched clean PGO A/B, paired UHO, registered SPRT and stop rule |
| Extension/depth authority | Fixed-node tree/depth profile plus tactical suite at both fixed depth and equal node cost; true correctness canaries veto, while aggregate disagreement is diagnosed rather than cherry-picked |
| Texel fit | Verified label domain; hash-complete whole-start train/validation/frozen-test splits; exact all-slot instrument coverage and bake smoke; reconstruction, activation/identifiability and semantic bounds; settled trajectory, post-fit cohort/covariance review, baked PGO and SPRT |
| SPSA | Registered live surface and immutable horizon, bounded sensitivity pilot when needed, completed full-surface theta, fresh PGO bake and SPRT |
| Release | Prior-release STC/LTC, 4T direction, NPS, platform/ISA matrix and user-facing docs |

### Independence boundary

Both engines are GPL, so copying was never a licence question -- Rarog is
GPLv3 and so is Stockfish. The restriction was a design choice, and the
maintainer relaxed it on 2026-09-01.

**Constants may now be ported as SEED VALUES.** Problems, dependencies,
populations, failure modes, mechanisms and their constants may all cross from a
reference. What does not change is what makes a constant Rarog's: a ported
value is a starting point, never a result.

1. A ported constant is on the DONOR's score scale. RAR-E06 refit Rarog's
   entire HCE, so its centipawn means something different; an imported eval
   weight is in the wrong units by construction.
2. So a ported constant rides the next fit. Port it, seed it, let 4.14's refit
   cycle move it, and expect the optimum to sit somewhere else.
3. Nothing about the gates changes. A ported mechanism still passes the same
   conversion floors, theory vetoes and registered SPRT as one written here.
   Provenance never substitutes for a verdict.

Search margins are the exception worth naming: they are tuned against a
specific eval scale and node distribution, and Rarog's were re-fitted after the
HCE changed. Import search constants only as SPSA seeds, never as values.

Structural transcription of whole subsystems is still avoided, for the ordinary
engineering reason that a ported subsystem carries assumptions its new host may
not satisfy -- not for licence. The frozen `hybrid` branches remain diagnostic
artifacts; they are never merged or shipped.

## 3. Accepted foundation through 2.3.2 and RAR-S70

| Work | Evidence / disposition |
|---|---|
| Broad selectivity fit | Accepted at +15.33 +/- 7.34 nElo |
| Zero-reduction LMR floor | Accepted at +9.13 +/- 5.45 nElo |
| Anchored HCE refresh | Accepted at +11.56 +/- 5.19 Elo, RAR-E05 |
| Typed TT evidence and provenance | Retained infrastructure; behavior-neutral at accepted defaults |
| Root abort/fallback and correctness coverage | Retained infrastructure |
| AArch64 TT prefetch | Accepted at +1.42% median NPS on M4 |
| Phase-4 ProbCut move filter | Accepted at +15.56 +/- 10.02 Elo, RAR-S57/S58 |
| Root-only LMR relief | Accepted at +2.33 +/- 1.85 Elo, RAR-S70 |

Retained default-off switches are not evidence. Each must be consumed by its
named step or removed:

| Owner | Retained surface |
|---|---|
| 4.15 | TT provenance consumers and raw/pruning/searched evaluation separation |
| 4.18 | Unconsumed continuation/capture correction and history alternatives |
| 4.18 or removal | NMP/IIR/singular provenance alternatives; extensions remain a measured null |
| 4.16 | `SelectivityProspectiveDepth` and cp-valued margins whose populations move under the fitted HCE |
| 8.0 | `RootConfPoolInstability`, `SmpIterationSkip` and high-thread ownership |

## 4. Phase 4 — strongest bounded pre-NNUE search and HCE

### Objective and measured interpretation

The clean no-adjudication RAR-O02 hybrid gave two aggregate observations:

| Contrast | Result | Meaning |
|---|---:|---|
| Stockfish search minus Rarog search, Rarog HCE held | about **+196.5 Elo** | Mature search can use Rarog's HCE much better; not an individual mechanism forecast |
| Stockfish HCE minus Rarog HCE, Stockfish search held | about **+328.6 Elo** | HCE remains a major population; not a sum of portable family gains |

The later matched search ablation initially appeared to assign 116 Elo to LMR
and 124.6 Elo to shallow pruning. Four LMR candidates then measured flat even
though Rarog's LMR base formula matched the reference within 2% and Rarog
ordered better. The corrected conclusion is the phase's central constraint:
the ablation differences measured each mechanism's marginal value inside a
different engine, not Rarog implementation headroom.

Fixed-node measurement subsequently corrected the residual too. Rarog is
**355.26 Elo behind at equal nodes**, its speed closes **104.5 Elo**, and after
the mask-160 comparison the non-LMR/non-shallow residual is about **83 Elo**,
not the obsolete 30. Current counters place qsearch and TT in that population:
Rarog runs about 1.60x the oracle's qsearch per node, hits the TT more and
converts less. Those remain high-value search questions, but Basilisk showed
that an HCE refit can materially move qsearch share while leaving other search
counters stable. Since Rarog's HCE population is larger and its complete
parameter surface has not been requalified, HCE qualification/refit is now
4.7–4.14 and the search-authority decision follows on the accepted HCE at 4.15.

### Completed steps

| Step | Status and durable conclusion |
|---|---|
| **4.0** Evidence, baseline and oracle freeze | Closed by RAR-M12 |
| **4.1** Instrumented oracle | Closed on `hybrid-diag` `de568b3` |
| **4.2** Differential observation harness | Closed by RAR-S55; all counter units and invariants must remain explicit |
| **4.3** Mechanism map and order freeze | Closed; reference divergences are questions, never target values |
| **4.4** Matched-ablation instrument and fixed-node correction | Closed; every mask bit proved live; marginal-value interpretation corrected |
| **4.5** LMR contract study | Closed with no accepted interior gain after four candidates; RAR-S70 root relief remains accepted |
| **4.6** Shallow-selectivity/rewrite continuation | **Closed with no accepted gain**; details below |

#### 4.6 closed disposition

- **4.6.1 Quiet SEE prune:** `QuietSeePruneDepth=6`, coefficient 25,
  completed 652 paired-score games against the oracle at
  **-247.39 +/- 23.69 Elo**. Against `G(0) = -250.77 +/- 13.12`, estimated gap
  closure is only **+3.38 +/- 27.08 Elo**. This is a stopped diagnostic null,
  not an SPRT boundary; the candidate stays off.
- **4.6.2 SearchCore rewrite:** Steps 13 and 16 were rebuilt together. It
  changed the fingerprint from 6,977,070 / 2.466 to 3,479,169 / 2.343 and
  solved 182/300 WAC positions against 167/300 on fewer nodes, but at the
  stopped paired sample it scored **-9.76 +/- 17.70 Elo** over 712 complete
  games, LOS 13.76%. It never reached the registered `[-5,5]` boundary. The
  wholesale rewrite was reverted by `c5e451d` because its structural and
  constant effects were inseparable and its best zero-game signals did not
  predict play.
- **4.6.3 Decision:** no selectivity SPSA and no second broad search rewrite.
  The planned SPSA entry condition was not met: neither an accepted replacement
  contract nor a shrinking matched gap exists. Re-entry requires new local
  evidence after the fitted HCE, not another Stockfish-shaped port.

### 4.2a Harness and instrument integrity sweep

Zero games. Basilisk's audit surfaced two harness failure classes that Rarog
shares: a native command whose nonzero exit was swallowed by the calling
PowerShell, and a user-facing option accepted in a mode that could not honor
it. Both produce a completed run with a plausible number that measures
something other than what was asked.

1. **4.2a.1 — done, `cb5ed2a`.** `sprt.ps1` could not start any gate invoked
   without `-OptionsA/-OptionsB`, including the registered RAR-E06 command:
   `d2c7788` dropped the empty-list return from the option-advertisement
   guard, and `$splitOpts` unrolls an empty pipeline to `$null`, which a
   `[string[]]` parameter rebuilds as a one-element array holding `$null`. The
   same `$null` also emitted a bare `option.` argument to fastchess. Fixed and
   verified in three directions. The `-NoAdjudication` wire, which had never
   played a game, was proved live over 20 games ending 20/20 by a rules
   result.
2. **4.2a.2 -- done.** Swept `tools/*.ps1` for native invocations whose
   `$LASTEXITCODE` is never read and for exit status taken through a pipe. None
   found: direct invocations check the status, and the measurement scripts
   assert on their PARSE instead, which is stronger -- it verifies the number
   exists rather than that the process returned 0.
3. **4.2a.3 -- done, `334c084`.** The match anomaly guard was zero-tolerance
   and had never run a match; it declared RAR-E06 invalid over 3 forfeits in
   3,915 games. Split by evidence: crash, illegal move, disconnect and protocol
   error stay absolute, time forfeits take a 0.5% ceiling that sits two orders
   of magnitude clear of both healthy runs (0.03-0.33%) and genuinely poisoned
   ones (5.5%, 34.9%).
4. **4.2a.4 -- done, `3fb9f57`.** Every parameter a script advertises must be
   honored or refuse to launch. `sprt.ps1` accepted `-Games` in modes that
   ignore it, exactly Basilisk's defect; an option silently ignored in one mode
   is the same class as a dead `--rset`.

### 4.2b Time-forfeit margin at test concurrency

Zero games to diagnose; a gate for any fix. RAR-M14 measured the floor from
RAR-E06's 3,915-game PGN and it is not what it looked like.

Every game ends having spent **97-99% of its entire clock**. The five longest
games in the match, 367 to 494 plies, sit at 97.5-98.7% and do not forfeit.
The three that did forfeit were 90, 98 and 121 plies -- **shorter** than the
131-ply median -- and one flagged while its own reported move times summed to
only 94.5% of budget. So this is neither clock mismanagement nor long-game
exhaustion. It is the gap between engine-reported thinking time and
harness-measured wall time, set against an aggregate slack of about 2% of a
~4.9s budget: roughly 100ms for a whole game. One descheduling event of that
size, with all 14 physical cores running engines while fastchess contends for
the same silicon, is a forfeit.

`Move Overhead` defaults to 10ms. `time_manager.rs` reserves `2*overhead` only
below ~520ms of clock, and its 30ms `smp_reserve` is gated on `threads > 1`,
so a single-threaded engine under a saturated runner has no equivalent
protection. The comment recording `0/3,460 at Threads=1` no longer holds at
this concurrency.

**4.2b.1 — done.** The diagnosis above closes this step. It is a measurement,
not a repair: every fix it implies is owned by **4.17**, because a change to
a time-management default alters playing behavior and takes its own gate.

### HCE maturity conclusion

The current-code comparison is
`analysis/hce_maturity_2026-08-25.md`. The old 2026-07 audit is historical:
its four concrete activation defects (`attacked2`, enemy rook/passers,
unstoppable passers and phalanxes) were fixed by `d5a6054` and are tested.

Rarog already has a broad approximately 1,200-parameter tapered HCE with
trace reconstruction, caches, material/PST, mobility, threats, nonlinear king
danger, imbalance, passers, specialized endgames, lazy evaluation, rule-50
damping and correction history. It is not yet mature under the Phase-4 bar
because current conditional semantics and residual calibration are incomplete,
and no whole-HCE fit has been run after the representations this programme may
change.

No HCE parameter family is frozen by historical acceptance. Material, PSTs,
mobility, threats, imbalance, sparse terms, king-danger inputs and every other
current `EvalParams` surface are covered by the exact 4.7.3 audit. The ten
material/PST gauge anchors and two invariant king values are the only fixed
slots. Non-parameterized scaling/endgame contracts remain structural questions,
not silently frozen coordinates.

| Family | Current maturity question |
|---|---|
| Score foundation | Phase/tempo/rule-50/lazy ordering and sign-preserving winnability |
| Material/PST/imbalance | Material-conditioned residuals versus compensating correlated terms |
| Pawns/passers | Blocker, file, exchange, race and conversion conditionality |
| Activity/space/threats | Pin-aware legal activity, usable space, safe pawn pushes and exact relations |
| King safety | Shelter/storm dimensionality, castling destination, pinned defenders, weak/flank inputs |
| Scaling/endgames | Exact material conversion, OCB scope, Syzygy-backed won/drawn/cursed separation |
| Calibration/data | Archive/content and all-slot audits complete; publish/freeze the qualified split, fit the full surface, review movement/cohort covariance, then measure full/lazy/search residuals on the accepted HCE |

Stockfish comparison may enumerate and test these contracts. Reciprocal family
ablation is optional coarse sensitivity evidence only; it cannot rank build
order by itself or assign recoverable Elo.

Manta strengthens the process, not the expected-value estimate. MAN-E19's
coherent coverage-plus-constrained-fit bundle won +35.91 +/- 11.19 Elo while
costing 36.2% evaluator throughput; MAN-E18/E20 show that a lower static loss
cannot rescue a semantically wrong or weak feature, and MAN-E21 shows that a
plausible faster mechanism can still lose games. Therefore categorical
semantics and instrument contracts precede their fits, while a complete
existing-surface refit precedes new structure. Static/NPS filters may reject
but never promote, and the whole fitted cluster pays its own game and
search-NPS gate.

Basilisk supplies a new ordering prior, not portable values. Its recent HCE
programme added sixteen terms and lost 77.92 Elo, then gained about 12 Elo by
removing those terms and refitting two old surfaces that had been incorrectly
excluded: nonlinear king safety (+2.64) and 768 PST weights (+9.52). A fit
reported as 348/1,190 parameters had hidden the omission. Its larger holdout
improvement lost while the 14-times-smaller one won. Therefore Rarog audits and
fits the complete existing representational surface before adding features,
and never ranks candidates by validation delta.

### 4.7 HCE data and instrument qualification

#### 4.7.1 Archive provenance, labels and capacity — COMPLETE

`analysis/hce_archive_audit_2026-08-31.md` is the durable record. The two
archives contain 600,000 independent starts, zero replays/parse errors,
6,501,318 unique eligible positions, exact WDL labels, 6,428 natural mates and
26,935 material signatures. Their manifests bind one clean predecessor engine,
one book/seed, disjoint ranges, 8,000 nodes/move and conservative
`datagen-v1` adjudication.

The previous 3,000,000-row target is impossible: train openings stop at
460,752 instead of the required 600,000. The measured exact contract is
**2,300,000 train + 127,778 validation + 127,778 frozen test**, equally
phase-balanced. More datagen is not owed before this corpus receives a verdict.

#### 4.7.2 Atomic corpus publication and hash freeze — COMPLETE

`hce-v2` contains 2,300,000 / 127,778 / 127,778 rows with every input/output
hash frozen. All targets are literal white-perspective self-play results
(`0`, `0.5`, `1`; blend 1.0). Its 600,000 games use disjoint entries 1–600,000
of `beast_seed.epd`; the book has 750,000 unique four-field FENs and no
duplicates. The runner now rechecks the CSV label domain and row counts, book
hash/cardinality/uniqueness, sidecars, non-wrapping ranges and replay count.

#### 4.7.3 Complete parameter-to-instrument audit — COMPLETE

`8d8f507` enumerates all 1,218 `EvalParams` scalars with an exact partition:

| Primary disposition | Slots | Instrument |
|---|---:|---|
| Identifiable linear surface | 1,194 | Sparse traced Adam fit |
| Nonlinear king-danger selectors | 12 | Integer coordinate re-evaluation |
| Material/PST algebraic gauge | 10 | Pin square 0 for pawn–queen in each phase |
| Invariant king material | 2 | Fixed at zero |

The nonlinear instrument also co-tunes the 40-entry king-safety table. The
`complete` group includes PSTs and every historically staged/sparse family;
“already tuned” freezes nothing. Strict complete vectors carry each stage into
the next. Reconstruction now compares directly with independently accumulated
`EvalTrace::raw`, rather than the former tautological residual check.

The end-to-end smoke moved nonlinear values through both linear stages, baked a
changed source and 7,170,826 / 2.468 candidate fingerprint, restored the source
byte-for-byte, forced a recompilation and recovered exact RAR-S70 at
6,977,070 / 2.466. CSR storage measured about 76 nonzero coefficients per
position, making the whole-surface run practical.

#### 4.7.4 Current-source Stockfish maturity map — COMPLETE

`analysis/hce_maturity_2026-08-25.md`, updated by the Manta/Basilisk and native
audits, classifies every existing family. It licenses the complete current
surface fit, not a Stockfish feature port. Missing richer king, conversion,
passer/threat and winnability conditionality remains post-fit structural
residue. Raw/lazy/corrected/qsearch/depth-N search interaction is intentionally
measured at 4.15 on the accepted HCE, because this fit can move those
populations.

### 4.8 Refit the complete existing HCE surface — COMPLETE, RAR-E06 ACCEPTED

This step tests whether Rarog is mis-calibrated before assuming it is
under-featured.

#### 4.8.1 Registered offline fit

The immutable schedule executed by `tools/texel/fit_complete.ps1` is:

| Setting | Registered value |
|---|---|
| Initialization | Current clean source vector at the invoking commit |
| Corpus | 2,300,000 train; 127,778 validation; 127,778 frozen test; equal five-phase reservoirs; seed 42 |
| Calibration | Fit K once on baseline validation, then pin it for every stage |
| Gauge/invariants | 10 PST anchors and 2 king-material zeros from 4.7.3 |
| Linear optimizer | Complete 1,194-slot sparse Adam, 200 epochs, learning rate 0.3, L2-to-stage-prior `1e-7` |
| Nonlinear optimizer | 12 danger selectors + 40 safety-table entries; 200,000 train positions; integer coordinate descent, at most 40 epochs |
| Alternation | nonlinear -> complete linear -> nonlinear -> 60-epoch complete linear polish |
| Selection | Best validation checkpoint within each fixed stage; serialize/reload the integer vector, then compare it by full re-evaluation with an explicit saved source vector; frozen test opened once after final selection |
| Stop | Exactly two nonlinear/linear cycles; no post-hoc epoch, schedule, data or K change |

Before fitting, the runner re-audits/publishes the corpus, verifies exact
1,218-slot coverage and trace reconstruction, emits full feature support plus
baseline cohort losses, and checks the accepted benchmark. After fitting it
emits every trajectory, parameter delta, validation/cohort table, one-shot test
loss, final complete vector, source patch, candidate binary/benchmark and
hashes. It tests the baked candidate in debug/release and clippy, then restores
source and the normal release binary. Offline loss is evidence, not Elo.

The first production invocation (`hce-fit-20260831_095443`) completed every
optimizer stage, but its frozen report compared stage 3 with the floating-point
polish vector rather than source with the persisted integer candidate. Its
captured patch was also text-corrupted. Those are harness failures, so the
reported delta is retired and no game gate is licensed. The repaired runner
saves source defaults explicitly, reloads the rounded final vector, evaluates
both with the full nonlinear evaluator, fixes cohort membership to the source,
and verifies that the raw UTF-8 patch applies after restoration. Because
`hce-v2/test.csv` was consumed, an untouched confirmation set from unused
opening starts was required; do not reopen or rename the original test.
`tools/texel/confirm_hce_fit.ps1` freezes the completed candidate and K, builds
a clean source engine, generates one game from each unused book entry
600,001--750,000, and hash-splits those 150,000 independent starts 50/50. It
extracts 127,778 equal-phase test positions from the held-out half (the other
127,778 positions are published but select nothing), mechanically verifies
literal WDL targets and provenance, then runs only the repaired one-shot exact
source-to-rounded-candidate comparison. Stockfish evaluations are never read.

That confirmation completed in `hce-confirm-20260831_230548`: 150,000 new
pure-WDL games from starts 600,001--750,000 produced 127,778 untouched test
positions with zero parse failures or replayed starts. The exact rounded
candidate improved loss from **0.12330291 to 0.12252203** (delta
**-0.00078088**, about **-0.63%**) and improved every registered broad cohort.
This closes 4.8.1 and licenses review and a prospective game gate; it does not
establish Elo.

SPSA is skipped here: deterministic traced and re-evaluation instruments own
every existing coordinate, so there is no unexplained live nonlinear residue
to justify it. A later small residue may enter SPSA only with activation,
interaction and curvature evidence.

#### 4.8.2 Review, bake and register the game gate

Review `summary.json`, every stage log, feature support, parameter movement,
semantic bounds, cohort regressions and the one-shot test result. A malformed,
unsettled or semantically invalid fit is rejected without games. If it passes,
apply the recorded eval patch on a clean branch, build final PGO, measure pooled
NPS and register one no-adjudication SPRT against the pre-refit HCE. Bounds and
budget are chosen prospectively from the observed prior using RAR-M10; offline
loss magnitude does not choose them.

Completed by RAR-E06. The exact vector changed 439/1,218 slots, stayed within
declared bounds, passed debug/release/all-target verification, and reproduced
**7,226,051 / 2.460** in clean PGO against **6,977,070 / 2.466**. Three-build
pooled PGO measurement found a real **-1.19% NPS** cost (95% bootstrap CI
**-2.29% to -0.48%**), which the clock gate must price. The prospective gate
is `[0,3]` nElo, 80,000 games, no adjudication. The common Basilisk/Rarog
fastchess, book, TC, concurrency and affinity instrument already owns the null
calibration; symmetrically omitting draw/resign termination changes duration
and variance, not arm placement, so the maintainer waived a duplicate 30k
null. Exact hashes, seed and stop rules are frozen in `EXPERIMENTS.md` RAR-E06.

#### 4.8.3 Strength verdict — ACCEPTED

RAR-E03/RAR-E04 and Basilisk established why the gate was mandatory: large
loss improvements can be neutral or catastrophically wrong, while Basilisk's
accepted +9.52 Elo came from only -0.43% holdout loss.

The gate resolved on 2026-09-01. H1 at **3,914 games**: **+22.04 +/- 7.51
Elo, +32.05 +/- 10.88 nElo**, LOS 100.00%, LLR 2.95, 44 minutes of wall time.
The complete vector is accepted whole and the accepted fingerprint moves to
**7,226,051 / 2.460**. Full record, artifact hashes and the anomaly
disposition are in `EXPERIMENTS.md` RAR-E06.

Two things are worth carrying forward. The effect was eight times the
bracket's upper bound, so RAR-M10's 47,200-game sizing overshot by an order of
magnitude -- sizing from an expected value is right, but a large true effect
resolves in a fraction of the budget, and the cap is not a schedule. And
offline loss again predicted neither sign nor magnitude: -0.63% test loss
preceded +32 nElo here, while Basilisk's -6.2% preceded -77.92 Elo.

### 4.8a Post-refit redundancy removal — CLOSED, NO GATE OWED

Basilisk's BAS-E25 removed terms that a complete covariant fit had shown to be
redundant and gained `+0.49 +/- 2.96` Elo as a simplification. The inventory
ran on the accepted vector (RAR-E07) and **the analogue does not transfer**:
BAS-E25 removed sixteen terms a previous Basilisk phase had *added*, and
Rarog's existing surface has no equivalent accumulation.

1. **4.8a.1 Inventory -- done.** From artifacts that already existed:

- The fit drove only **5 of 1,218** slots to zero, and switched **17**
  previously-zero slots back on. Three of the five are whole 1-slot terms:
  `passed_candidate_mg`, `passed_freestop_eg_per_rank`,
  `threat_safe_pawn_push_eg`.
- Of the 132 slots under the sparse cut, **90 have zero activations and are
  structurally unreachable** -- pawn PST ranks 1 and 8, passers and phalanxes
  on those ranks, threats against a king, impossible imbalance combinations,
  and the two king-material gauge zeros. **Unreachable is not redundant.** A
  pawn PST carries 64 entries because the index space is 64; removing the 16
  that cannot occur restructures an index space for no runtime gain.
- 12 more are the nonlinear danger selectors and 12 are co-tuned safety-table
  entries. The remaining 18 are rare but real, and **all 18 held** -- the
  fitter froze every under-supported coefficient rather than fitting noise
  into it.

Two instrument confirmations came free. The 12 fields with zero linear
activation are exactly `KS_DANGER_INPUTS`, independently reproducing 4.7.3's
1,194 + 12 + 10 + 2 partition from a different artifact; and the freeze
behaviour above is the sparse-cut contract working as specified.

2. **4.8a.2 Removal -- no cluster exists.** The three zeroed terms are inert --
they multiply by zero -- so deleting their code is behavior-neutral and
provable by the exact fingerprint, not a strength question. **4.18.2 owns that removal, not a gate.** One caveat for
whoever does it: `eval.rs`'s `new_terms_activate_on_curated_positions` asserts
that `passed_freestop_eg_per_rank` still *traces*, which it does even at a
zero coefficient. Deleting the feature breaks that test, so the test's
precondition changes in its own commit.

Two residuals go to 4.9 as observations, not candidates: the fit priced
candidate passers at zero in the middlegame and safe pawn pushes at zero in
the endgame. Those are statements about the current representation, and 4.9
decides whether a different representation expresses them better.

### 4.9 Structural HCE upgrades — CLOSED, NO CLUSTER OPENED

Open at most two dependency-complete structural clusters, and only for residual
signals the full existing-surface refit could not represent.

**4.9 is closed on RAR-E09's measured entry evidence.** Candidate passers
priced at zero in the middlegame and safe pawn pushes priced at zero in the
endgame were observations the surface already represented, not proof of a
missing feature. The completed residual analysis below found a label defect
instead. The rules here remain the contract for a justified future reopening.

1. **4.9.1 Entry evidence -- DONE, RAR-E09.** See
   `analysis/hce_residuals_2026-09-01.md`. It found **no residual the existing
   surface cannot represent**, and found a label defect instead: KR-K, a 100%
   theoretical win, is labelled a draw on 75% of its corpus positions, because
   Rarog scores a won rook ending at 426-487 cp while `datagen-v1`'s resign
   rule needs 600. The evaluator predicts 0.849 there against a label mean of
   0.625, so it is closer to the truth than its own training data and the fit
   has been pulling it down.
2. **4.9.2 Decision -- CLOSED, no cluster opened.** The cohort table in
   `analysis/hce_residuals_2026-09-01.md` covers 4.9's own named hypotheses and
   none of them licenses structure: king-attack sits +3.8% and threat +3.1%
   above global loss on very broad populations (67% and 93% of positions), and
   passer sits **6% below** it. The largest residual, KR-K, is fully
   representable and is a label defect. The opening cohort's +41% is outcome
   entropy, not evidence of mis-modelling -- separating "harder" from "wrongly
   modelled" needs an oracle's loss on the same positions, which does not
   exist here.

   **Retry trigger.** Reopen 4.9 only on a residual that a coefficient value
   cannot fix, measured on a non-degenerate cohort. Note that the passer
   bucket was itself degenerate until 2026-09-01 -- it selected 127,777 of
   127,778 positions -- so any pre-fix cohort claim about passers is void.
   RAR-E08 accepted tablebase-corrected labels, firing the follow-up trigger:
   correcting the labels may expose a residual the mislabelled data hid.
   That post-label audit remains owed; 4.14.7 owns the refreshed
   residual review and any resulting reopen/keep-closed disposition.
   King-safety and passer/threat conditionality remain hypotheses, not an
   order. Specialized endgame knowledge has independent evidence from the
   conversion audit: 4.9a supplied the foundation and 4.12 owns development.

For each cluster:

1. define categorical semantics and directional/counterfactual tests;
2. reconstruct every changed feature exactly through `EvalTrace` or a named
   nonlinear instrument;
3. locally refit the changed feature and **all materially covariant existing
   parameters**—historical group boundaries do not freeze them;
4. apply prospective semantic/support/loss/NPS filters as refutation only;
5. bake final PGO and run the registered no-adjudication SPRT;
6. accept or revert before selecting the next cluster.

Two fully fitted cluster failures close structural expansion and force a 4.7
re-audit; they do not authorize more feature inventory.

### 4.9a Endgame truth foundation -- DONE, five results SUPERSEDED

**Why this is in Phase 4, not Phase 5.** It was placed in Phase 5 on
2026-09-01 by following Basilisk's phase layout, which is not a reason. Three
Rarog-specific facts put it here:

1. These are HCE value and scale functions. Phase 4 is the phase that owns the
   HCE; Phase 5 is the NNUE runway and is required to be behavior-neutral.
2. 4.20 releases 2.4.0. Shipping a release that converts KBNK at 15% and then
   fixing it afterwards is the wrong order.
3. 4.14 consolidates the whole HCE after the last structural change. If
   endgame structure landed in Phase 5, 4.14 would be invalidated and a second
   whole-surface consolidation would have to be paid for.

This block keeps its `4.9a` numbering because `EXPERIMENTS.md`, `TRACKER.md`
and commit messages cite it. The OPEN work that used to live here was
renumbered on 2026-09-04 into 4.10-4.13 and 4.14-4.20; section 13 maps the old
numbers to the new ones. Completed leaves are never renumbered.

`analysis/endgame_conversion_audit_2026-09-01.md` is the baseline. Its
original table measured the pre-refit source; it has been re-run on the
accepted HCE with the same seed and positions. At 60,000 nodes/move over 100
fixed-seed random positions per family, KQK/KRK/KBBK/KBNK converted at
**94%/86%/76%/15%** before the refit and **94%/91%/86%/19%** after, against
Basilisk's latest **100%/100%/87%/54%**. At n=100 the binomial standard error
is ~3.5 pp, so only the KBBK movement is comfortably outside noise, and the
aggregate artifacts do not retain per-position outcomes for a paired test. **KBNK remains catastrophic at 19%**, with 73 of 100 games still dying on the
fifty-move rule. That is the audit's prediction holding: a complete
recalibration of every coefficient moved KBNK by about one standard error,
because the defect is gradient magnitude against the pruning margins that
consume it, not coefficient calibration. Fitting cannot repair a term whose
actionable signal is smaller than the margins it must survive. KBNK has
Basilisk's same Chebyshev plateaus, sub-pruning gradients and missing
piece-coordination guidance, and the complete HCE fit did not remove the
material-specific drawn-subset bias either. This is a
mandatory engineering program, not a feature-shopping license. The 4.9 cap of
two structural clusters does not apply: that cap bounds speculative feature
addition against residual signals, while this is defect repair against a
measured conversion failure and an explicit reference inventory.

The 4.9a sub-steps below are the completed record and are no longer the working
order. GUIDE and the execution/hold register determine the earliest unblocked
leaf; 4.11b remains between re-measurement and 4.12 endgame development.

**The order is set by a feedback loop, not by convenience.** Self-play labels
depend on the engine's own conversion ability, which depends on the evaluator,
which depends on the fit, which depends on the labels. It closes. Two things
act on it, and they decide the sequence:

- **Below 7 men, Syzygy is an external anchor** -- truth that does not depend
  on Rarog at all. That is what RAR-E08 decides how to use, and it is the only
  non-circular input available.
- **Above 7 men nothing anchors it**, so iteration is genuinely required. 4.14's
  mandated refit loop is where that is paid for; it is not a design failure.

The consequence is that **conversion improvements must precede data
regeneration**, because they improve the engine that produces the next round of
labels. So 4.9a ran label-independent work first (4.9a.2-4.9a.4), fixed the
label contract (4.9a.5) and regenerated (4.9a.6); 4.12 fits families only after
4.10 repairs the instrument and 4.11 re-measures what it produced.
Each turn of the loop then starts from a better generator rather than
re-deriving the same weakness. The bracketed `ref N` is the item's identity in the
20-function reference inventory, which is a different ordering and is preserved
so nothing is lost when the two are compared.

- **4.9a.1 Truth corpus -- DONE; result SUPERSEDED -> 4.11.1.** `tools/diag/endgame_truth.py` grades every
  strong-side move against Syzygy rather than every game against the clock,
  which is what fixes the +/-3.5 pp resolution problem of a 100-game
  conversion rate. Artifact
  `tools/results/hce-accepted/endgame-truth-accepted.json`, per-position
  records retained so a later run over the same seed is paired.
- **4.9a.2 Endgame-start cohort -- DONE.** `tools/diag/endgame_book.py` writes
  a Syzygy-verified EPD book of endgame starts, so the families that never
  arise from UHO openings can be measured at all. RAR-M15's tier 4 -- KQKR,
  KQKRPs, KRPPKRP -- occurred exactly zero times in 3,915 real games, so their
  cohort must be constructed rather than sampled.

  `endgame_cohort_v1.epd`: **788 unique positions across 21 families**, seed
  `0x4E9A2`, 60% theoretical wins and 40% theoretical draws, every position
  probed before it is written and its verdict recorded in the EPD comment.
  SHA-256 `D23A51CD01CC3ADEEB94AABE7239A6F44C14F297FF97A750203FDDB5DA9F7942`.
  `tools/books/` is gitignored, so the generator plus this seed and hash are
  the reproducible record.

  Drawn positions are included deliberately: a book of wins measures conversion
  only, and holding a draw is the other half of endgame skill -- it is also
  where the audit found the evaluator overconfident. Colour is not baked in;
  the harness plays each start from both sides, so the cohort is a paired A/B
  rather than a test of who drew the strong side.

  Three families are short and the shortfall is reported rather than padded:
  KQ-K and KR-K have **no drawn subset** to sample, and KNN-K yielded only 4
  wins in 24 because KNN vs K is drawn almost everywhere -- consistent with
  4.9a.1 finding it drawn in 100 of 100 positions. KRPPKRP is excluded
  entirely: seven men against six-man tables.

  **This book is an instrument, never training data.** Its positions are
  uniformly sampled rather than drawn from play, so feeding them to a fit would
  reweight the corpus toward a distribution the search never sees.
- **4.9a.3 Regression contract -- DONE; floors half SUPERSEDED -> 4.11.2.** The contract has two halves and they
  need opposite treatment: correctness is absolute, statistics are not.

  **Hard vetoes, in `tests/endgames.rs`.** Two EPD verdicts backed by 64
  Syzygy-verified cases spanning 17 reference families, generated once with
  seed `0xC0FFEE` and frozen into `tests/endgames.epd` so the suite needs no
  tablebases at run time. `tb-win`: a position Syzygy calls won must not score
  as drawn or lost -- sign only, because RAR-E09 measured a won KR-K at +426
  cornered and +487 centralised, so a tighter floor would be a calibration test
  that any refit trips. `tb-draw`: a drawn position must not be claimed as
  forced mate -- the mate claim alone, because a drawn KR-KP really is a rook
  up and demanding a small score there would assert a recognizer that does not
  exist. These span families deliberately: the audit's complaint about the
  older cases was that they test "direction and local mate recognition, not
  class-wide conversion".

  **Aggregate floors, in `tools/diag/endgame_floors.py`.** Conversion,
  win-preserving and DTZ-progress rates per family, compared with a 5-point
  noise allowance -- at n=100 the binomial standard error is about 3.5 points,
  so a bare "must not decrease" test fails on resampling alone. Floors
  **ratchet**: `--update` raises them from a passing run and refuses to lower
  any floor without `--allow-lower`, which exists to make "never relax a
  correctness test in the implementation commit" awkward to do by accident.
  Baseline floors are committed at `tools/diag/endgame_floors.json`, taken from
  the accepted HCE's truth corpus.

  Long fixed-search trajectories stay diagnostic: investigate individual
  movement, accept or reject in context, and never require every position to
  keep the same PV or mate length.

- **4.9a.4 Mate-drive cluster -- DONE; isolation accounting SUPERSEDED and
  re-accounted by RAR-M23; debts -> 4.12.7/4.12.9.** RAR-E10.
  KBN-K **19.4% -> 96.9%**, KBB-K **78.0% -> 100.0%**, `bench 13` unchanged at
  7,226,051 / 2.460, floors ratcheted. Three axes were needed -- resolution,
  magnitude and ratio -- and the ratio was nearly missed: the diagonal shape
  was first tested at ~1:1 against the king terms, measured worse than the
  Chebyshev version it replaced, and recorded as non-transferring. At ~6:1 it
  is the entire gain. Sweeping a mechanism's shape while holding its
  proportions fixed can refute it for the wrong reason.

  KBN-K's residue is now 4 positions where the engine gives away the bishop or
  knight and 1 stalemate, with zero fifty-move losses where there were 61. That
  is a different defect and no drive weight addresses it.

  **ACCEPTED 2026-09-01 on maintainer judgement, with no game gate.** This
  departs from the stated rule that only a registered SPRT accepts a candidate,
  and is recorded as a judgement call rather than a gate. What justified it:
  bench byte-identical, activation triply gated (`|eval| > 200` AND a bare
  losing king AND no pawn, rook or queen for the winner), the then-recorded
  isolation argument, theory vetoes and floors passing, and a tier-3 occurrence
  of 0.28% at which a `[0,3]` gate cannot resolve anything at any budget this
  project has. RAR-M23 supersedes that isolation argument: six families change
  through the promotion closure, including net debt in KBP-KB and KBP-KN.

  What that does not establish: bench-identical proves only that the 40 bench
  positions' trees never reach a minor-piece bare-king mate within depth 13,
  while real games at 3+0.03 reach greater depth with endgames on the board and
  do fire the term in roughly 1.6% of games. **Retry trigger: any
  endgame-shaped strength anomaly reopens this without needing new argument.**

- **4.9a.4 (original scope) Search-visible magnitude audit.** Measure every guidance gradient
  against the pruning margins and resolution that consume it, starting with
  KBNK/KXK mate drive and passed-pawn king approach. This is where KBNK's
  actual defect lives -- the truth corpus shows its technique is within 60% of
  optimal on what it converts (efficiency 1.58) while DTZ progress is 0.277,
  the lowest of any pawnless family -- so the fix is gradient magnitude, not
  another refit. Texel may correctly fit a rare multi-ply guidance term toward
  zero for static WDL loss while search needs an actionable magnitude; resolve
  that with sweeps, conversion and DTZ progress, not by freezing either value.
- **4.9a.5 RAR-E08: label contract -- ACCEPTED, +6.73 +/- 3.82 Elo.** H1 at
  13,432 games, LOS 99.97%, zero forfeits. Arm B's vector was the accepted head
  at **7,165,683 / 2.462**, superseded by RAR-E12 at 8,044,078 / 2.481. Texel theory predicted arm A and lost by 10.34
  nElo: the self-reinforcing label loop was the stronger effect, and correcting
  1.325% of rows paid.

  **What is adopted is the post-hoc relabel of <=6-man positions**
  (`tools/texel/relabel_tb.py`), cursed wins as draws, on an otherwise
  unchanged corpus. It is NOT `datagen-v3`, which adjudicates the game on
  tablebase truth and so changes the recorded result of every position sampled
  from it, including openings. `datagen-v3` remains untested; adopting it
  because "tablebase labels won" would be adopting a different change.

  Conversion cost, resolved at n=400: KBN-K's -5.1 pp at n=100 was noise
  (+0.5 pp, SE 1.5), and the one real regression is **KQ-KP -3.8 pp at 2.9 SE**
  -- owner **4.12.13**, retry at **4.12.22**. Aggregate weighted conversion is
  flat, 83.24% -> 83.45%.

  **SUPERSEDED and corrected, RAR-M24, 2026-09-06.** The v1 aggregate is
  invalid. The full matched v2 pair is **1255/1372 = 0.9147 -> 1254/1372 =
  0.9140**, not 83.24% -> 83.45%. The matched v2 400-position focus pair
  reproduces KQ-KP **390/396 -> 375/396, -15, -3.79 pp** exactly, so its
  historical causal debt remains with 4.12.13. RAR-M21 also shows the current
  head's 60k KQ-KP deficit closes at 200k and 600k; it is not a persistent
  deployment deficit. The Elo verdict is unaffected -- fastchess played those
  games, not this harness -- and so is the adoption. Text above remains as the
  historical record; evidence is `analysis/conversion_claims_correction_2026-09-06.md`.

- **4.9a.6 Regenerate on the winning contract.** Only after RAR-E08 reports.
  Hash-freeze under a new name; never edit `hce-v2` in place, since it is the
  corpus the accepted head was fitted on and has to stay reproducible.

  **The start book was the constraint, not the schedule.** `beast_seed.epd`
  holds exactly 150,000 positions in each of the five buckets, and the
  extractor's phase is MATERIAL, not ply, so a game started below phase 20 can
  never produce an `opening` row. Splitting the RAR-E08 pilot by the phase of
  each game's own start and preflighting each split shows only opening starts
  feed the opening bucket (3.4392 rows/game; every other start bucket is zero)
  while also being the most productive overall (13.31 rows across all buckets,
  because one game traverses every phase on the way down). Four fifths of a
  balanced book therefore cannot contribute to the bucket that binds, and the
  preflight -- which sizes GAMES -- asked for 1,113,504, more than the 750,000
  openings in the book. Unreachable at any schedule.

  Supply was never short: the read-only store `A:\Chess\Beast\data\txt\
  positions.txt` is ~125M positions, 36.8% of them opening-bucket, duplicate
  rate 0.02%. 150,000 was a quota.

  `tools/texel/build_book.py` builds a phase-WEIGHTED book, defaulting to the
  hedged **50/10/10/10/20**. The yield-maximising corner is 68/10/0/0/22; the
  hedge is taken instead because the corner makes every middlegame and endgame
  row a *reached* position correlated with the opening play that led there.
  `phase_book_v1.epd` (1,000,000 positions, seed `0x5EED2`, SHA-256
  `31E9B655...`) measures **1.6227** rows/game in the binding bucket against a
  predicted 1.720, and sizes 3.5M rows at **602,619 games**, about 5.3 hours at
  the pilot's 1,905 games/min. Openings 997,001-1,000,000 were consumed by the
  validation run, so the real segment starts at 1.

  Two secondary levers were measured and NOT taken: `--skip-start 2` costs 9%
  of opening rows (3.7796 -> 3.4392 on opening starts; Basilisk uses 0), and
  raising `--max-per-game` from 16 buys rows with within-game correlation
  rather than with games. Both are held so the corpus contract differs from
  `hce-v2` in the book alone. Derivation and reproduction commands:
  `analysis/texel_corpus_book_shape_2026-09-02.md`; the matrix regenerates with
  `python tools/diag/book_yield.py <datagen.pgn>`. The whole pipeline --
  resources, tools, settings, contract gates and traps -- is
  `analysis/texel_fitting_handbook.md`.

  **Done: `hce-v3` and `hce-v3-tb` are published and gate-verified.** 602,619
  games generated in 5 h 10 m (1,941 games/min), 3,888,888 rows extracted
  (3,500,000 / 194,444 / 194,444), every per-phase quota met exactly with the
  tightest bucket at 1.44x headroom, 0 parse errors and 0 replayed starts. The
  Syzygy relabel changed 113,046 train labels (3.230%) with 0 probe failures.

  The corpus is materially better than `hce-v2`, not merely bigger: 40
  adjudicated games against 312,918 (52.2%), 367,664 natural mates against
  6,428, mean 91.0 plies against 66.4. `hce-v2` resigned most games out, so the
  evaluator learned from outcomes that were asserted rather than played. **A
  gate on this fit therefore conflates row count, phase mix and label
  provenance** -- a legitimate cluster, but it must be registered as one.

  Two gaps found and fixed on the way, each in its own commit: `relabel_tb.py`
  never emitted a corpus manifest, so its output was not fittable and RAR-E08's
  was hand-built (`90e8939`); and `fit_complete.ps1`'s contract gates were
  pinned to `datagen-v1` at exactly 600,000 starts, so they are now a NAMED
  list of `(profile, starts)` pairs, verified to still reject an off-by-one
  count, a crossed pair and the untested `datagen-v3` (`ec34a34`).


- **4.9a.7 KRPKR scale -- DONE; conversion framing SUPERSEDED -> 4.12.2.** The
  DRAWN-cohort result stands and is the result the step was actually about: the
  share of theoretically drawn KRP-KR positions scoring above +100 cp fell
  **37.1% -> 25.8%**, measured by `tools/diag/endgame_drawn.py`, which plays no
  games and is untouched by the instrument defect. Only the reference's DRAW
  branches were ported; its two winning branches amplify above the neutral 64
  and an untested amplifier against a Texel-fitted surface is not something a
  drawn cohort can measure. The step is bench-VISIBLE (8,044,078 -> 6,901,489)
  because a depth-13 search reaches rook endings constantly, unlike 4.9a.4.

  **What is reopened is the conversion framing, not the mechanism.** The claim
  "52% conversion is not the defect, Stockfish manages only 47.9%" compared
  Rarog's 52% from `hce-accepted` with Stockfish's 47.9% from `reference-sf18`,
  two artifacts sharing **zero of 1,900 positions**, and both arms aborted 62 of
  100 KRP-KR games on correct rook-for-rook technique. The same-set pair is
  49.3% against 47.9%. Re-derived at 4.11.3.

- **4.9a.8 KRPKB scale -- DONE; conversion framing SUPERSEDED -> 4.12.6.** Same
  disposition. The drawn cohort's overclaim was 0.9574 before and after and only
  the mean moved, +347.2 -> +324.4; the reference addresses only ROOK pawns and
  returns partial scales, so a +350 evaluation scaled by 24/64 is still +131.
  The mechanism was asserted directly (`Some(24)` on a verified fortress, `None`
  on a non-rook pawn) because a narrow-mechanism null and a dead-wire null are
  indistinguishable in an aggregate -- that part of the design is sound and is
  the pattern later steps should copy. Its 95.7% residual overclaim at mean +347
  reads as material-imbalance pricing of rook-and-pawn against bishop rather
  than a missing recognizer, and is assigned to **4.12.22**.

**The rest of 4.9a moved.** The twenty reference functions, their gates and
their closure were 4.9a.9-4.9a.28; they are now **4.12**, behind the instrument
repair (4.10) and the re-measurement (4.11) that decide their order. The two
regressions owned inside that list travel with it: RAR-E08's **KQ-KP -3.79 pp**
and RAR-E12's **KBN-K dtz 0.7260 -> 0.6753**. RAR-M24 re-measured both with
the corrected instrument. KQ-KP remains a historical causal debt but closes
at 200k/600k in the current head; KBN-K remains a corrected matched-arm
regression whose individual-term attribution is unisolated.

### Superseded results, 2026-09-04

`analysis/endgame_truth_instrument_audit_2026-09-04.md` found three defects in
the endgame instrument and one in the mate drive's isolation argument.

**How an invalidated result is recorded, and why not as an open checkbox.**
The first attempt at this marked the affected leaves `REOPENED` and left their
boxes open, borrowing a convention from Basilisk that exempts a reopened leaf
from roadmap ordering. That was wrong here. Basilisk needs the exemption
because it did not renumber; Rarog did, so the repair already lives in properly
ordered leaves further down, and none of these five is independently
actionable -- 4.9a.1's repair simply IS 4.10.1 plus 4.11.1. An open box that
only a later leaf can discharge makes the board unrunnable top to bottom, which
is the one thing GUIDE exists to be.

So: **the STEP stays ticked, because it was done; its RESULT is marked
`SUPERSEDED -> <leaf>`, naming the open leaf that repairs it.** The debt cannot
evaporate, because the named owner is itself an open checkbox and 4.10.10 makes
the checker refuse a `SUPERSEDED` marker whose owner is missing or already
ticked. The rule generalises: nothing sits open ahead of the work that
discharges it.

- **4.9a.1 (truth corpus) -- DEBT DISCHARGED 2026-09-04.** Repaired at 4.10.1 and
  re-measured at 4.11.1; the corrected baseline is
  `tools/results/truth-v2-head/` and
  `analysis/endgame_truth_v2_baseline_2026-09-04.md`. The original defect, kept
  for the record: `endgame_truth.py` ended a playout the
  moment the strong side's piece count dropped. Shedding material is the winning
  method in most pawn technique. On the arm PLAN's numbers come from, the abort
  fired **264 times; 129 on clean wins, and 122 of those before the engine had
  played a single non-win-preserving move**, at a median abort ply of 5-20.
  Aggregate conversion 0.8345 was bounded above by 0.9235 from the old
  records; the matched re-run measured **0.9140** on the same binary.
- **4.9a.3 (regression contract) -- DEBT DISCHARGED 2026-09-05.** The floors were
  re-derived at 4.11.2 from the corrected head arm and the old file is kept as
  `endgame_floors_v1.json`. The original defect, for the record: the 64 frozen
  theory vetoes in `tests/endgames.rs` are static verdicts, play nothing and
  stand. The aggregate floors do not: their pawn-family conversion values are
  depressed by the abort and so are lenient exactly where 4.12 works next, and
  the run that produced the current `endgame_floors.json` **exists nowhere** --
  `tools/results/` is gitignored and no artifact on disk carries its numbers.
  Re-derived at 4.11.2.
- **4.9a.4 (mate drive) -- SUPERSEDED, isolation accounting only, owner 4.11.9.** The measured
  gain stands and is unaffected: in all six bare-king families any strong-side
  material loss reaches an insufficient-material position, which is tested one
  line earlier, so the abort is unreachable there by construction and zero
  `material_lost` outcomes appear in those families across all ten artifacts.
  What does not stand is the isolation ARGUMENT. RAR-E10 recorded "15 of 19
  families exactly unchanged"; per-position comparison of the same two artifacts
  gives **13 of 19**, and the change reaches **KBB-K, KBN-K, KPP-K, KBP-K,
  KBP-KB and KBP-KN** -- the last two each losing one conversion. The route is
  knight promotion: the dispatcher fires on a bare losing king with no pawn,
  rook or queen for the winner, and under-promotion manufactures exactly that
  material. A term's blast radius is its dispatcher condition's **promotion
  closure**, not the condition. Re-accounted at 4.11.9.
- **4.9a.7 and 4.9a.8 -- SUPERSEDED, conversion half only, owners 4.12.2 and
  4.12.6.** As above.

Deliberately NOT superseded, with reasons, so this is not relitigated later:

- **4.9a.2 (endgame cohort book)** probes every position before writing it and
  plays nothing. It is untouched. What it does owe is 4.10.7's development /
  held-out split, which is new work rather than a defect.
- **4.9a.5 (RAR-E08)** is a game verdict decided by fastchess, not by this
  harness. Only its conversion-cost side note is contaminated, and that is
  corrected in place at 4.11.10.
- **4.9a.6 (corpus regeneration)** did not use the playout instrument. Its label
  contract is nonetheless questioned from a different direction at 4.13, on
  evidence the corpus itself will supply at 4.11.8.
- **Every SPRT in the ledger.** Games were played by fastchess; this instrument
  never touched them.
- **`endgame_drawn.py` and every drawn-cohort number**, including 4.9a.7's
  37.1% -> 25.8%. It evaluates statically and plays nothing.

### 4.10 Instrument integrity and tooling upgrade

**This runs first, before any re-measurement and before any further endgame
work.** Every verdict in 4.11-4.13 is read through these tools, and the failure
that produced this step is the one AGENTS.md names as the dominant one: the
check that was run did not check what it was thought to check. Basilisk found
the same defect independently and its correction moved a 770-position baseline
by 68 positions; Rarog's own numbers are in
`analysis/endgame_truth_instrument_audit_2026-09-04.md`.

Nothing here changes the engine. These are tooling commits.

1. **4.10.1 Truth-instrument termination rule -- DONE.** The material abort is
   replaced by a TABLEBASE-TRUTH stop: the game plays on and the shed ply is
   recorded as the diagnostic `shed_material_ply`. `first_discard_ply` already
   carried the truth signal. `endgame_conversion.py` is deliberately untouched
   -- it covers only bare-king families, has no insufficient-material test of
   its own, and there `material_lost` is doing that job correctly.

   **The report schema is now `rarog-endgame-truth-v2`, and that is part of the
   fix rather than bookkeeping.** A v1 and a v2 report use the same field names
   for different quantities, so comparing them would manufacture a large fake
   improvement in exactly the pawn families 4.12 is about. `endgame_floors.py`
   rejects a v1 report by schema, and rejects any floors file lacking
   `truth_schema: rarog-endgame-truth-v2` -- which the committed
   `endgame_floors.json` does, so it fails closed until 4.11.2 re-derives it.

   Verified in `tools/diag/test_endgame_truth.py` (9 tests): the behavioural
   test, the shed-ply test and the no-`material_lost` test were each shown to
   **FAIL against the restored defective rule** before being accepted, per rule
   15. The bare-king isolation test passes under both rules, correctly -- it
   asserts that insufficient material terminates first, which was always true
   and is why RAR-E10 is safe. A live smoke run (4 positions per family, 3,000
   nodes, 40 plies -- a smoke, not a measurement) shows a KPP-K game shedding a
   pawn at ply 2 with the win intact and playing on, where v1 would have scored
   it a failure, and a KBN-K minor giveaway still ending as
   `insufficient_material`.
2. **4.10.2 Cohort identity -- DONE.** Every truth report now carries a
   `cohort` block -- family list, seed, positions per family, a SHA-256 over
   each family's FEN sequence and one over the fold of those -- and
   `endgame_floors.py` refuses to compare across differing digests. This was
   not hypothetical: three artifacts on disk share zero of 1,900 positions with
   the current generator, one of them is the artifact PLAN cited as the
   baseline, and the floors tool compared across them while its own comment
   asserted the two runs shared positions.

   **Comparison is PER FAMILY, deliberately.** A single-family re-run is a
   legitimate thing to do -- the family seed derives from the family NAME
   exactly so a subset reproduces the full run's positions -- so requiring the
   overall id to match would forbid it for no reason. What is refused is
   comparing a family measured on one position set against the same family
   measured on another.

   Position generation moved out of the play loop into `generate_family`, so
   the cohort is known before the first engine call and 4.10.3 can address
   positions by fixed index. **The refactor was proved position-identical**:
   regenerating all 19 families and comparing in order against
   `tools/results/e08-accepted/endgame-truth.json` matched **1900/1900**.

   **FROZEN COHORT IDENTITY, by content rather than by SHA.** The standard
   cohort -- seed `6200600`, the 19 `DEFAULT_FAMILIES` in declaration order,
   100 positions each -- has overall digest
   `fe4866045506636f884ee30526b4188c3def9ca9747f5960ea5c5e7cba5dbb5e`, with
   KBP-KB at `b730954492fafc8a30a8a3a4ee6e6d83eb3fdf8031fa8a9e1a6584eb830d32cb`.
   Both are pinned in `tools/diag/test_endgame_truth.py`, and a one-bit change
   to the family seed was shown to fail four tests. **4.11.1's two arms must
   both report that overall digest**; anything else is not a re-measurement of
   this baseline.
3. **4.10.3 Parallel playout -- DONE.** `--workers N` shards the cohort
   round-robin across independent one-thread engine processes and reassembles
   by fixed index. `--workers 1` stays a plain single-engine loop and is the
   REFERENCE; both paths feed the same `summarize()`, so identity is
   structural rather than hoped for.

   **Sharding is only safe because a position's result does not depend on what
   the engine played before it, and that was measured, not assumed.** Running
   KBN-K alone and again preceded by KPP-K in the same engine process gave 5/5
   identical per-position records: python-chess sends `ucinewgame` per position
   and Rarog resets on it. Had that failed, sharding would have silently
   changed results.

   **Verified end to end:** 3 families x 8 positions at 3,000 nodes, serial
   versus `--workers 5`, JSON identical apart from the recorded `workers`
   field -- same SHA-256 -- and no engine process left behind.

   **One real bug was found and fixed inside this leaf.** The first design kept
   the engine in a module global filled by a pool initializer and closed by
   `atexit`. Every shard finished, `24/24 positions` printed, and the pool then
   hung forever with five live `rarog.exe` children: closing a python-chess
   engine from an `atexit` handler races its asyncio loop thread. The engine's
   lifetime is now the task's, explicitly, and a test asserts the old shape
   cannot return. Worth recording because the failure was invisible in the
   log -- the run looked complete and simply never wrote its report.
4. **4.10.4 Prove the guards fire -- DONE, and the KBNK anchor did not.**

   **The finding: `kbnk_positions_are_driven_to_mate` passed under a broken
   drive.** Cutting `MOPUP_DIAGONAL` from 360 to 15 -- a 24x reduction, roughly
   the pre-4.9a.4 scale that converted KBN-K at 19.4% -- left the test green.
   That is Basilisk's BAS-E39 reproduced here rather than imported: an anchor
   passing under the very vector it exists to catch. Three causes, all of them
   the same mistake of not reproducing the instrument's conditions:

   - a **fresh `Searcher` per move**, so an empty transposition table every
     move, where the measurement persists one table across the game;
   - a **fixed depth of 10** instead of the instrument's 60,000-node budget;
   - a **40-ply move budget**, which cannot admit a discriminating position at
     all. The head needs **45-75 plies** from a centre-king start, so 40 plies
     silently restricted the suite to near-corner cases -- which mate even with
     a broken drive. The single frozen case was one of those.

   **Repaired and verified both ways.** One searcher for the whole game, a
   60,000-node budget, a 90-ply budget, and three discriminating positions
   frozen into `tests/endgames.epd`. Chosen empirically rather than by taste:
   head and mutant were built as separate binaries and run over 24 KBN-K
   positions at 60,000 nodes, giving **head 20/24 versus mutant 5/24** and 16
   positions where the head mates and the mutant does not; the three shortest
   (45, 49 and 55 plies) were frozen. The repaired anchor now **FAILS on the
   mutant and passes on the head**. Selecting positions on the mutant's failure
   is legitimate here and is stated rather than hidden: this is a regression
   ANCHOR against a known-bad state, not an estimate of anything.

   Cost is real and recorded: the endgame suite goes from 0.6s to **1.5s in
   release and 18.4s in debug**. That is the price of a suite that can fail.

   **Thin-sample refusal** is added to `endgame_floors.py` as `MIN_ELIGIBLE = 5`,
   sized against the cohort rather than by taste -- the smallest theoretical-win
   counts on the frozen set are KNN-K at 1 and KNN-KP at 23, so 5 excludes the
   degenerate family and keeps every real one. A rate below it is printed as
   thin rather than as a number, because the failure being prevented is a
   CONFIDENT wrong reading: one eligible position that fails reads as 0.0%,
   which looks like catastrophe and is emptiness. `kbnk_positions_are_driven_to_mate`
   also had no count guard at all and now has one; its sibling Syzygy vetoes
   already carried `checked >= 30` and `checked >= 25`.

   **The floor gate itself is now proved to fire**, end to end through the CLI:
   it blocks on a large family regression, stays quiet on equality and on a
   small dip, and a thin n=1 family at 0% cannot manufacture a verdict.

   The fingerprint obligation was already met by construction at 4.10.2 -- the
   cohort digest hashes FENs and nothing else, so adding a diagnostic field
   cannot move it. `shed_material_ply` was added at 4.10.1 and the digests did
   not change.

   Remaining thinness, recorded rather than papered over: the `kbnk-mate` set
   is 4 positions and the `tb-win`/`tb-draw` sets are 30 and 25. Widening the
   KBNK set belongs to **4.12.14**, which owns that family.
5. **4.10.5 Measurement-layer contract -- DONE.**
   `analysis/endgame_measurement_layers.md` states the four layers -- **theory
   truth** (per move, tablebase WDL), **move quality** (per move, DTZ progress
   and win-preservation), **conversion** (per position) and **game strength**
   (per game pair, SPRT at a real TC) -- with **occurrence** gating whether the
   first three can ever reach the fourth, and **drawn-share bias** recorded as
   the conversion-shaped measurement that is conversion's complement rather
   than a fifth layer.

   Precedence, each rule carrying its case: truth is an absolute veto and
   outranks conversion; conversion NEVER establishes strength; move quality and
   conversion can move in opposite directions on one change; strength never
   overrides truth; occurrence prioritises and is never evidence of value.
   Layers are never aggregated -- there is no exchange rate between a truth
   failure and a conversion gain -- and when two disagree, the disagreement is
   usually the finding. Bench identity and static fit loss belong to no layer.

   **Stamped, not merely documented.** `endgame_truth.py` writes a `layers`
   block declaring all four and marking game strength `NOT MEASURED HERE`;
   `endgame_conversion.py` and `endgame_drawn.py` each stamp their single
   layer, the latter as `drawn_share_bias` with a note that it plays no games
   and so is untouched by the RAR-E14 defect. `endgame_floors.py` prints its
   layer, node budget, ply limit and cohort digest above every verdict. Five
   tests enforce it, because a contract nothing checks is a wish.

   The document ends with 4.9a.7 worked through all six readings, since that is
   the step the contract would have saved: read on layer 3 it did nothing, read
   on drawn-share bias it moved 37.1% -> 25.8%, and the second is the one a
   SCALE function is validated on.
6. **4.10.6 Node budget as a first-class run condition -- DONE, and the screen
   budget turns out to be 2.6x below deployment.**

   `tools/diag/nodes_per_move.py` measures what a move actually costs by
   playing self-play games under a real clock, with the harness decrementing by
   MEASURED wall time so the engine receives genuine `wtime/btime/winc/binc` and
   exercises its own time management -- a fixed `movetime` would measure a
   different code path (RAR-M01). At **3+0.03**, 492 moves over 4 games on the
   accepted head: **median 153,466 nodes/move**, mean 176,208, p25 114,780,
   p75 210,088, p90 319,892, and **115,899 median in the endgame band**.

   **The maintainer's 12,000-game Colosseum arena on an Apple M4 independently
   gives ~148,000** (2.0 M nps at 74 ms/move, same TC) -- a different
   instrument on different hardware agreeing to within 4%. That agreement is
   the reason to believe the number rather than merely to have it.

   **Consequence, stated plainly: the 60,000-node endgame screen sits below the
   p25 of deployment**, so every fixed-node endgame verdict this project has
   taken is PROVISIONAL in the sense of rule 12. It does not make 60,000 wrong;
   it makes a verdict that turns on a move a 116,000-node search would see
   unfalsifiable at that budget, which is precisely how Basilisk rejected its
   own leading KBNK candidate (BAS-E45).

   **The 60k / 200k / 600k bracket is therefore justified rather than copied:**
   against this distribution 60,000 is below p25, 200,000 just above p75 and
   600,000 near the observed maximum. `tools/diag/endgame_budget_bracket.py`
   drives `endgame_truth.py` unchanged at each budget over the same cohort --
   one report, one budget, per the layer contract -- and REFUSES to tabulate if
   the arms measured different position sets, which would be RAR-E14's defect B
   with extra steps.

   **The primary budget stays 60,000 for 4.11.1.** Changing it in the same step
   as the termination rule would confound the one delta that step exists to
   isolate; 4.11.7 owns the bracket runs. Evidence:
   `analysis/node_budget_2026-09-04.md`. Re-measure after any time-management
   change (4.17) or large NPS movement.

   Prefer nodes to depth for cross-variant work -- equal depth is unequal work
   once an eval change shifts pruning, and favours whichever side prunes
   harder.
7. **4.10.7 Held-out confirmation tooling -- DONE.** `tools/diag/holdout.py`.

   **The split follows the POSITION, not its index.** Assignment is a hash of
   the FEN, so extending or reordering a cohort cannot reshuffle which half
   decides -- the same defect class as seeding a family by its list index, and
   the same fix.

   **The registration is written once and refuses to be rewritten.** Changing
   which half decides after seeing results is the same act as moving SPRT
   bounds and is equally invisible afterwards. It also refuses a registration
   with no runner-up, or one naming a runner-up outside the declared arms:
   Basilisk's leader WAS rejected on held-out data, and only the second arm
   kept that step from ending with nothing.

   **The paired test is McNemar's on the discordant positions only**, because
   agreement carries no information -- 200 positions both arms convert say
   nothing about which is better. Below 6 discordant positions the result is
   reported as INDETERMINATE rather than as a z, the same thin-sample
   discipline as `MIN_ELIGIBLE`.

   **`separation()` reports a plateau as a plateau.** "Best of N" without
   separation from its neighbours is not a winner; Basilisk's never separated
   (paired z +0.76 to +1.70).

   **A cohort that produced a verdict is SPENT for selection and still valid as
   a veto.** The asymmetry is not fussiness: "this candidate discards a won
   position" is a safety property and does not get less true from reuse, while
   "this candidate converts 74%" is an estimate and does, because the candidate
   was chosen partly on this data's noise.

   Verified: 23 tests, each guard exercised on a known-bad input as well as a
   good one, and the CLI refuses live to pair two reports over different
   cohorts.
8. **4.10.8 Datagen label audit -- DONE.** `tools/diag/datagen_label_audit.py`
   walks a PGN corpus, probes to the installed man-limit, and reports the share
   of tablebase clean wins the game did not win.

   Three decisions the number depends on, each one a way to get it wrong:
   a **missing table is UNKNOWN, never agreement** -- probing past the limit
   silently converts "no table" into "the label was right"; **cursed wins are
   excluded**, because WDL 1 is already drawn by the fifty-move rule and a game
   drawing one is correct play, not a defect; and **only the FIRST clean win a
   game reaches counts**, since later positions are consequences of how that
   one was played and counting them all would weight long technical endings for
   no reason.

   **Both denominators are reported**, because they answer different questions:
   `not_won / clean_wins` is how badly the endings are played, `not_won /
   games` is how much of the CORPUS carries a wrong label. Per family too, since
   the bias is expected to concentrate in rook and pawn endings.

   Sharding is by byte offset -- python-chess 1.11 has no `scan_offsets`, so
   `skip_game` in a loop supplies them -- with fixed-index round-robin so a
   sharded run audits exactly the games a serial run does.

   **Smoked on a real 2,700-game gauntlet PGN**: 299 games (11.07%) reached an
   adjudicable clean win, 27 of those (9.03%) were not won, so 1.00% of that
   corpus carries a contradicted result, with KRPP-KR the largest family at
   4/44. That is a MATCH corpus at tournament TC, not datagen, so it is a smoke
   test and not the measurement -- 4.11.8 runs this against `hce-v2` and
   `hce-v3-tb`, where the node budget is 8,000 and the share should be much
   worse.

   **The smoke found a crash the unit tests missed.** On a corpus reaching no
   clean win, `summarize` returned `None` correctly and its test covered that,
   but the PRINT path formatted `None` as a percentage and died. Crashing was
   the better of the two available failures -- printing 0.00% would have read
   as "no defect" on a corpus with no data -- and it is now `n/a`. Recorded
   because a unit test passing over the function while the caller is broken is
   exactly the shape 4.10.4 is about.
9. **4.10.9 Gate-runner provenance -- DONE, and mostly already there.**
   Auditing before changing anything found `tools/sprt.ps1` already refusing on
   a binary/manifest SHA mismatch, a non-bench verification, a tune build, a
   build-flavor mismatch, a compiler mismatch and identical binaries outside
   calibrate mode, and already recording repo revision, TC, adjudication, hash,
   threads, concurrency and affinity. 4.2a did that work; this leaf did not
   need to redo it and the record says so rather than claiming the ground.

   **Two real gaps, both closed.** A dirty tree was a WARNING and is now a
   REFUSAL: AGENTS.md's evidence rule says a ledger row must reproduce its
   artifact without the branch it came from, and a binary built from
   uncommitted changes cannot, by construction. A warning there is read once
   and forgotten, and by the time the row is questioned the tree is gone.
   `-AllowDirtyTree` exists for a deliberate throwaway screen and must be
   justified in the registration. And `-ExpectRevision` refuses to start unless
   both manifests record the registered revision, because a gate measuring a
   different revision than the one it registers is not evidence for that
   revision.

   **The termination policy is now written into the artifact**, naming the
   pooling hazard in the manifest itself: an adjudicated run must never be
   pooled with a natural-termination one, since two sampling processes with
   different draw rates bias a pooled estimate by the mixing ratio.

   **All three verified live against the real script**: the dirty tree refuses,
   `-AllowDirtyTree` passes it with a loud warning, `-ExpectRevision cafe9999`
   refuses, and a matching revision still reaches the flavor and compiler
   checks -- so the guards fire without a false refusal.

   Standing rules this leaf does not automate, recorded where the runner is
   used: never resume an SPRT after the candidate, either engine's options,
   book, TC, adjudication policy or hardware changed -- start a new experiment
   ID. **An interim reading is not evidence**: -4.29 +/- 6.06 at 28% of the way
   to a bound recovered to -1.40 +/- 4.07 by 7,720 games. **Build speed is a
   real confound**: two separately built binaries can differ ~1% in NPS with no
   behavioural cause, worth a couple of Elo at fast TC, so measure parity on an
   IDLE machine with INTERLEAVED repeats. Identical bench proves the SEARCH
   identical; it never proves the speed.
10. **4.10.10 Roadmap checker -- DONE.** `check_guide.py` now understands the
    `SUPERSEDED -> <leaf>` marker and enforces all three of its properties: it
    may sit only on a TICKED leaf (the step was done; it is its RESULT that is
    superseded), it must name a leaf that EXISTS, and that leaf must be
    UNTICKED. Together those stop an invalidated result from quietly becoming a
    closed one. **Scheduling clarification, 2026-09-05:** the former rule
    that the first unticked box always runs is superseded by the explicit
    hold/dependency register. The earliest unblocked leaf is next; held work
    stays unticked and visible. This checker validates structure, not machine
    availability or dependency readiness.

    The step-set comparison now runs BOTH ways. The old check was one-way, so a
    PLAN item with seven sub-steps listed as five in GUIDE passed -- and did,
    with GUIDE's titles also off by one against PLAN's. Distinguishing a PLAN
    DEFINITION from a cross-reference is the whole difficulty: PLAN writes a
    definition as `**4.10.1 Some title...**` and a reference as either bare
    prose or bold-with-nothing-after, so the pattern requires a title to follow
    the number. Without that, every owner pointer in the prose would be read as
    an undefined step.

    **Verified by mutation, four ways**, each against the real files: a
    SUPERSEDED marker moved onto an unticked leaf fails; pointed at an
    already-ticked leaf fails; pointed at a nonexistent leaf fails; and
    deleting one GUIDE sub-step that PLAN defines fails naming it. Restored, the
    board is clean at 146 steps.

    `--next N` was added earlier as a slice of this leaf and prints the
    actionable queue generated from the checkboxes, so the work list cannot
    drift from the board the way a hand-written one would.
11. **4.10.11 Shipped-constant guards -- DONE.** `src/eval.rs` derives
    `MOPUP_MAX = MOPUP_DIAGONAL * 7 + MOPUP_KING_CHEB * 7 + MOPUP_KING_MAN * 14`
    -- the exact supremum, since each factor peaks at those values -- and
    asserts `MOPUP_MAX < MATE_SCORE - MOPUP_ASSUMED_MAX_PLY` in a `const _: ()`.

    **A `const` assertion is the point, not a stylistic choice.** It holds in
    every build type including release, where a validator inside an
    option-setter only exists in tuning builds. Basilisk shipped exactly that
    gap: its bound was enforced in the setter, the RELEASE default was validated
    by nothing, and its existing test exercised the validator rather than the
    constant (BAS-E52). If this bound ever fails, the drive can manufacture a
    mate score out of king geometry and the search will believe it.

    **The cross-module half.** `MAX_PLY` lives in `search.rs` and is private --
    it is in fact duplicated privately in three modules already -- and the
    evaluator should not depend on the search to know its own safety bound. So
    `eval.rs` mirrors it as `MOPUP_ASSUMED_MAX_PLY` and
    `search::tests::mopup_mirror_matches_the_real_ply_horizon` ties the two
    together in the module that legitimately sees both. The mirror cannot drift
    silently.

    **BOTH assertions verified to fire**, per rule 15: raising
    `MOPUP_DIAGONAL` to 5,000 fails the build with the written message, and
    changing the mirror to 64 fails the test with its own. Restored, and the
    fingerprint is **6,901,489 / EBF 2.458** exactly -- byte-identical to the
    accepted head, which is what a behaviour-neutral engine change owes.
12. **4.10.12 Build-configuration audit -- DONE.**
    `tools/diag/feature_matrix.py` enumerates all **16 subsets** of the four
    declared features (`tune`, `diag`, `ablate`, `texel`) and runs
    `cargo check --all-targets` on each. **All 16 compile**, in 52 seconds
    total -- `check` rather than `build` is what makes the whole matrix
    runnable on demand instead of only in CI.

    **Single-feature coverage is not enough, and that is demonstrated rather
    than argued.** Injecting a defect behind
    `#[cfg(all(feature = "diag", feature = "ablate"))]` leaves the default
    build, `--features diag` and `--features ablate` all GREEN, and only
    `diag,ablate` fails. CI checked exactly those three and would have shipped
    it.

    CI gains the matrix and a plain `ablate` build: `ablate` was the one
    declared feature CI never built at all, and it is the paired-ablation
    instrument -- needed the moment an ablation is wanted, which is never a
    convenient moment to discover a build error.

    **`--all-features` is deliberately not used** anywhere in the matrix. It is
    not a shipped configuration: it enables `texel`, which bypasses the eval and
    pawn caches, and AGENTS.md records a depth sweep whose conclusion was
    reversed by exactly that binary being left in `target/release/`. The tool
    prints that reminder on success and flags every combination whose binary
    must never be measured.

    A test ties `SHIPPED_FEATURES` to `Cargo.toml`'s `[features]` block, so
    adding a feature and forgetting to cover it fails the suite -- a matrix that
    silently stops covering a feature is worse than no matrix, because it
    reports success over a shrinking set.

### 4.11 Re-measurement and re-derivation

Nothing in 4.12 onward may be ordered or gated on a number produced by the old
instrument. This step re-measures what the repair invalidates and re-derives
everything computed from it. Corrections are recorded IN PLACE: superseded text
is marked superseded rather than rewritten, and experiment identifiers are
stable.

1. **4.11.1 Re-run both truth arms -- DONE.** Three arms, one frozen cohort
   (`fe486604...`), 30 workers.
   Full record: `analysis/endgame_truth_v2_baseline_2026-09-04.md`.

   **The instrument delta, isolated.** The first comparison written for this
   step compared the v1 RAR-E08 arm against the v2 CURRENT-head arm, moving two
   things at once -- exactly the confound this cluster exists to prevent. Caught
   by checking the recorded engine paths rather than by remembering which binary
   was which, and repaired by re-running the RAR-E08 binary, which still
   existed. Matched arms:

   | Arm | v1 | v2 | Instrument delta |
   |---|---:|---:|---:|
   | RAR-E08 head | 0.8345 | **0.9140** | +109, +0.0794 |
   | reference | 0.9016 | **0.9920** | +124, +0.0904 |

   Engine only, both under v2: RAR-E08 head 0.9140 -> **current head 0.9300**.

   Exactly six families move on the instrument fix -- KRP-KB +40, KRP-KR +32,
   KPP-K +23, KBP-KN +8, KBP-KB +4, KBP-K +2 -- and every bare-king family moves
   by zero. The isolation argued at RAR-E14 and proved by construction now holds
   empirically at full scale.

   **Corrected baseline: head 1276/1372 = 0.9300, reference 1361/1372 = 0.9920,
   deficit 85 positions.** Paired matrix: both 1273, head only 3, reference only
   88, **neither 8**. The genuinely hard residue is eight positions. The v1
   paired matrix cannot be computed for comparison -- `reference-sf18` was run
   without `--per-position` -- which is why this step required re-running the
   reference arm rather than re-analysing it.

   **Defect C is closed by reproduction.** The floors recorded KBN-K conversion
   0.8980 (n=98) and dtz progress 0.6753 (n=3178) from a run that existed
   nowhere; the current-head arm reproduces both to four decimals with identical
   n. KBN-K contains no `material_lost` outcomes, so v1 and v2 agree there by
   construction, which is what makes the reproduction meaningful rather than
   lucky.
2. **4.11.2 Re-derive the floors -- DONE.** `endgame_floors.json` is rebuilt
   from `tools/results/truth-v2-head/`, stamped `truth_schema:
   rarog-endgame-truth-v2` and carrying the cohort fingerprint
   `fe486604...`. **Weighted conversion floor 0.9300 over n=1371, 18 families.**
   The old file is preserved unmodified as `endgame_floors_v1.json` -- superseded,
   not deleted -- and the tool still refuses it by name, which is the 4.10.1
   guard doing its job on the real artifact.

   **KNN-K now has no floor at all, and that is correct.** It contributes one
   theoretical win in 100 positions, below `MIN_ELIGIBLE`, so all three of its
   rates are reported as thin rather than as numbers. A floor on n=1 cannot be
   breached at any sigma and would sit in the report looking measured while
   being unmeasurable. The family is not thereby unguarded: the hard theory
   vetoes in `tests/endgames.rs` are per position and do not care about sample
   size. This is why the aggregate denominator is 1371 and not 1372.

   **4.12.14's KBN-K target is re-derived and confirmed on a real artifact.**
   `tools/results/truth-v2-e08/` gives the RAR-E08 head **dtz 0.7260 over
   n=2989** and conversion **0.9184 (90/98)** -- matching the pre-`b711d4d`
   floors to the digit -- while the current head gives **0.6753 over n=3178**
   and **0.8980 (88/98)**, matching the post-`b711d4d` floors to the digit.
   Both halves of the lost run are now reproduced, so RAR-E14's defect C is
   fully closed rather than merely re-derived on one side. Note the reference
   reaches **0.7555**, so 0.7260 is a restoration target, not a ceiling.

   **The gate was proved to fire on real data, not on a synthetic input.**
   Judging the RAR-E08 arm against the accepted head's floors BLOCKS on KP-K
   (-3.6 SE), KQ-K (-3.9 SE) and KQ-KP (-4.3 SE) dtz progress, and reports
   KBN-K dtz improving +4.4 SE in that direction. Those are exactly RAR-E12's
   registered findings seen in reverse -- it claimed KP-K, KQ-K and KQ-KP
   improving beyond 2 SE and disclosed KBN-K at -4.4 SE. An instrument rebuilt
   on corrected data independently reproducing a registration it was not fitted
   to is the strongest check available here.
3. **4.11.3 Re-derive the attained reference results -- DONE.**
   `tools/diag/endgame_reference_results.py` freezes
   `tools/diag/endgame_reference_results_v1.json`, replacing RAR-E11 per family.

   **The name is load-bearing and so is the artifact's self-description.**
   Basilisk's equivalent field was called `attained_single_engine_ceiling`, was
   read downstream as an ACCEPTANCE TARGET, and was wrong in seven families by
   77 positions -- far too lenient in exactly the families the next phase would
   work (BAS-E50). This artifact carries its own limits in a
   `what_this_is_not` list: not a ceiling, not an acceptance target, not
   transferable to another budget or cohort, and the paired union proves only
   that each position was converted by at least one engine -- never that one
   engine can convert the union. A test asserts no FIELD is named a ceiling.

   **Validation precedes reproduction, and every check fails closed:** identical
   schema, cohort digest, per-family digest, node budget, ply limit, positions
   per family, seed, hash, family set, and exact FEN/theory pairing position by
   position. Two arms differing in any of those are not two arms of one
   measurement. All ten refusals are exercised on the input each exists to
   reject.

   | | Clean wins | Candidate | Attained reference | Deficit | Neither |
   |---|---:|---:|---:|---:|---:|
   | **total** | 1372 | 1276 | **1361** | **85** | **8** |

   Paired matrix: both 1273, candidate only 3, reference only 88, neither 8.

   **The hard residue collapsed to 8 positions and is almost entirely one
   family.** KNN-KP holds 7 of the 8; KRP-KB holds the last one. **Seventeen of
   nineteen families have a ZERO neither-bucket** -- every clean win in them is
   convertible by at least one engine at 60,000 nodes. There is no broad set of
   genuinely hard positions here; there is one hard family.

   KNN-KP is also the only family where the candidate converts positions the
   reference does not (2 of its 3 candidate-only positions; KRP-KR has the
   third). That is worth carrying into 4.11.6: a family where both engines are
   weak and they fail on DIFFERENT positions is a different proposition from one
   where the reference simply wins.
4. **4.11.4 Drawn-share bias census -- DONE, and it reorders the list.**
   1,500 positions per family over 19 families, tablebase-filtered to the
   theoretical draws, each searched at 60,000 nodes. Artifact
   `tools/results/drawn-census/drawn-v1.json`; full table in
   `analysis/drawn_share_census_2026-09-05.md`.

   | Family | drawn/1500 | overclaim | mean cp |
   |---|---:|---:|---:|
   | KRP-KB | 38 | **1.0000** | +328.1 |
   | KR-KN | 796 | **1.0000** | +346.0 |
   | KR-KB | 1002 | **0.9960** | +307.4 |
   | KBP-KB | 884 | 0.6086 | +142.0 |
   | KNN-KP | 1009 | 0.5768 | +159.6 |
   | KBP-KN | 635 | 0.5071 | +143.6 |
   | KRP-KR | 482 | 0.3071 | +84.0 |
   | KNN-K | 1499 | **0.0000** | +0.0 |

   **Rook against a lone minor is priced as close to winning, always.** KR-KN
   overclaims EVERY one of its 796 drawn positions and KR-KB 998 of 1002, at
   means of +346 and +307 -- far above the material edge the fitted evaluator
   assigns, so positional terms are stacking on top of it in positions that are
   theoretically dead. There is no drawishness scaling for these endings.

   **KNN-K is the control and it is perfect**: 1,499 drawn positions, zero
   claimed. That is what makes the failures legible as missing knowledge rather
   than as an instrument artifact. KQ-K, KR-K and KBB-K have no drawn subset at
   all, confirming from measurement what 4.9a.2 predicted from theory.

   **The two rankings barely overlap, which is why this had to precede 4.11.6.**
   KR-KB and KR-KN have conversion deficits of 2 and 3 -- among the smallest in
   the cohort -- and are its two worst drawn-share offenders. Ranking on
   conversion alone would have put them near the bottom. The mirror image is
   KQ-KR: the largest conversion deficit at 23, and only 2 drawn positions in
   1,500, so it is a pure technique problem. This is 4.9a.7's lesson arriving
   from the other direction -- there a working scale change was nearly called
   null because it was read on conversion; here three families would have been
   called healthy for the same reason.

   **Consequence for 4.12.1:** the provisional table classifies KRKB (ref 7) and
   KRKN (ref 8) as VERDICT functions, and their measured defect is scale-shaped.
   4.12.1 owns the classification and should revisit those two against this
   evidence rather than against the donor's taxonomy -- what matters is which
   instrument can see the defect.

   **Two instrument defects were found and fixed here.** The tool's numbers were
   ORDER-DEPENDENT: `engine.analyse` was called with no `game=` token, so no
   `ucinewgame` was sent between positions and the transposition table carried
   over. Caught by the serial-versus-sharded byte-identity check, where KBP-KB
   read 0.702 serially and 0.750 over six workers on the same positions; fixed
   by forcing `ucinewgame` per position, after which the two are byte-identical.
   Prior drawn-cohort numbers (4.9a.7, 4.9a.8) were paired within themselves and
   remain valid as comparisons, but their absolute rates carried this
   contamination. Separately, a completed census died on its write because the
   tool never created its output directory, losing 28,500 positions of work.
5. **4.11.5 Occurrence census split by root -- DONE. The defect is present,
   and the first answer said it was not.** `tools/diag/endgame_occurrence.py`,
   artifact `tools/results/occurrence/occurrence-v1.json`, write-up
   `analysis/endgame_occurrence_split_2026-09-05.md`.

   | endgame root = <= men | endgame roots | middlegame evaluations | share |
   |---:|---:|---:|---:|
   | 7 | 0 | 80,589 | **1.0000** |
   | 8 | 3 | 35,435 | **0.4397** |
   | 10 | 8 | 4,494 | **0.0558** |
   | 12 | 10 | 242 | 0.0030 |

   **Three roots out of forty produce 56% of every reference-family evaluation;
   eight produce 94%.** At a 7-man threshold the suite contains no endgame roots
   at all -- its smallest position is 8 men -- and the census looks perfectly
   clean. That was the first reading, and moving the line by ONE MAN turns it
   into "more than half of this measurement comes from three positions". The
   tool therefore sweeps and prints the whole curve; `--endgame-men` selects only
   which threshold gets the detailed table. A report quoting one threshold would
   be quoting a choice, not a measurement.

   Four families read **zero over all forty roots** and no threshold argument
   touches them: KBNK, KNNK, KNNKP and KRKN. The families that dominate the
   census -- KRPPKRP 5.88%, KQKRPs 4.41%, KRPKR 3.46% -- are exactly the ones
   that collapse when late-material roots are removed, and KQKRPs is reached only
   from endgame roots at a threshold as low as 8.

   **The tension for 4.11.6 is explicit: KR-KN has tree occurrence ZERO and the
   worst drawn-share bias in the cohort** (4.11.4: 796/796 overclaimed at mean
   +346). Those must be reconciled, not averaged.

   The honest conclusion is that **the bench suite is a weak instrument for this
   question**: 40 positions chosen to fingerprint the SEARCH, not to sample the
   game distribution. `bench_counters.py` gained per-position retention to make
   the split possible; summing stays its only printed output, so the rule that
   file exists to enforce is untouched. Verified under `--features diag` with
   `bench 13` reproducing 6,901,489 / EBF 2.458.
**Reordered 2026-09-05.** The re-ranking was numbered 4.11.4 and sat BEFORE
two of its own three inputs -- the drawn-share census and the occurrence census.
Ranking a list half made of SCALE functions on conversion deficit alone would
have repeated 4.9a.7's mistake in a new form, so the four open leaves were
renumbered to put the inputs first: drawn-share becomes 4.11.4, occurrence
4.11.5, the re-rank 4.11.6 and budget transfer 4.11.7. Section 13 records the
mapping. Prompted by the maintainer asking for the drawn-share census first,
which is what exposed the dependency inversion.

6. **4.11.6 Re-rank the reference-function list -- DONE and REGISTERED.**
   `tools/diag/endgame_ranking.py` freezes `tools/diag/endgame_ranking_v1.json`
   and 4.12's leaves are renumbered into that order, so the board runs top to
   bottom. Rules were fixed before the output was looked at, and are in the
   tool's header.

   **Registered order:** KRPKR, KRPKB, KXK, KRKP, KRKN, KBPKB, KRKB, KBPKN,
   KPK, KQKP, KPKP, KNNKP, KBNK, KQKR, KNNK, then MEASURE FIRST (KPsK, KBPsK,
   KBPPKB, KQKRPs), then KRPPKRP as unverifiable.

   **Ranked by DEFECT SHAPE, not the donor's taxonomy.** The larger of a
   family's conversion defect and its drawn-share defect decides its kind, so a
   family cannot be promoted by being mediocre twice and cannot be called
   healthy because the wrong instrument was read. That reclassifies **KRKP,
   KRKN, KRKB, KBPKN, KPK, KPKP, KNNKP and KNNK from verdict to scale** -- five
   of them sit in the top nine.

   **Three method corrections were made after seeing the first output and before
   registering anything.** A measured zero is not a certain zero: RAR-M15 found
   KQKR in 0 of 3,915 games, which by the rule of three bounds the rate at
   ~0.077%, so board occurrence is floored there rather than annihilating the
   family. Unmeasured is not unimportant: five functions have no cohort family
   at all and are grouped as **MEASURE FIRST** ordered by occurrence, not
   dumped at the bottom -- KPsK is 4.19% of games and has never been measured.
   And a family measured at zero defect is labelled "close it" rather than
   ranked as merely low.

   **Tree occurrence is a flag, not the multiplier.** In principle it is the
   more direct gate -- a scoring defect misguides the search wherever the
   evaluator is called. But 4.11.5 measured that instrument as weak (three of
   forty roots produce 56% of the census), so using it as a multiplier would
   grant it more authority than that finding allows. The two contradictions are
   named rather than averaged: **KRKN has 100% drawn-share bias and ZERO tree
   occurrence**, and **KQKRPs is 4.41% of the tree and 0 of 3,915 games**.

   **RETRY TRIGGER, registered now.** This order rests on board occurrence
   measured over 3,915 games and tree occurrence over 40 bench positions. A
   36,400-game rated tournament exists on disk. If family occurrence is
   re-measured over real games at real time control, **this ranking is
   re-derived and 4.12 renumbered again** -- the tool takes the artifacts as
   inputs precisely so that is a re-run rather than a rewrite. No leaf below
   4.12.1 should be worked without checking whether that measurement has
   landed.

   **TRIGGER FIRED, 2026-09-05 -- see 4.11.12.** The order above is SUPERSEDED
   by `tools/diag/endgame_ranking_v2.json`; it is kept here because a frozen
   artifact whose inputs cannot be reconstructed is not evidence, and because
   the correction it received is the point. Board occurrence was not merely
   under-sampled -- RAR-M15's classifier capped positions at six men, and its
   three zeros were not zeros.
7. **4.11.7 Budget transfer -- DONE, RAR-M21, 2026-09-06.** Protocol,
   per-family results, limitations and decisive cases are recorded in
   `analysis/endgame_budget_transfer_2026-09-05.md`; reproduction driver:
   `python tools/diag/run_4117_registered.py`.
   Repeated the decisive family verdicts at
   60k / 200k / 600k nodes. A verdict that does not reproduce at a
   game-representative budget is provisional: Basilisk rejected its leading
   candidate at 60,000 nodes on a losing move that a 200,000-node search sees.
   Use `tools/diag/endgame_budget_bracket.py` and the run/selection contract
   in 4.10.6–4.10.7. Deployment measured 153,466 nodes/move median at 3+0.03;
   60k is below p25 (`analysis/node_budget_2026-09-04.md`). Keep the corrected
   cohort and arm fingerprints fixed, name decisive cases before running,
   and record which conclusions transfer before 4.11b changes the board.
   **Measured:** all 19 families, 100 positions each, both engine arms and
   all three budgets; both 60k reports reproduced exactly. Rarog converted
   **1276/1336/1346 of 1372**, Stockfish **1361/1363/1362**; net deficits
   **85/27/16**. KBN-K and KQ-KP reach full conversion at 200k and 600k;
   their 60k shortfalls are not persistent higher-budget defects. KQ-KR's
   deficit persists at **23/13/3**, KNN-KP at **9/6/8**, KRP-KR at **5/2/1**,
   KRP-KB at **5/1/1**, KBP-KN at **2/2/2**, KP-KP at **1/1/1**. KR-KP
   closes at 600k. Gains hide losses: Rarog gains/loses **70/10** then **19/9**
   positions across successive budgets. Preserve paired FENs in each family
   review; causal search follow-up belongs to 4.15.1 only when demonstrated.
   Static drawn-share evidence and historical matched-arm debts are separate
   and remain owned. Full report and byte-preserving evidence ZIP:
   `analysis/artifacts/budget-transfer-20260905.zip`. No engine changes or
   strength games; debug/release tests, fmt, Clippy and 156 tooling tests pass.
8. **4.11.8 Datagen label audit -- DONE, RAR-M22, 2026-09-06.** Audited the
   hash-verified source-game PGNs of `hce-v2` and `hce-v3-tb` (the latter is
   derived from `hce-v3`) with 3–6-man Syzygy and the existing 4.10.8 tool.
   Both corpora used 8,000 nodes; no budget comparison exists. First clean wins
   not won: `hce-v2` **26,316/134,948 = 19.50%**, **4.39% of 600,000 games**;
   `hce-v3` source **54,186/266,490 = 20.33%**, **8.99% of 602,619 games**.
   The latter reaches clean wins nearly twice as often, so the all-game shares
   are not directly a technique comparison. `hce-v3-tb` separately corrected
   125,643 ≤6-man CSV rows; this game-level audit neither calls 8.99% its
   remaining row-error rate nor proves whole-game correction. Full method,
   families, raw evidence and disposition:
   `analysis/datagen_label_audit_2026-09-06.md`. 4.13.1 owns row-level
   game-to-extraction lineage and separate relabel/whole-game-adjudication
   choices. No games, HCE change or strength claim.
9. **4.11.9 Mate-drive blast radius -- DONE, RAR-M23, 2026-09-06.** Re-read
   the paired 4.9a.4 pre/post reports over their identical 19-family,
   60,000-node, seed-6200600 cohort and verified every FEN/index/theory field
   before differencing. The promotion closure changes six families: direct
   KBB-K **78/100 -> 100/100** and KBN-K **19/98 -> 95/98**; promotion-reached
   KPP-K **75/98 -> 75/98**, KBP-K **90/94 -> 92/94**, KBP-KB
   **18/26 -> 17/26**, and KBP-KN **45/57 -> 44/57**. KBP-KB and KBP-KN each
   therefore carry a **net one-conversion causal debt**: same cohort, shipped
   mechanism, family result worse. Their offsetting paired gains do not erase
   the debt; KBP-K's +2 net result and KPP-K's zero net result remain closure
   regression guards, not debt. The source reports predate 4.10's material-shed
   repair, so this matrix establishes the historical mechanism and debt
   ownership, **not** current conversion rates or floors. Full matrix, hashes
   and byte-preserved reports: `analysis/mate_drive_promotion_closure_2026-09-06.md`.
   The required family-template closure rule is recorded under 4.12 below;
   KBP-KB and KBP-KN are owned by 4.12.7 and 4.12.9. No engine change or games.
10. **4.11.10 Corrections in place -- DONE, RAR-M24, 2026-09-06.** Reran the
    preserved RAR-E08 baseline/head and RAR-E08/E12-candidate pairs with the
    repaired v2 instrument and identical per-position cohorts. RAR-E08's v1
    aggregate **83.24% -> 83.45%** is superseded by **1255/1372 = 0.9147 ->
    1254/1372 = 0.9140**. Its KQ-KP focus result survives exactly:
    **390/396 -> 375/396, -3.79 pp**, so it remains a historical causal debt
    for 4.12.13, qualified by RAR-M21's closure at 200k/600k. RAR-E12's v1
    **0.8345 -> 0.8477** is superseded by **1254/1372 = 0.9140 ->
    1278/1372 = 0.9315**, +24; KQ-KP DTZ progress improves but conversion is
    **96/98 -> 94/98**, so “debt repaid” was overbroad. RAR-E11 remains
    superseded in full: reference **1361/1372 = 0.9920**, current head
    **1276/1372 = 0.9300**, reference worse in no family. Originals remain in
    `EXPERIMENTS.md` as superseded history. Evidence and byte-preserved inputs:
    `analysis/conversion_claims_correction_2026-09-06.md`.
11. **4.11.11 A panic must be reported where the harness keeps it -- DONE.**
    Rarog 2.3.2 lost one game in ~5,200 to `EngineCrash` in the 2026-09-04
    rating tournament (Colosseum incident `20260904-230039-002`: black to move,
    first search of the game, ~20 ms in at depth 10, then stdout EOF). **The
    cause cannot be established, and that is the finding.** The incident
    retains the full UCI transcript, the clocks and the position, and retains
    no exit status and no stderr -- and neither does any other incident in that
    run, including five for a different engine, so the silence is a property of
    the pipeline rather than evidence about the death.

    Ruled out by measurement rather than by reading: **Threads=1**, so the
    worker pool is empty and no SMP race is available; `SyzygyPath` was never
    set, so the tablebase FFI never ran; the TT allocates through
    `try_reserve_exact` with a 1 MiB fallback, so allocation failure is handled;
    400 replays of the incident's exact UCI session produced no failure.

    What was fixed is Rarog's half of the instrument gap. Release sets
    `panic = "abort"` and the default hook writes only to stderr;
    `src/crash_report.rs` now mirrors every panic to **stdout** as one
    `info string` line and then chains to the previous hook, so stderr and
    `RUST_BACKTRACE` are unchanged. Proved live rather than assumed: a real
    `catch_unwind` panic produces exactly one report naming thread, location and
    message; the string is present in the built binary and absent from released
    2.3.2; and `bench 13` reproduces **6,901,489 / EBF 2.458** exactly.

    **Scope, stated honestly.** This makes a PANIC diagnosable. It cannot see a
    death that never reaches the Rust runtime -- an access violation, an illegal
    instruction, an external kill. Telling those two classes apart is the whole
    value: a report on stdout names the line, and no report narrows the next
    search. A structured-exception handler would close the rest and is a new FFI
    site against the frozen unsafe floor (principle #8), so it stays an option
    to take deliberately, exactly like 4.8a's CPUID guard. Two gaps are
    Colosseum's, not Rarog's, and are recorded here because it is the same
    maintainer: its stderr tail reaches no incident report at all, and its
    100 ms reap window never yielded an exit status in six crashes.

    **Disposition:** investigation of that historical tournament crash is
    closed; it is not a standing request for forensic work or new exception
    handlers. Reopen only on a new actionable reproducer/evidence or an
    explicit maintainer request. This does not waive ordinary lifecycle tests
    or 4.11b.3's independently reproduced parser/counter defects; no connection
    between those defects and the tournament incident has been established.
12. **4.11.12 Occurrence re-measured over 36,400 rated games -- DONE, and
    4.12 renumbered to v2.** Discharges 4.11.6's retry trigger.
    `tools/diag/endgame_board_occurrence.py` classifies every position of every
    mainline; `endgame_ranking.py` now takes board occurrence as an ARTIFACT
    (`--board-occurrence`), so the constants it used to carry are a fallback and
    `endgame_ranking_v1.json` still reproduces exactly.

    **Calibrated before it was applied.** Run first over RAR-M15's own retained
    corpus, 13 of 20 families agree to four decimals -- including KBN-K's
    published count of exactly 11 games, and both aggregate figures (52.69%
    against 52.7% reaching six men, 60.87% against 60.9% reaching seven). The
    seven differences were each then measured rather than argued: RAR-M15
    capped positions at **six men** and counted a plural strong side, which
    reproduces its KRPKR (0.1004), KRPKB (0.0123) and KBPsK (0.0192) figures
    exactly. The gate FOUND the KBPsK difference; it was not anticipated.

    **Two of RAR-M15's three zeros were never zero.** KRPPKRP occurs in 5.40% of
    Rarog's games, KQKR in 0.63%, KQKRPs in 0.42% -- and both of the first two
    can be exhibited from RAR-M15's OWN games. The rule-of-three floor was
    applied to the wrong problem: these were not thin samples but positions that
    were there and were not looked at. `analysis/endgame_conversion_audit`'s
    "KQ-KR's -25.0 pp is the largest gap and worth nothing" is corrected in
    place; KQKR moves from 4.12.15 to **4.12.10**.

    **The fourth most common family in the set cannot be measured at all.**
    KRPPKRP is seven men, so the local tables adjudicate none of it. It stays
    last because its evidence cannot be produced, but 4.12.21 now records a
    TOOLING GAP rather than a rare ending.

    **Rarog reaches rook-against-a-lone-minor at ~1.6x the pool rate** (KR-KN
    0.51% against 0.32%, KR-KB 0.46% against 0.32%) -- the two families 4.11.4
    measured at 100% and 99.6% drawn-share overclaim. Stated as a HYPOTHESIS:
    an engine that prices a dead draw at +346 may be steering into it, which
    also makes engine-scope occurrence endogenous. 4.12.4 owns testing it.

    **Ranks 3-8 are a band, not an order.** The whole-pool derivation
    (`--occurrence-scope all`) orders them differently
    while agreeing on ranks 1-2 and on all of 16-20. No leaf may be argued on
    its position inside that band. Full derivation:
    `analysis/endgame_occurrence_tournament_2026-09-05.md`.

### 4.11b Board correctness and HCE throughput, before endgame development

**Integrated 2026-09-05 after reconciling the a0aeb68 roadmap.** Finish the
remaining 4.11 leaves first, then execute this section before 4.12. Those
4.11 leaves and board leaves 4.11b.1–4.11b.16 are now complete: 4.11b.9 by
acceptance, 4.11b.10–4.11b.12, 4.11b.14 and 4.11b.15 by justified `NO_CHANGE`,
4.11b.13 by contract tightening, 4.11b.16 by a banked **+1.421%** pooled-PGO
throughput qualification, 4.11b.17 by RAR-E15's accepted gate and 4.11b.18 by
RAR-M42's refresh. **4.11b.19 was inserted on 2026-09-09 (RAR-M44)** after a
post-closure review found that every generator returned its 520-byte `MoveList`
through a `memcpy` that Basilisk's harness and search both avoid; its (a) and
(b) are implemented, its NPS run is owed, and it precedes 4.12.1. The insertion preserves 4.11.12's registered v2 endgame order until
changed evidence warrants a rerank. Repairs at 4.11b.3/4.11b.5 are implemented;
the remaining optimizations and cluster qualification are still open. NNUE-only work belongs to 5.2, 5.3, 5.6 and 6.4.

Evidence: RAR-M20, [board audit](analysis/board_audit_2026-09-05.md), and
[measurement recipe](analysis/board_benchmark_recipe_2026-09-05.md).
Measured Rarog was ca03a46, Basilisk d734766, Reckless 91b56c2 plus the archived
benchmark-only adapter. Rarog subsequently advanced through a0aeb68: its board,
benchmark and Cargo manifest were unchanged, but endgame ranking, occurrence
evidence and panic reporting changed. Refresh source and roadmap again before
starting implementation; a0aeb68 is the audited snapshot, not a frozen future baseline.

#### What the measurements change

On the Ryzen 9 5950X, native optimized non-PGO builds, three cyclic rounds of
the same five-root workload gave these median throughputs (million operations/s):

| Workload | Rarog | Basilisk | Reckless | Basilisk speedup over Rarog |
|---|---:|---:|---:|---:|
| Legal moves | 447.131 | 642.646 | 339.844 | 43.7% |
| Legal captures | 98.204 | 120.138 | 61.597 | 22.3% |
| Generation + make/unmake | 42.521 | 55.031 | 23.494 | 29.4% |
| Start-position perft(4) | 273.741 | 382.726 | 177.944 | 39.8% |
| Two-ply simulation | 351.809 | 513.537 | 246.626 | 46.0% |

These are workload speedups, NOT whole-search NPS or Elo gains. Total host busy
time was 6.25-9.17%, including the benchmark; this was an active desktop, not an
idle laboratory. Some cells varied materially: Rarog simulation round medians
spanned 8.0%, and Reckless make/unmake spanned 23.7%. Every Basilisk round still
beat every Rarog round in all five columns. Use the direction and broad scale
for prioritization; do not use these data to accept a small candidate.

Consequently, after correctness, **profile and prioritize legal generation and
the complete node's board work**, not only a cached king square or undo-record
packing. The old July perft report's 6.4% gap is a different historical workload
and does not override this measurement. Reckless is an architecture/correctness
reference, not a microbenchmark speed target: its native board maintains
threats, pins, checking squares and repetition state; its move list also stores
a score per move. The adapter keeps that work and removes only NNUE arithmetic
via NullBoardObserver. That disables NNUE arithmetic, so the remaining gap
does not isolate NNUE enablement cost. Preserve the raw baseline for 5.2.1,
price event/scaffold costs at 5.2.5/5.3.4, and isolate actual-network updates
and inference at 6.4.3. Do not delete useful state to win this benchmark.

The historical native-value SEE row measured 46.676 / 58.814 / 39.722 million
captures/s for Rarog / Basilisk / Reckless and was not comparable: vectors and
contracts differed. **RAR-M29 / 4.11b.6 SUPERSEDES that row for ranking.** With
all three adapters on 100/300/300/500/900/20000 and the same ten verdicts,
current medians are **44.923 / 58.335 / 40.823 million captures/s**. Basilisk
leads Rarog by 29.86%; Rarog leads Reckless by 10.04%, but Rarog's rounds span
12.20% and the third falls behind Reckless, so treat magnitudes as directional.
The old-to-new change does not measure injection overhead: Rarog's corrected
4.11b.5 kernel also changed. Source inspection corrects Rarog's historical
SEE vector to **100/320/330/500/900/20000**, not the previously recorded 32000
king sentinel. Actual value fitting remains after final HCE in 4.15.3-4.15.4.

#### Execution contract

- Work one leaf at a time and record a result or a justified no-change
  disposition. "Conditional" means measure its prerequisite and close it if
  absent; it does not authorize speculative implementation.
- Separate correctness repairs, behavior-preserving optimizations and tuning.
  Preserve a correctness-only candidate so speed refactors can be compared
  against a board with the same chess semantics. Never tune away a failing
  correctness test or restore a known defect merely to recover an old number.
- A behavior-preserving candidate must retain its immediate baseline's exact
  search fingerprint AND pass targeted board-state/legal-move tests. The
  section-entry fingerprint may change at the SEE repair; explain and record
  that change rather than forcing 6,901,489 forever.
- No HCE coefficient refit, speculative search-policy change, NNUE accumulator,
  full threat-feature maintenance, or Chess960 implementation is hidden here.
  Preserve a seam for future work, not its current runtime cost.
- Follow AGENTS.md: debug and release, fmt, all-feature/all-target Clippy,
  direct exit statuses, exact feature rebuild before measurement, native
  feature/ISA tests for touched backends, source/binary hashes, separate engine
  and tooling/doc commits. Never measure a texel/all-features engine.
- Internal optimization leaves do not each earn a game gate. One
  dependency-complete playing change is qualified at 4.11b.17. Bench counts
  and cross-engine microbenchmarks never substitute for that verdict.

**Active workflow register.** Evidence and detailed contracts are in the
numbered leaves below and the linked board analyses. A state/class does not
lift dependency order. GUIDE maps each class to the current GPT and Claude
recommendations without coupling this roadmap to model generations.

| Leaf | Workflow state | Class | Question or goal | Evidence / dependencies | Ready-to-advance condition and deciding gate |
|---|---|---|---|---|---|
| 4.11b.8 | CLOSED (candidate withdrawn) | R3 | Local pin gains established; useful whole-search value unresolved | RAR-M31; restoration c44608a | Prior x-ray path restored and qualified; independent oracle retained; retry belongs to 4.11b.10 only if justified |
| 4.11b.9 | CLOSED (ACCEPTED) | I2 | Fused ordinary quiet relocation integrated | RAR-M33 `5c439da`; RAR-M32 VOID; test `8a73cfd` | Parity exact, isolated +16.3/+17.3/+19.3%, full-search +0.876% with 95% [+0.050%, +2.055%] excluding zero on a verified-idle host. Behaviour-neutral, no game gate owed |
| 4.11b.10 | CLOSED (`NO_CHANGE`) | R3 | No shareable duplication remains; sharing into SEE is forbidden | RAR-M34; `analysis/pin_check_sharing_2026-09-08.md` | Closed on structure, not cost: producers share no subexpression and the 4.11b.5 contract bars SEE reuse. Reopen only for a genuinely new consumer |
| 4.11b.11 | CLOSED (`NO_CHANGE`) | I2 | Incremental attacker maintenance built, verified correct, measured slower | RAR-M35; `analysis/see_kernel_2026-09-08.md` | Verdicts were exact but `threshold SEE only` fell 1–3%; stage-1 screen rejected and stage 2 never ran. Future SEE work targets the king-legality test |
| 4.11b.12 | CLOSED (`NO_CHANGE`) | R2 | King lookup is 0.502%; a 0.25% ceiling is below instrument resolution | RAR-M37; RAR-M36 refreshed profile | Closed without a floor, which would have been post-hoc: the best possible version cannot be distinguished from zero by any budget used here |
| 4.11b.13 | CLOSED (tightened) | R2 | Search headroom reserved before the hot path; canonical-move contract pinned | RAR-M38 `f70ac19`; `analysis/history_contracts_2026-09-08.md` | Behaviour-neutral at 7,601,220 / EBF 2.474; five contract tests, clone test proven to fail on regression. No speed claim |
| 4.11b.14 | CLOSED (`NO_CHANGE`) | R3 | No board region above 6.7%; all three alternatives lose on measured grounds | RAR-M39; RAR-M36 profile; `analysis/representation_2026-09-08.md` | Gate for opening an implementation not met; footprint pinned by const assertions naming this leaf |
| 4.11b.15 | CLOSED (`NO_CHANGE`) | R3 | All four policies kept with independent dispositions; identity separation pinned | RAR-M40 `df94b7d`; `analysis/draw_policy_2026-09-08.md` | Engine untouched at 7,601,220 / EBF 2.474; no playing change proposed, no bundle rescued |
| 4.11b.16 | CLOSED (QUALIFIED) | V | Correctness matrix passed; +1.421% [+0.953%, +1.764%] pooled-PGO throughput banked | RAR-M41; `analysis/cluster_qualification_2026-09-08.md` | Null pair unbiased at +0.222% [-0.130%, +0.630%]; behaviour identical throughout; no Elo claimed |
| 4.11b.17 | CLOSED (ACCEPTED) | V | Integrated cluster gated and accepted | RAR-E15; `analysis/playing_gate_2026-09-08.md` | H1 at 1,950 games under a symmetric `[-5,5]` registered before games: +12.12 +/- 10.17 Elo, +18.40 nElo, LLR 2.96. Magnitude imprecise; no subcomponent credited |
| 4.11b.18 | CLOSED (refreshed) | V | Affected evidence refreshed and versioned; 4.12 order verified unchanged | RAR-M42; `analysis/endgame_refresh_2026-09-09.md` | Layer-1 theory identical on all 19 families, floors PASS both arms, order rederived and reproduces registered v2 exactly. One non-blocking KRP-KB report owned by 4.12.6 |
| 4.11b.19 | CLOSED (banked + `NO_CHANGE`) | I1 | (a)+(b) banked +2.48% NPS; (c) rejected in search at -0.55% and reverted; (d) re-measured and RAR-M43 superseded. Remove the 520-byte move-list return copy from bench and search; bounded constant-factor screen; corrected comparison record | RAR-M44; `analysis/movelist_delivery_2026-09-09.md`; probe `tools/results/board-copy-probe-20260909/` | (a) `55e228a` and (b) `021dc98` landed on the exact fingerprint 7,601,220 / EBF 2.474 (magic and PEXT) with zero 520-byte `memcpy` sites left in the fat-LTO binary, four before; the registered pooled-PGO run measured **+2.48% [+2.29%, +2.65%]** against a +0.5% floor, null pair -0.21%, so (b) is **BANKED**; (c) promoted two of four candidates on the bench (+12.7% legal moves) and its bundled run then measured **-0.55% [-0.76%, -0.30%]** whole-search NPS, below the floor and negative, so both commits were **REVERTED** at `39542b7`; (c) conditional on the parity bench; (d) re-measures the four arms and supersedes RAR-M43's gap table |

1. **4.11b.1 Freeze the audit and comparison -- DONE.** RAR-M20 and its committed evidence preserve the
   three-engine results, limitations, raw output, binary/source hashes, exact
   build commands and full Reckless adapter patch. Original Rarog audit:
   63 selected debug tests and 64 release tests passed; 1,975 independent
   python-chess positions, 48,462 hinted make/unmakes and 107,648 magic-slider
   occupancies passed. None of that disproves the focused failures below.

2. **4.11b.2 Strengthen the instrument and regression corpus -- DONE
   (RAR-M25, 2026-09-06).** Kept
   cross-engine-board-v1 immutable for historical comparison. Add a versioned
   coverage profile containing real single/double checks, evasions, legal and
   illegal EP, promotions/underpromotions, both castlings, sparse endgames and
   long histories. The v1 position named "in-check" is NOT in check; none of
   its five roots has a promotion or EP move, and Kiwipete supplies eight of
   its ten captures. Assert these categories mechanically for the new corpus.
   Outside timing, compare sorted legal/capture move identities against an
   independent implementation, exact perft/divides, canonical FEN, every
   incremental key and full make/unmake restoration. Add independent
   coordinate-ray tests for every slider occupancy on magic AND PEXT.
   Exercise both normal and hinted make, both staged-generation paths, null
   roundtrips, randomized unwind and clone/history growth.
   Preserve the five combined v1 workloads, but add separately named isolated
   primitives with precomputed inputs to distinguish generation from mutation
   and capture generation from SEE. Prove allocation freedom and live output
   barriers, including compiler-specific barriers; Basilisk's non-GNU/Clang
   fallback barriers are no-ops. Record per-sample data and a full build/ISA
   manifest. Do not relabel the old tests/board_performance.rs as v1: it uses
   the legacy corpus/estimator. Done when preflight fails on deliberately
   wrong moves, work counts or state, and the corrected corpus passes.
   `board-v2` now supplies ten externally generated profiles with all required
   categories. `tests/board_v2.rs` compares canonical FEN, sorted legal and
   capture identities, perft/divides and incremental-state restoration; its
   structural negative controls reject a wrong EP move, wrong perft total and
   a mutated board. It also covers normal/hinted make, staged paths, null
   roundtrips, a 600-ply unwind and independent cloning. Every relevant
   slider occupancy agrees with coordinate rays on magic and PEXT. The separate
   `rarog-board-v2` benchmark precomputes its inputs and reports seven raw
   samples for legal generation, capture generation, staged generation,
   make/unmake and threshold SEE. Its warmed-primitives allocation guard
   reports zero allocations; `black_box` plus a printed checksum supplies the
   portable live-output barrier. RAR-M25's archived run and manifest are at
   `analysis/artifacts/board-v2-20260906/`; recipe and limits are in
   `analysis/board_v2_instrument_2026-09-06.md`. These are instrument and
   correctness results, not an HCE-speed or strength comparison.

3. **4.11b.3 Repair parser and counter boundaries -- DONE (RAR-M26,
   2026-09-06).** Move::from_uci("aé1")
   currently panics because a four-byte string is sliced through a UTF-8
   character. Validate the ASCII move grammar before indexing; retain the
   documented controlled UCI error policy rather than silently accepting bad
   tokens. A FEN ending "b - - 0 65535" can be accepted and then overflow
   fullmove on a black move: debug panics, release wraps to zero. Widen the
   counter or explicitly specify checked/saturating behavior across parsing,
   real/null make and undo. Test the accepted maximum and the first rejected
   value, for both colors and both profiles. The new crash-report hook reports
   a panic; it does not repair either cause. Done when malformed moves take the
   intended error path and all accepted counter states remain well-defined.
   `Move::from_uci` now rejects non-ASCII input before byte indexing, so the
   documented fatal `position ... moves <bad-token>` path reports a controlled
   UCI error instead of aborting at a UTF-8 boundary. Fullmove remains a
   deliberately bounded `u16`: FEN zero still normalizes to one, 65,535 is
   accepted, and 65,536 is rejected. Real and null black moves saturate at the
   accepted maximum; white moves preserve it, and undo restores the original
   state in all four cases. The targeted tests first failed on the old panic
   and overflow, then passed in debug and release. An exact default-feature
   rebuild reproduced `bench 13` at **6,901,489 / EBF 2.458**. This confirms
   the ordinary benchmark tree is unchanged; the targeted boundary tests are
   the evidence for the corrected malformed/high-counter behavior.

4. **4.11b.4 Define SEE contracts and independent fixtures -- DONE
   (RAR-M27, 2026-09-06).** Inventory every
   full-SEE, threshold-SEE and quiet-aware caller in search/move ordering.
   Specify captures, ordinary quiets, capture/quiet promotions, EP and castling
   separately: distinguish a true exchange verdict from a deliberate
   "always admit this move" policy. Do not demand that a heuristic SEE solve
   a full tactical search. Use a small independent legal same-square
   capture-tree oracle and explicit expected arithmetic, not see_ge == see
   as the sole oracle. Cover king recaptures, defended king destinations,
   pins released/created during exchanges, x-rays and promotion recaptures.
   The quiet promotion a7a8q in
   "7r/P7/7k/8/8/8/8/K7 w - - 0 1" returns +800/true although ...Rxa8 gives
   -100. This is an existing explicit shortcut: decide its caller contract
   before changing it. Done when every special-move policy has a named
   consumer and an independent test or a documented heuristic disposition.
   Completed inventory: ten threshold calls (two diagnostic-only) and one
   full-SEE call, including downstream ordering, pruning, LMR and history
   consumers. [Contract and evidence](analysis/see_contract_2026-09-06.md)
   defines each move class and the optional-stop material oracle's limits.
   Eighteen python-chess fixtures supply independent truth; two Rust contract
   tests and six existing pin tests pass in both profiles. Three explicitly
   ignored acceptance tests fail in both profiles and are owned by 4.11b.5.
   Five Python tests prove arithmetic rejection, legality, all promotion replies,
   color symmetry and actual pin creation/release. No production code changed.
   Quiet-promotion immediate-gain policy is retained: production ordering uses
   a separate promotion tier and currently has no SEE caller for that class.

5. **4.11b.5 Repair the confirmed SEE exchange defects -- DONE
   (RAR-M28, 2026-09-07).** Entry defects and acceptance contract: on
   "7k/8/2p5/3pK3/8/8/3R4/8 w - - 0 1", d2d5 gives
   Rxd5 cxd5 Kxd5 = 100 - 500 + 100 = -300. Entry full see returned -400,
   and see_ge(move, 0) incorrectly returned true in debug and release.
   board.rs's entry exchange loops broke on an opposing king in next_attackers
   even when the selected recapturer was a PAWN. Implement the correct
   selected-king legality/parity logic; prove behavior around thresholds
   -301, -300, -299 and 0. Reference Reckless board/see.rs's explicit
   attacker == King condition and Stockfish Position::see_ge's king branch;
   do not copy their value scales or their special-move exemptions.
   Keep the production value vector fixed. Extend the same narrowly stated
   contract to any further demonstrated exchange defects from 4.11b.4.
   RAR-M27 adds two mandatory independent cases: in
   `2k5/2n5/2B5/3p4/8/8/8/2R1K3 w - - 0 1`, c6d5 creates a pin on
   Nc7, so +100 is correct (entry -230/false). In
   `7k/8/8/8/8/7K/pR6/1r6 w - - 0 1`, b2b1 permits a2b1q:
   500-500-800=-800 (entry 0/true). Repair evolving pin geometry and
   recapture-promotion accounting in BOTH kernels, including the promoted
   occupant's value on further recapture. The external oracle explores all
   promotion choices; never accept its first generated reply as optimal.
   Remove the three named ignores in `tests/see_contract.rs` once repaired;
   retain explicit quiet/promotion/castling shortcut policies. Do not require
   general tactical minimax or erase a failing fixture as an LVA approximation
   without a concrete branch-choice explanation. No new SEE scale tuning.
   Record the search-fingerprint change and affected pruning/ordering sites.
   Done when independent fixtures pass; playing qualification remains open.

   **Completion:** engine/test commit `fce0b44` replaces stale pin pairs with
   legal recapturer selection under evolving occupancy, terminates only on
   a legal selected king, and accounts for recapture promotion gain/new victim
   value. Full SEE folds the legal LVA sequence; threshold SEE uses an explicit
   positive-limit recurrence with equality preserved. Production values and
   special-move shortcut policies are unchanged. All three ignores are removed.
   **41 independent fixtures** pass, including a promoted piece captured again,
   a later-created pin and color mirrors; **1,802 legal captures** pass parity.
   Complete suites: **268 debug / 269 release**, zero failures/ignores; six
   Python tests, fmt and all-feature/all-target Clippy pass. After the exact
   no-feature rebuild, valid UCI `bench 13` is **7,601,220 / EBF 2.474**,
   previously 6,901,489 / 2.458 (**+10.14% nodes**, not Elo or NPS evidence).
   This is the development baseline for subsequent neutral work; 4.11b.17
   still owns playing qualification. All ordering/pruning/LMR/history consumers
   of SEE can receive changed answers; no per-consumer attribution is claimed.
   A process-argument bench invocation was a detected no-op and is explicitly
   invalidated in the evidence. Recipe, raw logs, identities and exact engine
   diff: `analysis/see_repair_2026-09-06.md` and
   `analysis/artifacts/see-repair-20260906/`.

6. **4.11b.6 Separate SEE value injection from value tuning -- DONE
   (RAR-M29, 2026-09-07).** Introduce a
   board/search-owned value interface that preserves the existing production
   vector exactly. The benchmark must be able to pass the v1 contract vector
   without changing eval.rs or the playing defaults. Static/generic
   specialization is acceptable; avoid an unmeasured dynamic-dispatch cost in
   the hot path. Test injected and production values against independent
   fixtures, prove a deliberately absurd injected value changes a known
   verdict THROUGH THE BENCH HARNESS, and prove default-injection identity
   against the corrected 4.11b.5 baseline. Then restore the comparable SEE
   column in a separate tooling commit. Coordinate the normalized peer
   adapters and check answers, not just ten function calls.
   This pulls the measurement prerequisite forward from 4.15.5; tuning and
   ownership of fitted values still remain in 4.15.3-4.15.4. No premature fit.

   **Completion:** engine/test `46f1af2` adds the board-owned `SeeValues`
   interface. Production remains 100/320/330/500/900/20000 and has no runtime
   option or dynamic dispatch; explicit production injection agrees on all 41
   external fixtures. The normalized vector also agrees with independently
   scored values on all 41. Complete suites pass 270 debug / 271 release with
   zero failures/ignores; seven Python oracle checks, fmt and Clippy pass.
   Exact production `bench 13` remains **7,601,220 / EBF 2.474**.

   The benchmark wire is live: a rook value of 1 flips the independent defended-
   pawn probe false -> true. All peer preflights report the same vector and ten
   move/verdict answers. Three cyclic timing rounds produce Rarog/Basilisk/
   Reckless medians **44.923/58.335/40.823 M captures/s**. Every timed host-load
   check passes; Rarog's 12.20% round span prevents a precise small-gap claim.
   The historical native row is retained but superseded for ranking. RAR-M19,
   RAR-M27 and RAR-M28's 32000 SEE-king statement is corrected to the actual
   pre-existing 20000 sentinel. No values were fitted and no games run.
   Evidence and exact peer adapters:
   `analysis/see_value_injection_2026-09-07.md` and
   `analysis/artifacts/see-normalized-20260907/`.

7. **4.11b.7 Profile board work in actual HCE search -- DONE (RAR-M30,
   2026-09-07).** Freeze representative
   opening, middlegame, check-heavy, promotion and sparse-endgame cohorts at
   realistic search budgets. Measure generation, legality, check queries,
   SEE, make/unmake and history allocation as shares of full-search time.
   Distinguish plain perft make from search's make_move_with_check, and
   standalone captures from the shared-pin staged picker. Inspect emitted
   code if a native sampler is unavailable; label that evidence appropriately.
   Count hot caller frequencies and allocation events using observational
   instrumentation proven live, with instrumentation-off identity. Set the
   following leaves' priorities from actual search cost and a written time
   budget; a 40% microbench gain in a 2% search component is not 40% NPS.

   The frozen 20-root suite used four opening, middlegame, check-heavy,
   promotion and sparse-endgame positions at 600,000 nodes. Sixty
   instrumentation-off comparisons matched depth, seldepth, reported nodes,
   score type/value and best move; PV and ponder move were not compared. Across
   161,989 native samples, generation/legality is **6.751%**, make/unmake
   **7.143%**, checks **5.177%** and SEE **5.304%** of process time. Add/remove
   relocation is a 2.998% overlapping subregion; pin computation 1.003%, check-info 0.912%,
   and king lookup 0.544%. Counters over 30,604,224 nodes show 89.02% of real
   makes use the check-hinted path, threshold SEE is 92.77% of SEE calls, and
   25,718,154 history pushes cause **zero** growth events. Therefore perft make
   and standalone captures are rejected as search proxies, history allocation
   has no speed budget in this state, and 4.11b.8 remains first. No games or
   strength claim. Evidence: `analysis/board_search_profile_2026-09-07.md` and
   `analysis/artifacts/board-search-profile-20260907/summary.json`.

8. **4.11b.8 Optimize legal generation and move-list delivery — CLOSED;
   candidate withdrawn (RAR-M31, 2026-09-07).** This was the
   first speed candidate because the measured Basilisk gap is 43.7% in legal
   generation and 46.0% in two-ply simulation. Inspect compile-time
   color/mode specialization, setwise pawns, pin discovery, king-target
   safety, and repeated large move-list returns/initialization. Check emitted
   code before claiming that Rust actually copies a returned list.
   Basilisk board.cpp::gen_legal and Reckless board/movegen.rs show different
   ways of combining targets and pins; Reckless's scored MoveEntry output
   does more stores than Rarog's compact MoveList. Preserve required output,
   all underpromotions and exact capture/quiet partition. Preserve existing
   order when practical; if order changes, classify and measure the search
   behavior change explicitly. Done with independent move-set/perft parity,
   controlled within-Rarog A/B and a full-search NPS result.

   **Research card (historical scope; retry requires fresh registration).** The decision needed was which,
   if any, named local contract accounts for enough of M30's **6.751%**
   full-search region to justify a candidate. Known evidence is RAR-M20's
   directional 43.7% peer-workload gap, RAR-M25's live separated instrument
   and RAR-M30's full-search share/caller counts. Candidate explanations are
   compile-time color/mode specialization, setwise pawn/pin/king-target work,
   or avoidable list initialization/delivery; credible alternatives are that
   LLVM already removes the apparent copies, the peer gap belongs to different
   representation/compiler work, or a faster local primitive does not improve
   the deployed search. Generation order interacts with picker ordering,
   pruning and tree shape; pin state interacts with 4.11b.10.

   Cheapest discrimination: inspect optimized emitted code, then use M25's
   isolated primitives and a minimal switchable within-Rarog variant before a
   pooled full-search run. The research owner must preregister the exact
   expected movement and confidence before seeing that result; this migration
   invents neither a magnitude nor an Elo prior. Falsify a cause if the emitted
   operation/copy is absent or its isolated removal is flat; stop the leaf
   `NO_CHANGE` if no candidate clears a predeclared whole-search practical
   floor with exact move-set/perft/order semantics. Promote only one explicit
   mechanism whose producer/output/consumers, invariants, test corpus, baseline,
   performance floor and rejection rule are frozen.

   **Disposition: withdraw the candidate; no further standalone campaign.** `2ea279f`
   predates the `b592b40` research card above. Its measurements had no
   predeclared practical whole-search floor, so they cannot retrospectively
   satisfy that card. Research disposition now restores the previous x-ray
   algorithm in `c44608a`, retaining the independent pin oracle. This is a
   prioritization decision under insufficient whole-search evidence, not proof
   of zero benefit, regression or incorrectness. Local gains are established;
   another campaign would target small whole-search value just before 4.11b.10
   revisits producer lifetime/consumers. No retrospective threshold or new
   timing result is used to reject it. 4.11b.9 is unblocked. A retry at 4.11b.10
   needs remaining-cost evidence and a prospective production-build floor,
   uncertainty rule and fixed budget. RAR-M31 measured `2ea279f`, not the
   later LazyMargin/cache/search revisions. Restoration from `b90232b` keeps
   those revisions: 274 debug / 275 release tests pass, fmt and Clippy pass,
   fresh no-feature builds both reproduce **7,601,220 / EBF 2.474**, and 20
   profile roots match the standard harness identity fields. No new PEXT,
   full-PV/ponder, NPS or playing result is claimed; detailed evidence is linked below.

   The withdrawn candidate replaced four occupied-board x-ray lookups with two
   empty-board slider lookups plus an all-occupancy BETWEEN scan. A candidate
   pin requires exactly one intervening piece, of our color; enemy blockers
   break the ray too. No pin cache or new board state is introduced. Existing
   staged sharing, legal move order and all promotion choices are preserved.
   Independent mailbox ray walks check both kings; external board-v2 move-set,
   partition and perft oracles pass on magic and PEXT. Debug/release suites,
   fmt and Clippy pass. Both backends keep **7,601,220 / EBF 2.474**; 480 paired
   root comparisons match depth, seldepth, nodes, score, full PV and best/ponder.

   Three alternating board-v2 rounds show median legal/capture/staged throughput
   gains **8.54% / 11.43% / 7.41%**. These supported the original retention
   decision, not an established whole-search speedup: twelve alternating pairs per backend
   give generic **+0.57%** (bootstrap 95% interval **−0.51% to +1.00%**) and
   PEXT **+1.45%** (**−1.57% to +4.26%**). PEXT had more background load.
   These are non-PGO measurements; playing qualification remains 4.11b.17.
   MoveList already avoids full-array initialization and pawns already use
   setwise generation. A tuple-construction trial failed to remove the observed
   520-byte staged-list copy in library assembly and was dropped. Broader
   specialization/delivery redesign was not bundled into this candidate;
   4.11b.14 can revisit delivery after checking actual production callers.
   Evidence and reproduction: `analysis/movegen_2026-09-07.md` and
   `analysis/artifacts/movegen-20260907/`.

9. **4.11b.9 Measure fused ordinary-piece relocation -- DONE, ACCEPTED
   (RAR-M33, 2026-09-07, `5c439da`).** Price a dedicated
   move_piece(from, to) path against remove_piece + add_piece. Update the
   moving piece and both occupancies with a combined from/to mask, mailbox
   endpoints and the required keys; keep captures, EP, promotion and castling
   correct. Reference Basilisk board.cpp::move_piece, not a broad C++ port.
   Inspect whether LLVM already combines the current operations, and verify
   every redundant field after make AND unmake. Retain only a measured
   benefit with the same chess semantics.

   **Implementation handoff.** For ordinary non-capture/non-special relocation,
   the producer is `make_move`, stored state is mailbox, piece/color occupancy,
   piece/full/pawn/minor keys and PST bookkeeping, and consumers are legality,
   evaluation, repetition/TT identity and undo. Add no new representation or
   playing policy. Preserve capture, EP, promotion, castling and null paths;
   prove ordinary make/unmake reconstruction and fingerprint identity. Emitted
   code plus M25 primitive timing is the cheapest screen; controlled full-search
   performance is the deciding gate. No maintainer game job is owed if behavior
   remains identical.

   **Disposition: ACCEPTED.** `Board::move_piece` now serves the
   `flags == QUIET` branch of `make_move_inner` and `unmake_move`, updating both
   mailbox endpoints, the piece and colour occupancies, `all_occ` and the
   applicable pawn/minor/non-pawn keys with one from/to mask and one paired key.
   Captures, double pushes, en passant, promotions, castling and null moves keep
   their existing paths and the position hash stays caller-owned.

   Behaviour is neutral: both no-feature builds reproduce **7,601,220 / EBF
   2.474** and all **640** paired root answers match on depth, seldepth, nodes,
   score, best move, full PV and ponder. Under the RAR-M33 contract frozen
   before the run, the isolated `make/unmake only` primitive gained
   **+16.33/+17.30/+19.32%** across three alternating rounds, and 32 alternating
   1,200,000-node pairs measured a **+0.876%** median with a 95% bootstrap
   interval of **+0.050% to +2.055%** — excluding zero, so the frozen rule
   retains the candidate. RAR-M30's 7.143% make/unmake weight projected +0.96%,
   so the mechanism behaved as reasoned. Emitted code grew **468 -> 542**
   instructions, refuting "LLVM already fuses this" while confirming the fused
   path trades code size for fewer dependent memory operations.

   **RAR-M32's earlier `NO_CHANGE` is VOID, not a negative result.** It was
   taken while a Manta SPRT held the host at 50.2–53.4% CPU busy against
   3.7–5.8% for the comparable 4.11b.8 run; the same baseline code re-measured
   **40.7% faster** once idle. The runner now asserts host idleness rather than
   annotating it, and that gate was proven to fire under a deliberately absurd
   threshold before this run was trusted.

   Two limits are recorded rather than smoothed over. The registered interval
   half-width was predicted at 0.33–0.46% and came out at **1.003%**, because
   the projection scaled a bootstrapped median's variance by naive `sqrt(n)`;
   future board timing registrations must derive precision from the resampled
   statistic actually used. And the lower bound sits near +0.05%, so the
   evidence supports "the gain is above approximately zero", not a bankable
   0.9% floor — no Elo is claimed. Pooled-PGO qualification of the integrated
   board work remains **4.11b.16**; playing qualification remains 4.11b.17.
   The targeted per-piece-class relocation test is in `8a73cfd`. Evidence and
   reproduction: `analysis/relocation_2026-09-07.md`,
   `tools/results/relocation-411b9-v2/` (ignored, local; `candidate.patch`
   archived there) and the voided `tools/results/relocation-411b9/`.

10. **4.11b.10 Share useful pin/check information -- DONE, `NO_CHANGE`
    (RAR-M34, 2026-09-08).** 4.11b.8 restored the
    previous x-ray calculation; RAR-M31 retains the alternative and its local
    gains, but no proven whole-search gain. 4.11b.9 has now been accepted and
    changed the make/unmake path, so RAR-M30's region shares are stale for that
    region; remeasure remaining cost against `5c439da` before deciding whether
    to retry it or alter state lifetime. Search already
    passes check hints and shares pins across capture/quiet stages. Do not
    implement that work again. Measure the remaining duplication between
    compute_pinned, check_info and SEE king-safety queries, including both
    kings. 4.11b.5 removed stale see_pins; any sharing must respect the evolving
    exchange occupancy and cannot restore original-position pin masks. Compare
    node-local lazy sharing with per-ply cached state; explicitly invalidate
    or restore it through real/null moves, undo and worker cloning. Reckless
    InternalState/update_threats and Stockfish StateInfo/set_check_info are
    reference ownership models, not instructions to eagerly compute every
    threat map. Keep only caches whose consumers repay their maintenance.

    **Disposition: `NO_CHANGE`, closed on structure rather than cost.** The
    three producers do not overlap. `compute_pinned` queries from **our** king
    against **their** sliders; `check_info` queries from **their** king against
    **our** sliders; their only common input is `all_occ`, so neither can serve
    the other in any node. `see_recapturer` tests king safety against the
    **evolving** exchange occupancy `after = occ ^ from`, so substituting a
    real-position mask is exactly the stale `see_pins` defect 4.11b.5 repaired
    — a correctness boundary, not a cost tradeoff. The cross-ply candidate
    (parent `check_info` versus child `compute_pinned` at the same square)
    fails on both changed occupancy and a different predicate: `check_info`
    accepts a sole blocker of either colour, `compute_pinned` requires a sole
    friendly blocker behind an x-ray pass. The one genuine sharing opportunity
    — one pinned set per node across capture and quiet stages — is already
    implemented by the 10.3 speed pass and measurably active: **422,246**
    staged quiet generations cost **zero** extra `compute_pinned` calls.

    Activation is low regardless. At stride 1 on `bench 13`, `compute_pinned`
    runs 0.274 times per node and `check_info` 0.295, both already lazily
    gated, while SEE threshold queries run 0.993 per node. Counter units were
    reconciled before differencing: the 163,486 gap between generator calls and
    `compute_pinned` calls is exactly the `generate_captures` early-out.

    Region cost was **not** re-profiled for this leaf: a fresh ETW capture
    needs an elevated prompt and is a maintainer job, and it cannot make
    un-shareable work shareable. The post-`5c439da` share update is derived
    arithmetically instead and moves every unchanged region by at most 0.06
    percentage points. A fresh profile is still owed to **4.11b.11**, where
    region size genuinely decides, and 4.11b.11 must re-baseline SEE: the
    4.11b.9 benchmark showed `threshold SEE only` down 1.77/0.98/1.68% in all
    three rounds, most likely code layout after `make_move_inner` grew.

    Do not reopen this on donor-engine similarity. Reopen only if a new
    consumer appears that needs pin or check geometry Rarog does not already
    compute once per node. Evidence:
    `analysis/pin_check_sharing_2026-09-08.md` and
    `tools/results/pinshare-411b10/` (ignored, local).

11. **4.11b.11 Optimize the corrected SEE kernel -- DONE, `NO_CHANGE`
    (RAR-M35, 2026-09-08).** Carry attacker sets through
    the exchange and add newly exposed bishop/rook rays after removing the
    selected attacker, rather than recomputing all attackers twice per
    exchange step. Reuse valid pin information and preserve selected-king
    legality, recapture promotion gain/new victim value and threshold boundary
    parity. Preserve all 41 external fixtures from 4.11b.5; do not reinstate a
    moving-piece-loss shortcut that ignores a later promotion. Reference Reckless board/see.rs,
    and test its logic against the Rarog contract rather than assuming it is
    an exact oracle. Evaluate real search thresholds as well as zero; keep
    a corpus with more than the v1 ten captures. Default-value verdicts must
    be identical to the corrected baseline. Compare normalized benchmark
   throughput and whole-search NPS separately.

   **Implementation handoff.** Change only attacker-set maintenance inside the
   full/threshold SEE kernels. Inputs remain the board occupancy, move and
   `SeeValues`; stored transient state is the evolving occupancy/attacker set;
   consumers remain the existing ordering/pruning/LMR/history callers. The
   RAR-M28 selected-king, created-pin, x-ray, promotion/new-victim and equality
   semantics are fixed. All 41 external fixtures, parity cases and production
   fingerprint must remain exact. Normalized M29 throughput and controlled
   full-search NPS decide retention; do not fit values or restore stale pins.

   **Disposition: `NO_CHANGE`, rejected on throughput, not on defects.** The
   candidate was implemented in full: a carried all-colour attacker set built
   once per exchange, selected with `attackers & color_occ(side)`, extended by
   `see_expose` with only the ray that vacating `from` can open. It was correct
   and verified hard -- `bench 13` exact at 7,601,220 / EBF 2.474, all 41
   external fixtures passing, and a `debug_assert_eq!` comparing the carried set
   against a fresh `attackers_to_color` on EVERY SEE call, proven live by a
   deliberate sabotage that made the parity walk panic at once. With that guard
   active the debug suite passed 275/275.

   It still lost. The registered stage-1 screen required `threshold SEE only`
   to improve in all three alternating rounds with a median of at least +5%;
   it measured **-2.92 / -10.42 / -0.69%**. Zero rounds up, so stage 2 never
   ran and the production path is withdrawn; `src/` is byte-identical to
   `8d7da2c`. Round 1 was visibly disturbed on unrelated columns, so the honest
   effect is rounds 0 and 2, a **1-3% regression**; discarding round 1 would
   not change the verdict and no discard was applied.

   **A premise of this leaf was wrong and is corrected here.** The two
   `attackers_to_color` calls per exchange step are not duplicates. The first
   builds the recapturer set at the target; the second is the mandatory
   per-candidate **selected-king legality test**, at a different square, under
   a different occupancy, evaluated once per candidate rather than once per
   step. A carried set of the target's attackers is structurally incapable of
   serving it. The optimization could therefore address at most one of the two
   queries while adding a fixed entry cost -- `attackers_to` builds both
   colours' unions where the old first step built one -- which `see_ge_impl`
   pays on a large share of its 7.55M early-exiting threshold calls.

   **Future SEE work targets the king-legality test, not the attacker set.**
   Reopen carried attacker sets only if that test is first made cheaper or
   rarer by a separately qualified change AND a fresh profile still shows the
   recapturer rebuild as a material share. Donor-engine similarity is not
   evidence; this measurement is direct evidence against. Evidence:
   `analysis/see_kernel_2026-09-08.md` and `tools/results/see-kernel-411b11/`
   (ignored, local; `registration.md` frozen before timing).

12. **4.11b.12 Decide whether to cache king squares -- DONE, `NO_CHANGE`
    (RAR-M37, 2026-09-08).** Basilisk maintains two
    king squares; Rarog and Reckless extract them from bitboards. Prototype
    only if 4.11b.7 still shows a material cost after shared geometry work.
    Include cache maintenance, layout and every special make/unmake in the
    measurement. Add the fields to independent consistency reconstruction.
    Close without a cache if the complete candidate has no robust benefit.

    **Disposition: `NO_CHANGE`, no prototype built.** The refreshed RAR-M36
    profile reads king-square lookup at **0.502%** (RAR-M30: 0.544%), and the
    conditional trigger — material cost remaining after shared-geometry work —
    is not met. 4.11b.10 and 4.11b.11 both closed without touching the board,
    so nothing redistributed this cost; the small move is only 4.11b.9
    re-weighting every share.

    **The register asked for a predeclared practical floor and none was
    registered.** The measurement is already exposed twice, so declaring a floor
    now would be choosing a number to fit a result. The decision therefore does
    not use one. It rests on instrument capability: the 2x-local whole-search
    ceiling is **0.25%**, while RAR-M33's qualification run measured a bootstrap
    half-width of **1.003%** and RAR-M35's heavier design projected 0.6-0.7%.
    The best possible version of this candidate is two to four times smaller
    than the uncertainty of the gate that must accept it, so it cannot reach
    `LOCAL_QUALIFIED` however it is implemented.

    The realistic gain is smaller still. `king_sq` is one bitboard load plus a
    `tzcnt`; a cache replaces the `tzcnt` with a field load and removes no
    memory access. The 0.502% is an overlapping mechanism share already counted
    inside the generation, check-query and SEE regions. Against that, the cache
    must be maintained through castling (which moves the king) but not promotion
    or null moves, restored on undo for both colours, copied on worker cloning,
    and added to independent consistency reconstruction — the same class of
    derived state whose staleness went undetected in 4.11b.5's `see_pins`.

    Reopen only if a profile shows king lookup above **2%**, where a 2x-local
    improvement would reach a resolvable 1% ceiling, AND a caller is found that
    calls it in a loop rather than once per node. Donor similarity is not a
    trigger. Evidence: `analysis/king_square_cache_2026-09-08.md`.

13. **4.11b.13 Make history capacity and mutation contracts explicit -- DONE
    (RAR-M38, 2026-09-08, `f70ac19`).** Preserve
    arbitrary legal game-history length; do not replace the vector with a
    silently clamped fixed array. Reserve game history plus maximum search
    headroom before the hot path, including worker clones; prove no mid-search
    growth after reservation and exact repetition/history restoration.
    Current initial capacity is 128 and clone preserves capacity, not a
    guaranteed search margin. Review public mutable derived fields and APIs
    such as is_legal versus legal_move: the latter returns canonical flags,
    whereas a boolean does not canonicalize the caller's original Move.
    Current TT callers use legal_move correctly; preserve that property.
    Narrow mutation access where useful without adding hot validation work.

    **Disposition: tightened, behaviour-neutral, no speed claim.** The history
    vector is kept — a clamped fixed array would silently drop repetition
    evidence in a long game. `Board::reserve_history` is `pub(crate)`, and
    `search_impl` reserves **`MAX_PLY`** on the root once, before any hot path
    or helper exists. `Board::clone` already preserved capacity; that is now
    documented as the mechanism by which the reservation reaches every worker,
    since each helper receives its root by cloning.

    **The gap was real but invisible to the instrument that looked for it.**
    Peak history depth is `game_plies + search_depth`, so an ordinary 64-move
    game reaches `len == capacity` at 128 and reallocates on the next search's
    first push. RAR-M30 nonetheless measured **zero** growth across 25,718,154
    pushes because `bench` builds every position from FEN, leaving game history
    empty. The fix is therefore justified on contract grounds, not on an event
    count; no speed benefit is claimed in either direction.

    **`is_legal` audited.** `Move::from_uci` always yields `QUIET`, so only the
    canonical move from `legal_move` carries real flags; a caller that plays its
    own input instead corrupts make/unmake. `is_legal` has **no production
    callers** — its three uses are test assertions that never play the move —
    and all five search sites bind what `legal_move` returns, so the property
    holds. It is now stated in the doc comment and pinned by a test asserting
    the raw flags are `QUIET` and the canonical flags differ, across a double
    push, a capture and a castle.

    Five contract tests cover headroom above a played game, no reallocation
    across a 128-ply walk, capacity surviving a clone and the clone then walking
    without reallocating, exact hash/history restoration on unwind, and the
    canonicalization difference. **The clone test was proven live**: reverting
    `Clone` to drop capacity made it fail, and only it failed. Behaviour-neutral
    at **7,601,220 / EBF 2.474**; debug 280 / release 281, fmt and Clippy clean.
    Evidence: `analysis/history_contracts_2026-09-08.md`.

14. **4.11b.14 Decide whether a larger representation change is justified --
    DONE, `NO_CHANGE` (RAR-M39, 2026-09-08).**
    Only open an implementation if the preceding profile still identifies
    substantial representation cost. Compare the current 12 colored-piece
    bitboards with six type plus two color bitboards, undo-by-inverse-key-work
    with compact per-ply restoration, and legal versus selectively checked
    generation where actual search consumers justify it. Reckless uses six
    type boards plus colors and copies InternalState; Stockfish restores
    StateInfo through a chain. Neither is automatically faster here.
    Keep the mailbox, useful auxiliary hashes, compact moves, and supported
    magic/PEXT portability. Register one bounded architecture comparison,
    include cache footprint and HCE search consumers, and stop if it fails to
    beat the optimized current design. Large rewrites require stronger
    evidence, not an exemption from it. Full NNUE stacks remain Phase 5.

    **Disposition: `NO_CHANGE`; no comparison registered, no implementation
    opened.** The gate is not met. RAR-M36 puts no board region above **6.7%**
    (make/unmake 6.677%, generation 6.556%, SEE 5.239%, checks 5.179%), and
    those are the costs of doing the work, not of the representation — a
    different representation trades their constant factors rather than deleting
    them. This session's two direct experiments agree: 4.11b.9 won +0.876% and
    4.11b.11 lost 1-3%, which is not how a representation leaving large wins
    behaves.

    Each named alternative is refused on measured grounds. **Six type boards
    plus colours**: `Board` is **264 bytes** and the variant saves **48**, with
    both figures far inside L1 and neither near a boundary that matters, while
    every `pieces(color, piece)` gains a load and an AND across **208 call
    sites**, **102 of them in `eval.rs`** — the largest region in the profile at
    29.49%. **Per-ply state copying**: `UnmakeInfo` is **24 bytes**, so a
    128-ply stack costs **3 KiB** and stays in L1; copying whole board state
    would cost **128 x 264 = 33 KiB** and leave it, an elevenfold increase to
    avoid inverse work that 4.11b.9 reduced to one fused mask. **Selectively
    checked generation**: the cost is already amortized — RAR-M34 measured
    422,246 staged quiet generations at zero extra `compute_pinned`, and
    `gives_check_fast` runs 25,540,503 times against `gives_check_full`'s
    49,385, about **517:1**.

    Only a compile-time guard was added: `size_of::<Board>() <= 264` and
    `size_of::<UnmakeInfo>() <= 24`, as upper bounds since padding may differ
    between supported targets and only growth invalidates the argument. Each
    message names this leaf, so a field addition that breaks the premise fails
    the build instead of drifting past it. Proven live by adding a `[u64; 4]`
    field. Behaviour unchanged at **7,601,220 / EBF 2.474**; debug 280 /
    release 281, fmt and Clippy clean.

    Reopen only if a profile shows a **single board region above 12%** AND a
    specific mechanism is named by which the new representation removes work
    rather than relocating it. Donor similarity is not a trigger. Evidence:
    `analysis/representation_2026-09-08.md`.

15. **4.11b.15 Record the draw-state policy boundary -- DONE, `NO_CHANGE`
    (RAR-M40, 2026-09-08).** Audit rule-50 clock,
    null-move boundaries, pre-root versus in-search repetition, and repetition
    keys versus TT/evaluation keys as separate contracts. Mate at clock 100
    already correctly outranks the draw: preserve its passing tests.
    Current null-clock/cross-null/root-aware changes were tried and rejected
    in historical games; read tests/draw_semantics.rs and the linked history
    before proposing a retry. Do not bundle them into a speed refactor or
    treat the earlier combined losses as proof about each independent part.
    Record keep/change/retry-trigger for each policy. Any newly justified
    playing change gets its own dependency-complete registration, not an
    automatic Stockfish/Reckless port. Never put rule-50 buckets into the
    repetition identity merely because a TT key might use them.

    **Disposition: `NO_CHANGE` on all four policies; two contracts pinned by
    test in `df94b7d`. Engine source untouched, fingerprint holds.**

    *What RAR-S18 establishes and what it does not.* Arm A (null-clock +
    cross-null fence + root-aware) measured **−7.21 ± 6.03**; arm B (the same
    without root-aware) measured **−11.91 ± 7.67**. Both exclude zero, so both
    bundles were harmful. **Neither isolates a single part.** B is worse than A
    by 4.70, which might suggest root-awareness was the least harmful
    component, but the intervals overlap heavily (`[−13.24, −1.18]` against
    `[−19.58, −4.24]`) so that ordering is not supported. No disposition below
    leans on these numbers as evidence about one part.

    **1. Rule-50 clock — KEEP, no retry trigger.** Mate on the 100th-clock move
    outranks the draw; four tests cover mate, check-with-escape, stalemate and
    mate-in-one at clock 99. Correctness is not a tuning surface.

    **2. Null-move boundaries — KEEP.** Nulls increment the clock and clear en
    passant. The cross-null question resolves on structure rather than Elo:
    `is_repetition` compares the full hash including side to move, so crossing a
    null can only produce a false NEGATIVE, never a false positive; and
    `can_declare_draw` is reached only from `game_result` and the root
    tablebase gate, where history contains no nulls. The rejected fence
    therefore guarded a scoring imprecision inside search, not a legality
    defect. *Retry trigger:* a measured case where a cross-null match changes a
    **root** best move, with its own registration. Semantics and donor ports
    are not triggers.

    **3. Pre-root versus in-search repetition — KEEP; partial root-awareness
    already exists.** The aggressive twofold is a deliberate strength heuristic;
    the arbiter keeps `is_repetition(3)`. The guard at `search.rs:2218` is
    `ply > 0`, so the root is already never scored a draw in search — the
    rejected change was a further one. *Retry trigger:* a demonstrated case
    where the twofold costs a **won game**; node counts do not qualify.

    **4. Repetition versus TT and evaluation keys — KEEP, audited clean, now
    pinned.** Three distinct identities: repetition uses the position hash only;
    the TT key is `board.hash` with the clock applied on READ as a mate-score
    correction in `tt::score_from_tt`; the evaluation cache stores a
    `halfmove_clock` compared for equality (`eval.rs:1254`) as entry validity,
    not in any hash. The prohibition holds and the TT key does not use rule-50
    buckets either. A test now asserts two positions differing only in the clock
    share one hash. A second test records that the scan bound is a **cost**
    choice: nulls advance the clock so the bound can reach past an irreversible
    move, which is harmless because such a move changes piece placement
    permanently and those positions carry a different hash.

    *Proving the tests live took three sabotage attempts* — via
    `check_consistency` (a path the draw tests never call) and via `from_fen`
    before the clock is parsed (still zero, so a no-op) — before mixing the
    clock in after parsing failed the test, and only it. A sabotage that does
    not visibly change the thing under test proves nothing.

    Debug 282 / release 283, fmt and Clippy clean; `bench 13` unchanged at
    7,601,220 / EBF 2.474. Evidence: `analysis/draw_policy_2026-09-08.md`.

16. **4.11b.16 Qualify the integrated board candidate -- DONE, QUALIFIED
    (RAR-M41, 2026-09-08).** Run debug/release
    board/SEE/parser/draw tests, independent randomized move/state comparison,
    both slider backends and supported-target checks for touched code,
    fmt/Clippy, and feature-off behavior. Rebuild exact production features,
    record fingerprint and explain all differences from section entry.
    Run interleaved within-Rarog native A/B with per-position distributions,
    then revision-matched pooled-PGO whole-search NPS. Record host load,
    affinity, compiler/ISA, warmup, medians and spread; require an idle enough
    host and a gain exceeding the predeclared noise/practical floor to bank
    a small speed claim. Revisit regressions by cohort. Do not infer Elo from
    a cross-engine throughput ratio or suppress a correctness failure.

    **Disposition: QUALIFIED; a speed claim is banked, no Elo is claimed.**

    *Correctness matrix, all passing.* Debug **282** / release **283**;
    `see_contract` 8/8 including all 41 external fixtures, `see_pins` 6/6,
    `draw_semantics` 8/8; randomized `board_differential` and `fuzz_lite`;
    **72** tests under the PEXT slider backend; `cargo fmt --check` and Clippy
    `--all-features --all-targets` clean; feature-off default throughout; and
    the production fingerprint **7,601,220 / EBF 2.474** on all six PGO
    binaries. *Difference from section entry:* exactly one, the 4.11b.5 SEE
    repair that set the current fingerprint; every later board change preserved
    it, which the six independent checks confirm. *Not runnable here and stated
    as such:* the supported-target cross build, because
    `aarch64-pc-windows-msvc` is not installed and the vendored fathom C code
    needs a cross `cl.exe`. Touched code is architecture-neutral and the
    4.11b.14 assertions are upper bounds; this belongs on the ARM64
    compatibility host.

    *Performance.* Arms are `1d720af` against head — exactly the fused
    relocation, history reservation and footprint assertions — and both are
    behaviour-identical, so trees match and NPS at fixed nodes is clean. Six
    PGO binaries, three per arm; **PGO build variance was verified rather than
    assumed**, two builds of identical source differing by hash. A **null pair**
    of same-revision builds measured **+0.222%, 95% [-0.130%, +0.630%]**,
    containing zero: the instrument is unbiased, and its upper bound became the
    effective floor. The main comparison over 96 alternating pairs at 2,000,000
    nodes measured **+1.421%, 95% [+0.953%, +1.764%]**, 91/96 pairs faster, max
    host busy 9.11% against a 15% gate. Every registered condition passed.

    *Calibration.* Projected half-width ~0.5% from RAR-M33's **measured**
    1.003%; got **0.405%** — deriving the projection from a measured width
    rather than a variance model corrected RAR-M33's miss. The estimate exceeds
    RAR-M33's non-PGO +0.876% but is consistent with it, since that interval
    ([+0.050%, +2.055%]) contains 1.421%; PGO may amplify the fused path, and
    this arm also carries `f70ac19`/`20ee114`. The registration's warning that
    variance might exceed the effect did not materialise.

    *Claimed:* +1.421% [+0.953%, +1.764%] whole-search NPS under production
    pooled-PGO settings. *Not claimed:* any Elo. The ~2 Elo per 1% NPS constant
    would suggest ~+2.8 Elo, but that is an inference, not a measurement, and no
    games were played. The playing gate remains **4.11b.17**. Evidence:
    `analysis/cluster_qualification_2026-09-08.md`.

17. **4.11b.17 Register and qualify the playing cluster.** The SEE repair and
    any other deliberate search-affecting behavior require one
    dependency-complete game verdict before becoming the accepted foundation
    for 4.12. Register hypothesis, exact baseline/candidate and rebuild
    recipes, bounds, cap, book, clock, adjudication and stop rule BEFORE
    games. Size from RAR-M10 at the expected effect: unknown-sign repair
    uses a justified symmetric bracket, pure simplification permits a small
    loss, and a justified gain uses the default near-zero bracket. Do not
    invent a numerical prior or a success threshold after a screen.
    If the gate rejects or caps, investigate implicated search consumers and
    re-register a coherent repair; do not relax the oracle or proclaim a
    known-bug baseline correct. Behavior-neutral speed-only leaves do not
    each require their own SPRT. Update the accepted fingerprint only with
    the actual integrated verdict and its evidence.

18. **4.11b.18 Refresh affected endgame evidence and close -- DONE
    (RAR-M42, 2026-09-09).** Before 4.12,
    compare the accepted board head with the exact 4.11 head on the corrected
    frozen endgame corpus/budgets and existing theory/draw safeguards. SEE
    changes may change conversion/search occurrence even if HCE coefficients
    stay fixed. Reuse unchanged reference arms and static-eval artifacts when
    their dependencies really are unchanged; do not rerun them reflexively.
    Version changed baselines/floors and rederive the registered 4.12 order if
    its measured inputs change. Never silently overwrite 4.11 evidence.
    Close each conditional leaf with an explicit measured no-change result
    where appropriate, archive the rebuild recipes and source/binary
    fingerprints, synchronize GUIDE/PLAN, and hand Phase 5 the accepted state
    contract. The section is done only after the necessary gate and evidence
    refresh, not after a faster perft number.

    **Disposition: refreshed, floors PASS, 4.12 order verified UNCHANGED.
    Section 4.11b closes.** Both arms are the binaries RAR-E15 already gated, so
    nothing was rebuilt; every instrument is node-budgeted and seeded, so none of
    this depends on host load.

    **What could be reused was checked, not assumed.** The registered order's
    dominant input is a frozen 36,400-game Colosseum corpus classified by
    material, which no engine can change. But `endgame_measurement_layers.md`
    describes drawn-share bias as "per position, static", and it is **not**:
    `endgame_drawn.py` takes `--engine` and searches every position. Reusing it
    on that reading would have been reuse on a false premise.

    **Mechanism.** Over the frozen 83-position `tests/endgames.epd` at 60,000
    nodes, 19 positions search differently, split exactly: **34.5% (19/55) of
    positions with material on both sides, 0.0% (0/28) bare-king**. SEE fires
    only where captures exist. This is why conversion is the wrong instrument
    here — its four families are all bare-king and it came back **byte-identical**
    across both heads, which alone would have produced a false "nothing changed".

    **Layer 1 is clean.** `endgame_truth.py`, 19 families x 100 positions at
    60,000 nodes, cohort digest `fe4866045506636f...` matching the registered
    floors. **Theory verdicts identical on every family; no clean win newly
    discarded.** Under the precedence rule that truth is an absolute veto, that
    is the result that had to hold.

    **Floors PASS on both arms.** The 4.11 head reproduces the registered
    aggregate exactly (0.9300 -> 0.9300), validating the instrument before the
    candidate is read; the accepted head reads 0.9300 -> **0.9336** (+0.4 SE, not
    significant). Eleven families moved in both directions, the largest being
    **KRP-KR conversion 0.9178 -> 0.9726** — notable because KRP-KR is the top
    family in the registered order at 10.04% occurrence — and **KQ-KR dtz
    progress +2.7 SE**, a ratchet candidate. **The floors file was NOT updated**:
    KRP-KB's win-preserving rate fell 2.2 SE, and raising one floor while
    lowering another in the same commit as the change that moved them is exactly
    what the rule against relaxing a check alongside its own change forbids.

    **The order was rederived, not asserted.** With the same held-constant
    inputs, the 4.11 head reproduces `endgame_ranking_v2.json` across all twenty
    families exactly, and the accepted head's order equals it. The first
    rederivation used `--occurrence-scope all` and disagreed with the registered
    v2; that was a wrong parameter, since v2 records its provenance as
    `Rating Tournament [engine], 10,000 games`. 4.11.6's retry trigger is not
    fired: occurrence was not re-measured.

    **Versioned, not overwritten:** `endgame_drawn_census_v2.json` and
    `endgame_truth_baseline_v2.json` are new; every v1 artifact and
    `endgame_floors.json` are byte-identical. **Owed:** KRP-KB win-preserving
    0.9990 -> 0.9949 (−2.2 SE), reported and non-blocking, owner **4.12.6**;
    blocking if a later change pushes it past 3 SE. **Not done and stated:**
    `endgame_reference_results_v1.json` and `endgame_tree_occurrence_v1.json`
    were held constant to isolate the census effect, so this work does not
    establish that they are current. Evidence:
    `analysis/endgame_refresh_2026-09-09.md`.

19. **4.11b.19 Caller-owned move-list delivery and generator constant factors
    -- CLOSED 2026-09-09 / I1 (RAR-M44). (a)+(b) banked +2.48% NPS; (c)
    rejected in search and reverted; (d) re-measured.**
    Inserted after the section closed. Research is done; this leaf is
    implementation, cheap deterministic qualification and one pooled-PGO
    throughput measurement, followed by a bounded, conditional screen.
    Evidence and the exact probe: `analysis/movelist_delivery_2026-09-09.md`.

    **Research card.** *Mechanism:* `generate_legal_movelist`,
    `generate_captures`, `generate_captures_pinned` and `generate_quiets_pinned`
    build a `MoveList` in a local and return it by value; in the fat-LTO
    production build LLVM does not apply return-value optimisation and each
    normal return path ends in `memcpy(out, local, 520)`. Basilisk's search
    and its benchmark harness both write into a caller-owned `MoveList&`, and
    its harness comment records that the copy had been "measured as if it
    were move generation". A within-Rarog probe that only removed the copy
    moved the cross-engine bench columns **legal captures +40.5%, legal moves
    +11.2%, two-ply +4.6%**, with the untouched SEE and perft controls at
    +0.05% and -0.69%. *Interactions:* none on behaviour. The generated move
    set and order are unchanged, so the search fingerprint must stay exactly
    **7,601,220 / EBF 2.474**; the staged picker scores the list into a
    `ScoredMoveList` immediately, so the caller-owned list is a short-lived
    stack local, not new per-ply state. *Invariants:* `MoveList`'s
    `MaybeUninit` prefix contract (only `..len` is initialised; `clear` only
    resets `len`); the bench's frozen work quanta 128/10/128/10/197281/4597
    and the `cross-engine-board-v1` profile; perft and board-v2 oracle parity
    on magic and PEXT; debug and release suites. *Falsifier:* the pooled-PGO
    whole-search NPS interval includes zero or its point estimate is below the
    **+0.5%** practical floor RAR-M41 used; then (b) closes `NO_CHANGE` and only
    (a) and (d) remain, because a harness correction is owed regardless of
    speed. One pooled run decides; do not rerun to chase a number.

    **Prospective prediction, frozen before measurement:** (b) gains **+0.5%
    to +1.5%** whole-search NPS under the RAR-M41 protocol (three independent
    PGO builds per arm, `tools/nps_multibuild.ps1 -Cycles 10 -Repeats 3`, idle
    host, null pair reported). Confidence: moderate on sign, low on magnitude;
    RAR-M36's 6.556% generation share bounds it from above.

    **Sub-steps, in order. Engine and tooling/doc changes in separate commits.**

    (a) **Harness parity -- tooling commit. DONE `55e228a`.** In
    `benches/board.rs` make `legal_movegen`, `capture_gen`, `make_unmake` and
    `game_simulation` reuse
    caller-owned `MoveList`s created before timing (one scratch list, plus
    `outer`/`inner` for the two-ply workload), exactly as Basilisk's
    `tests/board_performance.cpp` does. Leave `see_captures` and `perft`
    untouched: they are the session controls in (b) and (d). Keep the profile
    string, corpus, quanta, estimator and preflight unchanged; update the
    module comment to say why lists are caller-owned. Verify: preflight
    prints `PASS` with the same six quanta; `cargo fmt --check`;
    `cargo clippy --all-features --all-targets` zero warnings.

    (b) **Production delivery -- engine commit. DONE `021dc98`; BANKED at
    +2.48% NPS.** Add to `src/board/moves.rs` `MoveList::clear(&mut self)`
    (`len = 0`, no element writes). Add to
    `src/board/movegen.rs` the out-parameter forms and route the by-value
    forms through them so there is one generator body per kind:
    `generate_legal_into(&Board, &mut MoveList)`,
    `generate_captures_into(&mut Board, &mut MoveList)`,
    `generate_captures_pinned_into(&mut Board, &mut MoveList) -> Bitboard`,
    `generate_quiets_pinned_into(&Board, Bitboard, &mut MoveList)`; each
    clears the list first. Mirror them on `Board` next to the existing
    wrappers (`generate_legal_movelist`, `generate_legal_captures_pinned`,
    `generate_legal_quiets_pinned`). Convert every hot caller to a local
    `let mut list = MoveList::new();` followed by the `_into` call -- at
    registration these were `MovePicker::staged` and the `GenerateQuiets`
    stage in `src/search.rs`, the root and in-check/excluded-node full-list
    sites in `negamax`, in-check quiescence, and `movegen::perft`; re-locate
    them with `rg "generate_(legal_movelist|captures|legal_captures_pinned|legal_quiets_pinned)\("`
    because line numbers drift. Delete by-value wrappers that end up unused
    rather than leaving dead code; tests may keep a by-value convenience if
    they need one. Mechanical proof, not eyeballing: rebuild the lib with
    `cargo rustc --release --lib --no-default-features -- --emit=asm` under
    the RAR-M20 flags and require that no `callq memcpy` preceded by
    `movl $520, %r8d` remains inside the four generators; archive the grep
    output with the commit hash. Then: fresh no-feature build reproduces
    **7,601,220 / EBF 2.474** on magic and PEXT; `cargo test` debug AND
    release; fmt; all-feature/all-target Clippy; board-v2 oracle tests and
    `tests/board_differential.rs` pass. Hand the maintainer the pooled-PGO
    NPS run (RAR-M41 protocol, prediction above) and record the result in
    EXPERIMENTS under RAR-M44 before touching (c).

    **What landed, 2026-09-09.** The two commits are in dependency order --
    the engine API first (`021dc98`), then the harness that consumes it
    (`55e228a`) -- because the caller-owned bench cannot compile before the
    `_into` forms exist and the two must stay in separate commits.
    One deviation from the registered caller list, inside the leaf's
    mechanism and reported rather than silently taken: **ProbCut's capture
    generation** in `negamax` reached the same copy through
    `Board::generate_legal_captures` and was converted with the rest. It was
    the only 520-byte copy left in the binary after the registered sites were
    done, and it is the site the assembly check found. No by-value wrapper
    became unused -- tests, `benches/board_v2.rs` and `src/diag.rs` still use
    all of them -- so none was deleted. Verification, all on the final state:
    the fat-LTO binary asm has **zero** `movl $520` + `callq memcpy` sites,
    against **four** on the pre-change tree (`movegen::generate_captures`,
    `movegen::generate_legal_movelist`, `MovePicker::staged`,
    `MovePicker::next`), and the same scan still reports the copy inside the
    surviving by-value wrappers, so a clean line is a measurement rather than
    an absent pattern; `bench 13` reproduces **7,601,220 / EBF 2.474** on
    magic and on PEXT; `cargo test` 26/26 debug and 26/26 release including
    `board_v2` and `board_differential`, plus 76 tests under the PEXT slider
    backend; `cargo fmt --check` clean and `cargo clippy --all-features
    --all-targets` at zero warnings from a clean rebuild, with the bench
    target proved covered; the parity bench's preflight passes on the frozen
    quanta and fails with exit 101 when one is perturbed. Evidence:
    `tools/results/movelist-delivery-20260909/` (ignored: `asm_lib_after.txt`,
    `asm_bin_before_after.txt`, `fingerprint.txt`, `harness_parity.txt`,
    `asmcheck.py`, `where520.py`).

    **(b)'s registered measurement, run 2026-09-09 on an idle host.** Three
    PGO builds per arm through `tools/nps_multibuild.ps1 -Cycles 10
    -Repeats 3`, base at `f10b999`: **+2.48% whole-search NPS, 95% bootstrap
    [+2.29%, +2.65%]** (3,142,298 -> 3,220,173 n/s; best-of +2.81%). Null
    pair `cand-1` vs `cand-2`, same revision: **-0.21% [-0.57%, +0.23%]**, so
    the instrument carries no arm-level offset. All six binaries reproduce
    7,601,220 / EBF 2.474 and all six hashes differ. **BANKED** -- the lower
    bound is more than four times the registered +0.5% floor. **The frozen
    prediction of +0.5% to +1.5% missed high, in magnitude only**; the
    calibration is in RAR-M44 and `analysis/movelist_delivery_2026-09-09.md`.
    NPS is not Elo and none is claimed; behaviour is unchanged, so RAR-E15's
    verdict stands and no game gate is owed. This unblocks (c).

    (c) **Bounded constant-factor screen -- CLOSED `NO_CHANGE` 2026-09-09.
    Two of four candidates cleared the bench gate; the bundled pooled-PGO run
    then measured -0.55% whole-search NPS and both were reverted. Class I2.** The remaining ~32% legal-moves gap is also
    constant factor; the candidates below were seen in the same assembly and
    are listed in priority order. Rules: one candidate at a time on top of
    (b); measure each with the parity bench from (a) using the RAR-M43 runner
    discipline (affinity, three alternating rounds, controls within 1%);
    predeclare the expected column movement before running; promote a
    candidate into one bundled pooled-PGO run only if it moves legal moves
    or two-ply by **at least +3%** with controls flat and the fingerprint
    exact; stop when the list is exhausted, and do not add candidates
    mid-leaf.
    1. Force inlining of the out-of-line helpers that the LTO'd
       `generate_legal_movelist` still calls: `push_pawn_move_flags` (four
       call sites, `#[inline]` did not take), `is_attacked_with_occ` (per
       king target), `compute_pinned`. Expect a modest single-digit gain;
       check code size does not explode the generator.
    2. Const-generic colour: `gen_moves_pinned::<CAPTURES, QUIETS, US>`
       with one runtime dispatch at each public entry, specialising
       `gen_pawn_moves`, `gen_unpinned_pawn_quiets/captures`, `gen_castling`
       and the pinned-pawn loop, as Basilisk's `template<Color Us>` does.
       Move set and order must be byte-identical; the board-v2 move-set
       oracle is the test.
    3. `MoveList::push` bounds check: `self.moves[self.len]` is checked in
       release. Either prove elision or replace with `get_unchecked_mut`
       behind a `debug_assert!`, documented as a KEEP-UNSAFE with the
       measured reason, following the existing 9.0 policy in `moves.rs`.
    4. Lowest priority, ceiling small: the `LazyLock<AttackTables>` state
       checks (seven per generation, cold branches) and the `Vec`-backed
       slider tables. Try only if 1-3 leave more than a 15% column gap, and
       only as a layout change that keeps runtime initialisation.
    Explicitly out of scope: changing the move encoding to Basilisk's
    flag-free `make_move(from, to)`; that is a representation change owned
    by 4.11b.14's reopen rule, and it would touch every consumer of
    `Move::is_capture`.

    **(c) OUTCOME, 2026-09-09.** All four registered candidates were screened,
    one at a time, in the registered order; none was added. Each prediction was
    frozen in `tools/results/movelist-c-screen-20260909/predeclared.md` before
    its run. A null pair of the base binary against itself was measured first
    and put every column inside +/-1%, so the instrument resolves the +3% gate.

    | Candidate | legal moves | two-ply | verdict |
    |---|---:|---:|---|
    | 1 force inlining | **+4.68%** | **+4.59%** | PROMOTED `be5c02a` |
    | 2 const-generic colour | **+7.38%** | **+7.78%** | PROMOTED `c969ccd` |
    | 3 unchecked `MoveList::push` | **-9.57%** | **-9.95%** | REJECTED, reverted |
    | 4 hoist `LazyLock` checks | **-4.25%** | **-1.54%** | REJECTED, reverted |

    Cumulative from the (b) head: legal moves +12.7%, two-ply +12.7%, legal
    captures +10.7%. Fingerprint exactly 7,601,220 / EBF 2.474 on magic and
    PEXT at both promoted steps, with 26/26 debug and release tests, fmt and
    Clippy clean each time.

    **A registered rule that could not be applied as written, reported before
    any candidate ran.** (c) inherits "controls within 1%" from the RAR-M43
    discipline. That worked for (b), where `see_captures` and `perft` kept
    by-value delivery and were genuinely untouched. It cannot work for (c):
    the change is in the GENERATOR and both controls call it inside their
    timed region, so a generator win is EXPECTED to move them and the literal
    rule would reject a candidate for working. Session validity was taken from
    the interleaved order plus the measured null pair instead; the columns are
    still reported. **The promotion gate was not loosened** -- >= +3% on legal
    moves or two-ply plus an exact fingerprint, exactly as registered.

    **The screen's durable result is a negative one, and it replicated.**
    Candidates 3 and 4 both provably removed work at the source level and both
    made the generator slower, by 9.6% and 4.3% on legal generation. Mechanism
    is unresolved and was deliberately not chased. **Retry trigger: neither
    may be retried until there is evidence explaining WHY removing the check
    costs, not merely a new idea for removing it.** What paid was the opposite
    move -- giving the compiler more information, by forcing inlining and by
    making the colour a compile-time constant.

    **(c) bundle run, REGISTERED and frozen before any run.** Same RAR-M41 instrument, `tools/nps_multibuild.ps1 -Cycles
    10 -Repeats 3`, three PGO builds per arm. Base is the (b) head -- the same
    three binaries that were (b)'s candidate arm, so the two measurements
    chain without a gap. Prediction **+0.8% to +2.2%** whole-search NPS, floor
    **+0.5%**, one run, null pair reported. The band sits below (b)'s +2.48%
    on purpose: (c) moves legal moves about as much as (b) did but legal
    captures only +10.7% against (b)'s +40.5%, and search generates captures
    far more often. **Consequence fixed in advance: below the floor, or an
    interval including zero, closes the bundle `NO_CHANGE` and REVERTS both
    commits** -- unlike (a), nothing in (c) is owed regardless of speed -- and
    (d) must then be re-run. **Stated risk the microbenchmark cannot see:**
    candidate 2 roughly doubles the generator (`generate_legal_into` assembly
    898 -> 1489 lines), and a bench running one hot generator does not pay the
    I-cache cost a real search does. Commands and binaries:
    `tools/results/movelist-c-screen-20260909/HANDOFF.md`.

    **(c) BUNDLE RESULT and REVERT, 2026-09-09.** One run on an idle host:
    **-0.55% whole-search NPS, 95% bootstrap [-0.76%, -0.30%]** (3,227,009 ->
    3,209,296 n/s), null pair **+0.06% [-0.46%, +0.44%]**. Below the +0.5%
    floor and NEGATIVE with an interval excluding zero. The registered
    consequence was applied without argument, which is the entire reason it
    was written down beforehand: **`be5c02a` and `c969ccd` reverted at
    `39542b7`**, `src/board/movegen.rs` restored byte-identical to the (b)
    head, re-verified at 7,601,220 / EBF 2.474 on magic and PEXT, 26/26 debug
    and release, fmt and Clippy clean. **The prediction missed in SIGN** --
    +0.8% to +2.2% predicted, -0.55% measured -- and the postmortem is
    deliberately thin: the stated pre-run risk (candidate 2 doubling the
    generator, whose I-cache cost this bench cannot see) is the leading
    explanation and is **not established**; the pair was gated as one bundle,
    so neither candidate is individually credited or blamed. **Retry rule:
    neither candidate returns without a whole-search instrument that can
    separate them, and a microbenchmark column is not that instrument.**

    (d) **Re-measure and correct the record -- DONE 2026-09-09, measured
    twice, class V then M.** Rerun the
    RAR-M43 four-arm comparison with the parity harness, in one session,
    against the archived hash-verified Basilisk and Reckless binaries and the
    exact ca03a46 control, following `analysis/board_benchmark_recipe_2026-09-05.md`.
    Then add a supersession note to RAR-M43 and
    `analysis/board_comparison_2026-09-09.md`: the generation gap it reported
    was partly the harness, and its Elo arithmetic omitted SEE and check
    queries. Do not delete the old table; mark it superseded and link the new
    one. Close the leaf with GUIDE and PLAN in the same commit.

    **(d) OUTCOME, 2026-09-09.** Four arms in one session, RAR-M43's runner
    reused unchanged except for the head binary path, affinity mask 4, three
    cyclic rounds, host busy 4.8-5.7% against the recipe's 12% threshold.
    **The control reproduced this time**: the identical `ca03a46` binary is
    within **-0.96% to +1.25%** of its RAR-M43 readings and Basilisk within
    +/-1.2%, so this session is comparable to RAR-M43's rather than
    within-session only -- RAR-M43 itself could not say that, its control
    having drifted 0.7-6.1% against RAR-M20.

    | Gap to Basilisk | RAR-M20 | RAR-M43 | **now** |
    |---|---:|---:|---:|
    | legal moves | 44.5% | 46.2% | **17.7%** |
    | legal captures | 20.9% | 26.2% | **-16.0%, Rarog 19% ahead** |
    | two-ply | 46.5% | 38.9% | **18.6%** |
    | perft(4) | 39.2% | 38.2% | **16.2%** |
    | make/unmake | 31.3% | 11.8% | **7.3%** |
    | threshold SEE | -- | 34.9% | **29.4%** |

    RAR-M43 and `analysis/board_comparison_2026-09-09.md` carry supersession
    notes naming both errors -- the harness copy in the generation columns and
    the Elo arithmetic that omitted SEE and check queries -- with every raw
    figure retained. The corrected arithmetic on measured gaps: closing the
    remaining generation and make/unmake gaps entirely is worth about **+1.5%**
    whole-search NPS rather than +2.9%, about **+2.7%** with SEE included at
    this session's matched-value 1.294x (which independently reproduces
    RAR-M29's ~30%), and check queries are still not in that number, so it is a
    floor. Roughly 3 to 5 Elo at the project's ~2 Elo per 1% constant, **no Elo
    claimed**, far below the 250-355 Elo search deficit -- the prioritisation
    stands. New record: `analysis/board_comparison_411b19_2026-09-09.md`;
    evidence `tools/results/board-compare-d-20260909/`.

    **(d) was then RE-MEASURED, because the (c) revert removed the code that
    table described.** Second session, same instrument and runner, on the (b)
    head: control within -1.32% to +1.81% and Basilisk within -1.21% to
    +0.37% of the RAR-M43 session. **Final gaps to Basilisk: legal moves
    46.2% -> 33.2%, legal captures 26.2% -> -5.5% (Rarog 5.8% AHEAD), two-ply
    38.9% -> 33.1%, perft 38.2% -> 27.2%**, with make/unmake 11.8% -> 12.3%
    and threshold SEE 34.9% -> 35.0% -- both untouched by this leaf and both
    flat, which is the table's internal consistency check. The first session
    is retained as superseded, because the difference between the two IS the
    finding: the (c) head was up to 13% faster on every board column and half
    a percent slower in search. Corrected arithmetic on the final gaps:
    closing generation and make/unmake entirely is worth about **+2.4%**
    whole-search NPS, about **+3.9%** with SEE at this session's matched-value
    1.350x, check queries still uncompared so it is a floor -- roughly 5 to 8
    Elo at the project constant, **no Elo claimed**, far below the 250-355 Elo
    search deficit. RAR-M44's own directional prediction for (b) scores as
    **essentially exact on legal moves** (about 32% predicted, 33.2% measured)
    and **overstated by about half on captures** (about 11% ahead predicted,
    5.8% measured).

    **LEAF CLOSED 2026-09-09.** Banked: (a)'s harness correction and (b)'s
    **+2.48%** whole-search NPS. Closed `NO_CHANGE`: (c). Corrected: RAR-M43's
    gap table and its Elo arithmetic. Fingerprint unchanged at 7,601,220 /
    EBF 2.474 throughout, so no game gate was owed and RAR-E15's verdict
    stands. **The leaf's most transferable result is about the instrument:**
    (b) removed work and +11% legal moves became +2.48% of search; (c) moved
    work around and +12.7% legal moves became -0.55%. A board microbenchmark
    column is not a proxy for search speed here, and its sign need not
    survive. Next: **4.12.1**.

    **Not this leaf:** any HCE, search-policy or pruning change; any change to
    generated move order; a game gate -- the section's playing verdict RAR-E15
    stands and a behaviour-neutral change owes none.

### 4.12 Endgame reference functions (was 4.9a.9-4.9a.28)

Audit, implement where absent, and test each of the twenty reference functions.
The set is 20, not 18: Stockfish 11 carried 22 and `KNPK`/`KNPKB` were later
removed, while current NNUE Stockfish and Reckless no longer provide a
comparable dispatcher, so the final pre-NNUE Stockfish table is the reference.
Reference code supplies cases, failure modes and seed constants; a seed is not a
result. Rarog's present meaningful coverage is 7/20.

**The following rules govern this list.**

- **Recognizers and scale functions are validated by different instruments.**
  Use the Rarog defect kind in the current ranking table, not the historical
  Stockfish function category: KRKN, KRKB, KRKP and KPK currently own
  scale/overclaim questions here. A scale candidate requires an independently
  labelled drawn cohort and the registered family gate; a conversion candidate
  requires move/conversion truth and its gate. A scale doing nothing on won
  positions is not a failure. Tablebase labels (or another justified independent
  truth source) still matter for classifying draws; lack of local 7-man truth
  is not waived merely because a candidate is called a scaler. The unknown
  families first establish coverage, defect and deciding instrument at their
  own leaf. 4.12.1 confirms each classification before implementation.
- **A term's blast radius is its dispatcher condition's PROMOTION CLOSURE.**
  Promotion manufactures material, so a term keyed on "no pawn, rook or queen
  for the winner" is reachable from any pawn family through under-promotion.
  State which families reach a term by promotion and include them, or test all
  twenty-one and let the data name them. Rarog's own mate drive changes six
  families this way (4.11.9).
- **Every family leaf records the same closure block.** Name the dispatcher's
  complete runtime condition, its direct root-material matches, and its
  promotion closure: either prove which families can reach the condition on a
  legal truth-preserving path, including under-promotions and material sheds,
  or test all twenty-one families. Report paired gained and lost conversions,
  not only the net. A same-cohort net conversion loss from a shipped mechanism
  is debt even when other families gain; a nonnegative-net family remains a
  regression guard. Do not turn a historical, instrument-contaminated matrix
  into a current floor: re-measure it on the current corrected instrument when
  a family leaf needs a current rate.
- **A guidance gradient has a FLAT MAXIMUM in whatever it does not
  reference.** Rarog's mate drive scores the two kings and the bishop's colour
  and nothing else, so among moves that leave the losing king on the same
  diagonal it is indifferent to where the bishop and knight stand -- and the
  search tiebreak among equal-scoring moves can then pick a losing one. That is
  exactly RAR-E10's recorded residue: four positions where the engine gives away
  a minor. Check what a drive is blind to before adding weight to what it
  already sees.
- **DTZ slack confounds cross-family conversion comparison.** An eligibility cut
  equalises FEASIBILITY, not MARGIN: a DTZ-10 root in a 50-halfmove budget has
  40 spare and a DTZ-50 root has none. Compare within family, or match slack.
- **Condition on family before believing a cross-family correlation.** An
  apparent king-distance effect vanished and flipped sign once conditioned on
  family; it was family composition. A negative derivation that closes a planned
  feature is a good outcome, not a failure.
- **Tablebase scope is smaller than it looks.** Check how many 7-man families
  the list truly needs before treating one as blocked. The Lichess API
  (`tablebase.lichess.ovh/standard?fen=...`) covers 7 men free and returns
  per-move categories in one response, and agreed exactly with local Syzygy on
  WDL and DTZ in every position tested -- excellent for spot checks, poor as a
  harness basis, because a frozen cache only covers positions one arm visited.
  `endgame_truth.py` should take a repeatable `--syzygy` so a 7-man directory
  can be added alongside the 3-4-5-6 set.

**The table below is the current v2 order**, registered at 4.11.6 and
re-derived at 4.11.12 from the 36,400-game occurrence artifact. Its authoritative
input is `tools/diag/endgame_ranking_v2.json`; preserve the v1 artifact and
historical derivation above. KRPPKRP is 5.40% of Rarog's games, not absent;
its blocker is local 7-man truth. KQKRPs is 0.42% of games and 4.41% of the
measured tree. Ranks 3–8 form an evidence band, not a proven priority separation.

RAR-M21 at 4.11.7 now qualifies the conversion inputs by budget: KBN-K and
KQ-KP reach full conversion at 200k/600k; KQ-KR's deficit falls 23/13/3,
while KNN-KP remains non-monotone and behind. The table's numerical defect
inputs remain the frozen v2 inputs, not newly measured higher-budget values.
Do not use a vanished 60k conversion shortfall as a persistent defect, or
cancel a separate static-draw/causal debt because conversion improved.
The board cluster at 4.11b may change these inputs further.
4.11b.18 refreshes only affected evidence and registers a new ranking version
if necessary; 4.12.1 checks the latest artifact rather than blindly rerunning
or overwriting v2. Rank alone never accepts a family change.

| Step | Function | ref | Kind | Defect | Board | Tree | Owner note |
|---|---|---:|---|---:|---:|---:|---|
| 4.12.2 | KRPKR | 13 | scale | 0.307 | 0.0403 | 0.03462 | 4.9a.7 ported the draw branches; 30.7% overclaim remains |
| 4.12.3 | KXK | 3 | verdict | 0.020 | 0.3778 | 0.00222 | largest occurrence in the set; mechanism at 4.9a.4 |
| 4.12.4 | KRKN | 8 | scale | 1.000 | 0.0051 | 0.00000 | **100%** overclaim at +346; tree occurrence ZERO, and Rarog reaches it at 1.6x the pool rate -- 4.12.1 asks whether the defect is causing the occurrence |
| 4.12.5 | KRKB | 7 | scale | 0.996 | 0.0046 | 0.00022 | **99.6%** overclaim at +307; same over-representation as KR-KN |
| 4.12.6 | KRPKB | 14 | scale | 1.000 | 0.0045 | 0.00041 | 4.9a.8 covered rook pawns; **100%** overclaim at +328 remains |
| 4.12.7 | KBPKB | 17 | scale | 0.609 | 0.0067 | 0.00016 | 60.9% overclaim at +142; owns 4.11.9 promotion-closure debt |
| 4.12.8 | KRKP | 6 | scale | 0.264 | 0.0125 | 0.00241 | 26.4% overclaim at +72 |
| 4.12.9 | KBPKN | 19 | scale | 0.507 | 0.0029 | 0.00019 | 50.7% overclaim; owns 4.11.9 promotion-closure debt |
| 4.12.10 | KQKR | 10 | verdict | 0.230 | 0.0063 | 0.00048 | largest conversion deficit (23). RAR-M15 read it as 0 of 3,915 games and PLAN called it 'worth nothing'; 4.11.12 measured **63 of Rarog's 10,000 tournament games** |
| 4.12.11 | KPK | 5 | scale | 0.046 | 0.0241 | 0.00231 | 4.6% overclaim; present bitbase |
| 4.12.12 | KPKP | 20 | scale | 0.038 | 0.0128 | 0.00230 | 3.8% overclaim, nearly clean |
| 4.12.13 | KQKP | 9 | verdict | 0.041 | 0.0117 | 0.00687 | RAR-E08's corrected historical -3.79 pp debt; current 60k gap closes at 200k/600k; drawn subset thin |
| 4.12.14 | KBNK | 4 | verdict | 0.102 | 0.0030 | 0.00000 | corrected complete-refit dtz 0.7260 -> 0.6753; individual-term attribution unisolated; tree occurrence ZERO |
| 4.12.15 | KNNKP | 2 | scale | 0.577 | 0.0000 | 0.00000 | 57.7% overclaim; holds 7 of the 8 hard residue positions |
| 4.12.16 | KNNK | 1 | scale | 0.000 | 0.0001 | 0.00000 | **no defect measured** -- 1,499 drawn, zero claimed; close it |
| 4.12.17 | KPsK | 16 | - | n/a | 0.0452 | 0.00893 | **MEASURE FIRST** -- 4.52% of Rarog's games, never measured |
| 4.12.18 | KBPsK | 11 | - | n/a | 0.0259 | 0.00034 | **MEASURE FIRST** -- 2.59% of Rarog's games, never measured |
| 4.12.19 | KBPPKB | 18 | - | n/a | 0.0050 | 0.00015 | **MEASURE FIRST** -- 0.50% of Rarog's games, never measured |
| 4.12.20 | KQKRPs | 12 | - | n/a | 0.0042 | 0.04410 | **MEASURE FIRST** -- 0.42% of games, 4.41% of the TREE |
| 4.12.21 | KRPPKRP | 15 | - | n/a | 0.0540 | 0.05881 | **5.40% of Rarog's games and UNVERIFIABLE at 7 men.** The fourth most common family in the set cannot be adjudicated by the local tables; record it as a tooling gap, not a rare ending |

**Active workflow register.** Every family remains research until its defect,
mechanism, interacting closure and deciding instrument are established. A
measured overclaim is evidence for a question, not an implementation handoff.
The family table and closure rules above supply evidence and dependencies;
the numbered leaves below supply advance/gate conditions. GUIDE maps classes
to current models. The seven-man hold remains in force.

| Leaf | Workflow state | Class | Current decision |
|---|---|---|---|
| 4.12.1 | RESEARCH | R3 | Revalidate order, causal owners and instrument kinds after the board disposition. |
| 4.12.2 | RESEARCH | R3 | Separate KRPKR theoretical coverage from scaling; protect wins. |
| 4.12.3 | RESEARCH | R3 | Resolve KXK dispatcher/promotion-closure interactions before changing it. |
| 4.12.4 | RESEARCH | R3 | Test KRKN scale versus endogenous steering explanations. |
| 4.12.5 | RESEARCH | R3 | Separate KRKB drawn scale, conversions and occurrence correlation. |
| 4.12.6 | RESEARCH | R3 | Establish KRPKB coverage beyond existing rook-pawn branches. |
| 4.12.7 | RESEARCH | R3 | Resolve KBPKB scaling with promotion-closure debt. |
| 4.12.8 | RESEARCH | R2 | Bounded KRKP scale audit; current defect does not select a mechanism. |
| 4.12.9 | RESEARCH | R3 | Resolve KBPKN scaling with promotion-closure debt. |
| 4.12.10 | RESEARCH | R3 | Distinguish KQKR budget sensitivity, guidance and conversion debt. |
| 4.12.11 | RESEARCH | R2 | Use the KPK bitbase to bound the remaining scale/integration question. |
| 4.12.12 | RESEARCH | R2 | Confirm whether the small KPKP overclaim warrants any change. |
| 4.12.13 | RESEARCH | R3 | Reconcile KQKP historical debt, thin draws and budget closure. |
| 4.12.14 | RESEARCH | R3 | Isolate KBNK mate-gradient, rule-50 and covariance explanations. |
| 4.12.15 | RESEARCH | R3 | Explain KNNKP residue and non-monotone budget response. |
| 4.12.16 | RESEARCH | R2 | Verify the established KNNK no-defect evidence, then close `NO_CHANGE`. |
| 4.12.17 | RESEARCH | R3 | Measure KPsK coverage and choose its instrument before implementation. |
| 4.12.18 | RESEARCH | R3 | Measure KBPsK coverage and separate scale from conversion. |
| 4.12.19 | RESEARCH | R3 | Establish KBPPKB fortress boundaries and independent cohort. |
| 4.12.20 | RESEARCH | R3 | Measure KQKRPs coverage and dispatcher interaction. |
| 4.12.21 | RESEARCH | R3 | Resolve or explicitly exclude the held seven-man evidence gap. |
| 4.12.22 | RESEARCH | R3 | Families are not ready to refit/gate until their research decisions exist. |
| 4.12.23 | READY_FOR_IMPLEMENTATION | V | Dependency-held closure matrix; execute only after all family dispositions. |

1. **4.12.1 Order and classification.** Adopt the latest registered ranking
   (v2 at 4.11.12 unless 4.11b.18 supersedes it), confirm the
   recognizer/scale classification above against the code, and register which
   instrument decides each family before any of them is worked.
2. **4.12.2 through 4.12.21** are one leaf per function, in the registered
   order. Each records whether coverage is full, partial or absent, adds its
   theory/Syzygy tests, states its measurement layer and node budget, and is
   measured on the cohort its KIND calls for.
   Before implementation, its research record must distinguish: the precise
   defect question; concrete IDs/measurements already known; leading and
   credible competing causal hypotheses; dispatch/promotion/search/HCE
   interactions; the cheapest discriminating test; falsifier and stop rule;
   a frozen prospective prediction with confidence; and the exact
   `READY_FOR_IMPLEMENTATION` condition. Unknown or unmeasured fields remain
   explicitly unknown rather than being filled by reference-engine analogy.
   Use PROCESS's research-packet format only when this will not fit concisely
   in the leaf.
3. **4.12.22 Dependency-complete family refits and gates, tiered by
   occurrence.** Group mutually dependent value, scale, search-guidance and
   generic HCE terms and refit every materially covariant current parameter. Do
   not freeze historical parameters and do not SPRT each recognizer alone.
   Tier 1 (>2% of games) takes a normal no-adjudication STC SPRT; tier 2
   (0.5-2%) an endgame-start cohort; tiers 3 and 4 accept on theory, Syzygy WDL
   and DTZ progress with the whole-match run demoted to a loss-permitting
   `[-1.75, 0.25]` no-regression check, because a change confined to 0.28% of
   games cannot produce a detectable whole-match Elo at any budget this project
   has. The current 4.12.6 baseline is 100% KRP-KB overclaim at mean +328 on
   4.11.4's drawn cohort; the older 95.7%/+347 result is historical. Establish
   the cause before choosing scale, imbalance pricing or guidance, and fit
   the resulting dependency-complete family cluster here.
4. **4.12.23 Closure.** All twenty present or excluded with a recorded
   theory-backed reason, hard tests passing, aggregate floors materially
   improved against the CORRECTED baseline, and accepted families transferring
   through STC/LTC plus an explicit endgame-start cohort. Archive the exact
   harness and defects so the NNUE path does not erase classical fallback
   knowledge.

**Rule-50 damping is an open question at 4.12.14, not a known defect.**
`eval.rs` applies `score -= score * rule50 / 199` after `apply_mop_up`, so the
mate-drive override band is damped along with everything else, and the obvious
hypothesis is that this erodes the gradient below the pruning margin exactly
when the fifty-move clock makes conversion urgent. Basilisk formed the same
hypothesis and **measured the opposite**. Measure it here too; do not assume
either sign.

**A note on where the endgame knowledge goes.** Modern Stockfish removed its
endgame evaluation entirely because NNUE learned it. That argument does not
transfer to an HCE engine: the knowledge still pays for Rarog's classical
evaluator, and Phase 9 keeps it as the fallback.

### 4.13 Datagen label truth and corpus contract

**Why this sits after 4.12 and before 4.14.** The feedback loop argued above
decides it: self-play labels depend on the engine's own conversion ability, so
conversion improvements must precede regeneration, and each turn of the loop
should start from a better generator. 4.12 improves the generator, 4.13 fixes
what the generator's games are allowed to claim, and 4.14 regenerates and
refits on the result. Reversing 4.12 and 4.13 would freeze a label contract
around an evaluator that is about to change.

4.11.8 supplies the measurement; this step decides what to do about it. If
Rarog's HCE tuning uses game-RESULT labels from self-play, those labels are
sound only if the games decide won endings correctly -- and Basilisk measured
**19.77% of tablebase clean wins not won at 8,000 nodes datagen, 13.65% at
25,000**, with roughly 43% of games reaching an adjudicable clean win, so about
**8.5% of all games carried a result contradicting tablebase truth**. The bias
is ONE-DIRECTIONAL toward draws, concentrated in rook and pawn families, and it
teaches the evaluator to undervalue exactly what wins endgames. Rarog's own
share is 4.11.8's output, not an assumption.

1. **4.13.1 Data/label pipeline audit and quantification.** Apply section 2's
   audit contract to game -> sampled position -> label -> extraction -> split ->
   fit/bake. Cover score perspective/units, WDL/cursed wins, halfmove clocks,
   deduplication/by-game leakage, sampling bias, manifests/resume and exact
   parameter-to-feature coverage. Reuse existing tests and 4.11.8; derive
   additional repair leaves before corpus publication when evidence requires.
   From 4.11.8: game-level first-clean-win contradiction by family, with cursed
   wins excluded. This is **not** a row count: measure the share of extracted
   rows carrying a contradiction through game-to-row lineage here, separately
   for raw and post-hoc-TB-corrected labels.
2. **4.13.2 Two arms, registered separately.** Post-hoc relabeling of positions
   and whole-game tablebase adjudication are DIFFERENT changes and must not be
   pooled: adjudication ends the game and so changes the recorded result of
   every position sampled from it, including the openings, while a relabel
   touches only the positions themselves. RAR-E08 already adopted the relabel;
   whole-game adjudication at the table limit remains untested and removes the
   bias for one probe per game. **Analyse the halfmove-clock interaction before
   relabeling anything further.**
3. **4.13.3 Do not buy this with nodes.** Raising datagen nodes is the weak fix
   and it is measured, not assumed: 3.1x the compute bought a 31% relative
   reduction, and the fit on the higher-budget labels measured +1.00 +/- 2.11,
   stopped unresolved, with LTC +0.29 +/- 5.46. Treating that point estimate as
   an improvement is the RAR-S61 error.
4. **4.13.4 Freeze the winning contract** under a new corpus name, with the
   audit report embedded in the manifest. Never edit an existing corpus in
   place.

### 4.13a HCE implementation, interaction and throughput audit

**Analysis first; not yet performed.** Apply the section-2 subsystem audit
contract before 4.14 refits the integrated evaluator. Reuse 4.7/4.9's completed
maturity/residual work and 4.12's newly accepted family evidence; this is not
permission to reopen rejected structural expansion without its retry evidence.

Review the complete current evaluation path: material and phase interpolation;
piece-square/activity/mobility terms; pawn structure/passers and pawn cache;
king danger/safety; threats; endgame dispatch/scales and mop-up; tempo, score
normalization, rule-50 damping and saturation; eval caches and search correction.
Check term activation/units, double counting, mutually cancelled terms, clipping,
phase-boundary behavior and discontinuities after captures/promotions. Trace
the final score through qsearch stand-pat, correction, pruning margins and TT
storage; deeper search score authority remains 4.15.1's owner.

Profile full evaluations, cache hit/miss paths and repeated board/attack work
on representative and narrow activation cohorts. Compare mathematically
equivalent implementations before attributing a gain to a changed heuristic.
Investigate whether redundant maintained state pays for its consumers; do not
remove it by local appearance alone. Cross-check every production term against
EvalTrace/extraction/re-evaluation and its fitting inventory, including packed
score arithmetic, debug/release overflow boundaries and parameter invalidation.

Deliver analysis/phase4_hce_composition_audit.md with the inventory, interaction
map, profiles, evidence-backed findings and prioritized no-change/repair/
optimization dispositions. Add only the resulting implementation and test
leaves here, before 4.14; define local fits/gates for changed clusters and let
4.14 own final whole-surface consolidation. Do not retune HCE merely to disguise
a neutral refactor's correctness or speed regression. A new accepted HCE later
requires 4.15 and 4.17 to revalidate their score/confidence consumers.

### 4.14 Iterated no-adjudication refit cycles

**At least one full refit cycle on no-adjudication data is owed
unconditionally** (maintainer decision, 2026-09-01), and further cycles run
while they keep paying. This is how the maintainer's previous Texel programme
worked: fit, gate, refit, gate, each cycle returning less, stopping when the
return fell away. The screen below still runs, because it sizes the corpus and
because "no refit" was previously its possible output -- it no longer is.

The evidence for doing it at all is much stronger than "the data is older".
`hce-v2`'s own termination cross-check says **312,918 of 600,000 games (52.2%)
ended by adjudication**, and of its 313,852 decisive games **307,424 (98.0%)
were called by the resign rule while only 6,428 (2.0%) were played to mate.**
The corpus taught the evaluator that winning means +600 cp held for three
moves. It almost never showed it what converting looks like. That is the
mechanism behind the drawn-subset overconfidence in the endgame audit, and it
is not something a better optimizer on the same data can repair.

What it does **not** rest on is "the engine is stronger now, so its labels are
better". Basilisk tested that three ways and it failed all three: the same fit
on its own 8k-node outcomes measured -2.85 +/- 3.11, on its own 25k-node
outcomes **+1.00 +/- 2.11** stopped unresolved with LTC +0.29 +/- 5.46, and on
Stockfish outcomes **-7.30 +/- 4.76**, the worst arm. Evaluation models the
value realizable by its own consuming search, so a stronger player is not
automatically a better teacher. Expect the gain to come from *what the games
show*, not from *who played them*.

**Three contaminations, and regeneration only fixes two of them.**

1. **Label truncation.** `hce-v2` was generated under `datagen-v1`: 52.2% of its
   600,000 games ended by adjudication and 98% of its decisive results were
   called by the resign rule rather than played to mate. Regeneration under
   `datagen-v2`/`v3` fixes this.
2. **Position distribution.** Its positions came from games played by an
   evaluator fitted on that same truncated data. Regeneration fixes this too,
   and it is why conversion improvements precede regeneration in 4.9a -- each
   turn of the loop should start from a better generator.
3. **Initialization.** The fit starts from the current accepted vector, which
   was itself fitted on contaminated data, and **regeneration does not fix
   this**. The mechanism is explicit in the tuner: the linear gradient is
   `grad/n + 2*lambda*(w - base_w)`, so the L2 term pulls toward the STARTING
   vector, not toward zero. The nonlinear king-danger stage is integer
   coordinate descent, local by construction, and stages select a best
   validation checkpoint within a fixed epoch budget rather than converging.

   The pull looks weak in practice -- at `lambda = 1e-7` it did not stop 439 of
   1,218 slots moving in RAR-E06 or 350 in RAR-E08's arm B -- but "looks weak"
   is an impression, not a measurement.

**The initialization question can be settled OFFLINE, and cheaply.** This is
the one place a loss comparison is valid: two fits on the SAME corpus with the
SAME labels differ only in where they started, so their losses are measured
against the same target and are directly comparable. That is exactly what makes
RAR-E08 need games and this not. One control fit from a neutral start,
compared on the same frozen test, answers it for the price of one fit and zero
games.

Do not adopt a from-scratch fit as the default without that evidence. The ten
PST gauge anchors and two invariant king values exist because the surface is
not fully identifiable, so a fresh fit can land in a differently gauged place,
and the current vector encodes accepted, gate-verified structure that a restart
discards. Basilisk's +9.52 came from unfreezing PSTs inside a full-surface fit
that started from existing values, not from a restart.

1. **4.14.1 Initialization control.** Run one cycle from a neutral start
   alongside the normal one, on the same regenerated corpus and labels, and
   compare frozen-test loss. If the neutral start is not better, initialization
   carries no material bias and the loop proceeds from the accepted vector.
   Record the number either way; this closes the question rather than leaving
   it a standing doubt.

2. **4.14.2 Opening supply -- reusable, and this was previously overstated as
   a blocker.** `beast_seed.epd` holds 750,000 unique openings and all were
   used once: 1-600,000 for `hce-v2`, 600,001-750,000 for the confirmation
   set. That does **not** exhaust them. The engine has changed, so the same
   opening produces different games and different positions, and the split is
   a deterministic hash of the game's start key
   (`extract.py::split_for_key`) -- so an opening always lands in the same
   split, and regenerating from the same book cannot migrate a position from
   test into train. Reuse is not merely allowed, it is the clean option.

   Two things do still hold. Within a corpus, an opening may be used once
   (`datagen.ps1` already refuses reuse; Basilisk's 93.3%-duplicate corpus is
   why). And a *fresh* set of openings buys a genuinely independent test
   rather than one covering the same starts as the previous cycle's, which is
   a weak but real form of familiarity. Preparing new openings is therefore
   worthwhile and approved -- it is a nice-to-have, not a precondition, and it
   must not hold up cycle 1.
3. **4.14.3 Composition screen.** Generate a pilot under `datagen-v2` on a
   disjoint segment and compare composition with the matching `datagen-v1`
   archive segment: endgame-phase unique yield, coverage over the 20 reference
   classes, decisive/draw ratio, natural mate count, mean game length. Zero
   fitting. This sizes the full run and predicts which families gain support;
   it no longer decides whether the run happens.
4. **4.14.4 Regenerate and republish.** The label contract is **4.13's
   decision, not this step's** -- come here with it already settled and frozen.
   Generate the full
   corpus, re-audit provenance and content to the 4.7.1/4.7.2 standard and
   hash-freeze it under a new name. Never edit `hce-v2` in place: it is the
   corpus RAR-E06 was fitted on and has to stay reproducible.

   **Which label contract to use is RAR-E08's question, not a settled call.**
   `datagen-v3` exists and works, but do not adopt it by default before that
   experiment reports. The argument for it, and the argument against, are both
   strong. For it: removing eval adjudication does not
   by itself make labels truthful; it makes them reflect what the datagen
   engine can actually convert at 8,000 nodes. 4.9a.1 measured that at 60,000
   nodes -- KBN-K 7%, KRP-KR 52%, KBB-K 86% -- and 8,000 is worse. A
   theoretically won endgame then gets played out, drawn on the fifty-move
   rule, and recorded as a draw, mislabelling every position sampled from that
   game. That is the same defect eval adjudication was accused of, arriving
   from the opposite direction. `datagen-v3` adds Syzygy adjudication at 6 men
   (`-tb -tbpieces 6 -tbadjudicate BOTH`), which is not an opinion but the
   position's true value, and deliberately keeps the fifty-move rule so a
   cursed win is labelled the draw it really is. A 40-game probe ended 20 of
   40 games on tablebase truth, 12 of them decisively.

   Against it: Texel fits the value realizable by the **consuming search**, and
   under that principle a KBN-K position Rarog converts 7% of the time really
   is a draw. Labelling it a win teaches the evaluator to steer into endgames
   it cannot convert -- which is the same failure mode as borrowing a stronger
   engine's labels, and Basilisk priced that at **-7.30 +/- 4.76**, the worst
   arm it ran. The counter-argument is that self-play labels are
   self-reinforcing: cannot convert, so labelled a draw, so the evaluator
   learns draw, so it never steers there, so it never learns to convert.

   RAR-E08 settles it by running both. Note that its design is a **post-hoc
   relabel at extraction, not `datagen-v3` adjudication**, and that is the
   better instrument: TB adjudication ends the game and so changes the result
   of every position sampled from it, including the opening ones, while a
   relabel touches only the <=6-man positions themselves. It also needs no
   regeneration -- one game set, two label sets, perfectly paired.

   **Do not carry tablebase adjudication into a strength gate either way.** A
   gate measures realized conversion skill; adjudicating on tablebase truth
   would credit both arms equally for an endgame only one of them can win.

   **More nodes per move is NOT the answer to the same problem, and this is
   measured rather than assumed.** Basilisk ran exactly that experiment: the
   same fit on its own 8k-node outcomes measured -2.85 +/- 3.11, and on its own
   25k-node outcomes **+1.00 +/- 2.11, stopped unresolved**, with LTC
   **+0.29 +/- 5.46**. Roughly 3x the datagen compute bought a result
   indistinguishable from zero. Treating that +1.00 point estimate as an
   improvement is the RAR-S61 error -- accepting on a point estimate whose
   interval contains zero. Tablebase truth fixes the endgame-label problem for
   free; node count does not fix it at 3x the price.
5. **4.14.5 Cycle 1.** Rerun the complete 4.8 linear/nonlinear schedule on the
   new corpus and the current model, open that cycle's own frozen test once,
   bake final PGO and run the registered no-adjudication SPRT against the
   accepted head.
6. **4.14.6 Loop and stop rule, registered before cycle 1 begins.** Run another
   cycle while the previous one **accepted its gate**; stop at the first cycle
   that does not. The stop rule is the gate itself rather than an Elo
   threshold, because an Elo threshold invented mid-loop is the same act as
   moving bounds -- and because a `[0,3]` nElo gate already encodes "is this
   still worth keeping". Each cycle needs its own untouched test and its own
   registration. Cap the loop at a game budget decided before cycle 1.
7. **4.14.7 Close.** Record the cycle table -- corpus, test, fit loss, gate
   result, cumulative Elo -- so the diminishing return is visible rather than
   remembered. Repeat RAR-E09's residual/cohort audit on the final
   truth-qualified corpus/HCE, discharging 4.9.2's post-label retry trigger.
   Record a keep-closed or explicitly justified reopening; do not leave the
   historical follow-up without a disposition.

A second data cycle beyond this loop requires a prospective changed-data
hypothesis supported by the preceding fit and game verdict. More games, labels
or epochs are not a default response to a failed fit.

### 4.15 Post-HCE search composition, performance and score authority

HCE fitting can change score scale, qsearch share and pruning populations.
Basilisk's +12-Elo HCE refit moved qsearch share from 30.8% to 35.1% while most
ordering/LMR statistics held; which metrics move is engine-specific. Therefore
the old RAR-S70 counters are priors, not a candidate basis.

#### 4.15.1 Search implementation, interaction and throughput audit

**Analysis first; scope expanded, audit not yet performed.** Apply section 2's
audit contract to the complete current search, not just qsearch/TT counters.
Cover iterative deepening/aspiration/PVS, move-picker stages, all histories and
their update conditions, ordering versus pruning reuse, SEE consumers,
qsearch checks/evasions/stand-pat/delta policy, pruning, null move, reductions,
extensions/singular verification, re-search and cutoff semantics. Trace
node/ply/depth/window/bound units, excluded moves, mate/draw/terminal scores,
abort handling, TT reuse and root/worker authority through their callers.

Look for incompatible variants and jointly harmful interactions, not merely
missing named features: e.g. a history update that improves ordering but
silently increases pruning; correction affecting both evaluation and margins;
reduction/extension stacks; stale TT evidence driving a pruning decision.
These are questions to test, not findings. Profile actual search and targeted
hard cohorts; preserve narrow correctness canaries and accepted retry rules.

The existing six-part authority study below remains required. Expand its
analysis artifact with the full inventory and interaction results, then add
evidence-backed sibling work before 4.16. 4.15.2 owns a justified authority
candidate; 4.15.3–5 own production SEE scale/fit/benchmark validation. Other
demonstrated search defects need their own named leaves rather than being
smuggled into that authority candidate. Reuse 4.11b's corrected SEE/board
baseline; cache mechanics are 4.15a, thread/lifecycle contracts 4.15b.



1. Compare the accepted HCE head with exact RAR-S70 at fixed nodes/time, then
   re-run the revision-matched oracle differential at sample stride 1.
2. Profile cumulative and per-iteration nodes over a full-suite shallow/mid
   segment and a fixed representative deep segment that reaches playing depth.
   Report aggregate and per-position median/min/max. Do not infer a target from
   one endpoint, cumulative shallow ratios, absolute cross-engine node counts
   or outlier-sensitive mean depth.
3. Measure main/qsearch TT probe, hit, cutoff and store authority; qsearch entry,
   stand-pat, generated/searched/pruned move reasons; raw/corrected/pruning/
   stand-pat/searched score ownership; and explicit same-unit denominators.
4. Prove each wire and UCI option live with an absurd value. Parameter sweeps
   use a real `go nodes`/`go depth` path; `bench` is valid only after proving it
   consumes that option.
5. If the evidence reopens extension/depth authority, pair average depth at
   fixed nodes with WAC at fixed depth **and equal node cost**. Register the
   screens and how disagreement is handled before the sweep. A true mate,
   legality or termination canary can veto; conflicting aggregate WAC/depth
   results are inconclusive until per-position/equal-cost analysis resolves the
   work-per-nominal-depth confound.
6. Write `analysis/phase4_qsearch_tt_authority.md` with the dependency map and
   an explicit candidate/no-candidate decision.

#### 4.15.2 Candidate and gate, only if 4.15.1 isolates one

The design prior is a Rarog-native authority bundle: preserve exact raw HCE;
keep a separate pruning value; refine only from compatible searched evidence;
and retain qsearch stand-pat/search/store provenance. Manta MAN-S19's +13.02
nElo corroborates the question, not a formula or expected value. Basilisk's
recent contract inventory likewise shows why internal coherence and actual
consumer semantics outrank feature parity or reference constants.

Implement the smallest dependency-complete change, prove switch-off identity,
fit only a justified continuous residue and run the registered `[0,3]` PGO
SPRT. If no unique signal exists, close without code.

The Basilisk 5.7.3 defect is **not present in current Rarog**. Rarog removed its
unconditional node-level in-check extension for +30.75 Elo, so it cannot compose
with a singular double extension into Basilisk's three-ply stack; Rarog also has
no equivalent `double_ext_max` path cap. RAR-S37 already found that tightening
the singular-double margin saved nodes while eliminating doubles cost nodes,
without a strength verdict. That old alternative remains a measured null for
4.18 removal unless fresh post-HCE evidence selects it under the extension gate
above.


#### 4.15.3 The SEE value scale: audit

Reuse 4.11b.6's behavior-neutral value interface and normalized benchmark.
This leaf audits the final HCE's scale and real callers; it does not rebuild
that interface. The historical vectors and call counts below describe the
2026-09-05 audit. Refresh them from the accepted source before measuring.


**`piece_value()` has not moved since 1.0.0, and the evaluator has been refit
four times underneath it.** Three vectors sit on consecutive lines in
`src/eval.rs`:

| Constant | Values | Status |
|---|---|---|
| `MG_VAL` | 88, 394, 418, 537, 1131 | Texel-fitted, inside the 1,218-slot surface |
| `EG_VAL` | 123, 239, 290, 486, 930 | Texel-fitted |
| `PIECE_VALUES` | 100, 320, 330, 500, 900, `MATE_SCORE` | **`d3f58a2` "Version 1.0.0", 2026-05-22; never tuned** |

`PIECE_VALUES` is the textbook Chess Programming Wiki "simplified evaluation"
vector. RAR-E05, RAR-E06, RAR-E08 and RAR-E12 each moved the evaluator's own
material and left it untouched. The evaluator now prices a knight at 394 mg /
239 eg while the search prices it 320; a queen at 1131/930 against 900; and it
puts a bishop 24 mg / 51 eg above a knight where the search puts it 10 above.

**This is already owed rather than newly invented.** Operating rule 7 says that
after an HCE changes, cp-valued search consumers are audited and, if justified,
fitted separately. These are cp-valued search consumers and the audit was never
run.

**Measured blast radius**, in `src/search.rs`: **10 executable `see_ge` /
`see_ge_quiet_aware` call sites**, plus **7 direct `piece_value` uses** in move
ordering -- MVV-LVA capture scores, promotion ordering bonuses, and a qsearch
delta-pruning margin at `stand_pat + piece_value(Queen) + 200 < alpha`. That
last one is the sharpest illustration: the margin is sized on a 900-cp queen
while the evaluator's queen is worth 1131 in the middlegame, so the guard is
systematically tighter than the scale it guards.

This leaf is ZERO GAMES. Establish whether the divergence changes decisions
before proposing a change: over a fixed position set, count how often the two
scales give a DIFFERENT `see_ge` verdict at each threshold actually used, and
how often MVV-LVA order changes. A large constant offset that never flips a
verdict is not worth a gate; a small one that flips ordering constantly is.
Report the count, not an argument.

#### 4.15.4 Fit the production SEE value policy and gate, if 4.15.3 justifies it

4.11b.6 already provides neutral value injection with unchanged playing
defaults. Choose the production policy through that same interface: a separate
SEE vector, an explicitly tied HCE scale, or another measured model. Manta's
parameterized values and Basilisk's dedicated SEE table are design references,
not evidence of a winning Rarog scale. Unequal SEE and HCE values alone do not
establish a defect. Do not repeat the board-interface work here.

**The values must not be frozen by accident again.** Whatever owns them, they
become a named, tunable surface: reachable by `--rset`, listed in the tuning
inventory, and eligible for 4.16's SPSA rather than sitting as a `const` nobody
revisits. Whether they should be tied to `MG_VAL`, tapered with phase, or fitted
independently is open -- SEE is a search heuristic answering "is this exchange
losing", which is not obviously the same question as "what is this piece worth
to the evaluation".

Prove switch-off identity against the current constants, then a registered
SPRT. A repair of unknown sign wants a symmetric bracket that can detect harm,
not `[0,3]`.

**This is not a regression and must not be gated as one.** The engine has
played every accepted SPRT with these values, so current strength already
includes them. The question is whether the coupling is leaving Elo on the
table.

#### 4.15.5 Revalidate normalized SEE timing after fitting

Initial contract-value injection and comparable timing are owned by 4.11b.6.
After 4.15.4, recheck that the benchmark still uses the versioned contract
P/N/B/R/Q/K = 100/300/300/500/900/20000 while the playing engine uses its
accepted policy. Verify peer verdicts/special-move contracts, not merely equal
call counts. Preserve separately labelled native/default results and prove
the injection wire live through the harness. Do not change playing values to
improve the benchmark. If 4.15.4 closes without a change, record a validation
of the existing interface rather than manufacturing a new implementation.
Tooling remains separate from engine commits.

### 4.15a TT, caches and memory implementation audit

**Analysis first; not yet performed.** Apply section 2. Inspect full/pawn/minor/
non-pawn key semantics and all TT/eval/pawn cache consumers: replacement,
entry publication/atomicity, generation/aging, collision verification, mate
distance normalization, score bounds/depth, rule-50/repetition identity,
parameter/net invalidation, thread sharing and root/new-game resets.
Reuse 4.11b's board-key review; do not duplicate it or disable caches to obtain
a falsely clean measurement. Review alignment, entry size, cache-line traffic,
allocation, prefetching and locality on real hit/miss populations.

Deliver analysis/phase4_cache_memory_audit.md. Compare valid alternatives with
controlled within-Rarog measurements; deleting validation/state cannot count
as an optimization if it changes the contract. Expand this owner with derived
repairs/tests/gates before 4.16. Refresh 4.15's affected search evidence and
benchmark baselines after accepted changes.

### 4.15b Threading, engine lifecycle, protocol and tablebase audit

**Analysis first; not yet performed.** Inspect current supported 1T/multi-thread
behavior and producer/consumer ownership: worker startup/stop/join, shared TT
and root results, cancellation during an iteration, completed-result authority,
new-game/position/options changes, cloning/reset and resource lifetimes.
Review UCI parsing/command dispatch/output ordering, go/stop/ponder/infinite/
nodes/depth/time interactions where supported, malformed-input policy,
allocation failures and deterministic restart. Explicitly list unsupported
commands/features; this audit does not authorize adding them.

Review Syzygy FFI ownership and thread safety, availability/failure paths,
WDL/DTZ and halfmove semantics, root versus interior probing, score/bound
conversion and castling/EP assumptions. Reconcile KPK/endgame truth with the
actual board/search contracts and datagen's separate adjudication semantics.
Price synchronization, polling, cloning, initialization and probe overhead;
use deterministic interleavings/controlled integration cases where possible,
not only stress runs that happen to pass.

Deliver analysis/phase4_engine_integration_audit.md and derived leaves before
4.16/4.17. Current correctness/robustness debt stays in Phase 4; speculative
high-thread/NUMA strategies remain 8.0. The historical tournament-crash hunt
closed at 4.11.11 is not reopened by this audit; report only newly demonstrated
issues with their own evidence.

### 4.15c Instrumentation, harness and build-delivery audit

**Analysis first; not yet performed.** Review diagnostics/counter sampling and
units; benchmark corpus/anti-optimization barriers and semantic preflight;
parser completeness and process exit/timeout handling; exact feature/ISA/PGO
provenance; instrument-off identity; game pairing/seed/clock/adjudication and
registered-stop handling; reproducible artifacts and statistical interpretation.
Reuse 4.10's repairs/tests, 4.11b.2 and 4.13's data lineage audit. Verify the
current end-to-end wiring instead of rerunning calibration without a changed
contract. Test constructs/behavior, not whether comments mention a safeguard.

Cover build configuration and supported dispatch paths, unsafe/FFI assumptions,
configuration persistence, option reachability, and shipped-versus-diagnostic
feature separation. Price instrumentation only in its diagnostic context,
then measure production with it off. Deliver
analysis/phase4_instrument_build_audit.md; add demonstrated repair/verification
leaves here before final search fitting or new strength claims rely on those
instruments. Earlier gates still owe their existing live-wire/provenance
checks; this later systematic audit does not waive those prerequisites.
A discovered invalid measurement suspends its dependent gate and corrects the
current record in place; historical values remain visibly superseded.

### 4.16 Optional post-HCE search SPSA

Open only if several live cp-valued RFP, null, futility, ProbCut, qsearch,
correction or LMR coordinates show a displaced interacting optimum. **The SEE
value vector joins this surface if 4.15.4 justifies a tunable production
policy**. Reuse 4.11b.6's interface; benchmark injection alone does not justify
SPSA. First run
a registered bounded sensitivity pilot, then audit the entire active
interacting surface. Pilot theta is neither candidate nor seed; the full tune
starts from accepted defaults and preserves its registered horizon under any
staged `StopAfter`. Never mix HCE and search coordinates.
Re-evaluate sensitivity after 4.15/4.15a–c's accepted changes; tune the live
interacting cluster when justified, not each internal leaf separately. If
SPSA is skipped, record its measured reason and preserve accepted defaults.

### 4.17 Time management — review, repair and gate

**This step owns all time-management work in Phase 4.** TM had no owner: its
findings were scattered across the ledger, RAR-X06's owner cell still pointed
at 4.9 (which is now HCE structure), and RAR-S47 left `RootConfTime` shipping
ON with six untuned consumers and nobody named. Anything touching the clock
enters here.

**Why here.** TM consumes root scores and confidence signals, and 4.8 just
changed the score scale those signals are expressed in. Measuring TM before
the accepted HCE would price a surface that no longer exists. It sits after
4.15's authority work, and before 4.18's cleanup and the 4.20 release gate,
so a clock change cannot arrive after the checkpoint that is supposed to
describe it.

1. **4.17.1 Time-management implementation and interaction audit.** Apply
   section 2 before the existing measurements/changes below. Trace clock units,
   budget allocation, overhead/reserve, soft/hard stop, node checks, completed
   root scores, aspiration/re-search instability, worker results and UCI timing.
   Measure compatibility with the accepted search/HCE and 4.15b's lifecycle,
   including changing thread counts and cancellation near deadlines. Separate
   deadline correctness from the strength policy deciding how long to think.
   Expand derived work here; avoid duplicate TM owners elsewhere.
   RAR-R01's +81 Elo and
   RAR-R02's `2*MoveOverhead` reserve were measured on the old harness and the
   pre-refit evaluator. The direction is retained; the magnitudes are not
   current priors. Re-measure on the accepted HCE before changing anything.
2. **4.17.2 Forfeit margin.** From RAR-M14: sweep `Move Overhead` against
   forfeit rate on a null pair. The background rate is ~0.08-0.17%, so
   distinguishing two values needs tens of thousands of games -- size it
   before running. `PROCESS.md` prices ~10 forfeits per 3,000 games at ~1 Elo,
   so at the observed rate the entire prize is ~0.2 Elo. This is tournament
   robustness, not a strength lever, and RAR-E06's three forfeits were all in
   positions already lost by 5 to 9 pawns. The specific gap to close is that
   `time_manager.rs` gates its 30ms `smp_reserve` on `threads > 1`, leaving a
   single-threaded engine under a saturated runner with only `2*overhead`.
3. **4.17.3 `RootConfTime` consumers.** RAR-S47 shipped the completed-root
   confidence clock ON after sizing it to level-neutrality (+0.09% total
   budget, longer on 295 iterations and shorter on 182). Its six identifiable
   consumers were never tuned. Tune them or remove the path; an inert
   mechanism with no owner is 4.18 material.
4. **4.17.4 Root-instability TM.** RAR-X06 reverified +6.46 +/- 4.12 in the
   reference engine while Rarog's own raw pool-view variant lost 5.54
   (RAR-R05). It may therefore enter only as one bounded input to a completed
   authoritative root snapshot, never as a direct multiplier. Retargeted here
   from 4.9.
5. **4.17.5 Gate.** One registered SPRT for the dependency-complete clock
   change. **Zero forfeits is a precondition, not the verdict** -- RAR-S54 and
   RAR-S57 both passed with zero forfeits while changing node counts by +23%
   and +5%, so a clean forfeit count proves only that the change is safe to
   measure. Never accept a TM change on a forfeit count alone.

### 4.18 Search cleanup and checkpoint

- **4.18.1 Coverage closure and dead-mechanism inventory.** Reconcile the
  current source/module/feature inventory against section 2's ownership table.
  Every subsystem and cross-boundary contract must have an audit/disposition;
  add a named analysis owner for anything omitted, before its dependent gate.
  Review dead/unreachable paths, retained options and observability, and confirm
  required audit findings are implemented/verified or explicitly held with a
  closure boundary. Future-only owners do not waive current correctness.
  Do not create a new candidate from an old rejected experiment without its
  retry evidence. Existing inventory: Basilisk-derived. It
  found history pruning nearly unreachable, and `double_ext_max` never binding
  even when cut from 200 to 16. A dead mechanism is an anomaly to explain, not
  automatically headroom: measure the population first, then either remove the
  safeguard or redesign it under this step. Report reachability for every
  retained switch in the §3 table.
- **4.18.2 Removal.** Remove every unconsumed 4.6 and retained default-off
  alternative without a future owner. Preserve only diagnostics with a named
  Phase-5/7 owner.
- **4.18.3 Checkpoint.** Re-run debug/release tests, all-feature/all-target
  clippy, exact benchmark, pooled-PGO NPS, fixed-time/fixed-node deficits and
  the accepted 4.15/4.16 game verdicts.

### 4.19 Final HCE/search checkpoint

Compare final head with exact RAR-S70 using revision-matched final-PGO binaries
and no adjudication. Record separately attributed HCE and post-HCE-search Elo,
NPS, fixed-node behavior, STC and LTC direction. Ablate surprising integrated
contributors and close every maturity classification.

The HCE is mature for this release only when:

- the current-source family map contains no unknown or first-draft row;
- every accepted representation reconstructs through `EvalTrace` and has
  activation/covariance plus a game verdict;
- every real parameter slot has a named, verified fitting instrument or a
  written invariant/gauge/unidentifiable disposition;
- the complete existing HCE refit and any post-structure consolidation have
  clean game verdicts;
- optional HCE/search SPSA is completed and gated or explicitly skipped;
- the fitted HCE remains a tested fallback and suitable datagen baseline.

### 4.20 Transfer, portability, SMP and release gate

1. Compare final head directly with 2.3.2 at STC, LTC `10+0.1` and 4T.
2. Record pooled-PGO NPS, benchmark, UCI, correctness, platform and ISA matrix.
3. Drop `-use-affinity` for multi-thread cells and calibrate a null pair under
   that topology.
4. Run a final no-adjudication target cohort including Basilisk and the oracle
   as diagnostic reference points.
5. Remove diagnostic scaffolding without a future owner; retain the ablation
   instrument and frozen oracle branches.

#### Release rule

- 2.4.0 requires cumulative STC point estimate at least **+40 Elo** over 2.3.2,
  95% lower bound above **+25 Elo**, positive LTC and 4T lower bounds, and all
  release gates.
- A cumulative result at or above +100 Elo with lower bound above +75 may
  justify a higher minor version by maintainer decision.
- Below the bar, ship 2.3.x only by explicit decision or close Phase 4 without
  a release. NNUE follows either way.

### 4.21 Universal binary investigation and qualification

**Investigation and testing, not automatic production adoption.** This is the
last Phase-4 step, after 4.20, with existing numbers preserved. It does not
retroactively block the 4.20 HCE release or change its accepted artifact.
Any later adoption changes the delivered executable and must repeat affected
4.20 release checks; an experimental binary cannot inherit that qualification.
No implementation or benchmark has been performed for this step yet.

Goal: determine whether one release executable per supported OS/processor
family can select the appropriate implementation at startup, removing normal
users' need to choose `base`/`avx2`/`pext`. Preserve explicit tier selection
internally for reproducible tests, diagnosis and native builds. OS/target
triples are distinct from CPU-feature tiers; do not promise one file for every
OS and processor family. Do not add NNUE-only runtime costs to the HCE board.

1. **4.21.1 Freeze reference behavior and Rarog constraints.** Pin the examined
   Stockfish source and inspect `src/universal/entry_x86.cpp`, its universal
   build/link rules, initialization and shared network embedding. Reference:
   [Stockfish 19 announcement](https://stockfishchess.org/blog/2026/stockfish-19/).
   Map Rarog's `xtask/src/main.rs` tiers, `rarog_pext` attack-table layouts,
   `src/main.rs`'s failed historical CPU guard, `build.rs`/Fathom, PGO, linking
   and `verify-isa`. Reuse 4.15c's evidence; investigate only changed contracts.
2. **4.21.2 Compare designs and specify dispatch.** Compare baseline startup
   selecting complete optimized engine variants with function-level dispatch
   for selected kernels. Explain symbol/global-state/FFI isolation, startup
   initialization, table layout, inlining and future NNUE ownership. Do not
   assume `target_feature` on an entry function specializes its callees.
   Define required CPU AND OS feature checks, slow-PEXT CPU policy, baseline
   fallback, selected-tier reporting and a test override that rejects unsupported
   tiers. The dispatcher and anything executed before selection must be built
   for the baseline; global native/AVX flags invalidate that safety argument.
3. **4.21.3 Build a bounded prototype.** Implement only the shortlisted design
   in an isolated experiment using existing build/measurement tools. Keep
   production defaults and normal release artifacts unchanged. Start with
   current HCE tiers; do not invent AVX-512/NNUE variants without useful work
   for them. Record exact source, compiler, features, PGO recipe and binary
   hashes. Derive implementation sub-leaves if the design needs substantial
   work rather than hiding a broad rewrite inside this prototype.
4. **4.21.4 Verify compatibility and chess identity.** Test each forced tier
   on a supporting CPU and automatic selection against the explicit policy;
   include baseline-only, fast-PEXT, slow-PEXT and OS-disabled-vector cases.
   Synthetic detector tests do not prove executable compatibility: use suitable
   hardware or ISA-constrained execution and label untested cells as gaps.
   Cover startup/static initialization, vendored Fathom, UCI/stop/thread lifecycle,
   debug/release tests, perft and targeted board/SEE transitions. Require exact
   accepted production bench identity per tier plus targeted parity. Adapt ISA
   verification to baseline entry paths and specialized code regions: a whole-
   binary prohibition on optional instructions is wrong for a universal binary.
5. **4.21.5 Measure cost against separate binaries.** Before running, freeze a
   minimal representative CPU/workload matrix and performance acceptance bounds.
   Compare equivalent PGO/features, fixed-node whole-search NPS, relevant board
   workloads, startup, binary size and memory. Alternate paired runs and record
   scatter; CPU feature availability alone does not identify the fastest tier.
   Use target-native performance evidence, not emulator timing. Preserve missing
   hardware tests as explicit holds; no blanket portability/performance claim.
6. **4.21.6 Decide adoption and own remaining work.** Deliver
   `analysis/universal_binary_feasibility.md` with the design, raw evidence,
   acceptance matrix, costs and an adopt/defer/reject recommendation. Retire
   user-facing architecture selection only after compatibility, parity and
   performance qualification; retain diagnostic overrides and rollback assets.
   Adoption must receive numbered implementation/release leaves and the affected
   4.20 checks before shipping. Hand future NNUE kernel qualification to 6.4.2
   and remaining dispatch/platform work to 8.1. Neither future owner repeats
   completed HCE investigation without a changed dependency.

## 5. Phase 5 — NNUE runway

Phase 5 creates the behavior-neutral runway for NNUE. HCE stays active and the
accepted Phase-4 fingerprint must survive every step. Reuse the accepted
4.11b board, its correctness tests and existing search context. Endgame
development remains in 4.12 before 4.14's whole-HCE consolidation.

- **5.1 Measurement corpus handoff.** Freeze the accepted 4.7 corpus and
  manifests as the NNUE residual/stage-gate source. Add only NNUE-specific
  labels or scale; preserve untouched splits.
- **5.2 Per-ply state and dirty pieces.** Preserve the board's chess contract
  while exposing the facts an incremental evaluator needs.
    - **5.2.1 Freeze and interpret the board-cost baseline.** Carry RAR-M20's
      [three-engine measurements](analysis/board_audit_2026-09-05.md) and
      [complete recipe](analysis/board_benchmark_recipe_2026-09-05.md) into
      enablement; preserve the raw rounds, hashes, ISA/build flags and full
      Reckless adapter patch. First measure the accepted final Phase-4 Rarog
      using both frozen v1 and 4.11b's extended coverage plus whole-HCE search.
      Keep the 2026-09-05 result as history, not the denominator for a changed
      toolchain/board/corpus. Rarog beat Reckless in all five comparable
      workloads, but Reckless used NullBoardObserver: NNUE arithmetic was
      absent, while native threats/pins/checking squares, repetition state
      and scored move lists remained. The gap therefore does NOT measure
      accumulator or inference cost, and does not isolate the cost of
      NNUE-related bookkeeping. Treat that explanation as a hypothesis.
      Start the staged cost ledger described in the audit; retain useful
      state until a controlled comparison prices its actual consumers.
    - **5.2.2 Define the factual move delta.** Describe piece removals,
      additions and relocations, including captured color/type/square,
      promotion identity change, EP's off-destination victim, both castling
      pieces and explicit null semantics. This record contains chess facts,
      not HCE weights or NNUE feature indices. Document whether each callback
      sees the before, intermediate or final board.
    - **5.2.3 Implement the update interface and ownership.** Choose a typed
      returned delta or statically dispatched observer with a no-op HCE path.
      Reckless BoardObserver/on_piece_change/on_piece_move/on_piece_mutate
      and Stockfish dirty-piece updates are reference seams. Preserve
      per-worker board/history ownership, clone/reset and arbitrary setup
      history; keep large evaluator state outside Board.
    - **5.2.4 Verify every transition independently.** Reconstruct the
      delta's final mailbox from its before-state and compare every legal
      move, including all special moves. Random branching, null roundtrips,
      complete unwind and cloned workers must reproduce every board/key and
      attack fact. Add explicit maximum-history/search-depth boundaries.
    - **5.2.5 Qualify the HCE path and price move events.** Compare the 5.2.1
      baseline with the same board plus factual events/no-op observation.
      Retain accepted fingerprint, targeted behavior and supported-ISA
      identities. Measure bookkeeping in the corrected board suite and
      pooled-PGO HCE search, with per-cohort costs and layout/allocation counts.
      Prove the event wire is live with an observing sink outside timing.
      Do not remove fields with real consumers or tune HCE to conceal a cost.
- **5.3 Accumulator scaffolding.** HCE remains active; actual inference is 6.3
  and incremental/SIMD implementation is 6.4.
    - **5.3.1 Allocate per-thread, per-ply evaluator state.** Keep accumulator
      arrays in evaluator-owned storage, with stack indices tied explicitly
      to make/unmake and search abort. Do not embed or copy a network-sized
      accumulator in every Board or undo record. Follow Reckless
      Network::push/pop as an ownership example, not its network dimensions.
    - **5.3.2 Specify validity and refresh.** Track each perspective's valid
      state, dirty pieces and refresh reason; define king bucket/mirroring and
      future output-bucket changes. Null changes side to move, not piece
      placement: handle evaluator perspective/metadata explicitly rather than
      inventing a piece delta. Reserve a king-bucket refresh cache with a
      documented invalidation contract.
    - **5.3.3 Verify the scaffolding against full refresh.** Use deterministic
      test features/reference vectors to check lazy update, branch reuse,
      king moves across bucket/mirror boundaries, special moves, reset and
      worker isolation. Actual trained-network integer parity is still
      required at 6.3/6.4; scaffold tests cannot discharge it.
    - **5.3.4 Gate the runway cost and behavior.** Extend 5.2.1's cost ledger
      with event-only versus event-plus-stack/dirty/validity bookkeeping,
      still with the same HCE. Record bytes per ply/thread, capacity, touched
      bytes, allocation events and refresh reasons; prove counters live and
      measure the uninstrumented builds. Debug/release, randomized unwind,
      exact HCE fingerprint, targeted cases and revision-matched pooled-PGO
      HCE NPS must pass. Archive the delta/refresh contract for the trainer and
      engine implementers. No numerical loss budget is invented after timing.
- **5.4 Trainer preflight.** Pin `D:/code/net_trainer`, Bullet, toolchain and
  GPU; verify conversion, shuffle, splits, manifests, reference vectors and
  resume semantics. Apply the subsystem audit contract to trainer data flow,
  feature ordering, quantization/export and engine fallback compatibility;
  6.0/6.3 own derived trainer hardening and actual-network conformance.
- **5.5 Runway gate.** Exact benchmark, debug/release tests, randomized unwind,
  reproducible pilot corpus and trainer conformance. Carry the staged board
  cost ledger into Phase 6; its costs are measured, not presumed zero.
- **5.6 Threat-map hooks, optional.** Decide from the selected network's actual
  relation features whether piece events need a before/intermediate/final
  board view or extra attack-transition data. Preserve an extension seam for
  discovered attacks and removed blockers. A simple piece-square NNUE does
  not justify eager full threat tensors. Reckless's threat-accumulator
  observers are a reference if relation inputs are selected; useful board
  pin/check caches remain independent. Record a no-change disposition if no
  additional hook is needed. Attribute any new maintenance cost separately
  in the 5.2.1 ledger, using semantically valid eager/lazy implementations.

Boundary rule: search consumes an evaluator score and evidence class, never
evaluator internals.


## 6. Phase 6 — baseline NNUE

- **6.0 Trainer hardening.** Strict CLI, deterministic splits, hashes, seeds,
  checkpoint selection and exact references.
- **6.1 Controlled data.** Generate 30–60M unique positions with by-game
  splits, deduplication, external and tablebase cohorts, manifest provenance
  and a validated score/result blend.
- **6.2 Baseline networks.** At least two seeds for documented widths and
  buckets; validation selects, untouched cohorts report once.
- **6.3 Scalar integration.** Implement the documented `quantised.bin`
  contract with integer-exact engine/NumPy/reference conformance and clean HCE
  fallback.
- **6.4 Incremental and SIMD.** Reuse 5.2/5.3's state contract and staged cost
  ledger; a network changes the workload, so the 2026-09-05 cross-engine gap
  cannot be subtracted to claim an NNUE cost.
    - **6.4.1 Prove actual-network incremental parity.** Compare every
      incremental result with scalar full refresh on the same trained net.
      Cover captures, EP, all promotions, castling, null, king bucket/mirror
      boundaries, reset/abort, cloned workers and branch reuse. Include
      threat/relation features only when this architecture consumes them.
    - **6.4.2 Qualify SIMD and supported targets.** Verify integer bounds,
      portable/x86/ARM bit identity, debug/release and the supported ISA matrix.
      Keep an independently computed scalar reference; SIMD-versus-SIMD
      agreement is insufficient.
    - **6.4.3 Attribute and gate the NNUE board/evaluator cost.** Extend the
      5.2.1 ledger with scalar full refresh versus lazy incremental versus
      SIMD on the SAME net and fixed legal move traces. Measure move events,
      state maintenance, feature updates, bucket refreshes and inference
      separately, then combined, with refresh rates and memory per worker.
      Compare HCE/NNUE board work using identical prerecorded transitions;
      whole-search HCE/NNUE NPS also changes tree/eval behavior and is a system
      result, not a causal board-cost estimate. Include quiets, captures, EP,
      promotions, castling, king bucket changes, null and sparse-endgame
      cohorts. Re-run frozen v1 and extended profiles, optionally Reckless
      with both no-op and real observers at its recorded network/version.
      Keep native SEE unranked unless values/contracts are normalized.
      Require exact same-net score parity and pooled-PGO whole-search NPS;
      retain the raw stage data for later architecture comparisons. The
      original RAR-M20 result is a reference, never an acceptance target.

- **6.5 Architecture loop.** Controlled data-versus-capacity experiments,
  progressing from output buckets to mirrored king buckets and then justified
  relation/multilayer inputs.
- **6.6 Gross search-scale safety.** Repair only clearly invalid scale/margins;
  broad search fitting waits for 7.3.
- **6.7 Baseline release.** Beat the accepted pre-NNUE master at STC/LTC,
  transfer at 4T, pass platform gates and archive every accepted net with its
  reproducible training manifest.

## 7. Phase 7 — NNUE frontier and final search fit

- **7.0** Residual and disagreement analysis by phase, material, king,
  tactical/endgame cohort, calibration and refresh cost.
- **7.1** Data frontier: scale, deduplicate, mine hard positions and refresh
  on-policy data only when a clearly stronger net changes the policy.
- **7.2** Architecture ladder: king/material/threat/pawn relation inputs,
  width/activation and refresh-friendly variants, one axis at a time.
- **7.3** Re-audit search/evaluator interactions after NNUE: score scale,
  correction/caches, pruning/margins, qsearch/SEE and root-confidence consumers.
  Reuse Phase-4 maps but revalidate the new evaluator's actual contracts.
  Expand demonstrated repairs into leaves before one justified post-NNUE search
  fit over displaced live coordinates, then PGO, SPRT, LTC and 4T.
- **7.4** Frontier gate against 2.3.2, the Phase-4 head and target engines.

## 8. Phase 8 — scaling, platforms and product completeness

- **8.0 High-thread and NUMA.** Price the measured depth-diversity deficit at
  4/8/16T; test helper depth/ordering/TT ownership and retained SMP switches.
- **8.1 Runtime dispatch and memory.** Reuse 4.21's universal-binary decision
  and evidence; own deferred adoption and later platform/NNUE dispatch extensions,
  alongside TT/net placement and large pages. Qualify changed architectures with
  target-native evidence; do not repeat the completed HCE investigation by default.
- **8.2 Product/platform.** Demand-led Chess960 and platform work; consider
  OpenBench-style distributed testing when typical gains reach 1–3 Elo.
- **8.3 Scaling release.** Full topology, clock, net, ISA and user-doc gate.

## 9. Optional post-NNUE classical fallback

Enter only if a serious king-conditioned NNUE, inference optimization and
data-scale retry fail and the maintainer explicitly abandons NNUE. Reuse the
4.7 residual corpus. Any family accepted in 4.9 or 4.9a is closed here.

- **9.1** King-safety semantic rework.
- **9.2** Material-specific winnability and scaling.
- **9.3** Passer/pawn conditionality.
- **9.4** Threat and usable-activity conditionality.
- **9.5** Material/phase specialization only as a last classical step.

Every fallback item is structure plus fit plus one gate, not additive term
accretion.

## 10. Release checklist

- [ ] Version, README, CHANGELOG and release notes agree.
- [ ] `cargo fmt --check` passes.
- [ ] Workspace/all-target tests pass in debug and release.
- [ ] All-feature/all-target clippy passes with zero warnings.
- [ ] Feature builds and tune-option inventory are correct.
- [ ] Benchmark fingerprint is recorded and every move explained.
- [ ] Local PGO asset passes UCI, benchmark and ISA verification.
- [ ] Prior-release STC/LTC and 4T direction pass the release rule.
- [ ] Hosted platform/CI matrix passes on the release commit.
- [ ] Commit locally; tag, push and publish only on maintainer instruction.

## 11. Documentation ownership

`GUIDE.md` is the short operator entry point: workflow boundaries, current
model mapping, reusable research/implementation prompts, every phase/step
checkbox, checkpoint and next action. Detailed rationale and records belong in
the files below. GUIDE lost Phases 6-9
during a shortening pass on 2026-08-30 and endgame work was filed under the
NNUE runway; `tools/diag/check_guide.py` now fails when a phase heading is
missing, so that class of drift is caught mechanically rather than by reading.

| File | Purpose |
|---|---|
| `GUIDE.md` | Short operator guide, model mapping, prompts, status board and next action |
| `PLAN.md` | Forward roadmap, workflow state, dependencies, readiness and deciding gates |
| `EXPERIMENTS.md` | Frozen predictions, measured evidence, calibration, retry triggers and recipes |
| `PROCESS.md` | Research/handoff template and recurring build, fit, gate and release procedures |
| `TRACKER.md` | History only; never a source of the next step |
| `analysis/hce_maturity_2026-08-25.md` | HCE/Stockfish maturity comparison and fitting policy |
| `analysis/hce_archive_audit_2026-08-31.md` | Archive provenance, content, capacity and quota |
| `analysis/endgame_conversion_audit_2026-09-01.md` | Conversion rates, 20-function inventory, defects, test policy |
| `analysis/basilisk_audit_2026-08-30.md` | Basilisk method/results audit and Rarog consequences |
| `analysis/manta_tooling_audit_2026-08-25.md` | Manta tool dispositions and imported measurements |
| `analysis/ablation_results.md` | Search-deficit measurements and corrected interpretation |
| `analysis/board_audit_2026-09-05.md` | Board defects, comparison, owner map and staged NNUE cost contract |
| `analysis/board_benchmark_recipe_2026-09-05.md` | Complete benchmark recipe, adapter patch, manifest and raw rounds |
| `analysis/artifacts/board-audit-20260905/` | Machine-readable native board baseline retained for 5.2.1 and 6.4.3 |

`GUIDE.md` and `PLAN.md` change in the same commit. Source, defaults and
reproducible artifacts outrank prose whenever documents disagree.

## 12. Reference tools and commands

| Tool / path | Purpose |
|---|---|
| `tools/sprt.ps1` | Paired pentanomial GSPRT; default 1T `3+0.03`, Hash 64, UHO |
| `tools/diag/phase4_differential.py` | Same-unit Phase-4 suite aggregation |
| `tools/diag/bench_counters.py` | Sum all per-position bench counter dumps |
| `tools/diag/endgame_truth.py` | Per-move Syzygy grading; corrected termination and cohort contract verified at 4.10 |
| `tools/diag/endgame_floors.py` | Cohort-stamped ratcheting floors; corrected baseline at 4.11.2 |
| `tools/diag/endgame_drawn.py` | Drawn-cohort overclaim; static, plays nothing, unaffected by 4.10.1 |
| `tools/diag/endgame_book.py` | Syzygy-verified endgame-start cohort |
| `tools/diag/endgame_search_occurrence.py` | Family frequency in the search tree; exclude endgame roots (4.11.5) |
| `tools/diag/datagen_label_audit.py` | Corpus labels against tablebase truth; new at 4.10.8 |
| `tools/diag/check_guide.py` | GUIDE/PLAN status-board consistency; enforces SUPERSEDED owners at 4.10.10 |
| `tools/branching_profile.ps1` | Hash-bound per-position and per-iteration depth/branching shape with robust aggregates; refutation evidence only |
| `tools/pgn_result.ps1` | Reconstruct complete-pair PGN results |
| `tools/build_test.ps1` | Hash-bound build manifests and exact benchmark qualification |
| `tools/spsa.ps1` | Registered targeted SPSA with immutable horizon and staged stop |
| `tools/texel/extract.py`, `extract_parallel.py` | Leak-resistant three-way phase-balanced extraction |
| `cargo xtask build --arch <arch> --pgo` | Production PGO asset |
| `cargo xtask verify-isa --arch <arch>` | Executable ISA contract |
| `hybrid/build.ps1` | Frozen diagnostic oracle package, hybrid branch only |
| `D:/code/net_trainer` | Phase-6 NNUE data/training stack |

```powershell
cargo fmt --check
cargo test --workspace --all-targets
cargo test --workspace --all-targets --release
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo build --release
'bench 13' | .\target\release\rarog.exe
```

## 13. Historical number map

Phase 4's OPEN work was reordered on 2026-09-04 so that instrument repair
precedes re-measurement and re-measurement precedes development. Completed
steps keep their numbers, so every reference in `EXPERIMENTS.md`, `TRACKER.md`
and git history stays valid; use this map rather than rewriting historical
evidence.

| Historical | Current | Note |
|---|---|---|
| 4.9a.1 - 4.9a.8 | unchanged | completed; five results SUPERSEDED, owners named |
| 4.9a.7 (KRPKR) | also 4.12.2 | leaf reopened for its conversion half |
| 4.9a.8 (KRPKB) | also 4.12.6 | leaf reopened for its conversion half |
| 4.9a.9 - 4.9a.26, plus 4.9a.7/.8 | 4.12.2 - 4.12.21, by function | twenty function owners; reordered by evidence, never translate by numeric offset |
| 4.9a.14 (KQKP) | 4.12.13 | owns RAR-E08's corrected historical KQ-KP debt; not persistent at 200k/600k |
| 4.9a.26 (KBNK) | 4.12.14 | owns corrected complete-refit DTZ regression; individual-term attribution unisolated |
| 4.9a.27 | 4.12.22 | dependency-complete family gates |
| 4.9a.28 | 4.12.23 | endgame closure |
| 4.10 (refit cycles) | 4.14 | sub-steps 4.10.0-4.10.6 become 4.14.1-4.14.7 |
| 4.11 (authority) | 4.15 | |
| 4.12 (search SPSA) | 4.16 | |
| 4.12a (time management) | 4.17 | |
| 4.13 (search cleanup) | 4.18 | |
| 4.14 (final checkpoint) | 4.19 | |
| 4.15 (release gate) | 4.20 | |
| -- | 4.10, 4.11, 4.13 | new: instruments, re-measurement, label truth |

Reordered again on 2026-09-05, inside 4.11 only, because the re-ranking was
placed before two of its own inputs:

| Historical | Current |
|---|---|
| 4.11.4 re-rank 4.12 | 4.11.6 |
| 4.11.5 budget transfer | 4.11.7 |
| 4.11.6 occurrence census | 4.11.5 |
| 4.11.7 drawn-share bias | 4.11.4 |

Board audit integration, 2026-09-05: **4.11b** is new and sits between the
remaining 4.11 work and 4.12. Existing 4.11/4.12 numbers and v2 rank order are
preserved. Initial SEE benchmark restoration moves from 4.15.5 to **4.11b.6**;
4.15.3–4.15.4 retain post-final-HCE scale audit/fitting and 4.15.5 revalidates.
Phase 5.2 owns the historical board baseline and event-cost attribution;
5.3 owns scaffold costs; 6.4 owns same-network incremental/SIMD attribution.

Subsystem-audit planning update, 2026-09-05: no existing step was renumbered or
removed. 4.11.7 was held in place at that update; the maintainer subsequently
authorized its compute run, completed as RAR-M21 on 2026-09-06. 4.11.8 later
completed as RAR-M22 and 4.11.9 as RAR-M23 on 2026-09-06. 4.11.10 is next.
New analysis owners are 4.13a (HCE) and 4.15a–c (storage, integration and
instruments); existing search, data, TM, NNUE and closure owners are expanded.
Their implementation substeps will be derived from the audits, not invented
before the evidence. 4.11b remains before 4.12; the older handoff omitting it
is superseded as a next-work instruction, not a source of roadmap authority.
