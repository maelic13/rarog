# RAR-S138 — D.6.7's repair: the ply cap at the donor's 240, its score bands with it; a zero-game read where the cap binds, then a 2,000-game fixed read with a harm rule — REGISTERED 2026-11-22, before either part is read

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-22, before part A's replay and
  before any game. Part A is the agent's; part B's games are the
  maintainer's command.
- **Kind:** PLAN D.6.7's repair, AGENTS' repair case 2: expected near zero
  in the rated configuration, where a small loss is acceptable for
  correctness (maintainer decision 2026-11-22: "zero-game read, then case 2").
- Disposition: open — registered 2026-11-22, neither part read yet.

## The change

- **Candidate `91bc561`:** `MAX_PLY` from 128 to 240, the donor's cap, in
  the search and in the table, and the mop-up's mirror of it. The mate
  band, the tablebase band and the decisive floor move with it
  (`TB_WIN_SCORE` 31,744 to 31,520, `TB_VALUE` 31,871 to 31,759), as the
  donor's do: with the bands fixed, a mate found past ply 128 would be
  stored as a tablebase-band score without its distance. The table's stored
  depth already saturates at 127. A test ties the table's cap to the
  search's. Precondition `788c216`, its own commit and read on both trees:
  the score display's test reads its band edges from the constants.
- **Baseline `b90e983`:** the cap at 128, `bench 13` 3,390,842 / EBF 2.317.
- **Fingerprint:** `bench 13` 3,374,901 / EBF 2.316. Only bench position 39,
  `1Q4R1/5k2/4rpp1/3K4/8/7p/8/8 b - - 2 9`, moves by itself (position 40
  through the shared table). Split by build: the search's cap alone leaves
  it at 355,397 nodes; the table's bands alone give the candidate's
  353,820. So the move comes from the bands. The likely carrier, not traced
  node by node: window arithmetic near mate scores (a margin added to a
  losing beta) lands between the old and new decisive floors and is now
  read as decisive.

## The four questions (AGENTS)

- **Mechanism.** At long time controls a line in an ending reaches ply 127
  and returns its raw evaluation (RAR-S137: 0.511% of searches at twenty
  times the games' node counts, none at 1 or 10/3). At 240 those lines run
  on, to the line bound or to quiescence.
- **Interactions.** The decisive band is read by the kernel's rules (null
  move, reductions, history, ProbCut's windows, the table's horizon rule);
  its floor moves down by 224. The per-thread arrays sized by the cap grow
  (the PV table from 128×128 to 240×240 moves).
- **Invariants.** Mates keep their distance through the table (unit tests
  at plies 0, 1 and 239); the table's cap is the search's (a test); every
  ordinary score prints below every tablebase win (the display's test).
- **Falsifier.** Part A: a correctness fault where the cap binds. Part B:
  the harm rule.

## Exposure, stated (AGENTS, a repair's registration)

- The cap's domain: none of RAR-S137's 4,107 searches at `3+0.03`'s node
  counts reaches ply 128, so a `3+0.03` read cannot see the cap; part B reads
  the bands' effect, part A the cap's.
- The bands' domain: 1 of the 40 bench positions; not measured in games.
- Speed, not a reading: six alternating unpooled `bench 13` runs give the
  candidate 0.997 of the baseline's median NPS, inside their spread.

## Part A: the zero-game read where the cap binds

- **Script.** `analysis/artifacts/d67-review-2026-11-22/replay_pair.py`
  (SHA-256 `d2fe6219…`), frozen with this registration.
- **Binaries.** Baseline `rarog-s137.exe` (`54f998d9…`, `b90e983`); arm
  `rarog-s138-arm.exe` (`117673dd…`, `id name Rarog dev-20261122-91bc5616`,
  `bench 13` 3,374,901 on the copy). Referee: Stockfish 19
  (`D:\chess\engines\stockfish\stockfish-19-windows-x86-64-universal.exe`),
  one thread, Hash 256, the 3-to-7-man Syzygy tables, 10,000,000 nodes.
- **Design.** The 14 games of RAR-S137's twenty-times read that hold its
  21 cap searches, replayed by each build as RAR-S137 replayed them (one
  process a game, Hash 128, one thread, the game's moves, `go nodes` at
  twenty times each recorded count). At every search: the best move, the
  last score, the largest seldepth, the move's legality. At each of the 21
  where the builds' moves differ, the referee searches the position after
  each move; the better reply score for the mover marks the better move.
- **Wire.** The baseline's replay reaches the cap again at all 21 of
  RAR-S137's cap searches (the replay is the same search).
- **Rule (frozen).** Part A stops the repair before any game on a
  correctness fault in either build's replay: an illegal move, a crash or
  stall, a mate score beyond 240 plies. A wire failure voids the read and
  stops it for diagnosis. Otherwise part B runs. The 21 cases are too few
  to read a direction: the move changes and the referee's tally are
  reported, not decided on.
- **Prediction (frozen).** The arm's seldepth passes 128 in at least 15 of
  the 21 (probability 0.8). The moves differ in at most 5 of the 21
  (probability 0.7). No correctness fault (probability 0.95).

## Part B: the 2,000-game read

- **Gate (frozen):** Colosseum `match`, 2,000 games at `3+0.03`, Hash 64, one
  thread, the wrapper's slot count, the UHO book in random order, no
  adjudication, seed 20261122, `tools/results/rar-s138-stc`. The runner is
  Colosseum and does not change. Arms `d67cap240` (`91bc561`) and `d67base`
  (`b90e983`), each built from a clean worktree by `tools/build_test.ps1`
  (pext, PGO).
- **Driver:** `analysis/artifacts/d67-review-2026-11-22/run_s138.ps1 -Step all`
  (SHA-256 `ad9793ae…`).
- **Harm rule (AGENTS' case 2, from the manifest's nElo):** below **−13
  nElo**. At about 7.8 nElo of standard error this catches a true −10 with
  probability 0.36 and −20 with 0.82, so "kept" means no harm beyond
  roughly −20 nElo.
- **Prediction (frozen):** 0 ± 8 nElo (80% band); probability 0.93 that the
  harm rule does not fire. Basis: RAR-S124 and RAR-S125, repairs of this
  case, read −6.2 and +5.9; this one changes one bench position in forty.
- **Consequences, frozen:**
  - *The rule does not fire:* `91bc561` is kept, D.6.7 closes, and the
    fingerprint record moves to 3,374,901 / EBF 2.316.
  - *The rule fires:* `91bc561` is reverted; D.6.7 stays open, and a rework
    that keeps the decisive floor where the rated configuration reads it is
    registered as a new experiment.

## Result

Not read yet.
