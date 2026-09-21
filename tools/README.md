# Tools

One line per tool: what it does, and where the method that uses it lives.
Run everything from the repository root. Measurement rules (rebuild with the
exact features, sum per-position counters, check exit status directly) are in
`AGENTS.md`; procedures are in `PROCESS.md`. Raw outputs go to ignored
`tools/results/`. Tool and fixture names are cited by ledger rows, so they do
not change.

## Games and harness

Colosseum CLI is the main path; fastchess and weather-factory stay installed and
working as the backup and the second opinion until at least release 2.5.0.
PROCESS's *Harness* section says when to run the backup as a cross-check.

### Main path — Colosseum CLI

| Tool | Purpose | Used by |
|---|---|---|
| `colosseum.ps1` | Gate, fixed match, null pair, tune or gauntlet on Colosseum CLI from the committed run files, with every provenance, equality, revision, fingerprint, policy and idle-host guard, a per-run manifest and a post-run fault check | PROCESS "Harness", "Common commands"; every run from B.2.6 on |
| `colosseum/` | The run files that hold Rarog's conditions, and `colosseum.pin.json`, which pins the runner by revision and SHA-256 | `colosseum.ps1`, `setup_tools.ps1`; `colosseum/README.md` |
| `spsa_config_to_colosseum.py` | Convert a registered surface to a Colosseum tune file for one horizon; `--check` refuses a file that has drifted from its JSON | `colosseum.ps1 -Mode spsa`; PROCESS "SPSA go/no-go procedure" |
| `diag/colosseum_parity.py` | Compare a Colosseum dry run with a recorded `sprt.ps1` manifest, field by field | RAR-M60; harness cross-checks |
| `diag/colosseum_recount.py` | Recount W-D-L, pentanomial, Elo and nElo from a run's PGN and check them against its record | RAR-M60, RAR-M61; re-reading a finished run |
| `diag/test_colosseum_guards.ps1` | Break one input per case and require the refusal that names it | the guard contract; run after touching a guard |
| `diag/test_colosseum_parity.py` | The recorded parity pair as a fixture, plus one mutation per compared field | the parity contract |

### Backup path — fastchess and weather-factory

| Tool | Purpose | Used by |
|---|---|---|
| `sprt.ps1` | Pentanomial GSPRT between two Rarog binaries via fastchess, with provenance, compiler-equality and dirty-tree guards; `-Mode calibrate` for null pairs | PROCESS "Harness"; every SPRT row up to B.2.5 |
| `spsa.ps1` | Set up and run a weather-factory SPSA tune; a surface naming `Core*` options requires a `b2core-tune` binary | PROCESS "SPSA go/no-go procedure"; `spsa_configs/README.md` |
| `pgn_result.ps1` | Recompute Elo, LOS and pentanomial counts from a fastchess PGN | re-reading a finished match |
| `watch.ps1` | Console-noise filter for long fastchess and weather-factory runs | operator convenience |
| `gauntlet.ps1` | The frozen Rarog 2.2.0 external gauntlet (hard-coded field) | historical; `colosseum.ps1 -Mode gauntlet` replaces it |

### Shared

| Tool | Purpose | Used by |
|---|---|---|
| `build_test.ps1` | Build a PGO or tune test binary, of the default arm or a feature arm (`-Features b2core`), with a bench-verified provenance sidecar into `tools/test_engines` | PROCESS "Common commands"; every gate and tune |
| `harness_common.ps1` | One implementation of every guard both paths enforce — idle host, runner pin, sidecar provenance, flavour and compiler equality, advertised options, tune surface — plus the affinity list and adjudication profiles | dot-sourced by `colosseum.ps1`, `sprt.ps1`, `spsa.ps1`, `datagen.ps1`, `build_test.ps1` and others |
| `setup_tools.ps1` | Stage the pinned Colosseum CLI, fastchess, the UHO book and the patched weather-factory | PROCESS "Toolchain and harness notes" |
| `pgn_depth_at_nodes.py` | Per-engine reported depth and time per move from a fixed-nodes PGN | tree-shape comparisons at equal nodes |

## Speed

| Tool | Purpose | Used by |
|---|---|---|
| `nps_build_pool.ps1` | Build N independent PGO binaries of one tier (or of a feature arm with `-Features`) at a clean head, fingerprint-checked, with a hash manifest | PLAN performance qualification; RAR-P24 |
| `nps_multibuild.ps1` | Interleaved pooled-PGO NPS A/B with a bootstrap CI | PLAN performance qualification; RAR-P24 |
| `nps_scaling.ps1` | Thread-scaling NPS on two pinned middlegames (uses `uci_probe.ps1`) | SMP scaling rows |
| `uci_probe.ps1` | Minimal UCI driver shared by the scaling scripts | `nps_scaling.ps1`, `diag_smp_sweep.ps1` |
| `profile_etw.ps1` | ETW sampling profile of a bench run, with a staleness guard (elevated shell) | profiling |
| `branching_profile.ps1` | Nodes-to-depth, iteration cost and branching on a fixed corpus | PLAN B.0 section 11 baselines |

## Search diagnostics (`--features diag` builds)

| Tool | Purpose | Used by |
|---|---|---|
| `diag/bench_counters.py` | Sum diagnostic counters over a whole `bench` run | AGENTS "Measurement"; neutrality checks |
| `diag/phase4_differential.py` | Counter-by-counter differential against the oracle build over `diag/phase4_suite_v1.epd` | AGENTS "Measurement"; PROCESS "Matched ablation" |
| `diag_search_quality.ps1` | Interaction map, first-move cutoff rate and LMR readouts over `bench` | search-quality rows |
| `diag_smp_sweep.ps1` | TT-hit and aspiration breakdown across thread counts | SMP diagnostics |
| `diag/fixed_budget_probe.py` | Fixed-node or fixed-depth probes over an EPD suite for several engines | PLAN B.0 section 11; B.2 screens |
| `diag/answer_compare.py` | Compare best moves and scores of Rarog and the oracle at fixed depth | answer-quality diagnostics |
| `diag/answer_nodes.py` | The same comparison at a fixed node budget | answer-quality diagnostics |
| `diag/nodes_per_move.py` | Actual nodes per move at a real time control | calibrating node budgets |
| `diag/feature_matrix.py` | `cargo check` every feature subset | CI "Feature matrix" |
| `diag/check_guide.py` | GUIDE/PLAN status-board and fingerprint consistency | AGENTS "Documents" |

## Board and SEE

| Tool | Purpose | Used by |
|---|---|---|
| `diag/board_v2_oracle.py` | Generate and verify the independent legal-move oracle | board correctness fixtures |
| `diag/board_v2_run.py` | Run the isolated board benchmark with reproducibility data | board speed rows |
| `diag/board_search_profile.py` | Board work on frozen full-search cohorts (`diag/board_search_profile_v1.epd`) | board profiling |
| `diag/board_search_profile_etw.ps1` | One symbolized ETW report per board-search cohort | board profiling |
| `diag/summarize_board_search_etw.py` | Recover and summarize those ETW traces | board profiling |
| `diag/see_contract_oracle.py` | Independent same-square exchange oracle for the SEE contract | SEE contract tests |
| `diag/normalized_see_compare.py` | Normalized cross-engine SEE comparison | SEE comparison rows |
| `diag/verify_normalized_see.py` | Verify a normalized SEE bundle without rerunning it | SEE comparison rows |

## Endgames and conversion

| Tool | Purpose | Used by |
|---|---|---|
| `diag/endgame_truth.py` | Syzygy-truth endgame corpus and conversion baseline | endgame rows; `diag/endgame_truth_baseline_v2.json` |
| `diag/endgame_conversion.py` | Deterministic bare-king conversion | endgame rows |
| `diag/endgame_drawn.py` | Drawn-subset census: overclaims in theoretically drawn endings | endgame rows |
| `diag/endgame_floors.py` | Aggregate conversion floors with cohort and schema guards | endgame acceptance |
| `diag/endgame_budget_bracket.py` | Repeat a family verdict across node budgets | endgame rows |
| `diag/endgame_occurrence.py` | Reference-family occurrence in the search tree, by root | endgame ranking input |
| `diag/endgame_board_occurrence.py` | Reference-family occurrence on the board in real games | endgame ranking input |
| `diag/endgame_ranking.py` | Rank the twenty reference functions on the measured layers | endgame ordering |
| `diag/endgame_reference_results.py` | Freeze the attained reference result per family | `diag/endgame_reference_results_v1.json` |
| `diag/endgame_book.py` | Endgame-start opening book with Syzygy-verified verdicts | endgame matches |
| `diag/conversion_audit.py` | Games thrown away after a persistent material advantage | PLAN conversion measurements |
| `diag/export_tournament_pgn.py` | Export one Colosseum tournament to a PGN archive | conversion and pool rows |

## Tuning and data

| Tool | Purpose | Used by |
|---|---|---|
| `audit_spsa_coverage.ps1` | Check a tune surface against `src/search/params.rs` | PROCESS "SPSA go/no-go procedure" |
| `spsa_convergence_model.py` | Compare SPSA horizons under the live schedule | SPSA registration |
| `datagen.ps1` | Deterministic self-play PGN segments for Texel data | PROCESS "Texel convergence procedure" |
| `texel-tuner/` | The Rust Texel tuner, its own Cargo workspace | `texel/README.md` |
| `texel/extract.py`, `texel/extract_parallel.py` | PGN to `FEN;target` datasets | `texel/README.md` |
| `texel/sample_fens.py`, `texel/build_book.py` | Datagen start books | `texel/README.md` |
| `texel/relabel_tb.py` | Replace <=6-man labels with Syzygy truth | RAR-E08 |
| `texel/bake_params.py` | Bake a tuner parameter dump into `src/eval.rs` | `texel/README.md` |
| `texel/fit_complete.ps1`, `texel/confirm_hce_fit.ps1` | Complete HCE fit and its confirmation corpus (pin the accepted fingerprint before use) | PROCESS "Texel convergence procedure" |
| `diag/book_yield.py` | Texel row yield per game by start phase | corpus design |
| `diag/datagen_label_audit.py` | Datagen results against tablebase truth | label-quality decisions |
