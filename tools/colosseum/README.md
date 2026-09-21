# Rarog's policy on Colosseum CLI

Committed run files for Colosseum CLI (`D:/code/colosseum`), staged as
`tools/bin/colosseum-cli.exe`. They hold the conditions every Rarog measurement
shares, so a command line carries only what belongs to one experiment: the
engines, the cap, the seed and the run directory.

The harness is qualified in its own repository (Colosseum PLAN Phase 10, GUIDE
10.9r: scale, verdict, ratings and the SPSA recovery test, with Rarog as the
validation engine). Rarog trusts a released binary and repeats none of it. A
null pair is owed here only on the trigger PROCESS names — a runner, scheduler
or CPU-topology change on this host — and `calibrate-null.toml` is what runs it.

| File | Workflow | Bracket or size |
|---|---|---|
| `common.toml` | inherited conditions only, never run alone | — |
| `sprt-default.toml` | the ordinary gate | `[0, 3]` nElo |
| `sprt-removal.toml` | a removal or simplification | `[-1.75, 0.25]` |
| `sprt-repair.toml` | a repair of unknown sign | `[-5, 5]` |
| `sprt-wide.toml` | a genuinely large prior, stated in the registration | `[0, 10]` |
| `match-fixed.toml` | a measurement with an interval, never an acceptance | 2,000 games |
| `spsa-tune.toml` | a tune | horizon per registration |
| `calibrate-null.toml` | the null pair, on its trigger | 30,000 games |
| `gauntlet.toml` | a rating gauntlet against a fixed field | per event |

Shared conditions: `3+0.03`, Hash 64, one thread, the UHO book in random order,
no adjudication, a 20 ms time margin, `--placement auto` with one physical core
of headroom, and engine processes kept per slot. Gates run 14 concurrent games;
a tune runs 15, because both perturbation arms share one slot.

## Running one

```powershell
./tools/bin/colosseum-cli.exe --run-file tools/colosseum/sprt-default.toml `
  tools/test_engines/<candidate>.exe tools/test_engines/<baseline>.exe `
  --max-pairs <cap from RAR-M10> --seed <n> --dir tools/results/<experiment>
```

The cap, the seed and the directory stay on the command line: the cap belongs to
the registration in `EXPERIMENTS.md`, and a seed is chosen and recorded per run.
Bounds, book and adjudication never change after games are seen.

`tournament` needs its command spelled on the command line, because its options
are parsed after the subcommand:

```powershell
./tools/bin/colosseum-cli.exe --run-file tools/colosseum/gauntlet.toml tournament run `
  --format gauntlet --engine <a>.exe --label "..." --rating <n> --fixed 2:<rating> ...
```

Inspect what a command will do before it plays anything, and keep the output
with the evidence:

```powershell
./tools/bin/colosseum-cli.exe --run-file tools/colosseum/sprt-default.toml `
  <a>.exe <b>.exe --max-pairs 40000 --seed 7 --dir tools/results/x --dry-run --json
```

## Tune surfaces

A registered surface lives in `tools/spsa_configs/config_<group>.json`. Convert
it once per horizon, because `c_end` is the perturbation at the end of the run:

```powershell
python tools/spsa_config_to_colosseum.py b23core --iterations 5000
```

That writes `tools/spsa_configs/colosseum/<group>.tune.toml`, the 82-coordinate
vector, and `<group>.run.toml`, which extends `spsa-tune.toml` and applies
`fixed_<group>.json` to both arms. `--check` fails when either file no longer
matches the JSON, so a surface cannot drift from its registration. Run a tune
with:

```powershell
./tools/bin/colosseum-cli.exe --run-file tools/spsa_configs/colosseum/b23core.run.toml `
  tools/test_engines/rarog-b23core-tune.exe --total-games <registered> `
  --seed <n> --dir tools/results/<experiment>
```

## Still owed

`tools/sprt.ps1` and `tools/spsa.ps1` remain the gate and tune path until
B.2.6.1's wrappers carry every guard they enforce today — an idle host, a
bench-verified sidecar, a manifest per run — and `setup_tools.ps1` stages a
tagged release and pins its SHA-256. fastchess and weather-factory stay
installed until then, and fastchess stays afterwards for periodic cross-checks.
