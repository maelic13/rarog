# Rarog HCE Improvement Plan

> Implementation guide for taking Rarog from the current `v2.1.0-codex-work`
> state to a measurably stronger hand-crafted-eval (HCE) engine, by porting
> already-written search/eval features under a proper SPRT + SPSA testing
> discipline. **No NNUE for now** — but keep the door open (see §11).
>
> This document is meant to be handed to an implementation model (see
> "Recommended model" at the bottom). Work **one phase at a time, one feature at
> a time**, and never merge a change that does not pass its SPRT gate.

---

## 0. Background — why this plan exists

Three independent attempts to improve `2.0.2` were tested in long Little Blitzer
round-robins (RR, 64 MB, 100 ms/move):

| Engine (branch)              | NPS      | Depth | Elo (run1 / run2) vs 2.0.2 |
|------------------------------|----------|-------|-----------------------------|
| 2.0.2 (baseline, `master`)   | ~2.61 M  | 13.4  | 0 / 0                       |
| 2.1.0 Codex Work (`v2.1.0-codex-work`) | ~2.61 M  | 13.4  | +0.1 / +0.6 (noise) |
| 2.1.0 Codex (`v2.1.0-codex`) | ~1.79 M  | 14.8  | −9 / −76                    |
| 2.1.0 Claude (`v2.1.0-claude`) | ~2.53 M | 13.5 | −33 / −70                   |

**Key conclusions:**

1. **`v2.1.0-codex-work` is behavior-identical to `2.0.2`** (same `bench 13`
   fingerprint after LMR-1024ths port: `5,318,762`). Its changes are safe micro-optimizations with **no
   measurable strength or NPS benefit**. Keep them as clean baseline; expect
   nothing from them.
2. **Speed is not the bottleneck.** Rarog runs at ~2.6 M NPS but only reaches
   depth ~13. Stockfish reaches depth ~24 at ~1.1 M NPS. The gap is **search
   efficiency (pruning/ordering/extensions) and eval quality**, not raw speed.
3. **The Codex and Claude branches added the *right* features but regressed**,
   because new search heuristics and eval terms are defined by their tuning
   constants. Shipping them with hand-guessed values loses Elo until tuned.
4. **None of the three attempts were SPRT-tested.** They were judged on noisy
   9,000-game round-robins. The missing piece is a fast, statistically valid
   test/tune loop — that is the true prerequisite, upstream of every feature.

**Therefore: build the harness first, tune what already ships, then port the
already-written features incrementally — each one SPRT-gated and SPSA/Texel-tuned
on entry.**

---

## 1. Inventory — what exists, and where

All commits below are reachable from the current repo. Branch heads:

| Branch              | Head      | Head subject     | What it contains |
|---------------------|-----------|------------------|------------------|
| `master`            | `5a8ce52` | Version 2.02     | Baseline (== 2.0.2) |
| `v2.1.0-codex-work` | `be4cdc0` | Phase 0 harness  | **Current.** Micro-opts only, == 2.0.2 behavior + Phase 0 tools |
| `v2.1.0-codex`      | `3de254f` | Step 15          | **Search-efficiency rewrite** (see below) |
| `v2.1.0-claude`     | `870fac0` | Release prep     | **Eval expansion + `tune.rs` harness** (see below) |
| `improvements`      | `8c453c1` | Version 2.0.1    | Small move-ordering refinements |

> Baseline reference for all diffs: `5a8ce52` (2.0.2). Diff a branch with
> `git diff 5a8ce52 <branch> -- <path>`. The codex/claude branches are built in
> incremental "Step N" commits (`git log --oneline 5a8ce52..<branch>`), which
> makes cherry-picking individual features straightforward.

### 1a. `v2.1.0-codex` — search efficiency (this is the "deeper search" work)

Diff: `src/search.rs` (+1452), `src/board/board.rs`, `src/board/attacks.rs`,
`src/board/zobrist.rs`, `src/move_ordering.rs`, `src/search_threads.rs`,
`src/tt.rs` (+194), `src/time_manager.rs` (+93, new), `src/params.rs` (+61, new),
`src/eval.rs` (+42).

Features added (each roughly isolated in a Step commit — inspect with
`git log --oneline 5a8ce52..v2.1.0-codex`):

- **ProbCut** — `probcut_allowed`, `probcut_margin`, `probcut_see_threshold`,
  `probcut_verification_depth`, `probcut_cutoff_score`, `probcut_tt_depth`.
- **Multi-cut** via singular search — `SingularDecision::MultiCut`.
- ~~**Extended correction history**~~ — **already in baseline.** Verified
  2026-06: `master` (5a8ce52) already contains the minor-piece, non-pawn, and
  continuation correction histories. Porting this from codex would be a no-op;
  it has been removed from Phase 2.
- **Threat-aware history** — `threatened_index`, `quiet_threat_score`.
- **TT-cutoff / fail-low-parent history** — `update_tt_cutoff_history`,
  `update_fail_low_parent_history`.
- **Time management** — modifications to `src/time_manager.rs` (the file itself
  already exists in baseline).
- **TT overhaul** — `src/tt.rs`.
- **`src/params.rs`** — search constants already extracted into `const fn`s
  (ProbCut margins, singular margins, history divisors, LMP base). **Not yet
  wired to UCI options.** This is a head start for Phase 1/2 tuning.

### 1b. `v2.1.0-claude` — eval expansion + tuning harness (this is the HCE + tuning work)

Diff: `src/eval.rs` (+606), `src/search.rs` (+268), `src/board/board.rs`,
`src/tune.rs` (+408, new), `src/lib.rs`.

- **`EvalParams` struct** in `src/eval.rs` — every eval weight hoisted into one
  struct with `DEFAULT` values that reproduce the baseline material/PST exactly.
- **New eval terms** (each with its own weights in `EvalParams`): tempo, passed
  pawns (defended / free-path / safe-path by rank), candidate passers, mobility
  (per piece), bishop pair, rook open/semi/seventh, knight outposts, pawn
  threats (minor/rook/queen), king safety (attacker weights, ring-square
  attacks, safe checks, danger² conversion, queen-relief), pawn shelter, pawn
  storm, rooks behind passed pawns. Lazy-eval margin for skipping expensive
  terms.
- **`src/tune.rs`** — tuning harness gated behind `--features tune`:
  - Eval: loads weights from a text file at `RAROG_TUNE_FILE` (one param per
    line, arrays space-separated) → ready for **Texel tuning**.
  - Search: documents the exact SPSA target list and LMR weighted-term defaults
    (e.g. `lmr_tt_pv=-463`, `lmr_exact_bound=+1405`, `lmr_cut_node=+1810`) and
    the recipe to extract them into a `SearchParams` struct loaded from
    `RAROG_SEARCH_TUNE_FILE`.

### 1c. `improvements` — small move-ordering refinements

Diff: `src/move_ordering.rs`, `src/search.rs`, `src/tt.rs`, `src/main.rs`.

- Check-awareness in ordering (`CHECK_UNKNOWN/TRUE/FALSE`, `move_gives_check`).
- SEE-based capture pruning (`see_ge(mv, -80*depth)`).
- Check-aware `quiet_history_score`.

Smallest and most self-contained of the three. **Status: already tried as the
first Phase 2 port and discarded** (H0 after ~11k games). Kept here for
inventory only — do not retry as-is.

### 1d. Existing tooling in `v2.1.0-codex-work` (reuse, don't reinvent)

- **`bench` UCI command** — `bench [depth]` runs a fixed position suite and
  reports a repeatable node fingerprint. `bench 13` == `5,318,762` on Phase 1 final baseline.
  **Use this as the regression-safety check: any "behavior-preserving" refactor
  must keep the fingerprint; any real change will move it — that's expected.**
- **`xtask`** — `cargo xtask build --arch pext --pgo` builds the optimized
  PGO asset for testing on this machine (see `xtask/src/main.rs`, `README.md`
  §PGO). `avx2` is for distribution; **`pext` is the correct arch for local
  testing** since the CPU supports BMI2/PEXT and it is slightly faster.
- **UCI options** are declared in `src/search_options.rs`
  (`get_uci_options()` → `option name … type spin …`). SPSA (weather-factory)
  sets parameters **through UCI options**, so Phase 1 must expose tunables here.
- `cargo bench --bench board` — board/movegen microbenchmarks.

---

## 2. Guiding principles (apply to every phase)

1. **SPRT-gate everything.** No change is "good" until it passes a sequential
   probability ratio test in self-play. Default bounds: `elo0=0 elo1=5`,
   `alpha=beta=0.05`. A pass means "≥0 Elo with 95% confidence"; tighten to
   `elo0=0 elo1=3` for small features.
2. **One change at a time.** Never merge a branch wholesale (that is exactly
   what regressed). Port a single feature, test it, keep or drop it, then move
   to the next.
3. **Tune on entry.** When a new heuristic/term is introduced, SPSA-tune (search
   constant) or Texel-tune (eval weight) its constants *before* the SPRT
   accept/reject decision. Most regressions here are untuned constants.
4. **Default-equivalence first.** When refactoring constants into a struct/UCI
   option, the defaults must reproduce current behavior exactly — verify with
   `bench 13` fingerprint before tuning.
5. **Commit each kept step separately with a descriptive message.** After a
   feature passes its SPRT gate, commit it on the integration branch before
   touching anything else. Never bundle multiple features in one commit — if
   something later needs reverting you want surgical precision. The
   incremental-step style of the codex/claude branches is the right model.
6. **Always use `pext --pgo` builds for testing.** Build test binaries with
   `cargo xtask build --arch pext --pgo`; the result lands in `target/dist/`.
   Use `tools/build_test.ps1` to build and copy to
   `tools\test_engines\` (kept separate from released engines) with a
   human-readable name. Never SPRT-test a `cargo build --release` binary — PGO
   changes the hot-path timing enough to affect measured NPS/Elo.
7. **Test time controls mirror reality.** SPRT at fixed 100 ms/move (`st=0.1`
   in fastchess) with the `SuperGM_4mvs.pgn` book — exactly the Little Blitzer
   condition. (SPSA runs slightly longer at `tc=1`; see Phase 0 settings table.)

---

## 3. Phase 0 — Testing & tuning harness (prerequisite, do this first)

**Goal:** a one-command, SPRT-gated self-play test, plus an SPSA tuning loop.
Nothing else proceeds until the SPRT harness reproduces the known null result.

### Tooling decisions (already scaffolded in `tools/`)

- **Match runner / SPRT: [fastchess](https://github.com/Disservin/fastchess).**
  Faster than cutechess-cli, no Qt dependency, built-in SPRT. **fastchess has no
  built-in SPSA** (a common misconception). Install a release binary to
  `tools\bin\fastchess.exe`. The cutechess **GUI** is still useful for
  eyeballing PGNs, but is not used to run matches.
- **SPSA tuner: [weather-factory](https://github.com/jnlt3/weather-factory).**
  The community-standard external SPSA driver; it perturbs UCI options and runs
  fastchess mini-matches (`use_fastchess: true`). Configs live in
  `tools/spsa_configs/` (see its `README.md`).
- **Test binaries: separate folder.** Build with `tools/build_test.ps1`, which
  produces `pext --pgo` binaries into **`tools\test_engines\`**, kept
  apart from release packaging outputs.

### Chosen settings (rationale)

| Setting | Value | Why |
|---|---|---|
| TC (SPRT) | `st=0.1` (100 ms/move, fixed) | **Exactly** the Little Blitzer deployment condition. Fixed movetime ⇒ time-management not exercised (correct — it isn't used in deployment). |
| TC (SPSA) | `tc=1` → 1+0.01 | Near the fast-blitz regime but long enough to avoid bullet timing jitter; weather-factory is tc-based (auto increment = tc/100). The final `st=0.1` SPRT bridges the small gap. |
| Hash | 64 MB | matches deployment |
| Threads (engine) | 1 | clean single-threaded comparison |
| Concurrency | **15** (physical cores − 1; this box has 16) | Fixed movetime is noise-sensitive to oversubscription — do **not** use the 30 logical processors from the old cutechess script. |
| Book | `SuperGM_4mvs.pgn`, `order=random`, `-games 2 -repeat` | matches the tournament; both colours per opening removes bias |
| SPRT model | `normalized` (nElo) | fastchess default; more TC-robust than logistic |
| SPRT gainer bounds | `elo0=0 elo1=5 alpha=0.05 beta=0.05` | accept H1 = real gain. Tighten `elo1=3` for tiny features. |
| SPRT simplify bounds | `elo0=-5 elo1=0` | non-regression test for cleanups (accept H1 = "not meaningfully worse"). |
| Draw adj. | `movenumber=40 movecount=8 score=10` | speeds up dead-drawn games |
| Resign adj. | `movecount=3 score=600 twosided=true` | `twosided` avoids false resigns when the two engines' evals disagree |
| SPSA `A` | iterations / 10 | weather-factory's only recommended change (`spsa.json`) |
| SPSA `a,c,alpha,gamma` | defaults | leave unchanged per weather-factory guidance |

### Steps

1. **Install helper tools locally** → `tools\setup_tools.ps1` creates
   `tools\bin\fastchess.exe` and `tools\weather-factory\`.
2. **Populate weather-factory's `tuner\` folder** with `tools\setup_spsa.ps1`
   — see `tools/spsa_configs/README.md` for the exact setup.
3. **Scripts are already written:** `tools/sprt.ps1` (fastchess SPRT, settings
   above) and `tools/build_test.ps1` (named `pext --pgo` builds into
   `test_engines`). The SPSA configs are in `tools/spsa_configs/`.
4. **Calibration smoke-test (do this first):** run `tools/sprt.ps1` with the
   released `codex-work` vs `2.0.2` binaries copied into `tools\test_engines\`
   (or pass explicit external paths if you keep released builds elsewhere). It
   **must** return accept-H0 / ~0 Elo — they are behavior-identical. If it
   returns H1, the harness is wrong; fix it before trusting anything else.

### Done when
`tools/sprt.ps1` reproduces the "codex-work ≈ 2.0.2" null result (accept-H0),
and a weather-factory dry run perturbs a (Phase-1) UCI parameter and runs games.

---

## 4. Phase 1 — Expose and SPSA-tune the *existing* constants

**Goal:** strength gain with zero new search behavior — only re-tuning values
that already ship in `codex-work`. Lowest risk, highest confidence.

### Steps

1. **Extract live search constants into a `SearchParams` struct** (model it on
   the codex branch's `src/params.rs` — port that file as the starting point).
   Targets already identified in `src/search.rs`:
   - Futility margin base/improving (`70`, `20`) — `search.rs:1003`
   - Razoring coefficient (`150`) — `search.rs:1007`
   - Null-move margins (`12`, `24`) — `search.rs:1012`
   - LMP prune margin base/improving (`90`, `25`) — `search.rs:1182`
   - Quiet-history prune coefficient (`-4000`) — `search.rs:1186`
   - SEE pruning threshold (`-80`, `/8`, `-800`) — `search.rs:1195`
   - Singular beta multiplier (`2`) — `search.rs:1215`
   - LMP count formula (`4 + 2·d²/3`) — `search.rs:2394`
   - Aspiration delta (`25`) — `search.rs:615`
   - LMR formula coefficients (`0.75`, `2.25`) and weighted terms documented in
     `v2.1.0-claude:src/tune.rs`.
2. **Expose each as a UCI spin option** in `src/search_options.rs` with default
   = current value, sensible min/max. Hide them behind a `tune` cargo feature or
   a `UCI_TuneMode` flag so production builds stay clean.
3. **SPRT gate #1 — default-equivalence** (refactor safety). After exposing the
   params but *before any tuning*, confirm the refactor changed nothing: `bench
   13` fingerprint unchanged is the primary proof; optionally an SPRT vs
   `codex-work` with symmetric bounds (`elo0=-3 elo1=3`) to confirm ~0 Elo. This
   gate proves the wiring is safe.
4. **SPSA-tune each group separately, with its own SPRT after each.**
   Ready-made weather-factory parameter sets exist for both groups:
   - `tools/spsa_configs/config_pruning.json` — futility / razor / null-move /
     LMP / SEE / aspiration / singular margins.
   - `tools/spsa_configs/config_lmr.json` — LMR weighted terms (blocked until
     the LMR weighted terms are ported from `v2.1.0-claude:src/tune.rs`).

   For each group, the workflow is:
   a. Run SPSA to convergence (`tc=1`, typically tens of thousands of games).
   b. Bake the tuned values in as the new defaults.
   c. **SPRT-confirm** vs the pre-tuning head at `st=0.1` (`elo0=0 elo1=5`).
      SPSA over-fits, so this game test at the real TC is the authority.
      If H1 accepted → keep and move to the next group.
      If H0 accepted → investigate (TC mismatch? bad ranges? rollback and move on).

5. **Close Phase 1 only with groups that pass their individual SPRTs.** If a
   tuned group remains inconclusive after a large confirmation run, revert its
   candidate defaults and document the rejection before moving on. For the
   current Phase 1 result, Group B pruning/margins passed H1 and was kept;
   Group A LMR stayed inconclusive after ~58k `[0,3]` SPRT games and was
   reverted to default-equivalent values.

> **Every SPSA run earns its own SPRT.** Each tuning group tunes different
> search behavior, may transfer differently to deployment TC, and must be
> independently confirmed. Gate #1 (step 3) proves the wiring is safe. Each
> group's SPRT in step 4 proves its tuned values actually help. Never collapse
> these — a passing gate #1 says nothing about strength, and tuning group B
> having passed says nothing about whether group A's values are good.

### Expected
+10–30 Elo, low risk. **This is the first real strength milestone.**

---

## 5. Phase 2 — Repairs & proven tuning (TM rewrite, history, LMR redo, FP)

**Goal:** fix Rarog-specific defects and harvest the proven, cheap tuning
wins — then move straight to the biggest lever (Phase 3 eval fitting).
Revised 2026-06-10 after source-level analysis of Rarog vs Basilisk vs
Reckless vs Stockfish plus empirical movetime and depth measurements
(findings below). Work **one item at a time**, tune its constants, SPRT-gate,
keep or drop.

> **Ordering rationale (based on the 2026-06 measurements, finding 10).**
> Cross-engine measurement shows where the strength gap actually lives:
> Basilisk — same eval as Rarog, same design — searches ~2.8 M NPS to
> Stockfish's ~1.0 M yet reaches depth 18 vs 21–23 at `movetime 1000`
> (effective branching factor ~2.2 vs ~1.8) and is still far weaker.
> **Speed is not the lever; eval accuracy is first, search selectivity
> second, time management/speed last.** Therefore: Phase 2 keeps only
> *repairs* (the TM bug is real and Rarog-specific) and *proven-cheap* tuning
> items (each already validated empirically or standard across SF/Reckless),
> Phase 3 fits the eval (biggest lever), Phase 4 runs the search-efficiency
> wave (EBF gap) plus the speed pass and margin re-tunes. The codex ports and
> NPS work that previously sat here moved to Phase 4 — they are speculative
> or small, and must not delay the eval work.
>
> Note on tuning-twice: Phase 3 changes the centipawn scale, so
> centipawn-denominated margins tuned in Phase 2 (futility, do-deeper) will
> shift and are re-tuned in the Phase 4 SPSA wave — that re-tune is already
> planned and is machine time, not lost work. It is NOT a reason to defer
> Phase 2 search items: the LMR coefficients (2.4) live in depth/move-index
> space and survive an eval re-fit, the cp-margin seeds come from Basilisk
> which shares Rarog's current eval, and a stronger engine produces better
> Phase 3 self-play data.

### 5.0 Analysis findings that drive the new ordering

These were verified directly against the engines (not hypotheses):

1. **The 10 ms/move collapse is a time-budget bug, not a search problem.**
   `Move Overhead` defaults to 10 ms and `compute_runtime_limits` computes
   `available = max(movetime − overhead, 1)`. At `go movetime 10` that is
   **1 ms → depth 1** (reproduced on the released 2.1.0 binary). With
   `Move Overhead=1` the same binary reaches depth 7 at 10 ms.
2. **The early-stop predictor discards budget at fixed movetime.** In
   `search_root`, `next_iteration_would_hit_hard`
   (`elapsed + 1.75·last_iter + 1 ≥ hard_ms`) stops the search entirely. Under
   a *clock* this banks time for later moves; under *fixed movetime* (our
   deployment: Little Blitzer ms/move) unused time is simply thrown away.
   Measured: with a 9 ms budget the engine stopped at ~5 ms (≈45% unused);
   the same fires on a fraction of 100 ms moves too.
3. **Rarog's eval is a near-port of Basilisk's eval** (same terms, same
   constants, same structure in `eval.cpp` vs `eval.rs`). The ~85 Elo gap to
   Basilisk 1.5.0 is therefore **search behavior + constants tuning + ~13%
   NPS**, not eval terms. (Eval *tuning* is still the Phase 3 lever — neither
   engine's weights have ever been fitted to data.)
4. **Why Rarog's Phase 1 LMR SPSA failed while Basilisk's identical group
   passed (+15.6 Elo):** Basilisk tuned the LMR **formula coefficients**
   (`LmrBase 75→60`, `LmrDivisor 225→209`, `LmrHistDiv 8192→7830`) alongside
   the node-type adjustments. Rarog's `config_lmr.json` only contained the 4
   adjustment terms; the highest-leverage knobs were never in the tune.
5. **Stockfish's time behavior, measured locally and confirmed in its source**
   (`timeman.cpp`, `search.cpp::check_time`, `iterative_deepening`):
   - **Fixed movetime is a pure hard limit.** SF stops when
     `elapsed >= movetime` — no `Move Overhead` subtraction, no early
     iteration skipping, abort mid-iteration, partial root results used.
     Verified on the local `stockfish.exe`: `go movetime 10` → **depth 12**
     with `time 10` on the final (partial, `upperbound`) info line;
     `go movetime 100` → stops at exactly `time 100`. Compare Rarog: depth 1
     at the same 10 ms (finding 1) and ~45–55% budget use (finding 2).
   - **Clock mode uses an optimum/maximum pair, not a prediction.** SF never
     "predicts whether the next iteration will finish". `timeman.cpp` computes
     `optimumTime`/`maximumTime` from the clock; the ID loop stops *between*
     iterations when `elapsed > optimum × (stability factors)` and aborts
     *mid-iteration* only at `maximum`. Reckless (`src/time.rs`,
     `src/search.rs:218-251`) uses the same soft/hard structure with a
     5-factor soft multiplier.
   - Our fastchess harness already passes `timemargin=20` (`tools/sprt.ps1`),
     and SF survives Little Blitzer at 10–100 ms with this exact behavior, so
     adopting it is safe in both test and deployment environments.
6. **Neither Stockfish nor Reckless ages history between searches.** Both rely
   purely on the gravity update formula
   (`entry += bonus − |bonus|·entry/MAX`) and persist all history across moves
   within a game, clearing only on new game. SF additionally *refills*
   `lowPlyHistory` at every search start (it is ply-relative to the root).
   Rarog's `update_hist_entry` (move_ordering.rs:121) **already uses the
   gravity formula**, yet `age_history()` (search.rs:2078) additionally halves
   ~5 MB of tables on every `go` — non-standard *and* a fixed ~0.5–1 ms tax
   per move. (Basilisk's "age every 2nd search" is a halfway house; go
   straight to the SF/Reckless behavior.)
7. **Per-move quiet futility pruning is standard everywhere but Rarog.**
   SF, Reckless (`search.rs:752-766`: `eval + a·depth + b·history + c ≤ alpha`
   with fail-soft `best_score` update, depth < 16), and Basilisk
   (`eval + 180 + 128·depth ≤ alpha`, depth ≤ 8) all have it. Rarog only has
   an eval-margin LMP variant at depth ≤ 3.
8. **Extended correction history is already in baseline** — removed from this
   phase (see §1a).
9. **Reckless is NNUE-only** — its `evaluation.rs` is a network forward pass;
   there is no HCE to compare against, but its *search* and *time manager*
   are state-of-the-art Rust references and are used heavily below and in
   Phase 4. Reckless-class strength is not reachable without NNUE (§11).
   Milestones at the deployment TC, in order: **M1** Stockfish-18-capped-2600
   (+67 on the local list), **M2** Basilisk 1.5.0 (+85), **M3** +150
   cumulative — after M2, add stronger reference opponents (SF capped 2800,
   then 3000) to the gauntlet so progress stays measurable.
10. **2026-06 depth/NPS measurements pin down the lever order.** Measured at
    `go movetime 1000`, single thread, same machine, on Basilisk 1.5.0 (the
    C++ sibling sharing Rarog's eval and search design): Basilisk ~2.8 M NPS
    reaches depth 18; Stockfish ~1.0 M NPS reaches depth 21–23. Effective
    branching factor ≈ 2.2 vs ≈ 1.8. Conclusion: the gap to Stockfish is
    **not** time management and **not** NPS — the levers in order are eval
    accuracy, then search selectivity, then everything else. At 10 ms/move
    Basilisk reaches depth 10 using 8 ms; SF depth 11–12; Rarog depth 7
    stopping at ~4–5 ms (Rarog's bug — item 2.2 here). Honest ceiling: full
    Stockfish is ~500–700 Elo above Basilisk 1.5; the best HCE engines ever
    built sit ~200 Elo below modern SF; Phases 2–4 realistically buy
    **+150–350 Elo**; true parity is the NNUE project (§11), and every phase
    here makes that switch cheaper and better-tested. The EBF measurement
    protocol is defined in §7 (Phase 4) and is tracked there as the phase
    metric.

### Recommended order (expected-Elo per effort, cheapest/safest first)

#### 2.1 Finish ProbCut (in flight, working tree)

Helpers + UCI tune options are ported; bench 13 = **4,632,725**. Run
`config_probcut.json` SPSA, bake values, SPRT-gate (`elo0=0 elo1=3`). Keep or
drop before touching anything else — do not leave half-tested work in the tree.

#### 2.2 Stockfish-style time management (replaces the old ad-hoc TM)

From finding 5. This is a rewrite of `src/time_manager.rs` plus the stop logic
in `src/search.rs::search_root`, adopting SF's model exactly. It covers **all**
play modes (movetime, clock±inc, movestogo, ponder, nodes, depth, infinite).

**(a) Data model.** Replace `RuntimeLimits { soft_ms, hard_ms }` semantics
with SF's: `optimum_ms` (soft, between-iterations) and `maximum_ms` (hard,
mid-iteration abort), plus `movetime_mode: bool`.

**(b) Fixed movetime** (`go movetime T`): `optimum_ms = maximum_ms = T` —
**no `Move Overhead` subtraction** (SF behavior; verified safe: our fastchess
harness passes `timemargin=20`, and SF itself plays Little Blitzer at
10–100 ms this way). No between-iterations stop at all in this mode: always
start the next iteration; the existing every-2048-nodes check in `check_stop`
(search.rs:2306-2327) aborts at `maximum_ms`. Never abort before depth 1
completes (Reckless guards this; Rarog's depth-1 iteration is fast enough that
the current code is already safe, but keep the guarantee explicit). If
time-forfeit losses ever show up in testing, subtract
`min(MoveOverhead, T/10)` as a safety valve — but try the pure version first.

**(c) Clock mode** (`go wtime/btime [winc/binc] [movestogo]`): port SF
`timeman.cpp` structure with its constants as seeds:
```text
mtg      = movestogo > 0 ? min(movestogo, 50) : 50
timeLeft = max(1, time + inc*(mtg-1) - move_overhead*(2 + mtg))
# sudden death / increment (movestogo == 0):
  logT     = log10(timeLeft / 1000)          # timeLeft in ms
  optConst = min(0.0029869 + 0.00033554*logT, 0.004905)
  maxConst = max(3.3744   + 3.0608*logT,   3.1441)
  optScale = min(0.012112 + (ply + 3.22713)^0.46866 * optConst,
                 0.19404 * time / timeLeft)
  maxScale = min(6.873, maxConst + ply/12.352)
# explicit movestogo:
  optScale = min((0.88 + ply/116.4)/mtg, 0.88 * time/timeLeft)
  maxScale = 1.3 + 0.11*mtg
optimum = max(1, optScale * timeLeft)
maximum = max(optimum, min(0.8097*time - move_overhead, maxScale*optimum))
```
(`ply` = game ply ≈ `2*fullmove`; SF clamps `mtg` down when under 1 s —
`if timeLeft_scaled < 1000 { mtg = scaledTime * 0.05 }` — keep that guard.)
The sketch above is orientation, not gospel: **when implementing, fetch the
current `timeman.cpp` from github.com/official-stockfish/Stockfish and follow
its `TimeManagement::init` line by line** (skip the `nodestime` branch — Rarog
doesn't support nodes-as-time). Don't agonize over the exact constants; they
are seeds for a later SPSA. The *structure* (optimum/maximum split,
overhead·(2+mtg) buffering) is the point.

**(d) Between-iterations soft stop** (clock mode only — replaces both
`next_iteration_would_hit_hard` and the current `effort_scale/score_scale`
block at search.rs:702-730): after each completed iteration compute
```text
fallingEval      = clamp(0.1187 + 0.0221*(prev_avg_score - score), 0.572, 1.708)
                   # prev_avg_score: running average of root scores, like
                   # Reckless `average`; SF uses bestPreviousAverageScore
bestMoveInstab   = 1.10 + 2.29 * tot_best_move_changes
                   # count root best-move changes this search;
                   # decay tot_best_move_changes /= 2 after each iteration
effortFactor     = clamp(interp(root_best_effort, 0.79→0.924, 1.0→0.71),
                         0.71, 0.924)
                   # root_best_effort already exists (search.rs:673)
totalTime        = optimum * fallingEval * bestMoveInstab * effortFactor
stop if elapsed > min(totalTime, maximum)
if legal_moves.len() == 1: stop after depth 2 (already implemented)
```
SF computes **no prediction of the next iteration's cost** — delete that
logic entirely.

**(e) Ponder**: keep Rarog's elapsed-preserving ponderhit. Add SF's
`stopOnPonderhit`: if the soft condition fires while pondering, remember it
and stop immediately when `ponderhit` arrives instead of starting fresh
iterations.

**Gates:** bench 13 unchanged (depth-limited — must be byte-identical).
Manual sanity: `go movetime 10` reaches depth ≥7 with *default* options and
final `info ... time` ≈ 9–10; `go movetime 100` final `time` ≈ 100;
`go wtime 10000 winc 100` uses roughly 300–600 ms early-game (compare SF
side-by-side). Then **two SPRTs**: (i) `st=0.1` `[0,5]` — the deployment
gain; (ii) a clock-mode non-regression at `tc=10+0.1` with simplify bounds
`[-5,0]` (clock TM is now exercised; don't ship it untested). Watch the
fastchess output for time losses — there must be zero.

#### 2.3 History maintenance per Stockfish/Reckless

From finding 6. Delete the per-search `age_history()` halving entirely
(`reset_search_state` passes `age_history=false` everywhere / remove the
call); histories persist across searches within a game (gravity in
`update_hist_entry` already bounds them) and are cleared on `ucinewgame`
(already implemented in `new_game`). Exception: **reset `low_ply_history` at
every search start** (it is indexed relative to the root ply; SF refills it
each search). Worker threads (`reset_worker_state_for_new_game`) get the same
treatment. Bench fingerprint changes — record it. SPRT `[0,3]`.

**Outcome 2026-06-12: dropped.** Implemented, SPRT `[0,3]` H0 at
-12.4 ± 6.2 Elo (6,514 games); reverted. The per-search halving carries real
Elo value (fresh history outweighs accumulated signal at this TC). The
halve-every-2nd-search fallback was skipped — a -12 Elo starting deficit
makes passing `[0,3]` implausible. Do not retry without new evidence.

#### 2.4 LMR formula coefficients + SPSA group A redo

From finding 4.
- Expose `LmrTableBase` (default 768 = 0.75·1024), `LmrTableDiv` (default
  2304 = 2.25·1024), and `LmrHistDiv` (default 8192, currently hard-coded at
  search.rs line with `quiet_hist * 1024 / 8_192`) as tune options.
  **Implementation:** replace `static LazyLock<LMR_TABLE>` (search.rs:31) with
  a `lmr_table: Box<[[i32; 64]; 64]>` on `Searcher`. Track the last-built
  `(base, div)` pair; rebuild in `reset_search_state` only when they change
  (zero overhead for non-tune builds). `lmr_hist_div` used live from
  `self.params`. Default-equivalence gate: bench 13 must be unchanged.
- `config_lmr.json` = all 7 params (4 existing + 3 new). Steps: existing
  ~80, new table params ~50 (ranges wider: base [512,1024], div [1536,3072],
  hist [4096,16384]). SPSA to convergence, bake, SPRT `[0,3]`.
  Basilisk's identical re-tune was worth +15.6 Elo; this is the most
  promising tuning item left.

#### 2.5 QSearch TT-bound stand-pat refinement (problem fix)

From the 2026-06-12 external review (verified against the code). The main
search refines its pruning eval with usable TT bounds
(`eval_for_pruning`, search.rs ~1019: Exact → tt_score; Lower and
tt_score > static_eval → tt_score; Upper and tt_score < static_eval →
tt_score). Quiescence uses the TT entry only as a cached *raw* eval and
never applies the same bound refinement to its stand-pat, missing cheap
cutoffs qsearch could take for free. Fix: apply the identical 3-arm bound
refinement to `stand_pat` after computing it (tt_score already probed at the
top of `quiescence`). Small, self-contained; bench changes — record it.
SPRT `[0,3]`.

#### 2.6 Singular double-extension budget cap (robustness fix)

From the same review, verified: the singular path grants `extension = 2`
(search.rs ~1276) with no per-line budget. Total growth is bounded by
`MAX_PLY = 128`, so it cannot run away entirely, but a pathological tactical
line can still consume a disproportionate share of the budget. Fix the
standard way: count double-extensions along the current line (a small
counter on the stack or passed down; SF capped at ~12 per line) and only
allow `extension = 2` while under the cap. Expected ~0 Elo at st=0.1 but
protects against rare time-loss blowups. Bench may change — record it.
SPRT `[-3,3]` (non-regression; accept if no loss).

#### 2.7 Per-move quiet futility pruning

From finding 7. In the move loop's non-PV pruning block (search.rs:1217-1250),
alongside the existing LMP conditions, add for quiet moves:
```text
if depth <= 8 && static_eval != VALUE_NONE
   && static_eval + fp_base + fp_coeff*depth <= alpha
   && !move_gives_check(...)        # preserve checkers, like existing prunes
{ skip move; optionally best_score = max(best_score, futility_value) fail-soft
  (Reckless search.rs:752-766) }
```
Seeds `fp_base=180`, `fp_coeff=128` (Basilisk); expose both as tune options,
SPSA (may share a run with 2.4 if ranges are set), SPRT `[0,3]`.
Note: these margins are centipawn-scaled and will shift after the Phase 3
eval re-fit; the Phase 4 SPSA wave re-tunes them (planned, not lost work).

#### 2.8 LMR "do deeper / do shallower" re-search

Standard in SF and Reckless, absent in Rarog: when the reduced LMR search
returns `score > alpha` and a full-depth re-search is triggered
(search.rs:1373-1386), adjust the re-search depth first:
```text
do_deeper    = score > best_score + deeper_margin + 2*reduction
do_shallower = score < best_score + shallower_margin
re-search at new_depth + (do_deeper ? 1 : do_shallower ? -1 : 0)
```
Seeds: `deeper_margin=64`, `shallower_margin=8`. Expose both as tune options,
SPSA-tune, SPRT `[0,3]`. Same centipawn-margin caveat as 2.7.

#### 2.9 Test-infra fix: debug builds overflow the main-thread stack (no SPRT)

Pre-existing, test-only: the `uci_process` integration tests fail in debug
builds because constructing the large `Searcher` on the default 1 MB Windows
main-thread stack overflows. Fix cheaply: the big history tables are already
boxed, so the likely culprit is a large stack-constructed intermediate
(`Searcher` contains several inline `[..; MAX_PLY]` arrays); either box
those, construct the `Searcher` directly on the heap, or run the engine's
main loop on a thread spawned with an explicit larger stack size (as SF
does). Zero search impact; gate: `cargo test` fully green in debug. No SPRT.

> **External-review disposition (2026-06-12, PROBLEMS.md from
> Antigravity/Gemini 3.1 Pro):** items 1 (eval untuned → Phase 3),
> 3 (quiet futility → 2.7), 6 (history formula → Phase 4) were already
> planned; 4 and 5 were valid and new → added as 2.5 and 2.6; 2 (age_history
> "tax") is overstated — the halving costs <1% of a 100 ms budget and
> removing it measured -12.4 Elo (2.3); 7 (EBF gap) is a summary, not an
> item. Gemini's proposal to defer all Phase 2 search tuning until after the
> eval re-fit was rejected: LMR params are not centipawn-scaled, the margin
> re-tune is already planned in Phase 4, and a stronger engine improves
> Phase 3 datagen quality.

> **That's the whole phase.** The codex ports (multi-cut, threat history,
> TT-cutoff history), the speed pass, and the other search features moved to
> Phase 4 — per finding 10, eval fitting (Phase 3) comes before speculative
> search work. The `improvements`-branch check-aware ordering item was
> already tried and **discarded** (H0 after ~11k games); do not retry it
> as-is.

### Per-feature checklist (use for every item above)

- [ ] Implement the single feature on the integration branch
      (`v2.1.0-codex-work`).
- [ ] `cargo build`, `cargo test`, `cargo clippy` clean.
- [ ] `bench 13` runs — for behavior changes record the new fingerprint; for
      the pure time-management change (2.2) it must be unchanged.
- [ ] Expose new constants as UCI options (Phase 0/1 mechanism, behind
      `--features tune`).
- [ ] SPSA-tune the new constants (skip for 2.2/2.3, which have no
      meaningful tunables — 2.2's TM constants are seeded from SF and only
      re-tuned later, in Phase 4, if clock play becomes a deployment target).
- [ ] **SPRT vs current integration head** (`elo0=0 elo1=3`; `[0,5]` for 2.2).
- [ ] If accept-H1 → commit. If accept-H0 → discard and document why in the
      tracker.

### Expected
All six items are high-confidence (each is a bug fix, was validated in
Basilisk, or is standard in SF/Reckless): **+30–70 Elo cumulative** at the
deployment TC, plus a dramatic fix at very fast TC (10 ms) and correct
behavior under real clocks. Milestone M1 (SF-capped-2600) should fall during
this phase or early in Phase 3.

---

## 6. Phase 3 — Texel-tune the eval (existing weights first, new terms second)

**Goal:** a data-fitted HCE. **Reframed 2026-06-10:** the original phrasing
("port eval terms, the eval is PST-only") was wrong — `src/eval.rs` *already
contains* material+PST, mobility (safe-square), king safety (attack units +
shelter + storm), passed pawns with extras, pawn structure, threats, bishop
pair, rook files/7th, outposts, hanging pieces, space, king proximity, and
endgame scaling. What it has never had is **a single weight fitted to data** —
every constant is hand-set (inherited from Basilisk, which is equally
untuned). The big Phase 3 lever is therefore *tuning what exists*, and only
then adding the genuinely-new `v2.1.0-claude` terms.

**Method decision:** use a **linear-trace gradient tuner** (the approach
described in Andrew Grant's *Evaluation & Tuning in Chess Engines* and used
by Ethereal and most modern HCE tuners), *not* coordinate descent and *not*
SPSA. Rationale, specific to Rarog's eval:
- Almost every term is `weight × count` — linear in the weight. PSTs and the
  king-safety `SAFETY` table are one-hot lookups (linear in the selected
  entry). The tapering phase is computed from the position, not from the
  weights. So for a fixed position the eval is (almost) an affine function of
  the parameter vector, and one cheap "trace" of feature counts per position
  lets the tuner recompute the eval and its exact gradient without ever
  calling `evaluate()` again.
- With traces, one full-dataset gradient step costs one sparse dot product
  per position — all ~900 parameters fit in minutes on CPU. Coordinate
  descent re-evaluates the loss once per parameter per step and cannot handle
  the 768 PST entries in tolerable time. SPSA needs *games* rather than
  labeled positions, orders of magnitude more expensive per parameter.
- The few genuinely non-linear pieces (drawish-endgame scaling, 50-move
  damping, mate-drive term, the king-zone unit-counting logic) are **frozen**:
  their effect is captured per-position in a `scale` factor and a `rest`
  constant (step 3.2), so reconstruction stays exact without tuning them.

### Steps

#### 3.0 `EvalParams` struct — default-equivalence refactor

Hoist **every** tunable weight in `src/eval.rs` into an `EvalParams` struct.
Complete field inventory (defaults = the current inline constants; line
numbers are current `src/eval.rs`):

| Group | Fields (mg, eg where paired) | Current defaults | Where |
|---|---|---|---|
| Material | `mg_val[6]`, `eg_val[6]` | `{82,337,365,477,1025,0}` / `{94,281,297,512,936,0}` | eval.rs:12-13 |
| PSTs | `pst_mg[6][64]`, `pst_eg[6][64]` | PeSTO values | eval.rs:17-78 |
| Passed pawns | `passed_mg[8]`, `passed_eg[8]` | `{0,5,10,20,35,60,100,0}` / `{0,10,17,35,62,100,170,0}` | eval.rs:417-418 |
| Passer extras | supported `(8; 6 + 4·rank)`, free-stop `(2·rank; 6·rank)`, safe-stop eg `8·rank`, candidate `(6,10)` | per code | eval.rs:432-454 |
| Pawn structure | doubled `(-10,-20)`, isolated `(-15,-20)`, connected `(7,5)`, backward `(-10,-15)` | | eval.rs:457-482 |
| Bishop pair | `(30,50)` | | eval.rs:520-523 |
| Rook | open `(25,10)`, semi-open `(12,8)`, 7th `(20,40)`, behind own passer `(15,25)`, enemy rook behind our passer `(-10,-20)` | | eval.rs:525-542, 744-781 |
| Knight outpost | `(25,15)` | | eval.rs:544-554 |
| Mobility | `mob_mg[4]`, `mob_eg[4]` (N,B,R,Q) | `{4,5,2,1}` / `{4,5,4,2}` | eval.rs:878-898 |
| Pawn threats | minor `(18,12)`, rook `(28,18)`, queen `(45,30)` | | eval.rs:568-586 |
| King safety | attacker units (N/B, R, Q), `SAFETY[16]` table | units `{2,3,5}`; `SAFETY = {0,0,10,25,40,60,80,95,105,110,112,114,115,116,117,118}` | eval.rs:655-673 |
| Shelter | missing-pawn `20` (king file) / `10` (adjacent), distance-1 `15`, distance-2 `7` | | eval.rs:675-705 |
| Storm | weight king-file `7` / adjacent `4`, rank ≥ 3 gate | | eval.rs:707-729 |
| Hanging | N/B `45`, R `60`, Q `80` | | eval.rs:852-858 |
| Passer–king proximity | `(2 + rel_rank)` multiplier base `2` | | eval.rs:814-828 |
| Space | `2` per square (mg only) | | eval.rs:619-632 |
| Tempo | `10` (mg only) | | eval.rs:360-364 |
| Trapped bishop | `(60,40)` | | eval.rs:831-843 |

**Frozen (kept in code, never traced or tuned):** `PHASE_W`, the mate-drive
mop-up term (eval.rs:599-616), opposite-colored-bishop scaling
(eval.rs:912-928), two-knights draw rule, 50-move damping (eval.rs:369-370),
and the king-zone unit-counting *logic* (the `SAFETY` values and unit weights
are tuned; how the zone is constructed is not).

Rust-specific implementation notes:
- **Uniform field shape:** represent every parameter as `[i32; N]` (scalars
  as `[i32; 1]`). Define the struct via a `macro_rules!` table so a single
  source of truth generates: the struct, `Default`, a
  `const NAMES: &[(&str, usize)]` (name, length) list, and
  `fn get(&self, name, idx) -> i32` / `fn set(&mut self, name, idx, v)`
  accessors via a generated `match`. (Rust has no usable `offsetof`
  reflection; the generated-match approach is simple and safe.)
- **`MG_TABLE`/`EG_TABLE` are `const`-baked** from material+PST by `const fn`
  (eval.rs:93-94, 230-245). They must become rebuildable at runtime: move
  them into `Evaluator` as boxed fields (`tables: Box<EvalTables>`, ~24 KB)
  built by `fn build_tables(&EvalParams)` — called once in
  `Evaluator::default()` and again whenever params load. Lookups change from
  `MG_TABLE[c][p][sq]` to `self.tables.mg[c][p][sq]`; everything else is
  untouched. (Do not try to keep the `const` path alive behind a feature
  flag — one code path, verified by the gates below, is simpler and the
  indirection is free in practice.)
- Keep `EvalParams` and the tables inside `Evaluator` (§11 guardrail) — the
  search must see no change.

**Gates:** bench 13 fingerprint identical; `cargo test` clean; release bench
wall-time within ~3% of parent (the indirection must not cost NPS — PGO
usually erases it).

#### 3.1 Tune-time loader and dumper (`--features tune` only)

- Env var `RAROG_EVAL_FILE` → load params (`name index value` per line,
  unknown name = hard error), re-run `Evaluator::build_tables`, **clear the
  pawn cache and the eval cache** (`Evaluator::clear_pawn_table` clears both).
- `dumpeval` console command (like `bench`) writes current params in the same
  format — defines the round-trip the tuner emits.
- Gate: dump → load → dump is byte-identical; release builds expose neither.

#### 3.2 Trace instrumentation + tuner binary

1. Cargo feature `texel` (never enabled in release or `tune` builds). Under
   it, add a trace field directly on `Evaluator`
   (`#[cfg(feature = "texel")] pub trace: EvalTrace`) — no thread-locals
   needed, each search thread already owns its `Evaluator`. `EvalTrace` has
   the same field layout as `EvalParams` but holds `i16` net counts
   (white − black). Reset it at the top of `evaluate()`. Every
   `mg += sign * W * n` site also records `trace.field[idx] += sign * n`, via
   a macro `tr!(self, field, idx, sign * n)` that expands to nothing without
   the feature — so normal builds compile to identical code. One-hot lookups
   (PSTs, `passed_mg[rel_rank]`, `SAFETY[units]`) trace the specific index
   that was read.
2. **Pitfalls (mandatory, Rarog has TWO eval caches):** under `texel`, bypass
   both the pawn cache (`eval_pawns` early return, eval.rs:394-399) and the
   whole-eval cache (`eval_table` hit, eval.rs:323-330) — cached entries
   don't re-emit trace counts and would poison reconstruction.
3. Per-position record: sparse `(param_index, white−black count)` pairs split
   mg/eg, `phase` (0..24), the multiplicative scale actually applied (OCB,
   50-move damping), and a `rest` constant = exact eval minus reconstructed
   linear part at defaults (absorbs frozen non-linear terms, making
   reconstruction exact). Store white-POV eval before the side-to-move
   negation; tempo is a traced feature (±1).
4. Tuner binary — a new workspace member (`tools/texel-tuner` crate, or an
   `xtask texel` subcommand) depending on the rarog lib with
   `features = ["texel"]`; use `rayon` to parallelize the per-position loss
   loop. Behavior:
   - loads `FEN;result` lines (result from White's POV: `1.0`/`0.5`/`0.0`),
     traces each position once, holds the sparse records in memory (1–2 M
     positions is fine);
   - **the objective** (define exactly this):
     `L(w) = (1/N) · Σ (result_i − σ(E_i(w)))²` with
     `σ(x) = 1 / (1 + 10^(−K·x/400))`, where `E_i(w)` is the reconstructed
     white-POV eval in centipawns:
     `E(w) = scale · ((mg(w)·phase + eg(w)·(24−phase))/24 + rest)`;
   - fit `K` first with all weights at defaults (coarse grid over
     `K ∈ [0.5, 2.0]`, then refine; minimize `L`) and freeze it;
   - optimize with **Adam** (lr ≈ 0.05, β₁=0.9, β₂=0.999, full-batch
     gradients), epochs until holdout loss stops improving for ~10 epochs;
     the gradient of `L` w.r.t. each weight follows from the chain rule and
     the stored sparse counts — no numeric differentiation anywhere;
   - `--tune <group>` masks (material / scalars / kingsafety / pst / all);
     untuned params stay at their loaded values;
   - optional small L2 pull toward the PeSTO defaults for PST entries
     (λ ≈ 1e-6), none for scalars; report train + holdout loss every few
     epochs; write the §3.1 file format.
   - **Acceptance test before any tuning run:** for 10,000 random dataset
     positions, reconstructed `E(default)` equals `evaluate()` **exactly**
     (integer-for-integer). Any mismatch is a trace bug; fix before tuning.

#### 3.3 Dataset (self-play primary)

1. Generate with fastchess + the current head binary, node-limited for speed
   and diversity. Write `tools/datagen.ps1` around:
   ```powershell
   tools\bin\fastchess.exe `
     -engine cmd=tools\test_engines\rarog-phase3-base-pext-pgo.exe name=A `
     -engine cmd=tools\test_engines\rarog-phase3-base-pext-pgo.exe name=B `
     -each tc=inf nodes=8000 option.Hash=16 option.Threads=1 `
     -openings file=tools\books\SuperGM_4mvs.pgn format=pgn order=random `
     -rounds 30000 -games 2 -repeat -concurrency 15 `
     -pgnout file=tools\texel\data\selfplay.pgn
   ```
   Node-limited games run ~1–2 s each (~60k games total) and the result
   labels reflect the engine's own playing style, which is what the eval is
   being fitted to predict.
2. Extraction (`tools/texel/extract.py` with `python-chess`; a Rust
   alternative via the `pgn-reader`/`shakmaty` crates is fine if avoiding a
   Python dependency is preferred — the *filters* are what matter):
   skip the first 8 full moves and final 6 plies; skip positions in check;
   skip positions where the **played move** was a capture/promotion (cheap
   quietness proxy); sample ≤12 random plies per game; dedup by FEN (first 4
   fields); holdout split **by game** (5% of games, not 5% of positions —
   positions from one game are correlated). Output `FEN;result` lines.
   Target ≥1.5 M train positions (vary `nodes=` 5000–12000 across runs for
   more variety). `Results.pgn`/`Results2.pgn` may supplement, but self-play
   labels are primary.
3. Only if the first SPRT in 3.4 fails: add the rigorous quietness filter
   (drop positions where qsearch ≠ static eval) and regenerate.

#### 3.4 Staged tuning — each stage SPRT-gated

Bake each accepted stage into `EvalParams` defaults, record `bench 13`, SPRT
vs the previous accepted head (Texel loss is never the verdict — the `st=0.1`
game test is):

| Stage | Group | Gate | Notes |
|---|---|---|---|
| 3.4a | Material only (~10 params) | `[0,5]` | Pipeline proof. If this fails: debug K-fit / dataset / reconstruction — do not proceed. |
| 3.4b | All scalars except king safety + PSTs (~85) | `[0,5]` | Mobility, pawn structure, passers, piece terms, threats, hanging, space, tempo. |
| 3.4c | King safety: `SAFETY[16]`, shelter, storm (~40) | `[0,3]` | Zone-composition logic stays frozen. |
| 3.4d | PSTs + material refit (~780) | `[0,5]` | L2 toward PeSTO; needs the full dataset. PSTs last: biggest block, easiest to overfit, scalars prove the pipeline first. |
| 3.4e | Global polish, everything unfrozen, low lr | `[0,3]` | Stop here regardless of outcome. |

Sanity rule per stage: wildly implausible values (flipped signs on
well-understood terms, wildly non-monotonic passer table) mean dataset/trace
bugs — fix before SPRT. A failed SPRT after a sane fit → revert the stage,
continue; one retry only if a concrete defect was found.

#### 3.5 New eval terms (after 3.4 — the tuner is what makes them pay)

Per-feature loop: implement → trace-instrument → Texel-retune the new group
plus affected neighbors → bake → SPRT `[0,3]` → keep or revert. Two sources,
one feature at a time:

**From `v2.1.0-claude` (already written for Rarog, re-tune on entry):**
candidate passers, safe checks / queen-relief king danger, danger²
conversion, per-rank defended/free/safe-path passer detail, lazy-eval margin
(a speed feature — SPRT decides).

**Bigger structural upgrades (implement fresh; do 0 first if any of the
others are attempted):**
0. *Attack-map infrastructure* (pure refactor, eval output identical, bench
   unchanged): compute once per `evaluate()` call
   `attacked_by[color][piece_type]`, `attacked[color]` (union), and
   `attacked2[color]` (attacked by ≥2); rewrite mobility, king safety, and
   the hanging-pieces term to use these instead of recomputing attacks per
   square (`eval_hanging_pieces` currently calls `attackers_to_color` per
   piece — eval.rs:785-812). Slight NPS gain expected.
1. *Threats package*: replace the flat hanging-piece term with
   per-piece-type `threat_by_minor[pt]` / `threat_by_rook[pt]` (mg,eg)
   tables, hanging refinement (attacked, and undefended or attacked twice
   while defended once), pawn-push threats (squares our pawns could attack
   after one safe push), and a restricted-piece bonus. All traced and tuned.
2. *King safety v2*: add safe-check units per piece type (squares from which
   a check could be delivered that the enemy attacks and we don't defend)
   and weak-ring units (king-ring squares attacked and not solidly
   defended), feeding the existing units → `SAFETY[]` funnel as new tunable
   unit weights.
3. *Per-count mobility tables*: replace linear `mobility × weight` with
   one-hot tables indexed by popcount (`mob_n[9]`, `mob_b[14]`, `mob_r[15]`,
   `mob_q[28]`, mg/eg each), initialized from the current linear values
   (`table[i] = i·w` — exactly eval-equivalent at the start, so bench is
   unchanged until tuned); refine the mobility area to exclude own
   king/queen squares and own blocked pawns.

#### 3.6 Phase boundary validation

Global re-pass already covered by 3.4e. Run the external gauntlet (§8),
update CHANGELOG, rebuild PGO assets.

### Expected
**+60–150 Elo** across the phase — the typical range for fitting a
hand-weighted eval of this size to data for the first time, the largest HCE
lever available, and it also feeds Phase 4 (search constants interact with
eval scale).

---

## 7. Phase 4 — Search-efficiency wave (close the EBF gap) + consolidation

**Goal:** reduce nodes-per-depth toward Stockfish's regime (finding 10:
EBF ≈ 2.2 vs ≈ 1.8) and harvest the Phase-3 interactions — after Phase 3 the
eval scale has changed, which mis-scales every search margin tuned against
the old eval.

**EBF measurement protocol (the phase metric).** Pick three fixed positions
— startpos plus two middlegame FENs from the bench suite (`src/bench.rs`) —
and always use the same three. For each: `go movetime 1000`, single thread,
64 MB hash, fresh engine start; record the last *completed* depth `d` and
total nodes `n`; estimate `EBF ≈ n^(1/d)`. Average the three. Run it once
before this phase starts (baseline; expect ≈ 2.1–2.3) and after each accepted
item; record the values in the tracker. The target is movement toward ≈ 1.9.
This is a *trend* metric, not a gate — SPRT remains the only accept/reject
authority (a change can lower EBF and lose Elo by pruning good moves).

### Steps

1. **Second SPSA wave over the search constants** (pruning group, then the
   Phase-2 LMR group, then the new Phase-2 params: futility, do-deeper,
   ProbCut) at the post-Phase-3 head. Same Phase 1 workflow: SPSA → bake →
   SPRT per group. This is not optional busywork — margins like
   futility/razoring are denominated in eval centipawns, and Phase 3 changed
   what a centipawn means. If clock-based play has become a deployment target,
   add the 2.2 TM constants as their own SPSA group (Reckless tuned all of its
   TM multiplier constants this way).
2. **Search-wave items with concrete specs** (one at a time, SPRT `[0,3]`
   unless noted):
   - **History formula upgrade**: replace the symmetric
     `history_bonus(depth)` (move_ordering.rs:127) — currently used for both
     the cutoff bonus and the penalty on tried-and-failed moves — with
     separately scaled linear bonus/malus, as all strong engines do:
     `bonus = min(bonus_mul·depth − bonus_sub, bonus_max)`,
     `malus = −min(malus_mul·depth − malus_sub, malus_max)`.
     Seeds: `170/90/1700` and `180/100/1500`. Expose all six, SPSA, SPRT.
   - **Qsearch TT-bound stand-pat refinement**: Rarog's main search already
     refines the pruning eval with usable TT bounds
     (`eval_for_pruning`, search.rs:1000-1009) but qsearch stand-pat
     (search.rs:1592-1631) does not — mirror the same bound logic there.
   - **LMR input: TT-move-is-capture**: at node entry compute
     `tt_capture = !tt_move.is_null() && tt_move.is_capture()`; add
     `+lmr_tt_capture` (default 1024, in 1024ths) to quiet-move reductions
     when true. Rationale: a tactical TT move means quiet alternatives are
     less likely to matter.
   - **Qsearch quiet checks at the first qs ply** (SF generates checking
     quiets at `DEPTH_QS_CHECKS`): at `qply == 0`, not in check, after
     captures fail to raise alpha: generate quiets, filter to
     `gives check && see_ge(mv, 0)`, search them like captures, cap at the
     first 4–6. If H0, retry once with a stricter entry gate.
   - **Double-extension cap**: Rarog's singular path can extend by 2 with no
     per-line budget (search.rs:1280-1284), allowing unbounded depth growth
     along one line. Track `double_exts` on the search stack, inherited by
     children; disallow the 2-ply extension beyond `double_ext_max`
     (default 8). Non-regression gate `elo0=-3 elo1=1` — this bounds
     tactical blowups more than it gains.
   - **Razoring restriction experiment**: try `depth <= 1` (from
     `depth <= 3`, search.rs:1017) — RFP covers most of razoring's range and
     the qsearch verification isn't free. SPRT both ways; keep whichever
     passes.
3. **Remaining codex ports** (moved from Phase 2; one at a time):
   - **Multi-cut / singular refinements** (`v2.1.0-codex`:
     `SingularDecision::MultiCut`, singular margins). Rarog already returns
     `singular_beta` on a failed singular search (a multicut form); port only
     the *incremental* codex behavior — diff carefully. Tune `singular_*`
     margins. SPRT-gate.
   - **Threat-aware history** (`threatened_index`, `quiet_threat_score`).
     Note Reckless's stronger formulation (`history.rs:14-32`: quiet history
     indexed by `[from_threatened][to_threatened]`) — prefer that shape if
     the codex port fails its SPRT. SPRT-gate.
   - **TT-cutoff / fail-low-parent history** (`update_tt_cutoff_history`,
     `update_fail_low_parent_history`). Tune divisors. SPRT-gate.
   - **(Optional) codex `tt.rs` overhaul** — measure in isolation; TT changes
     are easy to get subtly wrong. Skip unless a kept feature needs it.
4. **Speed pass** (moved from Phase 2; do whenever convenient within this
   phase — finding 10 demoted it: NPS is not the gap driver, but ~13% vs
   Basilisk is still worth collecting). Profile first (`cargo flamegraph`,
   `samply`, or VTune on the pext build); then apply these
   micro-optimizations — the documented source of Basilisk 1.4.9's NPS edge
   over Rarog: delay direct-check detection in the LMR path until the cheaper
   reduction gates have passed; cache direct-check masks during quiet move
   scoring; add a boolean attack-test helper so legality/outpost/hanging
   checks don't materialize full attacker bitboards for yes/no questions; add
   a fast non-insufficient-material early exit before expensive draw checks;
   scan the move picker with pointers and skip self-swaps. Each change:
   bench fingerprint unchanged → ≥5 bench runs → keep if ≥1% faster; one
   simplify-bounds SPRT (`[-3,0]`) over the batch. Target ≥2.9 M NPS.
5. **Modern refinements menu (Reckless-derived)**, one at a time, SPRT each
   (`[0,3]`). Reference implementations are in the local checkouts — read
   the source; adapt the *idea*, do not transplant NNUE-scaled constants.
   Roughly in order of expected value:
   - **Aspiration-window modernization** (Reckless
     `src/search.rs:98-167`). Three sub-ideas, separable: (i) center the
     window on a *running average* of root scores rather than the last score;
     (ii) grow `delta` with score magnitude (`delta += avg²/25704`) and widen
     asymmetrically on fail-low (`+26·delta/128`) vs fail-high
     (`+60·delta/128`); (iii) on repeated root fail-highs, re-search at
     `depth − reduction` (`reduction += 1` per fail-high). Rarog's loop:
     search.rs:616-688 (`aspiration_delta` param).
   - **Correction-magnitude-aware margins** — Reckless scales its RFP margin
     (search.rs:512-520), futility margin (757), and LMR reduction (812, 890)
     by `|correction_value|`. Rarog already computes the correction value
     every node (`corrected_eval_from_raw`); wiring its magnitude in is small.
   - **Hindsight reductions** (Reckless search.rs:469-499, labeled block):
     after a null/static observation, retroactively adjust the parent's
     reduction decision.
   - **Cutoff-count LMR term** (Reckless `td.cutoff_count`,
     SF `cutoffCnt`): track per-ply cutoff counts; reduce more when the
     child ply has produced many cutoffs.
   - **Bad-noisy futility pruning** (Reckless search.rs:768-779): prune
     SEE-losing captures when `eval + 80·depth + history-term + 24 ≤ alpha`.
   - **Qsearch SEE threshold derived from `(alpha − eval)`** (Reckless
     search.rs:1264) — Rarog has a fixed-form variant
     (search.rs:1674); compare and tune.
6. End of phase: external gauntlet + release per §8. Extend the gauntlet
   opponent list per finding 9's ladder (add SF capped 2800, then 3000, once
   M2 falls) so the rating signal doesn't saturate.

### Expected
+30–80 Elo. Cumulatively, Phases 2–4 should clear milestones M1
(SF-capped-2600) and M2 (Basilisk 1.5.0) and push toward M3 (+150 over the
current 3015) — consistent with finding 10's honest ceiling of +150–350 total
from HCE work. Beyond that plateau, HCE progress gets exponentially more
expensive per Elo (the classical-era Stockfish project spent years there with
thousands of donated cores); when this phase completes, revisit §11.

---

## 8. Release & regression discipline

- Keep `v2.1.0-codex-work` (or `master`) as the **gauntlet baseline**. After
  each phase, run a **multi-opponent gauntlet** (vs 2.0.2, Stockfish 18 capped
  2500 *and* 2600, Basilisk 1.5.0) in Little Blitzer to confirm the SPRT
  self-play gains transfer against external opponents (self-play can
  over-fit). As milestones fall (§5.0 finding 9), add stronger capped SF
  levels (2800, 3000) so the rating signal doesn't saturate.
- Rebuild the PGO asset (`cargo xtask build --arch pext --pgo`, or `avx2` for
  a distribution build) before any gauntlet — tuning changes the hot paths.
- Bump version + CHANGELOG only when a phase clears both SPRT and the external
  gauntlet.

---

## 9. Risks & gotchas

- **Untuned constants are the #1 failure mode** (proven by both prior branches).
  Never SPRT-judge a new heuristic before tuning its constants.
- **Full-budget movetime (Phase 2.2) relies on GUI tolerance.** Stockfish
  plays this way everywhere and our harness passes `timemargin=20`, but watch
  every fastchess run and Little Blitzer log for time-forfeit losses after
  2.2 lands. If any appear, enable the documented safety valve
  (subtract `min(MoveOverhead, movetime/10)`) rather than reverting the
  whole feature.
- **Self-play over-fit.** Confirm gains against external engines periodically.
- **SPSA needs UCI-exposed params.** If a constant isn't a UCI option,
  weather-factory has nothing to set — wire up the UCI option first (Phase 1
  step 2).
- **TT / Zobrist changes** (codex `tt.rs`, `zobrist.rs`) can introduce subtle
  correctness bugs that only show as a slow Elo bleed. Port them isolated and
  watch for hash-move legality assertions / `bench` instability.
- **Texel candidates are judged by SPRT, not by loss.** Lower training/holdout
  loss that does not survive the game test is overfitting; revert it.
- **Eval caches poison Texel traces.** Rarog's `Evaluator` has *two* caches
  (pawn table + whole-eval table); both must be bypassed under the `texel`
  feature and cleared whenever runtime weights change (§6 step 3.2).
- **Don't trust the `bench` fingerprint as a strength signal** — it only proves
  *behavior identity*. A changed fingerprint is neither good nor bad; only SPRT
  decides.
- **Time-management features** must be tested under real clocks, not fixed
  ms/move, or their effect is invisible.

---

## 10. Quick command reference

```powershell
# Inspect what a branch added, step by step
git log --oneline 5a8ce52..v2.1.0-codex
git diff 5a8ce52 v2.1.0-codex -- src/search.rs

# Cherry-pick a single feature step onto an integration branch
git checkout -b feat/probcut v2.1.0-codex-work
git cherry-pick <step-commit>      # or re-implement the isolated diff

# Regression-identity check (refactors only) — Windows PowerShell
echo "bench 13`nquit" | .\target\release\rarog.exe   # expect 4,713,975 on baseline

# Build a named pext-PGO test binary
#   → tools\test_engines\rarog-feat-probcut-pext-pgo.exe
./tools/build_test.ps1 -Suffix feat-probcut

# SPRT self-play — calibration smoke-test (expect accept-H0, ~0 Elo)
./tools/sprt.ps1 `
    -EngineA "tools\test_engines\rarog-v2.1.0-windows-pext-pgo-codex-work.exe" `
    -EngineB "tools\test_engines\rarog-v2.0.2-windows-pext-pgo.exe" `
    -NameA "CW" -NameB "2.0.2"

# SPRT self-play — new feature vs integration head (tight bound for small feature)
./tools/sprt.ps1 `
    -EngineA "tools\test_engines\rarog-feat-probcut-pext-pgo.exe" `
    -EngineB "tools\test_engines\rarog-head-pext-pgo.exe" `
    -NameA "ProbCut" -NameB "Head" -Elo1 3

# SPRT self-play — simplification / non-regression check
./tools/sprt.ps1 -Mode simplify `
    -EngineA "<cleaned>.exe" -EngineB "<head>.exe" -NameA "Clean" -NameB "Head"

# SPSA tuning (requires Phase 1 UCI options + weather-factory setup)
#   see tools/spsa_configs/README.md
cd tools\weather-factory; python main.py
```

---

## 11. NNUE readiness — keep the door open (NOT a scheduled phase)

**This plan is HCE-only. NNUE is explicitly out of scope for every phase above
and is not scheduled.** This section exists for one reason: so the HCE work in
Phases 1–4 does not accidentally make a *future* NNUE switch expensive. None of
the items below are tasks to do now — they are guardrails to observe **while**
doing the phases above. If you never go NNUE, you lose nothing by following them
(they are just clean design). If you ever do, they turn a rewrite into a swap.

### Why the architecture matters more than the feature

The dominant cost of a future HCE→NNUE switch is not training a network — it is
disentangling eval logic that leaked into the search. If eval knowledge lives
only in `src/eval.rs` behind the `Evaluator` struct, the switch is a localized
replacement. If piece values, mobility scores, and danger bonuses are inlined
into pruning margins and move ordering, the switch becomes a surgical rewrite.

### Guardrails to observe during Phases 1–4

1. **Single eval entry point.** The search must only reach eval through
   `Evaluator::eval()` (and its quiescence variant): a function that takes a
   board and returns a side-to-move score. Never call eval helpers from
   `search.rs` directly — not piece values, not PST lookups, not mobility
   counts. NNUE replaces what is *behind* that one call, not the search.

2. **No eval-scale assumptions in search constants.** NNUE networks use a
   different internal scale than HCE centipawns. Pruning margins, aspiration
   windows, and SEE thresholds must stay as standalone tunable numbers (which
   Phase 1 already guarantees via UCI params) rather than being derived from
   eval internals.

3. **`EvalParams` stays an `Evaluator` internal.** The `EvalParams` struct
   (ported in Phase 3 from `v2.1.0-claude`) holds HCE weights. Keep it inside
   `Evaluator`; do not leak it into `SearchOptions` or the search loop. A future
   NNUE build simply ignores `EvalParams` and `tune.rs` — possible only if the
   boundary stayed clean.

4. **Keep board state NNUE-friendly (incremental).** NNUE's accumulator is
   updated from exactly the squares that change on `make_move` / `unmake_move`.
   The board already exposes these hooks and incremental piece keys — preserve
   them. Do not regress to a from-scratch board scan per move, and do not stash
   eval state in the search stack that would later need replicating in an
   accumulator.

5. **Reusable, eval-agnostic infrastructure.** Two things you build for HCE are
   directly reusable for NNUE — keep them decoupled from HCE specifics:
   - The **Texel data pipeline** (quiet positions from PGNs, labeled by result)
     is also the standard NNUE supervised-training dataset format.
   - The **Phase 0 SPRT/SPSA harness** drives binaries over UCI and knows
     nothing about the eval — it works unchanged for an NNUE project.

### What a future NNUE switch would actually touch (reference only)

- `Evaluator::eval()` — replaced with NNUE inference (accumulator read + output
  layer).
- `Evaluator` gains `refresh_accumulator()` / `update_accumulator(mv)`, called
  from `Board::make_move` / `Board::unmake_move`.
- `EvalParams` disappears; the network file path becomes a UCI `string` option.
- `tune.rs` Texel code is repurposed for data generation; training moves to an
  external tool (bullet, marlinflow, etc.).
- Everything else — search, move ordering, SPRT harness, weather-factory — is
  unchanged.

NNUE, if ever pursued, is its own multi-phase project (net architecture,
training-data pipeline, SIMD accumulator, quantization), gated by the same Phase 0
harness. Begin it only after HCE tuning is genuinely exhausted.

### Verdict

No code changes needed now. The single discipline to enforce throughout
Phases 1–4: **never let the search know how the eval works.** If a reviewer
would need to understand mobility counting to understand a pruning condition,
that is a boundary violation — fix it then, not later.

---

## 12. Recommended model for implementation

**Primary driver: Sonnet 4.6 (medium).** Rationale:

- The work is **incremental, plan-driven, and test-gated** — port one feature,
  run the harness, read the verdict, decide, repeat. That is precisely where
  Sonnet 4.6 excels: disciplined multi-step tool use, following an explicit
  checklist, and staying inside guardrails. The SPRT/`bench`-fingerprint gates
  do the quality control, so the model doesn't need to "be right" in one shot —
  it needs to execute the loop faithfully, which Sonnet does reliably and
  cost-effectively.
- Most steps are **mechanical porting + wiring** (extract constants, expose UCI
  options, cherry-pick an isolated diff, write the harness scripts). High
  volume, low ambiguity → Sonnet 4.6 medium is the efficient fit.

**Optional specialist for Phase 2 internals: Codex 5.5 (medium).** The dense
search-algorithm ports (ProbCut verification search, correction-history indexing,
multi-cut/singular interaction) are the one place where a model with strong
algorithmic-code density helps. If Sonnet struggles on a specific search feature,
hand that single feature to Codex 5.5 medium, then return to Sonnet for the
test/tune loop.

**Bottom line:** Drive the whole plan with **Sonnet 4.6 medium**; escalate
individual gnarly search ports to **Codex 5.5 medium** only if needed. Do **not**
let either model merge a feature that hasn't passed its SPRT gate — the process,
not the model, is what guarantees the result.
