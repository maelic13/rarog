# RAR-P60 — macOS on an M4: RAR-M93's "Frontier HCE" field played again natively, a four-engine Colosseum GUI round robin at `3+0.03`, every engine arm64 — OBSERVED 2026-11-26, 2,400 of 2,400 games: the `d9` head −104.7 against Stockfish 16 HCE and −75.0 against Basilisk's rebuilt evaluation, as on the match host

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** played 2026-11-26 18:49 to 19:37 UTC by the maintainer
  in the Colosseum GUI (v1.2.1), tournament "Frontier HCE"
  (`8e471e28-37ff-4f21-b7c8-511fb4831393`), finished, 2,400 of 2,400
  games. Recorded by the agent from a copy of the GUI's database
  (`~/Library/Application Support/Colosseum/colosseum.sqlite`), read
  read-only.
- **Host:** the MacBook Air of RAR-P42, RAR-P43 and RAR-P45: Apple M4,
  four performance and six efficiency cores, 24 GB, macOS 27.2.1. A
  compatibility-test host, not the match host.
- **Why:** the maintainer's check that RAR-M93's reading, taken on the
  match host (Windows, x86-64) the same day, holds on Apple silicon with
  every engine native.
- **Kind:** an observation. Nothing was registered and no prediction was
  frozen, so it accepts and rejects nothing. That the host was idle is not
  established.
- Disposition: observation — every engine native arm64; the `d9` head
  3,507.8 [3,493.2, 3,522.7] on RAR-M89's scale (RAR-M93 3,502.3), −104.7
  [−129.8, −80.5] head to head with Stockfish 16 HCE (RAR-M93 −101.1) and
  −75.0 [−98.8, −51.8] with Basilisk's build (RAR-M93 −77.2); every pairing
  agrees with the match host inside its intervals; no fault.

## The field: all native

Read with `file` and `lipo -archs`: every binary is a thin arm64 Mach-O,
so nothing ran under Rosetta 2. Paths are under `~/Chess/Engines/`; full
hashes and modification times are in `engines.out`. Every file was last
modified before the first game (18:49:27 UTC), the latest at 18:42 UTC.

| Engine | Binary | Code | SHA-256 | Note |
|---|---|---|---|---|
| Rarog dev-20261126-0ffb85bd | `rarog/rarog-dev-20261126-0ffb85bd-macos-arm64` | **arm64, native** | `0fd70795…4eb1` | the maintainer's build of `d9` at `0ffb85b`; `bench 13` **3,694,298 / EBF 2.324** on this file (`dev_bench.out`), the fingerprint |
| Basilisk dev-20261126-41d952f2 | `basilisk/basilisk-dev-20261126-41d952f2-macos-arm64` | **arm64, native** | `1e2a9c5a…6e13` | the maintainer's build of Basilisk at `41d952f`, RAR-M93's commit |
| Stockfish 16 HCE | `stockfish/stockfish-16-hce-macos-arm64-pgo` | **arm64, native** | `271c43c8…760b` | `Use NNUE=false`, `Move Overhead` 10 |
| Stockfish 11 | `stockfish/stockfish-11-macos-arm64-native-pgo` | **arm64, native** | `b81f133b…40bf` | RAR-P45's file; `Move Overhead` 10 |

`0ffb85b` differs from RAR-M93's `0231c3e` in documents only
(`git diff 0231c3e 0ffb85b` touches no engine input), and the bench on
this file matches the fingerprint, so the two Rarog builds play the same
tree. The build settings of the two development builds (PGO, target CPU)
are not recorded in the database.

## The run

Round robin ×200, two colour-reversed games a pair on one
`UHO_Lichess_4852_v1.epd` position (random order, eight plies, seed
202611261947): 2,400 games, 400 a pairing, 200 complete pairs each, none
unpaired. `3+0.03`, one thread, Hash 128, **eight games at a time**, no
ponder, no adjudication, no move cap, no tablebases. Every game ended by
the rules: 1,527 mates, 513 repetitions, 244 insufficient material, 97
fifty-move draws, 19 stalemates. No forfeit, and no engine's clock went
below zero by the move times in the PGN comments (lowest: Stockfish 11
50 ms, Rarog 55 ms).

## Ratings

Bradley–Terry on game scores, fitted free on RAR-M89's scale: Stockfish 16
HCE and Stockfish 11 keep their RAR-M89 mean, 3,573.55. 95% intervals
from 200 resamples of colour-reversed pairs within each pairing
(`rating.py`).

| # | Engine | Rating | Score | RAR-M93 (match host) |
|---|---|---|---|---|
| 1 | Stockfish 16 HCE | 3,601.0 [3,592.9, 3,608.9] | 57.7% | 3,599.0 |
| 2 | Basilisk dev-20261126-41d952f2 | 3,586.6 [3,573.9, 3,600.6] | 55.0% | 3,578.8 |
| 3 | Stockfish 11 | 3,546.1 [3,538.2, 3,554.2] | 47.3% | 3,550.1 |
| 4 | **Rarog dev-20261126-0ffb85bd** | **3,507.8** [3,493.2, 3,522.7] | 40.1% | 3,502.3 |

Two anchors in four engines tie this scale less firmly than RAR-M93's
four in six.

**The GUI's −46 and −44 are its scale, not a loss.** The GUI held both
Stockfish builds at this machine's pool ratings, 3,545 and 3,497, and
moved only the two development builds, which it started from RAR-M93's
3,502 and 3,578 on the match host's scale. It ends them at 3,456 and
3,534; both fall by about the same amount because the two scales sit
apart, and the gaps are the fit's: Rarog 89 below Stockfish 16 HCE in the GUI, 93.2
in the fit.

## Head to head

200 colour-reversed pairs each; Elo with the 95% pentanomial interval.

| Pairing | W-D-L | Pentanomial | Elo | RAR-M93 (match host) |
|---|---|---|---|---|
| `d9` head – Stockfish 16 HCE | 71-141-188 | 32-81-62-22-3 | **−104.7** [−129.8, −80.5] | −101.1 [−121.0, −81.8] |
| `d9` head – Basilisk dev-20261126-41d952f2 | 84-147-169 | 24-69-81-20-6 | **−75.0** [−98.8, −51.8] | −77.2 [−96.4, −58.4] |
| `d9` head – Stockfish 11 | 119-126-155 | 22-48-80-44-6 | −31.4 [−55.7, −7.3] | −45.5 [−64.6, −26.7] |
| Basilisk's build – Stockfish 16 HCE | 114-156-130 | 10-53-90-37-10 | −13.9 [−36.1, +8.2] | −25.2 [−42.9, −7.6] |
| Basilisk's build – Stockfish 11 | 147-156-97 | 6-35-82-57-20 | +43.7 [+20.4, +67.3] | +28.0 [+8.5, +47.7] |
| Stockfish 11 – Stockfish 16 HCE | 101-147-152 | 17-54-98-25-6 | −44.5 [−66.5, −22.9] | 48.9 apart in the fit |

## What it shows

- **Yes, all four engines were native.** Every binary is arm64 only; no
  engine ran under emulation, unlike RAR-P43's Stockfish 11 and RAR-P45's
  Komodo and Critter.
- **RAR-M93 reproduces on Apple silicon.** The two pairings that matter
  agree to within 4: the `d9` head is −104.7 against Stockfish 16 HCE
  (−101.1 on the match host) and −75.0 against Basilisk's rebuilt
  evaluation (−77.2). The pre-NNUE goal's distance is about 100 on both
  architectures.
- **No platform effect is resolved.** The largest differences from the
  match host are Rarog +14.1 and Basilisk +15.7 against Stockfish 11 and
  Basilisk +11.3 against Stockfish 16 HCE, each well inside about ±30 for
  the difference. RAR-P59 saw Rarog 14.8 better against Stockfish 11 on
  Windows on ARM, also inside its intervals; two such reads on two ARM
  hosts are a lead, not a finding.
- **Robustness:** no forfeit, crash or time loss for the `d9` head in its
  1,200 games.

Speed and depth by engine, each engine's mean of per-game values (the
database's columns). These are leads, not measurements: every engine
counts nodes and reports depth its own way, eight games shared four
performance and six efficiency cores of a fanless laptop, and idleness was
not established.

| Engine | NPS | Depth | Move ms | NPS ÷ Rarog | On the match host (RAR-M93) |
|---|---|---|---|---|---|
| Stockfish 11 | 2.408M | 21.03 | 71.7 | 1.39 | 1.43 |
| Stockfish 16 HCE | 2.300M | 24.47 | 72.4 | 1.33 | 1.39 |
| Basilisk dev-20261126-41d952f2 | 1.763M | 19.27 | 71.8 | 1.02 | 1.03 |
| **Rarog dev-20261126-0ffb85bd** | **1.733M** | **15.35** | 73.8 | 1.00 | 1.00 (1.400M) |

**Artifacts:** `analysis/artifacts/frontier-hce-mac-2026-11-26/` (ignored):
- `colosseum-snapshot.sqlite`, a single-file backup of the GUI's database
  after the run (`9556dc20…4840`).
- `frontier_hce_mac.pgn`, the 2,400 games (`793e19b7…37d9`).
- `tournament_config.json` (`7292be56…9775`).
- `engines.out`, each binary's code type, hash and modification time, the
  start ratings and the host (`38e79bd8…331d`).
- `rating.py` (the fit, the bootstrap, the head to head, the speed and
  clock table, `86544bd4…0cac`) and its output `rating.out`
  (`02588eee…b31a`).
- `dev_bench.out`, the Rarog file's bench (`2dbe6073…5547`).
- `standings.webp`, the GUI's standings (`f6871234…a264`).

**Retry trigger or `closed`:** closed as an observation.
