# Rarog Texel tuning toolchain

Tools for Rarog's self-play-labelled HCE fits. The pipeline and Rust tuner are
fully implemented. Hydra contributed the useful five-reservoir sampling idea;
Rarog deliberately retains self-play game-result labels because its measured
Stockfish-distillation experiment lost 17.11 Elo (RAR-E03).

> **Roadmap note.** Historical comments inside the tuner may still use the
> phase numbers from the cycle in which it was built. Current scheduling is
> authoritative in `PLAN.md`: corpus/instrument qualification is 4.7, the
> complete existing-surface refit is 4.8, structural clusters are 4.9, and
> post-structure consolidation is 4.10. No historical tune group is frozen.

> **Three outputs feed the tuner, all in `FEN;target` text format** (one position
> per line; `target` is White-perspective expected score: `1` / `0.5` / `0`, or
> a float in `[0,1]`). The tuner selects on `train.csv` + `validation.csv` and
> reports `test.csv` once through `--test`.

---

## The dataset (decide the label source)

The tuner fits eval weights so that `sigmoid(eval)` predicts `target`. The
quality of the fit is bounded by the quality of the labels. Two label paths are
supported; **the game test (SPRT) decides which transfers** — you can build both
and compare.

### Path A — self-play labels (primary, fully functional now)

Label each position by the **result of a Rarog-vs-Rarog game** that passed
through it. This remains mandatory for Rarog: its Phase-6 Stockfish-distilled
fit improved offline loss but lost 17.11 Elo. We copy Hydra's reliable
five-reservoir sampling design, not its label source.

```
Beast FENs ──sample_fens.py──▶ beast_seed.epd ──datagen.ps1──▶ selfplay.pgn ──extract.py──▶ train.csv + validation.csv + test.csv
```

### Path B — Stockfish-WDL labels (archived diagnostic; rejected for Rarog)

Label each sampled position by a **strong engine's WDL/eval**, distilling its
static judgment into Rarog's weights (a common strong-HCE technique). Denser and
often higher-quality than self-play results, but can chase SF quirks that do not
transfer. Rarog measured exactly that failure in RAR-E03 (**−17.11 Elo**), so
do not use this path for a fit candidate. The `import_beast.py` converter was removed
on 2026-09-09 (last at `6fa6731`); reproduce that result from git history if ever needed, with `FEN<TAB>target` files (e.g. from
running an SF `go nodes`/`go depth` pass over the sampled FENs and writing its
WDL as the side-to-move target). No SF binary was found in this repo; point the
labeller at your capped/full Stockfish when you choose this path.

**The Beast source is read-only.** `A:\Chess\Beast\data\txt\positions.txt`
(7.1 GB of bare, unique FENs — no labels) is **streamed, never modified or
copied**. Treat it as an immutable position pool.

---

## Scripts

| Script | What it does |
|---|---|
| `sample_fens.py` | Streams the Beast pool into five equal phase reservoirs and writes a validated, deduped EPD seed book. Default 750k starts; 7 GB-safe. |
| `datagen.ps1` (in `tools/`) | Runs one deterministic self-play game per independent book entry. EPD books are rejected if any four-field FEN repeats; a fixed shuffle seed plus non-overlapping `Start`/`Rounds` segments makes pilot and continuation reproducible; each completed archive gets a provenance manifest. |
| `extract.py` | Reads PGN archives into exact phase quotas for train/validation/frozen-test. Whole supplied starts are assigned by stable hash, replay leakage is rejected, rule-50 identity is retained, and outputs plus a hash-complete manifest publish atomically. `--preflight-games` sizes the run first. |
| `extract_parallel.py` | Parallel front end with the same deterministic dataset contract; validates datagen sidecars/ranges and reports result, termination, mate, label and material coverage. |
| `fit_complete.ps1` | One-command qualified corpus publication plus complete alternating nonlinear/linear fit, verification, bake/test/bench artifact capture and safe source/binary restoration. |
| `confirm_hce_fit.ps1` | One-command untouched confirmation of the fixed production fit: 150k unused independent openings, pure self-play WDL, exact source-to-rounded-candidate report. |

---

## Current audited production fit

The 2026-08-31 archive audit proved that the existing 600,000 games support
2,300,000 train rows and 127,778 rows in each held-out split. The old 3M target
cannot meet its opening quota. The complete fit has already run; its original
test report was invalid and that test is now consumed. From a clean worktree,
confirm the fixed candidate on untouched starts with:

```powershell
pwsh -NoProfile -File tools\texel\confirm_hce_fit.ps1
```

The confirmation runner refuses changed engine source or candidate weights,
uses book entries 600,001--750,000 exactly once, validates the richer datagen
sidecar and literal WDL targets, and extracts 127,778 equal-phase test
positions from starts assigned to a 50% held-out split. It does no optimization
and does not read Stockfish evaluations. `summary.json` in the new
`tools/results/hce-confirm-<timestamp>/` is the entry point for review. It does
not run an SPRT or claim Elo.

Use `-Smoke` only to validate the pipeline on bounded legacy data. Production
mode refuses tracked worktree changes.

---

## Full self-play workflow (Path A)

Run from the repo root. **Hardware note:** auto-concurrency leaves two physical
cores free (14 games on a Ryzen 9 5950X). Pass a higher value such as 24 only
when maximum throughput matters more than responsiveness. Fixed-node games are
deterministic either way.

```powershell
# 0. The accepted post-capstone label generator is already built and has a
#    clean manifest: rarog-p1025a-zero-pext-pgo.exe (source 74d4426).

# 1. Sample a phase-balanced seed book from the evaluated Beast pool. The SF
#    target is used ONLY to reject nearly-decided seeds; game results remain
#    Rarog self-play labels. The external source stays read-only.
python tools\texel\sample_fens.py "A:\Chess\Beast\data\evaluated" `
    --out tools\texel\data\beast_seed.epd --count 750000 --min-pieces 6 `
    --target-min 0.05 --target-max 0.95 --max-read 40000000

# 2. Validate, then run ONLY a 20k pilot. Seed 10403 defines one reproducible
#    shuffled order; later segments retain it and begin at opening 20001.
.\tools\datagen.ps1 -Suffix p1025a-zero -Nodes 8000 -Rounds 20000 `
    -Start 1 -Seed 10403 -SetupOnly
.\tools\datagen.ps1 -Suffix p1025a-zero -Nodes 8000 -Rounds 20000 `
    -Start 1 -Seed 10403

# 3. Size the corpus BEFORE generating the expensive continuation. It reports
#    the limiting phase and a conservative recommended TOTAL game count.
python tools\texel\extract.py `
    tools\texel\data\selfplay-p1025a-zero-n8000-s1-g20000.pgn `
    --preflight-games 20000

# 4. Substitute the reported total N. This continuation is exactly N-20,000
#    games and cannot overlap the pilot or wrap around the 750k book.
$recommendedTotal = <N_FROM_PREFLIGHT>
.\tools\datagen.ps1 -Suffix p1025a-zero -Nodes 8000 `
    -Rounds ($recommendedTotal - 20000) -Start 20001 -Seed 10403 -SetupOnly
.\tools\datagen.ps1 -Suffix p1025a-zero -Nodes 8000 `
    -Rounds ($recommendedTotal - 20000) -Start 20001 -Seed 10403

# 5. One extraction across pilot + continuation. Set the target from the full
#    audit; for the current two archives it is exactly 2.30M train rows.
python tools\texel\extract_parallel.py `
    tools\texel\data\selfplay-p1025a-zero-n8000-s1-g20000.pgn `
    tools\texel\data\selfplay-p1025a-zero-n8000-s20001-g*.pgn `
    --out-dir tools\texel\data\hce-v2 --target-train 2300000 --jobs 14

# 6. Prefer fit_complete.ps1 for the registered complete schedule. Manual
#    commands below are diagnostic and must preserve complete-vector chaining.

# 7. Bake, THEN FORMAT, then verify the fingerprint. cargo fmt is not optional:
#    bake_params.py writes one long line per PST, so `cargo fmt --check` fails
#    until it runs. Skipping it once already produced a gate binary built from a
#    tree that would have failed CI (2026-08-04). Formatting cannot change
#    behaviour, so the bench must be identical before and after.
python tools\texel\bake_params.py tools\texel\out\eval_params.txt
cargo fmt
cargo fmt --check          # must pass before building anything gated
```

The completion contract is the **largest prospectively audited exact
equal-five-phase target**, not a copied corpus size or rough global estimate.
For the current inputs that is 2,300,000 / 127,778 / 127,778. Short input exits
2 without touching an existing dataset and reports the exact missing quota.
Keep the same book hash and seed for every segment; `datagen.ps1` records both
plus engine SHA-256/source commit, fastchess version, range, node limit and the
named `datagen-v1` adjudication profile in `*.manifest.json`.

---

## The Rust tuner

The tuner is built: `tools/texel-tuner` (binary `rarog-texel`), a workspace
member depending on the rarog lib with `features = ["texel"]`. Run it from the
repo root:

```powershell
# Reconstruction acceptance gate (run before any tuning):
cargo run --release -p texel-tuner -- --verify tools\texel\data\hce-v2\validation.csv
cargo run --release -p texel-tuner -- --audit-coverage
# Complete vectors can be chained without resetting a previous stage:
cargo run --release -p texel-tuner -- --tune complete `
    tools\texel\data\hce-v2\train.csv tools\texel\data\hce-v2\validation.csv `
    tools\texel\out\complete.txt --initial tools\texel\out\prior.txt
# Options include --initial FILE, --epochs N, --lr X, --l2 X,
# --max-positions N, --test FILE and --fix-k K.
# Groups: material pawnstruct passers rooks minors mobility threats hanging
#         misc kingsafety scalars pst all complete
```

The output file loads straight into a `--features tune` engine via
`RAROG_EVAL_FILE`, or is baked into `src/eval.rs` defaults once a stage's SPRT
passes its registered gate. Parallelism uses `std::thread` (no external crates), so the
engine stays dependency-free.

It was ported from Basilisk's `tools/texel/tuner.cpp` (a copy lived here until `6fa6731`). The reusable, engine-agnostic
parts (copied as *structure*, not C++):

- **Objective / Adam / K-fit** (`sigmoid`, `traced_loss`, `cmd_tune`, `fit_K`):
  pure math, transcribe directly.
- **Group masks** (`active_indices_for_group`): the staged-tuning groups
  (material / scalars / kingsafety / pst / all …) define local or whole fits.
- **`--verify`** (`cmd_verify`): the reconstruction acceptance test — traced
  reconstruction must equal the independently accumulated linear raw score
  integer-for-integer; `--weights` verifies a complete fitted vector.
- **`linear_delta_scale`**: captures Rarog's non-linear factors held fixed for
  one linear fit (OCB scaling, two-knights draw, 50-move damping) as the
  per-position `scale`. Mirror
  Rarog's `scale_drawish_endgames` + rule-50 damping here.
- **Output format** (`name index value` per line): matches the `RAROG_EVAL_FILE`
  loader.

The **engine-coupled** part is Rarog-side: an
`EvalParams` struct, an `EvalTrace` of net feature counts, a `reconstruct()`,
and a flat-parameter name/length list (Rust `macro_rules!` standing in for
Basilisk's `EVAL_PARAM_LIST` X-macro). The Rust tuner is a workspace member
depending on the rarog lib with `features = ["texel"]`.
