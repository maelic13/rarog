# Rarog development guide

<!-- board: generated from docs/PLAN.md by `python tools/diag/guide_board.py`; edit PLAN, never this block -->
**Now: C.2** (`V`): Datagen and label contract for the programme; corpus frozen under a new name; fitting manifest (free/fixed/excluded).

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

- [x] **C.0** Investigation: family map, residuals, donor conditioning, shared inputs, cluster order, refit protocol — DONE 2026-10-06
    - [x] **C.0.1** Evaluation meter at the phase start: the same-search gap at equal nodes and at equal time (RAR-O05) — DONE 2026-10-05
    - [x] **C.0.2** Donor-direction residual screen (RAR-E17, RAR-E18) — DONE 2026-10-05
    - [x] **C.0.3** Lazy path: the played evaluation omits imbalance above its gate; measure, then remove, repair or keep — DONE 2026-10-06
    - [x] **C.0.4** Research card for the first evaluation unit: king safety with winnability and scaling; sub-term attribution, shared inputs, the magnitude contract; freezes C.3's handoff — DONE 2026-10-06
- [x] **C.1** Evaluation restructure, behaviour-neutral: modules, one attack-map producer, `eval/params.rs`, `kpk` under `endgame/`; exact fingerprint — DONE 2026-10-06
- [ ] **C.2** Datagen and label contract for the programme; corpus frozen under a new name; fitting manifest (free/fixed/excluded) — **GAME_GATE / V**
- [ ] **C.3** First evaluation unit: king safety (danger units, safe/unsafe checks, weak ring, flank, shelter/storm) with winnability and scaling (C.5.1, C.5.2); shared inputs; one refit; margin block; one gate — **I2**
    - [ ] **C.3.1** King danger in the donor's shape: ring, accumulated attackers, weak ring, safe and unsafe checks, blockers, king-adjacent attacks, the reductions, the capped quadratic map; the old table and inputs removed; the fixture test — **READY_FOR_IMPLEMENTATION / I2**
    - [ ] **C.3.2** Shelter and storm by file and rank with the castling destination in the pawn cache; the linear terms replaced; the pawnless flank outside the index — **READY_FOR_IMPLEMENTATION / I2**
    - [ ] **C.3.3** Tuner: the nonlinear pass over the index coordinates in index units, the map scales and shelter tables in the linear groups, the two family masks, feature-support coverage — **READY_FOR_IMPLEMENTATION / I1**
    - [ ] **C.3.4** Refit on `hce-v4-tb`, static screens (the king family's residual re-read, the magnitude read), the tree read, PGO bake, gate-1 registration and handover — **READY_FOR_IMPLEMENTATION / V**
    - [ ] **C.3.5** Margin block, gate 2, the `10+0.1` read, the ledger row — **RESEARCH / V**
    - [ ] **C.3.6** Same-search deficit re-read on the accepted unit: the oracle package rebuilt with the C.3 evaluation in `rarog_hce.dll`, 1,000 games at 150,000 nodes a move against the Stockfish control; decides whether C.4 to C.7 open — **RESEARCH / V**
- [ ] **C.4** Threats and mobility cluster: mobility area, weak enemies, hanging, restricted, pawn push, queen threats; refit; gate — **RESEARCH / I2**
- [ ] **C.5** Endgame handling and winnability cluster — **R3**
    - [ ] **C.5.1** Classification and deciding instrument per family — **RESEARCH / R2**
    - [ ] **C.5.2** Generic winnability and scaling: pawn count, opposite bishops, rule-50 scale, complexity — **RESEARCH / I2**
    - [ ] **C.5.3** Conversion cluster: KXK, KBNK, KQKR; rule-50 damping interaction measured — **RESEARCH / I2**
    - [ ] **C.5.4** Rook versus minor cluster: KRKN, KRKB, KRPKB — **RESEARCH / I2**
    - [ ] **C.5.5** Rook and pawn cluster: KRPKR, KRKP, KPK, KPKP audit — **RESEARCH / R2**
    - [ ] **C.5.6** Measure-first families: KPsK, KBPsK, KBPPKB, KQKRPs — **RESEARCH / R2**
    - [ ] **C.5.7** Theory sweep: KBPKB, KBPKN, KNNKP, KNNK, KQKP from one dispatcher — **RESEARCH / I1**
    - [ ] **C.5.8** Endgame gate: endgame-start cohort SPRT plus STC SPRT; floors; conversion; 7-man exclusion — **RESEARCH / V**
- [ ] **C.6** Pawns and passers cluster; refit; gate — **RESEARCH / I2**
- [ ] **C.7** Material, imbalance, phase and pieces cluster; refit; gate — **RESEARCH / I2**
- [ ] **C.8** Refit cycles: regenerate, refit, gate; stop at the first non-accepting cycle — **RESEARCH / V**
- [ ] **C.9** HCE SPSA of nonlinear residue, or a written skip — **RESEARCH / V**
- [ ] **C.10** Joint search SPSA after the new evaluation: cp margins plus every mechanism whose firing rate moved 10% or more (the whole-surface tune B.6 left for here); rule-7c blocks; SPRT `[0,3]` — **RESEARCH / V**
- [ ] **C.11** Checkpoint: same-search deficit, conversion, NPS, pool gauntlet; freeze the classical evaluation — **RESEARCH / V**

## Phase D — Clock, threads, robustness

- [ ] **D.1** Time management: audit against the ADR-0065 checklist, soft/hard bounds with node-fraction multiplier, forfeit margin; SPRT `[0,3]` — **R2**
    - [x] **D.1.1** Time-management diagnosis, pulled forward: rec1's stalled re-search, rec2's search past a found mate, rec3's clock-independent stop; mechanism, frequency in games, the frozen fix and its gate — NO_CHANGE 2026-10-06
    - [ ] **D.1.2** Time-management audit and bound model: the ADR-0065 checklist, soft and hard bounds with the node-fraction multiplier, the forfeit margin, the won-ending stalls (RAR-R13); SPRT `[0,3]` — **R2**
- [ ] **D.2** Lazy SMP quality at 4T/8T: diversity, shared TT and correction, soft-stop voting; 4T SPRT `[0,5]`; its premise is contradicted by RAR-M46, so re-scope first — **R2**
- [ ] **D.3** Engine lifecycle and protocol robustness; `src/uci/` (planned) move; score normalisation research card (`analysis/uci_info_review_2026-09-16.md` item 6); zero crashes over pool tournaments — **R2**
- [ ] **D.4** Tablebase policy: probing depth/limits, WDL/DTZ in conversion, recogniser interaction — **R2**

## Phase E — Classical checkpoint and release

- [ ] **E.1** Attribution checkpoint: B.2.0 review re-run on the B.9/C.11 heads; STC, `10+0.1`, 4T against 2.3.2 and the B.9/C.11 heads; maturity checklist — **V**
- [ ] **E.2** Target gate: ≥50% against Critter 1.6a, Houdini 3, Rybka 4.1 and Fritz 16 at 1T and 4T (Rybka 4.1 replaced Rybka 4, 2026-09-19); the binding arm is 1T — **V**
- [ ] **E.3** Release 3.0.0 (E.2 met) or 2.6.0 through the tag-driven flow — **M**
    - [x] **E.3.1** Tag-driven release flow — DONE 2026-10-05
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
<!-- end of generated board -->

## Current checkpoint

| Item | Value |
|---|---|
| Next release | **3.0.0** at E.3 if the E.2 target gate is met, otherwise 2.6.0 |
| Released baseline | **2.5.0** on `master`, released 2026-10-05 from the `v2.5.0` tag; fingerprint **11,171,726 / EBF 2.512**. Over 2.4.0: +272.4 at `3+0.03` 1T, +260.5 ± 16.0 at `10+0.1`, +322.7 at 4T (RAR-M64, RAR-M65, RAR-M66) |
| Development head | `dev`, version **2.6.0-dev**: the 2.5.0 engine plus behaviour-neutral source changes recorded under PLAN D.3's change log (latest `6433760`, each with the exact fingerprint) and C.1's evaluation restructure into `src/eval/` (`801f1b4`, exact), then C.2's lazy gate in the `texel` build (`79c6808`, texel builds only, exact); fingerprint **11,171,726 / EBF 2.512**; `rustc 1.98.1`. A registration names the revision it builds from and that fingerprint. The search is frozen for Phase C at engine source `ee02ed1` (the measured binary and its SHA-256 are in PLAN's Phase C rules): a C-phase change touching `src/search/` returns to its owner leaf and is gated as a search change, and search coordinates move only in C.10's joint tune |
| Pool position, `3+0.03` 1T | **3,286.5 ± 15.7** (RAR-M64; 2.4.0 rates 3,001): Houdini 3 +37.5, Critter 1.6a +96.2, Fritz 16 +87.8, Rybka 4.1 +167.3, so all four E.2 targets pass at 1T |
| Pool position, `3+0.03` 4T | **3,340.0 ± 16.7** (RAR-M66): Houdini 3 +113, Critter 1.6a +148, Fritz 16 +132, Rybka 4.1 +210 |
| Search deficit | Closed: G(0) **+24.24 ± 8.01** against the oracle (RAR-O04), from −247.97 at 2.4.0 (RAR-O03) |
| Evaluation deficit | **+181.7 ± 19.0 Elo at equal nodes**, Phase C's meter baseline, and +266.3 ± 19.9 at equal time: Stockfish's classical HCE over Rarog's current one inside Stockfish's search (RAR-O05, 2026-10-05). RAR-O02's earlier about 329 was the 2.3.2 evaluation with unequal throughput. **Caveat (C.0 audit, 2026-10-06):** measured inside a search fitted to Stockfish's evaluation shape, where a shape mismatch alone costs about 105 Elo per node (RAR-E19/E20); the recoverable share is unknown and is re-read after the first accepted unit (C.3.6) |
| Evaluation residual (static layer) | On 194,444 held-out positions Stockfish's total adds **3.59%** to Rarog's outcome prediction and its term families **1.22%**; king safety +0.66% (+1.58% in the middlegame band), winnability +0.12%, mobility, pieces, material and space nil (RAR-E17, RAR-E18). The king residual is the donor's danger map (92% of it; inside the map the safe checks and the weak ring), not shelter or the flank terms; Rarog's zeroed danger inputs are a resolution limit of its bucket table; the scale factor adds +0.09% above six men (RAR-E21, 2026-10-06). With one Stockfish version for families and total the families read 1.85% and carry the whole 2.73% at seven men or more, so there is no total-level knowledge above six men to chase (RAR-E22). It ranks questions; it is not Elo |
| Speed | **2.33 MNPS**, −26.23% against 2.4.0 (RAR-P35); the host drifts between days, so compare pools interleaved only |
| Conversion | **15.8 draws and 0.8 losses per 1,000 games** after a persistent piece-up (RAR-M64's games; 2.4.0: 24.4 and 5.3) |
| Evaluation–search coupling | Feeding the frozen search the full fitted evaluation above the lazy gate costs **−104.5 ± 10.6 Elo at equal time and −110.0 ± 11.5 at equal nodes** (RAR-E19, RAR-E20, 2026-10-05/06), though that function predicts outcomes better statically: the cost is per node, in a search fitted to the played function. Every evaluation unit goes through PROCESS's *Evaluation change under a fitted search*: static screens, tree read, gate 1 (accepts when it passes), and only when it fails a margin block over the fixed 28-coordinate surface (`LazyMargin` added by the C.0 audit) and gate 2 (accepts), then a `10+0.1` read (maintainer decisions 2026-10-06; a flag no longer triggers the block, 2026-10-08) |
| Active experiment | C.2: RAR-M67 and RAR-E23 run (2026-10-07); RAR-E24 (the unchanged surface refitted on `hce-v4-tb`, gated by PROCESS's shape: gate 1 `[0,3]`, the `c2margin` block when gate 1 fails or the tree read flags, gate 2, cap 20,000 pairs a gate). Last run: RAR-E22 (C.0 audit), run 2026-10-06, zero games: with one Stockfish version the donor's families carry all of its gain above six men, superseding RAR-E18's total-level excess; RAR-E21's king attribution survives clipping |

## Holds and obligations

| Open hold / obligation | Resume or resolve when | Must be resolved before |
|---|---|---|
| Tags `archive/pr2-version-2.5.0`, `archive/pr3-tag-driven-release`, `archive/pr4-release-checks`: the `dev` chains squash-merged as PRs #2–#4 (400 commits the documents cite), joined to `dev` by `e887c3f` | The next PR to `master` has merged with a merge commit and `git for-each-ref --contains <tip>` lists `master` for each; then the maintainer deletes the three tags | Phase C's ref review at C.11 |
| Won-ending time stalls (rec1 cascade, rec2 no stop after a proved mate, rec3 overrun; RAR-R13): real, no measured cost at `3+0.03`/`10+0.1` | D.1.2 opens after Phase C, which owns the fix; earlier if RAR-R13's trigger fires (a game record shows a cost) | D.1.2 closes (owner) |
| KRPPKRP 7-man truth gap | Independent truth becomes available, or C.5.8 records an explicit exclusion | C.5.8 closes |
| KRP-KB win-preserving 0.9990 → 0.9949 (−2.2 SE, RAR-M42) | Non-blocking; blocking if a later change pushes it past 3 SE | C.5.4 closes (owner) |
| RAR-E24 gates (maintainer): gate 1 H1 (+31.0 ± 8.5 Elo, 2,442 games); the `c2margin` block done (60,000 games); gate 2 H1 (+35.9 ± 9.3 Elo, 2,214 games, `rarog-c2gate2-pext-pgo`, 12,351,448); the `10+0.1` read +39.1 ± 12.9 Elo, so the acceptance stands; the margin attribution read (gate-2 arm against gate-1 arm, 2,000 games) handed over | The read finishes; then the agent lands the candidate on `dev` (new fingerprint 12,351,448) and records the disposition | C.2 closes; C.3.4 (its fit needs RAR-E24's baseline) |
| Unstoppable-passer test one tempo generous in both move orders (`src/eval/passers.rs`; confirmed in the source by C.0, 2026-10-05) | C.6 opens: definition change plus refit, checked on tablebase-labelled pawn endings | C.6 closes (owner) |

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

## About this guide

The board at the top is generated from `docs/PLAN.md` by
`python tools/diag/guide_board.py`: edit PLAN's step heads (title, class,
DONE) and its workflow register, then regenerate; never edit the board by
hand. A closed phase shows its one- or two-sentence summary; its steps are
in `docs/archive/`. Everything below the board is kept by hand.
`python tools/diag/check_guide.py` must pass; it fails on a stale board.
