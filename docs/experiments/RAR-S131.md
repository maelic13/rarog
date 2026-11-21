# RAR-S131 — D.7.8.2: quiescence and evaluation-only entries at the donor's depth, 2,000 games — REGISTERED 2026-11-19, before any game; READ 2026-11-19: nElo +1.2 ± 15.2, kept

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-19, before any game. The run is the
  maintainer's command.
- Disposition: retained — read 2026-11-19: nElo +1.2 ± 15.2, above the −13 harm
  line, so the change is kept; fingerprint 3,348,508 / EBF 2.329. Inside
  the prediction's 80% band.
- **Kind:** PLAN D.7.8.2, a repair inside D.7's one cluster: no gate of its
  own (D.7, *How the open steps are worked*), one 2,000-game read against
  the build the step started from, under AGENTS' case-2 harm rule.
- **Inherits:** the audit's finding A5 (`analysis/d7_audit_2026-11-19.md`)
  and D.7.8.2's card (PLAN): every reader of a stored depth asks about
  depth 1 or more and reads the same under either encoding; the table's
  keep rule and victim rank are what change.

## The change (engine `e363908`)

A quiescence result and an evaluation-only entry are both stored at −1, the
donor's `TtDepth::SOME`, where Rarog stored them at 0 and −2. The keep rule
(`depth + 4 + 2·pv <= entry.depth`) then keeps a same-position entry three
plies deep (five on the PV) over a quiescence store, as the donor's table
does, and the two kinds rank level as victims. The null-move bound's hand
conversion of the quiescence depth (TT-7) goes. `bench 13` moves from
3,278,226 / EBF 2.313 to **3,348,508 / EBF 2.329**.

## Arms and run (frozen)

Built 2026-11-19 by `tools/build_test.ps1` (pext, PGO, rustc 1.100.0), each
from a clean tree:

| Arm | Commit | `id name` | Bench | SHA-256 |
| --- | --- | --- | --- | --- |
| `d782cand` | `e363908` | `dev-20261119-e3639085` | 3,348,508 | `500a09bd72fbe7e95e3938316117afda7193bc320632d329c09cf2d3f5c1331b` |
| `d782base` | `3e27052` | `dev-20261119-3e27052d` | 3,278,226 | `b6562d07f4e54aaade7cc327769d49069952368eb431a518dfc6590cc136434a` |

`d782base` is D.7.8.1's kept state (engine `e63b652`; `3e27052` adds only
a test commit). Colosseum `match` through `tools/colosseum.ps1`, 2,000
games at `3+0.03`, Hash 64, one thread, the wrapper's slot count, the UHO
book in random order, no adjudication, seed 20261122,
`tools/results/rar-s131`. The runner is Colosseum and does not change. The
dry run passed every preflight
(`tools/results/colosseum_match_rar-s131_20261119_201210.dry-run.json`).

```powershell
./tools/colosseum.ps1 -Mode match -EngineA tools/test_engines/rarog-d782cand-pext-pgo.exe -EngineB tools/test_engines/rarog-d782base-pext-pgo.exe -NameA d782cand -NameB d782base -ExpectBench 3348508,3278226 -Dir tools/results/rar-s131 -Games 2000 -Seed 20261122
```

**Exposure:** the table serves every searched node, so every game reaches
the change; how often the keep rule binds is not counted, and no database
query narrows it.

**Validity:** every game a rules result; a run with time losses or engine
faults in more than 0.5% of its games is void.

## Reading (frozen)

From the manifest's nElo (AGENTS case 2): **below −13 nElo** the change is
reverted and recorded, the finding left open in D.7.8.2; **anything else**
keeps it. At about 7.8 nElo standard error the rule catches a true −20
with probability 0.82; "kept" means no harm beyond roughly −20 nElo.

## Found before the games

The tests rebuilt on the kernel caught one side effect at zero games: in
KQK (`k7/8/8/8/8/2K5/8/1Q6 w`, Stockfish's mate in four) the search now
reports a mate in six from depth 10 to 17 and the shortest only at depth 18,
where it reported the mate in four from depth 9. The quiescence half alone
does it (built alone: the same bench and the same mates); the kept deeper
entry holds the longer mate. Reckless's search reports a mate in five at
every depth from 9 to 18. The mate is still found and played. The exact
check is an ignored test naming this finding (`3e27052`).
*Superseded 2026-11-19 by D.7.8.3 (RAR-S132):* with the root in the
kernel's own order, ties included, the mate in four is reported from
depth 8; the longer mate needed the classic root order with this keep
rule, and the check runs again.

## PRE-REGISTERED PREDICTION (frozen 2026-11-19, before any game)

- **`d782cand`:** **+1 nElo**, 80% band **−14 to +16**; P(below −13) 0.10.
- **Basis:** the keep rule binds only when a quiescence store meets a
  same-position entry three or four plies deep, which the tree's +2.1% says
  is uncommon; the donor's form keeps more searched information, slightly
  in its favour; against it, longer mates in won endings may cost a little
  conversion.
- **Most likely failure mode:** none visible at 2,000 games: the interval
  spans zero either way.
- **Evidence layer:** games, both arms PGO builds of the same toolchain.

## Result (read 2026-11-19)

Run by the maintainer 2026-11-19, 22 minutes, Colosseum CLI 0.5.0 (pinned
`8fa63cc`). The runner's report and the wrapper's manifest
(`tools/results/colosseum_match_rar-s131_20261119_202611.manifest.txt`):

- "Elo +0.7 +/- 8.8; nElo +1.2 +/- 15.2 over 2000 paired games"
- `d782cand` 498 W / 494 L / 1008 D; pentanomial 19, 249, 466, 241, 25
  ("recount of games.pgn agrees"); faults none, engine faults none.
- search, `d782cand`: depth 17.94 mean, 127,151 nodes and 61.3 ms a move,
  2,072,987 nps; `d782base`: depth 17.99 mean, 126,939 nodes and 61.3 ms
  a move, 2,071,934 nps.

**Reading:** +1.2 nElo is above the −13 harm line: the change is kept.

### Calibration (appended after the result)

Predicted +1 nElo, 80% band −14 to +16; read +1.2, at the centre. As
predicted the read resolves nothing finer than the harm rule; the KQK
mate-distance finding showed no visible cost at this control.
