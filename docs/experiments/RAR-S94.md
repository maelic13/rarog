# RAR-S94 — B.5.2.2 gate, the tablebase repair with tables configured — REGISTERED 2026-10-01, before any game

Indexed under *3. Search and selectivity › Registered, open* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**B.5.2.2 gate, the tablebase repair with tables configured — REGISTERED 2026-10-01, before any game.** The RAR-S93 binaries. Colosseum `sprt` with `tools/colosseum/sprt-repair-tb.toml`: **`[-5,5]` nElo** (RAR-S62's repair bracket, pentanomial normalized), cap **6,000 pairs**, UHO random, `3+0.03`, Hash 64, one thread, the same `SyzygyPath` on both sides, seed 20261016, `tools/results/b52-gate`; dry run at policy. Runs only if RAR-S93 passes. **Rule, fixed here:** H1, or the cap without H0, keeps B.5.2.1 and closes B.5.2. H0 (the repair costs at least 5 nElo with tables) returns B.5.2 to `RESEARCH`, and the change is reverted or re-derived there. Bounds, cap and book never change after games. **Prediction, frozen:** 0 ± 3 Elo. Tablebase roots only choose among result-equal moves, and the in-search bound semantics change little from larger roots, so the most likely outcome is the cap, about 12,000 games; H0 probability 0.05. Maintainer-run, about 2.5 hours at the cap.

## Result / disposition

**Played 2026-10-01** (`tools/results/b52-gate`, 16 min 33 s, recount equal): **H1 accepted at 776 pairs**, LLR +2.98, **+11.4 ± 8.5 Elo** (+23.3 ± 17.3 nElo; 420-763-369, [9, 121, 470, 162, 14]); 16 post-terminal pairs kept as evidence. **One time loss by the candidate** (1 of the harness's 7 allowed; round 745): at a 7-man root (KR v KQRBN, not in the tables) with about 58 ms on the clock, the engine's last info read `time 25`, `bestmove` arrived at 85.3 ms against a 78.1 ms deadline, and the process was held off the CPU for 54.2 ms with 641 page faults. That is consistent with a blocking read of cold table pages in an in-search probe: probe sites are the same in both binaries, the root was not ranked, and the PV extension's box is zero where the time reserve binds. The record's falsifier (a time loss with tables configured) fired by its letter.

## Conditional lesson

**Calibration:** predicted 0 ± 3 Elo and the cap; read +11.4 ± 8.5 and H1 at 776 pairs. Sign right, magnitude missed: decisive tablebase values reach pruning and the root, and that is worth more than a choice among result-equal moves. No rated pool configures tables, so this gain does not reach the ratings. **Closed 2026-10-01** by the maintainer: B.5.2.1 kept, B.5.2 closed; the time loss goes to D.3 (PLAN B.5.2.2). *Corrected 2026-10-01 from the failed-games record, superseding the in-search-probe reading above:* the stall was in the candidate's PV extension. On the forfeited move it printed the extension's "requires more time" notice before `bestmove d1h1`, a capture into the 6-man tables, where the extension's first call is a DTZ root probe on files the search never reads. The box was not zero there: the reserve decides the hard limit only below 5.25 overheads of clock (about 52 ms), not 52. The loss is the candidate's own and the falsifier named a real defect; the closure stands as the maintainer's decision and the defect is D.3's input.

## Source

PLAN B.5.2.2; RAR-S62 (the bracket); RAR-S93
