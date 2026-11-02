# Basilisk cross-check, 2026-11-02: what to check in Rarog after the running read

Placed in PLAN on 2026-11-02 at the maintainer's direction (§G gives the
placement of each item). Nothing was built, run or measured: a read was
playing on this host. Every Rarog claim below comes from reading the source at `121d947`
(branch `d2`), with file:line.

**Basilisk sources read:** `master` up to `43b4c35` (the squash commits
2026-10-24 to 10-30); branch `b3-pruning-extensions` (`2d006ac`, 100 commits
ahead of master, 24 of them touching `src/`); branch
`fix-conversion-floor-legality` (`b01981a`); `analysis/code_audit_2026-10-27.md`
(Basilisk's own audit: F1–F5, S1–S12, E1–E9, D1–D2, T1–T5); the current PLAN's
known-defects register and workflow register; the test function list of every
`tests/*.cpp`. Not read line by line: the archived PLAN
(`docs/archive/PLAN-closed-2026-10-30.md`, 2,476 lines) and the Rarog-audit
packets `analysis/rarog_audit_2026-10-24*.md`. Those packets describe what
Basilisk took **from** Rarog, so Rarog already has those items by construction.

**Already covered on Rarog's side:** Rarog's own
`analysis/engine_review_2026-11-01.md` (D.3–D.7) holds several of the same
findings. They are listed in §B with their Rarog IDs so that nobody files
them a second time.

## A. Correctness candidates Rarog has and PLAN does not own

### A1. The correction history is added to the evaluator's draw verdicts (Basilisk B.14, BAS-S44 accepted)

- **Basilisk:** `1b2515b` and `e883a13` (2026-11-01). The evaluator flags its
  draw verdicts (KNNK, a drawn KPK, the wrong-corner KBP-K, a pawnless side
  that cannot mate), and the search adds no correction to them. The bug
  showed up as soon as the correction was trained on decisive scores and
  inside exclusion searches (B.3.1 ticket 7). The shared structure keys then
  moved the wrong-bishop draws off 0. The guard was accepted by SPRT.
- **Rarog:** `scale_endgame` / `specialized_endgame_scale` return exactly 0
  for the drawn KPK, the wrong-corner KBP, KK/KNK/KBK/minor-against-minor and
  KNNK (`src/eval/endgame/mod.rs:52-58`, `148-209`).
  `corrected_eval_parts` adds the correction to every raw value with no
  exception (`src/search/correction.rs:173-188`). The end-of-node training
  (`src/search/node.rs:2087-2096`) has no `excluded` guard and no
  decisive-score guard. That is the configuration in which Basilisk saw the
  defect.
- **Status:** the mechanism is present by reading, and its size is unknown.
  This is the most important item on the list.
- **Check after the read:** a debug or diag probe that prints raw and
  corrected eval at those leaves after `bench 13` and after a search from
  Basilisk's wrong-bishop fixtures (tests `wrong_bishop_draw_in_search`,
  `recognised_draw_takes_no_correction`,
  `excluded_training_spares_recognised_draws`). If corrected ≠ 0, write the
  repair card: the gate case depends on whether bench moves.

### A2. Quiescence prunes captures by their gain even when the opponent holds a decisive material edge (`d8d30e9`)

- **Basilisk:** delta pruning, futility and the SEE threshold all read a
  capture's gain as the value of the piece taken. That is wrong when the
  capture ends the opponent's mating material: Kxh8 in KBB-K is a dead draw,
  not "+bishop". Basilisk now skips gain pruning when stand-pat is below
  −KNOWN_WIN (the oracle's `futilityBase > -KNOWN_WIN`).
- **Rarog:** delta pruning only runs above 8 men (`node.rs:2276`), so it does
  not apply here. Futility (`node.rs:2355-2362`) and the SEE threshold
  (`node.rs:2392-2396`) have no such guard. Rarog has no `KNOWN_WIN` constant,
  so the guard would need its own definition (for example "the side to move
  has no mating material left after the capture" or a band on stand-pat).
- **Status:** the mechanism is present. In Rarog it bites less, because
  KBB-K evaluates around +3,000, not 10,000. It still prunes a draw-saving
  capture whenever alpha sits far enough above stand-pat + piece value.
- **Check after the read:** run Basilisk's `test_endgames` fixture for this
  commit, and count `q_futility_skip` and `q_see_threshold_skip` on captures
  that leave the opponent without mating material.

### A3. A won KPK falls through to the generic evaluation, and nothing tests its sign

- **Basilisk:** F1 of the audit. Over all 662,704 legal KPK positions, 2,022
  Syzygy wins had a tapered score ≤ 0 for the pawn side (−192 to 0): mostly a
  2nd-rank pawn with the lone king adjacent. Basilisk now returns a definite
  known win. Its test compares every position with `KPvK.rtbw`.
- **Rarog:** a won KPK returns `None` and the normal evaluation scores it
  (`endgame/mod.rs:150-157`). Rarog has no sign-picking pawnless scaler, so
  it cannot serve Basilisk's 0. The same pre-repair situation can still
  happen, though: a wrong-signed leaf (Basilisk served −28 before A.11.7).
  `kpk.rs` has four unit tests and no Syzygy comparison.
- **Check after the read:** an exhaustive scratch program over all KPK
  positions: Rarog's static eval sign against Syzygy WDL
  (`D:/chess/tablebases/syzygy3456`). It is cheap, a minute of CPU. If any
  win is ≤ 0, Basilisk's evidence (cost in games ≈ 0, a leaf-only error) is
  the prior for a case-1/2 repair.

### A4. A `SyzygyPath` that loads no files is cached (Basilisk audit F4(d))

- **Rarog:** `syzygy.rs:157` returns early on the same path. `:191` caches
  the path whenever `tb_init` succeeds, even when `largest` is 0. So
  re-sending the same path after the files are fixed does nothing until the
  text changes. This is not in the engine review's PROTO list.
- **Check:** a `uci_process` case with an empty directory, then the same
  path with a 3-man file. The repair is I1 and leaves the fingerprint exact.

### A5. CI jobs have no timeouts (`2c6c4fc`)

- Rarog has 0 `timeout-minutes` in `ci.yml` and `release.yml`. A hung test
  (for example a wedged UCI child) burns six hours of runner time. This is
  tooling hygiene.

## B. Already done or owned in Rarog (no new PLAN item; Basilisk adds evidence)

| Basilisk item | Rarog | Basilisk evidence worth carrying over |
|---|---|---|
| Exclusion search with only one legal move fails low by one (`2c6c4fc`) | Done, `node.rs:2042-2049` | — |
| TT stores the eval without rule-50 damping (`7f19c7e`) | Done, `correction.rs:167-172` | — |
| Fail-soft quiescence (`057797d`) | Done (delta returns stand-pat, `node.rs:2286`) | — |
| Upper-case move text (`6e66c5d`) | Done, `board/moves.rs:192` | — |
| A failed `Hash` keeps the old table | Done, `tt.rs:339-346` | — |
| TB root line prints no bound (`5524bf7`) | Done, `488d34a` | — |
| Conversion samples must be legal (`b01981a`) | Not affected: `endgame_conversion.py:41` uses python-chess `is_valid()` | — |
| Child `cut_node` typing of NMP / ProbCut / singular (S1) | NODE-1, D.5 | Basilisk typed all three as the donor does inside Cluster 2 (B.3.1); Cluster 2 as a whole read +67.5 unfitted, with no separate read for S1 |
| TB probe before the TT cutoff (S11) | NODE-11, D.7 | LTC 10+0.1 with tables: +0.3 ± 3.2 (8,000 games), so a neutral repair |
| Zero / negative own clock gives an unbounded search | PROTO-1, D.3 | Test `zero_own_clock_is_bounded` |
| `go mate N` caps the depth at 2N−1 | PROTO-18, D.3 | — |
| A fail-low node stores its least-bad move | NODE-2, D.5 | Basilisk's "fail-low store" (B.3.1 ticket 9) landed **off by default**: it threw a fixture through IIR (research amendment 6). That is a prior for NODE-2's screen |
| Scale factor applied to the whole tapered score (E3) | EVAL-5, D.5 | — |
| Pawnless default factor / KQ against two minors (D1) | EVAL-6, D.5. Rarog's pawnless rule covers minors only, so KQ-v-minors is never scaled | Syzygy class rates: KQ-KBN 82.5% won, KQ-KBB 71.0%, KQ-KNN 73.5%; test `pawnless_scale_truth_table` |
| PV array read past `MAX_PLY` (F3) | Not applicable: Rust checks bounds (NODE-8 owns the cap) | — |
| Histories indexed without colour (S2) | Not applicable: tables are by coloured piece (`history.rs:72-80`) | — |
| Root never reduced (S4) | **Corrected in the second sweep:** Rarog does not reduce at the root or in check either (`node.rs:1759`, pinned by `lmr_never_reduces_at_the_root_or_in_check`; the `lmr_at_root` counter sits inside the gated branch). Deliberate, but the two cases were read only bundled (RAR-S74(e), −3.8 ± 9.6) | D.5 item 19 (H23) |
| Bare-king tiers / KR-K rule-50 drift / KQ-v-minors 1/16 | Not applicable as such: no tiers; damping is proportional (`correction.rs:181-184`) | Run `bare_king_material_is_monotone` anyway (§D) |
| Root equal-score tie rule | Rarog has none | — |
| Nodes counted twice at frontier/razoring | Owned: D.5 item 8 (DRIVER-11), a depth-0 entry counted at both the `negamax` and the `quiescence` entry | Basilisk's repair is in `6e66c5d` |

## C. Search features from Basilisk's Cluster 2, absent in Rarog and its PLAN

These are strength items, not defects, and each needs a Rarog research card
before anything else. Basilisk's donor is Stockfish 19 and Rarog's is
Reckless, so a Basilisk result is a prior, not evidence.

1. **Upcoming repetition (cuckoo).** Rarog's
   `analysis/search_programme_2026-09-13.md:242` lists it as a gap, and no
   leaf owns it. Basilisk's amendment 5: the test is a **cutoff at the draw
   window, not a draw score**. Tests: `cuckoo_tables_complete`,
   `cuckoo_no_false_positive`, `cuckoo_terminal`.
2. **seekMate** (Stockfish 19): with root score ≥ 2000 internal from depth
   16, RFP is capped at depth 6 and singular extensions are off. Basilisk
   seeded it at 12. It fits Rarog's won-ending conversion record (PLAN's 88
   draws and 19 losses).
3. **followPV**: quiet pruning off the previous PV, and IIR gated on it
   (S8). Test `follow_pv_predicate`.
4. **TT bound-mismatch penalty** (S10, `penalize(1)` at depth > 5).
5. **Check extension:** Basilisk measured that its check extension **costs
   22 Elo** at 3+0.03 (`bd0a71b`). Rarog appears to have none (no hit in
   `node.rs`), so nothing to do, but it is a datum against adding one.
6. Graded singular extensions and the per-line extension budget: Rarog
   already has the budget (`node.rs:292`).

## D. Basilisk tests worth running on Rarog (maintainer asked, 2026-11-02)

The "Rarog equivalent" column comes from a keyword search of `tests/` and
`src/`. "None found" means likely missing, not proven missing.

| Basilisk test (file) | What it catches | Rarog equivalent |
|---|---|---|
| `kpk_matches_syzygy` (test_endgames) | Every KPK eval sign against WDL (A3) | None: 4 unit tests only |
| `recognised_draw_takes_no_correction`, `excluded_training_spares_recognised_draws`, `wrong_bishop_draw_in_search` | A1 | None found |
| The `d8d30e9` fixture in test_endgames | A2 | None found |
| `pawnless_scale_truth_table` | Pawnless classes against Syzygy | Partial (`won_endings_are_not_scaled_toward_draw`) |
| `bare_king_material_is_monotone` (test_eval) | More material never scores lower against a bare king; promotion order Q > R > N/B from KBP-K | None found |
| `rook_promotions` (test_search) | Rook promotion only where the queen stalemates; a tie at 0 must not pick the rook | None found |
| `shortest_mate_not_first_mate` (test_search) | KQK `4K3/2Q5/6k1/8/8/8/8/8 w`: does not stop on a longer visible mate | None found |
| `near_mate_recognition` (test_endgames) | Mate found, and a KQK stalemate trap avoided, at moderate depth | Partial (`endgames.rs`, `search_strength.rs`) |
| `kbbk_drive_gradient`, `kbnk_corner_preference`, `kbnk_keeps_both_minors` | Mate-drive ordering; a low-budget canary for giving a minor away | Partial (mop-up tests in `mop_up.rs`) |
| `drawn_ending_bias` (`endgame_draws.epd`) | Overclaimed drawn endings, budgeted | Compare with RAR-E36's read |
| `tournament_infraction_positions` (test_search) | Legal bestmove in positions from real game incidents (`8/6K1/8/8/8/p7/P7/1k6 b - - 4 71`, `8/8/8/K3R3/3Q4/8/6p1/2k4q w - - 26 91`) | None found; cheap to run as-is |
| `corrupt_tt_move_is_not_searched` (test_search) | An illegal TT move injected into the table is never played or searched | Check `tests/tt_shared.rs`; none by name |
| `packaged_fens_strictly_legal` (test_search) | Every shipped FEN (bench, WAC, EPDs) parses under strict validation; Basilisk found four illegal `endgames.epd` positions and one in its suite | Partial (`canary_integrity.rs`); check WAC and bench too |
| `won_ending_does_not_sink_the_clock` (test_engine_threading) | A rising won score past the optimum ends the search instead of running to the maximum | None found (D.1.1's rec1/rec2 area) |
| `stale_stop_does_not_poison_next_ponder`, `stale_ponderhit_does_not_poison_next_ponder`, `immediate_ponderhit_*` (test_engine_ponder) | Ponder lifecycle races | Partial in `engine.rs` tests; check |
| `hash_setup_is_off_the_clock` (test_engine_ponder) | Hash allocation and clearing finish before `readyok`, not on the first move's clock | None found |
| `stopped_fail_high_line_names_the_move_played`, `threaded_last_line_names_bestmove` | The last `info` line's PV starts with the `bestmove` | Check `root_abort.rs` / `multipv.rs` |
| `rule50_mate_found`, `syzygy_rule50_root_scores` | Mate on move 100 beats the rule-50 draw; root TB scores near the clock limit | `draw_semantics.rs` partial |
| `empty_slot_with_key_zero_is_a_miss`, `penalize` (test_tt) | A zero key is not a hit on an empty slot | None found by name |
| `fen_counter_boundaries`, `malformed_go`, `unknown_setoption` (test_uci_protocol) | Parser robustness | `fuzz_lite.rs`, `uci_process.rs` partial |
| `test_stack_unwind` (single and four threads) | Clean unwind at depth with threads | `quiescence_stops_before_ply_stack_overflow` partial |

## E. Build, speed and tooling

- **Universal binary:** Basilisk ships one universal x86-64 binary
  (`9888164`, `0680393`). G.2 already records it and its three leads
  (`analysis/universal_binary_2026-09.md` §9); nothing to add.
- **Zero-weight evaluation terms:** Basilisk stopped computing them for
  **+2.35% NPS** (BAS-P25; at Rarog's ~2 Elo per 1% NPS that matters). For
  Rarog, take a census of production terms whose fitted weight is 0 or
  folds to a constant (EVAL-11 names `hanging_rook`/`hanging_queen` = 1 and
  trapped bishops at 0.04% of rows). This is a reading task.
- **ISA verifier:** Basilisk found its "portable" set did not forbid
  SSE4.1, AVX1 or BMI1 (`andn`, `tzcnt`). Rarog's `xtask` has an ISA check:
  verify which mnemonics its baseline forbids.
- **macOS toolchain unpinned:** Basilisk flagged the macOS Xcode major as
  unpinned. Check Rarog's workflows.
- **Texel tuner T1–T5:** Rarog's `tools/texel-tuner` was ported from
  Basilisk's. Check each: Adam with no convergence criterion (travel bounded
  by lr × epochs); the bake writing every differing slot with no provenance
  (a stale dump reverted 42 members in a dry run); hidden clamps outside any
  manifest; the linear stage breaking `safety_table`'s monotonicity, which
  leaves `clamp(lo > hi)`; coverage audit crediting a dead term. This is a
  reading task, and it matters before F.2.1's refit.

## E2. Saavedra's study and underpromotions (maintainer question, 2026-11-02)

- **Basilisk:** pruning underpromotions below the root lost Saavedra's
  `c8=R` (`1641e64` replaced it). A transposing rook line had displaced the
  queen in 59 of 228 gate positions, 7 of them errors by Stockfish 18. The
  root rule (a rook or bishop promotion beats the queen only by more than
  200 of exact score, the queen searched first) reached H0 under BAS-S33
  (−2.7 ± 4.1), was reverted (`c1791b3`), and was re-applied after BAS-S34
  read −0.3 ± 1.1 on the 3,159 positions where the arms differ (about
  0.02 Elo in ordinary play). The Saavedra and stalemate tests stayed
  throughout (`tests/test_search.cpp`, `test_rook_promotions`).
- **Rarog:** no record mentions Saavedra or underpromotion choice.
  Promotions are all generated, and nothing prunes them by piece (the
  picker gives only the queen a bonus, `movepick.rs:629`). The defect is
  unknown here, not shown absent.
- **Placed:** D.5 item 17, an investigation of zero games.

## F. Order once the read is done

1. Reading only (allowed now as well): E's census, the ISA set, the tuner
   T1–T5, the CI pin and timeouts.
2. Cheap builds and probes: A3's exhaustive KPK (about a minute), A1's probe
   (a debug build, `bench 13` plus the fixtures), A2's fixture, A4's UCI
   case, and the §D tests as an ad-hoc scratch harness against the current
   binary, before deciding which become `tests/` cases.
3. Placement is decided; see §G.

## G. Placement in PLAN (2026-11-02)

| Item | PLAN | Due |
|---|---|---|
| A1 correction on draw verdicts | D.5 item 15, first in D.5 | Before E.3.2 (GUIDE hold) |
| A2 quiescence gain pruning | D.5 item 16 | — |
| A3 KPK sign against Syzygy | D.4 item 19 | Before E.3.2 (GUIDE hold) |
| A4 cached empty `SyzygyPath` | D.7, Basilisk cross-check input | — |
| A5 CI timeouts, ISA list, macOS pin | D.4 item 20 | — |
| §C Stockfish 19 mechanisms | D.5 item 18 | — |
| §D Basilisk behaviour tests | D.4 item 21 | — |
| Zero-weight term census | D.6 item 20 (P19) | — |
| Texel tuner T1–T5 | F.2.1, before the fit | Before F.2.1 |
| Saavedra and underpromotions | D.5 item 17 | — |

## H. Second sweep (2026-11-02): the rest of Basilisk

At the maintainer's request the whole of Basilisk's recent work was read, not
only its correctness repairs. Four read-only agents covered it in parallel:
- the archived Phase A (`docs/archive/PLAN-closed-2026-10-30.md`, all of it),
  the delivery and instrument commits (`336df38`, `a4f9944`) and the PROCESS
  diff since `fac5064`;
- the current Phase B with its October analyses and BAS-S21–S44;
- PLAN §1–§3 and Phases C–G, `release_tiers.md`, DESIGN and the CHANGELOG;
- the experiment entries, every test file and the tooling diff.

Each agent checked its candidates against Rarog's source and PLAN. The
coordinator re-checked every medium- or high-value claim against the files:
`krpkr_scale`'s five zeros, `nps_read.py`'s unpinned bench, CI running no
document checks, the PGO use step, `fit_k`'s bounds, the harness's fault
rule, the seven ledger IDs in `src`, `format_score`, the `collect` job, the
WAC floor, the root LMR gate, the stop's placement and the effort-share
reader.

**Coverage.** Most of Basilisk's evaluation, release and process work either
came from Rarog or was already measured and decided here: the one-flank,
lone-queen and pawn-count scalers (RAR-E21/E22), the space term (RAR-E18),
the rear doubled passer (RAR-E44), the imbalance cache, the endgame king
danger, the truth veto, the two-rejection stop, the patch-release
procedure, game-history-aware tablebase ranking, the host-load measure, the
per-slot processes and the CPU advisory.

| ID | Item (Basilisk evidence) | Rarog proof | PLAN |
|---|---|---|---|
| H1 | CI aggregator that fails unless every gathered job succeeded (GitHub counts a skipped job as passing) | `release.yml` `collect` without `if: always()`; no aggregator in `ci.yml` | D.4 item 20 |
| H2 | A documents CI job (guide, citations, tool tests); ledger disposition agreement | `ci.yml` runs none of them | D.4 item 20 |
| H3 | PGO fails on unprofiled code | `xtask/src/main.rs:1080` passes `profile-use` only | D.4 item 20 |
| H4 | Linux glibc floor and pinned runner labels; static asset | `release.yml:117-136` dynamic, `-latest` | D.4 item 20, G.2 |
| H5 | Sanitiser job (Miri for Rust) | none | D.4 item 20 |
| H6 | Dev build names its commit (`a4f9944`) | `uci_protocol.rs:110-114` package version only | D.4 item 20 |
| H7 | Every hard draw verdict against Syzygy (BAS-E54: 18 of 3,000 won KRPKR called dead draws by the reference rules) | `eval/endgame/mod.rs:481-505` five `Some(0)` | D.4 item 19 |
| H8 | Picker contract fuzz (random-walk positions, random histories, foreign TT move) | `movepick.rs:793-835` six FENs, zeroed histories | D.4 item 21 |
| H9 | Liveness test per search parameter | `tools/audit_spsa_coverage.ps1` static only | D.4 item 21 |
| H10 | Floors with ceilings and staleness bounds | `tests/wac.rs:48` FLOOR 122, no upper bound | D.4 item 21 |
| H11 | Fixtures assert the score, not only the move | — | D.4 item 21 |
| H12 | Census of ledger IDs and step numbers in comments | 7 `RAR-` IDs in `src`; `mop_up.rs:129` | D.4 item 16 |
| H13 | Ordinary scores clamped below the tablebase display band | `search/mod.rs:334-345` | D.3 |
| H14 | Helper-thread panic tested in a release-profile process | `crash_report.rs:167` in-process, main thread | D.3 |
| H15 | Terminal root under `go infinite` reports once | `uci_process.rs:680-707` depth 5, presence only | D.3 |
| H16 | Castling rights not matching the rooks | `board.rs:2348-2357` rejects; process exits | D.3 (PROTO-6) |
| H17 | Priming recipe for the draw-verdict probe; BAS-S44 +7.4 ± 6.9 | — | D.5 item 15 |
| H18 | NODE-2 donor form +3.3 ± 9.1; consumers throw fixtures via IIR | `node.rs:665-689` | D.5 item 4 |
| H19 | Null move cut-only +5.0 ± 9.0 at `10+0.1` | `node.rs:929-945` | D.5 item 3 |
| H20 | Bench position 11's tree is its conversion band | same FEN | D.5 item 7 |
| H21 | Node double count: the time manager also reads it; +4.3 ± 5.9 bundled | `time.rs:337` | D.5 item 8 |
| H22 | Stockfish 19 mechanisms' reads (cuckoo 0.0 ± 9.3, PV pruning, seekMate depth, check extension at LTC); shuffling gate, history fills | absent | D.5 item 18 |
| H23 | Root and in-check LMR never read apart | `node.rs:1759` | D.5 item 19 |
| H24 | The fail-high stop's hole and its give-up form | `search/mod.rs:977-978` | D.5 item 20 |
| H25 | Deep-iteration cost screen with a watchdog | none in `tools/diag` | D.5 rules |
| H26 | Root's first move from the shared table | `node.rs:650-662` | D.2 |
| H27 | Stockfish 19's shared continuation, pawn and correction histories | per thread, `history.rs:72-80` | D.2 |
| H28 | Coherent table costs (BAS-C05) | RAR-P53 consistent | D.2 |
| H29 | TB probe after the cutoff: reads −70%, 2,285/2,286 wins converted | `node.rs:544` | D.7 |
| H30 | A release read with tables loaded | E.1 has no tables arm | E.1 |
| H31 | Foreign engines' faults tolerated up to 1% | `harness_common.ps1:960-1000` voids on any | E.2 |
| H32 | Phase shares in simplified endings | `material.rs:15` | E.4 |
| H33 | Corpus sized by a learning curve | F.2 fixes 30–60M | F.2 |
| H34 | `fit_k` silent at its bounds | `tools/texel-tuner/src/main.rs:1046-1060` | F.2.1 |
| H35 | Collinear passer rank coordinates | `pawns.rs:224-241`, `passers.rs:85-117` | F.2.1 |
| H36 | Speed priors against (BAS-P26) | — | F.5 |
| H37 | SSE4.1-POPCNT tier, SDE tier check | — | G.2 |
| H38 | Repair gates sized by exposure | AGENTS *Gating* fixed sizes | D.4 item 22 (decision) |
| H39 | Fitted-constant analysis before a bake | PROCESS SPSA step 9 | D.4 item 22 (decision) |
| H40 | Noise floor for the curvature rule | PLAN rule 8; `analysis/b33_sweep_2026-09-24.md:166` | D.4 item 22 (decision) |
| H41 | Different trees are not a speed read | `nps_read.py` accepts differing fingerprints | D.4 item 22 (decision) |
| H42 | Pinned bench in the NPS read | `nps_read.py:124-127` | D.4 item 22 (decision) |
| H43 | Link layout is not averaged by build pools (BAS-P20/P21) | — | D.4 item 22 (decision) |
| H44 | A correct repair that loses Elo is reworked, not reverted | AGENTS *Gating* silent | D.4 item 22 (decision) |
| H45 | Multi-pool NPS read; ordered night-queue launcher | — | D.4 item 22 (decision) |

**Decisions on H38–H45 (maintainer, 2026-11-02).** Adopted: H38 as a
field in a repair's registration (its exposure), with no new sizing
rule, since Rarog's static-differential case already spares confined
repairs a gate; H39 through `tools/spsa_theta_audit.py`, which replaces
C.10's untracked pilot re-audit and reproduces its stored report on the
C.10 long tune; H40; H41 as a warning, not a refusal, because release
baselines are read against the head on purpose; H43 as a recorded
caution; H44 reconciled with AGENTS' ban on rescuing a candidate through
its neighbours: the gate's revert stands, the defect stays open and the
rework is a new registration. Declined for now with retry triggers: H42
and both halves of H45 (PLAN D.4 item 22).

**Recorded only, no PLAN item.** These reads are null in Basilisk and have
no Rarog counterpart: B3ProbCutTt +0.9, B3SingTtDepth −1.0, B3SingNegative
0.0, CoreHistoryFills −0.5.

**Not verified:**
- the GitHub branch-protection settings;
- the Linux asset's glibc floor;
- today's WAC count;
- whether Rust honours the function-alignment flag;
- whether pinning narrows Rarog's NPS intervals;
- whether bench position 11's seldepth 127 is the conversion band.

All of these are reads for after the running measurement.
