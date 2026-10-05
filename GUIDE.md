# Rarog development guide

**Now: C.0** (`R3`), the evaluation programme's investigation on the frozen
search head. **Next release:** 3.0.0 at E.3 if E.2 is met, otherwise 2.6.0.

The board lists every phase; a closed phase is a short summary whose steps
PLAN and HISTORY keep. Below the board: the checkpoint, holds, model
mapping, prompts and how to work with the agent. GUIDE and PLAN change
together, and `python tools/diag/check_guide.py` must pass.

## Phase A — Reset: repository, instruments, baselines, consolidation release — CLOSED 2026-09-11

Documents, repository, toolchain (`rustc 1.98.1`) and CPU-tier assets were
reset, the conversion instrument built and the 2.4.0 baselines measured
(pool, 4T gauntlet, oracle deficit −248 Elo, 3.19 MNPS); 2.4.0 was released
at +54.8 ± 17.0 Elo over 2.3.2.

## Phase B — Search programme (evaluation frozen) — CLOSED 2026-10-05

The search was rebuilt and fitted in gated clusters (selectivity core;
null-move, ProbCut and singular extensions; quiescence; tablebase root; a
speed pass and a cleanup), turning the oracle deficit from −248 into
+24 ± 8 Elo. It was released as 2.5.0, +272 Elo over 2.4.0 at `3+0.03`,
and is frozen at `ee02ed1` for Phase C.

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
- [ ] **C.10** Joint search SPSA after the new evaluation: cp margins plus every mechanism whose firing rate moved 10% or more (the whole-surface tune B.6 left for here); rule-7c blocks; SPRT `[0,3]` — **V**
- [ ] **C.11** Checkpoint: same-search deficit, conversion, NPS, pool gauntlet; freeze the classical evaluation — **V**

## Phase D — Clock, threads, robustness

- [ ] **D.1** Time management: audit against the ADR-0065 checklist, soft/hard bounds with node-fraction multiplier, forfeit margin; SPRT `[0,3]` — **R2**
- [ ] **D.2** Lazy SMP quality at 4T/8T: diversity, shared TT and correction, soft-stop voting; 4T SPRT `[0,5]`; its premise is contradicted by RAR-M46, so re-scope first — **R2**
- [ ] **D.3** Engine lifecycle and protocol robustness; `src/uci/` (planned) move; score normalisation research card (`analysis/uci_info_review_2026-09-16.md` item 6); zero crashes over pool tournaments — **R2**
- [ ] **D.4** Tablebase policy: probing depth/limits, WDL/DTZ in conversion, recogniser interaction — **R2**

## Phase E — Classical checkpoint and release

- [ ] **E.1** Attribution checkpoint: B.2.0 review re-run on the B.9/C.11 heads; STC, `10+0.1`, 4T against 2.3.2 and the B.9/C.11 heads; maturity checklist — **V**
- [ ] **E.2** Target gate: ≥50% against Critter 1.6a, Houdini 3, Rybka 4.1 and Fritz 16 at 1T and 4T (Rybka 4.1 replaced Rybka 4, 2026-09-19); the binding arm is 1T — **V**
- [ ] **E.3** Release 3.0.0 (E.2 met) or 2.6.0 through the tag-driven flow — **M**
    - [x] **E.3.1** Tag-driven release flow (`release.yml`, `cargo xtask release-check`); 2.5.0 was released through it — DONE 2026-10-05
    - [ ] **E.3.2** Release cut: version 3.0.0 (E.2 met) or 2.6.0, the `[Unreleased]` changelog reviewed and dated, suites, the PR merged with a merge commit, the `v` tag pushed on instruction through E.3.1's workflow — **M**

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

## Current checkpoint

| Item | Value |
|---|---|
| Released baseline | **2.5.0** on `master`, released 2026-10-05 from the `v2.5.0` tag; fingerprint **11,171,726 / EBF 2.512**. Over 2.4.0: +272.4 at `3+0.03` 1T, +260.5 ± 16.0 at `10+0.1`, +322.7 at 4T (RAR-M64, RAR-M65, RAR-M66) |
| Development head | `dev`, version **2.6.0-dev** (`d6998db`), the 2.5.0 engine; fingerprint **11,171,726 / EBF 2.512**; `rustc 1.98.1`. The search is frozen for Phase C at engine source `ee02ed1` (the measured binary and its SHA-256 are in PLAN B.9): a C-phase change touching `src/search/` returns to its owner leaf and is gated as a search change, and search coordinates move only in C.10's joint tune |
| Pool position, `3+0.03` 1T | **3,286.5 ± 15.7** (RAR-M64; 2.4.0 rates 3,001): Houdini 3 +37.5, Critter 1.6a +96.2, Fritz 16 +87.8, Rybka 4.1 +167.3, so all four E.2 targets pass at 1T |
| Pool position, `3+0.03` 4T | **3,340.0 ± 16.7** (RAR-M66): Houdini 3 +113, Critter 1.6a +148, Fritz 16 +132, Rybka 4.1 +210 |
| Search deficit | Closed: G(0) **+24.24 ± 8.01** against the oracle (RAR-O04), from −247.97 at 2.4.0 (RAR-O03) |
| Evaluation deficit | **About 329 Elo** against Stockfish's classical HCE with the same search (RAR-O02): Phase C's target |
| Speed | **2.33 MNPS**, −26.23% against 2.4.0 (RAR-P35); the host drifts between days, so compare pools interleaved only |
| Conversion | **15.8 draws and 0.8 losses per 1,000 games** after a persistent piece-up (RAR-M64's games; 2.4.0: 24.4 and 5.3) |
| Active experiment | None open |

## Holds and obligations

| Open hold / obligation | Resume or resolve when | Must be resolved before |
|---|---|---|
| `archive/pr2`, `archive/pr3`, `archive/pr4` tags (PLAN B.10, *Ref review after the release*) | The next PR has merged with a merge commit and `git for-each-ref --contains` lists `master` for each tip; the maintainer then deletes them | Phase C's ref review at C.11 |
| KRPPKRP 7-man truth gap | Independent truth becomes available, or C.5.8 records an explicit exclusion | C.5.8 closes |
| KRP-KB win-preserving 0.9990 → 0.9949 (−2.2 SE, RAR-M42) | Non-blocking; blocking if a later change pushes it past 3 SE | C.5.4 closes (owner) |

Follow the earliest unblocked leaf. Held items stay unticked in place.

## Model mapping

PLAN records only stable capability classes. Edit this table when model
generations change; do not rewrite the roadmap. These are maintainer
judgments, not measured rankings. An investigation leaf (`R3`) spawns the
implementation and measurement sub-steps under its step.

| Class | Capability | Model — thinking mode |
|---|---|---|
| `R3` | Frontier causal/architecture research | Claude Fable 5.1 — High |
| `R2` | Bounded correctness-sensitive reasoning | Claude Opus 5 — High |
| `I2` | Difficult implementation | Claude Opus 5 — High |
| `I1` | Well-specified implementation | Claude Sonnet 5 — Medium |
| `M` | Mechanical/docs/provenance | Claude Sonnet 5 — Medium |
| `V` | Verification/measurement | Claude Sonnet 5 — High |

Claude models only, by maintainer decision 2026-09-25.

## Reusable research prompt

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

## Reusable implementation prompt

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
