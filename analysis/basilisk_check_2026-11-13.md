# Basilisk check, 2026-11-13: what Basilisk learned since the cross-check, read against Rarog

Placed after the 2.7.0 release and before D.7 (maintainer decision
2026-11-13). Research only: nothing was built or ported. Basilisk read at
`master` `d76a98c` and branch `b6-perft-pins` `fdb35fa` (its B.4 to B.6 work,
2026-11-02 to 11-13); Rarog at `basilisk-check` (2.8.0-dev, engine source as
released in 2.7.0); Reckless at `0756957`. Basilisk's results are what its
records claim; every Rarog statement was read in source, and the ones that
decide a card below were checked a second time by hand. The previous pass is
`analysis/basilisk_crosscheck_2026-11-02.md`; what Basilisk took from Rarog
(`analysis/rarog_import_2026-11-11.md` there) is left out.

**Outcome.** Two new research cards (D.6.29, D.6.30), new evidence for D.6.28,
BOARD-9 and H.4, and a list of departures for D.7.0's inventory. No new
correctness defect from Basilisk's own audit applies to Rarog unowned, and no
Basilisk gain is evidence for a single piece in Rarog: B.4 (+31.7 Elo) and
B.5 (+15.6) were gated as clusters on a kernel Rarog does not share.

## 1. Basilisk's rebuilt quiescence and root (B.4, B.5)

| Basilisk mechanism (record, result) | Rarog (source) | Where it goes |
|---|---|---|
| SEE scores a promoting recapture (BAS-S48, kept as a repair) | Has it, `board.rs:1489-1495` | nothing owed |
| SEE follows pins opened in the exchange (BAS-S49, kept) | Has it, `board.rs:1462-1483` | nothing owed |
| Fail-high stop at twice the optimum (BAS-R10, 2× over 1× +9.3 ± 4.4) | Has it at 2.0 (`params.rs:273`); Basilisk fires only when the failing move is the previous best | D.6.20 owns the hole |
| Delta rule deleted (BAS-S52 (c), +15.0 ± 8.0 on its fitted arm) | Kept above 8 men, `node.rs:2683-2712`; Reckless has none | D.7.0 |
| Recaptures exempt from futility and the alpha-relative SEE | Exempt from the count rule only | D.7.0 |
| Evasions ordered with history; losing capture evasions pruned once one holds | Quiet evasions skipped once one is not lost (`node.rs:2763`); losing captures searched | D.7.0 |
| No quiescence TT cutoff at PV nodes; fail-soft tail stores an upper bound | Cuts at every qnode (`node.rs:2619`), fail-hard tail | D.7.0 (fail-soft lost before) |
| Window centred on an effort-weighted average, width from the mean square | Centres on the last score ± 12 (`mod.rs:322-333`); `average_score` and `mean_squared_score` are computed per root move (`mod.rs:255-264`) and read only by tests | D.6.13, D.7.0 |
| One chained negative extension per line | Lines bounded at twice the iteration depth (D.6.7) | nothing owed |
| Ply cap: draw test first, a draw in check at the cap | The cap returns the evaluation before the draw test, also in check (`node.rs:590` before `:602`; `:2561` before `:2570`) | D.7.0 (Reckless's order); RAR-S121 found no search reaching the cap at `3+0.03` |
| **Stalemate test in quiescence** (B4Stalemate, inside the cluster, no read of its own) | **Lacks it**: out of check with no capture, quiescence never asks whether a legal move exists; Reckless lacks it too, so D.7 does not supply it | **D.6.29** |
| **A proven mate stands against an iteration that lost it** | **Lacks it**: each completed iteration overwrites `best_score` and the move (`mod.rs:1074-1080`); the mate watch stops the search and keeps nothing | **D.6.30** |

## 2. Basilisk's audit, evaluation and speed items

| Basilisk finding (record, result) | Rarog (source) | Where it goes |
|---|---|---|
| Draw verdicts take no correction (BAS-S44, +7.4 ± 6.9) | Read side done by D.6.15 (`correction.rs:180`); the training side still trains at verdict nodes and in exclusion searches | D.6.23 already holds the producer guard |
| Coloured histories (BAS-S46, +4.7 ± 3.0) | Already coloured, `history.rs:12-16` | nothing owed |
| Multi-cut correction overflow, history widths, `ponderhit` clock (B.6.4 D1, W1, D2) | Do not apply: no such bonus, capped bonuses, limits computed once at the root | nothing owed |
| TT-cutoff history cap (BAS-S59, not adopted) | Already capped, `node.rs:731` | nothing owed |
| Kings in the non-pawn correction key (BAS-S59 +11.6 ± 9.3 alone; BAS-S60 with razoring +2.2 ± 4.2 over 10,000 games, undecided) | Kings left out (`board.rs:1332-1339`); Reckless includes them | BOARD-9, D.7.0 |
| Fifty-move bucket in the TT key (BAS-S59, −4.5 ± 8.4, not adopted) | Raw key, as Reckless | BOARD-9 |
| Razoring in Stockfish's form (BAS-S59 +10.6 ± 9.4, not held jointly) | Rarog's own form | D.7 |
| An exclusion search's fail-high raises the parent's cutoff count (B.6.4 H1) | Same (`node.rs:2362`), as Reckless | D.7.0 |
| Zero-weight terms not computed (BAS-P25, +2.35% NPS) | Census not run | D.8.20 |
| Material table and one-walk material and PST (BAS-P29) | Has both | nothing owed |
| SEE attackers computed once (BAS-P29) | Screened at −0.31% (`analysis/c12_speed_pass_2026-10-17.md`) | negative prior |
| Quiet-stage threat maps built lazily (part of BAS-P29's +4%) | Built eagerly, slider loops included (`movepick.rs:500-516`) | H.4 |

## 3. The clock: where 2.7.0's unspent time goes (D.6.28)

RAR-M87 read 2.7.0 7.6 Elo below 2.6.0 against its field and found the head
ending its games with 616 ms left against 379. The mechanism, read and
computed (`analysis/artifacts/m87-release-read/tm_alloc.py`): below one
second, `sudden_death_horizon` (`time.rs:101-104`) sets the horizon to
`clock_s × TmHorizonPerSec` (50, range 20..=100), where 2.6.0 and Basilisk
keep 50 moves at every clock. At `3+0.03` with 10 ms overhead the optimum
is unchanged at and above 1,000 ms, then falls to 0.84 of 2.6.0's at 750 ms,
0.65 at 500 ms and 0.43 at 300 ms; the clock stops falling where the optimum
meets the increment, 416–575 ms for the head against 154 ms for 2.6.0
(Reckless's base, 428–487 ms, keeps a similar reserve). The measured cut is
smaller than the formula's (0.79 in moves 31–60), because the stop is checked
between iterations.

The rule was added against forfeits that RAR-R14 sized at about 0.01 Elo
(four in 226,180 games, OS holds at clocks under 170 ms); RAR-R17 read the
horizon at 100 against 53 head to head only (−0.9 ± 8.0), never against other
engines. In RAR-M87's field leg the harness margin was 2,000 ms, so there the
reserve guarded against nothing.

**Zero-game split of RAR-M87's games**
(`analysis/artifacts/m87-release-read/m87_clock_band.py`): each Rarog game
flagged when Rarog moved under the band with a non-mate |s| < 150.

| Band | Head − 2.6.0 in the band | Outside it |
|---|---|---|
| under 1,000 ms | −18.7 ± 8.5 Elo (6,106 and 5,525 games) | +28.4 ± 14.5 |
| under 500 ms | −33.8 ± 10.1 (3,723 and 3,638) | +9.0 ± 10.7 |
| head to head, 1,000 ms | −29.6 ± 15.9 | +75.3 ± 26.2 |

The split is biased (longer games run low, and selection is per build), but
the head enters the band more often, which by itself would flatter the head
in both buckets. The check could have refuted "the reserve costs Elo against
other engines" and did not; it localises the gap, it does not measure the
rule. Evidence layer: game outcomes by a post-hoc split, not a gate.

**Candidate (narrowest):** confine the contraction to where the forfeits
were, a horizon near 167 per second (the range widened past 100), read
against the field rather than head to head, with the forfeit count as the
invariant. Interactions: everything that reads the optimum or the maximum
(the soft target and its factors, the fail-high stop at twice the target,
the DTZ ranking cap, the tablebase extension's latest start, the four-thread
stop vote), and RAR-R15's fit, made under the shorter horizon.

## 4. Not checked

Rarog's TT replacement rules against Basilisk's four TT items (R4a–R4d) and
whether Rarog's null verification ever runs as quiescence: both kernel shape,
for D.7.0.
