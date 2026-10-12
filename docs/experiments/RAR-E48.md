# RAR-E48 — C.5.6 measure-first families (KPsK, KBPsK, KBPPKB, KQKRPs): conversion against Stockfish, drawn overclaim, cost in games, and RAR-E25's KBP-K progress breach retried

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-12, before any number was computed; agent-run (zero games).
- **Baseline SHA / candidate SHA / dirty-diff hash:** the current head, engine `38cbb02` (**14,197,472 / EBF 2.557**), as `analysis/artifacts/c523/rarog-head.exe`. Reference: Stockfish `bf2d8bf6…` (`D:/chess/results/budget-transfer-20260905/reference.exe`, RAR-E34's and RAR-E35's). Games: RAR-E42's gate 1 PGN (12,954 games). Floors: `tools/diag/endgame_floors.json`.
- **Research question:** PLAN C.5.6: measure the families C.5.1 left without a recogniser after C.5.2 (pawns against a bare king, bishop and pawns against a bare king, KBPP-KB, queen against rook and pawns), decide whether any specific recogniser is still justified, otherwise close them as served generically; and retry RAR-E25's waived breach (KBP-K DTZ progress 0.4102 → 0.3822 at 400 positions, −2.9 SE, conversion and win preservation unchanged).
- **Families (fixed):** KPP-K, KPPP-K (KPsK); KBP-K, KBPP-K (KBPsK); KBPP-KB (both bishop colours); KQ-KRP, KQ-KRPP (KQKRPs). All six men or fewer, so Syzygy grades them.
- **Hypothesis / proposed mechanism:** the generic evaluation, C.5.2's opposite-bishop rule and C.6's passer terms serve these families: the wins convert like Stockfish's, the drawn overclaims are moderate except where the opposite-bishop rule keeps the six-man level (KBPP-KB, by design, RAR-E46), and their cost in games is under a recogniser's worth. The KBP-K breach was a metric artefact: the truth tool marks DTZ progress as no measure of technique in pawn-move families.
- **Competing hypotheses:** (1) *KQ-KRP(P) is a conversion defect:* queen against rook and pawns is long and technical, and 60,000 nodes without tables fall short of Stockfish. (2) *KBPP-KB's overclaim costs games* (RAR-E46: 69.6% of its draws above +100 cp at search level). (3) *the KBP-K progress loss is real:* the engine converts but more slowly, and at the fifty-move rule that would cost wins at longer controls.
- **Interacting mechanisms / consumers:** the KPK bitbase (KP-K reached from KPP-K), the wrong-bishop KBP-K draw (`specialized_endgame_scale`), C.5.2's opposite-bishop rule, C.6's passer terms and square rule, the mop-up; C.5.7 (KBPKB, KQKP) and C.5.8.
- **PRE-REGISTERED PREDICTION (freeze before exposure):**
  1. *Conversion* at 200k, head against Stockfish: within 2 positions in every family (probability 0.6); if one falls short, KQ-KRPP (0.5 of that).
  2. *Drawn overclaim* (+100 cp): KBPP-KB at 50% or more (probability 0.7); every other family under 30% (0.7).
  3. *Games*: the seven families together cost at most 1 Elo (probability 0.8).
  4. *KBP-K breach*: the floors report passes on the current head (0.7), and the 400-position DTZ progress is within 2 standard errors of RAR-E25's pre-C.3 head (0.4102) (0.6).
  5. No family proceeds to a recogniser (0.75).
- **Falsification criteria:** void if a family's cohort hash differs between the head and the reference, or if the head arm does not bench 14,197,472.
- **Cheapest prior falsifier:** the drawn read (seconds).
- **Registered order (frozen):** into `analysis/artifacts/c56/`, with `FAM=KPP-K,KPPP-K,KBP-K,KBPP-K,KBPP-KB,KQ-KRP,KQ-KRPP`:
  1. `endgame_budget_bracket.py --families $FAM --positions 100 --budgets 60000,200000,600000 --workers 14`, once for the head (`--out-dir head`) and once for the reference (`--out-dir ref`); paired by index with cohort hashes asserted (`compare.py`, RAR-E36's logic).
  2. `endgame_drawn.py --engine <head> --families $FAM --positions 1500 --nodes 60000 --threshold 100 --workers 14 --output drawn-head.json`.
  3. `ending_games.py --pgn tools/results/rar-e42-gate1/games.pgn --families $FAM --output games.json`.
  4. The breach: `endgame_truth.py --engine <head>` at RAR-E25's settings (default families, 100 positions, 60,000 nodes, 100 plies, seed 6200600, hash 16, 14 workers) and `endgame_floors.py --report` on it; and `endgame_truth.py --families KBP-K --positions 400` at the same settings, read against RAR-E25's head (`analysis/artifacts/c3-screens/endgame/truth400-head.json`, 0.4102 over 4,998 checked moves; the cohort hashes must agree).
  - Nothing is added after the numbers are seen.
- **Rule (frozen):** a family **proceeds to a recogniser** only when both hold: (a) a residual (drawn overclaim of 50% or more over at least 100 drawn positions) or a conversion defect (3 or more positions fewer than Stockfish at 200k), and (b) a game-level expected value of 1 Elo or more. **The KBP-K breach is resolved** when the floors report on the current head shows no KBP-K breach beyond 2 standard errors and the 400-position DTZ progress is not below 0.4102 by more than 2 standard errors; otherwise it stays held in C.5.6 with a diagnosis owed.
- **Use, frozen:** a family that proceeds gets its own recogniser registration (as C.5.7's dispatcher would take it); otherwise C.5.6 closes `NO_CHANGE` with the families served generically. An unresolved breach keeps C.5.6 open on that item alone. KBPP-KB is also the retry path of the opposite-bishop boundary (RAR-E46): a recogniser that separates won from drawn KBPP-KB would let a lower factor act on the drawn ones only.
- **Result:** pending.
- **Artifacts / commits:** `519a566` (`ending_games.py`); `analysis/artifacts/c56/` (ignored storage).
