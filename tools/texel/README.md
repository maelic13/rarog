# Rarog Texel tuning toolchain

Tools for the **Phase 4 eval data-fit** (see `PLAN.md` §6–§8). Most of this was
ported from Basilisk's working pipeline (`D:/code/basilisk/tools/texel`) and
adapted to Rarog. The scripts here are **ready now**; the tuner binary itself is
wired up in **Phase 3.3** (it needs Rarog's eval to expose the `EvalTrace`,
`reconstruct()`, and the `EvalParams` flat-parameter list — none of which exist
until the Phase 3 eval refactor lands). See `reference/basilisk_tuner.cpp` for
the exact, proven design to port.

> **Two outputs feed the tuner, both in `FEN;target` text format** (one position
> per line; `target` is White-perspective expected score: `1` / `0.5` / `0`, or
> a float in `[0,1]`). The tuner reads `train.csv` + `holdout.csv`.

---

## The dataset (decide the label source)

The tuner fits eval weights so that `sigmoid(eval)` predicts `target`. The
quality of the fit is bounded by the quality of the labels. Two label paths are
supported; **the game test (SPRT) decides which transfers** — you can build both
and compare.

### Path A — self-play labels (primary, fully functional now)

Label each position by the **result of a Rarog-vs-Rarog game** that passed
through it. Pure, never over-fits to another engine's quirks, lower ceiling.

```
Beast FENs ──sample_fens.py──▶ beast_seed.epd ──datagen.ps1──▶ selfplay.pgn ──extract.py──▶ train.csv + holdout.csv
```

### Path B — Stockfish-WDL labels (optional, higher ceiling, needs an SF binary)

Label each sampled position by a **strong engine's WDL/eval**, distilling its
static judgment into Rarog's weights (a common strong-HCE technique). Denser and
often higher-quality than self-play results, but can chase SF quirks that do not
transfer. Use `import_beast.py` once you have `FEN<TAB>target` files (e.g. from
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
| `sample_fens.py` | Reservoir-samples N diverse FENs from the Beast pool (or any FEN/CSV) into a fastchess EPD **opening book**. Validates with python-chess, dedups, filters by piece count / check / quietness. 7 GB-safe (streaming). |
| `datagen.ps1` (in `tools/`) | Runs Rarog self-play at a fixed node limit from the book, appends to `data/selfplay.pgn`. Auto-concurrency = logical CPUs − 1. |
| `extract.py` | PGN → `FEN;result`. Skips opening (16 plies) / endgame (6 plies) / in-check / capture-or-promo positions, caps plies per game, dedups by FEN, **splits holdout by game** (no train/holdout leakage). |
| `import_beast.py` | For Path B: converts pre-evaluated `FEN<TAB>target` files to `FEN;target` train/holdout, converting side-to-move targets to White perspective. |
| `reference/basilisk_tuner.cpp` | The proven C++ tuner (Adam + golden-section K-fit + group masks + reconstruction `--verify`). **Reference for the Rust port in Phase 3.3** — do not build; it links Basilisk's eval. |

---

## Full self-play workflow (Path A)

Run from the repo root. **Hardware note:** sized for a Ryzen 9 5950X
(16C/32T). The defaults below leave the machine usable — raise `-Concurrency`
or `-Rounds` if you want the run to finish faster and the box can spare it.

```powershell
# 0. Build the current head as a PGO test binary (the datagen engine).
.\tools\build_test.ps1 -Suffix phase3-base

# 1. Sample a diverse opening book from the Beast pool (source stays intact).
#    --max-read caps the streaming pass; drop it for a fully uniform sample.
python tools\texel\sample_fens.py "A:\Chess\Beast\data\txt\positions.txt" `
    --out tools\texel\data\beast_seed.epd --count 50000 --min-pieces 6

# 2. Generate self-play games (node-limited, fast, diverse). Moderate
#    concurrency so the machine stays usable; raise it for unattended runs.
.\tools\datagen.ps1 -Suffix phase3-base -Rounds 30000 -Nodes 8000 `
    -Book tools\texel\data\beast_seed.epd -BookFormat epd -Concurrency 24
# Optional second pass for variety (different node count appends to the PGN):
.\tools\datagen.ps1 -Suffix phase3-base -Rounds 15000 -Nodes 5000 `
    -Book tools\texel\data\beast_seed.epd -BookFormat epd -Concurrency 24

# 3. Extract labelled positions (train + holdout, split by game).
python tools\texel\extract.py tools\texel\data\selfplay.pgn `
    --out-dir tools\texel\data --train train.csv --holdout holdout.csv

# 4. (Phase 3.3+) Verify reconstruction, then tune a stage:
#    rarog-texel --verify  tools\texel\data\holdout.csv
#    rarog-texel --tune kingsafety tools\texel\data\train.csv tools\texel\data\holdout.csv tools\texel\out\eval_params.txt
```

Target ≥ 1.5 M train positions (the `extract.py` warning enforces this). Each
node-limited game is ~1–2 s; ~60 k games on the 5950X is well under an hour at
`-Concurrency 24`.

---

## The Rust tuner (Phase 3.3 — DONE)

The tuner is built: `tools/texel-tuner` (binary `rarog-texel`), a workspace
member depending on the rarog lib with `features = ["texel"]`. Run it from the
repo root:

```powershell
# Reconstruction acceptance gate (run before any tuning):
cargo run --release -p texel-tuner -- --verify tools\texel\data\holdout.csv
# Stage a group (material first, PSTs/all last). out file is RAROG_EVAL_FILE format:
cargo run --release -p texel-tuner -- --tune material `
    tools\texel\data\train.csv tools\texel\data\holdout.csv tools\texel\out\material.txt
# Options: --epochs N (default 200), --lr X (default 0.3), --max-positions N.
# Groups: material pawnstruct passers rooks minors mobility threats hanging
#         misc kingsafety scalars pst all
```

The output file loads straight into a `--features tune` engine via
`RAROG_EVAL_FILE`, or is baked into `src/eval.rs` defaults once a stage's SPRT
passes (Phase 4). Parallelism uses `std::thread` (no external crates), so the
engine stays dependency-free.

It was ported from `reference/basilisk_tuner.cpp`. The reusable, engine-agnostic
parts (copied as *structure*, not C++):

- **Objective / Adam / K-fit** (`sigmoid`, `traced_loss`, `cmd_tune`, `fit_K`):
  pure math, transcribe directly.
- **Group masks** (`active_indices_for_group`): the staged-tuning groups
  (material / scalars / kingsafety / pst / all …) map 1:1 to Phase 4 stages.
- **`--verify`** (`cmd_verify`): the reconstruction acceptance test — reconstructed
  `E(default)` must equal `evaluate()` integer-for-integer (Phase 3 gate).
- **`linear_delta_scale`**: captures Rarog's frozen non-linear factors (OCB
  scaling, two-knights draw, 50-move damping) as the per-position `scale`. Mirror
  Rarog's `scale_drawish_endgames` + rule-50 damping here.
- **Output format** (`name index value` per line): matches the `RAROG_EVAL_FILE`
  loader (Phase 3.2).

The **engine-coupled** part is Rarog-side, built in Phase 3.1/3.3: an
`EvalParams` struct, an `EvalTrace` of net feature counts, a `reconstruct()`,
and a flat-parameter name/length list (Rust `macro_rules!` standing in for
Basilisk's `EVAL_PARAM_LIST` X-macro). The Rust tuner is a workspace member
depending on the rarog lib with `features = ["texel"]`.
