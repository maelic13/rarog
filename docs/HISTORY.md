# Rarog history

⚠ **THIS FILE IS HISTORY. It does not tell you what to do next.** Read
`GUIDE.md` for the current step and `PLAN.md` for what it involves.

## Numbering, and how to resolve an old reference

Three retired numbering schemes exist in the ledger, the analyses and source comments.
None of them is the current roadmap's, which uses lettered phases (`A.2.1`).

| Scheme | Where it appears | Resolve it in |
|---|---|---|
| Legacy Rarog phases 7–14 (`7.0b`, `8.2(a)`, `9.0a`, `10.3 speed pass`, `11.x`–`14`) | older source comments and tool prose, the oldest ledger rows, releases up to 2.3.1 | [docs/archive/GUIDE-legacy-2026-07-29.md](archive/GUIDE-legacy-2026-07-29.md) (the tracker) and [docs/archive/PLAN-legacy-2026-07-29.md](archive/PLAN-legacy-2026-07-29.md) (§S6, rationale per item), both verbatim from the 2.3.1 release commit `a5fd288` |
| Phase 4 roadmap before the 2026-09-04 renumbering (`4.9b`) | ledger rows and analyses written 2026-08-11…2026-09-03 | [docs/archive/GUIDE-phase4-tracker-2026-08-21.md](archive/GUIDE-phase4-tracker-2026-08-21.md); its old numbers map to the renumbered ones in section 13 of the archived Phase-4 PLAN |
| Phase 4 roadmap after the renumbering (`4.5`, `4.9a.4`, `4.11b.19`), Phases 5–9 | `EXPERIMENTS.md`, `analysis/*.md`, commits up to `c80df74` | [docs/archive/PLAN-phase4-2026-09-09.md](archive/PLAN-phase4-2026-09-09.md) and [docs/archive/GUIDE-phase4-2026-09-09.md](archive/GUIDE-phase4-2026-09-09.md); the retired-to-current map is the number map below |
| Current roadmap (`A`–`G`) | `PLAN.md`, `GUIDE.md`, ledger rows from RAR-M45 on | `PLAN.md` |

## The Phase-4 line, 2026-08-11 to 2026-09-09: what it established

The archived roadmap ran from the 2.3.2 release to the 2026-09-09 rewrite. Its
durable results, each with its ledger row:

| Result | Evidence |
|---|---|
| Search deficit against Stockfish's search with Rarog's evaluation: **250.8 ± 13.1 Elo** equal time, 355 equal nodes; LMR plus shallow pruning explain 272 ± 18 of it | `analysis/ablation_results.md`, RAR-S55…S70 |
| Evaluation deficit against Stockfish's classical HCE with the same search: **about 329 Elo** | RAR-O02 |
| Accepted strength: ProbCut move filter +15.56 ± 10.02; root LMR relief +2.33 ± 1.85; complete HCE refit +22.04 ± 7.51; TB-corrected labels +6.73 ± 3.82; hce-v3 refit +11.81 ± 5.33; board cluster with the SEE repair +12.12 ± 10.17 | RAR-S57/S58, RAR-S70, RAR-E06, RAR-E08, RAR-E12, RAR-E15 |
| Rejected or null: interior LMR contract changes, quiet SEE prune, the SearchCore rewrite (−9.76 ± 17.70, stopped), broad selectivity SPSA declined | 4.5, 4.6 dispositions in the archive |
| Board: SEE king legality, created pins and recapture promotions repaired against 41 external fixtures; fused relocation +17.5% make/unmake; caller-owned move-list delivery +2.48% NPS; generation constant-factor candidates measured −0.55% in search and reverted | RAR-M25…M44 |
| Endgame instruments: truth, drawn-overclaim, conversion, floors, occurrence and ranking tools with cohort digests and guard self-tests; registered family order v2; KBN-K conversion 19.4% → 96.9% from the mate drive | 4.9a, 4.10, 4.11 in the archive |
| Instruments: paired matched ablation against the frozen `hybrid` oracle; pooled-PGO NPS with null pairs; cross-engine board benchmark with harness parity; counter-unit discipline | `analysis/ablation_design.md`, `analysis/phase4_counter_spec.md`, `tools/nps_multibuild.ps1` |
| Fingerprint at the rewrite: `bench 13` **7,601,220 / EBF 2.474** at `c80df74` | GUIDE checkpoint |

What it did not do, and why the roadmap was rewritten: it was about to spend
twenty-three frontier-research leaves on endgame recognisers worth ten to
thirty Elo while the measured search deficit had one candidate leaf; the
maintainer's assumption that endgames were the last missing evaluation piece
was contradicted by the 329-Elo same-search evaluation gap and by the
110–220-Elo pool deficit to the strongest HCE-era engines.

## Number map: retired leaves that continue in the current roadmap

Every identifier from the archived roadmap is retired. Where a retired open
leaf continues here, this is the mapping; everything else is history.

| Retired | Continues as | Note |
|---|---|---|
| 4.12 (20 endgame functions) | C.5 (8 leaves) | rescoped from function coverage to conversion and generic scaling |
| 4.13, 4.14 (labels, refit cycles) | C.2, C.8 | inside the evaluation programme |
| 4.13a (HCE audit) | C.0 | |
| 4.15, 4.15a–c, 4.16, 4.18 (search audits, SPSA, cleanup) | B.0–B.9 | replaced by the search programme |
| 4.17 (time management) | D.1 | |
| 4.19, 4.20 (checkpoint, release) | E.1–E.3 | |
| A.3.4, A.5, A.6, A.7 (before 2026-09-10) | A.9, A.8, A.5, A.6 | Phase A reordered so the numbering matches execution: improvements, instrument and analysis, then the version bump, then baselines on the bumped binary, then the release. Only open leaves moved; A.1-A.3 keep their numbers |
| 4.21 (universal binary) | G.2 | investigated under A.4 and deferred 2026-09-10 as optional; `analysis/universal_binary_2026-09.md` holds the design and the revival triggers |
| Phase 5, 6, 7 (NNUE runway, baseline, frontier) | F | |
| Phase 8 (scaling) | G | |
| Phase 9 (classical fallback) | dropped | the classical evaluation stays as datagen baseline and fallback by construction |

## Completed current-roadmap work (dated records; PLAN owns IDs)

- **2026-10-05 — Documents restructured so sessions read less (maintainer
  request).** PLAN, PROCESS, HISTORY and EXPERIMENTS moved to `docs/`; GUIDE
  stays at the root. Closed Phases A and B and step E.3.1 moved verbatim to
  `docs/archive/PLAN-closed-2026-10-05.md` (PLAN 240 KB → 57 KB), after
  B.9's search freeze became a Phase C rule and the archive-tag item a
  self-contained GUIDE hold. GUIDE's board is generated from PLAN's step
  heads by `tools/diag/guide_board.py`, and `check_guide.py` fails on a stale
  board. The ledger became an index plus 244 entry files in
  `docs/experiments/` (index 604 KB → 53 KB), held in step by
  `check_guide.py`; the split also repaired three tables whose blank lines
  had left 48 rows unrendered on GitHub and six rows whose extra cells GitHub
  dropped. Every move was checked to keep the removed text verbatim.

- **2026-10-05 — `dev` reopened at 2.6.0-dev; PLAN E.3.1 DONE; tags
  reviewed.** `d6998db` bumps the version to 2.6.0-dev (PROCESS *Release*
  step 6): `bench 13` 11,171,726 / EBF 2.512 unchanged, `id name Rarog
  2.6.0-dev`, fmt and clippy clean, debug 374 and release 375 tests
  passing. E.3.1's owed exit check was met by the 2.5.0 release itself:
  PR #4's candidate run and the `v2.5.0` tag run each built nine assets at
  one fingerprint, and the tag run published them. The maintainer decided
  that `dev` reaches `master` by merge commits from now on, so development
  commits stay on `master` and need no archive tag. The three `archive/pr*`
  tags cut for the squash merges of #2–#4, which now preserve the commits
  the entry below found on no ref, retire once the next merge carries
  their chains into `master` (`e887c3f` joins them to `dev`). The four
  `arm/*` tags are retired: their patches change the deleted 2.4.0 search,
  and every commit only they held is in the 2026-09-27 bundle.
  `CHANGELOG.md`'s `[Unreleased]` now collects changes as they land
  (AGENTS, *Changes*). Record: B.10's *Ref review after the release*, now in
  `docs/archive/PLAN-closed-2026-10-05.md`.

- **2026-10-05 — PLAN B.10 DONE: Rarog 2.5.0 released, and PHASE B IS CLOSED.**
  The search release, from the frozen B.9 head (engine source `ee02ed1`,
  11,171,726 / EBF 2.512), licensed under 2.4.0's rule against 2.4.0:
  +272.4 at `3+0.03` 1T, +260.5 ± 16.0 at `10+0.1`, +322.7 at 4T (RAR-M64,
  RAR-M65, RAR-M66). It is the first release cut by E.3.1's tag-driven
  workflow. Its first tag run failed every cell on the declared
  fingerprint: the workflow read GUIDE's Released baseline row (2.4.0's
  7,601,220) instead of the Development head row; nothing was published.
  The repair, by one PR to `master`: the workflow reads the declared
  fingerprint through `cargo xtask declared-fingerprint`, runs on every PR
  to `master` as a candidate build, and `release-check` requires the release
  commit to mark its own version released, as 2.4.0's release commit did by
  hand. Phase B's lasting output is the search itself: the measured
  equal-time search deficit, 248 Elo at 2.4.0, closed (G(0) +24.24 ± 8.01).
  The squash merges that carried 2.5.0 to `master` left the B-phase
  commits the ledger cites on no ref; they are held by this machine's local
  history until preserved.

- **2026-10-04 — PLAN E.3.1 IMPLEMENTED: the tag-driven release flow,
  before the 2.5.0 tag.** `release.yml` replaces `build.yml`: a `vX.Y.Z` tag
  push validates (`cargo xtask release-check`: tag = `Cargo.toml` version,
  commit on `origin/master`, dated `CHANGELOG.md` section, whose body is the
  notes), builds the nine PGO assets read-only, asserts one `bench 13`
  fingerprint across them equal to GUIDE's checkpoint, then publishes in one
  write job. A manual dispatch rehearses without publishing. The GitHub form
  is never typed into again, and nothing exists on GitHub until every cell
  passed. The `v2.5.0` tag pushed at `bd1c1a5` before this landed (no release
  existed) is deleted and re-pushed on the commit that carries the workflow.
  The three reviewed local branches (`b33-block1-probe`, `b33-gate`,
  `diag/b23-theta3900`) were deleted; their diffs live in `analysis/arm_patches/`.

- **2026-10-04 — PLAN B.10 IMPLEMENTED: Rarog 2.5.0 prepared; Phase B closes
  with the `v2.5.0` tag.** `3575558` bumps the version to 2.5.0 on the frozen
  B.9 head (engine source `ee02ed1`), reproducing `bench 13` 11,171,726 / EBF
  2.512, with clippy clean and debug 374 / release 375 tests passing. The
  release is licensed under 2.4.0's rule, which the maintainer adopted for
  2.5.0 on 2026-10-04, by three direct reads against the 2.4.0 release:
  **+272.4 at `3+0.03` on one thread** (400 games, RAR-M64), **+260.5 ± 16.0
  at `10+0.1`** (1,000 games, RAR-M65) and **+322.7 on four threads** (400
  games, RAR-M66). **What 2.5.0 contains**, each individually gated: the
  selectivity core (+65.09 ± 23.26 unfitted, +138.60 ± 30.66 fitted) and its
  re-tune (+13.1 ± 5.4), cluster 2 (+50.5 ± 10.9), cluster 3 (+4.4 ± 2.9),
  the tablebase repair (+11.4 ± 8.5 with tables) and the speed pass
  (+35.5 ± 9.0) — sequential gates under different baselines, **not
  additive**; plus `MultiPV`, the cleanup and the protocol and time-loss
  repairs the changelog lists. It runs at 73.8% of 2.4.0's bench speed and
  beats the frozen oracle by +24.24 ± 8.01. The tag-and-branch review
  recommends deleting three local diagnostic branches, whose diffs are now in
  `analysis/arm_patches/`, and keeps the `arm/*` and `oracle/*` tags with
  their retirement conditions. The PR, merge, tag and GitHub release are the
  maintainer's.
- **2026-10-03 — PLAN B.9 CLOSED: the Phase B record; the search head frozen.**
  The B.9 head (engine source `ee02ed1`, 11,171,726 / EBF 2.512, binary
  `aac92114…` built at `24aefb4`) beats the frozen oracle by **+24.24 ± 8.01
  Elo** (RAR-O04), where 2.4.0 lost by 247.97 ± 10.89 (RAR-O03): the measured
  equal-time search deficit, the programme's target, is closed, a move of
  +272.2 ± 13.5. It rates **3,286.5 ± 15.7** at 1T on the held scale against
  2.4.0's 3,001 and passes every E.2 target at 1T, Houdini 3 at 55.4%
  (RAR-M64). It runs 26.23% slower than 2.4.0 in pooled bench NPS (RAR-P35)
  and about 2 plies shallower than the oracle in their games. Attributed Elo
  per cluster from the gates: the selectivity core +65.09 and +138.60
  (B.2.4a/b), the B.2.7 re-tune +13.1, cluster 2 +50.5, cluster 3 +4.4, the
  speed pass +35.5 (sum +307 ± 41 over differing baselines), the tablebase
  repair +11.4 with tables; directly, +272.4 head-to-head over 2.4.0. `ablate`
  was removed. The budget table's B row (120–200 planned) is corrected to 272
  ± 14 measured. PLAN B.9 holds the readings, the attribution table, the
  calibration and the freeze. Also found while preparing B.9: RAR-O03's depth
  figures had been mis-attributed, corrected in `e5d359e` and `408e3a9`.

  GUIDE's checkpoint carried the B-phase closures in two cells until B.9
  closed; they are moved here verbatim.

  *Current step, as it read before B.9 closed:* **B.9** (`V`), `RESEARCH`: the
  checkpoint (G(0), depth/EBF, NPS, conversion, pool gauntlet; remove
  `ablate`; freeze the search head); not started. **B.8 closed 2026-10-03:**
  the B.1 search, the arm features `b2core`, `b3proof` and `b4quiet`, 64 dead
  coordinates, the fourteen decided switches, 73 unowned counters and the dead
  PVS guard removed in eight engine commits, each at `bench 13` 11,171,726 /
  EBF 2.512; the no-regression read (RAR-P34) +1.16% NPS (95% t-interval
  +0.72% .. +1.59%); what each removed item did, how it was decided and how to
  restore it is in `analysis/b8_removed_2026-10-03.md`; acceptance waits on
  the auditor's verification. **B.7.3 closed 2026-10-03, `NO_CHANGE`
  (RAR-P33):** the SEE recapturer's exact rewrite (`8fc8ff9`, oracle test of
  4,411,664 comparisons, bench exact) read **−2.09%** NPS (95% t-interval
  −2.34% .. −1.84%) against the head's pool, closed by step 1 of the two-step
  read and reverted in `7e6e0ef`; B.7 closed with it. Phase C carries speed as
  a secondary requirement from 2026-10-03 (PLAN, Phase C's goal). **B.7 closed
  2026-10-02 by the maintainer, its sweep verified by the auditor from the
  artifacts (RAR-P31), and confirmed in games (RAR-S98): H1 at 1,051 pairs,
  +35.5 ± 9.0 Elo against the head before B.7, +16.7% nodes a second inside
  the games.** The speed pass, B.7.2.1–B.7.2.20 on the board. **Accepted
  (RAR-P28):** candidate 1, the quiet-stage early exit (`5fe42cf`), +8.85%
  NPS; candidate 2, the in-place scored list (`0d95763`), +6.93% over it;
  bench exact. The re-profile leaves `pick_next` at 6.26%. Candidate 3, the
  scan layout, closed `NO_CHANGE` by the maintainer after its falsifier read
  `s` = 1.106 and 1.093. **B.7.2.13, the audit's screen, done 2026-10-02
  (RAR-P30):** the TT prefetch before the make (A) read +1.48% (CI +0.91% ..
  +2.41%) and goes forward; the correction-slot prefetch (I) read +0.14%
  (−0.29% .. +0.75%), between the bounds, and is implemented after A by the
  maintainer's decision; the history-row prefetch (D, −1.87%) and the static
  attack tables (B, −0.49%) close `NO_CHANGE`; all four together +1.71%.
  **B.7.2.14–B.7.2.16 done 2026-10-02 (RAR-P31):** A (`a42fadc`) accepted at
  +1.69% (95% CI +1.56% .. +1.89%), bench exact. **B.7.2.17–B.7.2.20 done
  2026-10-02:** I read +0.37% over A, under the floor, and was reverted
  (`bec1292`, `NO_CHANGE`); the total, A's pool against the c2 pool again,
  +1.78% (+0.92% .. +2.56%). Profiled 2026-10-01 (RAR-P27,
  `analysis/b72_profile_2026-10-01.md`). **B.6 closed 2026-10-01, `NO_CHANGE`,
  by the maintainer** (RAR-S95, `analysis/b6_research_2026-10-01.md`): the
  finished tunes' journals show no gradient left where each stopped (RAR-S78's
  second half and every later block read as random walks; only
  `QsFutilityMargin` still travels), a read that cannot see a gain of a few
  Elo; a restart was estimated at +3 ± 3 Elo for 12.5 hours a block plus a
  gate, so the joint search SPSA is C.10, after the classical evaluation.
  **B.5 closed 2026-10-01 with B.5.2**: the maintainer kept B.5.2.1 (RAR-S93
  converted 453 of 453 clean tablebase wins on both sides; RAR-S94 H1 at 776
  pairs, +11.4 ± 8.5 Elo with tables). Two registered stop conditions had
  fired by their letter. The box rule is recorded as unsatisfiable for a box
  checked between Fathom calls: a replay put the 50 overruns on single calls
  of up to 16 ms that the page cache misses, so a predictive box was not
  built. The cold-table time loss at a 7-man root goes to D.3. Out of band
  2026-10-01: the last `info` line always describes `bestmove` (`97ea52d`, the
  flaky threaded test's cause, now with a deterministic test), the legacy
  search builds again (`2326153`), and a test-suite audit is recorded under
  B.8 for the maintainer's decision. **B.5.2.1 done 2026-10-01**: the root
  keeps its DTZ-best group, probes are off at a DTZ root and bound-correct
  elsewhere, tablebase values are decisive and print `cp ±20000`, the hard
  limit is the optimum at a DTZ root, PVs extend through the tables and give
  the ponder move; KQvK/KRvK are committed so CI tests it; bench 11,171,726
  after every commit. The stop rule refuted moving band values by ply in the
  table (window arithmetic reaches the band without tables), so the table
  keeps them as stored. **B.5.2's investigation closed 2026-10-01,
  `READY_FOR_IMPLEMENTATION`**: the one-move PV has three causes (the root
  cut, the single-move shortcut stopping at depth 2, the at-once zeroing-child
  probe); probed tablebase values were never decisive and the TT ignored their
  band; without the cut a tablebase root fell into a won-endgame time sink (30
  s of 60, D.1's input), so the root's hard limit becomes the optimum; Fathom
  is safe for root DTZ probes beside helper WDL probes; the extension costs
  1.3–7.5 ms warm. The fixture (KQvK, KRvK, 14,080 bytes) waits on the
  maintainer's licence decision. **B.5.4 closed 2026-10-01, `NO_CHANGE`**:
  optimism read −5.6 ± 9.4 Elo in RAR-S92's 2,000 games; the donors' gain did
  not transfer to a classical evaluation. **B.5.1 closed 2026-10-01,
  `NO_CHANGE` on both cards**: (a) the premise was false, because the default
  search has given the TT-cutoff history bonus since B.2 in Reckless's form,
  fitted twice; (b) draw-score randomisation (`CoreDrawJitter`) read −1.2 ±
  9.4 Elo in RAR-S91's 2,000 games against a prediction of +1 ± 10, so it is
  undecidable at this budget and stays off
  (`analysis/b51_research_2026-09-30.md`). **B.5's root cluster closed
  2026-09-30, `NO_CHANGE`**: one-retry aspiration was rejected at −18.3 ± 6.9
  (RAR-S89) and lost the same at `10+0.1`; a window twice as wide read −25.4 ±
  9.2 and a narrower one +2.1 ± 9.2 (RAR-S90), so the loop is kept and no tune
  is owed; zero-game reads had pointed the other way. The research's inventory
  leaves the time management multipliers (D.1) and six single coordinates
  (B.6) as what was never re-fitted on this search
  (`analysis/b5_research_2026-09-30.md`). **B.4 closed 2026-09-30**: RAR-S88's
  gate accepted H1 at 10,705 pairs, +4.4 ± 2.9 Elo; theta baked, evasion
  pruning on and `b4quiet` the default in `5a5c150` (11,171,726 / EBF 2.512).
  **B.4.3 closed 2026-09-30**: the tune's block 1 moved one coordinate a full
  step (`QsFutilityMargin` 150 → 178) and the stop rule ended it; evasion
  pruning was adopted by RAR-S87 at +7.8 ± 9.4 Elo (quiescence history off at
  −4.7 ± 9.4); the sweep read 2026-09-29 found `QsSeeClampHi`, `QsDeltaMargin`
  and `QsStandPatLerp` curved, and the preparation's review added
  `CoreRazorSquare` and `CoreNmpBase` and took `CoreRfpLinear` off its rail.
  **B.4.2 closed 2026-09-29**: the unfitted paired run read +0.7 ± 9.3 Elo in
  2,000 games; the component ablation puts the failed floors on the
  interpolation pair through the estimate B.2's reverse futility was fitted
  on, so the estimate's first consumers join B.4.3's surface; the
  implementation was reviewed with no defect and CI covers the arm. **B.4.1
  closed 2026-09-29**: the arm at its defaults reads 10,226,874 / EBF 2.519,
  WAC 236 / 270, canaries 92 with none lost, time-to-depth 0.85×; floors
  failed on depth-14 nodes, agreement and b15's cost, as B.3.2's did; T3 and
  T4's switches default off by the canary rule. Earlier: **research amendment
  1 closed 2026-09-29** (packet, RAR-S85) after T1 (`ac018d4`) tripped the
  canary rule on 2026-09-28 — the rule is kept (0 of 12 neutral perturbations
  of the head lose a canary), Q1 becomes the Stockfish stored-bit form, Q2's
  seeds the Stockfish pair 583 / 562; the amended T1 reads bench 10,953,303,
  WAC 235, canaries 92 with none of the 91 lost, and its commit must reproduce
  those numbers. **B.4's research closed 2026-09-28** (RAR-S85,
  `analysis/b4_research_2026-09-28.md`): on the cluster-2 head the quiescence
  reads 0.33 qnodes per interior node against the oracle's 0.58, WAC 232/266
  at 100k/400k, 91/116 canaries, branching 1.800; contract Q1–Q9, screens and
  predictions registered; the diag counters landed in the engine commit. **B.3
  closed 2026-09-27**: B.3.5's ponder race fixed in `ef1a24b` (bench
  unchanged; ponder-on smoke 400 games, 0 faults). **B.3.4 closed
  2026-09-27**: RAR-S84's gate accepted H1 at 804 pairs, +50.5 ± 10.9 Elo, the
  direction read +48.1 ± 9.5; theta baked and `b3proof` the default in
  `f53ca7d`. **B.2 closed 2026-09-25**: B.2.7's re-tune (RAR-S78) was baked
  (`52c46df`, 7,435,006 / EBF 2.457) and its `[0,3]` gate accepted H1 at +13.1
  ± 5.4 Elo in 3,081 pairs. **B.3.1** (cluster 2 implementation, `I2`) was
  **implemented on 2026-09-23**: the `b3proof` arm reads 7,479,114 / EBF 2.467
  with the off arm exact, and CI covers it
  (`analysis/b3_research_2026-09-23.md`, *Final implementation record*).
  **B.3.2** closed 2026-09-24 (paired run +18.4 ± 9.4 Elo; RAR-S80's
  categoricals adopt `SingularTtDepthMargin=2` alone); **B.3.3**: the bake
  (`3ca9aab`, arm 7,978,292 / EBF 2.465) and the curvature sweep are done (3
  of 7 coordinates curved, `analysis/b33_sweep_2026-09-24.md`). B.2.6 closed
  2026-09-22. **E.3.1** (tag-driven release flow, `I1`, added 2026-09-22) is
  independent tooling that may land at any point before E.3

  *Active experiment, as it read before B.9 closed:* **RAR-S73** (B.2
  selectivity core: B.2.4a passed 2026-09-16, +65.09 ± 23.26; RAR-S75 finished
  2026-09-20 (5,000 iterations, theta baked at 7,185,678); RAR-S76 peek at
  3,900 +118.72 ± 10.62; RAR-M55 and RAR-M56 (B.2.3.2 closed: fitted +97.69,
  unfitted −7.12 vs Rybka 4.1), **B.2.4b passed 2026-09-20, +138.60 ± 30.66**;
  RAR-S77 (B.2.3.3) passed, +4.43 ± 2.90; RAR-S78 (B.2.7) tune finished
  2026-09-24, theta baked, **gate H1 2026-09-25, +13.1 ± 5.4 Elo** in 3,081
  pairs, B.2 closed; close-out audit 2026-09-21,
  `analysis/b2_audit_2026-09-21.md`); **RAR-M45, RAR-M46, RAR-O03 and RAR-M48
  all resolved 2026-09-11**; RAR-M49 conversion re-read and RAR-M50 (B.0
  measurements) recorded 2026-09-13; RAR-M54 (Super Rating Tournament read, 42
  engines) recorded 2026-09-15, nothing moves. **D.2's premise is contradicted
  by RAR-M46 and the leaf needs re-scoping**
- **2026-10-03 — PLAN B.8: the B.1 search deleted** in `84de712`, with the
  `b2core` umbrella that selected it. It was the 2.4.0 search that
  `--no-default-features` compiled; its last fingerprint was 7,590,542 /
  EBF 2.473, at `10d0e83` (the record: `analysis/b8_removed_2026-10-03.md`).
- **2026-09-29 — PLAN B.4.1 CLOSED: the quiescence cluster implemented behind
  `b4quiet`.** The PV bit carried on depth-0 stores (Stockfish form), fail-high
  interpolation at 583 / 562, the count rule with check and recapture escapes,
  the SEE threshold on every capture, margins as coordinates, and evasion
  pruning and quiescence noisy history as switches, both off by the canary
  rule; two research amendments on the way (the PV-subtree marking and the
  check exemption each lost canaries). The arm reads 10,226,874 / EBF 2.519,
  WAC 236 / 270, time-to-depth 0.85×; its paired run is B.4.2's.
- **2026-09-27 — PLAN B.7.1 CLOSED: allocation guard.** A counting-allocator
  test (`8836013`) holds per-node search code to AGENTS' no-allocation rule:
  allocations may grow per iteration, never per node, and a planted
  per-node allocation fails it. Every allocation left in a search is at the
  root node; B.7's own work moved to B.7.2.
- **2026-09-27 — PLAN B.3.5 CLOSED, B.3 closed with it.** `go ponder` with
  `ponderhit` or `stop` in the same write never got a `bestmove`: a `stop`
  made the queued `go` stale and a `ponderhit` was erased as the search
  started. Both signals are scoped to their `go`'s epoch (`ef1a24b`), with
  race tests at Threads 1 and 4 that fail on the old code; bench unchanged
  at 12,897,901 / EBF 2.523.
- **2026-09-27 — PLAN B.3.4 CLOSED: cluster 2 accepted.** The `b3proof` arm (NMP
  with a verification region, ProbCut with the TT gate, graded singular
  extensions with Rarog's multi-cut rule, the −3 negative extension, LDSE, the
  LMR singular term, a per-line extension budget), fitted by RAR-S82/S83 in
  blocks from the measured singular margin, passed its `[0,3]` gate against
  the B.2.7 head at 804 pairs, **+50.5 ± 10.9 Elo** (RAR-S84), and became the
  default build at **12,897,901 / EBF 2.523**. B.3.5 (the ponder race) stays open
  under B.3.
- **2026-09-25 — PLAN B.2.7 CLOSED, B.2 closed with it.** The Colosseum re-tune
  of all 82 `CoreParams` coordinates from B.2.3's theta (RAR-S78, 150,000
  games) was baked at **7,435,006 / EBF 2.457** and accepted by its `[0,3]`
  gate at +13.1 ± 5.4 Elo in 3,081 pairs. Ten coordinates moved a full
  step, led by LMR terms B.2.3 had left near their seeds.
- **2026-09-20 — PLAN B.2 CLOSED (first close; reopened for B.2.7).** B.2.3's SPSA finished at N = 5,000
  (160,000 games) and its theta was baked; B.2.4a (+65.09 ± 23.26) and B.2.4b
  (+138.60 ± 30.66) accepted the fitted selectivity core, which became the
  default build at **7,185,678 / EBF 2.444**. RAR-S77 measured the tune's last
  1,100 iterations at +4.43 ± 2.90. B.2.5 then fixed seven UCI `info` defects,
  two of them reported in GitHub issue maelic13/rarog#1, and B.2.8 removed the
  start-up stalls that cost games on time.
- **2026-09-15 — PLAN B.2.2.4 and B.2.2 CLOSED:** the screen ladder for
  B.3–B.5 is written into rule 8. The paired run governs, the floors are
  diagnostics, time-to-depth replaces pooled NPS, a positional screen and the
  canary regression rule are added, and a curvature sweep precedes any SPSA.
- **2026-09-15 — PLAN B.2.2.3 CLOSED, RAR-S73:** the curvature sweep found
  three curved coordinates (`CoreLmpSquare`, `CoreLmrQuiet`,
  `CoreCorrUpdateSlope`), so B.2.3 runs; P6 read +5.59% against < 3%.
- **2026-09-15 — PLAN B.2.2.2 CLOSED, RAR-S74:** six paired runs adopted no
  switch, kept mate-residual training (+15 Elo), and reverted the clamp
  conversion (−7.64) to four SPSA coordinates at the donor's seeds;
  `b2core` 4,706,910 / EBF 2.391 again at `308abe9`.
- **2026-09-15 — PLAN B.2.2.1 CLOSED, RAR-S74: the seed-scale clamps
  converted and five categorical switches on the `b2core` arm.** Two LMR
  clamps still in the donor's evaluation units were converted (×0.457),
  moving the candidate's bench from 4,706,910 to 6,586,667 / EBF 2.433;
  five `CoreParams` switches (razoring guards, mate-range residual
  training, singular-node training, the donor's full-depth branch,
  reductions in check and at the root) landed one commit each with a
  constructing test, defaults at today's behaviour; a `b2core,tune` PGO
  build and the run commands were handed over and RAR-S74 registered
  before any game. The five runs then read −2.6, −15.1, +2.3, +1.7 and
  −3.8 Elo for the switches: nothing adopted, and refusing mate-range
  residuals in correction training costs about 15 Elo, so the donor's
  admission stands.
- **2026-09-15 — PLAN B.2.0.2 CLOSED, RAR-P26: MultiPV.** `option name
  MultiPV` (1–256), clamped to the root set after `searchmoves` and the
  tablebase filter. Above one line a cold root loop searches each line with
  the ranked moves excluded through the existing root restriction, keeps the
  ranked lines in score order, and takes `bestmove`, ponder and the time
  signals from line 1; helpers search one line and do not vote. At the
  default nothing moved: both fingerprints, the depth-10 `info` stream on
  both arms, pooled NPS +0.62% [−0.29%, +1.18%]. Two defects the frozen
  contract did not name were found and fixed before commit: lines after the
  first searched a ply shallower than reported (the excluded stored root
  move fed IIR; fixed as Stockfish and Reckless do, by not storing later
  lines' root result and giving them their own root TT move), and a stopped
  depth filled unsearched lines from root records carrying fail-low bounds
  (found in a 4-thread `go infinite` session; fixed to use the last
  completed depth's reported lines, with a node-stopped regression test).
- **2026-09-14 — PLAN B.2.1 CLOSED, RAR-S73: the selectivity core implemented
  behind `b2core` and accepted by its reviewer.** Sixteen commits: the
  `Searcher` split (ticket 0), the diag-only decision trace, the umbrella,
  then tickets 1–8 in the handoff's order (threats and stack producers; TT
  eval store, node-typed cutoff, five-bit age and replacement refusal;
  threat-keyed histories and the staged picker with shadow continuation
  corrections; correction tables and the corrected-eval formula; razoring,
  reverse futility and hindsight; move-loop pruning with direct checks
  surviving the skip; late-move reductions in the donor's shape; the history
  update policy), tests and CI. Off arm exact at every commit (7,601,220 /
  EBF 2.474, pooled NPS −0.13%); candidate 4,706,910 / EBF 2.391 unfitted
  with 80 `CoreParams` coordinates. Three defects found by the cluster's own
  tests and instruments (rule-50 damping stored with the eval, the quiet skip
  dropping a late mating check, a budget-chaotic KBNK anchor). A separate
  reviewer (`analysis/b21_review_2026-09-14.md`) upheld the four resolutions,
  found no defect, and left a speed warning for B.2.2: a single-build screen
  read the candidate at about 0.71x of the off arm against a 0.90x pooled floor. B.2.3 and B.2.4 are
  registered with B.0's predictions verbatim; no game has been played.
- **2026-09-14 — PLAN B.2.0.1 CLOSED, RAR-M53: the documents restructured, each
  to one purpose.** The two retired trackers were recovered and archived
  (the review had called one lost and the other a duplicate; neither was);
  PLAN's closed leaves became one line each with their text archived;
  EXPERIMENTS became one row per experiment with prose records in
  `analysis/ledger_records_2026-09-14.md`; PROCESS kept procedures and took
  the independence boundary and the registration template; `analysis/` got
  an index and an archive; `check_guide.py` fails on dead paths in current
  documents; GUIDE and AGENTS were shortened, AGENTS after the maintainer
  approved the draft. The ticket that untracked six logo variants was
  reverted at the maintainer's instruction: logos stay tracked. The same
  session withdrew B.0's null-calibration hold, which contradicted RAR-M03.
- **2026-09-14 — PLAN B.2.0 CLOSED, RAR-P25: the architecture review's twelve
  upgrades landed behaviour-neutral.** The review
  (`analysis/architecture_review_2026-09.md`, RAR-M51) found the layering
  sound and the weight in comments, public surface and duplication. The
  upgrades: dead aliases and counters deleted; visibility narrowed to what
  external crates use, one move-generation API, private `Board` fields; one
  definition each of the duplicated helpers; a tagged `EngineCommand` with its
  own `ClearHash`; an `InfoSink` output port for the search; the table's
  replacement policy written once over both backends (NPS +0.04% on its own);
  the texel tuner as its own Cargo workspace; a tools index; retired step
  numbers removed from the owned source, configuration and tools. Fingerprint
  7,601,220 / EBF 2.474 exact at every engine commit; pooled NPS +0.21%
  [−0.34%, +0.68%] against the B.1 pool. The `Searcher` split it designed went
  to B.2.1 as ticket 0.
- **2026-09-14 — PLAN B.1 CLOSED, RAR-P24: the search restructure, exact
  fingerprint and 6.30% faster.** Seven engine commits split `search.rs` into
  `src/search/` modules, introduced `NodeType {Root, Pv, NonPv}`, `ThreadData`,
  `SharedContext` and a sentinel `PlyArray` stack, removed the 44 parameters
  inert at default, the root-confidence subsystem, the SMP iteration skip and
  TT provenance (`evidence.rs`), and re-keyed the diagnostic counters. Every
  commit reproduced 7,601,220 / EBF 2.474 on magic and PEXT with all 40
  positions identical; the B.0 section 11 baselines reproduced exactly; pooled
  PGO NPS +6.30% [+5.78%, +6.84%], the removed per-node work. RAR-S65–S69 were
  superseded by B.2.
- **2026-09-13 — PLAN B.0 DONE, `NO_CHANGE` to source, RAR-M50: the search
  programme's investigation.** `analysis/search_programme_2026-09-13.md`.
  Rarog's branching factor over depths 4–14 is 1.630, below the oracle's 1.736
  and Reckless's 1.697, so the deficit is not per-ply growth; it is decision
  quality at a fixed budget (WAC at 100k nodes: oracle 242, Reckless 224,
  Rarog 200; median depth at 300k nodes 16 against 19). 46.7% of LMR
  reductions land in quiescence and 1.3% are re-searched. Decisions: node
  types with runtime `cut_node`; `ThreadData` and `SharedContext` without
  changing table ownership; `evidence.rs` deleted; frozen handoffs for B.1–B.3
  and B.2's prediction (+35 Elo after fitting, 90% [+5, +70]).
- **2026-09-11 — PLAN A.9 DONE: Rarog 2.4.0 released, and PHASE A IS CLOSED.**
  `dev` squashed into `master` as one `Version 2.4.0` commit, which ran the CI
  matrix for the first time on the 1.98.1 pin and **discharged that standing
  hold**; the tag then triggered the release CI and its per-tier PGO assets,
  each `verify-isa` clean with the `bench 13` fingerprint reproduced. The
  release is licensed by **RAR-E16: +54.77 ± 17.04 Elo over 2.3.2**, H1
  accepted at 742 games, with a 400-game 4T direction check at +79.53 ± 21.21
  and zero forfeits — and the ledger's own caution stands, that an SPRT which
  stops at 4.6% of its cap reports an effect biased upward, so the honest
  reading is "clearly and substantially positive, magnitude not settled".
  **What 2.4.0 contains**, each individually gated: the ProbCut move filter
  (+15.44 ± 8.06), root-only LMR relief (+2.33 ± 1.85 over 56,928 games), two
  complete HCE refits (+22.04 ± 7.51 and +11.81 ± 5.33), tablebase-corrected
  training labels (+6.73 ± 3.82) and the board cluster (+12.12 ± 10.17) —
  sequential gates under different baselines, **not additive**. Plus the SEE
  and boundary repairs, the KBN-K mate drive (19.4% → 96.9% conversion), the
  `go`-parse clock origin, the startup CPU advisory, argv-as-commands and the
  `help` command. **Phase A's lasting output is not the release but the
  instruments**: a document set a checker keeps honest, and four baselines on
  the released binary that Phase B and C are measured against.

- **2026-09-11 — PLAN A.7 and A.8 COMPLETE: 2.4.0 bumped, all four baselines
  measured on it.** The version bump (`c6a548f`) moved `Cargo.toml` to 2.4.0
  with `bench 13` unmoved at **7,601,220 / EBF 2.474**, and A.8 then measured
  the released engine rather than a predecessor of it. **A.8.1 (RAR-M45)**,
  twelve engines, 600 games per pair, 39,600 games, **zero forfeits and zero
  crashes**: Houdini 3 −224, Critter 1.6a −184, Houdini 1.5a −179, Fritz 16
  −147, Rybka 4 −99, Basilisk 1.10.0 −23, Basilisk 1.9.3 **−9**, Rarog 2.3.2
  **+70**. Two frozen predictions scored 4-of-6 and 3-of-3, both misses in the
  direction of Rarog being stronger, and the ±40 drift check passed on all
  eleven unchanged opponents. **A.8.2 (RAR-M46)** then refuted its own
  prediction *in sign*: 4T deficits are 26 to 75 Elo **smaller** than 1T
  against three of four targets, Rarog beats Basilisk 1.10.0 at 4T (**+25**)
  having lost at 1T, and the performance rating is 3034 against a frozen 3003.
  **A.8.3 (RAR-O03)** put G(0) at **−247.97 ± 10.89** with the evaluation
  proved constant three ways, and found the depth gap is only **0.97 ply**
  (corrected 2026-10-03 to 1.4 non-mate plies: the reading mis-attributed
  Black-to-move openings; RAR-O03 carries the correction) — so
  most of a 248-Elo deficit is decision quality, not depth, which corroborates
  the matched ablation from an independent direction. **A.8.4 (RAR-M48)** set
  the speed baseline at **3.19 MNPS** pooled median with the instrument
  resolving ±0.2%; its first attempt was discarded because the host was at
  12–16% CPU, and re-measuring the same binary idle moved it 2.3%. **Two
  planning consequences:** D.2's premise is contradicted and the leaf needs
  re-scoping against modern engines, and E.2's binding arm is **1T, not 4T**.

- **2026-09-10 — PLAN A.3.3 to A.6 CLOSED.** The time-forfeit repair started
  the search clock at `go` parse as both donors do: RAR-R11 measured **zero
  forfeits in 10,000 games** at −0.69 ± 3.62, and the competing overhead-sweep
  explanation was refuted at −81. **A.4** fixed what the universal-binary
  investigation found in the system actually shipped — the `cc` 1.3.0 → 1.4.5
  bump with the MSRV lockstep repaired (RAR-P21), a startup CPU advisory for
  the slow-PEXT trap, the measured `pext`-over-`base` figure of **+6.65%**
  (RAR-P22, showing chaining had overstated it by 0.50 pp), README asset
  guidance carrying those numbers, and argv handling so `rarog bench 13` no
  longer exits 0 having done nothing. Every A.4 change reproduced 7,601,220.
  **A.5** rebuilt the conversion instrument on PGN rather than Colosseum and
  reproduced its seed exactly (RAR-M47); **A.6** concluded that **Phase A
  refactors nothing**, handing its findings to B.1 and C.1.

- **2026-09-10 — Deleted-branch arms preserved.**
  An audit of every SHA cited in
  this file found 122 tokens: 84 reachable, 9 not our commits or already gone,
  and **29 dangling** — cited by a row but reachable from no branch or tag. Six
  were experiment arms with a tight, meaningful diff; their patches now live in
  `analysis/arm_patches/`, each verified to apply to a reachable baseline and to
  depend only on blobs that survive a prune. Thirteen touched no `src/` file and
  needed nothing: their content is already in the documents they edited. Nine sit
  deep on a 113-commit line that forked from `a5fd288` and was never merged, so a
  diff to any reachable base is a whole-branch snapshot rather than a recipe;
  those rows are closed findings, and `ba3170b` (RAR-S20), the only one with a
  parameter recipe, already carries its seven values and both fingerprints inline.
  That line is unreachable from refs but still held by the reflog, so `git gc`
  does not remove it. `analysis/arm_patches/README.md` has the detail.

  **Three cited SHAs were already gone before this audit** — `0ddc8e5` and
  `3ee4660` (RAR-P16) and `7693010` (RAR-S54). All three rows anticipated it and
  carry their recipes, which is why nothing was lost.
- **2026-09-09 — Test-engine store cleared.**
  `tools/test_engines/` held 183
  executables, 181 of them built on the retired 1.97.1 pin and 73 of those with
  no manifest at all, so they could not be used in a gate anyway — `sprt.ps1`
  refuses a pair whose compilers differ and warns when equality is not
  checkable. All of them were deleted on maintainer instruction after the A.3.1
  bump; rows that cite a path under `tools/test_engines/` now rest on their
  recorded recipe and fingerprint, which is what the ledger's own rule requires
  of them. Rebuild from the row when a binary is needed again. **Kept:** the two
  RAR-E16 gate arms, `tools/test_engines/ablate/` (the frozen matched-ablation
  oracle and its HCE glue, which B.9 still needs and which is not cheaply
  rebuilt), and `rarog-43b-cand.diff`, a recipe rather than an artifact.
- **2026-09-09 — PLAN A.1 to A.3.2 CLOSED: the reset itself.** New PLAN, GUIDE
  and HISTORY with the Phase-4 line archived and a mechanical checker
  (`check_guide.py`); twelve superseded tracked files removed; seven branches
  tagged and deleted with the oracle package archived; and a 42-parameter,
  55-seed feature inventory. The toolchain was pinned to **1.98.1** and
  qualified behaviour-neutral (RAR-P18). **RAR-E16, the consolidation release
  gate, accepted H1 at 742 games: +54.77 ± 17.04 Elo over 2.3.2**, with a
  400-game 4T direction check at +79.53 ± 21.21 and zero forfeits. Its lower
  bound of +37.73 cleared the section-4 rule, licensing **2.4.0**.

- **2026-09-09 — PLAN 4.11b.18 CLOSED, RAR-M42; SECTION 4.11b COMPLETE:**
  endgame evidence refreshed against the accepted board head. Layer-1 theory is
  identical on all 19 families — no clean win newly discarded — and floors PASS
  on both arms, the 4.11 head reproducing the registered aggregate exactly.
  The registered 4.12 order was **rederived** and reproduces
  `endgame_ranking_v2.json` across all twenty families, with the accepted head
  matching, so no renumbering is needed. Conversion alone would have said
  "nothing changed": its four families are bare-king and byte-identical, while
  the frozen corpus splits 34.5% of both-sides positions differing against 0.0%
  bare-king, because SEE fires only where captures exist. Evidence versioned as
  `endgame_drawn_census_v2.json` and `endgame_truth_baseline_v2.json`; every v1
  artifact and the floors file untouched. Owed: KRP-KB win-preserving -2.2 SE,
  non-blocking, owner 4.12.6. 4.12.1 is next.

- **2026-09-08 — PLAN 4.11b.17 ACCEPTED, RAR-E15:** the integrated 4.11b board
  cluster passes its playing gate. H1 accepted at **1,950 games**, 12% of the
  16,000 cap: Elo **+12.12 +/- 10.17**, nElo **+18.40 +/- 15.42**, LLR 2.96 of
  +/-2.94, LOS 99.03%, W-D-L 530-958-462, one timeout at 0.051% (below RAR-M14's
  floor). Bounds `[-5,5]` and every setting were registered before games and
  match the run manifest exactly. **The registered prior was badly wrong** — it
  predicted -4 to +4 nElo by treating a +10.14% node increase as a tax, but the
  tree grew because the search stopped pruning incorrectly, which is the
  opposite sign from widening a search. **RAR-M10 predicted 1,925 games against
  1,950 actual**, validating it outside its stated +/-6 nElo range. Magnitude is
  imprecise at +/-10.17 Elo and no subcomponent is credited. The development
  fingerprint 7,601,220 / EBF 2.474 now has its integrated verdict and is the
  accepted foundation for 4.12. 4.11b.18 is next.

- **2026-09-08 — PLAN 4.11b.16 QUALIFIED, RAR-M41:** the integrated board
  cluster banks **+1.421%** whole-search NPS, 95% [+0.953%, +1.764%], under
  production pooled-PGO settings on a verified-idle host, with behaviour
  identical to the baseline throughout. A null pair of two same-revision PGO
  builds measured +0.222% [-0.130%, +0.630%], confirming the instrument is
  unbiased and setting the effective floor empirically rather than by
  assertion. 91/96 pairs faster, max host busy 9.11%. Correctness matrix passed
  in full, including 72 tests under the PEXT backend and the fingerprint on all
  six binaries. Projected half-width ~0.5%, measured 0.405% — a calibration hit
  after RAR-M33's miss. No Elo claimed; the playing gate is 4.11b.17, which is
  next.

- **2026-09-08 — PLAN 4.11b.15 closed `NO_CHANGE`, RAR-M40:** all four draw
  policies kept, each dispositioned independently with its own retry trigger.
  RAR-S18's two losing arms do not isolate any single part, and that is stated
  rather than glossed. Cross-null repetition resolves structurally: the full
  hash includes side to move so crossing a null yields only false negatives,
  and the arbiter path never sees nulls at all. Partial root-awareness already
  exists via the `ply > 0` guard. Key separation audited clean and now pinned
  by two tests; proving the identity test live took three sabotage attempts.
  Engine untouched at 7,601,220 / EBF 2.474; debug 282 / release 283, fmt and
  Clippy clean. No games. 4.11b.16 is next.

- **2026-09-08 — PLAN 4.11b.14 closed `NO_CHANGE`, RAR-M39:** no larger
  representation change is justified. No board region exceeds 6.7%, so the
  leaf's own gate for opening an implementation is not met. Six type boards
  would save 48 bytes of a 264-byte Board that was never near a cache boundary,
  while taxing 208 `pieces()` sites, 102 of them in the 29.49% eval region.
  Per-ply state copying would cost 33 KiB against the current 3 KiB and leave
  L1. Legality is already amortized at 517:1 fast-to-full check calls. Only a
  compile-time footprint guard was added, proven live by adding a field.
  Neutral at 7,601,220 / EBF 2.474; debug 280 / release 281, fmt and Clippy
  clean. No games. 4.11b.15 is next.

- **2026-09-08 — PLAN 4.11b.13 done, RAR-M38:** `f70ac19` reserves `MAX_PLY`
  of history headroom on the root before the hot path, with worker clones
  inheriting it through `Board::clone`'s capacity preservation. The gap was
  real but invisible to the instrument that looked for it: peak depth is
  game plies plus search depth, so an ordinary 64-move game reallocates on the
  next search, yet RAR-M30 saw zero growth because bench starts every position
  from FEN with empty history. `is_legal` audited and found to have no
  production callers; its canonicalization trap is now documented and pinned.
  Five contract tests, the clone one proven to fail on regression. Neutral at
  7,601,220 / EBF 2.474; debug 280 / release 281, fmt and Clippy clean. No
  speed claim, no games. 4.11b.14 is next.

- **2026-09-08 — PLAN 4.11b.12 closed `NO_CHANGE`, RAR-M37:** king-square
  caching is not prototyped. The refreshed profile reads the lookup at 0.502%,
  whose 2x-local ceiling of 0.25% is two to four times smaller than the
  measured width of the instrument that would have to accept it, so the best
  possible version cannot be distinguished from zero by any budget used here.
  No floor was declared, deliberately: the number was already exposed twice and
  a threshold chosen now would be fitted to the result. `king_sq` is one
  bitboard load plus a tzcnt; a cache would add maintenance across castling,
  undo, worker cloning and consistency reconstruction for at most a quarter of
  a percent. No engine change, no games. 4.11b.13 is next.

- **2026-09-08 — board profile recipe recovered and refreshed, RAR-M36:**
  RAR-M30's per-sample attribution turned out to be a side effect of xperf
  failing to find the PDB, which `952711f` then fixed — silently switching the
  report to per-function aggregation and making the summarizer resolve function
  END addresses while reporting "100% resolved". The recipe is to deny xperf
  symbols deliberately (empty symbol path AND symcache AND no adjacent PDB).
  Refreshed shares at head: make/unmake 6.677% (was 7.143%), SEE 5.239%,
  generation 6.556%, check queries 5.179%, king square lookup 0.502%. The drop
  in make/unmake is 4.11b.9, measured by an instrument that knew nothing about
  it. A stale `piece_relocation_helpers` marker was fixed for `::move_piece`.
  Per-function view shows `see_recapturer` 4.35% vs `see_ge_impl` 0.87%,
  supporting RAR-M35's redirect. No engine change, no games.

- **2026-09-08 — PLAN 4.11b.11 closed `NO_CHANGE`, RAR-M35:** incremental SEE
  attacker maintenance was built, proven correct and still rejected. `bench 13`
  exact, all 41 external fixtures passing, and a debug equivalence assertion on
  every SEE call -- proven live by deliberate sabotage -- held across 275/275
  debug tests. But the registered stage-1 screen measured `threshold SEE only`
  at -2.92 / -10.42 / -0.69%, zero rounds up against a required +5%, so stage 2
  never ran and the change was withdrawn. A leaf premise was wrong: the two
  per-step `attackers_to_color` calls are not duplicates -- the second is the
  mandatory per-candidate king-legality test, which is where SEE cost actually
  sits and where future work should aim. No games, no Elo. 4.11b.12 is next.

- **2026-09-08 — PLAN 4.11b.10 closed `NO_CHANGE`, RAR-M34:** shared pin/check
  state is not worth building because there is nothing left to share.
  `compute_pinned` and `check_info` query different king squares against
  different slider colours; SEE king safety runs on the evolving exchange
  occupancy, where reuse is barred by the 4.11b.5 contract; and one pinned set
  per node across capture/quiet stages already exists (422,246 staged quiet
  generations at zero extra `compute_pinned`). Activation is low anyway:
  0.274 and 0.295 calls per node against SEE's 0.993. Counters exact at stride
  1 with nodes unchanged at 7,601,220. No ETW re-profile requested — it needs
  elevation and cannot change a structural finding. No code, no games, no Elo.
  4.11b.11 is next and owes an SEE re-baseline.

- **2026-09-07 — PLAN 4.11b.9 ACCEPTED, RAR-M33:** `5c439da` fuses ordinary
  quiet relocation behind `Board::move_piece`. Behaviour-neutral — both builds
  fingerprint 7,601,220 / EBF 2.474 and 640/640 paired root answers match
  including full PV and ponder. Isolated make/unmake +16.33/+17.30/+19.32%;
  full-search median +0.876%, 95% [+0.050%, +2.055%], excluding zero on a
  verified-idle host (max 11.80% CPU busy). Debug 275 / release 276 tests, fmt
  and Clippy clean. Interval half-width came out at 1.003% against a registered
  0.33–0.46% — a power-projection miss, recorded. No Elo claimed; PGO
  qualification is 4.11b.16 and the playing gate 4.11b.17. 4.11b.10 is next.

- **2026-09-07 — RAR-M32 VOIDED (superseded by RAR-M33):** the earlier
  measurement ran while a Manta SPRT held the host at ~50% CPU busy; the same
  baseline code re-measured 40.7% faster once idle. Its `NO_CHANGE` disposition
  is withdrawn. The harness recorded that load and nothing asserted on it, so
  the runner now fails on host load instead of annotating it. Retained below
  with explicit supersession rather than deleted.

- **2026-09-07 — PLAN 4.11b.9 originally closed `NO_CHANGE`, RAR-M32 (VOID):** the fused
  ordinary-relocation path was semantically exact (both builds 7,601,220 /
  EBF 2.474; 240 paired root answers match including full PV and ponder) and
  gained +16.28/+15.21/+15.27% on the isolated make/unmake primitive, but its
  full-search median of +1.016% carried a bootstrap interval of -0.450% to
  +3.609% and failed the rule frozen in `86e39f8` before timing. Emitted code
  grew (`make_move_inner` 468 -> 568 instrs), so LLVM was not already fusing
  it; the miss is instrument power, not mechanism. Production path withdrawn,
  `src/` byte-identical to `af83abf`; per-piece-class test retained in
  `8a73cfd`. Debug 275 / release 276 tests, fmt and Clippy pass. No games.
  4.11b.10 is next; a powered retry belongs to 4.11b.16.

- **2026-09-07 — PLAN 4.11b.8 closed by withdrawal, RAR-M31:** `c44608a`
  restores the prior pin calculation and retains the independent oracle.
  This supersedes the retention below, not the historical local gains; useful
  whole-search value remains uncertain. Debug 274 / release 275 tests, fmt,
  Clippy, fresh before/after fingerprints and 20 profile identities pass.
  No new timing/games. 4.11b.9 is next; conditional retry belongs to 4.11b.10.

- **2026-09-07 — measured work within PLAN 4.11b.8, RAR-M31:** simplified pin discovery in
  `2ea279f`; local legal/capture/staged generation gains 8.54%/11.43%/7.41%.
  Generic/PEXT search estimates +0.57%/+1.45% are inconclusive. Debug/release,
  independent board/pin oracles, PEXT checks, fmt and Clippy pass; fingerprints
  and 480 paired root answers match. Strength gate remains 4.11b.17.
  Evidence: `analysis/movegen_2026-09-07.md`. Leaf remains open: the later
  research contract requires a prospective whole-search floor absent from
  this run. Next is 4.11b.8 research disposition, before 4.11b.9.

- **2026-09-07 — PLAN 4.11b.7, RAR-M30:** profiled 20 frozen roots in five
  actual-search cohorts. Native samples put generation/legality at **6.751%**,
  make/unmake at **7.143%**, check queries at **5.177%**, and SEE at **5.304%**
  of full process time. Sixty instrumentation-off searches match their recorded
  depth, seldepth, nodes, score and best move; PV and ponder were not compared.
  The 30.6M diagnostic nodes show checked make dominates plain make and history
  grows zero times. Evidence: `analysis/board_search_profile_2026-09-07.md`.
  Next is 4.11b.8.

- **2026-09-07 — PLAN 4.11b.6, RAR-M29:** added neutral board-owned SEE
  values, proved production identity and the benchmark wire, and restored the
  normalized three-engine SEE comparison. All adapters agree on values and ten
  verdicts; medians are **44.923/58.335/40.823 M captures/s** for Rarog/
  Basilisk/Reckless, with Rarog's 12.20% scatter recorded. Full engine/tool
  checks pass; no fitting or games. Evidence:
  `analysis/see_value_injection_2026-09-07.md`. Next is 4.11b.7.

- **2026-09-07 — PLAN 4.11b.5, RAR-M28:** repaired evolving SEE legality,
  selected-king handling and recapture promotion accounting in `fce0b44`.
  All 41 independent fixtures and 1,802-capture parity checks pass; complete
  debug/release suites, Python, fmt and Clippy pass. Production fingerprint
  **7,601,220 / EBF 2.474**; playing qualification remains at 4.11b.17.
  Evidence: `analysis/see_repair_2026-09-06.md`. Next is 4.11b.6.

- **2026-09-06 — PLAN 4.11b.4, RAR-M27:** inventoried every SEE caller and
  special-move policy; added 18 external legal capture-tree fixtures. Confirmed
  king parity debt and found newly created pin/recapture-promotion defects;
  all three have failing acceptance tests owned by 4.11b.5. Eight Rust tests
  pass per profile (three explicit debt ignores), five Python checks and
  fmt/clippy pass. No engine code changed. Contract/evidence:
  `analysis/see_contract_2026-09-06.md`. Next is 4.11b.5.

- **2026-09-06 — PLAN 4.11b.3, RAR-M26:** made malformed non-ASCII UCI move
  tokens reject before byte indexing, retaining the controlled fatal UCI
  position policy instead of panicking. Defined `u16` fullmove saturation at
  65,535 for real/null black moves, with both colors and undo paths tested;
  65,536 is rejected. Debug/release suites, fmt and clippy pass; exact
  default-feature `bench 13` remains 6,901,489 / EBF 2.458. Next is 4.11b.4.

- **2026-09-06 — PLAN 4.11b.2, RAR-M25:** added the versioned board-v2
  external-oracle corpus and isolated benchmark without changing the frozen
  cross-engine v1 benchmark. The ten cases cover checks/evasions, EP,
  promotions, all castles, sparse material and long histories. Rarog's
  identity/perft/divide/state paths, magic/PEXT coordinate rays and a
  zero-allocation guard pass; the structural preflight negative controls reject
  wrong moves, work and state. Raw samples and a full local manifest are at
  `analysis/artifacts/board-v2-20260906/`. No engine or strength claim. Next
  is 4.11b.3.

- **2026-09-06 — PLAN 4.11.10, RAR-M24:** reran preserved RAR-E08
  baseline/head and RAR-E08/E12-candidate binaries through the repaired v2
  truth runner. RAR-E08 aggregate conversion is corrected from its invalid v1
  value to 1255/1372 -> 1254/1372; the KQ-KP 400-position regression survives
  exactly at -3.79 pp. RAR-E12 aggregate conversion is 1254/1372 ->
  1278/1372, but its KQ-KP conversion is 96/98 -> 94/98 despite improved DTZ
  progress. RAR-E11 is superseded in full: reference 1361/1372, current head
  1276/1372, reference worse in no family. The archive is self-reproducing at
  `analysis/artifacts/conversion-claims-correction-20260906.zip`; no engine
  change or strength claim. Next is board work at 4.11b.2.

- **2026-09-06 — PLAN 4.11.9, RAR-M23:** paired the archived 4.9a.4 reports
  over their identical 19-family cohort and established the mate drive's
  promotion closure. Six families change; KBP-KB and KBP-KN each fall one net
  conversion and now carry debt at 4.12.7/4.12.9. The old material-shed
  instrument makes this a historical causal matrix, not current conversion
  floors. Preserved reports, derivation and matrix in
  `analysis/artifacts/mate-drive-promotion-closure-20260906.zip`. No engine
  change, game or strength claim; its remaining conversion correction completed
  later as RAR-M24.

- **2026-09-06 — PLAN 4.11.8, RAR-M22:** audited all 1,202,619 source games
  for `hce-v2` and the `hce-v3` source of `hce-v3-tb`, at the hash-pinned
  8,000-node datagen budget with 3–6-man Syzygy. Raw game-result contradiction:
  hce-v2 26,316/134,948 clean wins (4.39% of all games); hce-v3 source
  54,186/266,490 (8.99%). `hce-v3-tb`'s 125,643 corrected ≤6-man rows remain
  distinct from this game-level instrument. Preserved reports/provenance in
  `analysis/artifacts/datagen-label-audit-20260906.zip`; 19 audit-tool tests
  passed. No engine change or strength claim. 4.13.1 owns row-level lineage.

- **2026-09-06 — PLAN 4.11.7, RAR-M21:** completed the authorized
  60k/200k/600k conversion bracket, all 19 families and both engines.
  Net reference deficit 85/27/16 of 1372 initially won starts. Both historical
  60k reports reproduced exactly. Preserved six reports, paired FEN changes,
  provenance and validation in `analysis/artifacts/budget-transfer-20260905.zip`.
  Debug/release tests, fmt, Clippy and 156 tooling tests passed. No engine
  change. 4.11.8 later completed as RAR-M22.

## Closed work through 2.3.2

Phases 0–3 built the engine, the harness, the correctness programme, the
search wave and the reproducible build/CI/PGO line, through 2.3.0 and the
2.3.1 ARM64 patch. The closed Phase-4 line shipped 2.3.2: broad selectivity
fit **+15.33 ± 7.34 nElo**, zero-reduction LMR floor **+9.13 ± 5.45 nElo**,
anchored Texel refresh **+11.56 ± 5.19 Elo**, the NMP mate-score clamp as a
correctness repair, AArch64 TT prefetch at **+1.42% NPS**, and the executable
ISA contract. Those three strength results used different estimators and are
not additive.

**Its item numbers 4.0–4.10 are retired** and are not reused by later
trackers. Ten abandoned parameters were removed with their accepted defaults
hardwired at the call sites; the root-gap observation stays in diagnostics but
cannot enter root confidence, because null-window rival scores made it
degenerate. Full detail and the retained-inert ownership table are in section 3
of [docs/archive/PLAN-phase4-2026-09-09.md](archive/PLAN-phase4-2026-09-09.md).
