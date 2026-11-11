# RAR-S115 — D.6.4's gate: a fail-low node stores no move, SPRT `[-5,5]`

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-11, before any game. The run is the
  maintainer's command.
- **Kind:** a playing-strength gate for a repair, AGENTS' repair case 3.
- Disposition: open — registered, no game played.
- **Inherits:** [RAR-S113](RAR-S113.md), the readiness read: 64% of the
  source's `Upper` stores change the stored move fresh and 53% game-like, and
  a TT move from an `Upper` entry is at 10% and 13% of interior nodes. The
  change searches 0.559 of the source's interior nodes fresh and 0.870
  game-like (0.905 at the median). LMR without a TT move goes from 20% to 57%
  of its computations and IIR from 12% to 15% of nodes. WAC is 237 against
  239 at 100k and 265 against 264 at 400k, with no lost tactic.
  [RAR-S111](RAR-S111.md): separable from D.6.3. [RAR-S114](RAR-S114.md):
  D.6.3's typing lost its gate and is reverted, so this lands on the source
  tree. The four questions are answered in RAR-S113 and not repeated here.
- **Order:** D.6.4 runs before D.6.3's registered rework (maintainer decision
  2026-11-11): its readiness and `bench 13` are registered for this tree, and
  the rework has neither yet.

## The arms

- **Candidate:** `45adef3` on `d6`, "Store a fail-low node's upper bound
  without a move": the main search's store passes no move for an `Upper`
  bound, so the table keeps the move it holds, and nothing else in the engine
  changes. `bench 13` **12,785,308 / EBF 2.513**, RAR-S113's arm 4.
  `a_fail_low_store_keeps_the_stored_move` pins it: a fail-low root store
  keeps an earlier entry's move, carries none in a fresh table (with the old
  store it carries the first move searched), and a fail high still stores its
  move.
- **Baseline:** `7a3087a` on `d6`, the head before the engine commit. Its
  engine behaviour is `ee0b3e4`'s: `91265f2` restored ProbCut's typing with
  comments and a test-only recorder, and the two test commits after it change
  no engine input. `bench 13` **13,038,236 / EBF 2.540**, the declared
  fingerprint.
- **Qualified on the final trees** (2026-11-11, each check once, exit status
  read directly): candidate `cargo fmt --check` 0;
  `cargo clippy --all-features --all-targets` 0 with no warning; debug tests
  0 (523 passed, 12 ignored); release tests 0 (525 passed, 11 ignored).
  Baseline debug tests 0 (522 passed, 12 ignored) and release tests 0 (524
  passed, 11 ignored). Both benches from a default-feature release build.
- **Test changes before the engine commit**, each in its own commit with its
  measurement:
  - `9210e5e`: the not-TT-PV double and triple singular margins' liveness
    probes move to `SEMI_SLAV` at depths 11 and 12. With the store change
    `TWO_PIECES` at 10 and 12 no longer moves either; read on both trees at
    depths 8 to 13 across the five positions, `SEMI_SLAV` is live for the
    double margin at 10 to 13 and the triple at 11 to 13 on both.
  - `7a3087a`: `a_helper_searches_every_line` asserted exactly three exact
    root scores in a MultiPV 3 mate search. With the store change the final
    iteration holds five: the three lines' moves and two that raised alpha
    before a better move overtook them (643 and 561 against mate). It now
    asserts the stated invariant, that the three best root moves are exact;
    with one line it fails.

## Binaries

Built by `tools/build_test.ps1 -SourceRoot` from a clean worktree at each
commit (`analysis/artifacts/d64-gate-2026-11-11/run_s115.ps1 -Step build`,
log `build.log` beside it), each with its sidecar:

- candidate `tools/test_engines/rarog-d64cand-pext-pgo.exe`, SHA-256
  `e055f425a75fae1fdab381d402291937f01ec4e16f8d1d11bbe4d644ae7db309`,
  `id name Rarog dev-20261111-45adef3a`, clean tree, bench 12,785,308 / EBF
  2.513;
- baseline `tools/test_engines/rarog-d64base-pext-pgo.exe`, SHA-256
  `80ade92502736e7059fa77d03f11144e99e803e385e0b8818137422b35041ab6`,
  `id name Rarog dev-20261111-7a3087aa`, clean tree, bench 13,038,236 / EBF
  2.540.

Both are `pext-pgo` builds with `rustc 1.99.1 (58cf8f9c3 2026-10-13)`. The
hashes, names and benches were read from the executables themselves and
agree with their sidecars.

**Command,** dry-run 2026-11-11 with exit 0 (host 2% busy with no engine,
harness or build process; the pinned runner colosseum-cli 0.5.0,
`81EA6CBA…` at `8fa63cc`; both fingerprints, the flavour and the compiler
checked; the resolved configuration matching policy
in every checked field, with side A the candidate's executable and side B the
baseline's, `[-5,5]` normalized, 6,000 pairs, seed 20261112;
`tools/results/colosseum_sprt_rar-s115-gate_20261111_130514.dry-run.json`):

```
pwsh -NoProfile -File D:/code/rarog/analysis/artifacts/d64-gate-2026-11-11/run_s115.ps1 -Step sprt
```

It runs `tools/colosseum.ps1 -Mode sprt -Bracket repair` with the arms above,
`-MaxPairs 6000 -Seed 20261112 -ExpectBench 12785308,13038236`, into
`tools/results/rar-s115-gate`.

## Exposure (AGENTS, a repair's registration)

A main-search node that fails low stores an `Upper` bound, and in a search of
depth 2 or more some node below the root does, so a move searched to depth 2
or more reaches the repair's domain. From RAR-S114's gate, the most recent one
(`tools/results/rar-s114-gate`, the 9,662 games of its official sample, each
game's PGN checked against its SHA-256 in `games.jsonl`), read by
`analysis/artifacts/d64-gate-2026-11-11/exposure.py` (SHA-256 `6a6c6a01…`,
log `exposure.log`):

| | count | share |
|---|---:|---:|
| games with a move in the domain | 9,662 | 100.00% |
| searched moves in the domain | 1,211,887 of 1,215,591 | 99.70% |
| of them with a mate score | 77,646 | 6.39% |
| moves searched under depth 2 | 3,704 | 0.30% |

So every gate game reaches the repair, and the cap and the harm rule are read
against full exposure. Inside a search the reach is RAR-S113's: a TT move from
an `Upper` entry is at 13% of interior nodes game-like, and half the source's
`Upper` stores change the stored move.

## The gate (frozen 2026-11-11, before any game)

- **Case 3** of AGENTS' *Gating*, as PLAN states for D.6.4: the change moves
  shared search signals in the rated configuration (the TT move at the next
  visit, IIR, the razoring guard, the no-TT-move LMR terms, the first-move
  slot and the fail-low parent-reward factor), and could be large either way:
  the tree is 0.87 of the source's game-like. Case 1 does not apply (the
  fingerprint moves outside any static domain) and case 2 does not (the
  change is to shared signals).
- **Runner:** Colosseum CLI, the pinned 0.5.0, through `tools/colosseum.ps1
  -Mode sprt -Bracket repair` (`sprt-repair.toml`, `[-5,5]` nElo,
  pentanomial, `model=normalized`, α = β = 0.05). It is never changed
  mid-way.
- **Conditions:** `3+0.03`, margin 20 ms, Hash 64, one thread, the UHO book in
  random order, no adjudication, the wrapper's slot count and pinned cores,
  **cap 6,000 pairs** (12,000 games, the case-3 cap of RAR-S62, RAR-S94 and
  RAR-S114), seed **20261112**, `tools/results/rar-s115-gate`, `-ExpectBench
  12785308,13038236`. Bounds, cap, book and adjudication never change after
  a game is seen. The runner's fault policy is adopted by reference.
- **Games to a bound (RAR-M10):** about 7,100 games at ±5, 11,800 at ±3 and
  none at 0; a near-zero repair runs to the cap. At about 5,950 games an hour
  (RAR-S114) the cap is about 2 hours.
- **Use (frozen):**
  - **H1, or the cap without H0:** `45adef3` stays on `d6`, the fingerprint
    record moves to 12,785,308 / EBF 2.513, the changelog gets its entry and
    D.6.4 closes. A cap stop is recorded with its interval as "no harm beyond
    about −5", never as a gain.
  - **H0** (the change costs at least 5 nElo): the engine change is reverted
    on `d6` in its own commit, and the defect stays open in D.6.4. The
    registered rework, as RAR-S113 names it, is the store change with the
    constants fitted around the defect refitted: IIR's depth and the
    no-TT-move LMR terms. It is a new registration with its own gate, not a
    rescue inside this one.
- **Then:** D.6.3's registered rework, the ProbCut typing with the fail-low
  reward kept, is implemented on whichever tree this gate leaves.

## Diagnostics, recorded before any game and not deciding

- The won-ending clock test (D.6.20's acceptance test) and the transposing
  rook (D.6.17) are not re-read for this arm; neither decides.

## PRE-REGISTERED PREDICTION (frozen before any game)

1. **The outcome:** H1, **0.35**; H0, **0.25**; the cap without either bound,
   **0.40**. For a gain: the donor keeps the old move at a fail low, the
   mechanism is active at 13% of interior nodes game-like, and the same depth
   costs 13% fewer nodes, which at a fixed clock buys depth. Against one: the
   fewer nodes come from more reduction (LMR without a TT move from 20% to
   57%), IIR and the no-TT-move LMR terms were fitted with the defect in place,
   and RAR-S114 showed a typing whose donor form was expected to help lose
   about 4 nElo.
2. **nElo at the stop:** **+1**, 80% band **[−9, +11]**. If a bound is
   crossed early the estimate is a first-crossing estimate, biased away from
   zero.
3. **Games to the stop:** a bound with probability 0.60, median 6,000 games;
   otherwise the cap.
4. **In the games' own figures** (the manifest's per-side search, which
   describe the games and decide nothing): mean depth higher than the
   baseline's by +0.1 to +0.6, 0.7; nodes a move within ±5% of the
   baseline's, 0.7.
5. **Most likely failure mode:** the deeper search does not pay for its
   reductions, the cost landing in the no-TT-move LMR population the
   constants were not fitted on; the games then read near zero or H0 with the
   depth up, and the rework's refit is the next step.
