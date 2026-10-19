# Phase C review, 2026-10-19: C.9's skip, C.10's design, and the HCE against Stockfish's classical evaluation

**Status: research review, class `R3`, on branch `c9` at `e09ce37` (engine
sources equal `6ade0e6`'s, **10,521,141 / EBF 2.521**). Zero games. Two
zero-game reads were registered before exposure and read once (RAR-E70,
RAR-E71); RAR-O07 received the addendum its registration asks for. No engine
code changed; no PLAN or GUIDE edit was made, the proposals are in section
5.** Every number below carries its layer: static held-out outcome loss,
node counts and firing rates are not Elo and have no exchange rate to it.

## Verdicts in one place

| Question | Verdict |
|---|---|
| 1. C.9 closed `NO_CHANGE` | **Stands, at the static layer.** The instrument was right for the question C.9 asks (a game tune of these coordinates with the weights fixed) and the frozen rule was applied correctly. Its limit is stated, not refuted: a coordinate with no outcome information can still move the search (the map's threshold moves 8% of rows by 3 cp and the tree by 84%), and the one-at-a-time sweep reads co-adaptation as "fitted" for the taper. The inventory missed the generic mop-up, which fires on 48% of the rows; RAR-E71 swept it (section 1.3). Do not reopen C.9. |
| 2. C.10 in progress | **Run it, with four corrections.** The firing-rate check selects a surface and forecasts nothing, and its contrast against the B.9 head mixes the evaluation's effect with the two margin blocks. The 17 history-unit coordinates go into the pilot, which is the only instrument that can decide them. The maintainer's relevance condition is not yet checked: a script is ready. The expected value of +8 Elo is a judgement whose lower band is optimistic; +5 with 80% band [−1, +15] is what the evidence supports. The blend stays out. |
| 3. HCE against Stockfish's classical | **Statically at parity above six men on Rarog's own positions, and about 100 Elo behind inside Stockfish's search.** On the current evaluation Rarog's score alone predicts self-play outcomes better than Stockfish's classical evaluation alone above six men (−0.44 ± 0.14%, RAR-E70), the reverse of the 2.4.0 reading (+1.43%). What remains of the same-search gap is mostly the meter's shape mismatch and the exact low-material knowledge the tablebases supply in play. The recoverable hand-crafted share is one lead, king in the middlegame band (+0.28% static), worth one zero-game attribution, not a unit on present evidence. **Recommendation: C.10, then C.11 as planned**, with that attribution run alongside C.10's pilot; RAR-O07 stays at C.11. |

## 1. C.9: the curvature sweep and the `NO_CHANGE` close

### Verdict

C.9's close stands. The decision followed its frozen rule, the numbers
reproduce from the raw dumps, and the wires held (checked independently on
2026-10-19: base K 1.6372, base loss 0.092712, the four coordinates'
gains recomputed from `tools/results/c9-sweep-20261019/*.scores` with an
independent K refit agree with `read.json` to four decimals; the base equals
`c132/head_played.csv` on all 194,444 rows). What the review adds is the
scope of the claim and one omission from the inventory.

### 1.1 Is static held-out outcome loss the right instrument?

**Yes for C.9's question; it answers a narrower question than "is there an
SPSA surface".** The three instruments on offer measure three different
things:

- *Static outcome loss, K refitted per point* asks whether the coordinate
  carries outcome information the fitted linear weights cannot reach. That
  is what an evaluation coordinate is for, and it is the layer the whole
  fit optimises. The K refit per point is the right choice: without it a
  coordinate that only rescales the score (the index cap, the taper's slope)
  would read as a gain or a loss for a change the search is invariant to.
  RAR-E69 confirms the refit changed no classification (largest difference
  0.021 points).
- *`bench 13` plus WAC at 100k* (PLAN rule 8's sweep) is a search-cluster
  instrument. For evaluation coordinates WAC measures tactics the residue
  does not touch, and `bench 13` is a fixed-depth node count that is
  chaotic under small evaluation changes: RAR-E69's `thr` 150 moved 7.8% of
  rows by 2.6 cp and grew `bench 13` by 84%; RAR-E19's `LazyMargin` 2000
  grew `bench 13` by 3.2% and one depth-14 search by 63%. Rule 8 itself says
  the bench column is a fingerprint. RAR-S86 and RAR-S89 record WAC
  predictions missing four of four and favouring a rejected candidate.
  Neither is a strength proxy; neither would have been a better arbiter.
- *Games* are the only arbiter of strength, and a 2,000-game read resolves
  about ±9 Elo (RAR-S100), far above what any of these coordinates could
  plausibly be worth.

**The limit, stated.** RAR-E19, RAR-E20 and RAR-E67 show the asymmetry: a
static *gain* does not carry into games under a fitted search (a 24% lower
loss above the gate lost 105 Elo per node; a 0.05% frozen-test gain lost
13 Elo). The converse, a static *null* implying no game effect, does not
follow either, because a coordinate can act through the search while
carrying no outcome information: `thr` and `ocb` are exactly such
coordinates, and the tree's response to `thr` shows the search feels them.
So C.9's `NO_CHANGE` means "no static surface for a game tune of these four
with the weights fixed", and that is what AGENTS' SPSA rule asks for (a
flat or monotone zero-game surface is evidence against the tune). A
game-level surface could exist only through search interaction, and that
interaction is what C.10 refits the search to: with the evaluation frozen
and the search re-fitted to it, a game tune of `thr` or `ocb` afterwards
would be chasing the search's co-adaptation, not the evaluation's
information. The record should say so (section 5).

**What would change this verdict:** a game record after C.10 showing losses
attributable to a nonlinear evaluation coordinate (RAR-E59's king-danger
lead, large middlegame scores converting 2.7 points under the rest, is the
one measured candidate; its retry trigger is already written). The cheapest
test then is a pair of 2,000-game switch reads on `KS_INDEX_CAP` (1,200
against 1,600, statically free at −0.012 ± 0.026%), RAR-S100's form,
25 minutes. It is not worth running now: a true effect of a few Elo is
invisible at that budget, and nothing in the record says the effect exists.

### 1.2 One coordinate at a time, the weights fixed: the taper

**The bias is real and it is the right read for C.9.** Bending the taper
with the 1,133 linear weights fixed at the values fitted under γ = 1 must
lose: every middlegame and endgame weight was fitted against the straight
share, so the sweep measures co-adaptation, not whether a bent taper with
weights refitted under it would predict better. RAR-E69's own prediction 2
said as much ("the weights were fitted under the linear taper, so bending
it costs", 0.7) and the result confirmed it, both ways, at every bend.

Three things keep this from reopening C.9:

1. *C.9's question is a game tune of the residue with the weights fixed.*
   An SPSA of γ in games would face the same co-adaptation, so the sweep is
   the right proxy for the tune C.9 would have run. A refit under another
   γ is a different leaf: a C.8-style cycle (an hour's fit, a `[0,3]` gate
   of about 12,000 games), not a tune.
2. *The static lever is bounded and small.* The quadratic through
   γ 0.75, 1 and 1.5 puts the co-adapted optimum at γ ≈ 1.07 for 0.008% of
   the loss. RAR-E58 and RAR-E59 bound what a phase re-shaping could add
   with everything refitted: the slope among decided scores is about 0.05%
   of the whole file's loss on the played score and under the line on the
   full score once errors count games (3.26 SE); remaining game length
   carries more of it than phase does (+1.556 against +0.809 once both are
   in), and that is label behaviour, not shape. A refit under γ ≠ 1 has at
   most that to recover, and C.8's two refits (RAR-E60, RAR-E61) have
   already moved K and the magnitudes since.
3. *The residual on the current head agrees.* RAR-E70 reads
   `rarog+phase_slope` at +0.175 ± 0.028% over all rows (per-row SE) and
   +1.167 at |rarog| > 500 only: the same calibration C.7 closed, on the
   refit.

**Is a refit under a changed taper worth running, given RAR-E58/E59?** No.
I recommend against it now. The cheapest test, if the maintainer wants the
question closed by measurement rather than by bound: refit the 1,133
weights at γ = 0.75 and at γ = 1.5 on `hce-v6-tb` (two runs of
`fit_complete.ps1`, about an hour each, the maintainer's compute), and read
each fit's frozen-test loss against RAR-E61's with game-clustered errors.
*Frozen prediction:* neither beats the γ = 1 fit by 0.10% at four clustered
SE (0.85); both are within ±0.05% of it (0.6). *Retry trigger:* a
regenerated corpus from the C head whose full-score phase slope reaches the
line with clustered errors.

### 1.3 Is the inventory complete?

**No. Two omissions, one of which matters and was swept.** The sources
under `src/eval/` hold these unfitted nonlinear or hard-coded quantities
(the fitting manifest `hce_fit_manifest_v6.tsv` fits 1,133 linear weights,
fixes 98 for structural reasons and routes the 23 king-danger coordinates
and both map scales to the nonlinear pass):

| Quantity | Where | In RAR-E69? | Assessment |
|---|---|---|---|
| Taper shape, `KS_MAP_THRESHOLD`, `KS_INDEX_CAP`, general opposite-bishop base | `mod.rs`, `king.rs`, `endgame/mod.rs` | swept | as recorded |
| **Generic mop-up weights (5, 4)** | `endgame/mop_up.rs`, `apply_mop_up`, the branch that is not a minor-piece mate | **no: listed as five men or fewer** | fires on every path whenever the running `(mg + eg)/2` passes 200 cp, whatever the material; RAR-E62's dump saw it on 93,575 of 194,444 rows (48%); up to 86 cp, in the frozen part the tuner reconstructs but cannot move. **Swept as RAR-E71.** |
| Mop-up gate (200 cp) | the same | no | a third continuous coordinate of the same term; not swept here (interacts with the weights; RAR-E71's `mop=0` point removes the branch, which answers the first question) |
| Phase weights `PHASE_W = [0, 1, 1, 2, 4, 0]` | `material.rs` | no | integer, discrete; decides which pieces count toward the taper and that pawns do not. RAR-E63 found men and pawns carry a dependence the taper does not, and RAR-E66 found it lives in transient material. Recorded, not swept: a change is structural and the men evidence points away from it |
| Map divisors (4096, 16) | `king.rs` | excluded as redundant | correct: both map scales are fitted |
| `KS_MAP_SCALE_MAX` (150) | `king.rs` | not a coordinate | a bound; both scales (102, 18) sit inside it |
| Minor-mate mop-up constants (360, 20, 10), `KNNKP_*`, `PURE_OCB_MIN_PAWNS`, the recognisers' fixed scales | `endgame/` | excluded as five men or fewer, discrete, or measured by C.5 | correct: `scale_endgame` asks the recognisers only at five men or fewer or with a bare king; C.5's instruments measured them |
| Trapped-bishop phase gate | `pieces.rs` | excluded as discrete | correct |
| `PIECE_VALUES` (100, 320, 330, 500, 900) | `mod.rs`, read by SEE, quiescence futility and delta, the correction's material count | not an evaluation coordinate | the search's piece scale; its consumers' margins are on C.10's surface |
| `LAZY_MARGIN`, rule-50 damping | `mod.rs`, search | correctly routed to C.10 | |

The mop-up's inventory error is the kind AGENTS warns about: the comment in
`mop_up.rs` says the gate "fires in plenty of middlegames", and RAR-E62 had
counted it, but the inventory took the term's name for its scope.

**RAR-E71 (registered `f999900`, run once; all 25 runner steps and the
reader exited 0; `read.json` `bd955c73…`):** the wires held (every row has a
game; the base equals `c132/head_played.csv` on all 194,444 rows and reads
10,521,141 / EBF 2.521; every point moves 47.6% of the rows). The weights
are **fitted**: removing the branch loses **−0.125 ± 0.017%** of the loss
(7.5 game-clustered SE), halving it −0.041 ± 0.008, 0.75x −0.017 ± 0.004;
1.5x +0.017 ± 0.008 (2.0 SE) and 2x +0.008 ± 0.015 are within the line.
The K refit matters here: at the head's K the removal reads −0.196 and the
doubling −0.052, so about a third of the term's static effect is a uniform
inflation of decided scores that the sigmoid's scale absorbs; the rest,
0.12% of the loss, is information the linear weights cannot reach (the
king piece-square tables were fitted with the push present and encode it
only on average). `bench 13` moved +1.1% at the removal and +41% at 2x. By
the frozen use C.9's close stands and the inventory is corrected. Static
loss, not Elo: a term that inflates decided scores is exactly the kind
RAR-E19 and RAR-E20 found statically better and worse in play, and nothing
here says the head's weights are wrong in play, only that removing them
would be a change with a measured static cost and no static case.

### 1.4 Should C.9 reopen?

**No**, on present evidence, with the inventory corrected in its record.
RAR-E71 found the omitted coordinate at its static optimum, so the omission
hid no surface. The two things that would reopen it are written as retry
triggers: a game record after C.10 attributable to a nonlinear evaluation
coordinate (RAR-E59's king-danger conversion lead), or a regenerated-corpus
refit whose full-score phase slope reaches the line under clustering.

**Competing explanations the review considered and could not exclude:** that
a joint surface of the four coordinates has curvature the one-at-a-time
sweep cannot see (implausible when two carry no outcome information and the
other two sit at their marginal optimum; untested); that the bench
hypersensitivity to `thr` is a game-level effect rather than fixed-depth
chaos (RAR-E19's per-position tree distribution is the instrument that
would tell; the C.0.3 positions and `tree_read.py` exist, about ten minutes;
not run because no decision depends on it).

## 2. C.10: the joint search tune

### Verdict

C.10 should run. C.10.1's two reads were done mechanically and read
correctly against their rule; their meaning is narrower than the card
states in two places, and the surface has one open decision the pilot
should make. The plan's shape (widenings, pilot, chained blocks stopped by
hand) is PROCESS's and rule 7c's. The expected value is overstated at the
low end.

### 2.1 Is the firing-rate check valid?

**Valid as a surface-selection device, which is all PLAN C.10 asks of it;
not valid as evidence of what the evaluation did to the search.**

- *Mechanically sound.* Stride-1 `bench 13` counters of `--features diag`
  builds from `git archive` of `ee02ed1` and the C head, both fingerprints
  reproduced, the B.9 dump equal to B.6's archived one on all 190 shared
  counters, per interior node, the same `rate_diff.py` method B.6 used
  (`counters-status.txt`, `rate_diff.txt`). 56 of 192 counters moved 10% or
  more.
- *Bench positions, not game positions.* The 40 bench positions are a
  fixed mixture; game positions at `3+0.03` from UHO have a different phase
  mix, and RAR-E19 showed `bench 13` under-reads tree effects on game
  positions (3.2% against 8.1% at depth 12 on 72 game positions, 22% in
  middlegames). For a 10% threshold on a *ratio of rates* that matters less
  than for a tree-size read: a mechanism whose rate moved 40 to 60% on the
  bench (quiescence futility 0.62, delta 0.52, SEE flips 1.59, the
  correction residual 0.54) will not reverse on game positions. The
  mechanisms near the threshold (null move 0.86, bad-noisy futility 0.87,
  singular 0.76 to 0.90, ProbCut 1.12) could cross it either way on a
  different population. The cheapest check is the same counters on
  `c03-lazy/positions_v1.epd` at depth 12 (the diag builds take a `go
  depth` and dump counters; about ten minutes). *Frozen prediction, if run:*
  of the 13 mechanisms in the card's table, at least 10 keep the sign of
  their movement on game positions (0.75); none of the four within 10% on
  the bench (reverse futility, razoring, LMP, LMR applied) crosses 10%
  (0.6). It does not change the surface: the coordinates it could add
  (LMP's count terms, LMR's structural terms) are the ones the pilot can
  cheaply include anyway (section 2.3).
- **The contrast is confounded, and the card does not say so.** The B.9
  head's 28 cp margins are not the C head's: `c2margin` and `c3margin`
  (RAR-E24, RAR-E25) retuned them, moving `QsFutilityMargin` from 178 to
  232 and `LazyMargin` from 600 to 423 among others. So "quiescence
  futility skips fell 38%" is the evaluation's new shape *and* the margin
  that was retuned for it, and the correction residual halving is partly
  the smaller scores (5 to 11%). For selecting a surface this is harmless:
  every retuned coordinate is on the surface as cp-valued anyway. For the
  card's hypotheses it matters: H-misfit ("the margins sit where the B-era
  evaluation put them") is false for the 28 margins, which sit where the
  C.3 evaluation put them; six units and a 5% scale change have moved since
  then, which is the real case for the tune. The rate table cannot
  distinguish H-misfit from H-absorbed, and the card should not read it as
  evidence for either.

### 2.2 Is the cp scale read valid?

**Valid, and borderline against the skip rule's wording.** Both heads'
played evaluation on 194,444 `hce-v6-tb` validation rows, the C side equal
to C.9's head point byte for byte (`scale-status.txt`). The C evaluation is
smaller by mean |score| 0.948 (5.2%), by median per-row ratio 0.969 (3.1%),
by least-squares slope 0.953 (4.7%); in the middlegame band 0.891 by mean
and 0.835 by slope. The skip rule says "within 5%" without naming the
statistic: the mean ratio is just past it, the other two inside. It does
not matter, because the first condition (56 counters moved 10% or more)
fires on its own, but the card's "5.2% smaller, past the skip rule's 5%"
reads a borderline number as decisive. Two cautions for the tune: the rows
are the C-era datagen's quiet positions, which is the right population for
a static scale but not the search's interior nodes; and a 5 to 17% scale
change is small next to what the two margin blocks already absorbed
(RAR-E24's refit moved the whole evaluation's scale and shape at once).

### 2.3 The 83-coordinate surface

**Mostly correct under the maintainer's three conditions; one open class,
one unchecked condition.**

*Condition 1, every coordinate relevant and moving `bench 13` one step each
way:* not yet checked. `analysis/artifacts/c10/bench_steps.py` does it
from `surface.tsv` with the tune build (`rarog-c10-tune.exe`, bench
10,521,141 at the defaults); each coordinate costs two benches, about four
seconds each, so the 83 take about twelve minutes and the 100 with the
history coordinates about fifteen. Smoked on two coordinates (each side
moved the bench: `AspirationDelta` 8 → 16,365,909, 20 → 10,118,152;
`CoreHistQuietBonusCap` 1,292 → 12,104,044, 1,772 → 13,465,986). The
command is in section 4.

*Condition 2, the 17 coordinates in history units* (nine `CoreHist*`,
`CoreHpSlope`, `CoreLmpHistory`, the two SEE history terms, the two LMR
history terms, and `CoreFpHistory` and `CoreBnfpHistory`, the last two
already in by their mechanism's rate): **decide them in the pilot, not by
argument.** The evidence either way:

- *For leaving them out:* their consumers' rates did not move. Late-move
  pruning 1.01, both SEE prunings 0.95 and 1.03, LMR applied 0.98, quiet
  cutoffs 0.99: the move-loop decisions that read history fire at the B.9
  rates on the C head, which says the history distribution at those
  decisions did not shift much. B.6's journals read no gradient on these
  coordinates at the end of RAR-S78 (the second half of the run within its
  null on the 74 untouched coordinates), and no cluster since retuned them.
  Histories are self-normalising in their own units (bonus and malus caps),
  so an evaluation's scale does not reach them directly.
- *For putting them in:* they have been unfitted through two search
  clusters and the whole evaluation programme; the evaluation changes which
  moves cut and so what the tables learn; and no counter measures the
  history distribution at the pruning decisions (B.8 removed the history
  pruning counter B.6's table used; `counters_c_stride1.txt` has none), so
  "its mechanism moved under 10%" is literally unmeasured for them, as the
  maintainer says.
- *What would decide them cheaply:* the pilot. SPSA's cost in games does
  not grow with the dimension; 17 more coordinates on 83 add perturbation
  noise to every gradient estimate (about a fifth more variance), which a
  128-iteration pilot tolerates. PROCESS step 6 then re-audits: a
  coordinate that the pilot does not move a full step is left out of the
  blocks; one it moves goes in. The alternative, a diag counter for history
  at the pruning decisions, is a search-code change in Phase C, which the
  freeze forbids even under a feature flag.

**Recommendation:** run the pilot over 100 coordinates (the 83 plus the 15
history-unit coordinates now out), and let the re-audit set the blocks'
surface. Register the pilot that way; the blocks' surface is registered
after it, as PROCESS has it.

*The aspiration growth trio* (`AspGrowthPct`, `AspGrowthHighPct`,
`AspGrowthAdd`, never fitted, staged inert at the old behaviour): in by the
fail-low rate (1.32). Keep them in. RAR-S90 read the aspiration shape at or
next to the top on the B head with three reads; a 32% rise in fail-low
nodes is a reason to let the pilot look. Note `AspMaxFails` stays out
correctly: seeded at 20 it never fires first, so it is inert at its seed,
exactly what condition 1 excludes.

*The singular family* (17 coordinates): 14 are in because they are in
evaluation units (the singular margin and every double and triple extension
margin are cp); only `CoreSingExactSpan`, `CoreSingMulticutLerp` and
`CoreLmrSingularCap` entered on the family's rate (`lmr_singular_term`
0.76, `singular_extension_truncated` 0.89, `singular_extend_one` 0.90).
Those three flags are weak (the counters are exact sums, so the weakness is
the 40-position sample, not noise in the count) but the cost of three
coordinates in a pilot is nil. Keep them; the re-audit decides.

*Every exclusion, with its reason, checked:* the seven `Tm*` (D.1's; not a
node population: correct); `CoreEvalRule50Damping` and `CoreNmpVerifyDepth`
(rarely reached at `3+0.03`: plausible, the rates are unmeasured; a
verification from depth 16 is near the root at a mean game depth of 15);
`CoreIirMinDepth` and `CoreSingularTtDepthMargin` (read by games, RAR-S96
and RAR-S80; depth gates: correct); `NmpMinNonPawnPieces` (a material
guard: correct); `AspMaxFails` (inert at its seed: correct); `CoreRfpLerp`,
`CoreHindsightDeepenReduction`, the LMP count terms, the SEE depth terms,
LMR's structural terms (not in evaluation units, mechanism within 10%:
consistent with the rule; the game-position cross-check in 2.1 is the only
thing that could move them, and the pilot can hold them at no cost in
games if the maintainer prefers completeness over a tighter gradient).
`QsCountLimit` fixed at 3 by RAR-S100's two reads (+3.1 ± 8.7 and
+1.9 ± 9.2 Elo, neither a direction): correct by its frozen rule, with the
honest note that ±9 Elo cannot tell 2, 3 and 4 apart.

### 2.4 Is the plan sound?

*The seven range widenings* each address a seed within two steps of a rail
that is not a natural bound; checked against `params.rs`: `AspirationDelta`
5 → 1 (seed 14, step 6), `LazyMargin` 200 → 100 (423, step 113),
`CoreFpLinear` 10 → 0, `ProbCutSeeGapScale` 100 → 200 (at the rail; above
100 the filter is stricter, a legitimate direction), `CoreCorrUpdateMax`
8192 → 4096 with the step halved to 248 (the rail is the lower end at 128;
narrowing the range is the right fix), `CoreSingularMargin` 192 → 96 with
step 6 (seed 17), `CoreSingExactSpan` 2 → 1 (seed 4, step 2). Sound, and
behaviour-neutral by construction (the clamp only). One caution on
`LazyMargin`: RAR-E19 measured 600 → 2000 at −104 Elo, so the response is
steep and nonlinear; a perturbation of ±113 around 423 may dominate the
difference signal in early iterations. It was on both margin blocks and
moved 600 → 423 there, so its gradient should be small now; if the pilot's
journal shows it dominating, a smaller step for it is the fix, registered
before the blocks.

*The pilot* (128 × 30 = 3,840 games, about 50 minutes) is PROCESS step 5's
default shape on Colosseum's 15 slots. Pilot theta is never promoted. Its
stop rule in the card (no full-step mover and a journal inside the
sign-redraw null) is RAR-S95's statistic and is right.

*The blocks* (2,000 × 30, chained from the last block's rounded centres,
stopped by the maintainer after reviewing movement, more blocks registered
beyond three) are rule 7c as amended on 2026-09-26 and the maintainer's
decision of 2026-10-19. One thing to register before block 1 so it cannot
drift: the surface and steps the blocks use are the re-audit's output, and
the registration names them. Nothing is baked between blocks.

*Is +8 Elo (80% band +2 to +18, H1 at 0.65) defensible?* **As a judgement,
partly; the low end is optimistic.** The evidence for the centre: one
margin block after one unit bought +19.3 ± 9.3 Elo over the untuned margins
(RAR-E24, on a flagged unit whose refit moved scale and shape at once); the
next block, `c3margin`, bought nothing measurable (C.3's gate 1 +34.2
against gate 2 +32.7); B.6 estimated a restart on an unchanged search at
+3 ± 3; RAR-S78's restart of 82 coordinates with a fresh schedule gained
+13.1 ± 5.4 when seeds were far from the optimum. Since `c3margin`, six
units landed without a block and the scale moved about 5% (11% in the
middlegame). That supports a centre of +5 to +8. The band's low end of +2
is not supported: B.6's own range for a restart reaches zero, the second
margin block measured zero, and RAR-E67's experience (a refit that moved
magnitudes 3 to 11 cp lost 13 Elo) says the search's co-adaptation to the
*current* head is already partly done by the six gates, which accepted only
candidates the frozen search liked. I would write **+5, 80% band [−1, +15],
H1 under `[0,3]` with probability 0.5**. The cost side is right: 16 to 43
hours, and at a true +4 Elo the `[0,3]` SPRT needs about 47,000 games
(RAR-M10), which is overnight compute.

### 2.5 Keeping the blend out

**Agree, for the card's reasons and one more.** It is an evaluation change
(rule 2 forbids mixing); a blended tune would need two arms and double the
blocks for a candidate whose best reading is −6.3 ± 6.6 Elo (RAR-E68); the
blend's own case is a margin block of its own, which is a unit, not a
tune. The additional reason: even scaled to the head's magnitude the blend
measured no gain (RAR-E68, 4,000 games), so its upside is unmeasured while
its downside is measured twice. The retry trigger is written (the
maintainer asks for it as a unit after C.10; a changed datagen budget or
label source).

### 2.6 Before or alongside C.10

1. **Run the relevance check** (`bench_steps.py`, about fifteen minutes)
   before the pilot; it is the maintainer's condition 1 and the command is
   ready (section 4). A coordinate flagged `inert` or `half` is decided
   before registration.
2. **Register the pilot over 100 coordinates** (section 2.3).
3. **Alongside the pilot, zero games:** RAR-E72, the king-band attribution
   (section 3.3), the one hand-crafted lead RAR-E70 found. It does not
   touch the search or the tune and it settles the only "more HCE work
   first" case before C.11.
4. Not before C.10: RAR-O07 (section 3.4), a game-position counter
   cross-check (2.1), a `KS_INDEX_CAP` switch read (1.1). None changes a
   decision C.10 depends on.

## 3. Rarog's HCE against Stockfish's classical HCE

### Verdict

**Statically, on its own positions and above six men, Rarog's evaluation
is now at parity or ahead of Stockfish's classical evaluation; inside
Stockfish's search it is about 100 Elo behind, and most of that gap is not
recoverable by hand-crafted terms.** The recoverable share on present
evidence is one lead, king in the middlegame band, at about 0.28% of the
static loss, worth one zero-game attribution and, only if that reads
conditioning, a unit.

### 3.1 The evidence, by layer

*The same-search meter (Elo inside Stockfish's search, equal nodes):*
+181.7 ± 19.0 on the 2.4.0 evaluation (RAR-O05), +129.4 ± 17.1 on the C.3
evaluation (RAR-O06; 52.4 ± 25.6 closed by C.2's refit and C.3's king unit,
0.8 of their gate sums). RAR-O07 is registered for C.11 with its frozen
prediction (+122 after C.4 and C.5.2) and, as of this review, the owed
addendum for the units since: C.6 (+6.4 ± 3.6), C.8's cycles (+11.0 ± 4.9,
+6.7 ± 3.7), C.13.1 (no read), all first-crossing estimates in Rarog's
search, transferred at 0.4 to 0.9: **+108, 80% band [+90, +126]**. C.12 and
C.10 do not reach this meter (it runs Rarog's evaluation in Stockfish's
search).

*Its caveat:* the meter runs a foreign evaluation through a search fitted
to Stockfish's evaluation shape. RAR-E19 and RAR-E20 measured what a shape
mismatch alone costs in a fitted search: a statically better function lost
−104.5 ± 10.6 Elo at equal time and −110.0 ± 11.5 at equal nodes through
Rarog's own search fitted to the played one. That is an analogue, not a
measurement of the meter's mismatch, and the hybrid converts Rarog's
centipawns into Stockfish's units by `PawnValueEg / 100` (the `hybrid`
README), so the scale is handled while the shape is not. RAR-O06's finding
that 0.8 of two gates' gains transferred says the meter does see evaluation
quality; it cannot say how much of the remaining +108 is quality.

*The static donor residual (held-out outcome loss, not Elo), on the current
evaluation (RAR-E70, read once from RAR-E67's `--within` report on
`hce-v4-tb` validation, one Stockfish version):*

| Cohort | `stockfish` alone over `rarog` | `rarog+stockfish` | `rarog+all_families` | sum of the nine single families |
|---|---:|---:|---:|---:|
| all rows | +0.41 ± 0.14 | +1.92 | +1.14 | +0.28 |
| men ≥ 7 | **−0.44 ± 0.14** | +1.22 | +1.21 | +0.34 |
| phase ≥ 96 (opening) | +0.63 ± 0.16 | +1.03 | +1.02 | +0.49 |
| phase 32–95 (middlegame) | −0.23 ± 0.22 | +1.10 | +1.06 | +0.49 |
| men ≤ 6 | +31.7 | +31.7 | +16.7 | +6.0 |

On the 2.4.0 evaluation the first column read +2.17 over all rows and
+1.43 at seven men or more (RAR-E17, RAR-E18). Above six men the sign has
turned: Rarog's score alone is the better predictor of these outcomes.
Stockfish's edge over all rows is its exact low-material knowledge and the
size of its winning scores (men ≤ 6: +31.7%, of which magnitude +12.9%),
which the tablebases and recognisers supply in play (RAR-E34: KQ-KR,
KBB-K, KQ-K level with Stockfish at 200k and 600k nodes; the engine probes
WDL at interior nodes).

*The units accepted since C.3, in Rarog's own search at `3+0.03`:* C.5.2
+4.5 ± 8.3 (STC read), C.4.3 +13.0 ± 5.3 (about 5 of it speed), C.6
+6.4 ± 3.6, C.8 cycle 1 +11.0 ± 4.9, cycle 2 +6.7 ± 3.7, C.13.1 none;
with C.2's +35.9 ± 9.3 and C.3's +32.7 ± 8.9 (gate 2), Phase C's evaluation
work sums to about +105 Elo of gate estimates, first crossings overstating,
inside the budget row's 100 to 160 and above C.0's 30 to 90 prior for the
family clusters. C.12 added +42.3 ± 9.5 at equal time by speed alone.

### 3.2 Where the remaining gap sits, by family, magnitude and calibration

From RAR-E70 (percent of Rarog's held-out loss in the cohort; per-row SE;
the line is 0.10% at four SE, above the cohort's magnitude gain):

- **King, middlegame band: +0.282 ± 0.052 (5.4 SE), above the band's
  magnitude gain (+0.079).** The one family with signal above six men. It
  was +1.58 before C.3 (RAR-E18), +0.357 after C.3 on the C.5.2 head
  (RAR-E30, reported there and not pursued because C.4 owned threats and
  mobility), +0.282 now after two refits. Over all rows +0.062; at seven men
  or more +0.081; with queens +0.140 ± 0.024.
- **Space, opening band: +0.137 ± 0.034 (4.0 SE), above +0.040,
  provisional.** C.13.1 removed Rarog's own space term as fitted to zero;
  the donor's is a different definition (safe central squares behind pawns,
  weighted by piece count and blocked pawns). It read +0.15 ± 0.04 in the
  same band on the 2.4.0 evaluation (RAR-E18) and has not moved. At the edge
  of the per-row line it will not survive game clustering (1.04 to 2.18
  times the per-row SE on these corpora, RAR-E62).
- **Mobility, opening band: +0.139 ± 0.039 (3.6 SE), under the line**,
  down from +0.184 on the C.5.2 head. Closed inside C.4 (RAR-E31); not
  reopened.
- **Threats +0.057, passed +0.051, pawns +0.019, pieces +0.012 at seven men
  or more:** the units took their signals (threats was +0.122 before C.4.3,
  passed +0.204 in its band before C.6).
- **Magnitude: +0.515 over all rows, +0.312 at seven men or more, +12.9 at
  six or fewer.** A monotone rescaling of Rarog's own score; minimax is
  invariant to it, so in Rarog's search it acts only through cp margins
  (C.10's) and in the meter through Stockfish's margins (the mismatch).
- **Calibration: the phase slope +0.175 over all rows and +1.167 ± 0.188 at
  |rarog| > 500; the Material row +0.319 ± 0.056 there.** Both repeat
  RAR-E57's readings on the C.6 head (+1.316, +0.333); RAR-E58 attributed
  the Material row to the lazy gate (gone on the full score; `LazyMargin`,
  C.10) and RAR-E59 the slope to calibration with remaining game length,
  under the line with game-clustered errors. Not knowledge.
- **The joint remainder (+1.2 above six men, four times the sum of the
  singles) is ensemble-like**: adding Rarog's score to Stockfish's gains
  +1.65% of Stockfish's error above six men, more than Stockfish adds to
  Rarog (+1.22%). Two differently shaped estimators of the same quantity
  average their errors; that gain belongs to neither and no ported term
  recovers it.

### 3.3 How much is recoverable, and the cheapest measurement

*Recoverable by hand-crafted terms:* the king band, at most. C.3's king
unit took a +0.66% residual (whole file, 16 SE) to +0.08 and measured
+32.7 ± 8.9 Elo in Rarog's search after its margin block; the remaining
+0.28 in one band is a fraction of that at the static layer, and RAR-E59's
king lead says the donor's and Rarog's maps differ in how their *large*
scores convert (king-dominant large scores 2.7 points under the rest, 2.9
clustered SE), which is magnitude, not conditioning. If the band's residual
is magnitude, a term cannot fix it and the margins or the map's scales own
it; if it is conditioning (which parts of the donor's danger index Rarog's
reproduces by fixture but weights differently in play), a king refinement
unit is a C.3-shaped job with a `[0,3]` gate.

*The cheapest discriminating test, proposed as RAR-E72 (zero games, about
fifteen minutes of compute, an hour of agent work; not run here):* the
donor's king term and its sub-terms against the current head in the
middlegame band, with Rarog's own danger-map output as a free direction in
the base. RAR-E21's instrument (`king_subterms.py` with the patched
`9587eeeb`, `sf9587_king_subterms.patch`) collects the donor's danger
components on the `hce-v4-tb` validation rows; `rarog-texel --dump-groups`
writes Rarog's `king_danger` output per row (added for RAR-E59); the model
is RAR-E58's nested held-out logistic with the spline comparator and
RAR-E59's game-clustered bootstrap. *Readings:* (A) `donor_king | C`, (B)
`donor_king | C + rarog_king_danger` (the re-weighting control), (C) leave
one donor component out of (B). *Rule:* conditioning when (B) is at the
line (0.10% at four clustered SE) in the middlegame band; magnitude when
(A) is at the line and (B) is not; closed when (A) is under the line
clustered. *Frozen prediction:* (A) at the line clustered (0.55); (B) under
it (0.6), so **magnitude or re-weighting, not conditioning (0.6)**; if (B)
holds, the components whose removal costs most are safe checks and the weak
ring, as in RAR-E21 (0.5). *Use:* conditioning opens a king-refinement
research card before C.11, decided with the maintainer; magnitude is
recorded for the map scales and C.10's margins and closes the lead;
nothing is built from the read.

*Recoverable by the search:* C.10 is that recovery; it moves nothing on the
meter.

*Not recoverable:* the ensemble remainder, the exact low-material knowledge
(handled in play), and the meter's shape mismatch.

### 3.4 Should RAR-O07 be read before C.10?

**No; keep it at C.11, as registered, with the addendum now appended.** The
reasons are not procedural:

- *No decision depends on it.* C.10 runs by its own skip rule, which
  fired. C.11's freeze is already in effect in substance: C.9 closed
  `NO_CHANGE`, the blend is out, and no evaluation change is scheduled; a
  surprising reading (no transfer since RAR-O06, at or above +129) would not
  reopen a unit, because the static layer names none above the line except
  the king band, whose test is zero-game and independent of the meter.
- *C.10 does not move it.* The meter runs Rarog's evaluation inside
  Stockfish's search; the evaluation is frozen for C.10 (rule 2). The number
  at C.11 is the number now, so reading early buys no information C.11
  would lack and spends a 15-minute idle-host slot the maintainer has three
  of a day.
- *The prediction is complete now.* The registration's one time-sensitive
  obligation was the addenda for units accepted since 2026-10-11, owed
  before the match; none had been written. They are appended (`f999900`),
  so the match can run whenever C.11 opens without an incomplete frozen
  prediction.

What would change this: if the maintainer wants the number to decide whether
the king-refinement unit, should RAR-E72 read conditioning, runs before or
after C.11. Then the order is RAR-E72 first (zero games), and RAR-O07 only
if it reads conditioning.

### 3.5 Recommendation

**C.10, then C.11 as planned**, with RAR-E72 run alongside C.10's pilot and
the classical evaluation treated as frozen from now (the record should say
so; section 5). **Against more HCE work first:** no family is above the
line above six men except king in one band, and its cheapest test is
zero-game and does not block C.10. **Against freezing sooner in form:**
C.11's checkpoint reads (conversion, NPS, the pool gauntlet) need C.10's
head, and the freeze in substance has already happened. The phase order
C → D → E → F is not re-proposed; nothing here is new evidence against it.

## 4. Decisions for the maintainer

| # | Decision | Recommendation |
|---|---|---|
| 1 | C.9: reopen on the inventory omission? | **No.** RAR-E71 read the omitted mop-up weights as fitted (removal −0.125 ± 0.017% of the loss at 7.5 clustered SE; 1.5x and 2x within the line). Record the corrected inventory and the static-layer scope of the skip in PLAN C.9 (section 5). |
| 2 | C.9: refit under a changed taper (RAR-E58/E59)? | **No, not now.** The lever is bounded at about 0.05% of the loss and is calibration; the retry trigger is a regenerated-corpus refit whose full-score slope reaches the line clustered. If measured anyway: two fits at γ 0.75 and 1.5, prediction neither beats γ = 1 by 0.10% at four clustered SE (0.85). |
| 3 | C.10.3: the 17 history-unit coordinates | **Into the pilot (100 coordinates); the re-audit sets the blocks' surface.** No cheaper evidence exists without a search-code change. |
| 4 | C.10.3: the maintainer's relevance condition | **Run before the pilot:** `python analysis/artifacts/c10/bench_steps.py tools/test_engines/rarog-c10-tune.exe analysis/artifacts/c10/surface.tsv analysis/artifacts/c10/bench_steps.tsv --also CoreHistQuietBonusSlope,CoreHistQuietBonusCap,CoreHistQuietMalusSlope,CoreHistQuietMalusCap,CoreHistMalusIndexScale,CoreHistContBonusCap,CoreHistNoisyBonusCap,CoreHistTtCutoffBonusCap,CoreHistFailLowBase,CoreHpSlope,CoreLmpHistory,CoreSeeQuietHistory,CoreSeeNoisyHistory,CoreLmrQuietHistory,CoreLmrNoisyHistory` (about fifteen minutes; any `inert` or `half` row is decided before registration). |
| 5 | C.10: the card's expected value | **Register +5, 80% band [−1, +15], H1 at 0.5**, with the margin-block confound (section 2.1) stated in the card. The plan itself is unchanged. |
| 6 | C.10: the blend | **Out**, as decided; retry trigger as written. |
| 7 | Phase C: more HCE work before C.11? | **Only if RAR-E72 reads conditioning.** Register and run RAR-E72 (zero games) alongside the pilot; otherwise freeze. |
| 8 | RAR-O07: read before C.10? | **No; at C.11**, with the addendum appended today. |
| 9 | PLAN and GUIDE | **Apply the section 5 edits** in the next documentation commit (not made here, as asked). |

## 5. Proposed PLAN and GUIDE changes (not applied)

1. **PLAN C.9's record**, after "C.9 closes `NO_CHANGE`": *"The skip is at the
   static layer: these coordinates carry no outcome information the fit
   cannot reach, and a game surface could exist only through the search,
   which C.10 refits to the frozen evaluation. RAR-E69's inventory placed
   the mop-up among the five-men recognisers; its generic branch fires
   wherever the running score passes 200 cp (48% of the held-out rows), and
   RAR-E71 swept its two weights: fitted (removal −0.125 ± 0.017% of the
   loss at 7.5 game-clustered SE, 1.5x and 2x within the line), so the
   omission hid no surface. The phase weights
   (`PHASE_W`) are the one discrete unfitted coordinate left unswept; the
   men evidence (RAR-E63, RAR-E66) points away from them."*
2. **PLAN C.10.1's record**: add *"The firing-rate contrast against the
   B.9 head includes the `c2margin` and `c3margin` blocks' retuning of the
   28 margins, so it selects a surface and does not attribute the movement
   to the evaluation alone; the cp scale's 5.2% is by mean |score| (3.1%
   by median, 4.7% by slope)."*
3. **PLAN C.10.3**: *"The pilot runs over the 83 coordinates plus the 15
   history-unit coordinates now out (100); the re-audit sets the blocks'
   surface. The relevance condition is checked by
   `analysis/artifacts/c10/bench_steps.py` before the pilot."* And the
   registration's expected value as in decision 5.
4. **PLAN C.11**: *"The classical evaluation is frozen in substance from
   2026-10-19 (C.9 `NO_CHANGE`, the blend out, no unit scheduled); C.11
   records it formally with its checkpoint reads. RAR-O07's addendum of
   2026-10-19 completes its prediction (+108, band [+90, +126])."* If
   RAR-E72 is registered, name it under C.11 as the one pre-freeze read.
5. **GUIDE's checkpoint, "Evaluation residual (static layer)"**: append
   *"On the current evaluation (RAR-E70, 2026-10-19, `hce-v4-tb` labels):
   above six men Rarog's score alone predicts outcomes better than
   Stockfish's classical evaluation alone (−0.44 ± 0.14%); the donor's
   joint edge there (+1.2%) is ensemble-like, four times the sum of its
   single families; king in the middlegame band +0.28 ± 0.05 is the one
   family with signal; magnitude +0.31 at seven men or more. Not Elo."*
6. **GUIDE's holds**: nothing new; RAR-O07's addendum obligation is
   discharged.

## Evidence and provenance

- C.9 verification: `tools/results/c9-sweep-20261019/` (`read.json`
  `c2e2ef54…`, every `status.txt` step 0), recomputed on 2026-10-19 from
  the raw `.scores` with an independent K refit; registration `af1ae26`,
  amendment `57c3c74`, close `ffbe3d1`.
- RAR-E70: `analysis/artifacts/c136/report-head.json` (`5728f2d0…`), read
  once after `f999900`; the full printout is in `docs/experiments/RAR-E70.md`.
- RAR-E71: `analysis/artifacts/c91/` (`patch_point.py` `def1f006…`,
  `run.sh` `8be14561…`, `read.py` `6e7dc0b7…`), `tools/results/c91-mop-20261019/`.
- C.10: `analysis/artifacts/c10/` (`rate_diff.txt`, `scale.txt`,
  `surface.tsv`, `counters-status.txt`, `scale-status.txt`); `bench_steps.py`
  `87f555dd…`, smoked on two coordinates.
- Hybrid oracle scaling: tag `oracle/hybrid`, `hybrid/README.md`.
