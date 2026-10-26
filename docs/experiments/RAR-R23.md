# RAR-R23 — D.1.6.3: the confined cascade repair, the known-win offset alone, read at equal speed with a harm rule — REGISTERED 2026-10-24, before any game; READ 2026-10-24: FAIL, harm at `10+0.1`

Indexed under *4. Root search, time management and SMP* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-24 before any game. The zero-game
  reads below are the agent's; the two matches are the maintainer's
  commands.
- **Kind:** D.1.6's re-scoped repair (RAR-R22, *Research resumed at the
  level above the mechanism*, item 4), under AGENTS' repair cases 1 and 2.
  Case 1: a static differential confines the change to its domain. Case 2:
  a 2,000-game equal-speed harm read with a frozen threshold, plus the
  `10+0.1` harm read the maintainer requires for any fallback (RAR-R17,
  2026-10-23).
- **Hypothesis:** the known-win offset, on its own, ends the won-ending
  stalls the probe measures and is no weaker than the defaults at either
  control. The offset is the evaluation half of RAR-R19's cluster: it values
  a bare-king ending the winner can mate in as a known win. Neither the
  window term nor the line budget nor the freeze is in this arm, and no
  clock stop.
- **What changed from the plan RAR-R22 left.** That plan paired the offset
  with a known-win window floor, `AspKnownWinWindow` (`3f4dcf4`, tune-only,
  0 = off). The zero-game reads below refute the floor's premise, so the
  floor is not in the arm and stays at 0.
- **Build:** `tools/test_engines/rarog-d16inst3-tune.exe`, built from the
  clean tree at **`3f4dcf4`**. That revision is `d1`'s head with the floor
  switch; its engine inputs differ from `2f17467` only by that commit.
  SHA-256 `A5A3CF06E386E3BA…F6C01F01E6`, `rustc 1.99.1`, `--features tune`,
  bench 13,987,223 at the defaults. With `MopupKnownWin` 5826 the same
  binary benches **16,175,922 / EBF 2.580**. Both sides run this binary:
  arm A sets `MopupKnownWin=5826` (RAR-R21's fitted value), arm B sets
  nothing. d1's clock pieces are in both arms, so they cancel.

## The zero-game reads (2026-10-24, agent-run)

Scripts and logs: `analysis/artifacts/d16-cascade-2026-10-23/confined-2026-10-24/`
(`reads_r23.ps1`, `fresh_diff.py`, `kw_entry.py`, `trace_fen.py`).

**1. The probe** (`tm_rec_probe_d16.py`, no clock stop; idle check 2.6%
CPU over 20 s, no engine or build process). The figure is the time of the
move as a multiple of its optimum; MAX means the move reached the hard
maximum.

| Clock, position | defaults | offset alone | offset + floor 1000 / 2000 / 3000 | offset + window term 14707 |
|---|---|---|---|---|
| `60000+600` rec1 | 1.05 | 0.60 | 0.60 / 0.59 / 0.59 | 1.77 |
| `60000+600` rec3 | 0.46 | 2.47 | 2.47 / 2.48 / 2.51 | **MAX** |
| `180000+2000` rec1 | 2.85 | 0.61 | 0.61 / 0.61 / 0.61 | 1.04 |
| `180000+2000` rec3 | **MAX** | 0.76 | 0.78 / 0.77 / 0.77 | **MAX** |

The offset alone keeps every row under the maximum. rec3 at `60000+600`
runs past its optimum: the defaults spend 0.46 × there, the offset 2.47 ×.
At every W the floor reproduces the offset's node counts exactly
(11,222,546; 47,415,502; 37,740,501). The old pair, the offset with the
squared window, runs rec3 to the maximum at both clocks on this build. In
RAR-R20 it did not, because the budget and the freeze were with it.

**2. The node-indexed traces** (200 M nodes, one thread; `kw_entry.py`;
the band edge is 5826 − 2000 = 3826 cp).

- **rec1, offset alone.** The root score climbs from 750 cp to 3,314 cp
  over depths 10 to 16. Each iteration walks 5 to 15 windows; depths 12 to
  14 have seven or eight fail-highs each, and depth 16 does not finish in
  the budget. No centre ever reaches 3,826, so the floor never fires.
- **rec1 at the defaults.** The same climb comes six plies later: nine
  re-searches and 135 M nodes at depth 19.
- **rec1 with the window term.** Each walk shrinks to 2 to 4 windows,
  because the term already widens the window at 1,000 to 3,000 cp. The
  known-win score reaches the root at depth 17.
- **rec3.** No configuration puts a centre in the band.
- **The floor's own case.** On rec1 after Black's rook is gone
  (`8/5R2/8/2k1PB2/8/4K3/8/8 b - - 0 90`), the root starts in the band. The
  floor cuts the bound lines from 25 to 1 or 2 and the nodes before the
  first mate score from 27,733 to 7,643. The mate in six arrives at depth 7
  either way, within 8 ms.

**3. The fresh-process differential** (`fresh_diff.py`: a fresh process per
position; the bench suite at depth 13, `won_endings_suite.epd` at depth
14).

- **The offset against the defaults.** It changes 13 of 40 bench positions,
  every one with 19 pieces or fewer (6 to 19 pieces: `KRPPP v KRPPP`,
  `KQRPPPPPP v KQRPPPPPPP`, `KQR v KRPPP`, …), and both won endings. That
  is the RAR-R22 addendum's result, reproduced on this build. The offset is
  added only where the losing side has a bare king
  (`src/eval/endgame/mop_up.rs`), so a position changes only through a
  subtree that reaches one.
- **The floor at 1000, 2000 or 3000 against the offset alone.** It changes
  0 of 40 bench positions and 0 of 2 won endings.
- **The window term against the offset alone.** It changes 32 of 40.
- **One cost the probe does not show.** rec1 at depth 14 takes 37.7 M
  nodes with the offset against 1.1 M without.

**Reading.** The premise of the floor was a jump of about 5,800 cp as the
offset enters the horizon. That is false here. The cascade that remains
with the offset is a fixed-width window walking up a score that climbs
below the band, in the score range of ordinary won positions. The only
device that shortened it, the window term, acts in that range and is
global. A floor confined to the band is inert wherever a stall was
measured, and saves milliseconds where it does act. **Recommendation:
W = 0, the floor dropped**, and the confined repair is the offset alone.
The floor does not support the form "a floor below the band". Any wider
window below the band is the window term again, which RAR-R22's addendum
put outside D.1.6.

## The reads (frozen)

- **`stc`:** Colosseum `match`, `-CategoricalTuneBuild`, 2,000 games at
  `3+0.03`, 20 ms margin, Hash 64, one thread, the wrapper's slot count, the
  UHO book in random order, no adjudication, seed 20261071, run directory
  `tools/results/rar-r23-stc`.
- **`ltc`:** the same arms, `match-fixed-ltc`, 1,000 games at `10+0.1`,
  seed 20261072, `tools/results/rar-r23-ltc`.
- **Commands:** `run_r23.ps1 -Read stc|ltc` beside the scripts above. Both
  dry-ran clean on 2026-10-24 (host 2–3%). The pinned runner is
  colosseum-cli 0.4.0. The resolved configuration puts
  `MopupKnownWin=5826` on engine A only.
- **The harm rules**, from the manifest's nElo:
  - `stc`: harm if the estimate is below **−13 nElo**. At 2,000 games the
    standard error is about 7.8 nElo (AGENTS' case 2). This catches a true
    −10 with probability 0.36 and −20 with 0.82.
  - `ltc`: harm if the estimate is below **−18 nElo**. At 1,000 games the
    standard error is about 11.0 nElo, and a 5% false alarm at a true zero
    sits at −18. This catches a true −10 with probability 0.23, −20 with
    0.57 and −30 with 0.86.
  - "No harm" therefore means no harm beyond roughly −20 nElo at `3+0.03`
    and −30 at `10+0.1`.
- **Correctness reads**, done above: the probe has no row at the maximum,
  and the differential confines the change to positions with 19 pieces or
  fewer whose trees reach a bare king.
- **Pass:** neither read below its threshold. **Fail:** either read below
  its threshold. Neither read's interval accepts a gain.

## Dispositions

- **On a pass, D.1.6.2 lands on `d1`, engine and docs in separate commits:**
  - `MopupKnownWin` defaults to 5826;
  - the fingerprint is re-declared: 16,175,922 / EBF 2.580 on this build,
    with the landed production build's own bench recorded then;
  - `TmFailHighTarget` and its test are deleted;
  - the unused instruments are removed: `AspFailHighReduce`,
    `AspFailHighRecover`, `AspRecentre`, `QsFailSoft`, `CoreLmrDeeperCap`,
    `CoreExtensionBudgetEighths`, `AspDeltaSquareDiv`, `CoreLmrDeeperBudget`,
    `TmDepthFreeze` and `AspKnownWinWindow`. A coordinate whose default is
    today's behaviour returns to its constant;
  - the changelog gets an entry along the lines of "won endings against a
    bare king are recognised as wins, so the engine no longer runs to its
    time limit converting them";
  - D.1.2.1's cascade half ticks on the probe; D.1.6 and D.1.6.3 close.
- **On a fail, nothing lands.** D.1.6 returns to RESEARCH with the
  estimate, and the maintainer chooses one of two closes:
  - D.1.6 `NO_CHANGE`: the stalls are accepted; the hold already records
    no measured cost at `3+0.03` and `10+0.1`, and the instruments are
    removed in a cleanup;
  - D.1.6's registered fallback, the clock stop at twice the target, read
    as RAR-R18 read it. That contradicts the addendum's "no clock stop in
    D.1.6", so it needs that decision.
- **Either way the window walk below the band stays out of D.1.6.** It is
  a search question: a fail-hard root and a 12 cp first window walking a
  climbing score, which is RAR-R22 item 3's "seven root fail-highs a
  search".

## PRE-REGISTERED PREDICTION (frozen before exposure)

- **`stc`:** −2 ± 10 nElo (about −1 ± 6 Elo). P(pass) 0.80.
  - Where the cost would come from: the change reaches only positions with
    19 pieces or fewer whose trees reach a bare king, and most such games
    are already decided. Rook endings like rec3 spend more (0.46 × to
    2.47 × at `60000+600`), and rec1-type wins search shallower per node.
  - The repair's gain at this clock is small. RAR-R20's 200-game read of
    the offset with its window was +3.5 ± 27.3 Elo.
- **`ltc`:** 0 ± 14 nElo, P(pass) 0.85. The stalls cost more at a longer
  clock, and so does the shallower search; no sign is preferred.
- **Time a move:** within ±1% of the defaults overall, from the PGN
  comments, P 0.8. The change is confined, so the openings are untouched.
- **Most likely miss:** a small negative at `stc` from the deeper per-node
  cost in rook endings, which a 2,000-game read cannot separate from zero.

## Results (2026-10-24, maintainer-run, after RAR-R24)

Both reads completed with 0 faults and 0 time losses. The figures are the
runner's final report lines (colosseum-cli 0.4.0). A is the offset, B the
defaults.

| Read | Elo | nElo | Harm threshold | W/D/L (A) | Pentanomial | Host at launch |
|---|---|---|---|---|---|---|
| `stc`, 2,000 games | **−2.3 ± 7.7** | **−4.4 ± 15.2** | −13 | 478/1031/491 | 15, 201, 585, 180, 19 | 3.7% |
| `ltc`, 1,000 games | **−11.1 ± 9.6** | **−25.0 ± 21.5** | −18 | 193/582/225 | 6, 96, 324, 72, 2 | 8.6% |

- **One `stc` run, interrupted once.** The first launch
  (`colosseum_match_rar-r23-stc_20261024_205603`, 18:56Z) was stopped by an
  interrupt after 79 games, at a report of −34.9 ± 41.2 over 61. The same
  command resumed it at 20:55Z: the log reads "resuming: 79 of 2000 games
  complete". Seed, directory and size are the registered ones.
- **Manifests:** `colosseum_match_rar-r23-stc_20261024_225520.manifest.txt`
  (`games.pgn` `ADD86800…6274A2CE4C`) and
  `colosseum_match_rar-r23-ltc_20261024_231447.manifest.txt`
  (`BB778AB4…A89B3EAF`), in `tools/results/`.
- **Time per arm.** The two arms share a binary name, so the PGN is split
  by `games.jsonl` (`nps_by_arm_n.py`, RAR-R21's reader with the game count
  read from the run). A against B: time a move +0.0% at `stc` and +0.2% at
  `ltc`, plies 1–20 within 0.3%, nps within 0.5%. The result is not a time
  artefact.
- **The wire is live** (`knownwin_by_arm.py`). Arm A reports a known-win
  score (|s| from 5,826 up to the mate range) in 536 of 2,000 `stc` games
  and 274 of 1,000 `ltc` games, 1,617 and 745 moves. Arm B does in 37 and
  47 games, from large material leads. The offset acted in about a quarter
  of the games at both clocks.

**Rule applied.**
- `stc` at −4.4 nElo is above its −13 threshold.
- **`ltc` at −25.0 nElo is below its −18 threshold: harm.**
- **RAR-R23 fails** under the frozen rule (either read below its
  threshold), and nothing lands.
- The `ltc` interval (−46.5 to −3.5 nElo) does not reach zero. That is
  harm at the registered 5% false-alarm rate, not a refutation at high
  confidence. The rule was frozen before any game and is applied as
  written.

**Disposition.** Nothing lands; D.1.6 returns to RESEARCH with this
estimate. The maintainer chooses between the two closes registered above:
- **D.1.6 `NO_CHANGE`.** The stalls are accepted, and the instruments,
  `AspKnownWinWindow` included, are removed in a cleanup.
- **The fallback clock stop at twice the target.** This needs the
  maintainer to reverse "no clock stop in D.1.6".

**Recommendation: `NO_CHANGE`.**
- Every arm that ended the probed stalls has cost Elo against its
  defaults: RAR-R22's cluster and this offset alone.
- No read has found a cost of the stalls themselves. RAR-R24's doubled
  maximum-hitting read +8.0 and +14.3 Elo against 2.6.0, and the hold
  records no measured cost at either control.
- The fallback stop cost about 15 Elo at the target (RAR-R18), and its
  conditioned form fires on a quarter of searches (RAR-R22, item 3).
- **What would change this:** a measured cost of the stalls in games, for
  instance a conversion read whose losses or draws come from moves at the
  maximum.

**Calibration.**
- **`stc`:** −4.4 nElo inside the predicted −2 ± 10, as predicted.
- **`ltc`:** −25.0 against the predicted 0 ± 14 (P pass 0.85), a miss in
  sign and magnitude.
  - The failed assumption was that a longer clock would weigh the repaired
    stalls and the deeper search equally.
  - The likelier mechanism, not measured here: the offset starts rec1's
    window walk six plies earlier (depth 10–16 against 19 at the defaults,
    with 33 × the nodes at depth 14). A longer clock reaches that climb in
    more of the quarter of games where the offset acts.
  - The confidence was also too high for a read of this size: the
    interval is ±21.5 nElo.
- **Time a move:** within ±1% at both clocks, as predicted.
