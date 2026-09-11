# Rarog board audit and measured comparison — 2026-09-05

**Status: audit complete; roadmap integrated as RAR-M20 on 2026-09-05.**
After the maintainer released the edit hold, PLAN/GUIDE were reconciled against
a0aeb68 and the analysis adopted. Engine repairs and optimizations remain
unimplemented. No Rarog engine source was changed in this documentation step.

**SEE UPDATE, RAR-M29 / 2026-09-07:** the native-value SEE row below remains
historical evidence but is superseded for cross-engine ranking. The normalized
replacement is Rarog/Basilisk/Reckless **44.923/58.335/40.823 M captures/s**
with identical 100/300/300/500/900/20000 values and ten move/verdict answers.
Rarog's 12.20% round span limits precision. See
`see_value_injection_2026-09-07.md`; do not use the old row as a current rank.

The board is a sound bitboard foundation, but it is not yet a best-in-class
implementation: focused probes demonstrate correctness defects, and Basilisk
is materially faster on the measured board workloads. Neither result justifies
a wholesale rewrite. Fix demonstrated defects first, profile actual HCE search,
then compare bounded changes. Preserve useful NNUE preparation while placing
its evaluator-specific implementation in Phase 5/6.

## Evidence and scope

Measured Rarog source: ca03a46db74197bc32c8cf3441359de421fcddd5.
Basilisk: d73476614701863e61871de62f12568b52191d79.
Reckless: 91b56c29861f0a5713204bdeffd6c45e9eb9f649 plus the included adapter.
Stockfish reference: 1dc0912d86dafb99e96d679a6ac76cbdf1553459; not timed.

Rarog advanced through 3be1c05 (endgame ranking), 60bd1f1 (panic reporting)
and a0aeb68 (rated-game occurrence) during this work. At the final source
comparison its board modules, board benchmark and Cargo.toml remained
unchanged from ca03a46. This does not freeze concurrent future work. The live
GUIDE still identified 4.11.7 budget transfer as current at the snapshot.

Supporting files: [manifest](artifacts/board-audit-20260905/manifest.json),
[reproduction instructions](board_benchmark_recipe_2026-09-05.md),
[PLAN](../PLAN.md), [GUIDE](../GUIDE.md).
The adopted plan contains 18 HCE board leaves, nine detailed Phase-5 leaves
and three Phase-6.4 leaves. It preserves the current 4.11.7 next step, every
existing checkbox status and Claude's registered v2 endgame ordering.

## Measured throughput

Millions of reported operations per second. Each cell is the median of three
round medians; each round uses 150 ms warmup and eleven 150 ms samples.

| Workload | Rarog | Basilisk | Reckless | Basilisk faster than Rarog |
|---|---:|---:|---:|---:|
| Legal moves | 447.131 | 642.646 | 339.844 | 43.7% |
| Legal captures | 98.204 | 120.138 | 61.597 | 22.3% |
| Generation + make/unmake | 42.521 | 55.031 | 23.494 | 29.4% |
| Start-position perft(4) | 273.741 | 382.726 | 177.944 | 39.8% |
| Two-ply simulation | 351.809 | 513.537 | 246.626 | 46.0% |
| Native-value threshold SEE — NOT comparable | 46.676 | 58.814 | 39.722 | not ranked |

“Basilisk faster” is B/R minus one. The corresponding Rarog deficits relative
to Basilisk are 30.4%, 18.3%, 22.7%, 28.5%, 31.5%; these are different denominators.

Host: AMD Ryzen 9 5950X, Windows, logical processor 2 pinned via affinity mask
4. Three cyclic engine orders: R/B/Q, B/Q/R, Q/R/B. Builds were completed before
timing, native optimized, no PGO. Rust: 1.97.1, LLVM 22.1.6, MSVC target, fat
LTO. Basilisk: Clang 22.1.8, MinGW target, thin LTO, PEXT. Rarog used
rarog_pext and no Cargo features; Reckless used native magic/AVX2 and
NullBoardObserver. Different languages/ABIs and necessary board bookkeeping
are part of this native-engine comparison, not controlled algorithm-only effects.

The predeclared total-host-busy ceiling was 12%; observed load including the
benchmark was 6.25–9.17%. This was an active desktop, not a precision idle-host
measurement. Round-median range as a percent of the median:

| Workload | Rarog | Basilisk | Reckless |
|---|---:|---:|---:|
| Legal moves | 0.71% | 0.76% | 7.75% |
| Legal captures | 3.02% | 0.30% | 13.08% |
| Make/unmake | 2.18% | 4.87% | 23.71% |
| Native SEE | 1.99% | 0.59% | 14.89% |
| Perft | 3.38% | 3.74% | 3.94% |
| Simulation | 7.97% | 2.78% | 7.58% |

The ranges are not confidence intervals. Every Basilisk round beat every
Rarog round, and every Rarog round beat every Reckless round, for all five
comparable workloads. That supports the ordering and broad scale on this
machine. It does not support a small optimization claim, whole-search NPS
projection, Elo claim or universal engine ranking. No games were played.

Reckless's adapter runs the same roots/work quanta using native legal moves,
native scored move lists and native state maintenance. It disconnects NNUE
arithmetic through the engine's existing no-op observer. Threats, pins,
checking squares, hashes and repetition bookkeeping remain. Its slower
numbers therefore do not show that its board design is inferior for its
actual search/network. Use its source for ownership and algorithm references;
do not copy the whole layout expecting a demonstrated speedup.

## Confirmed defects and policy questions

### A. SEE incorrectly stops when the next attacker set contains a king

Owner: proposed 4.11b.4–5. Source:
[src/board/board.rs](D:/code/rarog/src/board/board.rs), full see around 1245–1315
and threshold implementation around 1346–1420.

FEN: `7k/8/2p5/3pK3/8/8/3R4/8 w - - 0 1`, move `d2d5`.

The legal same-square exchange Rxd5 cxd5 Kxd5 is
100 - 500 + 100 = **-300**. Both debug and optimized probes give
**see = -400 and see_ge(0) = true**. This is not explained by unequal peer
piece values: it fails against Rarog's own fixed values.

The loops test whether the opposing king appears among next attackers after
removing the selected attacker. They can break even when that selected
attacker was a pawn. King legality must depend on the selected king capture
and correct exchange parity. Reckless's threshold loop tests the selected
piece type explicitly; Stockfish has a dedicated king branch.
Require independent fixtures at -301/-300/-299/0, and inspect all actual
search/pruning callers after repair. Preserve a correctness-only baseline
before speed refactors or value changes.

### B. A malformed non-ASCII UCI move panics

Owner: 4.11b.3. [moves.rs:196](D:/code/rarog/src/board/moves.rs:196).

`Move::from_uci("aé1")` has byte length four but slicing at byte two splits a
UTF-8 character. Both profiles panic. Validate the ASCII grammar before byte
or string indexing and preserve the intended controlled UCI error behavior.
The recently added crash reporter reports the panic, not a fix for its cause.

### C. Accepted fullmove state can overflow on an ordinary move

Owner: 4.11b.3. [board.rs:1667](D:/code/rarog/src/board/board.rs:1667)
and null increment around 1711.

FEN `7k/8/8/8/8/8/8/KR6 b - - 0 65535`, move `h8g8`:
debug panics; optimized wraps fullmove to zero. Define and test parsing,
real/null move, undo and maximum accepted state consistently. Widening or
explicitly bounded behavior is an implementation decision; accidental
debug/release disagreement is not acceptable.

### D. SEE special-move shortcuts need an explicit caller contract

Owner: 4.11b.4. Ordinary see_ge returns immediate gain for non-captures.
see_ge_quiet_aware extends ordinary quiets, but still bypasses quiet promotions.

FEN `7r/P7/7k/8/8/8/8/K7 w - - 0 1`, `a7a8q`: current full SEE is +800,
threshold zero passes; legal ...Rxa8 makes the same-square exchange -100.
That is an explicit shortcut, so first establish whether callers intend
“admit this move” or “estimate the exchange.” Do not silently impose exact
tactical semantics on an intentionally selective heuristic. Inventory
captures, ordinary quiets, both promotions, EP, castling, pins and king
recaptures separately. A SEE-vs-SEE test cannot establish correctness.

The archived Rust probe's recursive oracle uses Rarog legal generation, with
separate exchange arithmetic. The independent python-chess legal same-square
oracle in the archived verify_bundle.py source also confirms both -300 and -100. This is not a
claim that a normal SEE must solve all tactical alternatives or checks.

### E. Draw/null/repetition behavior is a policy boundary, not a speed fix

Owner: 4.11b.15. Read
[draw_semantics.rs](D:/code/rarog/tests/draw_semantics.rs) and its history.

Current search repetition, null-clock advancement and cross-null history
have deliberate semantics. Earlier combined attempts to change these policies
lost approximately 7.21 and 11.91 Elo in their historical tests. That is not
independent causal evidence about every component, and does not justify
either automatically retrying or universally banning a narrowly motivated fix.
Record keep/change/retry-trigger for each contract. Preserve the existing
mate-at-clock-100 behavior. Distinguish repetition identity from a potential
TT/evaluation rule-50 bucket; never corrupt repetition hashes to add TT policy.

## Instrument findings

Owner: 4.11b.2 and 4.11b.6.

1. The five v1 roots contain **zero checked sides to move**, zero legal EP and
   zero promotions. The last root's name “in-check” is misleading. Kiwipete
   has two castles and eight of ten total captures. Independent move counts
   are 20/48/31/3/26, captures 0/8/0/0/2. A future profile must add actual
   single/double checks, evasions, EP legality, promotions and sparse endings.
2. The common work vector is 128/10/128/10/197281/4597. Its preflight checks
   work totals, which cannot alone detect wrong identities or state. The
   Reckless adapter additionally checks legal membership, sorted identities,
   capture partition and hash/FEN roundtrips outside timing; all 15 printed
   root move sets were independently checked against python-chess.
3. The make/unmake row includes generation; SEE includes capture generation.
   Keep these useful combined workloads, but add separately named isolated
   primitives. Do not claim they directly measure the named primitive alone.
4. Native SEE vectors P/N/B/R/Q/K differ:
   Rarog 100/320/330/500/900/32000;
   Basilisk 100/300/300/500/900/20000;
   Reckless 109/403/435/679/1242/0. Native verdicts, termination points and
   special-move policies can differ. Coordinate injected contract values and
   semantic checks before treating the SEE timing as comparable.
5. Pull behavior-neutral value injection and benchmark restoration forward
   to 4.11b.6, preserving playing defaults. Keep actual post-HCE value fitting
   in 4.15.3–4.15.4 and revalidation in 4.15.5. Unequal HCE/SEE values alone
   are not proof of an Elo defect.
6. [tests/board_performance.rs](D:/code/rarog/tests/board_performance.rs) is
   the legacy corpus/estimator, not the new benches/board.rs v1 instrument.
   Older July results do not supersede this differently built/current comparison.
7. Basilisk's fallback output barriers outside GNU/Clang are no-ops. This
   measured Clang build uses its active barrier branch. Future compiler ports
   must prove work/output barriers and avoid a silently optimized-away harness.
8. Preserve exact feature/ISA manifests, source and binary hashes, direct
   statuses, distributions and live preflight. All-features enables texel
   and is never a performance build. The archived patch originally had an
   LF-normalized SHA while its file uses CRLF; byte and normalized hashes
   are now explicitly distinguished in the manifest. Content matches.

## Architecture review and optimization candidates

| Area | Finding / recommendation | Proposed owner |
|---|---|---|
| Existing representation | Keep the coherent 12 colored bitboards, two color occupancies plus combined occupancy, mailbox, full/pawn/minor/two non-pawn keys. Board measured 264 bytes plus dynamic history; Move is two bytes. Redundant fields with real consumers are not automatically waste. | 4.11b.13–14 |
| Actual search cost | Measure hot caller frequency and time before promising NPS. Search already uses hinted make and staged generation; standalone perft does not model all savings/costs. | 4.11b.7 |
| Legal generation / delivery | Largest clear throughput gap. Inspect color/mode specialization, setwise pawn/pin work, king safety and list construction/returns. Check emitted code before claiming an actual large struct copy. Preserve move sets and classify ordering changes. | 4.11b.8 |
| Ordinary relocation | Current remove+add may duplicate mailbox, occupancy and key work. Compare fused move_piece; LLVM may already combine some operations. Keep only a real measured improvement. | 4.11b.9 |
| Shared geometry | Check hints and capture/quiet pin sharing already exist. Measure residual repeated compute_pinned/check_info/see_pins before adding caches. Node-local lazy versus per-ply ownership is a choice to test. | 4.11b.10 |
| SEE kernel | After correctness, maintain attacker sets and expose slider x-rays incrementally instead of recomputing attackers twice per recapture. Test real thresholds, pins and selected-king legality. | 4.11b.11 |
| King squares | Rarog and Reckless use bitboard extraction; Basilisk caches. Prototype only if lookup remains material after other changes. Include maintenance/layout cost. | 4.11b.12 |
| History | Initial capacity 128 and cloning capacity do not guarantee root-history plus search headroom. Reserve before search/worker cloning and test no hot growth; retain arbitrary game histories. | 4.11b.13 |
| Mutation API | Public derived fields widen invariant obligations. is_legal returning bool does not canonicalize a caller's move flags; legal_move returns the canonical move. Existing TT uses legal_move correctly. Narrow APIs where useful, without hot validation overhead. | 4.11b.13 |
| Larger rewrite | Six type boards plus two colors, per-ply snapshots instead of inverse work, or selective legality are hypotheses. Require a remaining profile hotspot and bounded A/B after simpler improvements. | 4.11b.14 |
| Qualification | Independent debug/release/state/backend checks, then native A/B and pooled-PGO search. Preserve correctness-only baseline; register a coherent playing cluster rather than a gate for every internal leaf. Refresh dependent endgame evidence. | 4.11b.16–18 |

Reference implementation entry points, verified against local source revisions:

- Reckless [board.rs](D:/code/Reckless/src/board.rs): InternalState, update_threats,
  BoardObserver/NullBoardObserver; [makemove.rs](D:/code/Reckless/src/board/makemove.rs):
  generic observer and snapshot restoration; [movegen.rs](D:/code/Reckless/src/board/movegen.rs)
  and [movelist.rs](D:/code/Reckless/src/types/movelist.rs): native legal targets,
  scored entries, setwise delivery; [see.rs](D:/code/Reckless/src/board/see.rs):
  selected king legality and incremental attackers.
- Stockfish [position.h](D:/code/Stockfish/src/position.h) and
  [position.cpp](D:/code/Stockfish/src/position.cpp): StateInfo ownership,
  set_check_info, do_move and see_ge. Its representation/search contracts are
  references, not automatically Rarog requirements.
- Basilisk [board.cpp](D:/code/basilisk/src/board.cpp):
  gen_legal, move_piece, stored king-square state and dedicated SEE_VALUES.
  Its measured speed is a useful local target, not an independent correctness oracle.
- Upstream identity links: [Reckless](https://github.com/codedeliveryservice/Reckless)
  and [Stockfish](https://github.com/official-stockfish/Stockfish). This audit
  uses the pinned local revisions above, not an unverified claim about latest HEAD.

## NNUE work retained in its own phases

**5.2:** factual move deltas (including off-target EP victim, promotion identity,
both castle pieces and null), explicit observer timing, static/no-op HCE path,
independent transition reconstruction and clone/reset ownership.

**5.3:** evaluator-owned per-worker/per-ply arrays, dirty/valid state for both
perspectives, king bucket/mirror refresh, branch/abort/reset/null contracts.
Do not put full accumulators inside Board/undo. Deterministic scaffold tests
do not replace real integer NNUE parity.

**5.6:** reserve relation/threat update hooks only if the selected network needs
them. Reckless's observer callbacks support threat-feature work dependent on
intermediate occupancy. A simple piece-square network does not justify eagerly
maintained full threat tensors.

**6.3/6.4:** actual scalar inference and integer-exact incremental/SIMD parity,
supported-platform checks, overflow bounds and pooled-PGO performance.
[Reckless nnue.rs](D:/code/Reckless/src/nnue.rs), Network::push/pop and its
BoardObserver implementation, supplies a concrete Rust ownership reference.
Network dimensions/constants must come from Rarog's selected architecture.

## Validation actually performed

Earlier in this audit: 63 selected debug tests and 64 release tests passed
(board correctness, differential, draw semantics, SEE and fuzz selections);
1,975 independently generated python-chess positions matched legal moves,
captures and canonical FEN, with 48,462 hinted make/unmakes and 5,283 captures;
107,648 magic-slider occupancy/ray cases passed. Formatting and
all-feature/all-target Clippy were clean in that audit snapshot.

In the comparison: all nine native benchmark runs passed their preflights;
frozen work counts and all printed rates were mechanically parsed, aggregated
and rechecked against raw output. Archived binary hashes match. Independent
python-chess checks validated all 15 Reckless root move sets and both focused
exchange arithmetic examples. Debug/optimized focused executables reproduced
the SEE, Unicode and counter findings again when completing this bundle.

Limits: the broad test counts belong to the original audit snapshot, not a
new whole-repo pass after Claude's commits. No fresh PEXT exhaustive ray audit,
full feature/target matrix, PGO search timing or game-strength qualification
was performed here. The timed Rarog binary uses PEXT, but perft coverage is
not exhaustive PEXT validation. Stockfish was reviewed, not benchmarked.
Do not represent these targeted successes as proof of complete board correctness.

## NNUE enablement: retain the baseline and identify the actual costs

Owners: **5.2.1**, **5.2.5**, **5.3.4**, optional **5.6**, then **6.4.3**.

The measured Rarog/Reckless gap is NOT measured NNUE overhead. Reckless's
NullBoardObserver disables accumulator arithmetic. Its native board still
maintains threats, pins/checking squares, repetition state and scored moves;
those fields also have search consumers. Different layouts, algorithms,
compiler targets and workloads are additional explanations. Only controlled
measurements can distinguish them. Do not debit the full cross-engine gap
to future Rarog NNUE, or remove useful state merely to recover a historical rate.

5.2.1 retains this measurement permanently, then freezes the final accepted
Phase-4 Rarog board as the current enablement baseline. Keep v1 unchanged
and also run the extended 4.11b corpus. The original five roots have no check,
EP or promotion cases, so they cannot price all costly NNUE transitions.

| Stage | What is timed | What the comparison establishes | Owner |
|---|---|---|---|
| A | Accepted final Phase-4 board with HCE | Revision-matched enablement baseline; historical RAR-M20 remains separate | 5.2.1 |
| B | A plus factual move events, no-op observer, same HCE | Event/interface cost; exact HCE behavior required | 5.2.5 |
| C | B plus per-ply storage, dirty/validity/refresh bookkeeping, same HCE | Scaffolding and memory cost, without NNUE inference | 5.3.4 |
| D | Actual network, scalar full refresh on frozen legal traces | Reference arithmetic and complete evaluator work; HCE-vs-NNUE tree NPS is not causal board cost | 6.3 / 6.4.3 |
| E | Same network/traces, lazy incremental then SIMD | Update/refresh/inference savings with integer-exact parity | 6.4.1–6.4.3 |
| Optional relation state | Semantically valid eager vs lazy maintenance for the selected relation features | Additional relation-feature maintenance; changing network capacity is a separate experiment | 5.6 / 6.4.3 / 6.5 |

For every stage retain source/net/trace hashes, compiler/ISA/features, board
layout and bytes per ply/thread, capacities/allocations, update and refresh
counts, per-cohort medians/spread, and uninstrumented whole-search results.
Prove observational counters live; do not measure their overhead as production.
Use identical transition traces for causal board-cost comparisons. Whole HCE
versus NNUE search also changes positions visited, pruning and eval behavior.

Cover quiets, captures, EP, promotions, castling, nulls, king bucket/mirror
changes and sparse endgames. The reference adapter can later add a real
observer and recorded network, while retaining the frozen no-op variant.
Do not normalize native SEE rankings by silently changing playing values.
Do not mix new machines/toolchains/corpora into the old denominator.

## Handoff

PLAN/GUIDE and RAR-M20 now own the findings. The external drafts and
a0aeb68 snapshots remain historical comparison inputs. The current queue is
4.11.7–4.11.10, then 4.11b.2 onward; 4.11b.1 records this completed audit.
NNUE-related runtime work remains in Phase 5/6. The original broad test counts
are audit-snapshot evidence, not a new engine-test run for this docs change.

Read-only bundle validation (no timing or engine rebuild):

```powershell
python -B D:\chess\results\board-audit-20260905\verify_bundle.py
```


## Roadmap reconciliation at integration

Mechanical baseline: 151 existing GUIDE items, 82 open leaves, next 4.11.7.
Preserved every existing status and the twenty v2 function rows. Added 4.11b
between remeasurement and endgame development; normalized SEE injection moves
earlier while fitting remains after final HCE.

Corrected stale present-tense prose: 4.8's game gate is complete; 4.9's entry
audit is complete and found no structural cluster; 4.10's repair is complete;
4.12 uses v2 rather than the pre-correction order. Clarified that Rarog's
current defect kind need not match the old Stockfish function category and
that a drawn-cohort scaler still requires independent draw labels. The
per-function range is 4.12.2–4.12.21, not 4.12.2–4.12.14. Historical evidence
and rankings are retained. The post-label residual retry trigger has an
explicit closure owner at 4.14.7. GUIDE's overview was shortened rather than
absorbing these derivations.

Integration verification: GUIDE/PLAN checker passes with **182 items** and
**108 open leaves**, next **4.11.7**. All 151 previous statuses and all twenty
v2 function rows/data were mechanically preserved; only the audit completion
is newly checked. The 156 existing Python tooling tests pass. Local links,
fences, raw-round aggregations, byte-exact decoded adapter and unchanged board
source hashes were verified; staged whitespace checks pass. No fresh engine
build, engine-test matrix, timings or games were needed for this docs change.
