# Rarog Development Workflow Guide

This is the concise operational roadmap. Detailed rationale, contracts, gates
and lessons live in [`PLAN.md`](PLAN.md).

## Current checkpoint

| Item | State |
|---|---|
| Branch/release | `development` = 2.4.0 cycle; `master`/`v2.3.1` at `a5fd288` |
| Clean accepted head | Bench **6,502,902**, EBF **2.449**, `rarog-p1043-base-pext-pgo.exe` |
| Rating observation | Closed at the supplied 8,626/36,400 checkpoint; the mixed interim-constant result is not a gate or baseline. |
| Aspiration tune | Closed/rejected: stopped 2,510-iteration `p102a-snapshot` lost −4.16 ± 5.97 nElo over 13,000 gate games; retain `p1043-base`. |
| Evaluation | HCE frozen. No feature/weight/Texel work before NNUE. |
| Current phase | **Phase 4 — evidence-coherent pre-NNUE search** |
| Portability branch | `origin/arm_fix` = stale ARM-prefetch/TT-alignment/HCE-hoist experiments; inventory at 4.8, never merge wholesale |
| Next releases | **2.4.0 at 4.11**; baseline NNUE **2.5.0 at 6.7** |

The 2.4.0 target is direct paired superiority over every installed Rybka,
Critter 1.6a, Houdini 2.0c and Fritz 16. Basilisk is only the first rung.

## Closed phases

### Phase 1 — Foundations and robustness — ✅ 2.0.x–2.1.0

Built the Rust board/UCI/search/TT/history/SEE/Syzygy/time/SMP stack, testing
and release infrastructure; repaired early legality, draw, mate and clock bugs.

### Phase 2 — Evaluation and search expansion — ✅ 2.2.0

Delivered the major HCE/search program: +316 staged self-play and about +240
external transfer. Later evidence moved further evaluation work to NNUE.

### Phase 3 — Correctness, search wave and reproducible builds — ✅ 2.3.0/2.3.1

Banked correctness, ordering/in-check search, 4T SMP, CI and reproducible PGO.
Boundary transfer was +76/+78 Elo at 1T and +194 at 4T over 2.2.0, zero
forfeits; 2.3.1 restored Windows ARM64 PGO without changing search.

## Forward phases

### Phase 4 — Evidence-coherent pre-NNUE search (→ 2.4.0)

- [x] **4.0 Live experiments:** incomplete rating observation archived;
      stopped aspiration snapshot rejected; 6,502,902 baseline retained.
- [x] **4.1 Diagnostics:** deterministic sampled provenance, pruning recall/
      overlap, NMP/ProbCut/singularity, correction and root/SMP confidence;
      normal/diagnostic search equivalence verified.
- [ ] **4.2 Evidence/TT:** add result kinds and explicit consumer contracts
      while preserving the 10-byte TT unless measurements justify growth.
- [ ] **4.3 Qsearch/ProbCut:** stop stand-pat laundering, separate speculative
      cutoffs and improve safe evasion/capture ordering.
- [ ] **4.4 NMP/IIR/singular:** subtree null suppression, node/eval guards,
      PV-safe IIR, evidence-bound singularity and per-mechanism `tt_pv` gates.
- [ ] **4.5 History/correction:** prevent capture contamination, implement
      compact true continuation correction and evidence-selected contexts.
- [ ] **4.6 Selectivity:** one history-aware prospective-depth pipeline for
      LMP/futility/SEE/LMR, forcing-check classes and safe late evasions.
- [ ] **4.7 Root confidence:** connect root variance to aspiration, TM,
      completed legal fallback and SMP ownership.
- [ ] **4.8 Portability/ISA:** inventory `origin/arm_fix`; make x86 tier
      contracts executable, verify all native ARM runners/artifacts, target-
      measure prefetch/alignment/false sharing and archive each branch item.
- [ ] **4.9 Throughput/scaling:** profile accepted semantics and 1/2/4/8T
      without regressing the platform/ISA matrix.
- [ ] **4.10 One search SPSA:** freeze architecture, select ≤24 coordinates and
      run the only additional pre-NNUE fit plus post-fit ablations.
- [ ] **4.11 Release gate:** cumulative 1T/LTC/4T plus production platform/ISA
      matrix and Holm-adjusted
      paired wins over every Rybka, Critter 1.6a, Houdini 2.0c and Fritz 16;
      then release 2.4.0.

### Phase 5 — NNUE runway

- [ ] **5.0** freeze teacher/residual/search-disagreement corpora and contract.
- [ ] **5.1** per-ply state and complete dirty-piece make/unmake semantics.
- [ ] **5.2** accumulator ownership/refresh/full-recompute scaffolding.
- [ ] **5.3** pinned `D:/code/net_trainer` data/manifests/resume preflight.
- [ ] **5.4** bench-identical runway gate and integration branch.

### Phase 6 — Baseline NNUE (→ 2.5.0)

- [ ] **6.0** harden trainer CLI, splits, manifests, determinism/conformance.
- [ ] **6.1** controlled 30–60M initial data and label/mining A/Bs.
- [ ] **6.2** baseline networks with at least two seeds.
- [ ] **6.3** strict scalar loader/embedded net and exact references.
- [ ] **6.4** incremental accumulators and exact portable/x86/ARM64 kernels.
- [ ] **6.5** baseline data/architecture iteration one variable at a time.
- [ ] **6.6** gross NNUE-scale search safety calibration only.
- [ ] **6.7** HCE/STC/LTC/4T/external/parity gates and release 2.5.0.

### Phase 7 — NNUE frontier and final search fit

- [ ] **7.0** residual and search-disagreement analysis.
- [ ] **7.1** scale/deduplicate data, natural finishes and hard-position mining.
- [ ] **7.2** evidence-led king/threat/material/width architecture ladder.
- [ ] **7.3** the single post-NNUE search SPSA after architecture freezes.
- [ ] **7.4** contemporary-frontier and cumulative release gate.

### Phase 8 — Scaling, platforms and product completeness

- [ ] **8.0** high-thread/NUMA/root/TT/accumulator scaling.
- [ ] **8.1** advanced memory/network placement and runtime ISA dispatch.
- [ ] **8.2** demanded product or additional-platform work; baseline ARM64 and
      NNUE/NEON parity are already gates in 4.8 and 6.4/6.7.
- [ ] **8.3** scaling/platform release matrix.

### Phase 9 — Optional HCE fallback

Enter only after serious NNUE integration/data/architecture retries fail and
the user explicitly abandons that program.

- [ ] **9.0** document NNUE failure and approve HCE scope.
- [ ] **9.1** select a small residual-driven HCE program.
- [ ] **9.2** run one HCE fit and full external release matrix.

## What you run now

No long job is active or requested. Do not resume the old aspiration tuner or
reuse `p102a-snapshot`; its gate was rejected and the original config seeds are
the retained baseline. Phases 4.0 and 4.1 are complete, so the next numbered
implementation step is **4.2 — result evidence and TT contract**. Start another
SPRT/SPSA/gauntlet only when a later step explicitly prepares and requests it.

## Decision rules

| Situation | Action |
|---|---|
| Behaviour-neutral | Exact bench plus fmt/tests/performance evidence |
| Strength candidate | Registered SPRT; H1 accepts, otherwise revert behaviour |
| Root/TM/SMP | 1T STC/LTC plus 4T LTC, zero forfeits |
| Mechanism de-tunes consumers | Keep inert/ablatable until 4.10; post-fit ablation required |
| SPSA | Phase 4.10 and Phase 7.3 only unless new evidence authorizes another; never resume the rejected p102a run |
| NNUE baseline loses | Diagnose contract/data/training/architecture; do not jump to HCE |
| Target unavailable | Phase 4 stays open; rating-list inference is insufficient |

## Working rhythm

```text
You   -> Paste completed long-job artifacts or ask for the next step.
Model -> Implements, verifies, updates PLAN + GUIDE and commits without push.
You   -> Run only the requested SPSA/SPRT/gauntlet/datagen job.
```

## Common commands

```powershell
cargo fmt --check
cargo test
.\tools\build_test.ps1 -Suffix <name>
.\tools\sprt.ps1 -EngineA <candidate> -EngineB <baseline> `
  -NameA Candidate -NameB Baseline -Elo1 3
.\tools\nps_ab.ps1 -EngineA <candidate> -EngineB <candidate>
.\tools\nps_ab.ps1 -EngineA <candidate> -EngineB <baseline> -Rounds 12
```
