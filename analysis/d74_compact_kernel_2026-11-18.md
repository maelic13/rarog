# D.7.4: the compact kernel in the donor's form, behind one switch (2026-11-16 to 2026-11-18)

The kernel is Reckless `0756957`'s search written as Rarog's own code behind
the Cargo feature `compact-kernel`; the default build keeps the classic
kernel. This record holds the parity reads, the verification and the
independence figures. PLAN D.7.4 holds the decisions, the departures from the
commit order, the recorded items, the speed register and the squash
paragraph. The instrument is `analysis/d73_parity_instrument_2026-11-16.md`;
the handoff is `analysis/search_shape_inventory_2026-11-13.md` sections 3, 5,
7 and 8.

## Fingerprints

| Build | `bench 13` | Where |
|---|---|---|
| Classic, default features | **9,067,595 / EBF 2.479**, exact at every commit checked from `ef1048d` to `042e2b9` | the switch off |
| Compact at `cd4957a` | 6,072,606 / 2.426 | the 2026-11-16 reads |
| Compact at `856467e` (C-3, the donor's key) | 6,634,547 / 2.417 | equal to the 2026-11-16 kings diagnostic |
| Compact at `201e922` to `cb7c79b` | 5,506,554 / 2.409 | A2: the null bound reads a quiescence entry as the donor does |
| Compact at `a344748` to the head | **7,606,256 / 2.433** | a promotion push through the exchange |

The bench shares one table across its positions, so a compact count is a
fingerprint, not a tree size: `a344748`'s rise of 38% moved nearly every
position.

## Layer B: the four reads

The frozen rule (D.7.3): at depths 8, 11 and 14, |mean log ratio| ≤ floor +
0.03 + 3·SE against the twin's base run, the floor the largest of four salted
runs; each rate kind's mean difference in firings a position within
max(3·SE, 10% of the twin's rate); positions 206, 268, 310 and 446 excluded.

| Read | Depth 8 | Depth 11 | Depth 14 | Rates | Timeouts | Verdict |
|---|---|---|---|---|---|---|
| `cd4957a`, 2026-11-16 | +0.0078 ± 0.0253 | +0.0696 ± 0.0220 | +0.0954 ± 0.0247 | 34 of 36 (`research`, `see_margin` out by 1% and 7%) | 105, 310 | **FAIL** |
| Kings diagnostic, 2026-11-16, uncommitted, its diff not kept | +0.0059 ± 0.0257 | +0.0262 ± 0.0230 | +0.0312 ± 0.0247 | 36 of 36 | none | pass, a diagnosis, not a read |
| `cb7c79b`, 2026-11-17 | +0.0045 ± 0.0255 | +0.0278 ± 0.0238 | +0.0451 ± 0.0254 | 36 of 36 | none | **PASS** |
| `a344748`, 2026-11-17, the deciding read | −0.0098 ± 0.0257 | +0.0121 ± 0.0227 | +0.0188 ± 0.0226 | 36 of 36 | 446 (excluded set) | **PASS** |

The bands were ±0.127, ±0.120 and ±0.148 at `cb7c79b`, and ±0.128, ±0.117 and
±0.140 at `a344748`.

- **`cd4957a`'s read** was the decision read for that build and failed.
  Decision 1 of 2026-11-17 then took the donor's non-pawn key for its own
  reasons (PLAN D.7.4), not because the diagnostic passed.
- **`a344748`'s read** was declared before any result as the deciding read
  for the build that goes to D.7.5 (maintainer, 2026-11-17), the same rule
  unchanged; `cb7c79b`'s pass stays the read of that build.
- **Side figures, accepting nothing:**

  | Read | Summed positions at depth 14, compact / twin (496) | Rate band / twin's rate: median, largest | Timeouts against the twin's |
  |---|---|---|---|
  | `cd4957a` | 1.147 (the audit's) | 0.20, 0.44 | two; the twin times out on one to three a salted run |
  | Kings diagnostic | 1.277 (the audit's) | 1.14, 2.02 | none |
  | `cb7c79b` | 342,582,652 / 122,436,806 = 2.798 | **4.95, 12.92** | none |
  | `a344748` | 123,898,486 / 122,436,806 = **1.012** | 0.25, 0.50 | one, 446 |

  Depth 8 reads 0.973 and depth 11 1.128 at `a344748`; the five largest
  trees hold 30.5% of its depth-14 sum (184 19.3 M, 305 9.9 M, 473 4.7 M,
  318 2.0 M, 32 1.9 M), against 74.1% at `cb7c79b`.

**The rate rule's two limits.** The band is the larger of 3·SE and 10% of
the twin's rate. The 10% is a floor under the band, one-sided: it never
narrows below it, and it binds for two kinds in each of the last two reads.
The 3·SE limit has no ceiling, and a heavy tail in the per-position
differences widens it. At `cb7c79b`, position 305's 231 M-node tree put the
median band at five times the twin's rate, so that pass said little about
the rates; the depth rule, on log ratios, was not affected the same way. At
`a344748` the widest band is half a rate. The rule is frozen and decided as
it stands; a later instrument should bound the tail (a trimmed or
log-scale rate difference) before it is trusted on a tree with a runaway.

## Position 305: the SEE departure

- **First divergence.** At depth 1 the compact searched 256 positions and the
  twin 253; the quiescence SEE prune fired 0 of 2 times in the compact and 2
  of 2 in the twin, both on Black's d2-d1=Q. Scores agree at depth 1 once the
  units are converted (the twin prints `normalize_to_cp` of kernel units,
  the kernel's unit Rarog's × 2536/1024); `seldepth` does not compare (the
  donor raises it at PV nodes only).
- **Cause.** Rarog's threshold SEE answers a non-capturing promotion with its
  gain, queen less pawn, without playing the exchange, in both its forms. The
  donor's `see()` counts the same gain in `move_value` and then plays the
  exchange with the new queen at risk, so a push onto a defended square loses
  its pawn. The inventory's row SEE-2 to SEE-4 had it the other way round for
  SEE-4 (corrected there, 2026-11-18).
- **Fix** (`a344748`, maintainer "Yes, fix it now"): a third mode of the
  threshold SEE, `QuietSee::Exchange`, entered by
  `Board::see_ge_exchange_with_values`, plays every move through the
  exchange; the compact kernel's three SEE tests (the quiescence prune, the
  node's SEE margin and the picker's good-noisy split) use it. The classic
  kernel's two modes are unchanged, and so is its fingerprint. Test:
  `board::tests::the_exchange_test_puts_a_promotion_push_at_risk`; with the
  old answer planted back it fails.
- **Effect on 305** (depth 11, depth 14): the twin's base and four salts
  43,823–86,800 and 218,540–4,213,678; the compact at `cb7c79b` 1,042,224 and
  231,294,213 (88.8 s); with the fix 43,352 and 9,891,419 (4.0 s). Depths 1
  and 2 then equal the twin's exactly (253, 463) in every rule count both
  record.
- **Open.** A second divergence starts at depth 3: the first depth-3 root
  search returns −866 against the twin's −883, and depth 14 stays 2.3 times
  above the largest salted twin run. The other SEE departures (SEE-2, the
  donor's static pins only; SEE-3, no promotion gain on a recapture) are the
  first candidates. An input to D.7.8.
- **The classic kernel** uses the same shortcut. That is outside D.7, and
  only recorded.

## Positions 105, 305 and 310 at depth 14

On `a344748` (`layer-b2/three_positions.jsonl`), 120 s at most each; every
count reproduces layer B's exactly.

| Position | Compact | Twin, base | The twin's salted range | Reading |
|---|---|---|---|---|
| 105 | 593,373 (0.2 s) | 56,913 | to 2.45 M under salt 1 | inside the twin's own spread |
| 305 | 9,891,419 (4.0 s) | 3,434,165 | 218,540–4,213,678 | 2.3 × the largest; the depth-3 divergence above |
| 310 | 555,462 (0.2 s) | 1,936,298 | – | below the twin |

The brief asked whether each growth is extension chains or the window's fail
sequence. That run kept node counts, not info lines, so the attribution is
not made. At `cb7c79b` the 305 runaway was one depth-14 fail-low re-search
of about 195 M positions, and its cause was the SEE departure above, a
pruning difference, neither chains nor the window. The twin also fails low
often on 305 (5 to 9 windows a depth at depths 13 and 14).

## The trace and layer A

- **The twin, extended** (decision 6): its patch
  `tools/oracle-hybrids/reckless-0756957-rarog-hce-trace.patch`, SHA-256
  `ce0217e721ba89c1e214dfc3ff11a7ce1363893e524a64d3efe2476592d973ec`; the
  binary `4fe68f824aed141be2bc3bfd52b113832b6d96523fc3424165997152de5cd577`.
  Its bench is 4,158,758 with tracing off and on, and 20 of 20 stored base
  rows (every 25th of `f_base.jsonl`) reproduce exactly, nodes and rates: the
  twin's search is unchanged (B3).
- **Added inputs:** the stored capture refutation on `nmp_gate`, whether the
  exclusion search reached a move on `singular_result`, the node type on
  `q_stand_pat`, and the null search's outcome on `nmp_result`, which made
  `nmp_verify_depth` checkable. The parity test's `either()` and inferred
  verdict are deleted.
- **Eleven new kinds:** `draw_score` 154,535 records, `eval_bonus_gate`
  92,584, `malus_fade` 6,118, `null_bound` 3,418, `pick_threshold` 48,371,
  `probcut_value` 4,078, `q_cutoff_bonus` 26,389, `score_held` 3,500,
  `spread` 121,252, `tt_cut_bonus` 11,108, `tt_cut_return` 11,108. The six
  seeds read outside a rule now sit in pure rules a kind checks.
- **Capture** (B5): 250 positions, stride 512, no timeout, the same
  exclusions; the 36 old kinds' counts equal d73's. Combined SHA-256 (sorted
  per-file hashes, d73's method, which reproduces d73's own value)
  `1480a5ec363726ee5a00fa1b04f969d87e77cc56d41e5c0e12e3b594affafb14`;
  `summary.json` `63a47e89ae754ade136799313554b84039a167e4da929dace00a64b585d04e04`;
  354 MB in ignored storage.
- **Layer A** (`layer_a4.log`, at `042e2b9`): **47 kinds, 3,269,262 records,
  0 mismatches**; 40 plants, each failing its own kind and no other.
- **Seed coverage.** Each of 239 seed values is moved one step up and one
  down in turn (`cb7c79b`); a seed is detected when some record changes.
  Ten are named in the test with their reasons:
  - `fds_singular`, its offset and cap: dead in the donor itself. A singular
    search needs depth 5 and from depth 2 every move after the first is
    reduced, so the full-depth search runs only for the first move, before
    its score exists; none of 62,853 records has both.
  - `lmr_spread_thread`, `fds_spread_thread`: zero in a one-thread trace.
  - `pick_escape[0]`, `pick_escape[5]`: the donor gives pawns and kings no
    escape bonus.
  - `double_new_pv`, `triple_pv`, `triple_new_pv` (`042e2b9`, maintainer
    "Name them as sparse"): read only at a PV node whose singular search
    failed low; the trace holds 47 such records, 11 at a new PV node, none
    within a step of a margin. Sampling, not structure: a denser capture (about
    12 GB at stride 16) would test them.

## MultiPV and pricing

- **MultiPV** (decision 4, `cdda460`): under the compact kernel each line
  runs the donor's root window with its own running average, line 1's
  stability counts setting every line's base width, optimism a line, the
  stack reset and the fail-high reduction. Each line opens its window fresh;
  the donor carries the width and the reduction from one line to the next
  within an iteration, which would search line 2 a ply shallower than it
  reports. That point is the maintainer's to overrule. Bench position 11 at
  MultiPV 2, depth 13: 2,558,304 positions under the classic window,
  1,457,794 under the donor's; a test fails above 5,000,000. The classic
  driver and the one-line path are unchanged.
- **Pricing** (decision 5, `934213d`; `analysis/pruning_pricing_card_2026-11-04.md`'s
  contract): the forced store goes through the probe's redirection, the
  compact correction update is frozen in a probe, the probes' re-searches run
  the kernel that is switched on, and a plant moves one seed of the measured
  search while the probe reads the donor's table (pricing builds only).
  (a) The compact bench with pricing and sampling on equals the build
  without it, 5,506,554. (b) Identity: RFP 0 of 11,341, ProbCut 0 of 9,456
  and multi-cut 0 of 8,545 decisions missed; the null move 79 of 11,278
  (0.70%, upper bound 0.87%) and razoring 45 of 10,781 (0.42%, 0.56%), inside
  the card's 1% rule; the cause is not found (the child cutoff count, the
  node-count phase and the quiescence history reward are ruled out). (c) A
  planted futility constant (127 to 427) reads 2.68% against the honest
  1.73%; an RFP plant read 4.51% against 4.36%, too close to call. (d) The
  classic kernel under pricing benches 9,067,595 and its release suite
  passes. (e) Clippy is clean in every build. A crash at ply 127 in the new
  code was found and fixed before its commit.

## Verification

At the final engine state (`a344748` plus `042e2b9`'s test list):

| Check | Result |
|---|---|
| `cargo fmt --check` | exit 0 |
| `cargo clippy --all-features --all-targets` | exit 0, 0 warnings |
| `cargo test` (debug) | 557 passed, 0 failed, 13 ignored |
| `cargo test --release` | 559 passed, 0 failed, 12 ignored |
| Classic bench, fresh default release build | 9,067,595 / 2.479 |
| Compact bench, fresh `compact-kernel` build | 7,606,256 / 2.433, `id name` `dev-20261117-a344748d` |
| Layer A | 47 kinds, 0 mismatches, seed coverage as named |
| Allocation guard, search-path perft | in the test suites above, the guard over both kernels |
| Assembly (`--emit=asm`, `compact-kernel`, 2026-11-18) | no `memcpy`, `memmove`, `memset` or allocation in `compact_negamax` (both node types), `compact_quiescence` (both), `CompactPicker::next`, `compact_best_move_updates`, `compact_fail_low_reward`, the correction read and update, `compact_eval` or `see_ge_impl`; frames 4.6–5.1 KB |

**The root window's copies.** `search_root_window` copies about 4 KB three
times (4,104, 4,104 and 4,108 bytes; frame 13,512 bytes) where `274adaa` copies
once (frame 9,336): the compact root picker is moved by value. That is once a
root window, not a node, so it stays, and PLAN's speed register holds it.

**`cargo test --release --features compact-kernel`** (the kernel the default):
541 passed, 18 failed at `a344748`, the same 18 names as before the SEE fix;
none shows a compact defect. The one compact defect the suite found, root
records stamped with the reduced depth (`e81227e`), is fixed, and a test
fails without the fix. Each failure pins a classic form:

- **Classic node tests and classic-only forms (8):**
  - `a_fail_low_store_carries_its_best_scoring_move`: T8, dropped.
  - `a_null_move_adds_no_count`, `the_root_never_takes_a_null_move`,
    `the_verification_region_is_cleared_after_verifying_and_at_search_start`,
    `singular_decisions_run_in_a_middlegame_search`,
    `each_recursive_call_types_its_child_by_its_rule`: hooks only the classic
    node has.
  - `no_line_searches_past_twice_the_iteration_depth`: N6, dropped.
  - `no_line_spends_more_than_the_iteration_depth_on_extensions`: LR4,
    dropped.
- **Windows and info-line counts (6):**
  - `a_depth_one_search_counts_each_root_move_once`,
    `a_node_budget_waits_for_the_first_iteration`,
    `a_clock_spent_before_the_search_began_still_returns_a_first_iteration_move`,
    `search_writes_one_info_line_per_iteration_through_its_sink`,
    `a_proved_mate_ends_a_clock_search_after_three_iterations_but_not_analysis`:
    the donor's window opens from depth 1.
  - `a_root_fail_high_past_the_soft_target_plays_the_move_with_its_lower_bound`:
    its scan expects a classic fail high.
- **`every_search_parameter_moves_what_it_tunes` (1):** the compact kernel
  reads its seeds, not `SearchParams`.
- **`search_populates_persistent_root_move_records` (1):** a PV of length 1,
  because the donor's fail-high reduction searched depth 1 at a reported
  depth of 3.
- **`a_stored_win_never_answers_a_position_repeating_the_game_history` (1):**
  N5's twofold draw, dropped for the donor's rule.
- **`bare_king_promotion_choice_in_search` (1):** the promotion anchor of
  MP-1, dropped; D.7.8 asks D.6.17's question. D.6.17's and D.6.20's
  acceptance tests are ignored tests and were not run here.

No test was relaxed. The 18 are D.7.9's to retire or rewrite with the old
kernel.

## Speed

- **G1, the switch off** (`274adaa` against `cb7c79b`, four pext PGO builds
  an arm, every copy at 9,067,595, `tools/nps_read.py --no-regression`, run
  by the maintainer 2026-11-17): step 1 −0.29% (−0.72 to +0.14); step 2, six
  cycles, **−0.34%** (95% −0.63 to −0.04), no cycle disturbed: **PASS, no
  regression** by the method's rule. `analysis/artifacts/d74-nps-2026-11-17/g1-read/`.
- **G2, the compact kernel against the classic head** (compact pool at
  `1ad7eca`, engine source equal to `cb7c79b`, every copy at 5,506,554, two
  cycles): **+11.54%** nodes a second (95% +11.20 to +11.87), no cycle
  disturbed. The trees differ, so this is speed a node, not time to a depth.
  `g2-read/`. Not re-read at `a344748`.
- Two tooling defects found by G2 and fixed: xtask refused a feature with a
  hyphen (`df01401`), and its PGO training check looked only for the classic
  kernel's functions, which a compact build never runs (`1ad7eca`).

## Independence

- **Measured by the audit at `cd4957a`, not re-measured:** 54 of the compact
  `node.rs`'s 539 code lines are identical to a donor line, against 64 of the
  classic `node.rs`'s 1,460.
- **Transcribed, each named in its commit:** the interpolation toward beta,
  an `f32` `mul_add` truncated (`3bda2c3`); the cuckoo tables of a repetition
  one move away, van Kervinck's method (`efbbb69`).
- **Following the donor's statement order** (decision 3): in
  `src/search/compact/node.rs`, the ProbCut loop, the three searches of the
  move loop (the reduced search, the full-depth search, the PV search) and
  the node's tail (the best-move updates and the store); quiescence's
  order; and `Board::is_dead_material` in `src/board/board.rs`. D.11 names
  them.
- The trace stays in ignored storage; the parity test compares decisions on
  the same inputs, not code.

## Artifacts

`analysis/artifacts/d74-parity-2026-11-16/` (the 2026-11-16 reads and both
binaries, `rarog-compact-probes-cd4957a.exe` SHA-256
`256857235efd0aba698893277c1de9fdf06519b18fe2f8d1a66940eb25a116c0`), and
`analysis/artifacts/d74-parity-2026-11-17/` (ignored):

- the twin's tree and `make_trace_patch_v2.py`; B3's check (`b3_*`);
- `trace-a/` and the layer A logs (`layer_a*.log`);
- `layer-b/` (`cb7c79b`, binary `6a432e8d…33ea0`) and `layer-b2/` (`a344748`,
  binary `86d095ce037ef76e483199d983d8a1d890745332d59e14f294284df41cb6d8a8`),
  each with its bench, runs, filtered rows and comparison;
- `p305/`: the salted twin runs on 305, the per-depth rates and the
  diagnostic SEE build (`a3fb2af4…`);
- `pricing/` (phase E's validations) and `F/` (the compact suite's logs);
- `analysis/artifacts/d74-nps-2026-11-17/`: G1 and G2, their scripts and reads.
