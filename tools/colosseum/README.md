# Rarog's policy on Colosseum CLI

Committed run files for Colosseum CLI, staged as `tools/bin/colosseum-cli.exe`
by `tools/setup_tools.ps1` from the revision and SHA-256 in
`colosseum.pin.json`. They hold the conditions every Rarog measurement shares,
so a command line carries only what belongs to one experiment: the engines, the
cap, the seed and the run directory.

**Run them through `tools/colosseum.ps1`**, which carries the guards: an idle
host, the runner pin, each engine's sidecar, build-flavour and compiler
equality, `-ExpectRevision` and `-ExpectBench`, the advertised options, the tune
surface against the binary and the horizon, and a field-by-field check that the
configuration Colosseum resolves is the policy below. It writes a manifest that
hashes every input. After the run it reads the exit code as the command's
verdict (an SPRT's H0 exits 1 and a cap stop 4; only an invalid, cancelled or
failed run ends the wrapper), reads the faults from `colosseum-cli status
--json` and refuses any it cannot read, and recounts the pentanomial from the
PGN, oriented by the journal's sides, against the runner's own count. An
existing `-Dir` resumes at the seed it recorded.
Calling `colosseum-cli.exe` directly, as the examples below do, skips all of
that and is for inspection, not for a measurement anyone will cite.

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
| `match-fixed-ltc.toml` | the same at the direction-check control `10+0.1` (with `-BaseMs 10000 -IncrementMs 100`) | 1,000 games |
| `spsa-tune.toml` | a tune | horizon per registration |
| `calibrate-null.toml` | the null pair, on its trigger | 30,000 games |
| `gauntlet.toml` | a rating gauntlet against a fixed field | per event |

Shared conditions: `3+0.03`, Hash 64, one thread, the UHO book in random order,
no adjudication, a 20 ms time margin, `--placement auto` with one physical core
of headroom, and engine processes kept per slot. Gates run 14 concurrent games;
a tune runs 15, because both perturbation arms share one slot.

## Running one

```powershell
./tools/colosseum.ps1 -Mode sprt `
  -EngineA tools/test_engines/<candidate>.exe -EngineB tools/test_engines/<baseline>.exe `
  -NameA candidate -NameB baseline `
  -MaxPairs <cap from RAR-M10> -Seed <n> -Dir tools/results/<experiment>
```

`-Bracket removal | repair | wide` picks the other three run files, `-Mode match`
a fixed-length measurement, `-Mode calibrate` the null pair, `-Mode spsa` a tune
and `-Mode gauntlet` a rating gauntlet. `-DryRun` stops after the checks.

The same run, called directly (no guards, no manifest):

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
./tools/colosseum.ps1 -Mode spsa -Engine tools/test_engines/rarog-b23core-tune.exe `
  -ConfigGroup b23core -Iterations <N> -TotalGames <N*30> `
  -Seed <n> -Dir tools/results/<experiment>
```

The wrapper re-runs `--check` before every tune, so a surface that has drifted
from its registration cannot be tuned by accident.

## The runner pin

`colosseum.pin.json` names the source revision and the SHA-256. Only the hash
can be enforced — a stripped release executable does not carry its revision —
so `setup_tools.ps1` and every wrapper run compare hashes and refuse a
mismatch rather than substituting a build. Since 2026-09-22 the pin is the
published release `cli-v0.1.0` (`40a15b1b`): its `archive` entry carries the
asset URL and the digest GitHub serves for it, checked on download, and the
top-level `sha256` is checked on the extracted executable. Re-pinning to a
later release is an edit to that one file.

The released 0.1.0 reports the same version string as the local build that
preceded it, so the hash is the only identity that separates them — and it
resolves these run files to a configuration identical to that build's in all
439 dry-run fields.

## The backup path

`tools/sprt.ps1` and `tools/spsa.ps1` stay installed, working and documented as
the backup and the second opinion until at least release 2.5.0. Nothing here is
retired; PROCESS's *Harness* section names the three cross-check triggers.
Both paths share one implementation of every guard, in
`tools/harness_common.ps1`.
