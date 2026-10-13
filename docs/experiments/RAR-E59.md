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
- **Result (run 2026-10-13 14:46–14:48, once, after the registration commit `e5563a9`; `analysis/artifacts/c73/`, every step exit 0, read from `status.txt`; tree clean):**
  - *Wire (0):*
    - Provenance reproduces the manifest exactly: 612,747 games (551,551 / 30,449 / 30,747), 0 parse errors. All 194,444 rows match a game; 4 are ambiguous.
    - 1,222 rows were relabelled by Syzygy, none at men ≥ 7.
    - RAR-E58's `phase_slope|C` reproduces to the last digit (+1.3183 ± 0.2032 played, +0.9084 ± 0.2163 full).
    - The group dump's score equals the full score on every row.
  - *Reads 1 to 3* (|played| > 500: 55,544 rows from 18,602 games, about 3.0 rows a game; z by the game bootstrap, the linearised estimator within 0.01 of it everywhere):

    | Read | Played score | Full score (decides) |
    |---|---:|---:|
    | 1 `phase_slope\|C`, naive SE | +1.318 ± 0.203 | +0.908 ± 0.216 |
    | 1 the same, game-clustered | ± 0.258, **5.11 SE, line** | ± 0.279, **3.26 SE, under** |
    | 1 folds by game (reported) | +1.314, 5.06 SE | +0.862, 3.08 SE |
    | 2 men ≥ 7 (35,409 rows, 13,257 games) | +1.160, 4.46 SE, line | +0.845, **3.35 SE, under** |
    | 2 without the donor's specialised endgames (reported) | +1.140, 4.63 SE | +0.750, 2.87 SE |
    | 2 men ≥ 7 without them (reported) | +1.189, 4.62 SE | +0.867, 3.47 SE |
    | 3 `r·z(plies left)\|C` | +1.556, **6.04 SE, line** | +0.826, 3.13 SE |
    | 3 `phase_slope\|C + r·z` | +0.809, 3.46 SE | +0.701, **2.94 SE, under** |

    The game-clustered standard error is 1.29 times the naive one on the full score (1.27 on the played score). Phase and `log1p(plies left)` correlate +0.288 among decided rows.
  - *Read 4* (10,992 full scores above 500 cp at phase ≥ 32; agreement counts draws half; difference against the rest ± game-bootstrap SE):

    | Dominant contributor | Rows | Agreement | Draws | Losses | Against the rest | Mean phase | Mean \|score\| |
    |---|---:|---:|---:|---:|---:|---:|---:|
    | material | 8,008 | 0.955 | 3.8% | 2.6% | +0.012 ± 0.005 | 63 | 1,042 |
    | mobility | 0 | — | — | — | — | — | — |
    | threats | 27 | 0.981 | 3.7% | 0.0% | +0.030 ± 0.019 | 102 | 597 |
    | **king** | **1,106** | **0.927** | **6.8%** | **3.9%** | **−0.027 ± 0.009** | 82 | 667 |
    | other | 1,851 | 0.952 | 4.8% | 2.4% | +0.000 ± 0.006 | 68 | 802 |

    No group is below the rest by the rule. King is 2.7 points under it at 2.9 clustered SE, short of the 3-point clause. 983 rows have a frozen part without the king-danger flag.
  - *Post hoc, deciding nothing* (`post_hoc_king.py`, run after the result to name the king group's make-up): all 1,106 king-dominant rows have the king-danger map's flag set, mean frozen part 313 cp. None of the 983 unflagged frozen rows is king-dominant. So the king group is the danger map, not the mop-up.
- **Disposition:** by the frozen rule, **C.7 closes `NO_CHANGE`**. On the full score the slope falls under the line once standard errors count games (read 1, 3.26 SE), on rows the tablebases could not correct (read 2, 3.35 SE), and over game length (read 3, 2.94 SE). No contributor group is below the rest (read 4). Nothing reopens; nothing is built.
  - H1 is not needed to explain the slope: it survives men ≥ 7 about as strongly as all rows, so Syzygy relabelling is not its source.
  - H2 is supported on the played score: remaining game length carries more than phase (+1.556 against +0.809 once both are in).
  - H3's best case is the king-danger map: its large scores convert 2.7 points worse than the rest's, with more draws and losses, which is close to but under the registered line.
  - H4 stands by construction.
  - The played score's slope survives clustering (5.11 SE) where the full score's does not: the lazy score is the more phase-miscalibrated of the two.
  - Recorded for C.8 (K by phase; a refit that weights decided rows reads this slope) and for C.10 (`LazyMargin`, beside RAR-E58's note). The king-danger map's large-magnitude conversion goes with them as a measured lead, not a candidate. Not Elo.
- **PREDICTION CALIBRATION (appended after exposure):**
  - Original prediction (not rewritten): items 0 to 5 above.
  - Observed: (0) all four hit. (1) clustered SE at least 1.5 times naive, **miss** (1.29); the full slope at the line under clustering did not happen (0.35), consistent. (2) at the line at men ≥ 7 did not happen (0.3), consistent. (3) the full slope falls over `C + r·z`, hit; `r·z` at the line over `C` on the full score, **miss** (3.13 SE; it is 6.04 on the played score); positive correlation, hit (+0.288). (4) king more than 3 points below, **miss** narrowly (2.7); some group below by the rule did not happen (0.45), consistent; material agrees most, **miss** (threats 0.981 on 27 rows; material 0.955 is the highest of the large groups). (5) hit.
  - Which assumption failed: I took the design effect from the cap of 16 rows a game; in the decided cohort a game contributes about 3 rows, so clustering inflates the error by 1.29, not 1.5 or more. That was enough to move the full slope under the line, and the miss is in magnitude. I expected game length to act on the full score as strongly as on the played one. It acts mostly through the lazy score, which is itself a statement about the lazy gate.
- **Conditional lesson:** a corpus with a per-game cap still clusters its rows; count rows per game inside the cohort that decides, not across the file, before trusting a per-row standard error.
- **Retry trigger or `closed`:** closed; C.7 closes `NO_CHANGE`. Retry the king-danger magnitude lead if C.8's refit or C.10's tune leaves the map's large-score conversion at least 3 points under the rest at two clustered SE on a regenerated corpus, or if a game record shows losses from king-danger overestimates.
- **Artifacts:** `f3c6328` (`rarog-texel --dump-groups` and its tests); `analysis/artifacts/c73/` (ignored storage): `run_e59.sh`, `run.log`, `status.txt`, `source.txt`, `tree-status.txt`, `provenance.py` → `provenance.csv` (`6b1ea515…`), `provenance.json` and `provenance.log`, `build-tuner.log`, `groups-full.csv` (`d006dacf…`), `dump-groups.log`, `read_e59.py` → `read.json` (`efeee33a…`) and `read.log` (`03c85809…`), `post_hoc_king.py` → `post_hoc_king.log` (`c7713aca…`).
