# RAR-R24 — D.1.5: d1's clock pieces alone against the 2.6.0 engine, harm reads at `3+0.03` and `10+0.1` and the maximum-hitting rate — REGISTERED 2026-10-24, before any game

Indexed under *4. Root search, time management and SMP* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-24 before any game; the two matches
  are the maintainer's commands, the stall read the agent's.
- **Kind:** D.1.5's harm read of the clock pieces alone, which RAR-R22's
  addendum (item 5) found had never been run.
  - **What the arm carries:** D.1.3's mechanisms at D.1.4's seeds, which
    are `d1`'s defaults. The fail-high stop is off (`a0f9755`), and every
    D.1.6 instrument is at its neutral default.
  - **Why it matters:** every D.1.6 arm carried this clock, and RAR-R22's
    −7.4 ± 7.5 against the 2.6.0 engine therefore sums the clock and the
    cluster.
  - **What it is not:** D.1.5's closing gate. That stays as PLAN records it.
  - **Order:** PLAN orders D.1.5 after D.1.6, and the addendum recommends
    this read first. The maintainer decides; both arms are fixed binaries,
    so either order attributes cleanly.
- **Hypothesis:** at their seeds the clock pieces are no weaker than the
  2.6.0 engine at either control. They run to the hard maximum in about
  twice as many `3+0.03` games, early and at balanced scores.
- **Baseline:** the 2.6.0 engine, `rarog-d15head-pext-pgo.exe` (`468ed7b`,
  SHA-256 `273FEDF436D4864C…2446AAC0E`, bench 13,987,223).
- **Candidate:** `tools/test_engines/rarog-d1head-pext-pgo.exe`, a clean
  PGO build of `d1` at **`3f4dcf4`** (`cargo xtask build --arch pext
  --pgo`, `rustc 1.99.1`). SHA-256 `FB13F4C223CC7CF5…84538AD4B7`, bench
  **13,987,223** / EBF 2.565, version `2.7.0-dev`.
  - Its engine inputs differ from `2f17467` (the source RAR-R21 and RAR-R22
    built from) only by `3f4dcf4`'s tune-only `AspKnownWinWindow` at 0. The
    production build folds that branch out, and the bench is identical.
  - Checked with `git diff --stat 2f17467 3f4dcf4 -- src Cargo.toml
    Cargo.lock build.rs xtask .cargo`: two files, the one commit.
- **Earlier reads, all with the fail-high stop on, so none measures this
  arm:**
  - RAR-R17's seeds read, all seven mechanisms against the head: −14.9 ±
    8.8 Elo;
  - RAR-R18's removal of the stop against the seeds: +15.5 ± 9.3;
  - RAR-R16's fitted cluster: +0.6 ± 5.8 at 2,345 pairs, unresolved.

## The reads (frozen)

- **`stc`:** Colosseum `match`, 2,000 games at `3+0.03`, 20 ms margin,
  Hash 64, one thread, the wrapper's slot count, the UHO book in random
  order, no adjudication, seed 20261081, `tools/results/rar-r24-stc`. Each
  arm plays with its own clock and no option set.
- **`ltc`:** `match-fixed-ltc`, 1,000 games at `10+0.1`, seed 20261082,
  `tools/results/rar-r24-ltc`.
- **`stalls`:** `stall_scan2.py` over both reads' PGNs, the addendum's
  instrument. It counts a move at 90% or more of the hard maximum that
  `tm_table.limits` computes from the rebuilt clock. It reports, per side:
  stall moves, the share of games with one, the typical score and ply, and
  those games' results.
- **Commands:** `run_r24.ps1 -Read stc|ltc|stalls`, in
  `analysis/artifacts/d16-cascade-2026-10-23/confined-2026-10-24/`. Both
  matches dry-ran clean on 2026-10-24 (host 2%, the pinned colosseum-cli
  0.4.0, both benches matched).
- **The harm rules**, AGENTS' repair case 2, from the manifest's nElo:
  - `stc`: harm if the estimate is below **−13 nElo** (standard error about
    7.8 nElo at 2,000 games);
  - `ltc`: harm if the estimate is below **−18 nElo** (about 11.0 at 1,000
    games, the 5% false-alarm point at a true zero);
  - the power is as RAR-R23 states it.
- **The stall read decides nothing on its own.** It says whether the
  doubled maximum-hitting reproduces between PGO builds against 2.6.0, as
  well as on the tune build.

## Dispositions

- **No harm at either control:** the clock pieces at their seeds are not
  where RAR-R22's −7.4 comes from, which fits RAR-R21's equal-clock reads
  that put the cluster's share near −7. They stay on `d1`. D.1.6 proceeds
  by RAR-R23. D.1.5's closing read follows D.1.6 as PLAN records it, with
  this read as its baseline evidence.
- **Harm at either control:** the clock pieces cost at their seeds, and
  D.1.5 returns to research before any D.1.6 result is read against the
  2.6.0 engine.
  - The first suspect is the doubled maximum-hitting, if the stall read
    reproduces it.
  - The maintainer decides whether `d1`'s defaults turn the clock pieces
    off before the 2.7.0 cut.
  - RAR-R23 is unaffected, because it reads at equal clock.

## PRE-REGISTERED PREDICTION (frozen before exposure)

- **`stc`:** 0 ± 14 nElo (about 0 ± 8 Elo), P(pass) 0.85. RAR-R17's −14.9
  with the stop and RAR-R18's +15.5 for removing it cancel. Those were
  different reads, so the sum carries both reads' errors.
- **`ltc`:** −3 ± 16 nElo, P(pass) 0.80. RAR-R16's `10+0.1` direction read
  of the fitted cluster with the stop read −6.3 ± 11.7.
- **`stalls` at `3+0.03`:** d1head stalls in 30–45% of games against
  d15head's 15–20%, and d1head's stalls fall before ply 40 at balanced
  scores (P 0.8 for both). At `10+0.1` the two shares are within 5 points
  of each other (P 0.6).
- **Most likely miss:** a small negative at `stc` from the early stalls,
  which 2,000 games cannot separate from zero.
