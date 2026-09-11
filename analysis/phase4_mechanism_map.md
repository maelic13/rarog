# Phase-4 mechanism map and order freeze

Status: **frozen at PLAN step 4.3**, 2026-08-12. Consumed by clusters 4.5–4.9.

## What this is, and what it is not

A catalogue of **problems a strong search solves**, and what Rarog does about
each one today. The reference was read to populate the problem list and to see
one working set of answers; the output here is a **Rarog work list**, and it is
written to be usable with the reference closed. There is no column for "what
Stockfish does" as a target, because similarity is not a goal — see PLAN §4,
"The independence boundary".

Every verdict below is grounded in Rarog's own measurements: RAR-S52 (ordering
ratios), RAR-S53 (fixed-node depth), RAR-S54 (blind de-selectivity), RAR-S55
(the differential). Where nothing has been measured the verdict says so rather
than guessing.

Normalised figures quoted as `Nx` are from RAR-S55: firings per node searched,
with the 1.861 node-ratio divided out. 1.00 means "in line with tree size".
The LMP and ProbCut figures are the **corrected** ones — v2 and v3 of the
differential respectively; both original readings compared per-move against
per-node counters. See the two RAR-S55 corrections in `EXPERIMENTS.md`.

## Classification

| Verdict | Meaning |
|---|---|
| **SOLVED** | Rarog addresses this, and its own evidence says the answer is sound |
| **WEAKER** | The problem is present in Rarog and its answer is measurably worse |
| **DIFFERENT** | Rarog deliberately answers differently, with local evidence for it |
| **UNKNOWN** | Not yet measured here; needs work before it can be classified |

## The map

### Ordering and evidence — owner 4.5

| Mechanism | Problem it solves | Rarog today | Verdict |
|---|---|---|---|
| Staged move generation | Search the likely-best move first, so the rest can be cut cheaply | `MovePicker` enum inside `search.rs`, no staged guarantee contract | **SOLVED.** First-move cutoff 86.7–93.2% by cohort, *above* the reference's 83.5–87.0% everywhere. RAR-S52 already read 87.65% |
| TT move priority | Reuse the previously best move | Present; `move_seen_tt` 1.90x | **SOLVED** |
| Capture ordering / SEE staging | Try winning captures early, defer losing ones | Present; `move_seen_good_capture` 0.69x, `move_seen_bad_capture` 0.57x | **SOLVED** |
| Killers and counter-moves | Order quiets by sibling refutations | Present, folded into the quiet stage | **SOLVED** |
| Main / capture / continuation / low-ply history | Learn ordering from what actually cut | All present | **SOLVED** |
| Correction history | Correct static eval by observed residual | Present; the reference has none at all | **DIFFERENT** — Rarog-only, keep |
| Quiet supply per node | — | `move_seen_quiet` **2.11x**: the picker hands the search twice as many quiets per node | **UNKNOWN.** Consistent with a wider tree rather than an ordering fault, but not separated from selectivity yet |

**Cluster 4.5 verdict: the ordering premise is refuted.** Rarog orders *better*
than the reference on its own numbers and still loses ~196 Elo. Whatever the
deficit is, it is not here. 4.5 keeps its number and its scope but drops to a
low-expectation cluster: the remaining candidates are the history and
reduction contracts, not the picker.

### Reductions — owner 4.5

| Mechanism | Problem | Rarog today | Verdict |
|---|---|---|---|
| Late move reductions | Spend less depth on unlikely moves | `lmr_applied` **1.29x**, `reduction_depth_sum` **1.53x** | **WEAKER (suspect).** Mean reduction 2.19 plies against 1.85: Rarog reduces 29% more moves per node, each 18% deeper |
| Full-depth verification | Re-search when a reduced move fails high | `lmr_research` 0.53x | **WEAKER (suspect).** Half the re-search rate while reducing far more is the RAR-S53 signature — reduced moves that never get the chance to fail high |
| Zero-reduction floor | Let strong late moves escape reduction | Present and accepted (+9.13 nElo, 2.3.2) | **SOLVED** |

### Selectivity — owner 4.7

This is where the evidence concentrates.

| Mechanism | Problem | Rarog today | Verdict |
|---|---|---|---|
| Move-count pruning | Stop searching quiets once enough have failed | `lmp_nodes` **0.57x** — fires at 8.3% of nodes against 14.6% | **SOLVED, and the earlier reading was wrong.** The 13.35x first reported was `lmp_prune` (per move) against the oracle's per-node counter. Corrected: Rarog prunes by move count *less* often, not more |
| Quiet futility | Skip quiets that cannot reach alpha | `quiet_futility_prune` 0.44x | **UNKNOWN.** Lower, but LMP removes the population before futility sees it |
| SEE pruning | Skip moves that lose material outright | `see_prune` **0.18x**, and captures only | **DIFFERENT, and the scope is the question.** The reference also prunes *quiets* by SEE; Rarog does not. Rarog's low rate is mostly this scope difference, not a weaker threshold |
| Reverse futility | Return early when the static eval is far above beta | `rfp_cut` 1.38x | **SOLVED (probably).** Slightly hot, no evidence of harm |
| Razoring | Drop hopeless nodes into qsearch | `razor_drop` 1.61x | **UNKNOWN** |
| Null-move pruning | Prove a node fails high without searching a move | `nmp_attempt` 0.94x, `nmp_cut` **0.22x** | **WEAKER, and the sharpest single reading.** Rarog attempts null move as often and converts **19.2%** of attempts against the reference's **83.3%** |
| Null-move verification | Re-verify a null cutoff at high depth | Present; fires only at depth ≥ 13, so the depth-8 suite reads 0 on both | **UNKNOWN.** Needs a deeper suite run |
| ProbCut | Prove a capture beats beta with a shallow search | `probcut_attempt` (per move) **5.17x**, conversion per move 32.6% vs 71.9%; conversion per node 22.7% vs **25.2%** | **WEAKER, at the MOVE level only.** Restated 2026-08-14 on the corrected v3 counters. Per node the engines convert alike and Rarog produces 2.35x the oracle's search cutoffs; it pays 5.17x the moves for them. The old "22.7% against 91.2%" was 75.3% TT-served freebies the oracle gets without searching |

**Cutoff composition is inverted.** The reference takes 1.37 quiet cutoffs per
capture cutoff; Rarog takes 0.71 — `cutoff_quiet` 0.66x against
`cutoff_capture` 1.29x. Rarog's cutoffs come predominantly from captures where
the reference's come from quiets. This no longer has an LMP explanation, since
the corrected `lmp_nodes` shows Rarog pruning quiets at *fewer* nodes. It is
therefore an open question, and a real one: it is owned by 4.7 but has no
identified mechanism yet.

### Extensions and depth authority — owner 4.8

| Mechanism | Problem | Rarog today | Verdict |
|---|---|---|---|
| In-check extension | Search forcing lines deeper | **Removed** in Phase 8.2(a) | **DIFFERENT, with the strongest local evidence in the phase: removing it measured +30.75 Elo.** The reference extends (8,513 firings). Do not close this "gap" — RAR-X02 records Basilisk losing −10.17 doing the opposite |
| Singular extension | Extend a move that is the only one holding | `singular_attempt` **3.21x**, `singular_extend_one` 1.40x, `singular_multicut` 2.98x | **UNKNOWN.** Attempts 3x as often, extends only 1.4x as often — the verification search is being spent without producing an extension |
| Double / negative singular extension | Grade the extension by margin | Present in Rarog, **absent** in the reference | **DIFFERENT** — Rarog-only |
| IIR / IID | Get a TT move when there is none | Rarog uses IIR; the reference uses IID | **DIFFERENT.** Different mechanisms. The reference's own annotation prices IID at ~1 Elo, so this is low value either way |

### Static eval, TT and quiescence — owner 4.6

| Mechanism | Problem | Rarog today | Verdict |
|---|---|---|---|
| TT probe and bound cutoff | Reuse proven bounds | `main_tt_probes` 1.00x, `main_tt_hits` 1.13x, `tt_cut_lower` 0.91x | **SOLVED** |
| Unusable bounds | — | `tt_bound_not_usable` **2.24x** | **UNKNOWN.** Twice the rate of hits that cannot serve the window |
| Store bound kinds | — | `main_store_lower` 0.93x, `main_store_upper` 0.56x, `main_store_exact` 0.57x | **UNKNOWN.** Rarog stores proportionally far fewer upper and exact bounds |
| Typed TT provenance | Know which mechanism wrote an entry | Present; the reference has no producer field | **DIFFERENT** — Rarog-only |
| Quiescence size | Resolve captures before evaluating | `qnodes` **1.62x** | **WEAKER (suspect).** Rarog's qsearch is 62% larger per interior node |
| Qsearch TT use | — | `q_tt_hit` **2.58x**, `q_tt_cut` **4.25x** | **UNKNOWN.** Rarog's qsearch leans far harder on the table. Benign or a symptom of a bloated qsearch — 4.6 must separate the two |
| Stand-pat cutoff | — | `q_stand_pat_cut` 1.62x | **UNKNOWN**, tracks qsearch size exactly |

### Root and clock — owner 4.9

| Mechanism | Problem | Rarog today | Verdict |
|---|---|---|---|
| Aspiration windows | Search a narrow window and widen on failure | `asp_fail_high` 0.58x, `asp_fail_low` **0.28x** | **UNKNOWN.** Rarog fails low far less. A narrower effective window or a less volatile root — not separated |
| Root best-move changes | Detect instability for time management | `root_best_changes` **0.31x** | **UNKNOWN.** Rarog's root is three times more stable per iteration. Given RAR-S53 priced time management at ~0, this is a low-value lead |
| Completed-root authority | Never act on a partial iteration | Present, with accepted abort/fallback coverage | **SOLVED** |

## The order decision

**The provisional order is changed. Cluster 4.7 executes first.**

PLAN §4 said execution would follow the dependency order 4.5 → 4.6 → 4.7,
"because selectivity consumes the depth and history evidence that cluster A
owns", and committed to reordering here if 4.2–4.3 contradicted it. They do.

The dependency argument assumed cluster A would *improve* the inputs
selectivity consumes. That premise is refuted: Rarog's ordering is already
better than the reference's in every cohort, so there is little for 4.5 to
improve and correspondingly little for 4.7 to wait on. Meanwhile 4.7 is the
only cluster carrying a positive local result already — RAR-S54's blind,
untuned 15% de-selectivity shift measured +4.06 ± 3.71 — and the differential
concentrates its largest divergences there.

Item numbers do not move. 4.7 keeps its number and executes first; this is an
execution order, not a renumbering.

Execution order: **4.7 → 4.5 → 4.6 → 4.8 → 4.9**, with 4.4 before all of them
where a contract needs the per-ply state.

Accepted risk, stated: fitting selectivity before ordering means 4.5 could
later disturb 4.7's result. Rule 12 says constants are fitted around current
activations, and that is real. It is accepted because 4.5's expected movement
is now small — its own premise having been refuted — and because 4.10 is the
cumulative checkpoint where the combination is verified rather than assumed.
If 4.5 later moves ordering materially, 4.7 is re-verified against the accepted
head, not assumed to hold.

## The first cluster's shape

4.7 is about the **shape** of the selectivity surface, not its constants. The
closed Phase-4 line already fitted those constants for +15.33 nElo and left the
shape untouched, which is why fitting them again is out of scope.

The three leads, in the order the evidence ranked them. **One survives.**

1. **Null-move conversion.** 19.2% of attempts produce a cutoff against 83.3%.
   Rarog is paying for null-move searches it overwhelmingly does not use.
   Both counters are per-node on both engines, so this reading is comparable
   and stands. It is what 4.7a was built against.

2. **ProbCut move filter.** **Restated 2026-08-14**, not withdrawn. The
   ranked claim — "22.7% against 91.2%, at 2.33x the attempt rate" — was a
   per-node against per-move comparison and is gone: per node the two engines
   convert alike, 22.7% against 25.2%, and 75.3% of the oracle's ProbCut
   cutoffs were TT-served freebies that never ran a search. What the corrected
   counters show instead is one level down. Rarog searches **5.17x** the
   normalised ProbCut moves and converts **32.6%** of them against **71.9%** —
   the same attempt-often/convert-rarely shape as the null-move lead, at the
   move rather than the node. The oracle admits a capture only when SEE bridges
   `probcutBeta − staticEval` and stops at `2 + 2·cutNode`; Rarog admits any
   `see_ge(mv, 0)` and tries up to 8.

   *Separable, and not part of the above:* Rarog has no TT-served ProbCut
   shortcut. The oracle takes one at 0.89% of its nodes at zero search cost.
   Cheap, not selectivity, and must be attributed on its own if pursued.

   ~~Move-count pruning volume~~ — **withdrawn 2026-08-12.** The 13.35x was a
   per-move against per-node artifact; corrected, Rarog fires LMP at 0.57x the
   reference's per-node rate. See the RAR-S55 correction.

All three ranked readings were per-move-against-per-node comparisons. One
mechanism was withdrawn outright (move-count volume), one survived intact
(null-move conversion), and one was restated at a different level (ProbCut).
Each was caught by auditing the counter before designing against it, never
after a game budget had been spent.

**The cluster's binding claim has changed.** These three were placed in one
cluster because they compete for the same *quiet* population. That no longer
holds: LMP is not over-firing, and the surviving ProbCut finding is about
*captures* — the shallow-search move filter — not quiets at all. What 4.7a and
the ProbCut move filter share is a **failure shape**, entry admitted far too
cheaply into a speculative reduced search, not a contested population. That is
still a defensible bundle, but it is a weaker form of coherence than the map
originally claimed, and the 25–60 nElo cluster prior was not sized against it.
Re-sizing is owed before the bundle is gated.

## What this map does not license

- No mechanism above is a target because the reference has it. `check_extensions`
  is the standing example: the reference fires it 8,513 times and Rarog measured
  **+30.75 Elo for not having it**.
- No counter here accepts anything. Only a registered final-PGO SPRT does.
- UNKNOWN means unknown. Six mechanisms carry that verdict, and the honest
  consequence is that the cluster owning each must measure before it designs.
