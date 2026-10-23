# RAR-R20 — D.1.6.3: the cascade fix as one cluster of four parts; the probe, a wire check, and the gate's registration — REGISTERED 2026-10-23, before any game

Indexed under *4. Root search, time management and SMP* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-23, before any game. The probe and the
  200-game wire check are the agent's (idle host); the fit and the gate are
  the maintainer's.
- **Kind:** PLAN D.1.6.3, the gate of D.1.6's cluster, after D.1.6.1's
  research (RAR-R19) and the maintainer's decision of 2026-10-23 to have the
  whole fix, every part, done now and verified.
- **The cluster, four parts** (the research's device, RAR-R19): every part is
  a switch on `d1` at its neutral default and is baked on as the default on
  the arm.
  1. **A known-win evaluation** of a bare-king ending the winner can mate in
     (`mopup_known_win` 6000 in both phases, with the winner's sign; engine
     `571eb83`). Rarog's HCE valued king, rook and bishop against king at
     about eight pawns, Stockfish 11's classical evaluation at 55; the
     search had to carry the win across the horizon itself.
  2. **A score-scaled first window** (`AspDeltaSquareDiv` 10193: the first
     window's half-width is `AspirationDelta + score² / 10193`, Stockfish's
     form; `ff14793`). Alone the known win made the cascade worse; with this
     window the cascade is gone: one or two windows an iteration and a
     gradual score.
  3. **The line budget charged for deeper reduced searches**
     (`CoreLmrDeeperBudget` 1; `0366884`): a reduced move searched above its
     nominal depth draws on the line's extension budget, so no line grows
     past twice the iteration plus quiescence, the budget's stated invariant
     (Stockfish allows the deeper search only while the ply is under twice
     the root depth).
  4. **The deepening freeze** (`TmDepthFreeze` 5000; `097ea5d`): once a
     completed iteration finds half the soft target spent, the following
     iterations no longer deepen, each searched `3 (n + 1) / 4` plies short
     (Stockfish's `increaseDepth`), so a clock search past its target meets
     completed iterations whose cost no longer grows.
- **Baseline:** the 2.6.0 engine, `rarog-d15head-pext-pgo.exe` (`468ed7b`,
  `273FEDF4…`, bench 13,987,223), the same binary RAR-R16–R19 used.
- **Candidate:** the throwaway branch `d16-arm2` at `6fd195e` on `097ea5d`,
  recipe `analysis/arm_patches/6fd195e-rar-r20-d16fix4.patch` (SHA-256
  `f45cfa24…483bfa`; applied to `097ea5d` through a temporary index it
  reproduces the arm's tree `3678a90f…`), version `2.7.0-dev+d16fix4`.
  Binary `tools/test_engines/rarog-d16fix4-pext-pgo.exe`, SHA-256
  `95268163F88276FFE66F731CFDB29207B7703BA98C37AE60D0F99B6183F5C30F`, built
  clean by `tools/build_test.ps1 -SourceRoot` (`cargo xtask build --arch
  pext --pgo`, `rustc 1.99.1`); **bench 8,996,837**, 0.643 × the head's.
  Release tests on the arm: 33 suites, all passing.
- **The evaluation half** is a change to the frozen evaluation (engine
  source `9fb525e`), allowed by the maintainer on 2026-10-23 for this
  cluster; it moves `bench 13`, so the fingerprint is re-declared when the
  cluster lands.

## Verification before any game (2026-10-23, idle host)

**The probe, no clock stop on either side** (`tm_rec_probe_d16.py`; the
`diag,tune` builds with the switches set, about 1.3 × slower than PGO;
`tm_rec_probe_fix3.txt`, `tm_rec_probe_fix4.txt`):

| Clock | Position | Head (2.6.0 engine) | Parts 1–3 | Parts 1–4 |
|---|---|---|---|---|
| `60000+600` | rec1 | 4.1 s (0.88 × optimum), depth 18 | 5.7 s (1.21 ×), depth 16 | **2.7 s (0.58 ×)**, depth 16 |
| `60000+600` | rec3 | **30.3 s, the hard maximum** (6.87 ×) | 3.9 s (0.89 ×), depth 22 | **3.1 s (0.70 ×)**, depth 22 |
| `180000+2000` | rec1 | 37.5 s (2.47 ×), depth 19 | 12.3 s (0.81 ×), depth 17 | **9.4 s (0.62 ×)**, depth 17 |
| `180000+2000` | rec3 | **98.0 s, the hard maximum** (6.87 ×) | 30.8 s (2.16 ×), depth 24 | **6.9 s (0.49 ×)**, depth 23 |

Parts 1–3 end every probed stall under the maximum; part 4 ends every move
within the optimum at both clocks, so the result no longer depends on which
iteration crosses the target (the speed luck that failed RAR-R19's first
arm). The probe is re-run on the PGO arm below.

## The gate (frozen)

- **SPRT:** Colosseum `sprt-repair` (`[-5,5]` nElo, pentanomial,
  `model=normalized`, α = β = 0.05; AGENTS' repair case 3), `3+0.03`, 20 ms
  margin, Hash 64, one thread, 14 slots, the UHO book in random order, no
  adjudication, **cap 20,000 pairs**, seed 20261042, run directory
  `rar-r20-gate`; then `match-fixed-ltc` 1,000 games at `10+0.1`, seed
  20261043, `rar-r20-ltc`; then the probe on both PGO binaries.
  `run_r20.ps1 -Read gate|ltc|probe|wire` beside this entry.
- **Before the gate, the research recommends a fit:** C.10 fitted the
  aspiration and reduction constants around the fail-hard, flat-valuation
  search; the cluster changes what they trade off. A registered SPSA block
  over `AspirationDelta`, `AspGrowthPct`, `AspGrowthHighPct`, `AspGrowthAdd`,
  `AspDeltaSquareDiv`, `CoreLmrCorrection`, `CoreLmrPv`, `CoreLmrPvWindow`
  and `TmDepthFreeze` on a tune build of the arm's source (60,000 games,
  about ten hours) is the cheaper path to a gate that can pass; the gate on
  the unfitted seeds is registered here so the maintainer can choose.
- **Dispositions:** H1 lands the four parts on `d1` as the defaults,
  re-declares the fingerprint, deletes `TmFailHighTarget` and its test and
  writes the user-facing changelog entry; D.1.2.1's cascade half ticks on
  the probe. H0 refutes the cluster at these seeds; the fit is then the
  registered next step, not a device change. The cap records "unresolved".
- **Prediction, frozen before any game:** unfitted, **−6 ± 8 Elo** at
  `3+0.03` (80% band −14 to +2); H1 0.30, cap 0.25, H0 0.45. Bench 13 is
  about 0.73 × the head's (the known win and the squared window remove
  cascades in bench's own won positions; the budget removes 30% of the
  tree), which is a different tree, not a faster one. `10+0.1`: better than
  STC by a few Elo (the stalls and the deep won endings are worth more
  there), P(point estimate above STC's) 0.65. Most likely miss: the sign
  at STC, from the unfitted constants.
- **The 200-game wire check** (agent-run, `3+0.03`, the PGO arm against the
  head, seed 20261044, `rar-r20-wire`): it decides nothing about strength
  (±35 Elo); it shows the cluster plays whole games without fault and reads
  the arms' time use, and the manifest's `search:` lines confirm the arm is
  the baked one.

## PRE-REGISTERED PREDICTION (frozen before exposure)

1. The PGO arm's probe: PASS at both clocks, every move under 1.5 × its
   optimum (0.85).
2. The wire check: 0 faults, 0 time losses (0.9); the arm's time a move
   within ±5% of the head's (0.7); the arm's result within ±35 Elo (0.9).
3. The gate as above.

**Evidence layer:** the probe and the wire check are milliseconds and
counts; only the gate is Elo.

## Results (2026-10-23, agent-run, idle host)

**The probe on the PGO binaries** (`tm_rec_probe_r20.txt`; host 2.9% busy,
no engine, harness or build process):

| Clock | Position | Head | The arm (all four parts) |
|---|---|---|---|
| `60000+600` | rec1 | 4.4 s (0.93 ×), depth 18 | **2.3 s (0.49 ×)**, depth 16 |
| `60000+600` | rec3 | **30.3 s, the hard maximum** | **2.5 s (0.56 ×)**, depth 22 |
| `180000+2000` | rec1 | 40.0 s (2.63 ×), depth 19 | **8.3 s (0.55 ×)**, depth 17 |
| `180000+2000` | rec3 | **98.0 s, the hard maximum** | **98.0 s, the hard maximum**, depth 24 lowerbound 1624, 255 M nodes |

- **Seven of eight rows pass; one does not.** On the PGO arm, 1.3 × faster
  than the `diag,tune` build that passed all eight, rec3's depth 23
  completes under half the target, so the freeze has not engaged when depth
  24 begins; depth 24 then walks a cascade (the squared window at a score
  near 1,100 is 130 wide, the walk to 1,624 and beyond four or five
  windows) and each window at that depth costs tens of millions of nodes,
  the per-window cost RAR-R19 left open (one window at depth 21–24 in a rook
  ending: 75–170 M against the oracle's 1–3 M). Prediction 1 (PASS at both
  clocks, every move under 1.5 × optimum, 0.85) **missed on that row** and
  held on the other seven. Nothing in the cluster bounds one iteration that
  begins before the freeze engages; Stockfish's does not either, its windows
  at that depth are simply cheap.
- **What would bound it:** either the search-cluster half (the tree's cost
  per window at depth 20 and beyond in rook endings: pruning rates a third
  to a tenth of the oracle's, twice the singular attempts, effective depth
  twice nominal), a programme of its own, or an in-iteration clock piece,
  RAR-R18's fail-high stop at twice the target (probe PASS there, +12.2 ±
  8.8 against the stop at the target, Elo-neutral against no stop), which
  the maintainer set aside and which, with the cluster, would fire only on
  the rare iteration that outruns both the freeze and the window.

**The wire check** (`tools/results/rar-r20-wire`, 200 games at `3+0.03`,
host idle, 0 faults, 0 time losses, the runner's recount agreeing):
**−52.5 ± 23.9 Elo** (−107.5 ± 48.2 nElo) for the arm against the head.
Prediction 2 held on faults and time use and **missed badly on the result**
(within ±35: read −52.5 with its whole interval below −28). The unfitted
cluster is a large strength loss at `3+0.03`: bench 0.643 × says it searches
a different tree, not a faster one, and C.10's constants were fitted around
the old one. Two equal-speed decompositions on a tune build follow.
