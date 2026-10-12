# RAR-E51 — C.5.7 theory sweep (KBP-KB, KBP-KN, KNN-KP, KNN-K, KQ-KP): conversion against Stockfish, drawn overclaim, cost in games

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-12, before any number was computed; agent-run (zero games).
- **Baseline SHA / candidate SHA / dirty-diff hash:** the current head, engine `38cbb02` (**14,197,472 / EBF 2.557**), as `analysis/artifacts/c523/rarog-head.exe` (SHA-256 `b34990d3…`). Reference: Stockfish `bf2d8bf6…` (`D:/chess/results/budget-transfer-20260905/reference.exe`, RAR-E48's). Games: RAR-E42's gate 1 PGN (12,954 games, RAR-E48's).
- **Research question:** PLAN C.5.7: for the five sub-1% families, is any one served badly enough by the current evaluation to justify a recogniser in the endgame dispatcher, or do they close `NO_CHANGE`? What serves them today: KQ-KP has the textbook fortress scale (rook or bishop pawn on the seventh); KBP-KB with opposite bishops has C.5.2's opposite-bishop rule and, with a rook pawn and the wrong bishop, the wrong-corner draw; KNN-K is forced to 0; same-colour KBP-KB, KBP-KN and KNN-KP have no recogniser (C.5.1's record).
- **Families (fixed):** KBP-KB (generated with both bishop colours; the bishop-colour split is read inside it), KBP-KN, KNN-KP, KNN-K, KQ-KP. All five men or fewer, so Syzygy grades them.
- **Hypothesis / proposed mechanism:** the generic evaluation and the existing recognisers serve these families: the wins convert like Stockfish's, and whatever static overclaim the unrecognised families carry costs under 1 Elo in games because they occur in well under 1% of games each.
- **Competing hypotheses:** (1) *KNN-KP is a conversion defect:* the Troitzky wins are long and need the knights to blockade the pawn before mating, which 60,000 to 200,000 nodes without tables may not find. (2) *same-colour KBP-KB and KBP-KN overclaim and cost games:* nothing in the evaluation tells a drawn minor-piece-and-pawn ending from a won one, so draws are scored as a pawn up. (3) *KQ-KP's fortress rule is too narrow:* drawn KQ-KP outside the textbook case are scored as queen wins.
- **Interacting mechanisms / consumers:** `specialized_endgame_scale` (wrong-corner KBP, KQ-KP fortress), `opposite_bishop_scale`, the KNN-K zero in `scale_endgame`, C.6's passer terms and square rule, the mop-up, the search's tablebase probing in play (where Syzygy is loaded); C.5.8.
- **PRE-REGISTERED PREDICTION (freeze before exposure):**
  1. *Conversion* at 200k, head against Stockfish: within 2 positions in every family (probability 0.65); if one falls short, KNN-KP (0.5 of that).
  2. *Drawn overclaim* (+100 cp): KNN-K none (0.9); KNN-KP 50% or more (0.7); same-colour KBP-KB 50% or more (0.6); KBP-KN 50% or more (0.5); opposite-colour KBP-KB under 30% (0.5); KQ-KP 30% or more (0.5).
  3. *Games*: the five families together cost at most 1 Elo (probability 0.85).
  4. No family proceeds to a recogniser (0.6).
- **Falsification criteria:** void if a family's cohort hash differs between the head and the reference, if the head arm does not bench 14,197,472, or if the bishop-colour split of KBP-KB's drawn read does not reproduce `endgame_drawn.py`'s KBP-KB aggregate exactly.
- **Cheapest prior falsifier:** the drawn read (a minute).
- **Registered order (frozen):** into `analysis/artifacts/c57/`, with `FAM=KBP-KB,KBP-KN,KNN-KP,KNN-K,KQ-KP`:
  1. `endgame_budget_bracket.py --families $FAM --positions 200 --budgets 60000,200000,600000 --workers 14`, once for the head (`--out-dir head`) and once for the reference (`--out-dir ref`); paired by index with cohort hashes asserted (`compare.py`, RAR-E48's logic), KBP-KB's clean wins also split by bishop colour.
  2. `endgame_drawn.py --engine <head> --families $FAM --positions 1500 --nodes 60000 --threshold 100 --workers 14 --output drawn-head.json`.
  3. `drawn_split.py`: KBP-KB's drawn positions re-scored by `endgame_drawn.py`'s own worker on the same seed, nodes and threshold, kept by index and split by bishop colour; its total must equal step 2's KBP-KB.
  4. `ending_games.py --pgn tools/results/rar-e42-gate1/games.pgn --families $FAM --output games.json`.
  - Nothing is added after the numbers are seen. 200 positions a family rather than RAR-E48's 100, because three of these families have few clean wins (KNN-KP about one in five).
- **Rule (frozen, RAR-E48's):** a family (or KBP-KB's same- or opposite-colour half) **proceeds to a recogniser** only when both hold: (a) a residual (drawn overclaim of 50% or more over at least 100 drawn positions) or a conversion defect (3 or more positions fewer than Stockfish at 200k), and (b) a game-level expected value of 1 Elo or more (`ending_games.py`'s upper bound for the family).
- **Use, frozen:** a family that proceeds gets its own recogniser registration, built behind the dispatcher with Syzygy and promotion-closure tests and gated by what it can change (AGENTS, *Gating*); otherwise C.5.7 closes `NO_CHANGE` with the families served by what serves them today. KBNK's option on record (a startup distance-to-mate table) is not decided here.
- **Result:** pending.
