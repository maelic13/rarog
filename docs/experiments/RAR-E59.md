# RAR-E59 — C.7.3: RAR-E58's phase slope against the instrument, the labels, game length and one term

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-13, before any number below was computed; agent-run (zero games). Starting point: a consult on RAR-E58 (2026-10-13), whose four hypotheses and lean are taken as given.
- **Kind:** research. One zero-game read over RAR-E58's dumps, a provenance join and one contributor dump; it decides whether C.7 closes `NO_CHANGE`. No engine change.
- **Baseline SHA / candidate SHA / dirty-diff hash:** branch `c7` at `f3c6328`; engine sources equal `ab2a6ab`'s (**14,197,472 / EBF 2.557**). The played and full scores are RAR-E58's (`c71/scores-9587-head.csv` `c64d5de1…`, `c72/scores-9587-full.csv` `f6ae59d8…`). The contributor dump comes from `rarog-texel --dump-groups --full`, added in `f3c6328` with its tests. Its score column must equal RAR-E58's full score on every row.
- **What RAR-E58 left:** at |played| > 500 (55,544 rows) the phase slope `r·(phase/128 − 0.5)` adds +1.318 ± 0.203% (played) and +0.908 ± 0.216% (full) over a hinge-spline re-pricing of Rarog's score. Large scores convert more often toward the endgame.
- **Hypotheses (the consult's ranking):**
  - **H1** *phase-correlated label accuracy:* labels are game outcomes corrected by Syzygy at six men or fewer, and phase is material-based, so endgame rows carry near-exact labels.
  - **H2** *remaining game length:* a correct +600 with 60 plies left is less decisive than with 15, because the datagen engine blunders at its node budget; this is not transferable knowledge.
  - **H3** *one positional term overclaims at large magnitude*, king danger first; the only hypothesis C.7 could fix.
  - **H4** *the single-K logistic form itself*, true by construction and owned by C.8's refit and C.10's margin retune.
  - The standard error is suspect first: rows of one game share a label, and RAR-E58's standard error treats rows as independent.
- **What the corpus carries:**
  - `hce-v4-tb/validation.csv` is `fen;label`, with no game id, ply or label-source field. Its manifest records the source PGN (`selfplay-b9head-n8000-s1-g612747.pgn`, `53D3AF08…`, 612,747 games at 8,000 nodes a move, no adjudication). It also records the split rule: a stable hash of each game's start (`extract.split_for`, 5% validation and 5% test) and 30,449 validation games, at most 16 rows per game.
  - A row is `board.fen()` at a known ply of its game (`extract.process_game`). So `c73/provenance.py` replays the validation games with the extractor's own split rule and byte ranges and joins rows by FEN. That recovers each row's game, ply and game length exactly.
  - The game counts by split must reproduce the manifest's (551,551 / 30,449 / 30,747) and every row must find a game, or the read is void. A FEN met in more than one validation game is assigned to one and counted.
  - Label source: only rows of six men or fewer were eligible for Syzygy relabelling (1,222 of 194,444 changed), so men ≥ 7 is exactly the set the tablebases could not have corrected. The relabelled rows are counted against `hce-v4/validation.csv`, the pre-relabel labels, row by row.
- **Registered order (frozen):** `analysis/artifacts/c73/run_e59.sh`, once, every exit status to `status.txt`: tree check; provenance and its count check; tuner build and `--dump-groups --full`; `read_e59.py`. The read script reuses `donor_residual.py`'s fitting and RAR-E58's comparator code unchanged, and in this order:
  0. **Wire:** RAR-E58's `phase_slope|C` at |played| > 500 reproduces exactly on both scores (+1.318 ± 0.203, +0.908 ± 0.216; difference under 1e-9), and the group dump's score equals the full score on every row. Otherwise exit 1 and void.
  1. **Game-clustered standard errors** for `phase_slope|C` at |played| > 500, played and full: a bootstrap by game (2,000 resamples, seed 20261013) of the ratio `Σd/Σe₀` over the fixed held-out errors, and the linearised ratio estimator beside it. Reported beside it, deciding nothing: the gain with folds assigned by game rather than by alternating row.
  2. **Rows the tablebases could not have corrected:** `phase_slope|C` at |played| > 500 with men ≥ 7, played and full, game-clustered. Reported beside it: the same without the donor's specialised-endgame rows (an all-zero trace, RAR-E58's post-hoc finding), and men ≥ 7 without them.
  3. **Plies to the game's end:** with `z` the cohort-standardised `log1p(plies left)`, the gain of `r·z` over `C`, and of the phase slope over `C + r·z`, played and full, game-clustered; the correlation of phase with `log1p(plies left)` is reported.
  4. **Dominant contributor**, among full scores above 500 cp at phase ≥ 32:
     - Each row's full score is split into material (piece values), mobility, threats (with hanging), king (its linear terms plus the frozen part, which holds the king-danger map's output and the mop-up) and other.
     - The dominant contributor is the group with the largest contribution in the score's direction.
     - Per group: rows, outcome agreement with the score's sign (draws count half), draw share and loss share apart, the rest's agreement, and the difference with a game-bootstrap standard error.
     - The count of rows whose frozen part is non-zero without the map firing (the mop-up's share) is reported as the king group's limit.
  - Nothing is added after the numbers are seen.
- **Rule (frozen):** decisions read the **full score**, with **game-clustered** standard errors (the bootstrap). The played score is reported. *At the line* means a gain of at least 0.10% at four clustered standard errors. A group is **below the rest** when it has at least 300 rows, its agreement is more than 3 points under the rest's, and that difference is at least two clustered standard errors. The size guard and the significance clause are this registration's refinement of the consult's three-point rule, so that a handful of rows cannot decide.
- **Use, frozen:**
  - **C.7 reopens a candidate only if** the slope is at the line in 1, in 2 and in 3 (over `C + r·z`), and **exactly one** group is below the rest in 4. The shape is then a magnitude cap on that group's term, with its own registration, a static falsifier (the capped rows' agreement rises and the whole-file loss does not worsen) and one `[0,3]` SPRT prepared for the maintainer.
  - **Otherwise C.7 closes `NO_CHANGE`.** That covers the slope under the line after 1 or 2; the slope absorbed by game length in 3; and no group, or more than one, below the rest in 4. The finding is recorded for C.8 (K by phase) and C.10 (`LazyMargin`).
  - Nothing is built from this read.
- **PRE-REGISTERED PREDICTION (freeze before exposure):**
  0. The wire holds (0.97); the provenance reproduces the manifest's game counts and matches every row (0.95); fewer than 0.5% of rows are ambiguous (0.9); no row of men ≥ 7 was relabelled (0.99).
  1. The game-clustered standard error of the full slope is at least 1.5 times the naive one (0.6). The full slope stays at the line under clustering (0.35).
  2. At men ≥ 7 the full slope is at the line (0.3).
  3. Over `C + r·z` the full slope falls under the line (0.6); `r·z` itself is at the line over `C` (0.6); phase and `log1p(plies left)` are positively correlated among decided rows (0.8).
  4. King-dominant rows agree more than 3 points below the rest (0.35); some group is below the rest by the rule (0.45); material-dominant rows agree most (0.7).
  5. **C.7 closes `NO_CHANGE` (0.85).**
  - Expected Elo: none. Static loss ranks questions and accepts nothing.
- **Cheapest prior falsifier:** read 1, the instrument's own error.
- **Artifacts:** `analysis/artifacts/c73/` (ignored storage): `run_e59.sh`, `provenance.py`, `read_e59.py`, and the outputs they name. Tooling: `f3c6328`.
