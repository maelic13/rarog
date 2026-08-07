# Engine testing with Colosseum CLI

Rarog delegates generic UCI orchestration, statistics, SPSA scheduling,
affinity, persistence and result analysis to `colosseum-cli`. The engine stays
an ordinary UCI executable: it has no Colosseum manifest or build contract.

Install `colosseum-cli` separately and put it on `PATH`. Run all examples from
the Rarog repository root. The committed TOML files contain project policy;
engine paths, opening books, run directories and hardware concurrency remain
explicit arguments.

## Strength tests and calibration

The Rarog strength profiles preserve the historical `3+0.03`, Hash 64,
Threads 1, draw 40/8/10 and one-sided resign 600/3 conditions. New experiments
may remove `one-sided-resign-adjudication` to use Colosseum's safer two-sided
default. No opening book is bundled; add `--book <epd-or-pgn>` when one is
available. Choose concurrency for the current host and retain automatic
whole-core placement:

```powershell
colosseum-cli --run-file tools/colosseum/profiles/sprt-gainer.toml `
  sprt <candidate> <baseline> --book <book.epd> --concurrency 14

colosseum-cli --run-file tools/colosseum/profiles/sprt-simplify.toml `
  sprt <candidate> <baseline> --book <book.epd> --concurrency 14

colosseum-cli --run-file tools/colosseum/profiles/calibrate.toml `
  calibrate <engine> <byte-identical-copy> --book <book.epd> --concurrency 14
```

Calibration is optional evidence after changing the runner, clocks, placement
or host. It is not required before ordinary development tests.

## SPSA

Build a tune-enabled engine with Rarog's own build tooling, select one committed
parameter vector, and let Colosseum own the schedule and durable state:

```powershell
.\tools\build_test.ps1 -Suffix lmr -Tune
colosseum-cli --run-file tools/colosseum/profiles/spsa.toml `
  spsa <tune-engine> --tune tools/colosseum/tunes/lmr.toml `
  --book <book.epd> --concurrency 14 --dir <run-directory>
```

The vectors were converted from the former weather-factory 5,000-iteration
schedule. Their terminal perturbation is
`max(0.5, legacy_step * 5000^-0.102)`; the floor prevents an integer UCI option
from receiving identical plus/minus values. Use `spsa plan` before a long run,
`spsa status` while it is running, and `sprt --apply` to gate its final vector.

## Other workflows

```powershell
# UCI schema and compliance
colosseum-cli engine inspect <engine> --json
colosseum-cli engine check <engine> --json

# Fixed-node speed, pooled builds and thread scaling
colosseum-cli nps <candidate> --against <baseline> --nodes 10000000 --repetitions 12
colosseum-cli nps <engine> --nodes 10000000 --scale-threads 1,2,4,8 `
  --threads-option Threads --hash-policy fixed-total --hash-mb 64

# Round-robin or gauntlet; repeat --engine for every participant
colosseum-cli --run-file tools/colosseum/profiles/tournament.toml `
  tournament run --engine <rarog> --engine <opponent> `
  --book <book.epd> --concurrency 14 --dir <run-directory>

# Fixed-node self-play PGN for the engine-owned Texel extractor
colosseum-cli --run-file tools/colosseum/profiles/datagen.toml `
  match <engine> <engine> --book <seed.epd> --book-start 0 `
  --book-order random --games 20000 --concurrency 14 --dir <run-directory>
```

Use `stats`, `stats telemetry`, `suite` and `book` for offline result analysis,
EPD correctness suites and opening inspection. The generated `games.pgn` is
the input to `tools/texel/extract.py`; Colosseum does not own Rarog's sampling,
label extraction, fitting or source-value baking.

## Responsibility boundary

| Colosseum CLI owns | Rarog owns |
|---|---|
| UCI launch/probe, matches, SPRT, calibration and tournaments | Source, UCI option semantics and engine correctness |
| SPSA schedule, audit, persistence, resume and final-vector artifacts | Tune-enabled builds and baking accepted values into source |
| CPU topology/affinity, concurrency execution and seeds | Compiler, flags, PGO/ISA builds and build comparability |
| NPS experiments, PGN/result statistics, telemetry and EPD suites | Profiling and engine-specific diagnostic counters |
| Opening parsing, ordering and non-reuse policy | Book selection and generated Texel data/labels |

The retired PowerShell/Python harness implementations remain available in Git
history. Historical experiment records may still cite them; those citations
identify old evidence and are not current commands.
