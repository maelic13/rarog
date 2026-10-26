# RAR-P43 — macOS on an M4: a 26,400-game Colosseum GUI round robin at `3+0.03`, Rarog 2.2.0–2.6.0 against the field — OBSERVED 2026-10-23: no forfeit; the release gaps reproduce the match host's; RAR-R13's trigger not fired

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** played 2026-10-23 14:25–22:42 UTC by the maintainer in
  the Colosseum GUI (v1.1.0), tournament "Rating Tournament". Recorded by the
  agent from the GUI's own database
  (`~/Library/Application Support/Colosseum/colosseum.sqlite`, tournament
  `89da791e-be75-4239-baf1-aba4fa5f7cf0`), read from a copy.
- **Host:** the MacBook Air (Apple M4) of RAR-P42, a compatibility-test
  host, not the match host.
- **Binaries:** RAR-P42's Rarog 2.4.0, 2.5.0 and 2.6.0, plus 2.3.2
  `029d0cb3…c2d5c9` (`rarog-v2.3.2-macos-arm64-native-pgo`) and 2.2.0
  `add165e8…43b36c` (`rarog-v2.2.0-macos-arm64-pgo`), each answering its own
  `id name`. Basilisk, Manta and Hydra are native arm64; **Stockfish 11 is
  x86_64 under Rosetta 2**, as in RAR-P42.
- **Kind:** an observation, not a measurement anyone gates on. Nothing was
  registered.

## The run

- Twelve engines, round robin ×200 with two games a pair (colours reversed
  on one UHO_Lichess_4852_v1 position, random order, seed 99): **26,400
  games**, 400 a pair, 4,400 an engine. `3+0.03`, one thread, Hash 128,
  eight games at a time, tablebases off, ponder off, no adjudication.
- Ratings: Rarog 2.4.0 held at 3001, every other engine fitted.
- **Every game ended naturally:** 21,891 mates, 3,220 threefolds, 743
  insufficient material, 469 fifty-move, 77 stalemates. No forfeit, crash
  or illegal move for any engine. The GUI forfeits only past a 2,000 ms
  margin. Three clocks went below zero by the GUI's move times without
  forfeiting: 2.4.0 −52 ms, Basilisk 1.9.3 −22 ms, 2.3.2 −4 ms, one game each.
  Rarog 2.6.0's lowest was 43 ms.

## What it shows

Head-to-head Elo below is pentanomial over the 200 position pairs, with a
95% interval.

- **The release gaps reproduce the match host's:**
  - 2.6.0 over 2.5.0: **+139.0 ± 21.9** (196-160-44). RAR-M76 measured
    +153.4 ± 10.7.
  - 2.5.0 over 2.4.0: **+270.9 ± 29.0** (282-97-21). RAR-M64 measured +272.4.
  - Further down: 2.4.0 over 2.3.2 +60.5 ± 25.9, and 2.3.2 over 2.2.0
    +149.3 ± 25.1.
- **Against Basilisk:** 2.6.0 +344 ± 40 over 1.10.2; 2.4.0 −15.6 ± 27.0
  against 1.10.2 and +11.3 ± 27.4 against 1.9.3. Basilisk 1.10.2 is level
  with 1.9.3 (+6.9 ± 22.6).
- **Stockfish 11 is handicapped here.** It beat 2.6.0 by only +16.5 ± 25.1
  (134-151-115). On the match host the C head, which 2.6.0 was cut from, is
  about 90 below it (RAR-M75). Stockfish ran emulated at 1.68M nps, and
  RAR-P41 found the same handicap on Windows on ARM. Not usable for E.2.
- **Speed at one thread** (the engines' own reports): 2.6.0 2.14M nps,
  2.5.0 1.85M, 2.4.0 2.47M. 2.6.0 is 13% slower than 2.4.0 here; the match
  host measured −8.6% in games, and RAR-P41 measured −17% on ARM. Same
  direction, different positions and hosts.
- **The low-clock floor of RAR-R14 shows here too.** These are shares of
  each engine's games whose clock went below 100 ms after a move, rebuilt
  from the GUI's move times:
  - 2.6.0 17.7%, 2.5.0 29.9%, 2.4.0 37.4%, 2.3.2 38.0%, 2.2.0 56.2%;
  - Stockfish 0%, its lowest 167 ms.

  The d1 clock cluster (D.1.3) targets this; this run used the releases.
- **RAR-R13's first trigger did not fire for 2.6.0.**
  - 34 of its games were drawn after it had held +300 or more for six of
    its moves, and none was lost.
  - None of the 34 showed a mate for 2.6.0 or a won-position move at depth
    40 or more. Their longest won-position moves were depth 12–16, at
    clocks of 126–296 ms: low-clock moves, not deep stalls.
  - A root lower bound is not visible in the PGN, so that third signature
    could not be read.

**Artifacts:** `analysis/artifacts/rating-tour-mba-2026-10-23/` (ignored):
`rating_tournament.pgn`, the 26,400 games exported from the database
(131,755,928 bytes, `738b994d…6917d0`); `tournament_config.json`
(`1ec879fd…31f29`); `rating.py` (pairs, pentanomial Elo, clocks, NPS and
won-position outcomes, `075544f2…f3e2e`) and its output `rating.out`
(`eba4bd49…da13b`); `r13.py` (the RAR-R13 signatures, `6c829240…e23c5906`)
and its output `r13.out` (`0566adb9…95ac81`).

**Retry trigger or `closed`:** closed as an observation. RAR-R13's triggers
are unchanged.
