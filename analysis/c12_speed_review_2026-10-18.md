# C.12 third speed pass — review and record (2026-10-17/18)

The maintainer asked for a third, independent pass on Rarog's speed after the
overnight pass (`analysis/c12_speed_pass_2026-10-17.md`, +22.47% NPS, RAR-S99
accepted) and the day follow-up (`analysis/c12_speed_followup_2026-10-17.md`, no
code gain, large pages pending): a review of both, fresh research with the
earlier conclusions treated as hypotheses, and implementation with the
pooled-PGO read. It ran from the evening of 2026-10-17 into the night, on `c7`,
starting from `d942e9d` (engine source `60fedc5`, the `base8` pool). Every
candidate below was exact: `bench 13` printed **10,521,141 / EBF 2.521** on the
PEXT and the magic build of each.

## Result

**No engine change is kept.** Seven exact candidates were implemented, each as
its own commit with eight PGO builds an arm and read against the chain it sat
on; two more questions were answered from the earlier passes' pools. Two
candidates read as small gains below the acceptance bar, two as null, three as
losses. The two small gains together, the head's engine at `630e2b5`, read
**+0.41% (+0.28% .. +0.54%)** against `base8` over six cycles with eight builds
an arm: a real gain by its interval, under the protocol's +0.5% bar, so by the
rule declared before the reads both were reverted. The engine source on `c7` is
`60fedc5`'s again (`git diff 60fedc5 HEAD` touches only documentation); the
commits stay on the branch under their revert commits and
`git cherry-pick 6c89771 630e2b5` re-applies them without conflict, about 0.8
Elo at RAR-S99's rate, if the maintainer judges the bar too high for an
eight-build read. No game gate is prepared: nothing changed. The session's
null, a final read of the head against `base8` with the same engine source on
both arms, is +0.10% (−0.06 .. +0.26) over six cycles: the two gains' own
estimates (+0.19%, +0.17%) are the size of a self pair's offset, and only their
sum, whose lower bound +0.28% clears the null's upper bound, stands apart from
it. Measured afterwards at the maintainer's request with both arms on large
pages, the same pair reads +0.08% (−0.11 .. +0.27): not shown there, and the two
reads' intervals barely overlap, so the pair's true size is somewhere between a
few tenths of a percent and nothing. **Decision (maintainer, 2026-10-18):** the
pair is kept as a safe exact change, re-applied on `c7` as `842b586` (A) and
`c01db68` (C) with no gate; the closing suite was run again on that head
(`analysis/artifacts/c12c/final-ac/`).

## Review of the two passes

**The overnight pass.** Its 21 commits are exact by the fingerprint on both
backends and by the suites, and this pass found no exactness hole in the hot
path it read (the node loop, picker, histories, correction, TT, make/unmake); it
did not re-audit all 21 diffs line by line. Its measurements are sound where it
drew conclusions from four-build reads, and it labelled its two-build screens
correctly as "not shown". Two of its lessons are confirmed here with better
instruments: PGO must be in the screen for branch-shape and prefetch changes, and
the pick-time TT prefetch is the one prefetch that pays. One of its code shapes
turned out to carry a duplicate: the move loop's `node_ci` and the quiet
context each computed the node's check masks (6.25M calls on bench 13 for about
3.7M nodes reaching the loop, 2.3M of them the duplicate); sharing them is exact
and read +0.17% (+0.00 .. +0.34). Nothing else read in the night's code wants
simplifying for speed; the hand-unrolled three-slot probe is right as it is (its
branch-free form loses 2%, below).

**The day pass.** Its conclusion that "the engine is bound by memory latency, so
instruction-count cuts read flat" is too strong. The fresh profile (below) puts
the identifiable memory-stall lines at 14-17% of samples; the other 80-plus
percent is compute spread over many terms, which is where the night's +22% came
from. The day's candidates each removed a small amount of work (expected
0.1-0.5%) and read inside a four-build instrument's floor, which is "not shown",
not evidence of a memory bound. This pass's own work-reduction candidates behave
the same way: removing about two million calls of a 10-20 ns function reads
+0.2%, as compute, not as latency. What the memory-stall share does mean is that
large pages (+2.48%, held on branch `large-pages`) remain the one measured
latency win, and that software prefetching cannot substitute for them (below).

**Dropped on measurements too weak to show them.** From the night: the five
two-build drops (SEE attackers once −0.31%, pins from snipers −0.66%, quiet rows
−0.59%, quiescence TT-move legality after stand pat −0.53%, SEE fast path
−0.86%) were and are not shown; each has at most a few tenths of a percent to
win, and the SEE fast path is now resolved as a loss (below). From the day: the
quiet-SEE skip and the capture-SEE extension are resolved below; the hoisted
history contexts with fused scoring (`45d4c30` + `4806fad`) read
+0.01% (−0.55 .. +0.57) against `base8` with the day's four-build pool, not
shown either way and left out; the interleaved selection (+0.06%), the
correction prefetch on large pages (+0.27%) and the incremental non-pawn
material (−0.25%) were read with four builds and six cycles and are flat within
±1%, which is as far as that instrument reaches.

**Odd read 1, the quiet-SEE skip (`6285e3d`).** The day read it at +1.13%
(+0.54 .. +1.72) on large pages after two cycles and +0.26% (−0.94 .. +1.47)
without, and its bundle at +0.15% (−0.35 .. +0.65). Rebuilt alone on `60fedc5`
in a worktree (eight builds, the pool's manifest names the cherry-pick) and
read against `base8`:
**−0.58% (−0.78 .. −0.37)**. The skip costs more than it saves: a line-table
lookup and an unpredictable branch on every quiet in the pruning loop (5.9M
calls) against the attacker scan it skips on 28% of them. The +1.13% was a
two-cycle step-1 accept on a four-build pool that read low, as the day pass
itself concluded from the bundle; the bundle's +0.15% then hid this loss against
the other two commits.

**Odd read 2, the capture-SEE extension (−1.86%).** The day's pool `s2` read
against `base8` instead of `s1`: **−1.54% (−2.30 .. −0.77)**, which with the
bundle's +0.15% puts the extension itself near −1.7%. The loss is real and
independent of the `s1` pool. Its mechanism is not shown; one more unpredictable
branch on 7.8M capture-SEE calls and a changed inlining of `see_ge` at every
site are enough to explain it.

## Profile of the head

User-mode sampler on a PGO PEXT build of `60fedc5`'s engine with debug
information (`bench 13 4`, two runs, 18,889 samples; `prof/head/`):

| Region | Share |
|---|---:|
| evaluation | 21.6% |
| move ordering | 20.7% |
| node body | 17.2% |
| move generation | 9.6% |
| transposition table | 8.2% |
| SEE | 7.1% |
| make/unmake | 4.9% |
| evaluation correction | 4.4% |
| history updates | 4.3% |

Hottest lines: the TT probe's first compare (`tt.rs:795`, 4.30%); the
instruction after the pick-time TT prefetch (`node.rs:1484`, 3.53%, the page
walk the prefetch triggers under 4 KB pages); the quiet-scoring history loads
(`movepick.rs:563` 3.53%, `:568` 2.79%); the SEE attacker scan
(`attackers_to_color`, about 3.3% over its lines); `apply_bonus` 1.5%;
`key_after_hint` 1.4%; the selection scan about 4.5% over its lines; the
correction reads about 2.4%.

Event counts from the `diag` build (bench 13, stride 1): 6,874,486 interior
nodes and 2,324,192 quiescence nodes; 3,790,902 TT hits and 441,182 TT cutoffs;
2,521,833 reverse-futility cuts (37% of interior nodes); about 3.67M nodes reach
the move loop; the TT move is searched at 1,437,441 of them; 1,763,946 cutoffs,
88% on the first move; 6,253,406 `check_info` calls; 5,937,327 quiet and
7,756,147 capture SEE calls.

## Every candidate and read

Reads are `nps_build_pool.ps1` and `nps_read.py` (PROCESS's two-step rule), the
host checked idle before each cycle and this session idle throughout. "8v8" is
eight PGO builds an arm. Each engine candidate is one commit on `c7`; the
reverts follow it.

| Candidate | Commit | Base | Read | Verdict |
|---|---|---|---|---|
| A. Threats computed only where a node reads them: the reverse-futility test in two steps (the relief term, `CoreRfpThreat` = 14, decides only between the full and the relieved margin); quiescence computes threats only with a non-empty capture list | `6c89771` | `base8` | +0.13% (−0.15 .. +0.40) at 2 cycles; **+0.19% (−0.04 .. +0.42)** at 6, 8v8 | NO_CHANGE; provisional, then reverted with the bundle (`117ae28`) |
| B. The TT move's child cluster prefetched before the noisy fill (main search and quiescence), the pick-time prefetch skipping it | `a0a00b8` | pool a | **−0.00% (−0.28 .. +0.27)**, 8v8 | null; reverted (`280a3b0`) |
| C. The node's check masks shared between the move loop and quiet scoring (the picker receives the loop's lazily built masks) | `630e2b5` | pool a | +0.17% (−0.10 .. +0.45) at 2 cycles; **+0.17% (+0.00 .. +0.34)** at 6, 8v8 | NO_CHANGE; provisional, then reverted with the bundle (`cdd180e`) |
| E. The node's own cluster prefetched before its closing history and correction updates, the cutoff store moved after them | `6ea7536` | pool c | **−0.26% (−0.46 .. −0.05)**, 8v8 | loss; reverted (`2a1d267`) |
| J. The day's quiet-SEE skip alone on `60fedc5` (worktree cherry-pick of `6285e3d`) | — | `base8` | **−0.58% (−0.78 .. −0.37)**, 8v8 | loss; stays out |
| D. Build flag `-C llvm-args=-x86-cmov-converter=false` for the x86 tiers (xtask; pool d's binary has 3,847 `cmov` against pool c's 3,795, so the flag was live) | `1a17c40` | pool c | **−0.34% (−0.60 .. −0.09)**, 4v8 | loss; reverted (`a69e80e`) |
| K2. The day's hoisted contexts and fused scoring (`45d4c30` + `4806fad`, pool `p2`) | — | `base8` | **+0.01% (−0.55 .. +0.57)**, 4v8 | not shown; stays out |
| L. The single-threaded probe branch-free: three slot tests as selects, one hit-or-miss branch | `7f815f1` | pool c | **−2.01% (−2.23 .. −1.79)**, 8v8 | loss; reverted (`dfd3b7a`) |
| F. The quiet, pawn and continuation history tables allocated as 64-byte lines and indexed through them (the 64 KB quiet table had landed at a 16-byte offset, every 128-byte row on three lines) | `47a06f9` | pool c | **−2.88% (−3.11 .. −2.64)**, 8v8 | loss; reverted (`f180e98`) |
| Bundle: A + C, the head's engine at `630e2b5` | — | `base8` | +0.37% (+0.17 .. +0.57) at 2 cycles; **+0.41% (+0.28 .. +0.54)** at 6, 8v8 | NO_CHANGE by the protocol; both reverted by the pre-declared rule |
| K. The day's capture-SEE pool `s2` | — | `base8` | **−1.54% (−2.30 .. −0.77)**, 4v8 | the day's loss confirmed |
| A + C on large pages: both cherry-picked onto the `large-pages` commit (`ac7c13b`'s patch on `90700b2`) in a worktree, against that commit itself (both arms with large pages on, eight clean builds each; the day's `base-lp` pool is labelled dirty and was not used) | — | large pages alone | **+0.08% (−0.11 .. +0.27)**, 8v8 | not shown on large pages |

Each shortcut carries a debug assertion against the computation it skips, shown
live on debug `bench 12` (7,417,463 nodes) by a mutant: A's two-step test against
the full one (the relieved margin widened to twice the relief fires it), C's
shared masks against a fresh computation (masks computed for the other side fire
it), L's probe against the sequential scan (the slot-2 select inverted fires
it). B, E, D and F skip no computation.

Not implemented, with the reason: the look-ahead TT prefetch for picked moves
(B, its cheapest form, read null, and under 4 KB pages every added prefetch
pays a page walk); the pawn-cache prefetch at the make (the night's one-line
pawn entry read +0.02%, so the entry's misses are not visible); the row
prefetch of the history rows before quiet scoring (the prefetch family below);
folding mobility and the long diagonal into the attack-map fill (at most 0.1%);
the TT move's legality from the pinned set computed for generation (about
+0.3% expected after the pin computation moves ahead of early cuts; the same
class as A and C). Lazy capture generation after the TT move would save the
capture generation at every node the TT move cuts (about 1.3M of 3.67M loop
nodes) but is not exact: the noisy scores read a history the TT move's subtree
changes. It is the one lead worth several percent and belongs to a gated search
change, not to C.12.

## Lessons

- **The instrument's floor, and what it leaves on the table.** Eight builds an
  arm resolve about ±0.25% at two cycles and ±0.2% at six. The protocol's
  acceptance bar, +0.5% with the lower bound above zero, was set for four-build
  reads; at eight builds two changes of +0.2% each summed to +0.41%
  (+0.28 .. +0.54), real by its interval and unbankable by the bar, while a
  self pair of eight-build pools read +0.10% (−0.06 .. +0.26) at six cycles,
  the size of either change alone. Whether the
  bar should read "lower bound above zero at eight builds and six cycles" is a
  PROCESS decision for the maintainer; nothing here changes it.
- **Software prefetching is closed under 4 KB pages.** The night's history and
  correction prefetches lost, the day's correction prefetch on large pages was
  flat, and here the earliest TT-move prefetch read null and the store-side
  prefetch lost 0.26%. The pattern: on this host a prefetch into a table beyond
  the TLB's reach triggers the page walk itself (the profile's 3.5% on the
  instruction after the pick-time prefetch), so an added prefetch costs about
  what a demand miss's walk would; only the pick-time TT prefetch, which
  replaces that walk, pays. Revisit the leads only with large pages.
- **Conditional moves are not free.** The selection scan's cmov paid because
  its chain was loop-carried and mispredicted at every new maximum. A select
  chain feeding the probe's consumers lost 2%: it lengthened the dependency
  path that predicted branches let the core run ahead of. And switching off
  LLVM's cmov-conversion pass for the whole crate lost 0.34%: its decisions are
  right on balance, the selection scan was the exception.
- **Layout: alignment is not worth an indexing level.** Allocating the history
  tables as aligned lines did save a third cache line per row but cost 2.9%,
  because the two-level index (shift, mask, two address terms, a line bounds
  check) lands on the hottest loads in the engine. An aligned flat slice needs
  either `unsafe` or a runtime offset into an over-allocated vector; neither was
  tried.
- **Small work reductions read as compute.** Two million saved calls of a
  10-20 ns function read +0.2%; the day's "flat" candidates were of this size.
- **A bundle can hide a loss.** The day's bundle at +0.15% contained a −0.58%
  commit. When a bundle is flat, read its members alone before concluding they
  are.

## Verification

On the final head (engine source `60fedc5`): the debug and release suites, clippy
in the default and the `rarog_pext` configuration, `cargo fmt --check`, the
`tune`, `texel` and texel-tuner builds and `bench 13` on both slider backends
(local `analysis/artifacts/c12c/final/`): the debug suite passes 395 tests in
32 binaries and the release suite 396 (the fuzz test runs in release only),
none failed, the allocation guard included; clippy reports zero warnings in both
configurations (re-run with the crate recompiled, since the first run replayed
cached diagnostics); fmt is clean; the three builds succeed; both backends bench
10,521,141 / EBF 2.521. `check_guide.py` and `check_citations.py` pass on the
documents. A final eight-build pool of the head read against `base8`, the
same engine source on both arms, reads +0.16% (−0.06 .. +0.39) at two cycles
and **+0.10% (−0.06 .. +0.26)** at six, closed NO_CHANGE as a self pair should:
the session's null, and the yardstick for every number above.

## Large pages delivered (2026-10-18)

At the maintainer's request the `large-pages` branch (`ac7c13b`'s patch on `90700b2`) was audited
and brought onto `c7` (`ac7c13b`, then the audit commit `34eb4bc`). Findings:

- The allocator is sound. `AdjustTokenPrivileges` reports a missing right
  through the last error, and the code reads it; the request size is rounded
  up to whole large pages; a region starts on a 64 KB boundary, which covers
  every alignment the engine asks for; release passes the region's base with
  `MEM_RELEASE`; the trait's default `realloc` allocates by the new layout and
  releases by the old one, so a reallocation across the threshold is routed
  correctly on both sides. A refused large-page request falls back to ordinary
  pages, so a user without the right, or a system short of contiguous memory,
  gets today's engine.
- Three changes. Every Windows API the crate calls is declared once, in one
  `unsafe extern` block (`src/win32.rs`), which also absorbs `main.rs`'s timer
  declaration: large pages add four `unsafe` sites instead of six, and the
  floor is 23 (it was 19). The privilege answer lives in one state atomic with
  the page size stored before it under release/acquire ordering, and a system
  that reports no large-page minimum counts as denied. A page allocation that
  asked for large pages and was refused is counted, and `bench` prints
  `partial` instead of `on` when that has happened.
- Tests. `tests/large_pages.rs` drives the allocator directly: a region at the
  threshold comes back zeroed, 64 KB aligned, writable to its last byte and is
  released; a small layout is served; a reallocation across the threshold
  keeps its bytes both ways. The tests pass with or without the right.
- Kept as it was: the default `alloc_zeroed` (a page allocation is already
  zero, so the default's clearing is start-up waste only, and overriding it
  would be one more `unsafe` site for no speed in play).

The remaining 23 sites: FFI 10 (the eight Syzygy sites, the shared extern
block, the privilege call), intrinsics 4, the measured hot path 7, and the
allocator's `impl` plus its two blocks.

**More speed on large pages.** Three candidates were built as pools on a side
branch and read with both arms on large pages, eight builds an arm, on an idle
host (the first attempt at two of them ran while the machine was in use and
was set aside as disturbed):

| Candidate | Base | Read | Verdict |
|---|---|---|---|
| T. The page-allocation threshold lowered from 2 MB to 512 KB, so the pawn history (786 KB) and the pawn cache (1.3 MB) join the large pages, at most 2 MB of locked memory each per search thread | the audited head | +0.58% (−0.11 .. +1.28) at 2 cycles; **+0.36% (+0.06 .. +0.66)** at 6 | under the bar; kept by the maintainer's rule for safe small gains (`47bc1ba`) |
| B. The TT move's child cluster prefetched before the noisy fill (the pass's candidate B) | T | **−0.58% (−0.89 .. −0.28)** (a first read flagged disturbed) | loss on large pages too; dropped |
| E. The node's own cluster prefetched before its closing updates, the cutoff store after them (the pass's candidate E) | T + B | +0.71% at 2 cycles; **+0.99% (+0.79 .. +1.19)** at 6, accepted | over B, so re-read alone |
| E over T alone | T | +0.53% (+0.26 .. +0.80) at 2 cycles; **+0.32% (+0.11 .. +0.54)** at 6 | under the bar; kept by the same rule (`039ff22`) |
| T + E over the audited head, the delivery's total on large pages | the audited head | +0.27% (−0.09 .. +0.62) at 2 cycles; **+0.28% (+0.02 .. +0.54)** at 6 | under the bar, positive by its interval |

So on large pages the store-side prefetch, a loss under 4 KB pages (−0.26%),
turns into a small gain, as the page-walk reasoning predicted, while the
TT-move prefetch does not. The parts read about +0.3% each and +0.28% together,
the difference being the pools' own offsets. The maintainer's rule for this
delivery, safe exact changes kept below the protocol's bar, keeps T and E; the
protocol's own verdict on each is NO_CHANGE and is recorded as such.

**In the games.** A 400-game fixed match at `3+0.03` (UHO, Hash 64, one
thread, seed 20261018, 14 games at a time, `tools/results/c12-lp-ingame/`)
of the delivered head (`tools/test_engines/rarog-c12lp-pext-pgo.exe`,
`59cceee`, large pages on) against the C.12 gate binary
(`rarog-c12speed-pext-pgo.exe`, `a829601`, no large pages): the head searched
**+2.76% nodes a second** pooled over about 22,400 moves a side (median per
game +2.14%) and +0.04 plies deeper, read by RAR-S99's script with the mover
taken from the position (164 of the 400 games start with Black to move;
`ingame_speed.txt` in the match directory). The speed reaches the games at
about bench's rate. The match's own result, −8.7 ± 18.0 Elo (102-186-112),
is a measurement with an interval, consistent with the +5 Elo the speed
predicts and with zero; 400 games resolve nothing in Elo and were not meant
to.

## Local artifacts

`analysis/artifacts/c12c/` (ignored): `worklog.md` (the timestamped log, the
ranked list written before implementation, the decision rule declared before the
reads), `prof/head/` (the sampled binary, its PDB, samples, `drill.txt`,
`lines.txt`), `diag-counters.txt`, `pools/` (`a`, `b`, `c`, `d`, `e`, `f`, `j`,
`l`), `reads/` (one directory and log per read above, with `raw.json`), and
`final/` (the closing suites). The scratch alignment probe and the drill script
are in the session's scratchpad only; their findings are recorded above.
