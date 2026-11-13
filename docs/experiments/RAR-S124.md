# RAR-S124 — D.6.8's gate: each position counted once, a 2,000-game fixed read with a harm rule — READ 2026-11-13: kept, nElo −6.2 ± 15.2

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-13, before any game. The run is the
  maintainer's command.
- **Kind:** a repair's harm read, AGENTS' repair case 2, as
  [RAR-S123](RAR-S123.md)'s frozen decision rule names it.
- Disposition: retained — read 2026-11-13: nElo −6.2 ± 15.2 against the −13 harm
  rule; D.6.8 closed, fingerprint 9,077,016 / EBF 2.481.
- **Inherits:** RAR-S123. 16.1% of `td.nodes` on `bench 13` counted a
  position the search had already counted (the hand-off to quiescence
  12.3%, the exclusion search 2.9%, razoring 0.9%), and LMR's and
  ProbCut's re-searches at least 1.2% more.

## The contract and the repair

RAR-S123's rule counts each position the search enters by a move or a null
move once, and left one question to the implementation: does a re-search
count again? It does not. A re-search enters a position a move already
reached, so it adds nothing, which is also the donor's contract (one count
per move made, `search.rs:1475` at Reckless `0756957`). That contract is
what D.7.2's equal-node match needs. The null move is counted, as the frozen
rule says; the donor does not count it.

- **`ba1d2cd`, the repair:** `check_stop` becomes `count_node`, called once
  after each move the main loop, ProbCut and quiescence make and after the
  null move. `negamax` and `quiescence` only read `td.stopped` on entry.
  Every consumer reads the one count: reported `nodes` and `nps`, `go
  nodes`, the bench, the LMR spread's phase, the root effort share and the
  poll cadence. No old clock is kept.
- **Two test preconditions, each in its own commit before it, read on both
  trees:** `56da869`, the transposing promotion anchor pins the
  transposition (after `Rxa8 Rxa8` the queen and the rook reach one
  position) instead of a margin between two separate searches of a tie
  (481 against 435 on the old tree, 572 against 573 on the new);
  `10059f9`, the `CoreSingDoubleNotTtPv` liveness probe moves from the pawn
  ending at depth 9, live on the old tree at depths 8 to 13 and on the new
  at none, to the knight-up position at depth 10, live on both.
- **Its test:** `a_depth_one_search_counts_each_root_move_once`: a depth-1
  search from the start position counts 20, its root moves. On the old
  tree the same search counted 51, so the test fails there.

## Wire and fingerprint (read before registration)

- **Bench:** `bench 13` reads **9,077,016 / EBF 2.481** on `ba1d2cd`, against
  11,477,040 / EBF 2.537 on `10059f9` (both rustc 1.100.0).
- **What the count alone does, on one tree:** an uncommitted verification
  arm kept the old per-entry clock for the LMR spread only. Its old clock
  read exactly **11,477,040**, so its tree was the old one, and on that tree
  the new count read **9,475,302** (×0.826). RAR-S123's counts bound it from
  above at 9,489,974 (11,477,040 less its four paths and the counted
  re-searches, plus the 61 verifications, which no move reaches). The
  14,672 left is the PV's full-window re-searches, which RAR-S123 noted had
  no counter. The rest of the move to 9,077,016 is the spread's new phase.
- **Qualified on `ba1d2cd`:** `cargo fmt --check`; `cargo clippy
  --all-features --all-targets` and `--all-targets`, 0 warnings; `cargo test
  --no-fail-fast` 525 passed, 0 failed, 12 ignored; `--release` 527 passed,
  0 failed, 11 ignored (`analysis/artifacts/d68-impl/`).

## Exposure (AGENTS, a repair's registration)

Every search of every game reads the count: the LMR spread at every reduced
move, the effort share at every root move, the poll on every 2048th count.
So the share of recent gate games that reach the repair's domain is 100% by
construction, RAR-S122's 12,000 included; no database query can make it
lower. In the rated configuration (`3+0.03`, one thread, no node limit) play
changes only through those three: the spread's phase, the effort share's
ratio (re-searches are no longer in it) and a poll that fires about every
2,440 old units instead of 2,048, under a tenth of a millisecond later.

## Gate (frozen)

- **`stc`:** Colosseum `match`, 2,000 games at `3+0.03`, Hash 64, one
  thread, the wrapper's slot count (14), the UHO book in random order, no
  adjudication, seed 20261113, `tools/results/rar-s124-stc`. The runner is
  Colosseum and does not change.
- **Harm rule** (AGENTS' case 2, from the manifest's nElo): below **−13
  nElo**. At about 7.8 nElo of standard error this catches a true −10 with
  probability 0.36 and −20 with 0.82, so "kept" means no harm beyond
  roughly −20 nElo.
- **Driver:** `analysis/artifacts/d68-count-2026-11-13/run_s124.ps1`
  (`-Step build`, done; `-Step stc`, the maintainer's). Its dry run passed
  every preflight: `tools/results/colosseum_match_rar-s124-stc_20261113_123659.dry-run.json`.

## Binaries (appended before any game)

Built 2026-11-13 by `run_s124.ps1 -Step build`, each from a clean worktree at
its commit, rustc 1.100.0, `tools/build_test.ps1`.

| Arm | Commit | `id name` | Bench | SHA-256 |
| --- | --- | --- | --- | --- |
| `d68base` | `10059f9` | `dev-20261113-10059f95` | 11,477,040 | `67889e9e0a4b58c24ed9a2bdbbd5921293109f15f223f06a8178cdf5b47b7c0c` |
| `d68cand` | `ba1d2cd` | `dev-20261113-ba1d2cde` | 9,077,016 | `6d0d4313568771f6848da47d2056c2e77f28c250208e36a4642ef767d8b4ad3d` |

## PRE-REGISTERED PREDICTION (frozen 2026-11-13, before any game)

- **`stc`:** **0 ± 10 nElo** (80% band). P(the harm rule does not fire)
  0.94: the rule's own 5% false alarm at a true zero, and a little for a
  real loss through the effort share.
- **Basis:** the spread is a dither whose phase carries no meaning, so a
  new phase is neutral in expectation; the effort share's thresholds were
  fitted on a ratio that counted re-searches, and the PV move carries most
  of them, so its share falls slightly and the soft stop may come a little
  later; the poll's lag is far under the move overhead. None has a known
  sign at the size a 2,000-game read can see.
- **Most likely failure mode:** a loss of a few nElo through the effort
  share's refitted meaning, which this read cannot resolve.
- **Evidence layer:** games. The bench's −20.9% is a change of unit and of
  the spread's phase, not of work, and says nothing of Elo.

## Dispositions (frozen)

- **The harm rule does not fire:** D.6.8 closes, the fingerprint record
  moves to 9,077,016 / EBF 2.481, and the changelog gets its entry (reported
  `nodes` and `nps` and `go nodes` count each position once).
- **The harm rule fires:** `ba1d2cd` leaves `pre-2.7.0` by the gate, the
  defect stays open in D.6.8, and a rework is registered as a new
  experiment (AGENTS, *Gating*).

## Result (2026-11-13)

**The harm rule does not fire: nElo −6.2 ± 15.2, against the rule's −13.**
Elo −3.6 ± 8.9 over 2,000 paired games, W/D/L 480/1019/501 for the
candidate, pentanomial [24, 252, 474, 221, 29] (the manifest's recount of
`games.pgn` agrees). No time loss, no engine or infrastructure fault, no
abnormal game; host 1.3% busy at the start, no waiver. Run 2026-11-13
11:47–12:07 UTC by the maintainer; manifest
`tools/results/colosseum_match_rar-s124-stc_20261113_124724.manifest.txt`,
its log's result line 1205, the evidence in `tools/results/rar-s124-stc/`.
The manifest names both binaries by the SHA-256 above.

**In the games** (the manifest's search lines): the candidate searched
130,493 nodes a move in 66.6 ms against the baseline's 157,896 in 66.5 ms,
at mean depth 13.39 against 13.40. Its nodes a move are ×0.826 the
baseline's, the ratio the count alone gave on the bench's fixed tree: the
same work, counted in the new unit, so its reported 1.96M nps against 2.38M
is a change of unit, not of speed.

**Calibration (games).** Predicted 0 ± 10 nElo with P(no harm) 0.94.
Observed −6.2 ± 15.2: inside the band, the rule not fired. A hit at this
resolution; the read cannot separate −6 from 0, so the effort share's
direction, the named failure mode, is neither shown nor excluded. Depth and
time a move unchanged, which the prediction's "no real work changes"
implied.

**Disposition: retained.** D.6.8 closes. The fingerprint record moves to
9,077,016 / EBF 2.481 at `ba1d2cd`, and the changelog gets its entry. As
AGENTS' case 2 reads it, "kept" means no harm beyond roughly −20 nElo.
