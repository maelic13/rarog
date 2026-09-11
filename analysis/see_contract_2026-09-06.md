# SEE contracts — RAR-M27 / 4.11b.4

Source baseline: `6d1a670` (engine `a170f8c`). No production code changed
in this contract step. **Historical RAR-M27 snapshot:** “current” observations
and ignored tests below describe that baseline, not the development head.
The unresolved-defect status is **SUPERSEDED by RAR-M28 / 4.11b.5** (`fce0b44`):
all three debt tests are now active and pass. See
[the repair record](see_repair_2026-09-06.md) for the 41-fixture extension and
new development fingerprint. Playing qualification remains at 4.11b.17.

## Contract

Input is a legal, canonical, non-null move and an engine-range cp threshold.
Values are P/N/B/R/Q/K = 100/320/330/500/900/20000. The earlier 32000 entry
confused eval's `MATE_SCORE` with board SEE and is corrected by RAR-M29. Kings cannot be captured;
their numerical value is only an internal sentinel. Arbitrary i32 thresholds
are outside this contract. Do not substitute HCE fitted values in this leaf.

`see` is a material exchange estimate using a least-valued-attacker sequence.
`see_ge` answers its threshold counterpart for captures. Neither promises full
tactical minimax. On the curated forced/unambiguous exchange fixtures, demand
agreement with an independent legal capture tree, including equality at the
threshold. Branch-choice disagreements on more complex positions require
classification, not automatic changes to a heuristic.

| Initial move | Full SEE / ordinary threshold policy | Quiet-aware policy | Disposition |
|---|---|---|---|
| Ordinary capture | Exchange estimate / estimate >= threshold | Same | King legality, newly created pins and recapture promotions require repairs below |
| Capture promotion, including underpromotion | Victim + initial promotion gain, then exchange with promoted piece | Same | Do not exempt these from correctness because another engine exempts promotions |
| EP | +100, remove pawn from its actual square, then exchange on landing square | Same | External fixture opens rook file through removed pawn |
| Ordinary quiet, including double push | 0 / 0 >= threshold; immediate-gain policy | Exchange from zero initial gain | Quiet-aware pruning is disabled by default; ordinary SEE is not a hanging-piece classifier |
| Quiet promotion, including underpromotion | Promotion gain / gain >= threshold | Same shortcut | Keep explicit policy. No current production SEE call for this class; ordering uses separate promotion tier |
| Castling | 0 / 0 >= threshold | Current implementation enters exchange but legal king destination has no legal recapture, hence same output in fixtures | No current SEE caller. Castling is excluded by is_quiet_move (flags <= DOUBLE_PUSH) |

Quiet promotion a7a8q can be +800/true under the policy while its exchange
value is -100 after ...Rxa8. That is not evidence of a lost conversion or a
reason to change playing policy. A future caller wanting exchange truth must
request it explicitly; do not silently change this interface to make a timing
column comparable. Keep normalized value injection at 4.11b.6 and tuning at
4.15.3–4.15.4.

## Exhaustive production call-site inventory

Line references are for the baseline above; use enclosing functions after edits.
There are **10 threshold calls (two diag-only), one full call**, all in
`src/search.rs`; `src/move_ordering.rs` stores the resulting signed i16 metadata.

| Site | Input population / threshold | Consumer and coupling |
|---|---|---|
| negamax, 2808 | Legal captures, including EP/capture promotion; max(0, (probcut_beta-static_eval)*gap_scale/100) | ProbCut admission; quiet promotions absent from capture generation |
| negamax, 3151, diag-only | Losing captures; max(-coeff*depth-cap_hist/8,-max), depth <= 8 | Shadow pruning counters, not production decisions |
| negamax, 3223 | Ordinary quiets only; max(-quiet_coeff*sel_depth^2,-max) | Quiet-aware prune; QuietSeePruneDepth=0 disables it; checking moves exempt |
| negamax, 3241 | Losing captures; max(-coeff*sel_depth-cap_hist/8,-max), sel_depth <= 8 | Capture pruning, checking moves exempt |
| negamax, 3752 | Capture with SEE_UNKNOWN; threshold 0 | Resolve metadata for good/bad capture history bookkeeping |
| quiescence, 4061 | Non-promotion captures outside check; clamped alpha-stand_pat-margin | Adaptive SEE pruning |
| quiescence, 4065 | Negative picked.see outside check; bad-floor threshold | Also reaches capture promotions; preceding promotion exemption does not cover this floor |
| append_scored_moves, 4200 | Captures except TT move; full SEE | Full/evasion picker: magnitude affects score (32*SEE or SEE), sign propagates to pruning/reductions |
| score_tactical_move, 4272 | TT capture; threshold 0 | TT priority retained but stores negative sign when losing |
| score_tactical_move, 4286 | Other captures; threshold 0 | Tactical order and good/bad partition; promotions remain in early staged partition even with negative SEE |
| quiet_history_score, 4376, diag-only | Checking ordinary quiets; threshold 0 | Safe/losing census is trivially safe by policy; cannot measure hanging checks; production check bonus is unconditional |

Sign flows through MoveEvidence, prospective LMR depth (+1 reduction for a
losing nonquiet), capture pruning, reduction eligibility and capture-history
classification. `SEE_UNKNOWN` is an intentional sentinel for unscored TT
captures, not a numerical exchange loss. Do not widen this audit into changing
TT policy. A repair can affect both tree shape and history learning, so final
cluster qualification remains at 4.11b.17. Future search audit 4.15 must not use
the diag quiet-check census as evidence of exchange safety.

## Independent oracle and measured defects

`tools/diag/see_contract_oracle.py` uses python-chess 1.11.2, every legal
same-square recapture, all promotion choices, and material accounting. Both
sides may stop even when in check: this deliberately ignores tactical evasion
obligations, mate, off-square threats and draws. Initial move legality is
mandatory. No Rarog move generation or SEE code supplies truth. Fixed explicit
arithmetic is checked while regenerating the committed TSV, not learned from
the implementation under test. Five Python tests verify color mirrors,
restoration, actual creation/release of pins, four promotion replies, rejection
of an illegal initial move and rejection of deliberately wrong arithmetic.

The older `tests/see_pins.rs` is independent of the SEE swap loop but uses
Rarog legal generation and only one LVA reply. Retain it as a regression suite;
it does not replace this external all-reply oracle.

| Fixture | Independent arithmetic | Current full / see_ge(0) | Owner |
|---|---|---|---|
| king-after-pawn: Rxd5 cxd5 Kxd5 | 100-500+100 = -300 | -400 / true | 4.11b.5: selected-king legality and parity |
| pin-created: Bc6xd5 vacates c-file | +100; Nc7xd5 becomes illegal, exposing Kc8 to Rc1 | -230 / false | 4.11b.5: use evolving pin geometry |
| promotion-recapture: Rb2xb1 axb1=Q | 500-500-(900-100) = -800 | 0 / true | 4.11b.5: account for recapture promotion and subsequent promoted occupant |

Exact FENs/moves are in `tests/data/see-contract-v1.tsv`. The pin-release
fixture Bc6xd7 frees Rb7 and correctly returns 100-330=-230. Other fixtures
cover defended/undefended king destinations, x-rays, EP, both castles and
initial capture/quiet underpromotions. Three named ignored Rust tests express
the desired repairs; run them explicitly and remove their ignores at 4.11b.5.
Do not rewrite truth to match current outputs. Their expected failures were
confirmed individually by the test runner in both profiles (all three fail).

## Validation and reproduction

Raw observations and expected failures are retained locally (Git-ignored) under
`analysis/artifacts/see-contract-20260906/`. Debug and release observations
agree; each normal run passes 2 new tests plus 6 existing pin tests, with the
3 debts explicitly ignored. These are scoped tests for new fixtures; production
source/manifest/lockfile are unchanged. fmt and all-feature/all-target Clippy
pass. No fresh bench or timing comparison was needed or claimed.

```powershell
python tools/diag/see_contract_oracle.py
python -m unittest discover -s tools/diag -p test_see_contract_oracle.py
cargo test --test see_contract --test see_pins -- --nocapture
cargo test --release --test see_contract --test see_pins -- --nocapture
# Expected exit 101 with three failures BEFORE 4.11b.5:
cargo test --test see_contract -- --ignored
cargo test --release --test see_contract -- --ignored
```

Reference inspection: Reckless `91b56c2`, `src/board/see.rs`, checks the
selected attacker is King and masks attackers by current occupancy. Its header
claims promotions always pass, but its executable code computes initial
promotion gain and can reject them; only castling is unconditionally admitted.
Stockfish `1dc0912d`, `src/position.cpp::see_ge`, returns `0 >= threshold` for
non-NORMAL moves. Neither reference's special-move policy is Rarog's contract.
