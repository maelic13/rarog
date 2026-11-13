# RAR-S114 — D.6.3's gate: ProbCut's verification children typed as all nodes, SPRT `[-5,5]`

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-11, before any game. The run is the
  maintainer's command.
- **Kind:** a playing-strength gate for a repair, AGENTS' repair case 3.
- Disposition: rejected — H0 accepted 2026-11-11 at 9,662 games, nElo −3.7 ±
  6.9; reverted (`91265f2`).
- **Inherits:** [RAR-S112](RAR-S112.md), the readiness read: ProbCut's
  verification children fail high 2.19% fresh and 2.91% game-like and are
  typed `cut_node = true`; typed `false` they issue no fail-low parent reward
  (1.85% and 1.55% of the noisy-history updates), ProbCut cuts 1.8 and 4.4
  points less often per node, and the tree has no one direction (0.958 fresh,
  1.007 game-like). [RAR-S110](RAR-S110.md): the null-move child keeps `true`.
  [RAR-M85](RAR-M85.md): a later settle is a diagnostic. The four questions
  are answered in RAR-S112 and not repeated here.

## The arms

- **Candidate:** `d15050a` on `d6`, "Type ProbCut's verification children as
  all nodes": both ProbCut `negamax` calls, the verification and its
  re-search at `probcut_beta`, pass `cut_node = false`, and nothing else in
  the engine changes. `bench 13` **13,148,505 / EBF 2.559**, which is arm 2 of
  `analysis/arm_patches/d63-d64-rar-s110-screen.patch`. NODE-14's unit test,
  `each_recursive_call_types_its_child_by_its_rule`, pins the type every
  recursive call passes; with the old typing it fails on ProbCut's children.
- **Baseline:** `63813d8` on `d6`, the head before the engine commit. Its
  engine source is `ee0b3e4`'s, the same as `79f761d`'s; the three test
  commits and one documentation commit between them change no engine input.
  `bench 13` **13,038,236 / EBF 2.540**, the declared fingerprint.
- **Qualified on the final trees** (2026-11-11, each check once, exit status
  read directly): candidate `cargo fmt --check` 0;
  `cargo clippy --all-features --all-targets` 0 with no warning; debug tests
  0 (522 passed, 12 ignored); release tests 0 (524 passed, 11 ignored).
  Baseline debug tests 0 (521 passed, 12 ignored) and release tests 0 (523
  passed, 11 ignored). Both benches from a default-feature release build.
- **Test changes before the engine commit**, each in its own commit and each
  with its measurement in the commit and in PLAN:
  - `d41bae3`: the transposing-rook anchor is split. A promotion on f8 with a
    positive score at 100,000 nodes stays active; the queen-over-rook
    preference is D.6.17's ignored acceptance test. After `gxf8` with either
    piece, `Rxf8+ Bxf8` reaches the same position, a true tie; the
    candidate plays `g7f8r` (528 at depth 14) because the root keeps the
    rook it searched first.
  - `7b5ad7f`: the not-TT-PV double and triple singular margins' liveness
    probes move to `TWO_PIECES` at depths 10 and 12, live on both trees.
  - `b6914f0`: the won-ending clock test, red since `ee0b3e4` on the
    baseline too, is D.6.20's ignored acceptance test.

## Binaries

Built by `tools/build_test.ps1 -SourceRoot` from a clean worktree at each
commit (`analysis/artifacts/d63-gate-2026-11-11/run_s114.ps1 -Step build`,
log `build.log` beside it), each with its sidecar:

- candidate `tools/test_engines/rarog-d63cand-pext-pgo.exe`, SHA-256
  `b83c1a1bb7b23689c17f2384488b9b1961f4fd54380ac5ed85d2e72e3ba81496`,
  `id name Rarog dev-20261111-d15050af`, clean tree, bench 13,148,505 / EBF
  2.559;
- baseline `tools/test_engines/rarog-d63base-pext-pgo.exe`, SHA-256
  `f6ee7d44b7b58a628c73df79d9d2a5dd07d56dc14fc603efa7d3e35b2118b601`,
  `id name Rarog dev-20261111-63813d8a`, clean tree, bench 13,038,236 / EBF
  2.540.

Both are `pext-pgo` builds with `rustc 1.99.1 (58cf8f9c3 2026-10-13)`. The
hashes, names and benches were read from the executables themselves and
agree with their sidecars.

**Command,** dry-run 2026-11-11 with exit 0 (host 2% busy with no engine,
harness or build process; the pinned runner colosseum-cli 0.5.0, `81EA6CBA…`
at `8fa63cc`; both fingerprints, the flavour and the compiler checked; the
resolved configuration matching policy in every checked field, with side A
the candidate's executable and side B the baseline's, `[-5,5]` normalized,
6,000 pairs, seed 20261111;
`tools/results/colosseum_sprt_rar-s114-gate_20261111_103112.dry-run.json`):

```
pwsh -NoProfile -File D:/code/rarog/analysis/artifacts/d63-gate-2026-11-11/run_s114.ps1 -Step sprt
```

It runs `tools/colosseum.ps1 -Mode sprt -Bracket repair` with the arms above,
`-MaxPairs 6000 -Seed 20261111 -ExpectBench 13148505,13038236`, into
`tools/results/rar-s114-gate`.

## Exposure (AGENTS, a repair's registration)

ProbCut runs at a non-root, non-PV node of depth 4 or more, not in check,
whose beta is not a win, so a move searched to depth 5 or more with a score
that is not a mate reaches its domain below the root. From RAR-S108's gate,
the most recent one (`tools/results/rar-s108-gate`, the 11,108 games of its
official sample, each game's PGN checked against its SHA-256 in
`games.jsonl`), read by `analysis/artifacts/d63-gate-2026-11-11/exposure.py`
(SHA-256 `20ff5e62…`, log `exposure.log`):

| | count | share |
|---|---:|---:|
| games with a move in the domain | 11,108 | 100.00% |
| searched moves in the domain | 1,284,512 of 1,387,686 | 92.57% |
| moves with a mate score | 94,337 | 6.80% |
| moves searched under depth 5 | 8,837 | 0.64% |

So every gate game reaches the repair, and the cap and the harm rule are read
against full exposure. Inside a search the reach is RAR-S112's: ProbCut
children's rewards are 1.55% of the noisy-history updates game-like, and
ProbCut cuts at 45.4% of the nodes where it runs.

## The gate (frozen 2026-11-11, before any game)

- **Case 3** of AGENTS' *Gating*, as PLAN states for D.6.3: the typing changes
  shared search signals in the rated configuration (the child's TT-cutoff
  admission, the cut-node LMR terms, LDSE, the negative singular extension,
  every descendant's type, and the capture history through the removed
  reward), and could be large either way. Case 1 does not apply (the
  fingerprint moves outside any static domain) and case 2 does not (the
  change is to shared signals, not a contained classification).
- **Runner:** Colosseum CLI, the pinned 0.5.0, through `tools/colosseum.ps1
  -Mode sprt -Bracket repair` (`sprt-repair.toml`, `[-5,5]` nElo,
  pentanomial, `model=normalized`, α = β = 0.05). It is never changed
  mid-way.
- **Conditions:** `3+0.03`, margin 20 ms, Hash 64, one thread, the UHO book in
  random order, no adjudication, the wrapper's slot count and pinned cores,
  **cap 6,000 pairs** (12,000 games, the case-3 cap of RAR-S62 and RAR-S94),
  seed **20261111**, `tools/results/rar-s114-gate`, `-ExpectBench
  13148505,13038236`. Bounds, cap, book and adjudication never change after a
  game is seen. The runner's fault policy is adopted by reference.
- **Games to a bound (RAR-M10):** drift ≈ 8.3e-6 × 10 × true nElo, so about
  7,100 games at ±5, 11,800 at ±3 and none at 0; a near-zero repair runs to
  the cap. At about 5,000 games an hour the cap is about 2.5 hours.
- **Use (frozen):**
  - **H1, or the cap without H0:** `d15050a` stays on `d6`, the fingerprint
    record moves to 13,148,505 / EBF 2.559, the changelog gets its entry and
    D.6.3 closes. A cap stop is recorded with its interval as "no harm beyond
    about −5", never as a gain.
  - **H0** (the typing costs at least 5 nElo): the engine change is reverted
    on `d6` in its own commit, and the defect stays open in D.6.3. The
    registered rework is the typing with the fail-low reward kept: the two
    calls pass `false` and the ProbCut child still rewards the capture's noisy
    history on a fail low, which separates the two things this arm joins
    (RAR-S112). It is a new registration with its own gate, not a rescue
    inside this one.
- **Then:** D.6.4 (RAR-S113) is implemented on whichever tree this gate
  leaves: `bench 13` 9,892,346 on top of the typing, 12,785,308 without it.

## Diagnostics, recorded before any game and not deciding

- **The won-ending clock test** (`7r/5R2/8/2k1PB2/8/4K3/8/8 w - - 0 86`,
  `30000+300`), D.6.20's acceptance test: the candidate plays after **12.91 s**
  (50,875,931 nodes, the last line a depth-19 lowerbound) against the
  baseline's 9.46 to 9.82 s (37,767,960 nodes, depth 16), both over the test's
  8 s. A clock read in one position, not strength evidence.
- **The transposing rook:** the candidate plays `g7f8r` at 100,000 nodes
  where the baseline plays `g7f8q`; both win and the lines transpose
  (D.6.17).

## PRE-REGISTERED PREDICTION (frozen before any game)

1. **The outcome:** the cap without either bound, **0.70**; H0, **0.20**;
   H1, **0.10**. RAR-S112 gives no sign: the tree moves both ways, the reach
   is about 1.6% of the noisy updates, and the search constants were fitted
   with the old typing. Against a gain: the removed reward is a signal the
   capture history has been fitted around. For one: the donor types the
   verification child an all node (PLAN, D.6.3).
2. **nElo at the stop:** **−1**, 80% band **[−7, +5]**. If a bound is
   crossed early the estimate is a first-crossing estimate, biased away from
   zero.
3. **Games to the stop:** the cap, 12,000 games, with probability 0.70;
   otherwise median 7,000.
4. **In the games' own figures** (the manifest's per-side search, which
   describe the games and decide nothing): nodes a move within ±4% of the
   baseline's, mean depth within ±0.15, 0.7.
5. **Most likely failure mode:** a true effect of −2 to +2 nElo that the
   cap cannot resolve. It is read as "kept, no harm beyond about −5" and not
   as a gain, and the bracket is not narrowed afterwards.

## Result: the gate (2026-11-11)

**H0 accepted at 9,662 games: nElo −3.7 ± 6.9, Elo −2.2 ± 4.1**, LLR −2.974 in
[−2.944, 2.944], 4,831 pairs of the 6,000 cap. The candidate had 2,371 wins,
2,432 losses and 4,859 draws; pentanomial 132, 1,207, 2,193, 1,188, 111 (the
manifest's recount of `games.pgn` agrees). No time losses and no engine or
infrastructure faults. Colosseum also kept 32 post-terminal games, which the
sample excludes. The host was 0.3% busy at start, with no waiver. Both
binaries' SHA-256s, commits and bench counts in the manifest match the frozen
arms. Manifest:
`tools/results/colosseum_sprt_rar-s114-gate_20261111_104154.manifest.txt`.

In these games the candidate searched 157,625 nodes a move against 157,816, at
mean depths 13.23 and 13.20 and 66.4 ms a move each. That describes the games;
it is not a speed read. The two arms searched trees of the same size, so what
the typing cost lies in what the search decides, not in how much it searches.

**Calibration (gate).**

- P(H0) 0.20: the less likely outcome came true, a miss in confidence. The
  forecast read RAR-S112's tree, which moves both ways, as no sign, and
  weighed the donor's typing against the reward the capture history was
  fitted around as if they balanced. The games side with the second, but the
  gate cannot tell the reward from the type's other consumers.
- nElo at the stop, −1 with an 80% band of [−7, +5]: inside the band, with
  the sign the forecast leant to. The first-crossing estimate of −3.7 is
  biased away from zero by the early stop.
- Games to the stop, the cap with 0.70, otherwise a median of 7,000: the stop
  came at 9,662, before the cap.
- The games' own figures, nodes a move within ±4% and mean depth within
  ±0.15, 0.7: a hit, −0.12% and +0.03.
- The named failure mode, a true −2 to +2 that the cap cannot resolve, did not
  occur.

The gate rejects the arm as a whole. It cannot separate the dropped fail-low
reward from the type's other consumers (the child's TT-cutoff admission, the
cut-node LMR terms, LDSE, the negative singular extension, every descendant's
type); the registered rework separates them.

## Disposition (2026-11-11)

**H0: the typing is rejected and reverted.** `91265f2` on `d6` restores
`cut_node = true` at both ProbCut calls and keeps NODE-14's test, with
ProbCut's children pinned `true`; `bench 13` reads 13,038,236 / EBF 2.540
again, and the fingerprint record does not move. No changelog entry. The
defect stays open in D.6.3. The registered rework is the typing with the
fail-low reward kept: the two calls pass `false` and the ProbCut child still
rewards the capture's noisy history when it fails low. It is a new
registration with its own gate, not a rescue inside this one. D.6.4
(RAR-S113) is implemented on the source tree, where its `bench 13` is
12,785,308.
