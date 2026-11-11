# RAR-S122 — D.6.7's gate: every line bounded at twice the iteration's depth, SPRT `[-5,5]`

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-11, before any game. The run is the
  maintainer's command.
- **Kind:** a playing-strength gate for a repair, AGENTS' repair case 3.
- **Inherits:** [RAR-S121](RAR-S121.md), the screen.
  - No search reaches the ply cap at the gate games' node counts. 0.109%
    reach it at ten-thirds of them, and 1.962% at twenty times (22 of 60
    games).
  - At the cap, 22% of nodes are in check and 15% are draws by rule, and
    each returns a static eval.
  - Depth gained outside the extension budget carries 88% of those lines'
    plies beyond the iteration's depth. Quiescence carries none.

  RAR-S121 gives the four questions and the gate's correction from case 1 to
  case 3.

## The repair

The extension budget's own comment states a contract that the search did not
keep: "no line grows deeper than twice the iteration plus quiescence". The
budget holds positive extensions to the iteration's depth. Three other paths
deepen a line, and nothing held them:
- the late-move reduction's room above nominal depth: two plies, and two
  more at a PV parent;
- the re-search's extra ply;
- hindsight's ply.

The candidate keeps the contract. A node's depth is held to
`2 × iteration − ply`, before the hand-off to quiescence, and again after
hindsight. Nothing else changes.

**One choice, stated.** RAR-S121's record asked the implementation to split
the gain by source before choosing the bound. The bound chosen does not
depend on the split. It holds the sum of every out-of-budget gain to the
room the extensions left, whichever path produced it. That is the contract
as the code states it, and the source split would only aim a narrower bound
at one path. The split was not run.

## The arms

- **Candidate:** `aee0b3b` on `d6`, "Bound every line at twice the
  iteration's depth". `bench 13` **11,477,040 / EBF 2.537** (−12.0% nodes).
  `no_line_searches_past_twice_the_iteration_depth` pins it. In the race
  position, at iterations 1 to 10, no node's line exceeds its bound, the
  bound cuts some nodes, and the selective depth stays under the cap.
  With the clamp disabled the test fails ("a line overran its bound by 5 at
  depth 6").
- **Baseline:** `5f21606` on `d6`, the head before the engine commit. Its
  engine behaviour is `ee0b3e4`'s: the three commits after `9efa61e` change
  tests only. `bench 13` **13,038,236 / EBF 2.540**, the declared
  fingerprint.
- **Test changes before the engine commit,** each in its own commit, read on
  both trees:
  - `bd37b17`: the null-move, ProbCut and depth-4 singular probes searched a
    node at ply 1 with depth 8 or 4 and an iteration depth of 0. A real
    search reaches such a node only in iteration 9 or 5 or later, and the
    bound reads that iteration. They now set it. All four pass on both
    trees, debug and release.
  - `61a7cb4`: two singular liveness probes no longer moved on the
    candidate, `CoreSingDoubleNotTtPv` at `SEMI_SLAV` 11 and
    `CoreLmrSingularCap` at `TWO_PIECES` 9. A scan of five positions at
    depths 8 to 14 on both trees found the smallest pins live on both:
    `PAWN_ENDING` 9 and `TWO_PIECES` 10.
  - `5f21606`: `a_helper_searches_every_line` ranked a lower bound among the
    three best root scores. At equal depth the pool keeps a fail-high's lower
    bound over the upper bound its re-search returned (a root move at 667
    against an exact 644). The test now leaves lower bounds out of the
    ranking. It passes on both trees, and with one line instead of three it
    still fails on both.
- **Qualified on the final candidate tree** (each check once, exit status read
  directly):
  - `cargo fmt --check` 0;
  - `cargo clippy --all-features --all-targets` 0 with no warning;
  - debug tests 0 (524 passed, 12 ignored);
  - release tests 0 (526 passed, 11 ignored);
  - a default-feature release build, then `bench 13`.

## What the bound does, read on the candidate

RAR-S121's instrument on `aee0b3b`, built with `--features diag` (`bench 13`
11,477,040), on the same 60 replayed gate games and the race position. The
script is `analysis/artifacts/d67-impl-2026-11-11/read_cand.py`, log
`report_cand.log`.

| | candidate | baseline (RAR-S121) |
|---|---:|---:|
| race position, depth 10: selective depth | 30 | 127 |
| race position: cap nodes | 0 | 7,345,137 |
| ×1: interior nodes the bound cuts | 0.489% | — |
| ×1: searches with a cut | 94.3% (all 60 games) | — |
| ×20: searches reaching the cap | 2 (235 cap nodes) | 72 (1,589,474) |

The two ×20 searches that still reach the cap are deep endgame iterations, at
depths 63 and 84, where twice the iteration reaches the cap by itself. That
residual is the edge PLAN always expected, from iteration 63 on, and it is
outside this repair. At the cap such a node still returns a static eval.

**Tactics** (RAR-M85's rule, against RAR-S113's arm 0, the same source):
- WAC at 100k: 238 against 239, no canary lost.
- WAC at 400k: 260 against 264.
- Gross tactical harm: no.

## Exposure (AGENTS, a repair's registration)

The bound acts at the gate's own time control:
- it cuts 0.489% of interior nodes;
- it cuts in 94.3% of searches;
- it cuts in every one of the 60 replayed games of RAR-S115's gate.

So a `3+0.03` gate reads the bound's effect on the tree in nearly every
game. The cap repair it carries is not visible at `3+0.03`: no search there
reaches the cap. Its value lies at longer time controls, 1.96% of searches at
`60+0.6`'s node counts. An H0 here therefore reverts the bound for its effect
on the tree at STC. The defect stays open in D.6.7 by AGENTS' revert rule.

## Binaries

Built by `tools/build_test.ps1 -SourceRoot` from a clean worktree at each
commit (`analysis/artifacts/d67-gate-2026-11-11/run_s122.ps1 -Step build`,
log `build.log`), each with its sidecar:
- candidate `tools/test_engines/rarog-d67cand-pext-pgo.exe`, SHA-256
  `dbbd83b9ed6de5fa7e4b259eff54e12c2efa08ecee7e6668130079d2a18896a6`,
  `id name Rarog dev-20261111-aee0b3b9`, bench 11,477,040 / EBF 2.537;
- baseline `tools/test_engines/rarog-d67base-pext-pgo.exe`, SHA-256
  `7d4940c0f2a1c8eef8d3cc21795f9da529ec51163f636e2023e9785bab1bccb7`,
  `id name Rarog dev-20261111-5f21606b`, bench 13,038,236 / EBF 2.540.

Both are `pext-pgo` builds with `rustc 1.99.1 (58cf8f9c3 2026-10-13)`. The
hashes, names and benches were read from the executables themselves.

## Gate (frozen)

**Command,** dry-run 2026-11-11 with exit 0. Its preflight:
- the host idle at 0%, with no engine, harness or build process;
- the pinned runner, colosseum-cli 0.5.0, `81EA6CBA…` at `8fa63cc`;
- both fingerprints, the flavour and the compiler checked;
- the resolved configuration matching policy in every checked field.

The run: `[-5,5]` normalized, alpha = beta = 0.05, a cap of 6,000 pairs
(12,000 games), `3+0.03` with a 20 ms margin, Hash 64, one thread,
concurrency 14, no adjudication, the `UHO_Lichess_4852_v1` book with seed
20261114. The resolved configuration is
`tools/results/colosseum_sprt_rar-s122-gate_20261111_180439.dry-run.json`.

```
pwsh -NoProfile -File D:/code/rarog/analysis/artifacts/d67-gate-2026-11-11/run_s122.ps1 -Step sprt
```

**Rule.**
- **H1, or the cap without H0, keeps the bound.** `bench 13` 11,477,040
  becomes the declared fingerprint.
- **H0 reverts it.** The defect stays open in D.6.7, and a rework is a new
  registration, not a rescue inside this one.
- Bounds, cap, book and adjudication do not change after the first game.

## PRE-REGISTERED PREDICTION (frozen before any game)

1. The gate's interval holds 0. nElo between −4 and +4 at the stop: 0.6.
   Its sign: positive 0.5. The prior is TS-1's +3.88 ± 3.26 in Triumviratus,
   a narrower bound in another engine, and RAR-S115's lesson that a smaller
   tree is not strength. The node share the bound cuts at STC is small,
   0.49%.
2. The stop: H1 0.3, H0 0.3, the cap 0.4. RAR-M86 puts a case-3 gate at a
   true zero at 34%, 34% and 32%.
3. **Most likely failure mode:** the bound cuts the PV's +2 and the
   re-search ply, which the search's ordering relies on near the leaves of
   long PV lines. It shows as a small loss at STC, with no gain to offset it
   where the cap would bind.
