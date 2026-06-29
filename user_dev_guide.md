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
| Branch | `development` (targets the 2.3.0 release; the `v2.1.0-codex-work` integration branch was squash-rebased onto `master` and retired 2026-06-20; `claude`/`improvements` were deleted, fully stale) |
| Harness | Phase 0 complete: repo-local `fastchess`, weather-factory, SPRT, SPSA, PGO scripts |
| Test TC | SPSA and SPRT use `tc=3+0.03`; LTC confirmation `-TC "10+0.1"`; `-MoveTime 0.1` only as an optional legacy sanity check |
| Current head | **PHASE 4 COMPLETE + GAUNTLET PASSED — v2.2.0** (`rarog-phase47-polish-pext-pgo.exe`, `bench = 4,747,104`). Now on branch `development`, **Phase 5 step 1 prep complete** (the one post-eval search SPSA wave is wired; games next). |
| Last result | **External gauntlet (2026-06-24, 2700 games @ `tc=10+0.1`):** Rarog 2.2.0 = **+240 H2H over 2.1.0**, beats both Basilisk siblings and all prior Rarogs; even with SF-cap-2800, loses only to Critter 1.6a / SF-cap-2900. **CCRL ≈ 3000.** ~75% of the staged self-play gain transferred. Critter's LB time-forfeit was an LB artifact — clean in fastchess (`timemargin=1000`). |
| Immediate next work | **Phase 5 step 1 prep COMPLETE (code-only, no games yet):** widened `FutilityNotImproving`/`LmpNotImproving` ceilings to `[0,120]`; exposed `ProbCutMargin` `[60,400]`, the **futility-direction A/B** (`FutilityImprovingDir` 0/1), the **`LazyMargin`** option `[200,2000]`, and the **7-param TM group** (`TmOptScale`/`TmFallBase`/`TmFallSlope`/`TmInstabBase`/`TmInstabSlope`/`TmEffortHigh`/`TmEffortLow`, ×10000-scaled). New `config_tm.json` + `config_lazymargin.json`; `setup_spsa.ps1` groups `tm`/`lazymargin` wired; SPSA README documents all three. Bench still `4,747,104` (no-op), 159/159 tests pass, fmt clean, tune-only options hidden in release. **Next: you run the SPSA/A-B/SPRT gates** — see "Next Commands" below. |
| **Release status** | **v2.2.0 published.** Branch `v2.3.0` targets the next release, **2.3.0**, after Phase 5 closes. See "Releasing". |

### The Program In One Table (overview · model picker · Elo)

Read `PLAN.md` §6 for *why* the eval order. The golden rule: **Phase 2.9 quick
wins → build all eval structure (Phase 3, no games) → fit the eval once
(Phase 4) → search SPSA once (Phase 5).** Never tune search margins before the
eval is final — that compute is wasted when the eval rescales.

| Phase | What | Gate | Model(s) | Elo |
|---|---|---|---|---|
| **2.9** Robustness & free speed (**CLOSED**) | time-safety valve (28 forfeits), native `znver3` build, `BadCapture` struct shrink, remove `gives_check` board.clone, profile-gated bounds-checks (no-op) | `bench 13 == 5,446,782` + `cargo test`; `t=`→0 confirmed cross-harness; close-SPRT accepted (+2.0 Elo, LOS 86%) | **Sonnet 4.6 medium** (valve/build/shrink); **Opus 4.8 medium** (gives_check, bounds-checks) | +2.0 Elo + reliability |
| **3** Eval infrastructure & build-out | attack maps, `EvalParams`, Texel tuner, then king-safety / threats / mobility / pawn / imbalance / small-terms / endgame **structure**, every new sub-term seeded inert | `bench 13` stable except two documented re-baselines (`5,446,782` → `5,354,975` at 3.11b KPK; → **`4,978,006`** at 3.14 eval-cache fix) + reconstruction test + `cache==cold` test + unit tests (**no games**) | Sonnet 4.6 medium for refactors/scaffolding; **Opus 4.8 high** for king-safety (3.5), threats (3.6), imbalance (3.9), endgame/KBNK (3.11), tuner core (3.3); Opus 4.8 medium for mobility (3.7) & pawns (3.8) | 0 direct (enabler) |
| **4** Eval data-fit campaign | staged Texel fit that *activates* the new terms; king-safety first, material + PSTs last | SPRT `[0,5]`/`[0,3]` per stage at `tc=3+0.03`, LTC confirm | Sonnet 4.6 medium (driving); Opus 4.8 high if a fit is pathological | **+120–230** |
| **5** Search-efficiency wave | the one search-constant SPSA wave + history-formula split, no-aging retry, do-deeper, qsearch quiet checks, codex ports, modern refinements | SPSA → SPRT per group at `tc=3+0.03` | Sonnet 4.6 medium (driving); **Codex 5.5 medium / GPT-5.5 high** for dense ports | **+20–50** |
| **6** Non-NNUE ceiling (optional) | eval-refresh cycles on data from the stronger head + ride-along structural eval items (shelter/storm→danger); the multi-cycle grind | one joint refit + SPRT per cycle at `tc=3+0.03`; gauntlet between cycles | Sonnet 4.6 medium (driving); Opus 4.8 high for the structural eval change | **+10–40/cycle** |

Per-step model assignments are in `PLAN.md` §15. Elo figures are estimates;
**SPRT is the only verdict.** NNUE is the terminal option (`PLAN.md` §14).

> **Gauntlet read (2026-06-19, 35k games @ `tc=3+0.03`):** Rarog 2.1.0 (dev) =
> +66 over 2.0.2 (search work), but still **−19 vs Basilisk 1.5.0 and −73 vs
> Basilisk 1.5.1** — the sibling that *has* started eval tuning. The gap is the
> eval campaign Rarog hasn't done yet; this validates the eval-first plan. Rarog
> also searched ~1.3 plies shallower than Basilisk (d≈12.4 vs 13.8, nps 2.31M vs
> 2.76M) — the Phase 5 search/speed work is real but secondary. **Two cautions:**
> SF `UCI_Elo` is not a true anchor at this TC (run a slower-TC gauntlet with
> **Critter 1.6a** for a real number); and **Rarog 2.1.0 lost 28 games on time**
> (2.0.2: 0) — enable the documented time-safety valve **now** (`PLAN.md` §12),
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

> **CURRENT (2026-06-29) — Phase 5 step 1 prep is complete; the next runs are yours.**
> Everything below this box is the historical Phase 2.9/3 log, kept for record.
>
> The whole step-1 wave is wired behind `--features tune`. Build the tune binary once:
>
> ```powershell
> ./tools/build_test.ps1 -Suffix phase5-tune -Tune
> ```
>
> Then run the groups **one at a time**, each: SPSA → bake the values into
> `SearchParams::default()` → build a `pext --pgo` binary → SPRT vs the current
> head (`rarog-phase47-polish-pext-pgo.exe`) at `tc=3+0.03`, keep only on H1.
>
> ```powershell
> # SPSA groups (run, then report the final values back):
> ./tools/setup_spsa.ps1 -ConfigGroup pruning  -EngineSuffix phase5-tune
> ./tools/setup_spsa.ps1 -ConfigGroup lmr      -EngineSuffix phase5-tune
> ./tools/setup_spsa.ps1 -ConfigGroup futility -EngineSuffix phase5-tune
> ./tools/setup_spsa.ps1 -ConfigGroup probcut  -EngineSuffix phase5-tune
> ./tools/setup_spsa.ps1 -ConfigGroup tm       -EngineSuffix phase5-tune   # clock-only; LTC-confirm
> cd tools\weather-factory; python main.py    # stop with Ctrl-C when stable
> ```
>
> Two **non-SPSA** gates, do whenever convenient in this step:
> - **`FutilityImprovingDir` A/B** (`[-3,3]`): two fastchess engine configs, one
>   with `option.FutilityImprovingDir=1`, one `=0`; keep the winner (default 0 if
>   neither wins). See `tools/spsa_configs/README.md`.
> - **`LazyMargin` safety check** (`[-3,3]`): widen 600 → ~900/1200, confirm no
>   regression at the post-Phase-4 eval scale, *then* run the `lazymargin` SPSA.
>
> Report SPSA final values / SPRT verdicts back and I'll bake, gate, and move on.

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
# expect: Nodes searched  : 4978006   (current head, 3.14+; was 5354975 at 3.11b-3.12)
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

**3.2 (tune-time loader/dumper) — DONE.** `EvalParams::load_from_env`/
`load_from_str`/`dump` (`src/eval.rs`, `#[cfg(feature = "tune")]`):
`RAROG_EVAL_FILE` loads `name index value` lines (unknown name = hard
`panic!`, omitted fields keep defaults); `Evaluator::default()` calls it under
`tune` instead of `EvalParams::default()` — fresh construction already gives
fresh tables/caches, no separate reload path needed. `dumpeval` console
command (`src/uci_protocol.rs`) prints the round-trip format, gated so plain
release builds report it as an unknown command (verified). Round-trip
verified by hand (dump → edit → reload → dump, byte-identical) and by 3 new
`eval::tune_tests` unit tests (run with `cargo test --release --features
tune`). `bench 13` unchanged; no new clippy warnings in either build. —
Sonnet 4.6 medium.

**3.3 (trace + Texel tuner) — DONE.** Under `--features texel` the eval emits
a per-position `EvalTrace`; `tools/texel-tuner` (binary `rarog-texel`) ports
the Basilisk tuner (K-fit + Adam + group masks + `--verify` + clamps),
parallelised with std threads. Reconstruction is exact (unit test over >5 k
random positions). `bench 13` unchanged in production builds. Run:
`cargo run --release -p texel-tuner -- --verify <holdout.csv>` and
`... --tune material <train.csv> <holdout.csv> <out.txt>`. — Opus 4.8.

**Next: Phase 3.4 — self-play dataset (you run the games).** This step
generates training data; no engine code changes. The exact commands are in the
"Phase 3.4" tracker entry below and in `tools/texel/README.md`. **I prepare the
tooling and hand you the commands; I do not run the long datagen/SPRT myself.**
Then Phase 3.5 (king-safety v2) resumes bench-identical eval work.

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

> "bench 13 = 4,978,006 nodes."  (current head, 3.14+; was 5,354,975 at 3.11b-3.12)

For tuned candidates:

> "bench 13 = 5,612,008 nodes."

A changed bench fingerprint is expected after tuning or real search changes.
It is a behavior fingerprint, not an Elo score.

### Texel / Tuner Result (Phase 4)

Minimal:

> "King-safety fit: train loss 0.0974 → 0.0961, holdout 0.0979 → 0.0969."

Helpful extras (loss is never the verdict — SPRT is — but these tell the model
whether the fit is trustworthy before games are spent):

> "Bucket losses all held except pawn-endings (0.082 → 0.085); feature-support
> flagged the new safe-check[Q] weight with only 24 observations; safety-curve
> stayed monotonic under the shape constraint."

A regressing bucket, a sparse-feature warning, or an implausible sign/shape is a
stop-and-investigate signal **before** spending SPRT games.

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
in `PLAN.md` §11.1; the short version:

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
   run the external gauntlet (`PLAN.md` §11) to confirm the SPRT gains
   transfer.

**Legacy branches** (`PLAN.md` §11.2): `v2.1.0-codex`/`v2.1.0-claude` are
reference-only source branches for the still-pending Phase 5 feature menu —
keep until Phase 5 resolves every remaining idea. `claude`/`improvements`
were stale and have already been deleted (2026-06-20), along with
`v2.1.0-codex-work` itself, which was squash-rebased onto `master` instead of
kept as a separate integration branch.

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
- [ ] (calibration, anytime) Slower-TC gauntlet with a CCRL-rated anchor (Critter 1.6a / Fruit 2.1), pin with `ordo -A "<name>" -a <ccrl>` (`PLAN.md` §11).

### Phase 3 - Eval Infrastructure & Behaviour-Identical Build-Out (no games)

Every inert step is gated on `bench 13` stability + tests. Two documented
re-baselines: `5,446,782` → `5,354,975` at 3.11b (KPK bitbase), then →
**`4,978,006`** at 3.14 (eval-cache correctness fix — the eval is now pure).
See `PLAN.md` §7.

- [x] 3.0 Attack-map substrate (refactor) — **DONE.** `attacks_from_sq`/
      `attacked_by`/`attacked`/`attacked2` computed once in `eval_piece_activity`;
      mobility, king safety, hanging pieces now read them instead of
      recomputing `attacks_for`/`attackers_to_color`. `bench 13` unchanged
      (`5,446,782`), 50 tests pass. — Sonnet 4.6 medium
- [x] 3.1 `EvalParams` struct + runtime tables (default-equivalent) — **DONE.**
      ~50 fields via `macro_rules! eval_params!`; `MG_TABLE`/`EG_TABLE` now
      `Box<EvalTables>` rebuilt by `build_tables(&EvalParams)`. `bench 13`
      unchanged (`5,446,782`), 50 tests pass. — Sonnet 4.6 medium
- [x] 3.2 Tune-time loader + `dumpeval` (`--features tune`) — **DONE.**
      `EvalParams::load_from_env`/`dump`, `RAROG_EVAL_FILE` round-trip
      verified byte-identical; `dumpeval` unrecognized in plain release
      builds. `bench 13` unchanged (`5,446,782`), 50 tests + 3 new tune-only
      tests pass. — Sonnet 4.6 medium
- [x] 3.3 Trace + Texel tuner binary + reconstruction acceptance test — **DONE.**
      `--features texel` trace machinery in `src/eval.rs`; `tools/texel-tuner`
      (`rarog-texel`) ports the Basilisk tuner (K-fit/Adam/masks/`--verify`/
      clamps, std-thread parallel). Reconstruction exact (>5 k random
      positions); `bench 13` unchanged in production. — Opus 4.8
- [x] 3.4 Self-play dataset + extraction — **DONE 2026-06-22.** 330,000
      self-play games (phase3-base PGO binary, node-limited) → `extract.py`
      produced **2,190,548 train + 116,112 holdout** unique positions (≥1.5M
      target met), tuner `--verify` PASS (10,000/10,000 reconstruct exactly).
      Pipeline (`sample_fens.py` → `datagen.ps1` → `extract.py` → `--verify`)
      all run by the user; outputs in `tools/texel/data/`. — Sonnet 4.6 medium
- [x] 3.5 King-safety v2 — **DONE.** Single `danger` accumulator; weak ring,
      safe checks (per type), king-flank pressure, pawnless flank, queen-relief
      added seeded 0; conversion table lengthened 16→40 with the `.min(15)` cap
      removed (Texel-tunable tail). Bench `5,446,782` unchanged, reconstruction
      exact. Blockers/pins deferred to Phase 5 (needs pin masks). Danger-input
      weights are SPSA (Phase 5), not Texel. — Opus 4.8
- [x] 3.6 Threats package structure — **DONE.** threat_by_minor/rook (per
      victim), hanging-refined, safe-pawn-push, weak-piece, restricted-squares
      — all seeded 0, traced, added to the tuner `threats` group. Overloaded
      defender deferred. Bench `5,446,782` unchanged, reconstruction exact. — Opus 4.8
- [x] 3.7 Per-count mobility tables — **DONE.** `mob_{n,b,r,q}_{mg,eg}` one-hot
      tables seeded `i·old_weight`; eval indexes by safe-mobility count, traces
      one-hot. Tuner `mobility` group = 8 tables (clamped, non-decreasing).
      Bench `5,446,782` unchanged, reconstruction exact. — Opus 4.8 medium
- [x] 3.8 Pawn structure + passed-pawn detail — **DONE.** Rank-scaled connected
      (per-rank table, seeded constant); pawn levers, doubled-isolated,
      blocked-passer, ideal-blockader added seeded 0 (latter two in a new
      `eval_passer_blockade`). Added to tuner pawnstruct/passers groups. Bench
      `5,446,782` unchanged, reconstruction exact. Candidate-majority and full
      promotion-path passer-safety deferred to Phase 4.4. — Opus 4.8 medium
- [x] 3.9 Material imbalance hooks — **DONE.** SF-style quadratic form,
      `imbalance_ours[36]`/`imbalance_theirs[36]` seeded 0, phase-independent,
      no `/16` divisor (kept exactly linear → Texel-tunable). New tuner
      `imbalance` group (72 params). Bench `5,446,782` unchanged, reconstruction
      exact. — Opus 4.8
- [x] 3.10 Small positional terms (seeded 0; batch) — **DONE.** Bishop-pair
      pawn-scaling, bishop outposts, trapped rook, connected rooks, bishop
      long-diagonal-on-king, bad bishop, initiative/complexity, plus the two
      optional terms (closedness, central-king/lost-castling danger). New
      tuner `smallpos` group (16 params). Bench `5,446,782` unchanged,
      reconstruction exact. — Sonnet 4.6
- [x] 3.11a Scale-factor framework + KBNK corner-drive + endgame suite — **DONE (Opus 4.8).** `ScaleFactor` framework (`SCALE_NORMAL=64`, `scale_endgame`/`specialized_endgame_scale`), pawnless insufficient-material draws (KK/KNK/KBK/minor-vs-minor → dead draw), and the **KBNK corner-drive** (drives the bare king to the bishop's own-colour corner). Found & fixed an inverted corner-colour mapping (this engine puts a1 in `LIGHT_SQUARES`). Bench `5,446,782` unchanged, reconstruction exact.
- [x] 3.11b KPK bitbase + KBP wrong-corner draw — **DONE (Opus 4.8).** Exact KPK via a generated bitbase (`src/kpk.rs`): drawn KPK → `0`, won KPK falls through to normal eval. KBP wrong-coloured-bishop rook-pawn draw. **Re-baselined the bench fingerprint `5,446,782 → 5,354,975`** (KPK is reachable in the bench tree; correct draw recognition prunes faster — done with your sign-off).
- [x] 3.11c Endgame heuristics — **DONE (Opus 4.8), bench-neutral `5,354,975`.** KQKP fortress draw (rook/bishop pawn only — knight/centre pawns stay wins), conservative KRKP partial scale (never a forced draw), and OCB passed-pawn refinement. Four correctness unit tests + a KQKP-fortress EPD line. **Deliberately excluded:** KQ-vs-KR (a win → not scaled) and broad rook-endgame drawishness (a tunable Phase-4 term). *A first GPT-5.5 attempt shipped a broad rook scaler that collapsed bench ~29% and scaled won rook endings toward draw; it was dropped and reimplemented narrowly.*
- [x] 3.12 Gauntlet-driven additions (core) — **DONE (Opus 4.8).** Unstoppable passer (rule of square, eg), minor-behind-pawn, pawn islands, queen infiltration, king protector, and the SF-style piece-weighted space term — all seeded 0, new tuner `gauntlet` group (10 params). Bench `5,354,975` unchanged, reconstruction exact. *Deferred:* the optional low-yield trio (bishop x-ray, R+Q battery, slider-on-queen) and the winnable/complexity coupling (cross-term design → Phase 4).
- [x] 3.13a Endgame regression suite harness (`tests/endgames.epd` + `tests/endgames.rs`) — **DONE (Opus 4.8, with 3.11).** KBNK-mate playout + insufficient-material-draw cases + corner-direction test.
- [x] 3.13b Extend the suite — **DONE (Opus 4.8).** Added a `win` verdict (won endings score clearly, not zeroed) plus more KPK draws/wins and a rook-pawn KQKP fortress to `tests/endgames.epd`. KRKP/OCB partial-scales are covered by the `src/eval.rs` unit tests (noted in the EPD).
- [x] 3.14 Eval-cache correctness fix — **DONE (Opus 4.8).** Root cause was **not** the `eval_table` key (it's complete) but the **pawn cache**: the passed-pawn free/safe-stop bonuses depend on non-pawn occupancy yet were scored inside `eval_pawns`, cached by a pawn-only key. Moved them to `eval_passed_pawn_advance` (run every eval, outside the cache), keeping the eval value byte-identical (0 diffs on a fresh-evaluator walk) — only the cache is now exact. Bench re-baselined `5,354,975 → 4,978,006`; eval is now pure (`bench` identical cache-on vs cache-off). Permanent guard `tests/eval_cache.rs` (fails pre-fix, passes after); reconstruction stays exact.
- [x] 3.15 Eval inert-block gating — **INVESTIGATED & REJECTED.** Micro-opt had no headroom (loops already compiler-tight; a hand attempt *regressed* it). Inert-block gating recovered +15 % NPS byte-identically but **only at the seeded-0 head** — it does nothing once Phase 4 tunes the weights nonzero, so it's throwaway scaffolding for a gate we don't care about. Reverted. The durable lever is 3.16.
- [x] 3.16 Lazy eval — **ACCEPTED (Opus 4.8), +4.4 Elo.** Skip the expensive block (piece activity + imbalance) when the material+PST margin > `LAZY_MARGIN` (600, SPRT-tunable); the mop-up is extracted to `apply_mop_up` and runs on **both** paths so KBNK/KXK mating survives. Disabled under `--features texel` (tuner fits full eval); eval stays pure. Bench re-baselined `4,978,006 → 5,315,678`; per-node NPS ~2.50M→2.80M. **SPRT lazy-on vs lazy-off: +4.4 ± 3.9 Elo, LOS 98.7 %, H1, 15,314 games.**
- [x] Phase 3 gate — **MET / superseded.** ✅ reconstruction exact; ✅ per-term assertions; ✅ `cache==cold`; ✅ tests clean; ✅ lazy-eval non-regression (3.16, +4.4 Elo). The original vs-`p25` NPS SPRT is superseded (can't pass at seeded-0 — new terms are pure overhead until Phase 4 tunes them); real vs-`p25` check is the Phase-4 boundary. **Phase 3 is closed.**

### Phase 4 - Eval Data-Fit Campaign (the multiplier, +120–230 Elo)

Staged Texel fit; SPRT per stage at `tc=3+0.03`. Driving: Sonnet 4.6 medium
(escalate Opus 4.8 high if a fit is pathological). See `PLAN.md` §8.

- [x] 4.0 Tuner/data readiness gate — **DONE (autonomous parts).** Nonlinear king-safety support, feature-support diagnostics, bucketed holdout + targeted-data policy, phase-balanced sampling, blended-label support, regularization/shape constraints all built (sub-items below). Regen-dependent items (balanced sampling / blended labels) need a new datagen pass to bite; binary feature cache deprioritized (fast loads). Current 2.19M set suffices for Stage 4.1. (PLAN.md §8 Step 4.0).
  - [x] **Feature-support diagnostics built & run** (Opus 4.8). New `rarog-texel --feature-support <data.csv> [--max-positions N]` counts, per weight, how many positions can give it gradient signal (with phase breakdown), flagging any active in `< max(200, 0.05%·N)`. Run on the full 2.19M train set → three groups: **(1)** structural always-zeros to freeze forever (king material, pawn PST ranks 1/8, passer ranks 1/8); **(2)** all 11 nonlinear king-safety units (`king_safety_unit_*`, `ks_safe_check_*`, `ks_weak_ring`, `ks_queen_relief`, `ks_flank_attack`, `ks_pawnless_flank`) are **0** in the linear trace — they enter via the danger²→table lookup, so 4.1 must fit them by finite-difference/SPSA, **not** the linear gradient; **(3)** genuinely rare even at 2.19M — `pawn_lever`, `trapped_bishop`, `rook_trapped` (~700–1040 obs) → freeze at hand value. 153 weights under the cut total.
  - [x] **Bucketed-holdout reporter built** (Opus 4.8). Ten buckets: phase (open/mid/end), material (no-queens, OCB, rook-ending, pawn-ending), and king-attack / passer / threat. `rarog-texel --buckets <data.csv>` snapshots current-eval per-bucket loss (baselines: opening 0.160 noisiest, endgame 0.083 settled, pawn-ending 0.123 thin at 2.9k); **every `--tune` now prints a base→final per-bucket table** flagging any bucket that `<-- REGRESSED`. A stage is clean only if no bucket regresses — investigate *before* spending SPRT games.
  - [x] **Nonlinear king-safety fit path built** (Opus 4.8). The 11 danger-index inputs are invisible to the linear trace (they move the table *index*, not a coefficient). `rarog-texel --tune-kingsafety <train> <holdout> [out] [--epochs N] [--max-positions N]` **re-evaluates** positions with perturbed weights (texel-gated `Evaluator::set_params`) and co-tunes the 11 inputs + 40-entry safety table by integer coordinate descent (shrinking step; clamps keep the table non-decreasing, inputs ≥0). Smoke fit (6 epochs, 60k): dead inputs activated sensibly, table tail rose into the danger² shape, every bucket improved (holdout −0.00073, no regressions). This is the **engine for Stage 4.1** — the real full-data run + SPRT is 4.1 itself. Production bench unaffected (`set_params` is texel-only; `bench` = 5,315,678).
  - [x] **Regularization / shape constraints** (Opus 4.8). Shape constraints already in `clamp_weights` (monotonic safety table + passer bonuses, non-negative penalties/danger inputs, pinned king material). Added **L2-to-prior**: `--tune … --l2 <λ>` shrinks each active weight toward its hand-tuned default. It's a *guard* (off by default), not a default win — on well-supported groups it just pulls toward prior; use gentle λ (1e-6…2e-5) on sparse/suspect terms and confirm holdout holds.
  - [x] **Phase-balanced sampling capability** (Opus 4.8). `extract.py` now computes game phase (faithful to engine `PHASE_W`), prints train/holdout phase mix, and takes `--balance-phase R` to cap over-represented phase buckets. Domain (king-attack/passer/threat) balancing stays post-hoc via the bucketed reporter + targeted `sample_fens.py`. **Regen-dependent** — only bites on a new datagen pass (current set suffices for 4.1).
  - [x] **Blended labels** — no tuner change needed: `parse_target` already accepts any float in `[0,1]`, so `fen;0.62` works once datagen emits a WDL/score column. **Regen-dependent.**
  - [ ] Binary feature cache — **deprioritized** (measured load ~1–2 s, not the bottleneck the rationale assumed; revisit only if reruns stall).
- [x] 4.1 King safety group — **✅ ACCEPTED +42.5 Elo (Opus 4.8).** `--tune-kingsafety` on the full 2.19M set: holdout 0.10189→0.10105 (−0.00084), every bucket improved (opening −0.00175, king-attack −0.00094, passer −0.00093). Dead inputs activated (`ks_safe_check_queen 0→16`, `ks_pawnless_flank 0→12`, etc.), safety-table tail lifted 118→240–366 (monotonic). **SPRT KSafety41 vs Phase3Lazy: +42.47 ± 13.45 Elo, LOS 100%, LLR 2.95, H1 at 1266 games.** New head = `rarog-phase41-ksafety-pext-pgo.exe`; **bench 5,178,378**.
- [ ] 4.1b King-safety SPSA polish on top knobs (**optional** — you decide when reached). — Sonnet 4.6 medium
- [x] 4.2 Threats group — **✅ ACCEPTED +45.2 Elo (Opus 4.8).** Tuned `threats` + the old flat hanging term jointly (`threats42` group) so the fit resolves their overlap. 500 epochs, full set: holdout 0.10104→0.10004 (−0.00100), every bucket improved. The fit **drove the flat hanging term to ~0 data-driven** (minor 45→0, rook 60→2, queen 80→2 — the refined term absorbed it); per-victim threat tables activated. **SPRT Threats42 vs KSafety41: +45.22 ± 11.16 Elo, LOS 100%, H1 at 2032 games.** New head = `rarog-phase42-threats-pext-pgo.exe`; **bench 5,144,732**.
- [x] 4.3 Mobility tables — **✅ ACCEPTED +24.1 Elo (Opus 4.8).** Tuned `mobility` (132 params) on the 4.2 head. The bucketed reporter caught a **rook-ending regression at the full 400-epoch fit** (overvaluing active rooks in drawish endings); L2 couldn't fix it surgically, so early-stopped at the **clean 250-epoch boundary** — every bucket holds/improves, global 0.10004→0.09936 (−0.00068). Curves monotonic & SF-shaped. **SPRT Mobility43 vs Threats42: +24.07 ± 7.94 Elo, LOS 100%, H1 at 3716 games** (the clean fit cost no Elo). New head = `rarog-phase43-mobility-pext-pgo.exe`; **bench 5,181,289**.
- [x] 4.4 Remaining scalars — **✅ ACCEPTED +85.2 Elo (Opus 4.8).** New `scalars44` group (93 params: pawn structure, passers, rook files/7th, minors, space/tempo, small terms, gauntlet), excluding mobility/threats/hanging (done) and freezing the 3 sparse pairs. 700 epochs: holdout 0.09933→0.09644 (−0.00289, biggest stage), every bucket improved. A few terms (rook_7th, space, king_protector) fitted to 0 — data verdict. **SPRT Scalars44 vs Mobility43: +85.20 ± 18.75 Elo, LOS 100%, H1 at 678 games** (biggest stage of the campaign). New head = `rarog-phase44-scalars-pext-pgo.exe`; **bench 5,121,269**.
- [x] 4.5 Material imbalance — **✅ ACCEPTED +26.7 Elo (Opus 4.8).** Tuned the SF-style imbalance quadratic on the 4.4 head, 700 epochs: holdout 0.09640→0.09527 (−0.00113), 9/10 buckets improved (pawn-ending −0.0034) **with a deliberate, user-approved OCB regression +0.00048** (no clean fit exists — OCB drawishness is scaling, not material). **SPRT Imbalance45 vs Scalars44: +26.66 ± 8.49 Elo, LOS 100%, H1 at 3408 games** — the bet paid off. New head = `rarog-phase45-imbalance-pext-pgo.exe`; **bench 5,448,086**.
- [x] 4.6 Material + PSTs definitive refit — **✅ ACCEPTED +27.6 Elo (Opus 4.8).** Material + 768 PST entries (~778 params) on the 4.5 head, 400 epochs: holdout 0.09507→0.09352 (−0.00156), every bucket improved (pawn-ending −0.0037; OCB recovered −0.0017). Values sane — structural zeros held, material ratios unchanged (mg ~×1.1 scale shift to match K=1.70). **L2-to-PeSTO tested but froze the fit near prior (all-or-nothing), so shipped without it.** One sanity test flipped 3cp (enemy king was adjacent to the advanced pawn — new eval correctly discounts that); fixed the test. **SPRT Pst46 vs Imbalance45: +27.64 ± 11.23 Elo, LOS 100%, H1 at 2116 games.** New head = `rarog-phase46-pst-pext-pgo.exe`; **bench 5,794,671**.
- [x] 4.7 Global polish — **✅ ACCEPTED +65.0 Elo (Opus 4.8).** Low-lr joint fit (`all47` = everything linearly tunable, 3 sparse pairs frozen; 1172 params, lr 0.1) on the 4.6 head: holdout 0.09359→0.09273 (−0.00086), every bucket improved — the joint optimum captured cross-group gains the staging left behind. **Baked via new `tools/texel/bake_params.py` (1006 params, 124 fields), verified by bench-match** (tune binary on the dump = baked normal build = 4,747,104 exactly). Removed the now-complete `seeded_zero` gate. **SPRT Polish47 vs Pst46: +64.97 ± 13.11 Elo, LOS 100%, H1 at 1412 games** (far above the expected +15–25). New head = `rarog-phase47-polish-pext-pgo.exe`; **bench 4,747,104**. **PHASE 4 COMPLETE (staged ≈+316 self-play).**

### Phase 5 - Search-Efficiency Wave (+20–50 Elo)

The one search-constant SPSA wave + refinements, at the final eval scale.
Driving: Sonnet 4.6 medium; dense ports: Codex 5.5 medium / GPT-5.5 high.
See `PLAN.md` §9.

- [~] 5.1 Search-constant SPSA wave (pruning, LMR, futility, ProbCut margin, TM); incl. relocated 2.11 Group-B widen `[0,120]` and 2.5.2 futility-direction A/B. **Prep DONE (2026-06-29, code only, no games):** ceilings widened; `ProbCutMargin`, `FutilityImprovingDir` (0/1 A/B), `LazyMargin`, and the 7-param TM group all exposed (tune-gated, ×10000 where float); `config_tm.json` + `config_lazymargin.json` written; `setup_spsa.ps1` groups `tm`/`lazymargin` wired; SPSA README documents each. Bench `4,747,104` unchanged, 159/159 tests, fmt clean, options hidden in release. **Remaining = the user-run gates** (SPSA → bake → SPRT per group; the `FutilityImprovingDir` A/B `[-3,3]`).
- [~] 5.1b **Lazy-eval margin re-check** — **`LazyMargin` UCI option now exposed** (`[200,2000]`, seed 600, pushed to the evaluator each search start) + `config_lazymargin.json`. Still to run: **widen first + confirm no regression `[-3,3]` at the post-Phase-4 eval scale**, then SPSA-tune for NPS. (Lazy is off under `--features texel`; the mop-up runs on both paths, so mating is margin-independent.)
- [ ] 5.2 History bonus/malus split, then retry no-aging history.
- [ ] 5.3 do-deeper re-implementation (cp-coupled retry).
- [ ] 5.4 Qsearch quiet checks; razoring depth restriction; LMR TT-move-is-capture; mobility-area refinement.
- [ ] 5.5 Codex ports: multi-cut/singular, threat-aware history, TT-cutoff/fail-low-parent history, optional TT overhaul. — Codex 5.5 medium
- [ ] 5.6 Modern refinements (aspiration modernization, correction-magnitude margins, hindsight, cutoff-count LMR, bad-noisy futility, qsearch SEE threshold).
- [ ] 5.7 Profile-guided speed pass; end-of-phase gauntlet + release.

### Phase 6 - Non-NNUE ceiling: eval-refresh cycles (optional, +10–40/cycle)

The post-Phase-5 HCE-maturity grind — iterate the data-fit on data from the
now-stronger head. **Optional, evidence-driven** (enter only if the
end-of-Phase-5 gauntlet shows eval headroom). Full rationale: `PLAN.md` §10
(§6.0 analysis, §6.1 cycle 1, §6.2 iterate). *Was mis-numbered "4.8/4.9" under
Phase 4 / Phase 5 — now its own phase because it runs after Phase 5.*

- [ ] **6.1 Eval data-refresh, cycle 1 (PLAN.md §6.1).** Regenerate self-play with the new head and do **one consolidated eval refit** (not a re-stage: a single low-lr joint fit + the king-safety re-eval path + one SPRT). Stronger engine → cleaner WDL labels → tighter fit. Turn on **blended labels** + **`--balance-phase`** (the dormant Step-4.0 capabilities) on the regen. **Build two ride-along structural items into this refit** (PLAN.md §6.1): (1) **fold pawn shelter/storm into the king-danger input** — they exist but the Phase-4 fit zeroed `storm_*`/`shelter_missing_*` because a *linear* term can't capture the `danger²` interaction (best single new eval bet); (2) activate the §3.12 deferred trio. Expected **+10–40 Elo** (a correction, not a re-discovery). Evidence-driven, gated on the end-of-phase gauntlet — **not mandatory.**
- [ ] **6.2 Iterate (cycles 2–3) + stop condition (PLAN.md §6.2).** Repeat 6.1 on fresh data; stop when a cycle yields < ~+8 Elo, holdout stops dropping, or the gauntlet shows no field movement. Past that, the only classical lever left is king-bucketed PSTs — which is the NNUE input shape, so the honest move is **NNUE (§14)**, not more HCE tables.

> **Can we beat Critter (~3187) without NNUE? (PLAN.md §6.0, 2026-06-24.)** Yes,
> *possible* but it's a tuning-maturity grind, not a missing-feature gap: Rarog
> already has the full **Stockfish-11-class** HCE feature set (shelter+storm,
> passed-pawn king proximity, complexity, SF imbalance — all source-verified).
> SF11 reached ~3440 with this same shopping list, so the gap is **search depth +
> iterated tuning**, not exotic terms. Ranked non-NNUE levers: **(1) Phase 5
> search** (already first, invalidated by nothing) → **(2) iterated §6.1
> refresh** (1–3 cycles) → **(3) shelter/storm→danger** (rides §6.1) → (4) the
> deferred trio → (5) **king-bucketed PSTs = the NNUE gateway, NOT an HCE step**
> (it's the HalfKA input shape; if we reach for it, do NNUE instead). No
> reordering needed — Phase 5 → §6.1 is already optimal. *Note: the common claim
> that Berserk/RubiChess/Stash are 3300+ "HCE" is wrong — those are their NNUE
> ratings; their classical builds were ~3000–3150.*

### NNUE Readiness (terminal option, not scheduled)

- [ ] Not scheduled. Keep `Evaluator::eval()` the only search↔eval boundary so a
      future HCE→NNUE switch stays localized. See `PLAN.md` §14.

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
Nodes searched  : 4978006
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
- **Phase 3 inert steps must keep `bench 13 == 4,978,006`** (re-baselined at 3.14
  by the eval-cache fix, which made the eval pure; seed every
  new *tunable* eval sub-term inert). Phase 3 spends no games.
- Keep `Evaluator::eval()` as the only boundary between search and evaluation.

The process is the strength engine: tune, test, keep only what survives.
