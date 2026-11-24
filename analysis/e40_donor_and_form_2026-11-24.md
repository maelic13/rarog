# E.4.0 — The donor and the form: which classical Stockfish evaluation, and a rebuild or repairs

Research packet for PLAN E.4.0, 2026-11-24. No engine source changed. The
reads behind it are RAR-O13 (games, the maintainer's) and RAR-E74 (zero
games). Evidence layers are named where they are used: cross-entropy,
skill and decided-game counts are not Elo; an Elo figure is a played
match or says "inferred".

## Decision needed

The maintainer's, before anything is built:

1. **The donor:** Stockfish 11's classical evaluation (`sf_11`) or Stockfish
   16's (`sf_16`). `9587eeeb`'s was not read here; Basilisk's games put it
   level with `sf_16`.
2. **The form:** rebuild Rarog's evaluation whole in the donor's form, as
   D.7 rebuilt the search, or repair it family by family.
3. **The fit:** how a rebuilt evaluation's constants are fitted, given that
   the pipeline's objective cannot see the gap (finding 4).
4. **The disclosure:** a rebuild seeded from the donor's constants changes
   what D.11 has to say. CCRL, CEGT and SPCC do not judge originality, but
   ICGA rule 2 requires every source named, and the README's posture must
   stay accurate (PROCESS, *The independence boundary*).

## Known evidence

### 1. The donor's evaluation is worth about +70 inside the kernel (games)

RAR-O13, Colosseum, 2,000 games each, the kernel on the donor's evaluation
through a diagnostic bridge against the kernel on Rarog's, the scale
matched by K on results:

| Read | Elo of the donor arm, 95% |
|---|---|
| `sf_16`, 186,000 nodes a move | **+80.3 ± 10.8** |
| `sf_11`, 186,000 nodes a move | **+74.4 ± 10.6** |
| `sf_16`, `3+0.03`, the bridge's 8.8% returned as time odds | **+68.8 ± 10.4** |

- The kernel's margins, its evaluation wrapper and `LazyMargin` were fitted
  on Rarog's evaluation; the donor ran under them unfitted. Above E.0's +25
  rule by about ten standard errors.
- On the clock the donor arm searched 9.0% fewer nodes a move than the
  base. That is the donor evaluation's own cost inside the kernel, the
  bridge excluded, and it took about 11.5 of the 80 (±15, not resolved).
- At equal nodes, depth without mate scores was equal in the games (17.58
  against 17.54). The trees on the donor's evaluation are 1.22 to 1.30
  times larger at fixed depth on the kernel's positions, with a tail in
  decided endings.

### 2. The two donors are level in play; `sf_11` is ahead statically and cheaper

- In play: `sf_16` minus `sf_11` +5.9 at equal nodes, with an interval of
  about ±15.1, not read. Basilisk's BAS-O09 put `sf_16` and `9587eeeb`
  level within about 8 Elo in either search.
- Node cost inside the kernel, from the two equal-node matches' telemetry
  (the same bridge, 14 games at a time, not a timed read): `sf_11`'s arm
  1.114M nodes a second, `sf_16`'s 1.068M, so `sf_11`'s evaluation costs
  about 4% less of a node. At RAR-M92's 1.67 Elo a percent that is about +7
  on the clock, the size of `sf_16`'s point lead at equal nodes (inferred).
- Static cross-entropy against results (RAR-E74): `sf_11` predicts better
  than `sf_16` on every corpus, +0.341 ± 0.045% on `hce-v4-tb` and +0.462 ±
  0.078% on the kernel's games, and in every material band but the last.
  On Stockfish 19's labels (E.0) `sf_16` minus `sf_11` is −0.0044 skill
  [−0.0087, +0.0005], and −0.011 [−0.019, −0.004] at 25–44.
- `sf_16`'s lazy cut reads the search's root score, which the kernel must
  hand to the evaluation. Its halfmove damping and clamp duplicate the
  kernel's wrapper. Some of its tuning was done with NNUE on (Basilisk's
  caveat).
- Rarog's evaluation already follows `sf_11`'s design, and PROCESS names
  `sf_11` as the classical evaluation's reference until a later one "is
  shown to be the best there is". Nothing here shows that.

### 3. The gain is won in the middlegame, not in endings (RAR-E74 A)

The donor arm's wins minus its losses, by the material band where its own
score passed 150 for good (E.0's rule):

| Match | 45+ | 25–44 | 10–24 | 0–9 |
|---|---|---|---|---|
| `sf_11`, equal nodes | +337 | +108 | +32 | −55 |
| `sf_16`, equal nodes | +357 | +118 | +25 | −43 |
| `sf_16`, `3+0.03` | +294 | +100 | +47 | −50 |

- About three quarters of the net is decided with 45 pawns of non-pawn
  material or more. Read from the base's own scores, the endgame bands hold
  0.10 to 0.22 of it. The frozen reading is **broad**.
- Under 10 pawns of material the donor arm loses more decided games than it
  wins. That is where its trees grow most, and where Rarog's own endgame
  terms sit.
- E.4.2's endgame repairs cannot reach this gain; its prior band was +3 to
  +10.

### 4. Game results from Rarog's games cannot see the gap (RAR-E74 B)

Cross-entropy against results relative to Rarog's, each evaluation with its
own K; negative where the donor predicts better; game-clustered 95%:

| Corpus | Cohort | `sf_11` | `sf_16` |
|---|---|---|---|
| `hce-v4-tb` validation, the fitting corpus | all | −0.06 [−0.28, +0.15] | +0.28 [+0.05, +0.50] |
| | 45+ | **+0.58** [+0.18, +0.96] | **+1.00** [+0.59, +1.39] |
| | 25–44 | **+0.65** [+0.30, +1.03] | **+1.03** [+0.65, +1.42] |
| | 10–24 | −1.37 [−1.76, −0.99] | −0.66 [−1.06, −0.26] |
| The kernel's games (RAR-S139) | all | −1.82 [−2.37, −0.96] | −1.36 [−1.92, −0.50] |
| | 25–44 | **+1.61** [+1.06, +2.23] | **+2.08** [+1.52, +2.67] |
| Games the donor arm played (RAR-O13) | all | −5.0 | −4.9 |
| | 45+ | −8.6 to −9.3 | −7.9 to −8.9 |

- Where the donor's games are won, heavy material, the fitting corpus says
  Rarog's evaluation is the *better* predictor by 0.6% to 1.0%. Play says
  the donor is about +75 there.
- On games the donor arm shaped, the donor leads by 5% to 9%. A corpus's
  results favour the evaluation that produced its games, by more than the
  gap being measured. RAR-E70's "ensemble-like" reading, where Rarog's
  evaluation was ahead of `9587eeeb`'s above six men on this corpus, fits
  the same bias.
- Stockfish 19's labels, which no engine here produced, order the
  evaluations as play does: the donors lead by +0.024 to +0.029 skill, and
  most at 45+ (+0.035 to +0.040; E.0).
- By the frozen rule, results on `hce-v4-tb` are **not admissible** to
  choose a rebuild's fit. A whole-surface fit on this corpus would pull a
  donor port's middlegame weights back toward Rarog's (inferred).
- Distillation is not the escape either: RAR-E03's Stockfish-distilled fit
  improved held-out loss by 4.9% and lost 17.11 Elo.

## Unknowns

- How much of the +70 is the donor's *structure*: its families and their
  definitions, such as EVAL-3's x-ray and pin maps. How much is its
  *weights*, fitted by Stockfish's game-based tuning where Rarog's were
  fitted on its own results (finding 4)? No read separates them, and a
  rebuild takes both together.
- A native port's node cost. Inside the kernel the donor costs about 9% of
  the node rate. E.0's native profile had Stockfish 16's evaluation at 77
  ns a node in its own tree against Rarog's 95 in the kernel, a different
  population. A port in Rust could land either side.
- What refitting the kernel's evaluation-coupled margins and `LazyMargin`
  on the new evaluation adds or costs. RAR-O13 ran them unfitted.
- Whether Rarog's own endgame knowledge beats the donor's under 10 pawns
  of material, or the donor arm's losses there are its tree tail.
- Results at longer controls: nothing was read beyond `3+0.03`.

## Hypotheses

- **H1, a rebuild in `sf_11`'s form.** It carries most of RAR-O13's gain into
  Rarog at a native cost no worse than the bridge's. A fit by games over a
  bounded set of live coordinates keeps it, where a static fit on Rarog's
  own results would erode it.
- **H2, repairs.** A few families carry most of the gain and can be repaired
  in Rarog's structure. Against it: the gain is broad across the middlegame
  (finding 3), and the static compass that would pick the families points
  the wrong way there (finding 4).
- **H3, weights, not structure.** Rarog's structure is close enough, and a
  fit on an admissible objective would close most of the gap without a
  port. Not testable cheaply: it needs an admissible static objective,
  which does not exist yet, or an SPSA over Rarog's whole surface, which
  rule 4's zero-game sweep would first have to show is not flat.

## Interaction map

- *The donor evaluation's producers:* the board, and per-thread pawn and
  material tables (the port's own, as Rarog has its pawn cache now).
- *Its consumers in the kernel:*
  - the wrapper: correction history, material weighting, optimism and
    halfmove damping (kept in RAR-O13);
  - the transposition table's stored raw evaluation, keyed without the
    clock;
  - the kernel's static-evaluation pruning margins, fitted at D.7.11 on
    Rarog's scale, and `LazyMargin`, held at 475 since the classic search;
  - singular extension's and the TT estimate's thresholds. Their rates
    moved on the donor's evaluation: futility 7.0% against 20.1% of asks,
    the TT estimate 54.5% against 19.4% on bench position 15 (RAR-O13
    probes).
- *Duplicated signals:*
  - `sf_16`'s halfmove damping and the kernel's;
  - the donor's specialised endgames and scale factors against Rarog's
    draw verdicts, KPK and mop-up;
  - the donor's `Initiative` (`sf_11`) or `Winnable` (`sf_16`) term against
    the kernel's optimism.
- *Shared scale:* the K-matched scale (684/1024 for `sf_11`) puts the
  donor's scores on Rarog's. Every margin in the kernel reads that scale.

## Cheapest discriminating tests

Already run: RAR-O13's three matches and RAR-E74's two zero-game reads.
What remains is part of the unit, cheapest first:

1. *Parity by fixture.* The port against the bridge's `donor-parity` table
   on B.0's 10,039 positions, every value equal. No games.
2. *Native cost.* The port's node rate on the bench and on the game
   positions (D.16's two reads) against the bridge arm and the head. If it
   is slower than the bridge arm, H1's cost assumption is false. No games.
3. *The tree.* `tree_stats.py` at depth 14 on the 200 game positions
   against the bridge arm's 1.217. The port should reproduce the bridge
   arm's tree. No games.
4. *Rule 4's sweep.* A zero-game sweep of the live coordinates (the scale,
   the family scales, `LazyMargin`, the static-evaluation margins) for
   activation and curvature, before any tune.

## Prospective prediction

Not frozen here; the unit's registration freezes it. The packet's prior,
for the maintainer's decision: the rebuild, fitted and gated, at +55, 80%
band [+30, +80], at `3+0.03` against the head. The basis is RAR-O13's +68.8
at the donor's cost in the kernel, less something for an imperfect port
and an uncertain fit. Its most likely failure is the fit: a static fit
would erode the gain. The second most likely is the cost: a Rust port
slower than the bridge's C++.

## Falsifiers and stop conditions

- Parity by fixture not reached. The unit stops and is not gated in part.
- The port's node rate is below the bridge arm's. H1's cost premise fails:
  record it, and decide again on what remains.
- Rule 4's sweep is flat or monotone on every family scale. No SPSA; the
  unit goes to its gate seeded and scaled, which needs rule 2 waived for
  this unit (see the decision).
- The gate. A rebuild is a large prior, so the bracket is wider than
  `[0,3]`, stated at registration.

## Decision

**Recommendation: `READY_FOR_IMPLEMENTATION` for a rebuild in `sf_11`'s
form, as one unit, once the maintainer decides questions 3 and 4.**

- **Donor, `sf_11`.** It is level with `sf_16` in play, and cheaper in the
  kernel by about the size of `sf_16`'s point lead. It predicts results
  better on every corpus and is the simpler port: no root-score input and
  no duplicated damping. It is closer to Rarog's present design and is
  PROCESS's named reference. This is a choice between equals, not a
  measured superiority. `sf_16` remains the alternative if the maintainer
  prefers the later evaluation.
- **Form, a rebuild.** The gain is broad across the middlegame, so
  endgame repairs cannot reach it. A family-by-family programme would
  steer by a static objective that ranks Rarog's middlegame above the
  donor's. The rebuild's outcome has been played: about +70 at the donor's
  own cost. I recommend against starting E.4.2's or E.4.3's repairs before
  the rebuild. If the rebuild proceeds they fold into it as its endgame
  and king clusters, as PLAN says.
- **Fit, by games, not by the current static pipeline.** Rule 2's path:
  the donor's constants as seeds converted through the measured scale,
  then SPSA over the cluster's live coordinates after rule 4's zero-game
  sweep, then the gate. Results on `hce-v4-tb` choose nothing (RAR-E74).
  E.4.0's method question goes on as research, starting from an
  admissibility test: an objective may fit only if it orders known
  evaluations as play does. Both a mixed-producer corpus, such as games
  between different engines, and independent labels are candidates, and
  RAR-E03 is the prior against distillation alone.
- **What would change this:**
  - a disclosure posture that rules out seeding Rarog's evaluation from
    the donor's constants. Then the choice is H3: a fit of Rarog's own
    structure by an admissible objective, which does not exist yet;
  - parity not reached;
  - a native port slower than the bridge;
  - a flat sweep, which leaves the unit to a gate without a tune.

## Implementation handoff

Proposed for PLAN if the maintainer accepts. It is drafted here, not
entered into PLAN:

- **Goal:** Rarog's evaluation rebuilt in `sf_11`'s form in Rarog's own
  structure, at parity with the donor by fixture before any change of
  value, then fitted by games and gated as one unit.
- **Semantics:** `sf_11`'s `Eval::evaluate` at `eval`'s root, contempt
  zero, specialised endgames and scale factors included, on Rarog's score
  scale through the frozen K-matched scale (684/1024). The kernel's wrapper
  stays as RAR-O13 measured it.
- **Why it should work here:** RAR-O13 (+74.4 at equal nodes for `sf_11`;
  +68.8 on the clock for `sf_16`, the same bridge) and RAR-E74.
- **Files:** `src/eval/` rebuilt; the pawn and material tables in their
  per-thread form. The kernel's search is not changed by the port.
- **Invariants and tests:**
  - parity with the bridge's `donor-parity` table on every position;
  - the eval cache and pawn cache exact (`tests/eval_cache.rs`);
  - symmetry under colour flip;
  - the texel trace's reconstruction gate, if the trace is kept;
  - AGENTS' per-node rules: no allocation, no clone, no lock;
  - the new bench fingerprint recorded.
- **Instrumentation:** the bridge stays as the reference for parity; rate
  probes and `tree_stats.py`.
- **Cheap qualification:** the four tests above, D.16's two speed reads,
  fmt, clippy and the full test matrix.
- **The maintainer's gate:** registered after the sweep and the tune, with
  a wide bracket for the large prior.
- **Non-goals:**
  - no search change beyond the evaluation-coupled coordinates the sweep
    selects;
  - no `sf_16` terms;
  - no Rarog family kept beside the donor's unless the sweep or a screen
    shows it is worth its place, and then as a named addition with its
    own read;
  - no static fit on `hce-v4-tb`.
- **Adjacent mechanisms not to change:** the TT's evaluation slot, the
  correction history's update, and the time manager.
