# Code audit, 2026-08-19

Requested during the RAR-S63 run. **Nothing here is fixed.** Each item records
what is wrong, how it was confirmed, and what it would cost to act on, so a fix
can be commanded per item rather than as a batch.

Ordered by severity. Severity here means "how wrong is the engine's behaviour
against its own stated intent", not "how much Elo".

The audit deliberately looked hardest for the class that produced the ProbCut
desync (RAR-S62): **state written in pieces, where one piece can be missed and
the survivor is silently stale.** That class produced the top finding too.

---

## 1. `stack[ply].reduction` is stale whenever a move is not reduced — LIVE

**Severity: high. This is a defect in an ACCEPTED mechanism, and it is mine,
introduced at 4.5.4.**

`lmr_prior_reduction_adj` (adopted at 512/1024 ply, and the change that appears
to carry Cluster A's gain) reads `stack[ply - 1].reduction` to learn whether the
parent move was reduced. But `search.rs` writes that field on **one branch
only** — the LMR path. The `else` branch searches at full depth and never
writes `reduction = 0`.

So the field keeps whatever a previous sibling — or a previous *visit to that
ply from an entirely different subtree* — left behind. Concretely: move 5 is
reduced by 2 plies and writes 2; move 6 escapes LMR and writes nothing; move 6's
child reads 2 and believes its parent was reduced.

**Confirmed live, and large.** Adding `self.stack[ply].reduction = 0;` to the
`else` branch moves `bench 13` from **7,587,235 to 7,038,294 — −7.2% nodes**.
A field read this often, wrong this often, is not a corner case.

Compounded by singular verification, which re-enters `negamax` at the **same
ply** with an excluded move. That nested search runs its own move loop and
writes `stack[ply].reduction`, which then survives back into the parent's loop.

**Consequences for the record.** RAR-S61 (+4.50 ± 3.50) and the inferred +9.59
for prior-reduction were both measured with this bug present, so they price the
mechanism-as-implemented, not the mechanism-as-intended. RAR-S63, currently
running, compares two arms that both contain it identically, so **that
comparison stays valid** for its own question.

**Fix:** one line, plus a decision about whether `push_move` should own the
reset so the field cannot be written independently of the move again — which is
the structural lesson of RAR-S62.

**Cost:** the fingerprint moves, so Cluster A's numbers need re-measuring.

---

## 2. `improving` collapses to false for two plies after every check

**Severity: medium-high. A real mechanism, systematically disabled, in the
direction the evidence says is wrong.**

```rust
let improving = !in_check
    && ply >= 2
    && self.stack[ply - 2].static_eval != VALUE_NONE
    && static_eval > self.stack[ply - 2].static_eval;
```

When the node two plies back was in check its `static_eval` is `VALUE_NONE` by
construction, so `improving` is forced false — regardless of whether the
evaluation is actually improving. There is no fallback to `ply - 4`.

`improving` is not a minor input. It subtracts a full ply from the LMR
reduction and feeds `lmp_not_improving` in the move-count prune margin. Forcing
it false means **more reduction and more pruning**.

**Size of the affected population:** the v3 differential puts Rarog's
`nodes_in_check` at 26,430 of 272,867 nodes — **9.7%** — so roughly one node in
ten has its `ply - 2` in check and silently loses the term. (The reference is at
7.7%, and it walks back further.)

Direction matters: this biases toward selectivity, and over-selectivity is the
one diagnosis this project has replicated four times (RAR-S53, S54, S55, and 4.7
paying +15.56 Elo for pruning less).

**Fix:** walk back to `ply - 4` when `ply - 2` is unusable, as the reference
does. Behaviour-changing, needs a gate; natural owner is 4.5 or 4.10.

---

## 3. Killers are never cleared for descendant plies

**Severity: low-medium. A design difference, not a defect — listed so it is a
decision rather than an omission.**

`killers[ply]` is cleared once per search (`clear_history`). Within a search a
ply's killers persist across every sibling subtree that reaches that depth, so a
node can inherit killers from a positionally unrelated subtree.

Stockfish clears the grandchild's killers on node entry, which bounds how far a
killer can travel. Rarog does not.

This is defensible — killers are meant to be a cheap, noisy heuristic — and
Rarog's first-move cutoff rate is already **above** the reference's in every
cohort (RAR-S52/S55), so there is no evidence of harm. Recorded because it is
the kind of difference that should be intentional.

**Fix:** trivial if wanted; needs a gate; genuinely might do nothing.

---

## 4. Eval scratch `attacks_from_sq` is guarded by construction, not by assertion

**Severity: low. No defect found; recording the reasoning so the next reader
does not have to redo it.**

`Evaluator::attacks_from_sq` is a reused `[[Bitboard; 64]; 2]` scratch buffer,
written per occupied square and read later by the king-safety and threat
passes. Debug builds poison it with `u64::MAX` between evaluations.

The poisoning detects nothing on its own — a stale read returns all-ones and
produces a wrong value without asserting. Safety rests on the read sites
iterating only over squares the write loop covered, which they do.

**Not a bug.** But the protection is structural, and a future pass that reads a
square outside that set would be wrong in release and merely differently-wrong
in debug. A `debug_assert!(entry != Bitboard(u64::MAX))` at the read sites would
convert the argument into a check.

---

## Checked and found sound

Recorded so the same ground is not re-covered:

- **TT mate-distance handling.** `score_to_tt` / `score_from_tt` adjust by ply
  in the right directions and carry the rule-50 guard.
- **History saturation.** `update_hist_entry` uses the standard gravity form
  with `saturating_i16`; no overflow path.
- **`static_eval` writes.** Unconditional at the node, `VALUE_NONE` in check —
  no staleness, unlike `reduction`.
- **qsearch stack discipline.** `clear_move` precedes both early returns, so no
  stale move survives an abort or a cutoff.
- **negamax move-loop stack discipline.** No early return between `push_move`
  and `clear_move`.
- **Eval colour symmetry.** Covered by `tests/eval_invariants.rs`
  (`eval_is_colour_symmetric`), colour-flip plus vertical mirror.
- **No TODO/FIXME/HACK markers** anywhere in `src/`. The single `TODO` string
  is inside a comment explaining that the question is closed.

## Not covered by this pass

Time management, SMP/shared-state, syzygy, and the UCI layer were not audited in
depth. The request emphasised search and HCE, and those got the detailed pass.
