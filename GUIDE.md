# Rarog development guide

<!-- board: generated from docs/PLAN.md by `python tools/diag/guide_board.py`; edit PLAN, never this block -->
**Now: E.4.0** (`R3`): The donor and the form.

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
- [x] **D.4** Instruments from the Triumviratus review: the measurement-integrity reads, a search-path perft, pruning and reduction pricing, the loss table by cell, an identical-tree counter read — DONE 2026-11-06
- [x] **D.5** Behaviour-neutral repairs from the engine review: the table handle and the helper lifecycle, the fixed-depth vote, `Hash` at any size, dead code, test and CI gaps — DONE 2026-11-08
- [x] **D.6** Strength candidates from the engine review: kernel and evaluation deviations from the donor, each behind a zero-game screen and a registered gate — DONE 2026-11-23
    - [x] **D.6.1** Castling is neither quiet nor noisy — NO_CHANGE 2026-11-22
    - [x] **D.6.2** Quiescence cannot see a non-capturing queen promotion — NO_CHANGE 2026-11-22
    - [x] **D.6.3** Null-move and ProbCut children typed `cut_node = true` — NO_CHANGE 2026-11-22
    - [x] **D.6.4** A fail-low node stores its least-bad move as the TT move — NO_CHANGE 2026-11-22
    - [x] **D.6.5** The exclusion search uses the table-refined estimate — NO_CHANGE 2026-11-22
    - [x] **D.6.6** Captures generated and scored before the TT move — NO_CHANGE 2026-11-22
    - [x] **D.6.7** The ply cap — DONE 2026-11-22
    - [x] **D.6.8** A depth-0 entry is counted as two nodes — NO_CHANGE 2026-11-22
    - [x] **D.6.9** The endgame scale's target — CLOSED 2026-11-22
    - [x] **D.6.10** The pawnless default factor — CLOSED 2026-11-22
    - [x] **D.6.13** The aspiration fail-low window — NO_CHANGE 2026-11-22
    - [x] **D.6.15** The correction is added to the evaluator's draw verdicts — NO_CHANGE 2026-11-23
    - [x] **D.6.16** Quiescence prunes by gain against a decisive material edge — NO_CHANGE 2026-11-23
    - [x] **D.6.17** Underpromotions and Saavedra's study — NO_CHANGE 2026-11-23
    - [x] **D.6.20** The fail-high stop's hole — NO_CHANGE 2026-11-23
    - [x] **D.6.21** Won KPK positions read at or below 0 — CLOSED 2026-11-22
    - [x] **D.6.22** `krpkr_scale`'s hard zeros on won positions — CLOSED 2026-11-22
    - [x] **D.6.23** Wrong-bishop draws still misread through leaves the recogniser does not cover — CLOSED 2026-11-22
    - [x] **D.6.24** The KQKP fortress's hard zeros on won positions — CLOSED 2026-11-22
    - [x] **D.6.25** Two same-coloured bishops outscore a queen against a bare king — CLOSED 2026-11-22
    - [x] **D.6.26** Pawnless endings a minor ahead read as wins — CLOSED 2026-11-22
    - [x] **D.6.27** IIR has no node-type condition — NO_CHANGE 2026-11-23
    - [x] **D.6.29** Quiescence never scores a stalemate — DONE 2026-11-23
    - [x] **D.6.30** A later iteration can overwrite a proven mate — NO_CHANGE 2026-11-23
    - [x] **D.6.31** A null move adds a node to the count — NO_CHANGE 2026-11-23
    - [x] **D.6.32** The shortest mate's depth moves with the seeds — NO_CHANGE 2026-11-23
- [x] **D.7** Search shape: the kernel against its donor on Rarog's evaluation, rebuilt whole as one compact kernel if the reads or the maintainer say so — DONE 2026-11-21
    - [x] **D.7.0** Investigation: the inventory of departures, the seeds and the registration of the reads — DONE 2026-11-13
    - [x] **D.7.1** The fixed-match script takes a node budget and a move time per engine — DONE 2026-11-13
        - [x] **D.7.1.1** The Colosseum wrapper takes a move time, a node budget per engine and a foreign arm — DONE 2026-11-14
    - [x] **D.7.2** RAR-O09: the donor's search against the head at equal work, at equal move time and on the clock at two controls — DONE 2026-11-14
        - [x] **D.7.2.1** RAR-O10: Rarog's and Basilisk's evaluations and moves against a reference — DONE 2026-11-14
        - [x] **D.7.2.2** RAR-O11: the start points, per-node quality against speed, and evaluation cost — DONE 2026-11-15
        - [x] **D.7.2.3** The two search programmes compared from their ledgers — DONE 2026-11-16
        - [x] **D.7.2.4** RAR-O12: RAR-O11's equal-node reads again, at equal work in one node unit — DONE 2026-11-16
    - [x] **D.7.3** Parity instrument: Rarog and the hybrid on one position set — DONE 2026-11-16
    - [x] **D.7.4** The kernel in the donor's form, behind one switch — DONE 2026-11-18
        - [x] **D.7.4.1** The constants folded into Rarog's units and the scaffold deleted — DONE 2026-11-18
    - [x] **D.7.5** The unfitted paired run — DONE 2026-11-18
    - [x] **D.7.6** The fit — DONE 2026-11-19
        - [x] **D.7.6.1** The compact kernel's seeds exposed to a tune build — DONE 2026-11-18
        - [x] **D.7.6.2** The centres at 2,000 baked in as the kernel's seeds — DONE 2026-11-19
    - [x] **D.7.7** The classic kernel deleted — DONE 2026-11-19
    - [x] **D.7.8** Repairs from the audit — DONE 2026-11-20
        - [x] **D.7.8.1** ProbCut's exchange bar in exchange units — DONE 2026-11-19
        - [x] **D.7.8.2** The table's depth for quiescence and evaluation-only entries — DONE 2026-11-19
        - [x] **D.7.8.3** The root's moves ordered by the kernel's own scorer — DONE 2026-11-19
        - [x] **D.7.8.4** Bound lines in short searches — DONE 2026-11-19
        - [x] **D.7.8.5** The interrupted iteration — NO_CHANGE 2026-11-19
        - [x] **D.7.8.6** A helper's previous score at a new game — DONE 2026-11-19
        - [x] **D.7.8.8** Position 305's second divergence from the donor — DONE 2026-11-19
        - [x] **D.7.8.9** The speed register — DONE 2026-11-20
        - [x] **D.7.8.10** A finer kernel unit: the screen — NO_CHANGE 2026-11-20
        - [x] **D.7.8.11** The root's remaining moves re-scored as each is searched — NO_CHANGE 2026-11-20
        - [x] **D.7.8.12** The speed pass before the fit — DONE 2026-11-20
    - [x] **D.7.9** Clock use on the new kernel — DONE 2026-11-20
        - [x] **D.7.9.1** Reckless's time manager, whole — DONE 2026-11-20
    - [x] **D.7.10** The pre-fit read: the repaired kernel against the classic — DONE 2026-11-20
    - [x] **D.7.11** The finishing fit — DONE 2026-11-21
        - [x] **D.7.11.1** The tune build — DONE 2026-11-20
        - [x] **D.7.11.2** The surface files and the activation read — DONE 2026-11-20
        - [x] **D.7.11.3** The registration — DONE 2026-11-20
        - [x] **D.7.11.4** The drivers and their dry runs — DONE 2026-11-20
        - [x] **D.7.11.5** The pilot — DONE 2026-11-20
        - [x] **D.7.11.6** The run — DONE 2026-11-21
        - [x] **D.7.11.7** The read at the stop — DONE 2026-11-21
        - [x] **D.7.11.8** The theta baked — DONE 2026-11-21
    - [x] **D.7.12** The reported score calibrated — DONE 2026-11-21
    - [x] **D.7.13** The gate: the kernel against the classic — DONE 2026-11-21
    - [x] **D.7.14** Adoption and checkpoint — DONE 2026-11-21
    - [x] **D.7.15** If the kernel does not gain — CLOSED 2026-11-21
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
    - [ ] **D.8.12** P11: split `search/mod.rs` — **READY_FOR_IMPLEMENTATION / I1**
    - [x] **D.8.13** P12: the adapter boundary, inside D.3 — DONE 2026-11-04
    - [ ] **D.8.14** P13: one iterative-deepening driver — **RESEARCH / R2**
    - [ ] **D.8.15** P14: bench and WAC under one suite runner — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **D.8.16** P15: the evaluation's shape — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **D.8.18** P17: lints — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **D.8.19** P18: the node frame — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **D.8.20** P19: evaluation terms computed for a constant — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **D.8.21** P20: the kernel's seeds no search reaches — **RESEARCH / V**
- [ ] **D.9** Tablebase policy: probing depth/limits, WDL/DTZ in conversion, recogniser interaction — **RESEARCH / R2**
- [ ] **D.10** Large pages on Linux: transparent huge pages for the big tables — **READY_FOR_IMPLEMENTATION / I1**
- [ ] **D.11** Originality disclosure before the release cut: the donor material named, the licence settled, the notices shipped with the binaries — **R2**
    - [ ] **D.11.1** The donor inventory by licence, on the release head — **RESEARCH / R2**
    - [ ] **D.11.2** The licence decision — **RESEARCH / R2**
    - [ ] **D.11.3** The decision carried out — **RESEARCH / I1**
    - [ ] **D.11.4** The disclosure, on Coda's model — **RESEARCH / M**
- [ ] **D.12** Board review and rebuild: the contract audited, each component read against the donors' boards, rebuilt where another form measures better — **R2**
    - [ ] **D.12.0** Rarog's own board interactions, and whether Basilisk's leads are Rarog's best options — **RESEARCH / R3**
    - [ ] **D.12.1** Board contract audit and instruments — **RESEARCH / R2**
        - [ ] **D.12.1.1** Search replay with a verify mode and a stub control, and the engine screen — **RESEARCH / I2**
        - [ ] **D.12.1.2** Two counts on the search's trace: SEE asks and capture generations repeated at a node — **RESEARCH / V**
    - [ ] **D.12.2** Board review against Stockfish, Basilisk, Reckless and Triumviratus — **RESEARCH / R2**
        - [ ] **D.12.2.1** SEE's king-legality test from aligned sliders — **RESEARCH / R2**
        - [ ] **D.12.2.2** Unmake restoring the derived keys from its record — **RESEARCH / I1**
    - [ ] **D.12.3** Rebuild behind the board contract — **RESEARCH / I2**
    - [ ] **D.12.4** Board speed read and gate — **RESEARCH / V**
- [ ] **D.13** Correction tables shared between threads — **RESEARCH / R2**
- [ ] **D.14** The donor's thread vote on the new clock — **RESEARCH / R2**
- [ ] **D.15** Release packaging from Basilisk 1.11.0 — **I1**
    - [ ] **D.15.1** Each download an archive — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **D.15.2** Release notes as a summary and highlights — **READY_FOR_IMPLEMENTATION / I1**
- [x] **D.16** Speed read on game positions: the bench read checked against play, and one more speed pass on the compact kernel if they disagree — DONE 2026-11-24
    - [x] **D.16.1** The game-position speed read — DONE 2026-11-23
    - [x] **D.16.2** The bench read against play — DONE 2026-11-24
    - [x] **D.16.3** One more speed pass on the compact kernel — NO_CHANGE 2026-11-24
        - [x] **D.16.3.1** The kernel's node beside Stockfish's, function by function — DONE 2026-11-23
- [ ] **D.17** GitHub-hosted runners as a test lane for self-play SPRTs and tunes — **RESEARCH / R2**

## Phase E — Classical checkpoint and release

- [x] **E.0** Where the remaining strength is: speed, search, evaluation and how they fit each other, against the strongest hand-crafted engines — DONE 2026-11-23
- [ ] **E.1** Attribution checkpoint: B.2.0 review re-run on the B.9/C.11 heads; STC, `10+0.1`, 4T against 2.3.2 and the B.9/C.11 heads; maturity checklist — **V**
- [ ] **E.2** Target gate: ≥50% against Critter 1.6a, Houdini 3, Rybka 4.1 and Fritz 16 at 1T and 4T (Rybka 4.1 replaced Rybka 4, 2026-09-19); the binding arm is 1T — **V**
- [ ] **E.3** Release through the tag-driven flow; 3.0.0 is reserved for the first NNUE release — **M**
    - [x] **E.3.1** Tag-driven release flow — DONE 2026-10-05
    - [x] **E.3.2** Build names: one scheme for `id name`, the start-up line, the files and the build command, and an `sse42` tier — DONE 2026-11-04
    - [x] **E.3.3** The universal x86-64 binary: one release file per OS that picks its tier at start-up, at the single-tier build's speed — DONE 2026-11-04
    - [ ] **E.3.4** Release cut: the version, the `[Unreleased]` changelog reviewed and dated, suites, the release PR squash-merged, the `v` tag pushed on instruction through E.3.1's workflow — **M**
- [ ] **E.4** The hand-crafted evaluation in its donor's shape: the donor, a rebuild or repairs, endgame knowledge and the fit; the classical line's next release if it wins — **R3**
    - [ ] **E.4.0** The donor and the form — **RESEARCH / R3**
        - [x] **E.4.0.1** The donor's evaluation inside the kernel — DONE 2026-11-24
    - [ ] **E.4.1** The tuner before any fit — **I1**
    - [ ] **E.4.2** Endgame knowledge — **R2**
    - [ ] **E.4.3** The king-danger index rebuilt in the donor's structure — **R2**

## Phase F — NNUE (own data only)

- [ ] **F.0** Investigation: board events, accumulator ownership, trainer choice, data format, first architecture — **R3**
- [ ] **F.1** Board events and accumulator scaffolding, behaviour-neutral for HCE; cost ledger — **I2**
- [ ] **F.2** Scalar integration: `quantised.bin` contract, integer-exact conformance, HCE fallback — **I2**
- [ ] **F.3** Trainer hardening: a deterministic pipeline checked against a reference recipe on a pilot slice, and the learning curve that sizes F.4 — **I2**
- [ ] **F.4** Data generation at scale: the corpus F.3's learning curve sizes, splits, manifests, hashes — **V**
    - [ ] **F.4.1** Classical refit on search-visited data: the frozen HCE's whole surface refitted on F.4's dataset, gated as an evaluation change; the classical line's release if it wins — **V**
- [ ] **F.5** Incremental and SIMD: same-net parity on every move type, tiers, pooled NPS attribution — **I2**
- [ ] **F.6** The first nets and the search re-fit: baseline nets judged against one fixed anchor, then the search refitted to the network — **V**
- [ ] **F.7** Architecture ladder: output buckets, king buckets, relation/threat inputs; one axis at a time — **R3**
    - [ ] **F.7.1** Two-timescale evaluator: a slow network sets the fast evaluator's later-layer weights; an optional rung beside the ladder — **R3**
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
| Next release | Unscheduled. A kernel D.7 adopts releases as 2.8.0; later classical gains (E.4, F.4.1) release as 2.8.0 onward under Phase F's rule, and 3.0.0 is reserved for the first NNUE release, F.9 (maintainer decision 2026-10-22). It waits for the originality disclosure (**D.11**), whose hold was lifted for 2.7.0 only (maintainer decision 2026-11-13), and for **D.15**'s packaging from Basilisk 1.11.0: each download an archive carrying D.11's notices, and the release notes cut to a summary and highlights (added 2026-11-21) |
| Released baseline | **2.7.0** on `master`, released 2026-11-13 from the `v2.7.0` tag, an unscheduled release cut before D.7; fingerprint **9,077,016 / EBF 2.481**. Against 2.6.0 (RAR-M87): G −7.6 Elo [−14.4, −0.7] against a five-engine field, a loss by its rule, and +10.1 ± 8.7 head to head; shipped for its fixes by the maintainer's decision after the result. 2.6.0 (2026-10-22, 13,987,223 / EBF 2.565) was +153.4 ± 10.7 over 2.5.0 at `3+0.03` (RAR-M76); 2.5.0 (2026-10-05, 11,171,726 / EBF 2.512) +272.4 over 2.4.0 (RAR-M64) |
| Development head | `master` at `b862256` ("Raise the ply cap and score stalemates in quiescence", PR #21, 2026-11-23: `d6-review`, D.6's review on the kernel, squashed, its commits kept as the pull request's; before it `94d7f72`, "Rebuild the search as a compact kernel", PR #20, 2026-11-22), and `e0-research` from it for E.0: D.7.4's compact kernel, the only search since D.7.7 deleted the classic kernel (`d23b9cf`, 2026-11-19), with D.7.3's instrument; fingerprint **3,694,298 / EBF 2.324** on `master` since D.6.29 scored a stalemate in quiescence (`64993ff`, kept by RAR-S139 on the maintainer's decision; 3,374,901 / EBF 2.316 since D.6.7 raised the ply cap to 240, `91bc561`, kept by RAR-S138; 3,390,842 / EBF 2.317 since D.7.11.8 baked RAR-S135's centres at 2,000, `16f93cb`; 3,320,355 / 2.317 since D.7.8.3 ordered the root by the kernel's own scorer, `4271031`, RAR-S132 kept it; 3,348,508 / 2.329 since D.7.8.2 stored quiescence and evaluation-only entries at the donor's depth, `e363908`, RAR-S131 kept it; 3,278,226 / 2.313 since D.7.8.1 put ProbCut's exchange bar at the donor's scale, `e63b652`, RAR-S130; 3,488,485 / 2.336 since D.7.6.2 baked RAR-S128's centres at 2,000, `73966eb`; 7,543,485 / 2.422 since D.7.4.1's fold, `8489e46`; 7,606,256 / 2.433 at `a344748`); the classic kernel's last 9,067,595 / EBF 2.479 at `29f32fb`, archived as `d7classic`. Before it `basilisk-check` from `master` at `3c57a89` ("Release 2.7.0", PR #19, 2026-11-13, tagged `v2.7.0`; `638b56d`: the Rust 1.100.0 build, D.6.8, the SPSA rule and the 2.7.0 read, "Rust 1.100.0, count each position once and new SPSA tune", PR #18, 2026-11-13; `89b4a92`: D.6.1 to D.6.7, the donor-search reads and the roadmap rework, "Fix castling and queen promotions in the search and bound long lines", PR #17, 2026-11-13; D.5's behaviour-neutral repairs, `b202174`, "Repair the engine review's findings without changing play", PR #16, 2026-11-08; D.4's instruments and the roadmap's order, "Add the search instruments and put the roadmap in working order", PR #15, 2026-11-06; E.3.2's build names and E.3.3's universal binary are `5efe16f`, PR #14; D.2 and D.3, "Harden the UCI protocol and keep draw verdicts", PR #13, 2026-11-04; D.1 is `703fe78`, PR #12; the 2.6.0 release is `0127e58`, PR #11), version **2.8.0-dev** (a build reports `dev-<date>-<hash>`): the 2.5.0 search with Phase C's evaluation, C.12's speed pass and large pages, C.10's search tune and D.1's clock; D.3's protocol and shell repairs, D.6.1's castling class, D.6.2's queen promotion in quiescence, D.6.7's line bound, D.6.15's draw verdicts and D.5.24's physical-memory `Hash` bound; build names, a statically linked runtime and one universal x64 release binary; one PGO-trained kernel for every thread (`e057d84`); the classical evaluation frozen at engine source `9fb525e`; fingerprint **9,067,595 / EBF 2.479** since D.6.31 (`9d7b42f`, RAR-S125: a count is a move made, the null move not counted; 8,935,848 / EBF 2.480 since D.6.30, `0814689`: a proven win kept, repair case 1; 9,077,016 / EBF 2.481 since D.6.8, `ba1d2cd`, RAR-S124: each position counted once; 11,477,040 / EBF 2.537 since D.6.7, `aee0b3b`, RAR-S122; 13,038,236 / EBF 2.540 since D.6.2, `ee0b3e4`, RAR-S108; 11,946,003 / EBF 2.547 since D.6.1, `72c4e60`, RAR-S107; 15,625,852 / EBF 2.563 since D.5.5, `1450340`, RAR-S106; 14,749,043 / EBF 2.564 since D.6.15, `3a16fbb`; 13,987,223 / EBF 2.565 before); `rustc 1.100.0` since 2026-11-13 (RAR-P58: −0.24% pooled PGO NPS at the same fingerprint; 1.99.1 cost 1.63%, RAR-P36–P38). A registration names the revision it builds from and this fingerprint. Change records: `docs/HISTORY.md`, `docs/archive/PLAN-phaseD-done-2026-11-04.md`, `docs/archive/PLAN-closed-2026-10-22.md` |
| Pool position, `3+0.03` 1T | **3,428.8 ± 19.2** (RAR-M73, the C head, 2026-10-21), **+142.3 ± 24.8 over the B.9 head's 3,286.5 ± 15.7** (RAR-M64; 2.4.0 rates 3,001): Houdini 3 +172.8, Critter 1.6a +231.5, Fritz 16 +256.2, Rybka 4.1 +330.2, so all four E.2 targets pass at 1T; Rarog 2.4.0 +363.2 |
| Pool position, `3+0.03` 4T | **3,433.7 ± 19.3** (RAR-M74, the C head, 2026-10-21), **+93.7 ± 25.5 over the B.9 head's 3,340.0 ± 16.7** (RAR-M66): Houdini 3 +171.7, Critter 1.6a +242.2, Fritz 16 +249.1, Rybka 4.1 +350.3, all four E.2 targets pass at 4T. **Thread scaling fell:** in games the C head is 9.2% slower than the B.9 head at 4T while 29.8% faster at 1T (4T/1T 2.60 against 3.72; Colosseum CLI 0.4.0's figures), and its 4T performance is only +4.9 over its 1T rating (B.9 +53.5); held for D.2 |
| Pool position, wider field, `3+0.03` 1T | **3,478.9 ± 15.1** (RAR-M75, the C head, 2026-10-22): fourth in a round robin of nine with Houdini 3 and Critter held; Dragon 1 3,628.7, Stockfish 11 3,568.2, Houdini 6.03 3,498.5 (head −9 ± 32), Komodo 13.3 3,416.6 (head +53), Komodo 10.4 3,387.4, Houdini 4 3,303.2 (held 3,310). **CCRL-equivalent:** Blitz 1CPU **3,475 ± 82** (the held scale near 1:1 with it here; about 70 below CCRL Blitz's top-100 cut), 40/15 1CPU 3,407 ± 78 |
| Pre-NNUE goal, `3+0.03` 1T | **Stockfish 16 `Use NNUE=false`** (maintainer decision 2026-11-18, PLAN section 1). The adopted kernel (`d713kern`) **3,508.1** [3,499.9, 3,516.4] against its **3,604.7**, **96.6 below**, head to head **−100.6** [−119.4, −82.3]; fourth of fourteen, behind Berserk 4.7.0 HCE 3,543.6 and Stockfish 11 3,542.4, ahead of Ethereal 14.40 HCE 3,466.9 and Houdini 6.03 3,461.7; over 2.7.0 +63.5 in the field and +78.3 [+60.6, +96.4] head to head (RAR-M90, 2026-11-21, the field pinned at RAR-M89's ratings). 2.7.0 was 160 below and −164.8 head to head (RAR-M89, Basilisk's BAS-M18, 2026-11-17/18). RAR-M91 reads the merged build again in a twelve-engine round robin (2026-11-22, 52,800 games): **3,500.6** [3,495.1, 3,506.3], **−103.3** [−121.0, −86.0] head to head, fourth of twelve, Komodo 14.1 placed at 3,468.8 |
| Search deficit | The head is **+59.8 ± 8.1** over Stockfish classical's search on Rarog's evaluation, the bridge returned as time odds, and **−22.5 ± 7.9** against Reckless's search, Phase B's donor unchanged (RAR-O08, 2026-11-13); the second figure bundles tree shape, clock use and kernel speed; at equal work the shape reads level, `reckless-hce` +3.47 ± 8.37 (RAR-O09, 2026-11-14). Earlier meter, through a slower bridge: G(0) +24.24 ± 8.01 against the oracle (RAR-O04), from −247.97 at 2.4.0 (RAR-O03) |
| Evaluation deficit | **+99.2 ± 16.6 Elo at equal nodes** (RAR-O07, 2026-10-21): Stockfish's classical HCE over Rarog's frozen evaluation inside Stockfish's search, from +181.7 ± 19.0 at Phase C's start (RAR-O05), 82.5 ± 25.2 closed. The donor's remaining king signal is the index's structure (RAR-E72, RAR-E73); Phase F supersedes the classical evaluation, and an evaluation change under a fitted search follows PROCESS |
| Speed | The adopted kernel in games at `3+0.03` 1T (RAR-M90, Colosseum CLI 0.5.0): **1.92 M** nodes a second against 2.7.0's 1.96 M in the same games (−2.5%, another build), 2.2 plies deeper (16.33 against 14.14 without mate scores) at the same 66 ms a move. Against `d7classic` (RAR-S136): level at `3+0.03` (1.83 M against 1.82 M), −3% at `10+0.1` (1.91 M against 1.97 M), +3% at four threads (8.26 M against 8.00 M). No pooled bench read across the two kernels, whose bench trees differ (D.7.14); D.7.8.12's speed pass +2.71% pooled NPS before the fit. Earlier: the C head +28.92% pooled-PGO bench NPS over the B.9 head (RAR-P39) |
| Conversion | The adopted kernel **20.6 draws and 2.8 losses per 1,000 games** after a persistent piece-up (161 and 22 of 7,800, Poisson 95% 17.6–24.1 and 1.8–4.3; RAR-M90), against 2.7.0's 15.4 and 3.2 in RAR-M89's games; most of the rise is in the ending (131 draws against 82). The kernel also wins more (3,487 games against 3,103) and the count is a game, not a chance, so a worse rate is not shown: unresolved. Earlier: the C head 13.8 and 0.0 (RAR-M73), 2.4.0 24.4 and 5.3 |
| Active experiment | None running. **E.4.0.1 done 2026-11-24** (RAR-O13): inside the kernel, Stockfish 16's classical evaluation is **+80.3 ± 10.8** Elo over Rarog's at equal nodes and **+68.8 ± 10.4** at 3+0.03 with the bridge's 8.8% returned; Stockfish 11's **+74.4 ± 10.6** at equal nodes. Above the +25 rule, so a rebuild is not ruled out. **E.4.0's packet written 2026-11-24, waiting for the maintainer's decision** (`analysis/e40_donor_and_form_2026-11-24.md`, RAR-E74): the donors' extra wins are decided with heavy material, and on `hce-v4-tb` game results rank Rarog's evaluation above both donors there, so that corpus cannot fit a rebuild. Recommended: a rebuild in `sf_11`'s form, fitted by games. **D.16 done 2026-11-24.** D.16.3 closed `NO_CHANGE` (`analysis/d163_history_prefetch_2026-11-24.md`): its one item, the quiet-scoring history loads prefetched four moves ahead, read −4.74% (−6.49 to −2.99) on the bench and −3.67% on game positions, and was reverted, the source as before. Next: E.4.0.1. **D.16.2 done 2026-11-24** (`analysis/d162_bench_vs_play_2026-11-23.md`): both reads on two accepted speed changes. The table prefetch read bench +0.27% (−0.08 to +0.62) and games +0.70% (+0.44 to +0.95), agreeing by 0.007 points under the frozen rule; the node work read +1.23% and +1.29%. The bench read stays the speed read. **D.16.1 done 2026-11-23** (`415ec4f`): the game-position speed read, `tools/nps_read.py --positions tools/nps_positions_v1.jsonl`, 200 positions from RAR-S139's games drawn by thinking time, 186,000 nodes, Hash 64, a fresh process a position, the bench read's pools, estimator and steps; its self pair read −0.16% [−0.41%, +0.08%], none disturbed, step 1 in 6.6 minutes. **D.16.3.1 done 2026-11-23** (`analysis/d1631_node_profile_2026-11-23.md`): on E.0's 281 positions the head costs 412.7 ns a node against Stockfish 16 HCE's 311.2; the search side is +76.7 of the +101.5, the evaluation +31.5 (a third, not E.0's sixth), the table −7.3, the board level. Quiescence nodes are 32.7% of the kernel's tree and 46.9% of Stockfish's; per full-width node the kernel's node body is within 5% of Stockfish's, so most of the search side is the tree's shape (about 22 ns) and the kernel's own mechanisms (about 54: correction, threat maps, a dearer quiet). One neutral item reaches 1% of the samples (the quiet-scoring history loads prefetched ahead, ceiling 3.5%), so D.16.3 keeps that one item; the pass's band is 0% to 3.6% of node rate (0 to +6 Elo, inferred), not 4% to 12%. D.7.8.5 at `3000+30`: 0 of 300 searches stopped inside an iteration, so it stays closed. No board function above the donor's: D.12.1 to D.12.4 stay held. **E.0 done 2026-11-23** (`analysis/e0_remaining_strength_2026-11-23.md`): Stockfish 16 HCE searches 1.32 to 1.42 times the kernel's nodes a second, two thirds of that in search-side work a node, a sixth in the evaluation and none in the board; the evaluation in the donor's shape is ranked first (+25 to +70, a prior), search-side work second (+6 to +20). **RAR-M92 read the same day:** the kernel on 1.35 times the clock −43.7 ± 11.8 Elo against Stockfish 16 HCE over 1,600 games, a gain of +58.3 ± 17.3 over the baseline, so 1% of node rate is worth about 1.67 Elo against the meter; no fault or time loss. **The order of work was decided the same day** (maintainer; the list under the holds table): D.16 next, its profile D.16.3.1 first. **Landed 2026-11-23 as `b862256` (PR #21):** D.6's review on the kernel (D.6.7's ply cap, D.6.29's stalemate rule, the rest closed or moved into E.4.2), the WAC band at 218, the Trinity record with D.16 and D.17; E.0 is next, on `e0-research`. **D.6.32 closed `NO_CHANGE` 2026-11-23, which finishes D.6's review on the kernel and D.6:** the shortest mate shows as late on Reckless (KQK's mate in three at depth 20 on both) and on Stockfish 18 (27), so it is the shared search shape, not Rarog's, and it costs nothing in games (depth 20 on KQK in about 11 ms); E.0 is next. **D.6.29 done 2026-11-23** (RAR-S139): quiescence scores a stalemate in Stockfish's narrow form (`64993ff`), accepted by the maintainer's decision with the gate stopped at 6,046 games, nElo +1.8 ± 8.8 (harm beyond about −7 excluded, no gain shown); fingerprint 3,694,298 / EBF 2.324. **D.6's review on the kernel finished 2026-11-23:** D.6.15, D.6.16, D.6.17, D.6.20, D.6.27, D.6.30 and D.6.31 closed `NO_CHANGE` (the transposing-rook and won-ending clock tests run again, `dfb593c`); **D.6.32** closed after it (above). **D.6.13 closed `NO_CHANGE` 2026-11-22** on the kernel: its aspiration fail-low update is Reckless's, pinned by the parity test. **E.4 reworked 2026-11-22** (maintainer decision): the hand-crafted evaluation's programme, opening with E.4.0's research for the donor (Stockfish 11, 16 or `9587eeeb`'s classical evaluation) and the form (a rebuild as D.7's, or repairs); E.4.1 the tuner's checks, E.4.2 the endgame knowledge D.6's review found (D.6.9, D.6.10, D.6.21 to D.6.26 moved there), E.4.3 the king-danger unit. Basilisk's donor evidence is copied into `analysis/hce_donor_evidence_2026-11-22.md`. **D.6.10 closed 2026-11-22, moved into E.4.2:** 15% of the kernel's games reach a pawnless family Stockfish 11 scales and Rarog does not. **D.6.9 closed 2026-11-22, moved into E.4's refit (now E.4.2):** the endgame scale still multiplies the whole score, but in the kernel's games the forms differ mostly in pure opposite-bishop endings (1.5% of positions, about 4% of the score). **D.6.8 closed `NO_CHANGE` 2026-11-22** on the kernel: a node is counted once, when a move is made, as Reckless counts, and tests pin it. **D.6.7 done 2026-11-22** on the kernel: the ply cap at the donor's 240, its score bands with it (`91bc561`), kept by RAR-S138 (no correctness fault where the cap binds; nElo −11.6 ± 15.2 over 2,000 games against the −13 harm rule); fingerprint 3,374,901 / EBF 2.316. **RAR-S137 read 2026-11-22** (D.6.7's review): the kernel's searches reach the ply cap in none at `3+0.03`'s node counts or ten-thirds of them and in 0.511% at twenty times them, all endings from iteration 16, where the eight-times line bound passes the cap; D.6.7 stays open for the maintainer's choice of form. **D.6.6 closed `NO_CHANGE` 2026-11-22** on the kernel: its picker hands over the TT move before generating any capture, as Reckless's does. **D.6.5 closed `NO_CHANGE` 2026-11-22** on the kernel: its estimate is Reckless's, the plain evaluation at an exclusion node, and a rule test pins it. **D.6.4 closed `NO_CHANGE` 2026-11-22** on the kernel: a fail low stores no move and the table keeps the older one, as Reckless's do, and the kernel has no IIR. **D.6.3 closed `NO_CHANGE` 2026-11-22** on the kernel: every recursive call types its child as Reckless's does, and a test pins each type. **D.6.2 closed `NO_CHANGE` 2026-11-22** on the kernel: its noisy stage generates the quiet queen promotion, which quiescence and ProbCut search; a quiescence test pins it. **D.6.1 closed `NO_CHANGE` 2026-11-22**, the first step of D.6's review on the kernel: castling is quiet in the kernel's one partition, as in Reckless's, so the classic's defect has no instance left; zero games. **Phase E gains E.0 the same day** (maintainer request, after RAR-M91): research on where the remaining strength is, speed, search, evaluation and their fit, the kernel the slowest of RAR-M91's twelve engines at 1.58M nodes a second against Stockfish 16 HCE's 2.16M; **E.4 gains five tuner checks** from Basilisk's C.2.1.2 and a review of the fitting method before its refit. **Landed 2026-11-22 as `94d7f72` (PR #20):** D.7, the compact kernel. **RAR-M91 observed the same day:** the merged build 3,500.6, fourth of twelve, 100.2 below Stockfish 16 HCE, RAR-M90 reproduced; Komodo 14.1 3,468.8, +43.7 over Dragon 3.3 HCE head to head; Basilisk's first Phase C build level with 1.11.0. **D.7.10 done 2026-11-20: RAR-S134 read**, the repaired kernel with Reckless's time manager against `d7classic` at **+63.8 ± 9.1 Elo** (`3+0.03`) and **+65.0 ± 8.3** (`10+0.1`), 2,000 games each, no fault or time loss; the fit (D.7.11) runs at `3+0.03`. **D.7.11 researched 2026-11-20** (`analysis/d711_fit_2026-11-20.md`, zero games): the clock's fifteen coordinates are held at the donor's values and not fitted at `3+0.03`; the surface is 225 coordinates (sixteen whole-unit seeds and eight inactive ones out, seven new ones in, four of the kernel's exchange values among them); rule 7c's shape is kept, about 13.5 hours to the first checkpoint and 40.5 to the ceiling; `--max-time-losses 0` is withdrawn, because the pinned runner cannot resume a tune it invalidated and the new clock's lowest forfeit slack is 54 ms; the fit's own contribution is read once, at its stop, since D.7.13's gate would pass without it. **D.7.11.1 done 2026-11-20** (`6843b16`): the pawn table's limit, the five exchange values and the singular margin's stored-PV step are seeds, 249 tuning options, the fingerprint unchanged. **D.7.11.2 done the same day** (`66a1f14`): the activation read on the registered tune build is the record's exactly and the surface is 225 coordinates, the coverage audit clean. **RAR-S135 registered the same day** (D.7.11.3): the finishing fit, before any game, seeds frozen. **D.7.11.4 done the same day**: the pilot's, the tune's and a read's drivers, each run once through its dry path. **The pilot played the same day** (RAR-S135): 64 iterations at 153.7 an hour, so 13.0 hours to the first checkpoint and 39.0 to the ceiling; one time loss, the host's by the registered rule (a stall held all fifteen slots in one 48-second window); not interrupted, so the pause is tested in the tune's opening minutes (an amendment, maintainer decision the same day). **D.7.11.5 done the same day:** the tune cleared, the maintainer's to run (D.7.11.6). **D.7.11.6 done 2026-11-21:** the tune quiet at its first checkpoint, 2,000 iterations and 60,000 games without a fault, the largest |z| 3.09 against the line of 3.69, no rail; the moves small; the pause passed. **D.7.11.7 done the same day:** theta against the start **+7.8 ± 9.1 Elo** over 2,000 games, not wholly below zero, so theta is baked (D.7.11.8 next); no gain shown, and no gradient left by the tune's two windows. **D.7.11.8 held 2026-11-21:** the bake is prepared, but two mate tests fail on the baked seeds (KQK's mate in 3 first at depth 20, not 8; the warm-table test's node comparison flips); it waits on the maintainer's decision on those tests. **D.7.11.8 done the same day, which closes D.7.11** (maintainer decision: the tests measure their depths on the seeds, and the sensitivity goes to D.6.32, after D.7): three test commits, then the bake (`16f93cb`), fingerprint **3,390,842 / EBF 2.317**. **D.7.12 done the same day** (`analysis/d712_score_2026-11-21.md`): `score cp` is printed on a scale fitted on the baked kernel's own games, +100 an even chance of winning, and `UCI_ShowWDL` adds the win, draw and loss rates (`78d4681`, `61c9c51`); output only, the fingerprint unchanged. **D.7.13 registered the same day as RAR-S136**: the fitted, repaired kernel (`eb84cce`, pext PGO) against `d7classic` by SPRT `[0,3]` at `3+0.03`, cap 20,000 pairs, with 2,000 games at `10+0.1` and 2,000 at four threads beside it; the dry runs passed, and the run is the maintainer's (about 3 hours 35 minutes). **D.7.13 done the same day: RAR-S136 read, H1** at 520 pairs (+79.9 ± 12.3 Elo at the stop), with **+67.0 ± 8.1** at `10+0.1` and **+73.3 ± 8.1** at four threads, 2,000 games each, no fault or time loss; the clock's check does not fire. **D.7.14: adopted the same day** (maintainer decision): the margin block restated in the kernel's options (69 coordinates), the changelog's entry for the new search, D.11's facts named, D.8.21 open; **RAR-M90 registered**, the pre-NNUE meter read again, the kernel against RAR-M89's thirteen engines pinned at their ratings. **RAR-M90 read the same day, which finishes D.7.14 and D.7:** the kernel **3,508.1 ± 8.2**, 96.6 below Stockfish 16 HCE and −100.6 head to head, +63.5 over 2.7.0 in the field, no fault or time loss in 7,800 games; the checkpoint rows carry it. `d7-kernel` reaches `master` as one squash through the maintainer's pull request; the review of D.6 on the new kernel opens in the next. **D.7.9 done 2026-11-20** with D.7.9.1: **RAR-R34 read the same day, kept**, Reckless's time manager against the hybrid at **+30.0 ± 8.8 Elo** (`3+0.03`) and **+18.6 ± 8.2** (`10+0.1`), 2,000 games each, no time loss, nor in 200 games at four threads; it spends 0.75 of the base in the first twenty moves against 0.60. Before it, D.7.9's screen (`analysis/d79_clock_2026-11-20.md`, zero games): on D.7.8's games the kernel spends its budget as the classic did (85–86%, about 620 ms left at the end), but about 11% less a move while its clock is above a second, and its undecided moves under 500 ms halved; the low-clock horizon's field read is not registered (RAR-R17's direct read −0.9 ± 8.0, and the horizon acts only at `3+0.03`); thirteen clock coordinates join the fit's surface with their own activation read, `TmHorizonPerSec` held at 50; the clock constants were Stockfish's and Reckless's seeds, never fitted on Rarog. **D.7.9.1 opened the same day** (maintainer decision): one donor's time manager instead of the hybrid, Reckless's, whose budget and five factors were tuned with its search (an "LTC" SPSA session of 160,000 games and a six-thread one of 200,000, verified at `8+0.08` and `40+0.4`), Rarog's guards kept; it lands before D.7.10. **D.7.9.1 ported and qualified the same day** (`38e947c`; `bench 13` unchanged, every factor live): RAR-R34 registered and read that day (above). **D.7.8 closed 2026-11-20** with D.7.8.12, the speed pass before the fit: the table's cluster asked for as soon as a move is picked, and one fixed-capacity list for every move list, **+2.71% pooled NPS** (+2.11 to +3.31) against `b88700a`, the fingerprint unchanged; the correction and pawn prefetch read +0.14% and was reverted (`analysis/d7812_profile_2026-11-20.md`). D.7.8.11's zero-game screen `NO_CHANGE` (the donor's live re-scoring of the root changed the next move in 0.96% of re-scores, under the frozen 1%), and D.7.8.7 moved to D.13, after D.7. **RAR-S133 read 2026-11-20** (D.7.8.10 `NO_CHANGE`): the kernel on a unit half the size through 65 tune options against the fitted seeds, nElo −4.9 ± 15.2 over 2,000 games at 100,000 nodes a move, the interval not wholly above zero. **RAR-S132 read 2026-11-19** (D.7.8.3 done): the root ordered by the kernel's own scorer, ties included (`4271031`), against D.7.8.2's kept state, nElo +4.1 ± 15.2 over 2,000 games and no fault in 200 at four threads, kept; fingerprint 3,320,355 / EBF 2.317. Found at zero games: the root's tie order alone moved the tree by a third; KQK's shortest mate is back from depth 8; KRK's mate shows at depth 12 as nine. **RAR-S131 read 2026-11-19** (D.7.8.2 done): quiescence and evaluation-only entries at the donor's depth (`e363908`) against D.7.8.1's kept state, nElo +1.2 ± 15.2 over 2,000 games, kept; fingerprint 3,348,508 / EBF 2.329. Found at zero games: KQK's mate in four is reported as six until depth 18, as the donor's search does. **RAR-S130 read 2026-11-19** (D.7.8.1 done): ProbCut's exchange bar at the donor's 2,536 against the fitted 1,024, nElo −7.2 ± 15.2 over 2,000 games, not wholly below zero, so 2,536 is the seed (`e63b652`); fingerprint 3,278,226 / EBF 2.313. **The tests rebuilt on the kernel 2026-11-19** (`38528bc`): the kernel allows a mate in one in 1 of 123 WAC positions where the classic allowed none (its quiet skip drops direct checks, a dropped Rarog form not retried in D.7), and solves 201 WAC positions at 50,000 nodes against the classic's 232. **D.7 audited and reworked 2026-11-19** (`analysis/d7_audit_2026-11-19.md`, maintainer decisions the same day): RAR-S128 stopped at its 2,000 checkpoint and its centres baked in (D.7.6 and D.7.6.2 done; compact fingerprint 3,488,485 / EBF 2.336), **+56.8 ± 9.5 Elo** over the donor's seeds, the two wrapper coordinates alone +24.9 ± 9.0; one fold defect found, ProbCut's exchange bar. **RAR-S129 read 2026-11-19:** the unfitted kernel −11.6 ± 10.0 Elo against the donor's hybrid at equal nodes, clouded by the root driver's rule on an interrupted iteration, which RAR-O09 (1) shared. No gate at this state. **D.7.7 done 2026-11-19:** the classic kernel deleted (`d23b9cf`), `bench 13` 3,488,485 / EBF 2.336 before and after; its baselines `d7classic` and `d7baked` archived. Next, in order: the audit's repairs (D.7.8), clock use (D.7.9, moved from D.6.28), the pre-fit read against the classic (D.7.10), the finishing fit (D.7.11), the reported score calibrated (D.7.12), the one gate (D.7.13), adoption (D.7.14); after D.7 every D.6 step, closed or open, is reviewed on the new kernel. **D.7.4 done 2026-11-18** (`analysis/d74_compact_kernel_2026-11-18.md`): the compact kernel reaches parity with its donor, layer A 47 kinds without a mismatch and layer B's deciding read a pass on `a344748`; the switch off costs no speed (−0.34%, a pass), and the compact kernel is 11.54% faster a node. **D.7.4.1 done 2026-11-18** (`analysis/d741_fold_2026-11-18.md`): the constants folded into Rarog's units by dimension and the scaffold deleted, layer A without a mismatch and the positions-visited check a pass; compact fingerprint 7,543,485 / EBF 2.422; position 202 found a downgraded-mate band the compact kernel searches far past its donor, open for D.7.8 or before D.7.6. **RAR-S126 read 2026-11-18** (D.7.5 done): the folded kernel unfitted against the classic, nElo −11.1 ± 15.2 over 2,000 games, not clearly negative, so D.7.6's fit is next; 1.2 plies deeper at the same time, won endings converted better, the loss before the ending. **RAR-S127 read 2026-11-18** (zero games): position 202's trees are the donor's line shape running to the ply cap once a mate is in view, not a port defect; the line bound contains it but moves the bench −36%, so nothing changes before D.7.6. **RAR-S128 registered 2026-11-18** (D.7.6): the fit over 241 candidate seeds, an activation read and a curvature sweep first, then one rule 7c run to 6,000 iterations; D.7.6.1 done 2026-11-18: the seeds are tune options, compact lines are held to eight times the depth (the bench unchanged), and the surface is 230 coordinates; the pilot and the run are the maintainer's. **D.7 opened 2026-11-16** (maintainer decision on RAR-O12 and D.7.2.3): the search rebuilt whole as one compact kernel in the donor's form (Rarog's additions not carried, each tried again only on the adopted kernel), fitted as one unit and gated once on a gain (`[0,3]`); if it does not gain, Stockfish's form next (D.7.10). **RAR-S125 read 2026-11-15: kept** (D.6.31 done): the null move no longer counted (`9d7b42f`), nElo +5.9 ± 15.2 over 2,000 games against the −13 rule; fingerprint 9,067,595 / EBF 2.479; Rarog, Basilisk, Stockfish and Reckless now count alike. **RAR-O12 read 2026-11-16** (D.7.2.4 done, D.7.2.3 concluded): at equal work in Stockfish's count the start was level (+0.5 ± 11.9); Basilisk's search programme gained 55.2 ± 16.6 more per move made; speed and clock add 51.6 ± 16.1 at the ends and 18.1 ± 15.6 at the heads for Basilisk; Rarog's head +60.5 ± 11.5 at equal work, +42.4 ± 10.7 on the clock. **RAR-O12 registered, held** (D.7.2.4): until every arm counts as Stockfish does. **RAR-O11 read 2026-11-15** (D.7.2.2 done): on the clock the start was level (Rarog 2.4.0 −11.5 ± 11.7 against Basilisk 1.10.2) and Basilisk's search programme gained about 65 more than Rarog's (Rarog 2.5.0 −76.3 ± 11.1 against Basilisk's head); **its equal-node reads compared unequal node units** (old Rarog releases predate D.6.8's count, both Basilisks count re-searches), so the per-node and speed readings are withdrawn until a work-equalised re-read; Stockfish dev's search unfitted on Rarog's evaluation is 34.3 ± 12.0 worse than Reckless's at equal work; Rarog's evaluation is the cheapest per call and the most accurate of the four; Rarog's head +68.4 ± 9.0 over Basilisk's head (RAR-O10 (c)). **RAR-O10 read 2026-11-14** (D.7.2.1 done): against Stockfish 19 on 10,000 game positions, anchors passing, Rarog's evaluation is more accurate than Basilisk's (skill +0.0123 [+0.0016, +0.0235]; Basilisk's leads only with 10 to 24 pawns of material), and at 100 ms a move Basilisk's moves lose ×1.078 Rarog's (interval including zero), more with the most material; H1, Basilisk's lead is its search and fit, gains weight; not Elo. **RAR-O09 match (1) read 2026-11-14** (D.7.2 done): Reckless's hybrid **+3.47 ± 8.37** over the head at equal work, 3,000 games, valid by a recorded deviation (110 donor moves 3 to 5 nodes over budget); PLAN D.7's rule does not fire, D.7.3 to D.7.9 are held by the maintainer, and matches (2) to (4) stay optional (scope cut before any game, maintainer decision 2026-11-14). **D.7.0 closed 2026-11-13** (`analysis/search_shape_inventory_2026-11-13.md`): about a hundred departures from the donor, the new kernel in the donor's units, parity by decision records and tree statistics; D.7.1.1 gives the Colosseum wrapper a move time, a per-side node budget and a foreign arm. **RAR-M87 read 2026-11-13: a loss,** the 2.7.0 head G −7.6 Elo [−14.4, −0.7] below 2.6.0 against a five-engine field, +10.1 ± 8.7 head to head; the maintainer decided after the result to ship 2.7.0 as it is. **D.6.8 closed 2026-11-13** (RAR-S124): each position counted once (`ba1d2cd`) kept by its case-2 read, nElo −6.2 ± 15.2 against the −13 rule over 2,000 games; the same depth and time a move, nodes a move ×0.826, the count's unit; fingerprint 9,077,016 / EBF 2.481. **Landed 2026-11-13 as `89b4a92` (PR #17):** D.6.1 to D.6.7, RAR-O08 and the roadmap rework. **RAR-O08 read 2026-11-13:** three donor searches on Rarog's evaluation, six matches of 3,000 games. Reckless's search leads the head by 22.5 ± 7.9, Stockfish dev's trails Reckless's by 88.7 ± 7.7 and is level with Stockfish classical's (+4.4 ± 8.1), and the head leads Stockfish classical's by 59.8 ± 8.1. **PLAN reworked the same day** (`analysis/search_shape_decision_2026-11-13.md`): D.7 is the search-shape step, the kernel rebuilt whole as one compact kernel if D.7.2's reads or the maintainer say so, ahead of the open D.6 leaves, equal strength accepted; large pages on Linux is D.10 and the originality disclosure D.11; 2.7.0 is cut before D.7. **D.6.8 ready for implementation 2026-11-12** (RAR-S123): 16.1% of the node count on `bench 13` is a position counted again, mostly the hand-off to quiescence (12.3%) and the exclusion search (2.9%), so reported `nps` reads about 19% high; the fix counts each position once, gated by case 2. **D.6.7 closed 2026-11-11** (RAR-S122): the line bound (`aee0b3b`), which holds every line to twice the iteration's depth as the extension budget's comment states, reached the cap of its case-3 gate without a bound, nElo −1.0 ± 6.2 over 12,000 games, and is kept; that is no harm beyond about −5, not a gain. It cuts the searches reaching the ply cap at `60+0.6`'s node counts from 72 to 2 on replayed gate games (RAR-S121); fingerprint 11,477,040 / EBF 2.537. **D.6.6 closed `NO_CHANGE` 2026-11-11** (RAR-S120): generating captures after the TT move could save at most 0.82% of search cycles on replayed gate games, under the frozen 1%, and would reorder the top capture at 0.11% of nodes; C.12's "several percent" is superseded. **D.6.5 closed `NO_CHANGE` 2026-11-11** (RAR-S118, RAR-S119): under the donor's plain estimate LDSE fires at 23.8% of exclusion nodes on replayed gate games, but a paired probe finds the decision changed at only 1.96% of singular attempts, under the 2% floor; where it changes, the plain estimate is right 747 to 273 and cheaper, too rare to gate. Its re-entry sub-item closes `NO_CHANGE` too. **D.6.3 and D.6.4 closed `NO_CHANGE` 2026-11-11,** after both lost their gates. D.6.4 (RAR-S117, a replay of 60 gate games under eight arms): the store change's smaller tree and extra depth are IIR firing at revisited fail-low nodes (held, the depth gain falls from 0.66 to 0.07), and with the consumers held the kept move orders no better than the best-scoring one (1.022 of its nodes), so nothing is left to gate and the refit is not the right rework. D.6.3: its gate's H0 is weak evidence (RAR-M86: the case-3 gate reverts 34% of neutral repairs), nothing in its games differs and no mechanism points to a gain, so its rework is not gated. Both return only inside the search re-fit (F.6). New lead, D.6.27: IIR has no node-type condition. **RAR-S115 read 2026-11-11: H0.** D.6.4's store of no move at a fail low (`45adef3`) lost its gate, nElo −14.7 ± 13.8 at 2,452 games with mean depth up 0.89, and is reverted (`5d60017`); the fingerprint stays 13,038,236. **RAR-S114 read 2026-11-11: H0.** D.6.3's typing of ProbCut's verification children as all nodes (`d15050a`) lost its gate, nElo −3.7 ± 6.9 at 9,662 games, and is reverted (`91265f2`); the fingerprint stays 13,038,236. **D.6.2 closed 2026-11-10:** the queen push in quiescence and ProbCut (`ee0b3e4`) accepted by SPRT `[0,3]` at nElo +12.2 ± 6.5 over 11,108 games (RAR-S108); fingerprint 13,038,236 / EBF 2.540. **D.6.1 closed 2026-11-10:** castling classed quiet (`72c4e60`) kept by its harm read, nElo −4.9 ± 15.2 against the −13 rule (RAR-S107). **Landed 2026-11-04 as `c69761d` (PR #13):** D.2 closed `NO_CHANGE` (RAR-M83: four threads are worth 1.37 doublings of Rarog's own time against Reckless's 1.49 and Stockfish's 1.54; one logical CPU per physical core is the placement for every rated multi-thread read); D.3 done, with the protocol behind `src/uci/`; D.5.24's `Hash` bound and D.6.15's draw verdicts. Triumviratus is registered as a mechanisms-only donor and its small gains wait in Phase H. **Landed 2026-11-04 as `5efe16f` (PR #14):** E.3.2's build names (`sse42` +8.06% over `base`, RAR-P55) and E.3.3's universal binary (RAR-P56: +0.45% over the pext build), the x64 release asset, whose Intel SDE selection matrix passed in the first `Release` run. **Landed 2026-11-06 as `7d5fc06` (PR #15):** D.4.3's search-path perft; D.4.4's pricing instrument (fifteen of sixteen labels plant-validated; razoring's probe reproduces its decisions but its label stays out of screens, `analysis/pruning_pricing_card_2026-11-04.md`); D.4.5's loss-table tool done 2026-11-05, its run on the C head withdrawn before any reading and the read moved to H.3, on the network head's games (`analysis/loss_table_card_2026-11-05.md`, sections 10–11). The roadmap was reordered the same day so that its numbering is the order of work, with no step waiting for a later one (HISTORY's number map): D.5's repairs, D.6's candidates, D.7, D.8 and D.9, then Phases E to H in order. D.4.6 closed `NO_CHANGE` 2026-11-06, which finishes D.4: no counter beats the pooled builds, so the pooled read stays the speed check, and a per-process two-state lead goes to H.4 (`analysis/identical_tree_counters_card_2026-11-05.md`). Finished steps' records: PLAN and `docs/archive/PLAN-phaseD-done-2026-11-04.md` |

## Holds and obligations

| Open hold / obligation | Resume or resolve when | Must be resolved before |
|---|---|---|
| Originality disclosure (2026-10-23 review of 2.6.0): Stockfish's time-budget constants and Reckless's move-ordering and history constants were copied unchanged, the evaluation design is Stockfish 11's, Fathom is not named, and the README credits only "inspiration". The code and the evaluation weights are Rarog's own, and the move-similarity screen reads Rarog like an unrelated engine against both donors (56.4% Stockfish 11, 54.2% Reckless; related controls 67–69%). Since D.7's adoption (2026-11-21) the search is the compact kernel, whose shape follows Reckless's (`0756957`), with Reckless's time manager, two places transcribed and its constants refitted except those *What stays unfitted* lists (PLAN D.11, *Named at D.7.14*). Found 2026-11-23: three tracked Reckless patches in `tools/oracle-hybrids/` under the GPL label, and Phase F naming Reckless a primary network source | **D.11** (E.3.0 until 2026-11-13; widened 2026-11-23 from Coda's read, `analysis/coda_read_2026-11-23.md`): D.11.1 the donor inventory by licence at the version read, over source, tools and documents; D.11.2 the licence: GPL-3.0 kept by maintainer decision 2026-11-23, so Rarog never distributes Reckless's code (AGPL-3.0 since 2026-01-26); Stockfish is the primary donor and Reckless ideas only from that day (PROCESS); the card's other questions open; D.11.3 the decision carried out; D.11.4 the disclosure on Coda's model, its notices shipped by D.15.1 | The release after 2.7.0; lifted for the 2.7.0 cut by maintainer decision 2026-11-13 |
| Constants not fitted on Rarog after D.7.11 (`analysis/d711_fit_2026-11-20.md`, *What stays unfitted*): the clock's twenty-three, at the donor's values; the two thread spreads; `LazyMargin` against the new kernel; sixteen whole-unit seeds, fitted once in whole units. PROCESS's margin block is restated in the kernel's options (D.7.14, 2026-11-21) | Each at its owner step, where it is noted: the clock at **H.3**, or earlier on D.7.11's clock trigger (not fired by D.7.13's `10+0.1` read, RAR-S136); the spreads at **G.1**, on a four-thread deficit; `LazyMargin` at the next evaluation change (**E.4**, then **F.4.1**) through the margin block (69 coordinates since **D.7.14**); the whole-unit seeds at **F.6**; the eight seeds no bench reaches at **D.8.21**, open since D.7.14, which asks whether any is removed | The margin block before any evaluation change is registered; the others at their steps |
| D.12.1 to D.12.4, the board rebuild's leaves: held 2026-11-23 on E.0's packet, the board's work a node being level with Stockfish 16's (94 ns against 89; 88.2 against 88.0 in D.16.3.1's profile, which showed no board function above the donor's) | D.12.0 shows a board function that costs more than the donor's | F.1's board events, which D.12.0 serves |

Follow the earliest unblocked leaf in PLAN's register order, which the board's Now shows. Since 2026-11-13 (maintainer decision) that order leaves the numbering in two places: D.6.8 first (done 2026-11-13), then D.7 ahead of the other open D.6 leaves, each of which is asked first whether it is still an issue on the kernel D.7 leaves behind. 2.7.0, an unscheduled release with no step, was cut 2026-11-13 before D.7; the Basilisk check for new ports ran the same day (`analysis/basilisk_check_2026-11-13.md`: D.6.29, D.6.30, evidence for D.6.28, inputs to D.7.0). Held items stay unticked in place. Since 2026-11-19 (maintainer decisions, `analysis/d7_audit_2026-11-19.md`) D.7's open steps run in ascending order, D.7.7 to D.7.15, one at a time, and the review of every D.6 step on the new kernel follows D.7 in the next pull request. Since 2026-11-22 (maintainer decision) E.0, the research on where the remaining strength is, runs right after that review and ahead of D.8, and may reorder what follows it.

**Order of work from 2026-11-23** (maintainer decision on E.0's packet; every step keeps its number, a Phase E number inside Phase D's work included, and the board's Now follows this order):

1. **D.16**, the speed read and pass: D.16.3.1 (the profile beside Stockfish's, done 2026-11-23), D.16.1 (done 2026-11-23), D.16.2 (done 2026-11-24), D.16.3 (`NO_CHANGE` 2026-11-24); done 2026-11-24.
2. **E.4.0**, the evaluation's donor and form: E.4.0.1 (the donor's evaluation inside the kernel), then E.4.0's decision.
3. **D.8** to **D.11** in ascending order.
4. **D.12.0** only; D.12.1 to D.12.4 are held (the table above).
5. **D.13**, **D.14**, **D.15**, **D.17**.
6. **Phase E** as numbered: E.1, E.2, E.3, then E.4.1 onward.

## Model mapping

PLAN records only stable capability classes. This table names a model
family for each, never a version: versions move too fast to store
(maintainer decision 2026-11-13), and the family's current model is the
one meant. These are maintainer judgments, not measured rankings. An investigation leaf (`R3`) spawns the
implementation and measurement sub-steps under its step.

| Class | Capability | Model — thinking mode |
|---|---|---|
| `R3` | Frontier causal/architecture research | Claude Fable — High |
| `R2` | Bounded correctness-sensitive reasoning | Claude Opus — High |
| `I2` | Difficult implementation | Claude Opus — High |
| `I1` | Well-specified implementation | Claude Sonnet — Medium |
| `M` | Mechanical/docs/provenance | Claude Sonnet — Medium |
| `V` | Verification/measurement | Claude Sonnet — High |

Claude models only, by maintainer decision 2026-09-25.

## Reusable research prompt

> Investigate `<PLAN leaf>` as research, not implementation. Read PLAN,
> EXPERIMENTS, HISTORY's number map, linked analysis and relevant source;
> measured evidence outranks roadmap assumptions. Read the donor (Stockfish;
> Reckless for ideas only, never its code or constants) for mechanism,
> population and interaction, never for transcription. State the precise question, leading and competing
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
