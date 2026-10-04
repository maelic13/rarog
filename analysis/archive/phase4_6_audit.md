# 4.6 Cluster B — audit, 2026-08-20

The Audit step of the lifecycle, done before anything is built. Source: the
corrected differential `phase4_differential_v4_depth8.txt` plus a read of every
area PLAN 4.6 names.

**The first thing this audit did was invalidate its own headline.** The runner
normalised every `q_*` counter by main-search NODES while Rarog runs 1.60× more
qsearch per node. Fixed in the tool; every number below is post-correction.
That is the third instance of this denominator class in the project, and the
first inside the tooling rather than the engine.

---

## Leads, in the order the corrected evidence ranks them

### 1. `qnodes` **1.60×** — Rarog spends 60% more of its tree in quiescence

The headline, and it is not an artifact — this ratio is what the other `q_*`
counters are now normalised *by*. Every explanation below is a candidate cause
of it; none has been shown to be the cause.

### 2. `q_tt_cut` **2.46×** — no PV concept in quiescence

Rarog takes a qsearch TT cutoff at **26.8% of qnodes**; the oracle at **10.2%**.

Mechanism, and it is contract-shaped: the oracle guards its qsearch TT cutoff
with **`!PvNode`**. Rarog's `quiescence` has **no PV concept at all** — there is
no `is_pv` parameter in its signature — so it takes TT cutoffs everywhere,
including on PV lines, where a cutoff truncates the line the engine is about to
report and play.

This is the same shape as 4.7c, the only change that has paid in this phase: a
distinction the reference states explicitly and Rarog does not draw.

### 3. `q_move_cut` **0.66×** and `q_in_check` **0.64×** — no quiet checks in qsearch

Rarog's qsearch generates `generate_legal_captures()` when not in check, and a
full movelist only when in check. The oracle generates **quiet checks at the
first qply** (`DEPTH_QS_CHECKS`). So Rarog never searches a quiet checking move
in quiescence, which is consistent with both readings: fewer in-check qnodes,
and fewer cutoffs earned from searching moves.

⚠ **Do not treat this as free Elo.** Rarog measured **+30.75 Elo for removing
its check extension**, and RAR-X02 recorded Basilisk losing 10.17 doing the
opposite. Check-related mechanisms in this engine are co-adapted, and this is
the population with the worst track record for casual changes.

### 4. TT bound composition — Rarog hits MORE and converts LESS

| | Rarog | oracle |
|---|---:|---:|
| hit rate per probe | **67.4%** | 60.3% |
| cutoff per hit | **16.7%** | 19.5% |
| bound-not-usable per hit | **8.7%** | 5.9% |
| Exact share of stores | **3.2%** | 4.1% |

`tt_bound_not_usable` is **2.13×** normalised. Rarog finds an entry more often
than the reference and can use it less often: it stores proportionally fewer
`Exact` bounds — the only kind usable in any window — and correspondingly more
`Lower`. `tt_cut_exact` 0.65×, `tt_cut_upper` 0.73×, `tt_cut_lower` 0.87×; all
three conversion routes are below parity.

This is a producer-side question, not a consumer-side one: the probe is fine,
what is stored is the issue. PLAN 4.6 names "TT admission, replacement, PV and
bound propagation" and this is exactly that.

### 5. Opponent-worsening — **absent**, and PLAN 4.6 names it as a deliverable

Zero occurrences in `src/`. The reference derives an opponent-worsening signal
from the previous ply's static eval and feeds it into the reverse-futility
margin.

**4.5.1 already built the substrate**: `stack[ply - 1].static_eval` is available
and correct. PLAN 4.6 says "derive opponent-worsening from the 4.5 context and
give it an explicit consumer". The context exists; the consumer does not.

Related: `rfp_cut` is **1.41×** and `razor_drop` **1.65×** — Rarog's top-of-node
pruning already fires harder than the reference's, which is the surface an
opponent-worsening term would modulate.

---

## Checked and NOT a lead

- **qsearch stand-pat: parity.** `q_stand_pat_cut` is **1.05×** once normalised
  correctly (it read 1.62× before). 33.7% of qnodes on both engines. The
  pre-correction number would have sent us at a non-existent divergence.
- **Raw / corrected / pruning separation: already present.** 41 references to
  `raw_static_eval` and `eval_for_pruning`; `refine_eval` is mirrored in
  quiescence with the depth-0 refinement RAR-S02 accepted. Largely delivered by
  the 4.3a work; the end-to-end audit PLAN asks for is a review, not a build.
- **Delta pruning: present.** `occupied_count() > 8 && stand_pat + queen + 200
  < alpha`.
- **Evasions: present.** Full movelist when in check, with the mate return.
- **`main_tt_probes` 1.00×.** Probing is at exact parity — one probe per node on
  both engines.

## Not audited

Draw and mate-distance semantics were not re-derived: PLAN says preserve them,
they are covered by `tests/`, and nothing in the differential points at them.
Capture/promotion ordering *within* qsearch was not separated from the main
picker's ordering, because the counters do not distinguish them.

---

## What this suggests, and what it does not

The three strongest leads — no PV concept in qsearch, no quiet checks in
qsearch, and the TT bound composition — are all **contract distinctions the
reference draws and Rarog does not**, which is the profile of 4.7c rather than
of the five candidates that failed after it (three design differences and two
blind scalars).

That is a reason to rank them, not to believe them. RAR-S64 is the standing
warning: a mechanism with a clean bench signal measured exactly zero in games.
Each of these needs its own registered gate at `[0,3]`, and lead 3 carries a
specific hazard on top, because check populations in this engine have
repeatedly rewarded doing *less*.
