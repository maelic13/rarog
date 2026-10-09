# RAR-M43 — SUPERSEDED 2026-09-09 by RAR-M44(d); raw session retained

Indexed under *3. Search and selectivity › Search-accuracy decomposition* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**SUPERSEDED 2026-09-09 by RAR-M44(d); raw session retained.** Two findings are wrong: the generation gap it reports was partly the HARNESS -- its Rarog 'legal moves' and 'legal captures' columns timed a 520-byte `MoveList` return copy that Basilisk's harness had already removed, so the two engines' columns never measured the same work -- and its Elo arithmetic priced only generation and make/unmake, omitting SEE (5.239%) and the never-compared check queries (5.179%). Current table: `analysis/board_comparison_411b19_2026-09-09.md`. Nothing below is deleted. **Board comparison re-measured after 4.11b, 2026-09-09.** Four arms in ONE session: `rarog-head` built from `c1a7713` with the RAR-M20 recipe flags (`fd4c83af...`), plus the three EXACT binaries RAR-M20 measured, re-timed rather than reused — `rarog-ca03a46` (`40f8fa53...`), Basilisk `d734766` (`7eeaff0c...`), Reckless `91b56c2` (`449897a1...`), all hash-matching the RAR-M20 manifest. Affinity mask 4, 150 ms warmup plus eleven 150 ms samples per workload, three cyclic orders, host busy 5.01–6.25% against a 12% rejection threshold.

## Result / disposition

**The control did not reproduce RAR-M20, which governs how everything else may be read.** The identical `ca03a46` binary measured **0.7% to 6.1% faster today** (perft 273.741 -> 290.493), a session-level offset on unchanged code. Comparing today's Rarog against RAR-M20's recorded Basilisk figure would have attributed that offset to 4.11b, so every figure here is within-session only. **4.11b's board delta** (head vs ca03a46, both today): make/unmake **+17.48%** — independently reproducing 4.11b.9's own +17.97% — two-ply +5.44%, perft +0.67%, legal moves −1.15%, legal captures −4.16%, threshold SEE **−8.07%**. **Gap to Basilisk, then -> now:** make/unmake 31.3% -> **11.8%** (19.5pp closed), two-ply 46.5% -> 38.9% (7.6pp), perft 39.2% -> 38.2%, legal moves 44.5% -> 46.2%, legal captures 20.9% -> 26.2%.

## Conditional lesson

**The SEE column being 8.07% slower is the 4.11b.5 repair, not noise** — it added a per-candidate selected-king legality test the old kernel skipped. 4.11b.9 saw this column down 1–2% and attributed it to code layout; against the pre-repair binary the true cost is visible, and it is bought and paid for by RAR-E15's **+12.12 ± 10.17 Elo**. Generation was untouched because 4.11b.8 was withdrawn and 4.11b.10/4.11b.11 closed `NO_CHANGE`, so the unchanged generation gap is the expected outcome, not a shortfall. **The remaining board gap is worth single-digit Elo:** at RAR-M36's shares, closing generation and make/unmake to Basilisk entirely gives about **+2.9% NPS, ~+5.7 Elo** (it was ~+4.4% before 4.11b), of which RAR-M41 has already banked +1.421%. Board throughput is not search speed and no Elo is claimed from this instrument. Threshold SEE stays **not comparable across engines** (RAR-M19/M29); it is compared only between the two Rarog arms. Round spread reached 9.41% (Reckless) and 5.80% (ca03a46), so single-column differences below roughly 5% are unresolved. `cargo bench` emitted an executable with the **same filename** as the original build, distinguished only by hash — exactly the trap the recipe warns about.

## Source

`analysis/archive/board_comparison_2026-09-09.md`; `tools/results/board-compare-20260909/`; `analysis/board_benchmark_recipe_2026-09-05.md`; RAR-M20; RAR-M36; RAR-M41; RAR-E15
