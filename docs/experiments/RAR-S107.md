# RAR-S107 — D.6.1: castling classed neither quiet nor noisy, the screen and the harm read

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-09 before any read and any game. The
  screen is the agent's (deterministic, no idle host needed); the match is the
  maintainer's command.
- **Kind:** zero-game screen, then AGENTS' repair case 2 if the screen does not
  close the item: a classification repair that moves the fingerprint, expected
  near zero, where a small loss is acceptable for correctness.
- **Baseline:** `d6` at `5c6fbd6`, the same engine source as `b202174`, bench
  15,625,852 / EBF 2.563. The candidate is named when it is built.
- **Research question:** does castling's double classification reach search
  decisions often enough to be a defect in play, and does classing it quiet,
  as the donor does, cost strength?

## The defect, from the source

`Board::is_quiet_move` is `flags <= DOUBLE_PUSH`; castling has flags 2 and 3.
`is_noisy` is capture or promotion. So castling is neither, and the two
predicates split its handling:

| Site | Predicate | Castling is treated as |
| --- | --- | --- |
| Generator, staged picker, root scoring (`movepick.rs`) | `is_noisy` | quiet: emitted and scored with the quiets, dropped by the quiet skip |
| Best-move credit, fail-low reward, continuation context, correction training | `is_noisy` | quiet |
| Late-move, futility and history pruning (`node.rs`) | `is_quiet` | exempt |
| SEE-pruning threshold | `is_quiet` | noisy |
| LMR term | `is_quiet` | noisy: `830 − 182·h/1024` against quiet's `1932 − 316·h/1024 + gap` |
| Which list a failed move joins, so its malus | `is_quiet` | noisy: `update_noisy` |

Three consequences follow from the trace. The noisy slot
`[king, g1/c1/g8/c8][no capture][not threatened]` is written only by castling
(a non-capturing king move is otherwise quiet, and a legal castling square is
never attacked) and receives only maluses, so it can only fall toward −12,800.
At the floor the LMR term is 3,105, about 1.1 plies above quiet's 1,932 at zero
history, while at zero the noisy term is 830, about 1.1 plies below it. And the
castling move's quiet entries get bonuses but never a malus, so its place in
the quiet order can only rise.

The donor checked: Reckless's `is_quiet` is `is_present() && !is_noisy()`, and
`is_noisy` excludes castling (`src/types/moves.rs:64–72`), one predicate
throughout.

## Hypotheses

- **H1, leading:** the classification is a live but small defect: the slots sit
  near their floor, castling takes about a ply more reduction than a quiet would
  and escapes quiet pruning.
- **H2, inert:** castling comes early or as the TT move, so LMR and the quiet
  rules rarely reach it, and the classification changes almost no decision. The
  item would then close `NO_CHANGE`.
- **H3, a hidden benefit:** exempting castling from quiet pruning, and its
  rising quiet entries, help a move that search undervalues; a tune fitted around
  this would lose by the repair. The games read this, not the screen.

**Interactions.** Consumers: LMR, the three quiet pruning rules, the SEE
threshold, the quiet and noisy maluses and the counts they scale by. The tuned
LMR and pruning constants (C.10) were fitted with this quirk, but over a small
population. Castling's child search changes the TT and correction populations
only through those decisions.

**Invariants.** One predicate partitions the legal moves: every legal move is
exactly one of quiet and noisy, `is_quiet == !is_noisy`, covering castling,
under-promotion and en passant (NODE-14's test, which lands with the fix).
Castling legality, SEE and make/unmake are untouched.

**Falsifier and stop rule.** See *Closes `NO_CHANGE`* below; if that rule does not
fire, the fix is one predicate and the games judge it. A failed harm rule follows
AGENTS: the repair does not land, the defect stays open in D.6.1, and a refit is
a new registration.

## The screen (frozen)

**Agreement test.** `is_quiet_move(mv) == !is_noisy(mv)` over the legal moves of
the 40 bench positions and a castling position, on the unrepaired source.

**Counter read.** `analysis/arm_patches/d61-rar-s107-screen.patch`, on
`5c6fbd6`, adds `diag` counters and leaves the search's decisions alone; its
`bench 13` must reproduce 15,625,852. One thread, stride 1, summed by
`tools/diag/bench_counters.py`; the slots are read from each position's own dump
lines, since they are a state and not a count. Over the non-root move loop:

- `moves_seen`, `castle_seen`, `castle_tt`, the sum of castling's move index;
- `castle_shadow_lmp`, `_fut`, `_hp`: the quiet rule would have pruned it, with
  quiet history as the picker reads it, in rule order, so a move counts once;
- `castle_see_noisy_fail`, `castle_see_quiet_fail`: each threshold's verdict;
- `castle_lmr`: castling reached LMR at depth 2 or more, with the sum of
  (noisy term − quiet term), and how many differ by 512 or more either way;
- `castle_searched`, `castle_cutoff`, `castle_noisy_malus`, `castle_fail_low`;
- the four noisy slots and the four quiet entries at each position's end.

**Exposure (AGENTS, a repair's registration).** From RAR-S106's 2,000 games
(`tools/results/rar-s106-stc/games.pgn`, the most recent gate): the share of
games in which castling is legal at some played position, and in which it is
played. A root is the near bound; a search reaches further.

### Closes `NO_CHANGE` (frozen)

Any one closes the item, with the agreement test recorded as the contract gap
for D.8:

1. Castling is under 1 in 100,000 of `moves_seen`.
2. Of the castling moves in the non-root loop, under 1% would change a decision:
   a shadow prune, or an LMR term differing by 512 or more.
3. Under 1% of gate games ever have castling legal at a played position.

Otherwise the item is `READY_FOR_IMPLEMENTATION` for the repair and gate below.

## The gate (frozen, played only if the screen does not close the item)

- **Fix:** one predicate, quiet is `!is_noisy`. Nothing else moves: no constant,
  no history shape, no adjacent rule.
- **`stc`:** Colosseum `match`, 2,000 games at `3+0.03`, Hash 64, one thread, the
  wrapper's slot count, the UHO book in random order, no adjudication, seed
  20261209, `tools/results/rar-s107-stc`.
- **Harm rule** (AGENTS' case 2, from the manifest's nElo): below **−13 nElo**
  (2,000 games, standard error about 7.8). This catches a true −10 with
  probability 0.36 and −20 with 0.82, so "kept" means no harm beyond roughly −20
  nElo.

## PRE-REGISTERED PREDICTION (frozen before exposure)

- **Agreement test:** fails, on castling moves and nothing else. 0.99.
- **Slots:** none ever above 0. 0.99. Every slot written at least 50 times is at
  or below −6,000 at the end of `bench 13`. 0.8.
- **`castle_seen`:** 0.02% to 0.5% of `moves_seen`. 0.7.
- **`castle_tt`:** under a quarter of `castle_seen`. 0.6.
- **SEE:** no castling move fails either threshold: a legal castle's SEE is 0
  and both thresholds are capped at 0. 0.9.
- **Shadow prunes:** 5% to 30% of `castle_seen` in total. 0.5.
- **LMR:** the mean of (noisy − quiet) is positive, +200 to +1,200, more
  reduction than a quiet would take. 0.6.
- **Closes `NO_CHANGE`:** no rule fires. 0.85.
- **Exposure:** castling legal at a played position in 40% to 90% of games, played
  in 30% to 70%. 0.7.
- **Fingerprint under the fix:** within ±3%. 0.8.
- **`stc`:** 0 ± 15 nElo. P(no harm) 0.95, the rule's own false-alarm rate at a
  true zero.
- **Most likely failure mode:** the slots holding well above −6,000, because
  few of bench's positions keep castling legal and the slots then have few
  writes, which would be a miss in magnitude and not in mechanism.
