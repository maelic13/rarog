# Rarog development guide

<!-- board: generated from docs/PLAN.md by `python tools/diag/guide_board.py`; edit PLAN, never this block -->
**Now: D.2** (`R2`): Lazy SMP quality at 4T/8T: diversity, shared TT and correction, soft-stop voting; 4T SPRT `[0,5]`; its premise is contradicted by RAR-M46, so re-scope first.

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
    - [x] **D.1.1** Time-management diagnosis, pulled forward: rec1's stalled re-search, rec2's search past a found mate, rec3's clock-independent stop; mechanism, frequency in games, the frozen fix and its gate — NO_CHANGE 2026-10-06
    - [x] **D.1.2** Time-management audit and bound model: the ADR-0065 checklist, soft and hard bounds with the node-fraction multiplier, the forfeit margin, the won-ending stalls (RAR-R13); SPRT `[0,3]` — DONE 2026-10-26
        - [x] **D.1.2.1** Won-ending time stalls: rec1's cascade, rec2's missing stop after a proved mate, rec3's overrun (RAR-R13) — DONE 2026-10-26
        - [x] **D.1.2.2** The forfeit margin against OS holds — DONE 2026-10-26
        - [x] **D.1.2.3** The audit against the ADR-0065 checklist and the bound-model decision — DONE 2026-10-22
    - [x] **D.1.3** The clock cluster: RAR-R14's six clock-only mechanisms and the cross-move score trend — DONE 2026-10-22
    - [x] **D.1.4** The clock fit: a wire check, an activation read, then SPSA blocks over the clock coordinates — DONE 2026-10-23
    - [x] **D.1.5** The clock gate, D.1's closing read after D.1.6: the clock pieces at their seeds against the 2.6.0 engine with the cascade fix; harm reads at `3+0.03` and `10+0.1`, a four-thread read, the probe and the profile — DONE 2026-10-26
    - [x] **D.1.6** The root fail-high cascade in won endings: researched in the search, capped by the stop at twice the target — DONE 2026-10-26
        - [x] **D.1.6.1** Research: the cascade's mechanism in Rarog's loop, the donors' devices, the cheapest prototype and its zero-game reads, the registration — DONE 2026-10-23
        - [x] **D.1.6.2** Implementation: the cleanup, the unused D.1.6 instruments removed and the stop at twice the target kept — DONE 2026-10-26
        - [x] **D.1.6.3** Gate: the repairs' reads and gates, the probe and the differential — DONE 2026-10-26
- [ ] **D.2** Lazy SMP quality at 4T/8T: diversity, shared TT and correction, soft-stop voting; 4T SPRT `[0,5]`; its premise is contradicted by RAR-M46, so re-scope first — **RESEARCH / R2**
- [ ] **D.3** Engine lifecycle and protocol robustness; `src/uci/` (planned) move; score normalisation research card (`analysis/uci_info_review_2026-09-16.md` item 6); zero crashes over pool tournaments — **RESEARCH / R2**
- [ ] **D.4** Tablebase policy: probing depth/limits, WDL/DTZ in conversion, recogniser interaction — **RESEARCH / R2**

## Phase E — Classical checkpoint and release

- [ ] **E.1** Attribution checkpoint: B.2.0 review re-run on the B.9/C.11 heads; STC, `10+0.1`, 4T against 2.3.2 and the B.9/C.11 heads; maturity checklist — **V**
- [ ] **E.2** Target gate: ≥50% against Critter 1.6a, Houdini 3, Rybka 4.1 and Fritz 16 at 1T and 4T (Rybka 4.1 replaced Rybka 4, 2026-09-19); the binding arm is 1T — **V**
- [ ] **E.3** Release 2.7.0 through the tag-driven flow; 3.0.0 is reserved for the first NNUE release — **M**
    - [ ] **E.3.0** Originality disclosure before the release cut: the donor material named, the licence settled, the notices shipped with the binaries — **R2**
    - [x] **E.3.1** Tag-driven release flow — DONE 2026-10-05
    - [ ] **E.3.2** Release cut: version 2.7.0, the `[Unreleased]` changelog reviewed and dated, suites, the release PR squash-merged, the `v` tag pushed on instruction through E.3.1's workflow — **M**
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
- [ ] **G.2** Platform and product: Chess960 on demand, distributed testing, and the OPTIONAL universal binary (`analysis/universal_binary_2026-09.md`) — **I1**
- [ ] **G.3** Frontier: larger nets, data scaling, LTC search fit; CCRL top-50 gate — **R3**
<!-- end of generated board -->

## Current checkpoint

| Item | Value |
|---|---|
| Next release | **2.7.0** at E.3 after Phases D and E, cut only after **E.3.0**'s originality disclosure (maintainer decision 2026-10-23; `analysis/originality_review_2.6.0_2026-10-22.md`) (maintainer decision 2026-10-22: 3.0.0 is reserved for the first NNUE release, F.9; E.2's four targets already pass at 1T and 4T, RAR-M73, RAR-M74, and its result is recorded, not a version). Later classical gains (E.4, F.2.1) release as 2.8.0 onward under Phase F's rule |
| Released baseline | **2.6.0** on `master`, released 2026-10-22 from the `v2.6.0` tag; fingerprint **13,987,223 / EBF 2.565**. Over 2.5.0: +153.4 ± 10.7 at `3+0.03` 1T, +149.3 ± 15.1 at `10+0.1`, +149.7 ± 14.7 at 4T (RAR-M76). 2.5.0 (2026-10-05, 11,171,726 / EBF 2.512) was +272.4 over 2.4.0 at `3+0.03` (RAR-M64) |
| Development head | `d2` from `master` at `703fe78` (D.1's squash, "Rework time management", PR #12, 2026-10-26; the 2.6.0 release is `0127e58`, PR #11), version **2.7.0-dev**: the 2.5.0 search with Phase C's evaluation, C.12's speed pass and large pages, and C.10's search tune, plus D.1.3's clock cluster at its seeds (`5fec0d7`, the cross-move trend `1f185cb`; clock-only, its closing read is D.1.5) and the fail-high stop at twice the soft target (`a79efe4`, RAR-R25), with D.1.6's research instruments removed (`42b001f`); the classical evaluation frozen at engine source `9fb525e`; fingerprint **13,987,223 / EBF 2.565**; `rustc 1.99.1` (RAR-P36–P38, kept by maintainer decision). A registration names the revision it builds from and this fingerprint. Change records: PLAN D.3's change log, `docs/HISTORY.md`, `docs/archive/PLAN-closed-2026-10-22.md` |
| Pool position, `3+0.03` 1T | **3,428.8 ± 19.2** (RAR-M73, the C head, 2026-10-21), **+142.3 ± 24.8 over the B.9 head's 3,286.5 ± 15.7** (RAR-M64; 2.4.0 rates 3,001): Houdini 3 +172.8, Critter 1.6a +231.5, Fritz 16 +256.2, Rybka 4.1 +330.2, so all four E.2 targets pass at 1T; Rarog 2.4.0 +363.2 |
| Pool position, `3+0.03` 4T | **3,433.7 ± 19.3** (RAR-M74, the C head, 2026-10-21), **+93.7 ± 25.5 over the B.9 head's 3,340.0 ± 16.7** (RAR-M66): Houdini 3 +171.7, Critter 1.6a +242.2, Fritz 16 +249.1, Rybka 4.1 +350.3, all four E.2 targets pass at 4T. **Thread scaling fell:** in games the C head is 9.2% slower than the B.9 head at 4T while 29.8% faster at 1T (4T/1T 2.60 against 3.72; Colosseum CLI 0.4.0's figures), and its 4T performance is only +4.9 over its 1T rating (B.9 +53.5); held for D.2 |
| Pool position, wider field, `3+0.03` 1T | **3,478.9 ± 15.1** (RAR-M75, the C head, 2026-10-22): fourth in a round robin of nine with Houdini 3 and Critter held; Dragon 1 3,628.7, Stockfish 11 3,568.2, Houdini 6.03 3,498.5 (head −9 ± 32), Komodo 13.3 3,416.6 (head +53), Komodo 10.4 3,387.4, Houdini 4 3,303.2 (held 3,310). **CCRL-equivalent:** Blitz 1CPU **3,475 ± 82** (the held scale near 1:1 with it here; about 70 below CCRL Blitz's top-100 cut), 40/15 1CPU 3,407 ± 78 |
| Search deficit | Closed: G(0) **+24.24 ± 8.01** against the oracle (RAR-O04), from −247.97 at 2.4.0 (RAR-O03) |
| Evaluation deficit | **+99.2 ± 16.6 Elo at equal nodes** (RAR-O07, 2026-10-21): Stockfish's classical HCE over Rarog's frozen evaluation inside Stockfish's search, from +181.7 ± 19.0 at Phase C's start (RAR-O05), 82.5 ± 25.2 closed. The donor's remaining king signal is the index's structure (RAR-E72, RAR-E73); Phase F supersedes the classical evaluation, and an evaluation change under a fitted search follows PROCESS |
| Speed | The C head (2026-10-21/22): **+28.92%** pooled-PGO bench NPS over the B.9 head (RAR-P39; `rustc 1.99.1`'s −1.63% included, RAR-P37, its code generation −3.6% without PGO, RAR-P38); in games (Colosseum CLI 0.4.0) +29.8% at 1T and −9.2% at 4T over the B.9 head (RAR-M73, RAR-M74), and −8.6% at 1T and −19.1% at 4T against Rarog 2.4.0 in the same games; the evaluation 21.1% of search samples, B.9's 24.3% (RAR-P40) |
| Conversion | **13.8 draws and 0.0 losses per 1,000 games** after a persistent piece-up (RAR-M73's games, the C head; Poisson 95% 9.5–19.3 and 0–1.5), against the B.9 head's 15.8 and 0.8 (RAR-M64) and 2.4.0's 24.4 and 5.3: the draw rate's fall is not resolved |
| Active experiment | **RAR-P50 registered 2026-10-30, the maintainer's run (about 37 minutes, zero games):** which change from B.9 to C holds the four-thread shared-search loss. **RAR-P49 read 2026-10-30, shared search:** the C head's 4T scaling loss is entirely in the shared search, with none in the hardware. A bisect from B.9 to C is next. **RAR-P48 read 2026-10-29, Mixed:** the C head's 4T scaling loss is real under pinning and is neither the CCD split nor large pages. Hardware against shared search is unresolved, and the instrument is repaired next. **RAR-M77 played 2026-10-28:** in games at four threads large pages on are +2.7% NPS and +10.3 ± 5.9 Elo over off, so `LargePages` (`2b7bca9`) stays on by default. **D.1.6 closed 2026-10-26.** The won-ending stalls end on the fail-high stop at twice the target (`a79efe4`; RAR-R25 no harm: +4.3 ± 8.3 Elo at `3+0.03`, +4.9 ± 9.8 at `10+0.1`; every probed stall under the maximum). No repair in the search or evaluation landed (RAR-R22 H0, RAR-R23 harm at `10+0.1`, RAR-R27 H0 at −1.2 ± 3.0), and the instruments are removed (`42b001f`). **D.1.5 closed 2026-10-26 (RAR-R28): no harm.** `d1`'s head against 2.6.0 read +1.2 ± 8.9 Elo at `3+0.03`, −0.2 ± 7.8 at `10+0.1` and −8.0 ± 11.3 at four threads, with 0 time losses, rec2 in 5 ms and no game under 100 ms of clock. D.1's clock work is kept and D.1.2.2 ticks. **D.1.2.1:** won moves at ≥ 0.9 × the maximum are 0.24% and 0.85% (bar 0.2%), every such game won. Its residue read finds no cost: none in a game not won, no clock under 100 ms after, conversion within 13%, and a third of 2.6.0's rate at `10+0.1`. **D.1 closed 2026-10-26** by maintainer decision, the residue accepted. |

## Holds and obligations

| Open hold / obligation | Resume or resolve when | Must be resolved before |
|---|---|---|
| Originality disclosure (2026-10-23 review of 2.6.0): Stockfish's time-budget constants and Reckless's move-ordering and history constants were copied unchanged, the evaluation design is Stockfish 11's, Fathom is not named, and the README credits only "inspiration". The code and the evaluation weights are Rarog's own, and the move-similarity screen reads Rarog like an unrelated engine against both donors (56.4% Stockfish 11, 54.2% Reckless; related controls 67–69%) | **E.3.0**: rerun the scans on the release head, the maintainer settles the licence (Reckless is AGPL-3.0), then README, PROCESS's boundary and the binaries' notices | E.3.2, the 2.7.0 release cut |
| RAR-M74's finding: the C head's thread scaling fell (in-game NPS 4T/1T 2.60 against the B.9 head's 3.72; −9.2% against B.9 at 4T). **Located 2026-10-27 (RAR-P46), in both:** the engine's own 4T/1T is 3.28 against 3.80, though alone at 4T it is still +21% over B.9; under the games' conditions the C head alone enters a system-level slow state (−7 to −10%) in 5 of 12 passes. **RAR-P47, 2026-10-27:** with the new `LargePages` option the slow state appeared only with large pages on (2 of 16 passes, 0 of 16 off), consistent with the allocator owning it but not established; off costs about 1–4% NPS. **RAR-M77, 2026-10-28:** not in games: the same head in a self-match under RAR-M74's conditions reads 8.08 M nps with large pages on (+2.7% over off, +10.3 ± 5.9 Elo), 13% above B.9's 7.12 M | D.2 researches the engine's own 2T–4T loss, and separately why the head read 6.47 M against RAR-M74's field but 8.08 M against itself (candidate: opponents' threads on the shared slot) | D.2 (Lazy SMP quality at 4T/8T) |

Follow the earliest unblocked leaf in PLAN's register order, which the board's Now shows (the order can differ from the numbering). Held items stay unticked in place.

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
