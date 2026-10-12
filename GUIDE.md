# Rarog development guide

<!-- board: generated from docs/PLAN.md by `python tools/diag/guide_board.py`; edit PLAN, never this block -->
**Now: C.6.6** (`V`): Gate 1's read, the margin block when flagged, gate 2, the `10+0.1` read, the ledger row.

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
- [x] **C.2** Datagen and label contract for the programme; corpus frozen under a new name; fitting manifest (free/fixed/excluded) — DONE 2026-10-08
- [x] **C.3** First evaluation unit: king safety (danger units, safe/unsafe checks, weak ring, flank, shelter/storm) with winnability and scaling (C.5.1, C.5.2); shared inputs; one refit; margin block; one gate — DONE 2026-10-09
    - [x] **C.3.1** King danger in the donor's shape: ring, accumulated attackers, weak ring, safe and unsafe checks, blockers, king-adjacent attacks, the reductions, the capped quadratic map; the old table and inputs removed; the fixture test — DONE 2026-10-08
    - [x] **C.3.2** Shelter and storm by file and rank with the castling destination in the pawn cache; the linear terms replaced; the pawnless flank outside the index — DONE 2026-10-08
    - [x] **C.3.3** Tuner: the nonlinear pass over the index coordinates and the map scales in index units, the shelter tables in the linear groups, the two family masks, feature-support coverage — DONE 2026-10-08
    - [x] **C.3.4** Refit on `hce-v4-tb`, static screens (the king family's residual re-read, the magnitude read), the tree read, PGO bake, gate-1 registration and handover — DONE 2026-10-08
    - [x] **C.3.5** Margin block, gate 2, the `10+0.1` read, the ledger row — DONE 2026-10-09
    - [x] **C.3.6** Same-search deficit re-read on the accepted unit: the oracle package rebuilt with the C.3 evaluation in `rarog_hce.dll`, 1,000 games at 150,000 nodes a move against the Stockfish control; decides whether C.4 to C.7 open — DONE 2026-10-09
- [x] **C.4** Threats and mobility cluster: mobility area, weak enemies, hanging, restricted, pawn push, queen threats; refit; gate — DONE 2026-10-11
    - [x] **C.4.1** Residual step: the donor's threats and mobility families against the C.5.2 head (RAR-E30) — DONE 2026-10-10
    - [x] **C.4.2** Sub-term attribution: the donor's threats components and its mobility under four area definitions, rebuilt offline and verified exactly, read as directions (RAR-E31) — DONE 2026-10-10
    - [x] **C.4.3** Threats unit: the king threat and the safe-square slider threat on the queen in the donor's shape, Rarog's x-ray slider term removed, one strongly-protected set shared; refit; static screens; tree read; PGO bake; gate 1 — DONE 2026-10-11
    - [x] **C.4.4** Margin block when flagged, gate 2, the `10+0.1` read, the ledger row — DONE 2026-10-11
- [ ] **C.5** Endgame handling and winnability cluster — **R3**
    - [x] **C.5.1** Classification and deciding instrument per family — DONE 2026-10-09
    - [x] **C.5.2** Generic winnability and scaling: pawn count, opposite bishops, rule-50 scale, complexity — DONE 2026-10-10
        - [x] **C.5.2.1** One opposite-bishop rule across the six-to-seven-men boundary, the six-man wins kept by position knowledge — NO_CHANGE 2026-10-11
        - [ ] **C.5.2.2** The opposite-bishop boundary hazard: the two-regressor read, and one rule replacing the split if it holds — **RESEARCH / R2**
    - [x] **C.5.3** Conversion cluster: KXK, KBNK, KQKR; rule-50 damping interaction measured — NO_CHANGE 2026-10-11
    - [x] **C.5.4** Rook versus minor cluster: KRKN, KRKB, KRPKB — NO_CHANGE 2026-10-11
    - [ ] **C.5.5** Rook and pawn cluster: KRPKR, KRKP, KPK, KPKP audit — **RESEARCH / R2**
    - [ ] **C.5.6** Measure-first families: KPsK, KBPsK, KBPPKB, KQKRPs — **RESEARCH / R2**
    - [ ] **C.5.7** Theory sweep: KBPKB, KBPKN, KNNKP, KNNK, KQKP from one dispatcher — **RESEARCH / I1**
    - [ ] **C.5.8** Endgame checkpoint: conversion re-read, floors, theory vetoes, the KRPPKRP exclusion — **RESEARCH / V**
- [ ] **C.6** Pawns and passers cluster; refit; gate — **I2**
    - [x] **C.6.1** Residual step: the donor's passed and pawns families on the current head, by cohort (RAR-E39) — DONE 2026-10-11
    - [x] **C.6.2** The unstoppable-passer test against Syzygy and in the corpus (RAR-E40) — DONE 2026-10-11
    - [x] **C.6.3** Sub-term attribution: the donor's passed and pawns components rebuilt offline and verified exactly, read as directions — DONE 2026-10-11
    - [x] **C.6.4** Pawns-and-passers unit: block-square king proximity replacing Rarog's form, weak unopposed pawns, the square rule; trace slots, tuner lists, manifest v5 — DONE 2026-10-11
    - [x] **C.6.5** Refit on `hce-v4-tb`, static screens, tree and NPS reads, PGO bake, gate-1 registration and handover — DONE 2026-10-11
    - [ ] **C.6.6** Gate 1's read, the margin block when flagged, gate 2, the `10+0.1` read, the ledger row — **RESEARCH / V**
    - [ ] **C.6.7** Investigation: where Rarog's pawn and passer definitions differ from the donor's, which differences carry outcome information, and what else should change — **RESEARCH / R2**
- [ ] **C.7** Material, imbalance, phase and pieces cluster; refit; gate — **RESEARCH / I2**
- [ ] **C.8** Refit cycles: regenerate, refit, gate; stop at the first non-accepting cycle — **RESEARCH / V**
- [ ] **C.9** HCE SPSA of nonlinear residue, or a written skip — **RESEARCH / V**
- [ ] **C.10** Joint search SPSA after the new evaluation: cp margins plus every mechanism whose firing rate moved 10% or more (the whole-surface tune B.6 left for here); rule-7c blocks; SPRT `[0,3]` — **RESEARCH / V**
- [ ] **C.11** Checkpoint: same-search deficit, conversion, NPS, pool gauntlet; freeze the classical evaluation — **RESEARCH / V**

## Phase D — Clock, threads, robustness

- [ ] **D.1** Time management: audit against the ADR-0065 checklist, soft/hard bounds with node-fraction multiplier, forfeit margin; SPRT `[0,3]` — **R2**
    - [x] **D.1.1** Time-management diagnosis, pulled forward: rec1's stalled re-search, rec2's search past a found mate, rec3's clock-independent stop; mechanism, frequency in games, the frozen fix and its gate — NO_CHANGE 2026-10-06
    - [ ] **D.1.2** Time-management audit and bound model: the ADR-0065 checklist, soft and hard bounds with the node-fraction multiplier, the forfeit margin, the won-ending stalls (RAR-R13); SPRT `[0,3]` — **R2**
        - [ ] **D.1.2.1** Won-ending time stalls: rec1's cascade, rec2's missing stop after a proved mate, rec3's overrun (RAR-R13) — **R2**
        - [ ] **D.1.2.2** The forfeit margin against OS holds — **R2**
        - [ ] **D.1.2.3** The audit and the bound model: the ADR-0065 checklist, soft and hard bounds with the node-fraction multiplier; one SPRT `[0,3]` for D.1.2 — **R2**
- [ ] **D.2** Lazy SMP quality at 4T/8T: diversity, shared TT and correction, soft-stop voting; 4T SPRT `[0,5]`; its premise is contradicted by RAR-M46, so re-scope first — **R2**
- [ ] **D.3** Engine lifecycle and protocol robustness; `src/uci/` (planned) move; score normalisation research card (`analysis/uci_info_review_2026-09-16.md` item 6); zero crashes over pool tournaments — **R2**
- [ ] **D.4** Tablebase policy: probing depth/limits, WDL/DTZ in conversion, recogniser interaction — **R2**

## Phase E — Classical checkpoint and release

- [ ] **E.1** Attribution checkpoint: B.2.0 review re-run on the B.9/C.11 heads; STC, `10+0.1`, 4T against 2.3.2 and the B.9/C.11 heads; maturity checklist — **V**
- [ ] **E.2** Target gate: ≥50% against Critter 1.6a, Houdini 3, Rybka 4.1 and Fritz 16 at 1T and 4T (Rybka 4.1 replaced Rybka 4, 2026-09-19); the binding arm is 1T — **V**
- [ ] **E.3** Release 3.0.0 (E.2 met) or 2.6.0 through the tag-driven flow — **M**
    - [x] **E.3.1** Tag-driven release flow — DONE 2026-10-05
    - [ ] **E.3.2** Release cut: version 3.0.0 (E.2 met) or 2.6.0, the `[Unreleased]` changelog reviewed and dated, suites, the release PR squash-merged, the `v` tag pushed on instruction through E.3.1's workflow — **M**

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
| Development head | `master` (the trunk since 2026-10-09, when `dev` retired), version **2.6.0-dev**: the 2.5.0 engine plus behaviour-neutral source changes recorded under PLAN D.3's change log (latest `6433760`, each with the exact fingerprint) and C.1's evaluation restructure into `src/eval/` (`801f1b4`, exact), then C.2's lazy gate in the `texel` build (`79c6808`, texel builds only, exact), then RAR-E24's accepted baseline (`1abeb46`, 2026-10-08: the evaluation refitted on `hce-v4-tb` with the `c2margin` block's 28 margins, +35.9 ± 9.3 Elo at `3+0.03`, +39.1 ± 12.9 at `10+0.1`), then C.3's king-safety unit (RAR-E25, `dcef971`, 2026-10-09: the donor-shaped danger index and shelter and storm by file and rank, refitted on `hce-v4-tb`, with the `c3margin` block's 28 margins; gate 2 +32.7 ± 8.9 Elo at `3+0.03`, +24.4 ± 12.3 at `10+0.1`), then C.5.2's opposite-bishop rule (RAR-E29, `78deb3d`, 2026-10-10: the cohort SPRT `[0,10]` H1 at +12.7 ± 9.8 nElo, the STC harm read +8.3 ± 15.2 nElo), then C.4.3's threats unit (RAR-E32, `6c73d65`, 2026-10-11: the king threat and the safe-square slider threat on the queen, the x-ray slider term removed, refitted on `hce-v4-tb`; gate 1 +13.0 ± 5.3 Elo at `3+0.03`, a first-crossing estimate of which about +5 is the +2.57% NPS from the removed per-evaluation x-ray loop, at 2 Elo per 1%), both landed on `master` in `36f04d5` (PR #7, 2026-10-11) at bench 12,567,796; then, on branch `c5` and not yet on `master`, C.6's pawns-and-passers unit (RAR-E42, `38cbb02`, 2026-10-12: passer king proximity at the block square, weak unopposed pawns and the square rule for unstoppable passers, refitted on `hce-v4-tb`; gate 1 (RAR-E42, `[0,3]` at `3+0.03`) H1 at +6.4 ± 3.6 Elo, +10.7 ± 6.0 nElo (12,924 games; a first-crossing estimate), no screen flag, NPS +0.07%); fingerprint **14,197,472 / EBF 2.557**; `rustc 1.98.1`. A registration names the revision it builds from and that fingerprint. The search code is frozen for Phase C at engine source `ee02ed1` (the measured binary and its SHA-256 are in PLAN's Phase C rules): a C-phase change touching search code returns to its owner leaf and is gated as a search change, and search coordinates move only through a unit's margin block (accepted by its gate 2) or C.10's joint tune |
| Pool position, `3+0.03` 1T | **3,286.5 ± 15.7** (RAR-M64; 2.4.0 rates 3,001): Houdini 3 +37.5, Critter 1.6a +96.2, Fritz 16 +87.8, Rybka 4.1 +167.3, so all four E.2 targets pass at 1T |
| Pool position, `3+0.03` 4T | **3,340.0 ± 16.7** (RAR-M66): Houdini 3 +113, Critter 1.6a +148, Fritz 16 +132, Rybka 4.1 +210 |
| Search deficit | Closed: G(0) **+24.24 ± 8.01** against the oracle (RAR-O04), from −247.97 at 2.4.0 (RAR-O03) |
| Evaluation deficit | **+129.4 ± 17.1 Elo at equal nodes** on the C.3 evaluation (RAR-O06, 2026-10-09), the current meter: Stockfish's classical HCE over Rarog's inside Stockfish's search, 52.4 ± 25.6 below Phase C's baseline of +181.7 ± 19.0 at equal nodes and +266.3 ± 19.9 at equal time on the 2.4.0 evaluation (RAR-O05, 2026-10-05; equal time not re-read). RAR-O02's earlier about 329 was the 2.3.2 evaluation with unequal throughput. **Caveat (C.0 audit, 2026-10-06):** measured inside a search fitted to Stockfish's evaluation shape, where a shape mismatch alone costs about 105 Elo per node (RAR-E19/E20); the recoverable share is unknown; RAR-O06 refuted little transfer of Rarog's own gains into that search |
| Evaluation residual (static layer) | On 194,444 held-out positions Stockfish's total adds **3.59%** to Rarog's outcome prediction and its term families **1.22%**; king safety +0.66% (+1.58% in the middlegame band), winnability +0.12%, mobility, pieces, material and space nil (RAR-E17, RAR-E18). The king residual is the donor's danger map (92% of it; inside the map the safe checks and the weak ring), not shelter or the flank terms; Rarog's zeroed danger inputs are a resolution limit of its bucket table; the scale factor adds +0.09% above six men (RAR-E21, 2026-10-06). With one Stockfish version for families and total the families read 1.85% and carry the whole 2.73% at seven men or more, so there is no total-level knowledge above six men to chase (RAR-E22). On the C.5.2 head (RAR-E30, 2026-10-10, `hce-v4-tb` labels): threats +0.107 ± 0.020 over all rows and +0.122 at seven men or more, mobility +0.048 (+0.184 in the opening band), king +0.079, the whole donor +2.49. It ranks questions; it is not Elo |
| Speed | **2.33 MNPS**, −26.23% against 2.4.0 (RAR-P35); the host drifts between days, so compare pools interleaved only |
| Conversion | **15.8 draws and 0.8 losses per 1,000 games** after a persistent piece-up (RAR-M64's games; 2.4.0: 24.4 and 5.3) |
| Evaluation–search coupling | Feeding the frozen search the full fitted evaluation above the lazy gate costs **−104.5 ± 10.6 Elo at equal time and −110.0 ± 11.5 at equal nodes** (RAR-E19, RAR-E20, 2026-10-05/06), though that function predicts outcomes better statically: the cost is per node, in a search fitted to the played function. Every evaluation unit goes through PROCESS's *Evaluation change under a fitted search*: static screens, tree read, gate 1, a margin block over the fixed 28-coordinate surface (`LazyMargin` added by the C.0 audit) when gate 1 fails or a screen flagged, gate 2 (accepts), a `10+0.1` read (maintainer decision 2026-10-06). When gate 1 passes with no flag, the agent asks the maintainer whether a free night makes the block worth running anyway (2026-10-08, after RAR-E24's block measured +19.3 ± 9.3 Elo) |
| Active experiment | **RAR-E42** (C.6.5–C.6.6, the pawns-and-passers unit): fitted 2026-10-11 to **14,197,472 / EBF 2.557** (`38cbb02`); screens with no stop and no flag (the donor's passed residual in the endgame band +0.204 → −0.015, pawns unchanged at +0.120, tree +1.7%, NPS +0.07%); **gate 1 accepted 2026-10-12, H1 at +6.4 ± 3.6 Elo** (12,924 games, a first-crossing estimate, no fault); margin block skipped (maintainer decision 2026-10-12); the `10+0.1` read (1,000 games) is registered and with the maintainer. Last closed: **RAR-E41** (C.6.3), 2026-10-11, zero games: the donor's passed and pawn components rebuilt exactly on all 194,444 rows; our king's distance to the block square (+0.131 ± 0.017 in the endgame band) and weak unopposed pawns (+0.060 ± 0.013) carry the residual, the free path is pricing; C.6 is `READY_FOR_IMPLEMENTATION` (C.6.4). Before it: **RAR-E40** (C.6.2), 2026-10-11, zero games: on Syzygy truth the square rule's counted passers are won 0.740 against the engine rule's 0.683, so the square rule joins C.6's unit; in the corpus the disagreements were won 84% anyway and the lazy gate hides 85% of the term's firings in play. Before it: **RAR-E39** (C.6.1), 2026-10-11, zero games: the donor's passed family reads +0.204 ± 0.031 in the endgame band and +0.196 without queens, pawns +0.121 and +0.141 there; both nil above |score| 500 and with queens on; both go to C.6.3's attribution. Before it: **RAR-E38** (C.5.4, before any design), 2026-10-11, zero games: in RAR-E32's gate games KR-KN or KR-KB occurs in 1.43% and KRP-KB in 0.78%, no trade into them changed its mover's Syzygy result, wins 55 of 55 and drawn entries 65 of 68 held; expected value at most 0.18 Elo, C.5.4 closed `NO_CHANGE` on the maintainer's confirmation, with KRP-KB's win-preservation hold. Before it: **RAR-E37** (C.7's question from C.4.3's screen), 2026-10-11, zero games: the pieces rise after the x-ray term's removal is not the donor's weak-queen row (0.39 of +0.035; the rest in the knight, bishop and rook rows); nothing scheduled. Before it: **RAR-E36** (C.5.4's residual step), 2026-10-11, zero games: the rook-against-minor overclaim persists (KR-KN 98.4%, KR-KB 94.6% of draws above +100 cp), the wins convert; C.5.4 continues into a scale candidate. Before it: **RAR-E35** (C.5.3's transitions), 2026-10-11, zero games: the mate drive's boundary is a large static jump (KBNP-K +595 cp median without the winner's own pawn) that play does not feel (conversion and speed level with Stockfish); C.5.3 stays closed. Before it: **RAR-E34** (C.5.3), 2026-10-11, zero games: the old conversion deficits are gone on the current head (KQ-KR 23/13/3 → 1/0/0 behind Stockfish); KBN-K 2 of 98 behind at 600k only; C.5.3 closes `NO_CHANGE`. Before it: **RAR-E33** (C.5.2.1), 2026-10-11, zero games: free passed pawns separate won from drawn opposite-bishop endings but one rule on them loses above the boundary (−3.5 ± 0.9%); C.5.2.1 closes `NO_CHANGE`. Before it: **RAR-E32** (C.4.3 and C.4.4, the threats unit), accepted 2026-10-11 on gate 1, H1 at +13.0 ± 5.3 Elo (5,898 games), no screen flag; margin block skipped (maintainer decision 2026-10-11); `10+0.1` +12.9 ± 12.2 Elo (1,000 games); C.4 closed. Before it: **RAR-E31** (C.4.2, the sub-term attribution), 2026-10-10, zero games: the king threat and the safe-square slider threat on the queen carry the donor's threats residual (−0.108 and −0.047 points by removal), the ungated family loses −0.062, and the donor's mobility definition over Rarog's reads +0.144 ± 0.039 in the opening band, 3.7 standard errors against the registered four; C.4.3 is `READY_FOR_IMPLEMENTATION` with the two threat terms, mobility closed `NO_CHANGE` inside C.4. Before it: **RAR-E30** (C.4.1, the residual step), 2026-10-10, zero games: on the C.5.2 head the donor's threats family reads +0.107 ± 0.020% over all rows and +0.122 at seven men or more (signal), mobility +0.048 (signal in the opening band only, +0.184), king +0.079; C.4 continues into the attribution (RAR-E31, registered before any model is fitted). Before it: **RAR-E29** (C.5.2's gate): the opposite-bishop cohort SPRT `[0,10]` H1 at +12.7 ± 9.8 nElo (4,792 games), the STC harm read +8.3 ± 15.2 nElo (2,000 games); C.5.2 accepted, fingerprint 13,562,404 / EBF 2.546 |

## Holds and obligations

| Open hold / obligation | Resume or resolve when | Must be resolved before |
|---|---|---|
| Tags `archive/pr2-version-2.5.0`, `archive/pr3-tag-driven-release`, `archive/pr4-release-checks`: the `dev` chains squash-merged as PRs #2–#4 | Confirmed by the citation sweep (2026-10-09): every commit they hold is under PRs #2–#6 (`refs/pull/<n>/head`) and `check_citations.py` passes without them, so they hold nothing the documents need; the maintainer deletes them (`git push origin --delete archive/pr2-version-2.5.0 archive/pr3-tag-driven-release archive/pr4-release-checks`, then `git tag -d` each) | C.11's ref review |
| Won-ending time stalls (rec1 cascade, rec2 no stop after a proved mate, rec3 overrun; RAR-R13): real, no measured cost at `3+0.03`/`10+0.1`. Forfeit-margin evidence for D.1.2: the three time losses in gates since 2026-10-10 (RAR-E29 round 3068, RAR-E32 rounds 1777 and 1855; Colosseum's `failed-games` records) each came on a short search (2, 15 and 32 ms of engine time) whose process the OS then held for 88.0, 82.4 and 69.4 ms, so the move arrived at 94.3, 99.9 and 90.8 ms against deadlines of 89.2, 69.9 and 75.0 ms; all three in games already decided on the board. A margin under about 90 ms with that little clock left does not cover one such hold | Scheduled as PLAN **D.1.2.1** (the stalls) and **D.1.2.2** (the forfeit margin), first in D.1 after Phase C, gated with D.1.2.3; earlier if RAR-R13's trigger fires (a game record shows a cost) | D.1.2.1 and D.1.2.2 close (owners) |
| KRPPKRP 7-man truth gap | **Decided 2026-10-11: explicit exclusion** (PLAN C.5.5). The resume condition fired on 2026-10-09 (`syzygy7` holds KRPPvKRP and python-chess probes it), but extending the instruments to seven men for this one family costs about 5 to 6 hours, over the maintainer's half-day line; the family is read through held-out loss at seven men and C.5.8's game-level conversion. Retry on a KRPP-KRP-specific candidate or a game record showing a conversion loss there | C.5.8 records the exclusion |
| Opposite-bishop boundary hazard: C.5.2's rule scales pure endings of seven men or more hard and keeps the old rule at six or fewer, so one exchange can roughly double the score. Measured small: in RAR-E29's cohort the candidate made 280 first crossings into the six-man endings against the head's 260, and those games were not more drawn (77.9% against 82.1%); RAR-E33 closed the free-passer fix `NO_CHANGE` | Scheduled as PLAN **C.5.2.2** (the two-regressor read, passers and free passers in one rule at every men count, RAR-E33's addendum), after C.6 and before C.8; earlier on a game record showing a cost at the boundary | C.5.2.2 closes (owner), before C.8 |
| Unstoppable-passer test one tempo generous in both move orders (`src/eval/passers.rs`; confirmed in the source by C.0, 2026-10-05) | C.6 opens: definition change plus refit, checked on tablebase-labelled pawn endings | C.6 closes (owner) |

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
