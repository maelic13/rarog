# Analysis and local evidence

Development and experiment evidence are kept on the maintainer's primary
machine. Other machines run macOS and Windows-on-ARM compatibility checks.

Git retains the Markdown analyses, decisions, experiment recipes and result
summaries needed to develop Rarog. The following are local-only and ignored:

- `analysis/artifacts/`: raw results, manifests, frozen experimental adapters,
  candidate patches/vectors, validation logs and compressed evidence bundles.
- `analysis/*.txt`: standalone generated measurement reports.
- `tools/results/` and the existing ignored build/data directories: run outputs,
  executables, profiling traces and other generated material.

Paths to those files in older analyses deliberately refer to local evidence;
they will not be populated by a fresh clone. Preserve them on this machine.
Keep the recipe, relevant source/binary identity and result in the tracked
analysis or EXPERIMENTS ledger. Do not force-add a raw bundle to make its link
work on another machine. Arrange local evidence backups separately from Git.

Required fixtures under `tests/data/`, frozen ranking/floor inputs consumed by
diagnostic tools, reusable scripts, vendor source/licenses and the README logo
remain tracked. A generated origin alone does not make a required test input
disposable. Unused logo variants are kept locally under explicit ignore rules.

The storage cleanup removes files from the current Git index only. It preserves
their bytes and paths on disk and leaves earlier commits unchanged; old blobs
will therefore still contribute to repository history size.
