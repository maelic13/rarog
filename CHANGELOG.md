# Changelog

All notable changes to Rarog are documented in this file.

Rarog was released as Lynx through version `1.4.3`. The project was renamed
starting with version `2.0.0` to avoid confusion with an existing chess engine.

## [2.4.0] - 2026-09-11

A consolidation release. Its gate, RAR-E16, measured this head against a 2.3.2
binary rebuilt from the release recipe at `3+0.03`, 1T, accepting H1 after
**742 games at +54.77 ± 17.04 Elo**; a fixed-size 400-game `3+0.03` 4T
direction check read +79.53 ± 21.21 with zero forfeits, zero crashes and zero
protocol warnings. An SPRT decides rather than estimates and stops biased
upward, so the honest reading is "clearly and substantially positive, magnitude
not settled" — do not quote +54.77 as the release's Elo over 2.3.2. The
individual results below are sequential gates under different baselines and
estimators; they are not additive and must not be summed.

### Added

- `help` prints what the engine is, how to drive it and where the source lives.
  It had been advertised by the unknown-command message without ever existing.
- Command-line arguments are now executed as engine commands — `rarog bench 13`
  benches — and an unknown argument exits 2. Previously every argument was
  discarded silently with a success code.
- A startup advisory names the CPU tier mismatch it can detect: a capable CPU
  running the portable `base` asset, and a slow-PEXT part (Excavator, Zen,
  Zen+, Zen 2) running the `pext` asset, which is the slowest build for it.

### Changed

- **ProbCut move filter.** Capture eligibility is tied to the gap the capture
  must bridge and the cap applies to moves searched rather than examined,
  scaled by `cut_node`. Gated as a bundle at **+15.44 ± 8.06 Elo
  (+24.50 ± 12.78 nElo)** in 2,838 games; the RAR-S58 ablation credits the
  filter alone at +24.90 ± 16.01 nElo and its null-move partner at zero, so
  that partner was reverted and is not part of this release.
- **Root-only LMR relief** of 1.5 ply at ply 0, where the reduction formula
  previously could not see the root at all. **+2.33 ± 1.85 Elo
  (+3.58 ± 2.85 nElo)** over 56,928 games, for 6.6% fewer nodes.
- **Two complete evaluation refits.** The first whole-surface WDL refit of the
  1,218-slot HCE measured **+22.04 ± 7.51 Elo**; the later `hce-v3` refit on a
  602,619-game non-adjudicated corpus measured a further **+11.81 ± 5.33 Elo**.
- **Tablebase-corrected training labels.** Every training position with six men
  or fewer is relabelled to its Syzygy value with the fifty-move rule kept, so
  a cursed win is labelled the draw it is. **+6.73 ± 3.82 Elo** over the
  identical position set with self-play labels.
- **Board and move-generation cluster**, gated as one dependency-complete
  change at **+12.12 ± 10.17 Elo**: the SEE and UCI repairs listed below plus
  caller-owned move-list delivery and related throughput work, together worth
  +1.421% whole-search NPS.
- The search clock now starts when `go` is parsed, as both reference engines
  do, rather than after move setup. Re-measured at **zero time forfeits in
  10,000 games** and −0.69 ± 3.62 Elo, i.e. free.
- The build is pinned to `rustc 1.98.1` and `cc` 1.4.5, with `rust-version`
  held in lockstep with `rust-toolchain.toml` so a stale toolchain fails with
  a clear cargo message.

### Fixed

- **Static exchange evaluation** no longer treats king recaptures into
  attacked squares as legal, ignores pins it creates, or mis-values recapture
  promotions. This makes the search prune less — the fixed-depth `bench 13`
  node count rises 6,901,489 → 7,601,220 — and was gated as part of the board
  cluster above rather than assumed harmless.
- Two boundary defects on the board and protocol edge: the FEN fullmove counter
  saturates instead of overflowing at `u16::MAX`, so it is defined identically
  in debug and release, and `Move::from_uci` rejects non-ASCII input rather
  than indexing into the middle of a multi-byte character.
- **KBN-K conversion.** The mate drive had been passing its anchor test under a
  broken drive; repaired, conversion of the bishop-and-knight mate moves from
  **19.4% to 96.9%**, with no change to the search fingerprint — fingerprint
  equality alone never proved narrow-feature neutrality.
- Changing the `LazyMargin` UCI option now clears the evaluation cache. The
  margin decides which expensive evaluation terms run, so scores cached under
  the previous margin stayed readable and wrong.
- A panic is now reported on stdout, where a tournament harness records it,
  instead of on stderr where 2.3.2's single crash in ~5,200 games left nothing
  the operator could act on.

### Removed

- `lmr_prior_reduction_adj`, together with its consumer, its call sites and the
  now-unused reduction field it read. With the stale-read defect it depended on
  repaired, RAR-S64 measured the whole cluster at +0.39 ± 4.89 Elo against the
  head: the gain had been the bug. A knob parked at its no-op value is still a
  branch in the hot path and a coordinate in the tuning surface.
- The null-move entry half of the 4.7 selectivity bundle, reverted after its
  ablation measured its contribution in company at zero.

## [2.3.2] - 2026-08-11

A consolidation release: accepted search and evaluation gains since 2.3.1,
one search-correctness repair, and stronger cross-platform release guarantees.

### Changed

- Retained the accepted broad selectivity fit, zero-reduction LMR floor and
  anchored evaluation refresh. Their individual confirmation results were
  **+15.33 ± 7.34 nElo**, **+9.13 ± 5.45 nElo** and **+11.56 ± 5.19 Elo**,
  respectively. These are sequential experiment results under different
  estimators and must not be added into a single release-Elo claim.
- ARM64 transposition-table prefetch now emits `prfm pldl1keep`. On an Apple M4
  it improved median benchmark speed by **1.42%**, winning all 12 paired rounds,
  without changing the search fingerprint.
- Internal TT evidence and root-search fallback handling are more explicit and
  better covered by regression tests.

### Fixed

- Null-move pruning no longer promotes an unproven mate-range score into an
  authoritative cutoff. This correctness repair is retained by policy; its
  interrupted strength run showed no early regression signal but was not
  complete enough to support an Elo claim.
- Published CPU tiers now have an executable ISA contract. The portable x86-64
  build no longer inherits POPCNT from vendored tablebase code and requires only
  its documented SSE3 baseline; ARM64 release assets are checked for the
  required prefetch instruction.
- Release CI now verifies each finished asset's UCI handshake, deterministic
  benchmark and instruction-set contract before upload.

### Removed

- Ten abandoned experimental tuning switches/coordinates whose accepted
  defaults are now hardwired. This removes no accepted behavior and does not
  change the `bench 13` search fingerprint.
- The canceled pre-NNUE Phase-4 SPSA launch configuration. No tuning games had
  been played, and the estimated 320,000-game run lacked a credible material
  strength prior.

## [2.3.1] - 2026-07-29

Patch release restoring full release-build optimisation on Windows ARM64.

### Fixed

- **Windows ARM64 binaries are now profile-guided-optimized.** PGO was
  previously impossible on `aarch64-pc-windows-msvc`: linked with MSVC's
  `link.exe`, the instrumented binary emitted `.profraw` files with an empty
  symbol-name table and `llvm-profdata merge` rejected all of them. Linking
  with LLD instead resolves it, so `xtask` now forces `rust-lld` for that
  target and the release workflow builds the arm64 Windows asset with `--pgo`
  like every other cell. Measured locally at **+8%** search speed (3.11 M ->
  3.37 M nps on `bench 13`). The workaround is marked for removal once
  rust-lang/rust#156675 is fixed upstream.

  Note: the 2.3.0 notes below claim every published binary was
  profile-guided-optimized. That was true of the other eight assets but not of
  the arm64 Windows one, which shipped without PGO. It is accurate from this
  release onward.

## [2.3.0] - 2026-07-28

The correctness-and-search release: long-standing evaluation and search bugs
fixed, move ordering and in-check selectivity reworked, the parallel search
rebuilt, and every published binary now profile-guided-optimized.

Measured against `2.2.0` over roughly 19,000 games against a pool of established
engines:

| Condition | Gain over 2.2.0 |
|---|--:|
| 1 thread, 3s+30ms | **+76 Elo** |
| 1 thread, 10s+100ms | **+78 Elo** |
| 4 threads, 10s+100ms | **+194 Elo** |

Search speed rose about **11%**. The four-thread figure is larger because the
parallel-search rework only applies there. No losses on time occurred in any
condition tested.

The `bench 13` search fingerprint is `5,173,540` nodes.

### Added

- Parallel-search rework: threads publish per-root-move `(depth, score, bound)`
  to a shared pool and order their root lists from the pool's view; the soft
  stop is a symmetric majority vote across threads rather than the main
  thread's single estimate; per-thread reduction jitter decorrelates the
  search trees.
- Shared transposition table repacked to 10-byte slots with 16-bit key
  verification, restoring full entry density at `Threads > 1`.
- History bonus and malus are separately parameterised, and history coverage
  was extended across the capture, continuation, pawn and correction tables.
- Profile-guided optimisation for every published release binary, and a pinned
  Rust toolchain so a given release can be rebuilt exactly.
- Additional internal consistency checking in debug builds, and expanded
  regression test coverage.

### Changed

- Checking moves are always extended, replacing the previous conditional rule.
- Static-exchange evaluation is pin-aware; its consumers were re-tuned to match
  the more accurate result.
- Time management compares the current score against the average of prior
  iterations when deciding to extend on a falling evaluation.
- Search parameters now have a single source of truth, removing a class of
  configuration drift.
- Evaluation and move-ordering hot paths hoist node-invariant work out of
  per-move and per-piece loops.
- Build tooling: CPU tuning is now independent of the instruction-set choice.
  `cargo xtask build --arch {base|avx2|pext|arm64}` selects the instruction-set
  contract and the new `--native` flag additionally tunes for the building
  machine, so the two combine freely. Previously `native` was itself an `--arch`
  value that forced the BMI2/PEXT code path, which meant a CPU without BMI2
  could not produce a native build at all.
- A plain `cargo build --release` no longer applies `target-cpu=native`. Native
  tuning is opt-in via `cargo xtask build --arch <arch> --native`, and such
  binaries are marked `-native` and are not portable.

### Fixed

- Aspiration-window re-search could fail to terminate on sudden mate scores.
- A capture beta cutoff could train correction history on tactical results.
- Several evaluation terms had incorrect side-to-move or scaling semantics.
- Multi-threaded diagnostic counters were reset and dumped by every thread,
  making all multi-thread diagnostic output unreliable.
- Asking for a native build on a CPU without BMI2/PEXT produced a binary that
  executed an unsupported instruction. Requesting `--arch pext --native` now
  verifies the CPU reports BMI2 and refuses up front.

### Removed

- The 32-bit x86 build path; the engine is 64-bit only.

## [2.2.0] - 2026-06-24

The eval release. Phase 3 rebuilt the evaluation as a fully parameterised,
trace-instrumented function (every term seeded inert so the rewrite was
bench-fingerprint-identical), and Phase 4 then **fit that enlarged eval to data**
— a staged offline Texel tune over a 2.19M-position self-play set, with each
stage SPRT-gated at `tc=3+0.03`. This is the single largest strength jump in the
project so far: seven accepted stages (king safety, threats, mobility, pawn
structure / passers / minor & rook terms, material imbalance, material + PSTs,
and a final global joint polish), each confirmed against the previous head.

### Added

- King-danger evaluation: a single danger accumulator with attacker-unit,
  safe-check, weak-king-ring, king-flank, pawnless-flank and queen-relief inputs
  feeding a lengthened non-linear danger→score table (the classic danger² curve).
- Threat evaluation v2: per-victim threat-by-minor/rook tables, a refined
  weakly-defended-piece term (which subsumed and retired the old flat hanging
  penalty), safe-pawn-push threats, weak-piece and restricted-square terms.
- Per-count mobility tables for knight/bishop/rook/queen (replacing the flat
  per-move bonus), monotonic in the safe-mobility count.
- Expanded pawn-structure and passed-pawn evaluation (rank-scaled connected
  pawns, supported / free-stop / safe-stop / candidate passers, blocked-passer
  and ideal-blockader terms) and assorted positional terms (outposts, rook
  files/7th, bishop long-diagonal, space, king-centrality danger, and more).
- SF-style material-imbalance evaluation (a symmetric per-piece-pair quadratic).
- Exact endgame knowledge: a KPK bitbase plus scale-factor handling for known
  drawish and won material configurations (KBNK mating drive, wrong-corner
  bishop, opposite-coloured bishops, bare-rook and insufficient-material draws).
- A whole-position evaluation cache and a lazy-evaluation fast path that skips
  the expensive positional terms when the material+PST margin is already
  decisive (the mate-drive mop-up runs on both paths).
- Offline Texel-tuning tooling under `tools/texel-tuner/` and `tools/texel/`:
  a linear-trace gradient fitter with Adam, feature-support diagnostics, a
  bucketed-holdout loss reporter, a dedicated re-evaluation fit for the
  non-linear king-safety inputs, optional L2-to-prior, phase-balanced sampling,
  and a verifiable parameter-baking script.

### Changed

- Material values and piece-square tables were refit against the now-complete
  positional term set; the resulting values keep classical piece ratios.
- The `bench` fingerprint changed as eval behaviour changed (the Phase-3
  build-out alone stayed fingerprint-identical; Phase 4 activated the terms).

### Verified

- Seven Phase-4 stages each passed self-play SPRT at `tc=3+0.03` against the
  previous head (king safety +42, threats +45, mobility +24, scalars +85,
  imbalance +27, material+PST +28, global polish +65 Elo).
- A 2700-game external gauntlet at `tc=10+0.1` against a mixed field
  (Stockfish capped, both Basilisk siblings, Critter 1.6a, Fruit 2.1, and
  prior Rarog releases) confirmed the gain transfers: **+240 Elo head-to-head
  over Rarog 2.1.0**, beating both siblings and all prior releases. Anchored to
  published CCRL ratings, this places Rarog 2.2.0 at roughly **3000 CCRL**.

## [2.1.0] - 2026-06-19

Release focused on a repo-contained SPRT/SPSA testing harness and a long
sequence of individually SPRT-gated search-tuning and robustness fixes. The
single largest change is a Stockfish-style time-management rewrite that fixed
a movetime budget bug — by far the dominant contributor to the cumulative
strength gain. Several speculative search-feature ports were tried and
rejected by the harness; they are listed below for transparency, since they
cost real development time even though they did not ship.

### Added

- Added `SearchParams` for search constants and tune-mode UCI spin options
  behind `--features tune`, allowing weather-factory SPSA to perturb search
  parameters without exposing development options in production builds.
- Added default-equivalent 1024ths-of-a-ply LMR adjustment parameters
  (`LmrTtPvAdj`, `LmrExactBound`, `LmrShallowTt`, `LmrCutNode`, plus the LMR
  table formula coefficients) so LMR tuning can be done through UCI options
  without changing baseline behavior.
- Added per-move quiet futility pruning (separate from the existing static
  futility margin), SPSA-tuned and SPRT-confirmed.
- Added a clock-mode time-safety valve: an absolute `2*MoveOverhead` reserve
  that binds only in genuine time scrambles, leaving normal time allocation
  untouched.
- Added repo-local testing/tuning helpers under `tools/`, including fastchess
  SPRT, pext-PGO test builds, SPSA setup (weather-factory), local tool setup,
  opening book storage, test-engine storage, and weather-factory configs.
- Added an optional `target-cpu=native` build path (`cargo xtask build --arch native`)
  for local/own-match binaries that can use CPU-specific instructions beyond
  the portable `x86-64-v3`/`avx2`/`pext` release assets.

### Changed

- Rewrote time management in the style of Stockfish: clock-mode budgeting now
  ports SF's `timeman.cpp` structure (optimum/maximum from elapsed time,
  game-ply-aware scaling) and between-iteration stopping uses SF's
  falling-eval / best-move-instability / effort-factor heuristics. Fixed a
  movetime bug where a small budget (e.g. 100 ms/move) collapsed to depth 1
  because the overhead subtraction could reduce the budget to ~1 ms.
- Reworked quiescence search's TT-bound stand-pat handling.
- Baked in the accepted Phase 1 pruning/margin SPSA tune:
  `AspirationDelta=31`, `FutilityBase=86`, `FutilityImproving=49`,
  `RazoringCoeff=191`, `NullMoveDepthCoeff=15`,
  `NullMoveImprovingBonus=25`, `LmpBase=115`, `LmpImproving=57`,
  `QuietHistPruneCoeff=4419`, `SeePruningCoeff=81`, `SeePruningMax=811`,
  `SingularBetaMult=4`, and `LmpCountBase=2`.
- Baked in SPSA-tuned quiet futility margins (`FpBase=184`, `FpCoeff=117`)
  and a re-tuned LMR table after harness correction.
- Shrunk the `BadCapture` struct (16 → 3 bytes) and removed a per-call board
  clone from `gives_check`'s castling path (replaced with a direct rook-attack
  test), eliminating the one allocator touch on a search-reachable path.
- Removed the dead `do_shallower` LMR re-search arm: the move loop's
  `alpha >= best_score` invariant made its trigger condition provably
  unreachable, so it was a no-op that only cost a redundant guard check.
- Reduced repeated move-flag decoding in quiet/capture classification helpers.
- Reworked cached checker calculation to compute only opponent checking pieces
  instead of building a generic all-attacker set and masking it afterward.
- Split insufficient-material detection into faster early exits for positions
  with pawns or major pieces.
- Folded evaluation phase accumulation into the existing piece iteration,
  avoiding a redundant popcount pass over every piece bitboard.
- Deferred SEE classification for TT captures until it is actually needed for
  post-search capture-history bookkeeping.
- Made development/test helper paths repo-local (`tools/bin`, `tools/books`,
  `tools/test_engines`, `tools/weather-factory`) instead of relying on
  machine-global `D:\chess` helper paths.

### Fixed

- Fixed clock-mode time forfeits at fast time controls (28 forfeits in a
  ~2,200-game gauntlet, down to 0) via the new time-safety valve, without
  regressing normal-time allocation.

### Evaluated and rejected (for transparency, not shipped)

- ProbCut port from a search-efficiency rewrite branch: **−24.5 ± 8.5 Elo**,
  reverted to the original implementation.
- Persisting history tables across searches (dropping the per-search aging):
  **−12.4 ± 6.2 Elo**, reverted.
- A singular-extension double-extension budget cap: inconclusive (H0),
  reverted.
- An LMR "do-deeper" re-search margin (do-shallower proved structurally dead;
  do-deeper alone): **−1.4 ± 2.7 Elo**, reverted to the futility baseline.

### Internal (no behavior change)

- Began an eval-rewrite groundwork program (attack-map substrate, all tunable
  eval weights hoisted into a parameter struct) to prepare for a future
  data-fit tuning campaign. Bench-fingerprint-identical throughout; no effect
  on this release's playing strength.

### Verified

- Phase 1 Group B pruning/margin tune: SPSA ran 2271 iterations / 72672 games;
  SPRT accepted H1 after 19458 games with `nElo +6.17 ± 4.88`, LOS `99.34%`.
- Time-management rewrite: SPRT accepted H1 at `+81 Elo` (762 games,
  `st=0.1`); a separate non-regression SPRT confirmed no clock-mode
  regression.
- Quiescence TT-bound stand-pat refinement: accepted, `+6.5 Elo` (`st=0.1`).
- Per-move quiet futility pruning: SPRT accepted H1, `+7.98 ± 4.42 Elo`
  (nElo `+10.97`).
- Harness-corrected LMR retune (after unifying SPSA/SPRT at `tc=3+0.03`):
  SPRT accepted H1, `+4 Elo`.
- Phase 2.9-close batch non-regression SPRT (time-safety valve + robustness +
  micro-optimizations, cumulative): `15,976` games, `Elo +2.02 ± 3.62`,
  `nElo +3.01 ± 5.39`, LOS `86.3%`, LLR `2.39` (`81.2%` of the way to the H1
  bound and still trending up when accepted). Cross-harness time-loss check
  (fastchess PGN `[Termination "..."]` tags): zero time-forfeit terminations
  across all games.
- Verified `cargo fmt --check`, `cargo test --lib`,
  `cargo test --test engine_coverage --test search_strength`,
  `cargo build --release --features tune`,
  `cargo test --release --test uci_process -- --test-threads=1`, and the full
  `cargo test --release` suite.
- Rebuilt Windows `pext` and `avx2` PGO release assets. Both produced the
  current `bench 13` fingerprint of `5,446,782` searched nodes and responded
  correctly to `uci`; production builds expose no tune-only options.

## [2.0.2] - 2026-06-01

Patch release focused on tournament stability after two Little Blitzer
illegal-move artifacts revealed a search panic instead of an actually illegal
reported move.

### Fixed

- Fixed a rare quiescence-search panic when a deep tactical/check sequence
  reached the final fixed search-stack slot. Quiescence now stops before
  touching out-of-range PV/history stacks and returns a static corrected
  evaluation at the maximum ply, matching the main search's maximum-ply
  behavior.

### Added

- Added a direct regression test for the quiescence maximum-ply guard.
- Added regression coverage for the two Little Blitzer artifact positions that
  verifies search returns legal root moves.

## [2.0.1] - 2026-06-01

Patch release focused on search improvements for higher playing strength.

### Added

- Added Internal Iterative Reduction (IIR) for PV nodes when the TT move is
  completely absent (previously IIR was restricted to non-PV nodes).
- Added negative history updates for good captures (SEE ≥ 0) that were searched
  before a beta cutoff, consistent with existing treatment of bad captures and
  quiet moves.
- Added correction history updates for beta-cutoff nodes (Lower bound) when the
  search score exceeds the static evaluation, and for fail-low nodes (Upper bound)
  when the search score falls below the static evaluation. Previously correction
  history was only updated on PV/Exact nodes.

### Changed

- Unified capture history bonus at beta cutoff to use `history_bonus(depth)`
  (same formula as quiet history) instead of the flat `depth * depth` previously
  used for captures.

## [2.0.0] - 2026-05-29

### Changed

- Renamed the engine from Lynx to Rarog.
- Updated the UCI engine identity, Cargo package name, executable name, release
  asset names, repository metadata, documentation, tests, and build helpers for
  the Rarog name.
- Renamed the internal PEXT compile-time cfg from `lynx_pext` to
  `rarog_pext`.

## [1.4.3] - 2026-05-29

Patch release focused on a small retained search-strength update and release
build polish after the 1.4.2 asset changes.

### Changed

- Raised the ProbCut margin from `beta + 160` to `beta + 180`.
- Relaxed late-move pruning thresholds so deeper late quiets are pruned less
  aggressively.
- Changed capture move ordering to use full SEE values for profitable captures
  instead of only threshold SEE.
- Renamed PGO release assets to include a `-pgo` suffix before the executable
  extension so PGO and non-PGO builds can coexist in `target/dist`.
- Kept PGO training on the built-in `bench` workload after a Lynx-specific EPD
  training set tested slower than bench-only PGO on the final 1.4.3 code.

## [1.4.2] - 2026-05-29

Release focused on CPU-specific build assets, PGO release builds, and removing
the incomplete MultiPV analysis path while keeping the 1.4.1 single-PV
search/eval behavior intact.

### Changed

- Replaced GitHub AVX-512 release assets with BMI2/PEXT x86-64 assets.
- Switched release-asset construction to the cross-platform `cargo xtask build`
  helper for base x86-64, AVX2, PEXT, and ARM64 builds.
- Compacted PEXT slider metadata so PEXT builds no longer carry unused magic
  multiplier fields.

### Removed

- Removed UCI `MultiPV` support and the sequential MultiPV root-search path.

### Added

- Added a compile-time PEXT sliding-attack table path enabled with
  `--cfg lynx_pext` and BMI2 code generation.
- Added a runtime BMI2 check for PEXT builds so users get a clear error instead
  of an illegal-instruction crash on unsupported CPUs.
- Added optional PGO release builds through `cargo xtask build --arch <asset>
  --pgo`.

## [1.4.1] - 2026-05-28

Patch release combining the unreleased 1.4.0 TT-move safety work with search
hotpath cleanup and a small move-ordering update for checking quiet moves.

### Fixed

- Fixed fast legal validation for recaptures that remove a checking piece, such
  as `...gxf6` after `Nxf6+`, by removing the captured attacker from virtual
  attack bitboards as well as from occupancy.

### Changed

- Replaced clone-and-make based legal validation for TT-shaped and raw UCI
  moves with direct occupancy-based king-safety validation. This preserves the
  strict pseudo-legal checks from 1.4.0 while avoiding a full board clone on TT
  hits.
- Batched Lazy SMP shared node accounting so threaded search no longer performs
  an aggregate atomic node increment on every searched node. Final threaded
  search results still report exact total nodes by summing each thread result.
- Skipped duplicate TT capture scoring in the staged move picker after the TT
  move has already been emitted first.
- Added a direct-check ordering bonus for quiet moves so checking quiets are
  considered earlier by the move picker without changing legality filtering.
- Expanded low-ply quiet history from four plies to eight plies for stronger
  early-root move-ordering feedback.

### Added

- Added a board benchmark workload for direct legal move validation so TT/UCI
  move-validation cost is visible in local profiling.
- Added regression coverage for pseudo-legal pinned moves that must still be
  rejected by the final legal validator.
- Added regression coverage verifying that quiet direct-check moves receive the
  intended move-ordering bonus.
- Added regression coverage for the expanded eight-ply low-ply history window,
  including the boundary where ply eight is no longer recorded.

## [1.4.0] - 2026-05-28 (not released separately; included in 1.4.1)

Unreleased minor-version work focused on Stockfish-style transposition-table
move safety and search move ordering. These changes are included in the 1.4.1
release.

### Fixed

- Hardened TT/hash move use so stored moves are validated and canonicalized for
  the current board before search, quiescence, PV construction, or ponder
  fallback can use them. This prevents stale, aliased, or concurrently observed
  TT moves from entering searched PVs.
- Fixed UCI PV safety around malformed TT moves by rejecting impossible piece
  movement, moves from the wrong side, friendly-occupied destinations, illegal
  king movement, and malformed special-move encodings before `make_move`.

### Changed

- Reworked the move picker to emit a validated TT move first, then skip the
  duplicate when generated legal captures, quiets, or bad captures are visited.
- Changed UCI move parsing to use the same direct legal-move validator as TT
  move validation, while preserving canonical internal flags for captures,
  castling, en passant, and promotions.
- Added TT-derived ponder fallback from the child position when the searched PV
  does not already contain a ponder move.

### Added

- Added board-level pseudo-legal and legal move validation APIs for raw UCI or
  TT-shaped moves.
- Added regression coverage for friendly-occupied king moves, impossible king
  movement, malformed tournament-derived TT moves, TT-first move picking,
  TT-derived ponder fallback, and UCI PV replay at `Threads=1` and `Threads=8`.

## [1.3.4] - 2026-05-28

Patch release focused on completing Stockfish-compatible UCI ponder behavior
and Lazy SMP threaded search semantics.

### Fixed

- Fixed `ponderhit` handling so converting a ponder search to a normal search
  preserves elapsed thinking time instead of restarting the search clock. This
  matches Stockfish-style behavior and prevents tournament GUIs from allowing a
  ponder hit to consume an extra full move budget.
- Fixed threaded search accounting so UCI `nodes`, `tbhits`, stop, quit, and
  ponderhit state are shared across the main and helper threads rather than
  being reported or limited from the main thread alone.

### Changed

- Enabled threaded search for node-limited and MultiPV searches, with helpers
  using the same root move set and aggregate stop condition as the main thread.
- Reworked parallel best-result selection to use weighted votes from helper
  searches, with decisive scores, PV availability, depth, score, and the main
  thread used as tie-breakers.

### Added

- Added interactive UCI process regression tests for completed `go ponder`
  searches waiting for `ponderhit` or `stop` before emitting `bestmove`.
- Added regression coverage verifying that `ponderhit` after an already-spent
  `movetime` returns `bestmove` promptly rather than restarting the timer.
- Added UCI process regression tests for threaded `go nodes`, threaded
  `go infinite` plus `stop`, and threaded `go ponder` plus `ponderhit`.
- Added engine-level coverage for threaded aggregate node limits and threaded
  MultiPV legal root results.
- Documented release-preparation checks for `1.3.4`, including the release
  test suite and a short cutechess regression against `v1.3.3` at one and
  eight search threads.

## [1.3.3] - 2026-05-28

Patch release focused on UCI tournament compatibility and release-process
coverage.

### Added

- Added UCI `go perft N` support so GUI and tournament tools can ask the engine
  to count legal leaf nodes from the current position without starting a normal
  search.
- Added the advertised UCI `Ponder` option. This lets frontends discover that
  Lynx supports ponder-mode command flow even though the option itself is only a
  protocol setting.
- Added process-level UCI regression tests that launch the built engine binary
  and verify option advertisement, diagnostics, `go perft`, uppercase move
  input, and critical invalid-position handling.

### Changed

- Changed invalid `position` failures to report a clear critical UCI error and
  exit instead of continuing from the previous board state.
- Changed unknown UCI commands and unknown options to print explicit diagnostic
  messages.
- Changed empty `go` commands to run as an unbounded search until a limit or
  control command stops the search, instead of using a hidden shallow default.
- Changed UCI coordinate move parsing to accept uppercase square text, while
  still normalizing internally to the canonical lowercase move form.
- Changed `setoption` handling to wait for any active search to finish before
  applying engine configuration.

### Fixed

- Fixed compatibility with tournament managers that emit the common
  non-standard fullmove number `0` by normalizing it to fullmove `1` during FEN
  parsing.
- Fixed root draw positions, including fifty-move claim and dead-material
  positions, so the engine still returns a legal move when legal moves exist.
  `bestmove 0000` is now reserved for positions with no legal moves.

## [1.3.2] - 2026-05-28

### Added

- Added a true threshold `see_ge` implementation so search pruning and move
  ordering can test exchange safety without always computing full SEE.
- Added transposition-table prefetching before child probes in main search,
  quiescence search, null-move search, ProbCut, and MultiPV root searches.
- Added root-effort-aware soft time scaling so unstable or low-confidence root
  searches can spend more of the available soft budget.
- Added regression coverage for promotion SEE thresholds, staged bad-capture
  ordering, mixed valid/invalid `searchmoves`, and current-generation
  `hashfull` accounting.

### Changed

- Changed staged move picking to search good captures first, generate quiets
  lazily, and delay losing captures until after quiet moves.
- Changed tactical capture scoring and pruning to use threshold SEE in hot
  paths, reducing full-SEE work during search.
- Made late-move reductions cut-node aware and slightly more sensitive to
  losing captures and strong quiet history.
- Made quiescence pruning use dynamic threshold SEE and late losing-capture
  filtering.
- Changed `hashfull` reporting to count only entries from the current TT
  generation for both local and shared tables.

### Fixed

- Fixed SEE accounting for promotion captures by using the promoted piece as
  the next attacker in the exchange sequence.
- Fixed invalid `go searchmoves` input that matches no legal root move to fall
  back to the full legal root move list instead of returning a spurious draw.
- Fixed excluded-move searches so the excluded move is not counted as a legal
  searched move.

## [1.3.1] - 2026-05-28

### Added

- Added stricter FEN validation for complete ranks, legal pawn ranks, one king
  per side, adjacent kings, side-not-to-move check legality, castling-right
  consistency, and valid move counters.
- Added canonical en passant hashing so EP squares are kept in the position key
  only when a legal en passant capture exists.
- Added continuation correction history to the handcrafted-evaluation
  correction path.
- Added UCI `go searchmoves` root filtering and `go mate` depth handling.
- Added regression coverage for invalid FEN rejection, canonical EP handling,
  rule-50-aware TT mate recovery, insufficient-material search draws,
  `searchmoves`, and `mate` parsing.

### Changed

- Made transposition-table mate score recovery aware of the current rule-50
  counter to avoid reusing forced-mate scores past the 50-move horizon.
- Changed more search cutoffs to preserve fail-soft scores in TT storage and
  return values.
- Added staged main-search move picking so captures can be searched before
  quiet moves are generated at non-root non-check nodes.
- Reworked soft/hard clock allocation with a move-count horizon, explicit
  `movestogo` support, increment handling, and move-overhead reserve.
- Extended quiescence search depth and avoided static-evaluation fallback while
  still in check.
- Made quiet halfmove-clock increments saturating for robustness on high-clock
  positions.

### Fixed

- Fixed search scoring for insufficient-material positions so dead draws return
  an immediate draw result instead of non-zero material/PST scores.
- Fixed a crash path where illegal FENs with the side not to move already in
  check could reach move generation.

## [1.3.0] - 2026-05-27

### Added

- Added incremental pawn, minor-piece, and color-specific non-pawn structure
  keys for more precise pawn keys and richer correction history indexing.
- Added low-ply quiet history and pawn-structure-indexed quiet history to
  improve move ordering in the main search.
- Added additional continuation-history channels at wider ply distances.
- Added high-depth null-move verification to reduce tactical risk from
  aggressive null-move cutoffs.
- Added capture futility pruning in quiescence search.
- Added fifty-move-rule dampening to the handcrafted evaluation.
- Added regression coverage for pawn-key color/structure collisions and
  evaluation scaling near a fifty-move-rule draw.

### Changed

- Reworked static-evaluation handling so transposition-table entries store raw
  static evals and apply the current correction history on probe.
- Improved pruning selectivity with TT-assisted pruning evals, improving-aware
  futility margins, and refined shallow razoring/null-move conditions.
- Improved tactical move ordering with SEE-aware capture scores and capture
  history while ordering losing captures behind quiet candidates.
- Improved root time usage by extending past the soft limit after material
  score drops and by stopping early in forced single-legal-move positions.
- Improved clock allocation for explicit `movestogo` and sudden-death controls
  with more conservative hard limits and increment handling.

## [1.2.1] - 2026-05-25

### Changed

- Made timed-search allocation more conservative at fast controls by assuming
  a slightly longer remaining game and lowering the hard-stop cap. This reduces
  avoidable time forfeits while leaving fixed-depth and fixed-movetime search
  behavior unchanged.

### Added

- Added time-manager regression coverage for fast sudden-death clocks,
  fixed `movetime`, side-to-move clock selection, explicit `movestogo`,
  minimum allocation, and unbounded fixed-depth searches.
- Added UCI option/parser regression coverage for default `go`, invalid search
  limit values, invalid `setoption` values, and preservation of previous
  engine option values after rejected input.

## [1.2.0] - 2026-05-25

### Added

- Added UCI `MultiPV` option and basic sequential MultiPV root analysis output.
- Added `tbhits` reporting in UCI search info.
- Added Syzygy tablebase load summaries that report WDL and DTZ file counts and
  the largest loaded tablebase cardinality.
- Added root Syzygy move ranking through Fathom's DTZ root API, with WDL root
  fallback when DTZ tables are unavailable.
- Added regression coverage for tablebase file counting, Fathom root move
  decoding, local Syzygy root probing when tables are present, root repetition
  exposure, `tb_hits`, and the MultiPV search path.

### Changed

- Changed root tablebase behavior to filter and search tablebase-correct root
  moves instead of immediately returning a single zero-node root move.
- Improved Syzygy UCI defaults and practical root tablebase usage for common
  GUI and tournament-manager setups.
- Passed repetition state into root DTZ probing so root tablebase ranking can
  account for repeated positions.
- Specialized color-specific attack lookup and reduced repeated bitboard
  queries in evaluation hot paths for a modest NPS improvement.

### Fixed

- Fixed root move restriction handling so tablebase-filtered and
  helper-thread root move lists are actually respected by the root search.

## [1.1.0] - 2026-05-25

### Added

- Added optional Syzygy tablebase support using the vendored Fathom probe
  library, with UCI options for path, probe depth, probe limit, and
  fifty-move-rule handling.
- Added tests for Syzygy option parsing, Fathom result decoding, disabled-path
  probing, root move conversion, and malformed path handling.

### Changed

- Improved search selectivity and speed with faster check detection, stronger
  stale-TT internal iterative reduction, deeper null-move reductions,
  check-aware pruning, and adjusted late move reductions.
- Improved tactical reliability by continuing quiescence search through check
  evasions instead of falling back to static evaluation while in check.
- Stored ProbCut lower bounds in the transposition table to improve move
  ordering and cut reuse.

### Fixed

- Reduced the risk of illegal or tactically losing search choices from
  over-pruning checking moves and check evasions.

## [1.0.2] - 2026-05-25

### Changed

- Removed Intel macOS release assets from the GitHub release build matrix.
  macOS release binaries are now Apple Silicon/ARM64 only.
- Clarified release asset CPU targets and local native build behavior in the
  README.

## [1.0.1] - 2026-05-23

Patch release focused on UCI tournament reliability.

### Fixed

- Delayed `bestmove` emission for completed `go ponder ...` searches until
  `ponderhit`, `stop`, or `quit`, matching the UCI expectation that a completed
  ponder result is retained but not reported early.
- Delayed `bestmove` emission for completed `go infinite` searches until
  `stop` or `quit`.
- Preserved the `ponder` flag when parsing combined `go ponder infinite`
  commands.
- Flushed UCI `uciok`, `readyok`, and `bestmove` replies immediately for
  pipe-based GUIs and tournament managers.

### Added

- Regression tests covering UCI ponder and infinite bestmove-release behavior.
- Search and option-parser coverage for ponderhit conversion, infinite search
  limits, and temporary limit reset behavior.

## [1.0.0] - 2026-05-22

Initial Lynx release.

### Implemented

- Custom Rust board representation based on bitboards, mailbox lookup, fixed
  move lists, incremental make/unmake, and Zobrist hashing.
- Complete legal move generation for standard chess, including castling,
  en passant, promotions, checks, pins, repetitions, the fifty-move rule, and
  insufficient material detection.
- Perft and board correctness test coverage for common reference positions and
  special move edge cases.
- Search benchmark support through the UCI `bench [depth]` command.
- Release benchmark and performance tests for board operations.
- `cargo bench --bench board` board implementation benchmark for cross-version
  and cross-engine comparison.
- UCI `Threads` option with Lazy SMP-style persistent-worker search and packed
  shared-transposition-table support up to 1024 threads.
- Epoch-tracked UCI command control with prioritized `quit`, asynchronous
  `stop`/`ponderhit`, serialized idle `isready` handling, and in-order EOF
  shutdown for redirected UCI sessions.
- Threaded root result selection and fixed-depth helper root diversification for
  stronger Lazy SMP behavior during both timed searches and benchmarks.
- Capture-focused qsearch with bounded check evasions and SEE-based pruning.
- Transposition table, pawn cache, correction history, main history, capture
  history, continuation history, killers, and countermoves.
- Advanced search features: PVS, aspiration windows, null-move pruning,
  ProbCut, singular extensions, futility pruning, late move pruning, late move
  reductions, SEE move ordering/pruning, and quiescence search.
- Handcrafted tapered evaluation with material, PeSTO-style piece-square tables,
  pawn structure, passed pawns, mobility, rook activity, outposts, threats,
  king safety, draw scaling, and winning-king push.
- Release profile tuning with fat LTO, one codegen unit, and `panic = "abort"`.

### Completed

- Implemented all board, search, and evaluation logic inside the engine without
  external chess crates.
- Reworked UCI engine configuration around the custom board and searcher.
- Updated the built-in `bench` command to use the current UCI options,
  including `Threads`.
- Hardened transposition-table resizing so failed large `Hash` allocations keep
  the current table instead of leaving search without TT storage.
- Hardened the shared transposition table with full-key validation to avoid
  cross-thread partial-key collisions.
- Updated qsearch to avoid full legal move regeneration outside check and to
  reuse transposition-table results in tactical leaf searches.
- Made `ponderhit` reset the active search clock after ponder search converts
  to normal thinking.
- Made helper-thread creation fail gracefully if the OS cannot create every
  requested worker.
- Kept UCI node-limited searches on the main search path to preserve the
  requested node limit when `Threads` is greater than one.
- Rebranded the package, binary, UCI identification, release workflow, and
  documentation for Lynx.
- Set the initial Lynx release version to `1.0.0`.
- Configured GitHub release assets to publish portable x86-64 and ARM64 builds,
  plus optimized AVX2 and AVX-512 builds where supported.
- Expanded documentation for UCI support, benchmarking, testing, and engine
  internals.
