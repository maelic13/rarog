# RAR-E71 — C.9's inventory follow-up: the generic mop-up's weights on held-out outcomes, RAR-E69's sweep

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-19, before any sweep point was built on the validation rows; agent-run (zero games, about six minutes of builds and dumps).
- **Kind:** research, for the Phase C review (`analysis/phase_c_review_2026-10-19.md`, question 1). RAR-E69's inventory of the unfitted nonlinear evaluation coordinates placed the mop-up among the recognisers that act at five men or fewer. That is true of its minor-piece-mate branch and false of its generic branch: `apply_mop_up` (`src/eval/endgame/mop_up.rs`) runs on every path, full and lazy, whenever the running `(mg + eg) / 2` passes 200 cp, and then adds `sign · (5 · (file push + rank push) + 4 · (14 − king distance))` for the side ahead, up to 86 cp, whatever the material. RAR-E62's trace dump saw it fire on 93,575 of the 194,444 rows. Its two weights were never fitted (they are in the frozen part the tuner reconstructs but does not move), so they are a fifth continuous unfitted coordinate pair, swept here under RAR-E69's instrument and rule. Nothing is built into the engine.
- **Baseline SHA / candidate SHA / dirty-diff hash:** branch `c9` at the registration commit; engine sources equal `6ade0e6`'s (**10,521,141 / EBF 2.521**). The sweep points are scratch builds outside the repository, each recorded by its patch log and binary hash.
- **Points:** the two weights (5, 4) scaled together; `base` is the head, shared.

  | Point | edge push | king distance |
  |---|---:|---:|
  | `mop=0` | 0 | 0 |
  | `mop=0.5` | 3 | 2 |
  | `mop=0.75` | 4 | 3 |
  | base | 5 | 4 |
  | `mop=1.5` | 8 | 6 |
  | `mop=2` | 10 | 8 |

  `mop=0` removes the generic branch and is the point a removal would play; the minor-piece-mate branch is untouched at every point.
- **Instrument** (`analysis/artifacts/c91/`, SHA-256): `patch_point.py` `def1f006…` (RAR-E69's patcher with the `mop` coordinate; one anchor, asserted unique, line endings kept); `run.sh` `8be14561…` (RAR-E69's runner with this directory's patcher and `mop_up.rs` restored before every point); `read.py` `6e7dc0b7…` (RAR-E69's reader with its coordinate table replaced; the sigmoid, the K refit, the game-clustered bootstrap with seed 20261019, the wires and the classification are unchanged); RAR-E69's dump test `analysis/artifacts/c9/zz_c9_dump.rs` `3e38b455…`.
- **Rows:** `hce-v6-tb`'s validation file (194,444 rows, `7ada0924…`), each row's game from `c132/provenance.csv` (`f8d572c4…`), as RAR-E69 read them.
- **Reading (frozen):** RAR-E69's: per point, the tuner's sigmoid with K refitted to that point; the gain is the squared-error loss saved against the head in percent of the head's loss, with the game-clustered bootstrap (2,000 resamples). Beside it, not deciding: the gain at the head's K, the rows a point moves and by how much, and `bench 13`.
- **Classification (frozen), over the five non-head points:** *improving* when any point gains at least **0.10%** at **4 game-clustered SE**; otherwise *flat* when every point's gain lies within ±0.10%; otherwise *fitted*.
- **Wires (void if any fails):** W1 every row has a game; W2 the base point's scores equal `c132/head_played.csv` (`c99ebdc8…`) on every row; W3 the base point's `bench 13` reads 10,521,141; W4 every other point moves at least one row's score.
- **Registered order (frozen):**
  - `bash analysis/artifacts/c91/run.sh tools/texel/data/hce-v6-tb/validation.csv tools/results/c91-mop-20261019 <scratch> base mop=0 mop=0.5 mop=0.75 mop=1.5 mop=2`, once.
  - Then `python analysis/artifacts/c91/read.py tools/texel/data/hce-v6-tb/validation.csv tools/results/c91-mop-20261019 tools/results/c91-mop-20261019/read.json --provenance analysis/artifacts/c132/provenance.csv --head-played analysis/artifacts/c132/head_played.csv > tools/results/c91-mop-20261019/read.log 2>&1`, once, with the exit status read directly.
- **Known exposure, stated before the read:** a smoke of the whole path on the first 200 rows of `hce-v6-tb`'s training file (each row its own game), at `base` and `mop=2`: the base rebuilt to 10,521,141 and `mop=2` to 14,872,829 nodes, both dumps wrote 200 rows, and the wires passed. The smoke's gains were not printed or read. RAR-E62's activation count (93,575 rows) and RAR-E58's calibration table (large opening and middlegame scores convert less often than large endgame ones) are known.
- **Use, frozen:**
  - *Improving at `mop=0` or `mop=0.5`:* RAR-E69's inventory was incomplete in a way that matters. C.9 reopens as `RESEARCH` with the removal or halving as its candidate; its first screens are the static differential (which rows move, by phase and |score|) and RAR-E19's fixed-depth tree read on `c03-lazy/positions_v1.epd`; the gate is the maintainer's (a removal takes AGENTS' loss-permitting bracket).
  - *Improving at `mop=1.5` or `mop=2` only:* recorded as a magnitude lead of the kind RAR-E18, RAR-E19 and RAR-E20 measured (larger decided scores predict outcomes better statically and lose through a search fitted to the smaller ones). No engine change; C.9's close stands; the retry trigger is a game record of under-converted large middlegame scores after C.10.
  - *Flat or fitted:* C.9's close stands, and the review proposes the corrected inventory for PLAN C.9's record.
  - *Void:* fix the instrument, record why, rerun once.
- **PRE-REGISTERED PREDICTION (freeze before exposure):**
  1. The wires hold (0.9).
  2. *Activation:* every non-head point moves at least 40% of the rows (0.8), by a mean of 10 to 40 cp (0.7).
  3. *Shape:* **fitted (0.45)**, flat (0.30), improving (0.25); if improving, at `mop=1.5` or `mop=2` (0.6 of that) rather than at `mop=0` or `mop=0.5` (0.4).
  4. `mop=0`: **−0.10%**, 80% band [−0.35, +0.05]: the king piece-square tables were fitted with the push present, so removing it costs. `mop=2`: +0.02%, band [−0.15, +0.15].
  5. `bench 13` at `mop=0` differs from the head's by more than 10% (0.6): the term fires in middlegames and the tree is hypersensitive to it.
  6. The gain at the head's K differs from the refitted gain by at most 0.03 points at every point (0.7): the term is a conditional push, not a uniform scale.
  - Expected Elo: none here.
  - Most likely failure mode: prediction 3 misses toward improving at 2x, which the frozen use records as magnitude and not as a candidate.
- **Cheapest prior falsifier:** this sweep.
- **Result:** pending.
- **Disposition:** pending.
- **PREDICTION CALIBRATION (append after exposure):** pending.
- **Artifacts / commits:** this registration; `analysis/artifacts/c91/` and `tools/results/c91-mop-20261019/` (ignored storage).
