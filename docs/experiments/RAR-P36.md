# RAR-P36 — Experimental Dragon NNUE selector and single scalar speed read

## Experiment and conditions

2026-10-15, PLAN F.11, explicitly requested on `dragon_nnue`; baseline
`52cc6de`. Primary work: implementation/deterministic qualification, followed by
an informational performance read. The maintainer authorized an imported-model
comparison independently of Phase F's own-data release requirement. No games,
SPRT, tune, release acceptance or Elo hypothesis. Contract/prediction were frozen
in PLAN before implementation: native scalar inference should preserve exact
diagnostic scores; full-refresh scalar evaluation should be substantially slower.
Any score mismatch, stale cache/TT/correction, changed default HCE fingerprint,
illegal move, inference allocation or required check failure stops completion.

Engine commit `c8d1e8e`; tooling `4f6e0a9`. Rust 1.98.1,
x86_64-pc-windows-msvc, ordinary `cargo build --release --locked`, default
features, no PGO. The measured archived executable is
`analysis/artifacts/dragon-nnue-20261015/rarog-dragon-scalar-v2.exe`, SHA-256
`76c0b5dccefc2d2bf33440446d2ec9c5c4eac8f77dee93394b0d8138f05b09c4`. Model SHA-256
`c6e25a7fde5971dc47e78e19764c82b8d18a438bbd5ee4da065d061115064417`.
Runtime/model provenance is in `vendor/dragon-nnue/PROVENANCE.json`; the probe
checks and records both complete original executable identities.

Exactly one `bench 13` per evaluator, same binary, Hash 64, Threads 1,
scale 72, Classical then Dragon NNUE. No warm-up/repeats/pool. A preliminary
idle check refused at 36% CPU before either arm ran. The maintainer explicitly
authorized this pair despite shared-host load; `--allow-busy` is in the manifest.
CPU before arms was 15% and 2%, respectively; no engine/build process was detected.
That asymmetry prevents a precise speed interpretation. The temporary 3 KiB
per-node scalar-bias copy exception was registered before timing.

## Result / disposition

| Evaluator | Positions | Nodes | EBF | Time | NPS |
|---|---:|---:|---:|---:|---:|
| Classical HCE | 40/40 | 14,197,472 | 2.557 | 7,905 ms | 1,796,011 |
| Dragon NNUE, scalar | 40/40 | 17,728,094 | 2.604 | 85,064 ms | 208,408 |

NNUE/HCE throughput 0.116× (about 8.62× lower);
time-to-depth 10.76×. Both tree shape and evaluator cost change.
This is a rough single read under background load, with no interval and no Elo
conversion. Calibration: expected slower direction observed; magnitude was not
predicted and is not a robust kernel-cost estimate. HCE exactly preserves the
development fingerprint 14,197,472 / EBF 2.557.

Static parity uses the engine's new `eval` command and native Dragon `nnue`,
before Rarog search corrections. All 2,738 frozen rows pass in debug/release;
both original executables freshly match all 2,770 cases each, **5,540/5,540**
exact comparisons, with two changed-score controls rejected. Coverage: all 8
heads, every king square for each color, both sides, five scales, and 32 live
move-prefix positions including special moves and underpromotions. All fixture
positions passed Rarog's parser; no score or parser exclusion was needed.

Default debug/release suites pass; model-specific tests cover cache/mode/scale,
TT and correction clearing, live shared-TT helper ownership, make/unmake/null,
special moves and zero inference allocations. Legal short searches pass with
one/two workers. The archived UCI executable additionally plays 16 legal plies
across both evaluators and one/two threads, with each move validated by the host
parser; `eval` during infinite search keeps stop/quit responsive. fmt, all-feature/
all-target clippy and six Python parser/negative tests pass. The final ordinary
release rebuild is byte-identical to the measured archive.

Board generation, diff and changed-document link checks pass. The full
`check_guide.py` reports the same 14 missing historical artifact paths as
baseline `52cc6de`; there are no new path or structural problems.
`check_citations.py` cannot run because this clone has no PR refs. These existing
checkout/provenance limitations are retained in `docs-checks/`; no historical
reference was changed or invented to make the checks pass.

Disposition: **F.11 DONE for the requested experimental integration**. Classical
remains the default. Imported model strength, margin compatibility, intermediate
activation parity and optimized runtime performance remain unqualified. F.12
owns optional runtime optimization; this is not acceptance of F.0–F.10.

## Reproduction and raw evidence

Use the commands in [DRAGON_NNUE.md](../DRAGON_NNUE.md). Exact timing command:

```powershell
python tools/diag/dragon_nnue_bench.py `
  --engine analysis/artifacts/dragon-nnue-20261015/rarog-dragon-scalar-v2.exe `
  --model D:/code/dragon/assets/dragon-network.nnue `
  --out analysis/artifacts/dragon-nnue-new/bench --allow-busy
```

Raw root: `analysis/artifacts/dragon-nnue-20261015/`.
`checks-v2/` holds final command exits/logs; `native-probe-v2/` holds frozen
inputs, transcripts and every exact score; `play-smoke-v1/` holds the actual UCI
move/diagnostic records; `bench-v2/` holds all 80 position rows, summaries,
CPU checks and binary/model identity. Earlier `checks-v1/`, `native-probe-v1/`
and the idle refusal `bench-v1/` are retained. `manifest.json` records hashes.

## Conditional lesson and retry trigger

The scalar integration is usable and exact at the qualified diagnostic layer;
speed is the next runtime question. Optimize only against the frozen reference,
then remeasure with the host/load and build protocol that question needs. Any
playing-strength comparison must separate timed and fixed-node evidence and
account for Rarog's HCE-fitted search. No donor search or scale retune is part of
this reconstruction experiment.
