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

| Class | Capability | Model — thinking mode |
|---|---|---|
| `R3` | Frontier causal/architecture research | Claude Fable 5.1 — High |
| `R2` | Bounded correctness-sensitive reasoning | Claude Opus 5 — High |
| `I2` | Difficult implementation | Claude Opus 5 — High |
| `I1` | Well-specified implementation | Claude Sonnet 5 — Medium |
| `M` | Mechanical/docs/provenance | Claude Sonnet 5 — Medium |
| `V` | Verification/measurement | Claude Sonnet 5 — High |

Claude models only, by maintainer decision 2026-09-25.

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
| Development head | `dev`, version **2.5.0-dev**; fingerprint **11,171,726 / EBF 2.512** — cluster 3 (quiescence, `b4quiet`) accepted by RAR-S88's gate on 2026-09-30 at +4.4 ± 2.9 Elo and made the default in `5a5c150` (before it, cluster 2, proof searches and extensions, read 12,897,901 / EBF 2.523 from `f53ca7d`, accepted 2026-09-27 at +50.5 ± 10.9; the selectivity core with RAR-S78's theta read 7,435,006 / EBF 2.457 from `52c46df`, accepted 2026-09-25 at +13.1 ± 5.4; B.2.3's theta read 7,185,678 / EBF 2.444 from `14a7079`, the unfitted core 4,706,910 / EBF 2.391). B.8 deleted the superseded B.1 search in `84de712` (2026-10-03); `--no-default-features` built it, reporting `+legacy`, and its last fingerprint was 7,590,542 / EBF 2.473 (2.4.0's 7,601,220 until `5a5c150` fitted three quiescence thresholds it shared with the core). B.1, B.2.0 and B.2.0.2's MultiPV were behaviour-neutral at that fingerprint (RAR-P24, RAR-P25, RAR-P26), and B.2.2.1's clamp conversion (6,586,667) was reverted by `308abe9` after RAR-S74's run (g). Last verified on Windows ARM64 and macOS ARM64 at the 2.4.0 head (RAR-P19); pinned `rustc 1.98.1` |
| Pool position, `3+0.03` 1T | Houdini 3 −224, Critter 1.6a −184, Houdini 1.5a −179, Fritz 16 −147, Rybka 4 −99, Basilisk 1.10.0 −23, Basilisk 1.9.3 −9, Rarog 2.3.2 +70 (RAR-M45, 2026-09-11, 600 games/pair, 2.4.0 release); reproduced within the 200-game bands in the 42-engine Super Rating Tournament, where 2.4.0 scores 64.8% and Houdini 4 −323, Stockfish 5 −222, Stockfish 1.9.1–4 within ±35 (RAR-M54, 2026-09-15) |
| Pool position, `3+0.03` 1T, **2.5.0-dev** | Rating **3233** against 2.4.0's 3001 (the cluster-2 head, `2.5.0-dev+b3proof`, 12,897,901, native; 40,000-game gauntlet, pool held at its Super Rating Tournament ratings, RAR-M63, 2026-09-28; RAR-M57's fit at 3,900 read 3191): Houdini 4 −52, Houdini 3 −30, Stockfish 5 +16, Shredder 13 +21, Critter 1.6a +33, Houdini 1.5a +34, Fritz 16 +60, Rybka 4.1 +105, Basilisk 1.10.1 +200, Rarog 2.4.0 +225; E.2 at 1T: Rybka 4.1, Fritz 16 and Critter pass, Houdini 3 does not |
| Pool position, `3+0.03` **4T** | Houdini 3 −169, Fritz 16 −149, Critter 1.6a −109, Rybka 4 −73, Basilisk 1.10.0 **+25**, Rarog 2.3.2 +45, Rybka 3 +79; Perf 3034 vs frozen 3003 (RAR-M46, 2026-09-11) |
| Search deficit | **247.97 ± 10.89 Elo** equal time against the frozen oracle on the 2.4.0 head, evaluation proved constant (RAR-O03); non-mate depth gap 1.4 ply by the mean (the recorded 0.97 was mis-attributed, corrected 2026-10-03), so most of it is still decision quality; selectivity explains 272 ± 18. At equal nodes: WAC **200 vs 242** solved at 100k, median depth **16 vs 19** at 300k; Rarog's branching factor 1.630 is already below the oracle's 1.736 (RAR-M50) |
| Evaluation deficit | **about 329 Elo** against Stockfish's classical HCE with the same search |
| Speed | **3.27 MNPS** pooled median at the B.1 head, **+6.30% [+5.78%, +6.84%]** over the 2.4.0 pool (RAR-P24); the B.2.0 head **+0.21% [−0.34%, +0.68%]** over the B.1 pool (RAR-P25); the B.2.0.2 head **+0.62% [−0.29%, +1.18%]** over the B.2.1 head pool (RAR-P26). This host drifts by several percent between days, so compare pools interleaved only; Basilisk 3.71 (RAR-M48) |
| Conversion | **88 draws + 19 losses** after a persistent piece-up in 3,600 games vs the six HCE-era engines on the 2.4.0 release games; rate unchanged from the 2026-09-04 pool (57 + 12 in 2,400). Basilisk 1.9.3 in the same tournament 94 + 12, so RAR-M47's surplus reading is retired (RAR-M49, 2026-09-13, zero games); third sample 24.2 / 3.3 per 1,000 against the same six in the Super Rating Tournament, Basilisk 17.5 / 5.0 (RAR-M54) |
| Active experiment | **RAR-S73** (B.2 selectivity core: B.2.4a passed 2026-09-16, +65.09 ± 23.26; RAR-S75 finished 2026-09-20 (5,000 iterations, theta baked at 7,185,678); RAR-S76 peek at 3,900 +118.72 ± 10.62; RAR-M55 and RAR-M56 (B.2.3.2 closed: fitted +97.69, unfitted −7.12 vs Rybka 4.1), **B.2.4b passed 2026-09-20, +138.60 ± 30.66**; RAR-S77 (B.2.3.3) passed, +4.43 ± 2.90; RAR-S78 (B.2.7) tune finished 2026-09-24, theta baked, **gate H1 2026-09-25, +13.1 ± 5.4 Elo** in 3,081 pairs, B.2 closed; close-out audit 2026-09-21, `analysis/b2_audit_2026-09-21.md`); **RAR-M45, RAR-M46, RAR-O03 and RAR-M48 all resolved 2026-09-11**; RAR-M49 conversion re-read and RAR-M50 (B.0 measurements) recorded 2026-09-13; RAR-M54 (Super Rating Tournament read, 42 engines) recorded 2026-09-15, nothing moves. **D.2's premise is contradicted by RAR-M46 and the leaf needs re-scoping** |
| Current step | **B.9** (`V`), `RESEARCH`: the checkpoint (G(0), depth/EBF, NPS, conversion, pool gauntlet; remove `ablate`; freeze the search head); not started. **B.8 closed 2026-10-03:** the B.1 search, the arm features `b2core`, `b3proof` and `b4quiet`, 64 dead coordinates, the fourteen decided switches, 73 unowned counters and the dead PVS guard removed in eight engine commits, each at `bench 13` 11,171,726 / EBF 2.512; the no-regression read (RAR-P34) +1.16% NPS (95% t-interval +0.72% .. +1.59%); what each removed item did, how it was decided and how to restore it is in `analysis/b8_removed_2026-10-03.md`; acceptance waits on the auditor's verification. **B.7.3 closed 2026-10-03, `NO_CHANGE` (RAR-P33):** the SEE recapturer's exact rewrite (`8fc8ff9`, oracle test of 4,411,664 comparisons, bench exact) read **−2.09%** NPS (95% t-interval −2.34% .. −1.84%) against the head's pool, closed by step 1 of the two-step read and reverted in `7e6e0ef`; B.7 closed with it. Phase C carries speed as a secondary requirement from 2026-10-03 (PLAN, Phase C's goal). **B.7 closed 2026-10-02 by the maintainer, its sweep verified by the auditor from the artifacts (RAR-P31), and confirmed in games (RAR-S98): H1 at 1,051 pairs, +35.5 ± 9.0 Elo against the head before B.7, +16.7% nodes a second inside the games.** The speed pass, B.7.2.1–B.7.2.20 on the board. **Accepted (RAR-P28):** candidate 1, the quiet-stage early exit (`5fe42cf`), +8.85% NPS; candidate 2, the in-place scored list (`0d95763`), +6.93% over it; bench exact. The re-profile leaves `pick_next` at 6.26%. Candidate 3, the scan layout, closed `NO_CHANGE` by the maintainer after its falsifier read `s` = 1.106 and 1.093. **B.7.2.13, the audit's screen, done 2026-10-02 (RAR-P30):** the TT prefetch before the make (A) read +1.48% (CI +0.91% .. +2.41%) and goes forward; the correction-slot prefetch (I) read +0.14% (−0.29% .. +0.75%), between the bounds, and is implemented after A by the maintainer's decision; the history-row prefetch (D, −1.87%) and the static attack tables (B, −0.49%) close `NO_CHANGE`; all four together +1.71%. **B.7.2.14–B.7.2.16 done 2026-10-02 (RAR-P31):** A (`a42fadc`) accepted at +1.69% (95% CI +1.56% .. +1.89%), bench exact. **B.7.2.17–B.7.2.20 done 2026-10-02:** I read +0.37% over A, under the floor, and was reverted (`bec1292`, `NO_CHANGE`); the total, A's pool against the c2 pool again, +1.78% (+0.92% .. +2.56%). Profiled 2026-10-01 (RAR-P27, `analysis/b72_profile_2026-10-01.md`). **B.6 closed 2026-10-01, `NO_CHANGE`, by the maintainer** (RAR-S95, `analysis/b6_research_2026-10-01.md`): the finished tunes' journals show no gradient left where each stopped (RAR-S78's second half and every later block read as random walks; only `QsFutilityMargin` still travels), a read that cannot see a gain of a few Elo; a restart was estimated at +3 ± 3 Elo for 12.5 hours a block plus a gate, so the joint search SPSA is C.10, after the classical evaluation. **B.5 closed 2026-10-01 with B.5.2**: the maintainer kept B.5.2.1 (RAR-S93 converted 453 of 453 clean tablebase wins on both sides; RAR-S94 H1 at 776 pairs, +11.4 ± 8.5 Elo with tables). Two registered stop conditions had fired by their letter. The box rule is recorded as unsatisfiable for a box checked between Fathom calls: a replay put the 50 overruns on single calls of up to 16 ms that the page cache misses, so a predictive box was not built. The cold-table time loss at a 7-man root goes to D.3. Out of band 2026-10-01: the last `info` line always describes `bestmove` (`97ea52d`, the flaky threaded test's cause, now with a deterministic test), the legacy search builds again (`2326153`), and a test-suite audit is recorded under B.8 for the maintainer's decision. **B.5.2.1 done 2026-10-01**: the root keeps its DTZ-best group, probes are off at a DTZ root and bound-correct elsewhere, tablebase values are decisive and print `cp ±20000`, the hard limit is the optimum at a DTZ root, PVs extend through the tables and give the ponder move; KQvK/KRvK are committed so CI tests it; bench 11,171,726 after every commit. The stop rule refuted moving band values by ply in the table (window arithmetic reaches the band without tables), so the table keeps them as stored. **B.5.2's investigation closed 2026-10-01, `READY_FOR_IMPLEMENTATION`**: the one-move PV has three causes (the root cut, the single-move shortcut stopping at depth 2, the at-once zeroing-child probe); probed tablebase values were never decisive and the TT ignored their band; without the cut a tablebase root fell into a won-endgame time sink (30 s of 60, D.1's input), so the root's hard limit becomes the optimum; Fathom is safe for root DTZ probes beside helper WDL probes; the extension costs 1.3–7.5 ms warm. The fixture (KQvK, KRvK, 14,080 bytes) waits on the maintainer's licence decision. **B.5.4 closed 2026-10-01, `NO_CHANGE`**: optimism read −5.6 ± 9.4 Elo in RAR-S92's 2,000 games; the donors' gain did not transfer to a classical evaluation. **B.5.1 closed 2026-10-01, `NO_CHANGE` on both cards**: (a) the premise was false, because the default search has given the TT-cutoff history bonus since B.2 in Reckless's form, fitted twice; (b) draw-score randomisation (`CoreDrawJitter`) read −1.2 ± 9.4 Elo in RAR-S91's 2,000 games against a prediction of +1 ± 10, so it is undecidable at this budget and stays off (`analysis/b51_research_2026-09-30.md`). **B.5's root cluster closed 2026-09-30, `NO_CHANGE`**: one-retry aspiration was rejected at −18.3 ± 6.9 (RAR-S89) and lost the same at `10+0.1`; a window twice as wide read −25.4 ± 9.2 and a narrower one +2.1 ± 9.2 (RAR-S90), so the loop is kept and no tune is owed; zero-game reads had pointed the other way. The research's inventory leaves the time management multipliers (D.1) and six single coordinates (B.6) as what was never re-fitted on this search (`analysis/b5_research_2026-09-30.md`). **B.4 closed 2026-09-30**: RAR-S88's gate accepted H1 at 10,705 pairs, +4.4 ± 2.9 Elo; theta baked, evasion pruning on and `b4quiet` the default in `5a5c150` (11,171,726 / EBF 2.512). **B.4.3 closed 2026-09-30**: the tune's block 1 moved one coordinate a full step (`QsFutilityMargin` 150 → 178) and the stop rule ended it; evasion pruning was adopted by RAR-S87 at +7.8 ± 9.4 Elo (quiescence history off at −4.7 ± 9.4); the sweep read 2026-09-29 found `QsSeeClampHi`, `QsDeltaMargin` and `QsStandPatLerp` curved, and the preparation's review added `CoreRazorSquare` and `CoreNmpBase` and took `CoreRfpLinear` off its rail. **B.4.2 closed 2026-09-29**: the unfitted paired run read +0.7 ± 9.3 Elo in 2,000 games; the component ablation puts the failed floors on the interpolation pair through the estimate B.2's reverse futility was fitted on, so the estimate's first consumers join B.4.3's surface; the implementation was reviewed with no defect and CI covers the arm. **B.4.1 closed 2026-09-29**: the arm at its defaults reads 10,226,874 / EBF 2.519, WAC 236 / 270, canaries 92 with none lost, time-to-depth 0.85×; floors failed on depth-14 nodes, agreement and b15's cost, as B.3.2's did; T3 and T4's switches default off by the canary rule. Earlier: **research amendment 1 closed 2026-09-29** (packet, RAR-S85) after T1 (`ac018d4`) tripped the canary rule on 2026-09-28 — the rule is kept (0 of 12 neutral perturbations of the head lose a canary), Q1 becomes the Stockfish stored-bit form, Q2's seeds the Stockfish pair 583 / 562; the amended T1 reads bench 10,953,303, WAC 235, canaries 92 with none of the 91 lost, and its commit must reproduce those numbers. **B.4's research closed 2026-09-28** (RAR-S85, `analysis/b4_research_2026-09-28.md`): on the cluster-2 head the quiescence reads 0.33 qnodes per interior node against the oracle's 0.58, WAC 232/266 at 100k/400k, 91/116 canaries, branching 1.800; contract Q1–Q9, screens and predictions registered; the diag counters landed in the engine commit. **B.3 closed 2026-09-27**: B.3.5's ponder race fixed in `ef1a24b` (bench unchanged; ponder-on smoke 400 games, 0 faults). **B.3.4 closed 2026-09-27**: RAR-S84's gate accepted H1 at 804 pairs, +50.5 ± 10.9 Elo, the direction read +48.1 ± 9.5; theta baked and `b3proof` the default in `f53ca7d`. **B.2 closed 2026-09-25**: B.2.7's re-tune (RAR-S78) was baked (`52c46df`, 7,435,006 / EBF 2.457) and its `[0,3]` gate accepted H1 at +13.1 ± 5.4 Elo in 3,081 pairs. **B.3.1** (cluster 2 implementation, `I2`) was **implemented on 2026-09-23**: the `b3proof` arm reads 7,479,114 / EBF 2.467 with the off arm exact, and CI covers it (`analysis/b3_research_2026-09-23.md`, *Final implementation record*). **B.3.2** closed 2026-09-24 (paired run +18.4 ± 9.4 Elo; RAR-S80's categoricals adopt `SingularTtDepthMargin=2` alone); **B.3.3**: the bake (`3ca9aab`, arm 7,978,292 / EBF 2.465) and the curvature sweep are done (3 of 7 coordinates curved, `analysis/b33_sweep_2026-09-24.md`). B.2.6 closed 2026-09-22. **E.3.1** (tag-driven release flow, `I1`, added 2026-09-22) is independent tooling that may land at any point before E.3 |
| Next release | **3.0.0** if the E.2 target gate is met, otherwise 2.5.0 — cut at E.3 after the search and evaluation programmes. Nothing is released between now and that checkpoint unless a correctness repair forces a patch |

## Next and held work

**B.2 passed both gates.** B.2.4a on 2026-09-16 (+65.09 ± 23.26 in 432
games) accepted the unfitted core over the off arm; B.2.4b on 2026-09-20
(+138.60 ± 30.66 in 248 games) accepted the B.2.3-fitted arm over it. The
**fitted `b2core` arm at 7,185,678 / EBF 2.444 is the accepted head** and
the base for B.3, and it became the default build on 2026-09-20. B.2.3's
tune finished at N = 5,000 on 2026-09-20; its tail, the last 1,100
iterations, measured +4.43 ± 2.90 in RAR-S77. B.2.2 closed on 2026-09-15:
- the unfitted paired run measured +52.16 ± 10.73 Elo;
- no switch was adopted and mate-residual training stays;
- the clamp conversion was reverted to coordinates;
- the curvature sweep found three curved coordinates;
- the screen ladder for B.3–B.5 is now rule 8's *Cluster screens*.

**Colosseum CLI is the main harness** since 2026-09-21 (B.2.6.2): gates, fixed
matches, tunes, null pairs and gauntlets run through `tools/colosseum.ps1` from
the committed run files, with the runner pinned by revision and SHA-256 —
since 2026-09-25 the published `cli-v0.2.0` (`ca05dfa`; `cli-v0.1.0` from
2026-09-22 to then), which resolves Rarog's run files to policy in every
checked field and passes the 40-case guard suite.
fastchess, weather-factory, `sprt.ps1` and `spsa.ps1` stay installed, working
and documented as the backup and the second opinion until at least release
2.5.0; PROCESS's *Harness* section holds the cross-check triggers. The two
instruments agree where they have been compared (RAR-M60, RAR-M61).

B.7 keeps its place. B.3 is eligible: the B.2 head was accepted on 2026-09-20. B.2.0.2 closed on 2026-09-15
(MultiPV, RAR-P26). B.2.1 was accepted by its reviewer on 2026-09-14
(`analysis/b21_review_2026-09-14.md`). Binding findings sit at their
leaves in PLAN: D.2's premise is contradicted (RAR-M46) and E.2's binding
arm is 1T.

| Open hold / obligation | Resume or resolve when | Must be resolved before |
|---|---|---|
| E.3.1 tag-driven release flow (added 2026-09-22) | Any time: it depends on nothing in B–D; schedule it between leaves, not inside a registered experiment's window | E.3, the next release |
| KRPPKRP 7-man truth gap | Independent truth becomes available, or C.5.8 records an explicit exclusion | C.5.8 closes |
| KRP-KB win-preserving 0.9990 → 0.9949 (−2.2 SE, RAR-M42) | Non-blocking; blocking if a later change pushes it past 3 SE | C.5.4 closes (owner) |

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
    - [x] **A.8.3** Oracle deficit meter: G(0) = −247.97 ± 10.89, depth gap 0.97 ply as recorded (1.4 non-mate after the 2026-10-03 correction) — RAR-O03, DONE 2026-09-11
    - [x] **A.8.4** Pooled-PGO NPS baseline: 3.19 MNPS median, null +0.04% [−0.23%, +0.15%] — RAR-M48, DONE 2026-09-11
- [x] **A.9** Release 2.4.0: documents finalised, squashed to `master`, CI green, tagged and published with per-tier assets — DONE 2026-09-11

## Phase B — Search programme (evaluation frozen)

- [x] **B.0** Investigation: mechanism map, cluster contents, scale ratio 0.457 (eval) / 0.75 (SEE), B.2.2 screens registered as a branching window plus fixed-node quality, 116 oracle-anchored canaries, B.1–B.3 handoffs frozen (`analysis/search_programme_2026-09-13.md`, RAR-M50) — DONE 2026-09-13
- [x] **B.1** Search restructure: `search/` modules, `NodeType`, `ThreadData`, sentinel `PlyArray` stack; 44 inert parameters, root confidence, SMP skip and `evidence.rs` removed; exact fingerprint, pooled-PGO NPS +6.30% (RAR-P24) — DONE 2026-09-14
- [x] **B.2** Cluster 1 — selectivity core: TT eval storage, correction, histories, picker, move-loop pruning, LMR; both gates passed 2026-09-20, the re-tune B.2.7 accepted 2026-09-25 — DONE 2026-09-25
    - [x] **B.2.0** Architecture review of the B.1 head (RAR-M51) and its twelve behaviour-neutral upgrades: narrowed surface, tagged commands, search output port, TT policy factored once, tuner out of the workspace, tools index, comment hygiene; exact fingerprint, NPS +0.21% vs the B.1 pool (RAR-P25) — DONE 2026-09-14
        - [x] **B.2.0.1** Repository and document restructure: archived trackers, one-line closed leaves, one ledger row per experiment, `analysis/` index, dead-path check, rule-first AGENTS; logos stay tracked (RAR-M53) — DONE 2026-09-14
    - [x] **B.2.1** Implement to the B.0 handoff with table, picker, TT and unwind tests; behind `b2core`, 4,706,910 / EBF 2.391 unfitted, off arm exact; reviewer-accepted (RAR-S73, `analysis/b21_review_2026-09-14.md`) — DONE 2026-09-14
        - [x] **B.2.0.2** MultiPV: UCI option up to 256, root lines above 1; identity at `MultiPV = 1` on both arms (bench and `info` stream), pooled NPS +0.62% (RAR-P26) — DONE 2026-09-15
    - [x] **B.2.2** Diagnostics: oracle differential, depth at 300k, EBF, reference-anchored branching curve, tactical suite, 2,000-game unfitted run; screen thresholds registered by B.0. Screens run 2026-09-15: paired run **+52.16 ± 10.73 Elo** unfitted while four zero-game floors fail (`analysis/b22_screens_2026-09-15.md`); reviewed, the paired run governs, re-planned as B.2.2.1–B.2.2.4 — DONE 2026-09-15
        - [x] **B.2.2.1** Two seed-scale clamps converted (`b2core` 6,586,667 / EBF 2.433); five categorical switches (`CoreRazorGuards`, `CoreCorrTrainDecisive`, `CoreCorrTrainExcluded`, `CoreLmrFullDepth`, `CoreLmrCheckRoot`) with tests; `b2core,tune` PGO build; RAR-S74 registered — DONE 2026-09-15
        - [x] **B.2.2.2** Paired runs (a)–(g) read: nothing adopted, mate-residual training stays, the clamp conversion reads −7.64 ± 9.72 and is reverted by `308abe9` to four SPSA coordinates at the donor's seeds, `b2core` 4,706,910 / EBF 2.391 again (RAR-S74) — DONE 2026-09-15
        - [x] **B.2.2.3** Curvature sweep of the five §9 coordinates and the P6 profile: three curved (`CoreLmpSquare`, `CoreLmrQuiet`, `CoreCorrUpdateSlope`), so B.2.3 runs; P6 +5.59% (`analysis/b223_sweep_2026-09-15.md`) — DONE 2026-09-15
        - [x] **B.2.2.4** Screen rules for B.3–B.5 in rule 8: paired run governs, one ablation sweep in mechanism order, time-to-depth, positional screen (phase-4 suite until STS is placed), canary regression rule, sweep checklist — DONE 2026-09-15
    - [x] **B.2.3** SPSA on the `b2core` arm: 82 `CoreParams` coordinates (switches, `CoreIirMinDepth` and `CoreEvalRule50Damping` out), `config_b23core.json`, the `-Tune -Features b2core` tooling ticket, pilot 128 × 32, N = 5,000 in resumable sessions registered as RAR-S75, final theta baked, fitted-vs-unfitted diagnostic run. Preparation done 2026-09-15 (RAR-S75, binary `25467C63…`, setup proven at N = 5,000); tune finished 2026-09-20, theta baked (`14a7079`, 7,185,678 / EBF 2.444), B.2.3.3 passed — DONE 2026-09-20 — **IMPLEMENTED / V**
        - [x] **B.2.3.1** Checkpoint peek, theta at 3,900 vs the unfitted head: +118.72 ± 10.62 Elo in 2,000 games (predicted +10 ± 11), bench 6,199,302 / EBF 2.421, never baked (RAR-S76) — DONE 2026-09-19
        - [x] **B.2.3.2** Rybka 4.1 benchmark, 2,000 games each at harness conditions, `Max CPUs=1`: theta at 3,900 **+97.69 ± 12.84** (RAR-M55), the unfitted head −7.12 ± 12.78 (RAR-M56); the fit is worth +104.8 ± 18.1 against Rybka — DONE 2026-09-19
        - [x] **B.2.3.3** Tail SPRT, theta at 5,000 vs theta at 3,900, `[0,3]`: **H1 in 20,806 games, +4.43 ± 2.90 Elo**, so the last 1,100 iterations were worth about 4 Elo (RAR-S77) — DONE 2026-09-20
        - [x] **B.2.3.4** Pool gauntlet, 6,000 games at the Super Rating Tournament's conditions, pool held at its ratings: 2.5.0-dev (fit at 3,900, native) rates **3191** against 2.4.0's 3001; against the E.2 targets Rybka 4.1 59.1%, Fritz 16 52.3%, Critter 46.1%, Houdini 3 39.2% (RAR-M57); a follow-up Critter match reads −5.0, 95% −22 to +12, in 1,050 games (RAR-M58) — DONE 2026-09-19
    - [x] **B.2.4** Gate as two SPRTs: B.2.4a unfitted `b2core` vs the off arm `[0,10]` **passed 2026-09-16, +65.09 ± 23.26 in 432 games** (the unfitted arm is the accepted head); B.2.4b fitted vs unfitted `[0,10]` **passed 2026-09-20, +138.60 ± 30.66 in 248 games** (the fitted arm at 7,185,678 is the accepted head); the default flip landed 2026-09-20 (`a47e85b`, `58176a1`): `b2core` is a default feature, the default build is 7,185,678 / EBF 2.444 and `--no-default-features` keeps the legacy search at 7,601,220 — DONE 2026-09-20
    - [x] **B.2.5** UCI `info` conformance: winner's line after the SMP vote, `depth 0` line at a mated root, `nps` at `time 0`, `multipv 1` always, bounds in single-PV, seldepth convention, `<empty>` string-option value read as empty (GitHub issue #1); identity-gated, no SPRT (`analysis/uci_info_review_2026-09-16.md`); both fingerprints held at every commit — DONE 2026-09-20
    - [x] **B.2.6** Adopt Colosseum CLI as the main harness, now (maintainer decision 2026-09-21; the tune and B.2.4b are done, the harness is qualified in its own repository); fastchess, weather-factory, `sprt.ps1`, `spsa.ps1` and their infrastructure stay working as the backup and second opinion at least until release 2.5.0 — DONE 2026-09-22
        - [x] **B.2.6.1** Run files, the surface converter and `tools/colosseum.ps1`, which carries every `sprt.ps1`/`spsa.ps1` guard from one shared implementation plus an idle host, the runner pin and `-ExpectBench`; 30 of 30 configuration fields match the recorded `sprt.ps1` manifest. The 200-game live run on `cli-v0.1.0` completed clean (0 faults, recount agrees); it and the audit that followed found five wrapper defects (fault parser matching no build, verdict exit codes read as failures, null pairs recounted by name, absolute `-Dir`, resume seed), all fixed, then a sixth on review (the gauntlet's policy check could never pass), guard suite 36 of 36, every mode smoked live — DONE 2026-09-22
        - [x] **B.2.6.2** Colosseum documented as the main path with nothing retired: PROCESS's new *Harness* section, AGENTS, `tools/README.md` and `tools/colosseum/README.md` rewritten with the fastchess and weather-factory path as the named backup and its three cross-check triggers; RAR-M60, RAR-M61 and RAR-M62 recorded from the 2026-09-17/18 artifacts; the next tune's shape registered at 15 slots, 30 games per iteration, budget in games — DONE 2026-09-21
        - [x] **B.2.6.3** Re-pinned to the published `cli-v0.1.0` (`40a15b1b`, released 2026-09-22): the executable's SHA-256 and the archive's digest, both enforced by `setup_tools.ps1`, which staged it end to end. The release has no `SHA256SUMS`, so the per-asset digest GitHub serves is the source. Repeated on the released runner: the dry run resolves identically to the superseded build in all 439 fields, parity 30/30, guard suite 22/22 — DONE 2026-09-22
    - [x] **B.2.7** Re-tune on Colosseum of the coordinates at least one step from their seeds at N = 5,000, from the final theta; SPRT `[0,3]` vs the B.2.4b head; fully registered 2026-09-22 and amended the same day before any game: all 82 coordinates restarted at theta_5000 with RAR-S75's steps (`b27all`), N = 5,000 × 30 games, `r_end` 0.0031, `rarog-b27core-tune.exe` at 7,185,678, gate cap 40,000 pairs, dry run resolved; tune finished 2026-09-24 (150,000 games, 0 faults, no rail, 10 of 82 moved a full step), theta baked `52c46df` at 7,435,006 / EBF 2.457, gate binary built; **SPRT H1 accepted 2026-09-25, +13.1 ± 5.4 Elo (+20.9 ± 8.7 nElo) in 3,081 pairs** (RAR-S78); weather-factory stays installed until 2.5.0 — DONE 2026-09-25
    - [x] **B.2.8** First-search stalls: KPK bitbase and every table built at start-up, hash table converted at `setoption`, helper tables ready before the first search (`9a7b663`, `c0e6ef7`, bench unchanged, first KPK search 0.16 ms fresh vs 34 before); the 10,000-game confirmation was handed to Colosseum and never played (its 10.9m closed without it); 21,055 post-fix fastchess games lost none on time (RAR-M59) — DONE 2026-09-19
- [x] **B.3** Cluster 2 — NMP, ProbCut, singular/multi-cut/negative/LDSE extensions, IIR policy; researched on the fitted head 2026-09-23 (RAR-S79); SPRT `[0,3]` **accepted 2026-09-27 at +50.5 ± 10.9 Elo** (RAR-S84) and the default since `f53ca7d`; the ponder race (B.3.5) fixed the same day — DONE 2026-09-27
    - [x] **B.3.1** Implement behind `b3proof`: NMP at cut nodes with a verification region; ProbCut with the TT gate, margin-scaled verification and lerp return; singular with graded extensions, Rarog's multi-cut rule, demotion, −3, LDSE and the LMR term; the lone-move `alpha` return and a per-line extension budget (amendments 2–6; IIR stays the accepted rule). Arm **7,479,114 / EBF 2.467** (7,978,292 / 2.465 since B.3.3's bake, `3ca9aab`; 7,721,657 / 2.456 on the B.2.7-baked head, `52c46df`; 14,331,872 with RAR-S82's block-1 theta baked, `94cc1cf`), off arm exact, suites 329/330 and 348/349, cost screen 2.26×/1.33×, CI covers the arm — DONE 2026-09-23
    - [x] **B.3.2** Diagnostics: counters, screen ladder, deep-iteration cost screen, bit sweep, five categoricals, the 2,000-game unfitted paired run (maintainer-run); zero-game half done 2026-09-23 (three floors fail, quality at target); paired run **+18.4 ± 9.4 Elo** on 2026-09-24, above target; categoricals (RAR-S80) adopt `SingularTtDepthMargin=2` alone, three switches stay at default at `3+0.03` and `10+0.1` — DONE 2026-09-24
    - [x] **B.3.3** Bake `SingularTtDepthMargin=2` on the arm (one engine commit), curvature sweep, then the SPSA on Colosseum if curved (maintainer-run); theta baked; bake and sweep done 2026-09-24 (3 of 7 curved), RAR-S82's block 1 read +12.0 ± 9.5 over the untuned arm and +44.2 ± 9.6 over the B.2.7 head; its theta with the measured `1·d` margin was baked as the arm's defaults and RAR-S83's two blocks settled the surface (no full-step mover; theta = block 2) — DONE 2026-09-27
    - [x] **B.3.4** Gate: SPRT `[0,3]` vs the accepted head, cap 20,000 pairs (maintainer-run); default flip; **H1 accepted 2026-09-27 at 804 pairs, +50.5 ± 10.9 Elo** (RAR-S84; read +48.1 ± 9.5), theta baked and `b3proof` default in `f53ca7d` — DONE 2026-09-27
    - [x] **B.3.5** Ponder race: `ponderhit` or `stop` right after `go ponder` loses the `bestmove` (report of 2026-09-26); reproduced in all three scenarios at Threads 1 and 4 (a `stop` made the queued `go` stale, a `ponderhit` was erased as the search started); both signals scoped to their `go`'s epoch in `ef1a24b`, race tests fail on the old code; bench unchanged at both fingerprints, fmt, clippy, debug and release tests; ponder-on smoke 400 games against the unfixed gate binary, 0 faults — DONE 2026-09-27
- [x] **B.4** Cluster 3 — quiescence: fail-high interpolation, the PV bit on depth-0 stores, the donor count rule with check and recapture exemptions, margins as coordinates, evasion pruning as a switch; researched on the cluster-2 head 2026-09-28 (RAR-S85, `analysis/b4_research_2026-09-28.md`): the quiescence is 43% smaller per interior node than the oracle's, first-ply checks not owed, RAR-M19 decided (ordering scale fixed, margins absorb it); SPRT `[0,3]` ; implemented behind `b4quiet` 2026-09-29; **accepted 2026-09-30 at +4.4 ± 2.9 Elo** (RAR-S88) and the default since `5a5c150` — DONE 2026-09-30
    - [x] **B.4.1** Implement behind `b4quiet`: T1 the PV bit (Stockfish form) and the interpolations at 583 / 562 (`853bf7b`), T2 the count rule with its escapes, the SEE threshold on every capture and the margins as coordinates (`77f5676`), T3 `CoreQsEvasionPrune` and T4 `CoreQsNoisyHistory`, both default off (`ca1c208`, `b734caa`); the arm reads 10,226,874 / EBF 2.519, WAC 236, canaries 92 with none lost; two research amendments on the way — DONE 2026-09-29
    - [x] **B.4.2** Diagnostics: zero-game half (WAC 236 / 270, time-to-depth 0.85×, canaries 92 with none lost; floors failed on depth-14 nodes 4.03×, agreement 39, b15's cost); the 2,000-game unfitted paired run read **+0.7 ± 9.3 Elo** (0 faults); the component ablation puts the failed floors on the interpolation pair through the interior estimate; implementation reviewed, no defect, CI covers the arm; review decision: proceed — DONE 2026-09-29
    - [x] **B.4.3** Curvature sweep (three of seven curved), RAR-S87's categorical reads (evasion pruning adopted at +7.8 ± 9.4 Elo, quiescence history off), then the fourteen-coordinate SPSA (RAR-S86): block 1, 60,000 games, moved `QsFutilityMargin` a full step (150 → 178) and nothing else half a step, so the stop rule ended the tune; theta is block 1's centres, baked for the gate by recipe (`analysis/b43_gate_bake.patch`) — DONE 2026-09-30
    - [x] **B.4.4** Gate: SPRT `[0,3]` vs the accepted head, cap 20,000 pairs (maintainer-run); **H1 accepted 2026-09-30 at 10,705 pairs, +4.4 ± 2.9 Elo** (RAR-S88); theta baked, evasion pruning on and `b4quiet` the default in `5a5c150` (11,171,726 / EBF 2.512), after the allocation guard's precondition repair (`e5f61b5`) — DONE 2026-09-30
- [x] **B.5** Cluster 4 — root, aspiration, iterative deepening, PV; keeps B.2.0.2's MultiPV contract; researched 2026-09-30 (RAR-S89): the aspiration loop re-searches failed windows with 61% of the root's nodes, and one value (`AspMaxFails` 1) repairs it; gate rejected (−18.3 ± 6.9) and the surface read flat-to-falling (RAR-S90): the cluster's verdict is `NO_CHANGE`, the root kept; B.5.2 closed 2026-10-01 with B.5.2.1 kept (RAR-S94 +11.4 ± 8.5 Elo with tables) — DONE 2026-10-01
    - [x] **B.5.1** Research cards from the Gyatso read (2026-09-26): TT-hit history bonus `NO_CHANGE` (already in the head since B.2); draw-score randomisation `NO_CHANGE`, −1.2 ± 9.4 Elo in 2,000 games (RAR-S91), switch off — DONE 2026-10-01
    - [x] **B.5.2** Tablebase root, in-search probes and PV the Stockfish way (one-move PV, no ponder move, `cp 31743` at a tablebase root; added 2026-09-27): investigated 2026-10-01, three mechanisms and an empty tablebase band found; implemented 2026-10-01 (`analysis/b52_research_2026-10-01.md`); closed 2026-10-01, B.5.2.1 kept — DONE 2026-10-01
        - [x] **B.5.2.1** Implementation: DTZ ranks in Rust, the best-ranked group, bound-correct probes off at a DTZ root, the tablebase band (the TT keeps band values as stored: the stop rule refuted moving them), the UCI display, the time cap, the PV extension, the ponder move, the committed KQvK/KRvK fixture and its CI tests; bench 11,171,726 after every commit — DONE 2026-10-01
        - [x] **B.5.2.2** Tablebase-enabled gate: RAR-S93 (453/453 converted on both sides), RAR-S94 H1 at 776 pairs, +11.4 ± 8.5 Elo; two stop conditions fired by their letter: the box rule recorded as unsatisfiable for a box checked between Fathom calls (a replay put the overruns on single calls of up to 16 ms, page-cache misses), the cold-read time loss to D.3 — DONE 2026-10-01
    - [x] **B.5.3** Gate: one-retry aspiration (`AspMaxFails` 20 → 1), SPRT `[0,3]` vs the head (RAR-S89): **H0 at 1,991 pairs, −18.3 ± 6.9 Elo**, rejected, nothing baked; the zero-game reads pointed the other way — DONE 2026-09-30
    - [x] **B.5.4** Optimism research card: switch `CoreOptimism` built (`d754577`, default off); RAR-S92 read **−5.6 ± 9.4 Elo** in 2,000 games, `NO_CHANGE`, the switch stays off for B.8 — DONE 2026-10-01
    - [x] **B.5.5** The aspiration surface read by games (RAR-S90): delta 12 **+2.1 ± 9.2**, delta 45 **−25.4 ± 9.2**, one retry at `10+0.1` **−18.3 ± 8.5**; no direction, no tune, the loop kept: `NO_CHANGE` — DONE 2026-09-30
- [x] **B.6** Joint search SPSA, only if curvature justifies it; researched 2026-10-01 (RAR-S95): the finished tunes' journals show no gradient left where each stopped; skipped by the maintainer, `NO_CHANGE`, the joint tune is C.10 — DONE 2026-10-01
- [x] **B.7** Search speed pass on the new modules; pooled-PGO floor +0.5% per change: three changes accepted, +8.85%, +6.93% and +1.69% NPS, bench exact (RAR-P28, RAR-P31), confirmed in games at +35.5 ± 9.0 Elo (RAR-S98); reopened 2026-10-03 for B.7.3, which closed `NO_CHANGE` — DONE 2026-10-03
    - [x] **B.7.1** Allocation guard: a counting-allocator test that allocations grow per iteration, never per node (`8836013`); at most about 25 per iteration per thread against a budget of 64, all at the root node; a planted per-node allocation fails it — DONE 2026-09-27
    - [x] **B.7.2** Speed pass: profiled (RAR-P27); the quiet-stage early exit (+8.85%), the in-place scored list (+6.93%) and the TT prefetch before the make (+1.69%) accepted, bench exact; the scan layout, the correction-slot prefetch, the history-row prefetch and the static attack tables closed `NO_CHANGE`; verified by the auditor, closed by the maintainer — DONE 2026-10-02
        - [x] **B.7.2.1** Candidate 1, the quiet-stage early exit, with an oracle test against the old loop (2,000 random lists; a planted defect fails it) — DONE 2026-10-01
        - [x] **B.7.2.2** Candidate 1 live wire: over `bench`, scanned elements 578.6M → 231.8M and selections 41.07M → 14.80M, exactly the removed ones — DONE 2026-10-01
        - [x] **B.7.2.3** Candidate 1 deterministic qualification: bench 11,171,726 and legacy 7,590,542 exact; fmt; clippy zero; debug 376 and release 377 tests incl. the allocation guard — DONE 2026-10-01
        - [x] **B.7.2.4** Candidate 1 pooled-PGO NPS: **+8.85%** (95% CI +8.17% .. +9.48%), 20 cycles, idle host; above the +0.5% floor, inside the +4% to +9% prediction (RAR-P28) — DONE 2026-10-02
        - [x] **B.7.2.5** Candidate 2, the in-place scored list (`0d95763`); the three per-node 4,104-byte `memcpy` calls are gone from the pext PGO disassembly, nothing new appears — DONE 2026-10-01
        - [x] **B.7.2.6** Candidate 2 deterministic qualification: bench 11,171,726 and legacy 7,590,542 exact; fmt; clippy zero; debug 376 and release 377 tests incl. the allocation guard — DONE 2026-10-01
        - [x] **B.7.2.7** Candidate 2 pooled-PGO NPS over candidate 1: **+6.93%** (95% CI +6.38% .. +7.68%); above the floor; the +1% to +3% prediction missed in magnitude (RAR-P28) — DONE 2026-10-02
        - [x] **B.7.2.8** Re-profile: −9.7% samples for the same nodes (ordering −6.5%, copies −4.2% of the old total); `pick_next` still 6.26%, so candidate 3 qualifies on share, behind a falsifier — DONE 2026-10-02
        - [x] **B.7.2.9** Candidate 3 falsifier: on 536,249 recorded `bench` tails, `s` = **1.106 and 1.093** on an idle host (straddling 1.1; earlier loaded runs superseded), the ceiling about +0.6% before the second array's cost; the maintainer decides, `NO_CHANGE` recommended (RAR-P29) — DONE 2026-10-02
        - [x] **B.7.2.10** Candidate 3, scores held contiguously: not built, `NO_CHANGE` by the maintainer; the audit's variants on the engine's own entries read `s` 0.91 to 1.07 — DONE 2026-10-02
        - [x] **B.7.2.11** Candidate 3 deterministic qualification: not needed, candidate 3 closed — DONE 2026-10-02
        - [x] **B.7.2.12** Candidate 3 pooled-PGO NPS: not needed, candidate 3 closed — DONE 2026-10-02
        - [x] **B.7.2.13** Audit screen (RAR-P30), 20 cycles each against the head's pool: A (TT prefetch before make) **+1.48%** (CI +0.91% .. +2.41%) forward; I (correction-slot prefetch) +0.14% (−0.29% .. +0.75%) between, implemented after A by the maintainer's decision; D (history-row prefetch) −1.87% and B (static attack tables) −0.49% close `NO_CHANGE`; all four +1.71% — DONE 2026-10-02
        - [x] **B.7.2.14** A, the TT prefetch before the make, with the hint's exactness test (`a42fadc`, RAR-P31) — DONE 2026-10-02
        - [x] **B.7.2.15** A deterministic qualification: bench 11,171,726 and legacy 7,590,542 exact; fmt; clippy zero; debug 377 and release 378 tests incl. the allocation guard; `prefetcht0` 20 → 34 in every pool build — DONE 2026-10-02
        - [x] **B.7.2.16** A pooled-PGO NPS against the c2 pool: **+1.69%** (95% CI +1.56% .. +1.89%), accepted; inside the +0.91% to +2.41% prediction — DONE 2026-10-02
        - [x] **B.7.2.17** I, the correction-slot prefetch, over the head after A (`a459132`, reverted in `bec1292` by B.7.2.19) — DONE 2026-10-02
        - [x] **B.7.2.18** I deterministic qualification: bench and legacy exact; fmt; clippy zero; debug 377 and release 378 tests; `prefetcht0` 34 → 118 in every pool build — DONE 2026-10-02
        - [x] **B.7.2.19** I pooled-PGO NPS against A's pool: **+0.37%** (95% CI +0.17% .. +0.56%), under the floor, reverted, `NO_CHANGE`; inside the −0.29% to +0.75% prediction; CPU 21.6% after the read, a recorded confound — DONE 2026-10-02
        - [x] **B.7.2.20** The total, A's pool (the final head's engine source) against the c2 pool: **+1.78%** (95% CI +0.92% .. +2.56%) — DONE 2026-10-02
    - [x] **B.7.3** SEE recapturer: the attacker set built once and extended by what each removal reveals, the king query cached per side; exact (oracle test 4,411,664 comparisons, bench and legacy exact, `8fc8ff9`); the registered read (RAR-P33) **−2.09%** NPS (95% t-interval −2.34% .. −1.84%), closed `NO_CHANGE` by step 1 and reverted (`7e6e0ef`) — DONE 2026-10-03
- [x] **B.8** Cleanup: the B.1 search and the arm features, 64 dead coordinates, 14 decided switches, 73 unowned counters, the PVS guard, bench exact at every commit; `src` 31,487 → 26,110 lines, tune options 217 → 153, counters 265 → 192; RAR-P34 +1.16% NPS, regression excluded; record `analysis/b8_removed_2026-10-03.md` — DONE 2026-10-03
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
- [ ] **C.10** Joint search SPSA after the new evaluation: cp margins plus every mechanism whose firing rate moved 10% or more (the whole-surface tune B.6 left for here); rule-7c blocks; SPRT `[0,3]` — **V**
- [ ] **C.11** Checkpoint: same-search deficit, conversion, NPS, pool gauntlet; freeze the classical evaluation — **V**

## Phase D — Clock, threads, robustness

- [ ] **D.1** Time management: audit against the ADR-0065 checklist, soft/hard bounds with node-fraction multiplier, forfeit margin; SPRT `[0,3]` — **R2**
- [ ] **D.2** Lazy SMP quality at 4T/8T: diversity, shared TT and correction, soft-stop voting; 4T SPRT `[0,5]` — **R2**
- [ ] **D.3** Engine lifecycle and protocol robustness; `src/uci/` (planned) move; score normalisation research card (`analysis/uci_info_review_2026-09-16.md` item 6); zero crashes over pool tournaments — **R2**
- [ ] **D.4** Tablebase policy: probing depth/limits, WDL/DTZ in conversion, recogniser interaction — **R2**

## Phase E — Classical checkpoint and release

- [ ] **E.1** Attribution checkpoint: B.2.0 review re-run on the B.9/C.11 heads; STC, `10+0.1`, 4T against 2.3.2 and the B.9/C.11 heads; maturity checklist — **V**
- [ ] **E.2** Target gate: ≥50% against Critter 1.6a, Houdini 3, Rybka 4.1 and Fritz 16 at 1T and 4T (Rybka 4.1 replaced Rybka 4, 2026-09-19) — **V**
- [ ] **E.3** Release 3.0.0 (gate met) or 2.5.0: changelog, suites, PGO assets, ISA, cut by pushing a `v` tag through E.3.1's workflow, on instruction — **M**
    - [ ] **E.3.1** Tag-driven release flow (Colosseum's model): `git tag vX.Y.Z` on `master` and push; the workflow validates (tag = `Cargo.toml` version, commit on `master`, `## [X.Y.Z]` section in `CHANGELOG.md`), builds the nine PGO assets read-only, asserts one `bench 13` fingerprint across them, then one write job publishes with notes from the changelog; candidate mode rehearses without a tag; asset names unchanged; repair = delete tag, fix, retag; local `release-check`. Depends on nothing in B–D: **may land any time, must land before E.3** (added 2026-09-22) — **I1**

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
