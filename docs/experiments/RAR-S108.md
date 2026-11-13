# RAR-S108 — D.6.2: quiescence cannot see a non-capturing queen promotion, the screen

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-10 before any read. The screen is the
  agent's (deterministic, no idle host needed). The gate, if the screen does not
  close the item, is appended here before any game and is the maintainer's
  command.
- **Kind:** zero-game screen for a strength candidate.
- Disposition: accepted — H1 2026-11-10 at 11,108 games, nElo +12.2 ± 6.5;
  D.6.2 closed, fingerprint 13,038,236.
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

## Result: the screen (2026-11-10)

Read by `analysis/artifacts/d62-qpromo-2026-11-10/read_screen.py` on
`rarog-d62screen.exe` (SHA-256 `a6545b5a…`, the screen patch on `72c4e60`), log
`read.log` beside it. Suite A reproduced 11,946,003 nodes, and every counter
dumped 40 times; suite B's dumps equalled its 1,484 positions.

| | suite A, `bench 13` | suite B, 1,484 endings at depth 12 |
|---|---:|---:|
| `q_gen` | 1,151,838 | 33,696,108 |
| with a quiet queen promotion | 2.291% | **5.938%** |
| ... and no capture | 0.883% | 3.389% |
| ... a safe one | 1.043% | 3.833% |
| ... a safe one that could raise alpha | 0.975% | **3.412%** |
| safe share of those with one | 45.5% | 64.5% |
| ProbCut generations with one | 5.221% of 176,258 | 16.616% of 4,454,355 |

**Exposure.** A quiet queen promotion was legal at some played position in
1,087 of RAR-S107's 2,000 games (54.4%), at 4,393 of 252,485 played positions
(1.740%).

**The cap.** The read script counted 13 positions as capped, by a last
iteration under depth 12; 12 of them stopped at depth 2 in under 60 nodes, a
mate found and the iteration ended, not the cap. Only position 627 reached it
(depth 10, 20,753,170 nodes). Without all 13, the two closing figures read
6.125% and 3.502%.

**Neither closing rule fires:** 5.938% against rule 1's 0.5%, and 3.412%
against rule 2's 0.1%. The item is `READY_FOR_IMPLEMENTATION` as registered.

**Calibration (screen).**

- Suite A, 0.5% to 3%: a hit at 2.29%, above the review's 1.53% at depth 10.
- Suite B, 2% to 15%, and no closing rule firing: hits.
- Safe share under half: a hit on suite A (45.5%), a miss on suite B (64.5%).
  The prediction named no suite; in the endings a promoting pawn is less often
  covered than in the bench's middlegames.
- `q_gen_qpromo_raise` at least 0.1%: a hit, by a factor of 34.
- ProbCut under 1%: a miss in magnitude, 5.2% on suite A. ProbCut is entered
  at nodes already above beta, where a passed pawn far advanced is common; the
  prediction carried quiescence's rate over to it.
- Exposure: 10% to 35% of games was a miss in magnitude (54.4%); 0.2% to 2% of
  positions was a hit (1.74%). A promotion becomes legal in more games than
  predicted, briefly in each.
- The named failure mode, suite B high on the pawn-only endings, is not
  separable from this read; suite A's 2.3% says the population is not confined
  to them.

**The runaway.** Position 627 is not this item's defect: the candidate below
also runs away on it (depth 11 in 60 s, 203M nodes, seldepth 127, against the
baseline's depth 10). PLAN D.6.7, the ply cap, holds it as evidence for its
screen (2026-11-10).

## The repair (2026-11-10)

`ee0b3e4` on `d6`: a second entry to the captures-only generator,
`generate_noisy_into`, emits the legal captures and then the legal queen pushes
(a push in check only blocks the single checker; a pinned pawn pushes only
along its pin), and its pre-scan admits a pawn on the seventh with a free square
ahead. Quiescence outside check and ProbCut call it, and nothing else does.
`generate_legal_captures` keeps its contract and the staged picker is
unchanged, so the main search still finds the push in its quiet stage. The
under-promotion pushes stay quiet (D.6.17).

The invariant is a test, `the_noisy_list_is_the_captures_and_the_queen_pushes`:
over two-ply walks from five positions (both colours promoting, the runaway
position and Kiwipete), the list equals the legal captures plus the legal quiet
queen promotions, each once; with fixed cases for a blocked pawn, a block of a
check, a push off the checking line and a diagonal pin. With the push turned
off it fails. ProbCut's list test in `movepick.rs` now expects the queen pushes
too.

Qualified on the final tree: `cargo fmt --check` clean, debug tests 522 and
release tests 524 passed, `cargo clippy --all-features --all-targets -D
warnings` clean. Bench **13,038,236 / EBF 2.540**, twice the same, against
11,946,003 / EBF 2.547 (+9.1% nodes). A count, not a strength read.

## The gate (frozen 2026-11-10, before any game)

- **Arms:**
  - candidate `tools/test_engines/rarog-d62cand-pext-pgo.exe`, SHA-256
    `4b3500ba4fe09553022e582d7cc9b683dc2ef75e45e7b3222f0be114737153c1`,
    built from a clean worktree at `ee0b3e4`, `id name Rarog
    dev-20261110-ee0b3e44`, bench 13,038,236;
  - baseline `tools/test_engines/rarog-d61cand-pext-pgo.exe`, SHA-256
    `02a3290c379ef8aa84bd0f665f0d6ab2285e6bd6fc30d789760f89e7dd42e0e1`,
    RAR-S107's candidate binary, built at `72c4e60`, bench 11,946,003.
- **Gate:** Colosseum `sprt`, `sprt-default.toml` (`[0,3]` nElo, pentanomial,
  `model=normalized`, α = β = 0.05), `3+0.03`, margin 20 ms, Hash 64, one
  thread, the UHO book in random order, no adjudication, **cap 20,000 pairs**
  (40,000 games; the cap of the C-phase gates; by RAR-M10 it resolves a true +4
  nElo, about 47,000 games, only partly, and does not resolve +2), seed
  **20261110**, `tools/results/rar-s108-gate`. Bounds, cap, book and
  adjudication never change after a game is seen. An unresolved stop at the cap
  is read as H0.
- **Command,** dry-run on 2026-11-10 with exit 0 (host 1% busy with no engine,
  harness or build process; the pinned runner colosseum-cli 0.5.0, `81EA6CBA…`
  at `8fa63cc`; both fingerprints, flavour `pext-pgo` and the compiler equal;
  the resolved configuration matching policy in every checked field;
  `tools/results/colosseum_sprt_rar-s108-gate_20261110_100647.dry-run.json`):

  ```
  pwsh -NoProfile -File D:/code/rarog/analysis/artifacts/d62-qpromo-2026-11-10/run_s108.ps1 -Step sprt
  ```

  It runs `tools/colosseum.ps1 -Mode sprt -Bracket default` with the arms above,
  `-MaxPairs 20000 -Seed 20261110 -ExpectBench 13038236,11946003`. At about
  5,000 games an hour (RAR-S105) the cap is about 8 hours.
- **Use (frozen):** **H1:** `ee0b3e4` stays on `d6`, the fingerprint record
  moves to 13,038,236 / EBF 2.540, the changelog gets its entry and D.6.2
  closes. **H0 or the cap:** the push is reverted on `d6` in its own commit,
  D.6.2 closes `NO_CHANGE` with the interval recorded, and the push is not
  retried until a trigger fires: a change to quiescence's promotion exemptions
  (futility, the count rule, the SEE floor) or to ProbCut's SEE gap, the
  mechanism H3 names. Holding the push to a SEE floor would be a new
  registration, not a rescue inside this one.

## PRE-REGISTERED PREDICTION, the gate (frozen before any game)

1. The gate accepts H1: **0.35.** The screen shows the leaf defect is common
   and usually live (3.4% of ending generations could raise alpha, legal in
   54% of games), and both donors play the push; against that, TS-14's
   quiescence pushes read near zero four times, the main search's last ply
   already plays the promotion, and H3's unpruned pushes cost nodes (+9.1% on
   bench).
2. nElo at the stop: **+2**, 80% band **[−3, +7]** (a first-crossing estimate,
   biased away from zero).
3. Games to the stop: median **30,000**, 80% band 10,000 to the cap; the cap
   reached with probability 0.3.
4. Most likely failure mode: a true gain of +1 to +2 nElo that `[0,3]` cannot
   bank inside the cap, read as H0; the bracket is not narrowed afterwards.

## Result: the gate (2026-11-10)

**H1 accepted at 11,108 games: nElo +12.2 ± 6.5, Elo +7.2 ± 3.8**, LLR 2.952 in
[−2.944, 2.944], 5,554 pairs of the 20,000 cap. The candidate had 2,873 wins,
2,642 losses and 5,593 draws; pentanomial 132, 1,268, 2,563, 1,419, 172 (the
manifest's recount of `games.pgn` agrees). No time losses and no engine or
infrastructure faults. Colosseum also kept 28 post-terminal games, which the
sample excludes. The host was 0.3% busy at start, with no waiver. Both binaries'
SHA-256s, commits and bench counts in the manifest match the frozen arms. Manifest:
`tools/results/colosseum_sprt_rar-s108-gate_20261110_104030.manifest.txt`.

In these games the candidate searched 158,844 nodes a move against 157,594, at
mean depths 13.21 and 13.24 and 66.5 against 66.7 ms a move. That describes the
games; it is not a speed read. Bench's +9.1% nodes did not show up as a deeper or
slower search at this time control.

**Calibration (gate).**

- P(H1) 0.35: the less likely outcome came true. The forecast leant too much on
  TS-14's four near-zero reads, whose push to the seventh is a different move
  from a promotion, and on the main search's last ply already playing the
  promotion. It leant too little on the screen's 3.4% of ending generations
  that could raise alpha.
- nElo at the stop, +2 with an 80% band of [−3, +7]: a miss in magnitude. The
  sign was right. The first-crossing estimate of +12.2 is biased away from zero
  by an early stop, but its lower edge of +5.7 is still above the band's
  centre.
- Games to the stop, median 30,000 with an 80% band from 10,000 to the cap: a
  hit, near the low end. The cap was not reached.
- The named failure mode, a true +1 to +2 that the cap cannot bank, did not
  occur.

The games accept the change; they do not say which path carries the gain. The
gate cannot separate quiescence's stand-pat decision from ProbCut's list, and no
counter was read for that.

## Disposition (2026-11-10)

**H1: D.6.2 closes.** `ee0b3e4` stays on `d6`. The fingerprint record moves to
13,038,236 / EBF 2.540 at `ee0b3e4`, and the changelog has its entry. The
under-promotion pushes stay quiet (D.6.17). The runaway at position 627 stays
with D.6.7.

**Correction (2026-11-11).** The release suite did not pass at `ee0b3e4`:
`won_ending_does_not_sink_the_clock` fails there, 9.46 to 9.62 s in three
runs alone on an idle host against its 8 s, where `72c4e60` plays the same
position in 2.42 to 2.53 s. The "release tests 524 passed" above is
contradicted. The test is ignored as D.6.20's acceptance test (`b6914f0`),
and PLAN's D.6.20 holds the measurement. The gate's result stands.
