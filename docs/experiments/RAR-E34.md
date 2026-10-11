# RAR-E34 — C.5.3 residual step: bracketed conversion of KQ-K, KR-K, KBB-K, KBN-K and KQ-KR on the current head against Stockfish, and the search's rule-50 damping as the interaction arm

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-11, after both arms were built and benched and before any playout; agent-run (zero games: Syzygy-graded playouts).
- **Baseline SHA / candidate SHA / dirty-diff hash:** head = branch `c5` at `beb53b8`, engine source equal to `master`'s `36f04d5` (fingerprint **12,567,796 / EBF 2.535**). Interaction arm = that tree plus `analysis/artifacts/c53/damping0.patch` (one line in `src/search/params.rs`: `eval_rule50_damping` 100 → 0), reverted after its build. Reference = Stockfish `bf2d8bf6…` (RAR's 2026-09-05 budget-transfer study, `tools/results/budget-transfer-20260905/reference/truth-{60000,200000,600000}.json`).
- **Binary / compiler / PGO identity:** plain `cargo build --release` of each arm (`rustc 1.98.1`, no PGO). `analysis/artifacts/c53/rarog-head.exe` (`417b814b…`, bench **12,567,796 / EBF 2.535**, the declared fingerprint) and `rarog-damp0.exe` (`4cc281d2…`, bench **14,665,899 / EBF 2.545**, so the damping is live); patch `2f570230…`.
- **Research question:** C.5.3's card cites conversion deficits measured on the 2.4.0-era engine (2026-09-05, 100 positions a family, Syzygy-graded playouts against Stockfish at the same node budgets). KQ-KR was 23/13/3 positions behind at 60k/200k/600k nodes, KBN-K 10/0/0, KQ-K 2/0/0 and KR-K 4/0/0. Phase B rebuilt the search and Phase C refitted the evaluation since, and C.5.1 requires each family cluster to re-read its families on the current head first. Which deficits remain? Does the search's rule-50 damping, which shrinks a static score as the halfmove clock grows and so flattens a mate drive's gradient late in a bare-king ending, help or hurt conversion (the card: "sign not assumed")?
- **Hypothesis / proposed mechanism:** B's search and the refitted evaluation have closed most of the old KXK and KBN-K deficits at the deployment budgets. KQ-KR keeps one: its winning method (separate the rook from the king, then a fork or a mating net) is a deep search problem the mop-up term's edge-and-proximity pull does not encode. The damping, applied from the first move of a no-pawn ending, costs conversion near the fifty-move horizon.
- **Competing hypotheses:** (1) *all closed:* no family stays behind at 200k nodes, and C.5.3 has no defect to work on. (2) *damping helps:* the damping keeps the search from preferring long non-progressing lines with a high static score, and switching it off lowers conversion. (3) *neutral:* the damping moves conversion by less than this resolution (100 positions a family) can see.
- **Interacting mechanisms / consumers:** the mop-up term (`src/eval/endgame/mop_up.rs`), the specialised scales (none fires in these families), the search's rule-50 damping, its TT and the fifty-move handling. Consumers: C.5.3's unit, if any; C.10 (the damping is a search coordinate, so in Phase C it moves only through C.10's joint tune or a margin block, never through C.5.3).
- **PRE-REGISTERED PREDICTION (freeze before exposure):** paired against the reference, in converted positions out of the cohort's clean wins:
  1. KQ-K, KR-K and KBB-K convert all their wins at 200k and 600k (probability 0.85); KBN-K is at most one position behind at 200k (probability 0.7).
  2. KQ-KR is at least 3 behind at 200k (probability 0.5); point prediction 90 of 100 at 200k and 97 at 600k (the old 87 and 97).
  3. The damping-off arm differs from the head by at most two positions in every family and budget (probability 0.7). Where it differs, it converts more (probability 0.55).
  4. Probability that at least one family meets the residual rule below: 0.5.
  - Expected Elo: none.
- **Falsification criteria:** the read is void if any family's cohort hash differs from the reference's at any budget (`endgame_budget_bracket.py` refuses a disagreeing bracket), or if an arm does not reproduce its bench.
- **Cheapest prior falsifier:** the 60k arm alone; it runs first inside the bracket.
- **Registered order and stop rule (frozen):**
  - `python tools/diag/endgame_budget_bracket.py --engine analysis/artifacts/c53/rarog-head.exe --syzygy D:/chess/tablebases/syzygy3456 --families KQ-K,KR-K,KBB-K,KBN-K,KQ-KR --positions 100 --budgets 60000,200000,600000 --workers 14 --out-dir analysis/artifacts/c53/head`, and the same with `rarog-damp0.exe` into `analysis/artifacts/c53/damp0`. These are the 2026-09-05 study's settings: seed 6200600, 100 plies, Hash 16, the engine's tablebases off.
  - Paired comparison per family and budget against the reference's per-position records (the same FENs, by cohort hash), and between the two arms.
  - A descriptive game-level read: `tools/diag/conversion_audit.py` on RAR-E32's gate-1 and `10+0.1` PGNs for `rarog-c43threat-pext-pgo`, which is the current head's engine.
  - Nothing is added after the numbers are seen.
- **Use of the readings (frozen):**
  - A family has a **residual** if the head is at least 3 converted positions behind the reference at 200k, or at least 2 behind at 600k. Each family with a residual continues into C.5.3's diagnosis: its failed positions are classified by how they ended (ply limit, fifty-move, stalemate, insufficient material, a lost win), and the mechanism is named before any unit. Without a residual in any family, C.5.3 closes `NO_CHANGE` on this record.
  - The damping arm's difference is an **interaction reading only**. If the damping-off arm converts at least 3 more positions than the head in some family at 200k, the finding is recorded for C.10's joint tune (which owns `CoreEvalRule50Damping`) with this record as its evidence; C.5.3 does not change it. It accepts and rejects nothing.
- **Result:** pending.
- **Disposition:** pending.
- **PREDICTION CALIBRATION (append after exposure):** pending.
- **Artifacts / commits:** `analysis/artifacts/c53/` (ignored storage): both executables, `damping0.patch`, the bench outputs, and the bracket reports once run.
