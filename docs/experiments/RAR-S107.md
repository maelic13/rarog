# RAR-S107 — D.6.1: castling classed neither quiet nor noisy, the screen and the harm read

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-09 before any read and any game. The
  screen is the agent's (deterministic, no idle host needed); the match is the
  maintainer's command.
- **Kind:** zero-game screen, then AGENTS' repair case 2 if the screen does not
  close the item: a classification repair that moves the fingerprint, expected
  near zero, where a small loss is acceptable for correctness.
- Disposition: retained — kept 2026-11-10 by its harm rule: nElo −4.9 ± 15.2
  against −13; D.6.1 closed, fingerprint 11,946,003.
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

## Result: the screen (2026-11-09, appended after exposure)

**No closing rule fired: D.6.1 is `READY_FOR_IMPLEMENTATION`.** The evidence
layer is counters and node counts, not strength. Recipe:
`analysis/arm_patches/d61-rar-s107-screen.patch` on `5c6fbd6`; its `bench 13`
reproduced 15,625,852 in both runs, and the counters never steer.

**Agreement test:** fails as predicted: of 1,256 legal moves, 7 disagree, all 7
castling (`e1g1` ×3, `e1c1` ×2, `e8g8`, `e8c8`).

**Counters over `bench 13`, one thread, stride 1:**

| Read | Value |
| --- | --- |
| Moves in the non-root loop | 18,025,293 |
| Castling among them | 47,279, or 0.262%, 1 in 381 |
| Positions with castling in the loop | 4 of 40 (1, 2, 5, 14) |
| Castling as the TT move | 6,200, or 13.1% |
| Mean move index | 2.39 |
| Quiet rules would prune | 10,335, or 21.9%: late-move 7,162, futility 3,126, history 47 |
| Fails either SEE threshold | 0 and 0 |
| Reached LMR | 23,604, or 49.9% |
| Mean noisy term − quiet term | +3,395, 3.3 plies more reduction |
| Differ by 512 or more | 23,133, or 98.0% of those |
| Cut off / failed low | 7,794 (16.5%) / 39,316 |

A move can count both as a shadow prune and in LMR, so the share of castling
moves whose decision changes lies between 48.9% and 70.8%. The rule's bound was
1%.

**The slots at the end of `bench 13`:**

| Castle | Noisy writes | Noisy slot | Quiet entry (end; range) |
| --- | --- | --- | --- |
| White kingside | 1,390 | −12,800 | −4,672; −6,985 to 7,180 |
| White queenside | 880 | −12,800 | −365; −7,084 to 4,622 |
| Black kingside | 931 | −12,800 | −2,605; −6,795 to 7,038 |
| Black queenside | 62 | −11,539 | −4,248; −7,674 to 6,073 |

No noisy slot rose above 0 at any position. The floor is reached fast: a single
depth-9 search of one castling position takes a slot to −11,945.

**Exposure,** from RAR-S106's 2,000 games: a castling right at the start in 96.2%,
castling legal at a played position in 95.2%, played in 90.2%, and legal at 4.58%
of all played positions.

**A premise in this record was wrong.** "Its place in the quiet order can only
rise" is false: the quiet table is indexed by from and to alone, so a rook or queen
moving e1–g1 shares castling's entry, and those entries do go negative. Castling
itself still never takes a malus there, and the mean LMR difference implies the
entries were well above zero where castling was searched, roughly +7,000 if the
gap term is taken as zero (an inference, not a read). That difference is against
today's quiet history, which the repair will lower, so +3,395 overstates what
the repair changes.

### Calibration (the screen)

| Prediction | Read | Verdict |
| --- | --- | --- |
| Agreement fails on castling only, 0.99 | 7 of 7 castling | Hit |
| Slots never above 0, 0.99 | Never | Hit |
| Slots with 50 or more writes at or below −6,000, 0.8 | All four, −11,539 or lower | Hit |
| `castle_seen` 0.02% to 0.5%, 0.7 | 0.262% | Hit |
| `castle_tt` under a quarter, 0.6 | 13.1% | Hit |
| No SEE failure, 0.9 | None | Hit |
| Shadow prunes 5% to 30%, 0.5 | 21.9% | Hit |
| LMR mean +200 to +1,200, 0.6 | +3,395 | Miss in magnitude, right sign: the quiet term was taken at about zero history |
| No closing rule, 0.85 | None fired | Hit |
| Exposure 40% to 90% legal, 30% to 70% played, 0.7 | 95.2%, 90.2% | Miss in magnitude: most UHO lines keep castling open |
| Fingerprint within ±3%, 0.8 | −23.55% | Miss in magnitude and mechanism, below |

**The fingerprint.** `bench 13` reads 11,946,003 / EBF 2.547, against 15,625,852 /
EBF 2.563. That is not the repair's tree size. Depth 13 in a fresh process per
position (`per_position.py`, Hash 64): the 35 positions without castling rights
search identical trees, one of the five with rights is a 46-node search that does
not move, the other four read 1.168, 1.064, 0.944 and 1.178, and the total goes
from 11,627,843 to 11,709,838, **+0.7%**. The −23.55% is the table and histories
that bench carries from position to position, already known to swing it: valid
table mappings alone spanned 11.9M to 15.6M (RAR-S106). The failed assumption is
that a rare move could not move bench far. These are node counts and say nothing
of Elo.

## Implementation (2026-11-09)

- **`abb2b71`, the test's precondition, in its own commit:** the repair moved the
  trees of two of the liveness sweep's deep probes, so three singular parameters
  read as dead on them; none of the three changed. A read at depths 8 to 13 on the
  five positions, on both trees, gave probes live on both:
  `CoreSingDoubleNotTtPv` on the pawn ending at 9, `CoreLmrSingularCap` on the two
  pieces at 9 (neither has castling), and `CoreSingTripleNotTtPv` on the Semi-Slav
  at 11, the only pin of that parameter live on both.
- **`72c4e60`, the repair:** `Move::is_noisy` and `Move::is_quiet`, one the other's
  negation; the two old predicates are gone. No constant or history shape moved.
- **Tests:** every legal move's class judged from what it does on the board, and a
  search that must leave castling's noisy slots at 0 and write its quiet entries.
  With the old predicate put back, both fail (the slot read −11,945).
- **Qualified:** `cargo fmt --check`, debug tests 521 passed, release tests 523
  passed, `cargo clippy --all-features --all-targets -- -D warnings`, all exit 0.
- **Fingerprint:** stays recorded as 15,625,852 until the disposition below.

## Binaries (appended before any game)

Built 2026-11-09, each from a clean worktree at its commit, rustc 1.99.1,
`tools/build_test.ps1`. Command: `run_s107.ps1 -Step stc` in
`analysis/artifacts/d61-castling-2026-11-09/`. Its dry run passed every preflight:
idle host, runner pin, both fingerprints, build flavour, compiler and policy.

| Arm | Commit | `id name` | Bench | SHA-256 |
| --- | --- | --- | --- | --- |
| `d61base` | `5c6fbd6` | `dev-20261109-5c6fbd6e` | 15,625,852 | `25eafd21c45e7c2257c543a4c6ae01891abc380e45a6717cea16144bf87a5fe2` |
| `d61cand` | `72c4e60` | `dev-20261109-72c4e608` | 11,946,003 | `02a3290c379ef8aa84bd0f665f0d6ab2285e6bd6fc30d789760f89e7dd42e0e1` |

## Dispositions (frozen)

- **The harm rule does not fire:** D.6.1 closes, the fingerprint record moves to
  11,946,003 / EBF 2.547, and the changelog gets its entry.
- **The harm rule fires:** the repair leaves `d6` by the gate, the defect stays
  open in D.6.1, and a refit of the constants tuned around it is a new registration.

## Result: `stc` (2026-11-10)

**The harm rule did not fire: nElo −4.9 ± 15.2, against the rule's −13.** Elo
−2.8 ± 8.7, candidate 494 wins, 510 losses and 996 draws, pentanomial 20, 251,
476, 231, 22, with no time losses and no engine or infrastructure faults. Host
0.2% busy at start, no waiver; both binaries' hashes in the manifest match the
table above. Manifest:
`tools/results/colosseum_match_rar-s107-stc_20261109_225709.manifest.txt`.

This keeps the repair and claims no strength. By the registered power, "kept"
means no harm beyond roughly −20 nElo.

In these games the candidate searched 157,679 nodes a move against 159,276, at
the same mean depth of 13.21 and 66.4 against 66.3 ms a move. That describes the
games; it is not a speed read.

**Calibration (games):** a hit. The prediction was 0 ± 15 nElo with P(no harm)
0.95, and the read is −4.9, inside it. An interval of ±15 cannot tell a small
loss from a small gain.

## Disposition (2026-11-10)

**The harm rule did not fire: D.6.1 closes.** The fingerprint record moves to
11,946,003 / EBF 2.547 at `72c4e60`, and the changelog has its entry.
