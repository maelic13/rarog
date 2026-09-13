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
| Development head | `dev`, version **2.4.0**, identical in content to the released `master`; fingerprint **7,601,220 / EBF 2.474**, re-verified on Windows x86-64, Windows ARM64 and macOS ARM64 at this head (RAR-P19); accepted by RAR-E15 (+12.12 ± 10.17 Elo); pinned `rustc 1.98.1` since A.3.1 |
| Pool position, `3+0.03` 1T | Houdini 3 −224, Critter 1.6a −184, Houdini 1.5a −179, Fritz 16 −147, Rybka 4 −99, Basilisk 1.10.0 −23, Basilisk 1.9.3 −9, Rarog 2.3.2 +70 (RAR-M45, 2026-09-11, 600 games/pair, 2.4.0 release) |
| Pool position, `3+0.03` **4T** | Houdini 3 −169, Fritz 16 −149, Critter 1.6a −109, Rybka 4 −73, Basilisk 1.10.0 **+25**, Rarog 2.3.2 +45, Rybka 3 +79; Perf 3034 vs frozen 3003 (RAR-M46, 2026-09-11) |
| Search deficit | **247.97 ± 10.89 Elo** equal time against the frozen oracle on the 2.4.0 head, evaluation proved constant (RAR-O03); depth gap only 0.97 ply, so most of it is decision quality; selectivity explains 272 ± 18 |
| Evaluation deficit | **about 329 Elo** against Stockfish's classical HCE with the same search |
| Speed | **3.19 MNPS** pooled median, bench 13, PGO pext 1T, instrument ±0.2% (best-of 3.21 = the old 3.22); Basilisk 3.71 (RAR-M48) |
| Conversion | 57 draws + 12 losses after a persistent piece-up in 2,400 games vs the six HCE-era engines — **2026-09-04 pool, `2.4.0-dev` binary; the one meter not yet re-read on the release head** (RAR-M47 can, at zero game cost) |
| Active experiment | none; **RAR-M45, RAR-M46, RAR-O03 and RAR-M48 all resolved 2026-09-11**. **D.2's premise is contradicted by RAR-M46 and the leaf needs re-scoping** |
| Current step | **B.0 — the search programme investigation**. Phase A is closed; 2.4.0 is released |
| Next release | **3.0.0** if the E.2 target gate is met, otherwise 2.5.0 — cut at E.3 after the search and evaluation programmes. Nothing is released between now and that checkpoint unless a correctness repair forces a patch |

## Next and held work

**Phase A is closed and 2.4.0 is released.** Its records are in `HISTORY.md`:
the document and repository reset, the 1.98.1 pin, RAR-E16 licensing the
release at +54.77 ± 17.04 Elo over 2.3.2, the pre-release repairs of A.3.3 and
A.4, the conversion instrument, the consolidation analysis that decided Phase A
refactors nothing, the version bump, and four baselines measured on the
released binary.

**B.0 is the next executable leaf** — the search programme investigation, and a
`RESEARCH / R3` one, so it produces frozen handoffs rather than code. It owns
the question Phase A sized: **−247.97 ± 10.89 Elo** of equal-time search
deficit against the frozen oracle, of which the matched ablation attributes
**272 ± 18** to LMR plus shallow-depth pruning. A.8.3 added the sharpest clue —
the oracle leads by only **0.97 ply**, so this is decision quality at nearly
equal depth, not depth. B.0 ends with cluster boundaries, the scale ratio,
seeds and instruments frozen for B.1 to B.3.

Two Phase A findings bind later work, recorded in PLAN at the leaves that own
them. **D.2's premise is contradicted** — at 4T Rarog beats this reference
field rather than trailing it, so the leaf must be re-justified against modern
engines before it consumes work; the clean instrument is self-relative scaling
against Reckless and Stockfish. **E.2's binding arm is 1T, not 4T**, by 26 to
75 Elo on three of the four targets, so Phase B and C are judged against the 1T
column. The universal binary stays **optional and unscheduled** under G.2, its
design in `analysis/universal_binary_2026-09.md`.

One meter is owed rather than blocked: **conversion has not been re-read on the
released binary**. Every other figure in the checkpoint is; A.5's PGN
instrument (RAR-M47) can re-read it from the RAR-M45 games at zero game cost,
and it is a PLAN rule 10 meter, so it should land before B's first checkpoint.

| Open hold / obligation | Resume or resolve when | Must be resolved before |
|---|---|---|
| KRPPKRP 7-man truth gap | Independent truth becomes available, or C.5.8 records an explicit exclusion | C.5.8 closes |
| KRP-KB win-preserving 0.9990 → 0.9949 (−2.2 SE, RAR-M42) | Non-blocking; blocking if a later change pushes it past 3 SE | C.5.4 closes (owner) |
| Harness null calibration owed since adjudication removal (RAR-M17, 2026-09-01; none recorded) | One maintainer-run `-Mode calibrate` pair of the B.1 head against itself, recorded as a RAR-M row | B.2.4's SPRT starts |

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

- [ ] **B.0** Investigation: current search vs donor, cluster boundaries, scale ratio, seeds, instruments — **RESEARCH / R3**
- [ ] **B.1** Search restructure, behaviour-neutral: modules, `NodeType`, `StackEntry`; A.6 dead code removed; exact fingerprint — **RESEARCH / I1**
- [ ] **B.2** Cluster 1 — selectivity core: TT eval storage, correction, histories, picker, move-loop pruning, LMR — **RESEARCH / I2**
    - [ ] **B.2.1** Implement to the B.0 handoff with table, picker, TT and unwind tests — **RESEARCH / I2**
    - [ ] **B.2.2** Diagnostics: oracle differential, depth at 300k, EBF, reference-anchored branching curve, tactical suite, 2,000-game unfitted run; screen thresholds frozen at registration — **RESEARCH / V**
    - [ ] **B.2.3** SPSA over the registered live coordinates — **RESEARCH / V**
    - [ ] **B.2.4** Gate: SPRT `[0,10]` against the B.1 head, after the owed null calibration; ledger row and calibration — **RESEARCH / V**
- [ ] **B.3** Cluster 2 — NMP, ProbCut, singular/multi-cut/negative/LDSE extensions, IIR policy; SPRT `[0,5]` — **RESEARCH / I2**
- [ ] **B.4** Cluster 3 — quiescence: TT, corrected stand-pat, LMP, SEE margin; first-ply check generation measured against B.2's mate-threat canaries; SPRT `[0,3]` — **RESEARCH / I2**
- [ ] **B.5** Cluster 4 — root, aspiration, iterative deepening, PV/multi-PV; SPRT `[0,3]` — **RESEARCH / I2**
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
- [ ] **D.3** Engine lifecycle and protocol robustness; `src/uci/` move; zero crashes over pool tournaments — **R2**
- [ ] **D.4** Tablebase policy: probing depth/limits, WDL/DTZ in conversion, recogniser interaction — **R2**

## Phase E — Classical checkpoint and release

- [ ] **E.1** Attribution checkpoint: STC, `10+0.1`, 4T against 2.3.2 and the B.9/C.11 heads; maturity checklist — **V**
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
