# RAR-M52 — B.2.0.1 repository and document census on `4e60e45`, 2026-09-14, zero games, nothing changed

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**B.2.0.1 repository and document census on `4e60e45`, 2026-09-14, zero games, nothing changed.** `git ls-files` inventory; a script counting, for every tracked Markdown file, the other tracked files (`.md`, `.py`, `.ps1`, `.rs`, `.toml`, `.yml`) that cite it by file name; a scan of every tracked Markdown file for backticked repository paths (`analysis/`, `tools/`, `docs/`, `src/`, `tests/`, `benches/`) that exist neither in the index nor on disk; section, row and prose-line counts of `EXPERIMENTS.md`; grep censuses of policy phrases across the top-level documents; the first paragraph of every `analysis/` document read for its self-declared status. Artifacts in ignored `tools/results/b20-20260914/`, sha256: doc_census.txt `72175ae076811a49648bf3874f8c9df074784d52591bd08ab7fcc338c919e52e`, ledger_shape.txt `797d27ebebfaecc41d3c93e8f887e6341fc24f341c69417297b7f1a077ced72f`.

## Result / disposition

273 tracked files; top-level Markdown PLAN 1,508 / EXPERIMENTS 1,674 / HISTORY 928 / CHANGELOG 912 / PROCESS 448 / AGENTS 447 / GUIDE 249 / README 178 lines; **PLAN holds 734 lines of closed-leaf narrative** (Phase A 532, B.0 and B.1 202) and eight retained "Original scope" paragraphs; **HISTORY's 540-line forward tracker is a second copy of the archived Phase-4 board and its legacy-tracker section holds no items** although the resolution table points at it; EXPERIMENTS 183 rows in eight series, §2 with 589 prose lines around 20 rows, four gates (RAR-E06, E08, E12, E13) as prose beside 179 rows, §10 a second copy of PROCESS's template; **44 dangling repository paths** (PLAN 5 in current text, EXPERIMENTS 9, analysis 21, archive 8, `tools/texel/README.md` 1); README `cargo test --workspace` against CI's `-p rarog`; `analysis/` 71 files, 2 uncited, 13 superseded or pre-head audits beside 11 live contracts, no index; logo 9 tracked, 1 used, against the storage policy's claim; "adjudicat" in five documents (EXPERIMENTS 51, PLAN 15, PROCESS 13).

## Conditional lesson and retry trigger

The set is complete and mechanically checked where it matters but not single-purpose: each working document carries a second one inside it. All of it is documents-only work. **Prediction frozen for B.2.0.1:** PLAN ≤ 950, HISTORY ≤ 450 (+ a recovered 300–400 line legacy archive), EXPERIMENTS ≤ 1,450 with 183 rows unchanged, AGENTS ≤ 280, PROCESS ≤ 350, GUIDE ≤ 220 lines; dangling paths in GUIDE/PLAN/PROCESS/AGENTS 0; logo 3 tracked; 13 analyses archived, none deleted. Most likely failure: the legacy tracker is not recoverable in one piece from history. Repeat trigger: E.1 re-runs this recipe on the release head.

## Source

`analysis/repository_review_2026-09.md`; PLAN B.2.0.1; RAR-M51; `tools/results/b20-20260914/` (ignored, hashed above)
