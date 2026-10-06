# RAR-E23 — C.2 re-read of the donor-direction screen on `hce-v4-tb` validation rows, one Stockfish version for families and total

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-06, before the `hce-v4-tb` corpus exists and therefore before any of its rows is scored; agent-run after the corpus is frozen (zero games, static evaluation calls only).
- **Baseline SHA / candidate SHA / dirty-diff hash:** `dev` at this registration's commit; the evaluation scored is the head's (engine source unchanged in value since `ee02ed1`, fingerprint 11,171,726 / EBF 2.512). No candidate.
- **Binary / compiler / PGO identity:** as RAR-E17 and RAR-E22: the RAR-O03 oracle package `tools/test_engines/oracle-hybrid-2.4.0/` (`rarog-stockfish-hce-hybrid.exe` `42ABBDCA…41ED`, `rarog_hce.dll` `1E192A58…DED8`, Rarog's evaluation, equal to the head's raw evaluation by RAR-O04's premise check) for Rarog's played score and the control total; Stockfish 11 (`271B62DC…5B02`) for the collection pass; and the RAR-E21 instrumented Stockfish `9587eeeb` (`D33AC7A5…D5F0`, `tools/diag/patches/sf9587_king_subterms.patch`), archived on 2026-10-06 as `tools/test_engines/stockfish-9587eeeb-ksdump.exe`, for the families and the total. The read uses `9587eeeb`'s families and total, by RAR-E22's lesson.
- **Research question:** RAR-E17's retry trigger: does the family ranking survive labels from the B.9 search? RAR-E22 read, with one version, at seven men or more: `rarog+all_families` +2.73 ± 0.09% against `rarog+stockfish` +2.73 ± 0.09%, king +0.75%, material and imbalance +0.12, threats +0.10, passed +0.06, space +0.05, mobility +0.04, winnable +0.03, pawns +0.03, pieces +0.01; over all rows families +1.85 against total +3.59.
- **Hypothesis / proposed mechanism:** the ranking is a property of the two evaluations, not of which search labelled the games; labels from a stronger search are cleaner, so every gain may move a little, but king stays first by a wide margin.
- **Competing hypotheses:** (1) the `hce-v3` labels carried the E08 head's own blind spots, which the donor's families "explained"; with B.9 labels the king gain shrinks below the material or threats family. (2) Cleaner labels raise every family's gain proportionally (less label noise to fit through).
- **Interacting mechanisms / consumers:** C.3's order (king safety first) and C.3.4's static screen, which reads the king family's residual against the candidate.
- **PRE-REGISTERED PREDICTION (freeze before exposure):** gains in percent of Rarog's held-out mean squared error, `donor_residual.py analyse --within` (two-fold, refitted inside each cohort).
  1. Seven men or more: king is the largest single family (probability 0.85), at +0.45% to +1.05% (probability 0.7).
  2. Seven men or more: `rarog+all_families` within 0.6 points of +2.73% (probability 0.6), and its gap to `rarog+stockfish` under 0.3 points (probability 0.7).
  3. Seven men or more: mobility, pieces, space and pawns each under +0.15% (probability 0.75).
  4. All rows: `rarog+stockfish` between +2.5% and +4.5% (probability 0.7).
  - Expected Elo: none; static loss ranks questions and accepts nothing.
  - Most likely failure mode: prediction 1's band misses on the high side (cleaner labels, competing 2) with the ranking intact.
- **Falsification criteria:** the hypothesis is refuted if, at seven men or more, any family's gain exceeds king's by more than two of its standard errors. The instrument is void if any row fails to parse, if the control total and `9587eeeb`'s final evaluation correlate below 0.99 by rank over all rows, or if the row count differs from the validation file's.
- **Cheapest prior falsifier:** RAR-E22 itself (one version, `hce-v3-tb` labels); this is the label-source variation of it.
- **Registered gate and stop rule:** no gate; one collection over every validation row, one 9587 pass, one `analyse --within` run, no cohort added after the numbers are seen. **Use, frozen:** if king stays first at seven men or more, C.3 proceeds as frozen and the record says so; if another family exceeds king by the falsification line, C.3.4's gate registration is held and the reading goes to the maintainer before C.3 games, since C.3's order rests on RAR-E17/E21/E22.
- **Full conditions / provenance:**
  ```powershell
  python tools/diag/donor_residual.py collect --csv tools/texel/data/hce-v4-tb/validation.csv `
      --oracle tools/test_engines/oracle-hybrid-2.4.0/rarog-stockfish-hce-hybrid.exe `
      --sf11 D:/chess/engines/stockfish/stockfish-11-x64-bmi2.exe --out tools/results/donor-residual-hce-v4
  python tools/diag/donor_terms_9587.py --scores tools/results/donor-residual-hce-v4/scores.csv `
      --sf tools/test_engines/stockfish-9587eeeb-ksdump.exe --out tools/results/donor-residual-hce-v4/scores-9587.csv
  python tools/diag/donor_residual.py analyse --within --scores tools/results/donor-residual-hce-v4/scores-9587.csv `
      --out tools/results/donor-residual-hce-v4/report-9587-within.json
  ```
  The binaries' SHA-256 values are checked against the ones above before the collection; the corpus hash is the `hce-v4-tb` manifest's.
- **Result:** (to be appended)
- **Disposition:** (to be appended)
- **PREDICTION CALIBRATION (append after exposure):**
- **Retry trigger or `closed`:**
- **Artifacts / commits:** `tools/results/donor-residual-hce-v4/` (ignored storage).
