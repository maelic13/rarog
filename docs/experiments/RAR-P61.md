# RAR-P61 — Windows on ARM: RAR-M93's "Frontier HCE" field played natively on the Snapdragon laptop, a four-engine Colosseum GUI round robin at `3+0.03` from balanced SuperGM openings, every engine arm64 — OBSERVED 2026-11-26, 2,400 of 2,400 games: the `d9` head −87.8 against Stockfish 16 HCE and −64.1 against Basilisk's rebuilt evaluation

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** played 2026-11-26 18:49 to 19:39 UTC by the maintainer
  in the Colosseum GUI (v1.2.1), tournament "Frontier HCE"
  (`c5dcfca6-2d8b-4a71-89b7-b865cc66b4b7`), finished, 2,400 of 2,400
  games, at the same time as RAR-P60 on the M4. Recorded by the agent from
  a backup of the GUI's database (`%APPDATA%\Colosseum\data\colosseum.sqlite`),
  opened read-only.
- **Host:** **Windows 11 Pro on ARM (arm64, 10.0.26300)**, the Surface
  Laptop 7th Edition of RAR-P41, RAR-P44 and RAR-P59: Snapdragon X Plus
  X1P64100, ten cores. A compatibility-test host, not the match host.
- **Why:** the maintainer's check that RAR-M93's reading, taken on the
  match host (Windows, x86-64) the same day, holds on Windows on ARM with
  every engine native.
- **Kind:** an observation. Nothing was registered and no prediction was
  frozen, so it accepts and rejects nothing. That the host was idle is not
  established.
- **Not RAR-M93's openings.** This run drew its openings from the balanced
  `SuperGM_4mvs.pgn`; RAR-M93 and RAR-P60 drew theirs from the unbalanced
  `UHO_Lichess_4852_v1.epd`. Balanced openings draw more (47.0% of games
  here, 36.4% in RAR-P60) and so shrink every Elo gap, which confounds the
  host with the book in every comparison below.
- Disposition: observation — every engine native arm64; the `d9` head
  3,523.1 [3,507.8, 3,536.1] on RAR-M89's scale (RAR-M93 3,502.3), −87.8
  [−114.8, −61.8] head to head with Stockfish 16 HCE (RAR-M93 −101.1) and
  −64.1 [−86.9, −41.9] with Basilisk's build (RAR-M93 −77.2); every pairing
  inside its intervals of the match host's; the book differs; no fault.

## The field: all native

The code type is read from each binary's PE header
(`IMAGE_FILE_HEADER.Machine`, `eng.py`, RAR-P59's script): every binary is
arm64, so nothing ran under Windows' Prism emulator. Paths are under
`C:\Files\chess\engines\`; full hashes are in `engines.out`. Every file
was last modified before the first game (18:49:26 UTC), the latest at
18:35:29 UTC.

| Engine | Binary | Code | SHA-256 | Note |
|---|---|---|---|---|
| Rarog dev-20261126-0ffb85bd | `rarog\rarog-dev-20261126-0ffb85bd-windows-arm64.exe` | **arm64, native** | `a07dce00…2a0d` | the maintainer's build of `d9` at `0ffb85b`; `bench 13` **3,694,298 / EBF 2.324** on this file (`dev_bench.out`), the fingerprint |
| Basilisk dev-20261126-41d952f2 | `basilisk\basilisk-dev-20261126-41d952f2-windows-arm64.exe` | **arm64, native** | `6cdc6186…bd39` | the maintainer's build of Basilisk at `41d952f`, RAR-M93's commit |
| Stockfish 16 HCE | `stockfish\stockfish-16-arm64-oryon-classical-pgo.exe` | **arm64, native** | `689e4eee…f526` | RAR-P59's file; `Use NNUE=false`, `Move Overhead` 10 |
| Stockfish 11 | `stockfish\stockfish_11-arm64-oryon-pgo.exe` | **arm64, native** | `e260f82e…cbfd` | RAR-P44's and RAR-P59's file; `Move Overhead` 10 |

`0ffb85b` is RAR-P60's commit and plays RAR-M93's tree (RAR-P60, *The
field*); the bench on this file matches the fingerprint. The build
settings of the two development builds (PGO, target CPU) are not recorded
in the database.

## The run

Round robin ×200, two colour-reversed games a pair on one line of
`C:\Files\chess\books\SuperGM_4mvs.pgn` (random order, eight plies, seed
202611261947, `7aee2aab…fd5e`): 2,400 games, 400 a pairing, 200 complete
pairs each, none unpaired, 1,200 distinct lines with none reused. `3+0.03`,
one thread, Hash 128, **eight games at a time** on ten cores, no ponder,
no adjudication, no move cap, no tablebases. Every game ended by the
rules: 1,273 mates, 686 repetitions, 313 insufficient material, 112
fifty-move draws, 16 stalemates. No forfeit and no loss on time.

A second "Frontier HCE" on this host, from UHO at nine games at a time
(`3cddd1f9-789a-4b58-826b-e150dff45b4d`, 20:06 UTC), was stopped after 154
finished games; it is too short to read and is not used here.

## Ratings

Bradley–Terry on game scores, fitted free on RAR-M89's scale: Stockfish 16
HCE and Stockfish 11 keep their RAR-M89 mean, 3,573.55, as in RAR-P60.
95% intervals are the 2.5 and 97.5 percentiles of 200 resamples of
colour-reversed pairs within each pairing (`rating.py`).

| # | Engine | Rating | Score | RAR-P60 (M4, UHO) | RAR-M93 (match host, UHO) |
|---|---|---|---|---|---|
| 1 | Stockfish 16 HCE | 3,602.3 [3,593.9, 3,609.2] | 56.7% | 3,601.0 | 3,599.0 |
| 2 | Basilisk dev-20261126-41d952f2 | 3,598.4 [3,584.9, 3,612.7] | 55.9% | 3,586.6 | 3,578.8 |
| 3 | Stockfish 11 | 3,544.8 [3,537.9, 3,553.2] | 45.8% | 3,546.1 | 3,550.1 |
| 4 | **Rarog dev-20261126-0ffb85bd** | **3,523.1** [3,507.8, 3,536.1] | 41.7% | 3,507.8 | 3,502.3 |

**The GUI's −44 for both development builds is its scale, not a loss.**
As on the M4, the GUI held both Stockfish builds at this host's pool
ratings, 3,536 and 3,481, and moved only the two development builds from
RAR-M93's 3,502 and 3,578, ending them at 3,458 and 3,534. The gaps agree
with the fit: Rarog 78 below Stockfish 16 HCE in the GUI, 79.2 in the fit.

## Head to head

200 colour-reversed pairs each; Elo of the mean pair score with its 95%
normal interval on pair scores; pentanomial counts are the row engine's
pair scores 0, ½, 1, 1½, 2.

| Pairing | W-D-L | Pentanomial | Draws | Elo | RAR-P60 (M4) | RAR-M93 (match host) |
|---|---|---|---|---|---|---|
| `d9` head – Stockfish 16 HCE | 63-175-162 | 35-69-62-28-6 | 43.8% | **−87.8** [−114.8, −61.8] | −104.7 | −101.1 |
| `d9` head – Basilisk dev-20261126-41d952f2 | 64-199-137 | 20-67-83-26-4 | 49.8% | **−64.1** [−86.9, −41.9] | −75.0 | −77.2 |
| `d9` head – Stockfish 11 | 101-170-129 | 18-57-69-47-9 | 42.5% | −24.4 [−49.3, +0.3] | −31.4 | −45.5 |
| Basilisk's build – Stockfish 16 HCE | 92-213-95 | 10-48-85-49-8 | 53.2% | −2.6 [−24.8, +19.6] | −13.9 | −25.2 |
| Basilisk's build – Stockfish 11 | 156-160-84 | 8-34-68-58-32 | 40.0% | +63.2 [+37.0, +90.2] | +43.7 | +28.0 |
| Stockfish 11 – Stockfish 16 HCE | 66-210-124 | 16-65-85-29-5 | 52.5% | −50.7 [−73.1, −28.8] | −44.5 | 48.9 apart in the fit |

## What it shows

- **Yes, all four engines were native.** Every binary is arm64 by its PE
  header; unlike RAR-P59's Dragon and Komodo, nothing ran under emulation.
- **RAR-M93's order reproduces on Windows on ARM.** Stockfish 16 HCE and
  Basilisk's build level at the top (−2.6 head to head), Stockfish 11
  third, the `d9` head fourth. The `d9` head is −87.8 against Stockfish 16
  HCE and −64.1 against Basilisk's build, against −101.1 and −77.2 on the
  match host: about 13 smaller each, inside the intervals.
- **The smaller gaps are not a platform finding.** The book changed with
  the host, and balanced openings shrink gaps: Rarog's three pairings all
  read 7 to 21 better than in both UHO runs, which fits that. They do not
  shrink uniformly, though, since Basilisk's build reads +63.2 over
  Stockfish 11 against +43.7 and +28.0. One run cannot separate the book
  from the host; the same field from UHO on this host would.
- **Against Stockfish 11 the ARM lead holds.** The `d9` head reads −24.4
  here, −31.4 on the M4 and −45.5 on the match host; RAR-P59 saw 2.7.0
  14.8 better against Stockfish 11 on this host than on the Ryzen. Three
  reads on two ARM hosts now lean the same way, each inside its
  intervals, and this one carries the book confound: still a lead, not a
  finding.
- **Robustness:** no forfeit, crash or time loss for the `d9` head in its
  1,200 games.

Speed and depth by engine, each engine's mean of per-game values (the
database's columns). These are leads, not measurements: every engine
counts nodes and reports depth its own way, eight games shared ten cores
of a laptop, and idleness was not established.

| Engine | NPS | Depth | Move ms | NPS ÷ Rarog | RAR-P60 (M4) | RAR-M93 (match host) |
|---|---|---|---|---|---|---|
| Stockfish 11 | 2.372M | 20.02 | 71.4 | 1.22 | 1.39 | 1.43 |
| Stockfish 16 HCE | 2.341M | 23.26 | 71.7 | 1.20 | 1.33 | 1.39 |
| Basilisk dev-20261126-41d952f2 | 1.903M | 19.40 | 72.0 | 0.98 | 1.02 | 1.03 |
| **Rarog dev-20261126-0ffb85bd** | **1.945M** | **15.95** | 71.8 | 1.00 | 1.00 (1.733M) | 1.00 (1.400M) |

Rarog's node rate stands closest to the Stockfish builds on this host,
as RAR-P59 also found; whether that is the core, the compiler or the
build settings is not established.

**Artifacts:** `analysis/artifacts/frontier-hce-wos-2026-11-26/` (ignored):
- `colosseum-snapshot.sqlite`, a backup of the GUI's database after the
  run (`c1a47b75…2f93`).
- `frontier_hce_wos.pgn`, the 2,400 games (14,875,141 bytes,
  `0977b23d…7e09`), and `tournament_config.json` (`a7b8a4c0…7037`), written
  by `export.py` (`b46cb7bd…4b1c`).
- `engines.out`, each binary's code type, hash, size and modification date
  and its options (`214de92f…9e7f`), written by `eng.py` (`9f3a738c…b89f`).
- `rating.py` (the fit, the bootstrap, the head to head and the speed
  table, `aab0465a…c692`) and its output `rating.out` (`577f2216…26ab`).
- `dev_bench.out`, the Rarog file's bench (`1333fd59…4006`).
- `standings.webp`, the GUI's standings (`dd4c27cd…0977`).

**Retry trigger or `closed`:** closed as an observation. The book
question is answered by the same four engines from UHO on this host,
which the maintainer may run; nothing waits on it.
