# RAR-P45 — macOS on an M4: a 9,000-game Colosseum GUI round robin at `3+0.03`, ten engines, eight native arm64 and two x86-64 under Rosetta 2 — OBSERVED 2026-10-24: no forfeit; with a native Stockfish 11 the match host's gaps reproduce

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** played 2026-10-24 09:10–14:38 UTC by the maintainer in
  the Colosseum GUI (v1.1.0), tournament "Rating Tournament 2"
  (`9eda99b8-5491-46b3-81e1-f2cc58eea136`), finished, 9,000 of 9,000 games.
  Recorded by the agent from the GUI's database
  (`~/Library/Application Support/Colosseum/colosseum.sqlite`), read from a
  read-only SQLite backup. The tournament's per-engine snapshot gives each
  binary's path.
- **Host:** the MacBook Air (Apple M4, 10 cores, 24 GB, macOS 27.0.1) of
  RAR-P42 and RAR-P43, a compatibility-test host, not the match host.
- **Kind:** an observation, not a measurement anyone gates on. Nothing was
  registered. No bench was taken.

## Native and emulated engines

Read with `file` and `lipo -archs`. An x86-64 binary runs under Rosetta 2.
SHA-256 prefixes are listed here; the full hashes and modification times
are in `engines.out`. Every native file was last modified before the run
started, the latest at 08:40 UTC.

| Engine | Binary | Code | SHA-256 |
|---|---|---|---|
| Stockfish 11 | `stockfish/stockfish-11-macos-arm64-native-pgo` | **arm64, native** | `b81f133b…` |
| Houdini 6.03 | `houdini-6.03-macos-arm64-native-pgo` | **arm64, native** | `97b1777b…` |
| Fire 5192023 | `fire-5192023-hce-macos-arm64-native-pgo` | **arm64, native** | `442993b1…` |
| Rarog 2.6.0 | `rarog/rarog-v2.6.0-macos-arm64-native-pgo` | **arm64, native** | `fd0c95dd…` |
| Ethereal 13.00 | `ethereal-13.00-hce-macos-arm64-native-pgo` | **arm64, native** | `7800cdfc…` |
| Basilisk 1.10.2 | `basilisk/basilisk-v1.10.2-macos-aarch64-pgo` | **arm64, native** | `a7d6a5ba…` |
| Manta 1.2.1 | `manta/manta-v1.2.1-macos-arm64-native` | **arm64, native** | `7748003d…` |
| Fruit 2.1 | `fruit-2.1-macos-arm64-native-pgo` | **arm64, native** | `1dc8f342…` |
| Komodo 14.1 | `komodo-14.1-64-osx` | x86-64, Rosetta 2 | `20948bc9…` |
| Critter 1.6a | `critter-16a` (universal i386 + x86-64) | x86-64, Rosetta 2 | `89661872…` |

Paths are under `~/Chess/Engines/`. Rarog 2.6.0 is RAR-P42's binary. The
Stockfish 11, Houdini 6.03, Fire, Ethereal and Fruit files are arm64
builds made on 2026-10-24, not official releases, and the database does
not record where they came from. Fire and Ethereal are `hce` builds by
their file names. In RAR-P43 Stockfish 11 was x86-64 under Rosetta 2;
here it is native.

## The run

- Ten engines, round robin ×100, two games a pair (colours reversed on one
  UHO_Lichess_4852_v1 position, random order, seed 997): 9,000 games, 200 a
  pair, 1,800 an engine. `3+0.03`, one thread, Hash 128, eight games at a
  time. Tablebases off, ponder off, no adjudication.
- **Every game ended naturally:** 7,431 checkmates, 1,024 threefolds, 326
  insufficient material, 192 fifty-move, 27 stalemates. No forfeit, crash
  or illegal move for any engine, and no clock went below zero by the GUI's
  move times.
- **Ratings:** the GUI held Rarog 2.6.0 (3379), Basilisk 1.10.2 (3019) and
  Manta 1.2.1 (2679) at their entered ratings and fitted the other seven.
  A free Bradley–Terry fit with a per-position-pair bootstrap of 200
  resamples (`fit.py`) reproduces the GUI's gaps from Stockfish 11 down to
  Ethereal to within 2 and to Critter within 7. Below that the GUI's
  column is compressed by its held ratings: Rarog sits 9 higher there, and
  Basilisk, Manta and Fruit 21 to 34 lower. Each engine minus Rarog 2.6.0
  on the free fit, with its 95% interval:

  | Engine | Code | − Rarog 2.6.0 | Score | Speed ÷ Rarog |
  |---|---|---|---|---|
  | Stockfish 11 | native | **+108.7 ± 19.7** | 79.1% | 1.14 |
  | Houdini 6.03 | native | **+30.5 ± 17.7** | 71.1% | 1.19 |
  | Fire 5192023 | native | +8.1 ± 16.2 | 68.6% | 1.25 |
  | Rarog 2.6.0 | native | 0 | 67.8% | 1.00 (1.95M) |
  | Komodo 14.1 | Rosetta 2 | −18.7 ± 17.9 | 65.7% | 0.81 |
  | Ethereal 13.00 | native | −34.1 ± 17.6 | 64.0% | 1.24 |
  | Critter 1.6a | Rosetta 2 | **−301.4 ± 22.9** | 37.1% | 0.89 |
  | Basilisk 1.10.2 | native | −390.2 ± 25.5 | 30.0% | 1.21 |
  | Manta 1.2.1 | native | −743.4 ± 35.6 | 8.9% | 0.56 |
  | Fruit 2.1 | native | −765.3 ± 37.8 | 7.8% | 1.59 |

  Speed is total nodes over total move time from the move comments; Rarog
  2.6.0 ran at 1.95M.

## What it shows

- **The Rosetta artefact on Stockfish 11 is gone with a native build.**
  Head to head, Stockfish 11 beat 2.6.0 by **+101.8 ± 31.7** (93-71-36,
  pentanomial). Under Rosetta 2 in RAR-P43 it led by only +16.5 ± 25.1.
- **With native opponents, the Mac reproduces the match host's gaps.**
  Relative to Rarog, on the fit, against RAR-M75's standings (the C head
  2.6.0 was cut from) and RAR-P44's native Windows-on-ARM builds:
  - Stockfish 11: +108.7 here, +89.3 on the match host, +91.9 on the
    Surface.
  - Houdini 6.03: +30.5 here, +19.6, +24.4.

  Both differences lie within the combined intervals. Speed relative to
  Rarog also matches the match host for Stockfish 11 (1.14 here, 1.12
  there); Houdini 6.03 keeps a little less (1.19, 1.27).
- **Critter under Rosetta 2 shows a speed cost and no resolved Elo cost.**
  It is the same binary family as RAR-M75's but a macOS build. It runs at
  0.89 of Rarog's speed here against 1.43 on the match host, 0.62 of it.
  Its rating moved only −14.5 (−301.4 here, −286.9 there; RAR-P44 −262.3
  under Windows' emulator). As on the Surface, one engine cannot separate
  an emulation cost from noise. Komodo 14.1 has no native reference.
- **Low clocks rise with the field, not the binary.** The share of each
  engine's games whose clock went below 100 ms after a move: 2.6.0 35.8%
  (17.7% in RAR-P43, same binary), Basilisk 1.10.2 42.4% (24.5%), Ethereal
  61.4%, Stockfish 11 33.8% (0% under Rosetta). Rarog 2.6.0's lowest clock
  was 42 ms. The d1 clock cluster (D.1.3) targets this floor; this run used
  the release.
- **RAR-R13's first trigger did not fire for 2.6.0.** 32 of its games were
  not won after it had held +300 or more for six of its moves (31 drawn,
  one lost). None showed a mate for 2.6.0 or a won-position move at depth
  40 or more.
- **Not usable for E.2 or the checkpoint.** The host is not the match host,
  2.6.0 is the release and not the d1 head, the opponents are rebuilt or
  emulated, and the GUI's held ratings are not anchored to the match pool.

**Artifacts:** `analysis/artifacts/rating-tour2-mba-2026-10-24/` (ignored):
- `rating_tournament_2.pgn`, the 9,000 games exported from the database
  (45,425,704 bytes, `46e62c3e…db82`) by `export.py` (`f35d75ee…fd7a`).
- `tournament_config.json` (`2a1f1317…86a9`).
- `engines.out`, each binary's architecture, SHA-256 and modification
  time (`6f2d8c65…bd69`).
- RAR-P43's `rating.py` (`075544f2…f3e2e`, pairs, pentanomial Elo, clocks,
  NPS and won-position outcomes) and its output `rating.out`
  (`02a02fb8…5c48`).
- `fit.py` (the fit, bootstrap and speed, `b309ecbc…0752`) and `fit.out`
  (`e75acab4…fd27`).
- RAR-P43's `r13.py` (`6c829240…e23c5906`) and `r13.out`
  (`221c3600…743c`).

**Retry trigger or `closed`:** closed as an observation. RAR-R13's triggers
are unchanged.
