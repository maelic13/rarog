# C.4 — threats and mobility: what the donor's two families still carry on the C.5.2 head, which conditioning Rarog lacks, and whether a unit is justified

Research record for PLAN C.4 (`I2`, state `RESEARCH`), opened 2026-10-10
after C.5.2's acceptance (RAR-E29) with the same-search meter at +129.4
(RAR-O06). Zero games so far. Static outcome loss ranks questions and
accepts nothing; nothing here is Elo.

## Decision needed

Whether C.4 builds a threats-and-mobility unit in the donor's shape (one
refit, the margin block when flagged, one gate), builds one of the two
families only, or closes `NO_CHANGE`. PLAN opens the cluster with its own
residual step and closes it `NO_CHANGE` if that step finds none; the
programme document's prior is "threats under +5 Elo, below even odds;
mobility about 0" (section 10), written before the `hce-v4-tb` labels
moved mobility.

## Known evidence

Donor-direction residual of the two families, in percent of Rarog's
held-out squared error (± one standard error where recorded). A family
*has signal* at 0.10% or more at four standard errors (RAR-E17's rule).

| Reading | Rarog's evaluation | Labels | Cohort | Mobility | Threats |
|---|---|---|---|---:|---:|
| RAR-E17 | 2.4.0 (`ee02ed1`) | `hce-v3-tb` | all | +0.01 ± 0.00 | +0.07 ± 0.01 |
| RAR-E18 | same | same | best cohort | +0.11 ± 0.03 (phase 32–95) | +0.20 ± 0.04 (no queens) |
| RAR-E22 (one Stockfish version) | same | same | men ≥ 7 | +0.04 | +0.10 |
| RAR-E23 | same | `hce-v4-tb` | men ≥ 7 | **+0.21 ± 0.03** | +0.14 |
| C.3 pre-game screen (RAR-E25) | C.3 candidate | `hce-v4-tb` | all | +0.06 ± 0.01 | +0.11 ± 0.02 |
| same | | | men ≥ 7 | +0.07 ± 0.02 | +0.13 ± 0.02 |
| same | | | phase ≥ 96 | **+0.19 ± 0.04** | +0.14 ± 0.04 |
| same | | | phase 32–95 | +0.06 ± 0.03 | +0.11 ± 0.03 |
| same | | | phase < 32 | +0.02 ± 0.02 | +0.18 ± 0.04 |
| same | | | no queens | +0.02 ± 0.03 | **+0.20 ± 0.04** |

What the sequence says. The `hce-v4-tb` labels (B.9 search) raised
mobility from +0.04 to +0.21 at seven men or more on the same evaluation;
the C.2 refit on that corpus and the C.3 unit then took it back to +0.07,
leaving mobility's signal in the opening band. Threats sits at +0.11 to
+0.14 across evaluations and labels, largest without queens and in the
endgame band: a small, stable residual. For scale, king read +0.66 (+1.58
in the middlegame band) before C.3 and +0.09 after it; the king unit
measured +32.7 ± 8.9 Elo after its margin block. There is no exchange rate
from this layer to Elo, and no game-level defect of either family is on
record (the C.0 source audit's list, programme document section 12, names
none).

### Rarog's two families today

Threats (`src/eval/threats.rs`, weights in `src/eval/params.rs`, the C.3
fit):

| Term | Condition | mg / eg |
|---|---|---|
| `threat_minor/rook/queen` | any own-pawn attack on an enemy minor, rook or queen | 68 / 44 each (shared) |
| `threat_by_minor[victim]` | every enemy piece attacked by an own knight or bishop, by victim type | 0, 42, 77, 79, 70, 0 / 6, 28, 0, 0, 0, 0 |
| `threat_by_rook[victim]` | every enemy piece attacked by an own rook | 0, 24, 39, 4, 68, 0 / 11, 21, 30, 1, 18, 0 |
| `threat_hanging_refined[victim]` | enemy non-king piece we attack that is undefended, or attacked twice and defended once | 3, 23, 40, 30, 0, 0 / 48, 28, 14, 3, 0, 0 |
| `threat_safe_pawn_push` | per enemy non-pawn piece a pawn would attack after a push to a square no enemy pawn attacks | 34 / 2 |
| `threat_weak_piece` | per own piece attacked by a lower-valued enemy piece | −38 / 0 |
| `threat_restricted` | per square both sides attack that the enemy does not strongly protect | 8 / 0 |
| `hanging_minor/rook/queen` | the old flat hanging penalty | 0, 1, 1 (dead) |
| `slider_on_queen` (pieces.rs) | own rook or bishop on a line to the enemy queen through pawns only | 41 / 10 |

Mobility (`src/eval/pieces.rs`): one-hot per-count tables for knight,
bishop, rook and queen in mg and eg, fitted; the count is the piece's
attacks (true occupancy, no x-rays, no pin restriction) inside the
mobility area, which is every square not attacked by an enemy pawn and
not occupied by an own piece (`src/eval/attacks.rs`). The mg mobility sum
per side is an input of C.3's king-danger index.

### The donor's shape (Stockfish `9587eeeb`, `evaluate.cpp`)

Threats are gated by two sets: *strongly protected* (enemy pawn attacks,
or attacked twice by the enemy and not twice by us) and *weak* (enemy
pieces we attack that are not strongly protected). Minor attacks score on
weak pieces and on strongly protected non-pawn pieces; rook attacks on
weak pieces only; `Hanging` on weak pieces that are unattacked by the
enemy or (non-pawn) attacked twice by us; `ThreatByKing` on a weak piece
the king attacks; `WeakQueenProtection` on weak pieces the enemy queen
defends; `RestrictedPiece` as Rarog's; `ThreatBySafePawn` counts attacks
by pawns standing on *safe* squares (not attacked by the enemy, or
attacked by us); `ThreatByPawnPush` needs the push square safe in the
same sense and not pawn-attacked; `KnightOnQueen` and `SliderOnQueen`
count attacks on the squares around the enemy queen that lie in our
mobility area, are not our pawns and are not strongly protected, the
slider form needing a double attack, both doubled when the enemy has the
only queen. Its mobility area also excludes own pawns that are blocked or
on the two lowest ranks, the own king and queen, and the own king's
blockers (pinned pieces); bishop attacks x-ray through queens, rook
attacks through queens and own rooks; a pinned piece's attacks are cut to
the pin line. Its tables are per count and piece type, like Rarog's.

The conditioning differences that could carry the residual, by family:

- **Threats:** (T1) the weak/strongly-protected gating of minor and rook
  attacks, where Rarog scores every attacked piece; (T2) the safe-pawn
  and safe-push attacker conditions, where Rarog's pawn threat is
  unconditional and its push needs only a pawn-free push square; (T3)
  terms Rarog lacks: the king threat and the weak-queen-protection term;
  (T4) the queen threats on safe squares around the queen with the
  queen-imbalance doubling, where Rarog has an x-ray slider term only;
  (T5) hanging's exact definition (weak and either unattacked or
  double-attacked non-pawn), close to Rarog's refined term.
- **Mobility:** (M1) the area (blocked and low own pawns, own king and
  queen, pinned own pieces excluded); (M2) x-ray attacks through queens
  and own rooks; (M3) the pin-line restriction; (M4) table shape, which
  Rarog's fit already owns and which the refit on `hce-v4-tb` visibly
  moved (RAR-E23 to the C.3 screen).

## Unknowns

1. Which of T1 to T5 and M1 to M4 carries the residual; whether any part
   separates at all (RAR-E21 found 92% of king's in one component; the
   opening-band families in RAR-E18 were joint).
2. Whether the opening-band mobility residual is the area (M1) or the
   x-rays (M2), which have different costs and different side effects on
   the shared attack maps.
3. Whether threats' no-queens residual is T2 (pawn threats matter more
   without queens) or T1.
4. The per-node cost of the donor's area: pinned pieces are computed by
   move generation, not by the evaluator, and x-rays add two slider
   lookups per bishop and rook.

## Hypotheses

- **H1 (leading, threats):** the residual is conditioning, T1 and T2: the
  donor refuses credit for attacks on pawn-defended pieces and for pawn
  threats from unsafe pawns, which Rarog's unconditional counts cannot
  learn by re-pricing. Expected: the gated minor and rook terms and the
  safe-pawn term carry most of the threats gain by leave-one-out.
- **H2 (threats):** the residual is the terms Rarog lacks, T3 and T4.
- **H3 (leading, mobility):** the opening-band residual is the area (M1):
  blocked and low pawns and the queen's square are exactly what inflates
  counts in the opening, and the fitted tables cannot undo a definition.
- **H4 (mobility):** it is x-rays (M2), which only matter with queens on.
- **H0:** no component separates; the residuals are joint, and the choice
  is between the whole family in the donor's shape and `NO_CHANGE` on
  expected value.

## Interaction map

- **Shared producer.** Both families read `eval/attacks.rs`. Adopting
  x-ray attacks or the pin-line restriction in the producer would change
  `attacked`, `attacked2` and `attacked_by` for every consumer: C.3's weak
  ring, safe and unsafe checks and blockers, the hanging and restricted
  terms, passers' path safety. A C.4 design either keeps the producer and
  computes its own counts (cost) or changes the producer and accepts that
  C.3's inputs move under the refit; the choice is recorded before
  implementation and the king family's residual is re-read on the
  candidate (RAR-E25's screen) either way.
- **King danger.** The per-side mg mobility sum is a danger-index input;
  a new area or table moves the index in the middlegame band where C.3
  gained most.
- **Search coupling.** Threat and mobility terms enter mg and eg in the
  bands the margins were fitted on (RAR-E19/E20: −104.5 Elo for a
  statically better function); the tree read and the magnitude screen
  flag it and the margin block repairs it (PROCESS, *Evaluation change
  under a fitted search*). Threats also overlap what the search prices
  through SEE pruning and futility; a double count shows as tree growth.
- **Lazy gate.** Above 600 cp both families are skipped; the residual
  screen's Rarog column is the played score, so rows above the gate carry
  the families' absence by construction. The `|rarog| ≤ 500` cohort
  reads the band where the families act.
- **Duplicates inside Rarog.** The refined hanging term, the weak-piece
  penalty and the restricted term overlap the donor's `Hanging`, weak-set
  gating and `RestrictedPiece`; the x-ray slider-on-queen overlaps T4.
  The refit re-prices them; a unit removes what it replaces.
- **Neighbours.** C.6 (pawns, passers) reads the same pawn-attack sets and
  "strongly protected"; C.7 (pieces) owns the outpost and queen-file terms
  the donor keeps next to threats. Nothing here touches search code.

## Cheapest discriminating tests

1. **RAR-E30, the residual step (C.4.1), zero games, registered
   2026-10-10 before any model was fitted.** RAR-E23's donor rows with the
   C.5.2 head's played score substituted (RAR-E25's method), `analyse
   --within`. It decides whether either family stays open, by RAR-E17's
   rule in six registered cohorts. Its predictions are informed by the C.3
   screen (disclosed in the entry).
2. **Sub-term attribution, zero games (to be registered as RAR-E31 only
   if RAR-E30 keeps a family open).** The donor's threats components and
   its per-piece-type mobility rebuilt offline from the FEN (python-chess,
   `9587eeeb`'s definitions), verified exactly against the printed
   `threats` and `mobility` rows on every row before any loss is computed;
   then (a) leave-one-out of the threats components as directions, as
   RAR-E21 did for king; (b) for mobility, the donor's tables summed
   under four areas: the donor's, the donor's without x-rays, the donor's
   without the pin restriction, and Rarog's, to tell M1 from M2 and M3
   from M4; (c) the Rarog-shaped counterparts of T1 and T2 (ungated minor
   and rook counts, unconditional pawn threats) priced beside the donor's,
   to tell conditioning from pricing. Predictions, falsifiers and the
   readiness rule are frozen in that registration.
3. Only then, if a mechanism separates: the design, its per-node cost
   statement, and the cluster shape's screens, tree read and gates.

## Prospective prediction

RAR-E30's (frozen in its entry): threats +0.11% over all rows (band
+0.07 to +0.15) and +0.12% at seven men or more, with signal by the rule
(probability 0.6 and 0.7); mobility +0.05% over all rows, under the line
(0.85), and +0.18% in the opening band with signal there (0.7); every
family within 0.03 points of the C.3 screen outside six men or fewer
(0.85); king under +0.15% (0.85).

## RAR-E30, read 2026-10-10

On the C.5.2 head, gains in percent of its held-out squared error:

| Cohort | Mobility | Threats | King |
|---|---:|---:|---:|
| all | +0.048 ± 0.012 | **+0.107 ± 0.020** | +0.079 ± 0.014 |
| phase ≥ 96 | **+0.184 ± 0.044** | +0.138 ± 0.038 | +0.120 ± 0.038 |
| phase 32–95 | +0.054 ± 0.029 | +0.113 ± 0.031 | +0.357 ± 0.059 |
| phase < 32 | +0.012 ± 0.020 | **+0.178 ± 0.040** | −0.012 ± 0.006 |
| men ≥ 7 | +0.060 ± 0.016 | **+0.122 ± 0.020** | +0.105 ± 0.017 |
| no queens | +0.011 ± 0.021 | **+0.197 ± 0.042** | +0.004 ± 0.006 |

Threats has signal in four of the six registered cohorts and is the
largest regular family over all rows (passed +0.091 is next). Mobility
has signal in the opening band only. The predictions held except for the
donor's winnable family, which fell by 0.12 to 0.17 points in the endgame
and no-queens cohorts: that is the share of the donor's scale-factor
direction C.5.2's opposite-bishop rule took, a side reading that says the
screen also measures what an accepted unit captured.

In squared-error terms the two families together carry about a quarter
of what the king family carried before C.3 (+0.66 over all rows, +1.58
in the middlegame band). Threats' signal is in the endgame band and
without queens, where pawn threats and attacks on undefended pieces
decide material; mobility's is in the opening, where the area definition
(blocked and low pawns, the queen's square) differs most from Rarog's.

## Falsifiers and stop conditions

- Neither family has signal in any registered cohort: C.4 closes
  `NO_CHANGE` on RAR-E30, and this record says so.
- A family has signal but no component of it separates in the attribution
  (H0): the family is built whole only if its gain is at least 0.20% in
  the band where it acts; otherwise `NO_CHANGE` on expected value, with
  the retry trigger a regenerated corpus (C.8) or a changed search (C.10).
- The attribution instrument fails its exact reproduction on any row: the
  instrument is void and is repaired before any reading is used.
- Whatever the static layer says, acceptance is the gate's; a unit here
  is expected small (prior 0 to +10 Elo for both families together, below
  even odds of passing `[0,3]` on gate 1), and a failed pair of gates is a
  rejected cluster under PLAN rule 6.

## Decision

**`MORE_RESEARCH` after RAR-E30 (2026-10-10).** Both families stay open:
threats by signal in four registered cohorts, mobility by signal in the
opening band only. Neither is `READY_FOR_IMPLEMENTATION`: the mechanism is
not yet attributed, and a unit built on "the family in the donor's shape"
alone would be the kind of transplant the programme declined. The next
step is C.4.2, the attribution (RAR-E31), zero games, registered before
any model is fitted on the component directions. Its frozen use decides
what, if anything, a unit builds:

- threats: a component set carrying at least 0.10% at four standard errors
  in a cohort where the family has signal, and that Rarog's current terms
  cannot express by re-pricing (the gating and the safe sets, or a term
  Rarog lacks), is the unit's content; a residual that re-pricing of the
  Rarog-shaped counterparts recovers is a refit question, not a unit;
- mobility: the unit includes the area only if the donor's tables under
  the donor's area beat them under Rarog's area by at least 0.10% at four
  standard errors in the opening band, and the x-rays only if the
  no-x-ray variant loses as much; otherwise mobility closes `NO_CHANGE`.

## Sub-steps spawned under C.4 (classes in PLAN)

- **C.4.1** Residual step (RAR-E30) — `V`, done 2026-10-10.
- **C.4.2** Sub-term attribution (RAR-E31) — `V`: the offline instrument
  with its exact-reproduction check, then the read.
- The unit's design, implementation, refit and gate are added when C.4.2
  names a mechanism.

## What this did not establish

- No game-level read of either family exists; the static layer is the only
  evidence, and it ranks questions only.
- The donor's per-node cost of the area and x-rays in Rarog's structure is
  unmeasured; it is stated before implementation, as PLAN's speed
  requirement asks.
