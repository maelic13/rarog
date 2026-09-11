# Feature, option and parameter inventory — PLAN A.2.3

Classification of every Cargo feature, UCI option and `SearchParams` entry on
`dev` at `7d8b013`, from the source (every `.rs` under `src/`), not from
documents. Disposition rule from PLAN A.2.3: removals land in **B.1** (the
behaviour-neutral search restructure), never here; nothing in this document
changes engine behaviour.

Method: the 99 parameter declarations were extracted from `search_params!`
in `src/params.rs` and every consumer counted by name across the crate. All
99 have at least one consumer in `src/search.rs` (one also in `eval.rs`), so
"dead" below never means unreferenced; it means **inert at the shipped
default** (a guard that a zero default never opens, or an additive term that
adds zero) with no accepted evidence behind the non-default value.

## Cargo features

| Feature | Consumers | Disposition |
|---|---|---|
| `diag` | `diag_count!`/`diag_add!` at 150 sites in `search.rs`, 16 in `board.rs`, 12 in `movegen.rs`; `tools/diag/bench_counters.py`, `phase4_differential.py` | **Keep.** The oracle-differential instrument for B.2.2 and later clusters. B.1 must re-key the counters to the new mechanism names (B.0 owns the mapping). |
| `ablate` | `ablated()` at 11 sites in `search.rs`; `AblationMask` | **Keep until B.9**, then remove. The matched-ablation G(mask) instrument is meaningless once the mechanisms are replaced; B.9 re-measures G(0) only. |
| `tune` | exposes all 99 parameters as UCI spins | **Keep.** The SPSA wire for B.2.3, B.6, C.9, C.10. |
| `texel` | evaluator trace path, bypasses caches | **Keep.** Fitting instrument for every C-phase refit; never measured (AGENTS). |

## UCI options

Nine production options: `Hash`, `Clear Hash`, `Ponder`, `Move Overhead`,
`Threads`, `SyzygyPath`, `SyzygyProbeDepth`, `SyzygyProbeLimit`,
`Syzygy50MoveRule`. All consumed; none removed. D.3 audits their semantics.

## `SearchParams` — 99 entries

### Dead: inert at default, remove in B.1 (42 parameters, grouped below)

Each is a switch or an additive term whose default (0) leaves the code path
unreachable or the term zero. The mechanism the switch would have enabled is
either a rejected experiment, an unrun registration that B.2 supersedes, or a
provenance alternative without a named consumer. Removing them is
fingerprint-neutral by construction; B.1 proves it.

| Parameter | Consumer form | Provenance |
|---|---|---|
| `asp_center_avg_pct` | `> 0` guard | aspiration alternative, never accepted |
| `asp_magnitude_div` | `> 0` guard | same |
| `asp_fail_high_reduction` | multiplier, 0 | same |
| `asp_conf_wide_pct`, `asp_conf_narrow_pct`, `asp_conf_fail_pct` | only read under `root_conf_aspiration == 1` | RAR-S47 line, off |
| `quiet_see_prune_depth`, `quiet_see_prune_coeff` | `!= 0` guard | 4.6.1 quiet SEE prune, stopped null |
| `skip_quiets_on_move_count` | `!= 0` guard | 4.6.5, reverted with SearchCore |
| `selectivity_prospective_depth`, `selectivity_count_considered` | `!= 0` guards | 4.6.5, reverted |
| `nmp_suppress_null_in_verification`, `nmp_require_cut_node`, `nmp_use_static_eval`, `nmp_singular_guard` | `== 0`/`!= 0` guards | NMP provenance alternatives, no consumer named |
| `rfp_allow_tt_pv`, `razor_allow_tt_pv`, `nmp_allow_tt_pv`, `probcut_allow_tt_pv` | `!= 0` guards | tt-pv policy alternatives, off |
| `singular_reject_speculative`, `probcut_store_actual_score` | `!= 0` guards | 4.3c alternatives, off |
| `smp_iteration_skip` | `== 1` guard | SMP helper skipping; D.2 decides the SMP design from Reckless's shape, not from this switch |
| `root_conf_time`, `root_conf_aspiration`, `root_conf_pool_instability`, `root_conf_w_deviation`, `root_conf_w_effort`, `root_conf_w_window`, `root_conf_dev_scale` | `== 1` guards and their weights | RAR-S47 root confidence, shipped off; D.1 rebuilds TM from the donor shape |
| `lmr_relief`, `lmr_jitter_1t`, `lmr_min_reduced_depth`, `lmr_tt_capture`, `lmr_singular_relief`, `lmr_parent_movecount_relief`, `lmr_parent_movecount_min`, `lmr_stat_swing`, `lmr_stat_swing_margin` | additive terms of 0 or `!= 0` guards | RAR-S67/S68 registered-not-run, RAR-S65/S66 audit findings; B.2 replaces the formula |
| `tt_cutoff_bonus_pct` | `!= 0` guard | history alternative, off; B.2's TT-cutoff history bonus is the donor's form |
| `corr_skip_when_tt_refined` | `!= 0` guard | correction alternative, off |
| `corr_w_cont2`, `corr_w_cont4` | weights of 0 | continuation-correction alternatives, off; B.2 adopts the donor's plies 2 and 4 with seeds |

Consequence for the ledger: RAR-S65, S66, S67, S68 and S69 are
**REGISTERED, NOT YET RUN** experiments on mechanisms that B.2 replaces. B.1
records them as **superseded by B.2** with no retry trigger; their questions
(reduction magnitude, jitter, margin scale, killer travel, improving fallback)
are answered inside the new cluster's SPSA or not at all.

### Seed-for-B: live today, replaced by the B.2 cluster with donor-seeded successors (55)

Live coordinates of the current selectivity core. Their values are Rarog's
own seeds for the B.0 scale comparison; the B.2 handoff decides which survive
by name and which are replaced by the donor-shaped terms.

- Aspiration and root: `aspiration_delta`, `asp_growth_pct`,
  `asp_growth_high_pct`, `asp_growth_add`, `asp_max_fails`, `lmr_root_relief`
  (accepted, RAR-S70; B.5 decides whether the new formula subsumes it),
  `tm_fall_slope` (D.1).
- Pruning margins: `futility_base`, `futility_not_improving`,
  `razoring_coeff`, `nm_depth_coeff`, `nm_improving_bonus`,
  `nmp_min_non_pawn_pieces`, `lmp_base`, `lmp_not_improving`,
  `lmp_count_base`, `see_pruning_coeff`, `see_pruning_max`, `fp_base`,
  `fp_coeff`, `probcut_margin`, `probcut_see_gap_scale`,
  `probcut_move_cap_base`, `probcut_move_cap_cut_bonus`,
  `singular_beta_mult`, `singular_tt_depth_margin`, `singular_double_margin`.
- Quiescence: `qs_see_margin`, `qs_see_clamp_lo`, `qs_see_clamp_hi`,
  `qs_see_bad_floor`.
- LMR: `lmr_tt_pv_adj`, `lmr_exact_bound`, `lmr_shallow_tt`, `lmr_cut_node`,
  `lmr_table_base`, `lmr_table_div`, `lmr_hist_div`.
- Histories and correction: `check_bonus_safe`, `hist_bonus_mul`,
  `hist_bonus_sub`, `hist_malus_mul`, `hist_malus_sub`, `exact_bonus_pct`,
  `capture_malus_pct`, `surprise_bonus_pct`, `corr_capture_weight_pct`,
  `corr_rfp_scale`, `corr_fut_scale`, `corr_lmr_scale`, `corr_w_pawn`,
  `corr_w_minor`, `corr_w_own_np`, `corr_w_their_np`, `corr_w_cont`.

### Consumed and kept outside B (2)

| Parameter | Owner |
|---|---|
| `lazy_margin` | evaluator lazy path (`eval.rs`, 4 sites; `search.rs`, 4); C.1 owns it |
| `ablation_mask` | the `ablate` instrument; removed with the feature at B.9 |

Total: 42 dead + 55 seed-for-B + 2 kept outside B = 99, matching the
`search_params!` declaration count.

## Other retained surfaces

| Surface | Consumers | Disposition |
|---|---|---|
| Typed TT provenance (`src/evidence.rs`, 707 lines; 8 uses in `search.rs`, 1 in `tt.rs`; `tests/tt_provenance.rs`, `tests/engine_coverage.rs`) | search reads the producer field for diagnostics and the ProbCut store path | **B.0 decides.** The donor stores no producer field. Keep only if B.0 names a consumer that changes a search decision; otherwise B.1 removes it with its tests. |
| `tools/spsa_configs/*.json` (13 surfaces, current parameter names) | `audit_spsa_coverage.ps1`, `spsa.ps1` | **Keep until B.1**, which renames and removes parameters; B.2.3 registers a fresh surface and the old configs are deleted in the same tooling commit. |
| SMP probe scripts (`uci_probe.ps1`, `nps_scaling.ps1`, `diag_smp_sweep.ps1`) | none in the tree | Keep for D.2. |
| Answer harness and search-quality readouts (`answer_compare.py`, `answer_nodes.py`, `diag_search_quality.ps1`) | none in the tree | B.0 decides whether the B.2.2 diagnostics use them; otherwise B.8 deletes them. |

## Handoff to B.1

B.1 removes the 42 dead parameters and their guarded branches, re-keys the
diagnostic counters to the new module names, and proves the removal
fingerprint-neutral on magic and PEXT with the suites and pooled NPS. It
records RAR-S65 to S69 as superseded and deletes or regenerates the SPSA
configs in a separate tooling commit. Nothing here authorises changing a
default.
