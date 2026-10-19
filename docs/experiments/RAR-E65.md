# RAR-E65 — C.13.4: the static evaluation against its own 8,000-node search on the held-out rows

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-15, before any held-out row was searched. The run (about 6 minutes) is the maintainer's command; the agent prepares, registers and records.
- **Kind:** research, zero games. It maps where the static evaluation disagrees with its own deeper search, and reads whether that disagreement carries outcome information. Candidates go to C.13.5, or the leaf closes `NO_CHANGE`. Nothing is built.
- **Baseline SHA / candidate SHA / dirty-diff hash:** branch `c7` at the registration commit; engine sources equal `1b56071`'s. The binary is `analysis/artifacts/c134/rarog-head.exe` (`68f1e660…`): a plain `cargo build --release` of this tree, copied, with bench **10,521,141 / EBF 2.521** checked. No candidate.
- **Departure from PLAN's wording, stated plainly:** PLAN names the datagen PGN comments as the search score. Those scores were searched by the cycle-1 head (`bfd008a`), whose evaluation RAR-E61 has since refitted. Search minus *current* static would therefore mix the refit's change into the disagreement. This read re-searches each held-out position with the current head instead: `go nodes 8000`, a fresh hash (`ucinewgame`, Hash 16, Threads 1), deterministic. The PGN comments are not used.
- **Rows and inputs:** `hce-v6-tb/validation.csv` (194,444 rows, `7ada0924…`), and RAR-E63's artifacts: the family dump `c133/dump.csv`, the played scores `c132/head_played.csv` and the provenance `c132/provenance.csv`.
- **Instrument** (`analysis/artifacts/c134/`, ignored storage): `search_dump.py` (16 engine processes, the score of the last complete `info` line), `check_det.py`, `read_e65.py` (which imports RAR-E63's loader and clustered bootstrap) and `run_e65.sh`.
- **Known exposure, stated before the read:**
  - RAR-E62 to RAR-E64 on these rows: no corrected definition at the line. Material × men and mobility × men are C.13.5 candidates, and the search may re-express them.
  - The smoke on training rows: on 2,000 rows the search output is identical across two shardings; on 30,000 rows the wires and controls held, 1.7% of rows were mates or |S| ≥ 2000, the median depth was 6, and S correlates 0.937 with the static score. No gain or map was printed.
- **Research question:** where does Rarog's static evaluation systematically disagree with its own 8,000-node search, beyond a monotone calibration? Does that disagreement predict outcomes beyond the static score, as a missing material-signature term, a family mis-shaped in a phase band, or a repricing of the families?
- **Hypotheses:**
  - **H0:** the disagreement is tactical (each row is quiet, but the search still finds tactics and deeper plans the static score cannot express). Once calibrated, it carries little outcome information, and every direction stays under the line: `NO_CHANGE`.
  - **H-mat:** some material signatures (minor against pawns, the exchange, queen against pieces) are mis-scored in some phase band, and the material map's direction reaches the line.
  - **H-dom:** a family over- or under-claims where it dominates (the king-danger map in the middlegame, passers in the endgame), and the dominant map's direction reaches the line.
  - **H-fam:** the search reprices the families. This is the fitting-target question: it would argue for C.13.6's blended target, not a term.
- **Model (frozen):**
  - `S` is the search score and `E` the played static score, both White POV.
  - Rows with a mate or |S| ≥ 2000 are excluded from the search model, and their directions are 0.
  - Two folds by game.
  - Calibration: `g`, a least-squares hinge spline of `E` per fold.
  - The oriented residual `u = s·(S − g(E))`, with `s = sign(E)`.
  - **Maps** (descriptive; game-clustered SE of each bucket mean; buckets of at least 500 rows):
    - *material*: the favoured side's minus the other's queens, rooks, minors and pawns, each clipped to [−2, 2], by phase band (engine phase 0–7, 8–15, 16–24);
    - *dominant*: the family with the largest oriented contribution, by phase band.
  - **Directions**, cross-fitted (fitted on one fold, applied to the other):
    - `D_mat`: shrunk bucket means of `u` on the material map, `n/(n+500)`, buckets with at least 200 fitting rows;
    - `D_dom`: the same on the dominant map;
    - `D_fam`: least squares of `u` on the twelve oriented families within each phase band;
    - `D_search`: `S` itself, as context; it decides nothing.
  - **Reading:** each direction is read in outcome loss over RAR-E58's spline of the played score, with logistic fits per game fold, against RAR-E59's game-clustered bootstrap.
- **Cohorts:** deciding, all rows. Reported only: the non-lazy rows and the three phase bands.
- **Rule (frozen):** a direction is **at the line** when its gain is at least **0.10%** and at least **4 game-clustered SE** over all rows.
- **Wires (void if any fails):**
  - W1: the dump's played score equals `--dump-scores`.
  - W2: the family sums equal the trace's linear score.
  - W3: every row has a game.
  - W4: every row was searched.
  - W5: the first 2,000 rows re-searched with 7 workers equal the full dump.
  - Controls: the label as a direction gains at least 50%; a seeded Gaussian direction stays under 4 SE.
  - The tree is clean, the engine sources are unchanged, and the binary hash matches.
- **Registered order (frozen):** `bash analysis/artifacts/c134/run_e65.sh`, once, with every exit status written to `status.txt`. Nothing is added after the numbers are seen.
- **Use, frozen:**
  - `D_mat` at the line: its buckets with at least 4 clustered SE, by |mean u| × rows, are named as a material-signature candidate for C.13.5, with one definition card. Where they overlap RAR-E63's material × men, the two share that card.
  - `D_dom` at the line: the family-in-band buckets that carry it are named as mis-shape candidates for C.13.5's cards.
  - `D_fam` at the line: the families' repricing toward the search is routed to C.13.6, the fitting target, not to a term.
  - None at the line: **C.13.4 closes `NO_CHANGE`**, with the maps recorded for C.12 and C.13.6.
- **PRE-REGISTERED PREDICTION (freeze before exposure):** all rows, gains in percent.
  1. Every wire holds (0.9). The median search depth is 6 to 7 (0.8).
  2. `D_search` (context) gains more than +1% (0.8): the search knows much the static score does not.
  3. `D_fam`: +0.05, 80% band [−0.02, +0.20]; at the line (0.25).
  4. `D_mat`: +0.03, band [−0.02, +0.12]; at the line (0.15).
  5. `D_dom`: +0.03, band [−0.02, +0.12]; at the line (0.15).
  6. **No direction at the line, and C.13.4 closes `NO_CHANGE` (0.55).**
  7. The material map's largest buckets by |mean u| × rows are endgame (phase 0–7) imbalances (0.6).
  - Expected Elo: none. Static loss ranks questions and accepts nothing.
- **Cheapest prior falsifier:** this read.
- **Result (run by the maintainer 2026-10-15, 15:03–15:08, once, after the registration commit `7a5ab60`; `analysis/artifacts/c134/`, every step exit 0, read from `status.txt`):**
  - *Wires, all held:*
    - The tree was clean, the engine sources unchanged and the binary hash matched.
    - W1 to W4 held.
    - W5: the 2,000 rows re-searched with 7 workers equal the full dump.
    - Controls: label 100%, noise at most 0.34 SE.
  - *Search:* 191,149 usable rows; 2,437 mates and 858 at |S| ≥ 2000 excluded. Median depth 6. S correlates 0.935 with E. The calibrated residual `u` has a standard deviation of 179 cp.
  - *Directions in outcome loss* (± game-clustered SE; all rows decide):

    | Cohort (rows) | D_mat | D_dom | D_fam | D_search (context) |
    |---|---:|---:|---:|---:|
    | **all** (194,444) | **+0.046 ± 0.012** (4.0 SE) | +0.017 ± 0.013 | +0.019 ± 0.018 | +10.39 ± 0.21 |
    | non-lazy (144,537) | +0.066 ± 0.019 (3.5 SE) | +0.016 | +0.021 | +10.12 |
    | phase 0–7 (77,777) | +0.112 ± 0.035 (3.3 SE) | +0.007 | −0.004 | +19.65 |
    | phase 8–15 (52,030) | +0.018 | −0.012 | +0.003 | +11.95 |
    | phase 16–24 (64,637) | +0.045 | +0.054 ± 0.035 | +0.021 | +7.15 |

  - *By the frozen rule, no direction is at the line.* D_mat passes the 4 SE clause but not the 0.10% clause over all rows. Its reported-only endgame band (+0.112, 3.3 SE) passes neither.
  - *Maps (descriptive):*
    - Of 47 material buckets with at least 500 rows, 11 show the search departing from the static score by 50 cp or more on average; together they cover 9.6% of rows, and 12 of the 15 largest are in the endgame band.
    - The search is far less impressed than the static score by a queen or a rook up (−152 and −114 cp, and −160 with a queen and a minor up), and more by a minor or a rook with pawns up (+109 to +164 cp).
    - Yet the cross-fitted correction from these buckets predicts outcomes by only +0.046%. The search's large corrections there are mostly transient, the material of an exchange or promotion in progress that a quiet filter does not remove, and the game result does not follow them.
    - The dominant-term map's largest departures are under 25 cp: piece-square-dominant middlegame rows −13, passer-dominant endgame rows +14, threats-dominant middlegame rows −12.
  - *Context:* the 8,000-node search score adds +10.4% over the static score, and +19.7% in the endgame band. That is the size of what a search-informed fitting target could teach, C.13.6's question, not a term's.
- **Disposition:** observation. **By the frozen use, C.13.4 closes `NO_CHANGE`:** neither map names a C.13.5 candidate, and the family repricing does not reach the line, so nothing goes to C.13.6 from this rule. The maps are recorded for C.12 and C.13.6, and D_search's size is recorded as context for C.13.6's blended target. RAR-E63's two candidates are unchanged. Not Elo.
- **PREDICTION CALIBRATION (appended after exposure):**
  - Original prediction (not rewritten): items 1 to 7 above.
  - Observed:
    - (1) hit, both.
    - (2) hit: +10.4.
    - (3) D_fam +0.019 inside its band, not at the line, hit.
    - (4) D_mat +0.046 inside its band, not at the line, hit.
    - (5) D_dom +0.017 inside its band, hit.
    - (6) hit: `NO_CHANGE`.
    - (7) hit: 12 of the 15 largest material buckets are in the endgame band.
  - Which assumption held: the disagreement is mostly tactical. One the prediction did not anticipate: the size of the search's departures in the imbalance buckets (100 to 160 cp), which do not carry through to outcomes.
- **Conditional lesson:** a large, significant search-against-static departure in a bucket is not evidence of a missing term. Read it in outcome loss before naming a mechanism, since transient material is where search and static score differ most.
- **Retry trigger or `closed`:** closed. A retry needs a static evaluation with a different quiet filter, or a fitting target that takes search scores (C.13.6), either of which changes what this read measures.
- **Artifacts:** `analysis/artifacts/c134/` (ignored storage):
  - Scripts: `run_e65.sh`, `search_dump.py`, `check_det.py`, `read_e65.py`.
  - Binary: `rarog-head.exe` (`68f1e660…`), with `bench.log`.
  - Status and outputs: `status.txt`, `tree-status.txt`, `source.txt`, `search.csv`, `search.log`, `det.log`, `read.json`, `read.log`.
  - Smoke files: `smoke*`.
