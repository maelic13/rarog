# RAR-E69 — C.9's curvature sweep: the unfitted nonlinear evaluation coordinates on held-out outcomes

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-19, before any validation row was read at a sweep point; agent-run (zero games, about 15 minutes of builds and dumps).
- **Kind:** research, PLAN C.9's condition. PLAN rule 8's curvature sweep and AGENTS' SPSA rule ask for activation and curvature before any tune: a flat surface skips C.9 with a written reason. Nothing is built into the engine.
- **Baseline SHA / candidate SHA / dirty-diff hash:** branch `c9` at the registration commit; engine sources equal `6ade0e6`'s (**10,521,141 / EBF 2.521**). The sweep points are scratch builds outside the repository, each recorded by its patch log and binary hash.
- **The residue: what the tuner cannot fit.** The fitting manifest (`tools/texel/hce_fit_manifest_v6.tsv`) fits 1,133 linear weights; its 23 king-danger coordinates (the index weights and both map scales) get the nonlinear pass (C.3.3), and both map scales sit inside their bound (102 and 18 against 150). What neither pass sees:
  - **In the sweep** (continuous, active in the middlegame or across all phases):
    - `taper`: the shape of the phase taper. The share of the middlegame score becomes `round(24·(phase/24)^γ)/24`, in the lazy gate and the final interpolation only; γ = 1 is the engine. RAR-E58 and RAR-E59 found a phase slope in how large scores convert, calibrated away once errors were clustered by game, so this asks the same question of the taper itself.
    - `thr`: `KS_MAP_THRESHOLD` (100), the index the king-danger map must exceed.
    - `cap`: `KS_INDEX_CAP` (1600), the index clamp; the magnitude contract's static bound is relaxed in the scratch build only.
    - `ocb`: the base of the general opposite-bishop regime, `32 + 4·pawns + 4·passers` on the /48 basis, never fitted (the pure-ending regime was fitted in C.5.2).
  - **Not in the sweep, with the reason:**
    - `LazyMargin`, the rule-50 damping and the SEE piece values are search coordinates, C.10's.
    - The mop-up, KNN-KP and specialised endgame scales act at five men or fewer, where the labels are Syzygy-corrected and C.5's conversion instruments measured them (RAR-E56: the 24 C.5 families cost at most 1.31 Elo in a gate's games).
    - `PURE_OCB_MIN_PAWNS` and the trapped-bishop phase gate are discrete; the trapped-bishop weights are frozen as too sparse to fit.
    - The map's divisors (4096, 16) are redundant with the fitted map scales.
    - Four coordinates, not rule 8's five: no fifth continuous unfitted coordinate is active.
- **Points:** each coordinate at 0.5x, 0.75x, 1x, 1.5x and 2x, the others at the head; 1x is the head, shared.

  | Coordinate | 0.5x | 0.75x | 1x | 1.5x | 2x |
  |---|---:|---:|---:|---:|---:|
  | `taper` γ | 0.5 | 0.75 | 1 | 1.5 | 2 |
  | `thr` | 50 | 75 | 100 | 150 | 200 |
  | `cap` | 800 | 1200 | 1600 | 2400 | 3200 |
  | `ocb` | 16 | 24 | 32 | 48 | 64 |

  `ocb` at 48 and 64 both reach the /48 cap, so both remove the general regime's scaling; they are the same function.
- **Instrument** (`analysis/artifacts/c9/`, SHA-256):
  - `patch_point.py` `3366a940…`: applies one point to a scratch copy (`git archive HEAD`), each anchor asserted unique, line endings kept.
  - `zz_c9_dump.rs` `3e38b455…`: copied into the scratch tree's `tests/`, writes each row's played White-POV score.
  - `run_c9.sh` `a3d0b331…` (amended, below; registered as `50d5c0b4…`): per point, restore, patch, `cargo build --release`, dump, `bench 13`; every exit status to `status.txt`.
  - `read_e69.py` `897dafe6…`: the reader below.
- **Rows:** `hce-v6-tb`'s validation file (194,444 rows, `7ada0924…`), each row's game from C.13.2's provenance (`c132/provenance.csv`, `f8d572c4…`), as RAR-E62 to RAR-E66 read them.
- **Reading (frozen):** per point, the tuner's sigmoid `1/(1+exp(−K·s/400))` with K refitted to that point, so a coordinate is read for the outcome information it carries, not for the score's size (the size is C.10's magnitude question). The **gain** is the squared-error loss saved against the head, in percent of the head's loss, with RAR-E63's game-clustered bootstrap (2,000 resamples, seed 20261019). Beside it, not deciding: the gain at the head's K, the rows a point moves and by how much, and `bench 13` as the tree surface.
- **Classification (frozen), per coordinate over its four non-head points:**
  1. *improving* when any point gains at least **0.10%** at **4 game-clustered SE** (C.13's line);
  2. otherwise *flat* when every point's gain lies within ±0.10%;
  3. otherwise *fitted*: some point loses beyond the line, none gains to it, so the head is at least as good as anything in the sweep.
- **Wires (void if any fails):**
  - W1: every row has a game.
  - W2: the head point's scores equal `c132/head_played.csv` on every row (`c99ebdc8…`; the C.12 rewrites were proven exact, so the evaluation has not changed since).
  - W3: the head point's `bench 13` reads 10,521,141.
  - W4: every other point moves at least one row's score.
- **Registered order (frozen):**
  - `bash analysis/artifacts/c9/run_c9.sh tools/texel/data/hce-v6-tb/validation.csv tools/results/c9-sweep-20261019 <scratch> base taper=0.5 taper=0.75 taper=1.5 taper=2 thr=50 thr=75 thr=150 thr=200 cap=800 cap=1200 cap=2400 cap=3200 ocb=16 ocb=24 ocb=48 ocb=64`, once.
  - Then `python analysis/artifacts/c9/read_e69.py tools/texel/data/hce-v6-tb/validation.csv tools/results/c9-sweep-20261019 tools/results/c9-sweep-20261019/read.json --provenance analysis/artifacts/c132/provenance.csv --head-played analysis/artifacts/c132/head_played.csv > tools/results/c9-sweep-20261019/read.log 2>&1`, once, with the exit status read directly.
- **Known exposure, stated before the read:** a smoke of the whole path on 2,000 *training* rows (every hundredth of the first 200,000, each its own game) at the head and one point per coordinate (γ = 2, 200, 3200, 16). It proved the builds, the patches, W3 and W4, and the reader. Its numbers were seen: γ = 2 lost about 1% of the loss; the other three moved under 0.02%, inside one SE. RAR-E57 to RAR-E59's phase readings and C.5.2's opposite-bishop history are known.
- **Use, frozen:**
  - **No coordinate improving: C.9 closes `NO_CHANGE`**, the written skip. The evaluation enters C.10 as it is.
  - Any coordinate improving: C.9 stays in `RESEARCH` and drafts a game tune over the improving coordinates only, seeded at the best sweep point, in rule-7c blocks, gated by SPRT `[0,3]`. A static gain is not Elo (RAR-E67), so nothing is applied without that gate.
  - Void: fix the instrument, record why, and rerun once.
- **PRE-REGISTERED PREDICTION (freeze before exposure):**
  1. The wires hold (0.95).
  2. `taper`: *fitted* (0.7): the middlegame and endgame weights were fitted under the linear taper, so bending it costs; *improving* (0.15).
  3. `thr`: *flat* (0.85): the fitted `kd_constant` absorbs a threshold shift, and near the threshold the map is worth about 2.5 cp.
  4. `cap`: *flat* (0.6), since few rows pass an index of 1,600; *improving* at 800 or 1,200 (0.2), since shrinking the largest king penalties is the direction RAR-E30's magnitude recalibration points.
  5. `ocb`: *flat* (0.55); *improving* (0.15).
  6. **C.9 closes `NO_CHANGE` (0.6).**
  - Expected Elo: none here.
- **Cheapest prior falsifier:** this sweep.
- **Amendment before exposure (2026-10-19):** the first run of the registered order was void. The runner passed the rows and output paths on as given, relative to the repository, while the dump runs inside the scratch tree, so every point's dump exited 101 on a missing file and no validation row was scored (`tools/results/c9-sweep-20261019-void1/`, no `.scores` file). The runner now makes both paths absolute and refuses a missing rows file (`a3d0b331…`). The guard fired once on a missing file, and the fixed runner reproduced the smoke's head scores byte for byte from a relative path. The only number seen from the void run is the head's `bench 13`, 10,521,141. The registered order runs again, once, unchanged otherwise.
