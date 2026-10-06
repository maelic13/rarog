# C.0.3 — the lazy path: why the full evaluation loses in play, and what Phase C does about it

`dev` at `89b7a5c`, 2026-10-06; engine source `ee02ed1`, unchanged. Research
leaf, class `R2`. **No engine source changed.** Raw results are in
`analysis/artifacts/c03-lazy/` (ignored storage; the files are named below).

## Decision needed

RAR-E19 refuted the plan to remove the lazy gate: playing the full,
fitted evaluation in the nodes the gate now skips costs **−104.5 ± 10.6 Elo**
at `3+0.03`, of which speed explains about 11. Three decisions remain:
whether the played function changes at all (a repair that keeps the speed),
whether the fits should describe the function the engine plays, and what
the loss teaches Phase C about evaluation changes under a search fitted to
the current evaluation.

## Known evidence

**From RAR-E19.** `LazyMargin` 2000 against 600 on one tune build:
−104.5 ± 10.6 Elo, pentanomial [155, 395, 338, 103, 9]; in the games
−5.5% NPS and 0.9 ply less mean depth; `bench 13` +3.23% nodes; one
middlegame position +63% nodes at depth 14.

**Fixed-depth tree over game positions (today, zero games).** 72 positions
sampled from RAR-E19's own games (every 25th game, one position between
ply 16 and 90, not in check; `positions_v1.epd`), depth 12, each from a
cleared table, `tune,diag` build at `ee02ed1`, counters exact at stride 1
(`lazy_tree.py`, `tree_d12.txt`):

| Reading | 600 → 2000 |
|---|---|
| total nodes | 9,738,182 → 10,526,208, **+8.1%** |
| per-position ratio | geometric mean 1.057, median 1.12, min 0.31, max 3.58; 42 grew by more than 5%, 23 shrank |
| by root score at 600 | \|score\| ≤ 100: +13.4%; 101–300: +9.1%; > 300: −0.9% |
| by material | middlegame (non-pawn material ≥ 40): **+22.4%**; 20–39: +1.3%; endgame (< 20): **−11.5%** |
| interior nodes / quiescence nodes | +9.0% / +6.3% |
| quiescence stand-pat cuts | +1.1% |
| quiescence delta pruning | 9,556 → 102,171, **×10.7** |
| razoring | **+43%**; SEE pruning +23%; null-move attempts +19%, cuts +17%; ProbCut cuts +16.5% |
| aspiration | fail-highs +6.7% (+10% nodes), **fail-lows +16.7% (+25% nodes)** |
| correction history | updates +14.5%, residual sum +35%: **+18% residual per update** |
| static evaluation against the TT's | refinements +13%, delta sum +38%: +22% per refinement |
| static evaluation against quiescence | +21% per refinement |
| root score at depth 12 | \|difference\| median 22 cp, max 509 cp; identical in 9 of 72 |

The +63% was one position. Over game positions the tree grows 8%, and it
grows where the gate rarely fires at the root (balanced middlegames) and
shrinks where it fires in most nodes (endgames).

**Static, full function against played function (today, zero games).**
`rarog-texel --dump-scores` gives the full evaluation of every
`hce-v3-tb/validation.csv` row; RAR-E17's `scores.csv` gives the played one
(the oracle DLL, lazy gate on, its in-evaluation rule-50 damping undone).
Loss is the tuner's, K fitted once per function (`full_vs_played.py`,
`full_vs_played.txt`):

| Cohort | Rows | Full | Played | Played against full |
|---|---:|---:|---:|---:|
| all | 194,444 | 0.088696 | 0.088882 | +0.21% |
| below the gate (identical within 2 cp) | 162,181 | 0.105527 | 0.105554 | +0.03% |
| **above the gate** | **32,263** | **0.004089** | **0.005071** | **+24.0%** |
| above, decided label | 31,918 | 0.002126 | 0.003148 | +48.1% |
| above, drawn label | 345 | 0.185750 | 0.182942 | −1.5% |
| above, phase ≥ 16 / 6–15 / < 6 | 1,167 / 10,328 / 20,768 | | | +10.2% / +25.7% / +27.1% |

Above the gate the full function is **more extreme by a median 282 cp**
(mean +363) in the direction of the played score; mean |score| 1,504
against 1,141 cp. 16.6% of the rows are above the gate; they hold 1% of
the total squared error, and 99% of them have a decided label.

**So:** statically the full function is the better outcome predictor
exactly where the engine does not play it, and it loses about 100 Elo
when played there. The harm is not static quality.

## Unknowns

- The per-node share of the loss against the share that is tree size and
  time: RAR-E20 (registered, below).
- Which skipped block carries the per-node harm (imbalance, the bishop pair,
  mobility, threats, king safety); separating them needs an engine switch
  and was not built, because no decision here depends on it (see Decision).
- Whether the mismatch is specific to decided positions, whose labels are
  saturated and whose rows carry almost no gradient, or general.

## Hypotheses

| | Hypothesis | Predicts | Standing |
|---|---|---|---|
| H1 | **Co-adaptation.** The B programme fitted the search's cp-valued margins and its correction history against the played function. Fed the full one, static scores above the gate are larger and further from search-backed values, so razoring, delta, SEE and null-move fire out of calibration and aspiration fails low more often | per-node loss; the counters as observed; a large loss at equal nodes | supported by the counters; RAR-E20 reads the size |
| H2 | **Magnitude pile-up.** Fitted positional terms add about 360 cp on top of a material edge that already decides the position, so won positions are ordered by how many bonuses they collect | the same per-node loss; the remedy would be a cap or scale on the total, not a search retune | compatible; not separated from H1 by any read here |
| H3 | **Non-quiet nodes.** Threat and hanging terms at quiescence nodes make stand-pat pessimistic | more quiescence nodes, fewer stand-pat cuts | **not supported**: stand-pat cuts +1.1%, quiescence nodes +6.3% against interior +9.0% |
| H4 | **Time and tree only.** The loss is the 8% larger tree and the slower nodes | about zero at equal nodes | RAR-E20 |
| H5 | **Worse function.** The fit is wrong above the gate | higher static loss there | **refuted**: the full function's static loss is 24% lower there |

## Interaction map

Evaluation → static evaluation → reverse futility, razoring, futility,
null-move, ProbCut, SEE and delta margins (all cp-valued, all fitted in
Phase B against the played function); correction history, keyed by pawn and
piece structure, learning the played function's residual; the TT's stored
evaluation; aspiration windows around scores on this scale. Inside the
evaluation: the lazy gate reads the tapered running sum; the mop-up reads
`(mg+eg)/2`. Fitting: the `texel` build forces the gate off, so every fit
since RAR-E06 describes the full function and 16.6% of its rows are
positions the engine evaluates differently in play.

## Cheapest discriminating tests

Done, zero games: the fixed-depth tree read with counters (H3 out, H1
supported, the +63% corrected to +8%); the static comparison (H5 out).
Registered: **RAR-E20**, the full against the lazy function at 150,000
nodes a move, 2,000 games, the same binary and options as RAR-E19
(`tools/sprt.ps1 -Nodes`, as RAR-O05's equal-node arm). It removes the
clock and keeps every per-node decision.

## Prospective prediction

RAR-E20: **−70 Elo**, 80% band [−95, −40]; probability above −30: 0.10,
above 0: 0.03. Frozen in the entry before any game.

**Read 2026-10-06: −110.0 ± 11.5 Elo** (W-D-L 334-719-947, 2,000 games,
0 faults), equal to the equal-time loss within error (difference +5.5 ±
15.6). The "below −95" falsifier fired: the lazy arm's speed and smaller
tree were worth about nothing at this control, and the whole cost is per
node. H4 is refuted; H1 and H2 stand and are not separated from each other
or from a worse-for-this-search function by any read here. Calibration is
in the entry.

## Falsifiers and stop conditions

- A RAR-E20 read above −30 refutes H1/H2 as the main mechanism and makes
  H4 the lesson: time and tree size decide at this control.
- Below −95: the tree and speed were worth less than assumed; the per-node
  effect is nearly the whole −104.
- RAR-E19 is closed for removing or raising the gate on this search; no
  further read of that change without C.10's joint tune first.

## Decision

1. **The played function: `NO_CHANGE`.** Removal is refuted (RAR-E19).
   Raising the gate is the same change. The repair RAR-E19 named, imbalance
   and the bishop pair computed before the gate, adds magnitude in exactly
   the positions where added magnitude is what the search punishes (H1,
   H2); it is not built. The step at the gate stays, and the fits are
   brought to the played function instead (2).
2. **The fits describe the played function from C.2 on.** The `texel`
   build honours `LazyMargin` instead of forcing the gate off, so the
   fitted function is the one the engine plays; the trace macros inside the
   skipped block then never fire above the gate and the reconstruction
   check covers it. This is a repair of the fitting instrument, not a
   hypothesis: fitting the function as played is Texel's premise. Its
   static effect is small (the rows above the gate hold 1% of the loss and
   the full function's advantage there is 0.2% of the total); its effect on
   the weights is not known and is measured inside C.2's baseline gate,
   stated there as conflated with the new corpus. Handoff below.
3. **A standing zero-game screen for every evaluation candidate in Phase
   C:** the fixed-depth tree read on game positions (`lazy_tree.py`'s
   method: about 70 positions from the latest gate's games, depth 12, both
   arms, the per-position ratio distribution and the correction-residual
   counters), recorded beside `bench 13`, which under-reads it (3.2% against
   8.1%, and 22% in middlegames). PLAN's cluster shape, step (6), carries it.
4. **Recommendation to the maintainer, not decided here:** when a
   cluster's equal-time gate fails, register an equal-node companion read
   before counting the rejection under rule 6; a candidate that is better
   or level at equal nodes and loses at equal time, or whose tree read and
   residual counters move as RAR-E19's did, is a search mismatch to hold
   for C.10, not a rejected cluster. If two clusters show the mismatch, a
   bounded retune of the cp-valued margins before C.10 (one rule-7c block,
   search coordinates only) is the cheaper remedy, under rule 7b's
   case-by-case argument. RAR-E20's size sets how much weight this
   deserves.

**C.0.3 closed 2026-10-06 on RAR-E20's record.** The registered use of the
reading fired: PLAN's cluster shape now carries the fixed-depth tree read
and the equal-node companion before a failed gate counts under rule 6.
Decision 4 was adopted on 2026-10-06 as PROCESS's *Evaluation change under
a fitted search* (gate 1, a margin block over a fixed 27-coordinate
surface, gate 2, the `10+0.1` read), together with the unit shape for
Phase C (the first unit is king safety with winnability and scaling, C.3).
Nothing here blocks C.1 or C.2's generation.

## Implementation handoff (decision 2; owner C.2, after C.1 has moved the code)

- **Goal.** Under `--features texel`, `evaluate` takes the lazy path by the
  same rule as every other build.
- **Semantics.** `lazy` is computed from `self.lazy_margin` in every build;
  the `#[cfg(feature = "texel")] let lazy = false;` arm goes. The tuner's
  `Evaluator::default()` carries the default margin, which is the played
  default; nothing else in the tuner changes. The trace of a position above
  the gate holds material, piece-square, pawn, passer-advance and tempo
  coefficients only, and `raw` equals the played score less the mop-up's
  frozen part, as today.
- **Why local evidence says it is right.** 16.6% of the fitting rows are
  positions the engine evaluates without the activity block and imbalance;
  their gradient has been shaping weights the engine never plays there.
- **Invariants and checks.** Non-`texel` builds are untouched: the exact
  fingerprint by construction, confirmed by `bench 13`. Under `texel`:
  `trace_reconstructs_eval_exactly_over_random_playouts` passes; one new
  test takes a position above the gate (a queen against a rook, say) and
  asserts that every activity and imbalance coefficient in its trace is
  zero and that `raw` matches the played `evaluate` of a non-`texel`
  evaluator's semantics (through the frozen-part bookkeeping); `rarog-texel
  --verify` on `hce-v3-tb/validation.csv` passes; `--feature-support`
  reports the activity families supported on fewer rows than before, by
  about the 16.6% share. `fit_complete.ps1`'s smoke (an absurd vector moves
  the source and the fingerprint) is unchanged.
- **Non-goals.** No change to `LazyMargin`'s default, to the skipped set,
  to the tuner's schedule or to any weight.

## What this did not establish

Any Elo for a repaired lazy path; which skipped term carries the per-node
loss; whether a cap on the total above the gate (H2's remedy) would help,
which would be an evaluation change gated like one; whether the C.10
retune recovers the −104, which is RAR-E19's retry trigger.
