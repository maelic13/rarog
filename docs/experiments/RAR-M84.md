# RAR-M84 — The "Dev Test" Colosseum GUI round robin at `3+0.03`: the development head and Rarog 2.6.0 against a shared field — OBSERVED 2026-11-08/09: level head to head, the head 13.6 ± 9.9 Elo lower against the field; not a gate

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** played 2026-11-08 21:00 to 2026-11-09 06:44 UTC by the
  maintainer in the Colosseum GUI (v1.1.0), tournament "Dev Test". Recorded by
  the agent from the GUI's own database (`colosseum.sqlite`, tournament
  `73a47714-e34c-4604-8ad1-1828baaaeb05`), read-only, and the GUI's standings.
- **Host:** the match host. The run has no idle record: the GUI has no busy
  guard, and nobody checked the host during it.
- **Binaries:** `id name` and SHA-256:
  - Rarog `dev-20261108-32199adf` `4a553d4f…b1ec55`, a PGO pext build by the
    xtask (a build without PGO carries `-nopgo` in its name). Its bench reads
    **15,625,852 / EBF 2.563**, the head's fingerprint; `32199ad` differs from
    `b202174` in documents only.
  - Rarog 2.6.0 `274eab97…1ae1`, 2.5.0 `1f584523…4e2e` and 2.4.0
    `fdecf86b…94c5`, the published pext assets.
  - Basilisk `dev-20261108-9c9c2335` `05da6320…a3d6` and 1.10.2 `fa874a89…9459`.
  - Ethereal 13.07 `61f0270f…abe3`, MoveOverhead 10. Houdini 4 `a7363131…81a2`,
    HIARCS 15.4 `58c53437…1c95`, Stockfish 5 `c5f7b412…9b21`, Shredder 13
    `1cf43864…b630`, Critter 1.6a `a8988ac9…63d2`.
- **Kind:** an observation, not a measurement anyone gates on. Nothing was
  registered and no prediction was frozen, so it accepts and rejects nothing.
- Disposition: observation — played 2026-11-08/09, unregistered: head to head
  level (−4.8 ± 14.1), the head 13.6 ± 9.9 below 2.6.0 against the shared field;
  it accepts and rejects nothing.

## The run

- Twelve engines, round robin ×400 with two games a pair (colours reversed on
  one UHO_Lichess_4852_v1 position, random order, seed 15533351315): **52,800
  games**, 800 a pair, 8,800 an engine. `3+0.03`, one thread, Hash 128, fourteen
  games at a time, tablebases off, ponder off, no adjudication.
- Ratings: only the two development builds were written back; the others show
  their held ratings.
- **Every game ended naturally:** 39,672 mates, 8,676 threefolds, 2,689
  insufficient material, 1,524 fifty-move, 239 stalemates. The GUI shows no
  forfeit for any engine. It forfeits only past a 2,000 ms margin, so the
  clocks below are the stronger check: no game of the three Rarog builds read
  took a clock below zero by the GUI's move times.

## Standings

| Engine | W-D-L | Score | Perf | Held or written Elo |
|---|---|---|---|---|
| Rarog 2.6.0 | 5251-2276-1273 | 72.6% | 3421 | 3410 |
| Rarog `dev-20261108-32199adf` | 5073-2369-1358 | 71.1% | 3408 | 3408 (−12) |
| Ethereal 13.07 | 4807-2174-1819 | 67.0% | 3377 | 3356 |
| Basilisk `dev-20261108-9c9c2335` | 4333-2300-2167 | 62.3% | 3339 | 3339 (+93) |
| Houdini 4 | 3698-2022-3080 | 53.5% | 3271 | 3310 |
| Rarog 2.5.0 | 3325-2503-2972 | 52.0% | 3262 | 3281 |
| HIARCS 15.4 | 3031-2205-3564 | 47.0% | 3227 | 3236 |
| Stockfish 5 | 2850-2358-3592 | 45.8% | 3219 | 3221 |
| Shredder 13 | 2696-2392-3712 | 44.2% | 3209 | 3201 |
| Critter 1.6a | 2608-2183-4009 | 42.0% | 3192 | 3192 |
| Basilisk 1.10.2 | 1061-1666-6073 | 21.5% | 3025 | 2995 |
| Rarog 2.4.0 | 939-1808-6053 | 20.9% | 3018 | 3001 |

W-D-L is from the database and matches the GUI. Perf and Elo are the GUI's
(`standings.webp`).

## What it shows

Head-to-head Elo is pentanomial over the 400 position pairs, with a 95%
interval.

- **Head to head the two Rarog builds are level:** the head against 2.6.0
  **−4.8 ± 14.1** (209-371-220).
- **Against the ten shared opponents the head is lower:** 2.6.0 +188.9 ± 7.0,
  the head +175.3 ± 6.9, a gap of **−13.6 ± 9.9**, taking the two reads as
  independent. Each row is the head minus 2.6.0, point estimates:

  | Opponent | 2.6.0 | Head | Gap |
  |---|---|---|---|
  | Rarog 2.4.0 | +410.0 | +358.8 | −51.2 |
  | HIARCS 15.4 | +193.8 | +150.8 | −42.9 |
  | Basilisk 1.10.2 | +403.2 | +368.8 | −34.4 |
  | Stockfish 5 | +211.1 | +181.7 | −29.4 |
  | Rarog 2.5.0 | +163.0 | +143.6 | −19.4 |
  | Houdini 4 | +163.5 | +150.3 | −13.2 |
  | Shredder 13 | +196.7 | +189.7 | −7.0 |
  | Ethereal 13.07 | +50.7 | +45.4 | −5.3 |
  | Basilisk dev | +73.2 | +79.5 | +6.4 |
  | Critter 1.6a | +200.2 | +218.0 | +17.7 |

  Each opponent row carries about ±18 to ±33 on each side, so no single row
  means much. The gap as a whole barely clears zero and is one read. RAR-R28,
  D.1's closing direct match, read the clock work +1.2 ± 8.9 against 2.6.0,
  and this run's direct read agrees with it. What is new is only the field's
  reading. Self-play can hide a gap a field shows, or this can be noise; one
  unregistered run does not separate them.
- **The depth column is not a search change.** It reads 11.79 for the head
  against 15.89 for 2.6.0, but the head now stops on a proved mate. Split by
  score, from the PGN comments with the mover taken from the start position
  (per-game means reproduce the column in all 8,800 games of each build):

  | Moves | Build | Share | Depth | Nodes a move | ms a move | In-game nps |
  |---|---|---|---|---|---|---|
  | Non-mate | 2.6.0 | 91.32% | 12.62 | 143,492 | 76.1 | 1,884,450 |
  | Non-mate | Head | 91.00% | 12.26 | 133,182 | 72.9 | 1,828,162 |
  | Mate | 2.6.0 | 8.68% | 45.42 | 57,780 | 21.2 | 2,720,906 |
  | Mate | Head | 9.00% | 7.61 | 11,159 | 4.1 | 2,689,581 |

- **On non-mate moves the head searches 7.2% fewer nodes:** 4.2% less time and
  3.0% lower in-game nps. The time matches RAR-R28's 3–5%. The GUI's nps column
  (1.91M against 2.08M, −8.1%) overstates the speed gap: it averages per game,
  over moves of different kinds. In-game nps is not an NPS read by PROCESS's
  method, and the two builds' game populations differ, so the −3.0% is not a
  measured speed loss. Nodes are not Elo, and this does not attribute the gap.
- **The clock floor D.1 targets holds in a field:**

  | Build | Clock used, mean | Lowest clock, median | Games under 100 ms |
  |---|---|---|---|
  | 2.6.0 | 0.897 | 116 ms | 46.6% |
  | Head | 0.838 | 376 ms | 5.8% |
  | 2.5.0 | 0.916 | 67 ms | 61.7% |

  Clock used is the time spent over `3000 + 30 × moves`. The lowest clocks seen
  were 31 ms for 2.6.0, 83 ms for the head and 30 ms for 2.5.0.
- **The release gap reproduces:** 2.6.0 over 2.5.0 **+163.0 ± 18.2**
  (447-256-97). RAR-M76 measured +153.4 ± 10.7.
- **Basilisk's development build** is 2.6.0's −73.2 ± 18.3, the head's
  −79.5 ± 18.7, and +318.1 ± 24.5 over its own 1.10.2.

**Artifacts:** `analysis/artifacts/dev-test-2026-11-08/` (ignored):
`dev_test.pgn`, the 52,800 games exported from the database (289,981,798 bytes,
`ecbf2971…e005`); `tournament_config.json` (`dab605ee…79d7`); `rating.py` (pairs,
pentanomial Elo, the mate split, clocks, binary hashes, `b6a47c7e…2a04`) and its
output `rating.out` (`b33e41e1…7e47`); `dev_bench.out`, the head's bench
(`86463ba5…5c19`); `standings.webp`, the GUI's standings (`3bc007ea…4d81`).

**Retry trigger or `closed`:** closed as an observation; nothing changes on it.
If a later field read of a Rarog head shows the same gap against 2.6.0, the
gap becomes a research question for its owner leaf.
