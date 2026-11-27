# Donor inventory by licence, at the release head (D.11.1, 2026-11-27)

What Rarog holds from other engines at the release head, by donor, by the
licence of the version it was taken from, and by kind, over everything
tracked: source, tools and documents. PLAN D.11.1 owns the step; D.11.2
decides on it and D.11.4 publishes it in Coda's form
(`analysis/coda_read_2026-11-23.md`, *How Coda discloses its sources*). It
reruns the 2.6.0 review (`analysis/originality_review_2.6.0_2026-10-22.md`)
on the head and widens it. Not legal advice.

Evidence layer: source reading and scans of source text. No build, no game,
no change to `src/`. Every donor was read at a named commit with
`git -C <donor> show <hash>:<path>` or `git -C <donor> archive <hash> src`,
never from a working tree. No Reckless code is quoted here: its findings are
given by path, count and identifier name.

## The head read

- Rarog: branch `d11` at `e621f78`, whose `src/` is byte-identical to
  `master` at `db51df6` (`git diff --quiet db51df6 e621f78 -- src`, exit 0).
  The releases compared: `v2.5.0` (`855a092`), `v2.6.0` (`0127e58`),
  `v2.7.0` (`3c57a89`).
- The principle applied, as Coda's: ideas, algorithms and functional
  constants are not protected expression; source text, names and structure
  are. Each finding says which it is, and the licence of the donor at the
  version it came from.
- The rule since 2026-11-23 (PROCESS, *The independence boundary*): GPL-3.0
  kept, Stockfish the primary donor, Reckless ideas only, and what the tree
  already holds from Reckless is this inventory's to list and D.11.3's to
  resolve.

## The licence landscape

| Donor | Version read | Date | Licence at that version | What Rarog took from it |
|---|---|---|---|---|
| Reckless | `c3c89eae3` | 2023-02-07 | none (before any `LICENSE`) | an idea read for a stalemate score; nothing in the code |
| Reckless | `b00fa6ca2` to `a0a239741` | 2023-04-16 to 2025-02-22 | MIT | nothing: Rarog began 2026-05-22 |
| Reckless | `a0a239741` to `29d9f6c0c` | 2025-02-22 to 2026-01-26 | none (all rights reserved) | nothing |
| Reckless | `12af143`, `dd0e676`, `45ea6a9`, `91b56c2`, `31d9cd6`, `ffe556d`, `aec5cb2` | 2026-05-31 to 2026-10-28 | AGPL-3.0 | B.2's move ordering, history, correction and TT-cutoff constants (2.5.0 to 2.7.0); readings |
| Reckless | `0756957` | 2026-11-07 | AGPL-3.0 | the compact kernel's shape, its seeds, the clock (D.7) |
| Reckless | `8d7d1dd` | 2026-11-13 | AGPL-3.0 | a profiled reference build; nothing in the code |
| Stockfish | `sf_11` (`c3483fa9`), `9587eeeb` | 2020-01-15, 2020-07-30 | GPL-3.0 | the classical evaluation's design; the king-safety fixtures |
| Stockfish | `sf_16` (`68e1e9b3`) | 2023-06-24 | GPL-3.0 | the donor-evaluation bridge (E.4.0.1); a reference opponent |
| Stockfish | `sf_19` (`edb0d9db`) and development commits to `1e7e6bde` | 2026-07-11 to 2026-11-14 | GPL-3.0 | mechanisms and behaviour; the reported score's form (D.7.12) |
| Triumviratus | `0c18907` (networks `5f3dc60`) | 2026-11-03 (2026-10-28) | GPL-3.0 | measurement methods behind D.4's instruments; nothing in the code |
| Fathom | vendored in `c7afa1c` | 2026-05-25 | MIT | the Syzygy probing code, compiled into every binary |

How each licence was read:

- Reckless: `git -C D:/code/Reckless log --follow -- LICENSE` gives three
  changes: created `b00fa6ca2` (MIT), deleted `a0a239741`, AGPL added
  `29d9f6c0c`. `git show <hash>:LICENSE` reads AGPL-3.0 at all nine cited
  commits from `12af143` to `8d7d1dd`, and no file at `c3c89eae3`.
- Stockfish: `Copying.txt` reads GPL-3.0 at all fifteen distinct cited
  commits in the local clone. Three cited commits are not in it (`af110e02`, `d2d046c2a`,
  `5f3dcb92e`); their licence was not read.
- Triumviratus: no local clone. Both repositories' `COPYING` were read from
  the ignored snapshot (`analysis/artifacts/triumviratus-reference/`): GPL-3.0,
  SHA-256 `3972dc97…` in each, equal to the pinned manifests in
  `analysis/triumviratus_snapshot/`, which a blobless clone verified file by
  file on 2026-11-03.
- Fathom: `vendor/fathom/LICENSE`, MIT, copyright Ronald de Man (2013-2018),
  basil00 (2015) and Jon Dart (2016-2025).

So everything Rarog took from Reckless for its code came from AGPL-3.0
versions: Rarog began four months after Reckless's licence became AGPL.

## Reckless (AGPL-3.0): what the head holds, by kind

### 1. The kernel's shape (shipped)

The compact kernel (`src/search/compact/`, the only search since `d23b9cf`)
was rebuilt from D.7.0's inventory of Reckless `0756957`: its node, its
quiescence, its staged picker, its window and the rule set in `rules.rs`
follow Reckless's search, by design (PLAN D.7, *Independence*). The shape is
architecture and mechanism, an idea in the principle's sense; it is also the
one point where a derivative-work argument rests on the whole rather than on
a part.

The text is Rarog's own, with one exception that is new since 2.6.0.
`tools/diag/originality_scan.py` against Reckless `0756957`:

- Raw shared runs of 20 or more tokens: 32 runs, 904 of 216,271 tokens
  (0.42%; 2.6.0 read 458 of 135,077, 0.34%). Six of the 32 are artefacts of
  the scan, which blanks strings: lists of quoted names matched Reckless's
  bench list.
- The positive control, rerun on these trees: Reckless `ffe556d` against
  `0756957` shares 77,987 of 78,254 tokens (99.7%) in raw runs, so the scan
  sees copied text.
- Of the other 26, eleven are in the kernel, against Reckless's `search.rs`:
  nine in `src/search/compact/node.rs` (255 of its 8,000 tokens, 3.19%; the
  longest 43 tokens) and two in `quiescence.rs` (53 of 1,683, 3.15%). They
  are fragments of the move loop, a few lines each, and they keep Reckless's
  identifiers: `quiet_moves` and `noisy_moves` filled under the same
  `move_count < 32` test, `tt_move_score`, `last_critical_ply`, the
  extension added to `new_depth` on the first move only, the principal
  variation re-search's depth floor, the mate-distance clamp, and the
  condition for the fail-low reward. At 2.6.0 every shared run was
  boilerplate.
- The rest are boilerplate as at 2.6.0: bitboard operator impls
  (`src/board/bitboard.rs`, four), the colour enum (`piece.rs`, two), FEN
  writing (`fen.rs`, `recogniser_tests.rs`), the board's text display
  (`board.rs`), a fixed-capacity list
  (`moves.rs`), the stack entry's default (`src/search/stack.rs`, the same
  type name as Reckless's, `StackEntry`), `is_decisive` (`score.rs`), the
  node-type markers (`src/search/node.rs`) and the quiet-history update's
  signature (`history.rs`).
- With identifiers normalised (40 or more tokens), the 34 runs are
  declaration lists, struct fields and option tables, as at 2.6.0: no
  control flow.

Reading: the kernel was not copied, but its move loop is closer to
Reckless's expression than any part of 2.6.0 was. The fragments are short
and generic in what they do; they are identical in naming. Under the rule of
2026-11-23 they are what a rewrite in Rarog's own names removes, a
behaviour-neutral change.

### 2. The clock (shipped)

Reckless's time manager whole (`38e947c`, D.7.9.1), checked against
`0756957`'s `src/time.rs` and `src/search.rs`:

- The budget (`src/search/time.rs`, `budget`): the formula and all seven
  constants are Reckless's: the 15 ms overhead, the soft scale's three
  constants 0.0594, 0.0492 and 0.0386, the hard scale 0.7281, the increment
  share 0.75 and the cyclic hard multiple 5.0. The expression is Rarog's
  own (named constants, its own structure and guards).
- The soft limit (`src/search/params.rs`, `tm_*`): Reckless's five factors
  and their fifteen constants, fourteen at Reckless's values
  (3.1838, 2.6554, 0.546, 0.7426, 0.7214, 1.4031, 1.2881, 0.044, 0.716,
  1.2664, 0.0416, 0.8642, 1.15, 0.2526) and the score-trend slope converted
  into the kernel's units (0.0480 × 2,536 / 1,024 = 0.1189).
- Rarog's own: the reserve under Move Overhead and its four-thread extra,
  the proved-mate stop, the first-iteration guard, the tablebase box, the
  fail-high stop (2.0, Rarog's seed), movetime and the thread vote.

The twenty-two are the clock's twenty-three of
`analysis/d711_fit_2026-11-20.md`, *What stays unfitted*, less the
fail-high stop. None has been fitted on Rarog.

### 3. Constants still at the donor's values (shipped)

The kernel's shipped seed table, `Seeds::FITTED` in
`src/search/compact/rules.rs`, against `Seeds::DONOR`, the donor's values
converted into Rarog's units (scratch `seed_compare.py`): **32 of 251
scalars are still the donor's**:

- the two thread spreads, `lmr_spread_thread` 27 and `fds_spread_thread` 26;
- the unreached and floor seeds that D.8.21 owns, among them `rfp_min_16`,
  `nmp_min_16`, `window_stability_cap`, `tt_cut_fifty` and
  `pick_check_base`;
- depth thresholds and small counts the fits held or returned to
  (`eval_bonus_depth`, `probcut_depth_base`, `singular_tt_depth`,
  `ldse_depth`, `fp_depth`, `bnfp_depth`, `q_move_limit` and others);
- one value of 150 or more, `fds_log` 207, and the pawn's exchange value 109.

The other 219 moved in RAR-S128 and RAR-S135's fits. Among the shipped
values of 150 or more, 17 of 143 occur somewhere in Reckless's source, and
12 of 143 in Stockfish 19's, the chance rate: only `fds_log` is a donor
value.

Outside the table: the ply cap 240 (`src/score.rs`, `91bc561`,
2026-11-22), Reckless's maximum ply (Stockfish's is 246), a functional bound;
and the exchange bar's scale 2,536, Rarog's own measurement of Reckless's
evaluation units (0.4038), not a Reckless constant.

### 4. The two transcribed places (shipped)

Named in their commits:

- `3bda2c3`: the interpolation toward beta in single precision with a fused
  multiply-add, truncated toward zero, the rounding the seeds were fitted
  with (`rules.rs`). A numerical form, functional.
- `efbbb69`: upcoming-repetition detection by van Kervinck's method
  (`src/board/cuckoo.rs`), a published algorithm with one natural form,
  which Stockfish (GPL-3.0) also implements; its null-move window follows
  Reckless's.

### 5. B.2's constants in the releases (published, gone from the head)

`tools/diag/originality_constants.py` on each release's `src/search/` against
Reckless `ffe556d` and `0756957`, non-round literals of 150 or more outside
`params.rs` and tests:

| Release | Hard-coded literals | In Reckless | In Stockfish 19 | By file, in Reckless |
|---|---|---|---|---|
| 2.5.0 | 50 | 37 | 11 | movepick 14, history 15, node 6, correction 2 |
| 2.6.0 | 47 | 37 | 8 | the same 37 |
| 2.7.0 | 63 | 45 | 17 | the same 37, and 8 standard perft counts in `path_perft.rs` (chance) |

The same 37 constants, adopted at B.2 with their control flow, shipped in
2.5.0, 2.6.0 and 2.7.0. They left the head with the classic kernel
(`d23b9cf`). The tuned defaults in those releases' `params.rs` are at the
chance rate (23, 18 and 18 of 84 to 88 in Reckless; 16, 14 and 15 in
Stockfish 19).

### 6. Reckless material in tracked files that do not ship

- **The three oracle patches**, `tools/oracle-hybrids/reckless-0756957-*.patch`:
  1,222 lines in all, of which 465 context and 21 removed lines are
  Reckless's own source and 596 added lines modify Reckless. They are AGPL
  material. `tools/oracle-hybrids/README.md` now names both licences and
  says the patches ship nowhere; the patch files carry no notice. They are
  in published history since PR #20.
- **The donor seed table**, `Seeds::DONOR` (`#[cfg(test)]`, `rules.rs`): 251
  scalars, Reckless's values in Rarog's units; 94 of its 139 values of 150 or
  more are Reckless literals unchanged (dimensionless seeds convert to
  themselves). With it `src/search/compact/parity.rs` (`#[cfg(test)]`) holds
  Reckless's two correction saturations, 16,418 and 14,605. Neither is in a
  binary, but both are a tracked list of the donor's constants, which the
  rule of 2026-11-23 bars. The parity test already reads its Reckless trace
  from ignored storage.

### 7. Comments that name Reckless

Six lines name Reckless explicitly, all accurate references to behaviour:
`src/search/stack.rs:14` (its `PlyArray`), `src/search/mod.rs:729` (how it
counts nodes), `src/search/options.rs:119` (when it stamps the start time),
`src/search/time.rs:166` and `time/tests.rs:110` (its movetime default),
`tests/multipv.rs:174`. The unnamed "donor" comments are under *Source
comments* below.

## Stockfish (GPL-3.0): what the head holds

GPL-3.0 is Rarog's own licence, so each item needs naming, not resolving.

- **The classical evaluation's design** (`src/eval/`): Stockfish 11's term
  set, king danger, shelter and storm, threats, passed pawns, quadratic
  imbalance, initiative and endgame rules, with the forms fitted from
  `9587eeeb`'s king-safety fixtures. The weights are Rarog's own:
  `tools/diag/originality_eval.py` puts 14 of the 75 live evaluation literals
  of 150 or more in Stockfish 11's evaluation files and 14 in its other
  files, the chance rate (2.6.0: 12 of 58). The scan's only long numeric run
  against Stockfish 11 is the test fixture of the shelter and storm tables in
  `src/eval/king/tests.rs` (29 values), not compiled into the engine.
- **Endgame knowledge after Stockfish 11:** the KRPKR rule
  (`src/eval/endgame/mod.rs`, Stockfish 11's case analysis), the KPK
  bitbase by the retrograde method Stockfish uses (Rarog's own generator,
  `kpk_generate.rs`), and the twenty endgame families of the last
  hand-crafted dispatcher (`src/diag.rs`).
- **The reported score** (D.7.12, reference `1e7e6bde`): the win-rate form
  and `UCI_ShowWDL`, with Rarog's own constants (`analysis/d712_score_2026-11-21.md`).
- **Mechanisms named in comments:** quiescence's stalemate test in its
  narrow form (`64993ff`), the poll after a main-thread probe (`1b5f9d1`),
  the selective depth reset per iteration, the best thread's `bestmove`
  output, the node count, the CPU tier exception, the development build's
  name. Ideas or functional conventions.
- **Stockfish's old time budget** (2.6.0's seventeen constants) left the code
  with the classic kernel: none of its constants is in `src/`.
- **Tracked patches** (Stockfish's source in their context lines, GPL-3.0):
  `tools/oracle-hybrids/stockfish-def601cc-rarog-hce.patch` (265 lines),
  `tools/donor-eval/sf11-bridge.patch` and `sf16-bridge.patch` (179 and 216),
  `tools/diag/patches/sf9587_king_subterms.patch` and
  `sf9587_plain_maps.patch` (201 and 20). The two READMEs beside the first
  three name the licence; `tools/diag/patches/` has no note.
- **Fixtures generated by Stockfish** (data, not code):
  `tests/data/king-danger-9587eeeb-plain.tsv` and
  `king-shelter-9587eeeb.tsv`, 500 positions each.
- **Refs, not the tree:** the three `oracle/*` tags carry a vendored
  Stockfish tree (GPL-3.0).

## Triumviratus (GPL-3.0)

Its measurement methods shaped D.4's instruments (the search-path perft, the
pricing of pruning and reductions) and several of its leads were read
(`analysis/triumviratus_review_2026-11-03.md`). No source file names it
(`git grep -i triumvir -- src tests` is empty) and no code or tuned value of
it is in the tree. Nothing to resolve; the disclosure names it for methods.

## Fathom (MIT) and other third-party data

- `vendor/fathom/` (Fathom's `tbprobe.c`, `tbchess.c` and headers, one local
  patch for UTF-8 paths on Windows) is compiled into every binary by
  `build.rs`. MIT requires its copyright and permission notice in every
  copy: the release archives must carry `vendor/fathom/LICENSE` (D.15.1).
  README does not name Fathom.
- The upstream Fathom commit vendored is not recorded (`c7afa1c` added the
  files; the notice's dates run to 2025).
- `tests/fixtures/syzygy/` holds seven files of the standard Syzygy set,
  generated data with their provenance in that folder's README; `src/wac.epd`
  is the public WAC suite. Neither is a donor.

## Tracked documents: donor text and citations

- **Citations by path:** 65 tracked files outside `src/` name a donor source
  file (a Stockfish C++ file, or a `.rs` path beside "Reckless"), by the
  scratch scan `donor_text.py`; 18 of them give line numbers. A citation is
  not a copy. (The 2026-11-23 count of fifteen was narrower; this one counts
  every path.)
- **Quotation:** every inline code span of 40 or more characters on a line
  naming a donor, every fenced block, and every span using a donor's own
  idiom (`ss->`, `thisThread`, `bestValue`, `td.`, `Score::`) was read. None
  quotes donor code beyond a single expression of a few tokens, such as a
  condition or a formula; the fenced blocks near a donor's name are Rarog's
  own code and commands. The candidates are what these patterns select, not
  a reading of every line: prose that paraphrases donor code without a
  code span or a donor's name nearby is not covered.
- **Prose about Reckless:** the D.7 analyses describe Reckless's search in
  detail, as mechanism and inventory, without its code. Whether such study
  material stays public is D.11.2's open question.

## Source comments that name a donor, checked against the code

- 82 comment lines in `src/` and `tests/` say "the donor" without naming
  one, and its meaning changes with the file: Reckless in the kernel, the
  clock and the table (`compact/`, `time.rs`, `params.rs`'s clock block,
  `score.rs`, `tt.rs`, `see.rs`); Stockfish 11 in `src/eval/params.rs` and
  `eval/king/tests.rs`; Stockfish's development tree in `tb.rs:266` (its
  `callsCnt` reset, `analysis/d9_tablebase_policy_2026-11-25.md`) and
  `root.rs`. A reader cannot tell which donor is meant.
- Checked against the code, the claims hold: "the donor's budget" and its
  constants are Reckless's (above); "the donor's 240" is Reckless's maximum
  ply; "a seed no fit moved keeps the donor's" holds for the 32; the
  correction saturations in `parity.rs` are Reckless's.
- The one that overstates: `src/search/compact/mod.rs:1`, "The search kernel,
  in its donor's form", which reads as a port of unnamed code when the text
  is Rarog's own apart from the fragments in *1*.
- No comment in `src/` calls anything verbatim, ported, copied or
  transcribed, which was Coda's finding. Two commit messages say
  "transcribed" and "followed as such" (`3bda2c3`, `efbbb69`), accurately.
- The rule of 2026-11-23 asks for "a comment that credits the idea, never a
  port": the 82 lines are what D.11.3 rewrites, naming the donor and the
  idea.

## What this inventory does not cover

- The move-similarity screen (D.11 item 1) was not rerun: it needs a
  release-head binary and an idle host, and no decision before D.11.4 uses
  it. The search changed since 2.6.0, so D.11.4's disclosure should not
  quote 2.6.0's 54.2% for the head without it (`tools/diag/sim_test.py`,
  as in the 2.6.0 review, with the head's binary).
- Token runs shorter than 20 are not measured; the kernel's resemblance below
  that is not known.
- Three Stockfish commits outside the local clone; Fathom's upstream commit;
  Triumviratus read from its snapshot, not by `git show`.
- Engines read as references or played as opponents that are not donors
  (Basilisk, Manta, Coda, Trinity, PlentyChess, GyatsoChess, Ethereal,
  Berserk and the gauntlet engines): their licences are not inventoried.
- The git history is not inventoried commit by commit; it holds the Reckless
  patches and the B.2 constants in published commits.

## Recommendation for D.11.2 and D.11.3 (the maintainer's decision)

Route 2 is decided. What it must resolve, cheapest and clearest first:

1. **The three Reckless patches** leave the tree, untracked with
   `git rm --cached` as D.11.3 says. They are the only Reckless source text
   in the tree. I recommend doing this first and alone.
2. **`Seeds::DONOR` and `parity.rs`'s two constants** move to ignored
   storage beside the trace the parity test already reads, so the test keeps
   working on this host and the tree lists no Reckless constant.
3. **The kernel's text fragments** (*1*) are rewritten in Rarog's own names
   and order, behaviour-neutral, `bench 13` exact (`I1`).
4. **The 82 "donor" comments** name the donor and the idea, with
   `compact/mod.rs:1` first; behaviour-neutral, in the same engine commit
   as 3 or its own.
5. **The clock** (22 constants and the formula): the largest functional
   dependence. I recommend replacing it with Stockfish's time manager,
   seeded from Stockfish's constants, which PROCESS allows for a GPL donor,
   rather than refitting Reckless's form from a derivation of Rarog's own: it
   is cheaper, and a refit of Reckless's shape keeps its formula. Either is a
   strength change with its own registration and gate.
6. **The 32 seeds at the donor's values**: the eight unreached at D.8.21 as
   planned; the two thread spreads at G.1; the rest are depth thresholds and
   small counts that a fit held. I recommend one registered refit of the 32
   from starts not the donor's, after the clock, not 32 experiments.
7. **The kernel's shape** stays, disclosed as Reckless's architecture: a
   rewrite from ideas would be a new kernel, and the cost is not warranted
   while the text is Rarog's own.
8. **The releases 2.5.0 to 2.7.0** stay as published, and the disclosure
   names B.2's 37 constants in them; the git history stays and is said to,
   as Coda's does.

D.11.2's other questions stay the maintainer's: the README line on how Rarog
is developed, whether the study material (items 1 and 2, the D.7 analyses)
stays public, and the history.

## Recipe

From the repository root, trees exported into the ignored
`analysis/artifacts/d111-inventory-2026-11-27/trees/` with
`git -C <repo> archive <rev> src | tar -x -C <dir>` for Rarog `HEAD`,
`v2.5.0`, `v2.6.0` and `v2.7.0`, Reckless `0756957`, `ffe556d` and
`8d7d1dd`, Stockfish `sf_11` and `sf_19`; then:

```sh
A=analysis/artifacts/d111-inventory-2026-11-27; T=$A/trees
python tools/diag/originality_scan.py $A/scan-head $T/rarog-head reckless0756957=$T/reckless-0756957 sf19=$T/sf19 sf11=$T/sf11
python tools/diag/originality_constants.py $A/const-rarog-head.tsv $T/rarog-head "src/search/*" recklessffe556d=$T/reckless-ffe556d reckless0756957=$T/reckless-0756957 sf19=$T/sf19
python tools/diag/originality_eval.py $A/evaltrees $A/eval-head.tsv
python tools/diag/originality_scan.py $A/scan-control $T/reckless-ffe556d reckless0756957=$T/reckless-0756957
```

`originality_constants.py` ran the same way on each release tree, and
`originality_eval.py` on `evaltrees/` holding the head as `rarog/`,
`sf_11` as `sf11/` and Reckless `0756957` as `reckless/`. Every run exited 0.
The filters and readers are scratch scripts kept beside the outputs in
`$A/scripts/` (SHA-256 prefixes: `big_consts.py` e686d52109287d9b,
`seed_compare.py` e6e5aef2e90765ec, `run_cover.py` f23a7105109f0c88,
`show_runs.py` b4050a8a08c3796b, `donor_text.py` e19bfc137d2bd624,
`donor_spans.py` f091c44e7dcba66d, `licence_at.py` 05ddf852a276d2f1); each
is a few lines over the tracked scripts' output, described where its number
is used above.
