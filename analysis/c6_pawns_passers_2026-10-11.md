# C.6 — pawns and passers: what the donor's two families still carry on the current head, and whether the unstoppable-passer test should change

Research record for PLAN C.6 (`I2`, state `RESEARCH`), opened 2026-10-11 by
the maintainer's order of work (PLAN, Phase C, *Order amended 2026-10-11*).
Zero games so far. Static outcome loss ranks questions and accepts nothing;
nothing here is Elo.

## Decision needed

Whether C.6 builds a pawns-and-passers unit (one refit, the margin block when
flagged, one gate), repairs only the unstoppable-passer test it owns, or
closes `NO_CHANGE`. PLAN opens the cluster with its own residual step and
closes it `NO_CHANGE` if that step finds none; the unstoppable-passer test is
a defect confirmed in the source (C.0, 2026-10-05), decided on its own
evidence.

## Known evidence

Donor-direction residual (Stockfish `9587eeeb`'s per-term trace beside
Rarog's played score, two-fold held-out logistic models, `hce-v4-tb`
validation, 194,444 rows), in percent of Rarog's held-out squared error. A
family *has signal* at 0.10% or more at four standard errors (RAR-E17's
rule, as C.4.1 used it).

| Reading | Rarog's evaluation | Cohort | Passed | Pawns |
|---|---|---|---:|---:|
| RAR-E17 | 2.4.0 | all | +0.06 | +0.04 |
| RAR-E22 (one Stockfish version) | 2.4.0 | men ≥ 7 | +0.06 | +0.03 |
| RAR-E32's screen, head (C.5.2) | `73c4c17` | all | +0.091 | — |
| RAR-E32's screen, candidate = **the current head** | `6c73d65` | all | **+0.091 ± 0.013** | +0.028 ± 0.009 |
| same file, read for B8 (2026-10-11) | same | men ≥ 7 | +0.078 ± 0.014 | +0.029 ± 0.008 |
| same | same | men ≤ 6 | +0.348 ± 0.095 | +0.21 ± 0.12 |

**Exposure.** The last three rows were read before this card, to justify the
order of work (B8). None of them reaches the line on its own (+0.091 is
under 0.10; +0.348 is 3.7 standard errors). The other cohorts of the same
report (the phase bands, with and without queens, the |score| bands) have not
been looked at; C.6.1 registers them before they are.

### Rarog's two families today

`src/eval/pawns.rs` (pawn-cached): passed by relative rank (mg, eg),
supported passers, candidate passers, doubled, isolated, doubled-isolated,
connected and phalanx by rank, backward, lever, pawn islands. `src/eval/passers.rs`
(per evaluation): free-path, safe-path, free-stop and safe-stop bonuses per
rank, the unstoppable passer, rooks behind passers (own and enemy), blockade,
king proximity to the passer. The per-evaluation terms run below the lazy
gate only.

### The unstoppable-passer test

`eval_unstoppable_passers` counts a passed pawn with an empty path, against a
defender with no piece, when the defending king's distance to the
promotion square exceeds `steps` with the defender to move and `steps − 1`
with the pawn's side to move (`steps` = the pawn's moves to promote, one fewer
from its second rank). The rule of the square, counting the king's capture of
the new queen, catches the pawn at a distance of `steps + 1` with the
defender to move and `steps` with the pawn's side to move, so the engine's
threshold is **one tempo generous in both orders**: example, a pawn on a5
and the black king on e5 with Black to move (1…Kd6 2.a6 Kc7 3.a7 Kb7
4.a8=Q+ Kxa8) is counted unstoppable. The weight is 74 cp eg, one slot.
**Interaction found on 2026-10-11:** the term is computed after the lazy
gate, so in the played evaluation it fires only when the score before it is
under 600 cp; on the trace test's own position (a7 pawn, black king on h1)
the played score is 728 cp and the term is skipped, the full score 962.

### The donor's shape (Stockfish `9587eeeb`, `evaluate.cpp` and `pawns.cpp`)

Passed: a bonus by rank; above the third rank, the enemy king's distance to
the block square against our king's, weighted by rank; a free or safe path
and a safe or defended block square scaled by rank; less for a passer whose
block square is occupied or attacked; a file adjustment. No rule-of-the-square
term: the donor's search and its race handling do that work. Pawns: isolated,
backward, doubled, connected by rank with phalanx and support, weak and
unopposed, weak lever, blocked. Read for mechanism, never transcribed.

## Unknowns

1. Does either family carry signal in a cohort not yet read, above all the
   endgame band, where passers decide?
2. Is the engine's unstoppable test wrong often enough, on Syzygy truth and in
   the fitting corpus, that a definition change is worth a refit, given that
   the lazy gate already hides it above 600 cp?

## Hypotheses

- **H1 (passed carries endgame conditioning):** the donor's passed family has
  signal in the endgame band or without queens, where king proximity and path
  safety decide races.
- **H2 (pawns has none):** the pawn-structure family stays under the line in
  every cohort; Rarog's structure terms are already rich.
- **H3 (the defect is real but small):** the square rule fires on fewer
  positions with a clearly higher share of real wins, and the positions where
  the two rules disagree are a fraction of a percent of the corpus.
- Competing: (a) the passed signal is magnitude, not conditioning (it rises
  with |score| and vanishes inside a band); (b) the generous tempo is harmless
  because a pawn one tempo short still wins through the other pawns or the
  attacking king; (c) the lazy gate hides most of the defect in play.

## Interaction map

Producers: the pawn cache (passed set, attacks), the attack maps (path
safety), the king squares. Consumers: the lazy gate (the per-evaluation
passer terms run below it), the opposite-bishop rule (counts passers), the
search's margins (C.10) and the endgame scale factors. Shared signals: the
passer count feeds C.5.2's opposite-bishop scale; any change to the passed
set moves that rule. The unstoppable term overlaps the free-path, safe-path
and king-proximity terms, which the refit already weighs.

## Cheapest discriminating tests

- **C.6.1 (RAR-E39):** the residual step, read on the unseen cohorts of the
  current head's report. Zero games, seconds.
- **C.6.2 (RAR-E40):** the unstoppable test on Syzygy-labelled pawn endings
  (both move orders), the engine's rule verified row by row against the
  engine's own trace, then each rule's share of real wins, and its activation
  in `hce-v4-tb` validation with and without the lazy gate. Zero games,
  minutes.

Predictions, falsifiers and the frozen use are in the two registrations.

## Readings so far (2026-10-11)

- **RAR-E39 (C.6.1):** both families have signal: passed +0.204 ± 0.031 in
  the endgame band and +0.196 ± 0.035 without queens, pawns +0.121 ± 0.028
  and +0.141 ± 0.031; both nil above |score| 500 and with queens on. The
  residual is conditioning inside the band, not magnitude. Both go to the
  attribution (C.6.3).
- **RAR-E40 (C.6.2):** the square rule's counted passers are Syzygy wins
  0.740 of the time against the engine rule's 0.683, so it replaces the
  engine's rule inside C.6's unit (a definition change, fitted with the
  unit, never gated alone). Expect little from it: the disagreements in the
  corpus were won 84% of the time anyway, and the lazy gate hides 85% of the
  term's firings in play.

- **RAR-E41 (C.6.3):** the instrument (`tools/diag/pawn_subterms.py`)
  reproduced the donor's `Passed` and `Pawns` rows on all 194,444 rows. Three
  components carry (0.04 points at four standard errors): our king's
  distance to the block square and the next square (+0.131 ± 0.017 in the
  endgame band, +0.123 without queens), weak unopposed pawns (+0.060 ±
  0.013 in the endgame band) and the free-path bonus (+0.083 ± 0.017
  without queens, a shape Rarog already has: pricing). The enemy king's
  distance does not reach the line (+0.038). The donor's rank bonus is worse
  than nothing beside Rarog's (its removal gains 0.30 points), and its
  passed components with free weights gain four times its own weighting,
  so the refit's weights matter more than the donor's constants.

## Decision (2026-10-11)

**`READY_FOR_IMPLEMENTATION`** by RAR-E41's frozen use: a unit of three
changes, built together, refitted together, gated together.

## Implementation handoff (C.6.4)

**1. Passer king proximity in the donor's shape, replacing Rarog's form.**
For each passed pawn in Rarog's own passed set (the set is not changed) of
the side `us`, at relative rank `r` (0-based) of 3 or more, with
`w = 5r − 13`, `block` the square in front of it and `next` the square in
front of `block`, and `d(k, s) = min(chebyshev(k, s), 5)`:
- `passer_king_them_eg += sign · d(their king, block) · w`
- `passer_king_us_eg   −= sign · d(our king, block) · w`
- `passer_king_next_eg −= sign · d(our king, next) · w`, only when
  `block` is not the promotion square (`r ≠ 6`).

Three eg-only weights, one slot each, traced by those counts. Seeds from
the donor's 19/4, 2 and 1 internal units per `w` converted at 100/206 and
rounded: (2), (1), (1) cp per count. `eval_passed_pawn_king_proximity`'s
current body (one weight on the symmetric difference of the kings'
distances to the pawn itself, plus the frozen `rel_rank` constant carried
in `frozen_eg`) is removed with `passer_proximity_base`; nothing frozen
remains in this term. It stays where it runs now, after the lazy gate.

**2. Weak unopposed pawns.** In `eval_pawns` (pawn-cached), a pawn that
Rarog scores isolated or backward, and that has no enemy pawn ahead on its
own file, takes one penalty `pawn_weak_unopposed` (mg, eg), once per pawn
even when it is both isolated and backward. Seed (6, 13) cp (the donor's
(13, 27) at 100/206). Rarog's isolated and backward definitions are not
changed.

**3. The square rule** (RAR-E40): in `eval_unstoppable_passers` the
threshold becomes `steps + 1` with the defender to move and `steps` with
the pawn's side to move (was `steps` and `steps − 1`). Nothing else in the
term changes.

**Trace and tuner.** New fields `passer_king_them_eg`, `passer_king_us_eg`,
`passer_king_next_eg` (one slot each) and `pawn_weak_unopposed_mg`/`_eg`;
`passer_proximity_base` removed;
the texel reconstruction test and the activation test cover the new slots;
the tuner's group lists and clamps follow (`passer_proximity_base` leaves
`MISC` and its clamp); a manifest v5 frees the new slots under C.2's rules,
differing from v4 by exactly those rows (`fit_manifest.py`,
`--write-defaults`, `--feature-support`, `--audit-coverage`).

**Per-node cost.** The proximity loop visits the same passers as today
with three capped distance lookups instead of two; weak unopposed lives in
the pawn cache (free on a hit); the square rule changes a comparison. Expect
pooled NPS within ±0.5%.

**Invariants and tests.** Crafted positions for each proximity count
(including `r = 6`, where `next` is skipped, and the cap at 5), for weak
unopposed (isolated unopposed, backward unopposed, opposed, both isolated
and backward counted once), and RAR-E40's square-rule cases in both move
orders (the a5/e5 position no longer counts with the defender to move).
The eval and pawn caches stay pure functions of the position.

**Interactions.** The opposite-bishop rule counts passers from the same
set, which does not change. The proximity term overlaps the free- and
safe-path terms and the mop-up only through the fit. The lazy gate decides
where the per-evaluation terms (1 and 3) act, as today.

**Then (C.6.5, a separate registration before the fit):** the refit on
`hce-v4-tb` under manifest v5 (`fit_complete.ps1`, RAR-E32's settings), the
static screens (RAR-E39's donor screen re-read on the candidate's file and
RAR-E41's components on it, the magnitude read), the tree read, the pooled
NPS read, the PGO bake and gate 1 `[0,3]`, by PROCESS's *Evaluation change
under a fitted search*; C.6.6 holds the margin block when flagged, gate 2
and the `10+0.1` read. Prior: small; the families' residuals are about
0.2% of held-out loss in the endgame band, and RAR-E32's threats unit took
+0.107 to +0.037 for +13 Elo at gate 1 (about +5 of it speed).

## Sub-steps spawned under C.6 (classes in PLAN)

- C.6.1 Residual step (`V`).
- C.6.2 The unstoppable-passer test against Syzygy and in the corpus (`V`).
- C.6.3 Sub-term attribution (`V`), RAR-E41.
- C.6.4 The unit: block-square king proximity, weak unopposed, the square
  rule (`I2`), from the handoff above.
- C.6.5 Refit, static screens, tree and NPS reads, PGO bake, gate 1 (`V`).
- C.6.6 Margin block when flagged, gate 2, the `10+0.1` read, the ledger
  row (`V`).
