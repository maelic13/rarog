# RAR-R34 — D.7.9.1: Reckless's time manager on the kernel against the hybrid it replaces, 2,000 games at `3+0.03`, 2,000 at `10+0.1` and a 200-game four-thread fault smoke — REGISTERED 2026-11-20, before any game; READ 2026-11-20: +30.0 ± 8.8 Elo at `3+0.03`, +18.6 ± 8.2 at `10+0.1`, no time loss, kept

Indexed under *4. Root search, time management and SMP* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-20, before any game. The three runs
  are the maintainer's commands.
- **Kind:** PLAN D.7.9.1, a form inside D.7's one cluster, chosen by
  maintainer decision (one donor's time manager, not a hybrid). It has no
  gate of its own: D.7.13 gates the cluster. These are harm reads, with the
  forfeit count as the invariant.
- **Inherits:** `analysis/d79_clock_2026-11-20.md`. Rarog's clock was a
  hybrid never fitted on Rarog. On the kernel it spent 0.59–0.61 of the
  base in its first twenty moves, where the donor's spent 0.84–0.87
  (RAR-O08), and it put 30% of its time past the optimum. Reckless's
  budget and its five factors were tuned together with its search
  (Reckless pull requests 956 and 1072).

## The change (engine `38e947c`)

- **Budget:** in sudden death the soft bound is `(0.0594 − 0.0492 ·
  e^(−0.0386 · fullmove)) · main + 0.75 · inc` and the hard bound is
  `0.7281 · main + 0.75 · inc`. Under `movestogo` the base is
  `main / movestogo + 0.75 · inc`, the soft bound one base and the hard
  bound five. Each bound stays within the main clock (`main` is the clock
  less Move Overhead, so within `main + inc` under `movestogo`), and 15 ms
  comes off each. These are constants, not fit coordinates.
- **Soft limit:** the soft bound times five factors.
  - Node share: `3.1838 − 2.6554 · share`, floor 0.546.
  - Score trend: `0.7426 + 0.1189 · fall`, clamped to 0.7214–1.4031. The
    fall is from the previous search's score, and the slope is the donor's
    0.0480 folded by 2536/1024.
  - Best-move stability: `1.2881 − 0.0440 · k`, floor 0.716.
  - Score stability: `1.2664 − 0.0416 · k`, floor 0.8642. Both counts are
    the root window's.
  - Best-move changes: `1.15 + ln(1 + 0.2526 · n)`, with `n` counted in
    the iteration, the donor's way.
  - The fifteen constants are tune options (`TmNodesBase` to
    `TmChangesScale`).
- **Guards kept:**
  - the reserve under Move Overhead (`time − 2·overhead`, and 30 ms more
    at several threads);
  - the proved-mate stop, the first-iteration guard and the tablebase box;
  - the fail-high stop at twice the soft target;
  - the single-move shortcut, `movetime` and the thread vote. The vote is a
    measured strict majority; exempting two threads cost −15.85 Elo.
- **Removed:** the sub-second horizon, Stockfish's budget, and the
  falling, instability, stability and effort terms.
- **Checks:**
  - `bench 13` 3,320,355 / EBF 2.317, unchanged: the clock never acts at
    a depth limit.
  - Tests: debug 540 passed, release 542; `texel` and `tune` libraries 292
    each. Clippy with all features and targets at zero warnings; `fmt` clean.
  - Release parameter coverage: every new coordinate moves the soft target
    at one end of its range.
- **Activation read at zero games** (`tools/diag/tm_activation.py`, a
  `diag` build of `c86bb60`, 300 book positions at `3000+30`, a fresh game
  each, idle host; `analysis/artifacts/d79-clock-2026-11-20/tm_activation_d791.txt`):
  - Every factor moves, and none is at one end on more than 90% of
    iterations.
  - The score trend sits at a clamp on 98%: low 56.2%, high 42.2%. In a
    fresh game the fall is read from zero; in games it is read from the
    previous move's score.
  - The fail-high stop ends 10 of 300 searches, at 2.09–3.40 times the
    last target. The old clock's rate was 11 of 300 (D.7.8.5).

## Arms and runs (frozen)

Built 2026-11-20 by `tools/build_test.ps1` (pext, PGO, rustc 1.100.0),
each from a clean detached worktree:

| Arm | Commit | `id name` | Bench | SHA-256 |
| --- | --- | --- | --- | --- |
| `d791cand` | `38e947c` | `dev-20261120-38e947c6` | 3,320,355 | `ff55f198bf9a8a668a898ed3f6cd08ade220c9d697aaf5ed58f493316fcfec66` |
| `d791base` | `3bb4330` | `dev-20261120-3bb4330f` | 3,320,355 | `63dbe403f7b30ecdeecf0aeed00d7252b576120ef08661f0ca020826d19e2ff4` |

`d791base`'s engine inputs are `0690de9`'s, D.7.8's closing state
(`git diff 0690de9 3bb4330` touches no engine input). Both run through
Colosseum `match` with `tools/colosseum.ps1`: Hash 64, the UHO book in
random order, no adjudication, a 20 ms margin.

1. **`3+0.03`:** 2,000 games, one thread, 14 slots, seed 20261126,
   `tools/results/rar-r34`. Dry run
   `colosseum_match_rar-r34_20261120_105038.dry-run.json`.
2. **`10+0.1`:** 2,000 games (`match-fixed-ltc.toml`), one thread, 14
   slots, seed 20261127, `tools/results/rar-r34-ltc`. Dry run
   `colosseum_match_rar-r34-ltc_20261120_105042.dry-run.json`.
3. **Four threads, faults only:** 200 games at `3+0.03`, three games at a
   time on four cores each, one logical CPU per physical core (RAR-M82),
   seed 20261128, `tools/results/rar-r34-4t`. Dry run
   `colosseum_match_rar-r34-4t_20261120_105045.dry-run.json` (placement
   explicit, 2 to 30 even).

**Cost:** about 25 minutes, 75 minutes and 12 minutes on an idle host.

**One command runs all three**, in the order 1, 3, 2:
`analysis/artifacts/d79-clock-2026-11-20/run_r34.ps1` (SHA-256
`7badad718c2011b8…`).
- It retries a start once a minute, but only while the harness refuses the
  host as busy before any game.
- It never restarts a read that ran; run again, it resumes an interrupted
  read with the same seed.
- It reads each manifest's time losses and stops the chain on one by
  `d791cand`.
- Its `-DryRun` path ran through all three reads on 2026-11-20, and its
  manifest checks read RAR-S132's manifest correctly.

**Exposure:** every clock search, so every move of every game.

## Readings and dispositions (frozen)

- **Validity:** every game ends in a rules result. A run with time losses
  or engine faults in more than 0.5% of its games is void.
- **The forfeit invariant:** a time loss by `d791cand` in any of the three
  runs returns the port to research before any Elo is read; the
  baseline's losses are reported. The clock profile
  (`analysis/artifacts/d79-clock-2026-11-20/clock_screen.py`) is read
  beside each one-thread run.
- **Keep rule** (D.7.8's reads): the port is kept unless the `3+0.03` or
  the `10+0.1` nElo interval lies wholly below zero. Kept, it lands
  before D.7.10, which then reads the kernel with it, and its fifteen
  coordinates join D.7.11.1's surface. Not kept, it returns to research,
  and the hybrid's thirteen coordinates go to the fit as D.7.9 set them.
- Nothing changes after a game.

## PRE-REGISTERED PREDICTION (frozen before any game)

1. **Clock profile at `3+0.03`, `d791cand`, 80% bands:**
   - first twenty own moves 0.72–0.95 of the base (centre 0.84;
     `d791base` about 0.59);
   - spend 89–96% of the budget (centre 93%; about 85.5%);
   - end clock median 150–480 ms (about 620);
   - floor median 80–350 ms; sides under 100 ms 3–35%.
2. **Elo:**
   - `3+0.03`: +4 Elo, 80% band −6 to +14; P(above zero) 0.68;
     P(interval wholly below zero) 0.04.
   - `10+0.1`: +3 Elo, 80% band −8 to +14; P(above zero) 0.62.
3. **Faults:** no time loss by `d791cand` in all three runs, 0.80; one or
   more at `3+0.03`, 0.15. Its low clocks are new, and the reserve is
   what stands between them and a forfeit.

- Most likely outcome: the port spends the donor's profile, reads a few
  Elo up inside both intervals, and is kept.
- The failure that would surprise: time losses at one thread. That would
  mean the donor's 15 ms is not enough under this harness's margin.

## Results (played 2026-11-20 by the maintainer)

All three ran on an idle host (0.5%, 0.5% and 3.8% busy, no waiver),
with 0 faults and 0 time losses on either side, and every game ended in
a rules result. In each run the recount of `games.pgn` agrees with the
runner. The `10+0.1` run stopped at 779 games when the remote console
running it died. It resumed from its checkpoint with the same seed and
command, and completed with all 2,000 games.

| Run | Manifest | Elo | nElo | Pentanomial | Time losses (cand, base) |
| --- | --- | --- | --- | --- | --- |
| `3+0.03` | `colosseum_match_rar-r34_20261120_111143.manifest.txt` | **+30.0 ± 8.8** | **+51.9 ± 15.2** | 10, 191, 455, 305, 39 | 0, 0 |
| `10+0.1` | `colosseum_match_rar-r34-ltc_20261120_124957.manifest.txt` | **+18.6 ± 8.2** | **+34.6 ± 15.2** | 12, 189, 494, 290, 15 | 0, 0 |
| Four threads, faults only | `colosseum_match_rar-r34-4t_20261120_113410.manifest.txt` | (+31.4 ± 25.9) | (+58.7 ± 48.2) | 0, 18, 49, 30, 3 | 0, 0 |

Search, from the manifests:

| Run | Arm | Depth (mean) | Nodes a move | Time a move |
| --- | --- | --- | --- | --- |
| `3+0.03` | `d791cand` | 17.93 | 130,709 | 63.9 ms |
| `3+0.03` | `d791base` | 18.19 | 128,442 | 61.6 ms |
| `10+0.1` | `d791cand` | 21.58 | 432,227 | 209.6 ms |
| `10+0.1` | `d791base` | 21.86 | 440,132 | 205.7 ms |

**Disposition: kept.** Neither interval lies below zero, and `d791cand`
lost no game on time. The port stays on `d7-kernel` and lands before
D.7.10, and its fifteen coordinates join D.7.11.1's surface. The four
threads' Elo decides nothing.

**The clock profile** (`clock_screen.py`; outputs `r34_stc_profile.txt`,
`r34_ltc_profile.txt` and `r34_4t_profile.txt` in
`analysis/artifacts/d79-clock-2026-11-20/`; every game's checks passed):

| | `3+0.03` cand | `3+0.03` base | `10+0.1` cand | `10+0.1` base | 4T cand | 4T base |
| --- | --- | --- | --- | --- | --- | --- |
| First 20 moves, share of base | 0.746 | 0.602 | 0.750 | 0.586 | 0.735 | 0.609 |
| Spend, share of budget | 89.8% | 86.2% | 92.3% | 90.2% | 90.0% | 86.8% |
| End clock (median) | 428 ms | 576 | 868 | 970 | 421 | 562 |
| Floor (median) | 226 ms | 436 | 217 | 479 | 228 | 463 |
| Sides under 100 ms | 32.1% | 0% | 0% | 0% | 6.0% | 0% |
| Undecided moves under 500 ms | 24.5% | 5.2% | 8.1% | 1.2% | 22.2% | 4.8% |

The port spends the early game as the donor does (0.75 of the base in
twenty moves, against the hybrid's 0.60 at both controls). It reaches the
middle game with less clock, plays the late game on a lower floor, and
loses no game on time.

**D.7.8.5's trigger, re-read for the new time manager:** B.5's
interrupt instrument (`analysis/artifacts/d78-night/b5_interrupt.py`, the
`diag` build of `c86bb60`, 300 book positions, idle host;
`interrupt_d791.txt`).
- Searches stopped inside an iteration: **0 of 300** at `3000+30`,
  `1000+30` and `300+30`, against the old clock's 1, 4 and 24.
- Moves discarded: none.
- The trigger (2% of searches at a clock discarding a move) has not
  fired. The fail-high stop ended 9, 12 and 7 searches.

**Not settled:** the moves the manifests count over the 20 ms margin by
`h=` (charged time less the time in the last `info` line). The candidate
had 3,777 against 3,705 at `3+0.03`, and 11,163 against 6,361 at
`10+0.1`. These are not holds and no game was lost. Since the screen
finds no stop inside an iteration, the source of the excess is not
settled.

**Calibration:**
1. *The clock profile:* held. The first twenty moves read 0.746 (band
   0.72–0.95, low end) and the spend 89.8% (89–96%, low end). The end
   clock (428 ms), the floor (226 ms) and the sides under 100 ms (32.1%)
   all fell inside their bands.
2. *Elo:* missed in magnitude at both controls, the sign held.
   - `3+0.03` read +30.0 against a predicted +4 (80% band −6 to +14);
     `10+0.1` read +18.6 against +3 (band −8 to +14).
   - The assumption that failed: I priced the change as a redistribution
     of time worth a few Elo, by analogy with RAR-R17's null read of the
     horizon. That read moved only the late game's time. The size of
     this gain says the hybrid's early under-spend and its factors, not
     its reserve, were what cost.
   - Kind: magnitude, from a mechanism weighted wrongly.
3. *Faults:* held. No time loss by `d791cand` in any run.

Evidence layer: two 2,000-game fixed reads, a diagnostic inside D.7's
cluster. They keep the form and accept nothing; D.7.13 gates the
cluster.
