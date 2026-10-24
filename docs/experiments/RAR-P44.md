# RAR-P44 — Windows on ARM: the "Rating - Top" Colosseum GUI round robin at 3+0.03, twenty engines, nine native and eleven emulated — OBSERVED 2026-10-23/24: no Rarog fault in 7,014 games; the native pairs reproduce the match host's gaps

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** played 2026-10-23 23:03 to 2026-10-24 07:39 UTC by the
  maintainer in the Colosseum GUI (v1.1.0), tournament "Rating - Top"
  (`86f6eb3d-4219-450a-be45-7ec5ed7a90a6`). Stopped at **24,683 of 38,000
  games**, no game in flight. Recorded by the agent from the GUI's database
  (`%APPDATA%\Colosseum\data\colosseum.sqlite`), read through a read-only
  SQLite backup while the GUI was running. The tournament's per-engine
  snapshot gives each binary's path.
- **Host:** Surface Laptop 7th Edition, Snapdragon X Plus, Windows 11 on ARM,
  the compatibility-test host of RAR-P41. It is not the match host. Its
  bottom half, "Rating - Bottom", was still running when this was recorded
  and is not part of this entry.
- **Kind:** an observation, not a measurement anyone gates on. Nothing was
  registered. No bench was taken, because the bottom-half run was using the
  host.

## Native and emulated engines

Read from each binary's PE header (`IMAGE_FILE_HEADER.Machine`; `eng.py`).
An x86-64 binary runs under Windows' Prism emulator. SHA-256 prefixes are
listed here; the full hashes are in `engines.out`. Every native file's
modification time is earlier than the start of the run, and none changed
after it.

| Engine | Binary | Code | SHA-256 |
|---|---|---|---|
| Stockfish 11 | `stockfish\stockfish_11-arm64-oryon-pgo.exe` | **arm64, native** | `e260f82e…` |
| Fire 5192023 | `Fire_5192023-arm64-oryon-pgo.exe` | **arm64, native** | `6f538ee1…` |
| Ethereal 12.75 | `Ethereal12.75-arm64-oryon-pgo.exe` | **arm64, native** | `69976e8b…` |
| Houdini 6.03 | `houdini\Houdini_6.03_Pro_arm64-pgo.exe` | **arm64, native** | `3c434109…` |
| Rarog 2.6.0 | `rarog\rarog-v2.6.0-windows-arm64-native-pgo.exe` | **arm64, native** | `8af64659…` |
| Rarog 2.5.0 | `rarog\rarog-v2.5.0-windows-arm64-native-pgo.exe` | **arm64, native** | `e304cc9b…` |
| Rarog 2.4.0 | `rarog\rarog-v2.4.0-windows-arm64-native-pgo.exe` | **arm64, native** | `295d2513…` |
| Basilisk 1.10.2 | `basilisk\basilisk-v1.10.2-windows-aarch64-pgo.exe` | **arm64, native** | `affb61d0…` |
| Basilisk 1.9.3 | `basilisk\basilisk-v1.9.3-windows-aarch64-pgo.exe` | **arm64, native** | `b37d318d…` |
| Komodo 14.1 | `komodo\komodo-14.1-64bit.exe` | x86-64, emulated | `9320b52f…` |
| Houdini 5.01 | `houdini\Houdini_5_Pro_x64.exe` | x86-64, emulated | `7b7d3bc2…` |
| Houdini 4 | `houdini\Houdini_4_Pro_x64B-v1x.exe` | x86-64, emulated | `a7363131…` |
| Houdini 3 | `houdini\Houdini_3_Pro_x64.exe` | x86-64, emulated | `b0e7a631…` |
| HIARCS 15.4 | `hiarcs\DeepHiarcs15.4-engine.exe` | x86-64, emulated | `58c53437…` |
| Fritz 16 | `rybka\Fritz 16.exe` | x86-64, emulated | `264f2eed…` |
| Shredder 13 | `EngineDeepShredder13UCIx64.exe` | x86-64, emulated | `1cf43864…` |
| Fritz 15 | `rybka\Fritz 15.exe` | x86-64, emulated | `a223ec3d…` |
| Critter 1.6a | `Critter_1.6a_64bit.exe` | x86-64, emulated | `a8988ac9…` |
| Houdini 1.5a | `houdini\Houdini_1.5a_x64.exe` | x86-64, emulated | `b4e4282b…` |
| Rybka 4.1 | `rybka\Deep Rybka 4.1 SSE42 x64.exe` | x86-64, emulated | `15901a12…` |

Paths are under `C:\Files\chess\engines\`. The Stockfish 11, Fire, Ethereal
and Houdini 6.03 files are arm64 builds, not the official x86 releases, and
the database does not record where they came from. RAR-P41's bullet run used
Houdini 5.01's `pext` build and Komodo 14.1's `bmi2` build, according to its
incident logs; this run used the plain x86-64 builds of both. Houdini 3,
Houdini 4 and Critter are the same files as RAR-M75's on the match host,
with the same hashes.

## The run

- Twenty engines, round robin ×100, two games a pair. One thread and Hash
  128 for every engine, `3+0.03`, eight games at a time. The UHO Lichess
  4852 book at random (seed 3455). Tablebases off, ponder off, no
  adjudication, a 2,000 ms forfeit margin. Each engine's move overhead was
  set to 10 ms or its buffer to 0 (`tournament_config.json`). About 2,468
  games an engine.
- **Terminations:** 19,142 checkmate, 3,256 threefold, 1,313 insufficient
  material, 865 fifty-move, 105 stalemate, **2 time forfeits**. Both time
  forfeits were Houdini's, and neither was a slow move: the engine never
  answered `go`, with 0.44 s (Houdini 5.01, emulated, round 418) and 1.55 s
  (Houdini 6.03, native, round 1222) left on its clock.
- **Ratings:** the GUI keeps Rarog 2.4.0 fixed at 3001 and fits every other
  engine. A Bradley–Terry fit with the same anchor, and a per-pair bootstrap
  with 200 resamples (`fit.py`), reproduces the GUI's column to within 4.
  Each engine minus Rarog 2.6.0, with its 95% interval:

  | Engine | Code | − Rarog 2.6.0 | Score | Mean NPS |
  |---|---|---|---|---|
  | Stockfish 11 | native | **+91.9 ± 19.0** | 81.9% | 2.01M |
  | Fire 5192023 | native | +29.4 ± 18.3 | 76.2% | 2.03M |
  | Houdini 6.03 | native | **+24.4 ± 19.8** | 75.7% | 1.95M |
  | Rarog 2.6.0 | native | 3,386.3 | 73.3% | 2.04M |
  | Komodo 14.1 | emulated | −40.0 ± 17.7 | 69.0% | 1.35M |
  | Ethereal 12.75 | native | −40.5 ± 17.0 | 68.9% | 2.78M |
  | Houdini 5.01 | emulated | −94.3 ± 18.0 | 62.7% | 1.74M |
  | Rarog 2.5.0 | native | **−137.0 ± 18.5** | 57.6% | 1.80M |
  | Houdini 4 | emulated | −188.7 ± 19.4 | 51.1% | 2.09M |
  | Houdini 3 | emulated | −230.5 ± 18.3 | 45.8% | 2.06M |
  | Shredder 13 | emulated | −243.2 ± 18.3 | 44.3% | 1.58M |
  | HIARCS 15.4 | emulated | −252.9 ± 19.0 | 43.1% | 1.47M |
  | Critter 1.6a | emulated | −262.3 ± 17.6 | 41.9% | 1.87M |
  | Houdini 1.5a | emulated | −290.3 ± 18.9 | 38.5% | 2.12M |
  | Fritz 16 | emulated | −323.4 ± 19.3 | 34.5% | 0.58M |
  | Fritz 15 | emulated | −347.3 ± 18.6 | 31.7% | 0.53M |
  | Rarog 2.4.0 | native | **−385.3 ± 18.9** | 27.5% | 2.40M |
  | Basilisk 1.10.2 | native | −394.9 ± 20.5 | 26.4% | 2.50M |
  | Rybka 4.1 | emulated | −401.8 ± 20.4 | 25.7% | 0.09M |
  | Basilisk 1.9.3 | native | −416.0 ± 21.1 | 24.3% | 2.45M |

  The NPS figures are the engines' own reports, averaged over games. Rybka
  and Fritz report reduced node counts, so their NPS cannot be compared
  with the others'.

## What it shows

- **Robustness on ARM at `3+0.03`:** none of the three Rarog releases
  forfeited, crashed or made an illegal move in 7,014 games. That includes
  2.6.0's old clock.
- **With native opponents, the ARM host reproduces the match host's gaps.**
  RAR-M75 measured them at the same control, thread count, hash, book
  family and margin, with the C head that 2.6.0 was cut from:
  - Stockfish 11: +91.9 here, +89.3 there.
  - Houdini 6.03: +24.4 here, +19.6 there.

  Both opponents run a different binary on each host. RAR-P41 had 2.6.0
  level with an emulated Stockfish 11; with a native build, that artefact
  is gone.
  On the MacBook Air, Stockfish 11 under Rosetta 2 led 2.6.0 by only
  +16.5 ± 25.1 (RAR-P43): the same artefact on macOS.
- **The emulated engines show no uniform handicap.** Against RAR-M75's
  standings, again relative to Rarog:
  - Houdini 4: −188.7 here, −175.7 there.
  - Houdini 3: −230.5 here, −191.9 there.
  - Critter 1.6a: −262.3 here, −286.9 there.

  These are the same binaries, and the shifts run from −39 to +25 against a
  combined interval of about ±25. Three engines cannot separate an
  emulation cost from noise. RAR-P41's large losses (Houdini 5.01 −618,
  Komodo 14.1 −229) came from its `pext`/`bmi2` builds. The plain x86-64
  builds here lost far less against their entered ratings: Houdini 5.01 −44
  and Komodo 14.1 −22 on the GUI's Δ column. Where those entered ratings came
  from is not recorded, so the Δ column is a hint only.
- **Emulation does cost speed; the Elo cost is small and not resolved.**
  Total nodes over total time from the move comments (`speed.py`), each
  engine divided by Rarog 2.6.0's 1.81M, against the same ratio on the
  match host (RAR-M75, the C head's 2.43M):
  - Houdini 3: 1.14 here, 1.82 there (0.63 of it).
  - Houdini 4: 1.16 here, 1.70 there (0.68).
  - Critter 1.6a: 1.09 here, 1.43 there (0.76).
  - The native rebuilds keep more: Stockfish 11 0.91, Houdini 6.03 0.90.

  The emulated engines run about a quarter to a third slower relative to
  Rarog, yet their three Elo shifts average −9 ± about 15. The shifts allow
  anything from no cost to a few tens of Elo. They do not show that
  emulation is free.
- **The host held a steady speed.** Rarog 2.6.0's speed by tenths of the
  run stays between 1.76M and 1.85M, with no downward trend. Under the
  bottom half's identical load on 2026-10-24, sampled for 10 s, all ten
  cores were busy at 2.19–2.52 GHz of the X1P-64-100's 3.40 GHz, on mains
  power with the Balanced plan.
- **Rarog against Rarog,** over the whole pool:
  - 2.6.0 over 2.5.0: +137.0 ± 18.5, against RAR-M76's +153.4 ± 10.7.
  - 2.5.0 over 2.4.0: +248.2 ± 19.3, against RAR-M64's +272.4.

  Both are lower than on the match host. The first lies within the combined
  interval.
- **Not usable for E.2 or the checkpoint.** The host is not the match host,
  2.6.0 is the release and not the d1 head, several opponents are emulated
  or rebuilt, and the run was stopped two thirds of the way through.

**Artifacts:** `analysis/artifacts/rating-top-wos-2026-10-24/` (ignored):
- `rating_top.pgn`, the 24,683 finished games exported from the database
  (`cd0dee97…4ed805`).
- `tournament_config.json`, the configuration and the per-engine snapshot
  (`e6a77eab…d5166a`).
- `eng.py` (PE machine, size and hash of each binary, `9f3a738c…7fb89f`)
  and its output `engines.out` (`afdf94cd…213ca3`).
- `stand.py` (standings, NPS, depth and Rarog 2.6.0's pairs,
  `7e7162ca…462a8d`) and `stand.out` (`72838b7e…a5cd7f`).
- `fit.py` (the anchored fit and bootstrap, `52dcd26d…ed8d3b`) and `fit.out`
  (`2a4a469f…7fa89b`).
- `speed.py` (speed from the move comments, `4cee81d2…42bd4b`) and `speed.out`
  (`6d2aa6f8…7e7f97`).
- `tf.py` (the time forfeits, `edcac2cf…44a11c`) and `tf.out`
  (`e323d43b…3ed6b5`).
- `export.py` (`c45eeb8e…be8620`).
- `incidents/`, Colosseum's two forfeit reports (`3a15658a…5ff2e4`,
  `3088e95a…325e1d`).

**Retry trigger or `closed`:** closed as an observation. This entry records
emulation per engine, which RAR-P41's trigger asked of a rated ARM pool; a
pool that rates Rarog on ARM would still need native builds of the engines
around it.
