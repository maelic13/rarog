# RAR-R28 — D.1.5: D.1's closing read, `d1`'s clock against the 2.6.0 engine, harm reads at `3+0.03` and `10+0.1`, a four-thread read, the probe and the profile — REGISTERED 2026-10-26, before any game; READ 2026-10-26: no harm; D.1.2.2 ticks, D.1.2.1 misses its won share

Indexed under *4. Root search, time management and SMP* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-26 before any game. All reads are
  the maintainer's commands; the profile read may be the agent's.
- **Kind:** D.1.5's closing read, in the form its PLAN text fixed when
  D.1.6 was ordered first: AGENTS' repair case 2 with no Elo claim.
  RAR-R16's `[0,3]` SPRT ended unresolved, and its arm no longer exists.
- **Hypothesis:** D.1's clock work as it stands on `d1` is no weaker than
  the 2.6.0 engine at `3+0.03` or `10+0.1`, and it repairs the clock
  defects D.1.2 named. That work is D.1.3's mechanisms at D.1.4's seeds:
  - the sub-second horizon (`TmHorizonPerSec` 50);
  - the proved-mate stop;
  - the first-iteration guard;
  - the stability and effort factors;
  - the cross-move trend;
  - the fail-high stop at twice the target (D.1.6, RAR-R25).

  The defects it repairs are rec2's missing stop, the won-ending stalls
  and the forfeit margin's clock floor.
- **What is already known.**
  - The clock pieces alone read +8.0 ± 8.9 Elo at `3+0.03` and +14.3 ±
    11.2 at `10+0.1` against 2.6.0 (RAR-R24).
  - The stop at twice the target read +4.3 ± 8.3 and +4.9 ± 9.8 against
    off at equal speed (RAR-R25).
  - RAR-R16's four-thread direction read of the fitted cluster, with the
    stop at the target, was −10.4 ± 11.6.
- **Candidate:** `tools/test_engines/rarog-d1head3-pext-pgo.exe`, a clean
  PGO build of `d1` at `6fcfac2` (engine source `42b001f`), SHA-256
  `10843E4CF7311E3A…51387C25CD`, `rustc 1.99.1`, bench **13,987,223**,
  version `2.7.0-dev`.
- **Baseline:** the 2.6.0 engine, `rarog-d15head-pext-pgo.exe` (`468ed7b`,
  SHA-256 `273FEDF436D4864C…2446AAC0E`, bench 13,987,223).

## The reads (frozen)

- **`probe`:** rec1, rec2 and rec3 at `60000+600` and `180000+2000`, both
  binaries, a fresh process per search, the probe's own idle check
  (`probe3.py`, `tm_rec_probe_d16.py`'s method with rec2 added).
- **`stc`:** Colosseum `match`, 2,000 games at `3+0.03`, 20 ms margin,
  Hash 64, one thread, the wrapper's slot count, the UHO book in random
  order, no adjudication, seed 20261121, `tools/results/rar-r28-stc`.
- **`ltc`:** `match-fixed-ltc`, 2,000 games at `10+0.1`, seed 20261122,
  `rar-r28-ltc`.
- **`4t`:** 1,000 games at `3+0.03`, Threads 4, Hash 512, three games at a
  time, four cores a game, seed 20261123, `rar-r28-4t`. These are RAR-M66's
  conditions.
- **`profile`:** `tm_profile_d15.py` on the three runs, with the
  candidate's horizon at 50.
- **Commands:** `run_r28.ps1 -Read probe|stc|ltc|4t|profile` in
  `analysis/artifacts/d15-close-2026-10-26/`, in that order. The three
  matches dry-ran clean on 2026-10-26: host 0–8%, the pinned
  colosseum-cli 0.4.0, both benches matched, and the four-thread run
  resolved to four cores a game.
- **Harm rules** (AGENTS' case 2, from the manifest's nElo): `stc` and
  `ltc` each harm below **−13 nElo** (2,000 games, standard error about
  7.8). This catches a true −10 with probability 0.36 and −20 with 0.82.
  The `4t` read is a direction read and decides nothing on its own.
- **The ticks, as PLAN fixed them:**
  - **D.1.2.1 (rec2 half):** no candidate probe row at the maximum, rec2
    under 2 s at both clocks, and the profile's won and mate shares at
    ≥ 0.9 × the maximum below 0.2% in `stc` and `ltc`.
  - **D.1.2.2:** no candidate time loss in any run, and at `3+0.03` the
    candidate's clock-floor median above 350 ms with under 3% of games
    touching 100 ms.

## Dispositions

- **Neither harm rule fires:** D.1's clock work is kept for 2.7.0 as a
  correctness change, and the changelog gets its user-facing entry for
  the clock pieces. D.1.5 closes; D.1.2, and with it D.1, closes once both
  of its open sub-steps tick.
- **A tick criterion misses with no harm:** that sub-step returns to
  research with the read, and the rest stands.
- **A harm rule fires:** D.1.3's clock pieces return to the 2.6.0 clock.
  The fail-high stop stays, on its own read (RAR-R25). The clock goes back
  to research with this read.
- The `4t` read below −15 Elo opens a held item under D.2, the thread
  question RAR-M74 already holds. It does not by itself reverse the
  one-thread result.

## PRE-REGISTERED PREDICTION (frozen before exposure)

- **`stc`:** **+12 ± 14 nElo** (about +7 Elo), P(pass) 0.97. That is
  RAR-R24's +13.7 for the clock pieces plus a small share of the stop's
  +8 from RAR-R25; the two reads are not additive, hence the width.
- **`ltc`:** **+20 ± 16 nElo**, P(pass) 0.95. RAR-R24 read +27.3, and the
  stop gains more where stalls are longer.
- **`4t`:** −2 ± 15 Elo. RAR-R16's −10.4 carried the stop at the target,
  which cost about 15 Elo.
- **Probe:** PASS with rec2 under 2 s at both clocks, P 0.85; RAR-R18 read
  rec2 at 5 ms with the mate stop.
- **Profile, D.1.2.1:** won and mate shares below 0.2%, P 0.6. RAR-R16's
  `10+0.1` won share was 0.47% with the stop at the target, now raised to
  twice it.
- **Profile, D.1.2.2:** the clock-floor median above 350 ms with under 3%
  of games at 100 ms, P 0.6. The horizon at its seed 50 contracts below a
  second of clock.

## Results (2026-10-26, maintainer-run, idle host)

All three matches completed with 0 faults and 0 time losses. The figures
are the runner's final report lines (colosseum-cli 0.4.0). A is `d1`'s
head, B the 2.6.0 engine.

| Read | Elo | nElo | Harm threshold | W/D/L (A) | Pentanomial | Host at launch |
|---|---|---|---|---|---|---|
| `stc`, 2,000 games | **+1.2 ± 8.9** | **+2.1 ± 15.2** | −13 | 490/1027/483 | 29, 225, 485, 232, 29 | 5.5% |
| `ltc`, 2,000 games | **−0.2 ± 7.8** | **−0.3 ± 15.2** | −13 | 472/1055/473 | 10, 222, 536, 223, 9 | 0.0% |
| `4t`, 1,000 games | −8.0 ± 11.3 | −15.3 ± 21.5 | (direction) | 208/561/231 | 5, 127, 261, 100, 7 | 1.5% |

- **Manifests:** `colosseum_match_rar-r28-{stc_20261026_173818,ltc_20261026_175903,4t_20261026_190949}.manifest.txt`
  in `tools/results/`.
- **Time and depth, A against B** (the manifests' `search:` lines):
  - `stc`: 67.0 against 70.3 ms a move, non-mate depth 13.27 against
    13.66;
  - `ltc`: 223.0 against 231.1 ms, 16.56 against 16.88;
  - `4t`: 65.3 against 68.8 ms, 13.74 against 14.09.

  `d1`'s clock spends 3–5% less a move than 2.6.0's, at every control.

**The probe** (`tm_rec_probe_r28.txt`), as a multiple of the optimum:

| Clock, position | `d1` head | 2.6.0 |
|---|---|---|
| `60000+600` rec1 / rec2 / rec3 | 0.85 / **0.00 (5 ms)** / 0.88 | 0.85 / **MAX** / **MAX** |
| `180000+2000` rec1 / rec2 / rec3 | 1.34 / **0.00 (5 ms)** / 0.88 | 2.30 / **MAX** / **MAX** |

The head has no row at the maximum and ends rec2 in 5 ms at both clocks.
The script's FAIL verdict lists the 2.6.0 rows.

**The profile** (`profile_rar-r28-*.txt`), for the head:

| Read | Won moves at ≥ 0.9 × max | Mate+ moves at ≥ 0.9 × max | Clock floor p50 | Games with floor < 100 ms (2.6.0's) | Forfeits |
|---|---|---|---|---|---|
| `stc` | **23 of 9,555 (0.24%)**, every game won | 0 (0.00%) | 429 ms (173) | 0 (680) | 0 |
| `ltc` | **86 of 10,122 (0.85%)**, every game won | 7 (0.13%) | 730 ms (447) | 0 (0) | 0 |
| `4t` | 6 of 4,352 (0.14%) | 0 | 432 ms (154) | 0 (41) | 0 |

**Rules applied.**
- **Harm: neither rule fires** (+2.1 and −0.3 nElo against −13). D.1's
  clock work is kept for 2.7.0, and D.1.5 closes.
- **D.1.2.2 ticks.** There is no candidate time loss, and at `3+0.03` the
  clock-floor median is 429 ms (above 350) with no game touching 100 ms.
- **D.1.2.1 misses one criterion.**
  - The probe passes and rec2 ends in 5 ms.
  - But the won share at ≥ 0.9 × the maximum is 0.24% at `3+0.03` and
    0.85% at `10+0.1`, above the frozen 0.2%. The mate share passes.
  - By the disposition, D.1.2.1 returns to research with this read. D.1.2,
    and with it D.1, stays open on it.
  - Every game with such a move was won, so the read shows a time-use
    defect, not a measured loss.
  - The likeliest mechanism is the stop's known gap: one long window that
    never fails high runs to the maximum (RAR-R25). That is the case the
    LMR correction cap closed in the probe (RAR-R26) and that gated H0
    (RAR-R27).
- **`4t`:** −8.0 ± 11.3 Elo, above the registered −15 trigger, so no item
  opens. It sits in the direction RAR-M74's held thread question already
  points.

**Calibration.**
- **`stc`:** +2.1 inside the predicted +12 ± 14, low.
- **`ltc`:** −0.3 against +20 ± 16, a miss in magnitude. RAR-R24's +27.3
  for the clock pieces alone over 1,000 games did not reproduce over
  2,000. The assumption that the two reads add was wrong; the better
  estimate of the clock work at `10+0.1` is this one, near zero.
- **`4t`:** −8.0 inside −2 ± 15.
- **Probe and rec2:** as predicted (0.85).
- **D.1.2.2:** held (0.6).
- **D.1.2.1's won share:** missed (0.6). The stop at twice the target
  bounds cascades that fail high, not single long windows, and at
  `10+0.1` those occur in about 1% of won moves.
