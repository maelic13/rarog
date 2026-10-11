# RAR-E41 — C.6.3 sub-term attribution: the donor's passed-pawn and pawn-structure components, rebuilt from the FEN and verified exactly, read as directions

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-11, after the instrument was built and smoke-verified on 4,000 rows and before any loss was computed with it; agent-run (zero games).
- **Baseline SHA / candidate SHA / dirty-diff hash:** the current head's evaluation (`6c73d65`'s, 12,567,796 / EBF 2.535), through RAR-E39's scores file. Instrument `tools/diag/pawn_subterms.py` (`8d93ccb`), on `threat_subterms.Side`'s attack maps (RAR-E31's, exact on 194,444 rows).
- **Binary / compiler / PGO identity:** none built. Scores: `analysis/artifacts/c43-screens/scores-9587-cand.csv` (`7963307b…`), Stockfish `9587eeeb`'s per-term trace on `hce-v4-tb` validation with the head's played score substituted. The donor's source is read from the `oracle/hybrid` tag (`hybrid/stockfish/src/evaluate.cpp`, `pawns.cpp`).
- **Research question:** RAR-E39 found both families carry signal in the endgame band and without queens (passed +0.204 and +0.196, pawns +0.121 and +0.141). Which components carry it, and are they terms Rarog lacks or terms Rarog has and prices differently?
- **Components (fixed):** passed: `p_rank` (the rank bonus), `p_king_them` (the enemy king's capped distance to the block square, ×19/4, by rank weight `w = 5r − 13`), `p_king_us` (our king's distance to the block square ×2 and to the next square, by `w`), `p_path` (the free-path bonus 35/20/9/0 by `w`), `p_defended` (+5 by `w` when the block square is defended or our rook or queen stands behind), `p_file` (the file adjustment), over the donor's passed set; pawns: `w_connected`, `w_isolated`, `w_unopposed` (weak unopposed, on isolated and backward pawns), `w_doubled_isolated`, `w_backward`, `w_doubled`, `w_lever` (weak lever: two enemy pawns attack an unsupported pawn), `w_blocked` (a pawn on the fifth or sixth rank blocked by an enemy pawn).
- **Rarog's counterparts (fixed before any number, from `src/eval/params.rs`, `pawns.rs`, `passers.rs`):**
  - *Same shape, so a carrier is pricing:* `p_rank` (passed by rank), `p_path` (free and safe path and stop per rank), `w_connected` (connected and phalanx by rank), `w_isolated`, `w_doubled`, `w_doubled_isolated`, `w_backward`.
  - *Lacking or in another shape:* `p_king_them` and `p_king_us` (Rarog's proximity is one weight on the symmetric difference of the kings' distances to the pawn itself, uncapped, not to the block square, no second push); `p_defended` (Rarog has rooks behind passers, not a defended block square); `p_file`; `w_unopposed`; `w_lever` (Rarog's lever term is one of our pawns attacking one of theirs, weight 0); `w_blocked` (Rarog has a blocked-passer term only).
- **Hypothesis / proposed mechanism:** the donor's passed residual is its king-proximity conditioning on the block square (asymmetric, capped, weighted by rank) and its path safety; the pawns residual is the weak-unopposed and blocked-pawn terms Rarog lacks.
- **Competing hypotheses:** (1) *pricing:* the carriers are the rank and path bonuses Rarog already has, so a refit, not a term, is the lever. (2) *spread:* no single component reaches the line; the family's gain is many small parts (as RAR-E31 found for threats' gated terms). (3) *the passed set:* the donor's set (its levers and candidate rules) differs from Rarog's simple one, and the residual follows the set rather than any bonus; this read cannot separate that and would show it as every passed component carrying a little.
- **Interacting mechanisms / consumers:** C.6's unit and refit; RAR-E40's square rule (already in the unit); C.5.2's opposite-bishop rule (counts passers with Rarog's set); the lazy gate (Rarog's passer terms after `eval_pawns` run below it); C.8's refit.
- **PRE-REGISTERED PREDICTION (freeze before exposure):**
  1. *Verify* passes on every row (probability 0.95).
  2. *Passed:* `p_king_them` is a carrier in the endgame band (probability 0.6); `p_path` is a carrier there (0.5); `p_rank` is not (0.6).
  3. *Pawns:* at least one of `w_unopposed`, `w_blocked`, `w_lever` is a carrier (0.4); otherwise the pawns residual is spread or pricing.
  4. *Separate weights:* the passed components fitted separately add at least 0.05 points over the family in the endgame band (0.5).
  5. *Outcome:* C.6 becomes `READY_FOR_IMPLEMENTATION` with at least one carrier Rarog lacks (0.6).
  - Expected Elo: none.
- **Falsification criteria:** void if verify finds a mismatch on any row where the donor computed the family, or if the shuffled null of either family reads outside ±0.03% in any cohort.
- **Cheapest prior falsifier:** the 4,000-row smoke verify (passed).
- **Registered order and rule (frozen):** into `analysis/artifacts/c6/subterms/`: `pawn_subterms.py collect --scores <the file above> --out . --workers 14`, then `verify` (any mismatch voids), then one `analyse`. Nothing is added after the numbers are seen.
  - A component is a **carrier** if removing it from its family costs at least **0.04 points at four standard errors** (paired, `rarog+<family>-<component>` against `rarog+<family>`) in the endgame band (phase < 32) or without queens.
- **Use, frozen:**
  - Carriers in the *lacking* list are C.6's unit content: C.6 becomes `READY_FOR_IMPLEMENTATION` with them (and RAR-E40's square rule), built in Rarog's structure in the donor's shape, each replacing the Rarog term it overlaps (the donor's block-square proximity replaces Rarog's pawn-distance form if either king component carries), refitted together, and the research record freezes the handoff.
  - Carriers in the *same shape* list are pricing: no new term, left to the unit's refit (or C.8's).
  - With no lacking carrier, C.6's family part closes `NO_CHANGE` as pricing; RAR-E40's square rule then waits for C.8's refit; the retry trigger is C.8's refit leaving the residual.
- **Result:** pending.
- **Artifacts / commits:** `8d93ccb`; `analysis/artifacts/c6/subterms/` (ignored storage).
