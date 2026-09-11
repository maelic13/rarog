# Phase-4 differential counter specification

Status: **contract**, written at PLAN step 4.1, consumed by 4.1 (oracle side)
and 4.2 (Rarog side). Both implementations must satisfy this document rather
than each other, so a disagreement is resolved here, not in whichever code was
written second.

## Why this file exists before either implementation

"Matched name for name" is meaningless without matched *definitions*. RAR-S25
recorded the failure mode directly: a rate is only meaningful when numerator and
denominator are collected at the same point in the code. Two engines emitting a
counter called `lmr_applied` from structurally different places produce a number
that looks comparable and is not, and the whole value of 4.2 is that its
divergences are trustworthy enough to choose the next cluster.

So each counter below fixes: what increments it, exactly where, and which
denominator it is read against.

## Output contract

Both sides emit, at search completion, one line per counter:

```text
info string diag <name> <value>
```

Names are `lower_snake_case` and are the literal label. Counters reset once per
`go`, on the main thread, before any helper is spawned — Rarog has already been
bitten by resetting per-thread, which silently wiped every earlier thread's
contribution and made every multi-thread diagnostic number junk. Values are
process-global; the differential suite runs one thread, but the reset rule holds
regardless.

Diagnostics are compile-time gated on both sides. With them **off**, Rarog must
reproduce `bench 13` = 6,519,711 / EBF 2.449 exactly, and the oracle must
reproduce its frozen `75d0d43` behaviour. The instrumented oracle build is a
diagnostic artifact: it never plays a rating game and never replaces the frozen
tournament binary.

## Comparability tiers

Every counter is in exactly one tier. Mixing them is the error this section
exists to prevent.

| Tier | Meaning | Use |
|---|---|---|
| **Core** | Same mechanism, same definition, both engines | Divergence selects work |
| **Rarog-only** | No analogue in `9587eeeb` | Describes Rarog; never a gap |
| **Oracle-only** | Mechanism Rarog does not have | Candidate idea; never a target |

A Rarog-only counter reading differently from nothing is not a finding. An
oracle-only mechanism existing is a *question for 4.3*, not a defect.

## Core counters

`SF step` refers to the numbered `// Step N.` comments in the vendored
`search.cpp`, which are that file's own structural skeleton.

### Group 0 — denominators

Instrument these first. Every rate below names the denominator it divides by,
and both must be collected at the same point.

| Name | Definition | Rarog site | SF site |
|---|---|---|---|
| `nodes` | Interior (non-qsearch) nodes entered | `search.rs` negamax entry | `search<>()` after Step 1 |
| `qnodes` | Quiescence nodes entered | **gap, see below** | `qsearch<>()` entry |
| `nodes_in_check` | Subset of `nodes` where side to move is in check | negamax entry | `ss->inCheck` at entry |

### Group 1 — move ordering and cutoffs (cluster 4.5)

Denominator: `cutoff_quiet + cutoff_capture`, which is every beta cutoff at a
real, non-excluded interior node.

| Name | Definition | SF step |
|---|---|---|
| `cutoff_quiet` | Beta cutoff whose move was quiet | 19 |
| `cutoff_capture` | Beta cutoff whose move was a capture or promotion | 19 |
| `cutoff_first_move` | Beta cutoff at move index 1 | 19 |
| `best_rank_1` | Cutoff at move index 1 | 19 |
| `best_rank_2_3` | Cutoff at move index 2–3 | 19 |
| `best_rank_4_7` | Cutoff at move index 4–7 | 19 |
| `best_rank_8_plus` | Cutoff at move index ≥ 8 | 19 |
| `move_seen_tt` | Move supplied by the picker as the TT move | movepick |
| `move_seen_good_capture` | Move supplied as a good capture | movepick |
| `move_seen_quiet` | Move supplied as a quiet | movepick |
| `move_seen_bad_capture` | Move supplied as a deferred bad capture | movepick |

`best_rank_*` and `cutoff_first_move` deliberately overlap: `best_rank_1` must
equal `cutoff_first_move`, which is a free cross-check that both sides agree on
what a move index is. If they disagree, the instrumentation is wrong.

### Group 2 — reductions (cluster 4.5)

| Name | Definition | SF step |
|---|---|---|
| `lmr_applied` | A reduced-depth search was performed for a move | 16 |
| `lmr_research` | A reduced search failed high and was re-searched at full depth | 17 |
| `reduction_depth_sum` | Sum of applied reduction in plies | 16 |

Read `reduction_depth_sum / lmr_applied` as mean reduction. Do not read
`lmr_research / lmr_applied` as an error rate without also reading Group 1: a
low re-search rate with poor `best_rank` distribution means moves are being
reduced that never get a chance to fail high, which is the failure RAR-S53
priced at 2.5 plies of unusable depth.

### Group 3 — selectivity (cluster 4.7)

| Name | Definition | SF step |
|---|---|---|
| `razor_drop` | Razoring dropped the node into qsearch | 7 |
| `rfp_cut` | Reverse/child futility returned early | 8 |
| `nmp_attempt` | A null-move search was started | 9 |
| `nmp_cut` | Null move produced a cutoff | 9 |
| `nmp_verify_attempt` | Verification search started | 9 |
| `nmp_verify_pass` | Verification confirmed the cutoff | 9 |
| `nmp_verify_fail` | Verification refuted the cutoff | 9 |
| `probcut_nodes` | A node passed the ProbCut entry gate — per NODE, counted before any move is picked, so nodes where nothing is eligible are included | 10 |
| `probcut_attempt` | A ProbCut search was started — per MOVE. Several can fire at one node (oracle `2 + 2·cutNode`, Rarog up to 8) | 10 |
| `probcut_cut` | ProbCut produced a cutoff — per NODE, so its denominator is `probcut_nodes`, never `probcut_attempt` | 10 |
| `lmp_prune` | A move was skipped by move-count pruning | 13 |
| `quiet_futility_prune` | A quiet move was skipped by futility | 13 |
| `see_prune` | A move was skipped by an SEE threshold | 13 |

### Group 3b — prune recall and overlap (cluster 4.7)

The point of 4.2, and the reason node savings are not enough: a smaller tree can
be worse. These are counted per *move considered at a prunable node*, before any
prune is applied, so overlap is observable.

| Name | Definition |
|---|---|
| `prune_shadow_moves` | Moves reaching the shallow-pruning stage — the denominator |
| `prune_shadow_lmp` | Of those, would be pruned by move count |
| `prune_shadow_futility` | Of those, would be pruned by futility |
| `prune_shadow_see` | Of those, would be pruned by SEE |
| `prune_shadow_check_exempt` | Of those, exempted because the move gives check |
| `prune_shadow_overlap_two_plus` | Of those, would be pruned by two or more families |

`prune_shadow_overlap_two_plus / prune_shadow_moves` is redundancy. A high value
means the families are paying for each other's work rather than covering
different populations. Rarog measured this at 0.47% once (RAR-S21), which was
weak evidence against deduplication being a prize — that measurement predates
this spec and is not directly comparable.

### Group 4 — extensions and depth authority (cluster 4.8)

| Name | Definition | SF step |
|---|---|---|
| `check_extensions` | Node extended because the move gives check | 14 |
| `singular_attempt` | A singular verification search was started | 14 |
| `singular_extend_one` | Extended by one ply | 14 |
| `singular_extend_two` | Extended by two plies | 14 |
| `singular_multicut` | Singular search produced a multi-cut return | 14 |
| `singular_negative_extension` | Depth reduced by the singular path | 14 |

### Group 5 — transposition table (cluster 4.6)

| Name | Definition | SF step |
|---|---|---|
| `main_tt_probes` | Interior-node TT probes | 4 |
| `main_tt_hits` | Probes returning an entry for this position | 4 |
| `tt_cut_exact` | Cutoff taken on an exact bound | 4 |
| `tt_cut_lower` | Cutoff taken on a lower bound | 4 |
| `tt_cut_upper` | Cutoff taken on an upper bound | 4 |
| `tt_bound_not_usable` | Hit whose bound could not serve this window | 4 |
| `main_store_exact` | Interior store with an exact bound | 19 |
| `main_store_lower` | Interior store with a lower bound | 19 |
| `main_store_upper` | Interior store with an upper bound | 19 |

### Group 6 — quiescence (cluster 4.6)

| Name | Definition |
|---|---|
| `q_in_check` | Qnodes entered in check |
| `q_tt_hit` | Qsearch TT probe returned an entry |
| `q_tt_cut` | Qsearch cutoff taken from the TT |
| `q_stand_pat_cut` | Qsearch cutoff taken from stand pat |
| `q_move_cut` | Qsearch cutoff taken from a searched move |

### Group 7 — root and aspiration (cluster 4.9)

| Name | Definition |
|---|---|
| `root_iterations` | Completed iterative-deepening iterations |
| `root_best_changes` | Root best-move changes across iterations |
| `asp_fail_high` | Aspiration re-searches after a fail high |
| `asp_fail_low` | Aspiration re-searches after a fail low |

## Rarog-only — do not port, and do not read as gaps

`9587eeeb` predates all of these, or Rarog solves something it does not have.
Their absence on the oracle side is a fact about 2020 Stockfish, not a finding.

| Family | Why there is no analogue |
|---|---|
| `correction_*` | Correction history postdates this revision entirely |
| `rootconf_*` | Rarog's root-confidence model has no counterpart |
| `lazy_*` | Rarog HCE lazy evaluation; the oracle calls the same HCE without it |
| `store_kind_*`, `tt_move_inherited*` | Rarog's typed TT provenance; the oracle has no producer field |
| `tt_pv_veto_*`, `contradict_*`, `refine_*` | Rarog's TT-bound evidence model |
| `shadow_4_*` | Retired shadow slots from the closed Phase-4 line |
| `worker_*` | Rarog's SMP vote merge |

## Learned at 4.1, from the first instrumented reading

Recorded here because each one would otherwise be misread as a finding.

| Observation | Consequence |
|---|---|
| `singular_extend_two` and `singular_negative_extension` are **structurally absent** in `9587eeeb` — both mechanisms postdate it | They read 0 always. Never read as "the oracle extends less": Rarog has them, the reference does not. These are Rarog-only in the oracle's direction |
| `nmp_verify_*` only fires at depth ≥ 13 | A short-depth suite reads 0. Not a gap; size the suite before reading them |
| `prune_shadow_moves` excludes quiets the picker withheld under LMP | The threshold is enforced in the move picker, not by a `continue`, so LMP-removed moves never reach the shadow stage. LMP's overlap with futility and SEE is observable only at the boundary move. `lmp_prune` is therefore counted in the picker |
| A piped `go … quit` aborts the search before it starts | The suite must drive `bench`, which is synchronous. A naive `go depth 10` returns `bestmove a2a3` on the *frozen* binary too — this is a harness trap, not an engine defect |
| `probcut_cut` initially exceeded `probcut_attempt` | The TT-served ProbCut shortcut returns without running a search. 4.1 made it count as both so the rate could not read above 1 — **that hid it.** Corrected at 4.7c prep: it is a cutoff but not an attempt, and now carries its own oracle-only `probcut_tt_served`. Forcing an invariant to hold is not the same as making the counters comparable |
| `probcut_attempt` was per MOVE on the oracle and per NODE in Rarog | Both engines now carry `probcut_nodes` (per node) and `probcut_attempt` (per move). RAR-S55's "22.7% against 91.2%" divided cuts-per-node by cuts-per-move-tried, and the `2.33×` attempt ratio compared nodes against moves. Corrected at 4.7c prep — see the RAR-S55 correction note in `EXPERIMENTS.md` |

### Invariants that must keep holding

These are free cross-checks. If one separates, the instrumentation is wrong,
not the engine. Verified on the oracle at bench depth 9:

| Invariant | Reading |
|---|---|
| `best_rank_1` == `cutoff_first_move` | 1433 == 1433 |
| `best_rank_1 + best_rank_2_3 + best_rank_4_7 + best_rank_8_plus` == `cutoff_quiet + cutoff_capture` | 1726 == 1726 |
| `main_tt_probes` == `nodes` | 6967 == 6967 |
| `main_store_lower` == cutoff total | 1726 == 1726 |
| `probcut_cut` ≤ `probcut_nodes` | holds on both engines. **Replaces** `probcut_cut ≤ probcut_attempt`, which held only because the two counters were in different units and could not falsify anything |

## Oracle-only — questions for 4.3, never targets

| Name | Mechanism | Note |
|---|---|---|
| `iid_applied` | Internal iterative deepening, SF step 11 | Rarog implements **IIR** instead. These are different mechanisms and must not share a name or be differenced. SF's own annotation prices IID at ~1 Elo, so this is a low-value question |
| `probcut_tt_served` | ProbCut satisfied straight from the TT: a stored entry at depth ≥ `depth − 3` already exceeds `probcutBeta`, so the node returns without searching anything | Rarog has no such path — its ProbCut always searches. Counted inside `probcut_nodes` and `probcut_cut`, so **subtract it** before reading the oracle's cut rate as a *search* conversion. Whether Rarog should acquire this shortcut is a 4.7c question, and it is not the same question as the entry contract |

## The sampling split — 4.2's central design problem

Found at 4.2 by auditing call sites rather than assuming the names matched.

Rarog's diagnostics are **two families**, as `diag.rs` says in its own header:
legacy event counters are exact, while Phase-4 added a deterministic 1/1024
position sample for the wider interaction map. The core set straddles that
line. Roughly half of it — `move_seen_*`, `best_rank_*`, `q_*`, `tt_cut_*`,
`tt_bound_not_usable`, `main_store_lower`, `nmp_attempt`, `probcut_nodes`,
`probcut_attempt`, `singular_attempt`, `prune_shadow_moves` — is **sampled** on
the Rarog side and **exact** on the oracle.

The split runs *inside* families, not just between them: `probcut_nodes` and
`probcut_attempt` are sampled while `probcut_cut` is exact, exactly as
`nmp_attempt` is sampled while `nmp_cut` is exact. That is deliberate and is
left alone — the resolution below is what makes every such pair readable, and
lifting individual counters out of their guards in the hottest file in the
engine is what it was chosen to avoid. **The consequence is a hard rule: no
rate built from these counters may be read at the default stride.**

Joining a 1/1024 sample against an exact count is exactly the error this
document was written to prevent, one level up: the names match, the numbers do
not mean the same thing, and the ratio silently reads 1000× off. It is the
denominator lesson of RAR-S25 applied across engines instead of within one.

Three ways out, and the decision:

| Option | Verdict |
|---|---|
| Make the core set exact on the Rarog side, leaving the wide sampled map untouched | **Chosen.** Sampling exists to bound cost on the wide interaction map, not on the ~59 core counters. A diagnostic build may pay for exactness |
| Make the oracle sample identically | Rejected: it would have to reproduce Rarog's hash, ply and domain mixing exactly, which couples the two implementations instead of both satisfying this spec |
| Scale the sampled side by 1024 | Rejected: valid only in expectation, and it converts a exact cross-check such as `best_rank_1 == cutoff_first_move` into a noisy near-equality that can no longer falsify anything |

Until the core set is exact on both sides, **only the already-exact core
counters may be compared**. The sampled ones are Rarog-internal readings.

## Exact mode, and the three invariant failures it exposed

`RAROG_DIAG_SAMPLE_STRIDE=1` makes every sampled Rarog counter exact in one
place, instead of lifting seventeen counters out of their guards in the hottest
file in the engine. The stride must be a power of two; unset, it stays 1024, so
every historical reading (RAR-S21/S22/S24) still reproduces. `bench 13` is
6,519,711 in all three configurations — diagnostics off, on at 1024, on at 1.

Running the invariants against Rarog in exact mode immediately broke three of
them. **This is the cross-check doing its job**: every one would have silently
poisoned the 4.2 differential, and each is a definition mismatch rather than an
engine defect.

| Invariant | Rarog, exact | Verdict |
|---|---|---|
| `best_rank_1` == `cutoff_first_move` | 668,275 vs 475,494 | **FIXED 2026-08-12.** Now 475,494 == 475,494 |
| rank buckets == `cutoff_quiet + cutoff_capture` | 829,922 vs 537,976 | **FIXED 2026-08-12.** Now 537,976 == 537,976 |
| `main_tt_probes` == `nodes` | 2,336,660 vs 4,022,611 | **FIXED 2026-08-12.** Now 2,336,660 == 2,336,660 |

**`best_rank_*` measures a different thing in each engine.** Rarog records the
rank of the *best move found at any node*, guarded only by `diag_best_rank > 0`,
so it includes PV nodes where the best move merely raised alpha and no cutoff
happened. The oracle records the rank *at which a beta cutoff occurred*. Rarog's
population strictly contains the oracle's, which is exactly why the numbers look
comparable and are not — the RAR-S25 lesson, one level up. **Resolved 2026-08-12.** `best_rank_*` keeps its
spec meaning of cutoff rank and is now counted exactly in Rarog's beta-cutoff
block, in the same guard as `cutoff_quiet`/`cutoff_capture` so the buckets sum
to that denominator by construction. Rarog's original measurement was renamed to
`best_move_rank_*` and is **Rarog-only**: it still reads 668,275 at rank 1, and
differencing it against the oracle is now a naming error rather than a silent
one.

**`nodes` itself was mis-defined, and `main_tt_probes` was the symptom.**
Rarog incremented `nodes` at function entry, *before* the `depth <= 0` hand-off
to quiescence. Every node that immediately became a qnode was therefore counted
twice — once as `nodes`, again as `qnodes` — which inflated the Group 0
denominator by 1,685,951 on `bench 13` and silently deflated every rate taken
against it. The oracle never had this problem because its quiescence is a
separate function reached before `search<>()` is entered at all.

**Resolved 2026-08-12** by counting after the hand-off, so `nodes` means
interior nodes actually searched. `nodes` fell from 4,022,611 to 2,336,660 and
`main_tt_probes == nodes` now holds exactly. This is the most consequential
find of 4.2: it was not a counter bug but a *denominator* bug, and it would
have skewed every Group 1, 3 and 5 rate in the differential in the same
direction, which is precisely the kind of error that looks like a finding.

## Mechanism divergences found at 4.2 — inputs for 4.3

| Mechanism | State | Note |
|---|---|---|
| **In-check extension** | Oracle extends (`check_extensions` = 68 at bench depth 9); **Rarog does not extend at all** | Rarog removed the unconditional in-check extension in its Phase 8.2(a) and measured **+30.75 Elo** for removing it. RAR-X02 records the opposite sign in Basilisk, which lost −10.17 removing its own. This is a first-class *intentionally different* result with local evidence, not a gap to close. 4.8 owns any revisit, and must start from Rarog's +30.75, not from the reference's behaviour |
| Double and negative singular extension | Rarog has both; the oracle has neither | Recorded at 4.1 from the other direction |
| IID vs IIR | Oracle has IID; Rarog has IIR | Different mechanisms, never differenced |

## Known gaps on the Rarog side (4.2 work)

| Gap | Detail |
|---|---|
| `qnodes` | **CLOSED 2026-08-12.** An exact `qnodes` now increments at qsearch entry alongside the sampled `sampled_qnodes`. Bench stays 6,519,711 with diagnostics both off and on |
| `prune_shadow_*` | **Audited 2026-08-12 — NOT comparable, see below** |
| `reduction_depth_sum` | **CLOSED 2026-08-12.** It does sum applied reduction, correctly, but was behind the sample guard while its denominator `lmr_applied` was exact, so the mean read 1024x low. Now exact. Rarog's mean reduction on `bench 13` is **2.960 plies** |

### `prune_shadow_*` is not comparable as implemented

The two engines apply SEE pruning to different move populations, so the family
sets the shadow counters range over are not the same:

| | Rarog | Oracle |
|---|---|---|
| LMP / futility shadow | quiet moves | quiet moves |
| SEE shadow | **captures only** (`else if is_capture && see < 0`) | **quiets** (the `lmrDepth`-scaled threshold) |
| `prune_shadow_overlap_two_plus` | `lmp && futility` only — SEE lives in a mutually exclusive branch, so SEE can never overlap | any two of `lmp`, `futility`, `see`, all on quiets |

So Rarog's overlap figure is structurally incapable of including SEE, while the
oracle's is dominated by it. Differencing them would compare a two-family
redundancy against a three-family one and read the gap as a finding. This also
explains why RAR-S21's 0.47% overlap looked so much lower than the oracle's
~5.2%: they are not the same measurement.

Resolution is owned by 4.7, which is the cluster that owns selectivity
architecture anyway: decide whether Rarog's capture-only SEE pruning is
intentionally different, then define the shadow families to match whatever
survives. Until then these six counters are **Rarog-internal readings**, not
differential inputs.

None of these are oracle-side work. They are recorded here so 4.2 starts from a
list rather than a rediscovery.

## What these counters may not be used for

- They may not accept or reject a candidate. Only a registered final-PGO SPRT
  does that.
- A divergence is a question about where to look next, never a target value.
  Closing a gap in a counter is not an outcome.
- Node savings are not strength. Group 3b exists precisely because the tempting
  reading — fewer nodes is better — is the one that has already cost Rarog Elo.
- Values from different suites, depths or node budgets are not comparable. The
  4.2 suite is versioned for this reason.
