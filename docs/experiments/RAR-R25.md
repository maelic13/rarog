# RAR-R25 — D.1.6: the fail-high stop at twice the soft target, read alone as a harm read at `3+0.03` and `10+0.1`, with the probe — REGISTERED 2026-10-25, before any game

Indexed under *4. Root search, time management and SMP* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-25 before any game. The probe and
  the two matches are the maintainer's commands, or the agent's once the
  host is idle; the reads wait for Basilisk's measurements to finish.
- **Kind:** a clock cap read as a repair, under AGENTS' repair case 2. The
  maintainer re-opened it on 2026-10-25 ("if it helps, as long as we do not
  hurt play strength"). It is not the fix of the cascade, which D.1.6's
  research card pursues in the search; it bounds the symptom meanwhile.
- **The device** (`f3549ff`, off by default since `a0f9755`):
  `TmFailHighTarget`, in ten-thousandths of the soft target. When a root
  window fails high on the main thread past that multiple of the target,
  the search ends on the move that failed high, with its lower bound.
  - RAR-R18 read it at the target as costing about 15 Elo, because it ended
    ordinary fail-highs. At twice the target its replay fired on 2 of 300
    first-move searches, and the probe passed.
  - Its strength at twice the target was never read directly. +3.3 ± 12.8
    Elo for having it off is arithmetic across two reads against a third
    arm, not a measurement.
- **Hypothesis:** at twice the target the stop ends the won-ending stalls
  the probe measures and is no weaker than having it off at either control.
- **Its known gap.** The stop is checked only when a window returns failing
  high (`src/search/mod.rs`, the fail-high branch). A single window that
  starts before twice the target and grinds on without failing high still
  runs to the hard maximum. Basilisk's A.8.17 stop has the same shape. The
  probe's rec3 row at `180000+2000` is where this would show: the defaults
  on this build ran it to the maximum on 2026-10-24 (RAR-R23).
- **Build:** both sides run `tools/test_engines/rarog-d16inst3-tune.exe`
  (`3f4dcf4`, SHA-256 `A5A3CF06E386E3BA…F6C01F01E6`, bench 13,987,223).
  Arm A sets `TmFailHighTarget=20000`; arm B sets nothing, which leaves the
  stop off. d1's clock pieces are in both arms and cancel. Bench does not
  read the clock, so the fingerprint cannot move.

## The reads (frozen)

- **`probe`:** `tm_rec_probe_d16.py`, rec1 and rec3 at `60000+600` and
  `180000+2000`, both arms, a fresh process per search, the probe's own
  idle check. **The probe passes when no arm-A row reaches the maximum.**
- **`stc`:** Colosseum `match -CategoricalTuneBuild`, 2,000 games at
  `3+0.03`, 20 ms margin, Hash 64, one thread, the wrapper's slot count, the
  UHO book in random order, no adjudication, seed 20261091,
  `tools/results/rar-r25-stc`.
- **`ltc`:** `match-fixed-ltc`, 1,000 games at `10+0.1`, seed 20261092,
  `tools/results/rar-r25-ltc`.
- **Command:** `run_r25.ps1 -Read probe|stc|ltc`, in
  `analysis/artifacts/d16-cascade-2026-10-23/confined-2026-10-24/`.
- **Dry runs:** both matches dry-ran clean on 2026-10-25 (host 1–3%, no
  engine process, the pinned colosseum-cli 0.4.0, both benches matched).
  The resolved configuration puts `TmFailHighTarget=20000` on engine A
  only.
- **The harm rules**, from the manifest's nElo, as RAR-R23 states them with
  their power:
  - `stc`: harm below **−13 nElo** (standard error about 7.8);
  - `ltc`: harm below **−18 nElo** (about 11.0).
  - "No harm" means no harm beyond roughly −20 nElo at `3+0.03` and −30 at
    `10+0.1`. Neither read can show the absence of a small loss.
- **Pass:** the probe passes and neither match is below its threshold.
  **Fail:** either match below its threshold, or an arm-A probe row at the
  maximum.
- **Time a move per arm** from the PGN (`nps_by_arm_n.py`): the stop leaves
  time on the clock when it fires. That is read and reported, and decides
  nothing.

## Dispositions

- **On a pass,** `TmFailHighTarget` defaults to 20000 on `d1` in its own
  engine commit, with a changelog entry: the engine no longer spends its
  whole time limit on one move of a won ending. The fingerprint is
  unchanged. D.1.2.1's cascade half ticks on the probe.
  - Retry trigger: when D.1.6's search research lands a fix, the stop is
    re-read against it and deleted if the stalls are gone without it.
- **On a fail,** the stop stays off and is deleted when D.1.6 closes. A
  failed probe row is recorded as the gap above, with its window and nodes.
- Either way the research card's work continues; this read decides only
  the cap.

## PRE-REGISTERED PREDICTION (frozen before exposure)

- **`probe`:** PASS, P 0.7.
  - rec1 at `180000+2000`: about 2 × its optimum, against 2.85 × off.
  - rec3 at `180000+2000`: ends near twice its optimum, because the off
    arm's maximum came with 36 lower-bound lines, so fail-highs are
    frequent there.
  - The failure that would surprise least: rec3 reaches the maximum through
    one window, the gap above.
- **`stc`:** −2 ± 10 nElo, P(pass) 0.85. The stop fires on under 1% of
  searches here, and the time it leaves is small.
- **`ltc`:** −2 ± 14 nElo, P(pass) 0.85. Cascades past twice the target are
  likelier at a longer clock, in positions already decided.
- **Time a move:** within ±1% of the stop off overall, P 0.8.
