# RAR-S108 — D.6.2: quiescence cannot see a non-capturing queen promotion, the screen

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-10 before any read. The screen is the
  agent's (deterministic, no idle host needed). The gate, if the screen does not
  close the item, is appended here before any game and is the maintainer's
  command.
- **Kind:** zero-game screen for a strength candidate.
- **Baseline:** `d6` at `bc90ebf`, engine source `72c4e60`, bench 11,946,003 /
  EBF 2.547.
- **Research question:** how often does quiescence stand pat, or fail to beat
  alpha, while a safe queen promotion it never generates is on the board? Is that
  often enough to be worth a gate?

## The defect, from the source

Outside check, quiescence generates `generate_legal_captures_into`, and ProbCut
does too. Pawn moves there target enemy occupancy only, so a pawn on the seventh
with a free square ahead promotes only in the quiet generator, which neither
caller runs (`8/P6k/8/8/8/8/8/K7 w - - 0 1` yields no captures). The staged
picker of the main search does see the promotion: it is generated with the
quiets and ordered first among them.

Both donors put the queen push with the noisy moves: Reckless's
`append_noisy_moves` pushes `PromotionQ` and leaves the three under-promotion
pushes with the quiets (`src/board/movegen.rs`, `generate_moves`); Stockfish's
`make_promotions` emits the queen for `CAPTURES` and the others for `QUIETS`
(`src/movegen.cpp:85–101`, `7cdd5e89`).

## Hypotheses

- **H1, leading:** a leaf with a pawn one step from promoting, a free square and
  nothing to capture is read as its static value, low by most of a queen, and the
  main search's horizon carries that error back. Generating the queen push in the
  captures-only generator fixes the leaf.
- **H2, redundant:** the main search's last ply already plays the promotion, and
  the passed-pawn terms already price a pawn on the seventh, so few leaves are
  wrong, and fewer still by enough to change a move.
- **H3, harmful:** quiescence exempts promotions from futility, the count rule
  and the SEE floor, and the SEE of a quiet promotion is its gain alone (it never
  reads a recapture), so every queen push would be searched, defended or not:
  more nodes for little. Triumviratus's push to the seventh in quiescence read
  −0.77, +1.1, −1.8 and +0.23 (TS-14).

**Interactions.** Quiescence's stand pat, delta pruning (the whole node goes
when the stand pat plus a queen and a margin is below alpha; a promotion gains a
queen less a pawn, so this stays valid), the count rule and SEE floor (both exempt
promotions), ProbCut's list (its SEE gap would admit every queen push for the same
reason), `has_pseudo_capture` (it must return true for a pawn on the seventh with
a free square, or the generator returns before it looks), and the TT move in
quiescence. The staged picker's contract does not change if only the
captures-only generator changes, since the staged path has its own capture
generation.

**Invariants.** Each generator emits legal moves only, and none twice: the
captures-only list is the staged capture list plus the queen pushes, which the
staged quiet list still holds. Under-promotion pushes stay quiet-generated
(D.6.17 owns them).

## The screen (frozen)

`analysis/arm_patches/d62-rar-s108-screen.patch`, on `72c4e60`, adds `diag`
counters that never steer; its `bench 13` must reproduce 11,946,003. At each
quiescence capture generation outside check, past the stand pat and delta
pruning, and at each ProbCut generation:

- `q_gen`: the generations;
- `q_gen_qpromo`: those with a legal quiet queen promotion;
- `q_gen_qpromo_empty`: with it and an empty capture list;
- `q_gen_qpromo_safe`: with one whose square the opponent does not attack once
  the pawn has left;
- `q_gen_qpromo_raise`: with a safe one where the stand pat plus a queen less a
  pawn exceeds alpha;
- `pc_gen`, `pc_gen_qpromo`: the same first two for ProbCut.

**Suites.** (A) `bench 13`, the review's population, which read 1.53% at depth 10.
(B) An endgame-rich suite from RAR-S107's 2,000 games (the latest gate): from each
game, the first played position with ten men or fewer, if it has one, at depth
12 in one process with `ucinewgame` before each search. Summed from each
search's own dump, with the dump count asserted equal to the positions.

**Exposure,** from the same games: the share in which a quiet queen promotion is
legal at some played position, and the share of played positions with one.

### Closes `NO_CHANGE` (frozen)

Either closes the item:

1. Under 0.5% of suite B's `q_gen` have a quiet queen promotion (PLAN's rule).
2. Under 0.1% of suite B's `q_gen` have a safe one that could raise alpha
   (`q_gen_qpromo_raise`): too sparse to move a gate.

Otherwise the item is `READY_FOR_IMPLEMENTATION`: the queen push added to the
captures-only generator and its pre-scan, nothing else, gated by an SPRT
registered here before any game.

### Amendment before exposure (2026-11-10): suite B's node cap

Suite B as first run never finished: after two hours in one process it had
printed nothing (its output was buffered) and was stopped, so no suite B number
was seen. Timing every 50th position put a search at 0.08 s on average; one
position runs away. Position 627,
`8/2p2P1P/2n2P2/1k6/p7/6N1/P5K1/8 b - - 0 44`, with three white pawns one step
from promoting, reached only depth 10 in 60 s on the unpatched `72c4e60` engine
(194M nodes, seldepth 127). Each of positions 630 to 1483, searched in a fresh
process, finished depth 12 in 3.1 s at most, the largest at 12,969,944 nodes.

Each suite B search is therefore `go depth 12 nodes 25000000`, about twice the
largest normal search, with everything else as registered. A search stopped by
the cap still dumps its counters, so the dump count stays asserted equal to the
positions. The read lists every position that hits the cap; the closing rules
are read on the capped sums, and also without those positions.

That runaway is a finding of its own, not this item's: a search can lose most of
its depth in a promotion-rich ending. Who owns it is decided with the result.

## PRE-REGISTERED PREDICTION (frozen before exposure)

- **Suite A:** `q_gen_qpromo` 0.5% to 3% of `q_gen`, near the review's 1.53%. 0.7.
- **Suite B:** `q_gen_qpromo` 2% to 15%. 0.7. Neither closing rule fires. 0.75.
- **Safe share:** under half of `q_gen_qpromo` are safe. 0.6.
- **`q_gen_qpromo_raise`:** at least 0.1% of suite B's `q_gen`. 0.7.
- **ProbCut:** `pc_gen_qpromo` under 1% of `pc_gen` on suite A. 0.7.
- **Exposure:** a quiet queen promotion legal at a played position in 10% to 35%
  of games, at 0.2% to 2% of played positions. 0.6.
- **Most likely failure mode:** suite B reads high on the pawn-only endings,
  where promotion races are the whole position, and says little about the
  middlegame leaves that most of a game's searches visit.
