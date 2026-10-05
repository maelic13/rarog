# RAR-S37 — Phase-4.4c: potential-singularity guard, tightenable NMP material floor, and the double-extension margin as a …

Indexed under *3. Search and selectivity › Rejected, neutral or deferred* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Phase-4.4c: potential-singularity guard, tightenable NMP material floor, and the double-extension margin as a coordinate. Landed inert, sized alone on `bench 13`, plus the registered first bundle measured as a set. Idle host; defaults inert on normal, diag and tune builds.

## Result / disposition

**Diagnostic.** Alone versus 6,502,902: `NmpSingularGuard=1` 7,239,391 (+11.33%), `NmpMinNonPawnPieces=2` 7,352,355 (+13.06%), `=3` 7,122,560 (+9.53%), `SingularDoubleMargin=60` **6,356,465 (−2.25%)**. The registered first bundle (`SingularRejectSpeculative` + `NmpSuppressNullInVerification` + `RazorAllowTtPv` + `NmpDecisiveGuard`) measures **6,401,087, i.e. 1.57% FEWER nodes than baseline**.

## Conditional lesson and retry trigger

Two structural findings. (a) The material floor is **non-monotone** — 3 costs less than 2 — another instance of "non-monotone ≠ converged", and a warning against reading these as a smooth curve. (b) Restricting double extensions by a **larger margin saves** nodes (−2.25%) while removing them outright via `SingularMaxExtension=1` **costs** them (+6.57%), though both reduce doubles: the margin downgrades only marginal cases while the cap also downgrades strongly-singular ones, and losing a critical line costs more work elsewhere than it saves. `SingularDoubleMargin` stays inert rather than joining the bundle because 60 was an arbitrary probe and baking an untuned constant is the RAR-S13 trap; it belongs in the 4.10 fit. The bundle being *cheaper* than baseline is the point — unlike RAR-S34's candidate it will not be fighting a speed headwind. ⚠ But see the resolvability arithmetic in `PLAN.md` 4.4c: at its ~5 nElo prior this bundle sits inside the indifference region a 16,000-game `[3,10]` gate cannot resolve.

## Source

`src/params.rs`; `tests/zugzwang.rs`; Plan 4.4c
