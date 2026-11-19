# RAR-S133 — D.7.8.10: the kernel on a unit half the size, emulated through the tune options, 2,000 games at 100,000 nodes a move — REGISTERED 2026-11-20, before any game; READ 2026-11-20: nElo −4.9 ± 15.2, NO_CHANGE

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-20, before any game. Run by the
  agent overnight on the maintainer's delegation of 2026-11-19 ("you are
  allowed to run long matches and reads during this night").
- **Disposition:** read 2026-11-20: nElo −4.9 ± 15.2, the interval
  [−20.1, +10.3] not wholly above zero: D.7.8.10 closes `NO_CHANGE`.
  Inside the prediction's 80% band.
- **Kind:** PLAN D.7.8.10, a `V` screen inside D.7's cluster, its rule
  frozen in PLAN: an interval wholly above zero sends an `I1` step that
  folds the finer unit in before D.7.9; anything else closes the step
  `NO_CHANGE`. It accepts nothing by itself.
- **Inherits:** the audit's finding A3 and decision 5
  (`analysis/d7_audit_2026-11-19.md`): the fitted wrapper hands the kernel
  0.50 to 0.82 of the HCE's value in whole units and the margins were
  folded to that unit; the donor works about 3.5 times finer.

## The arms (frozen)

Both are one binary, the tune build of `991bf15` (engine `b88700a`),
`tools/test_engines/rarog-d7810-tune.exe`, `id name`
`dev-20261120-991bf151`, SHA-256
`cfbc557c81d046702b0d4133585698da8067311f1b75f3c845e1ffb2c0ee43ef`, `bench
13` 3,320,355 / EBF 2.317 at its defaults, the fingerprint.

- **`d7810unit`:** the defaults, the fitted seeds.
- **`d7810fine`:** the same rules on a unit half the size. The 65 options
  in `analysis/artifacts/d78-night/finer_unit_options.txt`, from the tune
  build's own defaults: 52 seeds in evaluation units doubled, 13
  coefficients per evaluation unit halved, and `wrap_divisor` halved, which
  doubles every evaluation. `bench 13` with them: 3,417,466 / EBF 2.311.
  `KWrapDivisor` halved alone reads 5,337,997, so the options reach the
  search and the others carry the unit.

**The classes** (`analysis/artifacts/d78-night/finer_unit.py`,
`seed_fold.tsv`): each seed's factor at D.7.4.1's fold, its folded value
(`8489e46`) over the donor's value in the scaffold (`a344748`), is a power
of k = 1024/2536 to within 2%: k (18 seeds) is evaluation units, 1/k (10)
per evaluation unit, 1 (181) neither. The 38 seeds the fold renamed
(sixteenths and 1024ths) are classed by meaning: 35 margins and clamps in
evaluation units (razoring, reverse futility, the null move's margin,
ProbCut's raise, the singular margins, both futilities, the re-search
bounds, the correction tables' limits, the aspiration window's terms) and
three coefficients into exchange units (`probcut_see_scale`, `q_see_scale`,
`q_see_correction_scale`). Two wrapper seeds are set by hand: `wrap_divisor`
halves, and `optimism_scale` stays, because optimism is added in the raw
evaluation's units ahead of the divisor (the fold scaled it only because
the raw unit changed then); `optimism_offset`, compared with root scores,
doubles.

**What the emulation does not reach:** `q_see_correction_scale` is not a
tune option, so the correction's term in quiescence's exchange bar doubles
with the correction, up to its cap of 68; constants that are not seeds keep
their old size on the finer unit (the draw score's ±2, the decisive and
tablebase bands, the futility values' rounding to whole units); score
output halves in size, which no adjudication reads.

## The run (frozen)

Colosseum `match` through `tools/colosseum.ps1`, `-CategoricalTuneBuild`,
2,000 games at 100,000 nodes a move each (one root driver, no clock, so the
time manager's slopes do not enter), Hash 64, one thread, the wrapper's slot
count, the UHO book in random order, no adjudication, seed 20261125,
`tools/results/rar-s133`. The runner is Colosseum and does not change. The
dry run passed every preflight and showed all 65 options on arm A and none
on arm B
(`tools/results/colosseum_match_rar-s133_20261120_001200.dry-run.json`, the command below).

```powershell
./tools/colosseum.ps1 -Mode match -CategoricalTuneBuild -EngineA tools/test_engines/rarog-d7810-tune.exe -EngineB tools/test_engines/rarog-d7810-tune.exe -NameA d7810fine -NameB d7810unit -OptionsA (Get-Content analysis/artifacts/d78-night/finer_unit_options.txt).Split(',') -ExpectBench 3320355,3320355 -Nodes 100000 -Games 2000 -Seed 20261125 -Dir tools/results/rar-s133
```

**Exposure:** every evaluation and every margin, so every game.

**Validity:** every game a rules result; engine faults or forfeits in more
than 0.5% of the games void the run.

## Reading (frozen)

From the manifest's nElo and its 95% interval: **the interval wholly above
zero** sends PLAN's `I1` step, the finer unit folded into the kernel before
D.7.9; **anything else** closes D.7.8.10 `NO_CHANGE`, with the counts. The
evidence layer is games at equal nodes: no clock, and the two arms run the
same arithmetic at the same speed.

## PRE-REGISTERED PREDICTION (frozen 2026-11-20, before any game)

- **`d7810fine`:** **+3 nElo**, 80% band **−10 to +16**; P(interval wholly
  above zero) 0.12.
- **Basis:** the rules are the same and the tree moves 2.9%, so the effect
  is the rounding a coarse unit costs; the margins already carry sixteenths
  where a whole unit would move them more than 1%, which leaves little to
  gain; the unscaled constants above work slightly against the fine arm.
- **Most likely failure mode:** a null inside ±15 nElo, read `NO_CHANGE`.

## Result (read 2026-11-20)

Run 2026-11-20 00:13 to 00:32 local, 18 minutes, Colosseum CLI 0.5.0
(pinned `8fa63cc`), placement automatic with a core of headroom, the
wrapper's idle check passed on the first attempt. The runner's report and
the manifest
(`tools/results/colosseum_match_rar-s133_20261120_001346.manifest.txt`,
both arms `git_sha` `991bf15`, bench 3,320,355):

- "Elo -3.1 +/- 9.7; nElo -4.9 +/- 15.2 over 2000 paired games"
- `d7810fine` 563 W / 581 L / 856 D; pentanomial 47, 230, 459, 222, 42
  ("recount of games.pgn agrees"); faults none, engine faults none.
- search, `d7810fine`: depth 18.33 mean, 69,366 nodes a move;
  `d7810unit`: depth 18.40 mean, 69,230 nodes a move; both 56.3 ms a move
  (a move's last completed iteration, so under the 100,000 budget).

**Reading:** the interval [−20.1, +10.3] is not wholly above zero:
**`NO_CHANGE`**. The kernel keeps its fitted unit, and D.7.11's fit runs
on it.

### Calibration (appended after the result)

Predicted +3 nElo, 80% band −10 to +16, P(interval wholly above zero)
0.12; read −4.9, inside the band, below the centre. The basis held: a
rescale that moves the tree 2.9% shows no effect at 2,000 games. Whether
the unscaled constants cost the fine arm a little, or there is nothing
to gain, this read cannot say; it was built only to catch a gain the size
of the interval.
