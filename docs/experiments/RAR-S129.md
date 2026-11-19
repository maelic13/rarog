# RAR-S129 — D.7 audit: the compact kernel against its donor's hybrid at equal nodes, and the evaluation wrapper's share of RAR-S128's interim gain — REGISTERED 2026-11-19, before any game

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-19, before any game. Both reads are
  run by the agent inside the D.7 audit the maintainer asked for
  2026-11-19 ("measure anything extra that you need"), with RAR-S128's tune
  paused at iteration 2,000.
- Disposition: open — registered 2026-11-19; both reads still to play.
- **Kind:** two diagnostic fixed matches. They accept nothing and gate
  nothing; D.7.7's gate is unchanged.
- **Inherits:** RAR-O09 (1) (the hybrid +3.47 ± 8.37 Elo over the head at
  equal work), RAR-S126 (the unfitted compact kernel −6.9 ± 9.5 Elo against
  the classic at `3+0.03`), D.7.4's parity (layer A without a mismatch,
  layer B inside bands of ±0.12 to ±0.14 in log tree size) and RAR-S128's
  interim read (centres at 2,000 against the seeds: 702 W / 920 D / 378 L).

## Why

1. **Parity in strength was never read.** Layer A compares decisions on the
   same inputs and layer B tree sizes inside a band of about ±13%. Neither
   sees the state that feeds the rules (the table, the histories, the root),
   and neither is Elo. RAR-S126's arithmetic leaves room: the hybrid was
   level with the head at equal work (+3.5), the compact kernel searched
   4.4% more nodes a move than the classic in games (about +9 at PROCESS's
   2 Elo per 1%), and it read −6.9: about 19 below that sum, three standard
   errors of the two reads' own noise, before the conversion's uncertainty.
   A read at equal nodes separates a port or state defect (the shortfall
   stays) from the clock side (it goes).
2. **What the fit is fitting.** At 2,000 the two wrapper coordinates
   travelled furthest (`KWrapRawBase` 21,032 to 16,385, `KWrapDivisor`
   27,015 to 32,596: the evaluation the kernel sees shrinks by 29% with
   every piece on and by 35% with none), half of it inside 250 iterations.
   Whether the interim gain is mostly that one scale says whether the
   fold's scale (mean |network| over mean |HCE|, 2536/1024) is the kernel's
   main misfit, and how much the other 228 coordinates have yielded.

## Use of the readings (frozen)

- **(P) parity.** Interval wholly below zero: the port is weaker than its
  donor's form in a way the parity instrument does not see; the audit
  reports it as a defect to diagnose (table, histories, root) before more
  is invested in a fit, and RAR-S128's continuation is weighed against it.
  Anything else: the port is taken as its donor's equal in strength to the
  interval's width, the audit says so, and RAR-S126's shortfall is then
  read as the clock side's (Rarog's time management and root driver
  driving the donor's kernel), a lead for D.6.28 and the next fit's
  surface, not a port defect.
- **(W) the wrapper's share.** Read against the interim read's gain: the
  share of it the two wrapper coordinates carry alone. It feeds the audit's
  recommendation on the fold's scale and on continuing RAR-S128; it stops
  or changes nothing in the tune by itself.

## Arms and runs (frozen)

Colosseum `match` through `tools/colosseum.ps1`, 2,000 games each, Hash 64,
one thread, the UHO book in random order, no adjudication, seed 20261120.
Dry runs passed every preflight 2026-11-19
(`colosseum_match_rar-s129-parity_20261119_135235.dry-run.json`,
`colosseum_match_rar-s129-wrap_20261119_135253.dry-run.json`).

- **(P)** `d75cand` against `reckless-hce`, **100,000 nodes a move each**
  (both count a move made, the null move not counted, since D.6.31), no
  clock. `d75cand` is RAR-S126's candidate, `rarog-d75cand-pext-pgo.exe`
  (`2820eaa`, `compact-kernel`, bench 7,543,485, SHA-256 `a8ead317…`): the
  port as built, before the line bound, which binds no bench tree and which
  the donor lacks. `reckless-hce` is RAR-O09's arm, a foreign engine
  (`analysis/artifacts/rar-o09-2026-11-14/reckless-hce.exe`, Reckless
  `0756957` with the hybrid patch on Rarog's frozen evaluation, SHA-256
  `8935a1b8…43296`). Directory `tools/results/rar-s129-parity`.

  ```powershell
  ./tools/colosseum.ps1 -Mode match -EngineA tools/test_engines/rarog-d75cand-pext-pgo.exe -EngineB analysis/artifacts/rar-o09-2026-11-14/reckless-hce.exe -NameA d75cand -NameB reckless-hce -ForeignB -Nodes 100000 -Games 2000 -Seed 20261120 -ExpectBench 7543485 -Dir tools/results/rar-s129-parity
  ```

- **(W)** the tune binary on both sides (`rarog-d76fit-tune.exe`,
  `f7c89fb`, SHA-256 `a536482f…`, `-CategoricalTuneBuild`), `3+0.03`:
  `d76wrap2000` sets only `KWrapRawBase=16385` and `KWrapDivisor=32596`,
  the interim read's values; `d76seed` keeps every seed. Directory
  `tools/results/rar-s129-wrap`.

  ```powershell
  ./tools/colosseum.ps1 -Mode match -CategoricalTuneBuild -EngineA tools/test_engines/rarog-d76fit-tune.exe -EngineB tools/test_engines/rarog-d76fit-tune.exe -NameA d76wrap2000 -NameB d76seed -OptionsA KWrapRawBase=16385,KWrapDivisor=32596 -ExpectBench 7543485,7543485 -Dir tools/results/rar-s129-wrap -Games 2000 -Seed 20261120
  ```

**Validity:** every game a rules result; a run with time losses or engine
faults in more than 0.5% of its games is void. (P): no move of either side
reports more than its budget plus two nodes (RAR-O09's amended tolerance).

**Reading:** each manifest's Elo and nElo with their 95% intervals.

## Zero-game companion, read before the games

`bench 13` on the tune binary (SHA-256 `a536482f…`), one UCI session each
(`activation.py`'s runner): the seeds **7,543,485 / EBF 2.422**; the centres
at 2,000 **3,488,485 / 2.336**; the two wrapper values alone **5,678,657 /
2.398**; the other 228 alone **4,171,048 / 2.344**. A fingerprint, not a
tree size and not Elo: the fit has roughly halved the tree at a depth, and
both parts move it.

## PRE-REGISTERED PREDICTION (frozen 2026-11-19, before any game)

- **(P):** `d75cand` **−5 Elo**, 80% band **−20 to +10**; the interval
  wholly below zero with probability 0.2. Basis: every rule reproduces the
  donor's decision and tree sizes agree, so near zero; taken down a little
  for RAR-S126's shortfall against the sum of RAR-O09 (1) and the speed
  read, and for the kept departures (Rarog's table and root, SEE-2 and
  SEE-3, position 305's second divergence).
- **(W):** `d76wrap2000` **+30 Elo**, 80% band **+10 to +50**, a little
  over half the interim read's gain. Basis: the wrapper pair travelled
  furthest and fastest, and the first hundred iterations' score spread was
  twice the game noise, which a few steep coordinates explain; against it,
  the other 228 move the bench as much as the pair does.
- **Most likely failure mode:** (W) reads near the whole interim gain, and
  the fit of the other 228 coordinates is then unproven at 2,000.
- **Evidence layer:** games. (P) at fixed nodes says nothing of speed or
  clock; (W) is on a tune build without PGO, both sides alike.
