# RAR-O01 — Stage-1 evaluator-isolation experiment

Indexed under *3. Search and selectivity › Search-oracle observations* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Stage-1 evaluator-isolation experiment. Branch `hybrid` at `75d0d43`: Stockfish `9587eeeb` board/search/TT/time management calls the exact released Rarog 2.3.2 HCE through a checked Rust DLL ABI; the same executable with `Use Rarog HCE=false` is the exact-revision Stockfish HCE control. Colosseum round robin x200, 2,400 games, `3+0.03`, 1T, paired UHO, concurrency 14, tablebases/ponder off; draw adjudication `\|cp\| <= 5` for 10 moves after ply 80 and resign adjudication `\|cp\| >= 1000` for 5 moves.

## Result / disposition

**Observation, completed.** Hybrid–Rarog 275-111-14, score 82.63%, about **+270.9 Elo**; Hybrid–Basilisk 1.9.3 248-100-52, 74.50%, about **+186.2 Elo**; Stockfish-HCE–Hybrid 309-64-27, 85.25%, about **+304.8 Elo**. Average NPS: control 2.3M, hybrid 1.5M, Basilisk 2.5M, Rarog 2.4M. The extreme ordering is clear, but evaluator-dependent adjudication makes the exact gaps unsuitable as release claims.

## Conditional lesson

The shipped Rarog HCE supports much stronger play under a mature search even while paying substantial adapter/throughput cost. This contradicts the premise that another broad HCE constant fit is the highest-value next step. It does not separate individual search or HCE mechanisms or prove that Stockfish contracts transfer independently.

## Source

Colosseum “Rarog Hybrid testing”, 2026-08-11; `hybrid/README.md`; PLAN §4
