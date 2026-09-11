# Rarog history

⚠ **THIS FILE IS HISTORY. It does not tell you what to do next.** Read
`GUIDE.md` for the current step and `PLAN.md` for what it involves.

## Numbering, and how to resolve an old reference

Three numbering schemes exist in the ledger, the analyses and source comments.
None of them is the current roadmap's, which uses lettered phases (`A.2.1`).

| Scheme | Where it appears | Resolve it in |
|---|---|---|
| Legacy Lynx/Rarog phases (`8.2(a)`, `9.0a`, `10.3 speed pass`) | source comments, oldest ledger rows, the tracker below | the tracker section of this file |
| Phase 4 roadmap (`4.5`, `4.9a.4`, `4.11b.19`, `4.12.7`), Phases 5–9 | `EXPERIMENTS.md`, `analysis/*.md`, commits up to `f10b999`…`c80df74` | [docs/archive/PLAN-phase4-2026-09-09.md](docs/archive/PLAN-phase4-2026-09-09.md) and [docs/archive/GUIDE-phase4-2026-09-09.md](docs/archive/GUIDE-phase4-2026-09-09.md); the retired-to-current map is PLAN section 6 |
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

## Legacy tracker (retired numbering, frozen)

The detailed per-step checklist of the pre-Phase-4 line, split out of
`GUIDE.md` on 2026-08-21. **Item numbers here are FROZEN and retired**, and any
UNCHECKED forward item below was **superseded** by the Phase-4 line and then
by the current roadmap. The old numbers are kept because commits,
`EXPERIMENTS.md` rows and analysis documents cite them; the numbering schemes
do not correspond and are not meant to.

## Completed current-roadmap work (dated records; PLAN owns IDs)

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
  proved constant three ways, and found the depth gap is only **0.97 ply** — so
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

**Its item numbers 4.0–4.10 are retired** and are not reused by the tracker
below. Ten abandoned parameters were removed with their accepted defaults
hardwired at the call sites; the root-gap observation stays in diagnostics but
cannot enter root confidence, because null-window rival scores made it
degenerate. Full detail and the retained-inert ownership table are in PLAN §3.

## Forward tracker

<!-- FORMATTING RULES for this tracker — follow them, they get broken often:
     1. ONE step per `- [ ]` bullet. Never join two steps on one line with
        "·" (e.g. "4.4 foo · 4.5 bar") — each gets its own bullet, always.
     2. Continuation lines indent 6 spaces so they align under the text after
        "- [ ] ". SUB-ITEM INDENT IS 4 SPACES with 10-space continuations.
     3. Status boxes: `[ ]` todo · `[~]` ONLY while genuinely in flight (a
        gate running right now) · `[x]` finished — accepted, rejected,
        deferred or parked. Anything resolved is `[x]`, never `[~]`.
     4. Every item opens with its STEP NUMBER, then (for `[x]` items) a
        BRACKETED OUTCOME TAG in bold, so the reader orients by number
        first and reads the result second (number BEFORE tag, never the
        reverse):
            - [x] 4.5 **[ACCEPTED +22.13 ± 7.28, LOS 100%]** Cluster A ...
            - [x] 4.6b **[REJECTED −6.6]** retry — ...
            - [x] (b) **[DEFERRED → 4.8]** late evasions — ...
            - [x] 4.4 **[PARKED → 5.1]** dirty-piece deltas ...
            - [x] 4.2 **[DONE, no games]** Diagnostic counters ...
        Tags: ACCEPTED <elo> · REJECTED <elo> · DEFERRED → <item> ·
        PARKED → <phase> · DONE · FIXED. Put the Elo in the tag, detail
        after. The number must be ON the bullet line itself.
     5. Bullet order: step number, outcome tag, short title, then detail.
     6. NEVER renumber existing items. PLAN.md freezes item numbers because
        commits and history reference them. To insert before the first item
        use a .0; to subdivide, use letter sub-items like 4.5(a)/(b).
     7. Mirror any status/number change into PLAN.md in the same commit.
     8. Blank line AFTER the `###` phase heading, then NO BLANK LINES between
        bullets at all. A blank line splits one list into two and the
        renderer re-spaces everything around it.
     9. ONLY NUMBERED STEPS live in the tracker. A recurring procedure or a
        checklist is NOT a step — it never gets ticked, so an unticked box
        reads as outstanding work forever. Those go in
        `## Recurring procedures`, and the owning step links to them.
    10. Wrap at ~76 columns. Do not let one bullet run to 100+ columns
        because the sentence "felt continuous".
    11. Quick mechanical check after editing — all five must be zero:
        blank lines inside a list; bare `    - ` sub-bullets without a box;
        unnumbered `- [ ] **` pseudo-steps; lines over 78 columns; and
        `(a)`/`(b)` labels written mid-sentence inside a parent's own
        continuation instead of as their own `    - [ ]` line. -->

The model implements and verifies locally; the maintainer runs only the long
game jobs. One item is open at a time and each candidate gates against the
then-current accepted head. Macro-order: **A** search work (4.0–4.10) → **B**
HCE work (4.11–4.18) → **C** transfer and release (4.19) → **D** NNUE (5
runway → 6 baseline → 7 frontier) → **E** scaling (8) → **F** contingent
classical fallback (9, last, may never run). Per-item rationale is in
`PLAN.md` §4–§9.

### Phase 4 — Reference-accelerated search and HCE work (→ conditional 2.4.0)

- [x] 4.0 **[DONE, no games]** Evidence, baseline and oracle freeze — RAR-M12.
      2.3.2 reproduced from `dev` `5294e2c` (code byte-identical to `master`,
      doc-only diff), rustc 1.97.1 as pinned. fmt and all-feature clippy
      clean; tests 258/0 debug and 259/0 release, the one-test gap being a
      documented release-only `cfg`. Bench **6,519,711 / 2.449**; tune build
      advertises 101 options with all ten removed absent and the inert ones
      present; PGO PEXT asset reproduces the fingerprint at SHA-256
      `389E234E…05046E28` and passes `verify-isa`. Oracle binaries re-hashed
      byte-exact. Budget and stop rules registered. `hybrid` and `spsa_impr`
      pushed, so the oracle is no longer single-machine.
- [x] 4.1 **[DONE, no games]** Instrumented oracle — `hybrid-diag` at
      `de568b3`, implementing `analysis/phase4_counter_spec.md`. Verified:
      diag OFF reproduces the frozen binary exactly (bench 136,903, bestmove
      e7e3, zero diag lines); diag ON gives the same 136,903, so the
      instrumentation does not perturb the tree. Built-in invariants hold —
      `best_rank_1` == `cutoff_first_move`, rank buckets sum to the cutoff
      total, `main_tt_probes` == `nodes`. Build with `make build
      ARCH=x86-64-bmi2 COMP=mingw diag=yes`. **The suite must drive `bench`; a
      piped `go … quit` aborts before the search starts.**
- [x] 4.2 **[DONE, no games]** Differential observation harness — RAR-S55.
      Versioned fixed suite (UHO openings, quiet middlegames, tactics, checks,
      zugzwangs, endgames) at fixed depth/nodes, 1T. Counters for TT
      producer/consumer kind, **prune recall and overlap** (not node savings —
      a smaller tree can be worse), **correction attribution**, history
      attribution, move source, cutoff index, LMR/re-searches, pruning,
      extensions, aspiration and root ownership. Off ⇒ bench 6,519,711
      exactly; on ⇒ same best move and nodes. Run it against the 4.1 oracle:
      the counters that diverge most select the work. **Shadow-record**
      stand-pat, ProbCut, NMP/IIR/singular, checking-move LMR and
      root-confidence concerns — each is owned by the cluster that reaches it,
      else 7.3. Recording is mandatory; acting here is not permitted.
- [x] 4.3 **[DONE, no games]** Mechanism map and order freeze —
      `analysis/phase4_mechanism_map.md`. **Execution order changed on the
      evidence: 4.7 runs first**, then 4.5, 4.6, 4.8, 4.9; numbers unchanged.
      Ordering premise refuted (Rarog's first-move cutoff beats the reference
      in every cohort), so 4.5 drops to low expectation. Six mechanisms are
      classed UNKNOWN and their owning cluster must measure before designing.
      Classify each reference contract as equivalent / intentionally different
      (with reason) / missing / coupled to a later consumer, against the Rust
      owners in PLAN §4. If the evidence contradicts the cluster order, edit
      `PLAN.md` **before** implementing — never after seeing games.
- [x] 4.4 **[DONE, nothing required]** Search-consumed board state. Audited
      against 4.7's three leads (null-move entry, move-count volume, ProbCut
      entry): they consume `improving`, `eval_for_pruning`, depth and beta,
      all already available, and the one board-state input their pruning block
      uses is `CheckInfo` — already a per-node lazy cache (`node_ci`) with
      per-move memoisation. Building persistent
      pins/blockers/`plies_from_null` now would be speculative state with no
      consumer, which rule 2 forbids and the step itself warns against.
      Deferred to whichever cluster needs one, else 5.1. Cache only the
      per-ply state a 4.5–4.9 contract actually consumes: `CheckInfo`,
      pins/blockers, check squares, `plies_from_null`, repetition distance.
      Bench-identical where behavior-neutral; pooled-PGO NPS gate where it is
      a layout change. The evaluator-facing dirty-piece delta contract stays
      owned by 5.1 — do not let this grow into the NNUE runway.
- [x] 4.5 **[REJECTED, no gain]** Cluster A — per-ply authority, ordering,
      histories and LMR. All five sub-items closed. RAR-S64 took H0 at
      +0.39 ± 4.89 once the stale-reduction defect was fixed, so the
      +4.50 RAR-S61 saw was the defect. Structural work retained at no
      strength claim; `lmr_prior_reduction_adj` removed. Head
      7,467,143 / EBF 2.477. **Unexecuted scope was numbered, not
      dropped: history semantics → 4.9b, reduction contract → 4.8.1.**
      One dependency-complete cluster. Prior 15–45 nElo; it rests on evidence,
      reduction and re-search coordination, not raw ordering quality.
    - [x] (1) **[DONE, no games]** Rarog search context: `NodeContext`
          replaces the three parallel per-ply arrays. Behaviour-neutral —
          bench 6,922,439 / 2.451 exactly — and NPS-neutral at +0.11%,
          CI −0.14%..+0.48% over three PGO builds per arm (RAR-P17). TT/PV
          evidence, prior reduction, statistical score, cutoff count,
          previous-PV following and continuation keys are deliberately NOT
          added: nothing consumes them yet, so they land with 4.5.2–4.5.4.
    - [x] (2) **[DONE, no games]** Move-picker contract: a named `Stage`
          enum replaces three implicit cursor comparisons, and
          `Stage::GenerateQuiets` replaces the `quiets_generated` bool.
          Behaviour-neutral, bench 6,922,439 / 2.451. Legality and duplicate
          guarantees are now asserted, not assumed — three tests cover both
          paths including in-check, 251/245 total. Quiet suppression is
          CLOSED as intentionally different; see the note under this
          cluster. **Previous-PV following moves to 4.5.3**, whose evidence
          work is where a previous-PV move would be scored.
    - [x] (3) **[DONE, no games]** Evidence ownership. Continuation key in
          `NodeContext`, derived by `push_move`, which is now the only way to
          put a move on the stack — that found and fixed the ProbCut piece
          desync (bench 6,922,439 → 7,467,143). Every continuation site reads
          the stored key. Continuation-malus asymmetry measured and REJECTED
          (RAR-S59): it is a disguised selectivity increase, not an ordering
          fix.
    - [x] (4) **[DONE, no games]** Reduction/re-search contract. Prior-
          reduction authority ADOPTED at 512/1024 ply (RAR-S60): cutoffs per
          node rise faster than nodes and first-move cutoff improves 88.04% →
          88.18%. bench 7,467,143 → 7,587,235. Cutoff count REJECTED as inert
          — a cutoff breaks the move loop, so a per-visit count is 0 or 1.
          Statistical score and TT/PV evidence REJECTED as per-ply fields:
          both are node-local (`quiet_hist`, `tt_pv`) and already threaded.
          Previous-PV following REJECTED as redundant with `Stage::TtMove`.
          **All six of 4.5.1's deferred fields are now disposed**, so (5) may
          close. No dormant switches were left behind.
    - [x] (5) **[REJECTED, no gain]** Fit, gate and ablation. RAR-S61 was
          unresolved at `[3,10]`; RAR-S64 re-measured after the
          stale-reduction fix and took H0 at `[0,10]` in 8,088 games,
          +0.39 ± 4.89 Elo. The whole +4.50 RAR-S61 saw was the defect.
          `lmr_prior_reduction_adj` removed. Structural work retained at no
          strength claim.
      **SCOPE NOT FULLY EXECUTED — both halves now have owners.** Cluster A
      aimed at three maturity contracts. Per-ply authority closed, though as
      "Rarog needs fewer fields than the plan listed" — three of four rejected
      as node-local, inert or redundant, which the plan's own rule allows as
      an intentionally different answer. The other two were marked done
      prematurely and are now numbered rather than left to a catch-all:
      **history semantics → 4.9b** (ageing, decay and seed policy, update
      attribution, check/capture context in indexing, evaluation-difference
      training). Placed before 4.10 because 4.10 re-runs the 4.2 suite as its
      evidence base and that is not meaningful while the history contract is
      unsettled.
      **reductions and re-search as ONE contract → 4.8.1**, where 4.8 already
      owns LMR and depth authority. `lmr_reduction_units` still takes eleven
      loose arguments; the zero-reduction floor and full-depth verification
      were never audited.
- [~] 4.6 **Cluster B — static eval, TT and quiescence.** AUDIT STARTED
      2026-08-20; see the audit note under this item.
      pruning and searched evidence distinct. Audit TT
      admission/replacement, PV/bound propagation, qsearch stand-pat,
      corrected eval, prior-square futility, capture/promotion ordering,
      evasions and checks. Derive opponent-worsening from 4.5. Measure,
      never import, reference blends and thresholds. Preserve draw and
      mate-distance semantics; finish with any justified cluster-only
      fit, final-PGO SPRT, NPS and ablation.

      **4.6 AUDIT COMPLETE — `analysis/phase4_6_audit.md`.** It began by
      invalidating its own headline: the runner normalised every `q_*`
      counter by main-search NODES while Rarog runs 1.60x more qsearch
      per node. Fixed in the tool (v4 reading). `q_tt_cut` 4.25x →
      **2.46x**, `q_stand_pat_cut` 1.62x → **1.05x, parity**, and
      `q_move_cut` / `q_in_check` FLIP from slightly high to **0.66x /
      0.64x**. Third instance of this denominator class, first one
      inside the tooling. **Leads, ranked:** (1) `qnodes` **1.60x**, the
      headline and not an artifact; (2) `q_tt_cut` **2.46x** — the
      oracle guards its qsearch TT cutoff with `!PvNode` and Rarog's
      `quiescence` has NO PV concept at all; (3) `q_move_cut` **0.66x**
      — Rarog generates captures only in qsearch, the oracle also
      generates quiet checks at the first qply — ⚠ but Rarog measured
      **+30.75 for REMOVING its check extension**, so this population
      has the worst track record for casual changes; (4) TT bound
      composition — Rarog hits MORE (67.4% vs 60.3%) and converts LESS
      (16.7% vs 19.5%), with `tt_bound_not_usable` at **2.13x** and a
      smaller Exact share of stores (3.2% vs 4.1%); (5) **opponent-
      worsening is absent** and PLAN 4.6 names it — 4.5.1 already built
      the substrate (`stack[ply-1].static_eval`) and the consumer does
      not exist. **NOT leads:** qsearch stand-pat is at parity;
      raw/corrected/pruning separation is already present from 4.3a;
      delta pruning and evasions are present; TT probing is at exact
      parity. All three top leads are contract distinctions the
      reference draws and Rarog does not — the 4.7c profile, not the
      profile of the five candidates that failed after it. That ranks
      them; it does not make them true. Each needs its own `[0,3]` gate.
    - [x] (1) **[DONE, no games]** TT admission and bound composition. BOTH
          producer-side hypotheses checked and neither is a defect.
          **Admission:** suppressing the bare stand-pat store — 35.87% of all
          stores — makes `tt_bound_not_usable` WORSE, 9.5% → 14.9% per hit,
          with total TT cutoffs −10.3% against a 7.5% smaller tree. Those
          entries earn their slot. **Replacement:** quality is
          `depth − age_delta/4`, so depth-0 qsearch entries are already
          evicted
          first; the reference weights age ~2x harder but keeps the same depth
          dominance. **Conclusion:** the 2.13x not-usable divergence is a
          SYMPTOM, not a policy defect. It follows arithmetically from a
          Lower-heavy store mix (67.8% vs 55.7%), which follows from 67.5% of
          stores being depth-0 qsearch entries, which follows from `qnodes`
          1.60x. The cause is the search shape, and it is lead (1) of the
          audit — not the TT.
    - [ ] (2) **Quiescence PV contract.** Oracle guards its qsearch TT cutoff
          with `!PvNode`; Rarog's `quiescence` has no `is_pv` at all.
          `q_tt_cut` 2.46x, but partly downstream of (1), so it runs after.
    - [ ] (3) **Opponent-worsening.** Named by 4.6, absent from the engine;
          4.5.1 built the substrate. ⚠ the reference's form makes RFP fire
          MORE and Rarog already runs `rfp_cut` 1.41x — direction first.
    - [ ] (4) **Quiet checks in quiescence.** Explains `q_move_cut` 0.66x.
          LAST: adds work to a qsearch already at 1.60x, and touches the
          population where removing the check extension measured **+30.75**.
      **Answer-led sub-steps (5)-(8), from `answer_compare.py` — a different
      generator: what the search RETURNS, not how often it fires. ⚠ agreement
      is a PROXY; the objective is strength, and each still owes a gate.**
    - [ ] (5) **Mate-answer disagreements.** **5 of 50** positions where one
          engine sees a forced mate and the other does not. FIRST because it
          does not depend on the proxy: missing a mate is worse play, not
          merely different. Each case individually diagnosable.
    - [ ] (6) **Cohort agreement — ENDGAME; the zugzwang lead is WITHDRAWN.**
          25% for zugzwang was a TT-contamination artifact (no `ucinewgame`
          between positions). Clean: zugzwang **62.5%**, and **endgame worst
          at 50%** against 66.7-75%. Endgame is where a shared eval should
          make two searches agree MOST. **n=10: widen before building.**
    - [ ] (7) **Premature conviction — SURVIVED the correction.** Rarog
          revises **1.50** vs **2.16** and self-survives more (72% vs 60% at
          depth 7) yet lands elsewhere a third of the time. Strongest
          surviving lead; matches `root_best_changes` 0.29x independently.
    - [ ] (8) **Score volatility.** **422** cp/iteration against **199** —
          more than double — with a MORE stable move. Explain before trusting.

- [x] 4.7 **[ACCEPTED +15.56 ± 10.02, LOS 99.89%]** Cluster C — main
      selectivity; nElo +24.90 ± 16.01 (RAR-S57 gated the a+c bundle at
      +24.50 ± 12.78; RAR-S58 showed 4.7c carried all of it, so 4.7a was
      reverted and the shipped contract is 4.7c alone). Razoring,
      reverse futility, NMP verification, ProbCut, move-count and history
      pruning, quiet/capture futility, in dependency order with prospective
      searched depth used consistently. Categoricals before constants; no
      broad SPSA — and none was run: the curvature probe in (e) ruled it out.
      The prior was re-derived to 5–15 nElo once 4.7b was withdrawn; the
      result beat it by 60%, and is ~3.8x RAR-S54's blind uniform scalar
      (+4.06 ± 3.71), which is what PLAN 4.7 predicted of a structural
      rework. Owns the NMP/IIR provenance switches and
      `SelectivityProspectiveDepth`. Merged to `dev`; the candidate branch
      `p47c-probcut-filter` can be deleted.
    - [x] (a) **[DONE, no games]** Counter comparability and the corrected
          reading. `probcut_nodes` (per node) and `probcut_attempt` (per move)
          now exist on both engines, and the oracle's TT-served returns are
          split out as `probcut_tt_served`. Re-ran the 4.2 suite as
          `analysis/phase4_differential_v3_depth8.txt`. Oracle side `2682f64`
          on `hybrid-diag`; Rarog side `cf4e475`; reading `8142d5a`.
    - [x] (b) 4.7a **[REVERTED]** null-move entry.
          Primary gate becomes `nmp_eval >= beta`, the old margin re-homed
          onto raw `static_eval`. `nmp_attempt` −21.4%, `nmp_cut` −2.4%,
          conversion 19.2% → 23.8%. RAR-S56. Do not gate standalone: PLAN
          rule 3 says substeps are not expected to win alone.
    - [x] (c) 4.7b **[REJECTED, no games]** Move-count volume. The 13.35x
          divergence was a per-move against per-node artifact; corrected,
          Rarog fires LMP at 0.57x the reference's per-node rate. Withdrawn
          before any code was written against it.
    - [x] (d) 4.7c **[ACCEPTED in the bundle]** ProbCut move filter. SEE
          threshold tied to `probcut_beta − static_eval`, cap counts moves
          searched and scales with `cut_node`. Moves −56.6% while keeping
          91% of cutoffs; conversion per move 32.6% → 68.4% against the
          oracle's 71.9%. Cost is not uniform — one endgame got 46% cheaper.
    - [x] (e) **[DONE, no games]** Fit decision: **no SPSA.** PLAN rule 4
          makes it conditional on curvature. A zero-game sweep shows
          `ProbCutMargin`'s conversion surface flat at 61.8–65.5% across a
          2x range, the move cap inert at 0.72–0.74% moves per node, and
          only the gap scale monotone. The condition is not met.
    - [x] (f) **[DONE]** Registered as RAR-S57 at `[3,10]` nElo, cap 16,000,
          on a re-derived 5–15 prior. ⚠ The bounds and prior were fixed in
          writing before the run and used verbatim, but the ledger row was
          filed after the result. Rule 2 wants it filed first.
    - [x] (g) **[ACCEPTED]** Bundle gate closed at 2,838 games, a fifth of
          the cap, zero time forfeits. The rule 7 ablation (RAR-S58) then
          found 4.7c reproduces the whole effect and 4.7a contributes −0.40
          nElo, so 4.7a was reverted. Shipped head is the 47c-only arm that
          passed its own SPRT: fingerprint 6,922,439 / EBF 2.451.
- [ ] 4.8 **Cluster D — extensions and depth authority.** Check, singular,
      double/possible-higher/negative extension, IIR and excluded-move
      semantics against TT provenance, 4.5 context and LMR. Add locally
      justified TT-move reliability, multi-cut correction and shuffling
      guards. Preserve mate/abort and NMP-clamp correctness. Refit only the
      activated surface, then gate the integrated contract.
- [ ] 4.9 **Cluster E — root search and clock handoff.** Aspiration retries,
      completed-root and interrupted-fallback authority, stability and the
- [ ] 4.9b **History semantics — inherited from 4.5.3.** Ageing, decay and
      seed policy; update attribution; check/capture context in history
      indexing; evaluation-difference training as a Rarog candidate. The
      unexecuted half of Cluster A. Runs BEFORE 4.10, whose 4.2 suite re-run
      is not meaningful while this is unsettled. Stockfish's history events
      are candidates, not defaults — RAR-S59 caught one that looked like a
      plain omission and measured as a disguised selectivity increase.
      decision to start another iteration. Measure extra-iteration behavior
      against Rarog's root-confidence model. Settle root evidence before total
      time; tune and gate real-clock changes separately.
- [ ] 4.10 **Search integration, second selectivity pass, fit and freeze.**
      Re-run 4.2 after 4.5/4.6/4.8/4.9 and close every search-map contract.
      **Read `analysis/phase4_10_obligations.md` FIRST.** Every deferral
      to this step is collected there with its evidence: the structural
      work 4.10 does NOT own (history ageing/decay/attribution, and the
      reduction contract — both unexecuted halves of 4.5), the live
      leads (deliberate selectivity randomisation,
      `NullMoveImprovingBonus` as a volume knob, TT-served ProbCut), the
      rejections with their evidence so they are not retried blind, and
      the two live switches owed a disposition. Written because a catch-
      all clause is where structural work goes to be forgotten.
      **Inherited from 4.5.4 (RAR-S60): Stockfish `cutoffCnt`.** A ply-
      slot recency counter — reset via `(ss+2)->cutoffCnt = 0`, so it
      accumulates across sibling visits — consumed as `if
      ((ss-1)->cutoffCnt > 3) r++`. Rejected at 4.5.4 because that is a
      selectivity increase and every reading this project owns says
      Rarog prunes too much. Re-open here only if the second-pass
      evidence changes that, and note the counter-point: it is
      CONDITIONAL on plies that demonstrably cut often, unlike the
      blanket increases already rejected.
      Own any 4.7-adjacent issue intentionally excluded from protected 4.7,
      including changed NMP/ProbCut/futility populations. If structural work
      moved continuous optima, run one targeted search SPSA over activated
      coordinates only; complete theta, PGO and SPRT it. Compare directly with
      2.3.2 at 1T STC and re-run RAR-S53: at `-Nodes 250000`, mean depth
      fall toward ~14 while Elo rises. Freeze the head, then re-review EV
      before 4.11.
- [ ] 4.11 **HCE baseline and reciprocal-oracle freeze.** Make the 4.10 head
      the immutable HCE baseline; record source/binary hashes, benchmark and
      NPS, and a no-adjudication reproduction slice of Stockfish-HCE versus
      Rarog-HCE under the frozen oracle search. Register the HCE budget and
      stop rules before changing evaluation code.
- [ ] 4.12 **Differential evaluator harness and contract map.** Versioned
      legal corpus with scores, phase, terms, activation, covariance and cost.
      Map score/lazy, mobility/x-rays, pawn shelter/storm, king danger,
      passers, winnability/scaling and specialized endgames from `9587eeeb`.
      Classify every contract; generic ideas need separate local evidence.
      Off ⇒ 4.11 fingerprint exactly. Teacher fit cannot accept a candidate.
- [ ] 4.13 **Cluster F — score, winnability and endgame dispatch.** Material/
      PST and phase ownership, tempo, score grain/POV, rule-50 damping, space
      gating, winnability/complexity and a mature queen/rook/pawn/bishop
      endgame registry. Reference formulas are hypotheses, not correctness.
      Structural and Syzygy invariants precede local Texel, PGO/NPS and
      no-adjudication SPRT.
- [ ] 4.14 **Cluster G — pawns, passers and pawn-dependent scaling.** Pawn
      cache, weak-unopposed/lever/doubled/backward/opposed semantics, blocked
      support, edge files, path safety, progression and king distances.
      Activation/covariance first, joint pawn/passer Texel after structure.
      Repeat only after validation and SPRT both improve; reject prettier fits
      that lose games.
- [ ] 4.15 **Cluster H — activity, threats and space.** Pin-aware mobility,
      bishop/rook x-rays, reachable/bad outposts, bishop-pawn severity,
      trapped-rook geometry, files, king-ring/queen pressure, weak/restricted/
      hanging/king threats and material-gated space. Pooled-PGO NPS on attack
      changes; refit mobility tables and traced terms before SPRT.
- [ ] 4.16 **Cluster I — king safety and nonlinear imbalance.** Rank/file and
      blocked/unblocked shelter/storm, castling destinations, attack units,
      single/multiple safe and unsafe checks, weak squares, pinned defenders,
      flank camp, pawnless flanks and mobility/score feedback. Texel fits
      direct and one-hot table weights; targeted SPSA fits bucket selectors
      only after structure passes. Bake theta, PGO and SPRT it.
- [ ] 4.17 **HCE convergence, search compatibility and cost.** Measure lazy/
      cache behavior, parent-child stability, pruning bounds, NPS and endgame
      pathologies.
      Run anchored whole-HCE Texel cycles on activated weights with fixed
      train/validation/untouched splits; PGO/SPRT each final fit. Repeat only
      while validation and the baked SPRT both improve; stop at the first
      no-gain/failed cycle. Then separately SPSA only moved search margins
      with HCE frozen. Never mix search and HCE coordinates.
- [ ] 4.18 **Cumulative HCE checkpoint and ablation.** Revision-matched
      final-PGO comparison against 4.10, adjudication off. Ablate surprises,
      remove unowned alternatives, close every 4.12 classification and record
      search-versus-HCE attribution. UNKNOWN or first-draft contracts fail the
      maturity bar even with positive Elo.
- [ ] 4.19 **Transfer, portability, SMP and release gate.** Direct comparison
      with 2.3.2; confirm LTC `10+0.1` and 4T direction, benchmark and pooled
      NPS, the platform/ISA matrix, UCI conformance and the correctness suite.
      Require zero UNKNOWN maturity-map items, remove unowned scaffolding and
      resolve obsolete switches. Final no-adjudication target cohort includes
      Basilisk 1.9.3 and the 4.1 oracle. Drop `-use-affinity` for 4T and
      re-calibrate the null pair. **2.4.0** needs ≥ +40 Elo STC with the 95%
      lower bound above +25, plus positive LTC/4T lower bounds; ≥ +100 with a
      lower bound above +75 may justify a higher minor version.

### ━━━ NNUE CUTOFF ━━━ (Phase 5 opens the NNUE line)

### Phase 5 — NNUE runway (bench-identical or NPS-gated per step; no games)

- [ ] 5.0 **Frozen measurement corpus.** Quiet, tactical, endgame, rule-50,
      phase-balanced and search-disagreement cohorts with deep **external**
      teacher cp/WDL labels plus Syzygy WDL/DTZ, by-game train/validation/
      untouched-test separation, exact cohort labels, paired counterfactuals
      and per-candidate residual reports. No engine footprint, so it may be
      pulled forward into Phase-4 SPRT downtime.
- [ ] 5.1 **Per-ply state and dirty pieces.** Extend 4.4's structure to the
      full reversible state, then add the dirty-piece delta contract for
      quiets, captures, EP, promotions, castling and null. Adopt the Reckless
      `BoardObserver` shape: three events emitted at the exact mutation
      points, a generic `make_move<T>` so the null observer costs nothing, a
      compact pre-make stack channel for accumulators and a during-make
      observer channel for threat features. Randomized make/unmake compares
      against a full refresh every ply.
- [ ] 5.2 **Accumulator scaffolding.** Per-thread and per-ply ownership,
      refresh markers and debug full-recompute seams. The accumulator lives
      with the search worker, not inside the copyable `Board`. HCE keeps
      running unchanged and the search stays fingerprint-identical. No
      inference yet; reserve the king-bucket refresh-cache slot for 6.5.
- [ ] 5.3 **Trainer preflight.** Pin trainer, Bullet, toolchain and GPU;
      verify conversion, shuffle, deterministic splits and manifests,
      reference vectors and resume semantics. Malformed or lossy input fails
      loudly.
- [ ] 5.4 **Runway gate.** Exact benchmark, fmt, tests in debug and release,
      randomized unwind, reproducible pilot corpus and trainer conformance.
      Create an NNUE integration branch only after this passes.
- [ ] 5.5 **Threat-map hooks (optional).** Reserve the dirty-threat interface
      so threat inputs can land in 7.2 without another make/unmake rewrite.

### Phase 6 — Baseline NNUE via net_trainer (→ 2.4.0 or 2.5.0)

- [ ] 6.0 **Trainer hardening.** Strict CLI, train/validation/untouched-test
      splits, checkpoint selection, hashes, seeds and exact references.
- [ ] 6.1 **Controlled data.** 30–60M unique teacher positions at 10–20
      sampled positions per game, label blend λ selected on validation, seeded
      from a diverse EPD book. Add by-game/trajectory splits, dedup, the
      frozen 5.0 test set and a dataset manifest (source engine and net SHA,
      search budget, book, λ, seed, trainer commit). Do not train mainly on
      positions adjudicated early by the same evaluator.
- [ ] 6.2 **Baseline networks.** Documented widths and buckets, at least two
      seeds; validation chooses within a run, untouched cohorts are used once.
- [ ] 6.3 **Scalar integration.** Implement `nnue_format.md` in Rarog from the
      reference Rust example: chess768 → (H×2, perspective, SCReLU) → 8
      material output buckets, QA=255, QB=64, SCALE=400. **Acceptance gate is
      the integer-exact conformance vectors**, which replaces any custom
      header scheme; embed the net hash for provenance. Require layout
      validation, malformed/truncated rejection and a clean HCE fallback.
- [ ] 6.4 **Incremental and SIMD.** Dirty deltas per ply and thread,
      randomized incremental-versus-full parity across castling/EP/promotion/
      null, integer bound proof, and portable/x86/ARM64 kernels bit-exact and
      target-native PGO-smoked. Hard pooled-PGO NPS gate before any games.
- [ ] 6.5 **Architecture loop.** One axis at a time with two seeds. Step to
      king-conditioned inputs (trainer v2, mirrored king buckets) as the
      minimum serious architecture, consuming 5.2's reserved refresh-cache
      slot. Capacity follows data; data-scale comparisons hold architecture
      fixed and vice versa. Do not declare NNUE complete without testing king
      conditioning.
- [ ] 6.6 **Gross search-scale safety.** Adjust only clearly invalid margins
      or clock scale. The broad fit waits for 7.3.
- [ ] 6.7 **Baseline release.** Beats the accepted pre-NNUE master at STC and
      LTC, transfers at 4T, passes external checks, zero
      incremental-versus-reference mismatch. Archive the 6.1 manifest and
      trainer commit with each accepted `quantised.bin`. **2.4.0 only if Phase
      4 did not use it**; otherwise 2.5.0.

### Phase 7 — NNUE frontier and final search fit

- [ ] 7.0 **Residual and disagreement analysis.** By phase, material, king,
      tactical and endgame cohort, plus calibration, refresh cost and
      teacher-search disagreement.
- [ ] 7.1 **Data frontier.** Scale and deduplicate, natural finishes,
      hard-position mining, controlled label/depth A/Bs against untouched
      sets, and fresh on-policy data with each clearly stronger net.
- [ ] 7.2 **Architecture ladder.** King/perspective buckets, threat and
      material inputs, width and activation, refresh-friendly variants. Each
      relation-input family is a full architecture revision, not an
      engine-side patch; add one family at a time and pick by measured
      residuals.
- [ ] 7.3 **One post-NNUE search fit.** First resolve the retained categorical
      switches no Phase-4 cluster reached, then register only the continuous
      coordinates whose optimum likely moved. cp margins do not transfer
      across evaluators; structural mechanisms do. Coordinate count and
      horizon come from activation, curvature and budget — not a remembered 24
      or 5,000.
- [ ] 7.4 **Frontier gate.** Direct comparison of 2.3.2, the Phase-4 head and
      the baseline NNUE, plus calibrated matches against contemporary target
      engines. This is where the Basilisk gap is re-measured.

### Phase 8 — Scaling, platforms and product completeness

- [ ] 8.0 **High-thread and NUMA.** Price the depth-diversity deficit at
      4T/8T/16T; test the retained pool-instability and iteration-skipping
      switches, first-touch placement, TT/accumulator sharing and false
      sharing. Keep the score/depth-weighted vote merge. Measure helper TT
      write policy and helper diversity rather than intuiting them.
- [ ] 8.1 **Runtime dispatch and memory.** Consider a baseline universal
      binary selecting specialized kernels, plus TT/network placement and
      large pages. No specialized-binary startup CPU guard — see Recurring
      procedures.
- [ ] 8.2 **Product and platform.** Demand-led Chess960 and FRC coverage or
      other platform work; also holds the parked large-page/NUMA TT, shared-TT
      atomic packing, AVX-512/VNNI kernels, match-manifest schema and
      distributed testing.
- [ ] 8.3 **Scaling release.** Full topology, clock, net, ISA and user-doc
      gate.

### Phase 9 — Contingent classical fallback (only if NNUE is abandoned)

- [ ] 9.0 **King-safety semantic rework.** Closed without retry if 4.16 landed
      it. Otherwise: activation instrumentation by queen presence and phase,
      legal versus geometric safe checks, storm conditioning, reachable
      shelter, and joint danger-input fits.
- [ ] 9.1 **Winnability and material-specific scaling.** Replace the sign-only
      initiative term; residual tables by exact material signature, Syzygy
      WDL/DTZ as direct evidence, sign-preserving non-amplifying scalers only.
- [ ] 9.2 **Passer and pawn conditionality.** Blocker ownership and type,
      rear-line openness, connected-passer semantics, candidate-passer
      exchange conditioning, and a short-horizon race diagnostic.
- [ ] 9.3 **Threat conditionality.** SEE-safe pawn pushes, restricted mobility
      per affected piece rather than board-global, cheap pin/overload
      relations. NPS-check first; do not hand-write a threat net one scalar at
      a time.
- [ ] 9.4 **Broad positional repairs.** Queen infiltration on the full enemy
      attack map, bad-bishop conditioning, space usability (all three weights
      fit to zero, so the representation is the problem) and conditioned
      rook-on-seventh.
- [ ] 9.5 **Material and phase specialization.** Bucketed coefficients,
      king-bucketed PSTs, queen-presence gates. Worst time-to-Elo on the list;
      only if NNUE is abandoned outright.
- [ ] 9.6 **Lazy-margin conditioning.** Only if dual-eval data shows a
      material sign-flip cohort; margin by non-pawn material and king danger.
- [ ] 9.7 **OCB material-scope refinement.** A small material hierarchy for
      the opposite-coloured-bishop scaler with non-amplification, sign,
      pure-OCB, plus-minor and plus-major tests. Cheap and high-confidence, so
      it is the natural first item here.

