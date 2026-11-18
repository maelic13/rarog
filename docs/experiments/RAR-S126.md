# RAR-S126 — D.7.5: the folded compact kernel unfitted against the classic kernel, 2,000 games — REGISTERED 2026-11-18, before any game

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-18, before any game. The run is the
  maintainer's command.
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
