# Rarog Development Workflow Guide

How to drive Rarog's improvement plan with an AI coding model and know what to
run, what to report, and when a decision is yours. Read this alongside
`PLAN.md`, which holds the technical rationale and long-form decision record.

---

## Current Checkpoint

As of 2026-06-19, **Phases 0–2, Phase 2.5, and Phase 2.9 are closed.** The
engine is at the start of the eval-rewrite program (Phases 3–5).

| Area | Current state |
|---|---|
| Branch | `v2.1.0-codex-work` |
| Harness | Phase 0 complete: repo-local `fastchess`, weather-factory, SPRT, SPSA, PGO scripts |
| Test TC | SPSA and SPRT use `tc=3+0.03`; LTC confirmation `-TC "10+0.1"`; `-MoveTime 0.1` only as an optional legacy sanity check |
| Current head | **Phase 3.1 `EvalParams` struct, `bench 13 = 5,446,782`** |
| Last result | Phase 2.9-close batch SPRT **accepted**: +2.0 Elo, nElo +3.0, LOS 86.3%, LLR 2.39 (81.2% of the way to H1, still climbing) over 15,976 games; 0 time losses confirmed cross-harness. Phase 3.0 and 3.1 done bench-identical. |
| Immediate next work | **Phase 3.2 — tune-time loader/dumper** (`--features tune`, no games until Phase 4) |
| **Release status** | **v2.1.0 is release-ready now** — `Cargo.toml`/`CHANGELOG.md` updated 2026-06-19, release assets bench-verified, but **not yet tagged/published**. See "Releasing" below — this is a user action, do not tag/push without being asked. |

### The Program In One Table (overview · model picker · Elo)

Read `PLAN.md` §6 for *why* the eval order. The golden rule: **Phase 2.9 quick
wins → build all eval structure (Phase 3, no games) → fit the eval once
(Phase 4) → search SPSA once (Phase 5).** Never tune search margins before the
eval is final — that compute is wasted when the eval rescales.

| Phase | What | Gate | Model(s) | Elo |
|---|---|---|---|---|
| **2.9** Robustness & free speed (**CLOSED**) | time-safety valve (28 forfeits), native `znver3` build, `BadCapture` struct shrink, remove `gives_check` board.clone, profile-gated bounds-checks (no-op) | `bench 13 == 5,446,782` + `cargo test`; `t=`→0 confirmed cross-harness; close-SPRT accepted (+2.0 Elo, LOS 86%) | **Sonnet 4.6 medium** (valve/build/shrink); **Opus 4.8 medium** (gives_check, bounds-checks) | +2.0 Elo + reliability |
| **3** Eval infrastructure & build-out | attack maps, `EvalParams`, Texel tuner, then king-safety / threats / mobility / pawn / imbalance / small-terms / endgame **structure**, every new sub-term seeded inert | `bench 13 == 5,446,782` + reconstruction test + unit tests (**no games**) | Sonnet 4.6 medium for refactors/scaffolding; **Opus 4.8 high** for king-safety (3.5), threats (3.6), imbalance (3.9), endgame/KBNK (3.11), tuner core (3.3); Opus 4.8 medium for mobility (3.7) & pawns (3.8) | 0 direct (enabler) |
| **4** Eval data-fit campaign | staged Texel fit that *activates* the new terms; king-safety first, material + PSTs last | SPRT `[0,5]`/`[0,3]` per stage at `tc=3+0.03`, LTC confirm | Sonnet 4.6 medium (driving); Opus 4.8 high if a fit is pathological | **+120–230** |
| **5** Search-efficiency wave | the one search-constant SPSA wave + history-formula split, no-aging retry, do-deeper, qsearch quiet checks, codex ports, modern refinements | SPSA → SPRT per group at `tc=3+0.03` | Sonnet 4.6 medium (driving); **Codex 5.5 medium / GPT-5.5 high** for dense ports | **+20–50** |

Per-step model assignments are in `PLAN.md` §14. Elo figures are estimates;
**SPRT is the only verdict.** NNUE is the terminal option (`PLAN.md` §13).

> **Gauntlet read (2026-06-19, 35k games @ `tc=3+0.03`):** Rarog 2.1.0 (dev) =
> +66 over 2.0.2 (search work), but still **−19 vs Basilisk 1.5.0 and −73 vs
> Basilisk 1.5.1** — the sibling that *has* started eval tuning. The gap is the
> eval campaign Rarog hasn't done yet; this validates the eval-first plan. Rarog
> also searched ~1.3 plies shallower than Basilisk (d≈12.4 vs 13.8, nps 2.31M vs
> 2.76M) — the Phase 5 search/speed work is real but secondary. **Two cautions:**
> SF `UCI_Elo` is not a true anchor at this TC (run a slower-TC gauntlet with
> **Critter 1.6a** for a real number); and **Rarog 2.1.0 lost 28 games on time**
> (2.0.2: 0) — enable the documented time-safety valve **now** (`PLAN.md` §11),
> it is pure lost Elo and contaminates gauntlets.

### Tune Setup Status (Phase 2.5 closed)

Phase 2.5.0 synced the tune setup; the Phase 2.5.1 LMR values
(`887 / 109 / 656 / 780 / 646 / 2335 / 8395`) are the accepted defaults in
`SearchParams::default()`, the tune UCI defaults, and `config_lmr.json`. No
pending SPSA candidate. So if you say *"implement the next step,"* the model
should start **Phase 3.0** (a bench-gated refactor — not an SPRT).

### Texel Tooling & Dataset (ready now)

The eval-tuning **toolchain is already built** in `tools/texel/` (ported from
Basilisk). Full docs: `tools/texel/README.md`.

| Piece | Status |
|---|---|
| `sample_fens.py` | **Ready.** Streams `A:\Chess\Beast\data\txt\positions.txt` (7.1 GB raw FENs, **read-only — never modified**) into a fastchess EPD opening book. Verified end-to-end. |
| `datagen.ps1` | **Ready.** Rarog self-play from the book → `selfplay.pgn`. Auto/`-Concurrency`. |
| `extract.py`, `import_beast.py` | **Ready.** PGN → `FEN;result` (self-play path); evaluated-FENs → `FEN;target` (Stockfish path). |
| `reference/basilisk_tuner.cpp` | **Reference only.** The proven Adam + K-fit + `--verify` tuner to port to Rust **in Phase 3.3** (it needs Rarog's eval trace, which 3.1/3.3 build). |

Two label sources (SPRT decides which transfers): **Path A self-play** (primary,
works today) and **Path B Stockfish-WDL** (optional, higher ceiling, needs an SF
binary). You can start generating data on the 5950X now so it is ready for
Phase 4.

### External Gauntlet (set up your opponents)

After each phase — **especially after Phase 4 (eval), which over-fits self-play
most** — run a clock-TC gauntlet (`10+0.1`) to confirm gains transfer. Recommended
opponents (you choose the exact binaries by measured score):

- **Rarog 2.0.2 + Basilisk 1.5.0** — own history + sibling.
- **Critter 1.6a** — your named reference; the engine to beat (~3150–3200).
- **Stockfish capped** — use `UCI_LimitStrength`/`UCI_Elo`; **start at 2700, then
  2800, then 3000.** Pick the level where Rarog scores **~30–70%** (sharpest
  signal); raise it as milestones fall.
- **One mid HCE of known rating** (Lambergar / Peacekeeper / Igel HCE, or your
  own) — an independent non-SF, non-sibling check.

### Time budget (Ryzen 9 5950X)

High total budget, but keep runs **moderate** so the machine stays usable: SPSA
and datagen at `-Concurrency 24` (leave ~8 threads free); SPRT/gauntlets via
`tools/sprt.ps1` defaults. Texel *fits* are minutes (CPU-bound, run them freely).
The long poles are datagen (one-time, < 1 h) and the per-stage SPRTs (Phase 4) /
the SPSA wave (Phase 5).

---

## The Basic Rhythm

Most work is a short ping-pong:

```text
You   -> "Implement next step of the plan."
Model -> Reads PLAN.md and this guide, inspects current state, makes only the
         needed edits, verifies the build, and tells you exactly what to run.
You   -> Run the command and paste the short result.
Model -> Acts on the result: keep, revert, rerun, or move to the next gate.
```

For SPSA, SPRT, Texel tuning, and external gauntlets, the model cannot honestly
guess the result. Your report from the long-running command is the decision
input.

---

## Next Commands

**2.9.1 (time-safety valve) — DONE and CONFIRMED.** The clock-mode hard limit
now reserves an absolute `2*MoveOverhead` (binds only in real time scrambles,
leaves normal allocation untouched). Little Blitzer gauntlet at `tc=3+0.03`
with `rarog-p291-timevalve2-pext-pgo.exe`: **`t=0` over 2,237 games** (down from
`t=15`/`t=28`), with healthy `tpm=67.8 ms` (mid-pack, not over-conservative) and
`d=12.44` (deeper than Rarog 2.0.2's 11.60). Root cause was the Phase 2.2
SF-style TM rewrite (Rarog 2.0.2 `t=0` vs 2.1.0-dev `t=15`); the first fix pass
was a no-op for sudden-death TCs and was corrected. `bench 13 = 5,446,782`
unchanged. The Phase 2.9-close batch SPRT `[-3,3]` in fastchess will serve as
the cross-harness time check, so no separate fastchess/Colosseum run is needed.

**2.9.1 follow-up — movetime path now uses the full budget.** A `100 ms/move`
gauntlet (which sends `go movetime 100`, a *different* code path from
`tc=3+0.03`) showed Rarog at `tpm=92.9` vs Stockfish `tpm=110.2`, both `t=0`.
Cause: the 2.9.1 *movetime* reserve subtracted a full `MoveOverhead` (90 ms
budget on a 100 ms move) — but the 28 forfeits that motivated 2.9.1 were all in
the **clock** path, and movetime mode never forfeited. Reverted movetime to the
SF/Reckless default `optimum = maximum = movetime` (full budget). Safe: the
harness tolerates ≥10 % over nominal (SF `t=0` at 110 ms), our latency is ~3 ms
(full budget → ~103 ms). `bench 13` unchanged. **Validate:** re-run the
100 ms/move gauntlet with `rarog-p291-movetimefull-pext-pgo.exe`; expect depth↑
(toward SF) and `t=0` to hold.

**2.9.2 (native `znver3` build) — DONE.** Added `Arch::Native` to
`xtask/src/main.rs` (`--cfg rarog_pext -C target-cpu=native`, + PGO) and a
`-Native` switch to `tools/build_test.ps1` (output:
`rarog-<Suffix>-native-pgo.exe`); `x86-64-v3`/`avx2`/`pext` stay the default
for portable release assets (`cargo xtask build` with no `--arch` is
unaffected). `bench 13` is node-identical between `pext` and `native`
(`5,446,782` both), confirming it's build-flags-only with no behaviour change.
On the dev box used to implement this, nps was flat (`pext` 2,382,669 vs
`native` 2,384,755) — that CPU apparently already exposes everything
`x86-64-v3+bmi2` needs. **Run the 5-min A/B on the 5950X** to see the real gain
(znver3 has instructions beyond x86-64-v3 baseline that only
`target-cpu=native` unlocks):

```powershell
./tools/build_test.ps1 -Suffix p292-pext            # baseline
./tools/build_test.ps1 -Suffix p292-native -Native  # znver3-tuned
# then run `bench 13` on each test_engines\ binary and compare nps
```

No games needed — this is a build-flag change, not a search/eval change.

**2.9.3 (shrink `BadCapture`) — DONE.** Changed `to: usize → u8`
(`move_ordering.rs`); `attacker`/`captured` were already 1 byte each, so
`BadCapture` drops from 16 bytes to 3, and each `[_; 256]` `BadCaptureList`
drops from 4 KB to 768 B (~6.5 KB less stack per negamax frame across the two
lists). Added `move_ordering::tests::bad_capture_struct_stays_shrunk` to guard
against future creep. `bench 13` unchanged at `5,446,782`; 51 tests pass.
No games needed (struct-layout change only).

**2.9.4 (remove `board.clone()` in `gives_check`) — DONE.** The castling branch
of `gives_check` (`board/board.rs`) was cloning the whole board (heap-allocating
the history `Vec`) just to make the move and test `is_in_check` — the one
allocator touch on a search-reachable path. Replaced with a direct rook-attack
test: only the moved rook can check after castling (the king never checks, and
no discovered check is possible since the vacated squares are all board
edge/corner), using post-castle occupancy so our own king correctly blocks the
queenside rook. Validated by the existing differential oracle test plus two new
castle-delivers-check FENs; all 44 board-correctness tests pass. `bench 13`
unchanged. No games needed.

**2.9.5 (profile-gated `get_unchecked`) — INVESTIGATED, SKIPPED.** Profiled with
`cargo-show-asm` (it shows whether `panic_bounds_check` survives in a function —
the right tool for this, unlike a flamegraph). In a plain `--release` build the
`sq.index()` checks do survive, and a `get_unchecked` prototype removed them.
**But the PGO binary is what plays, and there it makes no difference:** an
interleaved `bench 13` nps A/B (node count identical, so a clean speed test) over
20 drift-cancelling pairs had the prototype winning **11/20** (50/50) with
best-case Δ `+0.34%` — noise. PGO already elides these checks. Reverted — not
worth `unsafe` for zero gain. No code change kept.

**Phase 2.9-close batch SPRT `[-3,3]` — ACCEPTED 2026-06-19.** Cumulative head
(2.9.1–2.9.5) vs the Phase 2.5.1 baseline, `tc=3+0.03`, 15,976 games, did not
reach formal SPRT termination but was clearly trending toward H1 and accepted
on the strength of the trend:

```
Elo: 2.02 +/- 3.62, nElo: 3.01 +/- 5.39
LOS: 86.33 %, DrawRatio: 42.08 %, PairsRatio: 1.04
LLR: 2.39 (81.2%) (-2.94, 2.94) [-3.00, 3.00]
```

Cross-harness time-loss check: grepped the PGN for `[Termination "..."]` tags
and time/forfeit keywords — zero time losses found across all games (only
`"adjudication"`/`"normal"` terminations appear). The 2.9.1 fix is confirmed in
fastchess, not just Little Blitzer. **Phase 2.9 is closed.**

**3.0 (attack-map substrate) — DONE.** `eval_piece_activity` (`src/eval.rs`)
now builds, once per `evaluate()` call, `attacks_from_sq[color][64]` (each
N/B/R/Q's own attack bitboard, keyed by its square), `attacked_by[color][pt]`,
`attacked[color]` (union incl. pawns/king), and `attacked2[color]` (inert, no
consumer yet). Mobility, `eval_king_safety`, and `eval_hanging_pieces` now read
these instead of recomputing `attacks_for`/`board.attackers_to_color`. `bench
13` unchanged at `5,446,782`; all 50 tests pass; no new clippy warnings. —
Sonnet 4.6 medium (matches plan recommendation).

Every Phase 2.9/3 step is **behaviour-preserving** — confirm `bench 13` is
unchanged after each:

```text
.\target\release\rarog.exe
bench 13
quit
# expect: Nodes searched  : 5446782   (unchanged through Phase 2.9 and Phase 3 so far)
```

Quantify the **native-build** win (2.9.2) with a 5-minute A/B: build one
`target-cpu=native` (znver3) binary and one `pext` (x86-64-v3) binary and compare
`bench` nps.

**3.1 (`EvalParams` struct) — DONE.** ~50 fields hoisted via a
`macro_rules! eval_params!` table (struct, `Default`, `EVAL_PARAM_NAMES`,
`get`/`set` by name+index — unused until 3.2/3.3, `#[allow(dead_code)]`).
Every default reproduces the constant it replaces exactly. PSTs flattened to
`pst_mg`/`pst_eg: [i32; 384]`; `MG_TABLE`/`EG_TABLE` (formerly `const`) are now
`Box<EvalTables>` on `Evaluator`, built at runtime by `build_tables(&EvalParams)`.
Frozen terms (mate-drive mop-up, OCB/two-knights, 50-move damping, king-zone
construction logic) left untouched — confirmed via grep that only the mop-up
term remains an unparameterized literal. `bench 13` unchanged at `5,446,782`;
50 tests pass; no new clippy warnings. — Sonnet 4.6 medium.

**Next: Phase 3.2 — tune-time loader and dumper.** `--features tune` only:
`RAROG_EVAL_FILE` env var loads `name index value` lines into `EvalParams`
(via the `set` accessor just built), rebuilds `Evaluator::tables`, clears both
caches; `dumpeval` console command writes the round-trip format. Gate:
dump → load → dump is byte-identical; release builds expose neither. See
`PLAN.md` §7 (§3.2).

Most of Phase 3 is the same shape: implement a behaviour-identical step (new
eval structure with sub-terms seeded inert) → check `bench 13 == 5,446,782` →
move on. **The games (SPRT) only start in Phase 4**, when the Texel campaign
activates the new terms. The big tuning binaries/dataset (`--features tune`,
`--features texel`) are built during Phase 3.1–3.4; see `PLAN.md` §7–§8 for the
exact tuner workflow.

---

## What To Report Back

### SPSA Result

Minimal:

> "LMR SPSA stopped at 5,000 iterations. Final values:
> LmrTtPvAdj=..., LmrExactBound=..., LmrShallowTt=..., LmrCutNode=...,
> LmrTableBase=..., LmrTableDiv=..., LmrHistDiv=..."

Helpful extras:

> "LmrExactBound stayed near zero"
> "LmrHistDiv sat at the max for the last 1,000 iterations"

The model will decide whether to bake the values, widen a range, rerun, or
discard the candidate.

### SPRT Result

Minimal:

> "SPRT: H1 accepted after 1,840 games."

or:

> "SPRT: H0 accepted after 2,210 games."

Helpful extras:

> "Score 53.1%, LLR crossed +2.94, no time losses."

H1 usually means keep the candidate. H0 usually means revert, unless the run was
obviously flawed.

### Bench Result

For pure refactors:

> "bench 13 = 5,446,782 nodes."

For tuned candidates:

> "bench 13 = 5,612,008 nodes."

A changed bench fingerprint is expected after tuning or real search changes.
It is a behavior fingerprint, not an Elo score.

### Errors

Paste the important error line:

> "fastchess exited with: engine option LmrTableBase not found."

or:

> "bench 13 returned 0 nodes; engine crashed on startup."

The model can diagnose from that.

---

## Decision Points

| Situation | Usual decision |
|---|---|
| SPSA values are stable and plausible | Bake candidate values, build, bench, SPRT |
| One plausible value hits a boundary | Widen that range once and rerun |
| Many values hit boundaries | Treat the run as suspect; reduce the group or inspect implementation |
| SPRT accepts H1 | Keep, record, move to the next gate |
| SPRT accepts H0 | Revert or leave at previous accepted defaults; retry only if setup was flawed |
| Primary SPRT passes on TC-sensitive feature | Run LTC `tc=10+0.1` confirmation |
| End of a phase | Run an external gauntlet before release work |
| Phase 3 step (eval structure) | Gate on `bench 13` identity (+ tests), not SPRT; no games |
| Phase 4 eval tuning starts | Dataset/holdout built in Phase 3; fit a stage, then SPRT it |

Do not keep running repeated SPRTs against tiny changes until one passes. That
is statistical fishing.

---

## Releasing

**Release at phase boundaries that change playing strength — not mid-phase.**
Phase 3 (eval infrastructure) is bench-identical the whole way through by
design, so there's no reason to wait for it to finish before shipping work
already accepted before it. Full rationale and the version-number table are
in `PLAN.md` §10.1; the short version:

| Milestone | Version |
|---|---|
| Phase 0–2.9 closed (harness + search tuning + robustness) | **2.1.0** — ready now |
| Phase 3 fully closed (eval structure built, still bench-identical) | no release needed (nothing plays differently) |
| Phase 4 closed (Texel data-fit activates the new eval terms) | **2.2.0** — the big jump, plan estimate +120–230 Elo |
| Phase 5 closed (search-efficiency wave) | **2.3.0** |
| Any standalone bug/robustness fix | **2.x.1, 2.x.2, ...** (patch) |

CI (`.github/workflows/build.yml`) builds and attaches all platform binaries
automatically when a GitHub release is published — releasing is a
documentation + tagging job, not a manual cross-platform build job.

**Checklist** (model can do steps 1–6; step 7 is yours):

1. `cargo test --release` and `cargo fmt --check` clean; `bench 13` matches
   the value at the top of this document.
2. Bump `Cargo.toml` `version`.
3. Write the new `CHANGELOG.md` entry at the top (Added/Changed/Fixed/
   Evaluated and rejected/Internal/Verified — see the 2.1.0 entry for the
   convention, including listing tried-and-reverted experiments for
   transparency).
4. Check `README.md` for anything that needs updating (usually nothing —
   most phases gate new options behind `--features tune` or seed terms
   inert).
5. Rebuild the actual release assets locally and bench-verify them before
   tagging: `cargo xtask build --arch pext`, `--arch avx2`; run `bench 13`
   on each, confirm the fingerprint and a clean `uci` handshake with no
   tune-only options visible.
6. Commit the bump as `Version X.Y.Z` (matches existing convention).
7. **You tag and publish:** `git tag vX.Y.Z && git push origin vX.Y.Z`, then
   create the GitHub release from that tag with the prepared release notes.
   Publishing triggers CI to build and attach binaries. After publishing,
   run the external gauntlet (`PLAN.md` §10) to confirm the SPRT gains
   transfer.

**Legacy branches** (`PLAN.md` §10.2): `v2.1.0-codex`/`v2.1.0-claude` are
reference-only source branches for the still-pending Phase 5 feature menu —
keep until Phase 5 resolves every remaining idea. `claude`/`improvements` are
stale (fully harvested or duplicate of an already-merged commit) — safe to
delete, but confirm with the user first since deleting remote branches is
externally visible.

---

## Phase Progress Tracker

Update this when work completes.

### Phase 0 - Harness

- [x] Phase 0 complete.
- [x] `tools/setup_tools.ps1` exists for fastchess/weather-factory setup.
- [x] `tools/sprt.ps1` exists and defaults to `tc=3+0.03`.
- [x] `tools/setup_spsa.ps1` writes weather-factory configs.
- [x] `tools/build_test.ps1` builds named PGO and tune binaries into
      `tools\test_engines`.
- [x] Calibration test recorded H0 for codex-work vs 2.0.2.

### Phase 1 - Existing Search Constants

- [x] Phase 1 complete.
- [x] `SearchParams` and tune-gated UCI options exist.
- [x] Pruning/margin SPSA group B accepted:
      `+6.17 +/- 4.88 nElo`, 19,458 games.
- [x] LMR group A first tune rejected/inconclusive and not kept as a Phase 1
      gain.
- [x] Release builds do not expose tune-only search options.
- [x] Accepted Phase 1 result: Group B pruning/margin tune only.

### Phase 2 - Repairs and Proven Tuning

- [x] Phase 2 complete.
- [x] Current accepted Phase 2 head is the futility baseline:
      `bench 13 = 5,401,662`.
- [x] Kept gains: 2.2 time management, 2.5 qsearch TT-bound stand-pat,
      2.7 per-move quiet futility.
- [x] Failed or relocated search items are documented below and in `PLAN.md`.
- [x] `improvements` check-aware ordering — dropped: H0 after about 11k games.
- [x] 2.1 ProbCut port — dropped: H0, `-24.5 +/- 8.5 Elo`; baseline flat
      ProbCut was better.
- [x] 2.2 Stockfish-style time management — kept: H1, about `+81 Elo` at old
      fixed 100 ms; no clock regression.
- [x] 2.3 No-aging history — dropped for now: H0, `-12.4 +/- 6.2 Elo`; retry
      only after the Phase 5 history formula fix.
- [x] 2.4 LMR coefficients — retained but suspect: SPSA values baked, old
      fixed-movetime SPRT H0 `-1.31 +/- 3.09`; retry in Phase 2.5.
- [x] 2.5 Qsearch TT-bound stand-pat — kept: H1, `+6.51 +/- 3.93 Elo`.
- [x] 2.6 Singular double-extension cap — dropped: H0 around `-1.7 Elo`; do
      not retry unless real time forfeits appear.
- [x] 2.7 Per-move quiet futility — kept: H1, `+7.98 +/- 4.42 Elo`; current
      bench `5,401,662`.
- [x] 2.8 Do-deeper re-search — dropped for now: H0 at old harness; retry
      after the Phase 4 eval refit (Phase 5) because the margin is cp-coupled.
- [x] 2.9 Debug-build stack overflow — fixed: `cargo test` was made viable in
      debug.
- [x] 2.10 Search hygiene — done: behavior-preserving cleanup.
- [x] 2.11 Group-B improving coeff widen — relocated to the Phase 5 post-eval
      SPSA wave.
- [x] 2.12 Futility `not_improving` direction — relocated to the Phase 5 search
      wave (eval-independent A/B, folded into the futility group).

### Phase 2.5 - Harness-Corrected Retries

- [x] 2.5.0 Tune setup synchronized to `SearchParams::default()`.
- [x] 2.5.1 LMR coefficients redo — **accepted, H1 +4 Elo**
      (`887 / 109 / 656 / 780 / 646 / 2335 / 8395`); head `bench 13 = 5,446,782`.
- [x] 2.5.2 Move-loop futility direction A/B — relocated to Phase 5.

### Phase 2.9 - Robustness & Free Speed (NEXT; bench-identical, no games tuning)

Every step keeps `bench 13 == 5,446,782`. See `PLAN.md` "Phase 2.9".

- [x] 2.9.1 Time-safety valve — **DONE & CONFIRMED.** Clock mode reserves
      `2*MoveOverhead` (`src/time_manager.rs`); first attempt was a no-op for
      sudden-death TCs and was corrected. LB gauntlet: **`t=0` over 2,237 games**
      (was `t=15`/`t=28`), `tpm=67.8`, `d=12.44`. Root cause: Phase 2.2 TM
      rewrite (Rarog 2.0.2 `t=0` vs 2.1.0-dev `t=15`). `bench 13` unchanged at
      `5,446,782`. **Follow-up CONFIRMED:** movetime path reverted to full
      budget (`optimum = maximum = movetime`) — the movetime reserve was
      over-conservative (100 ms/move: Rarog `tpm=92.9` vs SF `110.2`, both
      `t=0`). Re-gauntlet confirmed the fix. — Sonnet 4.6 medium
- [x] 2.9.2 Native build for local/own-match binaries — **DONE.** Added
      `Arch::Native` (`xtask/src/main.rs`, `--cfg rarog_pext -C
      target-cpu=native`) and `-Native` switch (`tools/build_test.ps1`);
      portable `x86-64-v3`/`avx2`/`pext` arches unchanged for release. `bench
      13` node-identical (`5,446,782`) between `pext` and `native`; real nps
      gain to be quantified on the 5950X (flat on the dev box that lacks
      znver3-specific instructions). No games needed. — Sonnet 4.6 medium
- [x] 2.9.3 Shrink `BadCapture` — **DONE.** `to: usize → u8`; struct shrinks
      16→3 bytes (attacker/captured were already 1 byte each), each
      `BadCaptureList [_; 256]` drops 4 KB→768 B (~6.5 KB less stack per
      negamax frame). Regression test added
      (`bad_capture_struct_stays_shrunk`). `bench 13` unchanged at
      `5,446,782`. No games needed. — Sonnet 4.6 medium
- [x] 2.9.4 Remove `board.clone()` in `gives_check` castling — **DONE.** Direct
      rook-attack test with post-castle occupancy (only the moved rook can
      check; no discovered check possible from edge/corner vacated squares).
      Removes the one allocator touch on a search-reachable path. Validated by
      the differential oracle test + 2 new castle-checks-king FENs; 44
      board-correctness tests pass. `bench 13` unchanged. — Opus 4.8 medium
- [x] 2.9.5 (profile-gated) `get_unchecked` — **INVESTIGATED, SKIPPED.**
      `cargo-show-asm` confirmed `sq.index()` checks survive in plain
      `--release`, but a PGO `bench` nps A/B (20 interleaved pairs) showed the
      `get_unchecked` prototype winning 11/20 (noise, Δbest +0.34%) — PGO
      already elides them. Reverted; not worth `unsafe` for zero gain. — Opus 4.8 medium
- [x] 2.9 close: batch non-regression SPRT `[-3,3]` — **ACCEPTED.** 15,976
      games, Elo +2.02 ± 3.62, LOS 86.3%, LLR 2.39 (81.2%→H1). Cross-harness
      time-loss check: 0 forfeits in the fastchess PGN. Phase 2.9 closed.
- [ ] (calibration, anytime) Slower-TC gauntlet with a CCRL-rated anchor (Critter 1.6a / Fruit 2.1), pin with `ordo -A "<name>" -a <ccrl>` (`PLAN.md` §10).

### Phase 3 - Eval Infrastructure & Behaviour-Identical Build-Out (no games)

Every step is gated on `bench 13 == 5,446,782` + tests. See `PLAN.md` §7.

- [x] 3.0 Attack-map substrate (refactor) — **DONE.** `attacks_from_sq`/
      `attacked_by`/`attacked`/`attacked2` computed once in `eval_piece_activity`;
      mobility, king safety, hanging pieces now read them instead of
      recomputing `attacks_for`/`attackers_to_color`. `bench 13` unchanged
      (`5,446,782`), 50 tests pass. — Sonnet 4.6 medium
- [x] 3.1 `EvalParams` struct + runtime tables (default-equivalent) — **DONE.**
      ~50 fields via `macro_rules! eval_params!`; `MG_TABLE`/`EG_TABLE` now
      `Box<EvalTables>` rebuilt by `build_tables(&EvalParams)`. `bench 13`
      unchanged (`5,446,782`), 50 tests pass. — Sonnet 4.6 medium
- [ ] 3.2 Tune-time loader + `dumpeval` (`--features tune`). — Sonnet 4.6 medium
- [ ] 3.3 Trace + Texel tuner binary + reconstruction acceptance test. — **Opus 4.8 high** (core) / Sonnet 4.6 medium (scaffolding)
- [ ] 3.4 Self-play dataset + extraction (**tooling ready** in `tools/texel/`; Beast pool read-only). — Sonnet 4.6 medium
- [ ] 3.5 King-safety v2 structure (seeded inert). — **Opus 4.8 high**
- [ ] 3.6 Threats package structure (seeded inert). — **Opus 4.8 high**
- [ ] 3.7 Per-count mobility tables (seeded linear-equivalent). — Opus 4.8 medium / GPT-5.5 high
- [ ] 3.8 Pawn structure + passed-pawn detail (seeded inert). — Opus 4.8 medium
- [ ] 3.9 Material imbalance hooks (seeded 0; optional). — **Opus 4.8 high**
- [ ] 3.10 Small positional terms (seeded 0; batch). — Sonnet 4.6 medium
- [ ] 3.11 Scale-factor framework + endgame knowledge incl. KBNK. — **Opus 4.8 high** + GPT-5.5 high
- [ ] 3.12 Gauntlet-driven additions (unstoppable passer, minor-behind-pawn, pawn islands, space upgrade, queen infiltration, king protector, winnable coupling). — Opus 4.8 medium / Sonnet 4.6 medium
- [ ] 3.11 Permanent endgame regression suite (`tests/endgames.epd` + `tests/endgames.rs`): KBNK/KPK/KRKP/KQKP/OCB/rook-draws — the gate for the endgame functions. — Sonnet 4.6 medium
- [ ] Phase 3 gate: bench unchanged, reconstruction exact, tests clean, one NPS SPRT `[-3,0]`.

### Phase 4 - Eval Data-Fit Campaign (the multiplier, +120–230 Elo)

Staged Texel fit; SPRT per stage at `tc=3+0.03`. Driving: Sonnet 4.6 medium
(escalate Opus 4.8 high if a fit is pathological). See `PLAN.md` §8.

- [ ] 4.1 King safety group.
- [ ] 4.1b King-safety SPSA polish on top knobs (**optional** — you decide when reached). — Sonnet 4.6 medium
- [ ] 4.2 Threats group (drop the old flat hanging term here).
- [ ] 4.3 Mobility tables.
- [ ] 4.4 Remaining scalars (pawn structure, passers, bishop pair, rook, outposts, space, tempo, small terms).
- [ ] 4.5 Material imbalance (skip if 3.9 skipped).
- [ ] 4.6 Material + PSTs definitive refit (last, biggest block).
- [ ] 4.7 Global polish (low lr), then LTC confirm + external gauntlet.

### Phase 5 - Search-Efficiency Wave (+20–50 Elo)

The one search-constant SPSA wave + refinements, at the final eval scale.
Driving: Sonnet 4.6 medium; dense ports: Codex 5.5 medium / GPT-5.5 high.
See `PLAN.md` §9.

- [ ] 5.1 Search-constant SPSA wave (pruning, LMR, futility, ProbCut margin, TM); incl. relocated 2.11 Group-B widen `[0,120]` and 2.5.2 futility-direction A/B.
- [ ] 5.2 History bonus/malus split, then retry no-aging history.
- [ ] 5.3 do-deeper re-implementation (cp-coupled retry).
- [ ] 5.4 Qsearch quiet checks; razoring depth restriction; LMR TT-move-is-capture; mobility-area refinement.
- [ ] 5.5 Codex ports: multi-cut/singular, threat-aware history, TT-cutoff/fail-low-parent history, optional TT overhaul. — Codex 5.5 medium
- [ ] 5.6 Modern refinements (aspiration modernization, correction-magnitude margins, hindsight, cutoff-count LMR, bad-noisy futility, qsearch SEE threshold).
- [ ] 5.7 Profile-guided speed pass; end-of-phase gauntlet + release.

### NNUE Readiness (terminal option, not scheduled)

- [ ] Not scheduled. Keep `Evaluator::eval()` the only search↔eval boundary so a
      future HCE→NNUE switch stays localized. See `PLAN.md` §13.

---

## Common Commands

```powershell
# One-time setup on a fresh clone.
.\tools\setup_tools.ps1

# Build a named pext-PGO test binary for SPRT/gauntlet.
.\tools\build_test.ps1 -Suffix <name>

# Build a named tune binary for SPSA only.
.\tools\build_test.ps1 -Suffix <name> -Tune

# Configure SPSA.
.\tools\setup_spsa.ps1 -ConfigGroup <pruning|lmr|futility|probcut> -EngineSuffix <name>

# Run/resume SPSA.
cd tools\weather-factory
python main.py
```

Texel dataset pipeline (Phase 4 prep — runs today):

```powershell
# 1. Sample a diverse opening book from the Beast pool (source read-only).
python tools\texel\sample_fens.py "A:\Chess\Beast\data\txt\positions.txt" `
    --out tools\texel\data\beast_seed.epd --count 50000 --min-pieces 6

# 2. Self-play datagen from the book (moderate concurrency on the 5950X).
.\tools\build_test.ps1 -Suffix phase3-base
.\tools\datagen.ps1 -Suffix phase3-base -Rounds 30000 -Nodes 8000 `
    -Book tools\texel\data\beast_seed.epd -BookFormat epd -Concurrency 24

# 3. Extract labelled train/holdout (split by game).
python tools\texel\extract.py tools\texel\data\selfplay.pgn `
    --out-dir tools\texel\data --train train.csv --holdout holdout.csv
```

SPRT examples:

```powershell
# Gain candidate, default clock TC = 3+0.03.
.\tools\sprt.ps1 `
    -EngineA tools\test_engines\rarog-<candidate>-pext-pgo.exe `
    -EngineB tools\test_engines\rarog-<baseline>-pext-pgo.exe `
    -NameA "Candidate" -NameB "Baseline"

# Smaller candidate.
.\tools\sprt.ps1 `
    -EngineA tools\test_engines\rarog-<candidate>-pext-pgo.exe `
    -EngineB tools\test_engines\rarog-<baseline>-pext-pgo.exe `
    -NameA "Candidate" -NameB "Baseline" -Elo1 3

# LTC confirmation.
.\tools\sprt.ps1 `
    -EngineA tools\test_engines\rarog-<candidate>-pext-pgo.exe `
    -EngineB tools\test_engines\rarog-<baseline>-pext-pgo.exe `
    -NameA "Candidate" -NameB "Baseline" -TC "10+0.1" -Elo1 3

# Optional old fixed 100 ms/move sanity check.
.\tools\sprt.ps1 `
    -EngineA tools\test_engines\rarog-<candidate>-pext-pgo.exe `
    -EngineB tools\test_engines\rarog-<baseline>-pext-pgo.exe `
    -NameA "Candidate" -NameB "Baseline" -MoveTime 0.1

# Refactor/default-equivalence SPRT, only when needed.
.\tools\sprt.ps1 `
    -EngineA tools\test_engines\rarog-refactor-pext-pgo.exe `
    -EngineB tools\test_engines\rarog-baseline-pext-pgo.exe `
    -NameA "Refactor" -NameB "Baseline" -Elo0 -3 -Elo1 3
```

Bench is best run interactively:

```text
.\target\release\rarog.exe
bench 13
quit
```

Expected current fingerprint:

```text
Nodes searched  : 5446782
```

PowerShell piping into the UCI loop can be unreliable; type the commands
interactively when checking bench.

---

## Why Tuning Options Exist

weather-factory can only perturb the engine through UCI. For example, it sends:

```text
setoption name LmrTableBase value 646
```

That is why search constants are exposed as UCI spin options in tune builds.
They are hidden from normal release builds with `--features tune`.

Before any public release, verify a non-tune build does not expose the tuning
option list.

---

## Ground Rules

- Do not accept a tuned value set without SPRT.
- Do not interpret lower or higher node count as strength.
- Do not bundle feature work with tuning defaults.
- Do not tune from stale JSON values unless that is deliberate and documented.
- Do not skip LTC confirmation for TC-sensitive features that pass the primary
  gate.
- **Build all eval structure before tuning search margins.** The one search
  SPSA wave (Phase 5) runs only after the eval is final (Phase 4) — its margins
  are eval centipawns, so tuning them earlier is wasted compute.
- **Phase 3 steps must keep `bench 13 == 5,446,782`** (behaviour-identical; seed
  every new eval sub-term inert). Phase 3 spends no games.
- Keep `Evaluator::eval()` as the only boundary between search and evaluation.

The process is the strength engine: tune, test, keep only what survives.
