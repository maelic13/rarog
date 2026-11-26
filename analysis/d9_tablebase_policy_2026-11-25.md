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

## D.9.3's rule table, 2026-11-25

Step 1 of *D.9 from here*, zero runs. Every rule of the compact kernel that
reads a decisive score, or that acts at a node in check before the moves,
beside Stockfish's form of it. Stockfish is read at the `sf_19` tag, the
release whose binary passed the three roots (`src/search.cpp` there), in
the order the census points: the node off the PV with a decisive beta, the
node in check, quiescence.

| Rule | Rarog | Stockfish 19 | Differs |
|---|---|---|---|
| Mate distance bounds | `node.rs:141` | 797–798 | no |
| In-search probe and its verdict | `node.rs:155`, `tb_verdict` | 923–972 | no |
| TT cutoff | `rules.rs:1313`, refuses at a counter of 90 | 893, refuses at 96 | the counter only |
| Razoring | `rules.rs:1412`: not in check, **`alpha < 827`**, the TT move not quiet, no lower bound | 991: no guard but the PV | **yes, W1** |
| Reverse futility | `rules.rs:1461`: `!is_loss(beta)`, `!is_win(estimate)` | 996: `!is_loss(beta)`, `!is_win(eval)` | no |
| Null move, gate and result | `rules.rs:1512`, `:1576`: `!is_loss(beta)`; a decisive result fails | 1011: `beta >= -2000`; a win does not return | no in a band window |
| Everything before the moves, in check | razoring, reverse futility and the null move ask `!in_check`; **ProbCut does not**, and in check its `eval` is `VALUE_NONE` (32,002), so its "`eval >= beta`" holds with no stored score, and its SEE bar, `probcut_beta − eval`, admits every capture | 975: `goto moves_loop` | **yes, W2** |
| ProbCut's gate | `rules.rs:1619`: **`!is_win(beta)`**, so it runs at a losing beta | 1058: `!is_decisive(beta)`, `depth >= 3` | **yes, W3** |
| ProbCut's value | `rules.rs:1668`: a decisive score returns | 1092: stored, not returned | yes; Rarog prunes more, no variant |
| Small ProbCut | absent | 1100–1103: `!is_decisive(beta)` | off in a band window anyway |
| The move loop's pruning gate | `node.rs:790`: `!is_loss(best_score)` | 1166: the same, with non-pawn material | no in a band window |
| Late-move pruning | `rules.rs:1833`: `!is_win(beta)` | none | yes, V3 (read) |
| Quiet futility and its value | `rules.rs:1862`, `node.rs:849`: `!is_decisive(best_score)` | 1218–1221: the same, with `!is_win(futilityValue)` | no |
| SEE and history pruning | not in check | SEE also in check | Rarog prunes less in check, only where the gate is open |
| Singular extension, multi-cut | `rules.rs:1684`, `:1767`: `!is_decisive` | 1247, 1277: the same; singular off under `seekMate` | no |
| Late-move reduction at a winning beta | `rules.rs:1992`: one ply more | none | yes; Rarog reduces more, no variant |
| Late-move reduction, alpha's gap to the estimate | `rules.rs:1996`: no guard, clamped to −0.21 and +0.30 of a ply; in check the estimate is `VALUE_NONE`, so the gap is at its floor | 1354: `!is_decisive(alpha)`; in check `eval` is the static evaluation two plies back | **yes, W4** |
| Quiescence: the TT, stand pat, softening, pruning | `rules.rs:2286`–`2323`, `quiescence.rs:115`–`120`: `!is_decisive` at PV nodes, `best_score > −TB_WIN_SCORE` | 1733, 1787–1790, 1871: the same forms | no |
| Root fail high | `rules.rs:2430`: reduction held at 1 on a decisive score | resets per window | not in a node |

**Reading.** Four differences can widen a search in a band window, and
three of them are new:
- **W1, razoring's alpha limit.** A node off the PV with a winning beta
  has alpha in the win band, far above any evaluation. Stockfish razors
  it to quiescence at every depth where `eval < alpha − 482·depth²`, which
  at a band alpha is every depth up to about six. Rarog never razors it,
  because alpha is above 827. Such nodes are the children of every
  loss-band node, whose moves the census counts as searched with the gate
  closed; under V2 with V3 they carry 13.7M of root 30's moves.
- **W2, ProbCut in check.** At an expected cut node in check without a
  stored score, Rarog searches every legal capture to quiescence and then
  at reduced depth before the moves. Stockfish skips this. Of the
  loss-band moves under V2 with V3, 79% are at nodes in check.
- **W3, ProbCut at a losing beta.** A stored ordinary score clears a
  loss-band `probcut_beta`, so the gate opens and every capture is
  searched against a window in the band.
- **W4, the alpha-gap reduction at a decisive alpha and in check.** It is
  a fifth of a ply at most, so it is the least likely of the four.

The frozen prediction of step 1 (at least one difference beyond V3's,
0.7) holds: there are four.

## D.9.3's second variant read, registered 2026-11-25 before it ran

Step 2 of *D.9 from here*. One variant per difference, each the form
Stockfish has, and their union, registered together before any is built:
- **W1:** `razors` without `alpha < razor_alpha_limit`.
- **W2:** `probcut_gate` false in check.
- **W3:** `probcut_gate` with `!is_decisive(beta)` for `!is_win(beta)`.
- **W4:** the alpha-gap term of `late_move_reduction` zero at a decisive
  alpha.
- **WU:** W1 to W4 with V3.

Each is a `diag` build of an exported copy of `546adc4` with the counters
and the census, built by this read's copy of `build_variants.sh` and
`variant_d93.py`, never committed. The read is the first one's:
`tools/diag/tb_probe_read.py` on roots 26, 30 and 11 at depth 20 with a
30M cap, against the census baseline `v0c.exe`; then `counters.py` on the
three roots for each variant; then `bench 13` for each variant.

Frozen predictions:
- W1 finishes root 26, 0.7, and all three roots, 0.4. Its `bench 13`
  changes, 0.85, since alphas above 827 occur without tables.
- W2 finishes all three, 0.1. It changes `bench 13`, 0.8.
- W3 finishes all three, 0.1. Its `bench 13` is exact, 0.55, since a
  losing beta needs a mate in view without tables.
- W4 finishes all three, 0.05. Its `bench 13` changes, 0.5.
- WU finishes all three, 0.55. Its `bench 13` changes: V3 alone does.
- Under W1, the moves searched at win-band nodes off the PV fall by more
  than half on root 30, 0.6.

Falsifiers and the stop rule, from *D.9 from here*, applied as written:
- If no variant, WU included, finishes all three roots, the width is not
  in these rules. D.9.3 then closes `NO_CHANGE` on the record's triggers,
  with no third round.
- If a variant that leaves `bench 13` exact finishes all three, it is a
  repair of case 1 and its registration is written, with D.9.2's read on
  all 48 roots as its activation and correctness read.
- If only variants that change `bench 13` finish all three, the smallest
  such set goes to the maintainer as a gate to decide, registered on its
  own. The finding stands either way; no game is played here.

### The second variant read's result

The builds, each a `diag` build of an exported copy of `546adc4` with the
counters and the census, are under `analysis/artifacts/d93w-2026-11-25/`
with the scripts that made them. Each variant is the one-line change the
registration names (W1 drops the conjunct `i.alpha < s.razor_alpha_limit`
from `razors`; W2 puts `!in_check &&` before `probcut_gate`; W3 reads
`!is_decisive(i.beta)` for `!is_win(i.beta)` in `probcut_gate`; W4
multiplies the alpha-gap term by `i32::from(!is_decisive(i.alpha))`), and
its `bench 13` is its fingerprint:

| Build | SHA-256 | `bench 13` / EBF |
|---|---|---|
| `v0c.exe`, the census baseline (the first read's) | `ec272807…` | 3,694,298 / 2.324 |
| `w1c.exe`, razoring without its alpha limit | `744bd49b…` | 3,097,711 / 2.315 |
| `w2c.exe`, no ProbCut in check | `ca41c292…` | 3,650,025 / 2.320 |
| `w3c.exe`, no ProbCut at a decisive beta | `1913f2d8…` | 3,351,026 / 2.319 |
| `w4c.exe`, no alpha-gap term at a decisive alpha | `bba8b861…` | 3,243,195 / 2.319 |
| `wuc.exe`, W1 to W4 with V3 | `af49b640…` | 3,114,639 / 2.303 |

    python tools/diag/tb_probe_read.py --epd tools/books/ocb_cohort_v1.epd --men 7,8 --per-count 24 --depth 20 --max-nodes 30000000 --only 26,30,11 --syzygy D:/chess/tablebases/syzygy3456 --out analysis/artifacts/d93w-2026-11-25/read_w.json v0c=analysis/artifacts/d93-2026-11-25/v0c.exe w1c=analysis/artifacts/d93w-2026-11-25/w1c.exe w2c=analysis/artifacts/d93w-2026-11-25/w2c.exe w3c=analysis/artifacts/d93w-2026-11-25/w3c.exe w4c=analysis/artifacts/d93w-2026-11-25/w4c.exe wuc=analysis/artifacts/d93w-2026-11-25/wuc.exe

It exited 0, as did `counters.py` for the five variants and for the
baseline (`counters_w.txt`, `counters_v0c.txt`). The read ran on the tool
as extended for the clock read, which takes a root's depth from its last
exact `info` line; Rarog prints a bound line only past 10M nodes, and no
root counted complete in this read or an earlier one passed 2.2M, so the
rule changes no count. Depth reached inside the 30M cap, and the nodes of
the last finished iteration:

| Root | Baseline | W1 | W2 | W3 | W4 | WU |
|---|---|---|---|---|---|---|
| 26 | 17, short | **20** at 510,630 | **20** at 2,171,167 | **20** at 1,112,438 | **20** at 736,222 | **20** at 833,039 |
| 30 | 15, short | 15, short | 18, short | 14, short | 14, short | 16, short |
| 11 | **20** at 1,903,586 | 17, short | 19, short | 17, short | 18, short | 19, short |

The census to the cap, moves searched at nodes off the PV:

| Root 30 | Baseline | W1 | WU |
|---|---|---|---|
| Interior nodes, and inside the tables | 22,962,746, 22,251,824 | 21,079,803, 20,321,713 | 21,372,376, 20,632,521 |
| Moves with beta in the win band | 8,086,242 | 4,126,876 | 3,786,067 |
| Moves with beta in the loss band, and of them with the gate closed | 7,567,992, 3,497,209 | 2,804,492, 289,572 | 2,433,723, 90,307 |

**Reading.**
- No variant finishes all three roots, WU included. By the stop rule the
  repair search ends here, with no third round.
- What the four differences do is real and does not reach the cost.
  Razoring at a band alpha halves the moves at win-band nodes on root 30
  and removes nine tenths of the loss-band moves searched with the gate
  closed. WU removes 97% of those. Yet the interior nodes stay at the cap,
  and 96% of them are still inside the tables: the width the census
  measured is gone, and the search inside the tables below undecided
  probes, V2's mechanism, remains. WU did not include V2, because V2 is not
  a difference from Stockfish.
- The three roots do not rank variants. Across the nine variant builds
  read (V1, V2, V3, V2 with V3, W1 to W4, WU), eight finish root 26, one
  finishes root 11, which the baseline finishes, and none finishes root
  30. The roots were chosen because they exploded, so any change to the
  tree moves where an explosion falls. "Finishes all three" is therefore
  a strict and noisy criterion, not a measure of how much of the cost a
  rule removes.

**Calibration**, appended; the predictions above stand as written.
- W1 finishing root 26 (0.7) held, as it did for every variant but V1.
  W1 finishing all three (0.4) did not happen. Its `bench 13` changed
  (0.85), held.
- W2, W3 and W4 each finishing all three (0.1, 0.1, 0.05) did not happen.
- W3's `bench 13` exact (0.55) was wrong: 3,351,026. A losing beta occurs
  in the bench without tables far more often than a mate in view would
  suggest. W2's change (0.8) and W4's (0.5) both happened.
- WU finishing all three (0.55) was wrong, and overconfident. The miss is
  in mechanism: the census's width, which WU removes, was taken for the
  whole cost, and the nodes inside the tables are a second part that none
  of the differences touches.
- W1 halving root 30's win-band moves (0.6) missed narrowly: −49.0%.

**Decision, 2026-11-25.**
- D.9.3's repair search closes `NO_CHANGE` by its stop rule.
- What is established stands: the explosion is a defect of Rarog's
  search in band windows, with tables. The four rules above and V2 are
  its contributing parts, and no single rule is its cause.
- What is not known: whether WU with V2 repairs it. That is a natural
  next variant, read after the results, so the stop rule does not let it
  run in this round. It is the first thing to try if the leaf reopens.
- None of the parts is a case-1 repair: every one but V2 changes
  `bench 13`. A repair of the defect is therefore a playing change in
  every configuration, and needs a strength gate that a near-zero prior
  does not justify on this evidence alone.
- The clock read stays, because it now has a use: it says whether the
  defect costs time where CCRL rates. If no root uses twice the time
  with tables, the leaf closes `NO_CHANGE` for the rated configuration,
  with the analysis trigger alone. If roots do, the defect has a measured
  cost in the rated configuration. That fires the reopen trigger, and
  the leaf registers its next step from there: the containment
  (*D.9 from here*) for the roots already decided, and WU with V2 as the
  repair candidate, with a gate for the maintainer to decide.
- D.9.2 closes `NO_CHANGE` with the repair search, by its own rule. Its
  triggers: a repair of the band-window explosion lands (then its
  registered read runs again, unchanged), or a run with tables shows
  table reads costing time.
- Reopen triggers for D.9.3's repair, beyond those in *D.9 from here*:
  the clock read finding roots at twice the time; a pool or release
  qualification that adopts tables.

### The clock read's command, prepared 2026-11-25

The registered read (*D.9 from here*), on a default build of the
committed tree. `clock.exe` is built from `bb4cf045`, which adds the
tool's clock support, and its engine source is the same as `546adc4`'s.
`id name` is `Rarog dev-20261125-bb4cf045`, SHA-256
`972c8a3855054bb8c02d298177732f279e9b3ae7d970bfd55527c7ba1b37f129`,
`bench 13` 3,694,298 / EBF 2.324. The arm without tables is first, so the
summary's `tb` entry lists the roots that use at least twice the time
with tables (`roots_at_least_twice`), gives the median ratio
(`median_wall_ratio`), and each arm's longest search (`max_wall_ms`):

    python tools/diag/tb_probe_read.py --epd tools/books/ocb_cohort_v1.epd --men 7,8 --per-count 24 --go "wtime 120000 btime 120000 winc 1000 binc 1000" --syzygy D:/chess/tablebases/syzygy3456 --tables-off notb --out analysis/artifacts/d93w-2026-11-25/read_clock.json notb=analysis/artifacts/d93w-2026-11-25/clock.exe tb=analysis/artifacts/d93w-2026-11-25/clock.exe

It is a timed read and needs an idle host. Its length is unmeasured: 96
searches at the first move of a 2+1 clock.

### The clock read's result, 2026-11-25

The maintainer ran the prepared command on an idle host; the output is
`read_clock.json` beside the binary. Evidence layer: wall time from `go`
to `bestmove` per root, one thread, the first move of a 2+1 clock, the 48
roots of one opposite-bishop cohort. These are not games and not Elo.

All 96 searches answered. At this clock the hard limit is 88,099.7 ms:
`HARD_SCALE` 0.7281 of the clock less Move Overhead, plus 0.75 of the
increment, less 15 ms. Five searches across four roots ended at it, each
spending 73% of the clock on one move (corrected 2026-11-26 from "Four
searches", which counted the four roots and missed root 30's two arms):

| Root | Men | Without tables | With tables |
|---|---|---|---|
| 11 | 7 | 3,360 ms, depth 22, `cp 697` | **88,100 ms**, last finished depth 23 at 554 ms, `cp 5929` |
| 14 | 7 | 5,165 ms, depth 33, `cp -36` | **88,100 ms**, last finished depth 31 at 1,287 ms, `cp -171` |
| 26 | 8 | 1,148 ms, depth 17, `cp 1329` | **88,101 ms**, depth 17, `cp 19987` |
| 30 | 8 | **88,100 ms**, last finished depth 15 at 178 ms, `cp -385` | **88,100 ms**, last finished depth 15 at 322 ms, `cp -849` |

- On the other 44 roots the two arms cost the same: 61.4 s of wall time
  with tables against 61.8 s without. No other root takes twice the time
  (the most is 1.69, root 34). Roots 20, 21 and 29 take at most half.
- Root 30 without tables was read again at depth 20 with a 30M-node cap
  (`r30_notb_d20.json`). It finishes depth 15 at 495,516 nodes, as under
  the clock, and depth 16 does not finish in 30M nodes. **The explosion
  does not need tables.** Root 30 has it in the configuration every
  Rarog pool plays.
- Depth 20 does not reach root 14's stall (depth 32) or root 11's (depth
  24). That is why the fixed-depth reads never showed root 14 and showed
  root 11 only under some variants.

**Against the registration.**
- Roots that take at least twice the time with tables: three (11, 14 and
  26), inside the predicted 3 to 12. Held.
- Roots 26 and 30 among them: 26 is, 30 is not. Root 30 ends at the hard
  limit in both arms, a ratio of 1.00. **Missed.**
- The median ratio over the 45 other roots: 1.07, inside 0.8 to 1.25.
  Held.
- Every search answers inside its hard limit: held, to the node poll.
  The longest search, 88,101.1 ms, is 1.4 ms past the computed limit,
  against a ceiling of 119,980 ms. No search lost on time.
- The falsifier, no root at twice the time, did not fire.
- Calibration: the prediction carried confidence 0.5 as a whole, and it
  failed on one clause. The miss was taking root 30's explosion for a
  defect of the tables, when it had been read only with them.

**Decision, 2026-11-25.**
- The reopen trigger fired: three roots take twice the time with tables.
  The `NO_CHANGE` trigger for a hard-limit stop with tables at a root of
  seven men or more fired with it. D.9.3's repair reopens.
- The scope widens. Root 30 shows the explosion without tables, so the
  defect is in Rarog's search in every configuration; tables make it
  more common.
- The containment would cover one of the five stops: root 26, whose
  finished score is the tablebase win. Roots 11, 14 and 30 finish on
  ordinary scores. I recommend against making it the next step: it would
  leave four of the five stops, and the repair candidate's read is
  cheaper to answer first. The containment stays the fallback, a case-1
  change for decided roots only, if the candidate fails.
- Next: the repair candidate's read below.

### The repair candidate's read, registered 2026-11-25 before it ran

- **Candidate:** WU with V2, that is W1 to W4, V3 and V2, applied by
  `variant_d93.py` to an export of `1ed64e6` (engine source the same as
  `546adc4`'s). It is a default release build and is never committed:
  `cand.exe`, `id name` `Rarog dev-20261125-nogit`, SHA-256
  `1c7b82ec293c2a38148659b1345e7478a42a0ab98bc4751ccdc2eb02da7cceaf`,
  build log `cand_build.log`. Its `bench 13` was read when it was built,
  before this registration, so it is not a prediction: 3,114,639 / EBF
  2.303. That is WU's count, since V2 acts only at a probe.
- **Instrument:** the clock read's, unchanged, on all 48 roots, with the
  candidate in both arms. All 48 roots, because at a fixed depth eight
  of nine variants moved the explosion to another root rather than end
  it, and only the rest of the roots can show a moved stall. It is a
  timed read, so it needs an idle host.
- **Reading:** per arm, the searches that end at the hard limit (more
  than 88,000 ms of wall time), against the baseline's four.
- **Frozen prediction:**
  - With tables, no search ends at the hard limit: confidence 0.4.
  - Without tables, root 30 still ends at it: 0.5. V2 does not act
    there, and none of W1 to W4 finished root 30 at a fixed depth with
    tables.
  - None of the 44 roots that the baseline finished in both arms ends at
    the limit in either arm: 0.6.
- **Decision rule:**
  - No search at the limit with tables: the candidate becomes the
    repair's registration. It moves `bench 13` and changes shared search
    signals in every configuration, which makes it case 3 of
    `docs/rules/gating.md` (`[-5,5]`, about 12,000 games) unless the
    maintainer decides otherwise. It is registered on its own, with its
    exposure, and the gate is the maintainer's to run.
  - A stop at the limit without tables only: the same, with that stop
    recorded open in D.9.3 as its own finding.
  - Any stop at the limit with tables refutes the candidate at this
    read. The next step then becomes a design question (`R3`): why
    Rarog's search inside the tables, and at root 30 without them, is
    wider than its donors'. The containment is the fallback for the
    decided roots, and leaves `bench 13` exact.
- **Command,** about 4 minutes if no search stalls and about 1.5 minutes
  more for each stall:

    python tools/diag/tb_probe_read.py --epd tools/books/ocb_cohort_v1.epd --men 7,8 --per-count 24 --go "wtime 120000 btime 120000 winc 1000 binc 1000" --syzygy D:/chess/tablebases/syzygy3456 --tables-off notb --out analysis/artifacts/d93w-2026-11-25/read_cand.json notb=analysis/artifacts/d93w-2026-11-25/cand.exe tb=analysis/artifacts/d93w-2026-11-25/cand.exe

### The repair candidate's read: the result, 2026-11-26

The maintainer ran the registered command on an idle host; the output is
`read_cand.json` beside `cand.exe`. The evidence layer is the clock
read's: wall time per root, one thread, the first move of a 2+1 clock,
zero games.

All 96 searches answered. Four searches across three roots ended at the
hard limit, against the baseline's five across four roots, and not on the
same roots:

| Root | Arm | Baseline | Candidate |
|---|---|---|---|
| 11 | with tables | **88,100 ms**, depth 23 | 12,605 ms, depth 18, `cp 1144` |
| 14 | with tables | **88,100 ms**, depth 31 | 1,837 ms, depth 29 |
| 14 | without tables | 5,165 ms, depth 33 | **88,100 ms**, last finished depth 31 at 4,493 ms |
| 26 | with tables | **88,101 ms**, depth 17, the win | **88,105 ms**, depth 21, the win (`cp 19984`) |
| 30 | with tables | **88,100 ms**, depth 15 | **88,100 ms**, last finished depth 16 at 345 ms |
| 30 | without tables | **88,100 ms**, depth 15 | **88,100 ms**, last finished depth 12 at 799 ms |

- With tables, the candidate ends roots 11 and 14 and leaves roots 26
  and 30 at the limit. Without tables, it moves a stall onto root 14,
  which the baseline finished in 5.2 s.
- On the 44 roots that the baseline finished in both arms, the candidate
  took 62.4 s with tables (baseline 61.4 s) and 79.5 s without tables
  (baseline 61.8 s). No score class changed against the baseline with
  tables; 23 of the 48 best moves with tables differ. These are
  descriptions only: neither number is a strength measure.
- The longest search, 88,104.9 ms, is 5.2 ms past the computed limit,
  against a ceiling of 119,980 ms. No search lost on time.

**Against the registration.**
- With tables, no search ends at the hard limit (0.4): **wrong**. Roots
  26 and 30 do.
- Without tables, root 30 still ends at the limit (0.5): held.
- None of the 44 roots that the baseline finished in both arms ends at
  the limit (0.6): held. The new stall is on root 14, which the clause
  did not cover, because the baseline stopped on it with tables.
- Calibration: the bet that the union and V2 together would close the
  stops was the one in doubt, and it failed, as eight of nine fixed-depth
  variants had. The pattern holds under a clock: each change moves the
  explosion between roots and arms rather than ending it.

**Decision, 2026-11-26.**
- By the registered rule, a stop with tables refutes the candidate. No
  repair is registered and no gate goes to the maintainer.
- What is now established: the explosion is in Rarog's search in a band
  window, with or without tables. The four rules that differ from
  Stockfish's and the probe rule are among its parts, and removing them
  all does not end it. The variant search has run its course: three
  rounds, ten builds and one clock read of a candidate, with no
  candidate that closes the stops.
- The next step is the design question the rule names (`R3`): why
  Rarog's search in a band window, inside the tables and at root 30
  without them, grows where Stockfish's and Reckless's do not. It is
  read from the two searches' structure, not from another variant
  round, and it decides between three ends:
  - a repair with a mechanism, read under this instrument and then
    registered as a gate;
  - the containment for decided roots, which leaves `bench 13` exact and
    covers root 26 only;
  - closing D.9.3 `NO_CHANGE` with the cost recorded (four of 96
    first-move searches at 2+1 at the hard limit, on one cohort of
    seven- and eight-man roots) and its triggers.
- No engine change is made while the leaf is research. The candidate,
  `cand.exe`, stays archived as the read's binary.

## D.9.3's design question, 2026-11-26

Research on why Rarog's compact search explodes on these roots where
Stockfish's does not, read from the two searches' structure and deterministic
single-thread traces (`analysis/artifacts/d93r3-2026-11-26/`; zero games).

### Donor priors without tables, and the observational microscope

First, the two donors on root 30 without tables (`Threads 1`, `Hash 64`,
`uci_trace.py`, `r30_sf19_notb.txt` and `r30_reckless_notb.txt`):

- **Stockfish 19** (`sf_19` release binary) passes root 30 without tables:
  depth 16 at 1,840,961 nodes (`cp -846`), depth 20 at 2,146,007 (`cp -898`),
  depth 29 at 23,815,306 (`cp -1621`) and depth 30 at 28,984,809 (`cp -1967`).
  With tables (`r30_sf19_tb.txt`) it reaches depth 37 at 12,436,059 nodes and
  then hits the 60M cap on move 1 of depth 38 once mating distance enters view.
- **Reckless 0.9.0** (binary only, ideas-only reference; its UCI parser ignores
  `nodes` beside `depth`) **does not** pass root 30 without tables inside 30M
  nodes: it finishes depth 19 at 2,257,795 nodes (`cp -669`), then spends
  **97.2M nodes** on depth 20 across sixteen aspiration fail-lows (finishing
  depth 20 at 99,471,439 nodes, `cp -7289`) and **92.7M nodes** on depth 21
  (192,193,100 nodes, `mate -20`). With tables (`r30_reckless_tb.txt`) it
  reaches depth 23 in 10.8M nodes, depth 25 in 47.4M and depth 29 in 75.7M, and
  at depth 30 jumps to **723,001,279 nodes** (`mate -15`). The compact kernel's
  donor shares the class (`RAR-S127`).

Second, an observational-only `diag,probes` build of the baseline (`obs_v0.exe`,
SHA-256 `09e5a770f62c573f67b3f38b1b089610415d2a381fdf26f5b89ca5247dd635ff`,
`bench 13` 3,694,298 / EBF 2.324) and of the refuted candidate (`obs_cand.exe`,
SHA-256 `781cb6bf23584e369015f3f91c29061c441c19bf396a833e65b80b5af929cdf8`,
`bench 13` 3,114,639 / EBF 2.303) was run to a 30M-node cap across the stalling
roots and their preceding finished iterations (`obs_d93.py`, `run_obs.py`,
`obs_summary.txt`, all exit 0):

| Search (30M cap) | Int. nodes | `ply >= 2d` | `ply >= 4d` (`max_ply`) | `LINE_BOUND` hits | `hasQ` | Mean `\|corr\|` | LMR `+1/+2` ext % | Sing. `+3` | LDSE `+1` |
|---|---|---|---|---|---|---|---|---|---|
| `r30` notb `d15` (`v0`, finishes) | 316,254 | 7.7% | 0.0% (38) | 0 | 81.4% | 126.7 | 2.7% | 2,769 | 24,992 |
| `r30` notb `d16` (`v0`, stalls) | 17,944,706 | **66.7%** | **51.5%** (127) | 252,363 | 98.9% | **413.3** | **53.3%** | 91,576 | 2,233,116 |
| `r30` notb `d13` (`cand`, stalls) | 18,928,922 | **74.0%** | **46.3%** (103) | 419,452 | 98.4% | **449.8** | **41.8%** | 107,542 | 2,489,852 |
| `r30` tb `d16` (`v0`, stalls) | 16,840,796 | **97.2%** | **93.1%** (119) | 1,599,685 | 99.7% | **429.4** | **45.5%** | 133,035 | 2,325,291 |
| `r30` tb `d17` (`cand`, stalls) | 17,703,703 | **97.0%** | **96.4%** (127) | 1,887,891 | 99.7% | **427.7** | **27.6%** | **626,729** | 1,151,364 |
| `r26` tb `d18` (`v0`, stalls) | 17,569,134 | **97.9%** | **95.1%** (143) | 2,054,848 | 99.8% | **445.3** | **45.4%** | 220,637 | 2,513,709 |
| `r26` tb `d22` (`cand`, stalls) | 17,323,626 | **96.2%** | **93.4%** (175) | 1,380,189 | 99.8% | **378.4** | 8.3% | **949,015** | 1,278,193 |
| `r11` tb `d24` (`v0`, stalls) | 17,545,680 | **94.4%** | **90.9%** (191) | 634,363 | 98.3% | **432.8** | **34.5%** | 129,156 | 2,316,062 |
| `r14` tb `d32` (`v0`, stalls) | 17,208,998 | 25.9% | 0.04% (138) | 0 | 7.7% | 291.5 | **35.2%** | 105,478 | 1,416,623 |
| `r14` notb `d32` (`cand`, stalls) | 19,679,262 | 0.9% | 0.0% (108) | 0 | 6.2% | 155.9 | 10.3% | 210,895 | 1,692,613 |

### Structural diagnosis against Stockfish 19

First, a correction to *The clock read's result*: on root 30 without tables at
depth 16, **zero** nodes in the first 30M have `beta` in the decisive band
(`obs_win_band = 0`, `obs_loss_band = 0`; 81.5% have `|beta|` in `1,000..3,000`
and 2.5% in `3,000..31,519` from rule-50-downgraded mates blended by `lerp`).
The explosion does not require a decisive-band window; a decisive-band window is
the fastest way to trigger it.

Three mechanisms separate Rarog's compact search from Stockfish 19 (`sf_19`,
`src/search.cpp`), and the table above shows how they interact:

1. **Unbounded depth inflation past `2 · root_depth` (`D1`, the carrier on
   roots 26, 30 and 11).** On roots 26, 30 and 11 — both with and without
   tables, and under `cand` (`WU + V2`) as well as the baseline — **66.7% to
   97.9% of all interior nodes sit past `2 · root_depth` and 46.3% to 96.4%
   past `4 · root_depth`**, running until `LINE_BOUND` (`8 · root_depth − ply`,
   up to `MAX_PLY` 240 since D.6.7) clamps remaining depth (`252k` to `2.05M`
   hits, mean remaining depth still 3.5 to 7.3 deep in the tail). Four rules
   keep `child_depth >= depth` over 100+ plies once pawns promote (`hasQ` 98.3%
   to 99.8%):
   - **Correction-history saturation (`mean_|corr|` 378 to 450, near the 555
     ceiling).** `correction_update` (`compact/rules.rs:2254`) has no
     `!is_decisive(best_score)` guard (training 2.15M to 3.34M times on
     decisive scores with tables, and 727k times on mates without tables on
     root 30 alongside `±1,800` queening scores). Across the few pawn and
     non-pawn keys of a 6-to-8-man board the five correction slots saturate.
     In `late_move_reduction` (`:1988`), `-lmr_correction · |corr| / 1024`
     (`lmr_correction` = 9,252) subtracts **3.3 to 4.9 plies**, against a base
     `274 · ilog2(depth)` that does not grow with `move_count` (Stockfish's
     `reductions[d] * reductions[mn]` does, and its `|correctionValue| / 26310`
     removes at most ~1.5 plies) plus `-1,011` on Queen checks (32% to 47% of
     LMR calls). `r` turns negative, and `lmr_depth` (`:2038`) allows
     `new_depth + 2` (`depth + 1`, **one ply deeper than the parent**) at every
     ply: **45% to 53% of all LMR calls on `v0` extend late moves by +1 or +2
     plies**, and 75% to 78% do not reduce at all. Stockfish 19 explicitly
     forbids LMR extensions at `ss->ply >= 2 * rootDepth` (`search.cpp:1403`:
     *"To avoid search explosion, extensions are not allowed deep, relative to
     rootDepth, in the search tree"*).
   - **Singular triple extensions (`:1759`–`1765`).** The same saturated
     `|corr|` drives both `double` and `triple` bars below zero, turning
     **99.8% to 99.99% of singular extensions into +3-ply extensions**
     (`new_depth = depth + 2`; **949,015** times on `r26` `d22` and **626,729**
     on `r30` `d17` under `cand`, where LMR reduced more and singular took over
     the tail). Stockfish penalises double and triple margins past `rootDepth`
     (`:1291`–`1293`) and disables singular extensions under `seekMate`
     (`:351`, `:1278`).
   - **Low-depth singular extension (`ldse_extends`, `:1782`).** Absent in
     Stockfish; adds +1 ply (`new_depth = depth`) on Move 1 at every
     out-of-check cut node with `depth <= 7` and `estimate <= alpha − 10`,
     which holds automatically whenever `alpha` is in the win band or a wide
     queening window (1.15M to 2.51M firings per 30M nodes).
   - **Deeper re-searches (`research_depth`, `:2055`) and hindsight (`:1383`).**
     Add +1 ply on LMR fail-highs (258k to 608k times) and after large parent
     reductions (69k to 463k times) at any ply.
2. **Why decisive-window guards (`W1`–`W4`, `V2`, `V3`) cannot contain `D1`
   (`D2`).** Even inside Root 26's band-window stall (`v0` `d18`), **42.1%** of
   interior nodes (`7.41M` of `17.57M`) have non-decisive `(alpha, beta)`:
   singular exclusion searches open ordinary windows around `tt_score − margin`,
   `null_bound` (`:1553`) substitutes an ordinary `tt_score` for a winning
   `beta`, and `MultiCut` (`:1768`) and `ProbCut` (`:1672`) `lerp` ordinary
   scores toward decisive `beta`, injecting non-decisive phantom scores in
   `3,000..31,519` (`1.88M` to `3.08M` nodes; Root 11's `cp 5929` at depth 23).
   Meanwhile in check (34% to 36% of interior nodes on `v0`), Rarog disables
   all move-loop pruning (`!in_check`), whereas Stockfish keeps LMP, history
   and SEE pruning active once `!is_loss(bestValue)`; and at wide `alpha`,
   `razors` is blocked by `alpha < 776` (and under `W1` by `!tt_quiet`,
   `bound != Lower`, and `104 · d²` against Stockfish's linear `342 · d`).
   Finally, `tt_cuts` (`:1317`, `tt_score < beta` against Stockfish's `<=`)
   requires `tt_depth >= depth + 1` for every `Bound::Lower` cutoff (`885k` to
   `2.15M` same-depth hits skipped).
3. **Root aspiration fail-low cascades at high ordinary depth (`D3`, the
   carrier on root 14 at depth 32 and an amplifier on root 30).** Root 14 at
   depth 32 (`v0` with tables, and `cand` without tables) has **zero**
   decisive-band nodes and **74.1% to 99.1% of its nodes below `2 · root_depth`
   (`ply < 64`)**: when tables (or `WU`) expose a loss in the 7-man
   opposite-bishop fortress, Reckless's `27/128` (+21.1%) fail-low widening
   (`asp_max_fails = 20`, against Stockfish's `47/128` = +36.7%) steps through
   more than a dozen depth-32 windows. And on root 30 with tables,
   `window_fail_low` (`:2417`) preserves earlier fail-high `w.reduction`
   (Stockfish resets `failedHighCnt = 0`), finishing iterations 9 to 15 at
   effective depths 5 to 9 before iteration 16 jumps to depth 15.

### The mechanism diagnostic read, registered 2026-11-26 before it ran

To test `D1` causally — and to separate Stockfish's one-line LMR extension cap
from the other three deep-ply extenders without `RAR-S127`'s −36% hard chop
(`2 · root_depth − ply`, which zeroes remaining depth at `2 · root_depth`) — two
diagnostic builds of an exported copy of `44836d3` (`obs_d93.py` counters kept,
never committed) are registered before either is built:

- **`d_lmr2`:** `lmr_depth` caps `-reduction / 1024` at `0` when
  `ply >= 2 * root_depth` (and keeps `+2` when `ply < 2 * root_depth`).
- **`d_ply2`:** `d_lmr2`, plus at `ply >= 2 * root_depth`: `extension =
  extension.min(0)` (no positive singular or LDSE extension), no `+1` ply in
  `hindsight_depth`, and no `+1` ply in `research_depth`. Remaining `depth > 0`
  still searches out naturally at `−1` ply per move.

**Measurements** (`Threads 1`, `Hash 64`, deterministic):
1. `bench 13` on `d_lmr2` and `d_ply2`.
2. `tools/diag/tb_probe_read.py --epd tools/books/ocb_cohort_v1.epd --men 7,8
   --per-count 24 --depth 20 --max-nodes 30000000 --only 26,30,11 --syzygy
   D:/chess/tablebases/syzygy3456` on `obs_v0`, `d_lmr2` and `d_ply2`; if
   either diagnostic finishes all three, the same depth-20 read on all 48 roots
   with tables.
3. Without tables on root 30 at `--depth 20 --max-nodes 30000000`, plus single
   traces (`nodes 30000000`) on root 30 without tables at depth 16, root 11
   with tables at depth 24, and root 14 with tables at depth 32.

**Frozen predictions:**
- `bench 13` moves downward on both builds (0.95): by 0.5% to 8% on `d_lmr2`
  and 2% to 15% on `d_ply2` (0.70), far smaller than `RAR-S127`'s −36% chop.
- With tables at depth 20 (30M cap): `d_lmr2` finishes roots 26 and 11 (0.85)
  and all three (0.55); `d_ply2` finishes all three of 26, 30 and 11 (0.80),
  with zero interior nodes at `ply >= 4 * root_depth` (0.99).
- Without tables on root 30 (30M cap): `d_ply2` finishes depth 16 (0.85);
  finishing depth 20 without tables has confidence 0.45, since Reckless itself
  takes 99.5M nodes at depth 20 across sixteen fail-lows into a 40-ply mate.
- Deeper clock-stall roots (30M cap, with tables): `d_ply2` finishes root 11
  at depth 24 (0.75) and leaves root 14 short at depth 32 (0.80), where 74.1%
  of the baseline's nodes already lie below `2 * root_depth`.

**Falsifiers:**
- `D1` is refuted as the necessary carrier of the deep-tail explosions if
  `d_ply2` leaves root 26 or root 30 short of depth 20 with tables, or root 30
  short of depth 16 without tables.
- LMR's negative reduction alone is sufficient if `d_lmr2` matches `d_ply2` on
  all four of those checks, and insufficient (proving singular `+3` and LDSE
  carry the tail when LMR is capped) if `d_lmr2` leaves one short that `d_ply2`
  finishes.

### The mechanism diagnostic read: the result, 2026-11-26

Both diagnostic builds were compiled from fresh `git archive HEAD` (`44836d3`)
copies with `obs_d93.py` and `diag_mech.py` (`--release --features diag,probes`)
and archived beside their logs and outputs in
`analysis/artifacts/d93r3-2026-11-26/` (`run_mech.py`, `run_mech.log`,
`mech_summary.txt`, `read_mech_3.json`, `read_mech_r30_notb.json` and
`mech_r*.txt`, all exit 0; evidence layer: deterministic single-thread node
counts and diagnostic counters at `Hash 64`, zero games, not Elo):

- **`d_lmr2.exe`:** SHA-256
  `2ccc96d535a87f4eeadc54970ad6b4bb41334fcc6443850dc7631f589ac331a6`,
  `bench 13` **4,235,377 (+14.65%) / EBF 2.352** (`d_lmr2_build.log`).
- **`d_ply2.exe`:** SHA-256
  `86bda611b3347b17b52b399199d2a70c438817b953c08a5a8de90bc8bf9d15d1`,
  `bench 13` **3,313,794 (−10.30%) / EBF 2.313** (`d_ply2_build.log`).

Neither build finished all three of roots 26, 30 and 11 at depth 20 with
tables, so the conditional 48-root depth-20 read did not trigger:

| Search (30M-node cap, 1T, `Hash 64`) | `obs_v0` (baseline) | `d_lmr2` | `d_ply2` | `d_ply2` `ply >= 2d` / `>= 4d` (`max_ply`) | `d_ply2` unreduced LMR (`r <= 0`) |
|---|---|---|---|---|---|
| Root 11, with tables, `depth 20` | **d20** at 1,903,586 (`cp 1456`) | d18 at 2,331,985 (`cp 1368`) | **d20** at 2,304,165 (`cp 19970`) | 29.7% / **0** (61) | 49.2% (35.2% at `0`) |
| Root 26, with tables, `depth 20` | d17 at 484,662 (`cp 19987`) | d19 at 722,716 (`cp 19989`) | **d20** at 22,850,764 (`cp 19991`) | 80.8% / **0** (59) | 64.5% (58.6% at `0`) |
| Root 30, with tables, `depth 20` | d15 at 857,473 (`cp -849`) | d16 at 15,810,668 (`cp -988`) | d16 at 12,297,646 (`cp -19982`) | 84.7% / **0** (54) | **71.5%** (**65.6%** at `0`) |
| Root 30, no tables, `depth 20` | d15 at 495,516 (`cp -385`) | d14 at 685,983 (`cp -430`) | d14 at 29,187,335 (`cp -2852`) | — | — |
| Root 30, no tables, `depth 16` | d15 (stalls in d16) | d14 (stalls in d15, `max_ply` 119) | d14 (stalls in d15, `max_ply` 63) | 46.9% / 807 (63) | **69.5%** (**49.7%** at `0`) |
| Root 11, with tables, `depth 24` | d23 (stalls in d24) | d18 (stalls in d19, `max_ply` 143) | d21 at 22,760,691 (`cp 19972`) | 89.8% / **0** (68) | 53.6% (49.5% at `0`) |
| Root 14, with tables, `depth 32` | d31 (stalls in d32) | d27 at 3,207,000 (`cp -140`) | d25 at 2,139,767 (`cp -174`) | 37.5% / **0** (80) | 71.9% (44.1% at `0`) |

**What the counters prove.**
1. **Both registered falsifiers fired, and together they finish the causal
   map.** First, `d_lmr2` fails on all three roots at depth 20 (`d18`, `d19`,
   `d16`) while `d_ply2` finishes roots 11 and 26: when LMR's `+1/+2` extension
   is capped past `2 · root_depth` on its own, `singular +3` (`526,733` on
   `r26`, `410,029` on `r11`), `hindsight +1` (`1,871,608` on `r26`, `501,564`
   on `r11`) and `ldse +1` (`1,772,399` and `498,863`) immediately take over the
   deep tail (`96.5%` and `88.3%` of interior nodes still at `ply >= 2d`,
   `max_ply` 159 and 143). Stockfish's one-line LMR cap is therefore
   insufficient in Rarog's kernel, and raises `bench 13` by +14.65%.
2. **Clamping all four positive extenders past `2 · root_depth` (`d_ply2`)
   completely removes the `4 · root_depth` tail (`0` nodes at `ply >= 4d` and
   `0` `LINE_BOUND` hits on every tablebase trace; `max_ply` 54 to 80 instead of
   119 to 191) — and still explodes inside `2d..3.5d`.** On root 30 with tables
   at iteration 17 (`2d = 34`), `d_ply2` reaches no deeper than `ply = 54`, yet
   spends `15,066,075` of `17,778,765` interior nodes (84.7%) in `ply 34..54`
   and stalls before finishing depth 17 (having already found the tablebase loss
   `cp -19982` at depth 16; on root 11 it likewise finds the tablebase win
   `cp 19970` at depth 20 and stalls at depth 22). Two mechanics inside the
   fitted kernel produce that `2d..3.5d` wall:
   - **Depth build-up below `2 · root_depth`:** at `ply < 2 · root_depth`,
     saturated `|corr|` (`433` to `466`) still triggers `singular +3` (`303,087`
     on `r30` `tb`, `511,541` on `r11` `d24`), `lmr +2/+1` (`674,724` on `r30`
     `tb`, `2,458,216` on `r30` `notb`) and `ldse +1` (`1.89M` to `2.27M`),
     pushing remaining `depth` up to ~20 plies as lines cross `2 · root_depth`.
   - **Collapse of LMR reduction to zero:** clamping `-reduction / 1024` at `0`
     prevents late moves from *extending* past `2 · root_depth`, but when
     `lmr_correction = 9,252` subtracts ~4 plies against a `move_count`-independent
     base `274 · ilog2(depth)`, `reduction` is negative on nearly every move, so
     clamping it at `0` leaves **`reduced = new_depth` (zero LMR reduction) on
     49.5% to 65.6% of all LMR calls** (`7,524,132` of `11,472,322` on `r30`
     `tb`), on top of unpruned checks (35% of nodes), non-decisive sub-windows
     (`D2`) and `27/128` aspiration fail-low cascades (`D3`). Stockfish's
     `reductions[d] * reductions[mn]` grows with `moveCount` and subtracts at
     most ~1.5 plies from `|correctionValue|`, so its late moves still reduce by
     3 to 4 plies even when correction history saturates.

**Calibration**, appended; the frozen predictions above stand as written.
- `bench 13` downward on both builds (0.95): **wrong on `d_lmr2`** (+14.65%,
  4,235,377) and held on `d_ply2` (−10.30%, 3,313,794, inside −2% to −15%).
  Confidence 0.95 missed an elementary alpha-beta effect: denying deep LMR
  extensions alone delays tactical refutations on shallow bench positions and
  widens sibling subtrees.
- Depth 20 with tables: `d_lmr2` finishing 26 and 11 (0.85) and all three (0.55)
  was **wrong** (0 of 3; overconfident because singular `+3` and hindsight `+1`
  substituted 1:1 for LMR extensions). `d_ply2` finishing 26 and 11 held, and
  finishing all three (0.80) **missed on root 30** (`d16`, stalling in `d17`).
- Zero interior nodes at `ply >= 4 · root_depth` on `d_ply2` (0.99): **held** on
  every tablebase trace (`0` on `r26`, `r30`, `r11 d20`, `r11 d24`, `r14 d32`);
  `r30` without tables recorded `807` of `18,753,287` (0.004%) at iteration 14
  (`4d = 56`, `max_ply = 63`) from depth accumulated before `ply = 28`.
- Root 30 without tables: `d_ply2` finishing depth 16 (0.85) was **wrong**
  (`d14`); not finishing depth 20 (0.55 against 0.45) held.
- Deeper clock-stall roots on `d_ply2`: finishing root 11 at depth 24 (0.75) was
  **wrong** (`d21`, `cp 19972`); leaving root 14 short at depth 32 (0.80)
  **held** (`d25`).

### Decision, 2026-11-26

**D.9.3 closes `NO_CHANGE`**, with the structural answer, the measured cost and
its triggers recorded.

- **Why a standalone search patch is rejected now (End State 1).** I recommend
  against implementing a search repair before a search re-fit. Across four
  rounds and twelve builds (`V1`–`V3`, `V2+V3`, `W1`–`W4`, `WU`, `cand`,
  `d_lmr2`, `d_ply2`), both donor-guard ports and deep-ply extension clamps are
  refuted as standalone cures. The explosion is not a localized port bug or a
  tablebase-only defect: Reckless 0.9.0 itself explodes to 99.5M nodes at depth
  20 without tables and 723.0M at depth 30 with tables on root 30. Ending it
  requires reshaping five coupled parts of the compact kernel together —
  `!is_decisive(best_score)` in `correction_update`, `move_count` scaling
  against `lmr_correction` in `late_move_reduction`, ply-relative extension
  budgets (`ply < 2 · root_depth` and singular depth penalties), in-check
  move-loop pruning when `!is_loss(best_score)`, and aspiration fail-low
  widening / `w.reduction` reset — and **re-fitting** the seed table around them.
  Changing those formulas unfitted moves `bench 13` by 10% to 16%, invalidates
  D.7.11's 225-coordinate fit across every ordinary game, and requires a Case 3
  gate (`[-5, 5]`, ~12,000 games without tables, 34% neutral rejection in
  `RAR-M86`) for a defect that caused **zero time losses** in `RAR-S93`,
  `RAR-S94` and all 96 clock-read searches. Those five items are carried to
  **F.6** (the search re-fit).
- **Why the decided-root containment is also rejected now (End State 2).** I
  recommend against clamping `maximum_ms` to `optimum_ms` when the last finished
  score is in the tablebase band. First, it covers **one of the five** baseline
  hard-limit stops (`root 26`, where `bestmove` is already a proved tablebase
  win with 31.9 s plus 1 s/move left and zero time-loss risk), leaves roots 11,
  14 and 30 (both arms) and all `go infinite` / fixed-depth analysis untouched,
  and has no precedent in Stockfish or Reckless (which shorten the clock only at
  DTZ roots, as Rarog already does). Second, it introduces a tactical failure
  mode on 7- and 8-man boards: when the previous iteration returned a tablebase
  win or loss and the next iteration fails low at the root — for example because
  a 50-move draw or repetition blocks `bestmove`'s route to the 6-man capture,
  or because one loss-band move resists far longer in practice — clamping the
  hard limit to `optimum_ms` (~1.4 s at `2+1`) aborts the fail-low recovery
  immediately and plays the refuted move.
- **Recorded cost at close.** On D.9.2's 48 seven- and eight-man opposite-bishop
  roots at the first move of `2+1` (`Threads 1`, `Hash 64`, `read_clock.json`),
  5 of 96 searches across 4 roots end at the 88.1 s hard limit (73% of the base
  clock): roots 11, 14, 26 and 30 with 3–6-man tables, and root 30 without
  tables; the other 44 roots spend 61.4 s total with tables against 61.8 s
  without (median ratio 1.07), and 0 of 96 searches lose on time. Fixed-depth
  and `go infinite` analysis stalls past 30M nodes on those roots once
  correction history saturates.
- **Triggers to reopen D.9.3 (and D.9.2 on its own triggers):**
  1. **F.6 (the search re-fit):** evaluate and jointly fit the five coupled
     mechanisms identified above (`correction_update` decisive guard, LMR
     `move_count` scaling vs `lmr_correction`, ply-relative extension budgets,
     in-check move-loop pruning at `!is_loss(best_score)`, and aspiration
     fail-low `w.reduction` reset / widening). When a search re-fit ends the
     explosion on D.9.2's 48 roots, D.9.2's registered probe-after-TT-cutoff
     read also reopens unchanged.
  2. **Before F.6:** a time loss in play (with or without tables) traced to a
     hard-limit stall; a rated Rarog pool or release qualification adopting
     tables and measuring clock harm; or a user report of an analysis hang that
     justifies an analysis-only or opt-in bound.
- **Remaining before D.9 closes:** only the maintainer's yes or no on RAR-S93's
  with-tables activation read in the release qualification (`D.9.4` in PLAN),
  and the timing of `d9`'s pull request.

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
