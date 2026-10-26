# RAR-P42 — macOS on an M4: a Colosseum GUI round robin at 120+1, Rarog 2.4.0–2.6.0 against the field — OBSERVED 2026-10-23: no Rarog fault in 132 games; 2.6.0 still searches past a proved mate at CCRL blitz, at no cost

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** played 2026-10-23 11:37–14:18 UTC by the maintainer in
  the Colosseum GUI (v1.1.0), tournament "Blitz Tour". Recorded by the agent
  from the GUI's own database
  (`~/Library/Application Support/Colosseum/colosseum.sqlite`, tournament
  `94acbe1a-2155-433f-b4c8-83cfd2aca2d9`), read from a copy.
- **Host:** MacBook Air, Apple M4 (10 cores, 24 GB), macOS, the
  compatibility-test host AGENTS names. It is not the match host.
- **Binaries:** the native arm64 PGO releases, each answering its own
  `id name`: Rarog 2.4.0 `e3e8e70e…35b2cb`, 2.5.0 `a3625c84…b231bd`, 2.6.0
  `fd0c95dd…62a68d` (`~/Chess/Engines/rarog/rarog-v<ver>-macos-arm64-native-pgo`).
  Basilisk 1.8.0–1.10.2, Manta, Hydra, Whitespine and Beast are native
  arm64; **Stockfish 11 is x86_64 and ran under Rosetta 2.** No bench was
  taken.
- **Kind:** an observation, not a measurement anyone gates on. Nothing was
  registered.

## The run

- Twelve engines, round robin ×2 with two games a pair, **264 games**,
  44 an engine; 120 s + 1 s (CCRL blitz), one thread and Hash 1024 for every
  engine, eight games at a time, Syzygy 3–5 on, ponder off, no adjudication,
  SuperGM four-move openings at random (seed 24), ratings never updated.
- **Every game ended naturally:** 220 mates, 37 threefolds, 6 fifty-move,
  1 insufficient material. No forfeit, crash or illegal move for any engine.
- Rarog's scores: 2.6.0 33-8-3 (84.1%), 2.5.0 29-14-1 (81.8%), 2.4.0
  23-11-10 (64.8%). The GUI's performance column (3312, 3303, 3030) is fitted
  to the ratings the maintainer typed in, so it is not a measurement.
- Mean NPS, the engines' own reports: 2.6.0 1.4M, 2.5.0 1.1M, 2.4.0 1.4M,
  Basilisk 1.6–1.7M, Stockfish 11 988k.

## What it shows

- **Robustness on macOS arm64:** no Rarog fault in 132 games with
  tablebases on. No Rarog clock fell below 1.3 s after a move, the floor
  every engine here reached in long games.
- **2.6.0 still searches past a proved mate at CCRL blitz** (RAR-R13's rec2).
  Of its eleven moves over 15 s, six were in won positions, five of them
  with a mate shown: mate 4 at depth 97–98 taking 26.8 s of 59.4 s and
  27.8 s of 67.2 s; mate 4 at depth 86 taking 26.8 s of 48.4 s, and at depth
  68 taking 23.4 s of 32.8 s; mate 7 taking 30.9 s of 85.6 s.
  - Every one of those games was won, so RAR-R13's second trigger (a record
    at `2+1` or longer showing such moves costing results) **did not fire**.
  - The d1 head's proved-mate stop (clock cluster D.1.3), on by default in
    clock searches, targets these moves; this run used the release.
- **Against Stockfish 11:** 2.6.0 0-1-3, 2.5.0 0-3-1. Stockfish ran
  emulated and still scored 41.5/44.
  - None of 2.6.0's three losses came from the clock. Two were lost by
    moves 18–19 with 75–80 s left. In the third, Rarog's best score was
    +126; it fell below −100 only at move 82, with 7.0 s left against
    Stockfish's 5.4 s.
  - The emulated x86 binary makes the pairing unusable for E.2. On the match
    host the C head, which 2.6.0 was cut from, is about 90 below Stockfish 11
    (RAR-M75).
- **Rarog against Rarog** is too small to read: 2.6.0–2.5.0 four draws,
  2.6.0–2.4.0 3-1-0, 2.5.0–2.4.0 2-2-0, each four games (about ±400). It does
  not contradict RAR-M76 (+153.4) or RAR-M64 (+272).

**Artifacts:** `analysis/artifacts/blitz-tour-mba-2026-10-23/` (ignored):
`blitz_tour.pgn`, the 264 games exported from the database
(`6d309d93…b500e`); `tournament_config.json` (`2921b645…49718`);
`tour.py` (pair results, clock use, won-position outcomes, `6ca10530…779518`)
and its output `tour.out` (`52b5424d…71012f`); `long.py` (moves over 15 s,
`efc7bb2b…f9651d`) and `long.out` (`f9777ebf…ef9439`); `sf.py` (the
Stockfish games, `c1edb1f5…c15317`) and `sf.out` (`5e2b2e3a…ce970`).

**Retry trigger or `closed`:** closed as an observation. RAR-R13's triggers
are unchanged.
