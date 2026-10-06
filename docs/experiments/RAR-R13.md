# RAR-R13 — D.1.1: the head's hard maximum in won endings; mechanisms of rec1–rec3 and their frequency in games — NO_CHANGE 2026-10-06

Indexed under *4. Root search, time management and SMP* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** 2026-10-06, agent research (PLAN D.1.1, pulled forward by the maintainer). No games; the untimed traces and the record reads are the agent's.
- **Baseline SHA / candidate SHA / dirty-diff hash:** engine source `ee02ed1`, Phase C's frozen head. No candidate.
- **Binary / compiler / PGO identity:** `tools/test_engines/rarog-b9head-pext-pgo.exe` (`aac92114…01ee`, 11,171,726 / EBF 2.512), the binary the Basilisk review timed.
- **Research question:** `analysis/basilisk_review_2026-10-06.md`, item 1. Without tablebases the head spends its hard maximum (52–58% of the clock) on rec1 and rec2 where Stockfish 19 does not, and stops rec3 at 10.4 s and depth 21 at both `60000+600` and `180000+2000`. What is each mechanism? Is rec2 rec1's cascade or a missing stop after a proved mate? What ends rec3? How often does each happen in games, and what does it cost?
- **Hypotheses examined:** (1) rec1 is B.5.2's aspiration cascade; (2) rec2 is the same cascade, or else (3) a search with no stop after a proved mate; (4) rec3 is stopped by something other than the clock, or else (5) by the clock's soft stop at iteration granularity.
- **Interacting mechanisms:** the aspiration loop (`Aspiration`, `AspMaxFails` 20, ×1.5 + 5 widening, centred on the previous score), the between-iteration soft stop (`soft_target_ms`), the hard maximum polled inside the tree (`check_stop`), and the ply cap (`MAX_PLY` 128).
- **Cheapest prior falsifier:** per-iteration UCI traces of the three positions, untimed (node-indexed and deterministic on one thread), plus counts from existing game records. No engine code changed.
- **Prediction:** none registered. The leaf is diagnosis, and its readings decide against implementation.
- **Result:** the full derivation is `analysis/d11_tm_diagnosis_2026-10-06.md`.
  - **rec1, hypothesis 1 holds:** at depth 32 the root fails high nine times on a window centred on the stale `cp 534`. Each fail-high returns exactly its beta (534 + 21, 36, 59, … 768). The tenth window, [513, 1,691], does not finish in 40 s here or 104 s in the review's run, with seldepth at the ply cap. Stockfish reads `cp 9165`.
  - **rec2, hypothesis 3 holds, not 2:** `mate 5` from depth 17 (24 ms) to depth 56 (1.43 s), with no bound lines, because aspiration is off at a mate centre. Depth 57 never finishes. Stockfish has no clock-mode mate stop either; its iterations stay cheap up to its depth cap of 245.
  - **rec3, hypothesis 5 holds:** depth 21 runs from 1.8 to 10.4 s after a six-step fail-high cascade. The soft target, from the formula with the trace's best-move history, is 2.1–2.7 s at `60000+600` and 6.7–8.7 s at `180000+2000`. Both fall inside that one iteration, so both clocks stop at its end.
  - **In games** (`tm_games.txt`): the RAR-M64 gauntlet at `3+0.03`, the RAR-M65 match at `10+0.1` and RAR-E19's self-play at `3+0.03`, 5,400 games and 334,245 head moves in all. Won-position moves at ≥ 0.9 × the maximum: 0.48%, 1.77% and 0.57%. Mate+ moves at that level: 0.24%, 2.59% and 0.46%, of which 20, 101 and 17 carry rec2's deep-mate signature. Every such move was in a game the head won, except one `10+0.1` draw whose long move was a depth-17 move at a 499 ms clock, not a deep stall. Unconverted won games with a won-position move at ≥ 3 × the optimum: 3, 1 and 3, each an ordinary depth-11–17 overrun below the maximum. Upper bound on the effect: about 0.5 Elo.
- **Disposition: NO_CHANGE.**
  - The aspiration loop's re-searches were measured useful (RAR-S89, −18.3 ± 6.9 for one retry; RAR-S17, −4.52 for re-centring), and changing the loop moves `bench 13`.
  - A clock-only fail-high stop would act on every overrunning iteration, so it is a time-formula change for D.1.2.
  - A mate-proved stop saves time only in won games.
  - Nothing lands ahead of Phase C, and its frozen head is unchanged.
- **Input to D.1.2:** moves at ≥ 3 × the optimum are 4.7% of non-won moves at `3+0.03` and 1.8% at `10+0.1`. Time past the optimum is 31.4% and 19.7% of all head time. A started iteration always completes, and a cascade can stretch it to about four times the soft target.
- **Conditional lesson:** on this head, the hard maximum is reached when one iteration cannot finish: a fail-high cascade on a rising score, or a proved mate whose next iteration grows to the ply cap. Both happen mainly where the game is already won, and the records show no result they cost at `3+0.03` or `10+0.1`. Frequency alone does not size a time defect; the outcome of the games it occurs in does.
- **Retry trigger:** reopen on any of these:
  - A game record at any rated control in which the head fails to win, or forfeits, after a won position, with the loss of the win preceded by one of these stalls: a move at ≥ 0.9 × the maximum at depth 40 or more, after a mate shown, or on a root lower bound.
  - A record at CCRL blitz (`2+1`) or longer showing such moves costing results.
  - D.1.2's candidate changing the bound model; it then reruns `tm_games.py` on its own games.
- **Artifacts / commits:** `analysis/artifacts/d11-tm-diagnosis-2026-10-06/` (ignored): `trace_rec.py` (`1a14b89e…`), `trace_rec1.txt` (`6c48d099…`), `trace_rec2.txt` (`e711e18e…`), `trace_rec3.txt` (`5c2e8ac3…`), `tm_games.py` and `tm_games.txt` (`706e207a…`), `tm_unconverted.py` and `tm_unconverted.txt` (`5df26f9d…`). Clock transcription: `analysis/artifacts/basilisk-review-2026-10-06/tm_table.py`.
