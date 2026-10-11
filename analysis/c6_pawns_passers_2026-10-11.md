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

## Sub-steps spawned under C.6 (classes in PLAN)

- C.6.1 Residual step (`V`).
- C.6.2 The unstoppable-passer test against Syzygy and in the corpus (`V`).
- Later steps (a sub-term attribution, a unit, its refit and gate) are
  spawned by what these two find.
