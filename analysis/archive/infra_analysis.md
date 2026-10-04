# Rarog Board and Infrastructure Analysis

**Audit date:** 2026-07-13  
**Engine version:** Rarog 2.3.0, `development` at `ff21dc1`  
**Scope:** board representation, move/state transitions, legal move generation,
SEE, hashing and repetition, evaluation-facing state, transposition-table
layout, build/release configuration, correctness/performance testing, and
strength-testing infrastructure.

This is a living document. It incorporates the applicable findings from
`D:\code\basilisk\analysis\infra_analysis.md`, but every transferred finding
was checked against Rarog rather than assumed to apply. Demonstrated defects
are separated from performance hypotheses that still require benchmarking and
SPRT.

## 1. Executive conclusion

Rarog's fundamental board representation is already competitive. The engine
has a coherent hybrid bitboard/mailbox board, compact 16-bit moves, fixed move
lists, incremental full and material-class Zobrist keys, legal en-passant
canonicalization, PEXT/magic sliding attacks, in-place make/unmake, and broad
perft/state tests. Replacing the representation wholesale is not justified.

The highest-confidence board/infrastructure losses are instead:

1. rule-50 draw handling can override checkmate;
2. null moves incorrectly advance the rule-50 clock;
3. search repetition is neither root-aware nor bounded by the last null move;
4. SEE counts absolutely pinned recapturers;
5. TT keys do not distinguish materially different rule-50 states;
6. the recursive move picker occupies roughly 9 KB before additional
   per-node lists;
7. shared-TT conversion approximately doubles the requested `Hash` memory;
8. official release binaries are not PGO builds even though local strength
   binaries are;
9. dependency/toolchain and match-result provenance are not reproducible;
10. no PR CI, property/differential testing, fuzzing, or full debug invariant
    checker protects the board.

The board is not concealing the whole remaining 200+ Elo gap. Correctness and
hot-path fixes here should be treated as mandatory and may cumulatively recover
tens of Elo, but the frontier-sized gain still depends on NNUE quality,
training data, and search re-tuning. The board redesign should therefore be
driven by a clean per-ply state and dirty-feature interface, not by cosmetic
similarity to Stockfish.

## 2. Reference engines and comparison boundary

The CCRL 40/15 table checked during the audit listed Stockfish 18, PlentyChess
7, Torch v4, Obsidian 16, and Reckless 0.8 at the top:
<https://www.computerchess.org.uk/ccrl/4040/>.

Torch is proprietary. Implementation comparisons therefore use the following
open-source revisions current at the time of the audit:

| Engine | Revision inspected | Relevant source |
|---|---|---|
| Stockfish | `9a8dd81` | [position state](https://github.com/official-stockfish/Stockfish/blob/9a8dd81dd7f98cbf02f16c59b4377d174d6eb4b5/src/position.h), [move/state updates](https://github.com/official-stockfish/Stockfish/blob/9a8dd81dd7f98cbf02f16c59b4377d174d6eb4b5/src/position.cpp), [search](https://github.com/official-stockfish/Stockfish/blob/9a8dd81dd7f98cbf02f16c59b4377d174d6eb4b5/src/search.cpp) |
| PlentyChess | `04e07a9` | [board state](https://github.com/Yoshie2000/PlentyChess/blob/04e07a98ee6ac104c30e7374450c94b96d94ef4d/src/board.h), [board updates](https://github.com/Yoshie2000/PlentyChess/blob/04e07a98ee6ac104c30e7374450c94b96d94ef4d/src/board.cpp), [search](https://github.com/Yoshie2000/PlentyChess/blob/04e07a98ee6ac104c30e7374450c94b96d94ef4d/src/search.cpp) |
| Reckless | `dd0e676` | [board state](https://github.com/codedeliveryservice/Reckless/blob/dd0e676007f2e53e1bc59054b24b6ca9003d9ca2/src/board.rs), [move picker](https://github.com/codedeliveryservice/Reckless/blob/dd0e676007f2e53e1bc59054b24b6ca9003d9ca2/src/movepick.rs) |

Ratings from different lists, hardware, books, and time controls are not
directly comparable. The implementation comparison is more useful than
subtracting Rarog's approximate rating from one headline number.

## 3. Applicability of the Basilisk findings

| Basilisk finding | Rarog status | Evidence/decision |
|---|---|---|
| Rule-50 draw overrides mate | **Applies; reproduced** | Sections 5.1 and 13 |
| Null move advances rule-50 | **Applies; reproduced** | `board.rs:1261-1289` |
| SEE accepts pinned recapturers | **Applies; reproduced** | `board.rs:925-945`, Section 5.4 |
| SEE accepts illegal king recaptures | **Not reproduced** | Tested both protected and unprotected king recaptures; add independent-oracle coverage |
| EP hash is pseudo-legal only | **Already handled** | `legal_ep_capture_exists()` checks king exposure; dedicated tests exist |
| Fixed history capacity | **Does not apply** | Rarog uses a reserved but dynamically growing `Vec<UnmakeInfo>` |
| Pins recomputed per generation stage | **Applies** | Captures and quiets independently call legal generation |
| Checker set fully recomputed after moves | **Applies** | `make_move()` always calls `calculate_checkers()` |
| Quiet-check generator scans all legal moves | **Does not apply** | Rarog qsearch currently searches captures/promotions, not a filtered all-moves quiet-check stage |
| EP TT validation copies Board | **Does not apply** | Rarog uses occupancy-based EP legality |
| Material/PST/phase rebuilt on evaluation | **Applies on cold eval** | Whole-eval cache masks hits; misses walk pieces and rebuild attack maps |
| Move encoding is wider than 16 bits | **Already handled** | `Move(pub u16)` |
| Missing per-ply derived state | **Applies** | Only `checkers` is persisted; pins/threats/check squares are reconstructed |
| Rule-50-aware TT key missing | **Applies** | TT is probed with raw `board.hash` |
| Child TT prefetch missing | **Already handled** | Rarog prefetches after real and null moves |
| Upcoming repetition missing | **Applies** | Only backward hash scanning exists |
| Chess960 castling support missing | **Applies, low standard-Elo priority** | Castling squares are hard-coded for standard chess |
| PGO exists but release workflow does not use it | **Applies** | Local `xtask --pgo` exists; release workflow omits `--pgo` |
| CPU tiers are fragmented | **Mostly handled** | PEXT is x86-64-v3 + BMI2; AVX2 is v3; future NNUE needs broader kernels/dispatch |
| No PR CI/property/fuzz/invariant coverage | **Applies** | Only release-publish workflow exists |
| Performance harness is not a reliable gate | **Applies** | Single short sample or best-of-three, no reliable affinity/statistics |
| Match artifacts lack reproducible manifests | **Applies** | PGNs exist, but build/test metadata and seeds are not persisted |

## 4. What Rarog already does well

### 4.1 Board representation

`src/board/board.rs:66-89` stores:

- 12 colored-piece bitboards;
- occupancy for both colors and combined occupancy;
- a 64-square mailbox;
- side, castling, en-passant, halfmove and fullmove state;
- full, pawn, minor, and per-color non-pawn hashes;
- cached current checkers;
- reversible history.

Measured layout on the audit compiler:

| Type | Size |
|---|---:|
| `Board` | 264 bytes |
| `UnmakeInfo` | 24 bytes |
| `Move` | 2 bytes |
| `MoveList` | 520 bytes |

This is a reasonable layout. Stockfish and Reckless use piece-type and color
bitboards plus a mailbox, while PlentyChess copies a substantially larger
complete board per ply and remains elite. Rarog's 12 colored bitboards trade
some stores and space for direct colored-piece access; that trade should be
changed only after full-search profiling with the intended NNUE evaluator.

### 4.2 Move generation and en-passant identity

`src/board/movegen.rs` performs strict legal generation with:

- single- and double-check handling;
- absolute pin filtering;
- legal king destinations;
- castling transit-square validation;
- en-passant discovered-check validation;
- staged capture and quiet entry points.

Unlike the Basilisk version audited alongside it, Rarog stores an en-passant
square only when at least one legal EP capture exists. Both FEN parsing and
double-pawn pushes call `legal_ep_capture_exists()`
(`src/board/board.rs:1220-1229`, `1520-1575`). Tests at
`tests/board_correctness.rs:1066-1116` cover pinned and non-capturable EP
canonicalization. This is correct modern repetition identity behavior.

### 4.3 Hashing and reversible state

Full, pawn, minor and non-pawn keys are incrementally maintained in the piece
add/remove helpers. The full key is restored from `UnmakeInfo`; make/unmake
piece operations reverse the specialized keys. History is dynamic, so the
fixed-capacity release overflow found in Basilisk does not transfer.

### 4.4 Build and strength-testing foundation

Rarog already has:

- fat-LTO release builds;
- explicit generic, x86-64-v3/AVX2, PEXT/BMI2, native, and ARM64 build modes;
- a complete Rust PGO generate/train/merge/use driver;
- a deterministic 40-position internal bench with node fingerprint and EBF;
- local fastchess SPRT, gauntlet, SPSA, Texel tuning, and data-generation tools;
- paired-color openings and LTC confirmation guidance.

These are meaningful strengths. The recommendations below make them
reproducible and continuously enforced rather than replacing them.

## 5. Demonstrated correctness and search-state problems

### 5.1 Rule-50 draw can override checkmate

**Severity:** P0 correctness  
**Locations:** `src/board/board.rs:830-842`, `1029-1045`;
`src/search.rs:954`, `1620`

Both board-level game result and search draw checks return draw at a halfmove
clock of 100 before determining whether the side to move is checkmated.

Reproduction:

```text
position fen 7k/8/5KQ1/8/8/8/8/8 w - - 98 1
go depth 2
  -> score mate 1, bestmove g6g7

position fen 7k/8/5KQ1/8/8/8/8/8 w - - 99 1
go depth 2
  -> score cp 0, bestmove g6h5
```

`Qg7#` is a quiet mating move. In the second position the child clock becomes
100 and search returns draw before recognizing checkmate.

Recommended behavior is equivalent to:

```text
rule50 >= 100 && (!in_check || at_least_one_legal_move_exists)
```

Only the exceptional checked-at-the-boundary case requires generating moves.
Add regression coverage at board-result, normal-search, and qsearch levels.

### 5.2 Null moves incorrectly advance the halfmove clock

**Severity:** P0/P1 search-state correctness  
**Location:** `src/board/board.rs:1261-1289`

`make_null_move()` clears EP, flips side, and executes:

```rust
self.halfmove_clock = self.halfmove_clock.saturating_add(1);
```

A null move is not a played reversible move. It should preserve the game
rule-50 clock and reset only a separate `plies_from_null` repetition boundary.

Focused probe:

```text
start halfmove=99
make_null_move()
observed halfmove=100, can_declare_draw_in_search=true
```

The present behavior can manufacture a draw inside a null-move subtree and
will also contaminate any future rule-50-aware TT key. Preserve
`halfmove_clock`, introduce `plies_from_null`, and test 98/99/100 boundary
positions through null make/unmake.

### 5.3 Repetition is not root-aware or null-aware

**Severity:** P0/P1 search correctness  
**Location:** `src/board/board.rs:1620-1635`

`is_repetition()` scans every two history entries up to `halfmove_clock` and
counts matching raw hashes. It receives neither the current search ply nor the
root-history boundary and cannot identify null moves.

Consequences:

1. A second occurrence whose only earlier copy is before the root may be
   treated as an immediate search draw. Modern search semantics normally
   require the first repeat to lie inside the current search; otherwise two
   historical copies are needed for a threefold claim.
2. The scan can cross a null-move boundary and match positions that do not
   belong to the same legal game line.
3. Rarog cannot implement efficient upcoming-repetition cutoffs without a
   clearer per-ply repetition state.

Recommended model:

```text
scan_limit = min(rule50, plies_from_null, available_history)
first match inside current search -> search draw
match before root -> require another historical match
```

Add tests where the matching occurrence is before versus after root, and where
a null move lies between current position and a matching historical hash.

### 5.4 SEE counts absolutely pinned recapturers

**Severity:** P0 functional search bug  
**Locations:** `src/board/board.rs:886-1027`, especially `925-945` and
`1003-1023`

Both `see()` and `see_ge()` calculate attackers from geometric attack sets and
occupancy, select the least valuable attacker, and reveal x-rays. Neither
filters an attacker that is absolutely pinned to its king.

Reproduction:

```text
FEN:  4k3/4n3/2p5/1B6/8/8/8/K3R3 w - - 0 1
Move: Bxc6
```

The black knight on e7 is pinned to the king on e8 by the rook on e1 and cannot
recapture on c6. Correct static exchange gain is the pawn, `+100`.

Observed:

```text
see(Bxc6)    = -230
see_ge(Bxc6, 0) = false
```

This has broad search fan-out. `see_ge()` is used in capture staging, ProbCut,
main-search SEE pruning, quiescence pruning, and capture-history decisions at
multiple sites in `src/search.rs`.

Fix options:

- compute blockers/pinners for the exchange position and exclude a pinned
  attacker while its pinner remains;
- or use a localized king-ray legality test when selecting every attacker.

The first option aligns with the proposed cached per-ply king geometry. Add a
slow legal exchange oracle for tests rather than comparing `see_ge()` only to
`see()`, because the current test compares two implementations sharing the
same omission.

### 5.5 Illegal king recaptures were not reproduced

The analogous Basilisk report found a separate king-recapture bug. Rarog's SEE
was probed with both cases below:

```text
4k3/3p4/8/1B6/8/8/8/K7   w - - 0 1, Bxd7
  black Kxd7 is legal       -> see=-230, see_ge(0)=false

4k3/3p4/8/1B6/8/8/8/K2R4 w - - 0 1, Bxd7
  black Kxd7 is illegal     -> see=+100, see_ge(100)=true, see_ge(101)=false
```

These results are correct. Rarog effectively lets a defended king capture be
answered by capturing the king at king value, which produces the correct
result in the tested exchange trees. This area still needs independent-oracle
and randomized coverage; it is not currently classified as a demonstrated
Rarog bug.

### 5.6 TT search key ignores the rule-50 horizon

**Severity:** P1 graph-history correctness/strength  
**Locations:** `src/search.rs:976-1005`, `1625-1638`; `src/tt.rs:383-395`

TT probe/store keys are raw `board.hash`, which intentionally excludes the
halfmove clock. `score_from_tt()` rejects mate distances that cannot fit before
the rule-50 boundary, but ordinary exact/lower/upper scores remain reusable.

The same piece placement at clocks 10 and 98 can therefore share a bound even
though its practical winning horizon is materially different. Stockfish,
PlentyChess, and Reckless mix a bucketed rule-50 component into the **search**
key while preserving the raw position key for repetition.

Recommended split:

```text
repetition_key = board.hash
tt_key         = board.hash XOR rule50_bucket[halfmove / granularity]
```

Implement only after null-move clock behavior is corrected. Test identical
positions at several halfmove values and include near-boundary ordinary TT
scores, not only mates.

## 6. Board and search hot-path losses

### 6.1 `MovePicker` and the recursive frame are oversized

**Confidence:** high  
**Strength effect:** strongest pure throughput candidate; benchmark and SPRT

Measured layouts:

| Type | Rarog | Current Reckless reference |
|---|---:|---:|
| `ScoredMove` | 12 bytes | — |
| `ScoredMoveList` | 3,080 bytes | — |
| `MovePicker` | 9,288 bytes | 2,608 bytes |
| `BadCaptureList` | 776 bytes | — |

The staged `MovePicker` embeds capture, bad-capture, and optional quiet scored
lists. Negamax additionally allocates a 520-byte quiet `MoveList` and two
776-byte `BadCaptureList`s (`src/search.rs:1228-1230`). More than 11 KB of
move-related state is therefore live in a recursive frame before other locals.

Both the engine thread and search helpers request 16 MB stacks
(`src/main.rs:9,31-42`, `src/search_threads.rs:16,123`). Large recursive frames
can cause Windows stack probing, extra cache traffic, and unnecessarily high
per-thread virtual/committed stack requirements.

Recommended redesign:

1. use one scored buffer and partition it in place into good captures, quiets,
   and bad captures;
2. retain delayed bad captures as `Move` plus only the metadata actually needed
   for history updates;
3. avoid simultaneously retaining whole scored capture and quiet arrays;
4. inspect optimized assembly for `__chkstk`/stack-probe calls;
5. reduce thread stack size only after a worst-case depth test proves the new
   bound.

Node count must remain identical for a layout-only patch. Measure best/median
NPS on fixed affinity, then run a simplify or gainer SPRT as appropriate.

### 6.2 Capture and quiet stages recompute king geometry

**Confidence:** high that work is duplicated; unknown net benefit of redesign

`MovePicker::staged()` first calls legal capture generation and later legal
quiet generation. Both paths compute pinned pieces. Capture generation also
runs `has_pseudo_capture()` before performing the real generation scan.

At nodes reaching quiets, this repeats:

- diagonal and orthogonal pinner discovery;
- check-mask setup;
- some king safety and piece setup;
- piece/slider scans in the capture precheck.

Possible experiments, kept as separate patches:

1. delete `has_pseudo_capture()` and measure full-search NPS;
2. calculate pin/blocker state once and pass it to both legal stages;
3. cache blockers/pinners/check squares in per-ply state;
4. prototype pseudo-legal staged generation with legality checked only after a
   move survives ordering/pruning.

Strict legal generation is not intrinsically weak; Reckless also uses strict
legal machinery. The objective is avoiding repeated geometry, not copying a
particular engine's API.

### 6.3 Checker information is calculated twice

**Location:** `src/board/board.rs:613-667`, `1258`, `1455-1466`;
`src/search.rs:1249-1348`

Search caches `board.gives_check(mv)` for pruning and extensions. After the
move, `make_move()` discards that result and always reconstructs the complete
checker set with pawn, knight, bishop, and rook attacks.

Stockfish accepts a precomputed `givesCheck` flag in `do_move()`. A safe Rarog
experiment is:

- pass `Option<bool>` or a trusted `gives_check` result to an internal move
  update;
- set `checkers = EMPTY` immediately for known non-checking moves;
- calculate the exact checker set only for checking moves and special
  discovered/EP/castling cases;
- keep the public safe path that computes everything when no hint exists.

Do not infer the exact checker square from a Boolean without handling discovered
and double checks.

### 6.4 Cold HCE evaluation reconstructs material and threats

**Location:** `src/eval.rs:1027-1153`, `1384-1435`

Rarog has a whole-evaluation cache keyed by full position hash and halfmove
clock, so exact cache hits are cheap. On a miss it:

- walks every piece for material/PST/phase;
- constructs per-square attacks;
- constructs per-piece attack unions, attacked-once/twice maps, and combined
  attacks;
- then consumes those maps in mobility, king safety, hanging-piece, and other
  terms.

The attack-map work is well shared **within one evaluation**, but none of it is
available to move ordering or the next position. Incremental HCE material/PST
totals could help the current branch, but avoid loading the board with fields
that become dead after NNUE.

Prefer universally useful state:

- piece counts/non-pawn material;
- dirty piece deltas;
- king bucket/change flags;
- optionally threat maps if they feed both move ordering and NNUE inputs.

### 6.5 TT child prefetch is already implemented

The Basilisk recommendation to add child prefetch does not transfer. Rarog
prefetches `board.hash` after real moves, ProbCut moves, qsearch moves, and null
moves (`src/search.rs:1088`, `1147`, `1348`, `1757`).

After adding a rule-50-adjusted TT key, these sites must prefetch the adjusted
key rather than the raw hash. That is a migration requirement, not a new
feature.

### 6.6 Startup attack-table generation

`ATTACKS` is a `LazyLock`. Generic builds search for magic multipliers at
startup; PEXT builds enumerate their attack tables. Five local launches during
the audit were approximately:

| Build | Startup range |
|---|---:|
| Generic magic | 375-429 ms |
| Native/PEXT | 174-199 ms |

This is not in-game Elo, but it affects UCI responsiveness, test orchestration,
and determinism. Bake stable magics/table metadata at build time or commit
generated constants. Keep the runtime attack arrays if that remains the best
cache/layout choice.

## 7. Transposition table and SMP infrastructure

### 7.1 Shared mode approximately doubles requested `Hash`

**Severity:** P1 UCI contract and scaling  
**Location:** `src/tt.rs:63-68`, `77-145`, `400-431`

Layout:

```text
LocalCluster  = 32 bytes: 3 x 10-byte entries + padding
SharedCluster = 64 bytes: 3 x 16-byte atomic entries + alignment padding
```

The initial cluster count is calculated from `LocalCluster`. When threads are
enabled, `shared_from_local()` creates the same number of `SharedCluster`s.
Requested memory therefore grows by roughly 2x; e.g. a nominal 64 MB TT becomes
approximately 128 MB, temporarily in addition to the local allocation during
conversion.

This is not automatically a single-thread Elo loss—extra TT capacity may even
help—but it violates the advertised memory limit and can cause cache/NUMA or
host-memory problems.

Recommended options:

1. allocate shared cluster count from the requested byte budget;
2. fit four 16-byte atomic entries in each 64-byte shared cluster;
3. or design a compact packed atomic format while retaining collision/torn-read
   protection.

Add tests for actual allocated bytes and cluster counts at 1 and multiple
threads.

### 7.2 Atomic design is safe but relatively expensive

Every shared probe loads two atomics per entry and validates a full key through
`key_xor_data`. This is robust against torn entry reads and SMP collisions, but
costlier than compact racy clusters used by some top engines. Preserve
correctness first. Only revisit packing after fixing the memory-size contract
and profiling multi-thread scaling.

### 7.3 No large-page or NUMA policy

Rarog uses ordinary Rust allocation for TT and private history tables. There is
no large-page allocator, first-touch policy, or NUMA replication/placement.
This is low priority for the current single-thread testing regime but becomes
relevant for large hash tables and high core counts. Measure scaling before
adding platform-specific complexity.

## 8. Per-ply state and NNUE-ready architecture

### 8.1 Current state boundary

`UnmakeInfo` stores captured piece, castling, EP, clocks, full hash, and old
checkers. Pawn/minor/non-pawn hashes are reversed through piece operations rather
than restored directly. The board does not persist:

- `plies_from_null` or root-aware repetition status;
- blockers/pinners for either king;
- per-piece checking squares;
- threat maps;
- dirty piece/threat updates;
- NNUE accumulator state.

### 8.2 Recommended separation

```text
Board / Position
  piece and occupancy bitboards
  mailbox
  side to move
  standard or future Chess960 castling metadata

StateInfo (one per ply)
  full/pawn/minor/non-pawn keys
  castling, EP, rule50, pliesFromNull
  captured piece and previous-state link/index
  checkers
  blockersForKing[2]
  pinners[2]
  checkSquares[piece type]
  repetition distance/status
  DirtyPiece / optional DirtyThreat delta

Per-thread evaluator
  NNUE accumulator stack
  king-bucket refresh cache
  quantized inference scratch
```

The accumulator should not be embedded in every copyable `Board`. Search
already has one position per worker; evaluator state should follow search ply.

### 8.3 Dirty update contract

Define a board update result usable by HCE experiments and NNUE:

```text
removed: [(color, piece, square)]
added:   [(color, piece, square)]
king_moved / king_bucket_changed
optional changed threat relations
```

It must cover normal moves, captures, promotions, en passant, castling, and
null moves. Random make/unmake tests should compare incremental NNUE state with
a full refresh after every ply.

### 8.4 Threat inputs should be staged

Current Stockfish propagates dirty threat information, and current PlentyChess
stores piece-specific threat maps for its threat-input NNUE. Rarog should land:

1. a correct baseline accumulator and refresh cache;
2. quantized SIMD inference and trainer reproducibility;
3. search re-tuning at the NNUE score scale;
4. threat features only after the baseline is stable.

The board hooks should permit step 4 without another make/unmake rewrite.

### 8.5 Upcoming repetition and Chess960

Upcoming repetition using reversible-move cuckoo keys belongs after repetition
state is corrected. It can identify repetition-producing moves before searching
every child.

Castling currently hard-codes E1/E8 kings and A/H rooks. Chess960 is not a
near-term standard-chess Elo target, but generalized rook-square/path metadata
would improve the state model and unlock FRC regression testing.

## 9. Build and release infrastructure

### 9.1 PGO exists locally but is not shipped

**Severity:** P1 user-facing performance  
**Locations:** `xtask/src/main.rs:232-279`; `.github/workflows/build.yml:79`

`cargo xtask build --arch ... --pgo` correctly:

1. builds an instrumented engine;
2. trains it with the internal depth-13 bench;
3. merges `.profraw` files;
4. rebuilds with `profile-use` and fat LTO;
5. emits a `-pgo` distribution artifact.

Local SPRT binaries use this path. The release workflow calls:

```text
cargo xtask build --arch <tier> --target <target>
```

without `--pgo`. Official assets are therefore ordinary O3/fat-LTO builds,
not the configuration used for local strength validation.

Recommended release flow:

- run tests on an ordinary build;
- PGO-build targets that can train natively on their hosted runner;
- smoke-test the exact final binary;
- upload the `-pgo` artifact with matching filename logic;
- record compiler, profile workload/depth, bench fingerprint, and SHA.

Cross-target builds cannot train locally; keep a documented non-PGO fallback
or use target-native runners.

### 9.2 Dependency and compiler inputs are floating

**Severity:** P1/P2 reproducibility

Rarog is an application/workspace, but `.gitignore` excludes `Cargo.lock` and
the lockfile is not tracked. There is also no `rust-toolchain.toml` or
`rust-version`; CI installs floating `stable`.

Consequences:

- rebuilding one commit later may select different `cc`/transitive crates;
- a newer rustc/LLVM can change code generation, node speed, PGO behavior, and
  even layout assumptions;
- a released binary cannot be reproduced from git SHA alone.

Commit `Cargo.lock`, pin or record the production Rust toolchain, and include
`rustc -vV`, Cargo version, target triple, and effective encoded rustflags in
release and candidate manifests.

### 9.3 ISA tiers are mostly coherent

Unlike Basilisk's split PEXT/AVX2 configuration, Rarog's PEXT tier uses
`target-cpu=x86-64-v3` plus BMI2 and its AVX2 tier uses x86-64-v3. The generic
tier uses `target-cpu=x86-64`; native is explicitly local-only. Asset naming is
clear.

Remaining work:

- runtime validation for the PEXT asset currently checks BMI2 only, while the
  binary also assumes the full v3 feature set;
- ARM64 uses a generic CPU target;
- there is no AVX-512/VNNI tier or runtime kernel dispatch.

Those gaps become important with NNUE. They are not a reason to add premature
SIMD complexity to the current HCE.

## 10. CI and board correctness testing

### 10.1 No push or pull-request CI

`.github/workflows/build.yml` runs only when a release is published. It does
not run on push/PR and does not execute formatting, clippy, tests, sanitizer
builds, or a bench signature before uploading assets.

Recommended split:

| Job | Suggested coverage |
|---|---|
| PR fast | `cargo fmt --check`, clippy, release tests on Linux and Windows |
| PR board | standard perft/hash signature and focused rule/SEE regressions |
| Sanitizer | ASan/UBSan on randomized state transitions |
| Nightly | longer differential random walks, deeper perft, optional aarch64 |
| Release | full ISA/platform matrix, PGO, smoke test exact upload artifacts |

### 10.2 Existing tests are broad but self-consistency-heavy

The audit ran 160 release tests successfully. Coverage includes CPW perft
positions, special moves, FEN validation, make/unmake, repetition, hashing,
SEE threshold agreement, eval cache invariants, search, UCI, threading,
Syzygy, and ponder behavior.

The demonstrated bugs show the missing layer:

- the repetition test covers game-level threefold, not root/null search
  semantics;
- the rule-50 test verifies that 100 is a draw, not checkmate precedence;
- SEE tests compare `see_ge()` with `see()`, allowing a shared legality bug;
- make/unmake snapshots compare FEN, full hash, and checkers, but not pawn,
  minor, non-pawn keys or independently reconstructed occupancy/mailbox state.

### 10.3 Add property and differential tests

Create a deterministic seeded random walker:

1. start from several valid FENs;
2. play random legal moves, including promotion/castling/EP-heavy starts;
3. after every move, independently rebuild and compare:
   - piece and color occupancy;
   - mailbox versus bitboards;
   - full/pawn/minor/non-pawn keys;
   - EP and castling validity;
   - checkers and future blockers/pinners/check squares;
   - incremental versus cold evaluation/NNUE refresh;
4. unmake the sequence and require complete field equivalence.

Compare legal move sets and perft over random legal positions against an
independent implementation such as Stockfish or python-chess. Do not use
Rarog's pseudo-legal generator as the oracle for Rarog's legal generator.

For SEE, implement a slow legal capture-tree oracle used only in tests. Cover:

- pinned pawns, knights, bishops, and rooks;
- a pinned piece moving legally along its pin ray;
- protected and unprotected king recaptures;
- pinners removed during an exchange;
- en-passant occupancy changes and promotions.

### 10.4 Fuzz targets and debug invariants

Useful fuzz entry points:

- FEN parsing/round-trip;
- UCI `position ... moves ...` parsing;
- make/unmake sequences;
- malformed TT move decoding/validation;
- SEE versus the slow oracle;
- castling and EP edge positions.

Add a debug-only `Board::assert_ok()` that recomputes every redundant field and
reports a labeled mismatch. Use it after every move in property/sanitizer
tests. It need not run in production search.

## 11. Performance benchmark methodology

### 11.1 Standalone board benchmark

`benches/board.rs` uses 150 ms warmup and one 750 ms sample per workload. It
reports aggregated moves/positions per second across five FENs. On the hybrid
Core Ultra 7 165H audit machine, repeated results varied too much—even cached
check reads varied heavily—for small optimization decisions.

Problems:

- no reliable affinity or core-class selection;
- one short measurement;
- no variance/confidence output;
- `moves/s` conflates calls per second with average generated move count;
- five positions do not stratify check/pin/EP/castling behavior;
- `check detection` measures a cached bitboard comparison, not check-state
  update cost.

`tests/board_performance.rs` separately chooses the best of three runs, which
favors boost/noise and is a smoke test rather than a regression gate.

### 11.2 Recommended harness

- pin to a known physical performance core where supported;
- warm until stable;
- collect 7-15 samples;
- report median, best, MAD/dispersion, calls/s, moves/call, and ns/call;
- print CPU, OS, compiler, flags, git SHA, and power policy;
- retain exact FEN sets in source.

Add focused workloads for:

- no check, single check, double check, and pin-heavy positions;
- capture-only versus quiet-only stages;
- `has_pseudo_capture()` alone;
- pin/blocker computation;
- `gives_check()` and child checker update;
- every special make/unmake type;
- EP validation;
- `see_ge()` thresholds;
- local versus shared TT probes/prefetch;
- HCE/NNUE cold refresh versus incremental update.

Every microbenchmark gain must be confirmed by full fixed-depth search NPS.
Search-behavior changes require SPRT even if NPS improves.

### 11.3 Current search baseline

The audit's current clean-tree, non-PGO `bench 13` measurement produced:

| Metric | Result |
|---|---:|
| Node fingerprint | 17,610,572 |
| Geomean EBF | 2.547 |
| Generic non-PGO best-of-three NPS | 1,306,034 |
| Native/PEXT non-PGO best-of-three NPS | 1,447,166 |

Native/PEXT was about 10.8% faster, but that comparison combines CPU targeting
and PEXT and is not an isolated board result. `PLAN.md` still lists a
13,541,282-node fingerprint for the prior accepted baseline, while current
HEAD is explicitly pending SPRT. A generated build/test manifest should make
the distinction between accepted baseline and experimental head unambiguous.

## 12. Strength-testing infrastructure

### 12.1 Existing strengths

`tools/sprt.ps1` uses fastchess normalized-Elo SPRT, paired colors, a fixed
opening book, one thread, explicit hash, clock-based STC, optional LTC, and
documented concurrency limits. `build_test.ps1` uses PGO for match binaries,
and gauntlet/SPSA/data tools are separated by purpose.

This is materially better than informal fixed-game gauntlets.

### 12.2 Reproducibility gaps

`build_test.ps1` does not automatically:

- run release tests before exposing a candidate;
- reject or record a dirty worktree;
- record git SHA/diff hash;
- record rustc/Cargo/linker, dependency lock, flags, target, or PGO profile;
- record the candidate bench fingerprint;
- verify that tune-off production defaults match tuned/baked values.

`sprt.ps1` writes the PGN but does not persist a complete machine-readable test
manifest or explicit opening-order seed. Result directories are gitignored and
there is no persistent patch/commit association.

Recommended candidate/match manifest:

```text
engine binary SHA-256
git SHA and dirty diff hash
Cargo.lock hash
rustc/Cargo/linker versions
target triple and encoded rustflags
PGO workload/depth and profdata hash
bench node fingerprint, EBF, and NPS samples
opening book path and SHA-256
random seed/order policy
TC, concurrency, hash, threads, adjudication and SPRT bounds/model
fastchess version
host CPU/OS and power/affinity policy
```

Have `build_test.ps1` emit a sidecar JSON next to each binary and have
`sprt.ps1` copy both engine manifests into the result directory.

### 12.3 Distributed testing

Rarog remains dependent on one machine and manual result management. At the
frontier, distributed services such as Fishtest and OpenBench compound many
small gains by providing reproducible builds, persistent results, automatic
stopping, and patch association:

- <https://github.com/official-stockfish/fishtest>
- <https://github.com/AndyGrant/OpenBench>

OpenBench adoption is not required before fixing board correctness. It becomes
increasingly valuable once typical accepted patches are only +1 to +3 Elo.

## 13. Prioritized remediation roadmap

### Phase A: correctness before tuning

1. Fix rule-50/checkmate precedence.
2. Preserve `halfmove_clock` across null moves.
3. Add `plies_from_null` and root-aware repetition semantics.
4. Correct SEE for pinned recapturers.
5. Add independent SEE and draw/repetition regression tests.
6. Add a rule-50-aware TT search key after steps 1-3.

Keep these patches separate. SEE changes have broad pruning/order effects and
need SPRT after their correctness suite passes.

### Phase B: recursive-state and hot-path cleanup

1. Collapse `MovePicker` to one reusable scored buffer.
2. Remove or justify `has_pseudo_capture()` by measurement.
3. Cache/reuse blocker and pinner geometry across capture/quiet stages.
4. Reuse precomputed `gives_check` during move updates.
5. Correct shared-TT byte sizing and measure thread scaling.
6. Bake deterministic magic metadata to reduce startup latency.

For each layout-only patch record identical node fingerprint plus NPS samples.
For behavior changes record SPRT.

### Phase C: NNUE-ready state

1. Introduce per-ply `StateInfo` without embedding the accumulator in `Board`.
2. Define and test dirty-piece deltas for every move type.
3. Add a per-thread accumulator stack and king-bucket refresh cache.
4. Validate incremental state against full refresh through randomized walks.
5. Implement baseline quantized/SIMD NNUE and re-tune search.
6. Add threat inputs only after the baseline and trainer are reproducible.

### Phase D: reproducible build, CI, and testing

1. Commit `Cargo.lock` and pin/record the Rust toolchain.
2. Add PR formatting/clippy/release-test CI.
3. Add sanitizer, property, differential, and fuzz jobs.
4. Publish and smoke-test actual PGO release artifacts.
5. Emit build and match manifests with hashes and seeds.
6. Replace noisy microbenchmark output with repeatable statistics.

### Phase E: later scaling and product features

1. Upcoming-repetition cuckoo lookup.
2. Large-page/NUMA-aware TT and high-thread scaling work.
3. Chess960 castling metadata and FRC regression coverage.
4. AVX-512/VNNI and architecture-specific NNUE kernels when justified by
   measured hardware coverage.
5. OpenBench or equivalent distributed testing.

## 14. Verification performed during this audit

- Ran the complete release test suite: **160 tests passed**.
- Ran `cargo fmt --all -- --check`: passed.
- Confirmed the worktree was clean before creating this document.
- Reproduced rule-50/checkmate precedence in the release engine.
- Reproduced null-move halfmove advancement and immediate search draw.
- Reproduced the pinned-recapturer SEE error directly through the board API.
- Verified protected and unprotected king-recapture SEE examples behave
  correctly.
- Verified Rarog's EP identity path performs legal, not merely pseudo-legal,
  capture validation.
- Measured relevant Rust type layouts and current generic/native search bench.
- Inspected board, move generation, SEE, search/MovePicker, TT, evaluator,
  threading, PGO/architecture build driver, release workflow, benchmarks, and
  local SPRT/gauntlet tooling.
- Compared current Stockfish, PlentyChess, and Reckless board/state/search
  implementations.

The central lesson is the same as in the Basilisk audit: perft and
self-consistency tests are necessary but cannot detect correlated omissions.
Independent legal oracles, randomized state reconstruction, and reproducible
test artifacts are the infrastructure needed for the next strength tier.
