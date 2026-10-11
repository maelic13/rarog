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
- **Result (run 2026-10-11 12:15–12:25, after the registration commit `2e6037f`; `run.sh`, both brackets exit 0; host idle; every family's cohort hash equal to the reference's at every budget; `compare.json`):** converted clean wins, reference / head / damping-off arm:

  | Family | Wins | 60k | 200k | 600k |
  |---|---:|---|---|---|
  | KQ-K | 100 | 100 / 100 / 100 | 100 / 100 / 100 | 100 / 100 / 100 |
  | KR-K | 100 | 100 / 100 / 100 | 100 / 100 / 100 | 100 / 100 / 100 |
  | KBB-K | 100 | 100 / 100 / 100 | 100 / 100 / 100 | 100 / 100 / 100 |
  | KBN-K | 98 | 98 / 98 / 97 | 98 / 98 / 98 | 98 / **96** / 98 |
  | KQ-KR | 100 | 100 / 99 / 99 | 100 / 100 / 99 | 100 / 100 / 100 |

  - **Residual rule:** KBN-K only (2 behind at 600k). The interaction rule did not fire: the damping-off arm is never 3 or more ahead at 200k (it differs from the head by +2, −1 and −1 in three cells).
  - **Diagnosis, as registered** (`kbnk_failures.py`, `replay.py`, `side_by_side.py`, `kbnk_speed.py`):
    - The two failed positions (`8/6k1/N7/8/2K5/8/8/3B4 w`, start DTZ 52; `6K1/6N1/8/2k5/8/2B5/8/8 w`, start DTZ 58) were replayed exactly in a fresh engine, ending in stalemate at plies 77 and 81 as the instrument recorded. Each had preserved the win on every graded move but the last.
    - The game was lost to the clock before the stalemate. By Syzygy's distance against the halfmove clock, the win was out of reach at about ply 67 (clock 68, 38 plies to go) and ply 74 (clock 75, after Bg5 raised the distance from 26 to 43). The engine scored 0 at the stalemating move, correctly, since the fifty-move draw was already forced.
    - The reported scores' swings (down to −930 for the bishop-and-knight side) are aspiration **lower bounds**: 72 of game 2's 81 final `info` lines are bounds. The static evaluation stayed at +1,100 to +2,400 cp for White.
    - Speed: KBN-K median plies to mate, reference 59 / 53 / 47 and head 57 / 57 / 57 at the three budgets; mate efficiency (plies per DTZ ply), reference 1.128 / 1.053 / 1.027, head 1.089 / 1.102 / 1.125, damping-off 1.155 / 1.100 / 1.091. The head's drive does not speed up with nodes, so starts near the fifty-move horizon can lose the race at any budget whose path is slow.
  - **Game level (descriptive):** `conversion_audit.py` on RAR-E32's 6,930 gate and `10+0.1` games for the head's engine: 126 draws and 5 losses after a persistent piece-up (131 thrown away), 10 wins and 127 draws saved from a persistent deficit. That is self-play against an equal opponent, not comparable with the pool figure in GUIDE's checkpoint.
- **Disposition:** **C.5.3 closes `NO_CHANGE` on expected value.**
  - The card's premise is gone: KQ-KR's 23/13/3 deficit is now 1/0/0, and KQ-K, KR-K and KBB-K convert everything.
  - The one residual is two KBN-K positions at 600k nodes, starting 52 and 58 plies from mate. The mechanism is a mop-up drive whose speed does not improve with depth; neither the stalemate (a symptom) nor the damping (no systematic effect) is the cause.
  - KBN-K occurs in 0.3% of games (ranking v2), so a fix moves too little to register a unit for.
  - **Retry trigger:** a KBN-K conversion failure in game records, or C.5.7 building its recorded KBNK distance-to-mate table option. Either would also replace the drive.
  - The damping finding (none at this resolution) is recorded for C.10, which owns `CoreEvalRule50Damping`. Not Elo.
- **PREDICTION CALIBRATION (appended after exposure):**
  - Original prediction (not rewritten): items 1 to 4 above.
  - Observed: (1) hit on both clauses. (2) **miss:** KQ-KR is level at 200k (prediction 90 against 100; probability 0.5 that it trailed by 3 or more). (3) first clause hit (at most 2 everywhere); second clause **miss** (one cell more, two fewer). (4) happened, through KBN-K at 600k, not through KQ-KR as I expected.
  - Which assumption failed: I scaled the old KQ-KR deficit down for Phase B's search but kept the mechanism (a deep search problem) alive. The rebuilt search solves KQ-KR outright at these budgets. The miss is in magnitude, from a stale prior.
- **Conditional lesson:** a deficit measured before a search rebuild is a hypothesis about the new engine, not a fact about it; re-read before planning work on it (C.5.1's rule, which this record confirms).
- **Retry trigger or `closed`:** closed `NO_CHANGE`; retry as above.
- **Artifacts / commits:** `analysis/artifacts/c53/` (ignored storage): both executables, `damping0.patch`, the bench outputs, and the bracket reports once run.
