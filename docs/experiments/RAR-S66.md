# RAR-S66 — SUPERSEDED by B.2, never run (B.1, 2026-09-14): the switch or parameter this arm needed was removed as inert …

Indexed under *3. Search and selectivity › Search-accuracy decomposition* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

*This entry had 6 cells in a table of 4 columns (Experiment and conditions, Result / disposition, Conditional lesson, Source); its parts are kept in their original order.*

## Part 1

**SUPERSEDED by B.2, never run (B.1, 2026-09-14): the switch or parameter this arm needed was removed as inert; its question is answered inside B.2's cluster and SPSA or not at all, no retry trigger.** **Audit finding 2 — `improving` loses its fallback after a check. REGISTERED, NOT YET RUN.** When the node two plies back was in check its `static_eval` is `VALUE_NONE`, so `improving` is forced false regardless of the real trend. There is no walk-back to `ply - 4`. The candidate adds one. Arm A `rarog-46improving` `b517991`, bench **6,969,327 / EBF 2.459**. Arm B `rarog-46base` `e2fd4e0`, bench **7,467,143 / EBF 2.477** — the accepted head. Final-PGO both, `3+0.03`, 1T, 64 MB, paired UHO, RAR-M13 adjudication. **Registered bounds `[0,3]` nElo, cap 60,000, fixed before any games.**

## Part 2

**STOPPED at 13,882 games, not resolved, NOT promoted.** Elo **+3.48 ± 3.72**, nElo **+5.41 ± 5.78**, LLR **+1.35** of ±2.94, W-D-L 3,643-6,735-3,504. **The LLR peaked at +2.44 around 8,480 games and receded for the next 5,400**, tracking the estimate down from +8.40 → +4.89 → +3.48 Elo. Four time forfeits occurred during a Windows Update window — **2 per side, exactly balanced**, so no score impact, but the window is recorded as a contamination caveat because external load on a timed match is what `-use-affinity` exists to prevent. Stopped deliberately once the projection was clear; recorded as a STOP, per RAR-S54.

## Part 3

**The bracket was right this time and the candidate still did not clear it, which is the useful part.** `[0,3]` would have accepted a true +4 nElo in ~47k games; the estimate fell through that range instead of holding. ⚠ **Not accepted despite a CI of [0.35, 12.03] that barely excludes zero.** That shape is exactly what RAR-S61 had — +4.50 ± 3.50 at LOS 99.41%, every point of it a stale-read bug. Accepting on a point estimate makes the gate decorative. There is no RAR-S51 retention argument either: this is not a correctness fix, it unlocks nothing, and `improving = false` after a check is a conservative default rather than a defect — there genuinely is no comparable static eval at `ply - 2`. **A cheap lesson about candidate selection:** the audit produced four findings, two were real defects and were fixed, and both of the DESIGN-DIFFERENCE items (killers, `improving` fallback) went to a gate and neither cleared it. Design differences from Stockfish are not latent Elo.

## Part 4

`tools/results/sprt_46improving_vs_Head_20260820_114036.{pgn,log}`; **Arm A recipe** (branch deleted): set `improving_ply4_fallback = 1` in `params.rs`; rebuild and confirm `bench 13` = **6,969,327 / EBF 2.459**. `analysis/code_audit_2026_08_19.md`; RAR-S61; RAR-S65; RAR-M10

## Part 5

**First gate registered at a bracket that can actually accept what it is measuring.** RAR-S61, S64 and S65 all used `[0,10]` or `[3,10]`, which drive a true +4 nElo to H0 in ~35k and ~20k games respectively — configured to reject their own candidates. Fishtest's shape is `[0,2]` STC / `[0,1]` LTC, narrow and anchored at zero. `[0,3]` is chosen over `[0,2]` because RAR-M10 was FITTED on `[0,3]` gates, so it is in-regime rather than an extrapolation; it accepts a true +4 in ~47k games and a true +5 in ~34k. ⚠ **The 60,000 cap is a budget decision, not a statistical one, and it cannot decide everything.** It accepts a true ≥4, and reaches H0 for a true ≤0 at ~79k — so a genuine dud will hit the cap unresolved and must be reverted anyway. A true +2 needs 236k and is out of reach at any budget this project has. The mechanism argument stands independently of the bench proxy, which is why this one earns an overnight run: 9.7% of nodes lose `improving` outright, and `improving` is worth a full ply of LMR reduction and feeds the LMP margin, so the bias is toward MORE selectivity — the direction RAR-S53/S54/S55 and 4.7 all contradict.

## Part 6

`tools/test_engines/rarog-46{base,improving}-pext-pgo.exe`; branch `p45-improving-ply4`; `analysis/code_audit_2026_08_19.md`; RAR-S65; RAR-M10
