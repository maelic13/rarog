# RAR-P41 — Windows on ARM: two Colosseum GUI tournaments on a Snapdragon laptop, 2.4.0–2.6.0 against the field — OBSERVED 2026-10-22/23: no Rarog fault in about 1,700 games

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** played 2026-10-22/23 by the maintainer. Recorded by the
  agent from the maintainer's screenshots of the Colosseum GUI (v1.1.0); no
  run directory or PGN was kept in this repository.
- **Host:** Surface Laptop 7th Edition, Snapdragon X Plus, Windows on ARM,
  the compatibility-test host AGENTS names. It is not the match host.
- **Binaries:** the Rarog 2.4.0, 2.5.0 and 2.6.0 releases and the pool
  engines. Their builds and hashes were not recorded, which limits this
  entry to observation.
- **Kind:** an observation, not a measurement anyone gates on. Nothing was
  registered.

## The two runs

**1. Rapid, four threads.** Created 2026-10-22 22:17.
- Ten engines, round robin ×2, **180 games**, 120 s + 1 s, two games at a
  time, tablebases on, ponder off, SuperGM four-move openings, ratings
  never updated (36 games an engine).
- **Every game ended naturally:** 123 mates, 37 threefolds, 11 insufficient
  material, 8 fifty-move, 1 stalemate. No forfeit or crash for any engine.
- Rarog performance: 2.6.0 3468, 2.5.0 3472, 2.4.0 3149, each about ±100
  at 36 games. 2.6.0 and 2.5.0 drew all four of their games.
- Mean NPS at four threads, the engines' own reports: 2.6.0 4.2M, 2.5.0
  3.8M, 2.4.0 4.8M.

**2. Bullet, one thread, stopped.** Created 2026-10-23 07:35, stopped at
3,712 of 36,400 games.
- Fourteen engines, round robin ×200, `3+0.03`, eight games at a time,
  tablebases off, the UHO book, ratings updated (about 530 games an
  engine).
- **No forfeit, crash or illegal move for any Rarog release:** about 1,590
  games between them. The four engine errors are all Houdini 5.01's (three
  time losses, one crash).
- Ratings: 2.6.0 3481, 2.5.0 3338, 2.4.0 3131.
- Mean NPS: 2.6.0 2.0M, 2.5.0 1.8M, 2.4.0 2.4M.

## What it shows

- **Robustness on ARM:** no Rarog fault in about 1,770 games across both
  controls, four threads with tablebases and one thread at bullet.
  - That includes 2.6.0's old clock, whose `3+0.03` floor is about 155 ms
    on the match host (RAR-R14), with no forfeit.
  - Evidence for D.3's robustness work and for the release platform list.
- **The pool's ratings are not comparable with the match host's or
  CCRL's.**
  - The engines that gained against their entered ratings are the native
    ARM builds: Rarog 2.4.0 +128, 2.5.0 +87, 2.6.0 +11, Basilisk +101.
  - Those that lost are x86-only, presumably emulated, their NPS low:
    Stockfish 11 −91, Komodo 14.1 −229 at 592k, Fire −303 at 662k,
    Ethereal −316, Houdini 5.01 −618.
  - 2.6.0 level with Stockfish 11 here says the field was handicapped; on
    the match host 2.6.0 is about 90 below it (RAR-M75). Not usable for
    E.2's targets.
- **Rarog against Rarog** roughly agrees with the match host, within this
  run's resolution (about ±40 for a difference at 530 games):
  - 2.6.0 over 2.5.0: +143 here, against RAR-M76's +153.4 ± 10.7;
  - 2.5.0 over 2.4.0: +207 here, against RAR-M64's +272.
- **A speed hint for D.2's hold, not evidence.** On this host 2.6.0 is
  faster than 2.5.0 at one and four threads (+11%), and 2.4.0 is the
  fastest. On the match host the C head was 29.8% faster than the B.9 head
  at one thread and 9.2% slower at four (RAR-M74).
  - The ARM figures are the engines' own reports over different positions
    and times a move.
  - The x86 build's PEXT path and large pages do not apply on ARM.
  - What would turn the hint into evidence: `tools/nps_scaling.ps1` on
    both hosts with recorded builds, the hold's own resolve condition.

**Retry trigger or `closed`:** closed as an observation. A rated ARM pool
needs native builds of the opponents, or their emulation recorded per engine.
