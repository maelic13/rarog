# D.9 — tablebase policy: research (2026-11-25)

D.9's research, on the compact kernel at `a8c116e` (engine source
`70ac02e`, fingerprint 3,694,298 / EBF 2.324). The items come from PLAN D.9:
the policy itself (probing depth and limits, WDL and DTZ in conversion,
interaction with the C.5 recognisers), the engine review's probe-path
findings of 2026-11-01, Basilisk's cached-path finding (A4), and the
`shakmaty-syzygy` candidate. No game was played. The evidence is code
reading against Stockfish (`D:\code\stockfish`, `1e7e6bde`, 2026-11-14),
the existing tests run with real tables, two live UCI checks and published
facts.

## Why D.9 matters: CCRL rates with tables, Rarog's pools do not

- CCRL's Blitz and 40/15 testing conditions both list
  "4, 5 or 6 piece tablebases"
  (`computerchess.org.uk/ccrl/404/about.html`,
  `computerchess.org.uk/ccrl/4040/about.html`, read 2026-11-25). The
  40/15 list header says 3-4-5 men. So the configuration the CCRL goal is
  rated in loads tables.
- None of Rarog's rated pools, gates or tunes sets `SyzygyPath`
  (`tools/colosseum/*.toml`). Only `tb-activation.toml` and
  `sprt-repair-tb.toml` do, and those were used for B.5.2.2. RAR-S94
  already recorded that its gain "does not reach the ratings".
- Exposure: 44.4% of 36,400 rating-tournament games reach six men or
  fewer, and 54.9% reach seven (`tools/diag/endgame_board_occurrence_v1.json`).
  These are games from an earlier pool, not re-read on the kernel.
- What tables are worth here: RAR-S94 read the B.5.2 tablebase repair at
  +11.4 ± 8.5 Elo with tables configured. That is a measured size for how
  much correct tablebase code matters in that configuration. It is not the
  value of the tables themselves.

## The policy against the primary donor

Stockfish is the primary donor. Each row compares Rarog's code with
Stockfish's.

| Item | Rarog (kernel) | Stockfish | Verdict |
|---|---|---|---|
| Root ranking | DTZ ranking of every legal move (`syzygy::rank_root_moves`), with WDL as the fallback. Clean wins rank equally unless DTZ is DTM. The rule-50 horizon and repetitions are ranked in. | `rank_root_moves(.., rankDTZ=false, time_abort)`, the same ranking. | Same |
| Search set at a tablebase root | Every move of the best rank. In-search probes are off when DTZ ranked, or when the root is not won. | The same: `Cardinality = 0` if DTZ is available or the root is not won. | Same |
| In-search probe gate | `ply > 0`, no excluded move, `depth >= SyzygyProbeDepth` at **every** piece count, `pieces <= min(loaded, limit)`, no castling, and counter 0 under the rule-50 option. | `pieces < cardinality` at any depth, `== cardinality` at `depth >= probeDepth`. `probeDepth = 0` when the limit exceeds the loaded size. Counter 0 always. | Differs only away from the defaults (PROTO-14). Rarog's relaxation with `Syzygy50MoveRule=false` is sound, because a WDL result without the rule does not depend on the counter. |
| Probe placement | **Before** the TT probe and cutoff (`compact/node.rs:151-176`). | After the TT cutoff (Step 7). | Deviation (NODE-11) |
| Probe score and bound | Win `TB_VALUE − ply` (a lower bound), loss its mirror (an upper bound), draw exact. Under the rule-50 option a cursed win or blessed loss scores 0. The result is stored at `depth + 6`. | The same, except that a cursed win or blessed loss scores ±2. | Minor deviation (below) |
| Tablebase band in the table | Stored root-relative, never moved by ply. Not downgraded near the fifty-move limit. | Ply-adjusted, and downgraded to `VALUE_TB_WIN_IN_MAX_PLY − 1` past `100 − r50c`. | Deviation (TT-11) |
| PV extension | Rarog's own. It runs in the main thread after the pool joins. Under a clock it gets half of `Move Overhead` and a start rule (RAR-S97). | `syzygy_extend_pv` with a time abort. | Own form, bounded |

**Conversion.** The ranking that decides conversion is Stockfish's. RAR-S93
converted 453 of 453 clean wins on the endgame cohort with tables, on the
classic kernel. D.7 did not touch `syzygy.rs` or `search/tb.rs`'s ranking.
The kernel's local real-table tests pass (below), including the 6-man
record positions. With the root inside the tables, the search only chooses
among moves of the same result. Neither a read nor a change is owed here.

**Recogniser interaction.** This is an inference, not a measurement. With
tables loaded, the C.5 recognisers and E.4.2's endgame defects at six men
or fewer still act in two places:
- in quiescence, which never probes;
- at interior nodes reached by a move that does not zero the counter,
  under the rule-50 option.

Once the root enters the tables, the root ranking decides the result class.
A capture that enters the tables zeroes the counter, so the in-search probe
fires there when `depth >= 1`. So under CCRL's conditions the ≤6-man part
of E.4.2's value is partly masked. That value should be read without
tables, as Rarog's pools are. How much the masking hides is unmeasured.
This is a note for E.4.2's ranking, not a D.9 item.

## The review's items on the kernel

Two kinds of evidence appear below. "Confirmed by code" means the current
source was read. "Confirmed live" means the head's binary was run
(`dev-20261124-a8c116e8`, a `cargo test --release` build, behaviour only).

| Item | Status on the kernel | Decision |
|---|---|---|
| PROTO-9: the PV extension and `tb_root_board` gate on the loaded size, not on `SyzygyProbeLimit` | Still present: `search/mod.rs:565` sets the board when `syzygy::largest() > 0`, and `syzygy::can_probe` bounds by `largest()`. Confirmed by code. | Repair, D.9.1 |
| PROTO-10: the pre-search root ranking and each extension step are unabortable FFI calls | Partly repaired since the review: `go movetime` now has a box (`time.rs:179`), the box is checked between probes, and under a clock the start rule keeps ten overheads (RAR-S97). Under `go movetime` no start rule applies, by design: `time.rs` and its test `the_tablebase_extension_is_unbounded_without_a_clock_and_boxed_under_movetime` hold that `go movetime` has no clock to protect, so the box alone bounds the overrun, at half of `Move Overhead`. The 14 ms the second review measured on `go movetime 10` is that box (10 ms plus up to 5 ms). The pre-search ranking is still one call with no clock check. | `NO_CHANGE` (below) |
| PROTO-11: Fathom calls `exit()` when a lazy table map fails | Inherent to Fathom. No incidence recorded. | `NO_CHANGE`; a trigger for the port |
| PROTO-12: the file-count notice splits on `;` on every platform | Still present (`syzygy.rs`, `.split(';')`). Confirmed by code; Windows is unaffected. | Repair, D.9.1 |
| PROTO-13: the dead `probe_root` and its `unsafe` | Already gone, removed in D.5. `syzygy.rs` holds 5 `unsafe` sites: the extern block and four calls. | Closed earlier |
| PROTO-14: `SyzygyProbeDepth` gates every piece count | Still present (`search/tb.rs`, `can_probe_syzygy`). It matters only when `SyzygyProbeDepth > 1`, because the search probes only at depth ≥ 1. | Repair, D.9.1 |
| PROTO-15: root probes are documented "NOT thread safe" and run beside helpers' WDL probes | This happens only in the no-clock `shown_line` extension (`report.rs:59`) at `Threads > 1` with the root outside the tables. Fathom's mutable statics (`tbprobe.c:301-514`) are written by `tb_init` alone. A table's lazy mapping is guarded by `tbMutex` with the acquire/release flag. `tb_probe_root_impl` writes only the caller's buffer. The final extension runs after the join (`threads.rs:477`). B.5.2 also ran a stress test of about 53 million probes per run on the Windows build, each checked against its thread's expected result (`analysis/b52_research_2026-10-01.md`). The Unix branch of Fathom's lock was never stress-tested. | `NO_CHANGE` in the code. A threaded fixture test goes into D.9.1, so that Linux and macOS CI run it. |
| DRIVER-15: the join waits for a helper blocked in a table read | Still present (`threads.rs:425-429`). It is bounded by one read, at most 54 ms measured (RAR-S94). | `NO_CHANGE` (below) |
| NODE-11: the probe runs before the TT cutoff | Still present (`compact/node.rs:151-176`). Basilisk's reorder read +0.3 ± 3.2 Elo at `10+0.1` with tables over 8,000 games, and cut table reads by 70% on six 7–8-man endings (H29). | Repair, D.9.2 |
| TT-11: the tablebase band skips the rule-50 downgrade | Present by design. The band is stored root-relative (`score.rs:32-38`), and B.5.2's stop rule refuted moving it by ply. The donor's test `VALUE_TB − v > 100 − r50c` therefore has no faithful counterpart here. The kernel's table cutoff already refuses to return at a counter of 90 or more (`tt_cut_returns`, seed `tt_cut_fifty` 90; Stockfish's refuses at 96). | `NO_CHANGE` |
| PROTO-24: a non-ASCII `SyzygyPath` cannot load on Windows | **Confirmed live.** The five fixture files copied into `ascii_tb` print "Found 3 WDL and 2 DTZ tablebase files (up to 3-man)". The same files in `tabulky_žluťoučký` print "SyzygyPath loaded no usable tablebases". Fathom's `_UNICODE` branch converts with `mbstowcs_s` under the C locale, so defining `_UNICODE` alone does not fix it. | Repair, D.9.1 |
| A4: an unchanged `SyzygyPath` is cached even when it loaded nothing | **Confirmed live.** A path to an empty directory prints "loaded no usable tablebases". After the fixture files were copied in, sending the same path printed nothing, and KQK at `go depth 6` read `score cp 348 … tbhits 0`. A fresh process on the same directory loads them and reads `score cp 20000 … tbhits 1`. | Repair, D.9.1 |

**Tests run, with the local 3–6-man set** (`RAROG_SYZYGY_PATH=D:/chess/tablebases/syzygy3456`):

- `cargo test --release --no-fail-fast --test multipv_syzygy --test syzygy_fixture -- --include-ignored`
  exited 0. 3 + 2 tests passed, including the 6-man record positions.
- `cargo test --release --lib syzygy` exited 0, with 9 tests passing.
- With `--include-ignored`, the known evaluation defects failed as their
  ignore reasons state:
  - D.6.21: 88 won KPK positions read at or below zero.
  - D.6.25: the bare-king promotion order.
  - D.6.26: the pawnless scale truth table.

  All three are E.4.2's items, not D.9's.

### The `NO_CHANGE` items and their triggers

- **PROTO-10 and DRIVER-15.**
  - The case for leaving them: at CCRL's clocks (2'+1", 40/15), a cold
    read of the measured size (≤ 54 ms) sits inside `Move Overhead`.
    Rarog's own short-clock pools play without tables. The one recorded
    tablebase time loss (RAR-S94) came from the extension, which RAR-S97
    fixed.
  - Trigger: a time loss at a tablebase root, or one inside a helper's
    read, in any run with tables. Or a rated pool that adopts tables below
    one second a move. Under `go movetime`, a forfeit or a harness timeout
    with tables loaded; an overrun up to half of `Move Overhead` there is
    the box working, not a defect.
  - Cheapest repair then: rank move by move through a single-position DTZ
    probe exported from the vendored C, with a deadline between moves, as
    Stockfish does. Fathom's `tb_probe_root_dtz` cannot serve, because it
    ranks every clean win equally (`analysis/tb_root_pv_2026-09-27.md`).
    Or skip the ranking once `elapsed + k·overhead` passes the hard
    ceiling.
- **PROTO-15.**
  - Trigger: a crash or a garbled line at `Threads > 1` with tables.
  - Cheapest check then: a threaded no-clock search on a 7-man root that
    captures into the 6-man tables, run in a loop.
- **The ±2 score for cursed wins and blessed losses.**
  - The case for leaving it: the domain is a cursed result under the
    rule-50 option, which is rare in games. It only orders draws among
    themselves.
  - Trigger: a game or test where scoring 0 instead of ±2 changes a move.
- **TT-11.**
  - Trigger: a stored tablebase score shown to cross the fifty-move limit
    and change a move. That requires a counter below 90 at the cutoff, so
    the guard does not catch it.

## `shakmaty-syzygy`: the two questions answered

These facts are from crates.io and docs.rs, read 2026-11-25.

- **Version and licence:** 0.28.1, published 2026-06-19 (the latest).
  The licence is GPL-3.0-or-later, which Rarog's GPL-3.0 can take. MSRV
  1.95; Rarog builds on 1.100.
- **(1) DTZ:** yes.
  - `probe_dtz` returns `MaybeRounded<Dtz>`.
  - `probe_wdl` takes the halfmove counter into account and returns an
    `AmbiguousWdl` (WDL50 with DTZ rounding).
  - `probe_wdl_after_zeroing` needs WDL tables only.
  - `best_move` returns a move and its DTZ.
  - Every probe returns `Result<_, SyzygyError>`.
  - Rarog's per-move DTZ root ranking can be rebuilt from `probe_dtz`
    after each move.
- **(2) Weight:**
  - Runtime dependencies: `shakmaty` 0.30 (a move-generation library),
    `arrayvec`, `bitflags`, `byteorder`, `once_cell`, `rustc-hash`,
    `tracing`, and `libc` on Linux. `memmap2` comes in optionally, behind
    the `mmap` feature.
  - Every probe needs a `shakmaty::Chess` built from Rarog's board.
  - The default filesystem reads with system calls, which its documentation
    calls "significant syscall overhead".
  - The memory-mapped path is `unsafe`, and turns I/O errors into
    `SIGSEGV`/`SIGBUS`. That is a process death like Fathom's `exit`.
  - The conversion cost and probe cost are not measured: building the
    crate needs it downloaded, which is the maintainer's call.
- **What it would fix:**
  - PROTO-11, but only on the default reading path, with the syscall cost.
  - PROTO-12 and PROTO-24, which D.9.1 fixes inside the current path.
  - The C build: the `cc` dependency, `TB_NO_HW_POP_COUNT`, and the
    unprofiled C object.
  - Five `unsafe` sites.
- **What it would cost:**
  - Rarog's first runtime dependency, with a second move generator.
  - A board conversion at every probe.
  - Parity work on C.5's truth source.
  - Possibly slower probes.

**Recommendation: against porting now.** Nothing it fixes is a measured
defect in a configuration that is rated, after D.9.1. Its strength effect
is near zero, and the risk falls on endgame truth.

- Trigger: an observed Fathom exit or mapping failure. Or F.1/D.12's board
  events wanting a Rust position type for probes. Or a maintainer decision
  under the clean-code policy's `unsafe` floor.
- Cheapest test then:
  - the probe and conversion cost on the endgame cohort's roots;
  - parity with Fathom on C.5's fixtures, and on the 6-man record positions
    for WDL, DTZ and the root ranking.

## Readiness

**D.9's policy closes `NO_CHANGE`:** root ranking, conversion and probe
depth at the defaults are Stockfish's. Two implementation steps are ready.

1. **D.9.1, the option and loading repairs (`I1`).** One commit each:
   - PROTO-9: the extension and the root board gate on the loaded size
     capped by the limit, `syzygy::largest().min(probe_limit)`. They must
     not gate on `shared.syzygy.largest`: `apply_tb_root` sets it to 0 at a
     DTZ root, which would turn the extension off exactly where it is
     wanted.
   - PROTO-14: Stockfish's depth rule.
   - PROTO-12: `std::env::split_paths`, with the test built through
     `join_paths`.
   - A4: cache only a path that loaded tables. The test is a `uci_process`
     case with an empty directory, then the fixture. `configure` receives
     every option on each `setoption`, so it keeps the last requested path
     on its own. Otherwise an unrelated option such as `Hash` would
     re-run `tb_init` and repeat the notice.
   - PROTO-10, its `go movetime` part: the extension's start rule under
     `go movetime`, by the clock's slack rule. Withdrawn on
     implementation, a false premise (*Corrections*, last item).
   - PROTO-15's test: a threaded no-clock search on a 4-man root that
     captures into the KQvK fixture. Helpers' WDL probes then run beside
     the main thread's DTZ extension on every CI platform.
   - PROTO-24: UTF-8 paths reach the Windows open. Either a small patch to
     the vendored open that converts with `MultiByteToWideChar(CP_UTF8, …)`
     and calls `CreateFileW`, or an embedded manifest with
     `activeCodePage` UTF-8. Implementation picks one and records it.

   PROTO-9 needs a committed 4-man pair, so its test separates "loaded"
   from "limited".

   - Gate: AGENTS case 1. Every default is unchanged, so `bench 13`
     reproduces exactly. PROTO-9's static differential is the 4-man test.
   - CHANGELOG: PROTO-24 and A4 are fixed defects a user would notice.
2. **D.9.2, the probe after the table cutoff (`I1`, then `V`).**
   Stockfish's order: the probe moves after the table cutoff and before
   the threat maps and the evaluation. A main-thread probe forces the
   next node's stop and clock poll, as Stockfish's `callsCnt = 0` does,
   because the poll otherwise waits up to 2,048 nodes, which may hold
   several cold reads.
   - Gate: AGENTS case 1, since no rated pool loads tables, so `bench 13`
     reproduces exactly. Its activation and correctness read is fixed-depth
     searches on 7–8-man roots with the local tables, before and after.
   - Frozen prediction:
     - table reads (`tbhits`) fall by 40% to 80% (Basilisk measured 70%);
     - nodes stay within ±10%;
     - no root changes its tablebase result class;
     - the fixture and local suites pass.
   - Falsifier: `tbhits` falls by less than 20%, or any result class
     changes. Then the step returns to research.

   A with-tables game read (case 2, 2,000 games) is not required by the
   rule. It is the maintainer's option, given that CCRL plays with tables.

   The read, registered 2026-11-25 before any exposure:
   - instrument: `tools/diag/tb_probe_read.py`, a fresh process per root,
     `Threads 1`, `Hash 64`, the local 3–6-man tables
     (`D:/chess/tablebases/syzygy3456`), `go depth 20`;
   - roots: the first 24 seven-man and the first 24 eight-man positions
     of `tools/books/ocb_cohort_v1.epd`, in file order;
   - builds: one before the change and one after, default features, each
     from a committed tree and archived with its hash;
   - reading: the totals of nodes and `tbhits` over the 48 roots against
     the prediction, and each root's score class (win, loss or neither);
     a root that does not reach depth 20 stops the run.

   Limits, stated before the run: the cohort is opposite-coloured-bishop
   endings, mostly drawn, so the class check is weak on wins; it is the
   only book here with seven- and eight-man roots. The depth was sized on
   two roots of the baseline alone (well under a second each at depth
   18). Fixed depth has no clock, so the post-probe poll is not read
   here; its contract is a unit test.

   **Amended 2026-11-25, before the candidate build ran.** The registered
   read stopped by its own rule on the baseline alone: two roots never
   reach depth 20.
   - Root 26 (`8/6P1/8/3b3P/5B2/3k1pK1/2p5/8 b - - 0 1`) reaches depth 17
     at 484,662 nodes, scoring a tablebase win, and its depth-18
     iteration was still unfinished at 30M nodes (11.6 s, about 2.6M
     nodes a second, so the speed is normal and the tree explodes).
     Without tables it finishes depth 18 at 3.2M nodes.
   - Root 30 (`8/8/P5bP/6B1/1K1p4/3k4/2p5/8 b - - 0 1`) reaches depth 15
     at 857,473 nodes, scoring −849, and does not finish depth 16 within
     30 s.
   - The build of the tree before D.9.1 (03049a2) behaves the same, so
     the explosion is older than D.9.1 and D.9.2. It is a finding of its
     own; see *A search explosion with tables* below.

   The amended read keeps the instrument, the roots and the prediction:
   - every search is capped at 100M nodes (`--max-nodes`), so a root
     ends instead of hanging;
   - primary: the 46 other roots at depth 20. The prediction and the
     falsifier are read on their totals, and a primary root that the
     new build leaves short of depth 20 counts as a changed result;
   - secondary, reported only: roots 26 and 30 at depth 15.

One recommendation outside D.9, for the maintainer: the release
qualification (E.1, E.3.4) could carry RAR-S93's with-tables activation
read. That is 1,500 games on the endgame cohort, about two minutes. It
would cover the configuration CCRL rates, which no pool covers. Its
parser `tools/diag/tb_activation.py` still computes the time budget by
the formula in use before D.7.9.1 (`optimum_ms`). That has to be brought
to `search/time.rs`'s `budget` before the read is used.

## D.9.2's result, 2026-11-25

The builds, default features, archived with their hashes under
`analysis/artifacts/d92-2026-11-25/`:
- before: `dev-20261125-3828d868`, D.9.1's close, SHA-256
  `82f155341b7c5be6b8d2d6663641043604d19e9ca3830e4495aeaced5179000d`;
- after: `dev-20261125-1b5f9d1e`, the probe after the cutoff and the
  post-probe poll, SHA-256
  `c9d451546bf796382c424e5eefa9768b2fb618c6ab45bdb2644a608e8df75b7f`,
  `bench 13` 3,694,298 / EBF 2.324;
- for the explosion only, the tree before D.9.1: `dev-20261125-03049a22`,
  SHA-256 `ad704df5b21abab5de862c7a64e8fbb8de96e4ca323749a89c6b38611270773c`.

The amended read, primary and secondary:

    python tools/diag/tb_probe_read.py --epd tools/books/ocb_cohort_v1.epd --men 7,8 --per-count 24 --depth 20 --max-nodes 100000000 --skip 26,30 --syzygy D:/chess/tablebases/syzygy3456 --out read_primary.json before=before.exe after=after.exe
    python tools/diag/tb_probe_read.py --epd tools/books/ocb_cohort_v1.epd --men 7,8 --per-count 24 --depth 15 --max-nodes 100000000 --only 26,30 --syzygy D:/chess/tablebases/syzygy3456 --out read_secondary.json before=before.exe after=after.exe

| Read | Before | After | Change |
|---|---|---|---|
| Primary, the 45 roots both builds finished at depth 20: nodes | 2,623,236 | 2,599,182 | −0.9% |
| Primary: `tbhits` | 118,225 | 56,865 | −51.9% |
| Secondary, root 26 at depth 15: nodes, `tbhits` | 312,682, 14,338 | 330,632, 5,945 | +5.7%, −58.5% |

- Per root on the 45: the median change in `tbhits` is −47.9%; node
  changes run from −21.7% to +4.1%, median 0.0%. No score class changes.
- Root 11 (`8/5K2/2B2p2/4pPb1/8/3k4/8/8 b - - 0 1`): the baseline
  finishes depth 20 at 1,903,586 nodes. The new build reaches depth 19 at
  1,917,508 and does not finish depth 20 within the 100M cap.
- Root 30 at depth 15: the baseline finishes at 857,473 nodes. The new
  build reaches depth 14 at 1,844,380 and does not finish depth 15 within
  the cap.

**Verdict: falsified.** By the amendment, a primary root that the new
build leaves short of its depth is a changed result, and a changed result
is the falsifier. D.9.2 returns to research, held on D.9.3. The reorder
is reverted. The post-probe poll stays, as its own repair: it reads only
the GUI and the clock, `bench 13` is exact with it, and its unit test
holds.

**Calibration**, appended; the prediction above stands as written.
- The mechanism held in sign and magnitude: table reads fell 51.9%
  (predicted 40% to 80%), nodes moved −0.9% (predicted within ±10%), and
  no finished root changed class.
- The miss is in interaction. The prediction did not foresee an older
  search explosion with tables loaded, which the reorder moves onto other
  roots and depths.
- The instrument also missed: the registered read had no node cap, so the
  explosion first showed as a hang. That was amended before the new build
  ran.

## A search explosion with tables

Found by D.9.2's read; owned by D.9.3.
- On three seven- and eight-man roots, with tables loaded, one
  iteration's tree grows from under 2M nodes to beyond 30M to 100M, at a
  normal speed of about 2.6M nodes a second.
- Root 26 does it at depth 18 in both builds and in the tree before D.9.1.
  Without tables it finishes depth 18 at 3.2M nodes and depth 19 at 12M.
- Root 30 does it at depth 16 in the baseline and at depth 15 with the
  reorder. Root 11 does it at depth 20 with the reorder only.
- `stop` is answered at once, so the search is not stuck inside one call.
- Under a clock the search stops on time and plays the move of the last
  finished iteration, so a game loses time, not the move. A fixed-depth
  search or `go infinite` never finishes the iteration. CCRL rates with
  tables.

Candidate explanations, none tested yet:
- the tablebase floor and cap at PV nodes against the aspiration window,
  so that the window keeps failing around a tablebase bound;
- tablebase results stored at `depth + 6` cutting where the search's own
  scores disagree, which makes iterations unstable;
- a pruning rule that turns off at decided scores (null move, ProbCut,
  futility near the score band), so that a decided line is searched
  without pruning.

Cheapest next test: in the exploding iteration of root 26 at depth 18,
count the aspiration steps and the nodes per root move in a `diag` build.
Then repeat with in-search probes off (`SyzygyProbeDepth` above the
depth) and the root's tables still loaded. That separates the probe from
the scores it leaves behind.

## D.9.3's diagnosis, 2026-11-25

Research on the explosion above, zero games. Every number is a node count
or a counter of a single-thread search at a fixed depth and node cap, so
it is deterministic and the host's load does not enter it.

### Where the nodes are

The instrument is a `diag` build of the tree at `546adc4` with counters
added for this read only (`instrument_d93.py` beside the outputs; never
committed). `bench 13` reads 3,694,298 / EBF 2.324 with them, and the
searches below print the baseline's node counts line by line, so they
observe and change nothing. The binary is `diag_tbx.exe`, SHA-256
`1ed64375168263e13479cf42ec390d244070145eeb20e537e3a1d3c6e9ee5234`, under
`analysis/artifacts/d93-2026-11-25/` with the traces.

    python analysis/artifacts/d93-2026-11-25/uci_trace.py --exe analysis/artifacts/d93-2026-11-25/diag_tbx.exe --fen "8/6P1/8/3b3P/5B2/3k1pK1/2p5/8 b - - 0 1" --go "depth 18 nodes 30000000" --syzygy D:/chess/tablebases/syzygy3456 --out analysis/artifacts/d93-2026-11-25/r26_tb_d18_diag.txt
    python analysis/artifacts/d93-2026-11-25/uci_trace.py --exe analysis/artifacts/d93-2026-11-25/diag_tbx.exe --fen "8/8/P5bP/6B1/1K1p4/3k4/2p5/8 b - - 0 1" --go "depth 16 nodes 30000000" --syzygy D:/chess/tablebases/syzygy3456 --out analysis/artifacts/d93-2026-11-25/r30_tb_d16_diag.txt

Both exited 0. An interior node is one entry to the node function past
the probe; "inside the tables" means six men or fewer.

| Counter, whole search to the 30M cap | Root 26, depth 18 | Root 30, depth 16 |
|---|---|---|
| Interior nodes | 24,652,416 | 22,962,746 |
| of them inside the tables | 24,287,016 (98.5%) | 22,251,824 (96.9%) |
| of them at a PV node | 204 | 111 |
| Quiescence nodes, and inside the tables | 4,604,650, 4,494,896 | 5,951,282, 5,718,498 |
| In-search probes answered | 1,993,308 | 1,983,003 |
| A win that reached beta, a loss at or below alpha, a draw: returned | 50,022, 524,485, 117,947 | 56,439, 592,225, 95,407 |
| A win below beta or a loss above alpha, off the PV: searched on | 109,673, 1,191,124 | 112,938, 1,125,941 |
| Floor, cap (PV nodes) | 20, 37 | 17, 36 |
| Table cutoffs taken inside the tables | 7,004,886 | 5,973,668 |
| Mean depth left at an interior node inside the tables | 5.06 | 4.87 |

- Root 26 scores a tablebase win (`cp 19987`, the win 13 plies from the
  root) at depths 16 and 17. Depth 18 opens one window, −30,002 to
  +32,001, and that one search does not return in 29.5M nodes. Root 30's
  depths 14 and 15 end on −724 and −849 after thirteen windows each,
  which swing between that score and the loss band, for 4,899 and 6,058
  nodes; depth 16's fifth window, −31,942 to −19,873, does not return.
- So the explosion is one search, not a window that keeps failing, and
  the floor and the cap act at a few dozen nodes. The record's first
  candidate is refuted as the cost. The window's swings on root 30 are
  real and cheap.
- Almost every node is a position the tables already decide. The search
  gets there below a probe that did not end its node: 65% of the answered
  probes are a win below beta or a loss above alpha, which only a window
  inside the tablebase band can produce, since a tablebase score is
  `TB_VALUE − ply` and a win found deeper than the window's own win is
  below its beta. Under the rule-50 option the next quiet move leaves a
  position no probe may answer (the counter is not zero), so the
  six-man position is searched as any other.
- Stockfish 19 on root 26 with the same tables reads the win at depth 17
  and reaches depth 27 at 581,720 nodes and depth 40 at 12,038,944
  (`r26_sf19_tb.txt`). It is a prior only: the donor's design, which has
  the same probe, verdict and counter rule, does not have to explode.

### What differs from Stockfish, read in the source

Two things on this path, and a third at the root.
- **The table keeps a tablebase score as stored** (`tt.rs`,
  `score_to_tt`): a win probed 9 plies from the root reads back as
  `TB_VALUE − 9` at a node 15 plies from it, where a fresh probe says
  `TB_VALUE − 15`. Stockfish stores every decisive score by its distance
  from the node. B.5.2 recorded this as open, "a tablebase value's
  distance in the table is path-relative", after its own change read
  11,189,746 on the classic kernel's bench: there ProbCut and a
  fail-hard quiescence wrote window bounds into the band without tables.
  Whether the compact kernel still does is unread.
- **Late-move pruning is off at a winning beta** (`lmp_prunes`,
  `!is_win(beta)`); Stockfish's has no such guard. Reverse futility and
  the null move are off at a losing beta in both.
- At the root a fail low keeps the depth reduction earlier fail highs
  built (`window_fail_low`), where Stockfish's resets its count. Root
  30's "depth 15" score of −849 comes from a search at depth 7. Noted,
  not part of this read: whether the kernel's own donor resets it was
  not checked.

### The separating read, registered before any variant ran

Three one-rule variants of the same tree, each a `diag` build with the
counters above, none a candidate for the tree as built:
- **V1, the band by ply in the table:** `score_to_tt` and `score_from_tt`
  move a score at or beyond `±TB_WIN_SCORE` by ply, as they move a mate;
  the rule-50 horizon stays on mates alone.
- **V2, an undecided probe ends its node off the PV:** `tb_verdict`
  returns `Return` where it returns `Search`.
- **V3, late-move pruning at a winning beta:** `lmp_prunes` without
  `!is_win(beta)`.

The read: `tools/diag/tb_probe_read.py` with the D.9.2 read's settings on
roots 26, 30 and 11, `--depth 20 --max-nodes 30000000`, the tree as built
and the three variants; then the counters on roots 26 and 30 for each
variant; then `bench 13` for each variant.

Frozen predictions, 2026-11-25:
- The tree as built leaves roots 26 and 30 short and finishes root 11
  (1,903,586 nodes at D.9.1's close).
- V1 finishes all three roots at depth 20 inside the cap, and the share
  of interior nodes inside the tables falls below a half. Confidence
  about 0.6. If it holds, the path-relative distance is the cause: the
  probe and the table disagree about the same position, so a window in
  the band is cut on distances that are not the node's.
- V2 finishes all three, confidence 0.9. It is the accounting check:
  every interior node inside the tables lies below an undecided probe.
- V3 alone leaves root 26 or root 30 short, confidence 0.65.
- `bench 13`: V2 reproduces 3,694,298 exactly, since nothing probes
  without tables. V1 and V3 each change it, confidence 0.6 each: the
  band holds mates' neighbours and window bounds without tables.

Falsifiers:
- The path-relative distance is refuted as the cause if V1 leaves root
  26 or root 30 short of depth 20.
- The pruning guard is refuted as sufficient if V3 leaves one short,
  and as necessary if V1 finishes both.
- The accounting is refuted if V2 leaves one short: then nodes inside
  the tables are reached some other way, and the diagnosis restarts.
- If V1 and V3 each fail and V2 holds, one more variant is allowed, V1
  with V3; nothing else is added after results are seen.

### The separating read's result

The builds are `v0.exe` to `v3.exe` beside the traces
(`build_variants.sh`, `variant_d93.py`; SHA-256 `c848da0a…`, `ebc513c3…`,
`c482f7ae…`, `f258d143…` in `hashes.txt` there).

    python tools/diag/tb_probe_read.py --epd tools/books/ocb_cohort_v1.epd --men 7,8 --per-count 24 --depth 20 --max-nodes 30000000 --only 26,30,11 --syzygy D:/chess/tablebases/syzygy3456 --out analysis/artifacts/d93-2026-11-25/read_variants.json v0=analysis/artifacts/d93-2026-11-25/v0.exe v1=analysis/artifacts/d93-2026-11-25/v1.exe v2=analysis/artifacts/d93-2026-11-25/v2.exe v3=analysis/artifacts/d93-2026-11-25/v3.exe

It exited 0. Depth reached inside the 30M cap, and the nodes of the last
finished iteration:

| Root | As built | V1, band by ply | V2, probe ends the node | V3, pruning at a winning beta |
|---|---|---|---|---|
| 26 | 17, short | 17, short | **20** at 645,030 | **20** at 600,181 |
| 30 | 15, short | 14, short | 16, short | 15, short |
| 11 | **20** at 1,903,586 | 18, short | 19, short | 17, short |
| `bench 13` | 3,694,298 | 3,694,298 | 3,694,298 | 3,293,080 / EBF 2.318 |

The counters (`counters.py`, exit 0, `counters_variants.txt`) under V2:

| Under V2, to the cap | Root 30 | Root 11 |
|---|---|---|
| The window that does not return | depth 17, −31,953 to −18,542 | depth 20, 1,500 to 31,894 |
| Interior nodes, and inside the tables | 17,044,339, 1,284 | 19,877,916, 527 |
| Interior nodes off the PV with beta in the win band | 7,097,726 | 3,108,702 |
| the same with beta in the loss band | 8,005,975 | 5,443,213 |

A census of the moves searched (`census.txt`, the tree as built on root
26 and V2 on root 30; `bench 13` exact with it):

| Moves searched at nodes off the PV | Root 26, as built | Root 30, V2 |
|---|---|---|
| Beta in the win band: nodes, moves | 7,913,155, 8,254,777 | 7,097,726, 17,221,000 |
| Beta in the loss band: nodes, moves | 7,240,388, 8,538,160 | 8,005,975, 7,421,609 |
| of those moves, the node's first | 4,223,364 | 4,848,886 |
| of those moves, later ones searched with the best score still a loss | 4,314,796 | 2,572,723 |
| of those moves, at a node in check | 4,450,200 | 4,537,745 |

Priors from two other engines on the same roots and tables, one thread;
they are not evidence about Rarog:
- Stockfish 19: root 26 at depth 27 by 581,720 nodes; root 30 at depth
  35 by 11,625,172; root 11 at depth 40 by 1,687,186.
- Reckless 0.9.0, the binary only: root 26 at depth 21 by 1,400,863;
  root 30 at depth 23 by 10,794,134; root 11 at depth 22 by 2,741,570.

**Reading.**
- The path-relative distance is refuted as the cause: V1 leaves all
  three roots short, root 11 among them, which the tree as built
  finishes.
- The accounting holds and its conclusion does not. With V2 the search
  inside the tables is gone (a few hundred to a few thousand nodes), and
  roots 30 and 11 still do not finish: the nodes move to seven- and
  eight-man positions searched in a window whose beta is in the decisive
  band. The unit of cost is therefore the **window in the decisive
  band**, on either side of the tables' edge, not the position inside
  the tables.
- In such a window nothing static can decide a node: no evaluation
  reaches the band, reverse futility and the null move are off at a
  losing beta, late-move pruning is off at a winning one, and a node
  whose first move returns a loss searches every other move with the
  move-loop pruning off, which is half or more of the later moves there.
  Half of the loss-band moves come from nodes in check. The search has
  become a distance proof, with a mate search's width.
- V2 and V3 each finish root 26 and neither finishes the other two.
- Both donors pass these roots at these depths, so the width is Rarog's:
  in its port, or in the tablebase code that is its own.

**Calibration**, appended; the predictions above stand as written.
- V1, predicted to finish all three at 0.6: wrong in sign and mechanism.
  A window in the band is not cut on wrong distances; it is not cut at
  all.
- V1's `bench 13`, predicted to change at 0.6: wrong. It is exact, so
  B.5.2's reason for keeping band values as stored, a changed search
  without tables, does not show on the compact kernel's bench. That is
  one bench, not a proof of neutrality, and V1 made root 11 worse.
- V2, predicted to finish all three at 0.9: wrong, and overconfident.
  The accounting it tested held; the miss was the assumption that the
  cost belongs to positions inside the tables.
- V3, predicted to leave a root short at 0.65, and its bench to change:
  both held.
- The extra variant the registration allowed (V1 with V3) had the
  condition "V2 holds"; it did not fire and was not run.

### One more variant, registered 2026-11-25 before it ran

**V2 with V3**, on the same three roots with the same read, the counters
and `bench 13`. It asks whether the two rules that each finish root 26
are together enough, which decides whether a repair can be the two of
them or needs a third mechanism found first.

Frozen predictions:
- root 26 finishes depth 20, confidence 0.9;
- roots 30 and 11 both finish, confidence 0.35: each rule alone left
  them further from the depth than V2 did, and the census points at the
  loss-band nodes, which neither rule touches;
- `bench 13` reads V3's 3,293,080, since V2 changes nothing without
  tables.

Falsifier and stop: if root 30 or root 11 stays short, the two rules are
not the repair, and no further variant is run in this read. The next
step is then the census's lead, as a new registration: what the donors
do at a node off the PV whose best score is still a loss, and at one in
check, when beta is in the decisive band.

### V2 with V3: the result

`v23c.exe` (SHA-256 `5da58490…`), the same read and counters, both exit 0
(`read_v23.json`, `counters_v23.txt`).

| Root | V2 with V3 |
|---|---|
| 26 | **20** at 1,081,830 |
| 11 | **20** at 4,905,994, the last window searched at depth 10 after fail highs |
| 30 | 19 (`cp -19980`), short at depth 20 in a fully open window |
| `bench 13` | 3,293,080 / EBF 2.318 |

On root 30 to the cap: 17,084,163 interior nodes, 576 inside the tables;
6,647,104 with beta in the win band and 6,560,836 with it in the loss
band. Of 7,489,756 moves searched at the loss-band nodes, 4,310,040 are a
node's first, the other 3,179,716 were all searched with the best score
still a loss, and 5,910,005 (79%) come from nodes in check.

**Verdict:** the falsifier fired. The two rules together finish roots 26
and 11 and move root 30 from depth 15 to depth 19, and they are not the
repair. By the stop rule no further variant ran.

**Calibration.** Root 26 finishing (0.9) held. Both other roots finishing
(0.35) did not happen, as the larger share of that prediction expected;
`bench 13` held.

## D.9 from here: the decision, 2026-11-25

Evidence layer throughout: node counts and counters at fixed depth, one
thread, three roots of one opposite-bishop cohort, and two foreign
binaries as priors. None of it is Elo, and no game was played.

**Is the explosion a defect or an artefact of fixed-depth reads?** A
defect in Rarog's search, which a fixed depth shows and does not cause.
- Established: its unit is a window off the PV whose beta is in the
  decisive band. Tables are what put a seven- or eight-man root's window
  there, so it appears with tables loaded. Once there, Rarog searches a
  distance proof at close to a mate search's width, inside the tables
  below probes that do not decide such a window, and outside them.
- Refuted: the floor and cap against the aspiration window as the cost
  (one window, a few dozen firings); the table's path-relative distances
  as the cause (V1); either rule alone, and both together, as the repair
  (V2, V3, V2 with V3).
- Not yet known: which of Rarog's rules make the band window wider than
  Stockfish's and Reckless's, both of which pass these roots at these
  depths; how often it occurs in games; what it costs in Elo.

**Does it matter where CCRL rates?** It is active there and in none of
Rarog's pools, because CCRL loads tables and the pools do not. What it
can cost is clock: the iteration is abandoned at the hard limit, and the
move played is the last finished iteration's. On root 26 that move's
score is already a tablebase win, so the result is not at risk there;
roots 30 and 11 explode at scores of −849 and +1,456, which the tables
have not decided. RAR-S93 and RAR-S94 played with tables and recorded no
time loss from this. The expected strength effect is therefore small and
unmeasured, and Basilisk's related repair read +0.3 ± 3.2. The time it
costs under a clock has never been read; the read below is registered
for that.

**Does it matter for analysis?** Yes. With tables, `go infinite` and a
fixed depth stop advancing on two of 48 roots by depth 20 (three with
D.9.2's reorder), for tens of millions of nodes.

**Order.** D.9.3 stays ahead of D.9.2, as research with a stop rule, and
D.9.2 stays held on it.
- D.9.2 does not close on its own value. Its measured value is 51.9%
  fewer table reads at a strength prior of zero, and its registered read
  was falsified by its own amendment. Reading it again under a rule that
  tolerates a short root would be an acceptance rule chosen after the
  result.
- D.9.2 reopens when D.9.3 lands a repair: the same registered read runs
  again, unchanged. If D.9.3 closes `NO_CHANGE`, D.9.2 closes `NO_CHANGE`
  with it. Its trigger then: a run with tables in which table reads are
  shown to cost time (a cold read past `Move Overhead`, or a node rate
  that falls with `tbhits`).
- I recommend against implementing a repair now. The two rules found are
  not sufficient, one of them (late-move pruning at a winning beta)
  changes the search without tables (`bench 13` 3,293,080), and a change
  to every game's search needs a strength gate that a tablebase defect
  with a near-zero strength prior does not justify on its own.

**D.9.3's next steps and its stop rule.**
1. *The rule table* (no host needed, zero runs): every rule in
   `compact/rules.rs` and `compact/node.rs` that reads `is_win`,
   `is_loss` or `is_decisive`, beside Stockfish's form of the same rule
   (`search.cpp`: the guards at steps 6 to 11, the move loop's
   `!is_loss(bestValue)` gate, late-move and futility pruning, the
   reductions' `!is_decisive(alpha)`, quiescence). Stockfish is the
   primary donor; Reckless teaches ideas only, and its binary is a
   prior. D.7.3's tracing twin would show the kernel's donor's own
   decisions in a band window, but its patches are among those D.11
   holds, so it is used only on the maintainer's word. The census
   gives the order to read in: the node off the PV
   with a losing beta whose best score is still a loss, the node in
   check, then quiescence. Frozen prediction: the table shows at least
   one more difference than V3's guard, confidence 0.7. Refuted if it
   shows none; then the width is not in a decisive-score rule, and the
   next place to look is what the kernel adds to its donor (the line
   bound, the proven-win rule, the root window keeping its reduction
   after a fail low).
2. *One more variant read*, registered with its own predictions before
   it runs: one variant per difference the table finds, on roots 26, 30
   and 11 with this read's instrument, cap and counters, and `bench 13`
   for each. Variants are built from an exported copy, as here, and
   never committed.
3. *The clock read* below (needs an idle host).
4. *Stop rule.* If step 2 finds a set of rules that finishes the three
   roots, the leaf writes the repair's registration: a part that leaves
   `bench 13` exact is a repair of case 1 with D.9.2's read on all 48
   roots as its activation and correctness read, every root finishing
   depth 20 in both builds; a part that changes `bench 13` is a playing
   change in every configuration and goes to the maintainer as a gate to
   decide, registered on its own. If step 2 does not find one, D.9.3
   closes `NO_CHANGE` with the triggers below and no third round.

**The clock read, registered 2026-11-25, not run.** It answers what the
defect costs in the rated configuration, which decides whether a repair
or a containment is worth a gate.
- Instrument: `tools/diag/tb_probe_read.py`, extended to take the `go`
  arguments and to record the time of the last `info` line and the wall
  time to `bestmove` (a tooling change with its own test).
- Roots and settings: D.9.2's 48 roots, a fresh process each,
  `Threads 1`, `Hash 64`, `go wtime 120000 btime 120000 winc 1000 binc
  1000` (CCRL Blitz's control at its first move), one arm with the local
  tables and one without, one default build of the committed tree,
  archived with its hash.
- Reading: per root, the time used in each arm and their ratio; the
  number of roots that use at least twice the time with tables.
- Frozen prediction: 3 to 12 of the 48 roots use at least twice the time
  with tables, roots 26 and 30 among them; the median ratio over the
  rest is 0.8 to 1.25; every search answers inside its own hard limit.
  Confidence 0.5: the roots are known to depth 20 only, and a clock
  reaches deeper.
- Falsifier: no root uses twice the time. The defect is then confined to
  searches without a clock, and D.9.3 closes `NO_CHANGE` for the rated
  configuration with the analysis trigger alone.
- It is a timed read: an idle host, by `docs/rules/measurement.md`.

**A containment, recorded and not proposed.** At a DTZ root the hard
limit is already the soft one, because the result is decided. The same
rule for a search whose last finished score is in the tablebase band
would bound root 26's cost under a clock, leave `bench 13` exact and
do nothing for analysis or for roots 30 and 11. It is worth registering
only if the clock read shows the cost and step 2 finds no repair.

**Triggers if D.9.3 closes `NO_CHANGE`:** a hard-limit stop or a time
loss in a run with tables at a root of seven men or more; a user's
report of a stalled depth with tables; a rated pool or a release
qualification that adopts tables.

**What else D.9 owes before it closes.**
- Nothing further on the policy, D.9.1 or the `NO_CHANGE` items: none of
  their triggers fired in today's reads (no time loss, no crash, no
  move changed by a cursed score or a stored score past the fifty-move
  limit).
- TT-11's neighbour, found today: moving band values by ply leaves
  `bench 13` exact on the compact kernel, so B.5.2's obstacle is gone.
  It is still `NO_CHANGE`: it repaired nothing here and left root 11
  short. Its trigger is TT-11's.
- `shakmaty-syzygy` stays not ported. No trigger fired, and its probe
  and conversion cost stays unmeasured, since measuring it needs the
  crate downloaded, which is the maintainer's call.
- The recommendation outside D.9, RAR-S93's activation read with tables
  in the release qualification, has no PLAN step yet. It needs the
  maintainer's yes or no before D.9 closes, and `tb_activation.py`'s
  budget formula brought to `search/time.rs` if yes.
- At the close: D.9's parent marked, the board regenerated, and `d9`'s
  pull request, which is the maintainer's.

## Corrections, the same day

A second review of D.9 (2026-11-25), checked against the code and the
records, found these. They are corrected above:

- The PROTO-9 repair named the wrong gate: `shared.syzygy.largest` is 0
  at a DTZ root.
- The PROTO-10 trigger's repair named `tb_probe_root_dtz`, which ranks
  clean wins equally.
- The `go movetime` start rule was missing. (Withdrawn, last item.)
- B.5.2's probe stress test was not cited.
- The post-probe poll was missing.
- `tb_activation.py`'s budget formula is stale.
- `configure`'s re-send of every option was not noted.
- Implementing D.9.1 showed that the `go movetime` start rule added above
  rests on a false premise. `search/time.rs` and its test
  `the_tablebase_extension_is_unbounded_without_a_clock_and_boxed_under_movetime`
  hold that `go movetime` has no clock to protect, so the box alone
  applies; the movetime branch's comment records that `go movetime` never
  forfeited and that harnesses tolerate about 10% past the nominal time.
  A start rule of ten overheads would skip the extension at every
  `go movetime` of 100 ms or less, and Stockfish does not limit its
  extension under `go movetime` at all. The 14 ms measured on
  `go movetime 10` is the box working. The item closes `NO_CHANGE`, its
  trigger under PROTO-10 above; the other six repairs closed D.9.1 on
  2026-11-25.
