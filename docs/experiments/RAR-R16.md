# RAR-R16 — D.1.5: the clock gate, the fitted clock cluster against the head; SPRT `[0,3]` at `3+0.03`, then the `10+0.1` and four-thread direction reads, the probe and the profile

Indexed under *4. Root search, time management and SMP* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-23, before any game. The batch is the
  maintainer's command, about 9 to 10 hours on an idle host.
- **Kind:** PLAN D.1.5, the strength gate of D.1's clock cluster, the one
  gate of the cluster and its fit (AGENTS, *Gating*: one
  dependency-complete cluster, one gate).
- **The arms:** D.1.3's seven mechanisms with RAR-R15 block 1's rounded
  centres baked into the thirteen clock defaults, against the head they
  were built on. Block 2 was abandoned, so the fit is block 1 (RAR-R15,
  *Maintainer decision 2026-10-23*).
- **Baseline:** `d1` at `468ed7b`, the head before the cluster, whose engine
  inputs are the 2.6.0 release's search and evaluation.
  - `tools/test_engines/rarog-d15head-pext-pgo.exe`, SHA-256
    `273FEDF436D4864C0CE8B8A1DCAEE978304D1A440A7232F6292EBFA2446AAC0E`.
  - Built clean in a worktree at `D:/code/rarog-d15head`, `cargo xtask build
    --arch pext --pgo`, `rustc 1.99.1`; bench **13,987,223**; `id name
    Rarog 2.7.0-dev`.
- **Candidate:** the throwaway branch `d15-arm` at `082a780`, on `b96a9c1`.
  - `bake_d15gate.py` (`3ae073a9…`) rewrote the thirteen defaults in
    `src/search/params.rs` from `tools/results/d14-spsa-block1/result.json`,
    each anchor matched once and each current default asserted equal to the
    run's start value. All thirteen change.
  - Version `2.7.0-dev+d15fit`.
  - Recipe: `analysis/arm_patches/082a780-rar-r16-d15gate.patch` (SHA-256
    `929d0790…`). Applied to `b96a9c1` through a temporary index, it
    reproduces the arm's tree `087e0beb…`.
  - Binary: `tools/test_engines/rarog-d15fit-pext-pgo.exe`, SHA-256
    `8D9007B3B5828B327E22577C8028E8E1A815F62B5A2CB7C01EFD9028497D123A`,
    built clean in a worktree at `D:/code/rarog-d15arm`, same build;
    bench **13,987,223**; `id name Rarog 2.7.0-dev+d15fit`.
  - Release tests on the arm: 414 passed. The precondition commit `b96a9c1`
    first made five time tests read D.1.3's seeds explicitly; on the
    unchanged engine it passes 413 debug and 414 release, fmt and clippy
    clean.
- **Both arms:** same compiler, same PGO workload, built minutes apart.
  - A first pair, built in worktrees under the scratch directory, was
    discarded. Its build paths passed 260 characters, so the C compiler's
    family probe failed and `cc` fell back. Both binaries above came from
    short paths without that warning.
  - The fingerprints are equal by construction (no clock in `bench`), so
    `-ExpectBench` pins both and the `id name` lines tell the arms apart.
- **The wire, read in the games:** the manifest's `search:` lines give each
  arm's time a move. The candidate must spend more than the head at
  `3+0.03`, since the fit raised `TmOptScale × TmEffortHigh` ×1.233. If it
  does not, the baked values did not reach the play, and the run is void
  whatever its verdict.

## The batch (frozen)

`analysis/artifacts/d15-clock-gate-2026-10-23/run_d15.ps1` (`2360475b…`)
runs five steps in order. It stops on a refused run, a fault stop or a
failed read; a finished SPRT goes on to the reads whatever its outcome.

1. **Gate.** Colosseum `sprt`, `sprt-default.toml` (`[0,3]` nElo,
   pentanomial, `model=normalized`, α = β = 0.05), `3+0.03`, margin 20 ms,
   Hash 64, one thread, 14 slots, the UHO book in random order, no
   adjudication, **cap 20,000 pairs** (40,000 games), seed **20261031**,
   `tools/results/rar-r16-gate`. At RAR-M10's drift a true +4 nElo needs
   about 47,000 games, so the cap resolves +5 and does not resolve +2.
2. **`10+0.1` direction read.** `match-fixed-ltc.toml`, **1,000 games**,
   seed 20261032, `tools/results/rar-r16-ltc`. It resolves about ±14 Elo.
3. **Four-thread direction read.** `match-fixed.toml` at `3+0.03`, Threads
   4, Hash 512, three games at a time, `--cores-per-game 4` (each slot four
   physical cores, CPUs 2–9, 16–23 and 24–31 in the dry run; RAR-M66's and
   RAR-M76's conditions), **1,000 games**, seed 20261033,
   `tools/results/rar-r16-4t`.
4. **Timed probe.** `tm_rec_probe_d15.py` (`48d97620…`): D.1.3's probe
   pointed at the candidate, rec1–rec3 at `60000+600` and `180000+2000`,
   with Stockfish 19 beside it. It writes `tm_rec_probe_d15.txt` beside
   itself.
5. **Profile reads.** `tm_profile_d15.py` (`2fb4907f…`) on each run,
   `profile_<run>.txt`. It is RAR-R14's `tm_profile.py` with the
   candidate's maximum transcribed at its horizon (53 moves a second) and
   p99.9 labelled. With the horizon disabled it reproduces `tm_profile.txt`
   on `rel260-stc` line for line, the corrected label aside, and its
   optimum matches the engine's at 750 and 1,500 ms.

A dry run of the three matches on 2026-10-23 resolved to policy in every
checked field: `colosseum_sprt_rar-r16-gate_20261023_101852.manifest.txt`
`d6d25bca…`, `…ltc…101856` `c7a92576…`, `…4t…101900` `7d022602…`.

The four-thread run's first dry run resolved one physical core a game: the
runner's default, which no policy check caught. `colosseum.ps1` now refuses
it (`6c3a1a1`).

Bounds, cap, book and adjudication never change after a game is seen.

## Readings and dispositions (frozen; PLAN D.1.5)

- **H1:** the thirteen values land on `d1` from the arm, its version suffix
  dropped.
  - The fingerprint is re-declared, unchanged, and the user-facing
    changelog entry is written.
  - D.1.2.1 ticks if the probe shows no rec move at the hard maximum with
    rec2 under 2 s, **and** the gate's profile reads the candidate's won and
    mate moves at ≥ 0.9 × its maximum below 0.2% each.
  - D.1.2.2 ticks if the gate shows no candidate time loss, **and** the
    profile reads the candidate's clock-floor median above 350 ms with
    under 3% of games touching 100 ms.
  - D.1.2 and D.1 close with them. A failed tick keeps its step open with
    its reason.
- **H0:** the cluster returns to research with the estimate.
- **The cap:** bakes nothing and records "unresolved" with its interval.
- **On H0 or the cap,** the registered fallback is offered to the
  maintainer (PLAN D.1.5): mechanisms 1,
  4, 5 and 6 (the horizon, the mate stop, the fail-high stop, the
  first-iteration guard) at their seeds, without the stability, effort and
  cross-move factors or the fit. It is a 2,000-game read under AGENTS'
  repair case 2 with a harm rule at −13 nElo.
  - That arm needs a recipe patch: the effort share read per iteration
    again (RAR-R15, finding 4), `TmFallPrevSlope` 0, and the stability
    factor neutral (`TmStabBase` = `TmStabFloor` = 10000, `TmStabSlope` 0).
- **The direction reads** decide nothing. A `10+0.1` sign opposite to STC
  fires RAR-R14's `originalTimeAdjust` trigger. A four-thread loss beyond
  −10 Elo names the vote's interaction for D.2.
- **The mid-iteration share,** reported, deciding nothing: the fail-high
  stop's share of the candidate's moves (lowerbound last lines). It ended
  29% of the activation read's first-move searches.
- **Another clock fit:** registered only if H1 lands with an estimate of at
  least +10 Elo, from the gated head (RAR-R15's trigger).

## PRE-REGISTERED PREDICTION (frozen before any game)

Re-frozen at the fitted values from RAR-R14's +3 ± 4.

1. **Gate:** **+2 ± 4 Elo**. H1 0.30, cap 0.50, H0 0.20. The fit raised the
   level of spending, but its gradients were small: two full-step movers,
   and the scale converged by iteration 1,500.
2. **The wire:** the candidate's `ms a move` exceeds the head's at
   `3+0.03`, 0.95.
3. **`10+0.1`:** the same sign as STC with a larger point estimate (the
   stalls cost more there), +4 ± 14, 0.6.
4. **Four threads:** within ±10 Elo of zero, 0.8.
5. **Probe:** no rec move at the hard maximum, 0.9; rec2 under 2 s, 0.95.
6. **Profile at `3+0.03`:**
   - the candidate's clock-floor median above 350 ms, 0.8;
   - under 3% of its games touching 100 ms, 0.8;
   - won and mate moves at ≥ 0.9 × the maximum each below 0.2%, 0.75;
   - no candidate time loss, 0.9 (the head arm's rate, 1 to 3 in 40,000
     games, is the comparison).
7. **The fail-high stop's share** of the candidate's moves at `3+0.03`:
   20% to 35%.

- Most likely outcome: the cap, read as unresolved near +1 to +3, with the
  probe and the floor improving. Then the fallback read follows if the
  maintainer wants the robustness pieces landed.
- Most likely failure: the scale the fit raised costs late-game time at
  `3+0.03` (RAR-R14's H3). That would read as a negative STC estimate with
  a positive `10+0.1` one.

## Amendment 2026-10-23, before any game of the SPRT: the wire rule compared the wrong arms

**The error.** The registration's wire rule (*The wire, read in the games*,
and prediction 2) said the run is void unless the candidate spends more time
a move than the head at `3+0.03`, "since the fit raised `TmOptScale ×
TmEffortHigh` ×1.233". That ×1.233 is the fit's change against **the cluster
at its seeds**, not against the head. The cluster's own stops (fail-high,
mate) and factors change time against the head on their own, so the
candidate's time against the head says nothing about whether the baked
values reached the play. The rule tests the wrong arms: it could void a
valid gate or pass an invalid one. The agent wrote it, and it is withdrawn,
not reinterpreted.

**What showed it.** The two direction reads, already played, compare
candidate and head at `3+0.03` with four threads (67.4 against 69.1 ms a
move) and at `10+0.1` (230.4 against 232.3). The amendment uses neither
read's score.

**The replacement wire check, frozen before its games and before any SPRT
game:**
- **Arms:** the candidate against the cluster at its seeds:
  `tools/test_engines/rarog-d15seeds-pext-pgo.exe`, a clean PGO build of
  `b96a9c1` (the head of `d1` the arm was cut from), the same compiler and
  workload.
- **Match:** 200 games at `3+0.03`, Colosseum `match`, seed 20261034,
  `tools/results/rar-r16-wire`.
- **Read:** time a move at equal clock in plies 0–19 by
  `wire_resolution.py`. The PGN names the arms by their binaries.
- **Formula:** the baked scale terms multiply the target by about ×1.26
  (`TmOptScale` 1.1687, `TmEffortHigh` 0.9747 against 0.924, `TmStabBase`
  1.3132 against 1.2881). Expected ×1.15–1.25 after the factor checks'
  damping.
- **Live if the candidate spends at least 5% more a move,** twice the
  difference's SE at 200 games.
- **If not live,** the baked values are not proven to reach the play, and
  the SPRT is not started until they are.
- The rule's old form is not read for the SPRT. Prediction 2 is replaced
  by: the wire check reads live, 0.9.
