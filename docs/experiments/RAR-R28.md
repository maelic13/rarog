# RAR-R28 — D.1.5: D.1's closing read, `d1`'s clock against the 2.6.0 engine, harm reads at `3+0.03` and `10+0.1`, a four-thread read, the probe and the profile — REGISTERED 2026-10-26, before any game

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
