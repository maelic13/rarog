# RAR-P59 — Windows on ARM: the hand-crafted frontier played again, a nine-engine Colosseum GUI round robin at `3+0.03`, seven native arm64 and two x86-64 under emulation — OBSERVED 2026-11-20, stopped at 9,495 of 14,400 games: 2.7.0 fourth of nine, 137.5 below Stockfish 16 HCE

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** played 2026-11-20 09:40 to 15:15 UTC by the maintainer
  in the Colosseum GUI (v1.2.0), tournament "HCE Frontier (ARM)"
  (`4f25bc8a-4d69-459c-8262-29a9f2510923`). Stopped at **9,495 of 14,400
  games**, none in flight. Recorded by the agent from a read-only backup
  of the GUI's database (`%APPDATA%\Colosseum\data\colosseum.sqlite`).
  The field and its settings were prepared by the agent the same day.
- **Host:** **Windows 11 on ARM (arm64)**, Surface Laptop 7th Edition,
  Snapdragon X Plus, ten cores, the compatibility-test host of RAR-P41 and
  RAR-P44. It is not the match host.
- **Why:** RAR-M89's hand-crafted frontier, played on the Ryzen 9, repeated
  on this host with every engine that has a native Windows arm64 build,
  plus Dragon 3.3 and Komodo 14.1 under emulation by the maintainer's
  exception.
- **Kind:** an observation. Nothing was registered and no prediction was
  frozen, so it accepts and rejects nothing.
- Disposition: observation — stopped 2026-11-20 at 9,495 of 14,400 games; 2.7.0 fourth of nine, 137.5 below Stockfish 16 HCE on this host; not a gate.

## The field: native and emulated

The code type is read from each binary's PE header
(`IMAGE_FILE_HEADER.Machine`, `eng.py`). An x86-64 binary runs under
Windows' Prism emulator. Every file's modification time is earlier than the
first game (09:40:29 UTC), and its hash is the one in the tournament's
snapshot (`engines.out`). Paths are under `C:\Files\chess\engines\`.

| Engine | Binary | Code | SHA-256 | Origin |
|---|---|---|---|---|
| Stockfish 16 HCE | `stockfish\stockfish-16-arm64-oryon-classical-pgo.exe` | **arm64, native** | `689e4eee…` | built here from `sf_16` (`68e1e9b3`) |
| Berserk 4.7.0 | `berserk-4.7.0-arm64-oryon-pgo.exe` | **arm64, native** | `d2e63d39…` | built here from the `4.7.0` tag |
| Stockfish 11 | `stockfish\stockfish_11-arm64-oryon-pgo.exe` | **arm64, native** | `e260f82e…` | RAR-P44's rebuild, same file |
| Ethereal 14.40 HCE | `ethereal-14.40-hce-arm64-oryon-pgo.exe` | **arm64, native** | `ca1f3f12…` | built here from `0e47e9b`, no network |
| Houdini 6.03 | `houdini\Houdini_6.03_Pro_arm64-pgo.exe` | **arm64, native** | `3c434109…` | RAR-P44's rebuild, same file |
| Rarog 2.7.0 | `rarog\rarog-2.7.0-windows-arm64.exe` | **arm64, native** | `ec785099…` | the release; bench 9,077,016 / EBF 2.481 |
| Basilisk 1.11.0 | `basilisk\basilisk-1.11.0-windows-arm64.exe` | **arm64, native** | `6f2009d0…` | the release |
| Dragon 3.3 HCE | `dragon\dragon-3.3-64bit.exe` | x86-64, **emulated** | `77d50425…` | the official base build |
| Komodo 14.1 | `komodo\komodo-14.1-64bit.exe` | x86-64, **emulated** | `9320b52f…` | the official plain build, as in RAR-P44 |

- **The three builds made here:** MSYS2 `clangarm64`, clang 23.1.5, each
  profile-guided on its engine's own bench. Berserk and Ethereal: the
  sources compiled directly with `-O3 -flto -march=native -static`, Ethereal
  with `-DUSE_NNUE=0`. Stockfish 16 used `make profile-build ARCH=armv8-dotprod
  COMP=clang` with `-mcpu=native`, its net `nn-5af11540bbfe.nnue`, and the
  profile trained on `bench 16 1 13 default depth classical`, the
  evaluation that plays. Three portability changes only: `_setjmp(buf, NULL)`
  mapped to MinGW's arm64 `__mingw_setjmp` (Berserk, Ethereal), Ethereal's
  `_mm_malloc` mapped to `_aligned_malloc`, and an empty `libatomic` for
  Stockfish's link. Benches: Stockfish 16's default bench 2,593,605, as in
  RAR-M89; Berserk 3,150,010, as in its tag; Ethereal 14.40 HCE 2,946,917,
  with no published reference for a build without a network. The script is
  kept locally as `C:\Files\code\compile\frontier_arm64.sh`.
- **Dragon 3.3:** under Prism the avx2 build was 1.8% faster than the base
  build with `Use Regular Eval` and 6.3% slower with NNUE (one thread, four
  positions, two alternating rounds). The maintainer kept the base build
  for later NNUE use.
- **Basilisk 1.11.0** plays as RAR-M89's dev-20261117-e907e603 head: the
  release only removes the ablation build flavour, which is inactive in a
  default build.
- **Left out of RAR-M89's field:** Fire 8.3, rofChade 2.3 and Booot 6.5
  have no Windows arm64 build. Ethereal 13.07, Koivisto 4.83 and Rarog 2.5.0
  were dropped by the maintainer. Komodo 14.1 is new to the field.

## The run

- Round robin, 200 cycles of two colour-reversed games: 400 games a pair,
  14,400 scheduled. `3+0.03`, one thread, Hash 128, six games at a time,
  no ponder, no adjudication, no move cap, no tablebases. UHO
  `UHO_Lichess_4852_v1.epd`, random order, seed 20261117. Move overhead
  10 ms wherever an engine has the option; own books off.
- **Played:** 9,495 games, 263–264 a pair, 2,109–2,112 an engine. All but
  one are in complete colour-reversed pairs; the fit below uses the 9,494
  in pairs.
- **Every game ended naturally:** 6,436 mates, 1,859 repetitions, 715
  insufficient material, 439 fifty-move draws, 46 stalemates. No engine
  forfeited, crashed or played an illegal move, and the GUI log has no
  error on the day.
- **Host use:** the run paused from 10:56:30 to 12:54:10 UTC, while the
  maintainer's separate "HCE Blitz" tournament played 30 games at
  `120+1`; no game of the two overlapped. The GUI restarted at 12:54 and
  13:16; no finished game spans a restart. Other use of the laptop was not
  recorded.

## Ratings

Bradley–Terry on game scores, Stockfish 11 pinned at 3,481, its rating
on this host's library scale from RAR-P44's run (which pinned Rarog 2.4.0
at 3,001); the GUI pinned it the same way, leaving it out of its rating
write-back. 95% intervals from 200 resamples of game pairs within each
engine pair (`hce.py`). The fit reproduces the GUI's column to within 1.

| # | Engine | Code | Rating | Score | In-game nps | Depth |
|---|---|---|---|---|---|---|
| 1 | Stockfish 16 HCE | native | **3,535.5** ± 14.9 | 69.4% | 2.09 M | 25.5 |
| 2 | Stockfish 11 (pinned) | native | 3,481 | 61.6% | 2.13 M | 22.0 |
| 3 | Berserk 4.7.0 | native | 3,462.9 ± 15.0 | 58.9% | 2.23 M | 19.7 |
| 4 | **Rarog 2.7.0** | native | **3,398.0** ± 13.0 | 48.8% | 1.69 M | 12.8 |
| 5 | Ethereal 14.40 HCE | native | 3,394.9 ± 15.4 | 48.4% | 2.72 M | 19.2 |
| 6 | Houdini 6.03 | native | 3,392.9 ± 14.2 | 48.1% | 2.19 M | 13.6 |
| 7 | Komodo 14.1 | emulated | 3,344.1 ± 14.9 | 40.5% | 1.48 M | 14.4 |
| 8 | Basilisk 1.11.0 | native | 3,338.3 ± 14.9 | 39.7% | 1.74 M | 19.1 |
| 9 | Dragon 3.3 HCE | emulated | 3,304.4 ± 16.3 | 34.6% | 1.47 M | 13.8 |

The nps and depth columns are the engines' own reports averaged over
games; each engine defines depth its own way, so that column is not
compared.

## Rarog 2.7.0 head to head

263–264 games each; Elo with 95% pentanomial intervals over the
colour-reversed pairs.

| Opponent | W-D-L | Score | Elo | RAR-M89 (Ryzen 9) |
|---|---|---|---|---|
| Stockfish 16 HCE | 42-88-134 | 32.6% | −126.4 [−157.4, −97.1] | −164.8 |
| Berserk 4.7.0 | 66-77-121 | 39.6% | −73.5 [−104.7, −43.3] | −105.0 |
| Stockfish 11 | 53-110-101 | 40.9% | −63.9 [−94.1, −34.6] | −98.7 |
| Ethereal 14.40 HCE | 89-84-91 | 49.6% | −2.6 [−35.5, +30.2] | −23.2 |
| Houdini 6.03 | 84-93-87 | 49.4% | −3.9 [−34.4, +26.4] | −16.8 |
| Komodo 14.1 (emulated) | 100-91-73 | 55.1% | +35.7 [+6.3, +65.5] | not played |
| Basilisk 1.11.0 | 114-89-59 | 60.5% | +74.0 [+45.3, +103.8] | +37.8 |
| Dragon 3.3 HCE (emulated) | 123-87-54 | 63.1% | +93.0 [+64.0, +123.2] | +20.9 |

In the fit, each engine minus Rarog 2.7.0: Stockfish 16 HCE +137.5 ± 15.3,
Stockfish 11 +83.0 ± 13.0, Berserk +64.9 ± 14.2, Ethereal 14.40 HCE
−3.1 ± 14.2, Houdini 6.03 −5.1 ± 14.6, Komodo 14.1 −53.9 ± 14.9,
Basilisk −59.7 ± 14.2, Dragon −93.6 ± 16.1.

## Against the Ryzen 9 run

The two scales differ (RAR-M89 fixed its anchors' average to the match
host's library), so each engine is compared by its gap to Stockfish 11, the
engine pinned here and played in both. RAR-M89's intervals are about ±7,
these about ±14.

| Engine | Code here | Gap to Stockfish 11, here | RAR-M89 | Here minus RAR-M89 |
|---|---|---|---|---|
| Stockfish 16 HCE | native | +54.5 | +62.3 | −7.8 |
| Berserk 4.7.0 | native | −18.1 | +1.2 | −19.3 |
| Ethereal 14.40 HCE | native | −86.1 | −75.5 | −10.6 |
| Houdini 6.03 | native | −88.1 | −80.7 | −7.4 |
| **Rarog 2.7.0** | native | **−83.0** | **−97.8** | **+14.8** |
| Basilisk 1.11.0 | native | −142.7 | −136.7 | −6.0 |
| Dragon 3.3 HCE | emulated | −176.6 | −119.0 | **−57.6** |
| Komodo 14.1 | emulated | −136.9 | not played | |

## What it shows for Rarog

- **Where 2.7.0 stands on ARM:** fourth of nine, behind Stockfish 16 HCE,
  Stockfish 11 and Berserk, and level with Ethereal 14.40 HCE and
  Houdini 6.03 (gaps of 3 and 5, intervals about ±14). Among the native
  engines the order is the Ryzen's with two changes: Rarog moves from
  behind Ethereal and Houdini to level with them, and Berserk and
  Stockfish 11, level on the Ryzen, swap.
- **The goal's distance:** Stockfish 16 HCE is 137.5 ± 15.3 above in the
  field and −126.4 [−157.4, −97.1] head to head, against 160 and −164.8 on
  the Ryzen.
- **Rarog stands about 15 to 25 better here against the native engines.**
  Against Stockfish 11 it moved up by 14.8, while every other native
  engine moved down by 6 to 19. Each difference carries about ±16, so this is at the
  edge of the intervals and not attributed: code generation on Oryon
  against Zen 3, the engines' relative speed, six games at a time against
  fourteen, and a third as many games are all candidates. It is a platform
  observation, not a strength change; both runs played the 2.7.0 release.
- **Dragon's place here is emulation, not chess.** Dragon 3.3 HCE lost
  57.6 against Stockfish 11 relative to the Ryzen, and Rarog's +93.0 against
  it compares with +20.9 there. Komodo 14.1, also emulated, lands below the
  3,440–3,475 that RAR-M89 guessed for it on its own scale (−67 to −102
  from Stockfish 11 there, −136.9 here), consistent with a cost of the same
  kind, but no Ryzen read separates it. Neither emulated result says where
  Rarog stands against Komodo or Dragon.
- **Basilisk:** +74.0 [+45.3, +103.8] here against +37.8 [+16.5, +59.3] on
  the Ryzen with a play-identical head; the intervals overlap.
- **Robustness:** no Rarog forfeit, crash or illegal move in 2,110 games.
- **Speed:** 2.7.0 reported 1.69 M nps in play here against a one-thread
  bench of 2.29 M on this host, with nothing else running. RAR-M89 reported
  2.0 M in play on the Ryzen.
- **Not usable for a gate or the checkpoint:** the host is not the match
  host, two opponents are emulated and five are rebuilds, and the run was
  stopped at two thirds.

**Artifacts:** `analysis/artifacts/hce-frontier-wos-2026-11-20/` (ignored):
- `colosseum-snapshot.sqlite`, the read-only backup of the GUI's database
  taken after the run stopped (`051bf55c…3cfc00`).
- `hce_frontier.pgn`, the 9,495 finished games (`bebaa188…279475`).
- `tournament_config.json`, the configuration and the per-engine snapshot
  (`3be0f53d…9ffb5b`).
- `eng.py` (RAR-P44's, `9f3a738c…7fb89f`) and its output `engines.out`
  (`80523785…6d521d`).
- `export.py` (RAR-P44's with the PGN name changed, `8d9b9ab7…483f04`).
- `hce.py` (pairs, fit, bootstrap and head to head, `c1be3beb…efddf6`) and
  `hce.out` (`4ab7e0a9…adde98`).
- `rarog270_bench.out`, the 2.7.0 binary's bench on this host
  (`7c09aa47…594999`).

**Retry trigger or `closed`:** closed as an observation. If the maintainer
plays the remaining 4,905 games, this entry is updated with the full count;
no decision waits on it.
