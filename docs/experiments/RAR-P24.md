# RAR-P24 — B.1 search restructure, behaviour-neutral, COMPLETE 2026-09-14

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**B.1 search restructure, behaviour-neutral, COMPLETE 2026-09-14.** Engine commits `866cf9b` (44 inert parameters, root confidence, SMP iteration skip and their diagnostics), `6188d92` (TT provenance: `evidence.rs` becomes `tt::TtProbe`), `4b48c35` (module split), `360bdc6` (`ThreadData`), `3a9d614` (sentinel `PlyArray` stack), `0b74231` (`NodeType` Root/Pv/NonPv), `fcf8a2a` (diagnostic re-key); tooling `c35260d`. Checks per commit: `bench 13` on isolated magic and PEXT release builds, all 40 per-position node counts and scores diffed against the pre-B.1 head; fmt; clippy `--all-features` and the CI sets (default, workspace, diag, tune, ablate) at `-D warnings`; release and debug suites. Diag build at stride 1 diffed counter by counter; the oracle differential (`hybrid/stockfish/src/stockfish.exe`, the `oracle/hybrid-diag` build, depth 8) run on a pre-B.1 and a B.1 diag build; the B.0 §11 probes re-run on B.1 PGO binary `pext-1`. Pooled PGO: `nps_build_pool.ps1 -Arch pext -Builds 3` at `fcf8a2a` (clean), then `nps_multibuild.ps1 -Cycles 10 -Repeats 3` against the RAR-M48 pool A, hashes re-verified against its manifest; host idle (3.4% mean, 5.5% max before measuring). No prospective NPS prediction was registered beyond the ±0.5% done criterion; a non-PGO alternating sanity read (+7%) was seen after the first four commits, before the pooled run.

## Result / disposition

**Fingerprint exact at every commit: 7,601,220 / EBF 2.474, 40/40 positions identical.** Tests 283 → 284 release (sentinel test added, provenance and root-confidence tests deleted with their subjects). Diag counters: 181 surviving counters identical under the §6.4 rename map, 105 deleted absent, and `board_see_threshold_calls` lower by exactly 397,617 = the deleted `check_order_safe` census's `see_ge` calls. Oracle differential: pre- and post-B.1 reports identical (82 lines). §11 baselines reproduced exactly: branching 1.630 with identical per-depth rows, WAC 200 at 100k and 237 at 400k with identical per-position depth/nodes/bestmove, median depth 16 at 300k, oracle agreement 40/50. **Pooled-PGO NPS: base 3,074,305, B.1 3,267,941, +6.30%, 95% bootstrap [+5.78%, +6.84%]**; best-of +6.46%; per-build medians 3,068,722–3,079,290 base and 3,260,928–3,274,272 B.1.

## Conditional lesson and retry trigger

A behaviour-neutral restructure is not speed-neutral when it deletes per-node work: the removed prospective-depth switch computed `lmr_reduction_units` for every move whether or not it was reduced, and the root-confidence snapshot ran every iteration. The ±0.5% criterion guarded against regression; a gain this size is recorded, not rejected. **The RAR-M48 pool read 3.6% slower on this host than on 2026-09-11**, so absolute MNPS figures from different days are not comparable; B.2.2's NPS floor is measured interleaved against `tools/results/nps-b1-20260914/`. A.2.3's inventory declared 99 parameters where the macro held 110; counts in analysis must be read off the source.

## Source

PLAN B.1; `analysis/search_programme_2026-09-13.md` §6, §11, §13.1; `analysis/consolidation_2026-09-10.md`; ignored `tools/results/nps-b1-20260914/` (manifest sha256 pext-1 `6925AB18975DB005…`, pext-2 `69E1E5921EF2988D…`, pext-3 `31D04F03638B5D2B…`), `tools/results/b1-20260914/` (probes, branching, both differential reports); RAR-M48; RAR-M50
