# RAR-S139 — D.6.29's repair: quiescence scores a stalemate in Stockfish's narrow form; a zero-game replay first, then SPRT `[-5,5]` at `3+0.03` — REGISTERED 2026-11-23, before either part is read

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-23, before the replay and before
  any game. The replay is the agent's; the games are the maintainer's
  command.
- **Kind:** PLAN D.6.29's repair, AGENTS' repair case 3 as the card froze
  it on 2026-11-13: it moves shared quiescence scores and could matter
  either way (maintainer decision 2026-11-23: Stockfish's form, gated).
- Disposition: open — registered 2026-11-23, neither part read yet.

## The change

- **Candidate `64993ff`:** in the kernel's quiescence, after the move loop,
  out of check with no move searched, the side to move is scored the draw
  (`draw_score`, as the main search scores a stalemate) when it has no free
  single pawn push, no piece but pawns, the move into the node took a
  knight or more, and no legal move. The pawn pushes are tested first; the
  legal move list is built on the stack only after every filter. Test:
  `a_stalemate_after_losing_the_last_piece_reads_as_a_draw` (a side whose
  last rook is taken reads the draw; with a pawn free to push it keeps its
  stand pat), which fails with the rule switched off (−359).
- **Baseline `626d06b`:** engine source identical to `91bc561`, `bench 13`
  3,374,901 / EBF 2.316.
- **Fingerprint:** `bench 13` 3,694,298 / EBF 2.324. Fresh, one process a
  position, 36 of the 40 positions are identical; positions 15, 38, 39 and
  40, all endings, move, and the rest move through the shared table.

## The form and the donors

- **Stockfish** added the test on 2025-04-15 (`d2d046c2a`) and narrowed it
  on 2026-04-15 (`5f3dcb92e`). Its authors' own words: quiescence "like
  most alpha-beta based engines" does not test for stalemate "because
  doing it each leaf position is to expensive and costs elo"; the narrow
  test "does not gain elo, but prevents SF from loosing/drawing games in an
  awkward way". Its gates were no-regression: an LTC 0.04 ± 2.0 Elo over
  24,416 games before the first commit; the narrowing passed `[-1.75,
  0.25]` at STC and LTC and read 0.24 ± 0.9 Elo on a stalemate book.
- **Reckless** has none: its quiescence returns a mate in check and
  otherwise stands pat; its only stalemate score is the main search's, at
  a node with no legal move (its 2023 `c3c89eae3`). It follows the common
  practice Stockfish's message describes.
- The form departs from Reckless, the kernel's donor, and is the
  maintainer's decision (2026-11-06 rule), taken 2026-11-23.

## The four questions (AGENTS)

- **Mechanism.** A capture that strips a side to king and blocked pawns
  can stalemate it; quiescence read that node as its stand pat, a material
  deficit, where it is a draw (RAR-M87's replay on 2.7.0: 19.2% of searches
  held such a misread of 100 cp or more). The main search scores the
  stalemate a ply up only where it searches the capture.
- **Interactions.** The draw passes through quiescence's fail-high blend and
  is stored at quiescence's depth; parents' stand-pat refinement and the
  correction read it. The rule is root-blind and adds no contempt.
- **Invariants.** No allocation and no large copy per node: the filters are
  bitboard tests and the move list lives on the stack, built only when all
  three filters pass. The test above pins the rule both ways.
- **Falsifier.** The replay below: no search moved kills it before games.

## Exposure, stated (AGENTS, a repair's registration)

Of RAR-M91's 8,800 kernel games, 5,862 (66.6%) reach a position where a
side has only king and pawns, the rule's precondition, each by losing a
piece (`analysis/artifacts/d629-2026-11-23/kp_exposure.py`, SHA-256
`c3bf83d5…`, output `4019fe9c…`). The trigger itself, a stalemate inside a
quiescence tree, is rarer and is what the replay reads.

## Part A: the zero-game replay (the card's cheapest kill)

- **Script:** `analysis/artifacts/d629-2026-11-23/s139_pair.py` (SHA-256
  `5fc84d78…`), frozen with this registration; it reuses RAR-S138's replay
  module at one times the node counts.
  *Clerical correction, 2026-11-23, before any count was read:* the frozen
  script failed to start (its worker processes could not import the replay
  module it loaded by path); the import alone was made a plain one, now
  SHA-256 `12a0cf37…`. The design is unchanged.
- **Binaries:** baseline `rarog-s138-arm.exe` (`117673dd…`, `91bc561`, the
  baseline's engine source); candidate `rarog-s139-cand.exe` (`657b3d4a…`,
  `id name Rarog dev-20261123-64993ff7`, `bench 13` 3,694,298 on the copy).
- **Design:** RAR-S137's 60 RAR-M91 games (seed 20261122), each build in
  its own process a game, Hash 128, one thread, the game's moves, `go nodes`
  at each kernel move's recorded count. Deviation from the card, stated: the
  card named RAR-M87's 200 games of 2.7.0; these are the kernel's own.
- **Rule (frozen, the card's):** no search whose best move or score differs
  closes D.6.29 `NO_CHANGE` with no games; any difference sends it to part
  B. An illegal move in either build stops it for diagnosis.
- **Prediction (frozen):** the score differs in 1% to 5% of searches and the
  move in 0.2% to 1.5% (probability 0.7), most in endings; no illegal move.

## Part B: the gate

- **Gate (frozen):** Colosseum SPRT `[-5,5]` (the harness's `repair`
  bracket, normalized, alpha = beta = 0.05), `3+0.03`, Hash 64, one thread,
  the wrapper's slot count, the UHO book in random order, no adjudication,
  cap 6,000 pairs (12,000 games), seed 20261123, `tools/results/rar-s139-sprt`.
  The runner is Colosseum and does not change. Arms `d629cand` (`64993ff`)
  and `d629base` (`626d06b`), each built from a clean worktree by
  `tools/build_test.ps1` (pext, PGO).
- **Driver:** `analysis/artifacts/d629-2026-11-23/run_s139.ps1 -Step all`
  (SHA-256 `d2def5ff…`).
- **Prediction (frozen):** nElo 0 ± 4 (80% band), from Stockfish's
  no-regression reads. Stop, from RAR-M86's table for this gate at a true
  zero: H0 0.34, H1 0.34, the cap 0.32.
- **Consequences, frozen:**
  - *H1, or the cap without H0:* `64993ff` is kept, D.6.29 closes, and the
    fingerprint record moves to 3,694,298 / EBF 2.324.
  - *H0:* `64993ff` is reverted; D.6.29 stays open (AGENTS: a revert never
    closes the defect), and a rework is registered as a new experiment.

## Result

Not read yet.
