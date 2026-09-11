# Codebase consolidation analysis — PLAN A.6

Revision `7cffce5` on `dev`, 2026-09-10. Research leaf, class `R2`: this
document decides what Phase A refactors, what Phases B and C replace, and what
is deleted, and hands B.1 and C.1 their restructure scope. **No source
changed.** Line numbers below are at `7cffce5` and will drift; the region
names will not.

Method: every module was outlined mechanically (`rg` over `fn`/`struct`/
`impl`/`#[cfg]` declarations, cross-module `use` edges, lint suppressions, the
`search_params!`, `eval_params!` and `diag::counters::declare!` invocations,
and the crate items `tests/`, `benches/`, `xtask` and `tools/texel-tuner`
import). Nothing here rests on reading prose comments.

## Decision

**Phase A refactors nothing.** Every candidate refactor lives inside a file
that B.1 (`search.rs`, `params.rs`, `move_ordering.rs`, `search_threads.rs`,
`time_manager.rs`) or C.1 (`eval.rs`, `kpk.rs`) splits, and PLAN's rule is not
to refactor what a programme is about to replace. Moving code twice would
cost two fingerprint-and-NPS qualifications for zero strength, and A.7–A.9
must ship the head RAR-E16 licensed. The A.6 output is therefore this
document, corrections to PLAN's stale figures, and the two handoffs below.

## Crate inventory and disposition

24,641 lines under `src/` (PLAN's 24,103 predates A.4.2's `cpu_advice.rs`
and A.4.5's argv dispatch). Test code inside `src/` is counted with its file.

| Module | Lines | Owner today | Disposition |
|---|---|---|---|
| `board/` (9 files) | 5,467 | board, movegen, SEE, zobrist, attacks | **Keep as is.** Target layout says "as today". Only consumers change. |
| `search.rs` | 6,319 | everything from root to qsearch, picker, histories, correction, SMP glue, 1,031 lines of tests | **B.1 splits**, then B.2–B.5 replace mechanism by mechanism. |
| `params.rs` | 987 | `search_params!`, 99 entries | **B.1** → `search/params.rs`, minus the 42 inert entries (A.2.3). |
| `move_ordering.rs` | 252 | `ScoredMoveList`, `BadCaptureList`, history index math | **B.1** → `search/movepick.rs` (lists, `pick_next`) and `search/history.rs` (index math). |
| `search_threads.rs` | 522 | `SharedSearchState`, `WorkerPool` | **B.1** → `search/threads.rs`, minus the instability slots (dead, below). D.2 redesigns the SMP. |
| `time_manager.rs` | 371 | `compute_runtime_limits` | **B.1** → `search/time.rs`, joined by the `tm_*` helpers now in `search.rs`. D.1 rebuilds the policy. |
| `tt.rs` | 921 | table, entry packing, replacement | **Keep in place**; B.2 changes the entry format (stored eval, tt-pv) inside it. |
| `evidence.rs` | 707 | typed TT provenance (`NodeEvidence`, `MoveEvidence`, `OutcomeKind`) | **B.0 decides** (A.2.3 row stands): keep only with a named consumer that changes a search decision; otherwise B.1 removes it with `tests/tt_provenance.rs` and the `store_kind_*` counters. |
| `eval.rs` | 3,756 | evaluator, 137 `eval_params!` entries, trace, tune I/O, endgame scaling, tests | **C.1 splits**; C.3–C.7 replace family by family. |
| `kpk.rs` | 270 | KPK bitbase | **C.1** → `eval/endgame/`; its only consumer is `eval.rs:3090`. |
| `diag.rs` | 1,298 | 286 counters, probes, `smp` and `lazy_probe` submodules | **Keep**; B.1 re-keys the search counters to the new module names (B.0 owns the mapping); the two dead families below go with their mechanisms. |
| `engine.rs`, `engine_command.rs`, `uci_protocol.rs`, `search_options.rs`, `main.rs` | 2,055 | engine thread, command queue, protocol, options | **Keep**; D.3 moves them (with `bench.rs`, `wac.rs`) under `src/uci/` while auditing their semantics, never as a standalone step. |
| `syzygy.rs` | 684 | Fathom FFI | **Keep**; D.4 owns policy. |
| `infra.rs`, `crash_report.rs`, `cpu_advice.rs`, `bench.rs`, `wac.rs`, `lib.rs` | 1,032 | utilities and product surfaces | **Keep.** |

Module edges are clean for the split: `search.rs` is the only importer of
`params`, `move_ordering`, `search_threads`, `time_manager` and `evidence`
(plus `tt.rs` for `evidence`); `eval.rs` is the only importer of `kpk`. The
public crate surface the integration tests and tools rely on is small:
`board::*`, `search::{Searcher, SearchResult, SearchEvent, SearchExit}`,
`search_options::SearchOptions`, `params::SearchParams`, `eval::{Evaluator,
MATE_SCORE, piece_value}`, `tt`, `evidence::OutcomeKind` (three tests),
`wac`, `bench`, `syzygy`, `engine_command`. `tools/texel-tuner` imports only
`eval`, `board::Bitboard` and `board::Color`. Everything else is `pub(crate)`
or private and may move freely.

## Dead code list

A.2.3 classified 42 `SearchParams` entries as inert at default. Tracing
their guarded code finds three dead **subsystems** and a few loose items. All
are fingerprint-neutral to remove by construction (no default opens them);
B.1 proves it. None is removed in Phase A.

1. **Root confidence** — the largest. `RootConfidence` and its `impl`
   (`search.rs:229–356`, ~130 lines), `Searcher::root_confidence`
   (`1335–1391`), `tm_confidence_factor` (`425`), the seven `root_conf_*`
   parameters, `SharedSearchState::publish_instability` /
   `pooled_instability` and their slots (`search_threads.rs:133–165`),
   `diag::RootConfidenceShadow` and `record_root_confidence`
   (`diag.rs:947–1080`) with the 24 `rootconf_*` and 6 `shadow_*` counters,
   and ~210 lines of tests (`search.rs:5678–5888`, one in
   `search_threads.rs`). The snapshot is computed every root iteration and
   published to the pool even though every consumer is behind a `== 1` guard
   at default 0 (`1497` aspiration, `1760` time) or behind `diag` (`1672`):
   removing it is
   tree-neutral and a small speed gain. **Owner: B.1**, with the parameter
   removal. D.1 rebuilds time management from the donor shape and does not
   inherit this.
2. **SMP iteration skipping** — `SMP_SKIP_SIZE`, `SMP_SKIP_PHASE`,
   `helper_skips_iteration` (`search.rs:373–397`) behind
   `smp_iteration_skip == 1` (`1440`), plus its test (`5645`). **Owner:
   B.1**; D.2 decides the helper policy fresh.
3. **Continuation-index test helpers** — `move_ordering::cont_index` and
   `cont_row_base` exist only for one test
   (`#[cfg_attr(not(test), expect(dead_code))]`, `163`, `181`); the live
   code uses `NodeContext::cont_row_base`. **Owner: B.1**, move under
   `#[cfg(test)]` or fold into the test.
4. **Lazy-eval dual diagnostic** — `Evaluator::diag_lazy_dual`
   (`eval.rs:2250–2328`) and the 21 `lazy_*` counters re-evaluate every
   lazily-skipped position under `diag` to measure the lazy margin's cost.
   It is an instrument, not dead, but it is owned by the lazy path that
   `lazy_margin` (C.1) controls. **Owner: C.1**: keep if C.1 keeps a lazy
   path, else delete with it.
5. **Tune-only and texel-only surfaces** — `EvalParams::load_from_str` /
   `load_from_env` / `dump` (`tune`), the trace macros and
   `linear_delta_scale` (`texel`), `Evaluator::params/set_params/last_trace`.
   Live instruments, kept (A.2.3). C.1 moves them to `eval/trace.rs` and an
   `eval/params.rs` without changing their meaning.
6. **Lint suppressions** — 28 sites, all `#[expect]` except the one
   documented `#[allow(unused_mut)]` in `search_options.rs:122` and the
   feature-conditional `cfg_attr(..., allow(dead_code))` on `RootConfidence`
   (`search.rs:231`, goes with item 1) and the two eval cache entries
   (`eval.rs:1085`, `1095`, texel bypasses the caches). Nothing to act on.

The 2026-08-19 code audit (`analysis/code_audit_2026_08_19.md`) is now
disposed: item 1 (`stack[ply].reduction` stale) is **resolved** — neither the
field nor `lmr_prior_reduction_adj` exists at `7cffce5`; item 2
(`improving` after check) was **rejected** by RAR-S66 and is B.2's to
revisit inside the donor's improving rule; item 3 (killer travel) is
subsumed by B.2's picker; item 4 (`attacks_from_sq` guarded by
construction) becomes a `debug_assert!` in C.1's attack-map producer.

## Target layout, confirmed with three additions

PLAN's layout stands. A.6 adds three lines the inventory shows are needed;
the investigations may still adjust.

```
src/eval/params.rs         eval_params! (137 entries) and the tune/texel I/O
src/eval/attacks.rs        the single attack-map and mobility-area producer
src/eval/endgame/kpk.rs    the KPK bitbase, today src/kpk.rs
```

`eval/params.rs` mirrors `search/params.rs`; the macro and its 137 entries
are 370 lines that belong to no family. `eval/attacks.rs` names the producer
C.1 is told to create; today it is 60 lines inside `eval_piece_activity`
(`eval.rs:1639–1700`) writing `attacked_by[2][6]`, `attacks_from_sq` and the
`KsMaps` view that king safety reads.

## B.1 handoff — search restructure, behaviour-neutral

Scope is fixed by PLAN B.1; this section says where today's code goes.
B.0 may re-cut module boundaries; it may not change any mechanism here.

| Target module | From `search.rs` (line spans at `7cffce5`) and elsewhere |
|---|---|
| `search/mod.rs` | `SearchEvent`/`SearchExit`/`SearchResult` (100–132), `RootMove` (132–229), `Searcher` (634–785), `configure`/`new_game`/`clear_history`/`search`/`search_impl`/`reset_search_state` (956–1235), `search_root` (1391–1856, 465 lines: iterative deepening, aspiration, TM decisions), `syzygy_root_moves` and the syzygy helpers (1269–1335, 4753–4804), `check_stop`/`record_node`/info output (5017–5179), `format_score` |
| `search/node.rs` | `negamax` (2201–3885, 1,684 lines), `quiescence` (3885–4169), `nmp_material_ok`, `lmr_reduction_units`, `ReductionInputs` (554–578), `late_move_prune_count`, `move_gives_check`, `build_lmr_table`/`lmr_reduction` (76–100) |
| `search/stack.rs` | `NodeContext` → `StackEntry` (578–634), `push_move`/`clear_move`/`continuation_index`/`continuation_at` (4953–5006), `PlyArray` for the four `[T; MAX_PLY]` fields (`pv_table`, `pv_len`, `stack`, `killers`) |
| `search/movepick.rs` | `Stage`, `MovePicker` and impl (468–554, 785–956), `score_moves`…`score_tactical_move` (4169–4342), `move_ordering.rs` lists and `pick_next`/`diversify_root_scores` |
| `search/history.rs` | the seven history tables now fields of `Searcher`, `quiet_history_ctx`/`quiet_history_score`/bonus/malus/`update_*`/`age_history` (4342–4624), `move_ordering.rs` index math, `boxed_cont_tables` |
| `search/correction.rs` | `corrected_eval*`/`correction_value`/`attributed_residual`/`update_correction` (4697–4753, 4804–4953) and the four correction tables |
| `search/params.rs` | `params.rs` minus the 42 inert entries |
| `search/threads.rs` | `search_threads.rs` minus instability; `search_worker`/`search_parallel` (1856–2071), `select_parallel_result` and the vote helpers (5208–5278), `apply_shared_root_scores`/`record_root_move_search` (4624–4697), `next_jitter` |
| `search/time.rs` | `time_manager.rs`; `effort_term`/`tm_interpolate`/`tm_effort_factor`/`tm_instability_factor` (397–436) |
| deleted | dead items 1–3 above; `ablated` stays until B.9 |

Types B.1 introduces: `NodeType` constants (PV / Cut / All, replacing the
`is_pv`/`cut_node` booleans threaded through `negamax`), `StackEntry`,
`PlyArray<T>`, and a shared-context type if B.0 asks for one. The 1,031
lines of tests move with the code they test; the root-confidence and
iteration-skip tests are deleted with their subjects.

Invariants and checks (PLAN B.1 done criteria, restated): exact
`bench 13` **7,601,220 / EBF 2.474** on magic and PEXT; debug and release
suites; `cargo fmt --check` and `clippy --all-features --all-targets` at
zero; pooled-PGO NPS within ±0.5% of A.8.4. A module split inside one crate
under fat LTO should be NPS-neutral, but the instrument decides, not the
expectation. Interactions to watch: removing parameters changes the `tune`
UCI option list and every `tools/spsa_configs/*.json` (tooling commit,
A.2.3); removing the instability slots changes `SharedSearchState`'s
constructor; re-keying counters changes `tools/diag/bench_counters.py` and
`phase4_differential.py` inputs, so the differential must be re-run against
the oracle's names once and archived. RAR-S65–S69 are recorded superseded
in the same documentation commit.

## C.1 handoff — evaluation restructure, behaviour-neutral

| Target module | From `eval.rs` (line spans at `7cffce5`) |
|---|---|
| `eval/mod.rs` | score constants (13–33), `Evaluator` with both caches (1060–1227), `evaluate` (1227–1379: cache, phase, lazy gate, tempo, rule-50 damping), `evaluate_result`, `set_lazy_margin`, `piece_value`, `color_sign` and the small helpers (3026–3054) |
| `eval/params.rs` | `eval_params!` and its 137 entries (135–505), `EvalTables`/`build_tables` (1060–1085), the `tune` load/dump (533–619) |
| `eval/trace.rs` | `tr_mg!`/`tr_eg!` (505–533), `EvalTrace` methods generated by the macro, `linear_delta_scale`, the texel tests (619–761) |
| `eval/attacks.rs` | the substrate now inside `eval_piece_activity` (1639–1700): `attacked_by`, `attacks_from_sq`, `KsMaps` (1106–1121), the mobility area; one producer, read by pieces, king, threats and space |
| `eval/material.rs` | `MG_VAL`/`EG_VAL`/`PHASE_W`, PSTs and `build_default_pst` (30–135), `eval_imbalance` (2883–2940) |
| `eval/pawns.rs` | `eval_pawns` and the pawn cache entry (1379–1564), file/rank/passed-mask tables (763–1060) |
| `eval/passers.rs` | `eval_passed_pawn_advance` (1564–1622), `eval_rooks_behind_passers`, `eval_passer_blockade`, `eval_passed_pawn_king_proximity` (2713–2883) |
| `eval/pieces.rs` | the remainder of `eval_piece_activity` (mobility, outposts, rook files, trapped/long-diagonal terms), `eval_xray_trio` (2153–2250), `eval_trapped_bishops`, `eval_closedness` (2940–3007) |
| `eval/king.rs` | `eval_king_safety` (2496–2713), `eval_king_centrality_danger` |
| `eval/threats.rs` | the threat block of `eval_piece_activity` (1929–2010 region) and `eval_hanging_pieces` (2796–2847) |
| `eval/space.rs`, `eval/initiative.rs` | `eval_space` (2448–2496), `eval_initiative` (3007–3026) |
| `eval/endgame/` | `apply_mop_up` (2328–2448), `scale_endgame` and every `*_scale`/recogniser (3054–3621), `kpk.rs`, the endgame tests (3621–3756) |
| `diag_lazy_dual` | with the lazy path, per dead-code item 4 |

`eval_piece_activity` (1622–2153, 531 lines) is the one function C.1 must cut
rather than move: it holds the attack substrate, mobility, rook/bishop/knight
terms, threats and the king-safety inputs in one pass over the board. The
cut is legal only if the running `mg`/`eg` order is preserved exactly, because
the mop-up reads `(mg + eg) / 2` mid-evaluation (comment at 1304–1309) and
the lazy gate reads the partial sum (1321). `EvalTrace` keeps its meaning
per PLAN; `tests/eval_cache.rs` and `tests/eval_invariants.rs` are the
executable contract. Checks: exact fingerprint, suites, pooled NPS within
±0.5%, and the texel trace test `trace_reconstructs_eval_exactly_over_random_playouts`
under `--features texel` (never measured for speed).

## What B.0 and C.0 still own

Not decided here: the `evidence.rs` consumer question, the counter re-keying
map, the `NodeType` set, whether `Searcher` stays one struct or becomes
per-thread state plus shared context, the cluster contents, and every
eval-family boundary C.0 draws differently from today's functions. This
document fixes only where today's code goes if nothing else changes.
