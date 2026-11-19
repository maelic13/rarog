# RAR-S130 — D.7.8.1: ProbCut's exchange bar at the donor's scale, 2,536, against the fitted 1,024, 2,000 games — REGISTERED 2026-11-19, before any game

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-19, before any game. The run is the
  maintainer's command.
- **Kind:** PLAN D.7.8.1, a repair inside D.7's one cluster: no gate of its
  own (D.7, *How the open steps are worked*), one 2,000-game read under the
  step's own rule.
- **Inherits:** the audit's finding A1 (`analysis/d7_audit_2026-11-19.md`):
  ProbCut's picker holds captures to `probcut_beta − eval`, a distance in
  the evaluation's units, against the donor's exchange values; D.7.4.1's
  fold converted quiescence's bar (`q_see_scale`) and not this one, so the
  bar is 0.40 of the donor's, and layer B read `probcut_depth` firing 27%
  more a position.

## The change (engine `ee1c79c`)

The bar is a rule, `probcut_see_bar`, scaled by the seed
`probcut_see_scale` in 1024ths. The fitted table keeps **1,024**, the bar
the fit played: `bench 13` **3,488,485 / EBF 2.336** on all forty
positions, the fingerprint. The donor's table carries **2,536**, the
donor's form at the fold's scale. The tuning option `KProbcutSeeScale`
takes a declared range, 512 to 4,096 (the range rule's 512 to 2,048 would
not reach 2,536).

## Arms and run (frozen)

The tune binary on both sides, `rarog-d781-tune.exe` (built 2026-11-19 by
`tools/build_test.ps1 -Suffix d781 -Tune` from the clean tree at `ee1c79c`,
`cargo xtask build --tier pext --nopgo --features tune`, rustc 1.100.0,
SHA-256 `470fe1ce6a4549c19cb1d3cc2e7e9964cdf95d1193c058ff8c64ec2e8fed5634`,
sidecar bench 3,488,485), `-CategoricalTuneBuild`:

- **`d781donor`:** `KProbcutSeeScale=2536`, every other seed fitted.
- **`d781fit`:** every seed fitted (the scale 1,024).

Colosseum `match` through `tools/colosseum.ps1`, 2,000 games at `3+0.03`,
Hash 64, one thread, the wrapper's slot count, the UHO book in random
order, no adjudication, seed 20261121, `tools/results/rar-s130`. The
runner is Colosseum and does not change. The dry run passed every
preflight, the option on side A only
(`tools/results/colosseum_match_rar-s130_20261119_182804.dry-run.json`).

```powershell
./tools/colosseum.ps1 -Mode match -CategoricalTuneBuild -EngineA tools/test_engines/rarog-d781-tune.exe -EngineB tools/test_engines/rarog-d781-tune.exe -NameA d781donor -NameB d781fit -OptionsA KProbcutSeeScale=2536 -ExpectBench 3488485,3488485 -Dir tools/results/rar-s130 -Games 2000 -Seed 20261121
```

**Exposure:** ProbCut runs in every searched position class: on the forty
bench positions at depth 13 it ran at 302,278 nodes of 3,488,485, so every
game reaches the repair's domain; no database query narrows it.

**Validity:** every game a rules result; a run with time losses or engine
faults in more than 0.5% of its games is void.

## Reading (frozen)

From the manifest's nElo and its 95% interval (PLAN D.7.8.1's rule):

- **The interval wholly below zero:** the seed stays 1,024.
- **Anything else:** the donor's 2,536 becomes the fitted table's seed, in
  its own commit, and the fingerprint moves to its bench.

Either way the scale is a coordinate of D.7.11's surface. The read decides
the seed's starting value and accepts nothing as a gain.

## Zero-game companion, read before the games

`analysis/artifacts/d781/companion.py` (`companion.txt`): on the tune
binary, `bench 13` with the option set in the session; on a tune+probes
build of `ee1c79c`, the forty bench positions at depth 13 in one session,
the table shared as the bench shares it (the forty searches total the
bench's nodes exactly).

| Scale | `bench 13` | ProbCut runs | Captures tried | a 1,000 nodes | Verifications |
| --- | --- | --- | --- | --- | --- |
| 1,024 | 3,488,485 / 2.336 | 302,278 | 174,215 | 49.94 | 36,315 |
| 2,536 | 3,278,226 / 2.313 | 285,521 | 135,992 | 41.48 | 35,504 |

The donor's bar lets 17% fewer captures a node into ProbCut's search and
leaves the verifications almost unchanged: it removes captures that would
not have verified. A fingerprint and a rate, not Elo.

## PRE-REGISTERED PREDICTION (frozen 2026-11-19, before any game)

- **`d781donor`:** **−2 nElo**, 80% band **−20 to +16**; P(interval wholly
  below zero) 0.10.
- **Basis:** the donor's bar is its fitted form and trims captures that
  rarely verify (verifications −2%, attempts −17%), which should cost
  nothing and save a little time; against it, RAR-S128 fitted ProbCut's
  margin, depth and raise around the unconverted bar, so the 1,024 table's
  neighbours may have absorbed the defect, and a correct bar on an
  incorrectly fitted neighbourhood can read slightly worse.
- **Most likely failure mode:** a small loss from the fitted ProbCut seeds
  tuned to the wider bar; the interval still crosses zero.
- **Evidence layer:** games on a tune build without PGO, both sides alike.
