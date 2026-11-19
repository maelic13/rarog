# RAR-S132 — D.7.8.3: the root ordered by the kernel's own scorer, 2,000 games and a four-thread fault smoke — REGISTERED 2026-11-19, before any game; READ 2026-11-19: nElo +4.1 ± 15.2, no fault at four threads, kept

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-19, before any game. Both runs are
  the maintainer's commands.
- **Disposition:** read 2026-11-19: nElo +4.1 ± 15.2, above the −13 harm
  line, and no fault or time loss in the 200 four-thread games, so the
  change is kept; fingerprint 3,320,355 / EBF 2.317. Inside the
  prediction's 80% band.
- **Kind:** PLAN D.7.8.3, a repair inside D.7's one cluster: no gate of its
  own (D.7, *How the open steps are worked*), one 2,000-game read against
  the build the step started from, under AGENTS' case-2 harm rule, and 200
  games at four threads read for faults alone.
- **Inherits:** the audit's finding A8 (`analysis/d7_audit_2026-11-19.md`,
  inventory R6): the root's list was scored by the classic picker's scorer
  over the kernel's tables, its last user once D.7.7 had run.

## The change (engine `4271031`)

The root's list is written in the order the kernel's staged picker hands
moves over at every other node, ties included: the table move, the noisy
moves that clear the good-noisy bar, the quiets, the noisy moves that
failed it. The moves come from the staged generators and are put in order
by the staged picker's selection step, so the good-noisy bar's count of
good moves so far is the staged picker's. The whole list is ordered and
then narrowed to the root's lines, as the donor's root skips a move
outside its lines after its picker hands it over. The pool's shared scores
and a helper's rotation stay. The classic scorer and its constants go.
`bench 13` moves from 3,348,508 / EBF 2.329 to **3,320,355 / EBF 2.317**.

## Arms and runs (frozen)

Built 2026-11-19 by `tools/build_test.ps1` (pext, PGO, rustc 1.100.0), each
from a clean tree:

| Arm | Commit | `id name` | Bench | SHA-256 |
| --- | --- | --- | --- | --- |
| `d783cand` | `4271031` | `dev-20261119-4271031f` | 3,320,355 | `6f1e501fde361b10b0c2818015a79752610aaccc77a7a32bc13779d08ed405dc` |
| `d783base` | `e363908` | `dev-20261119-e3639085` | 3,348,508 | `500a09bd72fbe7e95e3938316117afda7193bc320632d329c09cf2d3f5c1331b` |

`d783base` is D.7.8.2's kept state, the file
`rarog-d782cand-pext-pgo.exe` that RAR-S131 played. Colosseum `match`
through `tools/colosseum.ps1`; the runner is Colosseum and does not change.

**The read:** 2,000 games at `3+0.03`, Hash 64, one thread, the wrapper's
slot count, the UHO book in random order, no adjudication, seed 20261123,
`tools/results/rar-s132`. The dry run passed every preflight
(`tools/results/colosseum_match_rar-s132_20261119_215257.dry-run.json`).

```powershell
./tools/colosseum.ps1 -Mode match -EngineA tools/test_engines/rarog-d783cand-pext-pgo.exe -EngineB tools/test_engines/rarog-d782cand-pext-pgo.exe -NameA d783cand -NameB d783base -ExpectBench 3320355,3348508 -Dir tools/results/rar-s132 -Games 2000 -Seed 20261123
```

**The fault smoke:** 200 games at four threads a side, the same clock,
Hash 64, three games at a time on four cores each, the explicit placement
of one logical CPU per physical core (RAR-M82), seed 20261124,
`tools/results/rar-s132-4t`. The dry run resolved to `placement: explicit
(2,4,6,8,16,18,20,22,24,26,28,30)`
(`tools/results/colosseum_match_rar-s132-4t_20261119_215302.dry-run.json`).

```powershell
./tools/colosseum.ps1 -Mode match -EngineA tools/test_engines/rarog-d783cand-pext-pgo.exe -EngineB tools/test_engines/rarog-d782cand-pext-pgo.exe -NameA d783cand -NameB d783base -ExpectBench 3320355,3348508 -Dir tools/results/rar-s132-4t -Games 200 -Seed 20261124 -Threads 4 -Concurrency 3 -ExtraArgs @('--cores-per-game','4','--placement','2,4,6,8,16,18,20,22,24,26,28,30')
```

**Exposure:** every search orders its root, so every game reaches the
change. At four threads the helpers' shared scores and rotation reorder
the same list.

**Validity:** every game a rules result; a run with time losses or engine
faults in more than 0.5% of its games is void.

## Reading (frozen)

From the read's manifest nElo (AGENTS case 2): **below −13 nElo** the
change is reverted and recorded, the finding left open in D.7.8.3;
**anything else** keeps it. At about 7.8 nElo standard error the rule
catches a true −20 with probability 0.82; "kept" means no harm beyond
roughly −20 nElo.

The smoke is read for faults alone, not for Elo: **any engine fault or
illegal move by `d783cand`**, or time losses in more than 0.5% of its
games, stops the step until the cause is found, whatever the read says.

## Found before the games

- **Ties at the root move the tree by a third.** A first form scored the
  same moves at the same scores but broke ties in the full move
  generator's order rather than the staged picker's: `bench 13` read
  4,408,144 / EBF 2.350 (+31.6%; one position grew from 166,254 nodes to
  830,025, 22 grew and 18 shrank), and the queen mate in three
  (`8/k7/8/8/3K4/8/8/1Q6 w`) was reported as a mate in seven at depth 10
  and as three only at 16, where `d783base` and Reckless's search report
  three from depth 8. The kept form, ties included, reports three from
  depth 8. The test that the root's order is the staged picker's now
  compares moves, not scores.
- **KQK's shortest mate is back.** With the kept form, KQK's mate in four
  (`k7/8/8/8/8/2K5/8/1Q6 w`) is reported from depth 8, where `d783base`
  reports six until depth 18 (RAR-S131, *Found before the games*). The
  longer mate needed the classic root order with the donor's keep rule;
  the check that D.7.8.2 ignored runs again.
- **KRK finds its mate later and longer.** In `k7/8/8/8/2K5/8/8/7R w`
  (Stockfish: a mate in five) the search first reports a mate at depth 12,
  a mate in nine to depth 18, where `d783base` reports six from depth 10
  and Reckless's search seven from depth 9 and six from 12. The mate is
  found and the line's score is its length; the check of that score read
  at depths 11 and 14 and now reads at 14 and 18 (`2f47a78`, verified on
  `d783base`'s engine first).

## PRE-REGISTERED PREDICTION (frozen 2026-11-19, before any game)

- **`d783cand`:** **+2 nElo**, 80% band **−13 to +17**; P(below −13) 0.10.
- **Basis:** the root now searches its moves in the order every other node
  uses, the order the kernel's seeds were fitted with; the classic
  scorer's features were the same and its weights close, so most roots
  open on the same few moves. The tree moves −0.8%. The endgame mate
  distances above move both ways and should not show at this count.
- **Most likely failure mode:** none visible at 2,000 games: the interval
  spans zero either way.
- **Smoke:** no fault in 200 games, P 0.95; the helpers' code is unchanged
  and reorders the list as before.
- **Evidence layer:** games, both arms PGO builds of the same toolchain.

## Result (read 2026-11-19)

Both run by the maintainer 2026-11-19, Colosseum CLI 0.5.0 (pinned
`8fa63cc`).

**The read** (22 minutes;
`tools/results/colosseum_match_rar-s132_20261119_215710.manifest.txt`):

- "Elo +2.3 +/- 8.3; nElo +4.1 +/- 15.2 over 2000 paired games"
- `d783cand` 502 W / 489 L / 1009 D; pentanomial 14, 233, 498, 236, 19
  ("recount of games.pgn agrees"); faults none, engine faults none.
- search, `d783cand`: depth 18.22 mean, 125,721 nodes and 60.8 ms a move,
  2,066,853 nps; `d783base`: depth 18.25 mean, 126,064 nodes and 60.9 ms
  a move, 2,070,785 nps.

**The fault smoke** (11 minutes, placement explicit as registered;
`tools/results/colosseum_match_rar-s132-4t_20261119_221859.manifest.txt`):
200 of 200 games scored, faults "time: 0-0; other: 0-0", engine faults
none; at four threads `d783cand` 8,784,602 nps and depth 20.22 mean,
`d783base` 8,790,418 and 20.32. Its score is not read.

**Reading:** +4.1 nElo is above the −13 harm line and the smoke has no
fault: the change is kept.

### Calibration (appended after the result)

Predicted +2 nElo, 80% band −13 to +17; read +4.1, near the centre. The
smoke's no-fault prediction held. The read resolves nothing finer than
the harm rule; the static root order is not the donor's live re-scoring
(found after the games: the donor re-scores the root's remaining moves
as each is searched), which this read does not test.
