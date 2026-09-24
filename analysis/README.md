# Analysis and local evidence

Development and experiment evidence are kept on the maintainer's primary
machine. Other machines run macOS and Windows-on-ARM compatibility checks.

Git retains the Markdown analyses, decisions, experiment recipes and result
summaries needed to develop Rarog. The following are local-only and ignored:

- `analysis/artifacts/`: raw results, manifests, frozen experimental adapters,
  candidate patches/vectors, validation logs and compressed evidence bundles.
- `analysis/*.txt`: standalone generated measurement reports.
- `tools/results/` and the existing ignored build/data directories: run outputs,
  executables, profiling traces and other generated material.
- `hybrid/`: the frozen oracle package the ledger's oracle rows ran (its
  executable and evaluation DLL, identified by hash in PLAN A.2.2's record).

Paths to those files in older analyses deliberately refer to local evidence;
they will not be populated by a fresh clone. Preserve them on this machine.
Keep the recipe, relevant source/binary identity and result in the tracked
analysis or EXPERIMENTS ledger. Do not force-add a raw bundle to make its link
work on another machine. Arrange local evidence backups separately from Git.

Required fixtures under `tests/data/`, frozen ranking/floor inputs consumed by
diagnostic tools, reusable scripts and vendor source/licenses remain tracked. A
generated origin alone does not make a required test input disposable.

**Every file in `logo/` stays tracked, used by the README or not.** The logos
are for users, not for code, so "unreferenced" is not a reason to remove one.
Never untrack, ignore or delete a logo file (maintainer decision, restated
2026-09-14 after B.2.0.1 untracked six variants and they were restored).

The storage cleanup removes files from the current Git index only. It preserves
their bytes and paths on disk and leaves earlier commits unchanged; old blobs
will therefore still contribute to repository history size.

## Index

Every tracked analysis, by class. **Contract**: a live invariant PLAN's
standing-contracts table points at. **Deliverable**: a current-roadmap decision
or design record, owned by the named leaf. **Record**: the evidence behind the
ledger rows named; frozen. **Archived**: superseded or historical, kept in
`analysis/archive/` with its banner. A new analysis gets a row here in the
commit that adds it.

| File | Title | Class | Owner or citing rows |
|---|---|---|---|
| [`board_comparison_411b19_2026-09-09.md`](board_comparison_411b19_2026-09-09.md) | Board comparison after 4.11b.19 — RAR-M44(d) | Contract | Cross-engine board benchmark parity |
| [`draw_policy_2026-09-08.md`](draw_policy_2026-09-08.md) | Draw-state policy boundary — RAR-M40 / 4.11b.15 | Contract | Draw, null, repetition and rule-50 policy |
| [`endgame_measurement_layers.md`](endgame_measurement_layers.md) | The four measurement layers (PLAN 4.10.5) | Contract | Endgame measurement layers |
| [`hce_archive_audit_2026-08-31.md`](hce_archive_audit_2026-08-31.md) | HCE self-play archive audit — 2026-08-31 | Contract | Texel instrument coverage |
| [`history_contracts_2026-09-08.md`](history_contracts_2026-09-08.md) | History capacity and mutation contracts — RAR-M38 / 4.11b.13 | Contract | History capacity and canonical moves |
| [`movelist_delivery_2026-09-09.md`](movelist_delivery_2026-09-09.md) | Move-list delivery probe — RAR-M44 | Contract | Caller-owned move-list delivery |
| [`phase4_counter_spec.md`](phase4_counter_spec.md) | Phase-4 differential counter specification | Contract | Diagnostic counter units and sampling |
| [`see_contract_2026-09-06.md`](see_contract_2026-09-06.md) | SEE contracts — RAR-M27 / 4.11b.4 | Contract | Board legality and SEE king legality |
| [`see_repair_2026-09-06.md`](see_repair_2026-09-06.md) | SEE exchange repair — RAR-M28 / 4.11b.5 | Contract | SEE created pins and promotions |
| [`texel_fitting_handbook.md`](texel_fitting_handbook.md) | Texel fitting in Rarog — the handbook | Contract | Texel data contract and fitting |
| [`ablation_design.md`](ablation_design.md) | Ablation harness: design, and why it is not a bisection over the oracle | Deliverable | B.9 |
| [`ablation_results.md`](ablation_results.md) | Paired ablation, results | Deliverable | B.9 |
| [`architecture_review_2026-09.md`](architecture_review_2026-09.md) | Architecture and design review — PLAN B.2.0 | Deliverable | B.2.0, E.1 |
| [`b21_review_2026-09-14.md`](b21_review_2026-09-14.md) | B.2.1 review — the `b2core` selectivity core against the B.0 handoff | Deliverable | B.2.1, B.2.2, B.3, B.7 |
| [`b22_screens_2026-09-15.md`](b22_screens_2026-09-15.md) | B.2.2 screens — the unfitted `b2core` candidate against B.0's registered numbers | Deliverable | B.2.2, B.2 re-plan, B.7 |
| [`b223_sweep_2026-09-15.md`](b223_sweep_2026-09-15.md) | B.2.2.3 — curvature sweep and the P6 profile on the `b2core` arm | Deliverable | B.2.2.3, B.2.3 |
| [`b22_review_2026-09-15.md`](b22_review_2026-09-15.md) | B.2.2 review — what the +52 Elo and the 0.683x NPS mean, and what precedes B.2.3 | Deliverable | B.2.2, B.2.3, B.7, B.3–B.5 screens |
| [`b2_audit_2026-09-21.md`](b2_audit_2026-09-21.md) | B.2 close-out audit (theta, fingerprints, both SPRTs recounted) and Colosseum CLI readiness for B.2.6 | Record | B.2.3, B.2.4, B.2.6, B.2.7, B.2.8, RAR-M59 |
| [`b3_research_2026-09-23.md`](b3_research_2026-09-23.md) | B.3 research on the fitted head: activation, donor re-read, interaction map, implementation contract, registered screens and predictions (RAR-S79) | Deliverable | B.3, B.3.1–B.3.4 |
| [`b32_screens_2026-09-23.md`](b32_screens_2026-09-23.md) | B.3.2 screens — the `b3proof` arm against the fitted head before any game: ladder, cost screen, time-to-depth, differential, bit sweep, categoricals, the paired-run handover | Deliverable | B.3.2, B.3.3, B.3.4, B.7 |
| [`b33_sweep_2026-09-24.md`](b33_sweep_2026-09-24.md) | B.3.3 — the `SingularTtDepthMargin=2` bake's proof and the zero-game curvature sweep of seven proof coordinates under a rule frozen first; why the SPSA surface is held | Deliverable | B.3.3, B.3.4, B.6 |
| [`consolidation_2026-09-10.md`](consolidation_2026-09-10.md) | Codebase consolidation analysis — PLAN A.6 | Deliverable | A.6 |
| [`endgame_occurrence_tournament_2026-09-05.md`](endgame_occurrence_tournament_2026-09-05.md) | Endgame occurrence over 36,400 rated games | Deliverable | PLAN section 1, C.5 |
| [`feature_inventory_2026-09-09.md`](feature_inventory_2026-09-09.md) | Feature, option and parameter inventory — PLAN A.2.3 | Deliverable | A.2.3 |
| [`ledger_records_2026-09-14.md`](ledger_records_2026-09-14.md) | Ledger records moved out of EXPERIMENTS.md, 2026-09-14 | Deliverable | `EXPERIMENTS.md` |
| [`repository_review_2026-09.md`](repository_review_2026-09.md) | Repository and document review — PLAN B.2.0.1 | Deliverable | B.2.0.1, E.1 |
| [`search_programme_2026-09-13.md`](search_programme_2026-09-13.md) | Search programme investigation — PLAN B.0 | Deliverable | B.0, B.1–B.3 |
| [`time_forfeit_2026-09-09.md`](time_forfeit_2026-09-09.md) | Time forfeits at `3+0.03` — diagnosis and repair (PLAN A.3.3, RAR-R11) | Deliverable | A.3.3 |
| [`uci_info_review_2026-09-16.md`](uci_info_review_2026-09-16.md) | UCI `info` line conformance against Stockfish and Reckless — PLAN B.2.5, D.3 | Deliverable | B.2.5, D.3 |
| [`universal_binary_2026-09.md`](universal_binary_2026-09.md) | Universal x86-64 binary — design record and deferral | Deliverable | A.4, G.2 |
| [`answer_harness_rset_correction.md`](answer_harness_rset_correction.md) | Correction: every `--rset` screen before this commit measured the defaults | Record | RAR-S70 |
| [`board_audit_2026-09-05.md`](board_audit_2026-09-05.md) | Rarog board audit and measured comparison — 2026-09-05 | Record | RAR-M20 |
| [`board_benchmark_recipe_2026-09-05.md`](board_benchmark_recipe_2026-09-05.md) | Reproducing the 2026-09-05 board comparison | Record | RAR-M20, RAR-M43 |
| [`board_search_profile_2026-09-07.md`](board_search_profile_2026-09-07.md) | Full-search board profile — RAR-M30 / 4.11b.7 | Record | RAR-M30 |
| [`board_search_profile_2026-09-08.md`](board_search_profile_2026-09-08.md) | Refreshed full-search board profile — RAR-M36 / recipe recovery | Record | RAR-M36 |
| [`board_v2_instrument_2026-09-06.md`](board_v2_instrument_2026-09-06.md) | Board-v2 instrument and correctness corpus | Record | cited by PLAN or another analysis |
| [`cluster_qualification_2026-09-08.md`](cluster_qualification_2026-09-08.md) | Integrated board cluster qualification — RAR-M41 / 4.11b.16 | Record | RAR-M41 |
| [`code_audit_2026_08_19.md`](code_audit_2026_08_19.md) | Code audit, 2026-08-19 | Record | RAR-S64, RAR-S65, RAR-S66 |
| [`conversion_claims_correction_2026-09-06.md`](conversion_claims_correction_2026-09-06.md) | 4.11.10 conversion-claim correction -- RAR-M24 | Record | RAR-M24 |
| [`datagen_label_audit_2026-09-06.md`](datagen_label_audit_2026-09-06.md) | 4.11.8 datagen label audit — RAR-M22 | Record | RAR-M22 |
| [`drawn_share_census_2026-09-05.md`](drawn_share_census_2026-09-05.md) | Drawn-share bias census (PLAN 4.11.4) | Record | cited by PLAN or another analysis |
| [`endgame_budget_transfer_2026-09-05.md`](endgame_budget_transfer_2026-09-05.md) | 4.11.7 budget transfer — RAR-M21 | Record | RAR-M21 |
| [`endgame_conversion_audit_2026-09-01.md`](endgame_conversion_audit_2026-09-01.md) | Rarog endgame conversion and recogniser audit — 2026-09-01 | Record | RAR-E06, RAR-E10 |
| [`endgame_occurrence_split_2026-09-05.md`](endgame_occurrence_split_2026-09-05.md) | Occurrence split by root, and why one threshold is a choice (PLAN 4.11.5) | Record | cited by PLAN or another analysis |
| [`endgame_refresh_2026-09-09.md`](endgame_refresh_2026-09-09.md) | Endgame evidence refresh after the accepted board head — RAR-M42 / 4.11b.18 | Record | RAR-M42 |
| [`endgame_search_occurrence_2026-09-03.md`](endgame_search_occurrence_2026-09-03.md) | Search-tree occurrence of the 20 reference endgame families — 2026-09-03 | Record | cited by PLAN or another analysis |
| [`endgame_truth_instrument_audit_2026-09-04.md`](endgame_truth_instrument_audit_2026-09-04.md) | Audit: the endgame-truth instrument, 2026-09-04 | Record | RAR-E14 |
| [`endgame_truth_v2_baseline_2026-09-04.md`](endgame_truth_v2_baseline_2026-09-04.md) | The corrected endgame truth baseline (PLAN 4.11.1) | Record | cited by PLAN or another analysis |
| [`hce_maturity_2026-08-25.md`](hce_maturity_2026-08-25.md) | Rarog HCE maturity against the classical Stockfish reference | Record | cited by PLAN or another analysis |
| [`hce_residuals_2026-09-01.md`](hce_residuals_2026-09-01.md) | Post-fit residual audit of the accepted HCE — 2026-09-01 (PLAN 4.9.1) | Record | RAR-E09 |
| [`king_square_cache_2026-09-08.md`](king_square_cache_2026-09-08.md) | King-square caching — RAR-M37 / 4.11b.12 | Record | RAR-M37 |
| [`mate_drive_promotion_closure_2026-09-06.md`](mate_drive_promotion_closure_2026-09-06.md) | 4.11.9 mate-drive promotion closure -- RAR-M23 | Record | RAR-M23 |
| [`movegen_2026-09-07.md`](movegen_2026-09-07.md) | Move generation optimization — RAR-M31 / 4.11b.8 | Record | RAR-M31 |
| [`node_budget_2026-09-04.md`](node_budget_2026-09-04.md) | What a move actually costs at 3+0.03 (PLAN 4.10.6) | Record | cited by PLAN or another analysis |
| [`phase4_mechanism_map.md`](phase4_mechanism_map.md) | Phase-4 mechanism map and order freeze | Record | RAR-S56 |
| [`pin_check_sharing_2026-09-08.md`](pin_check_sharing_2026-09-08.md) | Shared pin/check information — RAR-M34 / 4.11b.10 | Record | RAR-M34 |
| [`playing_gate_2026-09-08.md`](playing_gate_2026-09-08.md) | Integrated board cluster playing gate — RAR-E15 / 4.11b.17 | Record | RAR-E15 |
| [`relocation_2026-09-07.md`](relocation_2026-09-07.md) | Fused ordinary relocation — 4.11b.9 | Record | RAR-M32, RAR-M33 |
| [`representation_2026-09-08.md`](representation_2026-09-08.md) | Larger board representation change — RAR-M39 / 4.11b.14 | Record | RAR-M39 |
| [`see_kernel_2026-09-08.md`](see_kernel_2026-09-08.md) | Incremental SEE attacker maintenance — RAR-M35 / 4.11b.11 | Record | RAR-M35 |
| [`see_value_injection_2026-09-07.md`](see_value_injection_2026-09-07.md) | Neutral SEE values and normalized comparison — RAR-M29 / 4.11b.6 | Record | cited by PLAN or another analysis |
| [`texel_corpus_book_shape_2026-09-02.md`](texel_corpus_book_shape_2026-09-02.md) | The datagen book was the wrong shape, not the wrong size — 2026-09-02 | Record | cited by PLAN or another analysis |
| [`hce_analysis.md`](archive/hce_analysis.md) | Rarog HCE analysis | Archived | RAR-E03 |
| [`infra_analysis.md`](archive/infra_analysis.md) | Rarog Board and Infrastructure Analysis | Archived | RAR-C01 |
| [`search_analysis.md`](archive/search_analysis.md) | Rarog search analysis | Archived | RAR-C02, RAR-S11 |
| [`engine-choice-audit-2026-09-07.md`](archive/engine-choice-audit-2026-09-07.md) | Basilisk versus Rarog: development choice | Archived | uncited |
| [`board_comparison_2026-09-09.md`](archive/board_comparison_2026-09-09.md) | Board comparison after 4.11b — RAR-M43 | Archived | RAR-M43 |
| [`board_perft_compare.md`](archive/board_perft_compare.md) | Board-implementation speed: Rarog vs Basilisk vs Reckless vs Stockfish | Archived | RAR-P04 |
| [`speed_profile_8_12c.md`](archive/speed_profile_8_12c.md) | 8.12(c) Profile pass — where Rarog's search time actually goes | Archived | RAR-P05 |
| [`smp_analysis.md`](archive/smp_analysis.md) | SMP / threading analysis — 2026-07-22 | Archived | RAR-R06 |
| [`phase4_10_obligations.md`](archive/phase4_10_obligations.md) | 4.10 obligations — everything deferred there, with its evidence | Archived | RAR-S67 |
| [`phase4_6_audit.md`](archive/phase4_6_audit.md) | 4.6 Cluster B — audit, 2026-08-20 | Archived | uncited |
| [`manta_tooling_audit_2026-08-25.md`](archive/manta_tooling_audit_2026-08-25.md) | Manta tooling and measurement audit | Archived | uncited |
| [`basilisk_audit_2026-08-30.md`](archive/basilisk_audit_2026-08-30.md) | Basilisk method and results audit | Archived | RAR-E08, RAR-M18 |
| [`answer_harness_calibration.md`](archive/answer_harness_calibration.md) | The answer harness cannot rank candidates. Calibrated, 2026-08-21. | Archived | uncited |
