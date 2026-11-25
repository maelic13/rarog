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
| PROTO-10: the pre-search root ranking and each extension step are unabortable FFI calls | Partly repaired since the review: `go movetime` now has a box (`time.rs:179`), the box is checked between probes, and under a clock the start rule keeps ten overheads (RAR-S97). Under `go movetime` the start rule is still absent (`tb_extension_latest_start_ms` stays infinite), so the extension can begin after the move time; the second review reproduced 14 ms on `go movetime 10`. The pre-search ranking is still one call with no clock check. | The `go movetime` start rule: repair, D.9.1. The rest: `NO_CHANGE` (below) |
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
    one second a move.
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
     `go movetime`, by the clock's slack rule.
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

One recommendation outside D.9, for the maintainer: the release
qualification (E.1, E.3.4) could carry RAR-S93's with-tables activation
read. That is 1,500 games on the endgame cohort, about two minutes. It
would cover the configuration CCRL rates, which no pool covers. Its
parser `tools/diag/tb_activation.py` still computes the time budget by
the formula in use before D.7.9.1 (`optimum_ms`). That has to be brought
to `search/time.rs`'s `budget` before the read is used.

## Corrections, the same day

A second review of D.9 (2026-11-25), checked against the code and the
records, found these. They are corrected above:

- The PROTO-9 repair named the wrong gate: `shared.syzygy.largest` is 0
  at a DTZ root.
- The PROTO-10 trigger's repair named `tb_probe_root_dtz`, which ranks
  clean wins equally.
- The `go movetime` start rule was missing.
- B.5.2's probe stress test was not cited.
- The post-probe poll was missing.
- `tb_activation.py`'s budget formula is stale.
- `configure`'s re-send of every option was not noted.
