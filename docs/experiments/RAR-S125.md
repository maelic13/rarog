# RAR-S125 — D.6.31's gate: a count is a move made, the null move not counted, a 2,000-game fixed read with a harm rule

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-15, before any game; the gate is the maintainer's command.
- **Kind:** a contract change gated as a repair of case 2 (AGENTS, *Gating*), as D.6.8's count was (RAR-S124).
- **Why:** Stockfish counts a node in `do_move`, once per move made, quiescence's and ProbCut's included, and never for a null move. Reckless counts in its search `make_move` the same way, and its null move is never counted. Both were read in source: Stockfish `search.cpp` `do_move` and `do_null_move`; Reckless `0756957` `search.rs` `make_move` and line 597. Rarog's head, since D.6.8, also counts the position a null move reaches. That is about 1.7% of its count (RAR-O09's measured null-move share, 1.731% on 200 game positions). Every equal-node comparison with those engines needed a correction for it (RAR-O09's donor budget 127,800; RAR-O11 (e)). Basilisk is adopting Stockfish's count (maintainer, 2026-11-15). With this change, `go nodes N` buys the same work in Rarog, Stockfish, Reckless and Basilisk.
- **The change (`9d7b42f`):** the null move's `count_node` is removed. Every reader of the count reads the new one, as in D.6.8. A position reached by a move is counted when the move is made, as before; re-searches, the hand-off to quiescence and exclusion searches still count nothing.
  - *Test.* `a_null_move_adds_no_count`: a ten-iteration middlegame search that makes null moves reports exactly its moves made, a test-only tally at the three counting sites.
  - *Fingerprint.* `bench 13` 9,067,595 / EBF 2.479, from 8,935,848 / EBF 2.480. The count no longer includes null moves, and the LMR spread's phase moves with the count.
- **Mechanism and interactions:** three readers of the count change play, as in RAR-S124:
  - the LMR spread `(nodes + thread × 27) % 128` at each reduced move: a new phase, neutral in expectation;
  - the effort share at each root move: a ratio of two counts that both lose their null moves;
  - the poll, which fires about 1.7% later in positions searched.
  
  In the rated configuration (`3+0.03`, one thread, no node limit) nothing else reads the count. Under `go nodes` a budget buys about 1.7% more search than before.
- **Exposure (AGENTS, a repair's registration):** every search of every game reads the count, so the share of recent gate games that reach the change's domain is 100% by construction.
- **Invariants:**
  - The count is a move made: the test above checks it.
  - `go nodes` stops at the budget as before, the first iteration finishing first (`112eb14`).
  - `bench` counts the same thing as `nodes` and `nps`.
- **Gate (frozen):**
  - `stc`: Colosseum `match`, 2,000 games at `3+0.03`, Hash 64, one thread, the wrapper's slot count (14), the UHO book in random order, no adjudication, seed 20261115, `tools/results/rar-s125-stc`. The runner is Colosseum and does not change.
  - Harm rule (AGENTS' case 2, from the manifest's nElo): below **−13 nElo**. At about 7.8 nElo of standard error this catches a true −10 with probability 0.36 and −20 with 0.82, so "kept" means no harm beyond roughly −20 nElo.
  - Arms: `d631cand` at `9d7b42f` and `d631base` at `112eb14`, each built from a clean worktree by `tools/build_test.ps1`.
  - Driver: `analysis/artifacts/d631-count-2026-11-15/run_s125.ps1 -Step all` builds both arms and plays the match.
- **Binaries (appended before any game):** built 2026-11-15 by `run_s125.ps1 -Step build`, each from a clean worktree at its commit, rustc 1.100.0, `tools/build_test.ps1`:

  | Arm | Commit | `id name` | Bench | SHA-256 |
  | --- | --- | --- | --- | --- |
  | `d631base` | `112eb14` | `dev-20261113-112eb144` | 8,935,848 | `341056d51b9826faeef3756ac5a35deeedbafc26cc1e81451521c7ca9bf06f7f` |
  | `d631cand` | `9d7b42f` | `dev-20261115-9d7b42f9` | 9,067,595 | `105072328afe9b7290e089d2120945f9029947978d01c9e1e254b47cc83dca09` |

  The precondition (`947e5c5`): `every_search_parameter_moves_what_it_tunes` found `CoreSingDoubleNotTtPv` and `CoreSingTripleNotTtPv` dead on the candidate's tree at their probes (knight-up at depth 10, Semi-Slav at 12). A sweep of tune builds of both commits over the test's five positions at depths 8 to 14 found both trees live at knight-up 11 and Semi-Slav 13, so the probes were re-pinned there in their own commit before the change, and the test passes on both. Qualified on `9d7b42f`: `cargo fmt --check`; `cargo clippy --all-features --all-targets -- -D warnings`, 0 warnings; `cargo test --no-fail-fast` 529 passed, 0 failed, 12 ignored; `--release` 531 passed, 0 failed, 11 ignored. Dry run of `-Step stc` passed every preflight (host idle 2%, the pinned runner, both fingerprints, the policy): `tools/results/colosseum_match_rar-s125-stc_20261115_094727.dry-run.json`.
- **PRE-REGISTERED PREDICTION (frozen 2026-11-15, before any game):** `stc` **0 ± 8 nElo** (80% band). P(the harm rule does not fire) 0.94: the rule's own 5% false alarm at a true zero, and a little for a real loss. The basis is RAR-S124, a larger change of the same kind (a 17% shorter count), which read −6.2 ± 15.2.
- **Use of the reading, frozen:**
  - *The harm rule does not fire:* D.6.31 closes, the fingerprint record moves to the candidate's, and RAR-O12's arms are rebuilt in this count.
  - *The harm rule fires:* the change leaves the branch by the gate. The comparability need stays open in D.6.31, and a rework that gives the LMR spread a count of its own, so play does not move, is registered as a new experiment.
- **Result:** not measured.
- **Artifacts / commits:** this registration; the candidate commit; `analysis/artifacts/d631-count-2026-11-15/` (ignored).
