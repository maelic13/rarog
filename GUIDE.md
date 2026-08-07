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
- [x] **4.2 Evidence/TT:** result kinds and explicit consumer contracts added;
      the 10-byte TT is preserved. Persisted provenance is deferred: `flag_age`
      has zero spare bits, and the one cheap slot (age 5→4 bits) moves the
      bench fingerprint, so it needs a strength gate rather than a neutrality
      check (RAR-S22).
  - [x] **4.2a Typed evidence:** `src/evidence.rs` types all 7 producers and
        routes all 13 read sites through named capabilities; bench-identical at
        6,502,902 with 96 matching depth lines against the pre-refactor binary
        (RAR-S23).
  - [x] **4.2b Contradiction shadow:** measured, and it reversed its own
        hypothesis. A contradicting entry's move is best 91.79% versus 84.77%
        for an agreeing one, so the penalty goes on the SCORE consumers only
        and must leave ordering and IIR alone (RAR-S24). Carried into 4.3.
  - [x] **4.2c Throughput:** pooled 3-build PGO A/B in both directions,
        bias-cancelled −0.125% (≈0.25 Elo), all six builds at bench 6,502,902
        (RAR-S28).
- [ ] **4.3 Qsearch/ProbCut:** preserve the measured depth-0 eval behaviour,
      explicitly separate speculative ProbCut evidence at singular consumers,
      then improve safe evasion/capture ordering.
  - [x] **4.3a-i Arms landed inert:** four knobs (`EvalPruneTtMinDepth`,
        `SingularTtDepthMargin`, `ProbCutStoreDepthAdj`, `QsRefineMinDepth`)
        plus the provenance-hazard census; bench-identical at 6,502,902 on
        normal, diag and tune builds (RAR-S25, RAR-S26).
  - [x] **4.3a-ii Arm A value 2: REJECTED.** −1.49 ± 2.87 Elo / −2.33 ± 4.49
        nElo over 23,044 games, LLR −2.19, manual stop; default stays 0. It
        searched 43.8% fewer nodes for no measurable Elo (RAR-S27).
  - [x] **4.3a-iii Arm A value 1: REJECTED at formal H0.** −3.18 ± 3.23 Elo,
        LOS 2.66%, 18,436 games; its interval narrowly includes zero. The
        all-depth RAR-S30 shadow is diagnostic, not a causal explanation.
  - [x] **4.3a-iv Arms C/D: RETIRED UNRUN.** They are low-value depth proxies;
        C stays inert for the joint fit and explicit provenance supersedes D.
  - [x] **4.3b Arm B: PARKED, NOT BAKED.** Value 2 reached H1 on the tune binary
        (+3.35 ± 2.44 Elo), while RAR-S32 confirmed matching decisions and
        similar aggregate speed across build modes. It neither isolates
        ProbCut nor justifies another long final-PGO gate for ~3 Elo; default 3
        remains and value 2 stays as an inert 4.10 coordinate/ablation.
  - [ ] **4.3c Persisted provenance: IMPLEMENTED, GATE OPEN.** One speculative
        bit plus the actual ProbCut result; singular alone rejects that class.
        Candidate fingerprint 6,595,869; 863 otherwise-eligible singular seeds
        blocked in the diagnostic bench. Final-PGO `[3,10]`, max 12k, is next.
  - [ ] **4.3d In-check qsearch ordering:** staged evasions plus capture/SEE
        history and coherent delta/SEE/futility, after 4.3c.
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

**Run only the 4.3c final-PGO gate below.** Do not resume the old aspiration
tuner or reuse `p102a-snapshot`. Phase 4.3a is finished as far as games go:

| Arm | Knob | Verdict |
|---|---|---|
| A | `EvalPruneTtMinDepth` 2 | rejected, −1.49 ± 2.87 (RAR-S27) |
| A | `EvalPruneTtMinDepth` 1 | rejected at formal H0, −3.18 ± 3.23 (RAR-S29) |
| C | `QsRefineMinDepth` 1 | retired unrun (RAR-S29) |
| B | `SingularTtDepthMargin` 2 | tune H1, but parked/not baked; default stays 3 |
| D | `ProbCutStoreDepthAdj` 4 | retired unrun; superseded by explicit provenance |

Arm B is resolved: do not bake it and do not spend another gate on it. RAR-S32
is retained as a build-transfer diagnostic, not final-PGO strength evidence.
The accepted baseline therefore remains at `SingularTtDepthMargin=3` and the
6,502,902 fingerprint. The unaccepted 4.3c candidate deliberately changes the
tree to 6,595,869.

Build the accepted baseline from `d00e1ac` and the 4.3c candidate from current
`development` using the same pinned final-PGO pipeline. Then run exactly one
gate on the other idle machine:

```powershell
.\tools\sprt.ps1 -EngineA <rarog-43c-pgo.exe> `
  -EngineB <rarog-d00e1ac-pgo.exe> -NameA 43c -NameB Baseline `
  -Elo0 3 -Elo1 10 -MaxGames 12000
```

Only H1 promotes 4.3c. H0 or the unresolved 12k cap parks/reverts it; do not
start a tune/non-PGO SPRT first.

⚠ **Never run anything timed while a gate is playing**, and never compile —
not even `cargo check -j 2`. Both mistakes were made in this cycle: a compile
plausibly caused a burst of time losses in the arm-A value-1 run, and an NPS
pass taken during a live SPRT read −2.36% against a true −1.15% with the
cross-configuration direction reversed. Deterministic outputs (node counts,
fingerprints) survive contention; nothing timed does.

To reproduce the 4.2 audit reading, the diag build is required — the plain
release binary emits no `diag` lines:

```powershell
cargo build --release --features diag
.\tools\diag_search_quality.ps1
```

## Decision rules

| Situation | Action |
|---|---|
| Behaviour-neutral | Exact bench plus fmt/tests/performance evidence |
| Coherent strength candidate | Bake the release semantics, build candidate and accepted baseline with the final PGO pipeline, then run one registered gate. Default `[3,10]` nElo, maximum 12,000 games; only H1 promotes. |
| Small knob or tune option | Keep inert, bundle with its subsystem or defer to 4.10. Same-binary games may be a short diagnostic, never a mandatory first release SPRT. |
| Gate reaches H0 or 12k unresolved | Park/revert. Do not promote from a point estimate and do not spend another 20k games resolving a marginal effect. |
| Broad architectural bundle | Use `[0,10]` or `[3,12]` as prospectively registered; preserve switches so a failed bundle can be ablated. |
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
  -NameA Candidate -NameB Baseline -Elo0 3 -Elo1 10 -MaxGames 12000
.\tools\nps_ab.ps1 -EngineA <candidate> -EngineB <candidate>
.\tools\nps_ab.ps1 -EngineA <candidate> -EngineB <baseline> -Rounds 12
```
