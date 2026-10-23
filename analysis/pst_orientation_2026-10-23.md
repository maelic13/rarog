# PST orientation against PeSTO (2026-10-23)

Research record (`R2`, no engine change). Basilisk found on 2026-10-23 that
seven of its twelve PeSTO-seeded piece-square tables had been rank-mirrored
since v1.0.0: PeSTO publishes each table rank 8 first (index 0 = a8), Basilisk
indexes a1 = 0, and only some tables were flipped on import; its L2-anchored
refits kept the error (`D:/code/basilisk/analysis/pst_orientation_2026-10-23.md`).
The question here: does Rarog's evaluation, frozen at engine source `9fb525e`
(`bench 13` 13,987,223 / EBF 2.565), carry the same defect?

## Answer

**No table in the current evaluation is mirrored.** The same seven tables
(mg knight, bishop, queen; eg knight, bishop, queen, king) *were* mirrored
from the initial commit `c4f0bbe` (v1.0.0) through v2.1.0: each was identical,
entry for entry, to PeSTO as published, read with a1 = 0. Phase 4's staged
Texel fit (v2.2.0, 2026-06-24, the material and PST stage) moved exactly those
seven toward their rank flips and left no entry of any table identical to
PeSTO. Every later fit (the Phase C refits, L2 `1e-7` to the stage prior)
moved the tables by at most 0.04 of a flip, so the anchor preserved the
repaired tables, not the error. Two unanchored fit-direction tests on the
current tables confirm that the data's own optimum has the current
orientation in all twelve tables.

## Indexing

`Square::A1 = 0`, `H8 = 63` (`src/board/square.rs`). The tables are stored
White-POV and indexed by that square; Black reads `sq ^ 56`
(`build_tables`, `src/eval/params.rs`). PeSTO's arrays list rank 8 first, so a
correct import is the rank flip `i ^ 56` of the published array, and a verbatim
copy is mirrored: a White knight on f3 reads PeSTO's f6 value. The script
asserts this indexing from the source before comparing.

## The tables against PeSTO

Exact matches and Pearson correlation with PeSTO flipped (`=flip`, `r_flip`,
the correct import) and as published (`=raw`, `r_raw`, mirrored). Pawn tables
use ranks 2–7 (n = 48).

Initial commit `c4f0bbe` (identical through v2.1.0):

| table | n | =flip | =raw | r_flip | r_raw | verdict |
|---|---:|---:|---:|---:|---:|---|
| mg pawn | 48 | 48 | 0 | 1.000 | −0.182 | flipped PeSTO (correct) |
| mg knight | 64 | 0 | 64 | 0.521 | 1.000 | **unflipped PeSTO (mirrored)** |
| mg bishop | 64 | 0 | 64 | 0.550 | 1.000 | **mirrored** |
| mg rook | 64 | 64 | 0 | 1.000 | −0.325 | correct |
| mg queen | 64 | 0 | 64 | −0.095 | 1.000 | **mirrored** |
| mg king | 64 | 64 | 0 | 1.000 | 0.255 | correct |
| eg pawn | 48 | 10 | 0 | 0.952 | −0.429 | edited seed, correct orientation |
| eg knight | 64 | 2 | 64 | 0.868 | 1.000 | **mirrored** |
| eg bishop | 64 | 2 | 64 | 0.589 | 1.000 | **mirrored** |
| eg rook | 64 | 64 | 0 | 1.000 | −0.211 | correct |
| eg queen | 64 | 0 | 64 | 0.027 | 1.000 | **mirrored** |
| eg king | 64 | 0 | 64 | 0.644 | 1.000 | **mirrored** |

Working tree (`src/eval/material.rs`, unchanged since `9fb525e`; identical
verdicts at v2.6.0):

| table | n | =flip | =raw | r_flip | r_raw | verdict |
|---|---:|---:|---:|---:|---:|---|
| mg pawn | 48 | 0 | 0 | 0.895 | −0.184 | refit; correct orientation |
| mg knight | 64 | 0 | 0 | 0.758 | 0.604 | refit; correct orientation |
| mg bishop | 64 | 3 | 0 | 0.680 | 0.473 | refit; correct orientation |
| mg rook | 64 | 0 | 0 | 0.721 | −0.194 | refit; correct orientation |
| mg queen | 64 | 0 | 0 | 0.514 | 0.250 | refit; correct orientation |
| mg king | 64 | 0 | 1 | 0.649 | 0.377 | refit; correct orientation |
| eg pawn | 48 | 0 | 0 | 0.824 | −0.314 | refit; correct orientation |
| eg knight | 64 | 0 | 1 | 0.165 | 0.198 | refit; not separable by correlation |
| eg bishop | 64 | 1 | 1 | 0.053 | −0.182 | refit; correct orientation |
| eg rook | 64 | 2 | 0 | 0.506 | −0.227 | refit; correct orientation |
| eg queen | 64 | 2 | 0 | 0.289 | 0.417 | refit; not separable by correlation |
| eg king | 64 | 0 | 0 | 0.848 | 0.333 | refit; correct orientation |

Plain correlation confounds orientation with the flip-symmetric part of a
table (centre against edge, which both orientations share). Splitting PeSTO
`P` (a1-first) into `(P + flip P)/2` and `A = (P − flip P)/2`, the
correlation with `A` alone reads the orientation: positive is PeSTO's, negative
its mirror. Per release:

| table | `c4f0bbe`…v2.1.0 | v2.2.0 | v2.6.0 = worktree |
|---|---:|---:|---:|
| mg pawn | 0.77 | 0.69 | 0.70 |
| mg knight | **−0.49** | 0.09 | 0.16 |
| mg bishop | **−0.47** | 0.15 | 0.22 |
| mg rook | 0.81 | 0.56 | 0.56 |
| mg queen | **−0.74** | 0.18 | 0.18 |
| mg king | 0.61 | 0.23 | 0.22 |
| eg pawn | 0.82 | 0.62 | 0.68 |
| eg knight | **−0.26** | −0.06 | −0.06 |
| eg bishop | **−0.45** | 0.25 | 0.26 |
| eg rook | 0.78 | 0.45 | 0.47 |
| eg queen | **−0.70** | −0.09 | −0.09 |
| eg king | **−0.42** | 0.66 | 0.61 |

History as a fit-direction test: the movement between revisions projected
onto the start table's flip direction (1 = walked to its flip, 0 = no
orientation preference). The seven mirrored tables moved toward their flips
at v2.2.0 (+0.37 to +1.64); the five correct ones did not (−0.47 to +0.22);
nothing has moved since.

| table | `c4f0bbe`→v2.2.0 | v2.2.0→v2.3.2 | v2.3.2→v2.4.0 | v2.4.0→v2.6.0 |
|---|---:|---:|---:|---:|
| mg pawn | −0.47 | 0.00 | 0.01 | 0.01 |
| mg knight | **0.63** | 0.00 | 0.03 | 0.03 |
| mg bishop | **0.74** | 0.00 | 0.03 | 0.02 |
| mg rook | −0.04 | 0.00 | 0.01 | 0.03 |
| mg queen | **0.66** | 0.00 | 0.01 | 0.04 |
| mg king | 0.02 | 0.00 | 0.00 | 0.01 |
| eg pawn | 0.22 | 0.00 | −0.01 | −0.01 |
| eg knight | **0.37** | 0.00 | 0.00 | 0.00 |
| eg bishop | **1.01** | 0.00 | 0.01 | 0.01 |
| eg rook | −0.31 | 0.00 | 0.00 | 0.00 |
| eg queen | **0.41** | 0.00 | 0.00 | 0.00 |
| eg king | **1.64** | 0.00 | 0.02 | 0.00 |

## Fit-direction test on the current tables

The eg knight and eg queen tables sit near zero on the antisymmetric axis, so
the comparison alone cannot exclude a small residual mirror that the
L2-to-prior fits would preserve. The test: `rarog-texel --tune pst` (the 768
table entries and the ten material values, no pinned slot), `--l2 0`, K fixed,
2,000 epochs, two arms per dataset — **A** from the current tables and **B**
from all twelve tables rank-flipped. Under no orientation preference the arms
move by equal fractions toward the symmetric midpoint; a preferred orientation
shows as B walking back (projection near 1) while A stays, and as both arms
ending in the same place.

`hce-v6-tb` (the WDL dataset the current tables were fitted on,
`tools/results/hce-fit-20261014_212116`, K = 1.61660; train 3,500,000,
validation 194,444; the frozen test untouched):

| table | A: proj onto flip | B: proj onto flip | corr(A, B) | corr(A, flip B) | A: corr with PeSTO `A` | B: same |
|---|---:|---:|---:|---:|---:|---:|
| mg pawn | 0.19 | 0.79 | 1.00 | −0.22 | 0.71 | 0.73 |
| mg knight | 0.25 | 0.77 | 1.00 | 0.64 | 0.30 | 0.33 |
| mg bishop | 0.31 | 0.67 | 0.99 | 0.34 | 0.15 | 0.16 |
| mg rook | 0.29 | 0.78 | 0.99 | 0.45 | 0.35 | 0.45 |
| mg queen | 0.17 | 0.83 | 0.98 | 0.08 | −0.04 | 0.08 |
| mg king | 0.56 | 0.34 | 0.99 | 0.37 | −0.28 | −0.27 |
| eg pawn | −0.25 | 1.46 | 0.99 | −0.47 | 0.82 | 0.87 |
| eg knight | 0.50 | 0.56 | 0.98 | −0.19 | 0.14 | 0.10 |
| eg bishop | 0.11 | 0.94 | 0.99 | −0.65 | 0.30 | 0.30 |
| eg rook | 0.06 | 1.00 | 1.00 | −0.33 | 0.67 | 0.69 |
| eg queen | 0.19 | 0.82 | 0.99 | −0.19 | 0.20 | 0.27 |
| eg king | 0.09 | 0.99 | 0.99 | 0.34 | 0.44 | 0.51 |

Validation loss: A 0.0925921 → 0.0922043, B 0.1125669 → 0.0927119 (both best
at the last epoch; B is still closing the gap).

`hce-v6b-tb` (RAR-E67's blended-target set, K = 1.63724) reads the same: the
arms end within corr 0.96–1.00 of each other in every table, B's projections
are 0.33–1.15 against A's 0.05–0.47, and the endpoints' sign against PeSTO's
`A` matches the table above in all twelve (eg knight +0.13/+0.08, eg queen
+0.02/+0.08, mg king −0.21/−0.27).

Reading:

- Both arms end at the same tables, so the data has one optimum and it has
  the current orientation. The eg knight and eg queen tables carry little
  rank dependence in this engine's data (|corr with `A`| ≤ 0.27), but what
  there is lies on PeSTO's side. No residual mirror.
- The mg king is the one table whose fitted rank dependence is *opposite* to
  PeSTO's in both arms (−0.28, −0.27). Its seed was imported correctly and
  the fit has had the table free since v2.2.0; this is the data's preference
  with the king-danger terms carrying the king's placement, not an
  orientation defect, and the comparison at the top reads it as correct.
- The eg knight's A projection (0.50) is the largest in arm A because its
  flip direction is the smallest (17.5 cp mean); its absolute movement is
  ordinary.

## An observation outside the question

A free PST-only refit on the dataset that produced the current tables moves
them by 11–57 cp mean per entry (mg king 57, eg queen 40) and improves
validation loss by 0.42%. The complete fits hold the tables nearer their
stage prior than the data alone would (L2 `1e-7` on sparsely active
entries). This is a loss screen with every other term held, not Elo, and the
evaluation is frozen at `9fb525e`; it is recorded here as a fact about the
anchor, not as a candidate.

## Relevance to E.3.0

The piece-square tables were seeded from PeSTO (Ronald Friederich; the
tables as published on the Chess Programming Wiki). No entry of any table
has been identical to PeSTO since v2.2.0, and the fits have reshaped them,
but PeSTO is their origin and E.3.0's disclosure should name it.

## Evidence layer

Table comparison, fit loss and fit direction. None is Elo, and no games were
played.

## Recipe and artifacts

`analysis/artifacts/pst-orientation-2026-10-23/` (ignored; `SHA256SUMS.txt`
hashes every file):

- `pst_orientation.py`, SHA-256
  `25593e7fdc38e4b44de327e8dc67b2933eea3ab7db7cc88a4792c38290ad0fe3`: PeSTO
  embedded as published, extracted mechanically from the page fetched
  2026-10-23 (`pesto_cpw_2026-10-23.html`,
  `5b395ae971171051f6a13f082c536c25a9ea1127d88cb58bc364933eeb2d0468`) and
  cross-checked equal to Basilisk's independent transcription
  (`D:/code/basilisk/tools/results/pst-orientation-20261023/pesto.txt`,
  `9ba085cda79d8841245deecf41a8f6294add3cbd9fb0f8eebd66a59e0c66ef95`);
  asserts the indexing from the source, extracts the twelve consts at the
  initial commit, every `v*` tag and the working tree, and writes
  `pst_orientation_report.{txt,json}` and `run.log`.
- `fit_direction.py` (`2a2c4c79d1cc5e0b8f72cb19a6cf056a8ba28edf5e4c6721c8e2c7cc8374cb75`)
  and `pst_vector_tools.py`
  (`6054d374a64df1ffbf01cb5865452691ecac2aabb429fe904ed884ed307829fa`):
  the projections, endpoint comparison, antisymmetric reading, and the
  vector check and flip.
- Tuner: `cargo build --release --manifest-path tools/texel-tuner/Cargo.toml`
  on `d1` at `abd318e` (eval source identical to `9fb525e`); `--write-defaults`
  gave `defaults.txt`, checked equal to `src/eval/material.rs` in all 768
  entries; `defaults_flipped_all.txt` flips all twelve.
- Fits, 2,000 epochs each, with logs and output vectors beside them:

```
rarog-texel --tune pst tools/texel/data/hce-v6-tb/train.csv tools/texel/data/hce-v6-tb/validation.csv v6_fit_from_current_e2000.txt --l2 0 --fix-k 1.61660 --epochs 2000
rarog-texel --tune pst tools/texel/data/hce-v6-tb/train.csv tools/texel/data/hce-v6-tb/validation.csv v6_fit_from_flipped_e2000.txt --initial defaults_flipped_all.txt --l2 0 --fix-k 1.61660 --epochs 2000
```

  and the same pair on `hce-v6b-tb` with `--fix-k 1.63724`
  (`fit_from_*_e2000.*`); the 200-epoch pilots (`fit_from_*.{txt,log}`)
  are kept and read the same way, unconverged.
