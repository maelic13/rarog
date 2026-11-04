# Engine review 2026-11-01: defects, donor shape, Rust quality and architecture

Head reviewed: `a3dd950` on `d2` (2.7.0-dev; fingerprint 13,987,223 / EBF 2.565,
quoted from GUIDE, not re-run: no engine source changed). Donors read:
Reckless 0.10.0-dev at `aec5cb2e` (2026-10-28) for the search, threads and
clock; Stockfish tag `sf_11` and `9587eeeb` for the evaluation. Requested by
the maintainer as the input to new PLAN leaves after D.2. This document is the
tracked record; the eight reviewers' full reports, the census logs, the
assembly scan and the scripts are in ignored
`analysis/artifacts/engine-review-2026-11-01/` (`reports/`, `census-logs/`,
`architecture-out/`, `scripts/`).

## 1. Verdict

- **No memory-safety defect and no search-state corruption was found.** All 23
  `unsafe` sites are sound (18 sound, 5 sound but replaceable, 0 doubtful); the
  release assembly has no `memcpy`, `memmove` or `memset` in any node-path
  function, no allocator path reachable per node except the never-taken
  `Board::history` growth, and no `Arc` or lock on the node path. The kernel is
  a faithful port of the donor's per-node structure; nearly every difference is
  recorded or is a fitted constant. The board agrees with python-chess over
  about 60,000 positions and 62 million move triples (section 3.1).
- **Four robustness defects in the shell were reproduced in live UCI sessions**
  (section 3.6): a zero, negative or unparsable own clock runs an unbounded
  search; `setoption` during a search wedges the stdin thread for good; stdin
  EOF never ends a `go infinite` or finished ponder search; a non-UTF-8 input
  line exits the process silently with status 0. Each is a one-function fix,
  fingerprint-neutral, and belongs to D.3 before the 2.7.0 release.
- **One finding bears directly on D.2's four-thread loss and on every shipped
  multi-thread asset** (section 3.3): the search kernel is generic over the poll
  closure, so the binary holds three full copies of `negamax`, `quiescence`,
  `check_stop` and `search_root_window`; the PGO training run (`bench` at one
  thread) exercises only the engine's copy. In a replica of the xtask recipe,
  11 of the 17 search instantiations had all-zero profile counters and were
  placed in `.text$unlikely`; every thread of a `Threads > 1` search runs one
  of those copies. One Threads-2 training pass makes them all hot. This is the
  one mechanism found that fits a loss spread over several unrelated commits
  (RAR-P50). Its effect is unmeasured; its test is one build on RAR-P50's
  instrument.
- **Six unrecorded kernel and board deviations from the donor have strength potential**
  (section 3.2): null-move and ProbCut children typed `cut_node = true` where
  the donor passes `false`; a fail-low node stores its least-bad move as the
  TT move; castling is neither quiet nor noisy, so it gets noisy-history
  maluses it can never recover from; the singular exclusion search uses the
  table-refined estimate, which makes its low-depth extension unreachable;
  captures are generated and scored before the TT move is tried; quiescence
  cannot see a non-capturing queen promotion (section 3.1). Each has a
  cheap zero-game screen and a named gate; none may be implemented without it.
- **The evaluation has no correctness defect.** Its unrecorded forms are the
  endgame scale applied to the whole tapered score (the donor scales the
  endgame half only), the absent pawnless scale factor, three piece-term
  definitions, and the king-danger inputs computed on plain attack maps where
  the donor uses x-ray and pin-line maps (an input to E.4, not a re-proposal).
- **Architecture: the shape is right and still inward; the weight moved.**
  Since the B.2.0 review the crate grew 33% to 29,190 lines; in-file tests
  doubled to 23%, and instrumentation (`diag`, `texel`, `tune`) is about 2,300
  non-test lines, 10% of the source and 437 of `negamax`'s 1,662 lines. Two
  dependency cycles remain (`eval`/`tt`, `search`/`search_options`); the
  public surface is 657 items with 77 unreferenced outside their file. Against
  Reckless, after removing instrumentation, the non-evaluation code is 1.9
  times the donor's; the kernel itself is within 1.3 times, and the real gap
  is the root driver, the tablebase root, the time manager, the second table
  backend, the control plane and the platform leaves, most of it recorded.

What follows from this for PLAN: D.2 gains the PGO-coverage read as its first
sub-step and the shared-backend cost read as its second; D.3 (lifecycle)
receives the confirmed shell defects as its concrete work; three new leaves
hold the behaviour-neutral repairs (D.5), the gated strength candidates (D.6)
and the simplification programme (D.8); the tablebase items go to the
tablebase leaf (now D.9). Section 5 lists the assignment.

## 2. Method

Eight reviews ran in parallel on the idle host, each read-only on the
repository and each with its own private Cargo target directory: board and
move generation; the per-node kernel against Reckless mechanism by mechanism;
the search driver, lazy SMP and clock; the transposition table, the evaluation
caches and the page allocator; the evaluation term by term against Stockfish
11 (both commits); the shell, protocol, lifecycle and Syzygy FFI; a Rust
census (clippy pedantic and nursery, the 23 `unsafe` sites, the allocation and
copy census, the release assembly with `--emit=asm`, a PGO-profile replica,
type sizes, size counts); and an architecture pass repeating the B.2.0
recipe. Every finding carries file:line anchors; before a donor difference was
called unexamined the reviewer searched the ledger, the archived PLAN texts,
HISTORY and `analysis/` for its name, and the record is cited where one
exists.

The lead reviewer then verified the top findings against the source and the
donor, replayed the two decisive protocol sessions against the debug binary
(`uci_probe.py go_wtime_negative`: a search that ran past three seconds with
no limit; `setoption_during_infinite`: a process that had to be killed at
7.5 s with `stop` and `quit` unread), and read the PGO profile logs per
instantiation. Nothing was benched and no game was played; no fingerprint
claim below rests on a new measurement. Confidence per finding is the
reviewer's, with "verified" added where the lead reviewer confirmed it.

Reviewer IDs (`NODE-n`, `DRIVER-n`, `TT-n`, `EVAL-n`, `PROTO-n`, `RUST-n`,
`BOARD-n`, the architecture programme `P0`–`P14`) are kept so PLAN and the
artifact reports agree. Gate cases are AGENTS' *Gating*: case 1 reproduces
the fingerprint (no strength gate, an activation or correctness read), case 2
a pre-registered 2,000-game harm read, case 3 `[-5,5]`; a gain candidate
takes `[0,3]`.

## 3. Findings by area

### 3.1 Board and move generation

Instrument: a scratch probe crate against python-chess 1.11.2 over about
60,000 random-walk positions (1.14 M moves, 1,683 with an en-passant square),
62 M (from, to, promotion) triples through `legal_move`, 20,000 positions each
for `threats()` and `see_ge`, 6,000 insufficient-material cases, 400,000
mutated FENs and a Zobrist replay, plus the existing 84 board tests in debug
and 51 under the PEXT backend. No wrong-move, wrong-hash or wrong-state
defect: legal sets, the capture/quiet partition, `gives_check` and its fast
path, make/unmake with `check_consistency` after every step, threats, SEE,
insufficient material and the parser all agree with the oracle, and the five
board `unsafe` sites are sound by construction (the magic shift and `pext`
bound the index below the per-square width the `const` build asserts;
`MoveList::push` bounds-checks before it increments `len`). The board's
profile share is small (RAR-M36: make/unmake 6.7%, generation 6.6%, SEE 5.2%),
so no speed finding here is worth more than a few tenths of a percent.

| ID | Finding | Where | Confidence | Action, class, gate | Owner |
|---|---|---|---|---|---|
| BOARD-1 | Castling is neither quiet nor noisy (the kernel's NODE-3, found independently from the board side): `is_quiet_move` is `flags <= DOUBLE_PUSH`, castling has flags 2 and 3, `is_noisy` is capture or promotion; the picker files castling under the quiets and scores it with `quiet_score`, the move loop treats it as noisy, and its quiet history can only rise. | `board/board.rs:557–560`, `moves.rs:104–107`, `movepick.rs:168` | verified | define quiet once; the agreement test; gate case 2 | D.6 |
| BOARD-2 | The capture stage never generates a non-capturing queen promotion (pawn targets are enemy occupancy or the checkers), and quiescence outside check generates captures only, so a safe promotion to a queen is invisible to the stand-pat decision (`8/P6k/8/8/8/8/8/K7 w - - 0 1` yields no captures). Scratch counters over the bench positions at depth 10: 1.53% of capture generations had a legal quiet queen promotion, 0.59% with an empty capture list. The donor generates queen promotions with the noisy moves; B.4 exempted promotion captures from the count rule and never weighed this. | `movegen.rs:89–105`, `427–470`, `656–700`; `node.rs:2290–2295` | verified | R2 card (horizon, the quiescence count rule, delta pruning, the SEE floor, ProbCut's list, the `has_pseudo_capture` pre-scan); counter on an endgame suite, under 0.5% closes it; gate `[0,3]` | D.6 |
| BOARD-3 | `analysis/draw_policy_2026-09-08.md` Policy 2 says a repetition scan crossing a null move can only be a false negative; `is_repetition` walks the whole history, null entries included, with no `plies_from_null` bound, and two null moves by the same side while the opponent shuffles a piece out and back give a false twofold through the public API (reproduced; 46 of 7.6 M nodes at depth 12). The donor bounds every scan by `plies_from_null`. | `board.rs:2291–2306`, `1888–1923` | verified | correct the record, a characterisation test; an engine bound is a registered unit of its own | D.5 |
| BOARD-4 | `Move::from_uci` yields a `Move` that `make_move` accepts; in release a non-canonical capture leaves the captured bit set and `e1g1` castles without the rook. Every caller canonicalises today. | `moves.rs:186–210`, `board.rs:1718–1760` | verified (release and debug) | a `UciMove` newtype only `legal_move` and `parse_move` turn into a `Move`; case 1 | D.5 |
| BOARD-5 | `--cfg rarog_pext` without `+bmi2` compiles clean (a `pext` call that is `SIGILL` on a non-BMI2 CPU), and CI never runs the slider-table test under the PEXT backend (it passes when run). | `attacks.rs:461–468`; `ci.yml` | verified | `compile_error!` and a CI cell (with RUST-7) | D.5 |
| BOARD-6 | Two copies of `ep_capture_is_legal`, four near-identical attacked-square tests, two castling-condition blocks, and `legal_ep_capture_exists` returning an always-`Ok` `Result<bool, String>` that compiles to an out-of-line call from `make_move_inner` on every double push. | `board.rs:2228`, `movegen.rs:532`, `board.rs:1380`, `1620`, `1110`, `movegen.rs:1049`, `board.rs:2213` | verified (assembly) | merge; `bool`; fingerprint and the differential | D.8 (P3) |
| BOARD-7 | Test gaps: no standard perft beyond depth 3 (4 in release), no direct `gives_check_with` differential (the scratch run: 0 mismatches over 500,523 moves), no quiet/noisy agreement test, no null-crossing repetition test, no PEXT cell. | `tests/board_correctness.rs:386–475` | verified | add them (deeper perft release-only) | D.5 |
| BOARD-8 | The five board `unsafe` sites are sound by construction; the only unsafe-free zero-branch form of the slider lookup pads each table to a power of two (+236 KiB) and masks, unmeasured; `assume_init_ref` is a narrower spelling for `MoveList::as_slice` at the same count. | `attacks.rs:114–150`, `467`; `moves.rs:71` | verified | none | — |
| BOARD-9 | The non-pawn correction key excludes kings (the donor's includes them), and the table key carries no fifty-move bucket (the donor mixes one in for the TT and keeps the full key for repetition; Rarog applies the clock on read). | `board.rs:109–116`, `512` | likely; low value | R2 only, after D.6's first items | D.6 (note) |
| BOARD-10 | Hygiene: retired step numbers in `board.rs` and `zobrist.rs`; an `#[inline(always)]` splitting `make_move`'s doc block; `piece_type_at_unchecked` is checked; sentinels (`ep_sq` 255, `NO_PIECE`, `Move::NULL`) and positional bools against the clean-code policy, the hot ones with a speed argument; `game_result` reports a dead stalemate as `Draw`; FEN halfmove above 255 rejected where the donor saturates. | `board.rs:100`, `1691–1694`, `2089` | verified | with the next change on those lines | D.8 (P16) |

### 3.2 The per-node kernel (`src/search/node.rs`, `movepick.rs`, `history.rs`, `correction.rs`)

The mechanism table (TT cutoff, IIR, static eval and correction, improving,
razoring, reverse futility, null move, ProbCut, singular, LMR, PVS,
mate-distance pruning, draws, the ply cap, the TT store, quiescence, history
updates, the picker stages and quiet scoring) is in the artifact report; every
row is "same shape" or "deliberate and recorded" except the following.

| ID | Finding | Where | Confidence | Action, class, gate | Owner |
|---|---|---|---|---|---|
| NODE-1 | The null-move child and both ProbCut verification searches are typed `cut_node = true`; the donor passes `false` (`search.rs:606`, `664–684`, its fifth argument). The type feeds the child's TT-cutoff admission, the cut-node LMR terms, LDSE, the negative singular extension, the history bonus offsets and the fail-low parent reward, and flips the label of every descendant. Rarog's own B.3 data put ProbCut's verification success at 98%, which makes that child an all node; the null child depends on the NMP conversion rate (22% at the B.3 head). No record mentions it; the SPSA fits were made with this typing. | `node.rs:980–991`, `1195–1224` | verified that the argument differs; defect likely for ProbCut, open for the null move | R2 zero-game screen first: `diag` counters of the fail-high rate by node type for null children, ProbCut children and LMR children on `bench 13` at stride 1 (the donor's prior predicts ≥ 70% fail-high for nodes typed `true`); then I1 with a unit test that pins the `cut_node` each recursive call receives; gate case 3 `[-5,5]` as a repair, `[0,3]` only if the screen predicts a gain | D.6 |
| NODE-2 | `best_move` is set on every new best score, so an `Upper` node stores its least-bad move and overwrites the position's earlier move; the donor sets it only when alpha rises and keeps the old move. Consumers: IIR, the razoring guard (a stored quiet fail-low move blocks razoring at exactly its target nodes), the no-TT-move LMR term, the first-move slot, the fail-low parent-reward factor. Noted once (`analysis/b21_review_2026-09-14.md` §4) and never isolated. | `node.rs:1939–1942`, `2117–2126`; donor `search.rs:1050–1056` | verified; effect unknown | R2 screen: store `NULL` for `Upper` in a `diag` build, read `iir_applied`, razor counts, LMR no-TT-move share, `bench 13` nodes, WAC at 100k, canaries; then I1 + V, case 3 (a shared TT-move population) or `[0,3]` | D.6 |
| NODE-3 | Castling has flags 2 and 3; `is_quiet_move` is `flags <= DOUBLE_PUSH` and `is_noisy` is capture or promotion, so castling is neither: it skips quiet pruning, takes the noisy LMR base and SEE threshold, and when it fails it receives `update_noisy` maluses while a successful castle is credited in the quiet tables. The noisy slot for the castling king drifts to its floor and adds about a ply of reduction to castling. The predicate dates from the initial commit; the donor classes castling quiet. | `board/board.rs:558–560`, `board/moves.rs:104–107`, `movepick.rs:168`, `node.rs:1507` | verified by trace; size unmeasured, probably small | V: a unit test `is_quiet_move(mv) == !is_noisy(mv)` over legal moves with castling, under-promotion and en passant (fails today); read the four castling noisy slots after `bench 13`; I1 one-line fix; gate case 2 (the fingerprint moves) | D.6 |
| NODE-4 | `eval_for_pruning` is the table-refined estimate without an `excluded` guard; the donor keeps the plain eval in an exclusion search. In an exclusion node the TT bound is Lower or Exact and the window sits below `tt_score`, so the refined estimate always exceeds `alpha + margin`: LDSE can never fire there and the LMR `alpha_gap` term is pinned at its low clamp. | `node.rs:753–757`, consumers `1396`, `1783`; donor `search.rs:471–482` | verified; effect unmeasured | R2 counter "LDSE would fire inside an exclusion search"; I1 one-line guard; gate case 2 if under 2% of exclusion nodes, else `[0,3]` | D.6 |
| NODE-5 | Captures are generated and scored (`fill_noisy`) before the TT move is tried; the donor generates lazily after the hash move fails to cut. The TT move cuts at about 35% of loop nodes (C.12 review), which called this the one lead worth several percent of NPS. The ordering of captures after the TT subtree changes (scores read a history the subtree moves), so it is a playing change. | `node.rs:1478`, `movepick.rs:265–279`; donor `movepick.rs:49–60` | shape verified; size is the C.12 estimate | I2 with a frozen prediction (speed +x%, games neutral); V: pooled-PGO NPS, then case 2 | D.6 |
| NODE-6 | `quiescence<NODE>` never reads `NODE`; `Pv` and `NonPv` are two identical 355-line copies of the hottest recursion (and three more each through the poll generic, section 3.3). | `node.rs:2144–2498` (`NODE` only in the signature and the recursive call) | verified | I1: drop the parameter; fingerprint exact; pooled NPS read; gate case 1 | D.5 |
| NODE-7 | PV-node stores carry no `force` and there is no mid-loop `Lower` store when a PV node raises alpha; the donor has both. Matters most when a node is cut by the clock and with helpers. | `node.rs:2013–2022`, `2117–2126`; donor `search.rs:1069–1071`, `1191` | reading; small | R2 note for D.2 (SMP interplay), then a counter of refused PV stores; `force` is case 2, the mid-loop store an experiment of its own | D.2, D.6 |
| NODE-8 | `MAX_PLY = 128` is below the extension budget's bound (about twice the iteration depth plus 16); from about iteration 56 the cap, not the budget, ends lines, and the node returns a static eval with no in-check or draw handling. The "revisit if seldepth reaches it" trigger fired (seldepth 127 at depth 19–20 on bench position 11; D.1.1's rec2) with no recorded decision. | `mod.rs:56`, `node.rs:498–500`, `296–306`; donor 240, draw in check | likely; frequency unmeasured | R2 counter of nodes entering the cap per game at `3+0.03`, `10+0.1` and `60+0.6`; I2 raise the cap with the band constants if non-zero; gate case 1 if zero at STC, else an LTC harm read | D.6 |
| NODE-9 | `negamax` is 1,662 lines (437 of them `diag` and trace plumbing) with a ten-argument positional signature in which two bools flank a `Move`; NODE-1 is an instance of the hazard. | `node.rs:481–2142` | verified | I1 after P5/P6: a two-field `Copy` node-kind struct; cold blocks outlined `#[inline(never)]`; NPS-checked | D.8 |
| NODE-10 | The fail-low parent reward and the history updates run for singular-exclusion searches too (shared with the donor). | `node.rs:2112–2114` | speculative | counter first; only above 5% of rewards a guard, case 3 | D.6 (low) |
| NODE-11 | The tablebase probe runs before the TT probe, so with tables loaded a Fathom probe is issued at nodes the TT would have cut, and `tbhits` is inflated. | `node.rs:544–566` against `567–649` | verified | I1 reorder; fingerprint-neutral without tables; a TB-equipped identity check | D.9 |
| NODE-12 | Quiet pruning and LMR history is captured when the quiet list is generated, not per move; later siblings do not see earlier siblings' updates. | `movepick.rs:538–570`, `node.rs:1513–1519` | likely; tiny | none unless NODE-5 lands | D.6 (note) |
| NODE-13 | One heap `Vec` per root call (the `searchmoves` filter). | `node.rs:1433–1446` | verified | I1, `retain` in place | D.5 |
| NODE-14 | Test gaps that let NODE-1, 3 and 4 through: nothing pins the child `cut_node`; quiet/noisy consistency untested; no test of the estimate at an excluded node; the ply-cap test exercises only the entry guard. | `node.rs` tests | verified | V with each fix | D.5, D.6 |
| NODE-15 | Integer sentinels in the kernel interface (`VALUE_NONE`, `SEE_UNKNOWN`, `Move::NULL`) against the clean-code policy; every consumer is guarded today. | `movepick.rs:14`, `eval::VALUE_NONE` | verified | only when touching the types (`enum SeeClass` first) | D.8 |

Cleared by trace (no defect): stack look-backs at ply 0 and after a null move
(sentinel entries), re-entry of the exclusion and verification searches,
searched-move lists, `reduced_depth` clamps, integer overflow margins,
`i16` saturation, parameter ranges that could make `clamp` panic, the
`diag`/`tune`/`texel` paths (none writes search state), per-node allocation
(none), and the `StackEntry` copies (about 56 bytes).

### 3.3 The search driver, lazy SMP and the clock (`src/search/mod.rs`, `threads.rs`, `shared.rs`, `time.rs`)

**Shared memory on the node path.** Everything but the transposition table
and one counter line is per thread: histories (about 5.5 MB per thread),
correction tables (about 26 MB per thread), the pawn and imbalance caches, the
evaluator, the stack, the PV and the root records. The shared touches are: the
TT probe (12 relaxed loads of one 64-byte cluster), the final store (a scan of
up to six slots, then two relaxed stores, write-always except a much deeper
same-key entry), `store_eval` on a probe miss (a second store to the same
cluster, about 0.4–0.5 per interior node), prefetches (per picked move before
pruning, per move made, and the node's own cluster before its closing
updates), the batched node counter (`fetch_add(128)` every 128 nodes per
thread), the stop flag (a load every 128 nodes; every node under `go nodes`),
the helper poll (every 2,048 nodes), `tb_hits` (a `fetch_add(1)` per tablebase
probe), root-score publication (a CAS per completed root move) and the stop
votes. Every ordering is `Relaxed`. At four threads the counter line costs an
estimated 0.2–0.4% and TT coherence 1–2% of node time: not the 6–7% loss.

| ID | Finding | Where | Confidence | Action, class, gate | Owner |
|---|---|---|---|---|---|
| RUST-1 / DRIVER-3 / DRIVER-4 | **PGO covers one of three kernel copies.** `negamax`, `quiescence`, `check_stop` and `search_root_window` are generic over `P: FnMut() -> SearchEvent + ?Sized`; three closure types reach them (the engine's poll in `Engine::search`, `main_poll` in `search_parallel`, `helper_poll` in `spawn_search_worker`), so there are 3 × 2 copies of `negamax` (about 7,000 and 4,900 assembly lines each) and of `quiescence`, about 15% of the binary. The PGO training sends only `bench {depth}` at Threads 1. In a replica of the recipe, the 6 `main_poll` and 5 `helper_poll` instantiations had all counters zero (`negamax::<NonPv, Engine::search::{closure}>` entry count 5,136,956 against 0 for the worker's) and were placed in `.text$unlikely`, the untrained `negamax<NonPv>` at 6,680 lines against 15,568 for the trained copy; a Threads-2 training pass made all three copies `.text$hot`. Every thread of a `Threads > 1` search, the main one included, runs an untrained copy, and RAR-P49's Q compares exactly that against the trained copy in the 1T and 4×1T cells. | `node.rs:481`, `2144`, `365`; `mod.rs:1730`; `threads.rs:100–123`, `339–350`; `xtask/src/main.rs:1156-1176`; logs `census-logs/pgo_groups_A.txt`, `pgo_sections.txt` | verified from the profile logs and the recipe; the NPS effect is unmeasured | I1 either way, no strength gate: (a) take `&mut dyn FnMut() -> SearchEvent` at `search_impl`, `search_parallel` and `run_worker_job` so one copy exists and the 1T profile covers it (the poll runs once per 2,048 nodes), or (b) add a Threads-2 run to the training. Read on RAR-P50's instrument: Q at 4T and the 1T cell (must stay within 1%); prediction frozen by the reviewers: Q +0.03 to +0.10 in ln, killed under 0.02. Run one cure first, not both. | D.2 |
| DRIVER-2 / TT-6 / TT-4 | **The shared table backend costs more per operation than the local one, and nobody measured it (F16).** A shared probe is up to six (8-byte load, 2-byte load, `fold16`, compare); a store scans up to six slots through `load_any` and `unpack_entry`; the local form does the same over three plain entries. Fitting RAR-P50's medians to a fixed per-node cost gives 47–61 ns, about 12% of the head's node time; a cheaper node makes a fixed cost a larger share, which is what the I2 step of the bisect looks like. The 1T bench fingerprint never sees the shared path, and 1T and 4T play different replacement geometry (3-way against 6-way). F16 was handed to D.2 by the B.2.0 review and is absent from D.2's text. | `tt.rs:196–232`, `810–833`, `843–896`; `mod.rs:407–408` | likely; size is a fit | V: `bench 3` pooled at Threads 2 against Threads 1 (below `MIN_PARALLEL_DEPTH` the search stays serial but `configure` has made the table shared, so only the backend differs); prediction: head ratio 0.87–0.91, refuted at ≥ 0.97. Then, in cost order: the PGO fix, a `replacement` that reads only occupancy, depth and age bits (I1, fingerprint exact), skipping identical stores, and F16's answer (one atomic backend at every thread count: I2, 1T NPS pool ±0.5%, case 3 since the geometry moves the fingerprint) | D.2 |
| DRIVER-1 | Q is an NPS ratio and NPS mixes node cost with node mix: a shared table turns searched nodes into cheap probe-and-return nodes, so Q can fall when the evaluation gets cheaper with nothing wasted. RAR-R07/R08 showed the opposite failure of the proxy (3.95× NPS with lower depth). | RAR-P49 readings; `mod.rs:1789–1806` | likely | R2, zero games: RAR-R10's paired within-process time-to-depth harness, 4T against 1T, B.9 and head; prediction: the ratios differ by under 5% | D.2 |
| TT-5 / DRIVER-5 | The two pick-time prefetch sites (`1aaa39e`, `65f490e`) and the own-cluster prefetch are the only table-side change since B.9 (`tt.rs` is byte-identical to `24aefb4`); they fetch lines for moves that are then pruned, measured at one thread only (+1.5 to +1.7%). RAR-P50 named them first and stopped before isolating them. | `node.rs:1505`, `2348`, `1957`, `2060`; donor prefetches at the make only | verified (diff); effect unmeasured | R2 counters (prefetches per searched move: predicted 1.6–2.2 against B.9's 1.0); then one arm per site on RAR-P50's instrument; Q +0.00 to +0.02 each kills the family | D.2 |
| DRIVER-6 | A same-position store rewrites identical bytes (keeping the line Modified), and a probe miss stores twice (eval-only, then the result into the same slot). | `tt.rs:879–895`, `node.rs:701` | likely | counter of byte-identical stores at 1T (predicted 8–25%); at ≥ 10% compare before storing; case 1 | D.2 |
| DRIVER-7 / RUST-6 | `SharedContext`'s hot atomics (`stop_state`, `nodes`, `tb_hits`, `ponderhit`, `stop_votes`) share one unpadded line; `tb_hits` is a `lock xadd` per tablebase probe, up to one per node per thread in a 5–6-man endgame. The donor shards both counters per thread in 64-byte slots. | `shared.rs:97–121`, `mod.rs:1808–1812` | verified (layout and asm) | I1: thread-local `tb_hits` flushed on the 128-node boundary; `repr(align(64))` so the flag line is read-only; fingerprint exact; a 5-man 4T read as the activation read | D.2 |
| DRIVER-8 | Any 4T rate study with `diag` counters perturbs what it measures: each `diag_count!` is a contended RMW on adjacent process-global statics. | `diag.rs:20–35`, `596–618` | verified | instrument rule: thread-local `u64` fields in `ThreadData` summed at join behind a feature; never read NPS from a `diag` build | D.2 |
| DRIVER-9 / PROTO-1 | A zero, negative or unparsable own clock runs an unbounded search (section 3.6). | `time.rs:145–149`, `search_options.rs:314–325`, `553–565` | verified by replay | I1, case 1 | D.3 |
| DRIVER-10 / TT-1 | Idle helpers keep the previous shared table alive after a `Hash` change: `run_worker_job` moves the job's handle into the helper's `Searcher` and never releases it, so a resize holds old plus new until each helper's next job; with large pages that is locked memory, and the comment "released before its replacement so that two are never held at once" is true only on the page-backing path. | `threads.rs:226`, `321`; `mod.rs:359–381` | verified | I1 with TT-3; fingerprint exact; check: `Threads 4`, `Hash 1024`, a search, `setoption Hash 2048`, private bytes | D.5 |
| TT-3 / DRIVER-20 | `TranspositionTable`, `TtStorage` and `LocalTable` derive `Clone`; a clone of the local form copies the whole table and is safe today only because `make_shared` runs three statements earlier. D.2's 2026-10-06 audit asked for a handle only the shared form can produce. | `tt.rs:119`, `295`, `301`; `threads.rs:290`, `321` | verified | I1: `SharedTt(Arc<SharedTable>)` as the only thing a job carries; the worker drops it after sending its result; fingerprint exact | D.5 |
| DRIVER-11 | A depth-0 entry into `negamax` is counted twice: `check_stop` at the `negamax` entry and again at the `quiescence` entry it hands to. Reported nodes, `go nodes`, the bench count and the LMR spread (`(nodes + 27 × thread) % 128`) read the double count. | `node.rs:495`, `519–525`, `2153` | verified | R2 sum of `diag` interior and quiescence counts against `td.nodes`; then I2 (test `depth <= 0` first); a new fingerprint by construction, so register it alone, case 2 at most | D.6 |
| DRIVER-12 | `go depth N` at `Threads > 1` can report a deeper helper's result: helpers drop the depth limit and the weighted vote runs for every single-line search, so a helper at N+1 with a different move can win and print `info depth N+1`. The donor answers with thread 0 under a depth limit. | `threads.rs:254–263`, `388–392` | verified | I1: skip the vote under `limits.depth`; test at Threads 4; case 1 | D.5 |
| DRIVER-13 | The aspiration fail-low sets `beta` to the midpoint of the new alpha and the old beta (`[c − 27, c − 8]` on the first fail), while its doc says the middle of the old window; three `params.rs` notes describe pre-fit defaults. The shape is fitted, the words are not. | `mod.rs:274–287`, `234–240`; `params.rs:153–183` | verified | I1 comments now; a counter of fail-lows whose re-search fails high decides whether the old-alpha form is a `[0,3]` candidate | D.5, D.6 |
| DRIVER-14 / TT-8 | Each helper builds and discards a 64 MiB table at spawn (`Searcher::default()` then `TranspositionTable::new(1)`), touches about 32 MB of tables, and `isready` does not wait for any of it; `ucinewgame` clears helpers after the main thread. | `threads.rs:97–137`; `engine.rs:83–86` | verified | I1: a helper constructor without the default table; the worker acknowledges init and `NewGame`; case 1 | D.5 |
| DRIVER-15 | The join waits for the slowest helper, including one blocked in a tablebase read (54 ms observed for the root probe). | `threads.rs:363–371` | likely | R2 then I2: vote on the results received by a deadline | D.9 |
| DRIVER-16 / 17 / 22 | Donor divergences to record so they are not rediscovered as fixes: vote weight `+14` with a floor (donor `+10`), decisive results ranked by vote not by win-or-loss rule, quorum `floor(N/2)+1` without withdrawal (equal at 2–4 and 6 threads, different at 5, 7, 8, 16), ponder-move preference in the key; the SMP bundle (root rotation, pooled root scores, window re-centring) accepted whole in RAR-R03 and never decomposed; correction history per thread where the donor shares it. | `threads.rs:141–212`, `shared.rs:259–291` | verified | R2/V under D.2 or G.1 with the RAR-M78 instrument; removal brackets `[-1.75, 0.25]` per member at 4T | D.2, G.1 |
| DRIVER-18 | The forced-move shortcut (stop after depth 2 with one legal move) also fires for a one-move `searchmoves` under fixed depth or time, because the filtered root list feeds the count. | `mod.rs:1046–1052`, `493–512` | verified | I1: exclude non-empty `search_moves`; case 1 | D.3 |
| DRIVER-21 | Test gaps: no negative-clock test, no fixed-depth MT test, no concurrent hammer on `SharedCluster`, no MT abort test, no Hash-release test, no `searchmoves` shortcut test, no helper-readiness test. | `tests/tt_shared.rs`, `root_abort.rs` | verified | V with each fix | D.3, D.5 |

### 3.4 The transposition table, the evaluation caches and the page allocator (`src/tt.rs`, `src/eval/mod.rs`, `src/large_pages.rs`)

Table facts: local form 10-byte entries, 3-way 32-byte clusters, plain 16-bit
tag; shared form 6-way 64-byte struct-of-arrays clusters, one cache line,
relaxed atomics, tag `key16 ^ fold16(data)` (torn pairs rejected with 16-bit
strength); index by low key bits with a mask, tag from the top 16 bits; age
in five bits; replacement by `depth − 4 × generation delta`, same-key reuse,
a current-generation entry at least four plies deeper (six on PV) keeps its
result. Both evaluation caches (pawn table 16,384 × 80 B, imbalance table
16 KiB) are per thread, keyed by the full 64-bit pawn key and an exact count
signature, and are not a candidate for the four-thread loss; B.9 held the
same per-thread footprint. Checked sound: bit packing and its round trips,
`fold16`, the score conversions and the tablebase band below the mate cutoff,
excluded-move nodes storing nothing, the prefetch sites and both `unsafe`
hints, age arithmetic, `hashfull`'s sample, the pawn cache's dependencies and
the shelter cache key, the `PageAllocator` contract (one pure router for
`alloc` and `dealloc`, default `realloc`, zero-size and over-aligned layouts
never reach the page path, large-page requests rounded to the minimum, the
privilege check) and every `win32.rs` signature.

| ID | Finding | Where | Confidence | Action, class, gate | Owner |
|---|---|---|---|---|---|
| TT-7 | `Hash` is rounded down to a power of two with no notice: 1000 → 512 MiB, 2000 → 1024, 3000 → 2048, 32000 → 16384 (51–68% of the request used). The donor sizes exactly and indexes by multiply-shift with a low-bit verification key. `tests/tt_sizing.rs` uses powers of two only. | `tt.rs:780–788`, `752`, `765` | verified | I1 now: an `info string` notice, a README sentence, a test (`Hash 100` → 64); the any-size index (multiply-shift and a low-bit tag) changes the fingerprint at every size: case 2, only if the notice shows anyone sets such values | D.5 (notice), D.6 (index) |
| TT-2 | `resize` and `ensure_local` hold old and new tables together, and a `Hash` change at `Threads > 1` fills a table twice, single-threaded (a local one that `make_shared` then drops). | `tt.rs:339–358`, `370–386`; `mod.rs:373–381`, `407–409` | verified | I1: build the shared form directly when `threads > 1`; the "failed resize keeps the old table" property is pinned by a test and is the maintainer's trade | D.5 |
| TT-9 / DRIVER-19 | Six slots of 16-bit tags give twice the local form's false-hit rate per probe (9.2e-5 against 4.6e-5) and 64 times Reckless's (21-bit keys, three slots): about 700 wrong entries a second at 4T once clusters are full, each validated only in its move. | `tt.rs:558–560`, `89–96`, `245–248` | arithmetic verified; effect unknown | R2: a shadow full-key array in a `smpstat` build counting mismatched hits; a wider tag needs the entry repacked | D.2 |
| TT-10 / NODE-7 | No `force` for PV stores (see NODE-7). | `tt.rs:843–897` | reading | as NODE-7 | D.6 |
| TT-11 | Tablebase-band scores skip the rule-50 downgrade the donor applies to mates and tablebase scores alike; exposure bounded by the clock-90 no-cutoff guard. | `tt.rs:582–600` | speculative | a focused test documenting the intent; case 1 if changed | D.9 |
| TT-12 | Test gaps: the false-hit test cannot see an 8-bit tag (threshold 200 of 20,000 against an expected 0.006); `assert!(tt.hashfull() == 0 \|\| tt.hashfull() > 0)` is a tautology; no deterministic torn-pair test, no alignment assertion, no non-power-of-two size test, no local/shared replacement-agreement test. | `tests/tt_shared.rs`, `tt.rs:1201` | verified | V: an in-module torn-pair test, `false_hits <= 3` with a seeded set, the missing asserts | D.5 |
| TT-13 | `replacement`'s doc claims a bound clause the code lacks; `LocalCluster::holds` ignores occupancy while the shared one requires it; a 20-line paragraph narrates the Apple cache-line history; `make_shared` drops through an empty placeholder table; `store` takes `&mut self` although the shared form needs `&self`; `material.rs:120` says the imbalance is seeded 0. | `tt.rs:839`, `114`, `266–285`, `376–383` | verified | I1 docs and the `holds` symmetry (bench) | D.5, D.8 |
| TT-14 | `PawnEntry` is 80 bytes and straddles cache lines; a 56-byte packed form fits one line. Not a B.9-to-C change. | `eval/mod.rs:84–104` | speculative | only on a profile showing pawn-cache stalls | D.8 (note) |
| TT-15 | The shared table's allocation is infallible (a process abort near the memory limit, not the notice), first-touched by one thread; Linux and macOS get no huge pages. | `tt.rs:765–778` | verified | G.1 | G.1 |

### 3.5 The evaluation (`src/eval/`) against Stockfish 11

No correctness defect: colour and orientation masks, the king-danger inputs
and reductions, shelter and storm indices, the KRPKR, KRPKB and KPK ports and
the band bounds were read against the donor line by line; the symmetry sweep
and the `texel` reconstruction gate pass. The term table (about 80 rows) is in
the artifact report; the evaluation freeze holds (`git diff 9fb525e HEAD --
src/eval` is the six-line `reallocate_pawn_table`).

| ID | Finding | Where | Confidence | Action, class, gate | Owner |
|---|---|---|---|---|---|
| EVAL-3 | The king-danger inputs are computed on plain attack maps; the donor's bishop attacks see through queens, its rook attacks through queens and own rooks, and a pinned piece's attacks are cut to the king line (`evaluate.cpp:269–274`). C.3's 500-row fixture came from a donor build patched to plain maps, so it proves Rarog reproduces the plain donor and nothing about the donor as played. RAR-E72 placed the donor's king lead in the four inputs that read those maps; RAR-E73's refit from the donor's seeds enlarged the donor's residual, which is what different inputs look like. | `attacks.rs:146–149`, `155`; `king.rs:183–238`; `tools/diag/patches/sf9587_plain_maps.patch` | deviation verified; that it carries the lead is speculative | zero games: stack the plain-map patch on the sub-term patch and read RAR-E72's (B) with each bundle; frozen rule: x-ray and pin lines are the carrier if the plain bundle's (B) is at most half of +0.194; if so, an attack-map producer change folded into E.4 (moves mobility, threats and king danger together: case 3 with a refit) | E.4 |
| EVAL-5 | The endgame scale multiplies the whole tapered score, tempo included; both donor commits scale the endgame half only (`v = mg·p + eg·(128−p)·sf/64`). Invisible for dead draws and pure endings; acts for the opposite-bishop rule with pieces on the board and the partial recognisers at small phase. The constants were fitted inside Rarog's form (RAR-E26); the form itself was never read. | `eval/mod.rs:327–347`; `endgame/mod.rs:25–61`; `trace.rs:166–181` | verified | R2 offline: refit the three OCB constants with the scale on the endgame half on RAR-E26's cohort and compare held-out loss; then case 2 at most | D.6 |
| EVAL-6 | The donor's pawnless default factor (`material.cpp:195–203`: 0, 4 or 14 of 64 when a side has no pawns and a small material edge) is absent; K+B+N v K+B keeps two thirds of about a knight's value. Never a ranked family (the family list is SF11's endgame functions). | `endgame/mod.rs:186–209` | verified | R2: count the families in the game record first; at ≥ 0.5% of games the Syzygy drawn-overclaim read of RAR-E36; case 2 | D.6 |
| EVAL-10 | Definitions no experiment read: outposts use the current enemy pawn attacks, not the attack span (a knight on d5 against c7 counts until c6 is played); minor-behind-pawn counts own pawns only; the long diagonal reads the enemy king zone, not the two centre squares. | `pieces.rs:110–131`, `244–257` | verified; small | R2: an E62-style direction read, zero games; only a term at the line enters a refit cluster | D.6 |
| EVAL-1 | CI and the verification list never run the `texel` or `tune` evaluation tests (the integer-exact trace reconstruction gate among them; 37 pass today). | `.github/workflows/ci.yml:114–120`; `trace.rs:204–572` | verified | I1: one CI step per feature and a line in AGENTS' verification list | D.5 |
| EVAL-2 | All 91 EPD positions and every recogniser unit test have White as the stronger side; Black paths are reached by a few corpus rows (KRP-KB and KNN-KP by none). The Black paths read correct. | `tests/endgames.epd`, `endgame/mod.rs` tests | verified | I1: run the EPD and the unit FENs through `mirror_fen` in `tests/eval_invariants.rs` | D.5 |
| EVAL-4 | The "Stockfish 11" donor is two commits: tag `sf_11` and `9587eeeb` (2020-07-30); the king-safety fixtures, the safe-check split, the storm per rank, the queen-imbalance doubling and queen infiltration follow `9587eeeb`; PROCESS names one. | `docs/PROCESS.md`, `eval/mod.rs` header | verified | docs: name both and which module follows which | PROCESS (this change) |
| EVAL-7 | False or stale comments: imbalance and initiative "seeded 0" (fitted, non-zero), the lazy-gate rationale (the king map alone can subtract 637 cp against a 475 margin; the gate's support is RAR-E19/E20), the king cap bound, 54 roadmap-step references. | `material.rs:119–120`, `initiative.rs:11–14`, `mod.rs:304–306`, `king.rs:15–18` | verified | I1 comment-only commit | D.8 |
| EVAL-8 | Shape: `eval_piece_activity` 262 lines, `eval_pawns` 183, `eval_threats` 152 with ten arguments; about 122 hand-written trace sites beside 73 accumulations, where an omitted trace site is born. | `pieces.rs:57`, `pawns.rs:170`, `threats.rs:13` | verified | I1: a `Score` accumulator and one `term!` macro emitting both halves and both traces; a per-colour context; fingerprint exact plus the reconstruction test; case 1 | D.8 |
| EVAL-9 | `LIGHT_SQUARES = 0xAA55…` contains a1, c1, e1, g1 (the dark squares) and its comment says a1 is dark; every use is relative, so no bug, but a recogniser written from chess knowledge would come out inverted. | `board/bitboard.rs:29–31` | verified | I1 rename (`A1_COLOUR`/`H1_COLOUR`) or swap with the corner constants; case 1 | D.5 |
| EVAL-11 | Exact folds: the bishop pair is represented three times (flat, `imbalance_ours[0][0]`, per-pawn); `hanging_rook`/`hanging_queen` are 1; trapped bishops fire on 0.04% of rows. | `pieces.rs:86–100`, `threats.rs:166–195` | verified | only with E.4's rebuild | E.4 |
| EVAL-12 | Two parameter paths (compiled handle against runtime `EvalParams`): keep both; the production fold is a measured keep and the tuner needs the runtime form. In-file unit tests run the runtime path, `tests/` and `bench` the compiled one. | `eval/mod.rs`, `params.rs:325–359` | likely | none; `masks.rs` removable after E.4 | D.8 (P8, P10) |
| EVAL-13 | Fit anomalies: h7 pawn PST mg −29 against 139–254 on the rank; `mob_q_eg` 33 → 84 between counts 5 and 6; large Rarog-only terms. | `material.rs:16–20`, `params.rs:134` | verified | a second-difference penalty if E.4 reopens the linear surface | E.4 |

### 3.6 The shell, protocol, lifecycle and Syzygy FFI (`src/engine.rs`, `engine_command.rs`, `uci_protocol.rs`, `search_options.rs`, `syzygy.rs`)

B.2.0's protocol findings landed (the tagged command, one parser, the
duplicated helpers); presentation is half fixed (the search writes through
`InfoSink`; 19 `info_string!` sites remain, 15 in the option parser).

| ID | Finding | Where | Confidence | Action, class, gate | Owner |
|---|---|---|---|---|---|
| PROTO-1 | Clocks parse as `usize`, a failure becomes 0 with a notice, and `time.rs` reads 0 as "no clock", so `go wtime 0 btime 0`, `go wtime -50 btime 1000` and a missing own clock search until `stop`. Replayed: `go wtime -50 btime 1000 winc 0 binc 0` ran past 3 s with no limit. The donor keeps presence with `Option` and moves at once on a zero clock; Stockfish clamps to 1 ms. | `search_options.rs:314–325`, `553–565`; `time.rs:145–149` | verified by replay | I1: signed parse, `Option` per side, a present clock clamped to ≥ 1 ms; a unit test on `compute_runtime_limits` and a `uci_process` case; case 1 | D.3 |
| PROTO-2 | `set_option` calls `wait_for_search_finished`, which pushes a `Ready` behind a running `Go` and blocks the stdin thread; under `go infinite` the only line that can end the search is one that thread will never read. Replayed: killed at 7.5 s with `stop` and `quit` unread. Nothing needs the wait (the FIFO already orders `Configure` after the `Go`); the donor forwards only `isready`, `stop` and `quit` while searching. | `uci_protocol.rs:157–166`, `239–246` | verified by replay | I1: drop the wait (or answer with a notice); `uci_process` test; case 1 | D.3 |
| PROTO-3 | Stdin EOF pushes an ordinary FIFO `Quit` (deliberately, so a piped `bench` completes) and sets no flag an infinite or finished-ponder search polls, so the engine searches or sleeps forever when stdout stays open. The donor sets stop before queueing quit. | `uci_protocol.rs:44–48`; `engine.rs:128–150` | reviewer-confirmed (alive 16.5 s after EOF) | I1: an `eof` flag treated as `Stop` for `infinite` and `ponder` searches only; case 1 | D.3 |
| PROTO-4 | Every library `println!` panics when stdout is closed; under `panic = "abort"` the engine thread dies as a crash on a normal GUI shutdown order. | `engine.rs:23–27`, `417–424`; `lib.rs:10–15` | reviewer-confirmed | I1: `writeln!` on a locked handle and `request_quit` on `Err` (one "GUI gone" signal that also serves PROTO-3); case 1 | D.3 |
| PROTO-5 | A non-UTF-8 input line makes `read_line` return `Err`, which `unwrap_or_default()` turns into 0 bytes, i.e. EOF: the engine exits silently with status 0. | `uci_protocol.rs:44` | reviewer-confirmed | I1: `read_until` plus `from_utf8_lossy`; only `Ok(0)` is EOF; case 1 | D.3 |
| PROTO-6 | A bad `position` ends the process with `process::exit(1)` from library code, with no `bestmove` if a search runs; both donors keep the previous position. The exit has one advantage (the message reaches the retained transcript); a silent continue would play from a stale board. | `uci_protocol.rs:173–177`, `305–309` | verified | R2: decide and record in D.3; move the `exit` to `main.rs`; answer a pending `go` with `bestmove 0000` first. The reviewer recommends against changing the policy without one logged game that needed it | D.3 |
| PROTO-7 | `stop` during `bench` or `wac` ends the suite early and prints an ordinary-looking summary (`Nodes searched : 110427` against 1,679,897 for the same `bench 9`); the PGO trainer waits for that summary. | `engine.rs:152–323` | reviewer-confirmed | I1: print "bench interrupted" and no summary on `Stop`; case 1 | D.3 |
| PROTO-8 | A refused `Hash` is retried and re-announced on every later `setoption`, because option state lives twice (the protocol's copy and `SearchShared.hash_mb`) and every `Configure` resends the whole struct. | `mod.rs:359–381`; `uci_protocol.rs:157–165` | reviewer-confirmed | I1: remember the last requested size, or configure only the changed option; case 1 | D.3 |
| PROTO-9 | The tablebase PV extension and `tb_root_board` gate on the loaded tables' size, not on `SyzygyProbeLimit`, which the root ranking and in-search probes honour; with `SyzygyProbeLimit 5` and 6-man tables the extension still reads the 6-man root after the search, under the clock. | `mod.rs:515`; `syzygy.rs:503–511`; contrast `mod.rs:748`, `1712–1716` | verified by reading | V: a 4-man fixture pair first; I1: take `largest` from `shared.syzygy.largest`; case 1 with a static differential on the limited domain | D.9 |
| PROTO-10 | The pre-search root ranking is one unabortable FFI call with no clock check, and each PV-extension step is one such call; under `go movetime` the extension starts regardless and with no clock it is unbounded. Stockfish ranks per move with a time-abort callback. | `syzygy.rs:363–383`; `mod.rs:799–806`, `552–600`; `time.rs:128–143` | verified by reading | R2 then I2: rank per move (`tb_probe_root_dtz` is declared, unused) with a deadline, or skip the pre-search ranking near the ceiling; case 2 if the root ranking changes | D.9 |
| PROTO-11 | Fathom maps tables lazily on first probe from any search thread and calls `exit(EXIT_FAILURE)` when a mapping fails, invisible to `crash_report` and with no `bestmove`; its one `printf` bypasses Rust's stdout buffer. | `vendor/fathom/src/tbprobe.c:717–728`, `1540–1542`, `926–931` | verified by reading | documentation: a reason for D.9's pure-Rust prober candidate | D.9 |
| PROTO-12 | The tablebase file-count notice splits the path on `;` on every platform; Fathom uses `:` outside Windows; the unit test pins the Windows form. | `syzygy.rs:208–232`, `676–692` | likely | I1: `std::env::split_paths`; message only | D.9 |
| PROTO-13 | `syzygy::probe_root`, `RootProbe` and a dead `tb_init` branch (it returns `true` at both exits) carry an `unsafe` site for nothing; D.9's text still says six `extern` declarations and `tb_probe_root_dtz`, which the code does not declare. | `syzygy.rs:134–138`, `259–291`, `174–193` | verified | I1: delete; the unsafe count falls by one; fix the leaf text | D.5 |
| PROTO-14 | `SyzygyProbeDepth` gates every probe; Stockfish gates only the largest loaded size and sets the depth to zero when the limit exceeds the loaded tables. Default 1 never differs. | `mod.rs:1705–1710` | verified | I1 to the donor semantics; case 1 | D.9 |
| PROTO-15 | Root probes are documented "NOT thread safe" and run beside helpers' WDL probes in the no-clock PV extension; the vendored path shows only stack state, so probably safe, but outside the contract and untested. | `tbprobe.h:271–272`; `mod.rs:1615–1630` | speculative | V: run the extension after helpers stop, or a threaded test with a 4-man fixture | D.9 |
| PROTO-16 | Option ranges are declared twice (advertised string and clamp), the parser prints its own notices, and an invalid value still queues a full `Configure`. | `search_options.rs:204–213`, `411–473` | verified | I2 with P12: one option table generating both; `OptionUpdate::{Applied, Rejected, Unknown}`; a golden test of the `uci` lines | D.3 |
| PROTO-17 | The shell is 1,941 lines (about 1,250 without tests) against Reckless's 596; most of the difference is earned (two threads with epochs give correct `isready`, ponder and stop-before-start), but `run_bench` and `run_wac` are two copies of the suite loop and the controller prints 14 lines itself. | `engine.rs:152–414` | verified | I2: one `run_suite`; then the planned `src/uci/` move | D.3, D.8 |
| PROTO-18 | `go mate N` is a fixed-depth search to 2N−1 plies that returns whatever it has; `SearchLimits` uses 0 as "absent" for five fields. | `search_options.rs:337–344` | verified | low priority; an `Option` mate limit that stops on a proven mate | D.3 |
| PROTO-19 | The queued `PonderHit` command does nothing (the atomic delivers it); `bench`/`wac` push a `Stop` they do not need. | `engine.rs:82`; `uci_protocol.rs:194`, `212` | verified | I1 delete | D.3 |
| PROTO-20 | Test gaps for everything above; `multipv_syzygy` passes in 0.00 s without `RAROG_SYZYGY_PATH` (a skip reported green). | `tests/uci_process.rs`, `multipv_syzygy.rs:21–30` | verified | V: the cases above; `#[ignore = "needs RAROG_SYZYGY_PATH"]` | D.3 |
| PROTO-21 | The PGO training workload is single-thread `bench` only (see RUST-1). | `xtask/src/main.rs:1174` | verified | as RUST-1 | D.2 |
| PROTO-22 | The banner and the CPU advisory print before the handshake; acceptable, the advisory could be an `info string` after `uciok`. | `main.rs:33–44` | verified | none required | — |
| PROTO-23 | `infra`'s narrowing conversions are checked in debug builds only; the four `to_usize` sites are bounded by construction. | `infra.rs:30–121` | verified | none | — |
| PROTO-24 | A non-ASCII `SyzygyPath` cannot load on Windows (Fathom's ANSI `CreateFile` receives UTF-8 bytes). | `build.rs`; `tbprobe.c:322–341` | speculative | `_UNICODE` plus a UTF-8 conversion, or document ASCII paths | D.9 |

### 3.7 Rust quality census

- **Lints.** `cargo clippy --all-targets -- -W clippy::pedantic -W clippy::nursery`
  (default features): 904 unique hits over 48 lints; top `inline_always` 228
  (a measured policy), `unreadable_literal` 137 (the magic tables),
  `must_use_candidate` 108, `missing_const_for_fn` 90, `cast_lossless` 55,
  `doc_markdown` 40, `use_self` 37, `suboptimal_flops` 36. Worth adopting
  (RUST-10, 65 mechanical edits, same code): `cast_lossless`, `borrow_as_ptr`
  (four `&mut x` coerced to `*mut` at FFI calls; the crate already denies
  `raw_borrows_via_references`), `checked_conversions`, `manual_assert_eq`,
  `no_effect_underscore_binding`. Rejected: `suboptimal_flops` (`mul_add`
  changes the time manager's f64 results). Default-level clippy is at zero
  for default, `diag`, `tune`, `texel`, the workspace and the PEXT tier;
  `cargo fmt --check` passes.
- **Unsafe.** Of the 23 sites, `main.rs:106` (`timeBeginPeriod`) can become a
  2024-edition `safe fn` declaration (compile-checked on 1.99.1), and the four
  slider-table `get_unchecked` sites can collapse to one helper with identical
  codegen (`pext; add; mov`): floor 23 → 19 if the maintainer agrees (RUST-2);
  PROTO-13 removes one more. `_mm_prefetch` and `_pext_u64` need `unsafe` even
  with the features enabled globally (probe-verified). The Fathom FFI layouts
  match today (`TbRootMoveRaw` 528 B, `TbRootMovesRaw` 101,908 B) with nothing
  checking them at compile time (RUST-5: `const` size asserts and a test that
  parses the header's constants).
- **Hot path.** 301 `memcpy`/`memmove`/`memset` calls in the release
  assembly, none in any node-path function; the only allocator edge from the
  node functions is `Board::history`'s never-taken growth; no `Arc` refcount
  on the node path; one `lock incq` per `negamax` copy (`record_tb_hit`) and
  the 2,048-node poll's RMWs. Every `negamax` and `quiescence` copy has a
  5.0–5.6 KB frame (the `MovePicker` with its 4,104-byte scored list, the
  ProbCut and quiescence lists), so every node calls `__chkstk` (RUST-3,
  speculative, at most about 1%; a design change). Cold `memcpy` of
  stack-built tables in `HistoryTables::default` (65,536 and 21,504 bytes)
  and the 101,904-byte `TbRootMovesRaw` built twice per tablebase root rank
  (RUST-8).
- **Edition and recent Rust.** Let chains, `let … else`, `is_none_or`,
  `array::from_fn`, `select_unpredictable` and `cast_signed` are already in
  use; three plain `#[allow]`s can be `#[expect]`, twelve `#[expect]`s lack a
  reason, and `clippy::allow_attributes` is adoptable (RUST-9); `const fn`
  still rejects `for` on 1.99.1; `core::hint::cold_path` is stable
  (`likely`/`unlikely` are not); the KPK bitbase is the one table built at
  run time (524,288 bytes, exactly the large-page threshold, so it takes a
  2 MB locked page; a bit-packed `[u64; 1024]` built at compile time or by
  `build.rs` is 65 KB: RUST-4). CI has no PEXT-tier cell (RUST-7; the cell is
  clean today).
- **Size.** `src/` 29,190 lines: 6,753 test-gated (23.1%), 3,964 comment,
  1,514 blank, 16,959 code; `tests/*.rs` 7,774; `xtask` 2,160;
  `tools/texel-tuner` 3,222; `tools/**` 101 Python and 23 PowerShell files,
  29,594 lines. Largest functions: `negamax` 1,662, `quiescence` 355,
  `search_root` 351, `search_root_multipv` 279, `eval_piece_activity` 262,
  `eval_pawns` 183, `run_bench` 172, `make_move_inner` 169, `set_option`
  168, `search_parallel` 154.

### 3.8 Architecture and code volume (the B.2.0 recipe repeated)

| Quantity | B.2.0 (`a8b6640`) | Now (`a3dd950`) |
|---|---|---|
| `src/` files / lines | 36 / 21,916 | 51 / 29,190 |
| Non-test code / in-file test lines | 14,082 / 3,097 (14%) | 16,950 / 6,753 (23%) |
| `pub` items / zero outside-file references / test-or-tool-only | 593 / 98 / 33 | 657 / 77 / 48 (plus 32 `pub(super)`/`pub(crate)` that could be private) |
| Feature sites `tune` / `diag` / `texel` | 9 / 63 / 24 | 55 / 94 / 67 |
| Instrumentation, non-test lines | not counted | about 2,300 (`diag` 1,380, `texel` 514, `tune` 238 + `masks.rs` 142) |
| Retired-number references in comments | 453 | 193 (150 in `eval/`) |
| Functions over 150 lines | 10 | 11 |

Layering holds (board ← evaluation ← search ← engine ← protocol ← `main`;
`infra`, `diag`, `win32` leaves) with these exceptions: O1 `search` ↔
`search_options` (the option types and the `go`/`setoption` parser share a
file, and the parser imports `search::params`); O2 `eval` ↔ `tt` (score
constants split between them; `MAX_PLY` defined twice, `tt.rs:11` and
`mod.rs:56`; the decisive threshold written inline at nine sites); O3 `search`
→ `large_pages` (two edges, documented); O4 board geometry tables in
`eval/pawns.rs` and `board/movegen.rs`. Presentation: zero I/O in the core,
but the search formats the UCI grammar (`format_score`, seven info-line
functions, the same `info depth …` string written twice) and the option
parser prints 15 notices. Duplication is small and specific: the two root
drivers (53% of the MultiPV driver repeats the single-PV one), the `info`
line, two fixed-capacity lists, two `Compiled` handles, three attacked-square
predicates, two `ep_capture_is_legal`, two distance builders, the Fathom call
boilerplate. Answered and kept: the evaluation is written once and
parameterised through a handle; per-colour code is folded; the two stop-flag
sets are different layers; `board_v2` is a versioned instrument (RAR-M25);
`cpu_advice`, `crash_report`, `large_pages`, `wac`, `diag`'s counters, the
`tune` options and the `texel` trace all carry a record. Removable:
`trace_decision!` (about 190 lines, no consumer) and the attribution masks
(about 200 lines, no tool sets them), each with a retire condition.

The programme (owner D.8 unless stated; each item its own branch; `I1` with
the exact fingerprint, "NPS-guarded" adds the allocation guard and one
pooled-NPS read):

| # | Item | Delta | Risk |
|---|---|---|---|
| P0 | Commit the census scripts as `tools/diag/arch_census.py` with a test; E.1 re-measures with it | 0 | tooling |
| P1 | In-file tests into `<stem>/tests.rs` for the 13 files holding 5,391 test lines (`node.rs` 3,609 → 2,462, `search/mod.rs` 2,977 → 1,979, `time.rs` 966 → 392); test bodies untouched; counts equal before and after | files −5,391 | neutral by construction |
| P2 | One `score` module: the constants, one `MAX_PLY`, `MATE_BOUND`, `is_decisive`, `mate_in`; the nine inline sites; ends the `eval`/`tt` cycle | −10 | neutral |
| P3 | Geometry into `board::geometry`; one attacked-square predicate; one `ep_capture_is_legal`; one distance builder; a `TbPosition` method per Fathom entry point | −80 | NPS-guarded |
| P4 | Split `board/board.rs` (one 2,066-line `impl`) into `fen`, `see`, `draw`, `checks`, `make`, before F.1 touches make/unmake | file −1,500 | NPS-guarded |
| P5 | Delete `trace_decision!` (retire condition: no open leaf registers a decision-trace read) | −190 | neutral |
| P6 | Fold the 32 multi-line `diag` blocks into probe functions in `diag.rs` with empty `#[inline(always)]` twins; `negamax` 1,662 → about 1,270 with P5; counters equal at stride 1 | `node.rs` −250 | NPS-guarded |
| P7 | Visibility follows use, second pass (the 32 + 52 items; `pub(crate) mod diag`, `win32` narrowed) | 0 | neutral |
| P8 | Delete the family masks (maintainer confirms no open read sets them) | −200 | `tune` only |
| P9 | One `ArrayList<T, N>` under `MoveList` and `ScoredMoveList`; one `unsafe` fewer | −45 | NPS-guarded, memcpy check |
| P10 | One `Compiled<T>` in `infra`, a `runtime_params` cfg from `build.rs`, an `Evaluator` variant field | −40 | NPS-guarded (C.12's folds were measured only inside bundles) |
| P11 | Split `search/mod.rs` into `root`, `report`, `tb` (after D.2 closes) | file −2,000 | neutral |
| P12 | Adapter boundary inside D.3: option types to `search/options.rs`, the parser and its notices to `src/uci/`, a structured `InfoSink` record so score formatting leaves the search; removes O1 | −30 | neutral (D.3) |
| P13 | One iterative-deepening driver for single and MultiPV (R2 first: aspiration state, `MateWatch`, votes and the tablebase extension per line; `MultiPV 1` node-identical) | −100 to −130 | behaviour-affecting for MultiPV > 1 |
| P14 | Bench and WAC into `src/tools/`, one `run_suite`; delete `tests/board_performance.rs` (third copy of the workloads, gates nothing); `tests/common`; stale tool names (`AblationMask`, "the legacy search", `conversion_audit.py:30`) | −50 src, −310 test | neutral |

Held, with the reason: a stage split of `negamax` (R2 only if P5/P6 leave
it unreadable), a unified legal move generator (about −300 lines, highest NPS
risk, no defect behind it), the TT local backend (D.2, F16), the controller
thread and epochs (B.3.5), any `inline(always)` pass (B.2.0 F24), the backup
harness scripts (AGENTS), the three features themselves (all in use).

## 4. What was measured and what was not

Run: the eight reviews' reads; `cargo test` on the board, table, evaluation,
protocol and syzygy test binaries and the filtered library tests (all pass);
clippy pedantic, nursery and the five feature cells; `cargo fmt --check`;
a release build with `--emit=asm` (fat LTO, 383,027 lines) and its scans; a
PGO replica (instrumented build, `go movetime` at Threads 1 and 2,
`llvm-profdata` merge and show, two profile-use builds); `-Zprint-type-sizes`;
the UCI probe sessions; the symmetry sweep and the `texel` reconstruction
gate. Not run: `bench`, any NPS pool, any game. Every speed and strength
statement above is a prediction with its falsifier, not a result, and the
fingerprint is unchanged because no engine source changed.

## 5. Assignment to PLAN

| Owner | Items |
|---|---|
| D.2 (lazy SMP, research) | RUST-1/DRIVER-3/4 (the PGO coverage read and its cure), DRIVER-2/TT-6/TT-4 (F16, the shared backend's cost), DRIVER-1 (time-to-depth), TT-5/DRIVER-5 (prefetch sites), DRIVER-6, DRIVER-7/RUST-6, DRIVER-8 (instrument rule), TT-9/DRIVER-19, NODE-7 note, DRIVER-16/17/22 recorded |
| D.3 (lifecycle and protocol) | PROTO-1 to 8, 16 to 20, DRIVER-18, P12 |
| D.5 (behaviour-neutral repairs, new) | TT-1/3/DRIVER-10/20, TT-2, TT-7 notice, TT-12/13, DRIVER-12, DRIVER-13 comments, DRIVER-14/TT-8, NODE-6, NODE-13, NODE-14, PROTO-13, RUST-2, RUST-4, RUST-5, RUST-7, RUST-8, EVAL-1, EVAL-2, EVAL-9, BOARD-3, BOARD-4, BOARD-5, BOARD-7 |
| D.6 (strength candidates, new, each gated) | NODE-3/BOARD-1, BOARD-2, NODE-1, NODE-2, NODE-4, NODE-5, NODE-8, DRIVER-11, EVAL-5, EVAL-6, EVAL-10, TT-7 index, DRIVER-13 lever, NODE-7/TT-10, NODE-10, NODE-12, BOARD-9 |
| D.8 (simplification programme, new) | P0 to P11, P13, P14, NODE-9, NODE-15, EVAL-7, EVAL-8, RUST-3, RUST-9, RUST-10, RUST-11, TT-13 comments, PROTO-17, BOARD-6, BOARD-10 |
| D.9 (tablebase policy) | PROTO-9 to 12, 14, 15, 24, DRIVER-15, NODE-11, TT-11 |
| E.1 | the architecture re-run is this document; P0 is its instrument |
| E.4 | EVAL-3 first, EVAL-11, EVAL-13 |
| G.1 | TT-15, DRIVER-16's quorum at five threads and above |
| PROCESS | EVAL-4 |
