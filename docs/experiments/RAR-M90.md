# RAR-M90 — D.7.14: the pre-NNUE meter read again, the adopted kernel against RAR-M89's field, a 7,800-game Colosseum gauntlet at `3+0.03` — READ 2026-11-21: 3,508.1 ± 8.2, 96.6 below Stockfish 16 HCE, −100.6 head to head

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-21, before any game; the run is the
  maintainer's command.
- **Kind:** a measurement at a checkpoint (PLAN rule 10; PLAN D.7.14: "the
  pre-NNUE meter read again"). It accepts and rejects nothing.
- Disposition: observation — read 2026-11-21: 3,508.1 ± 8.2 with the field pinned, 96.6 below Stockfish 16 HCE and −100.6 head to head; it decides nothing.
- **Why now:** the maintainer adopted the new kernel on 2026-11-21 after
  RAR-S136's H1. The meter (PLAN section 1) is Rarog's gap to Stockfish 16
  with `Use NNUE=false`, read head to head and in a rated field; its last
  reading, RAR-M89, is of 2.7.0, the classic kernel.

## What plays

- **The seed:** `d713kern`, the binary RAR-S136 gated:
  `tools/test_engines/rarog-d713kern-pext-pgo.exe`, `eb84cce`, `id name`
  `dev-20261121-eb84cce0`, `bench 13` 3,390,842 / EBF 2.317, SHA-256
  `e2ad12685bc8f5c42254827d95fbffdf485180f83a73ffcfd6f443406d48ac99`, at
  its defaults.
- **The field:** RAR-M89's thirteen engines, the same files (each SHA-256
  checked against RAR-M89's prefix by the driver, all thirteen matched on
  2026-11-21), each with the options RAR-M89 recorded (move overhead 10 ms
  where an engine has it; Stockfish 16 `Use NNUE=false`; Dragon 3.3
  `Use Regular Eval=true`, `OwnBook=false`; Houdini 6.03 `NUMA
  Enabled=false`; Koivisto `OwnBook=false`), and each **pinned at its
  RAR-M89 rating** (`--fixed`):

  | Engine | Pin |
  |---|---:|
  | Stockfish 16 HCE | 3,604.7 |
  | Berserk 4.7.0 HCE | 3,543.6 |
  | Stockfish 11 | 3,542.4 |
  | Ethereal 14.40 HCE | 3,466.9 |
  | Houdini 6.03 | 3,461.7 |
  | Rarog 2.7.0 | 3,444.6 |
  | Dragon 3.3 HCE | 3,423.4 |
  | Basilisk B.11.1 head | 3,405.7 |
  | Ethereal 13.07 | 3,400.8 |
  | Fire 8.3 | 3,384.6 |
  | Rarog 2.5.0 | 3,301.5 |
  | rofChade 2.3 | 3,281.4 |
  | Koivisto 4.83 | 3,196.8 |

  Booot 6.5 stays out, as it did of RAR-M89's fit.

## Conditions (frozen)

- Colosseum CLI 0.5.0, the pinned build, through `tools/colosseum.ps1
  -Mode gauntlet` (`gauntlet.toml`): one seed, 300 cycles of 2 games,
  **600 games a pair in colour-reversed pairs, 7,800 games**.
- `3+0.03`, one thread, Hash 128, 14 games at a time, harness margin
  2,000 ms (the desktop's, which RAR-M89 played under), no ponder, no
  adjudication, no tablebases. UHO `UHO_Lichess_4852_v1.epd` in random
  order, seed 2026112190 (in no earlier registration, driver or run
  directory), `tools/results/rar-m90-meter`.
- Stop rule: one fixed run to its end, no early stop, no extension. A run
  voided by the wrapper's fault ceiling (78 faults, 1%) is repeated, not
  read.
- About 1 hour 40 minutes at RAR-M89's 4,824 games an hour.

## What is read (frozen)

1. **The meter.** The kernel's rating as Colosseum fits it with the field
   pinned, with its interval; the gap is 3,604.7 less that rating. Head to
   head, Colosseum's Elo against Stockfish 16 HCE with its 95% interval.
   Every pair's W-D-L and Elo is reported.
2. **The gain over the release in the same field:** the kernel's rating
   less 2.7.0's 3,444.6, and Colosseum's head-to-head Elo against 2.7.0.
3. **Conversion:** `tools/diag/conversion_audit.py --engine "Rarog
   d713kern"` on the run's `games.pgn`, draws and losses a piece up per
   1,000 games. The same tool on BAS-M18's PGN reproduces RAR-M89's 2.7.0
   row exactly (15.4 draws and 3.2 losses, 120 and 25 of 7,800;
   `analysis/artifacts/d714-meter-2026-11-21/conversion_m89_270.log`), so
   the two compare like for like. A stronger engine reaches more piece-up
   positions, so a rate a game is not a rate a chance.
4. **Speed in games:** Colosseum's search line for the kernel (nodes a
   second, depth, time a move), beside RAR-S136's.

Validity: every game a rules result; each engine's time losses and faults
reported; an engine losing more than 1% of its games on time is listed and
the kernel's rating given with and without those games, as in RAR-M89.

Uses: (1) and (2) are GUIDE's *Pre-NNUE goal* row and the 2.8.0 release
line's strength sentence; (3) is the *Conversion* row; (4) the *Speed*
row. None decides anything.

## The command (dry-run 2026-11-21)

```
pwsh -NoProfile -File D:\code\rarog\analysis\artifacts\d714-meter-2026-11-21\run_m90.ps1
```

`run_m90.ps1` (ignored, in `analysis/artifacts/d714-meter-2026-11-21/`),
SHA-256 `9119071e0cde4b3e024257d6e1c678ed7245397a41f69a19d71c8bb90f8e507c`,
checks every binary's hash, then runs the gauntlet, retrying a start once a
minute only while the harness refuses a busy host before any game; Ctrl+C
stops it at a committed boundary and running it again resumes it. Its dry
run passed every policy check on 2026-11-21 (manifest
`colosseum_gauntlet_rar-m90-meter_20261121_175432`): 7,800 games, thirteen
pairs of 600, the thirteen pins and every option above in the resolved
plan.

## PRE-REGISTERED PREDICTION (frozen before any game)

**Basis.** RAR-S136: the kernel +67.0 ± 8.1 Elo at `10+0.1` and +73.3 ±
8.1 at four threads over `d7classic`, which is 2.7.0 with the null move no
longer counted (RAR-S125, a neutral change); RAR-S134 +63.8 ± 9.1 at
`3+0.03` before the finishing fit, which added +7.8 ± 9.1. A gain over a
sibling usually shrinks against foreign engines.

1. **Rating:** 3,505, 80% band 3,480 to 3,530; so the gap to Stockfish
   16 HCE about 100 (75 to 125).
2. **Head to head against Stockfish 16 HCE:** −105 Elo, 80% band −135 to
   −75 (2.7.0: −164.8).
3. **Against 2.7.0 head to head:** +65, 80% band +45 to +85; the field
   gain (rating less 3,444.6) +60, 80% band +35 to +85.
4. **Place:** above Ethereal 14.40 HCE and Houdini 6.03, below Berserk and
   Stockfish 11, probability 0.6.
5. **Conversion:** fewer than 2.7.0's 18.6 thrown a thousand, probability
   0.6 (RAR-S126 saw won endings converted better by the kernel).
6. **Faults:** no time loss by the kernel, 0.9.

## Results (read 2026-11-21)

Played 2026-11-21 17:41 to 19:08 UTC on the match host in one attempt, not
interrupted; the host read 3.7% busy at the start, no waiver. 7,800 of 7,800
games scored, status `completed` (manifest
`colosseum_gauntlet_rar-m90-meter_20261121_184125`; `result.json`
`9d6e02c1…66c5`, `results.txt` `16714a40…9f1f`, `games.pgn`
`c658e545…4637`). Every game ended by the rules: 5,205 mates, 1,565
repetitions, 610 insufficient material, 366 fifty-move draws, 54
stalemates. **No fault and no time loss by any engine**, so nothing is
removed.

**1. The meter.** The kernel rates **3,508.1 ± 8.2** [3,499.9, 3,516.4]
with the field pinned, so the gap to Stockfish 16 HCE is **96.6**. Head to
head **−100.6** [−119.4, −82.3] (pentanomial; the trinomial interval is
[−123.3, −78.6]). The interval is the kernel's alone: each pin carries
RAR-M89's own, about ±7. Every pair, from the kernel's side, with
Colosseum's pentanomial 95% interval:

| Opponent | W-D-L | Elo |
|---|---|---|
| Stockfish 16 HCE | 103-225-272 | −100.6 [−119.4, −82.3] |
| Berserk 4.7.0 HCE | 168-204-228 | −34.9 [−54.6, −15.3] |
| Stockfish 11 | 151-216-233 | −47.8 [−67.3, −28.6] |
| Ethereal 14.40 HCE | 236-209-155 | +47.2 [+26.4, +68.4] |
| Houdini 6.03 | 207-223-170 | +21.5 [+1.7, +41.4] |
| Rarog 2.7.0 | 229-275-96 | +78.3 [+60.6, +96.4] |
| Dragon 3.3 HCE | 276-196-128 | +87.5 [+66.8, +108.8] |
| Basilisk B.11.1 head | 289-210-101 | +112.7 [+93.1, +132.9] |
| Ethereal 13.07 | 301-186-113 | +112.7 [+92.0, +134.1] |
| Fire 8.3 | 301-188-111 | +113.9 [+93.6, +135.1] |
| Rarog 2.5.0 | 368-193-39 | +214.0 [+192.9, +236.6] |
| rofChade 2.3 | 401-154-45 | +237.2 [+214.0, +262.4] |
| Koivisto 4.83 | 457-116-27 | +313.0 [+285.8, +343.6] |

Fourth of fourteen: behind Stockfish 16 HCE, Berserk 4.7.0 HCE (35.5
above) and Stockfish 11 (34.3 above), ahead of Ethereal 14.40 HCE (by 41.2)
and Houdini 6.03 (by 46.4). Counted one version a family among the
hand-crafted engines, Rarog is third, after Stockfish and Berserk.

**2. The gain over the release:** **+63.5** in the field (3,508.1 less
3,444.6) and **+78.3** [+60.6, +96.4] head to head.

**3. Conversion** (`conversion_audit.py`, output `conversion_m90.log` and
`.json` in `analysis/artifacts/d714-meter-2026-11-21/`), per 1,000 games
after a persistent piece-up, Poisson 95%:

| Engine and games | Draws | Losses | Thrown away |
|---|---|---|---|
| `d713kern`, RAR-M90 | **20.6** (161) [17.6, 24.1] | **2.8** (22) [1.8, 4.3] | 23.5 (183) |
| 2.7.0, RAR-M89 | 15.4 (120) [12.8, 18.4] | 3.2 (25) [2.1, 4.7] | 18.6 (145) |

Most of the rise is in the ending: 131 of the kernel's draws against 82.
The kernel also wins more, 3,487 games against 2.7.0's 3,103, so it reaches
more piece-up positions; the tool counts a game, not a chance, so a worse
rate a chance is not shown. It is not resolved. RAR-S126 saw the unfitted
kernel convert won endings better than the classic, a comparison of
another kind.

**4. Speed in games** (`results.txt`): the kernel 1.92 M nodes a second,
depth 16.17 mean (16.33 without mate scores, median 14), 126,201 nodes and
65.9 ms a move; 2.7.0 in the same games 1.96 M, 14.46 (14.14, median 13),
128,838 nodes and 65.6 ms. The kernel searches 2.5% fewer nodes a second
than the release's binary and goes 2.2 plies deeper at the same time a
move. RAR-S136 read it level with `d7classic`, a local build of the
classic, at `3+0.03` (1.83 M against 1.82 M); the release asset is another
build, so the 2.5% is not attributed. The kernel's `harness_hold_ms` (p999
260, max 1,405 ms, no move over the margin) is the interrupted iteration,
as in RAR-S136.

### Uses

1. and 2. fill GUIDE's *Pre-NNUE goal* row and PLAN section 1's meter, and
   are the strength sentence's source for the 2.8.0 release line.
3. fills the *Conversion* row, as unresolved.
4. fills the *Speed* row.

None decides anything; with them D.7.14 is done.

### Calibration

- Rating 3,508.1 against 3,505 (80% band 3,480 to 3,530): a hit; the gap
  96.6 against about 100.
- Against Stockfish 16 HCE −100.6 against −105 (−135 to −75): a hit.
- Against 2.7.0 head to head +78.3 against +65 (+45 to +85): inside the
  band, 13 high in magnitude. The field gain +63.5 against +60: a hit. The
  basis held: the gain against foreign engines is smaller than against
  the sibling.
- Place above Ethereal 14.40 HCE and Houdini and below Berserk and
  Stockfish 11 (0.6): a hit.
- Conversion under 18.6 thrown a thousand (0.6): **a miss**, 23.5. The
  assumption that failed: that RAR-S126's better endings against the
  classic would lower the meter's rate a game. The rate a game rose with
  the wins, and the instrument cannot separate a rate from its exposure,
  so the miss is in the instrument's reading; whether it is also in the
  mechanism is not known.
- No time loss by the kernel (0.9): none.

Evidence layer: ratings on RAR-M89's scale with the field pinned, and
head-to-head Elo, at `3+0.03` 1T; conversion counts and nodes a second are
not Elo.
