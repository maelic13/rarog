# RAR-M91 — The merged kernel in RAR-M89's field with Komodo 14.1 and Basilisk's development build: the "Dev Test Frontier HCE" Colosseum GUI round robin, twelve engines, 52,800 games at `3+0.03` — OBSERVED 2026-11-22: 3,500.6, fourth of twelve, 100.2 below Stockfish 16 HCE

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** played 2026-11-21 22:07 to 2026-11-22 07:53 UTC by the
  maintainer in the Colosseum GUI (v1.2.0), tournament "Dev Test Frontier
  HCE" (`658cb048-46e0-47af-b7c3-cd7b012e6d0a`), on the match host.
  Recorded by the agent from the GUI's database, opened read-only.
- **Why:** the first field read of `master` after D.7 landed (`94d7f72`,
  PR #20), beside RAR-M90's pinned gauntlet. It also places Komodo 14.1,
  which RAR-M89 left unplaced, and Basilisk's first Phase C build.
- **Kind:** an observation. Nothing was registered and no prediction was
  frozen, so it accepts and rejects nothing.
- Disposition: observation — the merged kernel rates 3,500.6 [3,495.1,
  3,506.3] on RAR-M89's scale, fourth of twelve, 100.2 below Stockfish 16
  HCE and −103.3 [−121.0, −86.0] head to head; RAR-M90 reproduced.

## The field

Every opponent is RAR-M89's file (SHA-256 prefixes as there) with RAR-M89's
options: move overhead 10 ms where an engine has it, Stockfish 16 `Use
NNUE=false`, Dragon 3.3 `Use Regular Eval=true` and `OwnBook=false`, Houdini
6.03 `NUMA Enabled=false`. New against RAR-M89:

| Engine | Binary (`D:\chess\engines\`) | SHA-256 | Note |
|---|---|---|---|
| Rarog dev-20261121-94d7f72d | `rarog\rarog-dev-20261121-94d7f72d-windows-x64-pext.exe` | `07d4068f…934c` | the maintainer's build of `master` at `94d7f72`; `bench 13` 3,390,842 / EBF 2.317 on this file (`dev_bench.out`), the gated kernel's fingerprint |
| Komodo 14.1 | `komodo\komodo-14.1-64bit-bmi2.exe` | `0d32bcbd…7fd` | RAR-M87's file; `Overhead ms=10`, `OwnBook=false` |
| Basilisk dev-20261121-f0ed0612 | `basilisk\basilisk-dev-20261121-f0ed0612-windows-x64-pext.exe` | `1d330f97…cc74` | Basilisk's first Phase C build: its evaluator split into family modules, 26 commits after `v1.11.0` |
| Basilisk 1.11.0 | `basilisk\basilisk-1.11.0-windows-x64-pext.exe` | `f6ca0a44…d756` | the release; plays as RAR-M89's B.11.1 head (RAR-P59) |

Left out of RAR-M89's field: Ethereal 13.07, Rarog 2.5.0, rofChade 2.3 and
Koivisto 4.83.

## The run

Round robin, 400 cycles of two colour-reversed games: 800 games a pair,
52,800 games, all finished and all in complete pairs. `3+0.03`, one thread,
Hash 128, 14 games at a time, no ponder, no adjudication, no move cap, no
tablebases; UHO `UHO_Lichess_4852_v1.epd` in random order, seed 4515313351.
Every game ended by the rules: 34,946 mates, 11,017 repetitions, 4,098
insufficient material, 2,388 fifty-move draws, 351 stalemates. The GUI's
forfeit column is empty for every engine.

## Ratings

Bradley–Terry on game scores, fitted on RAR-M89's scale: the nine engines
RAR-M89 rated (Basilisk 1.11.0 as its B.11.1 head) keep their RAR-M89 mean,
3,475.3, and every engine is free. 95% intervals from 200 resamples of
colour-reversed pairs within each pairing (`rating.py`). The GUI's
performance column agrees to within 9.

| # | Engine | Rating | Score | RAR-M89 |
|---|---|---|---|---|
| 1 | Stockfish 16 HCE | 3,600.8 [3,596.2, 3,605.2] | 69.0% | 3,604.7 |
| 2 | Stockfish 11 | 3,549.3 [3,543.8, 3,554.2] | 61.8% | 3,542.4 |
| 3 | Berserk 4.7.0 HCE | 3,535.5 [3,530.3, 3,540.4] | 59.8% | 3,543.6 |
| 4 | **Rarog dev-20261121-94d7f72d** | **3,500.6** [3,495.1, 3,506.3] | 54.5% | |
| 5 | Houdini 6.03 | 3,469.7 [3,464.8, 3,474.7] | 49.8% | 3,461.7 |
| 6 | Komodo 14.1 | 3,468.8 [3,463.3, 3,474.2] | 49.7% | |
| 7 | Ethereal 14.40 HCE | 3,465.2 [3,459.7, 3,470.0] | 49.2% | 3,466.9 |
| 8 | Rarog 2.7.0 | 3,433.3 [3,428.0, 3,438.5] | 44.3% | 3,444.6 |
| 9 | Dragon 3.3 HCE | 3,423.8 [3,419.4, 3,428.1] | 42.9% | 3,423.4 |
| 10 | Basilisk dev-20261121-f0ed0612 | 3,406.5 [3,400.6, 3,412.1] | 40.3% | |
| 11 | Basilisk 1.11.0 | 3,406.0 [3,401.5, 3,410.3] | 40.2% | 3,405.7 |
| 12 | Fire 8.3 | 3,394.1 [3,388.9, 3,398.7] | 38.5% | 3,384.6 |

The nine anchored engines move by −11.3 to +9.5 against RAR-M89; that is
this scale's own spread between two fields, so a difference of about ten
between the two reads is not readable.

## The merged kernel head to head

400 colour-reversed pairs each; Elo with the 95% pentanomial interval.

| Opponent | W-D-L | Elo | RAR-M90 |
|---|---|---|---|
| Stockfish 16 HCE | 141-287-372 | −103.3 [−121.0, −86.0] | −100.6 |
| Stockfish 11 | 205-284-311 | −46.3 [−63.5, −29.3] | −47.8 |
| Berserk 4.7.0 HCE | 231-240-329 | −42.8 [−61.0, −24.8] | −34.9 |
| Houdini 6.03 | 300-285-215 | +37.1 [+20.1, +54.2] | +21.5 |
| Komodo 14.1 | 298-285-217 | +35.3 [+17.0, +53.8] | not played |
| Ethereal 14.40 HCE | 298-268-234 | +27.9 [+10.2, +45.7] | +47.2 |
| Rarog 2.7.0 | 311-357-132 | +79.1 [+64.1, +94.4] | +78.3 |
| Dragon 3.3 HCE | 366-234-200 | +73.2 [+54.7, +92.0] | +87.5 |
| Basilisk dev-20261121-f0ed0612 | 367-271-162 | +91.1 [+72.8, +109.9] | not played |
| Basilisk 1.11.0 | 376-243-181 | +86.4 [+68.7, +104.6] | +112.7 (B.11.1 head) |
| Fire 8.3 | 398-263-139 | +116.7 [+100.0, +133.8] | +113.9 |

## What it shows

- **RAR-M90 reproduces.** The merged kernel rates 3,500.6 [3,495.1,
  3,506.3] here against 3,508.1 ± 8.2 there, −103.3 against −100.6 head to
  head with Stockfish 16 HCE, and +79.1 against +78.3 over 2.7.0. Every
  pair the two reads share agrees within their intervals; the largest
  differences, Basilisk (−26.3), Ethereal 14.40 (−19.3) and Houdini (+15.6),
  are each inside the combined interval.
- **Its place:** fourth of twelve, 100.2 below Stockfish 16 HCE, 35 to 49
  below Berserk and Stockfish 11, and about 31 above the group of Houdini
  6.03, Komodo 14.1 and Ethereal 14.40 HCE, against all three of which it
  scores above zero head to head. Counted one version a family, it is the
  third hand-crafted engine, after Stockfish and Berserk.
- **Komodo 14.1 is placed:** 3,468.8, level with Houdini 6.03 and Ethereal
  14.40 HCE and inside RAR-M89's guess of 3,440–3,475. It is 45.0 above
  Dragon 3.3 HCE in the fit and +43.7 [+28.1, +59.4] head to head, so
  Komodo is the stronger of the two on this host too, as on the ARM laptop
  (RAR-P59, +39.7 under emulation).
- **Basilisk's Phase C build plays as 1.11.0:** 3,406.5 against 3,406.0 in
  the fit, −3.5 [−17.7, +10.7] head to head; its evaluator's restructuring
  moves nothing measurable.
- **Rarog 2.7.0** reads 11.3 below its RAR-M89 rating, at the edge of the
  scale's spread above; nothing in it is attributed.
- **Robustness:** no forfeit, crash or time loss by either Rarog in their
  16,800 games.

**Artifacts:** `analysis/artifacts/dev-test-frontier-2026-11-21/` (ignored):
`dev_test_frontier.pgn`, the 52,800 games exported from the database
(319,196,412 bytes, `393115d4…79ca`); `tournament_config.json`
(`eca81c66…ae54`); `rating.py` (binaries, the fit, the bootstrap and the
head-to-head reads, `ae2bca65…9221`) and its output `rating.out`
(`bef8563e…126e`); `dev_bench.out`, the Rarog build's bench
(`65c6c572…6d3b`); `standings.webp`, the GUI's standings (`f041870f…fa3d`).

**Retry trigger or `closed`:** closed as an observation; nothing changes on
it.

## Speed and depth by engine (added 2026-11-22)

Read for E.0's research card, from the same tournament in Colosseum's
database (opened read-only): each engine's mean over its 8,800 games of
the per-game `white_nps`/`black_nps` and `white_depth`/`black_depth`,
taken by engine id, all 52,800 games finished. Leads, not measurements:
every engine counts nodes and reports depth its own way, and Colosseum's
depth includes moves with a mate score.

| Engine | NPS | Depth | Move ms |
|---|---|---|---|
| Ethereal 14.40 HCE | 2.989M | 19.64 | 72.7 |
| Berserk 4.7.0 HCE | 2.414M | 19.78 | 71.7 |
| Houdini 6.03 | 2.375M | 13.80 | 66.2 |
| Fire 8.3 | 2.313M | 13.19 | 48.1 |
| Stockfish 11 | 2.219M | 21.75 | 71.3 |
| Stockfish 16 HCE | 2.160M | 25.22 | 72.2 |
| Komodo 14.1 | 2.082M | 15.39 | 69.0 |
| Dragon 3.3 HCE | 2.074M | 14.85 | 61.8 |
| Basilisk 1.11.0 | 1.692M | 18.88 | 73.8 |
| Basilisk dev-20261121-f0ed0612 | 1.675M | 18.79 | 73.9 |
| **Rarog dev-20261121-94d7f72d** | **1.579M** | **15.03** | 71.4 |
| Rarog 2.7.0 | 1.523M | 12.72 | 70.8 |

The recipe: `games` joined to `tournament_engines` on the tournament id
`658cb048-46e0-47af-b7c3-cd7b012e6d0a`, both sides of each finished game
taken as one row (`white_id`, `white_nps`, `white_depth`, `white_move_ms`,
and the same for Black), averaged by engine id.

## Read again for E.0 (added 2026-11-23)

The table above is each engine's mean of per-game means. Parsed a move at
a time from the exported PGN (`analysis/artifacts/e0-2026-11-23/m91_phase.py`),
nodes over the time charged, pooled over every move: the kernel 1.438M,
Stockfish 16 HCE 2.127M, Stockfish 11 2.179M, Berserk 4.7.0 HCE 2.233M,
Ethereal 14.40 HCE 2.703M, Rarog 2.7.0 1.439M, Basilisk 1.11.0 1.520M.
Depth without mate scores: the kernel 15.40, Stockfish 16 HCE 18.08,
Stockfish 11 15.85, Berserk 18.09. E.0's packet
(`analysis/e0_remaining_strength_2026-11-23.md`) reads these, not the
table's figures.
