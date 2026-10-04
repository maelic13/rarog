# 4.10 obligations — everything deferred there, with its evidence

Written 2026-08-19 because deferrals to 4.10 were accumulating across ledger
rows, GUIDE bullets and commit messages, and a catch-all clause ("resolve every
remaining search-map item") is where structural work goes to be forgotten.

**4.10 owns consolidation TUNING and integration.** Several items below are not
tuning. Those are flagged, because filing structural work under a tuning step is
how it gets lost — and one of them is already the direct consequence of that
having happened once.

Nothing here may be dropped because an SPRT failed elsewhere. Each item carries
the evidence needed to act without re-deriving it.

---

## A. Structural work that 4.10 does NOT own — NOW ASSIGNED, 2026-08-19

These came out of Cluster A's closure. They are implementation, not tuning, and
assigning them to 4.10 by default would have been the mistake this document
exists to prevent. **Both now have owners in PLAN:**

- **A1 → new step 4.9b**, before 4.10, because 4.10 re-runs the 4.2 suite as
  its evidence base and that is not meaningful while history is unsettled.
- **A2 → 4.8.1**, a sub-item of 4.8, which already owns LMR and depth authority
  against the 4.5 per-ply context. Written as a sub-item so it cannot be closed
  by implication a second time.

They stay listed here with their evidence so 4.10 knows what it depends on.

### A1. History semantics — the unexecuted half of 4.5.3

PLAN 4.5.3 named: main/capture/continuation/low-ply/pawn/correction history
**indexing, check/capture context, normalization, ageing, and cutoff/fail-low
attribution**, plus measuring evaluation-difference training and seed/decay
policy as Rarog candidates.

**Done:** continuation-key consolidation; the continuation-malus asymmetry
measured and rejected (RAR-S59).

**Never touched:** ageing policy, decay policy, seed policy, update attribution,
check/capture context in history indexing, evaluation-difference training.

### A2. The reduction and re-search contract — the unexecuted half of 4.5.4

PLAN 4.5.4 named reductions and re-search authority as **one explicit
contract**. It does not exist. `lmr_reduction_units` still takes eleven loose
arguments and is not a contract over the per-ply context. The accepted
zero-reduction floor and full-depth verification were never audited.

---

## B. Mechanisms with evidence, awaiting a decision

### B1. Deliberate randomisation of the selectivity surface — the strongest lead

**Three independent measurements now say the same thing**, and it was only
visible once they were put side by side:

| | what was perturbed | result |
|---|---|---|
| RAR-S54 | blind uniform 15% de-selectivity shift, 12 constants | **+4.06 ± 3.71** over 14,196 games |
| RAR-S62 | ProbCut desync — arbitrary continuation row | beat correct indexing by ~5 Elo |
| RAR-S64 | stale prior-reduction — quasi-random subset reduced less | beat correct prior-reduction by ~4.5 Elo |

In the last two, the version reading a **wrong** value beat the version reading
the right one. Neither was designed; both were bugs. The common factor is not
tree size — RAR-S62's better arm had *fewer* nodes — it is **scattered
perturbation of an over-confident selectivity surface**.

The machinery already exists: Rarog runs per-thread LMR jitter (`next_jitter`)
for SMP diversification, and it is disabled at 1T (`shared_state.is_some()`),
which is precisely why this has never been tested as a strength mechanism.

**Proposed:** enable a tuned jitter magnitude at 1T and gate it. Note the
existing jitter is ±64/1024 chosen to be small enough not to distort the mean
reduction — a strength jitter probably wants to be larger, and its magnitude is
the thing to fit.

**Do not read this as "ship bugs."** The claim is that deliberate, bounded,
documented randomisation is a candidate mechanism with three-deep supporting
evidence.

### B2. `NullMoveImprovingBonus` is a volume knob, not a quality knob

RAR-S58: sweeping it 0 → 80 swings the tree **25%** (6,225,304 → 7,797,922
nodes) while null-move conversion does not move at all (27.6–28.8% across the
whole range). It is an activated coordinate with large effect and no quality
signal — exactly what a consolidation fit should own.

### B3. Shallow TT-served ProbCut cutoffs

Already assigned to 4.10 by PLAN. RAR-S58's counters sized the population: the
oracle takes a TT-served ProbCut return at **0.89% of its nodes** at zero search
cost, and Rarog has no such path at all. Cheap, not selectivity, and it must be
attributed on its own if pursued.

### B4. A changed NMP / ProbCut / futility population

Assigned to 4.10 by PLAN. 4.7c changed the ProbCut population substantially
(moves searched −56.6%), so the second-pass suite re-run should expect it.

---

## C. Rejected, with the evidence — do not re-open without new information

### C1. Stockfish `cutoffCnt` — rejected TWICE, on independent grounds

**RAR-S13 rejected it already: −7.78 ± 8.00 Elo**, as `cutoffCnt` plus a full
LMR-family SPSA. Its lesson is on the record: "a tuner can select a
sibling-local optimum" — the candidate won its own tuning self-play and then
lost to the accepted head.

**RAR-S60 rejected it again**, from the other direction: its consumer is
`if ((ss-1)->cutoffCnt > 3) r++`, a selectivity *increase*, which is the one
direction four readings contradict for this engine.

A correction is recorded in RAR-S60 and repeated here so the note is not
misread: the semantics are **knowable**, not guesswork. Stockfish resets
`(ss+2)->cutoffCnt = 0` on node entry — zeroing the *grandchild's* counter — so
a ply slot accumulates cutoffs across sibling visits, which is how it exceeds 3.
The 4.5.4 attempt reset per visit, making it 0-or-1 and inert. That was an
implementation error, not a barrier.

**Standing counter-point, recorded for fairness:** it is a *conditional*
increase aimed at plies that demonstrably cut often, unlike the blanket
increases rejected elsewhere. That is the only thing that would justify a third
attempt, and it would need to beat RAR-S13's −7.78 as its starting position.

### C2. `lmr_prior_reduction_adj` — dead, and its curvature evidence is void

RAR-S61 recorded an interior optimum in first-move cutoff at
`LmrPriorReductionAdj=768` and handed the curvature to 4.10. **That evidence is
void.** RAR-S64 then showed the mechanism was worth ~0 once the stale-reduction
defect was fixed, and it has been removed entirely — parameter, consumer and
the `NodeContext.reduction` field. There is nothing to tune. Recorded because
the hand-over note exists in RAR-S61 and would otherwise look live.

### C3. ProbCut child reduction semantics — moot

An open question was whether a ProbCut child should report its parent as
"reduced", given a ProbCut search is depth-reduced but not by LMR. Moot: the
field it concerned no longer exists (C2).

### C4. Continuation malus for failed quiets — rejected on measurement

RAR-S59. Ordering flat (88.04% → 88.09%), tree −7.5%, cutoffs −9.6%. Cutoffs
fell faster than nodes, so it is a selectivity increase disguised as an
evidence-hygiene fix. The switch was built, measured and removed.

---

## D. Switches currently live and owed a disposition

Both default OFF, so the accepted head is unchanged. Each must be accepted or
removed — they may not be left dormant.

| switch | finding | bench readings (both off = 7,467,143 / fm 88.04% / cut-node 0.0853) |
|---|---|---|
| `ImprovingPly4Fallback` | audit #2 | 6,969,327 / 88.27% / 0.0856 |
| `KillerClearGrandchild` | audit #3 | 6,556,136 / **88.70%** / 0.0856 |

Read with the caution this cluster earned: `cut/node` is **flat** in both, the
signature RAR-S59 used to unmask a disguised selectivity increase, and both make
the tree *smaller* — more selective — which is the direction the evidence
contradicts. What differs is the first-move cutoff rate, which RAR-S59's
rejected candidate did not move at all.

Audit #2 has a mechanism argument independent of the proxy: when `ply - 2` was
in check its eval is `VALUE_NONE`, so `improving` is forced false at **9.7% of
nodes** regardless of the real trend, costing a full ply of LMR reduction and
inflating the LMP margin.

---

## E. Method notes that must survive into 4.10

- **Bench proxies get no credit on their own.** RAR-S64 is the case: a
  mechanism adopted on a clean bench signal (cutoffs/node rising faster than
  nodes, first-move cutoff improving) measured **exactly zero** in games.
- **Measure cutoffs-per-node alongside the cutoff RATE.** The rate alone waved
  RAR-S59's candidate through; the ratio caught it.
- **Wide brackets REJECT small gains — use narrow ones anchored at zero.**
  `[0,10]` drives a true +4 nElo to H0 in ~35k games; `[0,3]` accepts it in
  ~47k. Fishtest's shape is `[0,2]` STC / `[0,1]` LTC. 4.10's consolidation
  gains are the most likely of all to be small, so this matters most here.
  See the Gating section of `AGENTS.md`.
- **Size the SPRT bracket from RAR-M10 before registering.** RAR-S61 spent
  16,000 games at `[3,10]` for LLR 0.39 because the candidate landed 0.42 nElo
  from the midpoint. `[0,10]` then resolved the same question in 8,088.
- **Components do not add.** RAR-S50 measured a 20.8-point swing between the sum
  of individuals and the set effect; this cluster produced three more instances,
  including a change costing 7.9% nodes standalone and 0.36% in company.
