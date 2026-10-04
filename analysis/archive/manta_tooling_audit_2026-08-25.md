# Manta tooling and measurement audit

Status: imported into the Rarog pre-NNUE plan

Audit date: 2026-08-25

Manta snapshot: clean `1a0f8ea1091031f0380ca2ea7d80a52f14c29118`

Rarog integration: tooling commit `d2c7788`; plan/guide commit containing this
record

Scope: every first-party executable/script/config under Manta `tools/` and its
CI wrappers was classified. Empty `bin`, `books`, `results`, `test_engines` and
Texel output placeholders contain no behavior and need no Rarog counterpart.

## Verdict

Manta supplied several useful measurement-hardening ideas, not an alternate
Rarog workflow. The reusable pieces are now native Rarog tools: authenticated
build assets, strict UCI-option discovery, anomaly-complete match manifests,
staged-but-immutable SPSA, a robust branching profiler, and leak-resistant
parallel Texel extraction. Rarog keeps its stronger two-sided adjudication,
pentanomial GSPRT, PGN reconstruction and Phase-4 counter/differential tools.

Manta's measurements reinforce the existing order: diagnose local search
authority before tuning; complete and semantically constrain HCE structure
before fitting; use static loss, signs and speed to refute rather than promote;
and require a clean game gate for every completed fitted cluster. They do not
show that a Manta formula or Manta's measured Elo is portable to Rarog.

## Tool disposition

### Adopted or upgraded in Rarog

| Manta source | Rarog disposition in `d2c7788` |
|---|---|
| `harness_common.ps1` | Added timeout/`uciok`-checked option discovery, option metadata, case/whitespace normalization, atomic JSON and shared match-anomaly checks |
| `build_test.ps1` | Upgraded to schema-v2 hash-bound assets: executable, tree, compiler, command, flavor, benchmark and size; fixed native PGO flavor/name collision |
| `sprt.ps1` | Validates both manifests and binaries, compiler/flavor equality, clean/tune status and live options; distinguishes fixed games from GSPRT; archives completion time plus PGN/log hashes; rejects every anomaly |
| `spsa.ps1` and configs | Added immutable registered horizon, separate staged `StopAfter`, games/iteration, live option range/default/lifetime checks and source/config/engine binding |
| `datagen.ps1` | Requires a clean hash-bound production-PGO asset and live options; records runner/tree/binary/output hashes, range and completion; refuses missing output/anomalies |
| `setup_tools.ps1` | Reuses local UHO source and normalizes pinned tuner sources before deterministic patching |
| `branching_profile.ps1`, `bench_positions.epd` | Ported as a generic Rarog UCI profiler over the versioned Phase-4 suite, with fixed Hash/Threads, fresh depth processes, per-position ratios and hash-complete report |
| `texel/extract.py`, `extract_parallel.py`, `audit_starts.py` | Rebuilt as native sequential/parallel extraction: stable whole-start three-way split, replay exclusion, retained rule-50 clock, exact phase quotas, deterministic reservoirs, atomic outputs and full hashes |
| `texel/test_pipeline.py` | Expanded Rarog extractor tests to cover split stability, replay leakage, rule-50 identity, quotas and manifests |

The Rarog `sprt.ps1` deliberately retains strength-v2's two-sided 600/3
adjudication for same-evaluator search A/Bs. Manta's strength-v1 one-sided
profile is older and was not imported. HCE and cross-engine games still run
without adjudication unless separately calibrated.

### Already stronger or equivalent in Rarog

| Manta tool | Rarog owner / reason not copied |
|---|---|
| `search_observe.zig`, `search_profile.ps1` | `tools/diag/phase4_differential.py`, revisioned diagnostics and counter specification aggregate both engines with explicit units and invariants |
| `watch.ps1` | Long Rarog runners already persist logs/manifests and can be observed without another state-changing wrapper |
| Colosseum bridge/config artifacts | Rarog's direct fastchess launch is asynchronous, resumable and hash-bound; Manta's own work found Colosseum checkpoint/PGN rewriting serialized datagen and checkpoint sharing failed on Windows |
| `pgn_result`-style result recovery | `tools/pgn_result.ps1` reconstructs complete pairs and is the authoritative stopped-run reader |
| Bench/counter parsing | `tools/diag/bench_counters.py` sums all position dumps; hand-rolled parsing is prohibited |
| NPS comparison | Rarog already has pooled-PGO self-pair validation and explicit small/large speed conversion rules |
| `texel/sample_fens.py` | Rarog already has the same streamed, phase-balanced seed sampler; no second copy needed |
| `texel/requirements.txt`, `test_positions.csv` | Dependency pin/test fixtures belong to each engine's native extractor and schema; Rarog's tests and environment remain authoritative |

### Engine-specific or one-off; do not port

| Manta source | Disposition |
|---|---|
| `board_bench.zig`, `eval_bench.zig` | Zig board/evaluator microbenches; keep Rarog's Rust benches and pooled search/evaluator NPS procedure |
| `differential_perft.zig` | Manta board implementation oracle; Rarog already has native perft/correctness tests |
| `generate_attack_tables.zig`, `generate_magics.zig` | Manta-specific generated tables and layouts |
| `policy_check.zig` | Manta compile-time feature-ledger check; Rarog's retained-option ownership is documented and tested in its own architecture |
| `eval_residual.zig`, `hce_fit.zig` | Coupled to Manta's HCE schema; Rarog uses `EvalTrace` and `texel-tuner` |
| `step_5_1_fastchess.ps1` | One-off historical gate replaced by the generic harness |
| `texel/fit_man_e20.py` | Candidate-specific MAN-E20 script; its semantic/static stop rule is imported, not its model |
| `.github` Zig setup, `tools/ci/install-zig.*`, `tools/zlint/` | Language-specific CI/lint infrastructure |
| `spsa_configs/*.json` | Manta parameter names/ranges are not portable; only the staged schedule and coverage discipline transfer |

### Concepts scheduled for native Rarog qualification at 4.7

Manta's `texel/audit_vector.py`, `fit.py`, `fit_sweep.ps1`, `score.py` and
`sweep_report.py` are tightly coupled to its coefficient schema, so copying
them would be a false integration. Rarog's tuner already has exact
reconstruction, feature support, cohort loss, validation selection, frozen
test reporting and L2. Step 4.7 now requires native evidence for the missing
contract: initial/free/fixed vector audit, covariance/identifiability, semantic
sign/bounds, full convergence trajectory and a hash-complete fit report.

## What Manta measured and what transfers

| Evidence | Result | Rarog consequence |
|---|---:|---|
| MAN-S19 raw/pruning/searched eval authority | **+13.02 +/- 7.21 nElo** | Process prior for 4.11: distinguish exact raw HCE, pruning refinement, qsearch baseline and TT searched evidence after the HCE refit. Not a formula or expected gain |
| Search overview | Tree about **1.50x wider per ply** than pinned reference while first-move cutoffs were **86.45%** | Ordering alone explained only part of Manta's gap; continuous selectivity fit was plausible there. Rarog must first measure its own per-position depth shape |
| MAN-S23 LMR desaturation | Failed its registered branching filter; one of 40 positions caused a **+41.5%** depth-12 reversal; earlier baseline mixed Hash 16/64 | Hash is part of the workload; use robust per-position shape and prospective zero-game refutation, never an endpoint average or tree size as acceptance |
| MAN-S25 correction residual | Static residual **-1.95%**, game gate **-1.40 +/- 6.03 Elo** | A live improving statistic explains mechanism, not Elo |
| MAN-E19 structural HCE plus constrained fit | **+35.91 +/- 11.19 Elo**, evaluator throughput **-36.2%** | Coverage -> coherent contracts -> constrained joint fit -> whole-bundle SPRT can work; the speed cost must be paid in the same gate and does not license another such loss |
| MAN-E18 fitted imbalance | About **-7 Elo** despite a tiny static improvement | Fit cannot rescue a semantically weak block; include covariant terms and reject the structure when the whole fitted cluster fails |
| MAN-E20 context-space | Required positive coefficient fitted negative, validation delta below floor, evaluator **-3.693%** | Prospectively registered semantic/loss/NPS checks may stop before games; none may promote |
| MAN-E21 shelter-danger coupling | Faster (**+1.461%**) but **-6.44 +/- 5.52 Elo** | Plausibility and speed are not strength evidence |
| HCE corpus and constrained fit | 1,162,814 unique starts; 3,000,000 train + 166,667 validation + 166,667 frozen test; test **0.104461517 -> 0.102574754**, all phases improved | Corpus scale was sufficient for Manta, but semantic rails mattered: the unconstrained fit improved loss while crossing required signs. Rarog must audit signs and convergence, not copy sizes as absolutes |
| Later HCE sweeps | Returned to the same attractor without meaningful improvement | Stop after the first no-gain/same-attractor cycle; more data or epochs need a concrete registered hypothesis |
| MAN-S26 sensitivity pilot | 128 x 32 = 4,096 games; five of six coordinates moved coherently, quiet futility returned to seed | A bounded pilot can authorize a full proposal, but its theta is neither a seed nor candidate |
| MAN-S27 whole-surface audit | Five-coordinate proposal withdrawn unrun after active LMR/LMP/history consumers were found missing | Audit the complete interacting active surface after the pilot, before spending full-tune games |
| MAN-S28 staged full SPSA | Ten coordinates, immutable 2,000 x 32 horizon, first stop 128; **prepared, not run** | Staged-stop mechanics transfer. There is no strength measurement to import |

## Plan incorporation

The conclusions are now explicit in:

- `PLAN.md` 4.11.1: fixed-hash per-position branching shape supplements the
  qsearch/TT authority baseline, as refutation evidence only.
- `PLAN.md` 4.8 and 4.12 plus `PROCESS.md`: bounded sensitivity pilot,
  complete surface audit, accepted-default full start, immutable horizon and
  no pilot/checkpoint promotion.
- `PLAN.md` 4.7: three-way leak-resistant corpus qualification and native
  vector/support/covariance/semantic/convergence evidence before HCE selection.
- `PLAN.md` 4.8–4.9: static semantic/loss/NPS filters may reject only; each
  dependency-complete locally fitted cluster pays a PGO game gate.
- `PLAN.md` 4.10: one constrained post-structure whole-HCE fit, settled trajectory, frozen
  test opened once, and stop on a failed or same-attractor follow-up.

The 2026-08-30 Basilisk audit subsequently moved the next step to **4.7 HCE
data/instrument qualification**. Search branching and qsearch/TT authority now
run at 4.11 on the accepted fitted HCE so their populations are not stale.
