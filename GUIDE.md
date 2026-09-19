# Rarog development guide

## How to work with the engine agent

1. Ask **"what measured defect are we fixing?"** before asking which feature
   to add. Keep unresolved chess or architecture reasoning in `RESEARCH`.
2. Use the cheapest useful falsifier before expensive coding or games. Search
   prior negative results and do not retry one unless its recorded trigger
   fired.
3. Promote to `READY_FOR_IMPLEMENTATION` only when the mechanism, semantics,
   local evidence, interactions, falsifier and accept/reject rule are frozen.
4. Implementation acts like a colleague on ordinary code structure, builds,
   tests and cheap qualification. It does not redesign or broaden the
   experiment; a false premise returns the leaf to `RESEARCH`.
5. The agent prepares and verifies long tournaments, SPRTs, SPSA, datagen,
   PGO and profiling jobs; the maintainer starts them (about three
   SPRT-sized runs per day).
6. Freeze the prediction before exposure; judge the postmortem against it.
7. A clean negative result is progress. Clusters, not features; compatibility
   over completeness; donor architecture, own implementation.

| State | Boundary / control |
|---|---|
| `RESEARCH` | Evidence, alternatives, interactions, prediction, falsifier and stop rule are being established. |
| `READY_FOR_IMPLEMENTATION` | Research decision frozen; implementation may make ordinary local engineering choices. |
| `IMPLEMENTED` | Intended semantics exist; no qualification claim. |
| `LOCAL_QUALIFIED` | Cheap correctness/performance checks passed; expensive gate prepared. |
| `GAME_GATE` | Registered playing gate running or resolved under maintainer control. |
| `CLOSED` | Accepted, rejected, no-change or deferred disposition and calibration recorded. |

### Current model mapping

PLAN records only stable capability classes. Edit this table when model
generations change; do not rewrite the roadmap. These are maintainer
judgments, not measured rankings. An investigation leaf (`R3`) spawns the
implementation and measurement sub-steps under its step.

| Class | Capability | GPT | Claude |
|---|---|---|---|
| `R3` | Frontier causal/architecture research | GPT-6 Astra — Extra High | Claude Fable 5.1 — High |
| `R2` | Bounded correctness-sensitive reasoning | GPT-5.6 Sol — High | Claude Opus 5 — High |
| `I2` | Difficult implementation | GPT-5.6 Sol — High | Claude Opus 5 — High |
| `I1` | Well-specified implementation | GPT-5.6 Terra — Medium | Claude Sonnet 5 — Medium |
| `M` | Mechanical/docs/provenance | GPT-5.6 Terra — Medium | Claude Sonnet 5 — Medium |
| `V` | Verification/measurement | GPT-5.6 Sol — High | Claude Sonnet 5 — High |

### Reusable research prompt

> Investigate `<PLAN leaf>` as research, not implementation. Read PLAN,
> EXPERIMENTS, HISTORY's number map, linked analysis and relevant source;
> measured evidence outranks roadmap assumptions. Read the donor (Reckless
> first, Stockfish second) for mechanism, population and interaction, never
> for transcription. State the precise question, leading and competing
> hypotheses, shared signals and interactions, and whether search,
> evaluation, tooling or instrument effects could explain it. Design the
> cheapest discriminating test first; freeze its prediction, confidence,
> falsifiers and stop rule before exposure. Spawn the implementation and
> measurement sub-steps the handoff needs under the investigation's step,
> with a class each. Finish `READY_FOR_IMPLEMENTATION`, `MORE_RESEARCH` or
> `NO_CHANGE`, with the evidence for that verdict.

### Reusable implementation prompt

> Implement `<PLAN leaf>` from its registered handoff. Treat the research
> decision, semantics, invariants and experiment design as fixed. Write the
> donor's mechanism in Rarog's own structure; do not transcribe. Use normal
> engineering judgment for code, focused builds/debugging/tests and cheap
> qualification. Do not broaden the mechanism, tune unrelated behaviour or
> continue other roadmap work. If a research premise is false, preserve useful
> instrumentation, document the contradiction and return the leaf to
> `RESEARCH`. Prepare but do not start maintainer-owned expensive jobs. Report
> changes, interactions, validation, remaining gate and false assumptions;
> update PLAN, GUIDE and EXPERIMENTS under their ownership rules.

## Status board

Every phase, step and sub-step is a checkbox here. Rationale and design live
in `PLAN.md`; durable evidence in `EXPERIMENTS.md`; procedures in
`PROCESS.md`; finished work in `HISTORY.md`. `GUIDE.md` and `PLAN.md` change
together, and `python tools/diag/check_guide.py` must pass.

## Current checkpoint

| Item | Value |
|---|---|
| Released baseline | **2.4.0** on `master`, the `Version 2.4.0` squash of this `dev` state; fingerprint **7,601,220 / EBF 2.474**, `rustc 1.98.1`, per-tier PGO assets. Accepted by RAR-E16 at **+54.77 ± 17.04 Elo** over 2.3.2 |
| Development head | `dev` after B.2.0.2, version **2.4.0**; fingerprint **7,601,220 / EBF 2.474** unchanged on magic and PEXT (B.1, B.2.0 and B.2.0.2's MultiPV at its default are behaviour-neutral, RAR-P24, RAR-P25, RAR-P26; B.2.1's candidate is compiled only with `--features b2core`, which reads 4,706,910 / EBF 2.391: B.2.2.1's clamp conversion (6,586,667) was reverted by `308abe9` after RAR-S74's run (g), RAR-S73); last verified on Windows ARM64 and macOS ARM64 at the 2.4.0 head (RAR-P19); pinned `rustc 1.98.1` |
| Pool position, `3+0.03` 1T | Houdini 3 −224, Critter 1.6a −184, Houdini 1.5a −179, Fritz 16 −147, Rybka 4 −99, Basilisk 1.10.0 −23, Basilisk 1.9.3 −9, Rarog 2.3.2 +70 (RAR-M45, 2026-09-11, 600 games/pair, 2.4.0 release); reproduced within the 200-game bands in the 42-engine Super Rating Tournament, where 2.4.0 scores 64.8% and Houdini 4 −323, Stockfish 5 −222, Stockfish 1.9.1–4 within ±35 (RAR-M54, 2026-09-15) |
| Pool position, `3+0.03` **4T** | Houdini 3 −169, Fritz 16 −149, Critter 1.6a −109, Rybka 4 −73, Basilisk 1.10.0 **+25**, Rarog 2.3.2 +45, Rybka 3 +79; Perf 3034 vs frozen 3003 (RAR-M46, 2026-09-11) |
| Search deficit | **247.97 ± 10.89 Elo** equal time against the frozen oracle on the 2.4.0 head, evaluation proved constant (RAR-O03); depth gap only 0.97 ply, so most of it is decision quality; selectivity explains 272 ± 18. At equal nodes: WAC **200 vs 242** solved at 100k, median depth **16 vs 19** at 300k; Rarog's branching factor 1.630 is already below the oracle's 1.736 (RAR-M50) |
| Evaluation deficit | **about 329 Elo** against Stockfish's classical HCE with the same search |
| Speed | **3.27 MNPS** pooled median at the B.1 head, **+6.30% [+5.78%, +6.84%]** over the 2.4.0 pool (RAR-P24); the B.2.0 head **+0.21% [−0.34%, +0.68%]** over the B.1 pool (RAR-P25); the B.2.0.2 head **+0.62% [−0.29%, +1.18%]** over the B.2.1 head pool (RAR-P26). This host drifts by several percent between days, so compare pools interleaved only; Basilisk 3.71 (RAR-M48) |
| Conversion | **88 draws + 19 losses** after a persistent piece-up in 3,600 games vs the six HCE-era engines on the 2.4.0 release games; rate unchanged from the 2026-09-04 pool (57 + 12 in 2,400). Basilisk 1.9.3 in the same tournament 94 + 12, so RAR-M47's surplus reading is retired (RAR-M49, 2026-09-13, zero games); third sample 24.2 / 3.3 per 1,000 against the same six in the Super Rating Tournament, Basilisk 17.5 / 5.0 (RAR-M54) |
| Active experiment | **RAR-S73** (B.2 selectivity core: B.2.4a passed 2026-09-16, +65.09 ± 23.26; B.2.3's tune at 3,900 of 5,000, RAR-S75; RAR-S76 peek at 3,900 +118.72 ± 10.62; RAR-S77 and RAR-S78 registered; B.2.4b pending); **RAR-M45, RAR-M46, RAR-O03 and RAR-M48 all resolved 2026-09-11**; RAR-M49 conversion re-read and RAR-M50 (B.0 measurements) recorded 2026-09-13; RAR-M54 (Super Rating Tournament read, 42 engines) recorded 2026-09-15, nothing moves. **D.2's premise is contradicted by RAR-M46 and the leaf needs re-scoping** |
| Current step | **B.2.3's tune in sessions** (at 3,900 of 5,000 on 2026-09-19; resume with `-LaunchOnly -Iterations 5000`), then **B.2.4b** (fitted vs unfitted `[0,10]`) and **RAR-S77** (5,000 vs 3,900, `[0,3]`), then the `b2core` default flip |
| Next release | **3.0.0** if the E.2 target gate is met, otherwise 2.5.0 — cut at E.3 after the search and evaluation programmes. Nothing is released between now and that checkpoint unless a correctness repair forces a patch |

## Next and held work

**B.2.4a passed on 2026-09-16** (+65.09 ± 23.26, H1 in 432 games): the
unfitted `b2core` arm (4,706,910 / EBF 2.391) is the accepted head of B.2
and the base for B.3; B.2.4b later gates the fitted arm against it, and
the feature becomes the default build once that is decided. B.2.3 is the
SPSA over the registered coordinates plus the four clamp bounds, less the
five categorical switches, N = 5,000 in sessions. B.2.2 closed on 2026-09-15:
- the unfitted paired run measured +52.16 ± 10.73 Elo;
- no switch was adopted and mate-residual training stays;
- the clamp conversion was reverted to coordinates;
- the curvature sweep found three curved coordinates;
- the screen ladder for B.3–B.5 is now rule 8's *Cluster screens*.

B.7 keeps its place. B.3 waits for an accepted B.2 head. B.2.0.2 closed on 2026-09-15
(MultiPV, RAR-P26). B.2.1 was accepted by its reviewer on 2026-09-14
(`analysis/b21_review_2026-09-14.md`). Binding findings sit at their
leaves in PLAN: D.2's premise is contradicted (RAR-M46) and E.2's binding
arm is 1T.

| Open hold / obligation | Resume or resolve when | Must be resolved before |
|---|---|---|
| KRPPKRP 7-man truth gap | Independent truth becomes available, or C.5.8 records an explicit exclusion | C.5.8 closes |
| KRP-KB win-preserving 0.9990 → 0.9949 (−2.2 SE, RAR-M42) | Non-blocking; blocking if a later change pushes it past 3 SE | C.5.4 closes (owner) |
| B.2.3's N = 5,000 tune (maintainer-run, RAR-S75) | Pilot done; the tune completes in sessions (3,900 of 5,000 on 2026-09-19) and the final theta is pasted | B.2.4b and RAR-S77 start |

Follow the earliest unblocked leaf. Held items stay unticked in place.

## Phase A — Reset: repository, instruments, baselines, consolidation release

Open active leaves show `workflow state / capability class`. Execution order
is the numbering: release first, baselines on the released binary.

- [x] **A.1** Document reset — new PLAN, GUIDE, HISTORY; archives; checker — CLOSED, 2026-09-09
- [x] **A.2** Repository and branch cleanup
    - [x] **A.2.1** Tracked-file cleanup: twelve one-off or superseded files removed, each with its last commit — DONE 2026-09-09
    - [x] **A.2.2** Branch and tag disposition: seven branches tagged and deleted, oracle package archived, stale worktrees removed — DONE 2026-09-09
    - [x] **A.2.3** Feature and option inventory: 42 inert parameters for B.1, 55 seeds for B.2, features kept — DONE 2026-09-09
- [x] **A.3** Release gate and pre-release repairs — CLOSED 2026-09-10; the release itself was cut at A.9
    - [x] **A.3.1** Toolchain bump 1.97.1 → 1.98.1, behaviour-neutral: fingerprint, suites, ISA and pooled NPS all clean (RAR-P18) — DONE 2026-09-09
    - [x] **A.3.2** Release gate RAR-E16: **H1 accepted at 742 games, +54.77 ± 17.04 Elo**; 4T check +79.53 ± 21.21, zero forfeits; 2.4.0 licensed — DONE 2026-09-09
    - [x] **A.3.3** Time-forfeit repair: clock starts at `go` parse as in both donors (`79d3974`); RAR-R11 0 forfeits/10k, −0.69 ± 3.62; overhead sweep refuted at −81 — CLOSED 2026-09-10
- [x] **A.4** Build, asset and CPU-selection improvements; per-tier assets stay (universal binary deferred to G.2 as optional) — CLOSED 2026-09-10
    - [x] **A.4.1** `cc` 1.3.0 → 1.4.5, MSRV lockstep repaired; ISA clean with base at popcnt 0, fingerprint held; no platform profiles Fathom (RAR-P21) — DONE 2026-09-10
    - [x] **A.4.2** Startup CPU advisory: slow-PEXT gate and under-tier hint; no new unsafe, no new crate; message proven present per asset — DONE 2026-09-10
    - [x] **A.4.3** Direct `pext` vs `base` NPS: **+6.65% [+6.48%, +6.89%]**, chaining overstated by 0.50 pp (RAR-P22) — DONE 2026-09-10
    - [x] **A.4.4** README asset guidance: measured tier costs, slow-PEXT exception, and the now-false "cannot detect" claim corrected — DONE 2026-09-10
    - [x] **A.4.5** Engine argv handling: arguments run through the stdin dispatch; unknown argument exits 2 — DONE 2026-09-10
- [x] **A.5** Conversion instrument: PGN-based, seed reproduced 57/12 and 40/12, lone-minor guard proven live (RAR-M47) — DONE 2026-09-10
- [x] **A.6** Codebase consolidation analysis: Phase A refactors nothing; B.1/C.1 handoffs, dead-code list with the root-confidence subsystem, three `eval/` layout additions — DONE 2026-09-10
- [x] **A.7** Version bump to 2.4.0: `id name Rarog 2.4.0`, fingerprint unmoved, CHANGELOG opened — DONE 2026-09-10
- [x] **A.8** Baselines on the 2.4.0 binary — all four resolved 2026-09-11
    - [x] **A.8.1** Reference pool refresh, 12 engines, 600 games per pair: 39,600 games, zero forfeits, both predictions scored — RAR-M45, DONE 2026-09-11
    - [x] **A.8.2** Four-thread gauntlet: 2,800 games, all seven predictions missed, 4T is the *easier* arm — RAR-M46, DONE 2026-09-11
    - [x] **A.8.3** Oracle deficit meter: G(0) = −247.97 ± 10.89, depth gap only 0.97 ply — RAR-O03, DONE 2026-09-11
    - [x] **A.8.4** Pooled-PGO NPS baseline: 3.19 MNPS median, null +0.04% [−0.23%, +0.15%] — RAR-M48, DONE 2026-09-11
- [x] **A.9** Release 2.4.0: documents finalised, squashed to `master`, CI green, tagged and published with per-tier assets — DONE 2026-09-11

## Phase B — Search programme (evaluation frozen)

- [x] **B.0** Investigation: mechanism map, cluster contents, scale ratio 0.457 (eval) / 0.75 (SEE), B.2.2 screens registered as a branching window plus fixed-node quality, 116 oracle-anchored canaries, B.1–B.3 handoffs frozen (`analysis/search_programme_2026-09-13.md`, RAR-M50) — DONE 2026-09-13
- [x] **B.1** Search restructure: `search/` modules, `NodeType`, `ThreadData`, sentinel `PlyArray` stack; 44 inert parameters, root confidence, SMP skip and `evidence.rs` removed; exact fingerprint, pooled-PGO NPS +6.30% (RAR-P24) — DONE 2026-09-14
- [ ] **B.2** Cluster 1 — selectivity core: TT eval storage, correction, histories, picker, move-loop pruning, LMR — **READY_FOR_IMPLEMENTATION / I2**
    - [x] **B.2.0** Architecture review of the B.1 head (RAR-M51) and its twelve behaviour-neutral upgrades: narrowed surface, tagged commands, search output port, TT policy factored once, tuner out of the workspace, tools index, comment hygiene; exact fingerprint, NPS +0.21% vs the B.1 pool (RAR-P25) — DONE 2026-09-14
        - [x] **B.2.0.1** Repository and document restructure: archived trackers, one-line closed leaves, one ledger row per experiment, `analysis/` index, dead-path check, rule-first AGENTS; logos stay tracked (RAR-M53) — DONE 2026-09-14
    - [x] **B.2.1** Implement to the B.0 handoff with table, picker, TT and unwind tests; behind `b2core`, 4,706,910 / EBF 2.391 unfitted, off arm exact; reviewer-accepted (RAR-S73, `analysis/b21_review_2026-09-14.md`) — DONE 2026-09-14
        - [x] **B.2.0.2** MultiPV: UCI option up to 256, root lines above 1; identity at `MultiPV = 1` on both arms (bench and `info` stream), pooled NPS +0.62% (RAR-P26) — DONE 2026-09-15
    - [x] **B.2.2** Diagnostics: oracle differential, depth at 300k, EBF, reference-anchored branching curve, tactical suite, 2,000-game unfitted run; screen thresholds registered by B.0. Screens run 2026-09-15: paired run **+52.16 ± 10.73 Elo** unfitted while four zero-game floors fail (`analysis/b22_screens_2026-09-15.md`); reviewed, the paired run governs, re-planned as B.2.2.1–B.2.2.4 — DONE 2026-09-15
        - [x] **B.2.2.1** Two seed-scale clamps converted (`b2core` 6,586,667 / EBF 2.433); five categorical switches (`CoreRazorGuards`, `CoreCorrTrainDecisive`, `CoreCorrTrainExcluded`, `CoreLmrFullDepth`, `CoreLmrCheckRoot`) with tests; `b2core,tune` PGO build; RAR-S74 registered — DONE 2026-09-15
        - [x] **B.2.2.2** Paired runs (a)–(g) read: nothing adopted, mate-residual training stays, the clamp conversion reads −7.64 ± 9.72 and is reverted by `308abe9` to four SPSA coordinates at the donor's seeds, `b2core` 4,706,910 / EBF 2.391 again (RAR-S74) — DONE 2026-09-15
        - [x] **B.2.2.3** Curvature sweep of the five §9 coordinates and the P6 profile: three curved (`CoreLmpSquare`, `CoreLmrQuiet`, `CoreCorrUpdateSlope`), so B.2.3 runs; P6 +5.59% (`analysis/b223_sweep_2026-09-15.md`) — DONE 2026-09-15
        - [x] **B.2.2.4** Screen rules for B.3–B.5 in rule 8: paired run governs, one ablation sweep in mechanism order, time-to-depth, positional screen (phase-4 suite until STS is placed), canary regression rule, sweep checklist — DONE 2026-09-15
    - [ ] **B.2.3** SPSA on the `b2core` arm: 82 `CoreParams` coordinates (switches, `CoreIirMinDepth` and `CoreEvalRule50Damping` out), `config_b23core.json`, the `-Tune -Features b2core` tooling ticket, pilot 128 × 32, N = 5,000 in resumable sessions registered as RAR-S75, final theta baked, fitted-vs-unfitted diagnostic run. Preparation done 2026-09-15 (RAR-S75, binary `25467C63…`, setup proven at N = 5,000); pilot done, tune at 3,900 of 5,000 (2026-09-19); RAR-S76 peek +118.72 ± 10.62; follow-ups RAR-S77, RAR-S78 — **IMPLEMENTED / V**
    - [ ] **B.2.4** Gate as two SPRTs: B.2.4a unfitted `b2core` vs the off arm `[0,10]` **passed 2026-09-16, +65.09 ± 23.26 in 432 games** (the unfitted arm is the accepted head); B.2.4b fitted vs unfitted `[0,10]` after B.2.3, then the default flip — **GAME_GATE / V**
    - [ ] **B.2.5** UCI `info` conformance: winner's line after the SMP vote, `depth 0` line at a mated root, `nps` at `time 0`, `multipv 1` always, bounds in single-PV, seldepth convention; identity-gated, no SPRT (`analysis/uci_info_review_2026-09-16.md`) — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **B.2.6** Adopt Colosseum CLI as the harness, once `cli-v0.1.0` is released and qualified in its own repository, B.2.3's tune has finished on weather-factory and B.2.4b is gated with `sprt.ps1`; fastchess stays staged as the second runner — **RESEARCH / M**
        - [ ] **B.2.6.1** Policy as committed run files and thin `sprt`/`spsa` wrappers that keep the sidecar, fingerprint, compiler-equality and dirty-tree guards; `setup_tools.ps1` stages the tagged release and pins its SHA-256; resolved-configuration parity against the `sprt.ps1` manifest and one short live run — **RESEARCH / I1**
        - [ ] **B.2.6.2** Retire the replaced scripts; rewrite PROCESS, AGENTS and `tools/README.md`; ledger rows for the 2026-09-17/18 parity runs; next tune registered at 15 slots, 30 games per iteration, budget in games — **RESEARCH / M**
- [ ] **B.3** Cluster 2 — NMP, ProbCut, singular/multi-cut/negative/LDSE extensions, IIR policy; SPRT `[0,5]` — **READY_FOR_IMPLEMENTATION / I2**
- [ ] **B.4** Cluster 3 — quiescence: TT, corrected stand-pat, LMP, SEE margin; first-ply check generation measured against B.2's mate-threat canaries; research card on the search's never-fitted piece-value scale (RAR-M19); SPRT `[0,3]` — **RESEARCH / I2**
- [ ] **B.5** Cluster 4 — root, aspiration, iterative deepening, PV; keeps B.2.0.2's MultiPV contract; SPRT `[0,3]` — **RESEARCH / I2**
- [ ] **B.6** Joint search SPSA, only if curvature justifies it — **RESEARCH / V**
- [ ] **B.7** Search speed pass on the new modules; pooled-PGO floor +0.5% per change — **RESEARCH / I1**
- [ ] **B.8** Cleanup: dead parameters, old picker, unconsumed provenance, ownerless diagnostics — **RESEARCH / I1**
- [ ] **B.9** Checkpoint: G(0), depth/EBF, NPS, conversion, pool gauntlet; remove `ablate`; freeze the search head — **RESEARCH / V**

## Phase C — Evaluation programme (search frozen)

- [ ] **C.0** Investigation: family map, residuals, donor conditioning, shared inputs, cluster order, refit protocol — **R3**
- [ ] **C.1** Evaluation restructure, behaviour-neutral: modules, one attack-map producer, `eval/params.rs`, `kpk` under `endgame/`; exact fingerprint — **I1**
- [ ] **C.2** Datagen and label contract for the programme; corpus frozen under a new name; fitting manifest (free/fixed/excluded) — **V**
- [ ] **C.3** King safety cluster: danger units, safe/unsafe checks, weak ring, flank, shelter/storm; refit; gate — **I2**
- [ ] **C.4** Threats and mobility cluster: mobility area, weak enemies, hanging, restricted, pawn push, queen threats; refit; gate — **I2**
- [ ] **C.5** Endgame handling and winnability cluster — **R3**
    - [ ] **C.5.1** Classification and deciding instrument per family — **R2**
    - [ ] **C.5.2** Generic winnability and scaling: pawn count, opposite bishops, rule-50 scale, complexity — **I2**
    - [ ] **C.5.3** Conversion cluster: KXK, KBNK, KQKR; rule-50 damping interaction measured — **I2**
    - [ ] **C.5.4** Rook versus minor cluster: KRKN, KRKB, KRPKB — **I2**
    - [ ] **C.5.5** Rook and pawn cluster: KRPKR, KRKP, KPK, KPKP audit — **R2**
    - [ ] **C.5.6** Measure-first families: KPsK, KBPsK, KBPPKB, KQKRPs — **R2**
    - [ ] **C.5.7** Theory sweep: KBPKB, KBPKN, KNNKP, KNNK, KQKP from one dispatcher — **I1**
    - [ ] **C.5.8** Endgame gate: endgame-start cohort SPRT plus STC SPRT; floors; conversion; 7-man exclusion — **V**
- [ ] **C.6** Pawns and passers cluster; refit; gate — **I2**
- [ ] **C.7** Material, imbalance, phase and pieces cluster; refit; gate — **I2**
- [ ] **C.8** Refit cycles: regenerate, refit, gate; stop at the first non-accepting cycle — **V**
- [ ] **C.9** HCE SPSA of nonlinear residue, or a written skip — **V**
- [ ] **C.10** Search cp-margin re-fit after the new evaluation; SPRT `[0,3]` — **V**
- [ ] **C.11** Checkpoint: same-search deficit, conversion, NPS, pool gauntlet; freeze the classical evaluation — **V**

## Phase D — Clock, threads, robustness

- [ ] **D.1** Time management: audit against the ADR-0065 checklist, soft/hard bounds with node-fraction multiplier, forfeit margin; SPRT `[0,3]` — **R2**
- [ ] **D.2** Lazy SMP quality at 4T/8T: diversity, shared TT and correction, soft-stop voting; 4T SPRT `[0,5]` — **R2**
- [ ] **D.3** Engine lifecycle and protocol robustness; `src/uci/` (planned) move; score normalisation research card (`analysis/uci_info_review_2026-09-16.md` item 6); zero crashes over pool tournaments — **R2**
- [ ] **D.4** Tablebase policy: probing depth/limits, WDL/DTZ in conversion, recogniser interaction — **R2**

## Phase E — Classical checkpoint and release

- [ ] **E.1** Attribution checkpoint: B.2.0 review re-run on the B.9/C.11 heads; STC, `10+0.1`, 4T against 2.3.2 and the B.9/C.11 heads; maturity checklist — **V**
- [ ] **E.2** Target gate: ≥50% against Critter 1.6a, Houdini 3, Rybka 4 and Fritz 16 at 1T and 4T — **V**
- [ ] **E.3** Release 3.0.0 (gate met) or 2.5.0: changelog, suites, PGO assets, ISA, CI with tag-equals-version and cross-matrix fingerprint assertions, tag on instruction — **M**

## Phase F — NNUE (own data only)

- [ ] **F.0** Investigation: board events, accumulator ownership, trainer choice, data format, first architecture — **R3**
- [ ] **F.1** Board events and accumulator scaffolding, behaviour-neutral for HCE; cost ledger — **I2**
- [ ] **F.2** Data generation at scale: 30–60M unique positions, splits, manifests, hashes — **V**
- [ ] **F.3** Trainer hardening and baseline nets, two seeds per configuration — **I2**
- [ ] **F.4** Scalar integration: `quantised.bin` contract, integer-exact conformance, HCE fallback — **I2**
- [ ] **F.5** Incremental and SIMD: same-net parity on every move type, tiers, pooled NPS attribution — **I2**
- [ ] **F.6** Search re-fit for the network — **V**
- [ ] **F.7** Architecture ladder: output buckets, king buckets, relation/threat inputs; one axis at a time — **R3**
- [ ] **F.8** Data frontier: on-policy refresh, deduplication, hard-position mining — **V**
- [ ] **F.9** NNUE release: beat the classical release at STC, LTC and 4T; platform matrix — **M**
- [ ] **F.10** CCRL top-100 gate — **V**

## Phase G — Scaling, platforms and the top 50

- [ ] **G.1** High-thread and NUMA: 8/16/32T, TT and net placement, large pages, affinity policy — **R2**
- [ ] **G.2** Platform and product: Chess960 on demand, distributed testing, and the OPTIONAL universal binary (`analysis/universal_binary_2026-09.md`) — **I1**
- [ ] **G.3** Frontier: larger nets, data scaling, LTC search fit; CCRL top-50 gate — **R3**
