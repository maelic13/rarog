# Rarog Strength And Speed Improvement Plan

This plan converts the current engine analysis and the comparison with Reckless
(`codedeliveryservice/Reckless`, inspected at commit `5dd029e`, 2026-05-31) into
implementation-ready steps.

Scope constraints:

- Do not add NNUE.
- Do not add NUMA-specific behavior.
- Do not add new public UCI options.
- Treat Reckless ideas as design references only. Do not copy code or constants.
- Implement and test one numbered step at a time. Search changes interact heavily,
  so every step should be benchmarked and self-play tested before moving on.

Current Rarog baseline:

- Architecture: compact Rust engine with custom bitboards, incremental
  make/unmake, legal move generation, Zobrist hashing, transposition table,
  pawn/eval cache, handcrafted tapered evaluation, UCI front end, Syzygy support,
  and Lazy SMP helpers.
- Search: iterative deepening, PVS, aspiration windows, TT cutoffs, IIR,
  null-move pruning with verification, ProbCut, singular extensions, futility,
  late move pruning, late move reductions, capture-focused qsearch, staged move
  picking, killers, countermove, main/low-ply/pawn/capture/continuation history,
  and correction history.
- Local context: `bench 10` searched about `1.34M` nodes in `642ms`
  (`~2.09M nps`) on the inspected machine. Board benchmark showed generally
  strong primitives, with eval and movegen not looking obviously broken.

## Measurement Discipline

Before changing behavior, make each change measurable.

Implementation:

- Keep using `bench` as a fast fingerprint for node count and nps.
- For search changes, collect at least:
  - `cargo test --release`
  - `cargo bench --bench board --quiet` when board/movegen/SEE/eval changes
  - `bench 10` or `bench 13` for node/nps/fingerprint
  - a small fixed-depth tactical suite if available
  - self-play or gauntlet tests before merging a strength patch
- When testing Elo, compare against the immediate previous commit, not against an
  older moving baseline.
- Expect search-speed changes to trade nodes for stronger selectivity. Do not
  judge by nps alone.

Suggested implementation order follows estimated Elo impact, with speed-only
items placed where they unlock later strength work.

## 1. Add Cached Threat State And Threat-Indexed Histories

Source: strongest idea from Reckless comparison.

Expected impact: very high Elo, moderate nps improvement after expensive
`gives_check` calls are reduced.

Rarog currently computes many attack facts on demand. Add board-level cached
threat data refreshed after every move. This enables better move ordering,
better history indexing, cheaper direct-check tests, and stronger pruning.

Files likely touched:

- `src/board/board.rs`
- `src/board/movegen.rs`
- `src/search.rs`
- `src/move_ordering.rs`
- `tests/board_correctness.rs`
- `tests/search_strength.rs`

Board state to add:

- `piece_threats: [[Bitboard; 6]; 2]` or equivalent.
  - For each attacking color and piece type, squares attacked by that piece type.
- `all_threats: [Bitboard; 2]`.
  - Union of all `piece_threats[color]`.
- `checking_squares: [[Bitboard; 6]; 2]`.
  - For side `color`, squares from which a piece type would give direct check
    against the enemy king.
- `pinned: [Bitboard; 2]`.
  - Pieces pinned to their own king.
- `pinners: [Bitboard; 2]`.
  - Enemy sliding pieces causing pins.
- Optionally `prior_all_threats` can be recovered from unmake history instead of
  stored separately.

Refresh rules:

- Refresh this state in the same places `checkers` is currently recalculated:
  after FEN parsing, `make_move`, `unmake_move`, `make_null_move`, and
  `unmake_null_move`.
- Compute threats from the opponent perspective carefully. The king should be
  excluded from occupancy when calculating enemy attacks used for king move
  legality, so sliders see through the king as appropriate.
- Keep `Board::is_attacked` as a correctness-oriented API, but search should use
  cached `all_threats[them]` where legal.

History changes:

- Replace or augment `main_history[color][from][to]` with:
  `quiet_history[color][from_threatened][to_threatened][from][to]`.
- `from_threatened = board.all_threats(them).contains(from)`.
- `to_threatened = board.all_threats(them).contains(to)`.
- Capture history should include whether the destination is threatened:
  `[attacker][to][captured][to_threatened]`.
- Keep existing pawn history, low-ply history, and continuation histories at
  first. They are already useful and less risky.

Move-ordering additions:

- Add `Board::is_direct_check(mv)` using cached checking squares:
  `checking_squares[us][moving_piece].contains(mv.to_sq())`.
- Replace hot `board.gives_check(mv)` calls in quiet ordering and pruning
  exceptions with `is_direct_check(mv)` where exact discovered-check handling is
  not required.
- Add quiet bonuses:
  - moving a threatened minor/rook/queen to a safe square
  - safe direct checks
  - safe moves that attack a vulnerable enemy piece
- Add quiet penalties:
  - moving to a square attacked by pawns/minors
  - moving king-near pawns in the opening/middlegame if king is castled or still
    on home rank

Testing:

- Add tests proving cached threat state matches direct attack queries across a
  suite of FENs after make/unmake.
- Add tests for direct-check detection on normal checks, discovered checks
  (should document whether direct check intentionally excludes them), promotions,
  and castling.
- Run `cargo test --release`, `cargo bench --bench board --quiet`, and `bench`.

Implementation guardrails:

- First add cached state and tests without changing search behavior.
- Then change history indexing.
- Then add ordering bonuses and replace hot direct-check calls.
- Tune after correctness is stable.

## 2. Add Upcoming Repetition Detection And Rule-50 TT Buckets

Source: Reckless comparison and independent GHI improvement.

Expected impact: high Elo, especially in fortress/repetition/50-move positions;
also improves TT safety.

Rarog detects actual repetitions through history scan. Add detection for
positions where a reversible move can force a repetition soon. This prevents
search from overvaluing lines that are actually draw-bound and reduces graph
history interaction.

Files likely touched:

- `src/board/board.rs`
- `src/board/zobrist.rs`
- `src/tt.rs`
- `src/search.rs`
- `tests/board_correctness.rs`
- `tests/search_strength.rs`

Implementation details:

- Keep the normal board hash unchanged for position identity and repetition.
- Add a `tt_hash()` or `search_hash()` method:
  `board.hash ^ ZOBRIST.rule50_bucket[halfmove_clock_bucket]`.
- Use this hash for TT probe/store and eval cache if the cache is intended to
  represent search value under rule-50 context.
- Use the original hash for legal repetition identity and Syzygy API calls.
- Add 16 rule-50 buckets:
  `bucket = halfmove_clock.saturating_sub(8) / 8`, clamped to `0..15`.
- Add `upcoming_repetition(ply)` using a cuckoo table of reversible piece-move
  hash differences:
  - Build a static table of non-pawn reversible moves.
  - For each earlier same-side state within the reversible halfmove window,
    compute the hash difference.
  - Check whether the difference corresponds to one legal reversible move whose
    between-squares path is empty.
  - Return true only when the repetition is after root, or when previous
    repetition state proves the draw is claimable.
- At non-root nodes, before normal pruning, if `upcoming_repetition(ply)` is
  true, raise `alpha` to draw score. If `alpha >= beta`, return draw score.

TT integration:

- Probe/store TT by `board.tt_hash()`.
- Keep TT mate-score recovery rule-50 aware as it already is.
- If this changes existing tests expecting raw `board.hash`, update the tests to
  distinguish position hash from TT hash.

Correction history:

- Bucket pawn/minor/non-pawn correction history by rule-50 bucket later in
  Step 12. The TT bucket can land first.

Testing:

- Add repetition-cycle FENs where a one-ply-ahead repetition is available.
- Add tests showing positions with same pieces but different halfmove buckets
  have identical `board.hash` and different `board.tt_hash`.
- Add regression tests where TT from a low halfmove clock does not reuse a mate
  or TB-like value past the 50-move horizon.

## 3. Replace Simple LMR With Context-Rich Scaled Reductions

Source: Reckless comparison.

Expected impact: very high Elo if tuned; can lose Elo if implemented too
aggressively.

Rarog currently uses an integer LMR table plus a few modifiers. Replace this
with scaled reductions in units of `1024`, allowing many small terms without
rounding away information.

Files likely touched:

- `src/search.rs`
- `src/move_ordering.rs`
- `tests/search_strength.rs`

Implementation details:

- Add a helper:
  `fn lmr_reduction_scaled(ctx: LmrContext) -> i32`.
- Keep the existing table initially as a base by returning
  `1024 * lmr_reduction(depth, move_index)`.
- Add context terms gradually:
  - increase reduction for cut nodes
  - increase reduction when no TT move exists
  - decrease reduction for `tt_pv`
  - decrease reduction when TT score is valid and above alpha
  - increase reduction when TT depth is shallow
  - decrease reduction for improving positions
  - increase reduction for quiets with bad history
  - decrease reduction for quiets with strong history
  - increase reduction for losing captures
  - decrease reduction when child is in check
  - adjust by previous-ply reduction to avoid repeated over-reduction
  - adjust by cutoff count at child ply
  - reduce pruning aggression when correction-history magnitude is high
- Add tiny deterministic jitter for SMP only:
  `((nodes + thread_id * prime) & 127) - 63`, scaled small. Rarog currently does
  not pass a thread id into search, so this can be deferred until Step 9.
- Reduced search depth:
  `reduced_depth = (new_depth - reduction_scaled / 1024).clamp(1, new_depth + 2)`.
- If reduced search returns above alpha:
  - For non-root, optionally adjust `new_depth += 1` if score is clearly above
    previous best, or `new_depth -= 1` if only barely above.
  - Re-search at full or partially restored depth.
- Preserve full PV re-search behavior.

Guardrails:

- Do not enable all terms at once. Add base scaled LMR first, then groups of
  terms.
- Keep mate/TB scores out of reduction formulas.
- Do not reduce direct checks, promotions, or in-check evasions until tested.

Testing:

- Existing search tests must remain stable.
- Add tactical positions where over-reduction previously risks missing a tactic.
- Compare node count and bench score fingerprint after each term group.

## 4. Make Late Quiet Pruning Skip Remaining Quiets

Source: Reckless comparison; high speed impact.

Expected impact: medium-high Elo and nps by avoiding useless late quiet
iteration.

Rarog already late-prunes individual quiets. Since quiets are ordered by score,
when a quiet fails strong LMP/futility and is not a checking exception, later
quiets are usually worse. Add a picker-level `skip_quiets` behavior.

Files likely touched:

- `src/search.rs`
- `src/move_ordering.rs`
- tests around staged picker ordering

Implementation details:

- Extend `MovePicker::next` to accept `skip_quiets: bool`, or add a method on
  the staged picker to advance from quiet stage to bad-capture stage.
- In the main move loop:
  - For quiet LMP fail: set `skip_quiets = true` and continue.
  - For quiet futility fail: set `skip_quiets = true` and continue.
  - For quiet history-prune fail at shallow depth: continue only the current move
    unless testing proves skipping all quiets is safe.
- Preserve exceptions:
  - in check
  - root node
  - PV/TT-PV nodes
  - direct checking quiets
  - promotions
  - mate-score windows

Testing:

- Add tests that bad captures are still emitted after quiets are skipped.
- Add tests that a direct-check quiet is not skipped by quiet LMP.
- Compare nps on `bench 10` and `bench 13`.

## 5. Refine ProbCut

Source: Reckless comparison.

Expected impact: medium-high Elo; can reduce nodes significantly.

Rarog has ProbCut with fixed `beta + 180`, top captures, qsearch, and
`depth - 4` verification. Make it more conditional and adaptive.

Files likely touched:

- `src/search.rs`

Implementation details:

- Run ProbCut mostly at cut nodes:
  `cut_node && !tt_pv && !in_check && !excluded`.
- Skip if beta is mate/TB-like.
- Require either:
  - TT score is valid and already suggests high score, or
  - static eval is near beta.
- Use:
  `probcut_beta = beta + margin - improving_bonus`.
- Generate captures/noisy moves with a SEE threshold:
  `threshold = probcut_beta - eval_for_pruning`.
- Use staged tactical picker or a ProbCut-specific picker that stops after bad
  captures.
- For each candidate:
  - make move
  - qsearch at `[-probcut_beta, -probcut_beta + 1]`
  - if qsearch passes, verify with reduced depth
  - set verification depth based on overshoot:
    larger overshoot can use shallower depth, smaller overshoot uses deeper
    verification
  - if adjusted beta is used and fails, retry at original beta once
- Store lower TT bound with the verified depth.
- Return a softened non-decisive score near beta instead of the raw overshoot.

Guardrails:

- Keep current ProbCut behind an easy-to-remove local branch or helper so A/B
  testing is simple.
- Never ProbCut singular-extension excluded searches.

Testing:

- Tactical suite before and after.
- Ensure no illegal PV or TT move regressions.

## 6. Improve Singular Extensions

Source: Reckless comparison.

Expected impact: medium-high Elo.

Rarog has basic singular extension. Make it more expressive: multi-cut,
negative extensions, and stronger margins.

Files likely touched:

- `src/search.rs`

Implementation details:

- Eligibility:
  - non-root
  - not excluded
  - TT move legal and present
  - depth at least `5`, or `5 + tt_pv`
  - TT depth at least `depth - 3`
  - TT bound is not upper
  - TT score is valid and not mate/TB-like
- Singular beta:
  - smaller margin for exact TT bounds
  - larger margin for lower bounds
  - increase margin for TT-PV/non-PV mismatch
- Run excluded search at `(depth - 1) / 2`.
- If excluded search fails low:
  - extension starts at `1`
  - extension can become `2` if fail-low margin is large
  - extension can become `3` only for very clear cases after testing
- If excluded search fails high above beta:
  - return a softened multi-cut score unless decisive.
- If excluded search is better than TT score:
  - discard TT move for ordering if the current implementation can do that
    safely.
- If TT score is already above beta or this is a cut node but not singular:
  - apply negative extension, e.g. `-1` first; test `-2` later.

Testing:

- Add tests for excluded-move search not storing TT for the excluded node.
- Add mate/tactic positions where singular extension should find deeper tactics.

## 7. Update Histories On TT Cutoffs And Fail-Low Parent Moves

Source: Reckless comparison.

Expected impact: medium Elo, low nps cost.

Rarog mostly updates histories after searched beta cutoffs. TT cutoffs also carry
useful ordering information.

Files likely touched:

- `src/search.rs`
- `src/move_ordering.rs`

Implementation details:

- In early TT cutoff:
  - If non-PV, no excluded move, TT lower bound cuts off, TT move is legal quiet,
    and previous ply searched only a few moves, update quiet history and
    continuation history for TT move.
  - Use a smaller bonus than searched cutoffs.
- On upper-bound fail-low:
  - If previous move was quiet and current node proves opponent had no good
    continuation, reward the previous move.
  - Scale by depth, previous move count, whether previous move was TT move, and
    how far `best_score` fell below eval.
- On noisy fail-low parent:
  - Optionally reward prior capture history when a capture refuted opponent
    resources.

Guardrails:

- Use small bonuses first. History overtraining can make move ordering brittle.
- Do not update histories from mate/TB scores.

Testing:

- Existing history tests should be extended to cover TT cutoff updates.
- Bench node count should usually drop if ordering improves.

## 8. Improve Time Management With Stability And Effort Signals

Source: both independent and Reckless comparison.

Expected impact: medium Elo at timed controls.

Rarog already uses root effort, score drop, and best-move stability. Add more
signals and make threaded search stop decisions less main-thread-only.

Files likely touched:

- `src/search.rs`
- `src/search_threads.rs`
- `src/time_manager.rs`

Implementation details:

- Track per iteration:
  - PV stability: same best move as previous completed depth
  - eval stability: score close to moving average
  - best-move changes within root search
  - best root move node fraction
  - score trend/drop from previous best score
- Soft time multiplier:
  - reduce when PV and eval are stable
  - increase when best move changes
  - increase when score drops sharply
  - increase when best root move has low effort/confidence
  - reduce when one root move dominates effort and remains best
- In threaded mode:
  - helpers vote for soft stop through shared state
  - stop when a majority or weighted majority agrees
  - hard stop remains absolute

Testing:

- Extend time-manager tests for stable vs unstable root signals by injecting
  synthetic counters if needed.
- Run short movetime UCI process tests to ensure no time forfeits.

## 9. Strengthen Lazy SMP Shared Root Statistics

Source: Reckless comparison, excluding NUMA.

Expected impact: medium Elo at multi-threaded search.

Rarog uses helpers with root offsets and selects among completed results. Improve
shared stats without changing UCI options.

Files likely touched:

- `src/search.rs`
- `src/search_threads.rs`

Implementation details:

- Add cacheline-aligned per-thread node counters to reduce atomic contention.
- Add shared per-root-move stats:
  - best completed depth
  - score
  - nodes
  - lower/upper/exact flag if available
  - vote count or weighted confidence
- Helpers publish root results after each completed depth, not only final result.
- Final selection:
  - prefer legal root move with greatest completed depth
  - break ties by weighted helper agreement
  - then by score adjusted for bound confidence
  - avoid replacing a deeper main result with a shallow helper outlier
- Add deterministic helper diversification:
  - root move offset already exists
  - add small LMR jitter once Step 3 has scaled reductions

Testing:

- Existing threaded determinism tests may need updating if helper diversification
  intentionally changes search. Keep single-thread deterministic.
- Add tests that final bestmove is legal even when helper result is incomplete.

## 10. Replace Hot Exact Check Tests With Cached Direct-Check Tests

Source: Reckless comparison; can be implemented as part of Step 1 or separately.

Expected impact: low-medium Elo, medium nps improvement.

Rarog uses exact `board.gives_check(mv)` in quiet scoring and pruning exception
paths. Exact checks are expensive because they must handle discovered checks and
special cases. Most pruning exceptions only need direct checks.

Implementation details:

- Add:
  `Board::is_direct_check(mv) -> bool`.
- It should return true when the moved piece attacks the enemy king from the
  destination using cached `checking_squares`.
- It may intentionally return false for discovered checks, en-passant discovered
  checks, and castling checks. Document this.
- Use direct check in:
  - quiet move ordering bonus
  - LMP/futility exceptions
  - qsearch late tactical pruning exceptions if safe
- Keep exact `gives_check` for:
  - legal validation
  - places where discovered checks must be recognized
  - tests that require exact behavior

Testing:

- Unit tests distinguishing direct checks from discovered checks.
- Bench before/after to ensure nps improves.

## 11. Improve TT Layout, Indexing, And Eval-Only Entries

Source: both independent and Reckless comparison.

Expected impact: medium nps, low-medium Elo.

Rarog TT is already robust. Improvements should preserve safety.

Files likely touched:

- `src/tt.rs`
- `src/search.rs`

Implementation details:

- Consider multiplicative indexing:
  `index = ((hash as u128 * cluster_count as u128) >> 64) as usize`.
  This can reduce low-bit dependency compared with masking.
- Add eval-only TT entries:
  - Represent a valid raw static eval without a search bound.
  - Probe can return static eval even when no bound cutoff is possible.
  - Search stores eval-only entry after evaluating a node without an existing TT
    entry.
- Keep full-key validation for shared TT.
- Preserve current mate and rule-50 score recovery.
- Replacement:
  - Prefer replacing empty, old, shallow entries.
  - Do not overwrite a much deeper current-age exact entry with shallow bound.

Testing:

- TT probe/store tests for eval-only entries.
- Hashfull tests still pass.
- Search PV legality tests with TT pollution positions still pass.

## 12. Add Rule-50 Buckets And Deeper Continuation Correction History

Source: Reckless comparison.

Expected impact: medium Elo.

Rarog correction history is already useful, but can be more context aware.

Files likely touched:

- `src/search.rs`
- `src/move_ordering.rs`

Implementation details:

- Add bucket dimension to correction histories:
  `[bucket][side][key]`.
- Bucket is the same rule-50 bucket from Step 2.
- Add continuation correction at offsets 2 and 4, not just the immediately
  previous move.
- Instead of indexing continuation correction by only previous piece/to, store a
  continuation-correction subtable pointer or index keyed by:
  - previous in-check flag
  - previous move noisy/quiet
  - previous moved piece
  - previous destination
  - current previous piece/to
- Correction value:
  - sum pawn correction
  - own and enemy non-pawn correction
  - minor correction if kept
  - continuation correction offsets 2 and 4
  - divide by tuned scale
- Use correction magnitude in pruning:
  - high absolute correction means static eval is less reliable
  - reduce RFP/futility/null/LMR aggression when correction magnitude is high

Testing:

- Correction-history tests for bucket isolation.
- Ensure history aging/clear remains fast enough.

## 13. Systematically Tune Search And Eval Constants

Source: independent.

Expected impact: very high over time.

Rarog has many handcrafted constants. The engine needs a disciplined tuning path
that does not expose new UCI options.

Implementation details:

- Create an internal `params` module with const getter functions.
- Under a non-release feature such as `spsa` or `tuning`, allow setters or
  generated values.
- Do not print these as UCI options in normal builds.
- Tune in groups:
  1. move ordering weights
  2. history bonuses/maluses
  3. LMP/futility/SEE pruning thresholds
  4. LMR terms
  5. null move and ProbCut margins
  6. correction history scaling
  7. eval terms
- Use self-play SPRT or another consistent statistical method.

Guardrails:

- Avoid tuning too many dependent parameters at once.
- Retest at both fast and longer controls.
- Keep a tactical regression suite to catch selectivity losses.

## 14. Handcrafted Evaluation Upgrades

Source: independent.

Expected impact: medium-high Elo if tuned; can lose Elo if untuned.

Current eval covers material/PST, pawns, passed pawns, mobility, rook activity,
outposts, threats, king safety, draw scaling, and winning king push. Add only
features with clear signal and low nps cost.

Candidate features:

- Better passed pawn evaluation:
  - connected passers
  - candidate passers
  - protected passers
  - path clearance and blockader quality
  - king race distance in pawn endgames
- King safety:
  - file-specific shelter
  - storm by enemy pawn distance
  - open/semi-open files near king
  - safe checks available to enemy pieces
- Threats:
  - minor attacks on queen/rook
  - rook attacks on queen/king-file pressure
  - hanging pieces with SEE-like cheap guard
- Draw scaling:
  - opposite-colored bishops
  - rook pawn with wrong bishop
  - low material with no pawns
  - fortress-like no-breakthrough pawn structures where detectable cheaply
- Material imbalance:
  - bishop pair already exists; add rook pair penalties, queen trade
    preferences, minor-vs-rook context if tuneable.

Implementation guardrails:

- Add feature groups one at a time.
- Avoid expensive full attack maps inside eval unless Step 1 cached threats make
  them cheap.
- Tune each group.

Testing:

- Add targeted eval tests only for clear invariants, not exact tuned values.
- Use self-play for actual Elo.

## 15. Build-Time Tables And Setwise Attack Helpers

Source: Reckless comparison.

Expected impact: medium nps; enables cheaper threat cache.

Rarog initializes attack tables at startup. It can also add setwise attack
helpers to compute attack unions faster.

Files likely touched:

- `build.rs`
- `src/board/attacks.rs`
- `src/board/bitboard.rs`
- `src/eval.rs`

Implementation details:

- Generate magic/PEXT lookup data at build time instead of searching magics at
  runtime.
- Add setwise helpers:
  - `pawn_attacks_setwise(pawns, color)`
  - `knight_attacks_setwise(knights)`
  - `bishop_attacks_setwise(bishops, occ)`
  - `rook_attacks_setwise(rooks, occ)`
  - queen as bishop union rook
- For scalar fallback, loop over set bits.
- For x86 AVX2/AVX512 specialized builds, optional vectorized setwise sliders can
  be added later, but a scalar helper is enough to simplify Step 1.
- Use these helpers in threat cache and eval mobility where they reduce repeated
  calls.

Testing:

- Setwise helper output must equal OR of per-piece attacks across randomized
  boards.
- Board benchmark should show no regression.

## Suggested Milestones

Milestone A: Safety and instrumentation

- Add measurement scripts or documented commands.
- Add direct-check helper tests if easy.
- No behavior-risky pruning changes yet.

Milestone B: Threat-aware ordering

- Implement cached threat state.
- Add threat-indexed quiet/capture histories.
- Replace hot exact check tests with direct-check tests.
- Benchmark and self-play.

Milestone C: GHI and repetition safety

- Add rule-50 TT hash buckets.
- Add upcoming repetition detection.
- Add correction-history buckets.
- Benchmark and self-play.

Milestone D: Search selectivity

- Scaled LMR.
- Skip-rest quiet pruning.
- Refined ProbCut.
- Improved singular extensions.
- History updates on TT cutoffs/fail-lows.
- Each substep needs independent testing.

Milestone E: Parallel and time behavior

- Better soft-time multiplier.
- Soft-stop voting.
- Shared root statistics.
- Threaded self-play and node-limit tests.

Milestone F: Tuning and eval

- Internal params module.
- Tune search constants.
- Add and tune low-cost eval upgrades.
- Consider build-time tables/setwise speedups if threat cache cost is visible.

## Implementation Notes For Future Agents

- Start every step by reading the current version of the touched files. Search
  code changes often land nearby, and stale assumptions are dangerous.
- Prefer small PR-sized patches. If a patch changes both search selectivity and
  evaluation, split it.
- Preserve single-thread determinism unless the specific step says otherwise.
- Treat threaded behavior separately from single-thread behavior.
- After any TT/hash/repetition change, run PV legality and draw tests.
- After any movegen/threat-cache change, run perft and board correctness tests.
- After any pruning/LMR change, run tactical positions and self-play.
- If a change improves nps but loses Elo, keep the branch for later tuning but
  do not merge it as a strength improvement.
