# C.13.7 — contempt and draw avoidance: research card

A research card at `R2` (PLAN C.13.7): is there a list-Elo lever against the
weaker pool engines, in what form, what it touches, and how it would be
falsified. Nothing is built. Any measurement belongs to E.2's pool gauntlet and
is registered there. The decision is the maintainer's.

## Decision needed

Whether to pursue contempt for E.2, and so for the rating lists, or close
C.13.7 `NO_CHANGE` with a retry trigger.

## Known evidence

- **The engine today scores every draw exactly 0.** Repetition, the
  fifty-move rule, insufficient material and stalemate each return 0 in the
  main search and in quiescence (`can_declare_draw_in_search`, `node.rs`).
  There is no contempt and no draw jitter, and the "Draw score" section of
  `src/search/params.rs` is empty.
- **The pool** (RAR-M64, the B.9 head at 1T, 400 games against each E.2
  target):
  - Rarog scores 55.4% against Houdini 3, 63.5% against Critter 1.6a, 62.4%
    against Fritz 16 and 72.4% against Rybka 4.1.
  - It draws 21–26% of those games.
  - On the held scale it is level with Houdini 3 and 88 to 167 Elo above
    the others. The current head is stronger than B.9 by C's accepted units.
- **The draw census** (`analysis/artifacts/c137/draw_census.py` → `draw_census.log`,
  zero games):
  - *Method:* the 1,600 games against the four targets replayed, each draw
    classified by how it ended. Rarog's own view before the draw locked in is
    the mean of its last five non-zero `s=` scores before the final run of
    zeros: worse ≤ −30 cp, level, better ≥ +30.
  - *Totals:* 369 draws (23.1% of games).

    | Ending | Rarog worse | Rarog level | Rarog better |
    |---|---:|---:|---:|
    | Threefold repetition | 53 | 75 | 103 |
    | Fifty-move rule | 17 | 9 | 7 |
    | Insufficient material | 25 | 28 | 40 |
    | Stalemate | 2 | 1 | 9 |

  - **The pool a draw offset can touch:** repetition and fifty-move draws
    reached while level (84, 5.2% of games) or after Rarog had thought itself
    better (110, 6.9%), so at most 194 games (12.1%). Dead-material draws (93)
    are untouched by a search offset.
  - **Where it costs:** repetition and fifty-move draws reached while worse
    (70, 4.4%). An offset makes a draw look worse, so Rarog would decline some
    of the saving repetitions when it was only slightly worse.

## Size of the lever (estimate, not a measurement)

- If an offset turned 30% of the 194 touchable draws into decisive games at
  65 to 35 in Rarog's favour, the gain would be 0.5 × (38 − 20) = 9 points in
  1,600 games, +0.56% score. That is **about +2 Elo** against these four.
  Doubling the conversion gives +4.
- The cost side (declined saving repetitions, and losses from overpressing
  against Houdini 3, which is level) is not counted.
- Against an engine of equal strength the expected effect is zero or negative,
  so self-play is the wrong instrument, and so is the CCRL field, where
  opponents are chosen near one's own level.
- **The lever is +1 to +4 Elo against the E.2 field at best, about 0 on a
  rating list, and it adds no playing strength.**

## Candidate forms

1. **A draw-score offset in the search (contempt).**
   - Form: a declared draw is worth `−c` to the root side to move and `+c`
     to the other, with `c` small (5–25 cp), possibly tapered by phase.
     Rarog would refuse a repetition or a fifty-move draw for any alternative
     it values above `−c`.
   - Lives in: the search layer only; the static evaluation is untouched.
   - It needs a UCI option, default 0 for analysis (which also restores today's
     behaviour), and a rated configuration.
2. **A sign-aware endgame scale.** The endgame scale (C.5.2's opposite-bishop
   rule, the specialised scales) shrinks drawish endings toward 0. With form 1
   alone, Rarog would still trade into a drawish ending that evaluates near 0
   rather than repeat at `−c`. Form 2 scales less when the root side is the
   one ahead.
   - It makes the score depend on who is to move at the root, so it cannot
     live in the static evaluation: it would break the standing
     colour-symmetry test (C.13.1) and the evaluation cache's purity. It
     belongs in the search's use of the score.

## Interaction map

| Mechanism | Interaction |
|---|---|
| Draw detection (`can_declare_draw_in_search`, two-fold inside the search) | Form 1's site; the offset's sign must follow the root side at every ply |
| Rule-50 damping in the search | It damps scores toward 0 as the clock grows. With contempt, the target is arguably `−c` for the root side; left at 0, a long manoeuvre looks better than repeating |
| Endgame scale, C.5.2's opposite-bishop rule, KPK, KXK | Drawish scores near 0 compete with `−c` draws; form 2's reason |
| Transposition table | Draw values depend on the root side. Entries stay consistent within one game (the root side is fixed), but analysis or a side change must clear or key them |
| Tablebase root (Syzygy) | A tablebase draw at the root: keep it exact (0), or offset it? A choice, stated before any build |
| Aspiration windows, the null-move and singular margins | All are relative, so a small `c` shifts nothing structurally, but near-draw nodes change their bounds |
| MultiPV and analysis | Must report 0 for draws with the option at 0 |

## Invariants (if built)

- The static evaluation is unchanged: the colour sweep, the evaluation cache
  and the fingerprint (with `c = 0`) reproduce exactly.
- At `c = 0` the search is byte-identical: bench equals the head.
- A draw's value is `−c` exactly when the root side is to move at the draw
  node, and `+c` otherwise. A focused test covers a repetition at even and
  odd ply.
- Mate and tablebase scores are untouched; the TT is not reused across a
  root-side change.

## Falsifier and stop rule

- **Cheapest prior falsifier:** this census. It bounds the touchable pool at
  12.1% of games and the gain at a few Elo, before any build.
- **If pursued:** E.2's 1T pool gauntlet, with and without the offset, on the
  same field and seed, registered there with its harm rule against Houdini 3,
  the level opponent. A result within the gauntlet's error of zero stops it.
  A self-play SPRT cannot accept it.

## Recommendation

**I recommend against building contempt now; close C.13.7 `NO_CHANGE`.**
- The estimated lever is +1 to +4 Elo against the E.2 field, about 0 on a
  rating list, and it adds no playing strength.
- It needs a root-aware layer in the search, a new UCI option, and a
  gauntlet to measure, which is a disproportionate cost for that size.
- **Retry trigger:** E.2's 1T gate is missed against one engine by 5 Elo or
  less. Contempt is then the cheapest registered lever, measured in E.2's
  gauntlet, form 1 only.
