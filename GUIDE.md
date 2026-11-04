# Rarog development guide

<!-- board: generated from docs/PLAN.md by `python tools/diag/guide_board.py`; edit PLAN, never this block -->
**Now: E.3.2** (`I1`): Build names: one scheme for `id name`, the start-up line, the files and the build command, and an `sse42` tier.

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

## Phase C — Evaluation programme (search frozen) — CLOSED 2026-10-22

The classical evaluation was rebuilt on Stockfish 11's model in fitted
clusters (a refit, king safety, threats, pawns and passers, endgame rules,
two refit cycles), the engine made 29% faster in bench and its search
re-tuned on the new evaluation: 82 ± 25 of the 182-Elo equal-node
evaluation deficit closed, and +142 ± 25 Elo at 1T and +94 ± 26 at 4T over
the B.9 head against the pool. The evaluation is frozen at engine source
`9fb525e` (13,987,223 / EBF 2.565); the C head rates 3,479 in a wider
field, about 3,475 on CCRL's Blitz scale.

## Phase D — Clock, threads, robustness

- [x] **D.1** Time management: audit against the ADR-0065 checklist, soft/hard bounds with node-fraction multiplier, forfeit margin; SPRT `[0,3]` — DONE 2026-10-26
- [x] **D.2** Lazy SMP quality at 4T/8T: diversity, shared TT and correction, soft-stop voting; 4T SPRT `[0,5]`; RAR-M78's deficit verdict was measured in a packed slot and is superseded (RAR-M82), the sibling-free comparison is RAR-M83 — DONE 2026-11-04
- [x] **D.3** Engine lifecycle and protocol robustness; `src/uci/` (planned) move; score normalisation research card (`analysis/uci_info_review_2026-09-16.md` item 6); zero crashes over pool tournaments — DONE 2026-11-04
- [ ] **D.4** Instruments from the Triumviratus review: the measurement-integrity reads, a search-path perft, pruning and reduction pricing, the loss table by cell, an identical-tree counter read — **R2**
    - [x] **D.4.1** Wall-clock overhead read — DONE 2026-11-03
    - [x] **D.4.2** SPSA schedule read — DONE 2026-11-03
    - [ ] **D.4.3** A search-path perft — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **D.4.4** Pruning and reduction pricing — **RESEARCH / R2**
    - [ ] **D.4.5** The loss table by cell — **READY_FOR_IMPLEMENTATION / V**
    - [ ] **D.4.6** An identical-tree counter read — **RESEARCH / R2**
- [ ] **D.5** Behaviour-neutral repairs from the engine review: the table handle and the helper lifecycle, the fixed-depth vote, the `Hash` notice, dead code, test and CI gaps — **I1**
    - [ ] **D.5.1** The table handle — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **D.5.2** A `Hash` change at `Threads > 1` fills the table twice — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **D.5.3** The fixed-depth vote — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **D.5.4** Helper start-up — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **D.5.5** The `Hash` rounding notice — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **D.5.6** The dead quiescence type parameter — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **D.5.7** The root `searchmoves` filter — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **D.5.8** Dead Syzygy code — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **D.5.9** The slider-table `unsafe` and the timer — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **D.5.10** Stack-built tables — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **D.5.11** The Fathom FFI layout guard — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **D.5.12** The PEXT tier in CI — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **D.5.13** The `texel` and `tune` evaluation tests in CI — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **D.5.14** The KPK bitbase at compile time — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **D.5.15** `Move::from_uci` into `make_move` — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **D.5.16** Names and comments that mislead — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **D.5.17** Tests the review found missing — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **D.5.18** A record corrected — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **D.5.19** The KPK evaluation and every hard draw verdict against Syzygy — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **D.5.20** CI hygiene — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **D.5.21** Basilisk's behaviour tests run against Rarog — **READY_FOR_IMPLEMENTATION / I1**
    - [x] **D.5.22** Measurement and gating methods from Basilisk — DONE 2026-11-02
    - [ ] **D.5.23** The correction bonus overflows `i32` at extreme depth and residual — **READY_FOR_IMPLEMENTATION / I1**
    - [x] **D.5.24** A `Hash` larger than the machine's physical memory is refused on every platform — DONE 2026-11-04
- [ ] **D.6** Strength candidates from the engine review: kernel and evaluation deviations from the donor, each behind a zero-game screen and a registered gate — **R2**
    - [ ] **D.6.1** Castling is neither quiet nor noisy — **RESEARCH / R2**
    - [ ] **D.6.2** Quiescence cannot see a non-capturing queen promotion — **RESEARCH / R2**
    - [ ] **D.6.3** Null-move and ProbCut children typed `cut_node = true` — **RESEARCH / R2**
    - [ ] **D.6.4** A fail-low node stores its least-bad move as the TT move — **RESEARCH / R2**
    - [ ] **D.6.5** The exclusion search uses the table-refined estimate — **RESEARCH / R2**
    - [ ] **D.6.6** Captures generated and scored before the TT move — **RESEARCH / R2**
    - [ ] **D.6.7** The ply cap — **RESEARCH / R2**
    - [ ] **D.6.8** A depth-0 entry is counted as two nodes — **RESEARCH / R2**
    - [ ] **D.6.9** The endgame scale's target — **RESEARCH / R2**
    - [ ] **D.6.10** The pawnless default factor — **RESEARCH / R2**
    - [ ] **D.6.11** Piece-term definitions no experiment read — **RESEARCH / R2**
    - [ ] **D.6.12** `Hash` at any size — **RESEARCH / R2**
    - [ ] **D.6.13** The aspiration fail-low window — **RESEARCH / R2**
    - [ ] **D.6.14** Low-value notes — **RESEARCH / R2**
    - [x] **D.6.15** The correction is added to the evaluator's draw verdicts — DONE 2026-11-03
    - [ ] **D.6.16** Quiescence prunes by gain against a decisive material edge — **RESEARCH / R2**
    - [ ] **D.6.17** Underpromotions and Saavedra's study — **RESEARCH / R2**
    - [ ] **D.6.18** Stockfish 19 mechanisms Basilisk adopted that Rarog lacks — **RESEARCH / R2**
    - [ ] **D.6.19** Root and in-check reductions never read apart — **RESEARCH / R2**
    - [ ] **D.6.20** The fail-high stop's hole — **RESEARCH / R2**
    - [ ] **D.6.21** Won KPK positions read at or below 0 — **RESEARCH / R2**
    - [ ] **D.6.22** `krpkr_scale`'s hard zeros on won positions — **RESEARCH / R2**
    - [ ] **D.6.23** Wrong-bishop draws still misread through leaves the recogniser does not cover — **RESEARCH / R2**
- [ ] **D.7** Large pages on Linux: transparent huge pages for the big tables — **READY_FOR_IMPLEMENTATION / I1**
- [ ] **D.8** Simplification programme from the engine review: tests into child files, the instrumentation behind probes, one score module, the board split, visibility, one list and one handle; about 860 source lines fewer and the largest file halved — **I1**
    - [ ] **D.8.1** P0: the census as a tool — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **D.8.2** P1: in-file tests into child files — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **D.8.3** P2: one `score` module — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **D.8.4** P3: geometry into one board module — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **D.8.5** P4: split `board/board.rs` — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **D.8.6** P5: delete `trace_decision!` — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **D.8.7** P6: the `diag` blocks into probe functions — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **D.8.8** P7: visibility follows use, second pass — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **D.8.9** P8: delete the family masks — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **D.8.10** P9: one `ArrayList<T, N>` — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **D.8.11** P10: one `Compiled<T>` — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **D.8.12** P11: split `search/mod.rs`, after D.2 — **READY_FOR_IMPLEMENTATION / I1**
    - [x] **D.8.13** P12: the adapter boundary, inside D.3 — DONE 2026-11-04
    - [ ] **D.8.14** P13: one iterative-deepening driver — **RESEARCH / R2**
    - [ ] **D.8.15** P14: bench and WAC under one suite runner — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **D.8.16** P15: the evaluation's shape — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **D.8.17** P16: comments — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **D.8.18** P17: lints — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **D.8.19** P18: the node frame — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **D.8.20** P19: evaluation terms computed for a constant — **READY_FOR_IMPLEMENTATION / I1**
- [ ] **D.9** Tablebase policy: probing depth/limits, WDL/DTZ in conversion, recogniser interaction — **RESEARCH / R2**

## Phase E — Classical checkpoint and release

- [ ] **E.1** Attribution checkpoint: B.2.0 review re-run on the B.9/C.11 heads; STC, `10+0.1`, 4T against 2.3.2 and the B.9/C.11 heads; maturity checklist — **V**
- [ ] **E.2** Target gate: ≥50% against Critter 1.6a, Houdini 3, Rybka 4.1 and Fritz 16 at 1T and 4T (Rybka 4.1 replaced Rybka 4, 2026-09-19); the binding arm is 1T — **V**
- [ ] **E.3** Release 2.7.0 through the tag-driven flow; 3.0.0 is reserved for the first NNUE release — **M**
    - [ ] **E.3.0** Originality disclosure before the release cut: the donor material named, the licence settled, the notices shipped with the binaries — **R2**
    - [x] **E.3.1** Tag-driven release flow — DONE 2026-10-05
    - [ ] **E.3.2** Build names: one scheme for `id name`, the start-up line, the files and the build command, and an `sse42` tier — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **E.3.3** The universal x86-64 binary: one release file per OS that picks its tier at start-up, at the single-tier build's speed — **RESEARCH / R2**
    - [ ] **E.3.4** Release cut: version 2.7.0, the `[Unreleased]` changelog reviewed and dated, suites, the release PR squash-merged, the `v` tag pushed on instruction through E.3.1's workflow — **M**
- [ ] **E.4** The king-danger index rebuilt in the donor's structure; the classical line's next release if it wins — **R2**

## Phase F — NNUE (own data only)

- [ ] **F.0** Investigation: board events, accumulator ownership, trainer choice, data format, first architecture — **R3**
- [ ] **F.1** Board events and accumulator scaffolding, behaviour-neutral for HCE; cost ledger — **I2**
- [ ] **F.2** Data generation at scale: 30–60M unique positions, splits, manifests, hashes — **V**
    - [ ] **F.2.1** Classical refit on search-visited data: the frozen HCE's whole surface refitted on F.2's dataset, gated as an evaluation change; the classical line's release if it wins — **V**
- [ ] **F.3** Trainer hardening and baseline nets, two seeds per configuration — **I2**
- [ ] **F.4** Scalar integration: `quantised.bin` contract, integer-exact conformance, HCE fallback — **I2**
- [ ] **F.5** Incremental and SIMD: same-net parity on every move type, tiers, pooled NPS attribution — **I2**
- [ ] **F.6** Search re-fit for the network — **V**
- [ ] **F.7** Architecture ladder: output buckets, king buckets, relation/threat inputs; one axis at a time — **R3**
- [ ] **F.8** Data frontier: on-policy refresh, deduplication, hard-position mining — **V**
- [ ] **F.9** NNUE release, 3.0.0: beat the classical release at STC, LTC and 4T; platform matrix — **M**
- [ ] **F.10** CCRL top-100 gate — **V**

## Phase G — Scaling, platforms and the top 50

- [ ] **G.1** High-thread and NUMA: 8/16/32T, TT and net placement, large pages, affinity policy — **R2**
- [ ] **G.2** Platform and product: Chess960 on demand and distributed testing — **I1**
- [ ] **G.3** Frontier: larger nets, data scaling, LTC search fit; CCRL top-50 gate — **R3**

## Phase H — Refinements after the big gains

- [ ] **H.1** Triage: every deferred small gain re-ranked on the network head — **R2**
- [ ] **H.2** Search refinements — **R2**
- [ ] **H.3** Time-management refinements — **R2**
- [ ] **H.4** Throughput at an identical tree on the network head — **R2**
<!-- end of generated board -->

## Current checkpoint

| Item | Value |
|---|---|
| Next release | **2.7.0** at E.3 after Phases D and E, cut only after **E.3.0**'s originality disclosure (maintainer decision 2026-10-23; `analysis/originality_review_2.6.0_2026-10-22.md`) (maintainer decision 2026-10-22: 3.0.0 is reserved for the first NNUE release, F.9; E.2's four targets already pass at 1T and 4T, RAR-M73, RAR-M74, and its result is recorded, not a version). Later classical gains (E.4, F.2.1) release as 2.8.0 onward under Phase F's rule |
| Released baseline | **2.6.0** on `master`, released 2026-10-22 from the `v2.6.0` tag; fingerprint **13,987,223 / EBF 2.565**. Over 2.5.0: +153.4 ± 10.7 at `3+0.03` 1T, +149.3 ± 15.1 at `10+0.1`, +149.7 ± 14.7 at 4T (RAR-M76). 2.5.0 (2026-10-05, 11,171,726 / EBF 2.512) was +272.4 over 2.4.0 at `3+0.03` (RAR-M64) |
| Development head | `d4` from `master` at `c69761d` (D.2 and D.3, "Harden the UCI protocol and keep draw verdicts", PR #13, 2026-11-04; D.1 is `703fe78`, PR #12; the 2.6.0 release is `0127e58`, PR #11), version **2.7.0-dev**: the 2.5.0 search with Phase C's evaluation, C.12's speed pass and large pages, C.10's search tune and D.1's clock; D.3's protocol and shell repairs, D.6.15's draw verdicts and D.5.24's physical-memory `Hash` bound; one PGO-trained kernel for every thread (`e057d84`); the classical evaluation frozen at engine source `9fb525e`; fingerprint **14,749,043 / EBF 2.564** since D.6.15 (`3a16fbb`; 13,987,223 / EBF 2.565 before); `rustc 1.99.1` (RAR-P36–P38, kept by maintainer decision). A registration names the revision it builds from and this fingerprint. Change records: `docs/HISTORY.md`, `docs/archive/PLAN-phaseD-done-2026-11-04.md`, `docs/archive/PLAN-closed-2026-10-22.md` |
| Pool position, `3+0.03` 1T | **3,428.8 ± 19.2** (RAR-M73, the C head, 2026-10-21), **+142.3 ± 24.8 over the B.9 head's 3,286.5 ± 15.7** (RAR-M64; 2.4.0 rates 3,001): Houdini 3 +172.8, Critter 1.6a +231.5, Fritz 16 +256.2, Rybka 4.1 +330.2, so all four E.2 targets pass at 1T; Rarog 2.4.0 +363.2 |
| Pool position, `3+0.03` 4T | **3,433.7 ± 19.3** (RAR-M74, the C head, 2026-10-21), **+93.7 ± 25.5 over the B.9 head's 3,340.0 ± 16.7** (RAR-M66): Houdini 3 +171.7, Critter 1.6a +242.2, Fritz 16 +249.1, Rybka 4.1 +350.3, all four E.2 targets pass at 4T. **Thread scaling fell:** in games the C head is 9.2% slower than the B.9 head at 4T while 29.8% faster at 1T (4T/1T 2.60 against 3.72; Colosseum CLI 0.4.0's figures), and its 4T performance is only +4.9 over its 1T rating (B.9 +53.5); held for D.2 |
| Pool position, wider field, `3+0.03` 1T | **3,478.9 ± 15.1** (RAR-M75, the C head, 2026-10-22): fourth in a round robin of nine with Houdini 3 and Critter held; Dragon 1 3,628.7, Stockfish 11 3,568.2, Houdini 6.03 3,498.5 (head −9 ± 32), Komodo 13.3 3,416.6 (head +53), Komodo 10.4 3,387.4, Houdini 4 3,303.2 (held 3,310). **CCRL-equivalent:** Blitz 1CPU **3,475 ± 82** (the held scale near 1:1 with it here; about 70 below CCRL Blitz's top-100 cut), 40/15 1CPU 3,407 ± 78 |
| Search deficit | Closed: G(0) **+24.24 ± 8.01** against the oracle (RAR-O04), from −247.97 at 2.4.0 (RAR-O03) |
| Evaluation deficit | **+99.2 ± 16.6 Elo at equal nodes** (RAR-O07, 2026-10-21): Stockfish's classical HCE over Rarog's frozen evaluation inside Stockfish's search, from +181.7 ± 19.0 at Phase C's start (RAR-O05), 82.5 ± 25.2 closed. The donor's remaining king signal is the index's structure (RAR-E72, RAR-E73); Phase F supersedes the classical evaluation, and an evaluation change under a fitted search follows PROCESS |
| Speed | The C head (2026-10-21/22): **+28.92%** pooled-PGO bench NPS over the B.9 head (RAR-P39; `rustc 1.99.1`'s −1.63% included, RAR-P37, its code generation −3.6% without PGO, RAR-P38); in games (Colosseum CLI 0.4.0) +29.8% at 1T and −9.2% at 4T over the B.9 head (RAR-M73, RAR-M74), and −8.6% at 1T and −19.1% at 4T against Rarog 2.4.0 in the same games; the evaluation 21.1% of search samples, B.9's 24.3% (RAR-P40) |
| Conversion | **13.8 draws and 0.0 losses per 1,000 games** after a persistent piece-up (RAR-M73's games, the C head; Poisson 95% 9.5–19.3 and 0–1.5), against the B.9 head's 15.8 and 0.8 (RAR-M64) and 2.4.0's 24.4 and 5.3: the draw rate's fall is not resolved |
| Active experiment | None running. **Landed 2026-11-04 as `c69761d` (PR #13):** D.2 closed `NO_CHANGE` (RAR-M83: four threads are worth 1.37 doublings of Rarog's own time against Reckless's 1.49 and Stockfish's 1.54; one logical CPU per physical core is the placement for every rated multi-thread read); D.3 done, with the protocol behind `src/uci/`; D.5.24's `Hash` bound and D.6.15's draw verdicts. Triumviratus is registered as a mechanisms-only donor and its small gains wait in Phase H. Next: E.3.2, the build names, then E.3.3, the universal binary (maintainer decision 2026-11-04: settled before Phase D continues); then D.4, the instruments (D.4.3, the search-path perft, first), then D.5's repairs and D.6's candidates. Finished steps' records: PLAN and `docs/archive/PLAN-phaseD-done-2026-11-04.md` |

## Holds and obligations

| Open hold / obligation | Resume or resolve when | Must be resolved before |
|---|---|---|
| Originality disclosure (2026-10-23 review of 2.6.0): Stockfish's time-budget constants and Reckless's move-ordering and history constants were copied unchanged, the evaluation design is Stockfish 11's, Fathom is not named, and the README credits only "inspiration". The code and the evaluation weights are Rarog's own, and the move-similarity screen reads Rarog like an unrelated engine against both donors (56.4% Stockfish 11, 54.2% Reckless; related controls 67–69%) | **E.3.0**: rerun the scans on the release head, the maintainer settles the licence (Reckless is AGPL-3.0), then README, PROCESS's boundary and the binaries' notices | E.3.4, the 2.7.0 release cut |

Follow the earliest unblocked leaf in PLAN's register order, which the board's Now shows; the register is in ascending order, which is the order of work (maintainer decision 2026-11-03). Held items stay unticked in place.

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

## About this guide

The board at the top is generated from `docs/PLAN.md` by
`python tools/diag/guide_board.py`: edit PLAN's step heads (title, class,
DONE) and its workflow register, then regenerate; never edit the board by
hand. A closed phase shows its one- or two-sentence summary; its steps are
in `docs/archive/`. Everything below the board is kept by hand.
`python tools/diag/check_guide.py` must pass; it fails on a stale board.
