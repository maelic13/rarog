# Rarog HCE Improvement Plan

> Implementation guide for taking Rarog from the current `v2.1.0-codex-work`
> state to a measurably stronger hand-crafted-eval (HCE) engine, by porting
> already-written search/eval features under a proper SPRT + SPSA testing
> discipline. **No NNUE for now** — but keep the door open (see §10).
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
   fingerprint `4,713,975`). Its changes are safe micro-optimizations with **no
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
- **Extended correction history** — minor-piece, non-pawn, and continuation
  correction histories (beyond the pawn correction already in baseline).
- **Threat-aware history** — `threatened_index`, `quiet_threat_score`.
- **TT-cutoff / fail-low-parent history** — `update_tt_cutoff_history`,
  `update_fail_low_parent_history`.
- **Time management** — `src/time_manager.rs`.
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

Smallest and most self-contained of the three → **ideal first feature to port**
to validate the harness.

### 1d. Existing tooling in `v2.1.0-codex-work` (reuse, don't reinvent)

- **`bench` UCI command** — `bench [depth]` runs a fixed position suite and
  reports a repeatable node fingerprint. `bench 13` == `4,713,975` on baseline.
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
   `D:\chess\engines\test_engines\` (kept separate from released engines) with a
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
  `D:\chess\fastchess\fastchess.exe`. The cutechess **GUI** is still useful for
  eyeballing PGNs, but is not used to run matches.
- **SPSA tuner: [weather-factory](https://github.com/jnlt3/weather-factory).**
  The community-standard external SPSA driver; it perturbs UCI options and runs
  fastchess mini-matches (`use_fastchess: true`). Configs live in
  `tools/spsa_configs/` (see its `README.md`).
- **Test binaries: separate folder.** Build with `tools/build_test.ps1`, which
  produces `pext --pgo` binaries into **`D:\chess\engines\test_engines\`**, kept
  apart from released engines in `D:\chess\engines\`.

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

1. **Install fastchess** → `D:\chess\fastchess\fastchess.exe` (or add to PATH).
2. **Clone weather-factory** and populate its `tuner\` folder — see
   `tools/spsa_configs/README.md` for the exact setup.
3. **Scripts are already written:** `tools/sprt.ps1` (fastchess SPRT, settings
   above) and `tools/build_test.ps1` (named `pext --pgo` builds into
   `test_engines`). The SPSA configs are in `tools/spsa_configs/`.
4. **Calibration smoke-test (do this first):** run `tools/sprt.ps1` with the
   released `codex-work` vs `2.0.2` binaries (both in `D:\chess\engines\`, not
   `test_engines\` — these are the already-distributed reference builds). It
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

5. **After all groups pass their individual SPRTs**, the Phase 1 milestone is
   complete. The combined tuned binary is the new integration head going into
   Phase 2.

> **Every SPSA run earns its own SPRT.** Each tuning group tunes different
> search behavior, may transfer differently to deployment TC, and must be
> independently confirmed. Gate #1 (step 3) proves the wiring is safe. Each
> group's SPRT in step 4 proves its tuned values actually help. Never collapse
> these — a passing gate #1 says nothing about strength, and tuning group B
> having passed says nothing about whether group A's values are good.

### Expected
+10–30 Elo, low risk. **This is the first real strength milestone.**

---

## 5. Phase 2 — Port search-efficiency features from `v2.1.0-codex`

**Goal:** close the depth gap. Port **one feature at a time** from the codex
branch, tune its constants, SPRT-gate, keep or drop.

### Recommended order (cheapest/safest first)

1. **`improvements` branch: check-aware ordering + SEE capture pruning.** Small,
   self-contained → use it to shake out the harness. Cherry-pick
   `move_ordering.rs` + the `see_ge(mv, -80*depth)` pruning. SPSA-tune the SEE
   coefficient. SPRT-gate.
2. **ProbCut** (`v2.1.0-codex`). Port `probcut_*` helpers + the in-search call
   site. SPSA-tune `probcut_base_margin` (188), `probcut_depth_margin` (4),
   `probcut_improving_bonus` (28), verification depth. SPRT-gate.
3. **Extended correction history** (minor / non-pawn / continuation). Port the
   history arrays + update/probe sites. Tune the correction divisors. SPRT-gate.
4. **Multi-cut / singular refinements** (`SingularDecision::MultiCut`, singular
   margins). Tune `singular_*` margins. SPRT-gate.
5. **TT-cutoff / fail-low-parent history**, threat-aware history. Tune divisors.
   SPRT-gate.
6. **Time management** (`time_manager.rs`) — only relevant if you test with real
   clocks rather than fixed nodes/ms; port last, validate against fixed-TC games.

> Skip the codex `tt.rs` overhaul unless a specific feature needs it — measure
> it in isolation; TT changes are easy to get subtly wrong.

### Per-feature checklist (use for every item above)

- [ ] Cherry-pick / re-implement the single feature onto a fresh branch off
      `v2.1.0-codex-work` (or the running integration branch).
- [ ] `cargo build`, `cargo test`, `cargo clippy` clean.
- [ ] `bench 13` runs (fingerprint *will* change — record the new value).
- [ ] Expose new constants as UCI options (Phase 0/1 mechanism).
- [ ] SPSA-tune the new constants (Phase 0 helper).
- [ ] **SPRT vs current integration head** (`elo0=0 elo1=3`).
- [ ] If accept-H1 → commit + merge into integration branch. If accept-H0 →
      discard and document why.

### Expected
This is where the depth-13→deeper gain lives. Each accepted feature is typically
+3 to +15 Elo; cumulative over several features is significant.

---

## 6. Phase 3 — Port eval terms from `v2.1.0-claude` (Texel-tuned)

**Goal:** richer, properly-tuned HCE. Same one-at-a-time discipline.

### Steps

1. **Port the `EvalParams` struct + `tune.rs`** from `v2.1.0-claude` onto the
   integration branch first, with **all new terms disabled / zero-weighted** so
   the eval is still default-equivalent (`bench 13` may differ slightly; verify
   SPRT ~0 vs head).
2. **Build a Texel data set**: extract quiet positions from your own
   `Results.pgn` / `Results2.pgn` (and any other large PGN), label by game
   result. Filter out positions in check / with hanging captures (quiescence).
3. **Enable and Texel-tune one term group at a time**, in this order
   (highest historical value first):
   1. Mobility (per piece)
   2. King safety (attacker weights, safe checks, danger² conversion, shelter,
      storm)
   3. Passed pawns (rank table, defended/free/safe-path extras, rooks behind
      passers)
   4. Bishop pair, rook open/semi/seventh, knight outposts
   5. Pawn threats, tempo, candidate passers
4. After Texel-tuning each group, **SPRT-gate** it in self-play (Texel loss
   reduction does not always equal Elo — the game test is the authority).
5. Re-run a **global Texel pass** over material/PST + all kept terms once the set
   is stable, then a final SPRT confirmation.

### Expected
A well-tuned mobility + king-safety + passed-pawn set is typically a solid gain
on a previously PST-only eval. This is the largest HCE lever short of NNUE.

---

## 7. Release & regression discipline

- Keep `v2.1.0-codex-work` (or `master`) as the **gauntlet baseline**. After
  each phase, run a **multi-opponent gauntlet** (vs 2.0.2, Stockfish 18-2500,
  Basilisk 1.4.9) in Little Blitzer to confirm the SPRT self-play gains transfer
  against external opponents (self-play can over-fit).
- Rebuild the PGO asset (`cargo xtask build --arch pext --pgo`, or `avx2` for
  a distribution build) before any gauntlet — tuning changes the hot paths.
- Bump version + CHANGELOG only when a phase clears both SPRT and the external
  gauntlet.

---

## 8. Risks & gotchas

- **Untuned constants are the #1 failure mode** (proven by both prior branches).
  Never SPRT-judge a new heuristic before tuning its constants.
- **Self-play over-fit.** Confirm gains against external engines periodically.
- **SPSA needs UCI-exposed params.** If a constant isn't a UCI option,
  weather-factory has nothing to set — wire up the UCI option first (Phase 1
  step 2).
- **TT / Zobrist changes** (codex `tt.rs`, `zobrist.rs`) can introduce subtle
  correctness bugs that only show as a slow Elo bleed. Port them isolated and
  watch for hash-move legality assertions / `bench` instability.
- **Don't trust the `bench` fingerprint as a strength signal** — it only proves
  *behavior identity*. A changed fingerprint is neither good nor bad; only SPRT
  decides.
- **Time-management features** must be tested under real clocks, not fixed
  ms/move, or their effect is invisible.

---

## 9. Quick command reference

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
#   → D:\chess\engines\test_engines\rarog-feat-probcut-pext-pgo.exe
./tools/build_test.ps1 -Suffix feat-probcut

# SPRT self-play — calibration smoke-test (expect accept-H0, ~0 Elo)
./tools/sprt.ps1 `
    -EngineA "D:\chess\engines\rarog-v2.1.0-windows-pext-pgo-codex-work.exe" `
    -EngineB "D:\chess\engines\rarog-v2.0.2-windows-pext-pgo.exe" `
    -NameA "CW" -NameB "2.0.2"

# SPRT self-play — new feature vs integration head (tight bound for small feature)
./tools/sprt.ps1 `
    -EngineA "D:\chess\engines\test_engines\rarog-feat-probcut-pext-pgo.exe" `
    -EngineB "D:\chess\engines\test_engines\rarog-head-pext-pgo.exe" `
    -NameA "ProbCut" -NameB "Head" -Elo1 3

# SPRT self-play — simplification / non-regression check
./tools/sprt.ps1 -Mode simplify `
    -EngineA "<cleaned>.exe" -EngineB "<head>.exe" -NameA "Clean" -NameB "Head"

# SPSA tuning (requires Phase 1 UCI options + weather-factory setup)
#   see tools/spsa_configs/README.md
cd D:\chess\weather-factory; python main.py
```

---

## 10. NNUE readiness — keep the door open (NOT a scheduled phase)

**This plan is HCE-only. NNUE is explicitly out of scope for every phase above
and is not scheduled.** This section exists for one reason: so the HCE work in
Phases 1–3 does not accidentally make a *future* NNUE switch expensive. None of
the items below are tasks to do now — they are guardrails to observe **while**
doing the phases above. If you never go NNUE, you lose nothing by following them
(they are just clean design). If you ever do, they turn a rewrite into a swap.

### Why the architecture matters more than the feature

The dominant cost of a future HCE→NNUE switch is not training a network — it is
disentangling eval logic that leaked into the search. If eval knowledge lives
only in `src/eval.rs` behind the `Evaluator` struct, the switch is a localized
replacement. If piece values, mobility scores, and danger bonuses are inlined
into pruning margins and move ordering, the switch becomes a surgical rewrite.

### Guardrails to observe during Phases 1–3

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
Phases 1–3: **never let the search know how the eval works.** If a reviewer
would need to understand mobility counting to understand a pruning condition,
that is a boundary violation — fix it then, not later.

---

## 11. Recommended model for implementation

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
