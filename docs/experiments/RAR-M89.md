# RAR-M89 — The hand-crafted frontier: a 14-engine Colosseum GUI round robin at `3+0.03` with Rarog 2.7.0 and 2.5.0 (Basilisk's BAS-M18) — OBSERVED 2026-11-17/18: 2.7.0 sixth of thirteen, 160 below Stockfish 16's hand-crafted evaluation; not a gate

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** played 2026-11-17 and 18 by the maintainer in the Colosseum
  GUI, registered and analysed on Basilisk's side as BAS-M18
  (`D:\code\basilisk\docs\experiments\BAS-M18.md`, Basilisk's branch
  `b11-checkpoint-release`). Recorded here from Basilisk's fit and audit
  files; nothing was re-run or re-derived.
- **Why it matters here:** it sets the pre-NNUE goal (PLAN section 1,
  maintainer decision 2026-11-18): Stockfish 16 with `Use NNUE=false`. It also
  places 2.7.0, the last release before D.7's search rebuild, among the
  strongest hand-crafted engines. 2.5.0 is in the field because it is Rarog's
  release at the end of its search programme, the same point in the plan as
  the Basilisk head.
- **Kind:** an observation. Rarog registered nothing and froze no prediction,
  so it accepts and rejects nothing.
- Disposition: observation — 2.7.0 rates 3,444.6 [3,437.4, 3,451.0], sixth of
  thirteen; Stockfish 16's hand-crafted evaluation 3,604.7, 160 above, −164.8
  [−187.1, −143.6] head to head; 2.7.0 over 2.5.0 +141.7 [+122.0, +162.3].

## The run

- A round robin of 14 engines, 600 games a pair in 300 colour-reversed pairs:
  54,600 games. `3+0.03`, one thread, Hash 128, fourteen games at a time, no
  ponder, no adjudication, no move cap, no tablebases. UHO
  `UHO_Lichess_4852_v1.epd`, random order, seed 20261117. Move overhead 10 ms
  wherever an engine has the option.
- **Every game ended naturally:** 38,677 mates, 9,595 repetitions, 3,857
  insufficient material, 2,192 fifty-move draws, 279 stalemates. No engine lost
  on time, crashed or played an illegal move.
- **Rarog 2.7.0:** the maintainer's local build
  `D:\chess\engines\rarog\rarog-2.7.0-windows-x64-pext.exe` (SHA-256
  `9DAD2C99…`) at its defaults, `LargePages` on. BAS-M17 read it −0.17%
  [−0.27, −0.08] in speed against the release asset, with the same bench,
  9,077,016.
- **Rarog 2.5.0:** `rarog-v2.5.0-windows-pext-pgo.exe` (`1F584523…`), one of
  five rated anchors.
- **Stockfish 16 HCE:** `D:\chess\engines\stockfish-16-x64-bmi2-classical-pgo.exe`
  (`041396A4…`), built from `sf_16` (`68e1e9b3`), `Use NNUE=false`. Stockfish
  16 evaluates purely classically then. This is its last hand-crafted
  evaluation, removed in `af110e02`; it is not the `9587eeeb` tag, which is
  only the last master before NNUE. Default bench 2,593,605.
- Booot 6.5's games are left out of the fit: it exposes no UCI options, so its
  setup could not be checked. Leaving it in moves no other rating by more than
  2.5.

## Ratings

Every engine was fitted freely, then the scale was shifted so that the five
anchors' average matches their Colosseum library ratings. The 95% intervals
come from 200 pentanomial resamples a pairing and leave out the anchors' own
uncertainty (`fit-rarog.txt`).

| # | Engine | Rating |
|---|---|---|
| 1 | **Stockfish 16 HCE** | **3,604.7** [3,598.3, 3,611.4] |
| 2 | Berserk 4.7.0 HCE | 3,543.6 [3,536.2, 3,551.5] |
| 3 | Stockfish 11 (anchor, library 3,532) | 3,542.4 |
| 4 | Ethereal 14.40 HCE (master `0e47e9b` without a network) | 3,466.9 [3,461.0, 3,472.3] |
| 5 | Houdini 6.03 (anchor, library 3,499) | 3,461.7 |
| 6 | **Rarog 2.7.0** | **3,444.6** [3,437.4, 3,451.0] |
| 7 | Dragon 3.3, `Use Regular Eval=true` | 3,423.4 [3,417.2, 3,430.9] |
| 8 | Basilisk `dev-20261117-e907e603` (B.11.1 head) | 3,405.7 [3,399.6, 3,412.5] |
| 9 | Ethereal 13.07 (anchor, library 3,356) | 3,400.8 |
| 10 | Fire 8.3 (anchor, library 3,423) | 3,384.6 |
| 11 | **Rarog 2.5.0** (anchor, library 3,281) | **3,301.5** |
| 12 | rofChade 2.3 | 3,281.4 [3,274.8, 3,287.9] |
| 13 | Koivisto 4.83 | 3,196.8 [3,188.8, 3,205.2] |

The pool's library ratings carry over to this field only to about ±40–50:
with only Stockfish 11 fixed, Houdini 6.03 lands about 48 below its library
rating, Fire 8.3 about 49 below, Ethereal 13.07 about 35 above, and only Rarog
2.5.0 within 30. Read gaps inside this field as the result and absolute
numbers as ±50. Fixing all five anchors left every contender within 2 of the
fit above.

## Rarog 2.7.0 head to head

600 games each; Elo with 95% pentanomial intervals.

| Opponent | W-D-L | Score | Elo |
|---|---|---|---|
| Stockfish 16 HCE | 68-199-333 | 27.9% | −164.8 [−187.1, −143.6] |
| Berserk 4.7.0 HCE | 110-204-286 | 35.3% | −105.0 [−125.7, −85.0] |
| Stockfish 11 | 105-224-271 | 36.2% | −98.7 [−119.3, −78.7] |
| Ethereal 14.40 HCE | 172-216-212 | 46.7% | −23.2 [−42.6, −3.9] |
| Houdini 6.03 | 182-207-211 | 47.6% | −16.8 [−36.5, +2.7] |
| Dragon 3.3 HCE | 224-188-188 | 53.0% | +20.9 [+0.5, +41.4] |
| Basilisk B.11.1 head | 234-197-169 | 55.4% | +37.8 [+16.5, +59.3] |
| Ethereal 13.07 | 243-202-155 | 57.3% | +51.3 [+31.0, +72.0] |
| Fire 8.3 | 250-207-143 | 58.9% | +62.6 [+43.2, +82.4] |
| Rarog 2.5.0 | 300-232-68 | 69.3% | +141.7 [+122.0, +162.3] |
| rofChade 2.3 | 344-172-84 | 71.7% | +161.2 [+139.5, +184.2] |
| Koivisto 4.83 | 426-125-49 | 81.4% | +256.6 [+231.5, +284.3] |

Rarog 2.5.0 against the same field: Stockfish 16 HCE −275.5, Stockfish 11
−247.2, Berserk −226.7, Ethereal 14.40 −160.5, Houdini 6.03 −159.1, Dragon HCE
−119.1, Ethereal 13.07 −105.6, Basilisk head −97.4, Fire −96.8, rofChade +8.7,
Koivisto +98.1.

## Conversion

Basilisk's `tools/diag/conversion_audit.py` at its standard settings (12
persistent plies, 3 material units, lone minors excluded): draws and losses
after a persistent piece-up advantage, per 1,000 of each engine's 7,800 games.

| Engine | Draws | Losses | Thrown |
|---|---:|---:|---:|
| Stockfish 16 HCE | 11.4 | 0.1 | 11.5 |
| Basilisk head | 10.6 | 5.4 | 16.0 |
| **Rarog 2.7.0** | 15.4 | 3.2 | **18.6** |
| **Rarog 2.5.0** | 15.1 | 7.8 | **22.9** |
| Berserk 4.7.0 HCE | 22.4 | 2.8 | 25.3 |
| Ethereal 14.40 HCE | 28.6 | 3.5 | 32.1 |

A stronger engine reaches more piece-up positions, so a rate a game is not a
rate a chance.

## What it shows for Rarog

- **Where 2.7.0 stands.** Sixth of thirteen, and fifth or sixth among
  hand-crafted engines counted one version a family: behind Stockfish 16,
  Berserk 4.7.0 and Ethereal 14.40, level with Houdini 6.03 inside the
  intervals, with Komodo 14.1 (not played; probably 3,440–3,475 here) unplaced.
  99 below second place, 160 below the top.
- **The goal's distance.** 160 in the field, −164.8 head to head. The last
  measured evaluation deficit alone is +99.2 ± 16.6 at equal nodes, and that
  was against Stockfish 11's evaluation inside Stockfish's search (RAR-O07);
  Stockfish 16 leads Stockfish 11 by about 62 here. Phase C followed Stockfish
  11's terms, so Stockfish 16's evaluation is the stronger classical donor
  candidate. Basilisk will choose its donor by measurement, each evaluation
  played inside the other's search; that is Basilisk's plan, not a finding.
- **Two releases apart.** 2.7.0 over 2.5.0 +141.7 [+122.0, +162.3]. RAR-M84
  read 2.6.0 over 2.5.0 +163.0 ± 18.2 in another field, and RAR-M87 read 2.7.0
  against 2.6.0 level to slightly lower; the reads agree within their
  intervals.
- **Conversion.** From 2.5.0 to 2.7.0 the losses from a piece up fell by more
  than half (7.8 to 3.2 a thousand). Against the target the gap is mostly
  draws (15.4 against 11.4), as RAR-M49's fifty-move and repetition finding
  said for 2.4.0.
- **The time control favours Rarog.** Rarog is tuned at exactly `3+0.03` 1T;
  Komodo, Houdini and Fire were tuned for longer games, so at CCRL-like
  controls Rarog's place is probably slightly worse than here.

## In-play speed and depth: observed

- The GUI showed 2.7.0 at **2.0 M nps** in play (bench median 2.34 M) and an
  average reported depth of **13.0**, against Basilisk's 20.1 and Stockfish 16
  HCE's 27.6. BAS-M17 saw 2.7.0 at 1.92 M in play where 2.6.0 ran at 2.43 M.
- **The release-to-release drop is the node count's unit, not speed.** D.6.8,
  in 2.7.0, counts each position once; RAR-S124 measured the same depth and
  time a move with nodes a move ×0.826. 2.43 M × 0.826 = 2.01 M, which is this
  tournament's figure. BAS-M17's 1.92 M leaves about 4% that no read
  separates from run-to-run variation; it is not read.
- **The depth column is not compared.** Each engine reports depth by its own
  definition, and Colosseum's depth includes mate-score moves; D.7's compact
  kernel is the response to the search deficit, and its depth is read against
  its donor there, not against this column.

## Evidence

All on the match host:

- PGN, 54,600 games with each move's score, depth, time and nodes:
  `D:\code\basilisk\tools\results\bas-m18\games.pgn`.
- Fits: `D:\code\basilisk\tools\results\bas-m18\fit-rarog.txt` (the
  anchors'-average fit with and without Booot, both Rarog versions' head to
  head) and `fit.txt` (the registered fit with the five anchors fixed, and
  the fit with only Stockfish 11 fixed).
- Conversion audits:
  `D:\code\basilisk\tools\results\b111-checkpoint\conversion\m18_*.txt` and
  `.json`.
- Basilisk's registration, result and readings:
  `D:\code\basilisk\docs\experiments\BAS-M18.md`.
