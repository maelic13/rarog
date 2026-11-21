# RAR-S126 — D.7.5: the folded compact kernel unfitted against the classic kernel, 2,000 games — REGISTERED 2026-11-18, before any game; READ 2026-11-18: nElo −11.1 ± 15.2, not clearly negative

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-18, before any game. The run is the
  maintainer's command.
- Disposition: observation — read 2026-11-18: nElo −11.1 ± 15.2, the interval
  [−26.3, +4.1] not wholly below zero, so not clearly negative by the
  frozen reading; recorded, and D.7.6's fit is next. Outside the
  prediction's 80% band.
- **Kind:** PLAN D.7.5, rule 8's paired screen: a diagnostic, not a stop
  (maintainer decision 2026-11-16). It accepts nothing; the one gate is
  D.7.7's, on the fitted kernel.
- **Inherits:** D.7.4's parity (`analysis/d74_compact_kernel_2026-11-18.md`)
  and D.7.4.1's fold (`analysis/d741_fold_2026-11-18.md`): layer A without
  a mismatch, the positions-visited check a pass; RAR-O08 (5) and RAR-O09
  (1) for the donor's search on Rarog's evaluation.

## Arms

Both from one clean worktree at `2820eaa`, so the switch is the only
difference between them:

- **`d75cand`:** the Cargo feature `compact-kernel`, the folded kernel with
  its seeds unfitted: `bench 13` **7,543,485 / EBF 2.422**.
- **`d75base`:** the default features, the classic kernel: **9,067,595 /
  EBF 2.479**, the head's fingerprint since D.6.31 (`9d7b42f`), unchanged at
  every D.7 commit.

The head's code outside the kernel is the same in both. Against `master`'s
head the classic arm differs only by D.7's behaviour-neutral changes, its
fingerprint identical and its speed a pass (G1, −0.34%, 2026-11-17).

## Binaries (appended before any game)

Built 2026-11-18 by `run_s126.ps1 -Step build` from a clean worktree at
`2820eaa`, rustc 1.100.0, `tools/build_test.ps1`, pext PGO. Both report
`id name Rarog dev-20261118-2820eaac`; the candidate's banner and sidecar
name `compact_kernel` (`cargo xtask build --tier pext --features
compact-kernel`). The dry run passed every preflight:
`tools/results/colosseum_match_rar-s126-stc_20261118_180547.dry-run.json`.

| Arm | Features | Bench | SHA-256 |
| --- | --- | --- | --- |
| `d75cand` | `compact-kernel` | 7,543,485 | `a8ead3175dbd2771255d25beb2b9993845d08eece1268a3377736b02454e27cc` |
| `d75base` | default | 9,067,595 | `2ee4b5e7b2c0429454410832672d463659b749d0e347f55acef5c36404bdf21f` |

## Run (frozen)

- **`stc`:** Colosseum `match`, 2,000 games at `3+0.03`, Hash 64, one
  thread, the wrapper's slot count, the UHO book in random order, no
  adjudication, seed 20261118, `tools/results/rar-s126-stc`. The runner is
  Colosseum and does not change.
- **Driver:** `analysis/artifacts/d75-unfitted-2026-11-18/run_s126.ps1`
  (`-Step build`, the agent's; `-Step stc`, the maintainer's).

## Reading (frozen)

From the manifest's nElo and its 95% interval:

- **Clearly negative:** the whole interval below zero. PLAN D.7.5's one
  diagnosis then looks for a defect (units, scaling, a contract the port
  broke), after the maintainer is asked (D.7 decision 5); a defect is fixed
  and the run repeated, no defect and the step goes on to the fit.
- **Anything else:** recorded, and D.7.6's fit is next.

The read also records, as diagnostics that decide nothing: mean depth and
nodes a move per arm, time losses, and the games that reach a won ending
with the fifty-move clock running (D.7.4.1's downgraded-mate band).

## PRE-REGISTERED PREDICTION (frozen 2026-11-18, before any game)

- **`stc`:** **+25 nElo**, 80% band **−5 to +55**. P(clearly negative)
  0.05.
- **Basis:** at equal thinking time Reckless's search on Rarog's evaluation
  led the head by 22.5 ± 7.9 Elo (RAR-O08 (5)), about +38 nElo at the 1.7
  nElo an Elo RAR-S125's games show; at equal work it was level, +3.47 ±
  8.37 (RAR-O09 (1)), so the lead is clock and speed, which the compact
  kernel carries natively (11.54% faster a node than the classic, G2). Taken
  down for what the hybrid had that this arm may not: the residual
  divergences (position 305 from depth 3, the downgraded-mate band, which
  can cost conversion), the fold's rounding, and Rarog's time management
  driving a kernel whose effort signals were the donor's.
- **Most likely failure mode:** a loss in won endings, where the compact
  kernel's trees on downgraded mates run one to two orders of magnitude past
  its donor's and an iteration may not finish.
- **Evidence layer:** games, unfitted. Bench and node counts of the two arms
  are different trees and say nothing of Elo.

## Result (read 2026-11-18)

Run by the maintainer 2026-11-18, 18:10 to 18:31 UTC, Colosseum CLI 0.5.0
(pinned `8fa63cc`). The runner's report and the wrapper's manifest
(`tools/results/colosseum_match_rar-s126-stc_20261118_191035.manifest.txt`):

- "Elo -6.9 +/- 9.5; nElo -11.1 +/- 15.2 over 2000 paired games"
- `d75cand` 522 W / 562 L / 916 D; pentanomial 47, 249, 424, 257, 23
  ("recount of games.pgn agrees"); faults none, time losses none.
- terminations: Checkmate 1,084, Threefold 687, InsufficientMaterial 166,
  FiftyMove 48, Stalemate 15.
- search, `d75cand`: depth 15.22 mean (non-mate 15.06), 131,915 nodes and
  64.3 ms a move, 2,052,018 nps; harness hold p99 45 ms, p999 205 ms, 2,530
  of 133,976 moves over the margin.
- search, `d75base`: depth 14.02 mean (non-mate 14.02), 126,316 nodes and
  64.4 ms a move, 1,960,896 nps; harness hold p99 12 ms, p999 95 ms, 863 of
  133,870 moves over the margin.

**Reading:** the 95% interval [−26.3, +4.1] includes zero, so the run is not
clearly negative. Recorded; D.7.6's fit is next (PLAN D.7.5).

### Diagnostics (decide nothing)

From `games.pgn` by `analysis/artifacts/d75-unfitted-2026-11-18/s126_diag.py`
(`s126_diag.txt`), each arm's own reported scores:

| | `d75cand` | `d75base` |
|---|---|---|
| Games where the arm reported +500 or a mate for itself | 532 | 584 |
| … of which won / drawn / lost | 522 / 8 / 2 (98.1%) | 562 / 20 / 2 (96.2%) |
| Games with an ordinary score of +3,000 or more (the downgraded-mate band) | 301, 300 won | 65, all won |
| Moves with an ordinary score of 3,000 or more either way | 2,018 | 213 |
| Fifty-move draws in which the arm had claimed +500 in its last 50 moves | 1 of 48 | 0 of 48 |

- **Won endings are not where the loss is.** The compact kernel shows the
  downgraded-mate band in five times as many games and converts from a
  winning score better, not worse. The 48 fifty-move draws (8 in RAR-S125's
  classic pair) are level endings: only one had a side claiming +500.
- **The loss is before the ending:** the classic arm reached a winning score
  in 52 more games.
- **Depth:** the compact arm searched 1.2 plies deeper on average at the
  same time and nearly the same nodes a move, and its per-node speed in
  games was +4.6%, against +11.54% on the bench (G2). The extra depth bought
  nothing visible here unfitted.
- **Harness hold:** the compact arm's moves exceed the margin three times as
  often. The hold is the harness's time on a move, not the engine's; the
  compact kernel's many bound lines per iteration are a candidate, not
  checked. No time was lost.

### Calibration (appended after the result)

Predicted +25 nElo (80% band −5 to +55); measured −11.1 ± 15.2: a miss in
sign and magnitude, outside the band. The assumption that failed: that
RAR-O08's +22.5 Elo at equal thinking time carries over to the compact
kernel. That lead included the donor's own clock, which spent about 8% more
of its budget (RAR-O09's basis); this arm runs on Rarog's time management,
and both arms used 64.3 to 64.4 ms a move, so the clock share was not here.
At equal work the donor's shape read level (RAR-O09, +3.47 ± 8.37), and this
run is consistent with that: depth gained, no Elo. The named most likely
failure mode, won endings, is refuted by the conversion figures above. The
miss is in mechanism (the clock was the lead's carrier) and confidence.
