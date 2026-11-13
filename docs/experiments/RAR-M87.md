# RAR-M87 — The 2.7.0 no-regression read: the release head and Rarog 2.6.0 against one field near their level, and head to head

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-13, before any game. Prepared and run by
  the agent, delegated by the maintainer the same day ("prepare and run it
  yourself this time").
- **Kind:** the registered read that the 2.7.0 cut needs (PLAN E.3, maintainer
  decision 2026-11-13: "no loss of playing strength against 2.6.0, shown by a
  registered match before the cut"). It accepts no engine change.
- Disposition: observation — the read finds a **loss**: G −7.6 Elo
  [−14.4, −0.7] against the field; head to head +10.1 ± 8.7. The release
  decision is the maintainer's, made after the result (see *Result*).
- **Why a field, not only head to head:** RAR-M84 (2026-11-08, unregistered)
  had the head level with 2.6.0 head to head (−4.8 ± 14.1) and 13.6 ± 9.9
  lower against a shared field, most of it far weaker than either build
  (+150 to +400 Elo), where a game says little. A field the builds score near
  50% against resolves the same gap in fewer games.

## Arms

| Arm | Binary | SHA-256 | Bench |
|---|---|---|---|
| Release head | `tools/test_engines/rarog-d68cand-pext-pgo.exe`, `id name` `dev-20261113-ba1d2cde`, PGO pext, rustc 1.100.0 | `6d0d4313568771f6848da47d2056c2e77f28c250208e36a4642ef767d8b4ad3d` | 9,077,016 / EBF 2.481 |
| Rarog 2.6.0 | `D:\chess\engines\rarog\rarog-v2.6.0-windows-pext-pgo.exe`, the published pext asset | `274eab9728432d25ee1e4342ec96f860390ad5ac76ca8c7052407b3bfe011ae1` | 13,987,223 / EBF 2.565 |

The head binary is RAR-S124's candidate, built from `ba1d2cd`. The branch tip
since then (`5d77934`) changes tooling and documents only: `git diff --stat
ba1d2cd HEAD -- src Cargo.toml Cargo.lock build.rs rust-toolchain.toml xtask`
is empty, so it is the release's engine.

## The field

One thread each, set under each engine's own option; Hash 64 for every
engine; every overhead option at 10 ms (maintainer decision 2026-10-22). The
field brackets Rarog 2.6.0's held 3,475 (`reference-engine-library` ratings,
the starting ratings only):

| Engine | Binary (`D:\chess\engines\`) | SHA-256 | Options |
|---|---|---|---|
| Stockfish 10 | `stockfish\stockfish-10-x64-bmi2.exe` | `25108b8c3db1f9e54b06722ba0049fc7ba3cd425b401f21db818e00214aba0f4` | `Move Overhead=10` |
| Komodo 14.1 | `komodo\komodo-14.1-64bit-bmi2.exe` | `0d32bcbd0d4276219c74f1ca0aaaaaa004248e2c6f31c74a53f5b7697b4a77fd` | `Overhead ms=10` |
| Fire 8.3 | `Fire_8.3_x64_bmi2.exe` | `c2967a3df8824ef65b8e1fd8cbdc92e5dc9b2eceef48a3ae34e1661adfaa1d9a` | none (no overhead option) |
| Houdini 5 Pro | `houdini\Houdini_5_Pro_pext_x64.exe` | `496260e86f53bc03a4eb3af6e041a1d47e4e331c05c7a9e7ce3498aada6b0bcd` | `Move Overhead=10` |
| Ethereal 13.07 | `Ethereal13.07-x64-pext-avx2.exe` | `61f0270f013718c9da0d326d9fc68ab63566ce45ef37798f9be7198ed870abe3` | `MoveOverhead=10` |

Houdini 6.03 (`e5c0fdc5…`, RAR-M75's binary) and Komodo 13.3 were the first
choices; on 2026-11-13 both exit before answering `uci` on this host, by the
harness's probe and from their own directories, so they were left out before
any game. Ethereal 12.75 is not on disk; 13.07 is RAR-M84's.

## The two legs (frozen)

1. **The field leg, which decides.** One Colosseum tournament through
   `colosseum.ps1 -Mode gauntlet`: `--format gauntlet --seeds 2
   --games-per-pair 2 --cycles 1000`, the two Rarog builds as the seeds.
   Colosseum's resolved plan is **20,000 games**: each seed plays each
   opponent 2,000 times, colour-reversed pairs, and the seeds do not meet.
   `3+0.03`, harness margin 2,000 ms (RAR-M75's, for the foreign engines),
   UHO_Lichess_4852_v1 in random order at seed 20261114, no adjudication,
   tablebases off, 14 games at a time, the pinned Colosseum CLI 0.5.0.
   Driver `analysis/artifacts/m87-release-read/run_m87.ps1`; its dry run
   passed every policy check, `dry-run.json` SHA-256 `a2eaa98b…401d`.
2. **The head-to-head leg, which decides nothing.** `colosseum.ps1 -Mode
   match`, the head against 2.6.0, **2,000 games**, `3+0.03`, Rarog's own
   20 ms margin, Hash 64, one thread, seed 20261115,
   `tools/results/rar-m87-h2h`. Its dry run passed every policy check,
   `ea6c1a38…cc55`. Played after the field leg.

**Stop rule:** each leg plays to its end, no early stop, no extension, no
re-run on the same binaries. A leg voided by a fault rule is repeated, not
read.

## The deciding quantity and the rule (frozen)

For each opponent j, Colosseum's own Elo of each seed against it, with its 95%
interval, from the run record (`results`, every pair); the agent reads these
and computes nothing Colosseum reports. Then, computed:

- G = the mean over the five opponents of (Elo of the head against j − Elo of
  2.6.0 against j): the head's strength against the field relative to 2.6.0.
- SE_G = √(Σ_j (s_head,j² + s_2.6.0,j²)) / 5, each s the pair interval's half
  width over 1.96; the two seeds' games are independent.

**The rule:** the read finds **no loss** when G ≥ −1.645 · SE_G, and a
**loss** otherwise: a one-sided 5% false alarm at a true zero, AGENTS' harm
convention. At the expected SE_G of about 3.5 Elo the line is near −5.8; it
then catches a true −5 with probability about 0.41, −10 with 0.88 and −15 with
0.996. **"No loss" therefore means no loss beyond roughly −10 Elo against this
field.** The head-to-head leg is reported with its interval and decides
nothing.

**Fault rule** (RAR-M74/M75's): any fault of either Rarog build (a crash, an
illegal move, a time loss) voids the field leg; an opponent's faults under 1%
of its games are reported and scored as the harness scored them; an opponent
over 1% leaves G for both seeds, and G is read over the rest. `colosseum.ps1`'s
own zero-tolerance exit does not void a run whose `result.json` exists; the
faults are read here.

## PRE-REGISTERED PREDICTION (frozen 2026-11-13, before any game)

- **G:** **−4 Elo**, 80% band **[−15, +7]**. P(no loss by the rule) 0.65.
- **Head to head:** **0 Elo**, 80% band [−12, +12].
- **Basis:** RAR-M84 read the field gap at −13.6 ± 9.9 and head to head
  −4.8 ± 14.1 on the 2026-11-08 head. Since then, each against its own
  predecessor: D.6.2 +12.2 nElo (about +7 Elo), D.6.1 −4.9, D.6.7 −1.0 and
  D.6.8 −6.2 nElo (about −2.8, −0.6 and −3.6 Elo), and rustc 1.100.0's −0.24%
  NPS (about −0.5 Elo): about 0 net. M84's field gap was read against
  engines both builds beat by 80% to 90%, where the scale is compressed and
  noisy, so its −13.6 is weak evidence; the head to head is the steadier
  anchor. Between the two, a small negative centre with a wide band.
- **Most likely failure mode:** a real field deficit of 5 to 10 Elo that head
  to head does not show, the pattern RAR-M84 suggested: a clock or tree
  change that costs against other engines and not against itself.
- **Evidence layer:** games against a fixed field; ratings are on this run's
  own scale. Nothing here maps to CCRL.

## Dispositions (frozen)

- **No loss:** the 2.7.0 cut's condition is met (PLAN E.3); the release
  proceeds by PROCESS *Release*.
- **Loss:** the cut does not proceed. The record names each opponent's gap,
  and the maintainer decides what follows; no rescue inside this read.

## Result

Played 2026-11-13 on an idle host. The field leg ran 14:07–17:40, 20,000 of
20,000 games scored, status `completed` (`result.json` SHA-256 `23860fb3…70f5`,
`games.pgn` `2f1c3875…4197`). The head-to-head leg ran 17:41–18:02, 2,000 games,
`completed` (`result.json` `434c3825…2e01`, `games.pgn` `b52e6e1e…5404`); it is
the frozen command, started from the maintainer's terminal by
`analysis/artifacts/m87-release-read/run_h2h.ps1` so they could watch it, after
the chain was cut to end at the field leg. **Faults: none** in either leg, no
time losses, no infrastructure faults, so the fault rule removes nothing.

**The field leg**, Colosseum's per-pair Elo from `results.txt` with its
pentanomial 95% interval:

| Opponent | Head | 2.6.0 | Head − 2.6.0 |
|---|---|---|---|
| Stockfish 10 | −81.2 [−92.2, −70.3] | −78.8 [−89.8, −67.9] | −2.4 |
| Komodo 14.1 | −13.2 [−23.9, −2.5] | −8.3 [−18.6, +1.9] | −4.9 |
| Fire 8.3 | +52.0 [+41.0, +63.0] | +71.0 [+60.8, +81.3] | −19.0 |
| Houdini 5 | +23.1 [+12.2, +34.2] | +32.4 [+21.6, +43.3] | −9.3 |
| Ethereal 13.07 | +42.8 [+31.3, +54.4] | +45.1 [+34.1, +56.1] | −2.3 |

**G = −7.6 Elo**, SE_G 3.50, the line −1.645 · SE_G = −5.76, 95% interval
[−14.4, −0.7]. **The read finds a loss.** The registration did not say which
of Colosseum's two pair intervals to use; with the trinomial ones SE_G is
3.94, the line −6.49 and the interval [−15.3, +0.2], also a loss. All five
gaps are negative and they do not differ beyond chance (χ² 3.2 on 4 degrees of
freedom); Fire's −19.0 is the largest. The pooled standings agree: 2.6.0
3,470.7 ± 6.9, the head 3,463.1 ± 6.9.

**Head to head**, which decides nothing: the head **+10.1 ± 8.7 Elo**, nElo
+17.7 ± 15.2, 521/1016/463, pentanomial [18, 224, 460, 278, 20].

**Calibration.** G was predicted at −4, 80% band [−15, +7], P(no loss) 0.65:
the result lies inside the band, the centre 3.6 low; the miss is in magnitude
and confidence, not sign. Head to head was predicted at 0 [−12, +12]; +10.1
lies inside. The failure mode named most likely, a field deficit that head to
head does not show, is what was read: the two legs differ by about 18 Elo,
about three standard errors.

**Clock use, a lead and not a cause** (evidence layer: time per move, not Elo;
`analysis/artifacts/m87-release-read/m87_time_use.py`, output beside it, the
clock rebuilt from each move's `t=` at 3,000 + 30 ms). The head thinks 65.5 ms
a move against 69.0 (0.949). On mate-scored moves 6.5 against 18.7, the stop on
a proved mate; on the rest 69.9 against 72.6 (0.962): more in its first ten
moves (113.0 against 102.4), level in moves 11–30, less in moves 31–60 (44.4
against 56.0, 0.79) and with under a second on its clock (42.2 against 49.1).
It ends its games with 616 ms left against 379 (medians 567 and 263), about
5% of a game's budget unspent. RAR-M84 read the same, 4.2% less time on
non-mate moves. The head-to-head games show the same profile (non-mate 0.960,
706 against 491 ms left; medians 625 and 280) while the head scored +10.1 there, so
the clock profile alone does not explain the field gap. It is a candidate
mechanism for a research card, untested.

**Disposition: loss.** By the frozen rule the 2.7.0 cut does not proceed on
this read. The maintainer decides what follows after seeing the result, a
one-off exception for 2.7.0 recorded as such (maintainer, 2026-11-13: "this
time I will make decision after we see how bad it is"); the decision is
recorded here when made.
