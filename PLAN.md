# Rarog HCE Improvement Plan

> Implementation guide for taking Rarog from the current Phase 2 accepted head
> to a measurably stronger hand-crafted-eval (HCE) engine, by tuning first,
> then adding individually gated search/eval features under a proper SPRT +
> SPSA/Texel discipline. **No NNUE for now** — but keep the door open (see
> §11).
>
> This document is meant to be handed to an implementation model (see
> "Recommended model" at the bottom). Work **one phase at a time, one feature at
> a time**, and never merge a change that does not pass its SPRT gate.

---

## Current checkpoint — implementation starts here

As of 2026-06-19, Phases 0–2, the Phase 2.5 harness-corrected retries, **and**
Phase 2.9 (robustness & free speed) are closed. The engine is at the start of
the **eval-rewrite program** (restructured Phases 3–5). **Read §6 first** — it
states the sequencing principle that the whole program depends on.

| Area | Current state |
|---|---|
| Branch | `v2.3.0` (off `master` at the v2.2.0 release point; `master` is at `Version 2.2.0`) |
| Completed | **Phase 0**, **Phase 1**, **Phase 2**, **Phase 2.5**, **Phase 2.9**, **Phase 4** are closed. v2.2.0 released and the external gauntlet passed (+240 Elo over 2.1.0, ~3000 CCRL — see §10). |
| Current accepted head | **PHASE 4 COMPLETE (v2.2.0)** — 4.7 global polish ACCEPTED (vs Pst46 **+64.97 ± 13.11 Elo, LOS 100%, H1**, 1412 games). Head = `rarog-phase47-polish-pext-pgo.exe`, **`bench = 4,747,104`**. Staged self-play total ≈ **+316**; external gauntlet confirmed **+240 real Elo** transfer. |
| Harness TC | SPSA and primary SPRT both use `tc=3+0.03`; LTC confirmation uses `tc=10+0.1` |
| Next implementation step | **Phase 5 step 1, in progress**: the one post-eval search-constant SPSA wave. **Done so far (code, no games yet):** widened `FutilityNotImproving`/`LmpNotImproving` ceilings `[0,60]→[0,120]`; exposed the previously-hardcoded ProbCut margin (`180`, search.rs:1108) as a new tunable `probcut_margin` field + `ProbCutMargin` UCI option, range `[60,400]`. Bench unchanged at `4,747,104` (no-op at defaults), all 159 tests pass. **Still to prepare:** the futility-direction A/B, the lazy-eval margin (`LAZY_MARGIN`) re-check/widen, and a new TM-constants SPSA group — then hand the user the SPSA run commands per group. |
| Program shape | **Phase 2.9** *robustness + free speed* (no games) → Phase 3 *build eval structure* (no games) → Phase 4 *fit eval once* → Phase 5 *search wave* (SPSA once) |

> **Bench fingerprint re-baseline (2026-06-22, Phase 3.11b).** The canonical
> `bench 13` fingerprint moved **`5,446,782` → `5,354,975`**. Cause: the new
> **KPK bitbase** (correct king+pawn-vs-king draw recognition) is reachable in
> the bench search tree, and scoring those positions as exact draws prunes
> slightly faster (~1.7 % fewer nodes). This is the **first intentional
> behaviour change in Phase 3** — steps 3.0–3.11a were all behaviour-identical
> at `5,446,782`; from **3.11b onward the canonical fingerprint is `5,354,975`**.
> Historical "bench 5,446,782 unchanged" notes on earlier steps remain accurate
> for those steps. The datagen base binary `rarog-phase3-base-pext-pgo.exe` was
> built pre-KPK at `5,446,782`; the Phase-3.4 dataset is unaffected (it is just
> `FEN;result` labels).
>
> **Bench fingerprint re-baseline (2026-06-23, Phase 3.14).** The canonical
> `bench 13` fingerprint moved **`5,354,975` → `4,978,006`** when the eval cache
> was made a true memoisation. Root cause: the passed-pawn free-stop / safe-stop
> bonuses depend on **non-pawn occupancy and enemy attacks**, yet they were
> scored inside `eval_pawns`, whose result is cached by a **pawn-structure-only**
> key — so the pawn cache (and the whole-eval cache built on top of it) returned
> stale values whenever pieces moved but pawns did not, and the engine played a
> *different* eval than a cold recompute (and than the Texel tuner fits). The fix
> moves those bonuses to `eval_passed_pawn_advance`, run every evaluation outside
> the cache; the eval value at every position is **unchanged** (verified), only
> the cache is now exact. Guarded permanently by `tests/eval_cache.rs`
> (cache == cold recompute). **3.11c is bench-neutral and unaffected.**

Closed-phase checklist:

- [x] Phase 0 — harness
- [x] Phase 1 — existing search constants
- [x] Phase 2 — repairs and proven tuning
- [x] Phase 2.5 — harness-corrected retries (2.5.1 LMR redo **accepted, H1 +4 Elo**; 2.5.2 relocated to Phase 5)
- [x] Phase 2.9 — robustness & free speed (2.9.1 time-safety, 2.9.2 native build, 2.9.3 BadCapture shrink, 2.9.4 gives_check clone removal, 2.9.5 get_unchecked investigated/no-op; close-SPRT accepted: +2.0 Elo, LOS 86%, LLR 81%→H1, 0 time losses cross-harness)
- [x] **Phase 3 — eval infrastructure & build-out CLOSED** (3.0–3.14 structure; 3.15 inert-gating rejected; 3.16 lazy eval **accepted, +4.4 Elo**, bench 5,315,678, eval pure). The seeded-0 vs-`p25` NPS gate is superseded (terms are pure overhead until tuned); real vs-`p25` check is the Phase-4 boundary.
- [ ] Phase 4 — eval data-fit campaign (staged Texel)
- [ ] Phase 5 — search-efficiency wave (deferred SPSA + refinements)

### Why this ordering (one-line version)

Build **all** eval structure first (Phase 3, bench-fingerprint-identical, *no
games*), fit the whole enlarged eval to data **once** (Phase 4), then run the
search-constant SPSA wave **once** at the final eval scale (Phase 5). Search
margins are denominated in eval centipawns, so tuning them before the eval is
final is wasted compute. Full rationale and the cheap-Texel-vs-expensive-SPSA
distinction are in **§6**. NNUE stays the terminal option (§13).

### Phase 2.5 setup sync (closed)

Phase 2.5.0 synchronised tune-build UCI option defaults and SPSA JSON seeds to
the baked defaults in `src/params.rs` for the LMR, futility, and pruning
groups. `src/search_options.rs`, `tools/spsa_configs/config_lmr.json`,
`config_futility.json`, `config_pruning.json`, and the SPSA `README.md` all
match `SearchParams::default()`. Keep `user_dev_guide.md` in sync with this
section.

---

## 0. Background — why this plan exists

> **Renumbering note (2026-06-18).** The forward plan was restructured into
> **Phases 3–5** (see §6 for the rationale): **Phase 3** builds the eval
> *structure* (behaviour-identical, no games), **Phase 4** fits the eval to data
> (staged Texel), **Phase 5** is the search-efficiency wave (the one search SPSA
> wave + refinements). Sections §0–§5 below (Background, Phases 0–2, Phase 2.5)
> were written under the *previous* numbering, where "Phase 3" meant the eval
> work and "Phase 4 / §7" meant the search wave. In that older text, read
> **"Phase 3"** as *the eval work (now Phases 3–4)* and **"Phase 4" / "§7 step
> N" (search)** as *Phase 5 / §9 step N*. Closed-phase records are kept verbatim
> as institutional history; the authoritative forward plan is **§6–§9**.

Historical context: three independent attempts to improve `2.0.2` were tested
in long Little Blitzer round-robins (RR, 64 MB, 100 ms/move):

| Engine (branch)              | NPS      | Depth | Elo (run1 / run2) vs 2.0.2 |
|------------------------------|----------|-------|-----------------------------|
| 2.0.2 (baseline, `master`)   | ~2.61 M  | 13.4  | 0 / 0                       |
| 2.1.0 Codex Work (`v2.1.0-codex-work`) | ~2.61 M  | 13.4  | +0.1 / +0.6 (noise) |
| 2.1.0 Codex (`v2.1.0-codex`) | ~1.79 M  | 14.8  | −9 / −76                    |
| 2.1.0 Claude (`v2.1.0-claude`) | ~2.53 M | 13.5 | −33 / −70                   |

**Key conclusions:**

1. **The original `v2.1.0-codex-work` baseline was behavior-identical to
   `2.0.2`** (same `bench 13` fingerprint after the LMR-1024ths port:
   `5,318,762`). That was the clean starting baseline; the branch has since
   advanced through Phase 1 and Phase 2. The current accepted head is no longer
   behavior-identical: `bench 13 = 5,446,782` (Phase 2.5.1 head).
2. **Speed is not the bottleneck.** Rarog runs at ~2.6 M NPS but only reaches
   depth ~13. Stockfish reaches depth ~24 at ~1.1 M NPS. The gap is **search
   efficiency (pruning/ordering/extensions) and eval quality**, not raw speed.
3. **The Codex and Claude branches added the *right* features but regressed**,
   because new search heuristics and eval terms are defined by their tuning
   constants. Shipping them with hand-guessed values loses Elo until tuned.
4. **None of the three attempts were SPRT-tested.** They were judged on noisy
   9,000-game round-robins. The missing piece is a fast, statistically valid
   test/tune loop — that is the true prerequisite, upstream of every feature.

**Therefore: keep using the harness, tune what already ships, then port the
already-written features incrementally — each one SPRT-gated and
SPSA/Texel-tuned on entry.**

---

## 1. Inventory — what exists, and where

All commits below are reachable from the current repo. Branch heads:

| Branch              | Reference | What it contains |
|---------------------|-----------|------------------|
| `master`            | current integration branch | Phases 0/1/2/2.5/2.9/3.0/3.1 accepted; current fingerprint is `bench 13 = 5,446,782`. The old `v2.1.0-codex-work` branch was squash-rebased onto `master` 2026-06-20 and deleted — this *is* that work, just consolidated. |
| `v2.1.0-codex`      | `3de254f` | Search-efficiency rewrite source branch (see below) — reference-only, kept for the Phase 5 feature menu |
| `v2.1.0-claude`     | `870fac0` | Eval expansion + `tune.rs` source branch (see below) — reference-only, kept for the Phase 5 feature menu |
| ~~`improvements`~~  | deleted 2026-06-20 | Small move-ordering refinements; already fully harvested (ported + reverted, commit `0465384`) |
| ~~`claude`~~        | deleted 2026-06-20 | Stale orphaned pointer, identical to an already-merged commit |

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

### 1d. Existing tooling in the integration branch (reuse, don't reinvent)

- **`bench` UCI command** — `bench [depth]` runs a fixed position suite and
  reports a repeatable node fingerprint. `bench 13` == `4,978,006` on the current
  head (Phase 3.14, after the eval-cache fix); it was `5,354,975` through
  3.11b–3.12 (KPK re-baseline), `5,446,782` through Phases 2.5.1–3.11a,
  `5,401,662` at the Phase 2.7 futility baseline, and `5,318,762` at the Phase 1
  final.
  **Use this as the regression-safety check: any "behavior-preserving" refactor
  must keep the fingerprint; any real change will move it — that's expected.**
- **`xtask`** — `cargo xtask build --arch pext --pgo` builds the optimized
  PGO asset for testing on this machine (see `xtask/src/main.rs`, `README.md`
  §PGO). `avx2` is for distribution; **`pext` is the correct arch for local
  testing** since the CPU supports BMI2/PEXT and it is slightly faster.
- **UCI options** are declared in `src/search_options.rs`
  (`get_uci_options()` → `option name … type spin …`). SPSA (weather-factory)
  sets parameters **through UCI options**. Tune-only search parameters are
  exposed behind `--features tune`; release builds must stay clean.
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
7. **Tune and gate at the *same* time control (revised 2026-06-17).** The SPSA
   tuner and the confirming SPRT must run the **identical** TC, or you tune
   under one condition and accept/reject under another — which manufactures
   "transfer failures." Unified TC: **`tc=3+0.03`** (clock + 1% increment,
   Stockfish convention, ~depth 16), with the `SuperGM_4mvs.pgn` book and
   concurrency 15. A **clock** TC (not fixed movetime) is mandatory so the time
   management rewritten in 2.2 is actually exercised, and so results generalize
   across time controls (the deployment goal — *various* TCs, longer-TC play
   the priority, not just 100 ms/move). Add an **LTC confirmation at
   `tc=10+0.1`** (Stockfish's STC) at phase boundaries and for TC-suspect
   features. The old fixed `st=0.1` (100 ms/move) condition is demoted to an
   *optional* phase-boundary sanity gauntlet (`sprt.ps1 -MoveTime 0.1`), never
   the per-feature gate. See the "Test-TC methodology" note below for the full
   rationale and the list of Phase 2 drops this re-opens.
8. **Always update `user_dev_guide.md` in lockstep with this PLAN.** Every
   time a feature is implemented, an SPRT/SPSA result comes in, or a step is
   reordered/dropped, update *both* `PLAN.md` and the `user_dev_guide.md`
   tracker — and refresh the guide's **"Next action"** pointer to the exact
   next command the user runs. The guide is the user's single source of truth;
   a stale guide breaks the ping-pong loop. Do this in the same commit as the
   code/PLAN change, never "later." (This is the most commonly forgotten step
   — treat it as non-optional.)
9. **Review SPSA output constants before baking them.** When a tuning run
   finishes, check every resulting value against its config bounds and re-read
   the code it feeds *before* committing: a value pinned at a bound, driven to a
   degenerate/no-op value, or near-inert under quantization is a signal to
   inspect the implementation and discuss — not to bake blindly. This caught the
   2.8 `ShallowerMargin` guard bug and surfaced 2.10–2.12. Full checklist in §5.

### Test-TC methodology (added 2026-06-17 — supersedes the old `st=0.1` gate)

**The flaw we corrected.** Through Phase 2 the SPSA tuner ran at `tc=1+0.01`
(a clock) while the confirming SPRT ran at `st=0.1` (fixed 100 ms/move). Those
are **two different conditions** — clock vs fixed movetime *and* different
effective depth — so a parameter optimized under one was judged under the
other. The plan kept recording "the SPSA optimum did not transfer to st=0.1"
(2.4, 2.8) as if it were bad luck; it was partly an artifact of the
methodology. **Fix: SPSA and the confirming SPRT now use the identical TC,
`tc=3+0.03`.**

**Why a clock, and why this number.** Deployment goal is strength across
*various* TCs, with longer-TC play the priority (not just 100 ms/move). A clock
TC (a) exercises the time management rewritten in 2.2 — fixed movetime never
does — and (b) generalizes. Increment is kept at **1 % of base** (the Stockfish
convention: large enough to avoid endgame time-scrambles, small enough that
base-time rationing is still tested; raise the *base* to go deeper, never fatten
the increment). `3+0.03` reaches ~depth 16 and, on the Ryzen 9 5950X at
concurrency 15, costs ~3 h per SPRT and ~1 day per SPSA — the chosen
balance of representativeness vs. iteration speed.

**Stockfish reference.** SF/Fishtest gates every patch at **two** clock TCs —
STC `10+0.1` then LTC `60+0.6` (increment always 1 %), never fixed movetime —
because ~10–20 % of STC passers fail LTC, and the flips cluster precisely in
the depth-sensitive search heuristics (reductions, extensions, TM). We adopt
the affordable single-box version: primary gate at `3+0.03`, **LTC
confirmation at `tc=10+0.1`** at phase boundaries and for TC-suspect features.

**Why fixed movetime specifically misjudged some features.** Under fixed
movetime every move gets exactly 100 ms, so you **cannot bank time** — any
feature whose value is *differential effort* (spend more on hard moves) shows
up as pure cost. That is the mechanism behind the 2.8 do-deeper failure (a
feature standard in SF/Reckless, which test on clocks). Depth-sensitive
reduction schedules (2.4 LMR) are mis-tuned the same way.

**Phase 2 drops this re-opens** (retry under `3+0.03` + an LTC check), split by
whether Phase 3's eval re-fit invalidates them:
- **2.4 LMR coefficients** (H0 −1.3) — depth-sensitive; Basilisk's identical
  re-tune passed at a clock TC. Eval-scale-**independent** → retried in
  **Phase 2.5, before Phase 3**.
- **2.8 do-deeper** (H0 −1.4) — the clearest TC artifact (banking mechanism
  above); **cp-coupled** → retried in **Phase 5 (§9 step 1a), after the
  re-fit** (retrying before Phase 3 would be throwaway).

**Drops it does *not* re-open:** 2.1 ProbCut (−24.5, the codex impl was simply
worse than the existing flat ProbCut); 2.3 no-aging history (−12.4, a
mechanistic formula issue already gated on the Phase 4 history-formula fix);
and **2.6 double-extension cap** (−1.7) — its failure is TC-*independent* (time
safety comes from the node-based stop, not depth-bounding; growth is already
`MAX_PLY`-capped), so the clock TC changes nothing for it. Revisit 2.6 only if
real time forfeits are ever observed (see §9 step 2).

**Harness wiring.** `tools/sprt.ps1` defaults to `tc=3+0.03` (`-TC "10+0.1"`
for LTC, `-MoveTime 0.1` for the optional 100 ms sanity gauntlet);
`tools/setup_spsa.ps1` writes `tc=3` into weather-factory's `cutechess.json`.

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
| TC (SPSA **and** primary SPRT) | `tc=3+0.03` (clock, 1% inc) | **Unified** as of 2026-06-17 (see the Test-TC methodology note in §2): tune and gate at the same condition so optima transfer. Clock ⇒ exercises the 2.2 time manager; ~depth 16; generalizes across TCs. ~3 h/SPRT, ~1 day/SPSA on the 5950X. |
| TC (LTC confirmation) | `tc=10+0.1` (Stockfish STC) | Phase-boundary + TC-suspect-feature gate; ~depth 18, ~10 h/run. Catches depth-sensitive flips. |
| TC (legacy sanity only) | `st=0.1` (100 ms/move, fixed) | The old Little Blitzer condition, now an **optional** phase-boundary gauntlet (`sprt.ps1 -MoveTime 0.1`), **not** a per-feature gate. |
| Hash | 64 MB | matches deployment |
| Threads (engine) | 1 | clean single-threaded comparison |
| Concurrency | **15** (physical cores − 1; this box has 16) | In self-play only the side to move computes, so ~16 games saturate 16 cores; oversubscribing (the 30 logical procs) halves NPS and distorts depth. |
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

   For each group, the workflow is (Phase 1 historically ran SPSA at `tc=1`
   and confirmed at `st=0.1`; from 2026-06-17 both use the unified `tc=3+0.03`
   — see the §2 Test-TC note):
   a. Run SPSA to convergence (`tc=3+0.03`, typically tens of thousands of games).
   b. Bake the tuned values in as the new defaults.
   c. **SPRT-confirm** vs the pre-tuning head at the same `tc=3+0.03`
      (`elo0=0 elo1=5`). SPSA over-fits, so this game test is the authority.
      If H1 accepted → keep and move to the next group.
      If H0 accepted → investigate (bad ranges? rollback and move on).

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
wins — then move straight to the biggest lever (the eval work, Phases 3–4).
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
> Phase 4 fits the eval (biggest lever; Phase 3 first builds the structure),
> Phase 5 runs the search-efficiency wave (EBF gap) plus the speed pass and
> margin re-tunes. The codex ports and NPS work that previously sat here moved
> to Phase 5 — they are speculative or small, and must not delay the eval work.
>
> Note on tuning-twice: the eval re-fit (Phase 4) changes the centipawn scale,
> so centipawn-denominated margins tuned in Phase 2 (futility, do-deeper) will
> shift and are re-tuned in the Phase 5 SPSA wave — that re-tune is already
> planned and is machine time, not lost work. It is NOT a reason to defer
> Phase 2 search items: the LMR coefficients (2.4) live in depth/move-index
> space and survive an eval re-fit, the cp-margin seeds come from Basilisk
> which shares Rarog's current eval, and a stronger engine produces better
> eval-tuning self-play data.

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
   Phase 5. Reckless-class strength is not reachable without NNUE (§13).
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
    **+150–350 Elo**; true parity is the NNUE project (§13), and every phase
    here makes that switch cheaper and better-tested. The EBF measurement
    protocol is defined in §9 (Phase 5) and is tracked there as the phase
    metric.

### Recommended order (expected-Elo per effort, cheapest/safest first)

#### 2.1 ProbCut (codex-branch port)

Helpers + UCI tune options were ported (working-tree bench 13 = 4,632,725),
SPSA-tuned via `config_probcut.json`, and SPRT-gated.

**Outcome 2026-06 — DROPPED (commit `426e6e8`).** SPSA settled at 165/1/31
(base/depth/improving); SPRT `[0,3]`: **H0**, −24.5 ± 8.5 Elo after 3,380
games. The codex-branch implementation (cut-node gating, SEE threshold,
verification search) was ~−25 Elo vs the original flat `beta+180` ProbCut
already in baseline. Reverted to the original; the codex helpers and tune
options were removed.

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

**Outcome 2026-06 — CONFIRMED (commit `72f1c54`).** Bench 13 = 5,318,762
(unchanged — behavior-preserving at depth-limited bench). (i) `[0,5]` st=0.1:
**H1**, +81.2 ± 19.5 Elo (nElo +106.6), 762 games — the broken
`max(movetime−10ms, 1ms)` had been capping to depth 1 at st=0.1. (ii) `[-5,0]`
clock-mode simplify: **H1** (no regression, also gains under a real clock),
+72.6 ± 18.8 Elo. Zero time forfeits in both runs. This is the largest single
Phase 2 gain.

#### 2.3 History maintenance per Stockfish/Reckless

From finding 6. Delete the per-search `age_history()` halving entirely
(`reset_search_state` passes `age_history=false` everywhere / remove the
call); histories persist across searches within a game (gravity in
`update_hist_entry` already bounds them) and are cleared on `ucinewgame`
(already implemented in `new_game`). Exception: **reset `low_ply_history` at
every search start** (it is indexed relative to the root ply; SF refills it
each search). Worker threads (`reset_worker_state_for_new_game`) get the same
treatment. Bench fingerprint changes — record it. SPRT `[0,3]`.

**Outcome 2026-06-12: dropped (mis-ordered, not wrong).** Implemented, SPRT
`[0,3]` H0 at -12.4 ± 6.2 Elo (6,514 games); reverted. The implementation was
faithful (delete `age_history()`, persist all tables, zero only the
root-ply-indexed `low_ply_history`); the loss is genuine and explainable.
SF/Reckless can skip per-search aging because their **separately-scaled
bonus/malus + gravity** self-regulate the table magnitudes. Rarog still uses
the *symmetric* `history_bonus(depth)` for both bonus and penalty
(move_ordering.rs:127) — it relied on the halving as a crude decay, so
removing the decay before fixing the formula loses Elo.

**Retry condition (forward-reference):** retry this no-aging change *after*
the Phase 5 "history formula upgrade" (§9 step 2) lands the separately-scaled
linear bonus/malus. That formula fix is the "new evidence" — it makes
no-aging viable the way it is in SF/Reckless. Sequence: land the bonus/malus
split + SPSA + SPRT first, then re-attempt no-aging on top of it as its own
`[0,3]` gate. Do not retry before that precondition.

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

**Outcome 2026-06-13: H0, params kept.** SPSA ran 3341 iters / 106k games
(tuned values: TtPvAdj=1110, ExactBound=98, ShallowTt=880, CutNode=1138,
TableBase=738, TableDiv=2334, HistDiv=8268; bench 5,303,734). SPRT `[0,3]`
H0 at -1.31 ± 3.09 Elo, LOS 20%, 26k games. The SPSA optimum did not
transfer to `st=0.1`. **Params retained** (they represent the tuner's best
estimate; reverting to the original hand-coded defaults would be worse).
Note: no engine-strength revert needed — params stay baked, only the SPRT
claim of "improvement" was rejected.

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

**Outcome 2026-06 — CONFIRMED (commit `374445e`).** Bench 13 = 4,600,151.
SPRT `[0,3]`: **H1**, +6.51 ± 3.93 Elo, LOS 99.94%, 16,018 games.

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

**Outcome 2026-06-14: H0, reverted.** Cap=12 measured -1.73 ± 3.12 Elo,
LOS 14%, 25k games. The engine relies on double-extensions at st=0.1 — the
cap is a regression, not neutral. Do not retry without stronger motivation
(e.g. evidence of actual time-loss games at tournament TC).

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

**Implementation 2026-06-14:** added inside the `is_quiet` prune block using
`eval_for_pruning` (the TT-refined eval the other prunes in this function use,
not raw `static_eval`) and a **plain skip — no fail-soft `best_score`
update**. The fail-soft variant was tried first and rejected: it stored an
inflated fail-low as a loose Upper-bound TT entry and bloated bench by ~22%
(5.60M vs 4.60M); the plain skip matches the existing LMP/SEE prunes and
gives bench 4,927,654. `FpBase`/`FpCoeff` exposed (tune feature),
`config_futility.json` added. Workflow: because Basilisk shares Rarog's
current eval, SPRT the seeds first `[0,3]`; run the futility SPSA only if
that is inconclusive.

**Outcome 2026-06-15 — CONFIRMED (commit `621c300`), new Phase 2 baseline.**
The Basilisk seeds (180/128) were inconclusive (LLR -0.71, +0.60 ± 2.41 Elo,
42k games) — the cp-margins did not transfer cleanly to `st=0.1`. SPSA on
`config_futility.json` (5858 iters / 187k games) settled at **FpBase=184,
FpCoeff=117**. Re-SPRT `[0,3]`: **H1**, +7.98 ± 4.42 Elo (nElo +10.97), LOS
99.98%, LLR 2.95, 12,542 games. Bench 13 = 5,401,662. Timeouts balanced
(3 Futility / 1 QS-TT). Lesson reaffirmed: even shared-eval seeds need their
own SPSA at the target TC before the gate.

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

**Implemented then fully reverted (2026-06-15/16).** Both arms were added in the
reducible LMR branch and the do-deeper arm SPSA-tuned. Key reusable finding: the
**do_shallower arm can provably never fire** — the move loop keeps
`alpha >= best_score`, so inside `score > alpha` we always have
`score > best_score`, making `do_shallower = score < best_score + margin`
unreachable for any margin ≥ 0. That arm (and its `ShallowerMargin` param) was
removed; the removal is kept independently as a correct simplification. The
do-deeper arm was then tuned/gated below.

**Outcome 2026-06-16 — DROPPED (do_deeper failed its gate; whole 2.8 reverted).**
The do-deeper-only re-SPSA converged cleanly to **DeeperMargin = 37** (3698 iters
/ 118k games, stable ~900 iters, mid-range in [0,200] — a healthy optimum, not a
floor pin). SPRT `[0,3]` DoDeeper vs Futility: **H0**, **−1.38 ± 2.66 Elo**
(nElo −1.88), LOS 15.5%, LLR −2.95, 35,060 games. The arm is the *only*
behavioral delta vs Futility, so −1.38 is its clean attribution — and it costs
~4% more nodes (bench 5,612,008 at margin 37 vs the Futility 5,401,662) for that
loss, i.e. marginally shallower searches at fixed time. Diagnosis: TC-transfer
failure — SPSA optimum at tc=1+0.01 does not hold at st=0.1, exactly like the 2.4
LMR tune. No fix exists (the best-case margin is one so high the arm never fires
= no feature), so re-tuning at st=0.1 or widening ranges cannot make it positive.
**Removed the `do_deeper` arm, `deeper_margin` param/option, `config_dodeeper.json`,
and the `dodeeper` setup_spsa group**; the re-search is plain at `new_depth`
again. Verified clean revert: bench 13 back to **5,401,662**, byte-identical to
the Futility (2.7) fingerprint — no SPRT needed for a revert to a gated baseline.
Futility (2.7) remains the Phase 2 baseline. The 2.10 hygiene and the
do_shallower-removal proof are kept (independent, behavior-preserving). Low-
priority Phase-4 revisit candidate, but the TC-transfer mechanism is eval-scale
independent, so the odds it flips positive after the eval re-fit are low.

#### 2.9 Test-infra fix: debug builds overflow the main-thread stack (no SPRT)

Pre-existing, test-only: the `uci_process` integration tests fail in debug
builds because constructing the large `Searcher` on the default 1 MB Windows
main-thread stack overflows. Fix cheaply: the big history tables are already
boxed, so the likely culprit is a large stack-constructed intermediate
(`Searcher` contains several inline `[..; MAX_PLY]` arrays); either box
those, construct the `Searcher` directly on the heap, or run the engine's
main loop on a thread spawned with an explicit larger stack size (as SF
does). Zero search impact; gate: `cargo test` fully green in debug. No SPRT.

**Outcome 2026-06-16 — DONE.** Root-caused: `main.rs`
constructed `Engine` (which owns the inline-array `Searcher`, dominated by
`pv_table: [[Move; 128]; 128]`) on the 1 MB main thread *before* moving it into
the existing 16 MB engine thread — and in debug (no copy elision) that
construction overflowed, so every `uci_process` test crashed with
`STATUS_STACK_OVERFLOW` (0xC00000FD) right after the banner. Fix: move
`Engine::new` *inside* the 16 MB thread closure (option (c), completing the
pattern `start()` already used). Did **not** box `pv_table` — it's written on
the hot PV-update path (search.rs:1451/1740), so boxing would add indirection
and violate "zero search impact"; the thread-construction fix changes nothing
about the search. Result: full `cargo test` green in debug (all 7 binaries:
46 + 44 + 1 + 29 + 7 + 14 lib/integration tests). Release bench unchanged at
5,401,662.

#### 2.10 Search hygiene cleanup (from the 2026-06-16 SPSA-constant audit)

Non-behavioral / low-risk fixes surfaced while auditing all previously-tuned
constants (audit findings 3 & 4). None is an active bug, but each is real and
cheap; bundle them so the search reads honestly before Phase 3.

- **Dead branch (finding 4) — DONE (commit `ad5e5e4`).** Removed the
  unreachable `r = if depth < 3 || searched < 2 { 0 } else { table }`
  (the `reducible` gate already guarantees `depth >= 3 && searched >= 2`).
  Behavior-preserving; bench 13 unchanged at 3,513,657.
- **Misleading names (finding 4) — DONE (commit `ad5e5e4`).** Renamed
  `futility_improving` → `futility_not_improving` and `lmp_improving` →
  `lmp_not_improving` (field, UCI option → `FutilityNotImproving` /
  `LmpNotImproving`, setter, `config_pruning.json`, README); they are added
  *when not improving*. Behavior-preserving; bench 13 unchanged.
- **Razoring returns qsearch unconditionally (finding 4) — DEFERRED.**
  search.rs:1036 returns `quiescence(...)` directly with no `if qscore < alpha`
  guard. Known simplification; decide whether to add the guard. Shape-changing
  → its own SPRT `[-3,3]`. Kept out of the behavior-preserving batch so it
  doesn't move the do-deeper baseline.
- **LMR quantization is near-inert for small 1024ths params (finding 3) —
  DEFERRED.** The reduction is integer-quantized: `reduction = (r >> 10)…`
  (search.rs:1370). Any adjustment well under ~512/1024 ply only changes the
  actual reduction probabilistically, so `lmr_exact_bound = 98` (≈0.1 ply) is
  effectively noise. **This is the structural reason Phase 2.4 scored H0.** Fix
  options: (a) drop/merge the sub-quantization knobs, or (b) move the reduction
  accumulator to finer fixed-point and round once. Overlaps the Phase 4 LMR
  re-tune — coordinate so we don't tune the same knobs twice. Shape-changing →
  SPRT `[-3,3]`.

Status: the two behavior-preserving items are **done** (bench 13 unchanged).
The two shape-changing items (razoring guard, quantization rework) stay
deferred with their own SPRT gates — deliberately not bundled before the
do-deeper SPSA/SPRT so they don't move the baseline.

#### 2.11 / 2.12 and the TC-suspect retries — RELOCATED (2026-06-17)

Phase 2 closes here; the remaining audit items and the TC-suspect drops were
split by whether **Phase 3's eval re-fit invalidates them**, so that
eval-independent work happens before Phase 3 and centipawn-coupled work after:

- **2.4 LMR coefficients redo** → **Phase 2.5** (before Phase 3). The Phase 2.4
  H0 (−1.3) was at the old fixed-movetime gate; LMR lives in depth/1024ths
  space (eval-independent), so retry it now at `tc=3+0.03` + LTC. Params are
  already UCI-exposed — pure tuning run, no code.
- **2.12 futility `improving` direction** → **Phase 2.5** (optional). Rarog
  prunes more when improving, opposite to the LMP move-count and to SF.
  Eval-independent behavioral A/B, gated `[-3,3]`.
- **2.8 do-deeper** → **Phase 5 §9 step 1a** (after the re-fit). A TC artifact
  (the banking mechanism), but its `DeeperMargin` is a centipawn margin — retry
  it once Phase 3 has set the eval scale, not before.
- **2.11 Group-B improving coeffs** (`FutilityNotImproving` 49/60,
  `LmpNotImproving` 57/60, pinned at the `[0,60]` ceiling; widen to `[0,120]`)
  → **Phase 5 §9 step 1**. Centipawn-coupled and re-tuned in Phase 5's SPSA
  wave anyway. (Names updated from the pre-2.10 `futility_improving` /
  `lmp_improving`.)
- **2.6 double-extension cap** — *not* retried; its failure is TC-independent
  (time safety is the node-based stop, not depth-bounding).

> **SPSA-constant review is mandatory after every tuning run.** When an SPSA
> run finishes, before baking the values, check each resulting constant against
> its config bounds and re-read the implementation it feeds:
> 1. **Pinned at a bound?** A value at/near a min or max means the optimizer
>    wanted to go further — either the bound is clipping a real gain (widen and
>    re-tune) or the parameter is degenerate/off (decide deliberately).
> 2. **Driven to a degenerate value (0, floor, a no-op)?** Re-read the code path
>    it controls — it may indicate an implementation flaw that wastes work or
>    biases the gradient, exactly like the 2.8 `ShallowerMargin` / missing
>    `newDepth > d` guard. Fix the implementation, then re-SPSA on corrected code.
> 3. **Near-inert (sub-quantization, dominated by another term)?** The tune may
>    be fitting noise; flag it rather than trusting the number (cf. 2.4 LMR).
>
> Surface anything suspicious and **discuss before baking** — do not silently
> accept tuner output. This guideline was added 2026-06-16 after the
> ShallowerMargin audit found three more cases (2.10–2.12) by this exact check.

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

> **Phase 2 is closed (2026-06-17).** Items 2.1–2.10 are resolved; the kept
> wins are 2.2 (TM, +81), 2.5 (qsearch TT stand-pat, +6.5), 2.7 (quiet
> futility, +8) — current head bench 13 = **5,401,662**. The harness was then
> corrected (unified `tc=3+0.03`), which re-opened TC-suspect drops: the
> eval-independent ones (2.4 LMR, 2.12) go to **Phase 2.5 before Phase 3**; the
> centipawn-coupled ones (2.8 do-deeper, 2.11 Group-B widen) to **Phase 4**
> after the eval re-fit. The codex ports (multi-cut, threat history, TT-cutoff
> history) and the speed pass were always Phase 4 — per finding 10, eval fitting
> (Phase 3) comes before speculative search work. **Next: Phase 2.5, then
> Phase 3.** The `improvements`-branch check-aware ordering item was already
> tried and **discarded** (H0 after ~11k games); do not retry it as-is.

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
Original forecast: **+30–70 Elo cumulative** at the deployment TC, plus a
dramatic fix at very fast TC (10 ms) and correct behavior under real clocks.

**Actual so far (2.1–2.10 resolved):** the gains landed where the bugs were —
2.2 SF-style time management **+81 Elo** (the broken movetime budget was the
dominant defect), 2.5 qsearch TT-bound stand-pat **+6.5**, 2.7 per-move quiet
futility **+8** — roughly +95 Elo at st=0.1, above the original range, almost
entirely from the 2.2 TM fix. The tuning/port items mostly did **not** transfer
to st=0.1 and were dropped: 2.1 ProbCut (−24.5), 2.3 no-aging history (−12.4,
retry deferred to Phase 4), 2.4 LMR redo (H0, params kept), 2.6 double-ext cap
(H0), 2.8 do-deeper (H0). Several of those drops were partly **harness
artifacts** (the old fixed-movetime `st=0.1` gate — see the §2 Test-TC note);
the eval-scale-independent ones (2.4 LMR, 2.12) are retried in **Phase 2.5
before Phase 3**, the cp-coupled ones (2.8 do-deeper, 2.11) in **Phase 4 after
the eval re-fit**. Milestone M1 (SF-capped-2600) is the next external check;
expect it to fall during Phase 3.

---

## Phase 2.5 — Harness-corrected retries (do before Phase 3)

**Added 2026-06-17.** When the test harness was corrected (unified SPSA+SPRT at
`tc=3+0.03`, LTC `tc=10+0.1` — §2 Test-TC note), it re-opened the Phase 2 drops
that failed *only* because of the old fixed-movetime `st=0.1` gate. Split them
by whether Phase 3's eval re-fit invalidates them:

- **Eval-scale-independent → retry here, before Phase 3** (Phase 3 does not
  touch them, and a stronger engine improves Phase 3 datagen).
- **Centipawn-coupled → Phase 4**, after the re-fit (retrying now is throwaway —
  Phase 3 shifts the margin). Those are 2.8 do-deeper (§9 step 1a) and 2.11
  Group-B widen (§9 step 1).

### 2.5.0 — Synchronize tune setup before SPSA (mandatory)

Before any Phase 2.5 SPSA run, make the tuning setup match the current engine
defaults.

The current source of truth is `SearchParams::default()` in `src/params.rs`.
Tune-build UCI option metadata in `src/search_options.rs`, JSON seed values in
`tools/spsa_configs/config_lmr.json` / `config_futility.json`, and the SPSA
README must not silently point weather-factory at older seed values. If the
implementation intentionally starts a retry from older seeds, it must say so in
both this PLAN and `user_dev_guide.md`.

Checklist:

- [x] Compare every LMR and futility parameter in `src/params.rs` against the
      corresponding tune UCI `default` in `src/search_options.rs`.
- [x] Update `config_lmr.json` so Phase 2.5 starts from the current baked LMR
      values (`1110 / 98 / 880 / 1138 / 738 / 2334 / 8268`).
- [x] Update `config_futility.json` to the accepted quiet-futility defaults
      (`FpBase=184`, `FpCoeff=117`) before any future futility retune.
- [x] Update `config_pruning.json` to the accepted pruning/margin defaults for
      future post-eval retunes.
- [x] Refresh `tools/spsa_configs/README.md` so its tables describe current
      accepted defaults and clearly label historical seeds.
- [x] Run `cargo test --release --test uci_process -- --test-threads=1` after
      the metadata/config change. A full debug `cargo test` can be slow/flaky
      in `uci_process` because debug UCI startup can exceed the current
      15-second test timeout on this machine; treat release UCI tests as the
      release-path smoke check.

### 2.5.1 — 2.4 LMR coefficients redo (primary)

The LMR group failed its Phase 2.4 SPRT (H0 −1.3) at the old fixed-movetime
gate; Basilisk's identical re-tune was +15.6 Elo at a clock TC (finding 4). The
seven params (`LmrTtPvAdj`, `LmrExactBound`, `LmrShallowTt`, `LmrCutNode`,
`LmrTableBase`, `LmrTableDiv`, `LmrHistDiv`) are **already UCI-exposed** from
Phase 2.4. This is a pure tuning run after 2.5.0's setup-sync; no search logic
changes are needed. LMR reductions live in depth / 1024ths space, so the eval
re-fit does not invalidate them.

Workflow: build the tune binary (`build_test.ps1 -Suffix p25-lmr -Tune`),
`setup_spsa.ps1 -ConfigGroup lmr -EngineSuffix p25-lmr`, run weather-factory at
`tc=3+0.03` to convergence, bake, then SPRT `[0,3]` vs the current head at
`tc=3+0.03` **and** an LTC `tc=10+0.1` confirmation. Keep only if both pass.
Note the §2.10 deferred LMR-quantization caveat (`lmr_exact_bound` ≈0.1 ply is
near-inert); if the redo is again inconclusive, take up the quantization rework
before concluding the knobs are dead.

**SPSA result + SPRT verdict (2026-06-18): ACCEPTED (H1, +4 Elo).**
weather-factory ran 2,681 iterations / 85,792 games at `tc=3+0.03` and produced
`LmrTtPvAdj=887`, `LmrExactBound=109`, `LmrShallowTt=656`, `LmrCutNode=780`,
`LmrTableBase=646`, `LmrTableDiv=2335`, `LmrHistDiv=8395` (baked in commit
`692d24a`). The primary SPRT at `tc=3+0.03` accepted **H1 at +4 Elo**. These
are the accepted defaults in `SearchParams::default()`, the tune UCI defaults,
and `config_lmr.json`. **New accepted head: `bench 13 = 5,446,782`.**

### 2.5.2 — 2.12 move-loop futility `improving` direction — RELOCATED to Phase 5

This optional eval-scale-independent A/B (Rarog's move-loop quiet-futility
margin shrinks when `improving`, opposite to SF's no-modulation) was never run.
It is **moved into the Phase 5 search wave** (§9, step 1 "futility-direction A/B"),
where it sits naturally beside the futility-group SPSA at the final eval scale.
It is not worth a separate pre-Phase-3 detour. Behaviour change → never ship
ungated; gate `[-3,3]`.

### Done — Phase 2.5 closed
2.5.0 setup-sync done; 2.5.1 LMR redo accepted (H1 +4 Elo, head bench
`5,446,782`); 2.5.2 relocated to Phase 5. Proceed to **Phase 2.9** (next), then
**Phase 3** (§6).

---

## Phase 2.9 — Robustness & free speed (THE NEXT PHASE — do before Phase 3)

**Why this exists (added 2026-06-19).** The 35k-game overnight gauntlet exposed
two cheap, **eval-independent** problems that are worth fixing *before* the long
eval campaigns, because they are quick, low-risk, and they make every later
SPRT/gauntlet trustworthy:

1. **Time forfeits** — Rarog 2.1.0 lost **28 games on time** (`t=28`) at
   `tc=3+0.03` where 2.0.2 lost 0. Pure lost Elo, and one-sided against
   non-forfeiting opponents, so it **biases every external gauntlet**.
2. **NPS deficit** — Rarog 2.1.0 ran **2.31 M nps / depth 12.4** vs Basilisk
   1.5.1's **2.76 M / 13.8** (~16%). Part is free (build tuning); part is cheap
   code wins identified in a 2026-06-19 Rarog-specific audit.

**Discipline:** every step here is **behaviour-preserving** — `bench 13` stays
`5,446,782` (the time valve changes time use, not fixed-depth node behaviour;
the speed items change storage/codegen, not search results). Gate each on the
fingerprint + `cargo test`; measure nps with ≥5 `bench` runs; one batch
non-regression SPRT `[-3,3]` at the end (and **watch `t=` drop to ~0** in a short
gauntlet). No eval-scale coupling → this does not interact with §6's
tune-once principle.

### Steps (cheapest/safest first)

#### 2.9.1 Time-safety valve — **highest priority** — Sonnet 4.6 medium
Eliminate the forfeits (PLAN §11 risks). On the fixed-movetime path subtract
`min(MoveOverhead, movetime/10)`; on the clock path add a hard
remaining-time floor so a single long iteration cannot overrun. Keep the
full-budget behaviour otherwise. Gate: `bench` unchanged; run a short gauntlet
vs SF-capped and confirm `t=` ≈ 0 before trusting any later number.

**Implementation 2026-06-19 (first pass — clock path was a no-op, corrected
below).** `compute_runtime_limits` (`src/time_manager.rs`) reserves
`safety_margin = min(MoveOverhead, movetime/10)` on the fixed-movetime path
(`budget = movetime - safety_margin`, floored at 1 ms;
`optimum_ms = maximum_ms = budget`) instead of the prior pure `soft = hard = T`.
The first clock-path attempt clamped to `time - overhead`, which is *looser*
than the SF formula's own caps (`optimum ≤ 0.19404·time`,
`maximum ≤ 0.8097·time − overhead`) in the sudden-death/increment branch, so it
**never bound and did nothing for a `tc=3+0.03` gauntlet** (movestogo=0). The
28→15 forfeit change across that retest was sample noise, not the fix.

**Diagnosis from the 2026-06-19 gauntlet (corrected).** In the same Little
Blitzer run, **Rarog 2.0.2 forfeited `t=0` but Rarog 2.1.0-dev forfeited
`t=15`** (of 7,275 games). The only TM difference between them is the Phase 2.2
SF-style rewrite, so **2.2 introduced the forfeits** — its low-time slack
(`0.19·time + overhead`, only ≈8–18 ms at `time=40–50 ms`) is thinner than the
old ad-hoc TM's, and the wall time the GUI charges also includes the latency
before our clock starts (`go` received → `self.start`) and the latency for
`bestmove` to reach the GUI. Under a loaded gauntlet those spike past the slack
and flag. The clock *is* polled every 2047 nodes in both `negamax` and
`quiescence`, so abort granularity (~1 ms) is not the issue; the slack size is.
Little Blitzer is a known confound (Critter forfeits ~100% in LB but 0% in the
user's Colosseum GUI at the same TC), but SF/Lynx/Rarog-2.0.2 all score `t=0`
in this very run, so 2.1.0-dev's 15 is a real over-aggression LB merely exposes.

**Corrected fix (clock path).** After the SF `optimum_ms`/`maximum_ms`
computation, enforce an **absolute reserve of `2*MoveOverhead`**:
`maximum_ms = min(maximum_ms, time - 2*overhead)` (floored at 1 ms), then
`optimum_ms = min(optimum_ms, maximum_ms)`. This binds only when
`time < ~52*overhead` (≈520 ms at the default 10 ms overhead) — i.e. only in
genuine time scrambles, leaving normal-time allocation (the +81 Elo from 2.2)
untouched (unit test `clock_normal_time_allocation_is_not_throttled_by_reserve`
confirms the SF percentage cap still binds at `tc=3+0.03`). `bench 13`
confirmed unchanged at `5,446,782` (debug build and the `pext --pgo` test
binary `rarog-p291-timevalve2-pext-pgo.exe`) — time allocation only, no
fixed-depth search change. Unit tests cover the fixed-movetime capped margin
(incl. the 10%-cap edge case), the clock reserve binding at low time, and the
no-throttle guarantee at normal time.

**Validation — CONFIRMED 2026-06-19.** Little Blitzer gauntlet at `tc=3+0.03`
(32 MB, `3000ms+30ms`) with the corrected binary
(`rarog-p291-timevalve2-pext-pgo.exe`): **Rarog 2.1.0-dev `t=0` over 2,237
games** (every engine in the run also `t=0`), down from the `t=15`/`t=28` that
the same harness produced pre-fix. The valve is confirmed in the very harness
that surfaced the regression; the formal Phase 2.9-close batch SPRT `[-3,3]` in
fastchess will double as the cross-harness time check, so no separate
Colosseum/fastchess run is required to close 2.9.1.

**Health check (no over-correction).** The `2*overhead` reserve did *not* make
Rarog play too fast: measured `tpm=67.8 ms`, mid-pack and slightly *above*
Basilisk 1.5.1 (66.4) and SF-2700 (64.5), and below Rarog 2.0.2 (70.8). Average
allocation is unchanged (the reserve binds only in scrambles, time < ~520 ms).
Depth `d=12.44` is ~1.3 plies below Basilisk (13.75) but **+0.84 over Rarog
2.0.2 (11.60)** — i.e. deeper than the old release, so the gap is the known
nps (~16 %, partly just `pext` vs Basilisk's `-march=native` → 2.9.2) + EBF /
selectivity deficit (Phase 5), *not* time management (Rarog spends more time
per move than Basilisk yet searches shallower). **2.9.1 closed; proceed to
2.9.2.**

**Follow-up 2026-06-19 — movetime path now uses the full budget.** A
`100 ms/move` gauntlet (movetime mode, `go movetime 100`) exposed that the
2.9.1 *movetime* reserve was a misattribution: it subtracted a full
`MoveOverhead`, so Rarog used `tpm=92.9` (90 ms budget + ~3 ms latency) while
Stockfish used `tpm=110.2`, both at `t=0`. The 28 forfeits that motivated 2.9.1
were all in the **clock** path (`tc=3+0.03` = wtime/btime/winc/binc), fixed by
the `2*overhead` reserve in the else-branch; movetime mode never forfeited.
Reverted the movetime branch to the SF/Reckless default `optimum = maximum =
movetime` (full budget, overhead *not* subtracted) — recovers the ~10 % of
thinking time the reserve was discarding. Safe because the harness tolerates
≥10 % over nominal (SF `t=0` at `tpm=110`), our pre/post latency is only ~3 ms
(full budget lands ~3 % over → ~103 ms, well under 110), and movetime mode
already showed `t=0`. `bench 13` unchanged (`5,446,782`); tests updated
(`movetime_uses_full_budget_as_hard_limit`, `movetime_ignores_move_overhead`,
`movetime_tiny_budget_is_at_least_one_ms`). Binary
`rarog-p291-movetimefull-pext-pgo.exe`; **re-run the 100 ms/move gauntlet to
confirm depth↑ and `t=0` holds.**

#### 2.9.2 Native build for local / own-match binaries (free, no code) — Sonnet 4.6 medium — **DONE 2026-06-19**
Added `Arch::Native` to `xtask/src/main.rs` (`--cfg rarog_pext -C
target-cpu=native`, no explicit `+bmi2` needed since `target-cpu=native`
already enables every feature the host CPU has) and a `-Native` switch to
`tools/build_test.ps1` (`rarog-<Suffix>-native-pgo.exe`); `x86-64-v3`/`avx2`/`pext`
remain the default for portable release assets — `cargo xtask build` with no
`--arch` is unaffected. Basilisk's local build is already `-march=native`, so
this also makes the nps comparison fair once run on the 5950X.

**5-min check (this dev box):** `bench 13` is node-identical between `pext`
and `native` (`5,446,782`, confirming the change is build-flags-only, no
behavior difference) and nps was flat (`2,382,669` vs `2,384,755`) — this
machine's baseline CPU apparently already exposes everything `x86-64-v3+bmi2`
needs, so it shows no headroom for `native` to claim. The real gain (if any)
will show on the 5950X, where the host CPU (`znver3`) has instructions beyond
`x86-64-v3` baseline that only `-C target-cpu=native` unlocks. Full
`cargo test --release` suite green. **No games needed (build-flag change,
bench-identical) — proceed to 2.9.3.**

#### 2.9.3 Shrink the `BadCapture` struct — **DONE 2026-06-19**
*(Capacity reduction was considered separately and left alone: the `[_; 256]`
lists are sized for pathological positions; if ever shrunk, make `push`
saturate rather than assume a smaller cap is safe.)*
Changed `to: usize → u8` in `BadCapture` (`move_ordering.rs:64-69`); `attacker:
Piece` and `captured: Option<Piece>` were already 1 byte each (`Piece` is
`#[repr(u8)]`, and `Option<Piece>` is niche-optimized to 1 byte since only 6 of
256 `u8` values are used). With `to` also 1 byte, `size_of::<BadCapture>()`
drops from 16 to 3 bytes (alignment 1, no padding) — each `BadCaptureList [_;
256]` drops from 4 KB to 768 B, ~6.5 KB less stack per negamax frame across the
two lists (`good_caps`/`bad_caps`). Added a regression test
(`move_ordering::tests::bad_capture_struct_stays_shrunk`) asserting
`size_of::<BadCapture>() <= 4` to catch future creep. Updated the three call
sites that read/write the field (`search.rs`: two `push` calls now pass
`mv.to_sq().0: u8` directly instead of `.index(): usize`; three reads of
`*.to` now `as usize` when feeding `update_capture_history`). `bench 13`
unchanged at `5,446,782`; full test suite green (51 tests, incl. the new
size guard); clippy clean (only the pre-existing `tt.rs` `too_many_arguments`
warning). **Bench-identical, no games needed — proceed to 2.9.4.**

#### 2.9.4 Remove the `board.clone()` in `gives_check` for castling — **DONE 2026-06-19**
Replaced the `self.clone(); make_move; is_in_check()` block in `gives_check`
(`board/board.rs`) with a direct test: derive `(rook_from, rook_to)` from the
castle flag + side, build `occ_after` (clear `king_from`/`rook_from`, set
`king_to`/`rook_to`), and return
`(ATTACKS.rook(rook_to, occ_after) & their_king_bb).any()`. **Correctness
argument** (documented in-code): after castling only the moved rook can check —
the king never gives check (kings can't be adjacent), and **no discovered check
is possible** because every vacated square (king's e1/e8, rook's corner
a1/h1/a8/h8) is on the board edge/corner and so can never lie strictly between
one of our sliders and the enemy king. The `occ_after` occupancy is required so
our own king on c1/c8 (queenside) correctly blocks the d1/d8 rook's ray toward
a1/a8. **Validation:** the existing differential oracle
(`gives_check_matches_make_move_check_state_for_curated_positions`, which
compares `gives_check` to clone+`make_move`+`is_in_check` over every legal move)
now also covers two added FENs where castling *delivers* a rook check
(`3k4/8/8/8/8/8/8/R3K3 w Q -` queenside→d1 checks d8;
`5k2/8/8/8/8/8/8/4K2R w K -` kingside→f1 checks f8), exercising the true branch;
all 44 board-correctness tests pass. Removed the one allocator touch
(history `Vec` clone) on a search-reachable path. `bench 13` unchanged
(`5,446,782`); clippy clean (no new warnings). **Bench-identical, no games
needed — proceed to 2.9.5.**

#### 2.9.5 (profile-gated) `get_unchecked` on proven-safe hot table indexes — **INVESTIGATED, SKIPPED 2026-06-19**
The move-scoring inner loop (`quiet_history_score`) and the eval material/PST
loop (`evaluate`) index multi-dimensional arrays with `color as usize` /
`sq.index()` that are provably in `0..2` / `0..64` but LLVM often cannot prove.
Gated on profiling — done with **`cargo-show-asm`** (`cargo install
cargo-show-asm`; inspects the compiled function for `core::panicking::
panic_bounds_check`, the decisive "does the bounds check survive" signal — a
sampling flamegraph can't answer that).

**Findings.** In a *plain* `--release` build the checks survive: the `color as
usize` (0..1) and `piece as usize` (0..5) indices generate none (LLVM proves the
enum-discriminant range), but the `sq.index()` (a `Square(u8)`, 0..255 to LLVM)
indexing into `[_; 64]` does. A prototype applying `get_unchecked` to the
square dimension in both named loops cleanly removed the PST-loop panics in the
`--release` asm. **But the gate is the PGO binary (what actually plays), and
there it buys nothing:** an interleaved `bench 13` nps A/B (identical node count
`5,446,782`, so a clean speed test) of the prototype vs the 2.9.4 head across
20 drift-cancelling pairs showed Δbest `+0.34%`, Δmedian swinging sign run to run
(`+2.5%`, `−1.1%`, `+1.9%`), and the prototype winning **11/20** pairs — i.e.
50/50, no effect. PGO already elides these checks (exactly the "PGO already
recovers much of this" caveat). **Decision:** reverted the prototype — not worth
trading `unsafe` for zero measurable nps. No code change kept; the existing
`get_unchecked` in `attacks.rs` (magic-table lookup) remains the one justified
spot. **2.9.5 closed (no-op) → proceed to the Phase 2.9-close SPRT.**

#### Phase 2.9-close batch SPRT `[-3,3]` — **ACCEPTED 2026-06-19**
Cumulative head (2.9.1–2.9.5) vs Phase 2.5.1 baseline, `tc=3+0.03`, 1 thread,
64 MB, `SuperGM_4mvs.pgn`. Run to 15,976 games without formal SPRT termination,
but clearly trending toward H1 and accepted on the strength of the trend rather
than ground out to a stop:

```
Elo: 2.02 +/- 3.62, nElo: 3.01 +/- 5.39
LOS: 86.33 %, DrawRatio: 42.08 %, PairsRatio: 1.04
Games: 15976, Wins: 4208, Losses: 4115, Draws: 7653, Points: 8034.5 (50.29 %)
Ptnml(0-2): [426, 1846, 3361, 1919, 436], WL/DD Ratio: 0.73
LLR: 2.39 (81.2%) (-2.94, 2.94) [-3.00, 3.00]
```

Cross-harness time-loss check (per the Little-Blitzer time-confound rule — LB
mismeasures time for some engines, so a single-harness count isn't trustworthy
alone): grepped the match PGN for `[Termination "..."]` tags and time/forfeit keywords
— **zero time losses** found (only `"adjudication"` and `"normal"` termination
types appear across all games). The 2.9.1 time-safety fix is confirmed in a
second harness (fastchess/Colosseum, not just Little Blitzer).

**Decision:** accept. Positive Elo, LOS 86%, LLR 81% of the way to the H1 bound
and still climbing, zero forfeits. Phase 2.9 is closed. **Proceed to Phase 3.**

### Expected
Robustness (forfeits → 0) + a few % nps → slightly deeper search. Small but real
Elo (~+5–15) at near-zero risk, and it de-noises all later testing. Then proceed
to **Phase 3** (§6).

---

## 6. Strategy & sequencing — implement first, tune once (read before Phases 3–5)

Phases 3–5 are a single eval-rewrite program whose ordering is dictated by one
principle:

> **Conserve game-based compute. Spend it once, at the end, on a final eval
> scale.**

There are two kinds of tuning with very different costs:

- **Texel weight-fitting** (the eval data-fit): a gradient fit over a fixed
  labelled dataset. Costs **minutes of CPU, zero games**. Re-running it after
  every eval change is cheap and expected — it is the per-term development
  inner loop, **not** a conserved resource.
- **SPSA** (search-constant tuning via weather-factory) and **SPRT** (the game
  gate confirming every accepted change): each costs **thousands of self-play
  games**. These are the conserved resources.

Search margins (futility, razoring, RFP, SEE thresholds, parts of LMR) are
denominated in **eval centipawns**. Rewriting or expanding the eval changes
what a centipawn means, so an SPSA wave over those margins run *before* the
eval is final is thrown away. Therefore:

1. **All eval structural work precedes any post-eval search SPSA** — Phases 3–4
   before Phase 5.
2. **The eval is fit to data once**, as a single staged campaign *after* the
   full eval structure exists (Phase 4) — not as two campaigns
   (tune → add terms → re-tune), which double-spends SPRT on the same regions.
3. **The search-constant SPSA wave runs exactly once, last** (Phase 5), at the
   final eval scale.

To make (2) possible, **every eval rewrite/expansion in Phase 3 is implemented
as a behaviour-identical refactor**: the new structure and all Texel trace
points go in, but new sub-terms are seeded **inert** (zero-effect, or
linear-equivalent to the term they replace), so the engine plays identically
and the `bench 13` fingerprint is unchanged (`5,446,782`). Phase 3 is therefore
gated **cheaply** — bench-fingerprint identity + the trace-reconstruction
acceptance test + unit tests — and spends **no games**. Only in Phase 4 does the
Texel campaign "activate" the new terms by fitting their weights to data, each
stage SPRT-gated. The single genuine exception is endgame/mate *behaviour*
(e.g. KBNK), which cannot be inert; it is confined to material patterns absent
from the bench suite (so bench stays identical) and is gated by unit tests.

**Why this is correct (the sequencing question, answered):** a naive "tune now
for some Elo" over search margins or a partial eval *is* wasted the moment the
eval is rewritten — exactly the concern. Texel is exempt because repeating it is
cheap. The per-feature SPRT gate is unavoidable (it is how Elo is banked) but is
not *wasted*, because each gate confirms a change we keep. The only avoidable
waste — premature SPSA and a double Texel campaign — is removed by this order.

### The gap this program closes (summary; full audit in §15)

**The search is mature; the eval is the gap.** Rarog's search is a modern stack
(PVS + aspiration, NMP with verification, ProbCut, singular extensions,
RFP/futility/LMP, multi-term LMR, five correction histories, staged SEE
move-picker, delta/SEE qsearch) — it does not lose 300 Elo on search. The eval
is a faithful, **never-data-fitted** port of Basilisk's, structurally thin in
the three terms that dominate a strong HCE:

- **King safety** — a capped 16-entry attacker-unit table (max **118 cp**), no
  safe-checks, no weak-ring, no queen-relief, no non-linear scaling
  (eval.rs:634-730).
- **Threats** — pawn-attacks-a-piece plus a flat hanging penalty only
  (eval.rs:568-586, 785-812).
- **Mobility** — linear with tiny weights `{4,5,2,1}` and a loose area
  (eval.rs:557-566, 878-898).

Plus **no scale-factor framework and almost no endgame knowledge** (OCB + KNN
only, eval.rs:912-939), so decided endgames are misplayed and KBNK is likely
not won.

Reference ladder (CCRL 40/15, approximate): SF 11 HCE ~3450 · Ethereal 12 HCE
~3380 · Critter 1.6a ~3150–3200 · modern small HCE (Lambergar/Peacekeeper)
~3150–3210 · Basilisk 1.5.0 ~3100 · **Rarog ~3015 / SF-capped-2600**. The
realistic HCE ceiling for this work is **+150–350** (finding 10); this program
targets the upper half by giving the data-fit enough real terms to bite on.

**Honest expectation per phase** (estimates; SPRT is the only verdict; gains
overlap and do **not** sum linearly):

| Phase | Work | Expected Elo |
|---|---|---|
| 3 | Eval infrastructure + behaviour-identical build-out | 0 direct (enabler) + ~1% NPS |
| 4 | Eval data-fit campaign (the multiplier) | **+120–230** |
| 5 | Search-efficiency wave (deferred SPSA + refinements) | **+20–50** |

This re-scopes the eval *upward* vs the old plan (the build-out adds the
high-value terms the old plan under-scoped as minor bullets) and the mature
search *downward* — which is the realistic read behind the "Phases 3/4 give
< 50" worry: it was right about the search half, wrong about the eval half once
the eval is given real structure to fit.

**Per-step model recommendations** appear inline below and are collected in §14.

---

## 7. Phase 3 — Eval infrastructure & behaviour-identical build-out

**Goal:** put the *entire* enlarged eval structure and all Texel trace points in
place **without changing play** (bench `5,446,782` preserved), so Phase 4 can fit
it all in one campaign. Nothing here is SPRT-gated for strength; every step is
gated on bench-fingerprint identity, the reconstruction acceptance test
(step 3.3), and `cargo test`. No self-play games are spent in Phase 3.

**Inert-seeding rules (apply to every build-out step 3.3–3.11):**
- A *new* sub-term is added with its weight(s) seeded **0** so its contribution
  is exactly zero until Phase 4 tunes it.
- A *replaced* term (e.g. per-count mobility tables) is seeded to reproduce the
  old term exactly (`table[i] = i · old_weight`).
- A *capped* term being un-capped (king-safety table) extends with entries
  seeded equal to the old cap value, so removing the cap is identical.
- A new specialised endgame function must fire **only** on material patterns
  that do not occur in the bench suite, so the fingerprint is unchanged.

### Steps

#### 3.0 Attack-map substrate (behaviour-identical refactor) — **DONE 2026-06-19**
`eval_piece_activity` now runs two passes per `evaluate()` call. Pass 1 builds,
for both colours: `attacks_from_sq[color][64]` (the per-piece attack bitboard
for every N/B/R/Q, keyed by occupied square — at most one piece per square, so
a flat array works), `attacked_by[color][piece_type]` (union per piece type,
incl. pawns/king), `attacked[color]` (union over all piece types), and
`attacked2[color]` (squares attacked by ≥2 of a colour, via the standard
running-OR-of-overlaps trick: `attacked2 |= attacked & new; attacked |= new`).
Pass 2 reuses these: the mobility loop reads `attacks_from_sq` instead of
recomputing `attacks_for`; `eval_king_safety` takes `their_attacks_from_sq`
(the opponent's array, already built in pass 1) instead of recomputing per
zone-attacking piece; `eval_hanging_pieces` replaced the two
`board.attackers_to_color` calls (each an independent 5-piece-type lookup) with
a single bitboard membership test against `attacked[them]`/`attacked[color]` —
correct by the same symmetric-attack-table argument `attackers_to_color`
itself relies on (a piece at `sq2` attacks `sq1` iff `sq1`'s attack pattern for
that piece type, evaluated as if the piece stood at `sq1`, includes `sq2`).
`attacked2` has no consumer yet (no current eval term needs it) — kept
inert/computed-but-unread, reserved for steps 3.6+. **Output is bit-identical**
— `bench 13 = 5,446,782` unchanged; all 50 tests pass; no new clippy warnings
(eval.rs has zero entries among the pre-existing warning set). The NPS effect
of removing the duplicate `attacks_for`/`attackers_to_color` work was not
separately measured — it's small and will be visible (if at all) in the
end-of-phase non-regression SPRT, per the original note. **Model used: Sonnet
4.6** (matches the plan's recommendation — mechanical, equivalence-gated).

#### 3.1 `EvalParams` struct — default-equivalence refactor — **DONE 2026-06-19**

Implemented as specified below: a `macro_rules! eval_params!` table generates
the `EvalParams` struct (every field `pub [i32; N]`, scalars as `[i32; 1]`),
`Default` (each field's default reproduces the constant it replaces exactly),
`EVAL_PARAM_NAMES: &[(&str, usize)]`, and `get`/`set` by `(name, idx)` — all
`#[allow(dead_code)]` since nothing calls `get`/`set`/`EVAL_PARAM_NAMES` until
3.2/3.3 wire up the loader and tuner. ~50 fields cover every group in the
inventory table below, generally split finer than the table's grouped columns
(e.g. "Passer extras" became 8 separate scalar fields — `passed_supported_mg`,
`passed_supported_eg_base`, `passed_supported_eg_per_rank`,
`passed_freestop_mg_per_rank`, `passed_freestop_eg_per_rank`,
`passed_safestop_eg_per_rank`, `passed_candidate_mg`, `passed_candidate_eg` —
so each coefficient in a per-rank linear formula is independently tunable;
the formula *structure* itself, e.g. `eg/per-rank`, is unchanged). PSTs are
flattened to `pst_mg`/`pst_eg: [i32; 384]` (`Piece::ALL` order × 64 squares)
via `build_default_pst`, reusing the original six per-piece consts as the
default source (no PST numbers retyped). `MG_TABLE`/`EG_TABLE` (formerly
`const`) became `EvalTables { mg, eg }`, built at runtime by `build_tables(&EvalParams)`
and stored as `Box<EvalTables>` on `Evaluator`, rebuilt once in
`Evaluator::default()`. Frozen terms (mate-drive mop-up, OCB/two-knights
scaling, 50-move damping, king-zone construction logic) were left as literals,
confirmed via grep (`sign * [0-9]` / `sign * (`) that only the mop-up term's
`5 * (...) + (14 - king_distance) * 4` remains un-parameterized, exactly as
intended. **Output is bit-identical** — `bench 13 = 5,446,782` unchanged; all
50 tests pass; no new clippy warnings (verified before/after: 34 in both, none
in `eval.rs` other than the pre-existing `eval_piece_activity` 8-arg
`too_many_arguments`, present before this step too). **Model used: Sonnet
4.6** (matches the plan's recommendation).

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
- Keep `EvalParams` and the tables inside `Evaluator` (§13 guardrail) — the
  search must see no change.

**Gates:** bench 13 fingerprint identical; `cargo test` clean; release bench
wall-time within ~3% of parent (the indirection must not cost NPS — PGO
usually erases it).

#### 3.2 Tune-time loader and dumper (`--features tune` only) — **DONE 2026-06-20**

`EvalParams::load_from_env()`/`load_from_str()`/`dump()` (all `#[cfg(feature =
"tune")]`, in `src/eval.rs`) implement the `RAROG_EVAL_FILE` round trip: one
`name index value` line per scalar (in `EVAL_PARAM_NAMES` order), unknown
field name is a hard `panic!` (catches stale/typo'd tuner output), a file
that omits fields is valid (omitted fields keep their default). `Evaluator::
default()` calls `load_from_env()` under the `tune` feature instead of
`EvalParams::default()` — since this always builds a fresh `EvalTables` and
fresh (empty) pawn/eval caches, "re-run build_tables, clear both caches" is
satisfied for free by construction, with no separate runtime-reload path
needed (each tuner iteration is a fresh process). `dumpeval` console command
(`src/uci_protocol.rs`) prints `EvalParams::load_from_env().dump()` —
gated `#[cfg(feature = "tune")]` so production builds don't even recognize
the command (verified: plain `cargo build --release` reports `Unknown
command: 'dumpeval'`). **Gate verified**: dump → edit one value → reload via
`RAROG_EVAL_FILE` → dump again is byte-identical (checked by hand and by the
new `eval::tune_tests` unit tests: round-trip identity, partial-file
defaults, unknown-field hard error). `bench 13` unchanged at `5,446,782` on
the plain release build; all 50 tests pass; no new clippy warnings in either
build configuration (34 in both, same pre-existing set).

#### 3.3 Trace instrumentation + tuner binary (+ reconstruction acceptance test) — **DONE 2026-06-22**

Implemented exactly as specified below. **Trace machinery** (`src/eval.rs`,
`#[cfg(feature = "texel")]`): the `eval_params!` macro now also generates
`EvalCounts` (same field layout, net white−black counts), `EvalTrace`
(`mg`/`eg` counts + `phase` + `frozen_mg`/`frozen_eg` + `raw`), `reconstruct()`,
`flat_coeffs()`, and `EvalParams::{FLAT_SIZE, to_flat, set_from_flat}`. Every
`mg += sign·W·n` / `eg += …` site records via `tr_mg!`/`tr_eg!` (no-ops without
the feature, so production builds are byte-identical — `bench 13 = 5,446,782`).
Both eval caches (pawn + whole-eval) are bypassed under `texel`. The two frozen
non-linear contributions (mate-drive mop-up; passer-king-proximity `rel_rank`
constant) accumulate into `frozen_eg` and are excluded from the linear `raw`,
so they land in the tuner's per-position `rest`. The king-safety table is traced
one-hot on the bucket index (the unit weights pick the bucket, so they are not
linear features — matches the reference). **Reconstruction gate**: a `texel`
unit test (`eval::texel_tests`) plays 400 random games and asserts
`reconstruct(defaults) == raw` on >5 k positions — exact, so it catches any
wrong count (not the reference's weaker tautological check). **Tuner binary**:
`tools/texel-tuner` (workspace member, binary `rarog-texel`, depends on the lib
with `features=["texel"]`), a faithful Rust port — golden-section K-fit,
full-batch Adam (lr 0.3, β 0.9/0.999), the staged group masks, `--verify`, the
`linear_delta_scale` (a `pub fn` in `eval.rs` mirroring `scale_drawish_endgames`
+ rule-50), domain clamps (penalty magnitudes ≥0, monotone passer/threat/danger
tables), and the `name index value` output that the Phase-3.2 `RAROG_EVAL_FILE`
loader reads. Parallelised with `std::thread::scope` (no external crate — engine
stays dependency-free). Validated end-to-end: `--verify` PASS, a `material`
tuning run loads → fits K → Adam reduces loss → writes 873-line output → loads
back via `RAROG_EVAL_FILE`/`dumpeval` byte-consistent. **Model used: Opus 4.8.**

> **Reference implementation already on disk.** Basilisk's complete, proven
> tuner is ported to `tools/texel/reference/basilisk_tuner.cpp`. It implements
> exactly this step in C++ (Adam, golden-section K-fit, `active_indices_for_group`
> masks matching the Phase-4 stages, the `linear_delta_scale` that captures the
> frozen non-linear factors as the per-position `scale`, `--verify`
> reconstruction, and the `name index value` output format). **Port its
> *structure*** — the only Rarog-specific work is the trace/`EvalParams`/
> `reconstruct` hookup below (this is why the tuner waits on 3.1/3.3). See
> `tools/texel/README.md` for the C++→Rust mapping (`EVAL_PARAM_LIST` X-macro →
> Rust `macro_rules!`).

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
     epochs; write the step 3.2 file format.
   - **Acceptance test before any tuning run:** for 10,000 random dataset
     positions, reconstructed `E(default)` equals `evaluate()` **exactly**
     (integer-for-integer). Any mismatch is a trace bug; fix before tuning.

#### 3.4 Dataset (self-play primary) — **DONE 2026-06-22 (user ran the games)**

**Result:** 330,000 self-play games (phase3-base PGO binary,
`tools/test_engines/rarog-phase3-base-pext-pgo.exe`, `bench 5,446,782`,
node-limited; Beast-seeded EPD book, two datagen passes appended) → `extract.py`
(`skip_start=16`, `skip_end=6`, `max_per_game=12`, holdout 5%, global dedup) →
**2,190,548 train + 116,112 holdout** unique positions (from 3,162,226 raw
candidates; ≥1.5M target met). Tuner `--verify` **PASS** (10,000/10,000
reconstruct exactly). Outputs at `tools/texel/data/{train,holdout}.csv`; source
PGN `tools/texel/data/selfplay.pgn`. This is the Phase-4 training corpus.

The full pipeline (`sample_fens.py` → `datagen.ps1` → `extract.py` →
`--verify`) was verified end-to-end and the long self-play run was performed by
the user per the standing rule. Exact commands (run from repo root):

```powershell
# 1. Sample a diverse opening book from the Beast pool (source stays intact).
python tools\texel\sample_fens.py "A:\Chess\Beast\data\txt\positions.txt" `
    --out tools\texel\data\beast_seed.epd --count 50000 --min-pieces 6
# 2. Self-play datagen (node-limited; appends to data\selfplay.pgn). USER RUNS.
.\tools\datagen.ps1 -Suffix phase3-base -Rounds 30000 -Nodes 8000 `
    -Book tools\texel\data\beast_seed.epd -BookFormat epd -Concurrency 24
#    Optional second pass for variety (different node count appends):
.\tools\datagen.ps1 -Suffix phase3-base -Rounds 15000 -Nodes 5000 `
    -Book tools\texel\data\beast_seed.epd -BookFormat epd -Concurrency 24
# 3. Extract labelled positions (train + holdout, split by game).
python tools\texel\extract.py tools\texel\data\selfplay.pgn `
    --out-dir tools\texel\data --train train.csv --holdout holdout.csv
# 4. Reconstruction gate, then (Phase 4) tune a stage:
cargo run --release -p texel-tuner -- --verify tools\texel\data\holdout.csv
```

Original spec retained below for reference.

**The toolchain is already built** (`tools/texel/`, ported from Basilisk's
working pipeline — see `tools/texel/README.md`). The scripts run today; only the
tuner binary waits on 3.3. Two label paths, both producing `FEN;target` text:

- **Path A — self-play (primary).** `Beast FENs → sample_fens.py → beast_seed.epd
  → datagen.ps1 (Rarog self-play) → extract.py → train.csv + holdout.csv`. Labels
  are Rarog's own game results — the eval is fitted to predict what Rarog does.
- **Path B — Stockfish-WDL (optional, higher ceiling).** Label sampled FENs with
  a strong engine's WDL and feed `import_beast.py`. Distils SF judgment into the
  HCE; can chase SF quirks that don't transfer. SPRT decides. (No SF binary is in
  the repo — point the labeller at your capped/full Stockfish if you take this
  path.)

**Position source — the Beast pool (read-only).**
`A:\Chess\Beast\data\txt\positions.txt` is 7.1 GB of unique, unlabelled FENs
(quality computer games *and* weak-human games — broad diversity, which is good
for eval generalisation). `sample_fens.py` reservoir-samples it (streaming, 7 GB-
safe) into a fastchess EPD opening book. **The source file is never modified or
copied** — treat it as immutable.

Pipeline (run from repo root; sized for a Ryzen 9 5950X — keep concurrency
moderate so the machine stays usable):
```powershell
.\tools\build_test.ps1 -Suffix phase3-base
python tools\texel\sample_fens.py "A:\Chess\Beast\data\txt\positions.txt" `
    --out tools\texel\data\beast_seed.epd --count 50000 --min-pieces 6
.\tools\datagen.ps1 -Suffix phase3-base -Rounds 30000 -Nodes 8000 `
    -Book tools\texel\data\beast_seed.epd -BookFormat epd -Concurrency 24
python tools\texel\extract.py tools\texel\data\selfplay.pgn `
    --out-dir tools\texel\data --train train.csv --holdout holdout.csv
```
`extract.py` skips opening (16 plies) / endgame (6 plies) / in-check /
capture-or-promo positions, caps ≤12 plies per game, dedups by FEN, and **splits
the holdout by game** (5% of games — positions from one game are correlated).
Target ≥1.5 M train positions (vary `-Nodes` 5000–12000 across runs for variety;
the second pass appends). At `-Concurrency 24` on the 5950X, ~60 k node-limited
games is well under an hour and leaves cores free.

Only if the first Phase-4 SPRT fails: add the rigorous quietness filter (drop
positions where qsearch ≠ static eval) and regenerate.

#### 3.5 King-safety v2 — structure (behaviour-identical; the #1 eval lever) — **DONE 2026-06-22**

`eval_king_safety` (`src/eval.rs`) is rebuilt into a single `danger` accumulator
fed the attack maps via a `KsMaps` bundle (from the 3.0 substrate). Implemented
inputs, all new weights seeded **0** so `danger == units` today (`bench
5,446,782` unchanged, reconstruction exact): **weak king-ring** (`ks_weak_ring`
— zone squares attacked by them, not defended by us or only-once-defended &
doubly-attacked); **safe checks per type** (`ks_safe_check_{knight,bishop,rook,
queen}` — enemy check-from squares the enemy attacks with that type and we don't
defend); **king-flank pressure** (`ks_flank_attack` — enemy attacks − our
defenses over the king's 3 files, clamped ≥0); **pawnless flank**
(`ks_pawnless_flank`); **queen-relief** (`ks_queen_relief`, a reduction when the
attacker has no queen). The **conversion table** `king_safety_table` is
lengthened 16→40, the `.min(15)` cap removed (now
`danger.clamp(0, len-1)`), entries 0..15 = old `SAFETY`, 16..39 = 118 (the old
cap), so today's play is identical and Phase 4 can shape the danger tail. The
table is Texel-tunable (one-hot per bucket); the danger-*input* weights are
non-linear bucket selectors, so they are **SPSA-tuned in Phase 5**, not Texel
(noted in the tuner — they have zero Texel gradient). **Blockers/pins near the
king is deferred to Phase 5** (`maps.own_occ` reserved): 3.0 did not build pin
masks, it is the lowest-value input of the set, and adding it ×0 with no real
pin computation would be a fake stub. **Model used: Opus 4.8.**

Original spec retained below.

Rebuild the king-safety term (eval.rs:634-730) into a single tunable
king-danger accumulator, seeded to reproduce today's output exactly, every new
sub-term seeded **0** so bench is unchanged. New `EvalParams` inputs:
- **Attacker units** — keep the per-type weights (N/B=2, R=3, Q=5) as tunable
  seeds feeding the danger sum.
- **King-ring weak squares** — ring squares in `attacked[them]` not in
  `attacked[us]`, or in `attacked2[them]` while defended only once. Seed 0.
- **Safe checks per piece type** — squares from which N/B/R/Q could give check
  that are attacked by `them` and not defended by `us` (attack maps from 3.0 +
  the enemy king's check rays). One weight per piece type, seed 0.
- **Queen-relief** — a danger *reduction* when the attacking side has no queen
  (the SF "no-queen" term). Seed 0.
- **King-flank attack/defense** — count enemy attacks vs our defenses on the
  king's flank (the SF `flank` danger input), and a **pawnless-flank penalty**
  when the king sits on a flank with no friendly pawns. Seed 0.
- **Blockers/pins near the king** — pieces pinned or blocking in front of our
  king do not really defend it; fold a danger contribution keyed off the pin
  masks (compute the pin masks in the 3.0 substrate while slider info is hot, the
  same masks 3.7's mobility-area refinement will reuse). Seed 0.
- **Conversion table** — replace the hard-capped `SAFETY[16]` (max 118) with a
  longer tunable `safety[N]` table (N≈40): seed entries `0..15` to today's
  values and `16..N-1` to `SAFETY[15]=118`, and **remove the `.min(15)` cap** —
  identical for current positions (units rarely exceed 15), and the tuner can
  shape the high-unit tail into the danger² curve strong engines use. Folding
  shelter/storm into the danger sum is optional; if done, seed for identical
  output.

**Gate:** bench `5,446,782` unchanged; reconstruction exact. **Model: Opus 4.8
high** — densest interacting term in the eval and the highest-leverage
correctness target; pairs with 3.6 (same substrate, same reviewer).

#### 3.6 Threats package — structure (behaviour-identical) — **DONE 2026-06-22**

Added to `eval_piece_activity` (per-color loop), all weights seeded **0** (bench
`5,446,782` unchanged, reconstruction exact), all linearly traced so Phase 4.2
can Texel-tune them: **threat_by_minor / threat_by_rook** (per victim type, mg/eg
arrays of 6); **threat_hanging_refined** (per victim, the generalized weak-piece
condition — enemy piece we attack that is undefended, or doubly-attacked while
defended only once; the old flat hanging penalty stays active and is dropped in
the Phase-4 tuned step); **threat_safe_pawn_push** (enemy non-pawn pieces a pawn
attacks after a safe single/double push); **threat_weak_piece** (our piece
attacked by a strictly lower-valued enemy piece); **threat_restricted** (squares
both sides attack that the enemy does not strongly protect). All added to the
tuner's `threats` group. **Overloaded defender deferred** (the plan marks it
optional/defer): it needs sole-defender bookkeeping that doesn't fall out cheaply
from these loops and is invisible to the bench/reconstruction gates while
seeded 0, so it is better added when actually tuned. **Model used: Opus 4.8.**

Keep the existing pawn-threat term and flat hanging penalty active; seed all
new threat fields **0**:
- `threat_by_minor[victim_pt]`, `threat_by_rook[victim_pt]` (mg/eg) — a `them`
  piece attacked by our minor / rook, indexed by victim type.
- **Hanging refinement** — `them` piece in `attacked[us]` that is undefended, or
  in `attacked2[us]` while defended only once (generalises the flat term; seed
  the refined weight 0, leave the old flat term until Phase 4 tunes the
  replacement, then drop the flat term in that same tuned step).
- **Threat by safe pawn push** — squares our pawns would attack after one push
  to a square not attacked by an enemy pawn. Seed 0.
- **Weak-queen / weak-piece** — our higher-value piece attacked by a
  lower-value enemy piece. Seed 0.
- **Restricted squares** — enemy moves into squares we attack and cannot safely
  defend. Seed 0.
- *(optional, higher-complexity)* **Overloaded defender** — an enemy piece that
  is the *sole* defender of a more valuable attacked target (or is pinned and so
  cannot leave): a static-tactical term most strong HCEs carry but SF-classical
  expresses only indirectly. Needs the attacker/defender counts from the 3.0
  substrate to identify "defended exactly once and that defender is itself
  needed elsewhere." Add only if it falls out cheaply from the threat loops
  already built here; seed 0, tuned in 4.2. Lower-priority — defer if it
  complicates the threats pass.

**Gate:** bench unchanged; reconstruction exact. **Model: Opus 4.8 high.**

#### 3.7 Per-count mobility tables (behaviour-identical) — **DONE 2026-06-22**

Replaced the linear `mob_mg[4]`/`mob_eg[4]` × count with one-hot per-count
tables `mob_n_{mg,eg}[9]`, `mob_b_{mg,eg}[14]`, `mob_r_{mg,eg}[15]`,
`mob_q_{mg,eg}[28]`, seeded `table[i] = i · old_weight` via a `mob_seed::<N>()`
const fn (old weights N/B/R/Q = 4/5/2/1 mg, 4/5/4/2 eg). The eval loop indexes
the table by the safe-mobility count (clamped to table length) and traces
one-hot; `mobility_index` removed. Mobility *area* (`safe & !own_occ`) left
exactly as today (the area refinement is a Phase-5 behaviour change). `bench
5,446,782` unchanged, reconstruction exact. Tuner `mobility` group updated to
the 8 tables, clamped `[-150,400]` and enforced non-decreasing in the count.
**Model used: Opus 4.8.**

Replace linear `mobility × weight` (eval.rs:557-566, 878-898) with one-hot
tables `mob_n[9] mob_b[14] mob_r[15] mob_q[28]` (mg/eg each), **seeded
`table[i] = i · old_weight`** so the eval is identical until tuned. Leave the
mobility *area* exactly as today (`safe & !own_occ`); the area refinement
(exclude own king/queen squares and own blocked pawns) is a behaviour change and
is a tuned/tested micro-step in **Phase 5** (it interacts with the fit — defer).
**Gate:** bench unchanged. **Model: Opus 4.8 medium** (or GPT-5.5 high) — large
table count, careful seeding.

#### 3.8 Pawn structure + passed-pawn detail — structure (behaviour-identical) — **DONE 2026-06-22**

Implemented (bench `5,446,782` unchanged, reconstruction exact, all traced /
added to the tuner `pawnstruct`+`passers` groups): **rank-scaled connected** —
`pawn_connected_{mg,eg}` became per-rel-rank `[8]` tables seeded constant (7,5),
indexed by the pawn's relative rank (identical until tuned); **pawn levers**
(`pawn_lever_{mg,eg}`, our pawn attacking an enemy pawn) and **doubled-isolated**
(`pawn_doubled_isolated_{mg,eg}`, extra penalty when both) in the pawn cache,
seeded 0; **blocked passer** (`blocked_passer_{mg,eg}`, our passer blocked by an
enemy piece on its stop square) and **ideal blockader** (`ideal_blockader_{mg,eg}`,
our knight directly in front of an enemy passer) in a new `eval_passer_blockade`
in the piece-activity pass (they need piece squares, so outside `PawnEntry`),
seeded 0. **Deferred** (documented): the **candidate/majority-breakthrough**
term (overlaps the existing `passed_candidate` term and is fiddly) and the full
**promotion-path passer-safety** SF-classical shape (large, high bug-risk while
seeded 0 and invisible to the bench/reconstruction gates) — both are better
added in the focused Phase-4.4 passer tuning step when they are actually
exercised. **Model used: Opus 4.8.**

Hoist the existing pawn terms (doubled/isolated/backward/connected,
eval.rs:457-482) and passed-pawn extras (417-455) into `EvalParams`, then add,
seeded inert:
- **Rank-scaled connected/phalanx** — replace the flat `(7,5)` connected bonus
  with a per-rank table seeded constant `(7,5)`; Phase 4 lets it grow by rank
  (strong engines scale this sharply).
- **Blocked passer** (enemy piece on the stop square), **pawn levers**, and
  **doubled-isolated** — new fields seeded 0.
- **Candidate passer / majority breakthrough** — a pawn that becomes passed in
  1–2 pawn moves (own majority on a file group with no/fewer enemy pawns ahead),
  *distinct* from already-passed and from the unstoppable rule-of-square logic
  (3.12). A small mg/eg bonus that the rank-scaled passer table doesn't capture
  because the pawn isn't passed yet. Seed 0; tuned with the passer scalars in
  4.4. (The "pawn levers" field above is the move-creating-a-lever signal; this
  is the resulting-majority signal — keep both, they fit independently.)
- **Promotion-path passer safety** — upgrade today's single "safe stop square"
  test to the SF-classical `passed` shape: score the attacked/defended status of
  the squares on the pawn's *path to promotion*, whether the immediate block
  square is defended, friendly/enemy king distance to the block and next squares,
  and rook/queen support behind the passer. Needs the attack maps (3.0) and piece
  squares, so it lives in the piece-activity pass, not `PawnEntry`. Seed 0; tuned
  with passers in 4.4. (Distinct from the unstoppable rule-of-square term in 3.12,
  which is a binary near-win; this grades how *safe* a still-stoppable passer is.)
- **Rook behind passed pawn (Tarrasch) + ideal blockader** — eg-weighted: a bonus
  when our rook is on the passer's file *behind* it (own passer → supports the
  push; enemy passer → restrains it), and a small bonus for a **knight on the
  square directly in front of an enemy passer** (the Nimzowitsch ideal
  blockader). Both are passer/endgame terms the rank table misses; this needs the
  piece squares so it lives outside the pawn-only cache (compute in the
  piece-activity pass, not `PawnEntry`). Seed 0; tuned with passers in 4.4 and
  folded into the 3.11 endgame scaling where relevant.
This extends `PawnEntry` (eval.rs:273-280); keep the pawn cache correct and make
sure the `texel` feature bypasses it (§11 Risks). **Gate:** bench unchanged.
**Model: Opus 4.8 medium.**

#### 3.9 Material imbalance hooks (behaviour-identical; optional block) — **DONE 2026-06-22**

Added `eval_imbalance` (called from `evaluate` after `eval_piece_activity`):
two 6×6 coefficient matrices `imbalance_ours[36]` / `imbalance_theirs[36]`
(indexed `pt1*6+pt2` over `[bishop_pair, pawn, knight, bishop, rook, queen]`,
lower triangle `pt2<=pt1` used), the SF-style symmetric quadratic form over
per-color count products, **all coefficients seeded 0** (bench `5,446,782`
unchanged, reconstruction exact). The redundancy terms (rook-pair,
knight-with-pawns, rook-with-pawns) are entries of these matrices
(`ours[rook][rook]`, `ours[knight][pawn]`, `ours[rook][pawn]`), not separate
fields, matching SF. The bishop-pair scalar stays active as today (folded into
the imbalance fit in Phase 4.5). **Deviation from SF, deliberate:** no `/16`
divisor — the value is `Σ coeff·countproduct` added phase-independently to mg
and eg, so the term is *exactly linear* in the coefficients and cleanly
Texel-traceable (count = net white−black count product, recorded in both mg and
eg). Since coefficients are seeded 0 and tuned from scratch, SF's `/16`
integer-scaling convention is unnecessary; the scale is the tuner's to find. New
tuner `imbalance` group (72 params); coefficients clamped `[-300,300]` (signed).
**Model used: Opus 4.8.**

#### 3.10 Small positional terms — structure (behaviour-identical, batch) — **DONE 2026-06-22**

Added, all seeded 0 (`bench 13 = 5,446,782` unchanged, reconstruction exact
against `tools/texel/data/holdout.csv`):
- **Bishop-pair-scaled-by-pawn-count** (`bishop_pair_pawn_mg/eg`) — additive on
  top of the existing flat `bishop_pair_{mg,eg}` bonus, weighted by `8 −
  total_pawns`. The flat term stays active; this is tuned alongside it and can
  absorb it once Phase 4 fits both.
- **Bishop outposts** (`bishop_outpost_mg/eg`) — mirrors the knight-outpost
  logic exactly (same rank/attack/defend conditions), applied per bishop.
- **Trapped rook by own uncastled king** (`rook_trapped_mg/eg`) — fires when
  the king is still on its home square with all castling rights for that side
  gone, the rook sits in its starting corner (a1/h1/a8/h8), and the rook's raw
  attack-minus-own-occupancy mobility is `≤ 3`.
- **Connected rooks** (`rook_connected_mg/eg`) — both own rooks aligned on a
  rank or file with nothing (`movegen::between`) between them.
- **Bishop on a long diagonal bearing on the enemy king**
  (`bishop_long_diagonal_mg/eg`) — bishop on one of the 12 non-corner squares
  of the two main diagonals (`LONG_DIAGONALS` bitboard) whose current attack
  set (blockers included) reaches the enemy king or a king-ring square.
- **Bad bishop** (`bad_bishop_mg/eg`) — penalty scaled by the count of own
  pawns on the bishop's own square colour (light/dark via
  `Bitboard::LIGHT_SQUARES`/`DARK_SQUARES`).
- **Initiative/complexity** (`initiative_weight`) — single coefficient times a
  complexity proxy (`total_pawns + king-file distance + both-flanks-pawns
  indicator`), signed by the current eg score's sign (mirrors SF's
  `Initiative`, simplified to one linear weight since it is seeded 0 and fit
  from scratch — no SF-style internal sub-tuning needed).

Both optional non-SF-classical terms were included too (cheap, a few lines
each):
- **Closedness / locked-centre** (`closedness_knight_mg`, `closedness_rook_mg`)
  — `rammed_pawn_count × own_piece_count × weight`, mg-only, one signed weight
  per piece type (knight expected positive, rook expected negative once
  tuned). Deliberately a single weight, not a table — 3.7's per-count mobility
  already prices a knight's reduced mobility in closed positions, so the only
  marginal lever left is the material-value swing itself.
- **Central-king / lost-castling danger** (`king_centrality_danger_mg`) — fires
  when the king is on its home square with all castling rights for that
  colour gone (same predicate as the trapped-rook check above, reused).

New tuner group `smallpos` (16 params) in `tools/texel-tuner`; clamps added
(bishop-pair-pawn ±20, most bonuses [0,200], initiative [0,30], closedness
±30 signed, king-centrality-danger [0,100]). All tuned in Phase 4.4/4.5.
**Model used: Sonnet 4.6.**

#### 3.11 Scale-factor framework + endgame knowledge — **DONE 2026-06-22 (3.11a framework+KBNK, 3.11b KPK+KBP, 3.11c KQKP/KRKP/OCB)**

Introduce a `ScaleFactor` (0–64, `NORMAL = 64`) applied to the endgame side of
the tapered score before the 50-move damping. Seed it to `NORMAL` everywhere
**except** the specialised functions below, which fire only on exact material
patterns absent from the bench suite (so bench stays `5,446,782`):
- generalise the OCB scaling (eval.rs:912-928) into the framework, scaled by
  passed-pawn count;
- rook-endgame drawishness; "extra material but no pawns and insufficient mating
  material → draw"; exact **KPK** handling (a compact bitbase or an opposition /
  rule-of-the-square evaluator) so KPK is scored correctly without tablebases and
  Syzygy stays optional; KQKP / KRKP / KQKR heuristics; KBP
  wrong-bishop-wrong-corner draw;
- a **correct KX K mop-up incl. KBNK driving the bare king to the
  bishop-coloured corner** — the generic corner-drive (eval.rs:599-616) cannot
  win KBNK reliably. Keep the generic mop-up for the general case; add the
  KBNK-correct corner only for the KBNK material pattern.
Each known-endgame function is a small, self-contained unit gated by a
**permanent endgame regression suite** (below), not SPRT — these positions are
too rare for self-play to measure. **Model:** framework + KBNK corner math
**Opus 4.8 high**; the individual endgame functions are well-bounded → **GPT-5.5
high** (or Sonnet 4.6 medium once the pattern exists). Buys Elo *and* removes
"threw away a won/drawn ending" losses that hurt against Critter-class opponents.

**Done in this pass (the Opus 4.8 high portion — framework + KBNK + suite):**
- **`ScaleFactor` framework** (`SCALE_NORMAL = 64`): `scale_endgame()` dispatches
  to `specialized_endgame_scale()` (returns a scale factor in `0..=64`; the
  tapered score is multiplied by `sf/64`) before falling through to the
  pre-existing OCB + KNNK handling. The OCB scaling keeps its **exact `/48`
  whole-score integer arithmetic** (folding it into a `/64` eg-side factor would
  change the integer result on OCB positions present in the bench tree, breaking
  the fingerprint) — so the OCB→framework *generalisation with passed-pawn
  scaling* is a behaviour-changing item deferred to **Phase 4**, not done here.
  `linear_delta_scale` (texel) mirrors the new factor for tuner fidelity.
- **Pawnless insufficient-material draws** via the framework: KK, KNK, KBK, and
  minor-vs-minor → forced dead draw (`sf = 0`). KBN-vs-K is deliberately *not*
  matched (it is a win). KNNK keeps its existing handling.
- **KBNK correct corner-drive**: for the exact K+B+N-vs-bare-K pattern, the
  mop-up drives the losing king to a corner the winning **bishop can actually
  reach** (its own colour) instead of the nearest corner. **Bug found & fixed
  while building this:** the corner constants must follow *this engine's* colour
  convention, in which `a1 ∈ LIGHT_SQUARES` — the light-reachable corners are
  a1/h8, the dark ones h1/a8. An earlier draft had them inverted (would have
  driven the king to the unmatable corner); the regression suite's
  difference-of-differences direction test catches that inversion.
- **Permanent endgame regression suite** (`tests/endgames.epd` +
  `tests/endgames.rs`): draws assert static eval `== 0`; the KBNK position is
  played out at fixed depth and must reach **checkmate** (not stalemate) by the
  bishop+knight side within a move budget; plus a confound-cancelling static
  test that the corner drive points to the bishop's colour. `cargo test` clean.

**Done in 3.11b (per-EG functions, this pass — Opus 4.8):**
- **Exact KPK** via a generated **bitbase** (`src/kpk.rs`, the standard
  Stockfish iterative/retrograde classification; pawn normalised to White,
  Black-pawn positions mirrored vertically). Plugs into
  `specialized_endgame_scale()`: a drawn KPK is forced to `0`, a won one falls
  through (`None`) so normal eval scores it. Unit tests cover key-square wins
  (both sides to move), the opposition draw, and the rook-pawn corner draw.
  **This moved the bench fingerprint** `5,446,782 → 5,354,975` — KPK is
  reachable in the bench tree (see the re-baseline note in the checkpoint); the
  fingerprint was re-baselined with the user's sign-off.
- **KBP wrong-coloured-bishop rook-pawn draw** (`kbp_wrong_corner_draw`): strong
  side has K + one bishop + pawns all on a single rook file, the bishop is the
  wrong colour to guard the queening square, and the bare defending king holds
  that corner → dead draw (`0`). Does **not** fire in the bench suite.
- Regression suite extended with KPK draw/win and KBP-draw positions.

**Done in 3.11c (lower-confidence endgame heuristics — Opus 4.8, bench-neutral
at `5,354,975`, reconstruction exact):**
- **KQKP fortress draw** (`kqkp_fortress_scale`): KQ-vs-KP is a fortress draw
  **only for rook/bishop pawns** (a/c/f/h) on the 7th with the defending king
  guarding the queening square and the queen's king far → `Some(0)`. Knight and
  centre pawns are **wins** and return `None` (not scaled). Encoded as a
  correctness test, not a tautological constant.
- **KRKP drawish** (`krkp_drawish_scale`): a conservative **partial** scale
  (×0.25) in the clear draw zone (pawn on the 7th escorted by its king, rook's
  king distance > 4). Never a forced draw, so a wrong guess cannot zero out an
  actually-won KRKP.
- **OCB passed-pawn refinement** (`opposite_bishop_scale`): the opposite-bishop
  draw-scaling is relaxed upward by passed-pawn count; passer-free positions
  keep the exact pre-3.11 `/48` value, so the fingerprint is unchanged.
- Regression suite extended with a KQKP-fortress draw EPD line and four
  correctness unit tests (incl. guards that knight-pawn KQKP and KQ-vs-KR stay
  clearly *winning*).

> **Deliberately NOT implemented (and why):** **KQ-vs-KR** is a *win*, so it is
> never scaled toward draw; and **broad rook-endgame drawishness** is left to
> Phase 4 as a **tunable** scale term rather than a hardcoded rule — a fixed
> blanket scale on common R+P-vs-R positions would mis-draw won endings. (A
> first GPT-5.5 attempt at 3.11c did exactly that: a broad rook scaler that
> collapsed `bench 13` ~29 % and scaled won rook endings toward draw; it was
> dropped and reimplemented narrowly here.)

**Model used (3.11a+3.11b+3.11c): Opus 4.8 high.**

**Permanent endgame regression suite (`tests/endgames.epd`).** Build this once
and keep it forever — it protects mating technique against any future eval/search
change, and is the gate for every 3.11 function. Format: one EPD per line with a
`bm`/`dm` (best move / draw-or-mate) or a `c0` comment giving the expected
verdict, e.g.:
```
8/8/8/8/8/3k4/8/3KB1N1 w - - 0 1 ; KBNK win, expect mate
8/8/8/4k3/8/8/4K3/4N1N1 w - - 0 1 ; KNNK vs bare king: draw (eval ~0)
8/8/8/8/8/4k3/4p3/4K3 b - - 0 1 ; KPK: known result by rule
```
Curate ~30–60 positions covering each function added (KBNK both corners, KPK
win/draw by the rule, KRKP, KQKP, KQKR, OCB-with-passers draws, rook-endgame
draws, the KBP wrong-corner draw, KNNK-vs-bare-king draw). A small Rust
integration test (`tests/endgames.rs`) loads the EPD and asserts: (a) static
eval sign/magnitude matches the verdict where applicable, and (b) for the mating
ones, a short fixed-depth search **actually delivers mate within the move budget
from the board** (drive a `go depth N` / `go movetime` loop to checkmate). Run it
in `cargo test`; it becomes part of the Phase 3 gate and CI thereafter. *If a
model cannot curate the FENs, the user can paste known textbook positions (KBNK,
Lucena/Philidor, KQ vs KP, etc.) — the test harness is the work, the positions
are public knowledge.*

#### 3.12 Gauntlet-driven eval additions (added 2026-06-19, after re-surveying SF-classical terms) — **core DONE 2026-06-22 (Opus 4.8)**

A fresh pass over the Stockfish-classical eval surfaced terms the §15 backlog
missed. Add each as inert-seeded structure (bench `5,354,975` unchanged) in the
step shown; they are tuned in the matching Phase-4 stage. **Models:** Opus 4.8
medium for the endgame/passer items, Sonnet 4.6 medium for the small terms.

**Done in this pass (all seeded 0, bench `5,354,975` unchanged, reconstruction
exact, new tuner `gauntlet` group of 10 params):**
- **Unstoppable passed pawn** (`unstoppable_passer_eg`, eg-only) — rule of the
  square: passed pawn with a clear path whose promotion the enemy king cannot
  reach in time (accounts for side-to-move and the 2nd-rank double-step).
- **Minor behind pawn** (`minor_behind_pawn_mg/eg`) — a knight/bishop with a
  friendly pawn directly in front of it.
- **Pawn islands** (`pawn_islands_mg/eg`, in the pawn cache) — penalty scaling
  with the count of maximal own-pawn groups on consecutive files.
- **Queen infiltration** (`queen_infiltration_mg/eg`) — our queen on relative
  rank ≥ 4 on a square no enemy pawn attacks.
- **King protector** (`king_protector_mg/eg`) — penalty proportional to the sum
  of each own minor's distance from our king.
- **Space-shape upgrade** (`space_piece_mg`) — SF-style space weighted by own
  piece count, added alongside the flat `space_weight` term (which stays active
  until Phase 4 retires it). *Simplification: the "extra credit for squares
  behind 2–3 pawns" nuance is omitted; the piece-count weighting is the main
  SF idea.*

**Deferred (note):** the optional low-yield trio (bishop x-ray on pawns,
rook+queen battery, slider-on-queen threat) and the **winnable/complexity
coupling** of the 3.10 initiative term with the 3.11 scale-factor framework —
the coupling is a cross-term design item better done with the data in hand, so
it is left for Phase 4.4/4.5. The **exposed-queen** penalty is folded into the
existing threats package rather than duplicated here.

- **Unstoppable passed pawn (rule of the square)** — a passed pawn the enemy king
  cannot catch (king outside the pawn's promotion square, accounting for
  side-to-move and the pawn's double-step). Worth a near-winning eg score; SF and
  every strong HCE has it. Add to **3.8 / 3.11** (eg-only, gated on a
  no-other-pieces-can-block check). *Highest-value item in this list.*
- **Minor behind pawn** (SF `MinorBehindPawn`): a knight/bishop directly shielded
  by a friendly pawn → small mg bonus. Add to **3.10**.
- **Pawn islands**: penalty scaling with the number of disconnected own-pawn
  groups. Add to **3.8** (pawn cache).
- **Space term upgrade**: replace the flat centre-files popcount (eval.rs:619-632)
  with the SF shape — safe squares behind own pawns in the centre, **weighted by
  own piece count**, with extra credit for squares behind 2–3 pawns. Add to
  **3.10**.
- **Queen infiltration / exposed queen** (SF `QueenInfiltration`): bonus for our
  queen safely deep in the enemy half; small penalty for a queen on a square the
  enemy attacks. Add to **3.10**.
- **King protector**: penalty proportional to each own minor's distance from our
  king (SF `KingProtector`) — pairs with king safety. Add to **3.10**.
- **Winnable/complexity coupling**: make the 3.10 initiative term and the 3.11
  scale-factor framework behave like SF's single `winnable` adjustment — scale
  the eg toward draw for opposite-coloured bishops / one-flank / few-pawn
  positions, and *up* for passed pawns on both flanks + king outflanking. Wire
  them together rather than as two independent terms.
- *(Optional, low yield)* bishop x-ray on pawns, rook+queen battery on a file,
  slider-on-queen threat — add to the **3.10** batch only if cheap.

**HCE source checklist — avoid SF-monoculture (do this term survey once, here).**
The §15 backlog and 3.12 were both derived almost entirely from
**Stockfish-classical**, which is why the terms this plan was missing (closedness,
central-king danger, overloaded defender — now folded into 3.6/3.10) are exactly
the *non-SF* ideas: they live in other strong HCEs, not SF. Before freezing the
Phase 3 term list, cross-check it against a small fixed panel of HCE evals and
pull in anything material that only one of them has:
- **Stockfish 11 / classical eval** (the SF terms — already the base here),
- **Ethereal** (last strong HCE-era release) — the cleanest tuned-HCE reference,
- **RubiChess** HCE-era / classical eval — independent term set,
- **one independent current HCE** of the user's choice (e.g. Igel-HCE, Lambergar)
  as a tiebreak / sanity source.
This is a *term-selection* checklist (what to build), distinct from the §10
*strength* ladder (who to play). One survey pass — not a recurring gate.

#### 3.13 Permanent endgame regression suite

The permanent endgame regression suite (`tests/endgames.epd` +
`tests/endgames.rs`) is described in detail in §3.11 above — it is the gate for
every specialised endgame function. Filed as its own step here so the numbering
matches `user_dev_guide.md`. **Status:** the harness plus the KBNK-mate playout,
the insufficient-material-draw cases, and the corner-direction test are **DONE**
(built with the 3.11 framework, Opus 4.8); extend it with KPK/KRKP/KQKP/OCB/
rook-draw lines as the 3.11 per-EG functions land. **Model:** Sonnet 4.6 medium.

#### 3.14 Eval-cache correctness fix — **DONE 2026-06-23 (Opus 4.8)**

> **Numbering note:** `3.12` is the gauntlet-driven eval-term survey and `3.13`
> is the endgame regression suite, so this fix is filed as **3.14**. It is still
> inside Phase 3 and therefore still gated to complete **before Phase 4** begins,
> as required.

**Root cause (found & fixed).** Not a hash-key problem in `eval_table` — the
`(board.hash, halfmove_clock)` key is complete. The impurity was in the **pawn
cache**: the passed-pawn **free-stop / safe-stop** bonuses depend on non-pawn
occupancy (`board.occupied()`) and enemy attacks (`attackers_to_color`), yet
they were scored *inside* `eval_pawns`, whose `(mg, eg)` result is cached by a
**pawn-structure-only** key (`pawn_key`). So when pieces moved but pawns did
not, the pawn cache (and the whole-eval cache built on its output) returned
stale values, and the engine played a different eval than a cold recompute — at
the current head, `bench 13` was `5,354,975` cached vs `4,978,006` cold (a clean
re-measure of the original ~10 % symptom).

**Fix.** Move the free-stop / safe-stop scoring out of `eval_pawns` into a new
`eval_passed_pawn_advance`, run every evaluation (outside any cache), **before**
`eval_piece_activity` so the running `mg`/`eg` seen by downstream nonlinear terms
(notably the mop-up's `(mg+eg)/2` test) is byte-identical to the pre-fix
ordering. Verified the **eval value at every position is unchanged** (a
fresh-evaluator walk over hundreds of positions shows 0 diffs vs the pre-fix
code); only the cache is now exact. The whole eval is a pure function of the
position: `bench 13` is now identical with the cache active or bypassed
(**`4,978,006`**), re-baselined from `5,354,975`. Permanent guard:
`tests/eval_cache.rs` walks positions through a long-lived evaluator and asserts
every result equals a fresh (cold) evaluator — it fails on the pre-fix code and
passes after. Reconstruction stays exact (the trace simply moved with the term).

**Symptom (measured).** Toggling the whole-eval cache (`Evaluator::eval_table`,
`EVAL_TABLE_SIZE = 32768`, keyed by `(board.hash, halfmove_clock)`) shifts the
fixed-depth `bench 13` node count by ~10 %: **5,446,782** with the cache active
vs **4,924,891** with cache reads bypassed. The probe preserved
`-C target-cpu=native` in both builds, so this is **not** an FMA/`f64`
LMR-table rounding artifact. For a *pure* `evaluate()` behind an *exact*
`(hash, halfmove_clock)` key, a cache hit must equal a cold recompute — so a
different search tree means the cache is returning values that differ from a
fresh evaluation. This is **pre-existing** (the cache predates Phase 3) and does
not affect any shipped Phase-3 fingerprint (every diff is fingerprint-*stable*),
but it means the engine plays a subtly different eval than the one the Texel
tuner fits, so it must be root-caused before Phase 4 spends games and a fit on
the wrong target.

**Why it must precede Phase 4.** The tuner traces and fits the *cold* `evaluate()`
(the `texel` feature bypasses both caches). If the cache the engine actually
plays with diverges from that cold eval, then (a) the fit optimises a function
the engine never exactly runs, and (b) it likely costs Elo today. Phase 3 only
needs fingerprint *stability*, which holds — but Phase 4 needs the played eval
and the fitted eval to be the **same function**.

**Hypotheses to check, cheapest first (root-cause, do not paper over):**
1. **Key incompleteness** — the most likely cause. `board.hash` (and/or the key
   pair) may not capture every input that `evaluate()` actually depends on, so
   two positions that evaluate differently collide on the same key. Audit that
   `evaluate()` reads *nothing* outside what `(board.hash, halfmove_clock)`
   uniquely determines (e.g. side-to-move folded into hash, castling/ep state,
   any `Evaluator` scratch state that should be per-position). Mirror against the
   Zobrist update sites.
2. **Stale / cross-contaminated entry** — an `Evaluator`-local field written
   during one position's eval and read on a later cache *hit* for another (state
   that should be recomputed but is instead carried over with the cached score).
3. **Pawn-cache interaction** — bisect `eval_table` vs `pawn_table` (`pawn_key()`):
   bypass each independently to attribute the 10 % to one cache. If it's the pawn
   cache, the same key-completeness audit applies to `pawn_key()`.
4. **Collision policy** — confirm the table stores/compares enough of the key to
   reject collisions (full key vs truncated index), and that replacement can't
   serve a wrong-key entry.

**Acceptance for the fix.** After the fix, bypassing vs using the eval cache must
yield the **identical** `bench 13` node count (whatever value that is — the fix
may legitimately move the shipping fingerprint, since it changes the played
eval). Add a **deterministic `cargo test`** that, for a few thousand positions,
asserts `cached_eval(pos) == cold_eval(pos)` integer-for-integer — this is the
permanent guard that the cache is a true memoisation. Re-baseline the canonical
`bench 13` fingerprint in this doc and `user_dev_guide.md` to the post-fix value
and note the change explicitly (it is the one *intended* fingerprint move in
Phase 3). No SPRT/long games for the root-cause itself; a non-regression SPRT can
be folded into the Phase 3 gate if the fingerprint moves.

**Model:** **Opus 4.8 high.** This is a subtle correctness/determinism bug
(hash-key completeness, cache-state lifetime) where a wrong "fix" silently moves
the fingerprint without addressing the cause — it needs careful source-level
root-causing, not a mechanical edit. Background-investigation repro chip already
exists (`task_c2666035`).

#### 3.15 Eval hot-loop cleanup / inert-block gating — **INVESTIGATED & REJECTED for Rarog 2026-06-23**

> **Why (measured, 2026-06-23).** The Phase-3 gate NPS SPRT vs the Phase-2.5
> head failed: **−32.6 ± 9.3 Elo, H0 accepted** (2694 games, `3+0.03`). Root
> cause is **NPS, not correctness**: `bench 13` NPS dropped **3,429,963 →
> 2,669,172 (−22 %)** because the enlarged eval computes every node even though
> every new weight is seeded 0; the bench *fingerprint* (node count) is blind to
> NPS, so it surfaced only at the gate.

**Two things were tried and both rejected:**
1. **Hot-loop micro-opt** (reuse the substrate, collapse the imbalance loops,
   trim sweeps) — profiling (an eval-throughput micro-bench) showed the loops are
   **already compiler-optimised**; a hand "collapse imbalance" attempt *regressed*
   it (307 → 324 ns/eval). There is no redundant *recomputation* to remove; the
   cost is genuine work.
2. **Inert-block gating** (skip a fully-seeded-0 block in production via an
   `EvalActive` flag) — this *did* recover +15 % NPS byte-identically, but it
   **only helps the transient seeded-0 head**: once Phase 4 tunes the weights
   nonzero, every block reactivates and the gating does **nothing** in the
   shipping engine. It is pure scaffolding for a gate we don't actually care
   about, so it was **reverted** — we optimise the *resulting* function, not the
   seeded-0 checkpoint. (Commit `66c7ba9`, dropped.)

**Conclusion.** There is no behaviour-identical NPS to recover here. The durable
lever is **3.16 lazy eval**, which speeds up the *tuned* eval too. **Important
corollary:** the seeded-0 enlarged eval is *strictly* pure overhead in
non-decided positions (new terms compute, contribute 0), so the **gate NPS SPRT
vs the Phase-2.5 head can never pass at the seeded-0 head** — it can only be
beaten once Phase 4 activates the terms and adds Elo. The right validation for
3.16 is a **lazy-on vs lazy-off** non-regression SPRT (does the approximation
cost Elo?), and the real vs-p25 comparison belongs at the Phase-4 boundary.

#### 3.16 Lazy eval (behaviour-changing, SPRT-gated) — **ACCEPTED 2026-06-23 (Opus 4.8)**

> **Status: ACCEPTED.** `LAZY_MARGIN = 600`; the mop-up is extracted to
> `apply_mop_up` and runs on **both** paths so mating technique (KBNK, KXK)
> survives a lazy skip (endgame suite still mates). Lazy is
> `#[cfg(not(feature = "texel"))]`, so the tuner fits the full eval; the eval
> stays pure (`cache==cold`) and reconstruction is exact. **`bench 13`
> re-baselined `4,978,006 → 5,315,678`**; per-node NPS **~2.50M → ~2.80M
> (+11.8 %)**, durable on the tuned eval.
>
> **SPRT (lazy-on vs lazy-off, `3+0.03`, 15,314 games): +4.42 ± 3.90 Elo, LOS
> 98.7 %, LLR 2.95 → H1 accepted `[-3,0]`.** Lazy is a *net gain*, not just a
> free speedup — the deeper search from +11.8 % NPS outweighs the coarser eval
> (and at seeded-0 the skipped terms contribute ~0). The margin can be re-checked
> after Phase 4 grows the positional weights.

The big lever. When the **tapered material + PST margin** (the cheap part of the
eval, already computed) is so large that no positional term could flip the
bound, **return early** and skip the expensive king-safety / threats / mobility /
imbalance / small-terms block (SF-style lazy eval). Material/PST are **not**
seeded 0, so the margin is meaningful and stable today.

Key constraints:
- **Disabled under `--features texel`** — the tuner must trace/fit the *full*
  eval, so the lazy early-return is `#[cfg(not(feature = "texel"))]`. The eval
  stays a pure function of the position (lazy is deterministic), so
  `tests/eval_cache.rs` (cache == cold) still holds and the cache stores the
  lazy value.
- **Conservative margin.** Too tight → wrong skips cost Elo; too loose → little
  NPS back. Start wide (skip only when |material+PST tapered| ≫ the largest
  plausible positional swing), measure, then tighten under SPRT.
- Changes the played eval for clearly-decided positions → **`bench 13`
  fingerprint moves** (re-baseline, documented).

**Gate:** (a) a **non-regression SPRT** `[-3,0]` of the lazy head vs the
pre-lazy (3.15) head — lazy must not lose Elo from over-eager skips; **and**
(b) re-run the **Phase-3 gate NPS SPRT `[-3,0]` vs the Phase-2.5 head** — it must
now **pass H1** (the 22 % NPS hole closed). Re-baseline the fingerprint, update
both docs. **Model: Opus 4.8 high** (search-sensitive margin; a wrong bound
silently weakens play).

> **Sequencing.** 3.15 (free, bench-identical) → 3.16 (behaviour-changing,
> SPRT-gated) → re-run the gate NPS SPRT. Only then is Phase 3 closed and Phase 4
> may begin. Doing this before Phase 4 keeps the per-stage tuning SPRTs from
> having to climb out of a 22 % NPS hole.

### Phase 3 gate (end of phase)

- `bench 13 == 5,315,678` (re-baselined three times in Phase 3: `5,446,782` →
  `5,354,975` at 3.11b for the **KPK bitbase**; → `4,978,006` at 3.14 for the
  **eval-cache fix**; → `5,315,678` at 3.16 for **lazy eval**). The whole eval is
  a pure function of the position (`tests/eval_cache.rs` guards cache == cold,
  and lazy is deterministic).
- **Reconstruction acceptance test passes:** for 10,000 random dataset positions
  the tuner's reconstructed `E(default)` equals `evaluate()`
  **integer-for-integer** with the full enlarged trace. Any mismatch is a trace
  bug — fix before Phase 4.
- **Per-term trace/eval regression assertions — DONE 2026-06-23.** (a) eval
  **colour symmetry** (`tests/eval_invariants.rs::eval_is_colour_symmetric` —
  `evaluate()` is invariant under a colour-flip + vertical mirror across 200+
  positions); (b) **seeded-zero inert-but-tunable**
  (`src/eval.rs::seeded_zero_terms_are_inert_but_tunable`, `--features texel` —
  a seeded-0 term fires with a nonzero trace count yet contributes 0 at the
  default weights, and perturbing the weight changes the reconstructed eval, so
  it is wired/tunable); (c) **nonzero activation**
  (`new_terms_activate_on_curated_positions` — curated positions trigger the
  passer/free-stop/safe-stop, mobility, imbalance, threat-by-minor, and
  minor-behind-pawn terms). This is what makes the Step 4.0 feature-support
  diagnostics trustworthy.
- **Eval-cache correctness (3.14) resolved:** bypassing vs using the eval cache
  yields the identical `bench 13` node count, the cache==cold-eval `cargo test`
  guard passes, and the canonical fingerprint is re-baselined to the post-fix
  value here and in `user_dev_guide.md`. This is a hard gate — Phase 4 must not
  start while the played eval and the fitted eval differ.
- `cargo test` clean (incl. the new endgame unit tests).
- ~~One non-regression / NPS SPRT `[-3,0]` vs the Phase 2.5 head~~ **— superseded.**
  This **FAILED (2026-06-23: −32.6 ± 9.3 Elo, −22 % NPS)** and **cannot pass at the
  seeded-0 head**: the new terms are pure overhead until Phase 4 tunes them, so a
  seeded-0 enlarged eval is strictly slower than the 2.5 eval for the same quality
  (no byte-identical optimisation can fix that — see 3.15). **Replacement gate:** a
  **lazy-on (3.16) vs lazy-off non-regression SPRT** (does lazy cost Elo at equal
  eval?). The real vs-`p25` comparison moves to the **Phase-4 boundary**, where the
  tuned eval + lazy NPS beats `p25`.
- **Eval-cost budget (whole enlarged eval).** The new terms are seeded 0 but
  their *structure still computes* (attack-map reads, threat/passer loops run,
  then multiply by 0), so the full eval cost is already paid at the Phase 3 head
  — measure it. Compare fixed-depth wall-time / NPS of the Phase 3 head vs the
  Phase 2.5 head (best-of ≥5 `bench`, native+pext). If the enlarged eval costs
  **>10–15 % NPS** beyond what 3.0's attack-map substrate saved, treat it as a
  defect to address *before* Phase 4 spends games on it: profile (`cargo asm`
  for surviving checks, `samply`/flamegraph for hotspots) and apply **lazy eval**
  (skip the expensive king-safety / threats / mobility block when the tapered
  material+PST margin already exceeds a safe bound) or hot-loop cleanup. *Rarog
  already has the whole-eval cache and pawn cache (§15), so "add an eval cache"
  is done — the lever here is lazy eval + structural cleanup, not caching.*
  Under-budget → carry on; only pay the lazy-eval complexity if the budget is
  breached.

---

## 8. Phase 4 — Eval data-fit campaign (staged Texel)

**Goal:** activate the entire Phase-3 structure by fitting it to data in **one**
staged campaign. The tuner, trace, dataset, and reconstruction test all exist
from Phase 3. Texel loss is never the verdict — the game test is (§2 Test-TC
note). Bake each accepted stage into `EvalParams` defaults, record `bench 13`,
SPRT vs the previous accepted head at the unified **`tc=3+0.03`**; confirm the
whole-phase gain at LTC `tc=10+0.1` at the boundary.

**Method (linear-trace gradient tuner — the Ethereal / Andrew Grant approach,
not coordinate descent, not SPSA):** for a fixed position the eval is an affine
function of the weight vector (every term is `weight × count`; PSTs and the
safety table are one-hot lookups), so one cheap per-position trace lets the
tuner recompute eval + exact gradient without calling `evaluate()`. Objective:
`L(w) = mean( (result_i − σ(E_i(w)))² )`, `σ(x) = 1/(1+10^(−K·x/400))`, with
`E(w) = scale · ((mg(w)·phase + eg(w)·(24−phase))/24 + rest)` reconstructed from
the stored sparse counts; fit `K` once at defaults and freeze it; optimise with
**Adam** (lr ≈ 0.05, β₁=0.9, β₂=0.999, full-batch) to a holdout-loss plateau;
optional small L2 toward PeSTO for PST entries only. Frozen non-linear pieces
(50-move damping, scale factors, mate-drive, zone-construction *logic*) live in
the per-position `scale`/`rest` constants so reconstruction stays exact.

#### 4.0 Tuner/data readiness gate — bucketed holdout, feature support, constraints (set up before staging) — Sonnet 4.6 medium; Opus 4.8 high for tuner-core changes

The single global holdout-loss number can fall while a *critical* eval domain
silently regresses (king attacks, passers, pawn endings) — and because we fit
the whole enlarged eval **once**, that damage would survive into every later
phase. Before staging, partition the holdout into **buckets** and report loss
**per bucket** every fit, not just the aggregate:
- game phase (opening / middlegame / endgame, by `phase`),
- material class (e.g. no-queens, opposite-coloured bishops, rook endings, pawn
  endings),
- king-attack positions (enemy pieces in our king ring),
- positions with a passed pawn,
- quiet positions carrying a static threat (a `them` piece in `attacked[us]`).
Bucketing reuses the trace counts already stored per position (3.3), so it is a
reporting layer, not new instrumentation. A stage is only "clean" if it improves
or holds **every** bucket; a bucket that regresses while global loss drops is the
signal to investigate *before* the SPRT, not after a confusing H0.

**Bucketed reporter — built (2026-06-24).** Ten buckets: phase
(opening `≥16` / middlegame / endgame `<6`), material class (no-queens, OCB,
rook-ending, pawn-ending — from the board), and king-attack / passer / threat
(from the trace family activations already computed for the row). Two entry
points: `rarog-texel --buckets <data.csv>` prints a current-eval snapshot (the
baselines), and **every `--tune` now prints a base→final per-bucket table** with
a `<-- REGRESSED` flag on any bucket whose loss rose under the fit. Holdout
baselines (116k): opening **0.1598** (noisiest), middlegame 0.1096, endgame
**0.0825** (settled), no-queens 0.0921, OCB 0.0841, rook-ending **0.0790**,
pawn-ending **0.1230** (only 2.9k positions — thin, watch it), king-attack
0.1047, passer 0.1022, threat 0.1039 (aggregate 0.1019). Verified on a `passers`
test fit: every bucket improved, strongest in endgame (−0.00054) and passer
(−0.00041), no regressions.

**Nonlinear king-safety fit path — built & validated (2026-06-24).** The
feature-support audit proved the 11 danger-index inputs are invisible to the
linear gradient. `rarog-texel --tune-kingsafety <train> <holdout> [out]
[--epochs N] [--max-positions N]` is the dedicated path: it **re-evaluates** the
dataset with perturbed weights (new texel-gated `Evaluator::set_params` rebuilds
the derived tables), so it sees the table-bucket nonlinearity the trace cannot.
It co-tunes the 11 inputs + the 40-entry safety table (51 params) by integer
coordinate descent with a shrinking step (robust to the table's step-function
shape and the integer grid; `clamp_weights` keeps the table non-decreasing and
the inputs non-negative — the shape constraints from the gate table). A
6-epoch / 60k smoke fit moved the dead inputs off zero sensibly
(`ks_safe_check_queen +12`, `ks_safe_check_bishop +8`, `ks_pawnless_flank +16`),
pushed the table tail up into the danger² shape (danger 15–28: 118 → 158–194),
and improved **every** bucket (holdout −0.00073; sharpest in opening −0.00134,
king-attack −0.00073, passer/threat). This is the **Stage-4.1 fit engine**; the
real run (full data, step→1) + SPRT is Stage 4.1 itself. Production bench
unaffected (`set_params` is `#[cfg(feature="texel")]`; `bench` = 5,315,678).

**Targeted-data policy (companion rule).** If a bucket regresses or is noisy, do
**not** globally retune or globally regenerate. Generate/import *targeted* quiet
positions for that bucket only (e.g. more king-attack or pawn-ending FENs via a
filtered `sample_fens.py` pass) and append. Global regen stays reserved for a
stalled *aggregate* holdout (the existing 4.x rule). This keeps the eval fit
honest in the domains HCE strength actually depends on (king safety, passers,
endgames) without paying to refit everything.

**Tuner/data readiness items (clear before staging).** Strong HCE evals are won
as much by the **tuning process** as by term selection; the Phase-3 structure
only pays off if the linear-trace fitter can measure and constrain it. The
bucketed-holdout + targeted-data rules above are two rows of this gate; the rest:

| Item | Requirement | Why |
|---|---|---|
| Nonlinear king-safety support | Rarog's king-danger uses a **danger² conversion + a long `safety[N]` table**, which the pure linear-trace gradient cannot fit. Either feed the tuner *linearised* traced inputs, or add an Ethereal-style finite-difference / special path for the conversion divisor, unit weights, and safety-curve shape. (This is also what the optional 4.1b SPSA polish targets — but decide the tuner support **here**, not after a failed fit.) | The fitter cannot learn knobs whose coefficient is zero-seeded or depends on the safety-table index. Stage 4.1 must not pretend to tune untunable params. |
| Feature-support diagnostics | Count nonzero observations per parameter and per bucket; warn / freeze / merge very sparse params before fitting. | Stops rare HCE terms (endgame, queen-pressure, restricted squares) from learning random signs or giant values off a handful of positions. |
| Phase/domain-balanced sampling | `datagen.ps1`/`extract.py` enforce quotas (or sampling probabilities) for opening/mid/endgame, pawn endings, passers, quiet threats, and king attacks. | Waiting for self-play to *naturally* supply rare terms leaves the tuner underdetermined. |
| Blended labels | Optionally train on `α·result + (1−α)·score_target` (a search-score / WDL teacher target). Engine output stays pure HCE — teacher labels are training data only. | Result-only labels are noisy; blended targets smooth the gradient and shrink the dataset needed. |
| Binary feature cache | A versioned trace/feature cache (schema + params hash + bucket metadata + labels) so repeated staged fits don't rebuild traces. | Phase 4 reruns many fits; a cache makes them fast and reproducible. |
| Regularization / shape constraints | L2-to-prior beyond PSTs, monotonic/smooth passed-pawn and safety curves, sign constraints on obvious penalties, optional PST smoothing. | The §15/early scalar experience shows broad fits produce implausible signs; constraints make the fit robust instead of decorative. |

**Gate status (2026-06-24).** Nonlinear king-safety support ✅ (`--tune-kingsafety`),
feature-support ✅, bucketed-holdout ✅, regularization/shape ✅ (see below).
Phase/domain-balanced sampling and blended labels are **regen-dependent** — the
*capability* is in place but only takes effect on a new datagen pass over the
2.19M set, which is the user's call (the current set is already sufficient for
Stage 4.1: 75k/116k holdout positions carry a king attack). Specifically:
`extract.py` now computes game phase (faithful to engine `PHASE_W`), always
prints the train/holdout phase mix, and takes `--balance-phase R` to downsample
over-represented phase buckets to `R×` the smallest. Domain (king-attack /
passer / threat) balancing stays **post-hoc** via the bucketed reporter +
targeted `sample_fens.py` append — replicating the engine's real eval buckets in
python-chess would risk diverging from them. **Blended labels need no tuner
change**: `parse_target` already accepts any float in `[0,1]`, so a `fen;0.62`
blended target works as soon as datagen emits a WDL/search-score column. Binary feature cache **deprioritized**: measured trace-build load is
~1–2 s for the full set (Rust + parallel `thread::scope`), so a versioned cache
would save seconds, not the minutes its rationale assumed; revisit only if reruns
become a bottleneck.

**Regularization / shape — built (2026-06-24).** Shape constraints already live
in `clamp_weights` (non-decreasing safety table, non-decreasing passer bonuses,
non-negative penalties/king-danger inputs, pinned king material & pawn rank-1/8
PST). Added **L2-to-prior** to the linear tuner: `--tune … --l2 <λ>` adds
`2λ(w−prior)` to each active gradient, prior = the current hand-tuned default. It
is a *guard*, not a default win — on a well-supported group it pulls the fit back
toward prior (measured: `passers` holdout 0.0974→0.0980 at λ=2e-5), so it is
**off by default** (`λ=0`) and meant for sparse/suspect terms or a fit that shows
implausible signs. Usable scale is gentle (`λ≈1e-6…2e-5`; `λ≥8e-5` fully pins to
prior). Verify holdout doesn't regress when applying it.

A stage is only "clean" when the aggregate holdout **and** the relevant buckets
hold or improve, the feature-support diagnostics are sane, and the reconstruction
test (3.3) still passes exactly.

**Feature-support diagnostics — built & run (2026-06-23).** `rarog-texel
--feature-support <dataset.csv> [--max-positions N]` counts, per flat weight,
the positions whose **linear trace** gives it a nonzero tapered coefficient (the
positions that can supply gradient signal), with an opening/mid/endgame phase
breakdown and mean |signal|. It flags any weight active in fewer than
`max(200, 0.05%·N)` positions. Run on the full 2.19M train set, three classes of
sparse weight emerged:

1. **Structurally always-zero — freeze permanently, never tune:** king material
   (`mg_val[5]`/`eg_val[5]`), pawn PST on ranks 1/8 (`pst_*[0..7,56..63]`), and
   passer-on-rank-1/8 (`passed_*[0,7]`). These *cannot* occur; a fitter touching
   them is fitting noise.
2. **Nonlinear king-safety units — zero in the linear trace (the critical
   finding):** all 11 of `king_safety_unit_{minor,rook,queen}`, `ks_weak_ring`,
   `ks_safe_check_{knight,bishop,rook,queen}`, `ks_queen_relief`,
   `ks_flank_attack`, `ks_pawnless_flank` report **0 activations**. They enter
   eval through the danger²→`safety[N]` table lookup, so the linear gradient is
   structurally blind to them. **This confirms the "Nonlinear king-safety
   support" gate row above is a hard blocker, not a caveat:** Stage 4.1 must fit
   these via a finite-difference / SPSA path (perturb weight → re-eval → ΔMSE),
   *not* the linear trace, which would leave them frozen at their seed. Only the
   `king_safety_table[40]` itself is linearly traceable (≈34% activation).
3. **Genuinely rare even at 2.19M (freeze at hand value or merge):**
   `pawn_lever` (0.03%), `trapped_bishop` (0.03%), `rook_trapped` (0.05%) — ~700–
   1040 activations each, too few for a stable sign/magnitude. `threat_rook`
   (0.34%) and `threat_queen` (0.22%) clear the cut on the full set and are
   tunable, but watch their per-bucket loss.

153 weights total fall under the cut on the full train set (210 on the 116k
holdout — the extra ~57 are borderline terms the larger set rescues). The
freeze/SPSA split above is the Step-4.1 entry contract.

**Stages — biggest lever first to bank/de-risk early; PSTs + material last so
they balance against the complete term set (fit once, no redo):**

| Stage | Group unfrozen | Gate | Notes |
|---|---|---|---|
| 4.1 | King safety (safety table + attacker / safe-check / weak-ring / relief weights) | `[0,5]` | Biggest lever; the 3.5 structure comes alive here. **✅ ACCEPTED +42.5 Elo** (see below). |
| 4.2 | Threats (minor/rook tables, hanging refinement, pawn-push, weak-piece, restricted) | `[0,3]` | Drop the old flat hanging term in the accepted tuned step. **✅ ACCEPTED +45.2 Elo** (see below). |
| 4.3 | Mobility (per-count tables) | `[0,3]` | **✅ ACCEPTED +24.1 Elo** (see below). |
| 4.4 | Remaining scalars (pawn structure, passers, bishop pair, rook files/7th, outposts, space, tempo, small positional terms) | `[0,5]` | Mobility-area refinement is Phase 5, not here. **✅ ACCEPTED +85.2 Elo** (see below). |
| 4.5 | Material imbalance | `[0,3]` | Skip if 3.9 was skipped. **✅ ACCEPTED +26.7 Elo** (the OCB-regression bet paid off; see below). |
| 4.6 | Material + PSTs definitive refit (~780) | `[0,5]` | L2 toward PeSTO; full dataset. Biggest block, last, balanced against everything above. **✅ ACCEPTED +27.6 Elo** (see below). |
| 4.7 | Global polish — everything unfrozen, low lr | `[0,3]` | Stop here regardless of outcome. **✅ ACCEPTED +65.0 Elo** (see below). |

Per-stage sanity rule: wildly implausible values (flipped signs on
well-understood terms, non-monotonic passer/safety tables) indicate a
dataset/trace bug — fix before SPRT, not after. A failed SPRT after a *sane*
fit → revert the stage and continue; one retry only if a concrete defect was
found. The reconstruction acceptance test in step 3.3 must pass before any fit.

**Dataset:** self-play primary (step 3.4). Regenerate with the strength-improved
head between major stages only if holdout loss stalls; otherwise reuse.

**Stage 4.1 — king-safety fit ✅ ACCEPTED +42.5 Elo (2026-06-24).** SPRT
`KSafety41` vs `Phase3Lazy`: **+42.47 ± 13.45 Elo** (nElo +61.06 ± 19.14),
LOS 100%, LLR 2.95, H1 at 1266 games (W422 L268 D576, 56.1%). A huge stage —
king safety is the biggest lever, and the 11 nonlinear danger inputs were
literally **0/untunable by the linear trace** before, so activating them
properly banked outsized Elo. `rarog-phase41-ksafety-pext-pgo.exe` is the **new
head** and the Stage-4.2 baseline; canonical bench **5,178,378**. History below.

Ran
`rarog-texel --tune-kingsafety train holdout --epochs 80` on the full 2.19M set
(K=2.045; coordinate descent converged at step 1, epoch 63). **Holdout
0.10188570 → 0.10104598 (−0.00084), every bucket improved, no regressions**
(opening −0.00175, middlegame −0.00106, king-attack −0.00094, passer −0.00093,
threat −0.00088, rook-ending −0.00073, endgame −0.00045). Fitted values
(sane — dead inputs activated, table monotonic & danger²-shaped):
- `king_safety_unit_rook` 3→2; `ks_safe_check_knight` 0→2, `…_bishop` 0→4,
  `…_rook` 0→4, `…_queen` 0→16; `ks_queen_relief` 0→2; `ks_pawnless_flank` 0→12.
  (`ks_weak_ring`, `ks_flank_attack`, `king_safety_unit_minor/queen` unchanged.)
- `king_safety_table` tail lifted from the old flat 118 cap to 240–366.

Baked into `EvalParams::default()`; all 61 tests pass incl. trace
reconstruction. **Bench 5,315,678 → 5,178,378** (eval changed, so node count
moves — expected in Phase 4). Candidate binary
`tools/test_engines/rarog-phase41-ksafety-pext-pgo.exe`. **SPRT to run (user):**

```
./tools/sprt.ps1 `
  -EngineA "tools\test_engines\rarog-phase41-ksafety-pext-pgo.exe" `
  -EngineB "tools\test_engines\rarog-phase3-lazy-pext-pgo.exe" `
  -NameA "KSafety41" -NameB "Phase3Lazy"
```
(`gainer` mode, H0 ≤0 / H1 ≥5 nElo, `tc=3+0.03`.) On H1 → mark accepted, this
is the new head. On H0 → `git revert` the 4.1 bake (the fit path & tooling stay).

**Stage 4.2 — threats fit ✅ ACCEPTED +45.2 Elo (2026-06-24).** SPRT `Threats42`
vs `KSafety41`: **+45.22 ± 11.16 Elo** (nElo +61.94 ± 15.11), LOS 100%, LLR 2.95,
H1 at 2032 games (W673 L410 D949, 56.5%). Another large stage — the threat
package (per-victim tables, refined hanging, weak-piece, safe-pawn-push) was
almost entirely seeded 0, so activating it banked big. `rarog-phase42-threats-pext-pgo.exe`
is the **new head** and Stage-4.3 baseline; canonical bench **5,144,732**. Fit
details below.

Tuned the `threats`
group **plus the old flat hanging term jointly** (new `threats42` tuner group),
so the fit resolves their overlap instead of dropping the flat term blind. Linear
Adam, 500 epochs, full 2.19M set (K=2.017). **Holdout 0.10104 → 0.10004
(−0.00100), every bucket improved, no regressions** (opening −0.00223, threat
−0.00103, king-attack −0.00096, pawn-ending −0.00096). Notable: the joint fit
**drove the flat hanging penalty to ~0** (minor 45→0, rook 60→2, queen 80→2) — the
refined hanging term absorbed it, so the plan's "drop the old flat hanging term"
happened **data-driven**, not forced. Per-victim threat tables activated
(`threat_by_minor` 0→41–73, `threat_by_rook` 0→24–50, `threat_weak_piece` 0→26);
the three base threat scalars converged to a common (38, 25). Baked into defaults;
all tests pass (the `seeded_zero_terms_are_inert_but_tunable` gate switched its
threat example to `threat_safe_pawn_push_eg`, still seeded 0). **Bench 5,178,378
→ 5,144,732.** Candidate `rarog-phase42-threats-pext-pgo.exe`. **SPRT to run
(user):**

```
./tools/sprt.ps1 `
  -EngineA "tools\test_engines\rarog-phase42-threats-pext-pgo.exe" `
  -EngineB "tools\test_engines\rarog-phase41-ksafety-pext-pgo.exe" `
  -NameA "Threats42" -NameB "KSafety41" -Elo1 3
```
(`gainer`, H0 ≤0 / H1 ≥3 nElo per the stage's `[0,3]` gate, `tc=3+0.03`.) On H1 →
new head; on H0 → `git revert` the 4.2 bake.

**Stage 4.3 — mobility fit ✅ ACCEPTED +24.1 Elo (2026-06-24).** SPRT `Mobility43`
vs `Threats42`: **+24.07 ± 7.94 Elo** (nElo +33.97 ± 11.17), LOS 100%, LLR 2.96,
H1 at 3716 games (53.5%). Smaller than 4.1/4.2, as expected for the later
incremental stages — and it confirms the clean 250-epoch fit cost no meaningful
Elo (the rook-ending caution was free). `rarog-phase43-mobility-pext-pgo.exe` is
the **new head**; canonical bench **5,181,289**. Fit details below.

Tuned the `mobility`
group (mob_n/b/r/q mg+eg, 132 params) on the 4.2 head. **The bucketed reporter
earned its keep here:** a full 400-epoch fit improved global holdout to 0.09922
but **regressed the rook-ending bucket** (+0.00007) — overvaluing active rooks in
drawish rook endings. L2-to-prior couldn't fix it surgically (all-or-nothing
revert to prior). So I early-stopped at the **clean boundary (250 epochs)**: every
bucket holds or improves (rook-ending −0.0000133, pawn-ending flat), global
**0.10004 → 0.09936 (−0.00068)** — ~83% of the max gain without the drawish-ending
damage. Fitted curves are monotonic & SF-shaped (e.g. `mob_b_eg` −25 for a trapped
bishop → +71). `mob_seed` helper removed (now baked). All tests pass. **Bench
5,144,732 → 5,181,289.** Candidate `rarog-phase43-mobility-pext-pgo.exe`. **SPRT
to run (user):**

```
./tools/sprt.ps1 `
  -EngineA "tools\test_engines\rarog-phase43-mobility-pext-pgo.exe" `
  -EngineB "tools\test_engines\rarog-phase42-threats-pext-pgo.exe" `
  -NameA "Mobility43" -NameB "Threats42" -Elo1 3
```
(`gainer`, H0 ≤0 / H1 ≥3 nElo, `tc=3+0.03`.) On H1 → new head; on H0 → `git revert`
the 4.3 bake.

**Stage 4.4 — scalars fit ✅ ACCEPTED +85.2 Elo (2026-06-24).** SPRT `Scalars44`
vs `Mobility43`: **+85.20 ± 18.75 Elo** (nElo +123.80 ± 26.15), LOS 100%, LLR
2.95, H1 at just 678 games (62.0%) — the effect was so large the LLR hit the
bound almost immediately. The biggest stage of the campaign, matching its
record holdout gain. `rarog-phase44-scalars-pext-pgo.exe` is the **new head**;
canonical bench **5,121,269**. Fit details below.

Tuned the remaining
positional scalars (new `scalars44` tuner group = pawn structure, passers, rook
files/7th, minors, space/tempo, small terms, gauntlet additions — 93 params),
**excluding** mobility/threats/hanging (done in 4.2–4.3) and **freezing** the
three feature-support sparse pairs (`pawn_lever`, `trapped_bishop`, `rook_trapped`).
Full 2.19M set, 700 epochs (K=1.987). **Holdout 0.09933 → 0.09644 (−0.00289) —
the largest single-stage gain yet, every bucket improved, no regressions**
(pawn-ending −0.0062, opening −0.0038, passer −0.0034, king-attack −0.0028).
Expected: this group had the most still-seeded-0 terms (gauntlet additions,
passer detail, rook/minor positional terms). Values are sane; several terms
(`rook_7th`, `space_weight`, `king_protector`, `space_piece`) fitted to 0 — the
data's verdict that they add nothing atop mobility/threats/open-file. Baked; the
`seeded_zero` gate dropped its now-activated `minor_behind_pawn` example (keeps
imbalance + `threat_safe_pawn_push_eg`). All tests pass. **Bench 5,181,289 →
5,121,269.** Candidate `rarog-phase44-scalars-pext-pgo.exe`. **SPRT to run
(user):**

```
./tools/sprt.ps1 `
  -EngineA "tools\test_engines\rarog-phase44-scalars-pext-pgo.exe" `
  -EngineB "tools\test_engines\rarog-phase43-mobility-pext-pgo.exe" `
  -NameA "Scalars44" -NameB "Mobility43"
```
(`gainer`, gate `[0,5]` → H1 ≥5 nElo, `tc=3+0.03`.) On H1 → new head; on H0 →
`git revert` the 4.4 bake.

**Stage 4.5 — imbalance fit ✅ ACCEPTED +26.7 Elo (2026-06-24).** SPRT
`Imbalance45` vs `Scalars44`: **+26.66 ± 8.49 Elo** (nElo +36.77 ± 11.66), LOS
100%, LLR 2.95, H1 at 3408 games (53.8%). **The user-approved OCB-regression bet
paid off** — the gain (+26.7, right in the predicted +25–30 band) clearly
outweighed the small, already-scaled OCB holdout wobble. Confirms the lesson: a
single small bucket regression on an already-scaled domain can be worth shipping
when the global gain is meaningful and the SPRT validates it.
`rarog-phase45-imbalance-pext-pgo.exe` is the **new head**; canonical bench
**5,448,086**. Fit details below.

Tuned the SF-style
imbalance quadratic (`imbalance_ours`/`imbalance_theirs`, lower triangle, ~36
firing coeffs) on the 4.4 head. Full 2.19M set, 700 epochs (K=1.857). **Holdout
0.09640 → 0.09527 (−0.00113); 9/10 buckets improved strongly (pawn-ending
−0.0034, endgame −0.0016, no-queens −0.0013) — but the OCB bucket REGRESSED
(+0.00048).** This was a **deliberate, user-approved exception** to the
all-buckets-clean rule: no clean fit exists because OCB drawishness is
positional/scaling (already handled by `opposite_bishop_scale`) while imbalance
is a material quadratic — the regression appears at every epoch count (even 40)
and freezing the bishop coeffs only halved it while losing ⅔ the gain. Rationale
to ship anyway: the ~25–30 Elo (extrapolated) gain is meaningful, OCB is a small
(7.5k) bucket already scaled toward draw in play, and the SPRT is the arbiter
with a clean single-commit revert. All tests pass (the `seeded_zero` gate dropped
its now-activated `imbalance_ours` example, leaving `threat_safe_pawn_push_eg`).
**Bench 5,121,269 → 5,448,086.** Candidate `rarog-phase45-imbalance-pext-pgo.exe`.
**SPRT to run (user):**

```
./tools/sprt.ps1 `
  -EngineA "tools\test_engines\rarog-phase45-imbalance-pext-pgo.exe" `
  -EngineB "tools\test_engines\rarog-phase44-scalars-pext-pgo.exe" `
  -NameA "Imbalance45" -NameB "Scalars44" -Elo1 3
```
(`gainer`, gate `[0,3]` → H1 ≥3 nElo, `tc=3+0.03`.) On H1 → new head. On H0 →
`git revert` the 4.5 bake (and consider the stage genuinely not worth it, per the
plan's "optional" flag).

#### 4.1b King-safety SPSA polish (optional, decide when reached)

After stage 4.1 is accepted, optionally run a **small game-based SPSA** over the
handful of highest-leverage king-safety knobs (the danger conversion divisor and
the safe-check / weak-ring / queen-relief unit weights — expose them as tune UCI
options first). Rationale: Texel optimises "static eval predicts result," not
game strength directly, and king safety is the term where that gap bites most;
SF/Ethereal both *game*-tune their king-safety scalars on top of the data fit.
Keep the group tiny (≤6 params) so it is cheap on the 5950X. SPRT `[0,3]` vs the
post-4.1 head; keep only if it passes. This is the **one** place worth spending
extra game-compute on the eval — the user will decide whether to run it here.

**Model:** the staged-campaign driving (run tuner, bake, build, SPRT, read
verdict, decide) is **Sonnet 4.6 medium**; escalate to **Opus 4.8 high** only if
a stage's fit looks pathological and the tuner/trace must be debugged.

### Expected
**+120–230 Elo** across the phase — fitting a hand-weighted eval *of this
enlarged size* to data for the first time is the single largest HCE lever, and
the Phase-3 build-out is what lifts the ceiling above the old "+60–150"
(tune-existing-terms-only) estimate. Confirm at LTC and run the external
gauntlet (§10) before declaring the phase.

**Stage 4.6 — material+PST refit ✅ ACCEPTED +27.6 Elo (2026-06-24).** SPRT
`Pst46` vs `Imbalance45`: **+27.64 ± 11.23 Elo** (nElo +36.61 ± 14.80), LOS 100%,
LLR 2.96, H1 at 2116 games (54.0%). The final big block lands.
`rarog-phase46-pst-pext-pgo.exe` is the **new head**; canonical bench
**5,794,671**. **Phase-4 staged total so far ≈ +251 Elo (4.1+4.2+4.3+4.4+4.5+4.6),
above the +120–230 estimate** — the Phase-3 build-out lifted the ceiling as
intended. Only 4.7 polish + LTC/gauntlet remain. Fit details below.

The final,
biggest block: material (`MG_VAL`/`EG_VAL`) + all 768 PST entries, ~778 params,
on the 4.5 head. Full 2.19M set, 400 epochs (K=1.704). **Holdout 0.09507 →
0.09352 (−0.00156), every bucket improved, no regressions** (pawn-ending −0.0037,
opening −0.0022, endgame −0.0017; OCB even *recovered* −0.0017 from the 4.5
wobble). Values sane: structural zeros held (pawn PST ranks 1/8 = 0); the only
extreme PST entries are pawns on the 7th rank (198–232, correctly huge); and the
material **ratios-to-pawn are essentially unchanged** (mg N4.1→4.4, Q12.5→11.9) —
the mg values inflated ~×1.1 only to match the lower fitted K, a benign scale
shift not a distortion.

**On the plan's "L2 toward PeSTO":** tested it (`--l2 1e-6`) and it **froze the
fit near the prior** (holdout 0.09502, banking just −0.00005 vs −0.00156) — the
same all-or-nothing behavior seen on every group. Since the unregularized fit was
already sane (the exact condition L2 was meant to enforce), shipped **without**
L2. *Tuner note: L2-to-prior in this Adam setup is a hard "freeze near prior"
lever, not a gentle regularizer; sparse-square robustness comes instead from the
zero-gradient-keeps-seed property (structural zeros) + clamps.*

One unit test (`evaluator_rewards_advanced_protected_passers_over_back_rank_pawns`)
flipped by 3cp — its "advanced" FEN had the enemy king adjacent to the e6 pawn,
and the new eval correctly discounts passers under enemy-king pressure. Fixed the
test to park the enemy king away from both pawn pairs (isolating the advancement
property); all tests pass. **Bench 5,448,086 → 5,794,671.** Candidate
`rarog-phase46-pst-pext-pgo.exe`. **SPRT to run (user):**

```
./tools/sprt.ps1 `
  -EngineA "tools\test_engines\rarog-phase46-pst-pext-pgo.exe" `
  -EngineB "tools\test_engines\rarog-phase45-imbalance-pext-pgo.exe" `
  -NameA "Pst46" -NameB "Imbalance45"
```
(`gainer`, gate `[0,5]` → H1 ≥5 nElo, `tc=3+0.03`.) On H1 → new head, then Stage
4.7 global polish. On H0 → `git revert` the 4.6 bake.

**Stage 4.7 — global polish ✅ ACCEPTED +65.0 Elo (2026-06-24).** SPRT `Polish47`
vs `Pst46`: **+64.97 ± 13.11 Elo** (nElo +91.97 ± 18.12), LOS 100%, LLR 2.95, H1
at 1412 games (59.2%) — far above the expected +15–25, because every staged fit
had optimized one group while the others sat at pre-tuned/seeded values, so the
joint pass corrected all the cross-group interactions at once.
`rarog-phase47-polish-pext-pgo.exe` is the **new head**; canonical bench
**4,747,104**. **PHASE 4 COMPLETE.** Staged self-play total ≈ **+316 Elo**
(4.1 +42.5, 4.2 +45.2, 4.3 +24.1, 4.4 +85.2, 4.5 +26.7, 4.6 +27.6, 4.7 +65.0) —
far above the +120–230 estimate. *Caveat: these are compounding STC self-play
SPRT gains; the external gauntlet (§10) will show the smaller real-opponent
figure — that transfer check is the immediate next step.* Fit details below.

Low-lr joint fit
(`all47` group = everything linearly tunable, the 3 sparse pairs kept frozen;
1172 params, lr 0.1, 300 epochs, K=1.674) on the 4.6 head. **Holdout 0.09359 →
0.09273 (−0.00086), every bucket improved, no regressions** — the joint optimum
found real cross-group gains the group-by-group staging left behind (material
drifted slightly, e.g. mg knight 415→396). Sane; 1006/1172 params moved, mostly
small.

**Baking method (reusable):** 1006 params across 124 fields is too many to
hand-edit, so added `tools/texel/bake_params.py` — bakes a complete tuner dump
into the `EvalParams` defaults (PST/material via their named consts, everything
else as inline macro literals, comments preserved). **Verified by bench-match:**
a `--features tune` binary loading `all47.txt` benches **4,747,104**; the baked
normal build benches the *same* 4,747,104 — proving the bake is exact. The
Phase-3 `seeded_zero_terms_are_inert_but_tunable` gate was removed (its last
seeded-0 example was activated by this polish; Phase 4 has now activated
everything it guarded). All tests pass. **Bench 5,794,671 → 4,747,104.** Candidate
`rarog-phase47-polish-pext-pgo.exe`. **SPRT to run (user):**

```
./tools/sprt.ps1 `
  -EngineA "tools\test_engines\rarog-phase47-polish-pext-pgo.exe" `
  -EngineB "tools\test_engines\rarog-phase46-pst-pext-pgo.exe" `
  -NameA "Polish47" -NameB "Pst46" -Elo1 3
```
(`gainer`, gate `[0,3]` → H1 ≥3 nElo, `tc=3+0.03`.) **This is the last eval
stage** — on H1 or H0, Phase 4 is done and the next step is the end-of-phase
**LTC confirm + external gauntlet** (the over-fit transfer check). On H0 →
`git revert` the 4.7 bake (the 4.6 head stays).

### 4.8 — Eval data-refresh iteration (decide AFTER Phase 5; not a Phase-4 redo)

**The current 2.19M dataset was self-played by the *pre-Phase-4* engine.** Once
Phase 4 (and Phase 5) make the engine ~+200 Elo stronger, regenerating self-play
data and re-fitting the eval once is a real, standard lever — the data-fit
bootstrap ratchet. **Record now so we discuss it at the right time; it is
optional and evidence-driven, not mandatory.**

- **Why it helps:** a stronger engine gives (1) cleaner WDL labels — a weak
  engine blunders won positions into draws/losses, mislabeling the Texel target;
  (2) better-distributed, more on-policy positions. Better labels → a tighter fit
  on the *same* (now-activated) terms.
- **It is NOT a re-stage.** The 4.1–4.6 staging existed to *activate* seeded-0
  terms and isolate per-group Elo — a one-time job. On fresh data, do a **single
  joint low-lr refit** of the whole activated eval (like 4.7) **plus the
  king-safety `--tune-kingsafety` re-eval path**, then **one SPRT** vs the head.
  Cost: datagen < 1 h + a minutes-long fit + one SPRT.
- **When:** **after Phase 5**, not immediately after 4.6 — Phase 5 makes the
  engine stronger too, so one refresh then banks the label-quality gains from
  *both* phases instead of regenerating twice. Gate the decision on the
  end-of-Phase-4 gauntlet/LTC (how much of the staged Elo transferred vs
  over-fit) and on whether holdout loss still has headroom.
- **Exploit the dormant Step-4.0 capabilities on the regen:** turn on **blended
  labels** (`α·result + (1−α)·score_target`; `parse_target` already accepts the
  float column — datagen must emit it) and **phase-balanced sampling**
  (`extract.py --balance-phase`). Both directly attack label noise — exactly what
  a second iteration should use.
- **Expected:** **+10–40 Elo** for the first refresh, less for any subsequent one
  (it is a correction, not a re-discovery). Strong HCE evals do 1–3 such
  iterations; past that the curve flattens and **NNUE (§13), not more HCE data,
  is the next lever.**

---

## 9. Phase 5 — Search-efficiency wave (deferred SPSA + refinements)

**Goal:** harvest the search gains that were deliberately deferred until the
eval scale is final. This is where the **one** search-constant SPSA wave is
spent (§6 principle: margins are eval-centipawn-denominated, so this must come
after Phase 4), plus the centipawn-coupled retries and a menu of modern search
refinements. Secondary aim: reduce nodes-per-depth toward Stockfish's regime
(finding 10: EBF ≈ 2.2 vs ≈ 1.8). Honest expected value: **+20–50 Elo total**
across the whole wave — the search is already mature (§13 audit), so these are
many small, individually SPRT-gated steps, not one big lever.

**Primary driver for the whole phase: Sonnet 4.6 medium** (run SPSA, bake, SPRT,
read verdict, decide). **Escalate the dense algorithmic ports** (multi-cut /
singular interaction, threat-aware history indexing, the codex `tt.rs` diff) to
**Codex 5.5 medium** or **GPT-5.5 high**.

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

1. **The one search-constant SPSA wave** at the **post-Phase-4 head (eval is
   final)**, all at the unified **`tc=3+0.03`** (SPSA and the confirming SPRT
   share this TC — see the Test-TC note in §2). Groups: pruning group, the
   Phase-2 LMR group, the futility group, and the ProbCut margin (currently a
   hardcoded `180`, search.rs:1097 — expose it as a UCI option and add it to the
   wave). Same Phase 1 workflow: SPSA → bake → SPRT per group. This is the
   compute we conserved by deferring it: margins like futility/razoring are
   denominated in eval centipawns, and Phase 4 changed what a centipawn means.
   - **Includes relocated 2.11**: re-tune the Group-B `FutilityNotImproving`
     (49/60) and `LmpNotImproving` (57/60) coefficients with the `[0,60]`
     ceilings **widened to `[0,120]`** in `config_pruning.json` — both were
     pinned at the old ceiling, plausibly clipping Elo. SPRT `[0,3]` vs the head.
   - **Includes relocated 2.5.2 — futility-direction A/B** (eval-independent but
     parked here): Rarog's move-loop quiet-futility margin shrinks when
     `improving` (prunes *more*), opposite to SF's no-modulation. A/B the
     conventional direction (larger margin when improving) and the
     no-modulation variant vs current, each gated `[-3,3]`. Cheap; fold into the
     futility-group work.
   - **Includes the Phase-3.16 lazy-eval margin re-check** (`LAZY_MARGIN`,
     currently a hardcoded `600` in `eval.rs`; accepted at the seeded-0 head,
     **+4.4 Elo**). This is a **safety** re-check first, a speed knob second:
     Phase 4 grows the positional weights, so the margin that guaranteed "no
     skipped term can flip the sign" at seeded-0 may become **too tight** (a
     lazy skip could then mis-evaluate a position whose positional swing now
     exceeds the gap). Expose it as a UCI option, **widen it first and confirm
     no regression `[-3,3]` at the post-Phase-4 eval scale**, then SPSA-tune for
     NPS. Lazy is disabled under `--features texel`, so this never touches the
     tuner. (The mop-up is already extracted to `apply_mop_up` and runs on both
     paths, so mating technique is margin-independent.)
   - Clock play is the test/deployment target, so **add the 2.2 TM constants as
     their own SPSA group** (Reckless tuned all of its TM multipliers this way)
     — the TM is exercised at `tc=3+0.03`.
**1a. do-deeper re-implementation** (the cp-coupled TC-suspect retry — do it
   here *after* the eval re-fit, under the unified `tc=3+0.03` + `tc=10+0.1`
   LTC). The eval-scale-*independent* TC-suspect retries (2.4 LMR, accepted
   +4 Elo; 2.12 futility direction, now folded into step 1 above) are resolved.
   - **2.8 do-deeper.** Re-add the `do_deeper` arm (`+1` ply when
     `score > best_score + DeeperMargin + 2·reduction`), `DeeperMargin` UCI
     option + `config_dodeeper.json`, SPSA, SPRT. Its value is differential
     effort, which a clock rewards and fixed movetime nullified (the banking
     mechanism). cp-margin → it belongs *after* the eval re-fit, not in
     Phase 2.5. If it passes here it both recovers Elo and validates the new
     methodology.

2. **Search-wave items with concrete specs** (one at a time, SPRT `[0,3]`
   unless noted):
   - **History formula upgrade**: replace the symmetric
     `history_bonus(depth)` (move_ordering.rs:127) — currently used for both
     the cutoff bonus and the penalty on tried-and-failed moves — with
     separately scaled linear bonus/malus, as all strong engines do:
     `bonus = min(bonus_mul·depth − bonus_sub, bonus_max)`,
     `malus = −min(malus_mul·depth − malus_sub, malus_max)`.
     Seeds: `170/90/1700` and `180/100/1500`. Expose all six, SPSA, SPRT.
     **After this lands and passes, retry Phase 2.3 (no per-search history
     aging)** as its own `[0,3]` gate — this formula fix is the precondition
     that makes no-aging viable (see §5 2.3 outcome). The −12.4 Elo from the
     first 2.3 attempt was because the old symmetric formula relied on the
     halving as a decay; with self-regulating bonus/malus that crutch is gone.
   - ~~**Qsearch TT-bound stand-pat refinement**~~ — **done as Phase 2 step 2.5**
     (commit `374445e`, H1 +6.5 Elo). Removed from this list.
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
   - ~~**Double-extension cap**~~ — **tried and dropped as Phase 2.6**
     (commit `8ccebd3`, reverted). Cap=12 measured -1.73 Elo, H0 in `[-3,3]`.
     No upside at any TC for Rarog: time safety is enforced by the node-based
     stop check (every-2048-nodes vs `maximum_ms`), not by bounding extension
     depth, and total growth is already capped by `MAX_PLY=128`. The cap only
     adds search-shape cost. Clock play is now the test target (`tc=3+0.03`),
     but the second condition is not met: do not re-attempt unless real time
     forfeits are observed — and even then the fix is the time manager, not the
     extension cap. (This is why 2.6 is excluded from the 1a retry set.)
   - **Razoring restriction experiment**: try `depth <= 1` (from
     `depth <= 3`, search.rs:1017) — RFP covers most of razoring's range and
     the qsearch verification isn't free. SPRT both ways; keep whichever
     passes.
   - *(optional, risky — try last, one at a time)* **Selective extensions à la
     SF**: a **passed-pawn-push extension** (extend a pawn push to the 6th/7th
     when safe) and a **castling/king-move extension**. These change tree size
     non-trivially and only tend to pay once the eval and the rest of the search
     wave are stable, so gate each on its own SPRT `[0,3]` and drop it if H0.
     Rarog already has singular extensions and tried/dropped the double-ext cap
     (Phase 2.6), so this is purely additive experimentation, not a fix.
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
   phase — finding 10 demoted it: NPS is not the gap driver, but the
   2026-06-19 gauntlet measured Rarog 2.1.0 at **2.31 M nps / depth 12.4 vs
   Basilisk 1.5.1's 2.76 M / 13.8** — a ~16–19% gap worth collecting, and a
   shallower search costs Elo directly).

   **4a. Free win, do first — match Basilisk's build tuning (no code change).**
   The 2.31 M figure is from the `pext` test binary, which is built with
   `target-cpu=x86-64-v3` (generic `-mtune`), whereas **Basilisk's local build
   uses `-march=native` (= `znver3` tuning)** on the same 5950X. Same *features*
   (both have PEXT/POPCNT/BMI2/AVX2), but Basilisk gets Zen3-specific
   instruction scheduling; `cargo build --release` already uses `target-cpu=native`
   via `.cargo/config.toml`, but `xtask --arch pext` and `build_test.ps1` do not.
   **Add a `--arch native` (or `znver3`) local path** to `xtask`/`build_test.ps1`
   (`--cfg rarog_pext -C target-cpu=native`, + PGO) and use it for the user's own
   5950X testing/deployment; keep `x86-64-v3`/`avx2`/`pext` for portable release
   assets. Expect a few % nps back immediately, and it makes the Basilisk
   comparison apples-to-apples. *(The architecture use is otherwise already
   good: PEXT sliders on Zen3, hardware `popcnt`/`tzcnt`/`lzcnt`, fat LTO, one
   codegen unit, `panic="abort"`, PGO.)*

   **4b. Profile, then micro-optimise.** Profile first (`cargo flamegraph`,
   `samply`, or VTune on the native+pext build) — do not guess. Likely
   candidates from the code audit, to confirm under the profiler:
   - **Bounds-check / `Option` overhead in board accessors.** `board.rs` has
     only one `get_unchecked`; hot accessors like `piece_on` return
     `Option<Piece>` and index the mailbox with safe `[idx]`. C++ Basilisk
     returns raw `PieceType` from a flat array with no checks. Audit the hottest
     accessors (`piece_on`, `piece_at`, `captured_piece`, mailbox reads in
     make/unmake, `attackers_to_color`) and use `get_unchecked` / raw-type
     returns where the index is provably 0..64 (Square is always in range).
   - The documented Basilisk-edge micro-opts (still valid): delay direct-check
     detection in the LMR path until the cheaper reduction gates pass; cache
     direct-check masks during quiet move scoring; add a **boolean** attack-test
     helper so legality/outpost/hanging yes/no questions don't materialize full
     attacker bitboards; fast non-insufficient-material early exit before
     expensive draw checks; scan the move picker with pointers, skip self-swaps.
   - The Phase 3 **attack-map substrate** already removes the per-square attack
     recompute in eval (the largest eval-side cost) — its nps gain lands earlier.
   Each change: bench fingerprint unchanged → ≥5 bench runs → keep if ≥1%
   faster; one simplify-bounds SPRT (`[-3,0]`) over the batch. Target: close the
   gap to Basilisk (≥2.7 M nps native).
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
6. End of phase: external gauntlet + release per §10. Extend the gauntlet
   opponent list per finding 9's ladder (add SF capped 2800, then 3000, once
   M2 falls) so the rating signal doesn't saturate.
7. **Then decide on the §4.8 eval data-refresh** — regenerate self-play with this
   now-much-stronger head and do one consolidated eval refit (blended labels +
   balanced sampling). This is the natural moment: doing it after Phase 5 banks
   both phases' strength into the new data in a single regen. Evidence-driven
   (see §4.8); not mandatory.

### Expected
+30–80 Elo. Cumulatively, Phases 2–4 should clear milestones M1
(SF-capped-2600) and M2 (Basilisk 1.5.0) and push toward M3 (+150 over the
current 3015) — consistent with finding 10's honest ceiling of +150–350 total
from HCE work. Beyond that plateau, HCE progress gets exponentially more
expensive per Elo (the classical-era Stockfish project spent years there with
thousands of donated cores); when this phase completes, revisit §11.

---

## 10. Release & regression discipline

- Keep `master` as the **gauntlet baseline**. After
  each phase, run a **multi-opponent gauntlet** to confirm the SPRT self-play
  gains transfer against external opponents — **eval changes (Phases 3–4) over-fit
  self-play more than search changes do, so the gauntlet matters most after
  Phase 4.** Run at a **representative clock TC** (the `10+0.1` LTC, optionally a
  longer control) rather than only 100 ms/move (§2 Test-TC note).

  **Recommended opponent ladder** (the user picks the exact binaries by measured
  performance on the 5950X — these are the targets):
  | Opponent | Why | ~Elo (CCRL) |
  |---|---|---|
  | Rarog 2.0.2 + Basilisk 1.5.0 | own-history + sibling sanity | ~2940 / ~3100 |
  | **Critter 1.6a** | the user's named reference ("loses easily") — the engine to beat | ~3150–3200 |
  | **Stockfish capped — start at 2700, then 2800, then 3000** | a *tunable* known-rating yardstick; **recommend the `UCI_LimitStrength`/`UCI_Elo` cap on a modern Stockfish** (set the cap, don't trust the number absolutely — calibrate by score) | as capped |
  | One mid HCE of known rating (e.g. **Lambergar / Peacekeeper / Igel HCE**, or another from the user's collection) | independent non-SF, non-sibling check near the target band | ~3050–3210 |

  Start each phase's gauntlet against the band you expect to be in; as a
  milestone falls (M1 SF-capped-2600 → M2 Basilisk 1.5.0 → beat Critter 1.6a),
  raise the capped-SF level so the rating signal does not saturate. **Pick the
  capped-SF level where Rarog currently scores ~30–70%** — that is where the
  signal is sharpest.

  **Calibration caveat (learned 2026-06-19):** SF `UCI_Elo` is calibrated at
  **120s+1s anchored to CCRL 40/4** — it is **not** a reliable absolute anchor at
  the `tc=3+0.03` SPRT TC (the cap throttles depth, but SF still plays SF-quality
  moves, so a "2700" cap performed at the *top* of the 35k-game overnight pool).
  Do not read CCRL numbers off the capped-SF labels. **For a real
  CCRL-anchored placement, run a separate small gauntlet at a slower TC (e.g.
  `tc=15+0.1` or `30+0.3`) that includes Critter 1.6a** — at `tc=3+0.03` Critter
  forfeited every game, so the one good external HCE anchor was lost. One clean
  Critter result is worth more than any capped-SF number.

  **How to estimate a CCRL rating (the gauntlet numbers are otherwise
  arbitrary).** Ordo's `-a 3000` just floats the pool *average* at 3000 — the
  absolute numbers mean nothing on their own; only the *relative* deltas are
  real. To put the pool on the CCRL scale:
  1. **Include ≥1 engine with a stable published CCRL 40/15 rating** in the
     gauntlet. Best: an engine that runs cleanly at your TC — **Critter 1.6a
     (~3180)** at a slower TC, or a classic stable anchor like **Fruit 2.1
     (~2783)**, or any released engine you have that is on the CCRL list.
     Two anchors spanning the range (one ~2800, one ~3200) let you check the fit.
  2. **Anchor Ordo to it:** `ordo-win64.exe -p Results.pgn -o rating.txt -a <ccrl>
     -A "<exact PGN engine name>"` — `-A` *pins* that engine to its CCRL number
     and scales everyone else relative to it (instead of `-a 3000` floating).
     With two anchors, run twice (once pinned to each) and average the offset, or
     pin the lower and confirm the upper lands near its CCRL number (the spread
     is your error bar).
  3. **Run that anchoring gauntlet at a TC as close to CCRL 40/15 as you'll
     tolerate** (e.g. `40/300` or a long `60+0.6`), single thread, on the 5950X —
     the closer the effective depth to CCRL's, the better the estimate.
  4. **Treat the result as ±50–100 Elo.** TC, hardware, book, and pool
     composition all differ from CCRL, so it is an *estimate*, not an official
     rating; the within-pool deltas remain exact. This is enough to know whether
     you are ~3100, ~3300, or closing on the ~3380–3450 top-HCE band.
- Rebuild the PGO asset (`cargo xtask build --arch pext --pgo`, or `avx2` for
  a distribution build) before any gauntlet — tuning changes the hot paths.
- Bump version + CHANGELOG only when a phase clears both SPRT and the external
  gauntlet.

**Phase 4 gauntlet RESULT (2026-06-24, `tools/gauntlet.ps1`, `tc=10+0.1`,
2700 games, gauntlet mode = Rarog 2.2.0 vs each of 9 opponents).** The gate is
**cleared** — the staged self-play gain transferred to the external field:

| Opponent | Rarog 2.2.0 score | H2H Elo | Note |
|---|---|---|---|
| Rarog 2.1.0 (prev release) | 80.0% | **+240** | real Phase-4 gain vs the engine 2.2.0 supersedes |
| Rarog 2.0.2 | 87.7% | +344 | |
| Basilisk 1.6.0 (sibling) | 70.5% | +152 | beats current sibling |
| Basilisk 1.5.0 | 78.3% | +222 | |
| Stockfish cap-2700 | 65.2% | +110 | |
| Stockfish cap-2800 | 49.3% | −5 | even |
| Stockfish cap-2900 | 40.5% | −67 | |
| Critter 1.6a (~3187 CCRL) | 15.7% | −292 | |
| Fruit 2.1 (2780 CCRL) | 85.2% | +304 | |

- **CCRL placement ≈ 3000.** Two-anchor bracket: Fruit-pinned → 3086,
  Critter-pinned → ~2890; midpoint ~2980. The single-Fruit pin (`ordo … -a 2780
  -A "Fruit 2.1"`) overstates because beating an old HCE by a wide margin
  inflates the gap; Critter (closest in strength) is the more trustworthy
  anchor. We quote **~3000 CCRL** and do not claim more.
- **Self-play overstated by ~75 Elo, as predicted.** The staged self-play sum
  was ≈+316; real cross-engine play vs 2.1.0 is +240 — ~75% transfer, an
  excellent rate for an eval-only campaign.
- **Little Blitzer time confound CONFIRMED and worked around.** Critter
  forfeited ~100% in LB even at 10+0.1 but played 300 clean games in fastchess
  (`timemargin=1000`, `concurrency=8`). Use fastchess, not LB, for any gauntlet
  with old/slow-IO engines.
- **Capped-SF runs hot at fast TC, as warned:** cap-2900 → ~3154, cap-2800 →
  ~3091. Relative milestones only; not CCRL anchors.

→ **v2.2.0 is cleared to tag and publish.**

### 10.1 Release checkpoints — when to cut a GitHub release, and what to call it

The CI workflow (`.github/workflows/build.yml`) builds and attaches binaries
for every platform/arch automatically `on: release: published` — cutting a
release is a documentation + tagging job, not a manual binary-build job.

**Cadence: release at phase boundaries that change playing strength, not
mid-phase.** Phase 3 (eval infrastructure) is bench-fingerprint-identical by
design — every step in it is a structural no-op until Phase 4 activates the
new terms — so there is no Elo reason to wait for Phase 3 to fully finish
before releasing the work already accepted before it. Concretely:

| Milestone | Version | Why this boundary |
|---|---|---|
| **Phase 0–2.9 closed** (harness + search tuning + robustness) | **2.1.0** | First real, cumulative, SPRT-confirmed strength gain since 2.0.2 (dominated by the 2.2 time-management fix, +81 Elo at `st=0.1`, plus several smaller confirmed tunes and a forfeit-elimination robustness fix). Already the version baked into `Cargo.toml`/`CHANGELOG.md` from early branch work — **never tagged/published**, so this is its first actual release, not a bump. Phase 3.0–3.1 (bench-identical eval-rewrite groundwork) can ride along since they change nothing observable. |
| **Phase 3 fully closed** (3.0–3.12, all eval structure built, still bench-identical) | *(no release — optional)* | Nothing plays differently; a release here would have identical Elo to 2.1.0. Skip unless a long gap or a bug fix makes a patch worthwhile. |
| **Phase 4 closed** (staged Texel data-fit activates the new eval terms) | **2.2.0** | The plan's own estimate is **+120–230 Elo** — the single biggest jump in the whole program. Clearly its own minor release. |
| **Phase 5 closed** (search-efficiency wave: deferred SPSA + refinements) | **2.3.0** | Another distinct, measurable strength jump (~+20–50 Elo estimate), and the last phase on the current roadmap. |
| Any bug fix or robustness fix outside the above (no eval/search behavior change intended) | **2.x.1, 2.x.2, ...** | Patch bump, following the existing `2.0.0→2.0.1→2.0.2` convention. |
| NNUE (§13, not scheduled) | **3.0.0** | A new evaluation paradigm, not an incremental tune — major bump if/when it ever happens. |

**Release checklist** (do this in order; nothing here should be skipped):

1. Confirm the target commit passes `cargo test --release` and
   `cargo fmt --check`, and that `bench 13` matches the value recorded at the
   top of this document for the current checkpoint.
2. Update `Cargo.toml` (`version = "X.Y.Z"`) — `Cargo.lock`'s `rarog` entry
   updates automatically on the next build.
3. Write the new `CHANGELOG.md` entry at the top (after the header, before
   the previous top entry): `## [X.Y.Z] - YYYY-MM-DD`, with `### Added` /
   `### Changed` / `### Fixed` / `### Evaluated and rejected` (for
   transparency — list speculative ports/tunes that were tried and reverted,
   with their measured Elo, matching the convention already used for 2.1.0)
   / `### Internal` (bench-identical groundwork, no behavior change) /
   `### Verified` (the SPRT/gauntlet numbers that justify the release).
4. Check `README.md` for anything version- or feature-specific that needs
   updating (new UCI options exposed in production, new highlights). Most
   phases in this plan gate new options behind `--features tune` or seed new
   terms inert, so README usually needs no change — confirm rather than skip.
5. Rebuild and bench-verify the actual release assets locally before tagging
   (`cargo xtask build --arch pext`, `--arch avx2`; run `bench 13` on each and
   confirm the fingerprint and a clean `uci` handshake with no tune-only
   options) — catches a packaging mistake before CI does the real multi-platform
   build.
6. Commit the version bump + CHANGELOG (+ README if touched) as one commit:
   `git commit -m "Version X.Y.Z"` (matches this repo's existing convention —
   see `git log --oneline | grep Version`).
7. **User action (not the model):** tag and push (`git tag vX.Y.Z && git push
   origin vX.Y.Z`), then create the GitHub release from that tag — paste the
   prepared release notes into the release description. Publishing the
   release triggers CI to build and attach all platform binaries
   automatically. Tagging/publishing is externally visible and is the user's
   call, not something to do unprompted.
8. After publishing, run the post-release external gauntlet (§10 above) to
   confirm the self-play SPRT gains transferred, and update the gauntlet
   baseline reference if appropriate.

### 10.2 Legacy feature branches — what to keep, what to drop

Branches `v2.1.0-codex` (search-efficiency rewrite source) and
`v2.1.0-claude` (eval expansion + `tune.rs` source) are **reference-only,
never released**. Phases 1–2 have already harvested several ideas from them
(the 1024ths LMR port, the SF-style time-management structure, ProbCut —
tried and reverted, check-awareness move-ordering — tried and reverted via
the `improvements` branch). The remaining un-ported material is tracked in
the Phase 5 search-feature menu (§9) — **keep both branches until Phase 5
is fully resolved** (every remaining idea either ported+accepted or
explicitly rejected), then archive or delete them.

Branches `claude` and `improvements` were stale (nothing left to harvest) and
were **deleted 2026-06-20** along with the long-lived `v2.1.0-codex-work`
integration branch, which was squash-rebased onto `master` instead of kept
as a separate branch — `master` is now the single integration branch.

---

## 11. Risks & gotchas

- **Untuned constants are the #1 failure mode** (proven by both prior branches).
  Never SPRT-judge a new heuristic before tuning its constants.
- **Time forfeits are now CONFIRMED, not hypothetical — fix before the eval
  campaigns (high priority).** The 2026-06-19 Little Blitzer gauntlet at
  `tc=3+0.03` showed **Rarog 2.1.0 dev lost 28 games on time** (`t=28`) where
  release 2.0.2 lost `t=0`; Basilisk 1.5.1 dev lost **65** (vs 1.5.0's 18). The
  full-budget movetime / clock TM is forfeiting under fast clocks. This is **pure
  lost Elo and it contaminates every external gauntlet** (SF never forfeits, so
  the loss is one-sided). Enable the documented safety valve **now**
  (subtract `min(MoveOverhead, movetime/10)` on the fixed path; add a hard
  remaining-time floor on the clock path) rather than waiting for the Phase 5 TM
  work or reverting the feature. Re-verify forfeits drop to ~0 before trusting
  any gauntlet number.
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
  feature and cleared whenever runtime weights change (§7 step 3.3).
- **Don't trust the `bench` fingerprint as a strength signal** — it only proves
  *behavior identity*. A changed fingerprint is neither good nor bad; only SPRT
  decides.
- **Time-management features** must be tested under real clocks, not fixed
  ms/move, or their effect is invisible.

---

## 12. Quick command reference

```powershell
# Inspect what a branch added, step by step
git log --oneline 5a8ce52..v2.1.0-codex
git diff 5a8ce52 v2.1.0-codex -- src/search.rs

# Cherry-pick a single feature step onto an integration branch
git checkout -b feat/probcut master
git cherry-pick <step-commit>      # or re-implement the isolated diff

# Regression-identity check (refactors only) — Windows PowerShell
echo "bench 13`nquit" | .\target\release\rarog.exe   # expect 4,978,006 on current head (3.14+)

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

## 13. NNUE readiness — keep the door open (NOT a scheduled phase)

**This plan is HCE-only. NNUE is explicitly out of scope for every phase above
and is not scheduled.** This section exists for one reason: so the HCE work in
Phases 1–5 does not accidentally make a *future* NNUE switch expensive. None of
the items below are tasks to do now — they are guardrails to observe **while**
doing the phases above. If you never go NNUE, you lose nothing by following them
(they are just clean design). If you ever do, they turn a rewrite into a swap.

### Why the architecture matters more than the feature

The dominant cost of a future HCE→NNUE switch is not training a network — it is
disentangling eval logic that leaked into the search. If eval knowledge lives
only in `src/eval.rs` behind the `Evaluator` struct, the switch is a localized
replacement. If piece values, mobility scores, and danger bonuses are inlined
into pruning margins and move ordering, the switch becomes a surgical rewrite.

### Guardrails to observe during Phases 1–5

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
Phases 1–5: **never let the search know how the eval works.** If a reviewer
would need to understand mobility counting to understand a pruning condition,
that is a boundary violation — fix it then, not later.

---

## 14. Recommended models per phase

The plan is incremental and test-gated, so *driving* each loop (build, run
harness, read the SPRT verdict, bake, document) is reliable, cost-effective
**Sonnet 4.6 medium** work. What changes per step is the **authoring** of the
feature: the eval rewrites are design-dense, correctness-critical code where a
larger model earns its cost; the mechanical refactors and the harness loop do
not.

**Rule of thumb:**
- **Opus 4.8 high** — design-dense, many interacting terms, subtle correctness,
  hard to bisect if wrong: king-safety v2 (3.5), threats package (3.6), material
  imbalance (3.9), scale-factor framework + KBNK corner math (3.11), and the
  Texel **tuner core** (trace exactness + K-fit + Adam gradient — the
  reconstruction acceptance test is unforgiving).
- **Opus 4.8 medium** (or **GPT-5.5 high**) — substantial but well-specified:
  per-count mobility (3.7), pawn-structure depth (3.8), the well-bounded
  individual endgame functions (3.11, after the framework exists).
- **Sonnet 4.6 medium** — mechanical, equivalence-gated, or loop-driving: the
  attack-map substrate (3.0), `EvalParams` hoist (3.1), loader/dumper + dataset
  scripts (3.2/3.4), the small-positional-term batch (3.10), the entire Phase 4
  staged-campaign driving, and the Phase 5 SPSA/SPRT driving + gauntlets.
- **Codex 5.5 medium** (or **GPT-5.5 high**) — dense search-algorithm ports in
  Phase 5: multi-cut / singular interaction, threat-aware history indexing, the
  codex `tt.rs` diff. Hand Sonnet the feature back for the test loop.

| Phase / step | Work | Model + mode |
|---|---|---|
| 3.0 | Attack-map substrate (refactor) | Sonnet 4.6 medium |
| 3.1 | `EvalParams` hoist + runtime tables | Sonnet 4.6 medium |
| 3.2 | Loader/dumper | Sonnet 4.6 medium |
| 3.3 | Trace + tuner binary + reconstruction test | **Opus 4.8 high** (core); Sonnet 4.6 medium (scaffolding) |
| 3.4 | Self-play dataset + extraction | Sonnet 4.6 medium |
| 3.5 | King-safety v2 structure | **Opus 4.8 high** |
| 3.6 | Threats package structure | **Opus 4.8 high** |
| 3.7 | Per-count mobility tables | Opus 4.8 medium / GPT-5.5 high |
| 3.8 | Pawn structure + passed-pawn detail | Opus 4.8 medium |
| 3.9 | Material imbalance hooks | **Opus 4.8 high** |
| 3.10 | Small positional terms (batch) | Sonnet 4.6 medium — **done, Sonnet 4.6** |
| 3.12 | Gauntlet additions (core) | Opus 4.8 medium / Sonnet 4.6 medium — **core done, Opus 4.8** |
| 3.11 | Scale-factor framework + endgames | framework/KBNK **Opus 4.8 high — done, Opus 4.8**; per-EG funcs GPT-5.5 high (remaining) |
| 3.13 | Permanent endgame regression suite | Sonnet 4.6 medium (harness done w/ 3.11) |
| 3.14 | Eval-cache correctness fix (before Phase 4) | **Opus 4.8 high** |
| 3.15 | Eval inert-block gating (NPS) | **investigated & rejected** (throwaway post-tune) |
| 3.16 | Lazy eval (NPS, SPRT-gated) | **implemented, Opus 4.8** (+11.8 % NPS; awaiting SPRT) |
| 4.1–4.7 | Eval data-fit campaign (driving) | Sonnet 4.6 medium (escalate Opus 4.8 high if a fit is pathological) |
| 5 (most) | Search SPSA + refinements (driving) | Sonnet 4.6 medium |
| 5 (dense ports) | Multicut/singular, threat-aware history, `tt.rs` | Codex 5.5 medium / GPT-5.5 high |

**Non-negotiable regardless of model:** never merge a change that has not passed
its gate — bench-fingerprint identity in Phase 3, SPRT in Phases 4–5. The
process, not the model, guarantees the result.

---

## 15. Appendix — component maturity audit (reference)

The audit behind §6's "search is mature, eval is the gap" conclusion. Reference
material — the work itself is sequenced in Phases 3–5. Verdict key: **Mature**
(leave it; only re-tune) · **Expand** (right shape, missing cases) · **Upgrade**
(structurally too simple) · **Rewrite** (replace the approach).

### 15.1 Evaluation (`src/eval.rs`)

| Component | Lines | State | Verdict → where |
|---|---|---|---|
| Material values + tapered phase | 12-14, 367 | PeSTO, standard 24-phase taper | **Mature** → tune 4.6 |
| PSTs (mg/eg, 6×64) | 17-78 | PeSTO, never refit | **Mature shape** → tune 4.6 |
| Pawn cache + whole-eval cache | 273-330, 385-399 | Two-level cache, correct | **Mature** |
| Passed pawns (rank table + supported/free/safe-path) | 417-455 | Good coverage | **Expand** → 3.8 (blocked-passer, attacked-path) |
| Pawn structure (doubled/isolated/backward/connected) | 457-482 | Connected is flat `(7,5)` | **Upgrade** → 3.8 (rank-scaled, levers) |
| Bishop pair | 520-523 | Flat `(30,50)` | **Expand** → 3.10 (scale by pawns) |
| Rook files / 7th / behind passer | 525-542, 732-783 | Solid | **Mature** → tune 4.4 |
| Knight outpost | 544-554 | Knights only | **Expand** → 3.10 (bishop outposts) |
| Mobility | 557-566, 878-898 | Linear, tiny weights, loose area | **Upgrade** → 3.7 / 4.3 |
| Pawn threats + hanging | 568-586, 785-812 | Pawn-only threats; flat hanging | **Rewrite** → 3.6 / 4.2 |
| King safety (units→SAFETY[16]) + shelter + storm | 634-730 | Capped 118 cp; no safe-checks/weak-ring/relief | **Rewrite** → 3.5 / 4.1 |
| Space | 619-632 | Minimal centre-files term | **Expand** → 3.10 / 4.4 |
| Passed-pawn king proximity | 814-829 | Present | **Mature** → tune 4.4 |
| Trapped bishop | 831-843 | Bishop only | **Expand** → 3.10 (trapped rook) |
| Mop-up / drive-to-corner | 599-616 | Generic `|eval|>200` term | **Upgrade** → 3.11 (KBNK correct corner) |
| Endgame scaling | 912-939 | OCB + KNN only; one scalar | **Rewrite** → 3.11 (scale-factor framework) |
| Attack-map reuse | (none) | Attacks recomputed per term/per square | **Rewrite** → 3.0 (shared maps) |
| EvalParams / data-fit | (none) | Every weight hand-set, untuned | **Build** → 3.1–3.3, fit in Phase 4 |

### 15.2 Search (`src/search.rs`, `move_ordering.rs`, `tt.rs`, `time_manager.rs`)

| Component | State | Verdict → where |
|---|---|---|
| PVS + aspiration window | Single-delta window | **Mature**; minor Reckless-style avg-centred/asymmetric → Phase 5 |
| Null-move pruning + verification | Standard | **Mature** |
| ProbCut | Present, **hardcoded margin 180**, depth≥4, not UCI-tunable | **Expand** → Phase 5 (expose + tune margin) |
| Singular extensions | Returns singular_beta (multicut-ish) | **Mature**; codex multicut diff → Phase 5 |
| RFP / futility / razoring / LMP | SPSA-tuned | **Mature**; re-tune at final eval scale → Phase 5 |
| LMR (table + 4 node-type adj + history) | SPSA-tuned, in 1024ths | **Mature** |
| Move ordering (TT/SEE/killer/counter + 6 histories) | Rich | **Mature** |
| `history_bonus` formula | Symmetric `min(d²+2d,1200)` for bonus *and* malus | **Upgrade** → Phase 5 (split bonus/malus) |
| `age_history` (halve ~5 MB every `go`) | Non-standard tax | **Rewrite** → Phase 5 (drop aging; enables 2.3 retry) |
| Correction history (pawn/minor/2×non-pawn/cont) | 5 tables | **Mature** |
| Qsearch (delta + SEE + capture-futility) | Strong; no quiet checks at qply 0 | **Expand** → Phase 5 |
| Transposition table | Full-key validated, shared in SMP | **Mature** |
| Time manager | Rewritten Phase 2.2 (SF-style soft/hard) | **Mature**; tune TM constants as an SPSA group → Phase 5 |

**Takeaway:** the search has **no** "Rewrite" item that gates strength except the
history-aging removal (Phase 5). The eval column has **four Rewrites** and several
Upgrades. That asymmetry is the whole story — it is why Phase 4 (eval fit) is the
multiplier and Phase 5 (search) is the +20–50 tail.

Sources for the reference ladder and term-value calibration:
[Stockfish (Wikipedia)](https://en.wikipedia.org/wiki/Stockfish_(chess)),
[King Safety — Chessprogramming wiki](https://www.chessprogramming.org/King_Safety),
[Stockfish Evaluation Guide](https://hxim.github.io/Stockfish-Evaluation-Guide/),
[SF PR #2401 — Elo estimates for terms](https://github.com/official-stockfish/Stockfish/pull/2401),
[Lambergar (HCE, CCRL ~3209)](https://github.com/jabolcni/Lambergar).
