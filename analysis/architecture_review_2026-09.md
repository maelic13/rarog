# Architecture and design review — PLAN B.2.0

Revision `a8b6640` on `dev` (the B.1 head plus the B.2.0 leaf definition),
2026-09-14. Research state of leaf B.2.0, class `R3`. **No source changed.**
This document is the review half of the leaf; §5 is the handoff for the
upgrade half (`I2`), and PLAN's register moves to `READY_FOR_IMPLEMENTATION`
with it. Every finding is classified **keep**, **upgrade now** (B.2.0 owns
it) or **upgrade in owner** (a named later leaf), with the evidence. Nothing
here changes playing behaviour: every accepted upgrade must reproduce
`bench 13` = 7,601,220 / EBF 2.474 and stay inside ±0.5% of the B.1 NPS pool.

Method. Every figure below was produced mechanically on the idle host and is
reproducible from the recipe in RAR-M51: the crate was outlined with `rg`
over declarations and `use` edges; a script counted, for each `pub` item,
the references outside its own file (in `src/`, `tests/`, `benches/`,
`xtask` and `tools/texel-tuner`); comment density and the retired-numbering
census are regex counts; the idiom census is `cargo clippy --all-targets -W
clippy::pedantic -W clippy::nursery` on the default features; the build was
timed after touching `src/lib.rs`; the fingerprint was read from the hashed
binary that build produced. Prose comments were read, but no finding rests
on one. Prior reviews are linked, not repeated: `analysis/consolidation_2026-09-10.md`
(A.6, module dispositions and the B.1/C.1 handoffs, which this review
confirms held), `analysis/archive/infra_analysis.md` (2026-07-13, board and
infrastructure) and `analysis/code_audit_2026_08_19.md` (search state
audit). The "Clean Architecture" yardstick used is the one the sibling
engine wrote down for itself in Manta's ADR-0001, -0006, -0007 and -0012: a
functional core with inward dependencies, an imperative shell that owns I/O
and lifecycle, concrete types on the hot path, presentation only in the
adapter, and one composition root.

## 1. Verdict

The engine's architecture is sound and its layering is already inward: board
← evaluation ← search ← engine ← protocol ← `main`, with `infra` and `diag`
as leaves and no module importing the protocol layer. B.1 gave the search a
scaffold that matches both donors (`NodeType`, `ThreadData`, sentinel
`PlyArray`). Nothing found here justifies a structural rewrite before the
clusters, and nothing found contradicts the A.6 decision that Phase A
refactors nothing.

What the review does find is **accumulated weight, not a wrong shape**:

- **Comments record history instead of explaining code.** 453 references to
  retired phase, step and ledger numbers in `src/`, 366 more across the
  tools and configuration; `params.rs` is 57% comment lines, `main.rs` 46%,
  and one search block is an empty `if` kept alive to hold a paragraph.
- **The public surface is three times its use.** 593 `pub` items; 98 are
  referenced nowhere outside their own file, 33 more only by tests and
  tools. Six alias pairs offer two names for one operation.
- **The protocol layer carries a boolean command bag and three copies of the
  same parser**, and `Searcher` owns engine resources (table, worker pool)
  next to per-thread state, so each helper thread carries an empty worker
  pool and a second copy of the engine's option state.
- **Presentation leaks into policy** at 22 `info_string!` sites and two
  `println!` sites, five of them inside `search/`.
- **Two footguns are structural, not habitual**: the `texel` feature unifies
  into every workspace-wide Cargo command, and the LMR table is built twice
  per `Searcher` because `Default` seeds it with constants the parameters no
  longer hold.

All of that is behaviour-neutral to remove, and §5 freezes the removal as
twelve ordered tickets. The one architectural question that matters for
strength — per-thread state versus engine-owned resources in the node
kernel, and where the shared tables live — is answered here as a design
(§4.3) and **handed to B.2.1 as its ticket 0** rather than executed now: the
cluster rewrites every line the split would touch, and moving them twice
would cost two fingerprint-and-NPS qualifications for zero strength, the
exact pattern A.6 refused.

## 2. Measurements

All on `a8b6640`, this host idle (CPU 6–7% before measuring), the pinned
1.98.1 toolchain. Raw outputs are archived in ignored
`tools/results/b20-20260914/` and hashed in RAR-M51.

### 2.1 Crate shape

| Quantity | Value |
|---|---|
| `src/` | 36 Rust files, 21,916 lines (about 3,200 of them in-file tests) |
| `tests/` | 22 test files, 5 fixtures, 6,172 lines; 24 in-file test modules in `src/` |
| `#[test]` attributes | 128 in `src/`, 165 in `tests/` (the suite runs 283 debug / 284 release after feature gates) |
| Tooling crates | `xtask` 1,771 lines (one file), `tools/texel-tuner` 2,719 lines (one file) |
| `tools/` | 99 tracked files, 61 of them under `tools/diag/` |
| Runtime dependencies | none; `cc` at build time for the vendored Fathom |
| Features | `tune` (9 `cfg` sites), `diag` (63), `texel` (24), `ablate` (2) |
| `unsafe` | 18 blocks, 2 `unsafe extern`/`fn` items, 16 `SAFETY:` notes |
| Lint suppressions | 16 item-level (15 `expect`, 1 justified `allow`), 2 module-level `allow` |
| Release build after touching `lib.rs` | 15.8 s wall (fat LTO, one codegen unit) |
| Production binary | 937,472 bytes, sha256 `f6b5d5bf…bf5b56` |
| `bench 13` on that binary | **7,601,220 / EBF 2.474** (exact) |

### 2.2 Coupling inside `search/`

`Searcher` has 19 fields. Method count and field access per file:

| File | Lines | Methods | `self.params` | `self.td` | `self.tt` |
|---|---|---|---|---|---|
| `mod.rs` | 1,572 | 47 | 17 | 59 | 8 |
| `node.rs` | 1,772 | 11 | 49 | 31 | 13 |
| `movepick.rs` | 721 | 27 | 0 | 6 | 0 |
| `history.rs` | 456 | 18 | 7 | 25 | 0 |
| `correction.rs` | 189 | 6 | 7 | 16 | 0 |
| `threads.rs` | 615 | 21 | 0 | 14 | 3 |
| `stack.rs` | 171 | 9 | 0 | 5 | 0 |

Six files extend `impl Searcher`; the kernel reads parameters 49 times and
per-thread tables 31 times through one `&mut self`. `worker_default()` is
byte-for-byte `Searcher::default()`.

### 2.3 Public surface

593 `pub`/`pub(crate)`/`pub(super)` items. **98 have no reference outside
their own file** (about 60 of them crate-`pub`), and **33 crate-`pub` items
are referenced only from `tests/`, `benches/`, `xtask` or the tuner.** The
integration tests import from nine modules: `board` (31 imports), `eval`
(9), `search` (7), `search_options` (7), `tt` (3), `wac` (2),
`search::params` (2), `engine_command` (1), `bench` (1). The tuner imports
only `eval`, `board::Bitboard` and `board::Color`. Everything else that is
`pub` is public by habit. Appendix A lists the 98.

`Board` exposes 68 public methods and 6 public fields. The fields are read
36 times outside `board/` (`hash` 18, `halfmove_clock` 10, `castling` 5,
`fullmove` 2, `side_to_move` 1) and **written nowhere** outside it; the
`side_to_move()` accessor is called 29 times against one raw field read.
Aliases with no distinct behaviour: `piece_on`/`piece_type_at`,
`en_passant`/`ep_square`, `make_move_unchecked`/`make_move` (24 calls of
the alias), `Move::source`/`from_sq` and `Move::dest`/`to_sq` (0 calls),
`Board::is_capture`/`Move::is_capture` and `Board::is_en_passant` (0 calls),
`infra::to_i8` (0 calls). Move generation is exposed eleven times on `Board`
and eleven times as free functions in `movegen`, six of the free functions
re-exported from `board.rs`; no external file imports a free generator.

### 2.4 Comments

| File | Lines | Comment lines | Share | Retired-number refs |
|---|---|---|---|---|
| `search/params.rs` | 514 | 295 | 57% | 64 |
| `main.rs` | 135 | 63 | 46% | 8 |
| `crash_report.rs` | 222 | 81 | 36% | 2 |
| `cpu_advice.rs` | 278 | 95 | 34% | 3 |
| `search/stack.rs` | 171 | 59 | 34% | 6 |
| `diag.rs` | 746 | 222 | 29% | 31 |
| `search/shared.rs` | 332 | 96 | 28% | 16 |
| `eval.rs` | 3,756 | 770 | 20% | 115 |
| `search/node.rs` | 1,772 | 349 | 19% | 66 |
| `tt.rs` | 1,059 | 168 | 15% | 14 |
| `board/board.rs` | 2,543 | 304 | 11% | 13 |
| `search/mod.rs` | 1,572 | 175 | 11% | 23 |

Retired-number census across `src/` (453 total): `Phase N` 138, `4.x` 119,
`9.x` 82, `8.x` 70, `10.x` 57, `RAR-` 38, `3.x` 32, `2.x` 31, `B.` 11,
`7.x` 10, `A.` 9, `5.x` 8, `D.` 2. Of the 453, **176 sit in files B.2.0
owns** (`board/` 25, `tt.rs` 14, `diag.rs` 31, `main.rs` 8,
`search_options.rs` 6, `uci_protocol.rs` 4, `engine.rs` 3, `cpu_advice.rs`
3, `infra.rs` 4, `crash_report.rs` 2, `wac.rs` 2, `bench.rs` 1, `lib.rs` 2,
`kpk.rs` 1, `syzygy.rs` 0, plus the scaffold files `search/mod.rs` 23,
`stack.rs` 6, `thread.rs` 4, `shared.rs` 16, `threads.rs` 11, `time.rs`
10), 115 in `eval.rs` (C.1) and 162 in the mechanism files `node.rs`,
`params.rs`, `movepick.rs`, `history.rs`, `correction.rs` (B.2–B.5).
Outside `src/`: 366 references in 54 files across `tools/`, `xtask`,
`build.rs` and `Cargo.toml`; `check_guide.py`'s 18 are live (it parses
IDs), and file names such as `phase4_differential.py` are cited by ledger
rows and must not change.

### 2.5 Idiom census

`clippy::pedantic` + `clippy::nursery` on the default features: **1,001
warnings**, zero under the crate's own lint wall. Top classes: 200
`inline_always`, 147 `must_use_candidate`, 132 `unreadable_literal`, 100
`cast_lossless`, 85 `missing_const_for_fn`, 55 `use_self`, 30
`doc_markdown`, 22 `large_stack_arrays`, 16 `suboptimal_flops`, 12 + 8
`cast_precision_loss`, 8 + 7 + 4 `redundant_pub_crate`, 7
`needless_pass_by_ref_mut`, 6 `redundant_closure`. None of these is a
defect; the census is a size estimate for the style classes below and a
baseline E.1 can re-read. `inline_always` is explicitly a **keep** (§4.10).

### 2.6 Diagnostics

`diag.rs` declares 182 counters; **3 are never incremented anywhere**
(`check_extensions`, `quiet_see_prune`, `skip_quiets_nodes`). `node.rs`
carries 92 `diag_count!`/`diag_add!` sites and 63 `cfg(feature = "diag")`
blocks; the board 28. All compile out of the default build, which the
fingerprint proves.

## 3. The architecture as found

### 3.1 Layers and dependency direction

```
main.rs ── composition root: crash reporter, engine thread, protocol loop
  │
uci_protocol.rs ── stdin → commands; search_options.rs parses `go`/`setoption`/`position`
  │           (engine_command.rs: EngineControl epochs + FIFO queue with a priority lane)
engine.rs ── the controller: one thread, owns Searcher, runs bench/wac, prints bestmove
  │
search/ ── mod.rs root loop; node.rs kernels; movepick/history/correction; threads/shared (SMP); time
  │   ├── tt.rs (table), syzygy.rs (Fathom FFI, process-global), params.rs
  │
eval.rs ── Evaluator with pawn and whole-eval caches; kpk.rs (private)
  │
board/ ── Board, movegen, attacks (magic or PEXT), moves, bitboard, square, piece, zobrist

infra.rs (checked narrowing), diag.rs (feature-gated counters), crash_report.rs, cpu_advice.rs, bench.rs, wac.rs
```

Every `use` edge points down this list. `search` imports `eval` for the
`Evaluator`, three score constants and `piece_value`; `eval` imports only
`board` and `infra`; `board` imports `infra`. No module below `engine`
imports `uci_protocol`, `engine` or `std::io`. Against Manta's ADR-0001 the
layering passes on four of five points: domain values (board), inward
policy (eval, search), a controller that owns resources and lifecycle
(engine), and a single composition root (`main`). It fails the fifth,
**presentation only in the adapter**: `search/mod.rs` prints the `info`
line and four notices itself, `search/threads.rs` one, and `search_options`
answers `No such option` with a bare `println!` (§4.1).

### 3.2 One `go`

The protocol thread stamps `limits.issued`, mints an epoch through
`EngineControl::start_replacing_search`, clones `SearchOptions` (board,
engine options, limits) into an `EngineCommand` and queues it. The engine
thread pops it, checks the epoch, calls `Searcher::search`, which clones the
board, reserves 128 plies of unmake history, resets per-search state
(`reset_search_state`), filters root moves through `searchmoves` and
Syzygy, and either runs `search_root` serially or `search_parallel`, which
converts the table to the shared backend, spawns or reuses helpers each
holding a whole `Searcher`, and votes on the result. Stop, quit and ponderhit
travel as atomics polled every 2,048 nodes; helpers poll the
`SharedContext` instead. The controller prints `bestmove` after the ponder
or infinite wait. This is the ADR-0007 control plane in substance: an
epoch minted by the controller, urgent atomics ahead of the ordered queue,
one owner of the position and options, workers that never format output
except the `info` line the main thread writes from inside the search.

### 3.3 State ownership

`Searcher`'s 19 fields split into three kinds that today share one struct:

| Kind | Fields | Owner it should have |
|---|---|---|
| Engine resources, one per process | `tt`, `hash_mb`, `worker_pool`, the four `syzygy_*` settings | the engine (or a `SharedContext` that exists at every thread count, as in Reckless) |
| Per-search configuration | `params`, `lmr_table` + `lmr_table_key`, `limits`, `start` | copied into each thread at search start |
| Per-thread search state | `td: ThreadData`, `evaluator`, `stopped`, `quit`, `pondering`, `ponderhit`, `stop_on_ponderhit`, `shared_state: Option<Arc<SharedContext>>` | the thread |

Consequences visible today: every helper is a full `Searcher` and so carries
its own empty `WorkerPool`, its own `TranspositionTable` handle that
`run_worker_job` overwrites per job, and its own copy of the Syzygy
settings; `worker_default()` exists to express a difference that does not
exist; the kernel's `&mut self` is the union of all three kinds, so nothing
in the type system says which state a helper may touch. Reckless's shape,
which B.0 §6.3 already named as the target, is one `ThreadData` per thread
holding `Arc<SharedContext>` (table, status, batched counters, votes, shared
correction) plus its board, stack, histories, evaluator and a `Box<dyn
UciWriter>`, with the kernels as free functions over `&mut ThreadData`. The
design for Rarog is in §4.3.

`ThreadData` itself is right: every history and correction table is
per-thread and boxed (measured, §4.10), the stack and PV are sentinel
arrays, the root records are separate from the compact root list for cache
reasons that are recorded. `SharedContext` is right and small. `WorkerPool`
is right: persistent threads with a channel each, joined on shrink and on
drop. The `TranspositionTable` is an enum over a local 32-byte-cluster table
and a shared 64-byte struct-of-arrays table, switched per search by
`make_shared`/`ensure_local`; both layouts are asserted at compile time and
their sizing is tested. `Evaluator` owns its pawn and whole-eval caches per
thread and is cleared by `new_game`. `syzygy.rs` keeps the path in a
`Mutex<String>` and the largest tablebase in an atomic because Fathom is
itself process-global; that is the honest boundary for a C library.
`diag::counters` are process-global atomics reset per `go`, which the
serial and parallel paths reset and dump at exactly one point each.

### 3.4 Lifecycle, error and protocol contracts

- **Threads.** Protocol thread (main), one engine thread with a 16 MiB
  stack, `Threads − 1` helper threads with 16 MiB stacks, all named. The
  stack size is spelled four times (`main.rs`, `search/threads.rs`,
  `engine.rs` tests, `.cargo/config.toml` for libtest).
- **Shutdown.** `quit` uses the priority lane and pre-empts a running bench;
  stdin EOF queues an ordinary quit that a running bench completes ahead of.
  `run_once` (argv commands) uses the EOF form deliberately, and a test pins
  it.
- **Errors.** Release panics abort; `crash_report` mirrors a panic to
  stdout as `info string PANIC …` first, because the harness keeps stdout and
  loses stderr. An invalid `position` is a **critical exit** with status 1
  (tested), not a rejected command; the two donors keep the previous
  position instead. Option parse errors are `info string` notices and keep
  the previous value (tested). Failed stdout flushes are swallowed, but
  `println!`/`info_string!` themselves panic on a closed pipe, so a GUI that
  disconnects mid-output ends the process through the panic path rather than
  through EOF. Mutex poisoning is `expect`ed everywhere. A `Hash` that cannot
  be allocated keeps the old size with a notice.
- **Time.** `limits.issued` is stamped at parse (RAR-R11); `check_stop` polls
  every 2,048 nodes; the SMP reserve and the `2 × overhead` floor are D.1's
  contracts and are tested in `time.rs`.
- **Determinism.** `Threads = 1` has no `SharedContext`, so jitter, voting and
  pool ordering are absent by construction; a test pins run-to-run identity.

### 3.5 Allocation and layout: measured versus historical

Choices carrying a measurement in the source or the ledger, all **keep**:
`MaybeUninit` move and score lists (−10% NPS when initialised, 2026-07-19);
boxed continuation tables rather than `Vec` (−2.1% NPS, commit 886916b);
`Board` ≤ 264 bytes and `UnmakeInfo` ≤ 24 bytes pinned by `const` asserts
(RAR-M39); `LocalCluster` 32 bytes and `SharedCluster` 64 bytes with equal
entry density (RAR-P12/P16); the `_into` generator forms (RAR-M44, +11.2%
on legal generation); the unmake-history reservation and capacity-preserving
`Board::clone` (tested); the cold `#[inline(never)]` root bookkeeping (a real
NPS loss when inlined, recorded in place); `check_info` per node (10.3
speed pass, profiled). Choices carrying only history: the 200
`#[inline(always)]` attributes were never individually measured, and the
`StackEntry` record's locality argument was measured null (+0.11%, RAR-P17)
and is kept as substrate, which its comment says. Neither is a reason to
change anything: an `inline(always)` audit is a measured job with an NPS
pool per site, not a style pass, and the review recommends against it (§4.10).

### 3.6 Tests and tooling

The test structure is layered correctly: unit tests beside the code (24
in-file modules, 3,200 lines), integration tests over the public surface
(23 files), process tests that spawn the binary (`uci_process.rs`, 14
tests), a bench fingerprint agreement job across five platforms in CI, and
frozen fixtures under `tests/data/`. Two files are grab-bags named after a
retired step rather than a subject (`engine_coverage.rs`, 29 tests over
options, TT, evaluator and search) and five in-file modules carry phase
names (`endgame_311c_tests`, `history_contract_tests`, `narrow_tests`,
`generated_param_checks`, and the two `tune`/`texel` modules are fine).
`board_performance.rs` is a self-benchmark inside the suite, not an
assertion on a floor, so it cannot flake.

Tooling: 99 tracked files, every measurement instrument the roadmap cites
(`bench_counters.py`, `phase4_differential.py`, `fixed_budget_probe.py`,
`branching_profile.ps1`, the endgame and conversion instruments,
`check_guide.py`) is referenced by PLAN, PROCESS or a ledger row. Two tools
have no tracked reference outside their own test file
(`build_phase4_suite.py`, `endgame_reference_results.py`; the latter's
output JSON is a frozen fixture other tools consume). There is no index:
a reader learns what `tools/` contains by opening 60 docstrings.

### 3.7 Comments

The dominant comment shape in every file is a numbered narrative: what a
retired step did, what it replaced, which ledger row measured it, and
sometimes a paragraph of argument with a reader who has left the project.
The measured reasons inside them are valuable and short; the narration
around them is what makes `params.rs` 57% comment. Three concrete shapes,
each with its treatment in §7: history that no longer describes the code
(delete), an identifier used as the explanation (drop the identifier, keep
the why), and a measured reason (keep, in one sentence). One block in
`node.rs` (line 237) is an empty `if in_check { }` whose body is a
paragraph about an extension removed in a retired phase; the block is dead
code holding a history note.

## 4. Findings

### 4.1 Dependency and presentation

**F1 — keep.** Inward, acyclic module dependencies (§3.1). No action.

**F2 — upgrade now: one output port for the search.** `search/mod.rs`
prints the `info depth …` line (`send_info_line`, once per iteration on
the main thread) and four `info string` notices; `search/threads.rs` prints
one; `search_options.rs` prints one bare `println!`. Introduce a small
sink trait owned by the search (`InfoSink: Send` with one `line(&str)`
method), implemented for stdout in the engine layer and by a recorder in
tests; `Searcher` holds a `Box<dyn InfoSink>`; workers get a silent sink
(they already pass `emit_info = false`). Notices raised while configuring
(`Hash` allocation, tablebase counts, helper spawn failure) go through the
same sink. Cost: one indirect call per completed iteration and per notice —
never at node frequency. This is the ADR-0007/0012 boundary and Reckless's
`UciWriter`. `search_options` is the adapter and may keep its `info string`
notices, but the bare `println!` becomes `info_string!` so every notice has
the same shape. Test: `tests/uci_process.rs` already asserts the printed
lines; add one recorder-based unit test that a search emits its `info`
lines through the sink.

**F3 — keep, owner D.3.** `process::exit(1)` on an invalid `position` and
the closed-pipe panic path (§3.4) are protocol contracts, tested and
visible to a GUI. Changing them is a protocol decision for D.3's UCI audit,
not a neutral refactor.

### 4.2 Public surface

**F4 — upgrade now: visibility follows use.** Rule: an item is `pub` only if
`tests/`, `benches/`, `xtask` or `tools/texel-tuner` names it; otherwise
`pub(crate)`; inside `board/` and `search/`, `pub(super)` or private where
only the file uses it. Appendix A is the starting list (98 items). Keep the
`lib.rs` module list as is: the integration tests reach nine modules and
the tuner two, and `kpk` is already private. Zero runtime effect; the
compiler rejects any miss.

**F5 — upgrade now: one name per operation.** Delete `Move::source`,
`Move::dest`, `Board::is_capture`, `Board::is_en_passant`,
`Board::piece_on`, `Board::en_passant`, `Board::make_move_unchecked` (call
`make_move`; the "unchecked" name promises a distinction the body does not
have), `infra::to_i8`, `Searcher::worker_default` (identical to `default`).
Keep `piece_type_at`, `ep_square`, `from_sq`, `to_sq`.

**F6 — upgrade now: one move-generation API.** Keep the `Board` methods and
make the `movegen` free functions crate-private; remove the six re-exports
from `board.rs` (no external importer). Delete the value forms that have no
caller (`generate_captures_pinned`, `generate_quiets_pinned` and their
`Board` wrappers) and point the four test uses of
`generate_legal_captures_pinned` at the `_into` form. The remaining shape is
one legal generator, one capture generator, one quiet generator, each with
an `_into` hot form and a value form for tests, plus the pinned pair the
staged picker uses. `perft` stays on `Board`.

**F7 — upgrade now: `Board` fields become private.** Add `hash()`,
`halfmove_clock()`, `castling()`, `fullmove()` accessors (`#[inline(always)]`,
identical codegen), use the existing `side_to_move()` and `occupied()`, and
make the six fields private. 36 read sites move to accessors; there are no
external writers, so nothing else changes. `board.castling.0 == 0` becomes
`board.castling().is_empty()` or an existing `has` test.

### 4.3 Ownership

**F8 — upgrade in owner (B.2.1 ticket 0), design frozen here.** Split
`Searcher` along §3.3:

- `SearchShared` (engine-owned, one per process, wrapped in `Arc` for the
  helpers): the transposition table, the Syzygy probe settings, and at
  `Threads > 1` the existing `SharedContext` members (stop state, batched
  node and tablebase counters, root scores, votes). Whether the table stays
  an enum over two backends or becomes the always-atomic table both donors
  use is D.2's decision (F16); B.2.1 must not change it.
- `SearchConfig` (per search, `Copy` or cheap clone): `SearchParams`, the
  LMR table, `RuntimeLimits`, the start instant, `analysis_mode`.
- `ThreadData` (per thread): today's fields plus the evaluator, the stop
  flags, the jitter state, and the sink from F2.
- Kernels become `negamax<NODE>(td: &mut ThreadData, cfg: &SearchConfig,
  shared: &SearchShared, board: &mut Board, …)` or methods on a
  `SearchContext<'a>` that borrows the three; either keeps concrete types on
  the hot path and lets the compiler see which state a helper touches.

Invariants B.2.1 must preserve when it does this: `Threads = 1` stays
deterministic and free of pool machinery (today enforced by the absence of a
`SharedContext`; after the split by `thread_count == 1`, which the jitter
and vote gates read instead of `shared_state.is_some()`); the fingerprint is
exact; the pooled NPS is within ±0.5% of the B.1 pool. Reason it is B.2.1's
and not B.2.0's: the split touches every `self.td`/`self.params`/`self.tt`
site in `node.rs` (93 of them) and the picker, history and correction
files, all of which B.2.1 rewrites; doing it first and rewriting second
qualifies the same lines twice.

**F9 — upgrade now: seed the LMR table from the parameters.**
`Searcher::default()` builds the table at `(768, 2304)` and
`reset_search_state` rebuilds it at the parameter defaults `(646, 2335)`
on the first search, so every `Searcher` (main and each helper) builds the
table twice. Seed `lmr_table_key` from `SearchParams::default()`. The table
values are unchanged, so the fingerprint is unchanged.

### 4.4 Duplication and dead code

**F10 — upgrade now.** One definition each for: `flush_stdout` (two
copies), the 16 MiB thread stack size (four copies, including the libtest
env in `.cargo/config.toml` which stays but references the constant in its
comment), the 64-bit `compile_error!` guard (`lib.rs` alone suffices; the
binary crate depends on the library), `RuntimeLimits::default()` (six
struct literals, one in production), the `parse_usize`/`parse_u64`/`parse_u32`
triple (one generic `parse_or_notice<T: FromStr>`), the Syzygy promotion
decoder (two copies of the same `match`), and `SearchLimits::reset_temporary_parameters`
plus `SearchOptions::reset` (assign `Default`). Delete the three never-fired
counters (§2.6). Delete the empty `if in_check { }` block in `node.rs`
only if B.2.1 has not reached it first; it is B.2.1's line otherwise.

### 4.5 Protocol layer

**F11 — upgrade now: a tagged command instead of a flag bag.**
`EngineCommand` is eleven fields, nine constructors that each spell all
eleven, and a dispatcher in `Engine::start` that tests the flags in a fixed
order to discover which command it holds. Replace it with an enum: `Go { options, epoch }`, `Stop { epoch }`, `Quit { epoch }`, `Bench { depth, repeats, options, epoch }`, `Wac { depth, options, epoch }`, `Configure(EngineOptions)`, `ClearHash`, `NewGame`, `PonderHit`, `Ready(Sender<()>)`. `ClearHash` becomes a command rather than a boolean inside `EngineOptions` that the protocol must remember to reset after sending (today `set_option` clears it by hand). `Engine::start` becomes one `match`. The `EngineControl` epoch protocol and the priority lane are unchanged. The eight `uci_protocol` and `engine` tests that read `.stop`/`.quit`/`.bench_depth` are rewritten as pattern matches; `tests/uci_process.rs` is the behavioural check.

**F12 — upgrade now: options carry only options.** Flatten `PositionState`
(one field, three uses) into `SearchOptions::board`; move `perft` out of
`SearchLimits` — it is a protocol command handled before any search and
never read by the searcher — into the `go` handler; drop `is_go_parameter`'s
second copy of the keyword list by parsing tokens in one pass. `Configure`
then carries `EngineOptions` alone rather than a whole `SearchOptions` with a
default board and limits.

**F13 — keep, owner D.3.** The 1 ms sleep loop in `wait_until_bestmove_allowed`
and `is_ready`'s round trip are correct and measured forfeit-free; a condvar
is a D.3 refinement. The `src/uci/` move the consolidation assigned to D.3
stands.

### 4.6 Transposition table

**F14 — upgrade now, NPS-guarded.** `clear`, `hashfull`, and the
replacement-slot selection in `store_local`/`store_shared` implement one
policy twice, once per backend. Factor the policy over a small slot-access
trait so each backend supplies only its load, store and clear; keep every
function `#[inline(always)]`. This is the one B.2.0 ticket that touches a
hot path, so it lands alone and is read against the NPS pool; if the pooled
result is outside ±0.5% the duplication is restored with a one-sentence
measured reason and the ticket closes as a measured keep.

**F15 — keep.** Entry packing, the 16-bit tag with the payload fold, the
4-bit age with its replacement penalty, `TtProbe` as the single decode
point, and the compile-time layout asserts. B.2 widens the age and
changes what is stored; nothing here pre-empts it.

**F16 — upgrade in owner D.2.** Whether the local backend earns its keep. Both
donors run one atomic table at every thread count; Rarog switches
representation per search and every probe, store and prefetch branches on
the enum. The serial cost of that branch is unmeasured and the multi-thread
scaling is measured good (3.89x at 4T, RAR-P12), so this is a D.2 question
with an NPS pool on each side, not a B.2.0 refactor.

### 4.7 Comments

**F17 — upgrade now (owned files), upgrade in owner (mechanism files).**
Apply §7 to every file B.2.0 owns (§2.4 lists them with their counts) and
to the repository configuration (`Cargo.toml` 12 references, `build.rs`,
`.cargo/config.toml`, `rust-toolchain.toml`, the two workflows), and to
the prose of the tools (366 references; file names and fixture names
frozen because ledger rows cite them). Leave `eval.rs` to C.1 and `node.rs`,
`params.rs`, `movepick.rs`, `history.rs`, `correction.rs` to B.2–B.5, with
one exception: a comment in a mechanism file that is plainly false today
(the LMR "seeds leave every scale at 0" note was one such, already
corrected) is fixed by whoever finds it.

### 4.8 Tests

**F18 — upgrade now (small).** Rename the phase-named in-file test modules
in owned files to `tests`; keep `engine_coverage.rs` where it is but split
it into `search_options.rs`, `tt.rs` and `search_behaviour.rs` only if the
F11/F12 rewrite already forces most of its edits (it will: eleven of its
tests read `SearchOptions` fields). Do not move any other test; churn
without a subject gain is not lean.

**F19 — keep.** The debug-and-release matrix, the process tests, the
fingerprint agreement job and the fixture policy. The
`search_params_defaults_are_sane` tripwires on inert values are B.2.1's to
retire with the parameters they guard.

### 4.9 Tooling and repository

**F20 — upgrade now: take the tuner out of the workspace.** `tools/texel-tuner`
depends on `rarog` with `features = ["texel"]`, so every `--workspace`
command unifies `texel` into the engine, which is the mechanism behind the
first Measurement rule in AGENTS and the `-p rarog, never --workspace`
warnings in PROCESS and CI. Give the tuner its own workspace
(`[workspace]` in its manifest, `exclude = ["tools/texel-tuner"]` in the
root), build it in CI by manifest path, and delete the `-p rarog, never --workspace` warnings in PROCESS and CI that
exist only because of the unification. The `--all-features` hazard on the
`rarog` package itself remains and its rule stays.

**F21 — upgrade now: a tools index and the two unreferenced tools.** Add
`tools/README.md`: one line per tool with what it measures and which
PROCESS procedure or PLAN leaf uses it, so the directory is readable
without opening 60 docstrings. Delete `build_phase4_suite.py` (its output
`phase4_suite_v1.epd` is frozen and tracked) and keep
`endgame_reference_results.py` (its JSON is consumed). Move
`uci_specification.txt` from the repository root into `docs/` (no
reference anywhere in the tree).

**F22 — observation, owner E.1.** Documentation weight: `EXPERIMENTS.md`
427 KB, `PLAN.md` 106 KB, `HISTORY.md` 64 KB, `CHANGELOG.md` 45 KB,
`analysis/` 71 files. It is what the process requires and is out of B.2.0's
scope; E.1's attribution checkpoint is where a per-phase ledger split
would be decided.

**F23 — keep.** `xtask` (ISA contract, PGO, packaging) and the CI matrix are
the release path and are correct; their comment hygiene is F17's.

### 4.10 Measured keeps

**F24 — keep, all.** Every layout choice in §3.5 with a measurement. In
particular: do **not** run a style pass over `inline(always)`,
`unreadable_literal`, `cast_lossless`, `use_self` or `must_use`. Each is
either hot-path (the first), cosmetic in tables of fitted constants (the
second) or a churn of hundreds of sites for no reader gain. `cast_lossless`
(100 sites, `as` where `From` works) is the one class a future pass could
take mechanically; it is not lean to do now.

## 5. Handoff — the B.2.0 upgrades

State `READY_FOR_IMPLEMENTATION`, class `I2`. Twelve tickets, each landing
as its own engine or tooling commit that reproduces the exact fingerprint,
with the NPS pool read once at the end (and once alone for T6). Order is
chosen so that code moves first and comments are written last, for the
final shapes.

| # | Ticket | Findings | Files | Check |
|---|---|---|---|---|
| T1 | Delete aliases, dead items, dead counters | F5, F10 (counters) | `board/moves.rs`, `board/board.rs`, `infra.rs`, `search/threads.rs`, `diag.rs` | build, suites, fingerprint |
| T2 | Visibility follows use; one generator API; `Board` fields private | F4, F6, F7 | `lib.rs`, `board/`, `search/`, `tt.rs`, `syzygy.rs`, tests | build, suites, fingerprint |
| T3 | One definition each (§4.4); LMR seed from the parameters | F9, F10 | `main.rs`, `engine.rs`, `uci_protocol.rs`, `search_options.rs`, `search/mod.rs`, `search/threads.rs`, `search/time.rs`, `syzygy.rs`, `.cargo/config.toml` | suites, fingerprint |
| T4 | Tagged `EngineCommand`; `ClearHash` command; options carry only options | F11, F12 | `engine_command.rs`, `engine.rs`, `uci_protocol.rs`, `search_options.rs`, `tests/engine_coverage.rs` | suites incl. `uci_process`, fingerprint |
| T5 | Search output port | F2 | `search/mod.rs`, `search/threads.rs`, `engine.rs`, `search_options.rs` | recorder unit test, `uci_process`, fingerprint |
| T6 | TT policy factored over the two backends | F14 | `tt.rs`, `tests/tt_shared.rs`, `tests/tt_sizing.rs` | suites, fingerprint, **NPS pool alone** |
| T7 | Tuner out of the workspace | F20 | `Cargo.toml`, `tools/texel-tuner/Cargo.toml`, `.github/workflows/ci.yml`, PROCESS, AGENTS | `cargo test -p rarog` both profiles, tuner builds by manifest path, feature matrix |
| T8 | `diag.rs` comment hygiene | F17 | `diag.rs` | diag build at stride 1 reproduces the RAR-P24 counter set |
| T9 | Tools: index, two deletions, prose hygiene, spec file to `docs/` | F17, F21 | `tools/`, `docs/`, `uci_specification.txt` | tooling tests (`pytest tools/diag`), `check_guide.py` |
| T10 | Comment hygiene, owned `src/` files and scaffold | F17 | §2.4's owned list | fingerprint; retired-number count in owned files = 0 |
| T11 | Comment hygiene, repository configuration and workflows | F17 | `Cargo.toml`, `build.rs`, `.cargo/`, `rust-toolchain.toml`, `.github/` | CI green on dispatch |
| T12 | Test module renames; `engine_coverage` split if T4 forced it | F18 | `tests/`, owned `src/` files | suites |

Forbidden in every ticket: any edit inside a `negamax`/`quiescence` body,
the picker, the histories or the correction tables beyond a visibility
keyword or a deleted dead block (F8 is B.2.1's); any change to a parameter
value, a table size, a margin, a replacement rule or a time formula; any
`allow` where an `expect` would do; any renamed tool or fixture a ledger row
cites; any comment that restates a step number.

Done criteria (from PLAN B.2.0, unchanged): `bench 13` = 7,601,220 / EBF
2.474 on magic and PEXT release builds after every engine commit; debug and
release suites; `cargo fmt --check`; clippy at `-D warnings` on the CI
feature sets; pooled-PGO NPS within ±0.5% of the B.1 pool
(`tools/results/nps-b1-20260914/`, `nps_multibuild.ps1` interleaved on the
idle host); `check_guide.py`; retired-number references in the owned files
and the repository configuration at zero; the ledger row.

Prospective prediction, frozen before implementation: fingerprint exact at
every commit; pooled NPS **+0.0%**, 95% inside [−0.5%, +0.5%], with T6 the
only ticket that could move it; `src/` shrinks by 700–1,200 lines,
mostly comment lines and the command constructors, with `params.rs`
untouched; no crate-`pub` item is left unreferenced outside `src/`; the default-feature clippy census stays at zero; no
test is deleted except the three that exist to read `EngineCommand` flags,
which become pattern matches. Most likely failure: T6 costs speed through a
lost inline in the local store, in which case the measured keep applies.

## 6. Handed to owners

| Owner | Item |
|---|---|
| B.2.1 | F8 (state split, ticket 0, design in §4.3); the empty `if in_check` block; the inert-value tripwires in `params.rs` tests; mechanism comments in `node.rs`, `params.rs`, `movepick.rs`, `history.rs`, `correction.rs` as each is rewritten |
| B.5 | `send_info` and root-loop comments, after T5 has given them the sink |
| C.1 | `eval.rs` comments (115 references) and its split; `kpk.rs` move |
| D.1 | `time.rs` mechanism comments beyond the scaffold pass; the effort-term constant RAR-S47 measured |
| D.2 | F16 (single table backend or not); `SharedContext` owning the table; shared correction tables; SMP comments in `shared.rs`/`threads.rs` beyond the scaffold pass |
| D.3 | F3 (invalid-position exit, closed-pipe path), F13 (condvar waits), the `src/uci/` move |
| E.1 | F22 (document weight); re-run this review on the B.9 and C.11 heads with the same recipe (RAR-M51) so the counts are comparable |

## 7. Comment rewriting guide

The rule (AGENTS, Changes): a comment explains the problem or the
invariant, briefly; no roadmap phase or step numbers; no ledger IDs as the
explanation; no narration of what an earlier version did; a comment that
only records history is deleted; a measured reason to keep a shape stays,
in one sentence. Applied to the three shapes found:

**History that no longer describes the code — delete.**

```rust
// Before (node.rs)
if in_check {
    // Phase 8.2(a): the unconditional in-check extension (`depth += 1`)
    // is REMOVED. It was the first of five stacked protections around
    // checked nodes and the prime EBF suspect — … Restore this line to
    // revert on H0.
}
// After: nothing. The block and the comment go; HISTORY.md has the record.
```

**An identifier used as the explanation — keep the why, drop the number.**

```rust
// Before (engine.rs)
// 9.0a: `search` takes `&SearchOptions` now, so this no longer clones
// a whole SearchOptions (Board + SearchParams) per `go`.
let result = self.search(&command.search_options, true, command.epoch);
// After: nothing — the signature says it. If a reason survives, it is
// stated without the number:
// By reference: cloning the options here copied the board and every
// parameter per `go`.
```

**A measured reason — keep, one sentence, with the number that proves it.**

```rust
// Before (thread.rs): a nine-line KEEP-PERF paragraph with a commit hash
// and a bisect narrative.
// After:
/// Boxed fixed-size arrays, not `Vec`s: the `Vec` form cost −2.1% NPS
/// because runtime lengths defeat bounds-check elision in the hot loops.
pub(super) cont_history: Box<[[i16; CONT_SIZE]; CONT_TABLES]>,
```

**A contract — keep, in the imperative.** "Only the prefix below `len` is
ever exposed; every element below `len` was written by `push`." is a
comment that explains an invariant and stays as it is.

Where a paragraph carries evidence the ledger does not (a bisect, a
measurement not in EXPERIMENTS), move the figure to the ledger row that
owns the change before deleting the paragraph; do not delete evidence.

## Appendix A — `pub` items with no reference outside their file

Crate-`pub` unless marked; grouped by file; line numbers at `a8b6640`.
Each is a candidate for `pub(crate)`, `pub(super)`, private, or deletion
(F5 marks the deletions). Square constants are listed once as a class.

- `board/bitboard.rs`: `RANK_1`, `RANK_8`, `NOT_FILE_A`, `NOT_FILE_H`
- `board/board.rs`: `piece_type_at` (448), `color_on` (489), `en_passant` (505), `pseudo_legal_move` (512), `assert_ok` (1221)
- `board/movegen.rs`: `ray_through` (895), `is_attacked_with_occ` (1016)
- `board/moves.rs`: `is_double_push` (182)
- `board/piece.rs`: `relative_square` (37), `UPDATE_MASK` (128)
- `board/square.rs`: 36 of the 64 square constants, `from_file_rank` (114), `File::from_char` (215), `Rank::from_char` (250) — the constants stay `pub` (a type's vocabulary is not measured by use), the three functions narrow
- `board/zobrist.rs`: `ZobristKeys` (7)
- `cpu_advice.rs`: `Tier` (38), `built_tier` (62)
- `crash_report.rs`: `ReportSink` (52), `panic_line` (60), `install_with` (113)
- `eval.rs`: `EvalCounts` (181), `EvalTrace` (194), `load_from_str` (535), `EvalTables` (1060) — C.1's file; listed for completeness
- `infra.rs`: `to_i8` (47, delete), `SmallInt` (56)
- `search/correction.rs`: `correction_value` (24, `pub(super)`)
- `search/history.rs`: `CAP_HISTORY_MAX` (11), `QuietHistoryCtx` (32), `pawn_row_base` (54), `pawn_history_index` (64)
- `search/movepick.rs`: `push_with_history` (45), `tt_scored_move` (363), `append_scored_moves` (390), `score_staged_captures` (454), `score_tactical_move` (502)
- `search/node.rs`: `NodeType` (33), `Pv` (43), `NonPv` (45), `ReductionInputs` (79), `late_move_prune_count` (93), `move_gives_check` (106), `nmp_material_ok` (134) — all `pub(super)`, file-local
- `search/shared.rs`: `NO_ROOT_SCORE` (37)
- `search/stack.rs`: `STACK_SENTINELS` (13)
- `search/threads.rs`: `WorkerJob` (19), `send_search` (82), `select_parallel_result` (134), `is_root_result` (169), `parallel_vote_value` (173), `vote_for_move` (178), `parallel_result_key` (185), `reset_worker_state_for_new_game` (215), `run_worker_job` (220), `search_worker` (268)
- `search/time.rs`: `EFFORT_TERM_FLOOR` (163), `effort_term` (167), `tm_interpolate` (175)
- `search_options.rs`: `SyzygyOptions` (7), `PositionState` (51, flattened by F12)
- `syzygy.rs`: `RootProbe` (150), `RootMoveProbe` (156), `RootMoveProbes` (163)
- `tt.rs`: `refine_eval_bound_only` (673)
- `uci_protocol.rs`: `handle_command` (73)
- `wac.rs`: `WacPosition` (26)

Crate-`pub` items used only by tests, benches, `xtask` or the tuner (33):
`SeeValues::as_array`, `is_pseudo_legal`, `is_legal`,
`generate_legal_captures`, `generate_legal_quiets`, `check_consistency`,
`see_with_values`, `see_ge_with_values`, `see_ge_quiet_aware`,
`see_ge_quiet_aware_with_values`, `game_result`, `attackers_to`, eight
square constants, `EVAL_PARAM_NAMES`, `flat_coeffs`, `FLAT_SIZE`,
`to_flat`, `set_from_flat`, `set_params`, `last_trace`,
`linear_delta_scale`, `TtEntry::is_pv_node`, `allocated_bytes`,
`capacity_entries`, `score_to_tt`, `score_from_tt`. These stay `pub`; the
tests are their consumers.
