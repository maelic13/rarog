# RAR-M80 — One game at a time against three: does it cost the four-thread side, Rarog's and Reckless's alike, on one host state — REGISTERED 2026-11-02, before any game

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-02 by the agent; the maintainer runs
  it (about 80 minutes).
- **Leaf:** PLAN D.2, the four-thread card's H7.
- **Question.** In [RAR-M79](RAR-M79.md) Rarog's four-thread side, playing
  one game at a time, ran 23% slower than with three games at once on the
  same openings, while its one-thread side ran 6% faster. That was not a
  registered reading, and it came 50 minutes after the leg it is compared
  with. Does it replicate, and does Reckless lose the same way on the same
  host state? One game a machine is the condition nearest a rating list.
- **Kind:** an in-game speed measurement. The games are played only for
  their move comments: no Elo is read, and no gate.

## Design

- Legs (`analysis/artifacts/d2-solo/leg.ps1`, `cb474d5b…`), each engine
  against itself, one thread (arm A) against four (arm B), `10+0.1`, Hash
  256 on both sides, `match-fixed-ltc.toml` (the UHO book, random order, no
  adjudication), seed 81 in every leg, so all four play the same 30
  openings, every slot four physical cores (`--cores-per-game 4`),
  Colosseum CLI 0.5.0 (pinned), 60 games a leg:

  | Leg | Engine | Games at a time |
  |---|---|---:|
  | `rarog-x1` | the d2 head with `e057d84` (`rarog-d2dyn-pext-pgo.exe`, `a545b6b8…`), defaults (large pages on) | 1 |
  | `reckless-x3` | Reckless 0.9.0 (`D:/chess/engines/reckless-windows-avx2.exe`, `b74ead56…`), `MoveOverhead` 10 | 3 |
  | `reckless-x1` | Reckless 0.9.0, `MoveOverhead` 10 | 1 |
  | `rarog-x3` | the d2 head | 3 |

- `analysis/artifacts/d2-solo/run.ps1` (`7110c96d…`) checks both engines'
  hashes, dry-runs all four legs before the first game, then plays them in
  that order (each condition's legs apart in time, the engines alternating),
  each after the harness's idle wait, and reads. A finished leg is skipped on
  a rerun and a half-played one resumes at its seed.
- **Reading** (`solo_read.py`, `158af433…`): per leg and side, NPS = the sum
  of each move's `n` over the sum of its `t − h`, as RAR-M79; **R =
  NPS(4T) / NPS(1T)**; per engine **L = ln(R_x1 / R_x3)** over the openings
  both its legs played. 90% intervals resample openings, one draw for every
  leg. Also printed: each side's change alone, and mean depths.

## Judgement

- **NOT REPLICATED** if L_rarog > ln 0.92 (one game at a time costs Rarog's
  four threads under 8% against one).
- Otherwise **RAROG'S OWN** if L_rarog − L_reckless ≤ −0.10, and **SHARED**
  if not.
- **INVALID** if the two engines' legs did not play the same openings.

## Exposure before registration

- RAR-M79's `solo` and `on` legs read with this script (`anchor.log`): over
  their 20 common openings L −0.323 (90% −0.372 to −0.272); alone, the
  one-thread side +5.8% and the four-thread side −23.4%.
- RAR-M78's threads legs read the same way (`m78_reckless_r.py`,
  `m78_reckless_r.log`): Reckless R 3.46, Rarog R 2.48, three games at once,
  in different sessions.
- The judgement's self-test (`judge_selftest.py`, `judge_selftest.log`)
  reaches all three outcomes on RAR-M79's games.
- No dry run yet: the host is busy with a maintainer's Basilisk tune, and
  `colosseum.ps1` refuses a busy host even for a dry run. `run.ps1` makes
  every leg's dry run after its idle wait and before the first game.

## Prediction, frozen before any game

| Reading | Probability |
|---|---|
| NOT REPLICATED | 0.20 |
| RAROG'S OWN | 0.40 |
| SHARED | 0.40 |

- **Levels:** L_rarog −0.25 (80% interval −0.40 to −0.05); L_reckless
  −0.12 (−0.35 to +0.05); R_rarog_x3 3.5 (3.1 to 3.8), R_rarog_x1 2.7 (2.3
  to 3.3); R_reckless_x3 3.4 (3.0 to 3.7).
- **Reasoning.** RAR-M79's loss is large and tight over openings, so it
  replicates unless it was itself a passing host state. The one-thread side
  speeding up alone fits a processor boosting a quiet host; the four-thread
  side slowing fits four threads waking on a quiet host after the opponent's
  move, which would meet Reckless too. What Rarog does differently on that
  path: its large pages (RAR-M79 tested them off only at three games at
  once), its timer resolution request, and its helpers' wake from the pool.
  So own and shared are even.

## Decision and stop rule

- **NOT REPLICATED:** RAR-M79's solo leg was its moment's host state too.
  H7 closes as host variation that short legs cannot locate; Rarog's
  four-thread scaling is measured again (RAR-M78's threads leg) beside
  Reckless's on the same night, with this reading's in-game R as each leg's
  validity check.
- **RAROG'S OWN:** an engine property. The next read is zero-game: a game's
  searches replayed with the opponent's idle gap between them, each
  four-thread search's NPS over its first 50 ms against the rest, with
  large pages on and off.
- **SHARED:** a host property both engines meet; no engine change. Rarog's
  four-thread scaling is measured again beside Reckless's in both
  conditions, and H3 and H5 resume as the card's work.
- One read; nothing in the legs, games, seed or thresholds changes after the
  first game.

## How to run

From PowerShell 7 on an idle host, about 80 minutes:

```powershell
pwsh -File D:\code\rarog\analysis\artifacts\d2-solo\run.ps1
```

It waits for an idle host before each step, resumes after an interruption,
and ends with `ALL DONE`.
