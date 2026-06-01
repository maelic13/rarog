# Rarog — Strength & Speed Improvement Plan

This plan turns the engine analysis into an ordered, implementable sequence of
steps. It targets **playing strength (Elo)** and **search speed (NPS)** without
introducing NNUE, NUMA, or new UCI options.

Rarog already implements a near-complete modern search (iterative deepening,
aspiration, PVS, TT, IIR, RFP, null-move + verification, ProbCut, razoring,
futility/LMP/SEE pruning, LMR, singular/double/negative extensions, full history
+ correction-history suites, lazy SMP, Syzygy). The latent Elo is therefore in
**(a) coarse/untuned search heuristics** and **(b) a generic, untuned classical
evaluation** (PeSTO PSQT + hand-set round-number weights). The steps below attack
both, starting with the highest-ROI self-contained code changes.

Comparison reference: Reckless (Rust, NNUE), grounded in its public source:
- https://github.com/codedeliveryservice/Reckless
- https://raw.githubusercontent.com/codedeliveryservice/Reckless/main/src/search.rs

---

## How to use this plan

- Implement **one step at a time**; do not batch unrelated changes.
- Within a step, land sub-items **incrementally** where noted, each behind its own
  test, so a regression is attributable.
- Tuning campaigns (Steps 8–9) should begin only once the relevant code is frozen.
  Step 8 (search SPSA) may start as soon as Step 4 is merged and run in parallel
  with Steps 5–7.

### Global testing methodology (applies to every step)

1. **SPRT** every change in fastchess/cutechess:
   - Tweaks: bounds `[0, 5]` Elo. Risky changes (pruning/extensions/king safety):
     `[0, 3]`.
   - Short time control (STC, e.g. 8+0.08 or 10+0.1) to gate; confirm
     pruning/extension/eval changes at long TC (LTC, e.g. 40+0.4 or 60+0.6).
   - A change "wins" only on SPRT pass, never on intuition or node count alone.
2. **Bench fingerprint:** `bench` (see `src/bench.rs`, depth 13, 16 FENs) prints a
   total node count. Any search change deterministically alters it — record the new
   number in the commit message (Stockfish-style). Single-threaded bench must stay
   deterministic.
3. **NPS items:** compare `bench` NPS across ≥5 runs; require the SPRT to be
   non-negative. Speed that costs Elo is a bug, not a win.
4. **Tuning items (Steps 8–9):** require a held-out validation set (lower MSE /
   SPSA convergence is necessary but not sufficient) **and** a final SPRT.
5. Keep `cargo test` green (perft, search-strength, TT/move-ordering regressions in
   `tests/` and inline `#[cfg(test)]`).

### Expected-impact disclaimer

In an already-strong engine, single patches are typically +3…+15 Elo; the large
gains are cumulative (especially the tuning campaigns). Ranges below are estimates,
not guarantees — SPRT is the arbiter.

---

## Step 1 — Fractional LMR + do-deeper/do-shallower, and cheap formula upgrades

**Goal:** Replace the coarsest, highest-leverage search heuristics with fine-grained,
history/correction-aware versions. This is the best Elo-per-hour work and is fully
self-contained in `src/search.rs`.

Land the four sub-items **separately**, each SPRT-gated, in this order.

### 1a. Fractional LMR with richer terms + do-deeper/do-shallower  `[from Reckless]`
**Why:** LMR is the single highest-leverage heuristic. Rarog's is an integer table
(`LMR_TABLE`, `src/search.rs:26-34`) with ~6 `±1` tweaks and clamps
(`src/search.rs:1283-1333`). A `/1024` fractional model with weighted terms plus
re-search depth adjustment is a meaningful step up.
**Expected:** +15…+30 Elo (once its terms are SPSA-tuned in Step 8). **Risk:** medium.

**How:**
1. Change the reduction representation to 1024ths. Precompute a base table:
   `base_1024[depth][move] = (1024.0 * (0.75 + ln(depth) * ln(move) / 2.25)) as i32`
   (keep the existing float formula; just scale by 1024 and store `i32`).
2. In the reducible branch of `negamax` (currently `src/search.rs:1283-1307`),
   build a fractional reduction `r = base_1024[d][m]` then add weighted terms
   (each a named constant so Step 8 can tune them), then `r >>= 10` and
   `clamp(1, new_depth.max(1))`. Suggested terms (signs: + = reduce more):
   - `-1024` if `improving`
   - `+1405` if TT bound is `Exact`
   - `+286` if `tt_depth < depth`
   - `+1810` if `cut_node`, and `+2113` more if `tt_move.is_null()`
   - `-463` if `is_pv` / `tt_pv`
   - quiets: `+2171 - 179 * quiet_hist / 1024`
   - noisy: `+1724 - 107 * cap_hist / 1024`
   - `-3403 * correction_magnitude / 1024` where `correction_magnitude =
     (corrected_eval - raw_static_eval).abs()` for this node (compute once, reuse)
   - `-939` if `in_check`
   - `+992` if `cutoff_count[ply]` is high (wire in 1c)
   Start with the existing behavior's terms and add the new ones one at a time if
   you want finer SPRT attribution; otherwise add all and tune in Step 8.
3. **do-deeper / do-shallower:** after the reduced zero-window search returns
   `score`, and **before** the full-`new_depth` re-search
   (`src/search.rs:1320-1333`):
   - if `score > best_score + 54` → re-search at `new_depth + 1`
   - else if `score < best_score + 8` → re-search at `new_depth - 1`
   - else → `new_depth`
   (`best_score` is the running best at this node; the two margins are tunable.)

### 1b. History-/correction-weighted pruning thresholds  `[from Reckless]`
**Why:** RFP, futility, LMP, and SEE thresholds are depth-only today; folding in
move history and `|correction|` lets good-history moves survive and bad ones die
sooner. Cheap and robust.
**Expected:** +8…+20 Elo cumulative. **Risk:** low.

**How (all in `src/search.rs`):**
- **RFP** (`:1006-1009`): subtract a `|correction|`-scaled term and a small
  "threats empty" bonus from the margin, e.g.
  `margin = (70 + 20*not_improving_i)*depth + a*|correction|/1024`.
- **Quiet move-loop futility** (`:1185-1189`): unify the existing `quiet_hist`
  cutoffs into one history-aware test:
  `eval_for_pruning + a*depth + b*quiet_hist/1024 <= alpha` (depth ≤ ~7).
- **LMP count** (`late_move_prune_count`, `:2390-2397`): add a history term so
  high-history quiets raise the count, e.g.
  `base + c*quiet_hist/1024` (clamped ≥ existing base).
- **Quiet SEE pruning:** add a quiet-move SEE prune in the move loop:
  `see_threshold = (-15*depth*depth + 52*depth - 23*quiet_hist/1024).min(0)`,
  prune if `!board.see_ge(mv, see_threshold)` and not a checking move.
  (Captures already use a SEE/cap-history threshold at `:1193-1205` — keep it.)

### 1c. Cutoff-count tracking → reductions/pruning  `[from Reckless]`
**Why:** Nodes where many earlier siblings failed high are "noisy"; reducing later
moves harder there is a known gainer.
**Expected:** +5…+12 Elo. **Risk:** low.

**How:** Add `cutoff_count: [u8; MAX_PLY]` to `Searcher` (init 0 on node entry).
When a child recursion returns a fail-high (the child's `score >= beta`), increment
`cutoff_count[ply]`. Consume it in the LMR term set (1a) and optionally to relax
LMP. Reset the counter when (re)entering a node at that ply.

### 1d. NMP cut-node guard + richer reduction; razoring & ProbCut formulas  `[from Reckless]`
**Why:** Small, well-isolated wins.
**Expected:** +4…+10 Elo total. **Risk:** low.

**How (`src/search.rs`):**
- **NMP** (`:1013-1018`): add `&& cut_node` to the guard; reduction
  `r = 4 + depth/4 + ((eval_for_pruning - beta)/200).clamp(0,3) + improving_i`
  (coefficients tunable in Step 8). Keep the depth ≥ 10 verification search.
- **Razoring** (`:1010-1012`): switch to a quadratic gate, e.g.
  `if depth <= 4 && eval_for_pruning + 200 + 250*depth*depth < alpha { qsearch }`.
- **ProbCut** (`:1068`): make the margin improving-aware:
  `probcut_beta = beta + 200 - 80*improving_i` (replaces fixed `+180`).

---

## Step 2 — Singular extensions: triple, low-depth (LDSE), multicut lerp  `[from Reckless]`

**Why:** Refines the present singular logic (`src/search.rs:1210-1246`).
**Expected:** +5…+12 Elo combined. **Risk:** low.

**How:**
- **Triple extension:** when `singular_score < singular_beta - margin3` (non-PV only),
  set `extension = 3`. Keep the existing double (`< singular_beta - 20`, `+2`) and
  negative (`tt_score >= beta` → `-1`) cases.
- **Multicut lerp:** replace `return singular_beta` (`:1242`) with a value lerped
  toward beta: `return singular_beta + (beta - singular_beta) * 34 / 100`.
- **LDSE (low-depth singular extension):** when
  `depth <= 7 && cut_node && eval_for_pruning <= alpha - 25`, grant `extension = 1`
  without running the full singular verification search.
- Optionally raise the trigger depth to `depth >= 4 + tt_pv as i32` to match Reckless.

---

## Step 3 — Aspiration-window improvements  `[from Reckless]`

**Why:** Cheaper, smarter root re-searches.
**Expected:** +3…+8 Elo. **Risk:** low.

**How (`search_root`, `src/search.rs:617-658`):**
- Score-scaled initial delta: `delta = 12 + best_score*best_score/16000`
  (replaces fixed `25`).
- Asymmetric widening: on fail-low `delta += delta*26/128`; on fail-high
  `delta += delta*60/128` (replaces the symmetric `×1.5` growth).
- On repeated **fail-high**, reduce the re-search `depth` by 1 (down to a floor of
  ~`completed_depth - 4`) to cut re-search cost while keeping the window.

---

## Step 4 — Minor search refinements  `[from Reckless / independent]`

**Why:** Cheap odds-and-ends that individually add a little. **Expected:** +5…+12
Elo combined. **Risk:** low. (After this step the search code is "frozen" enough to
start the Step 8 SPSA campaign in parallel.)

**How (`src/search.rs`):**
- Add **continuation-history offset 3** alongside 1/2/4/6 (`cont_score` `:1825`,
  the update block `:1912-1971`, and the `cont_history_3` field + init/clear/age).
- **Depth-scale the correction bonus** and retune the divisor: `update_correction`
  (`:2157`) bonus `(146*depth*diff/128).clamp(-4449, 2659)`; consider moving the
  read-side divisor (`/128` at `:2116`) toward Reckless's `/69` (tune in Step 8).
- Apply `-1` LMR when the move is a **killer or countermove** (currently only
  history-scaled).

---

## Step 5 — Speed (NPS): lazy eval + remove per-move allocations  `[independent]`

**Why:** Raise NPS with neutral Elo. `evaluate()` always runs the mobility +
king-safety + threat loops even when material+PSQT already proves the node is far
outside the window; and `gives_check` clones the whole board for castling while
being called for **every quiet move** during ordering.
**Expected:** NPS +several %; Elo ≈ neutral (must not regress). **Risk:** medium for
lazy eval (pruning relies on eval accuracy), low for the clone fix.

### 5a. Remove `Board::clone` in `gives_check` for castling + cheaper checkers
- `gives_check` (`src/board/board.rs:612-617`): instead of cloning and calling
  `is_in_check`, compute the rook's castle destination (F1/D1/F8/D8), build
  `occ_after`, and test `atk.rook(rook_to, occ_after) & their_king_bb` (plus the
  king never gives check). No allocation.
- (Optional, higher-risk) Derive `checkers` incrementally in `make_move`
  (`src/board/board.rs:1231`) from the moved piece's direct attacks + discovered
  sliders through `from`, instead of a full `attackers_to`. **Validate with full
  perft and the existing test suite before trusting it.**

### 5b. Lazy evaluation
- Split `evaluate` (`src/eval.rs:322`) into `eval_cheap` (material + PSQT + phase +
  tempo — already computed first) and the rest (`eval_pawns`,
  `eval_piece_activity`, scaling).
- Pass `alpha`/`beta` (or a single window) into the static-eval call sites in
  `negamax`/`quiescence`. If `eval_cheap + LAZY_MARGIN < alpha` or
  `eval_cheap - LAZY_MARGIN > beta`, return `eval_cheap` and skip the heavy loops.
- Use a **conservative** `LAZY_MARGIN` (~600 cp) and SPRT. Note: the existing
  full-eval cache + pawn-hash already absorb recompute on cache hits, so the win is
  mainly on cache-miss nodes — this becomes more valuable after Step 7 makes king
  safety heavier.

---

## Step 6 — Evaluation parameterization refactor (enabling, behavior-neutral)  `[independent]`

**Why:** Prerequisite for Steps 7 and 9. Every non-PSQT weight in `src/eval.rs` is a
hand-set round number; tuning is impossible until they live in one place.
**Expected:** 0 Elo by itself (must be a no-op). **Risk:** low, but verify the bench
node count is **unchanged**.

**How:**
- Introduce a single `EvalParams` struct holding every magic number currently inline
  in `eval.rs`: `mobility_mg/eg` per piece, `passed_mg`/`passed_eg` arrays, the
  king-safety `SAFETY` table + shelter/storm constants, threat values, outpost and
  rook-file/7th bonuses, `tempo`, hanging penalties, OCB/KNN scale factors, space
  weight, passed-pawn king-proximity coefficients.
- Provide `const DEFAULT: EvalParams` equal to today's values; keep the `const`
  lookup tables (`MG_TABLE`, etc.) buildable from it (or from PSQT params).
- Confirm identical bench output before/after.

---

## Step 7 — Evaluation content: king-safety overhaul + missing terms  `[independent]`

**Why:** The king-safety model (`src/eval.rs:633-729`) is the weakest eval
component (a flat 16-entry attacker-units table, no *safe checks*, no quadratic
scaling, no king-ring weakness), and the threat eval is thin (only pawn-threats +
hanging). These add understanding the search cannot recover.
**Expected:** king safety +15…+40 Elo, new terms +5…+20 Elo — **only once tuned in
Step 9.** **Risk:** medium-high (king safety regresses easily). Add params to
`EvalParams` from Step 6 so Step 9 tunes them.

**How:**
### 7a. King-safety overhaul
- Build a per-side `king_danger` accumulator:
  `Σ attacker_count * weight[piece]` over pieces attacking the king ring,
  `+ king_ring_attacks * w`,
  `+ Σ safe_check_bonus[piece]` where a *safe check* is a square that gives check
  and is **not** defended by the enemy king/pawns (and is reachable by that piece),
  `- pawn_shelter/storm` (keep existing logic),
  `+ no_enemy_queen` relief.
- Convert quadratically: `mg -= sign * king_danger*king_danger / 512` (clamp to a
  sane range). Keep it inside `eval_king_safety` and tune every weight in Step 9.

### 7b. Additional threat / positional terms (in `eval_piece_activity`, `src/eval.rs:495`)
- minor-attacks-minor, rook-attacks-queen, safe-pawn-push-threat.
- restricted-mobility penalty (pieces with mobility 0–1).
- bishop outposts on holes (extend the knight-outpost idea at `:543-553`), weighted
  by pawn support.
- connected / phalanx pawn bonus in `eval_pawns` (`src/eval.rs:384`).
- Every new weight is an `EvalParams` field → tuned in Step 9.

---

## Step 8 — SPSA tuning campaign of search constants  `[independent / Reckless priors]`

**Why:** Very high ROI. The RFP/futility/LMP/NMP/razor/ProbCut margins, the LMR base
divisor + all the Step-1a term weights, the history bonus cap
(`history_bonus`, `src/move_ordering.rs:127-129`), the aspiration delta, and the
do-deeper/do-shallower margins are all untuned.
**Expected:** +15…+40 Elo cumulative. **Risk:** low (tuning can't break correctness),
but expensive in games/compute. Can begin once Step 4 is merged; runs in parallel
with Steps 5–7.

**How:**
- Make the constants tunable **without shipping a UCI option** (per the constraint):
  gate a parameter loader behind `#[cfg(feature = "tune")]` that reads values from an
  env var / JSON file; the default build keeps compile-time constants.
- Prioritize: LMR base divisor + the Step-1a term weights; RFP & futility margins;
  NMP base/divisor; LMP multiplier; history bonus cap; aspiration delta; ProbCut &
  razor margins; correction-history divisor.
- Run SPSA (OpenBench or a local SPSA driver), ~40–80k games. Use Reckless's tuned
  values (see its `src/parameters.rs` and the search.rs formulas in the analysis) as
  **starting priors / sanity bounds**, not copies.
- Fold converged values back into the compile-time defaults; final SPRT.

---

## Step 9 — Texel/gradient tuning of the full HCE  `[independent]`

**Why:** Highest single Elo ceiling for a non-NNUE engine. Every non-PSQT weight is a
guess and the PSQT is generic PeSTO. Depends on Steps 6–7 (params must exist first).
**Expected:** +40…+100 Elo. **Risk:** overfitting — mitigate with a held-out
validation set and a mandatory SPRT.

**How:**
1. Build a `tools/tune` binary (or `xtask` subcommand).
2. Dataset: FEN + game result `{1.0, 0.5, 0.0}`. Use a public quiet-labeled set
   (e.g. Zurichess `quiet-labeled`) or self-generate from Rarog games, filtered to
   **quiet** positions (not in check; best move not a capture/promo).
3. Objective: minimize `Σ (result - sigmoid(K * eval))²`.
   - First fit the scaling constant `K` (golden-section search).
   - Then optimize `EvalParams` via mini-batch Adam or coordinate descent, calling
     the **engine's own `evaluate()`** (not a reimplementation) so tuned values match
     runtime behavior exactly. (Temporarily expose a white-relative raw eval for the
     tuner if needed.)
4. Hold out a validation split; stop on validation MSE, not training MSE.
5. Fold tuned values into the `EvalParams` defaults; final SPRT (STC gate + LTC
   confirm). King-safety weights from Step 7 are tuned here too.

---

## Quick reference — order, impact, effort, risk

| Step | Content | Source | Est. Elo | Effort | Risk |
|---|---|---|---|---|---|
| 1 | Fractional LMR + do-deeper; history/corr pruning; cutoff count; NMP/razor/probcut | R | +30…+70 | M | M |
| 2 | Singular: triple / LDSE / multicut lerp | R | +5…+12 | S | L |
| 3 | Aspiration windows | R | +3…+8 | S | L |
| 4 | Minor refinements (cont-hist 3, corr scaling, killer LMR) | R/I | +5…+12 | S | L |
| 5 | NPS: lazy eval + remove clone/checkers cost | I | speed | M | M/L |
| 6 | Eval param refactor (no-op enabler) | I | 0 | M | L |
| 7 | King-safety overhaul + new threat/positional terms | I | +20…+60 | M/H | M/H |
| 8 | SPSA tuning of search constants | I/R | +15…+40 | H (compute) | L |
| 9 | Texel tuning of full HCE | I | +40…+100 | H | M |

`R` = from Reckless comparison, `I` = independent. Elo ranges are pre-SPRT estimates.

**Critical path for fastest signal:** Step 1 first (best Elo/hour, self-contained),
then 2–4, then kick off Step 8 in parallel while doing 5–7, finishing with Step 9.
