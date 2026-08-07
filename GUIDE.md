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
- [x] **4.3 Qsearch/ProbCut — CLOSED, nothing baked.** Depth-0 eval behaviour
      preserved (restricting it lost twice), speculative ProbCut evidence is now
      separable at singular consumers via a persisted bit, and the
      evasion/capture-ordering half moved to 4.6. Five gates spent, no accepted
      strength change — the honest outcome, and the reason later steps bundle.
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
  - [x] **4.3c Persisted provenance: DONE — gate resolved, landed inert.** One
        speculative bit plus the actual ProbCut result, with singular alone
        rejecting that class. **Gate NOT promoted:** dead neutral at +0.35 ± 6.18
        Elo over 4,960 games, LLR −1.71 (RAR-S34). Attribution: the age bit is
        FREE (0.00% nodes), the singular rejection is free and 1.15% FASTER to
        depth, and the whole ~4.3% headwind is the bundled
        actual-ProbCut-score change (+5.55% TTD alone) which the contract does
        not need. So the infrastructure is retained and BOTH consumers became
        switches defaulting off — the head is back at **6,502,902 / EBF 2.449**,
        a behaviour-neutral landing owing no strength gate.
        `SingularRejectSpeculative=1` reproduces 6,490,746 and adding
        `ProbCutStoreActualScore=1` reproduces 6,595,869, both verified against
        the attribution's own builds. **4.4 turns the contract on inside its
        bundle**; neither switch may be flipped alone without a gate.
  - [x] **4.3d MIGRATED to 4.6, and 4.3 is CLOSED.** In-check qsearch ordering
        and capture/SEE history move to 4.6, which already owns the late-evasion
        mismatch and the check taxonomy, so it is the same work under another
        heading — and far too small for a fifth standalone gate. Number stays
        frozen as a historical reference.
- [x] **4.4 NMP/IIR/singular — CLOSED, all mechanisms inert and sized.** Subtree
      null suppression, node/eval guards, evidence-bound singularity and
      per-mechanism `tt_pv` eligibility all implemented behind switches, each
      individually measured. PV-safe IIR de-scoped by measurement (~1 sampled
      node). The gate it was to have run is owned by **4.10a**, where the
      accumulated bundle freezes the architecture.
  - [x] **4.4a Switches landed inert and sized (RAR-S35).** Five switches, all
        defaulting to baseline, bench still 6,502,902 on normal/diag/tune. Cheap
        set for the first bundle: `NmpSuppressNullInVerification` (−2.95% nodes)
        + `RazorAllowTtPv` (+0.11%) + 4.3c's contract (−1.15% TTD). Expensive and
        held back: `NmpAllowTtPv` +4.53%, `RfpAllowTtPv` +6.67%,
        `ProbCutAllowTtPv` +16.52%. PV-safe IIR de-scoped — population is ~1
        sampled node.
  - [x] **4.4b Guards landed inert and sized (RAR-S36).** `NmpDecisiveGuard`
        (0.00% — zero bench population, so a soundness guard not a strength arm),
        `NmpUseStaticEval` (+2.66%), `SingularMaxExtension=1` (+6.57%),
        `NmpRequireCutNode` (+14.42%). New `tests/zugzwang.rs` passes with every
        switch off, on individually, and all ten together — the only instrument
        that can verify a guard bench cannot see.
  - [x] **4.4c Remaining guards landed inert (RAR-S37).** `NmpSingularGuard`
        (+11.33%), `NmpMinNonPawnPieces` 2/3 (+13.06%/+9.53%, non-monotone),
        `SingularDoubleMargin=60` (−2.25%). The registered first bundle measures
        **1.57% FEWER nodes than baseline** — no speed headwind, unlike 4.3c.
  - [x] **4.4d Gate deferral DECIDED and handed off.** At a ~5 nElo prior the
        bundle sat in the dead zone a 16,000-game `[3,10]` gate cannot resolve
        (only ≤3 or ≥10 nElo resolve; see `PLAN.md` 4.4c). The decision is
        recorded and closed; **the gate itself is now owned by 4.10a**, which is
        where it must run because it is what freezes the architecture the fit
        needs. Nothing about 4.4 is outstanding.
- [x] **4.5 History/correction — CLOSED.** Capture attribution measured and a
      graded weight landed, true 2/4-ply continuation pairs added with
      centralized aging, a 9%-of-tree double-count found and guarded, and
      evidence-selected contexts resolved to ADD NONE. All inert; nothing baked.
  - [x] **4.5a Attribution measured (RAR-S38).** Capture-caused residuals average
        179.1 cp against 78.8 cp quiet — a **2.27x** ratio over 283,590 updates.
        So the capture guard was directionally RIGHT and its instrument wrong:
        exclusion discards 51.3% of training (RAR-S16, −55.98 Elo). Graded
        `CorrCaptureWeightPct` landed inert at 100; the weight is a 4.10
        coordinate, not a standalone gate.
  - [x] **4.5b Continuation pairs + centralized aging (RAR-S39).** Continuation
        correction extended from one 1-ply slot to compact 2- and 4-ply
        distances; all three tables now age through a single loop so they cannot
        drift out of scale and invalidate fitted weights. Both weights inert at
        0 and cost no table access; sized at 152 they run +6.33% and +12.02%
        nodes, so both are 4.10 coordinates rather than bundle members.
  - [x] **4.5c Double-counting found and guarded (RAR-S40).** `corr_abs` widens
        margins for a correction that a TT bound may have REPLACED — exact
        population **360,811 nodes, 9.0% of the tree**. `CorrSkipWhenTtRefined`
        removes it in that case and measures **−4.50% nodes**, the only 4.5 arm
        cheaper than baseline, so it joins the first bundle. Also fixed a stale
        comment claiming these scales were inert; the seeds are 3/3/27 and live.
  - [x] **4.5d Contexts measured — ADD NONE (RAR-S41).** Halfmove clock 0–19
        holds 98.64% of all 283,590 updates, so the 2.1x low-versus-high residual
        ratio has no population to learn from; and check/evasion is unreachable
        by construction, since correction trains only where `static_eval !=
        VALUE_NONE`. A context needs a distinct mean **and** a population —
        checking only the mean would have justified a useless table. The
        capture/quiet split (4.5a) remains the only context with both.
- [x] **4.6 Selectivity - CLOSED (absorbed former 4.3d):** one history-aware
      prospective-depth pipeline for
      LMP/futility/SEE/LMR, forcing-check classes and safe late evasions,
      plus in-check qsearch ordering and capture/SEE history. ⚠ Do NOT restrict
      depth-0 TT evidence here — that lost twice (RAR-S27/S29).
  - [x] **4.6a Late-evasion contradiction resolved (RAR-S42).** The LMR comment
        claimed evasions were reducible; the predicate said otherwise. The code
        was right — making them reducible costs **+14.83% nodes**, so the arm is
        a 4.10 coordinate, not a bundle member. Third comment/code mismatch this
        cycle: a comment asserting a mechanism's state is not evidence.
  - [x] **4.6b Shared prospective depth (RAR-S43).** LMP, futility and SEE now
        can read the depth LMR will actually search at, via one extracted
        reduction formula that both callers share — checked by a `debug_assert`,
        not just asserted in a comment. `SelectivityProspectiveDepth=1` measures
        **−8.70% nodes, EBF 2.449 → 2.424**: the largest cheap arm in Phase 4 and
        the strongest 4.10a bundle member. Inert at 0.
  - [x] **4.6c Check classes landed; other two items disposed (RAR-S44).** The
        flat 32,000 bonus became a tunable coordinate. The safe/losing SPLIT was
        implemented, measured NON-FUNCTIONAL and **reverted** (RAR-S45: zero
        losing-check population across 332,683 checks, because `see_ge` is
        trivially true for a non-capture — the predicate was wrong). Post-LMR depth feedback needs no new attempt: already rejected
        twice (Phase 2.8 -1.38, RAR-S14 -7.29). In-check qsearch ordering is
        already complete (`score_moves` when in check); the lazy-STAGING half is
        a throughput item and moves to 4.9.
- [ ] **4.7 Root confidence:** connect root variance to aspiration, TM,
      completed legal fallback and SMP ownership. ⚠ Aspiration has LOST twice
      (RAR-S17 −4.52, RAR-S20) — treat it as the highest-risk consumer.
  - [x] **4.7a Abort path covered (RAR-S46).** `bench` is fixed-depth and never
        aborts, so `root_interrupted_fallback` reads 0 and the fingerprint
        cannot see this path. New `tests/root_abort.rs` interrupts at swept
        budgets and pins four properties: legal move, no unproven mate score,
        no depth beyond what completed, and determinism. 9.8s.
  - [ ] **4.7b Remaining:** one `RootConfidence` snapshot consumed by aspiration
        and TM without double-counting, and worker instability pooled for TIME
        only, never for result ownership.
- [ ] **4.8 Portability/ISA:** inventory `origin/arm_fix`; make x86 tier
      contracts executable, verify all native ARM runners/artifacts, target-
      measure prefetch/alignment/false sharing and archive each branch item.
- [ ] **4.9 Throughput/scaling (also owns 4.6's in-check qsearch STAGING):**
      profile accepted semantics and 1/2/4/8T
      without regressing the platform/ISA matrix.
- [ ] **4.10 One search SPSA:** freeze architecture, select ≤24 coordinates and
      run the only additional pre-NNUE fit plus post-fit ablations.
  - [ ] **4.10a Accumulated-bundle gate — OWNS the deferred 4.4 gate.**
        ⚠ **Composition must be MEASURED as a set, not summed (RAR-S45):** the
        six cheap members below summed to −17% nodes but measure **+4.57% as a
        set**, the same headwind shape that landed 4.3c dead neutral. Rebuild
        the composition from measured subsets first. Must run
        BEFORE the fit: it is what freezes the architecture. Current cheap-set
        members, all inert and individually sized:
        `CorrSkipWhenTtRefined` (−4.50% nodes),
        `NmpSuppressNullInVerification` (−2.95%),
        `SingularRejectSpeculative` (−1.15% TTD),
        `RazorAllowTtPv` (+0.11%), `NmpDecisiveGuard` (0.00%).
        Add any cheap arm 4.6/4.7 produce, then re-check `PLAN.md` §2's
        resolvability table against the final composition and register bounds
        and cap **prospectively**. Keep every member ablatable so a failure can
        be attributed.
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

**No job is requested. Phase 4.3 is CLOSED with nothing baked** — 4.3a and 4.3c
are resolved and 4.3d has moved to 4.6. Do not resume the old aspiration tuner
or reuse `p102a-snapshot`.

| Arm / step | Knob | Verdict |
|---|---|---|
| A | `EvalPruneTtMinDepth` 2 | rejected, −1.49 ± 2.87 (RAR-S27) |
| A | `EvalPruneTtMinDepth` 1 | rejected at formal H0, −3.18 ± 3.23 (RAR-S29) |
| C | `QsRefineMinDepth` 1 | retired unrun (RAR-S29) |
| B | `SingularTtDepthMargin` 2 | tune H1, parked; default stays 3 (RAR-S31) |
| D | `ProbCutStoreDepthAdj` 4 | retired unrun; superseded by provenance |
| 4.3c | `SingularRejectSpeculative` | gate NOT promoted, neutral +0.35 ± 6.18; landed INERT (RAR-S34) |

The accepted head is therefore unchanged at **6,502,902 / EBF 2.449**. 4.3c's
provenance bit and age narrowing are retained and cost nothing; its two
behaviour switches default off and are verified to reproduce 6,490,746 and
6,595,869 when enabled.

**Next step is 4.4 — NMP/IIR/singular, which also owns 4.3c's gate.** Build the
subsystem with cheap deterministic ablations, then ONE material-gain gate on the
coherent final-PGO bundle with `SingularRejectSpeculative=1` included. No games
until that bundle exists — 4.3 spent ~100k games on five standalone gates and
banked nothing, which is the reason this step bundles.

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
| Coherent strength candidate | Bake the release semantics, build candidate and accepted baseline with the final PGO pipeline, then run one registered gate. Default `[3,10]` nElo, maximum **16,000** games; only H1 promotes. The cap is derived from the LLR drift calibration in `PLAN.md` §2 — a candidate exactly on H1 needs ~14,500 games, so 12,000 would have parked changes the bounds accept. |
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
  -NameA Candidate -NameB Baseline -Elo0 3 -Elo1 10 -MaxGames 16000
.\tools\nps_ab.ps1 -EngineA <candidate> -EngineB <candidate>
.\tools\nps_ab.ps1 -EngineA <candidate> -EngineB <baseline> -Rounds 12
```
