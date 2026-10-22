# C.12 follow-up speed pass — day record (2026-10-17)

The maintainer asked, after C.12's overnight pass (`analysis/c12_speed_pass_2026-10-17.md`,
RAR-S99 accepted), for further research into speed: the evaluation first, then
search and board, with Rust and chess-programming practice applied, bigger
rewrites allowed as long as behaviour stays exact, SPRT allowed sparingly (none
was played). Every candidate below was exact: `bench 13` printed **10,521,141 /
EBF 2.521** on each build.

## Result

- **No code change earned its place.** The three exact engine commits the day
  kept — continuation contexts hoisted out of the history update (`45d4c30`),
  quiets scored as the generator emits them (`4806fad`), the quiet SEE prune
  passed without an empty exchange (`6285e3d`) — read **+0.15% (−0.35 ..
  +0.65)** together with eight PGO builds an arm and six cycles, and **−0.11%
  (−0.75 .. +0.53)** on large pages. `90700b2` reverts them; the engine source on
  `c7` is `60fedc5`'s exactly.
- **Large pages: +2.48% (95% +2.13 .. +2.84)** on the C.12 head (`60fedc5` with
  the allocator, four builds, against eight of `60fedc5`). Exact. It needs six
  new `unsafe` sites against the frozen floor, so it waits on the maintainer:
  a commit on branch `large-pages` on top of `90700b2`, whose patch landed
  unchanged as `ac7c13b` (equal patch-id; the branch was deleted 2026-10-19).
  **Decision (maintainer, 2026-10-17):** left unmerged on its branch until the
  unsafe floor is reviewed; GUIDE's holds carry it, and G.1 owns large pages.

## Profile

The maintainer's ETW capture (`tools/profile/`, the engine of `6737b82`, `bench
13 5` and six games at 150,000 nodes a move), attributed with inline chains
(`summarize_board_search_etw.py --scheme search`):

| Region | bench | games |
|---|---:|---:|
| evaluation | 21.4% | 22.4% |
| move ordering | 20.5% | 20.5% |
| node body | 17.2% | 17.8% |
| move generation | 9.4% | 9.2% |
| transposition table | 8.3% | 7.9% |
| SEE | 7.0% | 6.3% |
| make/unmake | 4.8% | 4.7% |
| evaluation correction | 4.5% | 4.1% |
| history updates | 4.4% | 4.6% |

Bench and games agree within a point everywhere, so `bench 13` stays a fair PGO
training load. The hottest instructions were memory waits: the TT probe's first
compare (`tt.rs:795`, 4.3%), the instruction after the child's TT prefetch at
pick time (3.2%: the `prefetcht0` itself held at retirement, a page walk on the
64 MB table), the quiet history loads in quiet scoring (3.5% and 2.7%) and the
correction reads. The evaluation's 21% is spread over many terms, none above
3.3%.

With the large-page build (a user-mode sample, `bench 13 4`, 9,217 samples) the
probe's line fell to 1.6% and the prefetch's to 0.7%; quiet scoring's history
loads (3.8% + 2.2%) and the correction reads then lead.

## Every candidate

All exact. "4v4" is the recorded pooled read (`nps_build_pool.ps1`,
`nps_read.py`, two cycles, extended to six between the thresholds); "LP" means
both arms carry large pages.

| Candidate | Read | Verdict |
|---|---|---|
| History update: the four continuation contexts read once per update, the quiet-malus fade from a compile-time table (`45d4c30`), with quiets scored as generated (`4806fad`: the generator's sink receives the moving piece; a test checks every push names the piece on the origin square and the list order) | +0.02% (−1.03 .. +1.07), 4v4 | flat |
| Selection with four interleaved running maxima (`select_unpredictable`), merged by score then position; a test checks it against one running maximum on 24,000 lists | +0.06% (−0.82 .. +0.93), 6 cycles | flat, not kept |
| **Large pages**: a process allocator backing allocations of 2 MB or more with 2 MB pages (TT, correction tables, continuation histories) | +1.84% (+0.68 .. +3.00) over `4806fad`; **+2.48% (+2.13 .. +2.84) over `60fedc5`** | **gain** |
| Correction-slot prefetch after the make, on LP | +0.27% (−0.29 .. +0.83), 6 cycles | flat, not kept |
| Quiet SEE prune passed when no enemy piece can reach the destination (threats, plus no enemy slider on the move's line) (`6285e3d`) | +1.13% (+0.54 .. +1.72) on LP; +0.26% (−0.94 .. +1.47) without, 6 cycles | see the bundle |
| The same shortcut for captures at every SEE call site | −1.86% (−3.08 .. −0.65) | loss, not kept |
| Non-pawn material and squares kept on the search stack (each child set from its parent and the move; the evaluation reads it) | −0.25% (−1.12 .. +0.62) | flat, not kept |
| Quiescence and ProbCut SEE answered by the scoring pass's sign class when the threshold is on the far side of zero | −0.78% (−2.68 .. +1.12) | flat, not kept |
| `opt-level = 2` | −3.89% (−5.59 .. −2.20) | loss |
| Bundle `45d4c30`+`4806fad`+`6285e3d`, 8v8 | +0.15% (−0.35 .. +0.65), 6 cycles | flat: reverted |
| The same bundle, both arms on LP | −0.11% (−0.75 .. +0.53) | flat |

Each shortcut was checked by a debug assertion against the computation it skips,
on debug `bench 12` (7,417,463 nodes), and each assertion was shown live by a
mutant that fails it. Drafted and not built, because the pattern below
predicted nothing: mobility and the long-diagonal term read inside the attack-map
fill, dropping the per-square attack array.

## What the measurements say

- **The engine is bound by memory latency, not by instruction count.** Every
  change that removed work off the latency chain read flat (fused scoring,
  incremental piece-square sums, interleaved selection, a skipped SEE), and the
  SEE shortcut extended to every capture read negative. The change that
  shortened the latency chain itself, large pages, is the day's only gain.
- **A step-1 accept at two cycles can be a high draw.** The quiet SEE shortcut
  read +1.13% (interval above zero) on large pages, and the bundle containing it
  read −0.11% under the same conditions.
- **Build spread sets the floor.** Four PGO builds an arm resolve about ±1%
  here (one pool's builds spanned 1.7%); eight resolve about ±0.5%.
- **Rust specifics:** `select_unpredictable` (stable since 1.88) remains the way
  to force a conditional move; `opt-level = 3` beats 2 by 3.9%; fat LTO, one
  codegen unit and PGO stay. The 1.98 release notes add nothing that bears on
  this code.

## Large pages: what the decision involves

- **Mechanism:** `src/large_pages.rs` (`ac7c13b`'s patch on `90700b2`). On
  Windows the global allocator sends every allocation of 2 MB or more to
  `VirtualAlloc`: with `MEM_LARGE_PAGES` when the user holds "Lock pages in
  memory" (enabled in the process token at the first such allocation), with
  ordinary pages otherwise. Everything smaller goes to the system allocator.
  `bench` prints `Large pages : on/off`.
- **Exactness:** only the physical backing changes; tables, contents and search
  are the same (`bench 13` 10,521,141 on the PEXT PGO pools and the magic
  release build).
- **Cost:** six new `unsafe` sites (two FFI declaration blocks, the privilege
  call, the `GlobalAlloc` impl, one block each in `alloc` and `dealloc`),
  against the floor of 19. The trait's default `alloc_zeroed` and `realloc`
  are kept, so small zeroed allocations pay a memset (start-up only).
- **Reach:** a user without the right gets ordinary pages and no gain; this
  host has it (`whoami /priv`: `SeLockMemoryPrivilege`). Linux and macOS are
  untouched (transparent huge pages would be a separate `madvise` site).
- **Verification on the branch:** debug suite 395 tests in 32 binaries and
  release suite 396, none failed; `cargo clippy --all-features --all-targets`
  clean in the default and the `rarog_pext` configuration; `cargo fmt`; the
  `tune`, `texel` and texel-tuner builds.
- **Value:** at the 1.9 Elo per 1% three passes measured at `3+0.03`
  (RAR-S99), +2.48% is about +4.7 Elo. Confirming that in an SPRT would take
  tens of thousands of games; the engine is exact, so the NPS read is the
  evidence, and a short match read for in-game nodes a second (RAR-S99's
  diagnostic) would show the speed reaching games.

## Not tried, for a later pass

- A look-ahead TT prefetch: peek the next candidate within a stage and prefetch
  its child's cluster before searching the current move, without consuming the
  pick (the selection's tie rule makes it delicate).
- A pawn-cache prefetch at the make when the move changes the pawn key.
- Transparent huge pages on Linux (`madvise(MADV_HUGEPAGE)`), if large pages are
  accepted.
- Sharing the node's check masks with quiet scoring (screened −1.6% overnight
  without PGO, which misranks such changes, so not shown either way).

## Local artifacts

`analysis/artifacts/c12b/` (ignored): `worklog.md`, pools (`pools/`: `base`,
`base8`, `p2`, `s1`, `head8`, `lp`, `base-lp`, `head-lp` and the screens),
reads (`reads/`), the patches of every dropped candidate (`screens/*.patch`),
the large-page profile (`prof/lp/`), and the branch's closing suites
(`final/`).
