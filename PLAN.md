# Rarog development plan

This is the forward roadmap. It says what will be done, in what order, why,
and what decides each step. It does not record history: completed work lives
in [HISTORY.md](HISTORY.md), measured evidence in
[EXPERIMENTS.md](EXPERIMENTS.md), procedures in [PROCESS.md](PROCESS.md), and
the day-to-day status board with checkboxes in [GUIDE.md](GUIDE.md). The
pre-rewrite roadmap is archived verbatim at
[docs/archive/PLAN-phase4-2026-09-09.md](docs/archive/PLAN-phase4-2026-09-09.md);
every historical `4.x` reference in the ledger and analyses points there.

Rewritten 2026-09-09 as a battle plan with one objective and a measured
starting point. Phases are lettered (A–G) so that no new identifier collides
with any retired number cited in the ledger, the analyses or source comments.

## 1. Objective and gates

**Objective.** Make Rarog the strongest engine we can build, in two stages:

1. **Classical stage.** With the hand-crafted evaluation, beat the strongest
   HCE-era engines in the maintainer's own pool: **Critter 1.6a, Houdini 3,
   Rybka 4 and Fritz 16**.
2. **NNUE stage.** Train networks on Rarog's own data only, then reach the
   **CCRL top 100**, and later the top 50.

**The classical target gate (E.2).** Colosseum rating tournament, the
maintainer's fixed pool with Houdini 3 added, `3+0.03`, UHO book, no
adjudication, at least 400 games per pair, measured **both at 1 thread and at
4 threads**. The gate is met when Rarog's head-to-head score against each of
the four named engines is at or above 50% at 1T and at 4T, with the 95%
interval of the pooled four-engine score excluding a loss. Ratings inside that
pool are relative; CCRL numbers do not transfer to this control (see
`analysis/endgame_occurrence_tournament_2026-09-05.md`).

**The NNUE target gate (F.10).** A CCRL 40/15 or Blitz list rating inside the
top 100, established by CCRL's own testing after a public release.

### Where we start (measured, 2026-09-04 to 2026-09-11)

| Fact | Value | Source |
|---|---|---|
| Head-to-head at `3+0.03`, 600 games each, Rarog **2.4.0 release** | Houdini 3 **−224 ±29**, Critter 1.6a **−184 ±27**, Houdini 1.5a **−179 ±24**, Fritz 16 **−147 ±24**, Rybka 4 **−99 ±25**; Basilisk 1.10.0 −23 ±20, Basilisk 1.9.3 **−9 ±22**; Rarog 2.3.2 **+70 ±20**, Rybka 3 +82, HIARCS 14 +106, Shredder 12 +185 | RAR-M45, Colosseum `5e539523`, 2026-09-11 |
| Head-to-head at `3+0.03`, **4T**, 400 games each | Houdini 3 **−169**, Fritz 16 **−149**, Critter 1.6a **−109**, Rybka 4 **−73**; Basilisk 1.10.0 **+25**, Rarog 2.3.2 +45, Rybka 3 +79. Performance rating 3034 against a frozen 3003. **4T is the easier arm for three of the four targets**, by 26 to 75 Elo | RAR-M46, Colosseum `dfb84c19`, 2026-09-11 |
| Search deficit with Rarog's own evaluation | **−247.97 ± 10.89 Elo** at equal time on the 2.4.0 release head, evaluation proved constant, no adjudication (RAR-O03). Depth 19.68 against 20.65. The superseded −250.8 was adjudicated and is not comparable | RAR-O03; `analysis/ablation_results.md` for the ablation |
| Where the search deficit lives | LMR plus shallow-depth pruning explain **272 ± 18** of it, near-additively; everything else about 30 | matched ablation, mask 160 |
| Evaluation deficit with the same search | Stockfish's classical HCE beats Rarog's HCE by **about 329 Elo** | RAR-O02 |
| Speed | **3.19 MNPS pooled median** at bench 13, PGO pext 1T, ±0.2% instrument resolution (best-of 3.21, which is the 3.22 previously recorded); Basilisk 3.71; board work 24% of time, evaluation 29%, search loop 23% | RAR-M48; RAR-M36, RAR-M44 |
| Conversion | 57 draws and 12 losses after holding a piece-up advantage for 12+ plies, in 2,400 games against the six HCE-era engines; Basilisk 40 and 12. **Measured on the 2026-09-04 pool and the pre-release `2.4.0-dev` binary — it is the one meter in this table not yet re-read on the release head**, and A.5's PGN instrument (RAR-M47) can do so from the RAR-M45 games at zero game cost | replay of the 2026-09-04 tournament; instrument RAR-M47 |
| Fingerprint | `bench 13` **7,601,220 / EBF 2.474**; engine source unchanged since `c80df74`, accepted by RAR-E15, and reproduced by every 2.4.0 build in A.7, A.8.3 and A.8.4 | GUIDE checkpoint; RAR-M48 manifests |

Both halves of the engine have room of the same order. The search half is
attacked first because it is the larger measured single item, because a
stronger search produces better self-play labels for every later evaluation
refit, and because the selectivity stack is where the interaction problem is
worst: LMR, move-loop pruning, histories and correction feed each other, and
the evidence says increments to the current co-adapted optimum measure zero
while an unfitted wholesale rewrite lost. The plan therefore rebuilds the
search as a **coherent architecture adopted as a unit, seeded from a donor,
fitted locally, gated in clusters**, with the evaluation frozen until the
search checkpoint. The evaluation programme then does the same for the
evaluation families, with the search frozen, and a joint re-fit closes the
classical stage.

### Elo budget, stated so it can be wrong

| Programme | Measured deficit | Planned recovery | Basis |
|---|---:|---:|---|
| B search | 251 equal-time | 120–200 | selectivity explains 272; a Reckless-shaped stack fitted locally |
| C evaluation | 329 same-search | 100–160 | six families, whole-surface refits, endgame conversion |
| D clock, SMP, robustness | unmeasured | 15–40 | Reckless-shaped node-fraction TM; 4T quality |
| Speed inside B and C | — | 10–30 | per-node cost of the new search and evaluation modules |

If those bands are right the classical head lands within reach of Rybka 4 and
Fritz 16 and near Critter; Houdini 3 may only fall in the NNUE stage. Each
programme's checkpoint re-measures its deficit meter so a miss is seen as a
miss and the budget above is corrected rather than defended.

## 2. Operating rules

`AGENTS.md` is authoritative for measurement, verification, documents and
gating. The rules below decide order and acceptance in this roadmap.

1. **Donor architecture, own implementation.** Reckless is the primary donor
   for search, threading, time management and NNUE; Stockfish 11 is the donor
   for the classical evaluation and Stockfish 19 for NNUE and SMP details where
   Reckless is silent. Architecture, mechanisms, population choices and
   constants may all be taken. Code is written by us in our own structure;
   line-for-line transcription is used only where an algorithm has one natural
   form or where a different form provably loses throughput. Similarity to a
   donor is never an acceptance criterion; games are.
2. **Constants are seeds.** A ported constant sits on the donor's score scale
   and node population. It is converted through the measured scale ratio (B.0),
   seeded, fitted by SPSA over the cluster's live coordinates, and only then
   gated. Search and evaluation coordinates never share a tune.
3. **Clusters, not features.** The unit of implementation and of strength
   acceptance is one dependency-complete, co-adapted cluster: every mechanism
   that consumes or produces a shared signal moves together. A feature
   implemented so it can be reported exists for nothing. Internal sub-steps
   are compiled and diagnosed separately but are not expected to win standalone
   and do not get their own gates.
4. **Compatibility over completeness.** Before adding a mechanism, audit the
   ones it will feed or be fed by. Re-implement an existing feature when the
   donor's form of it composes better with its neighbours; keep ours when the
   evidence says ours is better. Ordering-to-pruning feedback, evaluation-to-
   margin coupling, TT masking and history gravity are the interactions that
   have bitten this project; name them in every cluster handoff.
5. **Each cluster: audit, register, implement, prove, explain, fit, gate,
   record.** Registration in `EXPERIMENTS.md` before any games: hypothesis,
   baseline SHA, bracket, cap, stop rule and the frozen prediction. Bounds
   default to `[0,3]` nElo; a large prior uses `[0,10]` or `[3,10]` and says
   why; a removal or unknown-sign repair uses a loss-permitting or symmetric
   bracket. Never change a gate after seeing games.
6. **Two rejected clusters in one programme stop it** and force a new evidence
   audit before a third is built.
7. **Every strength A/B runs with adjudication off**, at `3+0.03`, 1T, Hash 64,
   paired UHO, `fastchess -use-affinity`, concurrency 14, unless the
   registration states otherwise and why. Multi-thread gates drop affinity and
   calibrate a null pair first.
8. **State the measurement layer.** Theory truth, move quality, conversion,
   fixed-node tree shape, NPS and game strength are different units with no
   exchange rate. Counters, node counts, EBF, tactical suites and fit loss
   explain or screen; only a registered final-PGO SPRT or the target gate
   accepts.
9. **Freeze the prediction before exposure, append the calibration after.**
   A miss is recorded as sign, magnitude, mechanism, interaction, confidence
   or instrument. `NO_CHANGE`, refuted and too-sparse are successful outcomes.
10. **Deficit meters are re-measured at every programme checkpoint**: the
    equal-time gap against the frozen Stockfish-search oracle (B), the
    same-search gap against Stockfish's classical HCE (C), the conversion
    instrument (A.5), pooled-PGO NPS, and the pool score against the four
    target engines.
11. **Engine, tooling and documentation changes are separate commits.** PLAN
    and GUIDE change together. Completed IDs never change; open IDs may be
    renumbered with a map.
12. **Expensive jobs are the maintainer's**: SPRTs, SPSA, datagen, tournaments,
    PGO campaigns, long profiles. Budget: about three SPRT-sized runs per day.
    The agent prepares, verifies and hands over one runnable command.

### Workflow states and capability classes

`RESEARCH -> READY_FOR_IMPLEMENTATION -> IMPLEMENTED -> LOCAL_QUALIFIED -> GAME_GATE -> CLOSED`

`READY_FOR_IMPLEMENTATION` is the boundary at which the mechanism, semantics,
evidence, interactions, invariants, falsifier and accept/reject rule are
frozen; implementation owns ordinary engineering inside that contract and
returns a false premise to `RESEARCH` instead of rescuing it.

| Class | Required capability | Typical use here |
|---|---|---|
| `R3` | Frontier causal/architecture research | programme investigations (B.0, C.0, F.0), cluster design |
| `R2` | Bounded correctness-sensitive reasoning | audits with a known question, contract definition |
| `I2` | Difficult implementation | cluster implementation in search, evaluation, NNUE runtime |
| `I1` | Well-specified implementation | tooling, refactors with an exact fingerprint, ports from a written handoff |
| `M` | Mechanical documentation/provenance | ledger rows, archives, changelogs |
| `V` | Verification/measurement | qualification runs, gate preparation, re-measurements |

GUIDE maps classes to current model names and thinking modes. Investigation
leaves (`R3`) are expected to **spawn** implementation and measurement
sub-steps under their own step; the sub-steps listed below under an
investigation are the expected shape and are confirmed, split or replaced by
the investigation's handoff.

### Standing contracts

Live invariants that every change must keep. Their derivations are in the
linked analyses; this table is the index.

| Contract | Where it is written down |
|---|---|
| Board, legality, make/unmake, SEE king legality and created pins, 41 external fixtures | `analysis/see_contract_2026-09-06.md`, `analysis/see_repair_2026-09-06.md`, `tests/data/see-*.tsv` |
| History capacity and canonical-move contract | `analysis/history_contracts_2026-09-08.md` |
| Draw, null, repetition and rule-50 policy identities | `analysis/draw_policy_2026-09-08.md` |
| Board footprint assertions (`Board <= 264`, `UnmakeInfo <= 24` bytes) | `src/board/board.rs` const assertions |
| Caller-owned move-list delivery; no 520-byte return copy | `analysis/movelist_delivery_2026-09-09.md` |
| Diagnostic counter units and sampling | `analysis/phase4_counter_spec.md` |
| Measurement layers for endgame work | `analysis/endgame_measurement_layers.md` |
| Texel data contract, splits, instrument coverage | `analysis/texel_fitting_handbook.md`, `analysis/hce_archive_audit_2026-08-31.md` |
| Behaviour-neutral change = exact fingerprint plus targeted checks plus pooled NPS | `AGENTS.md` |
| Cross-engine board benchmark parity | `benches/board.rs`, `analysis/board_comparison_411b19_2026-09-09.md` |

## Phase A — Reset: repository, instruments, baselines, consolidation release

Short and mostly mechanical. It leaves the repository clean, ships the
accepted head as a release before the search programme rewrites the search,
and measures every deficit meter on that released binary, which is the head
the programmes start from. Execution order is the numbering: the release
(A.3) comes before the baselines (A.5) so that the baseline measurements are
release evidence as well and are taken on the exact shipped binary; the
universal-binary investigation (A.4) runs while the release gate's games are
being played and can still make this release if it passes its checks.

- **A.1 Document reset — `M`, CLOSED 2026-09-09.** New PLAN, GUIDE and
  HISTORY; the Phase-4 roadmap and GUIDE archived under `docs/archive/`;
  `check_guide.py` adapted to lettered phases; AGENTS and PROCESS references
  updated.
- **A.2 Repository and branch cleanup.**
    - **A.2.1 Tracked-file cleanup — DONE 2026-09-09.** Removed, all last
      present at `6fa6731`: the 4.11.7 study runners (`archive_4117.py`,
      `run_4117_registered.py`, `summarize_4117.py`; RAR-M21's outputs are
      archived locally), `run_board_search_profile_411b7.ps1` (one remote
      measurement; the reusable ETW capture and summarizer stay),
      `profile_probe.py` and `profile_attrib.ps1` (legacy duplicate-work
      probes, superseded by the ETW profile), `nps_ab.ps1` (superseded by
      `nps_multibuild.ps1`), `perft_compare.py` (superseded by the
      cross-engine board benchmark), `tools/texel/reference/basilisk_tuner.cpp`
      (a copy of Basilisk's tuner; the Rust port is the tool), `import_beast.py`
      (the rejected Stockfish-label path, RAR-E03), and `holdout.py` with its
      test (imported by nothing). Kept for named owners: the SMP probe scripts
      (D.2), the answer harness and search-quality readouts (B.0 decides), the
      SPSA configs (A.2.3 decides).
    - **A.2.2 Branch and tag disposition — DONE 2026-09-09.** The oracle
      package (`rarog-stockfish-hce-hybrid.exe` `da78a145…`, `rarog_hce.dll`
      `e43b602b…`, licences) lives in the untracked working-tree directory
      **`hybrid/dist/`** — **path corrected 2026-09-11**; the previously
      recorded `D:/chess/engines/oracle-rarog-hybrid-75d0d43/` does not exist
      and never did, so PLAN pointed at nothing. Both files were re-hashed at
      the corrected path and match the two SHA-256 prefixes above, so the bytes
      are intact; only the record was wrong. That DLL carries the **2.3.2**
      evaluation (built 2026-08-11) and must not be used for A.8.3. Tags `oracle/hybrid`
      (75d0d43), `oracle/hybrid-diag` (2682f64), `oracle/hybrid-ablate`
      (984f478), `arm/p410-jitter-1t` (e7965b9), `arm/p410-lmr-relief`
      (5dbeb52), `arm/p410-margin-relief` (e950f03) and `arm/p46-root-relief`
      (2a64941) were created and pushed; the seven branches were deleted
      locally and on `origin`, and three stale worktrees (a temporary
      `hybrid-ablate` checkout, `target/411b7-probe-work`,
      `D:/code/rarog-411b8-baseline`) were removed. Only `master` and `dev`
      remain. Every ledger SHA cited on those branches resolves through a tag.
      Observation for the maintainer: the local lightweight tags `v1.3.0`
      through `v2.3.0` point at different objects than the annotated tags on
      `origin` (`git push --tags` rejected twelve); `origin` is authoritative
      and `git fetch --force --tags` would realign them.
      **Local tag realignment, 2026-09-11.** Twelve version tags (`v1.3.0`,
      `v1.3.1`, `v1.3.2`, `v1.3.3`, `v1.3.4`, `v1.4.1`, `v1.4.2`, `v1.4.3`,
      `v2.0.0`, `v2.0.1`, `v2.0.2`, `v2.3.0`) pointed locally at commits on **no
      branch** while `origin`'s tags of the same names pointed at the live
      `master` lineage — for `v1.3.0` and `v2.0.0` the two commits carry
      byte-identical trees, which is the signature of a history rewrite whose
      force-push updated the branches but never the local tags. Realigned with
      `git fetch origin --tags --force`; all 28 now match `origin`.
      `--prune-tags` was deliberately NOT used: it would have been safe here
      (no local-only tags exist) but it is the command that would silently
      destroy the six ledger-cited `oracle/*` and `arm/*` tags if that ever
      stopped being true. Those six remain intentionally branchless per the
      tag-then-delete pattern above and were verified present afterwards.
      `v2.3.2` never differed, so RAR-E16's "built from tag `v2.3.2`" baseline
      provenance is unaffected. Prior targets recorded in the commit message.
    - **A.2.3 Feature and option inventory — DONE 2026-09-09.** All four
      Cargo features stay (`diag` and `tune` as instruments, `texel` as the
      fitting path, `ablate` until B.9); all nine UCI options are consumed.
      Of the 99 `SearchParams` entries, **42 are inert at default** (zero
      guards, zero additive terms, or weights behind an off switch) and are
      removed in B.1; **55 are live seeds** the B.2 cluster replaces with
      donor-shaped successors; `lazy_margin` belongs to C.1 and
      `ablation_mask` leaves with the feature. RAR-S65 to S69 are superseded by
      B.2 and recorded so at B.1; typed TT provenance and the SPSA configs are
      B.0/B.1 decisions. Evidence: `analysis/feature_inventory_2026-09-09.md`.
- **A.3 Release gate and pre-release repairs — `V`/`M`, COMPLETE 2026-09-10.**
  Establish that the accepted head is worth releasing and repair what the gate
  exposed, before the search programme changes it. The release itself is A.9,
  which runs after the Phase A work that belongs in it. The 2026-09-04 pool already has the head's
  predecessor at **+43.7 Elo head-to-head over 2.3.2** (400 games) and +29 in
  pooled pool score; ProbCut, root LMR relief, two HCE refits, TB-corrected
  labels and the board cluster are all gated individually. The version follows
  the release rule in section 4: 2.4.0 if the registered STC gate's point
  estimate is at least +40 with the lower bound above +25, else 2.3.3.
    - **A.3.1 Toolchain bump, behaviour-neutral — DONE 2026-09-09 (`ca8988a`),
      RAR-P18.** `rust-toolchain.toml` moved from 1.97.1 to 1.98.1
      (`48a229cea`), with no experiment in flight and before RAR-E16's binaries
      exist. Every done criterion met: fingerprint 7,601,220 / EBF 2.474 exact
      on the `x86-64`, `avx2` and `pext` plain builds and on three `pext` PGO
      builds; `cargo test -p rarog` debug and release green, tooling crates
      green, fmt and clippy clean; `verify-isa` clean on all four assets;
      pooled-PGO NPS −0.53% (95% CI −1.99% .. +0.08%), inside ±1%, with a
      same-source null pair at −0.42% (−1.30% .. +0.33%) showing the
      instrument cannot separate the compiler from per-build profile luck.
      **Of the two obligations this bump created, one is now paid.** RAR-P08's
      `rust-lld` Windows ARM64 PGO workaround was re-verified on 1.98.1 by
      **RAR-P19** (2026-09-10, `ef9c6ae`): the workaround still links, and
      `verify-isa` passed on an ARM64 PGO asset for the first time, on both a
      Windows ARM64 and a macOS ARM64 host, with the fingerprint matching
      x86-64 on both. **Discharged 2026-09-11:** the A.9 squash to `master` ran the CI
      matrix on the 1.98.1 pin for the first time and it went green, which is
      what the hold required; the hold is removed from GUIDE.
    - **A.3.2 Release gate — RAR-E16, DONE 2026-09-09: H1 ACCEPTED, 2.4.0
      licensed.** STC 1T H1 at 742 games of a 16,000 cap, **+54.77 +/- 17.04
      Elo** (nElo +81.73 +/- 25.00, LLR 2.95, LOS 100.00%), one time forfeit;
      the `3+0.03` 4T direction check read **+79.53 +/- 21.21** over 400 games
      with zero forfeits and zero protocol warnings. The release rule wants at
      least +40 with the lower bound above +25 and the lower bound is +37.73,
      so the version is **2.4.0** — subject to A.3.3 and the two 1.98.1 holds.
      The magnitude is not settled: a boundary stop at 4.6% of the cap biases
      the estimate upward, so the head is clearly and substantially stronger
      than 2.3.2 without +54.77 being its Elo. Registered as: the
      release candidate (A.3.1 head, PGO pext) against the 2.3.2 release binary,
      `3+0.03`, 1T, Hash 64, paired UHO, no adjudication, `[3,10]` nElo, cap
      16,000 games, plus a `3+0.03` 4T direction check of 400 games with zero
      forfeits. `3+0.03` is the gate's clock at both thread counts; `10+0.1` is
      a pre-release check rather than a condition of the verdict, and the 4T
      null pair is dropped as already-calibrated harness behaviour (maintainer
      decision 2026-09-09, before any gate game). Prediction frozen in the row. H1 with the point
      estimate at or above +40 and the lower bound above +25 licenses 2.4.0;
      any other H1 licenses 2.3.3; H0 stops the release and is itself a finding
      against the accepted-gains ledger. While these games run, the agent works
      A.4. **Prepared 2026-09-09, games not started.** Both arms are built with
      verified manifests; RAR-E16 carries their paths, hashes and fingerprints.
      The registered
      baseline artifact was wrong and was replaced before any game: the file the
      row named benches the development fingerprint, not 2.3.2's, and its
      `--native` flavour is never released and is refused by the harness's
      flavour guard. The replacement is built from tag `v2.3.2` with the release
      recipe and reproduces RAR-M12's recorded 6,519,711 / EBF 2.449. Bounds,
      cap, clock, book, adjudication and prediction are unchanged.
    - **A.3.3 Time-forfeit repair — `R2` investigation, `I1` fix, `V`
      validation, before the release.** Rarog forfeits on time in roughly one
      game per thousand at `3+0.03`, 1T, concurrency 14, on every recent SPRT
      regardless of arm: RAR-E16's STC run (game 114 of 743, the candidate),
      RAR-E15 (1 in 1,951), RAR-E09's runs (4 in 7,389 and 1 in 1,501). The
      seven forfeited games reconstructed from their PGN clocks agree with
      RAR-M14: at this control the base clock is three seconds, and five of
      the seven losers had spent their entire budget within ±0.1 s of zero by
      their own reported move times when they stalled, while two still had
      0.16 s and 0.48 s by their own accounting, which only harness-side wall
      time can explain. The engine keeps a hard ceiling of `time − 2 ×
      MoveOverhead` at 1T (`time_manager.rs`, `min_reserve`), with a further
      30 ms reserve only when `Threads > 1`, `MoveOverhead` defaults to 10 ms,
      and the harness allows a 20 ms margin; under a saturated 14-game host
      the wall-time jitter of one reply exceeds that reserve about once per
      thousand games. Sub-steps: (1) *diagnose*, `R2`: add the per-game clock
      reconstruction used here to `tools/pgn_result.ps1` or a sibling so every
      SPRT log reports forfeits with the loser's reconstructed clock and last
      move times; confirm from the seven games whether the stall is the last
      move overrunning `maximum_ms` (engine) or a reply lost to scheduling
      (harness), by comparing each loser's reported time with the wall time
      the PGN clock implies. (2) *fix*, `I1`: the engine-side candidate is a
      clock-proportional low-time floor, `min_reserve = max(2 × overhead,
      jitter_reserve)` applied at every thread count with `jitter_reserve`
      around 30–40 ms and scaled down when the remaining clock is under it,
      plus a check that the hard stop is evaluated against wall time including
      the bestmove write; the harness-side candidate is `Move Overhead` raised
      for the pool and SPRT profiles (RAR-M14's sweep, sized first: at 0.1% the
      background rate needs tens of thousands of games to distinguish two
      values). Fingerprint unaffected (time management is bench-invisible),
      so the repair is qualified by games, not by bench. (3) *validate*,
      `V`, registered as **RAR-R11**: a fixed-length 10,000-game paired run at
      `3+0.03`, 1T, concurrency 14, fix against the A.3.1 head, adjudication
      off, with the fix accepted when its arm forfeits at most a quarter of the
      baseline arm's forfeits and the paired Elo interval excludes −3 (a
      reserve costs thinking time; the loss must be bounded). Zero forfeits in
      the release gate's runs is a precondition of the release, never the
      verdict. Prediction frozen in the row.

      **Disposition 2026-09-10: CLOSED.** RAR-R11 ran 10,000 games overnight
      with **zero forfeits in either arm** and **-0.69 +/- 3.62 Elo**: the rate
      is undecidable on an idle night host (every earlier forfeit came from
      daytime runs), the Elo bound is met, and the repair stays on donor
      parity. RAR-R12 and BAS-E57 refuted the harness-reserve idea at **-81**
      and **-65 Elo**: `Move Overhead` is subtracted per move over the whole
      horizon, not reserved at the end. `Move Overhead` stays 10; a low-clock
      reserve is an engine-side term for D.1. The forfeit rate is watched in
      A.5's daytime pool runs. Diagnosis record: The seven forfeits are one population of 50–500 ms
      stalls in which the search cannot run its clock check. CPU contention
      alone did not reproduce them (5,600 searches under fourteen concurrent
      processes, worst overrun 2 ms). Both donors stamp the clock on the UCI
      thread while parsing `go` (Stockfish `limits.startTime`, Reckless
      `TimeManager::new`); Rarog stamped it on the engine thread after the
      command hand-off and configuration invalidation, so that latency under a
      loaded host was invisible to its budget. Repair `79d3974`: the clock
      starts when `go` is parsed (`SearchLimits.issued`), with a test that a
      search issued past its budget returns at once. An interim throttle of
      `info` output (`d93f808`) was reverted at `e3430d9`: neither donor
      throttles and the reproduction behind it measured the driver's own
      per-line lag. Bench-invisible by construction and reproduced at
      7,601,220 / EBF 2.474 on both slider backends; 567 tests, fmt and clippy
      clean. The low-time reserve was deliberately not changed and stays with
      D.1; stalls that land mid-search are the harness margin's problem for
      every engine, which is why RAR-R12 (Rarog) and BAS-E57 (Basilisk)
      measure `Move Overhead` 40 against 10 on the same binary in the same
      night run as RAR-R11. Basilisk already counts dispatch latency (its
      Step 5.4) and forfeits at the same rate, so no engine change is owed
      there. Evidence: `analysis/time_forfeit_2026-09-09.md`.
- **A.4 Build, asset and CPU-selection improvements — `I1` sub-steps.** The
  per-tier assets stay. RAR-P20 measured what the ladder is worth on an idle
  5950X — **avx2 over base +4.59% [+3.79%, +5.83%]**, **pext over avx2 +2.45%
  [+2.32%, +2.62%]** — so the tiers are real, and the remaining problem is that
  nothing helps a user pick the right one, and nothing warns a user whose CPU
  makes the fastest-looking asset the slowest. That, the `cc` bump and the
  Fathom build defects are what this step fixes. Replacing asset choice with
  runtime dispatch is a different and much larger change: it is recorded as
  **optional** work under G.2, with the full design in
  `analysis/universal_binary_2026-09.md`, and is not scheduled.
  **`--arch native` was considered and DROPPED, 2026-09-10.** `--arch` names a
  compatibility *contract* — which instructions an artifact may contain — and
  `verify-isa` exists to hold each value to that promise. `native` has no
  contract to check, being "whatever this CPU supports", so putting it in
  `--arch` would add the one value the tool policing `--arch` cannot police. It
  is a selection policy, not an architecture, which is why `--native` is already
  a separate and independent flag. Making the bare x86 default "best tier for
  this host" was the better version of the idea and was also declined: a default
  that varies by machine is a mild form of this project's one failure mode, and
  it would fork the slow-PEXT family list between `cpu_advice.rs` and `xtask`.
  The convenience it bought — not typing `--arch pext` — is already covered by
  the A.4.2 advisory and the A.4.4 README table.

  **A.4 ships inside 2.4.0, so every engine change in it is behaviour-neutral
  by contract.** Each must reproduce `bench 13` at **7,601,220 / EBF 2.474**,
  which is what keeps RAR-E16's verdict licensing this release. Fingerprint
  equality alone is not sufficient evidence and the leaves say so: the
  RAR-E12-era mate drive moved KBN-K conversion from 19.4% to 96.9% on an
  identical bench, so each leaf below also names the targeted check for the
  behaviour it actually touches.
    - **A.4.1 `cc` bump and Fathom build verification — DONE 2026-09-10
      (`1bf8171`), RAR-P21.** `cc` 1.3.0 -> 1.4.5 and the manifest floor to
      `"1.4"`; `verify-isa` holds on `base`, `avx2`, `pext`, `pext --pgo` and
      macOS `arm64 --pgo`, with **base at `popcnt 0`**; fingerprint
      **7,601,220** on all three x86 tiers; debug 283 and release 284 tests
      passed with zero failures; fmt and clippy clean. It also repaired an
      A.3.1 oversight, `rust-version` left at `1.97` against a 1.98.1 pin.
      **The macOS `-fprofile-use` question is answered with a negative that
      reframes it:** 1.4.5 does not close the gap, and capturing the Windows
      C invocation in full showed **no platform compiles Fathom with PGO** -
      macOS is merely the only one that warns. Negligible and uniform, and not
      worth fixing, since MSVC PGO cannot consume rustc's LLVM profile.
      Original scope, retained for the record: `cc` moves
      1.3.0 to 1.4.x. The manifest requirement `cc = "1.3"` already permits it,
      so only `Cargo.lock` pins the old version, and 1.4.5 was seen resolving
      cleanly during the A.4 investigation. This is not a cosmetic bump:
      `build.rs` compiles Fathom with a tier-dependent `TB_NO_HW_POP_COUNT`
      driven by `CARGO_CFG_TARGET_FEATURE`, and getting that wrong is what
      shipped **15 illegal `popcntq`** in the 2.3.0 and 2.3.1 baseline assets.
      It also carries one open defect: **RAR-P19 measured that on macOS the
      `cc` crate rejects the inherited `-fprofile-use`**, so `tbprobe.c` is
      built without the profile while the Rust half carries it, and Windows
      ARM64 shows no such warning. Establish whether 1.4.x closes that gap and
      record the answer either way, a negative included. Blast radius, so the risk is
      neither overstated nor misread: **tablebases work normally**; this
      concerns how `tbprobe.c` is compiled, not whether probing runs or is
      correct. The exposure is users who configure `SyzygyPath`. No
      measurement can be affected — `can_probe` returns false while
      `largest() == 0`, the gate harnesses never set the path, and datagen's
      tablebase flags go to fastchess rather than to the engine. Done criteria: `cargo xtask verify-isa` clean on `base`,
      `avx2`, `pext` and `arm64`; bench fingerprint **7,601,220** unchanged;
      debug **and** release suites green; `cargo fmt --check`; clippy
      `--all-features --all-targets` at zero warnings; the macOS question
      answered on the ARM64 compatibility host. A dependency change ships in
      its own commit, apart from tooling and documentation.
    - **A.4.2 CPU advisory at startup — DONE 2026-09-10 (`e66bb51`).**
      Implemented in `src/cpu_advice.rs`. **No new `unsafe` after all** —
      `__cpuid` requires no `target_feature`, so it is a safe function on the
      pinned toolchain and the unsafe floor is untouched; the leaf had assumed
      one block would be needed. No new dependency. Verified where it matters:
      each asset contains exactly the advice it can give (`base` and `avx2`
      carry the under-tier line and not the microcode line, `pext` the
      reverse), and running all three on the 5950X, `base` and `avx2` advise
      `pext` while `pext` stays silent — the guard fires, which its predecessor
      never did. UCI handshake intact, fingerprint 7,601,220 unchanged, tests
      debug 290 / release 291 with zero failures, fmt and clippy clean.
      **Not exercised on real hardware:** the microcoded branch, for want of a
      Zen 1, Zen 2 or Excavator host; it is unit-tested, its string is present
      in the shipped `pext` asset, and the CPUID decode is validated on this
      host. Original scope: two advisories, one of which
      fixes a defect we ship today. (a) **Slow-PEXT gate.** AMD Excavator
      (family 15h) and Zen/Zen+/Zen2 (17h) implement `pdep`/`pext` in
      microcode, so on those parts our `pext` asset is our *slowest* engine and
      nothing says so; Stockfish carries exactly one model-based exception in
      its entire source for this, and this is it. (b) **Under-tier hint.** A
      `base` or `avx2` asset on a more capable CPU should say what it is
      leaving on the table. **The constraint that decides the design:**
      `is_x86_feature_detected!` expands to `cfg!(target_feature = ...) ||
      runtime_detect(...)`, so in a tier that *statically requires* the feature
      it folds to a compile-time `true` and the branch is stripped — `main.rs`
      records that this is exactly why the old startup CPU guard never fired in
      a shipped asset. Both advisories sit on the working side of that: a
      `base` asset testing for AVX2/BMI2 does not statically require them, and
      a slow-PEXT test is family/model, not a feature test. Implementation:
      `core::arch::x86_64::__cpuid` leaves 0 and 1, **one documented `unsafe`
      block, and no new dependency** — `[dependencies]` is empty today and
      stays empty. A zero-crate probe was verified on the 5950X during A.4
      (`AuthenticAMD`, family 0x19, model 0x21, fast PEXT). It is an
      **advisory, not a refusal**: print and continue, never alter search,
      never alter the fingerprint. Done criteria: fingerprint unchanged; and
      the message must be **proven present in each built asset by searching the
      binary**, not by reading the source — the removed guard's whole failure
      was that its string was stripped and nobody looked.
    - **A.4.3 Direct `pext` versus `base` NPS — DONE 2026-09-10, RAR-P22.**
      **+6.65%, 95% CI [+6.48%, +6.89%]**, inside the frozen +6.0% to +8.5%
      band. The run earned its place: compounding RAR-P20's hops gives +7.15%,
      which the direct interval **excludes**, so A.4.4 quotes +6.65% and not
      the chained figure. The cause was not the session drift, which
      interleaving absorbed, but RUN2's 2.04 pp width against this run's
      0.41 pp. Original scope: the one
      number A.4.4 needs and RAR-P20 cannot supply. RAR-P20's runs cannot be
      chained into it: the same four `avx2` binaries drifted **+1.46%** between
      RUN2 and RUN3, so the compounded ~7.2% is an estimate, not a
      measurement. Reuse the pools and the harness — `tools/nps_build_pool.ps1`
      for four builds per tier if they are not still on disk, then
      `tools/nps_multibuild.ps1 -Cycles 10 -Repeats 3` with `base` as the
      baseline arm and `pext` as the candidate. About ten minutes on an idle
      box. Register it before it runs, with its prediction frozen; the honest
      prior is the compounded estimate, which is exactly what it is testing.
    - **A.4.4 README asset guidance — DONE 2026-09-10.** The Download section
      now carries the measured costs (`pext` 2.4% over `avx2`, 6.7% over
      `x86-64`, cited to RAR-P20 and RAR-P22, with the ~10-15 Elo scale for a
      reader who does not think in NPS), names Excavator alongside Zen 1 and
      Zen 2 in the slow-PEXT row, and **removes a claim A.4.2 had just made
      false**: the README said the engine "cannot reliably detect this about
      itself", which A.4.2 made true only of the crash case. The agent's draft
      split the two cases explicitly; the maintainer's prune (`f5a96e9`) deleted
      the passage instead, reaching the same end - no false claim survives - by
      a shorter route. The crash guidance it also carried still stands in the
      asset table's "Use when" column. Checked
      by diff and link, not rebuilt: the advisory prefix the README promises was
      confirmed present in the shipped asset. Original scope: record the measured cost of
      choosing the wrong asset, so the choice stops being folklore: A.4.3's
      `base`-to-`pext` figure, plus RAR-P20's +4.59% and +2.45% steps, next to
      the CPU requirement already listed per asset. State the slow-PEXT
      exception in the same place — an Excavator, Zen 1 or Zen 2 owner should
      be told to take `avx2`, not `pext` — so the guidance exists in writing
      even for someone who never sees A.4.2's startup line. Documentation only;
      no build, no engine change.
    - **A.4.5 Engine argument handling — DONE 2026-09-10 (`63843f5`).**
      Arguments now run through the same dispatch stdin uses; the loop body
      became `handle_command`, returning a `CommandOutcome` so a caller knows
      whether to keep reading and what to exit with. The load-bearing detail is
      which shutdown `run_once` uses: interactive `quit` calls `request_quit`
      and `push_priority`, cutting short work already dispatched, while EOF
      pushes an ordinary FIFO quit a bench completes ahead of. `run_once` uses
      the EOF form — the other would reproduce the very bug — and a test
      asserts the queued quit carries epoch 0. An unrecognised argument prints
      and exits **2**. Verified on both paths: `bench 13` reads 7,601,220 via
      argv and via stdin, a GUI passing no arguments sees an unchanged
      handshake, and the fingerprint did not move. Original scope: `main` goes
      straight
      to the UCI loop and never reads `std::env::args()`, so `rarog.exe bench
      13` prints the banner, reaches EOF, benches nothing and exits **0**. It
      cost real time during the A.4 investigation and it will cost it again.
      Either accept argv commands by feeding them to the same command path
      stdin uses, or reject unknown arguments with a message and a non-zero
      exit; silence with a success code is the one option that is not
      acceptable. Targeted checks, since this is an engine change: `bench 13`
      unchanged at 7,601,220 via stdin, an argv invocation now doing what it
      says, and the UCI handshake unaffected — a GUI passes no arguments and
      must see byte-identical behaviour.
- **A.5 Conversion instrument — DONE 2026-09-10 (`208e06c`), RAR-M47.**
  `tools/diag/conversion_audit.py` reads **PGN**, not Colosseum's database:
  the seed's dependency on one program's schema on one machine is replaced by
  `export_tournament_pgn.py`, which is now the only tool that knows that
  schema. Tournament `41768fe9` is exported, hashed and held in ignored
  storage; the tracked artefacts are the tools and two 0.9 KB frozen
  summaries that cite the archive by sha256. Fidelity was proven before the
  corrected numbers were trusted — `--no-exclude-lone-minor` reproduces the
  seed exactly at 57/12 and 40/12 — and the lone-minor guard was proven live
  by counting its own suppressions rather than assumed from an unchanged
  total. **Storage rule established: no games in Git, ever; export once, cite
  the hash, track only the summary.** Original scope: make the replay used on
  2026-09-09
  (`tools/results/conversion-replay-20260909/replay.py`) a tracked tool:
  `tools/diag/conversion_audit.py` reads a Colosseum tournament by id,
  replays every game of a named engine against a named opponent set, and
  reports draws and losses after a persistent material advantage (12 plies,
  at least a minor piece, lone-minor exclusions by material signature), by
  termination and by phase, plus saves from persistent deficits. Baseline:
  Rarog 57/12, Basilisk 40/12 on tournament `41768fe9`. The tool is re-run at
  every programme checkpoint; it is a diagnostic layer, never an acceptance
  layer.
- **A.6 Codebase consolidation analysis — DONE 2026-09-10, `NO_CHANGE` to
  source.** `analysis/consolidation_2026-09-10.md` inventories the crate at
  `7cffce5` (24,641 lines; `search.rs` 6,319 with a 1,684-line `negamax`
  and 1,031 lines of in-file tests; `eval.rs` 3,756 with 137 `eval_params!`
  entries) and decides that **Phase A refactors nothing**: every candidate
  sits in a file B.1 or C.1 splits, so moving it now would be qualified
  twice for no strength. Outputs: the B.1 and C.1 handoffs as tables from
  today's line spans to the target modules; a dead-code list that extends
  A.2.3's 42 inert parameters with the three subsystems they guard — root
  confidence (~600 lines across `search.rs`, `search_threads.rs`, `diag.rs`
  and `params.rs`, computed every root iteration and consumed by nothing at
  default), SMP iteration skipping, and two test-only index helpers — all
  removed in B.1; the disposition of the 2026-08-19 code audit (item 1
  resolved, item 2 rejected by RAR-S66, items 3–4 to B.2 and C.1); and three
  additions to the target layout below (`eval/params.rs`, `eval/attacks.rs`,
  `eval/endgame/kpk.rs`). The `evidence.rs` consumer question, the counter
  re-keying map and the `NodeType` set remain B.0's. Original scope:
  inventory the crate against the module layout the programmes will produce,
  and decide what is refactored now, what is replaced by B and C, and what is
  deleted; do not refactor what a programme is about to replace; output the
  B.1 and C.1 restructure handoffs, a dead-code list and the target layout.

Target module layout after B and C (the investigations may adjust it; the
three `eval/` additions are A.6's):

```
src/board/…                board, movegen, see, zobrist        (as today)
src/search/mod.rs          iterative deepening, root, aspiration
src/search/node.rs         negamax<NodeType>, qsearch
src/search/stack.rs        per-ply StackEntry, PlyArray
src/search/movepick.rs     staged picker and scoring
src/search/history.rs      quiet/noisy/pawn/continuation histories
src/search/correction.rs   correction histories and eval correction
src/search/params.rs       search_params! (tunable surface)
src/search/threads.rs      lazy SMP, shared context, voting
src/search/time.rs         soft/hard limits, node-fraction multiplier
src/tt.rs                  transposition table
src/eval/mod.rs            evaluate, phase, scale, tempo
src/eval/{material,pawns,pieces,king,threats,passers,space,initiative,endgame}.rs
src/eval/params.rs         eval_params! (tunable surface) and tune/texel I/O
src/eval/attacks.rs        the single attack-map and mobility-area producer
src/eval/endgame/kpk.rs    KPK bitbase (today src/kpk.rs)
src/eval/trace.rs          EvalTrace and fitting instrument
src/uci/…                  protocol, options, engine loop
src/diag.rs                counters (feature `diag`)
```

- **A.7 Version bump to 2.4.0 — DONE 2026-09-10 (`c6a548f`).** `Cargo.toml` to
  `2.4.0`, the `rust-version` line kept in lockstep with `rust-toolchain.toml`, and
  `CHANGELOG.md` opened for the release from the accepted ledger rows since
  2.3.2 (RAR-E16 +54.77 ± 17.04 licenses 2.4.0 under the section 4 rule; the
  individually gated ProbCut, root LMR relief, two HCE refits, TB-corrected
  labels and the board cluster are its content). Nothing else changes here: a
  version string is not an engine input, so `bench 13` must still read
  **7,601,220 / EBF 2.474**. This step exists as its own leaf because A.8's
  baselines must be measured on a binary that calls itself 2.4.0 — the RAR-E16
  baseline confusion began with a file whose version string and source did not
  agree, and asset names carry the version. **Verified:** a fresh
  `cargo build --release` benches **7,601,220 / EBF 2.474**, unmoved, and the
  binary answers `id name Rarog 2.4.0`; `Cargo.lock` follows; `rust-version`
  stays `1.98`. Debug 295 and release 296 tests pass, `cargo fmt --check` and
  `cargo clippy --all-features --all-targets` are clean with zero warnings.
  The `CHANGELOG.md` 2.4.0 section is written from the accepted rows and
  carries RAR-E16's non-additivity caveat; its date line was set to
  **2026-09-11** when A.9 tagged. Engine bump and documentation are separate
  commits.

- **A.8 Baselines on the release binary — `V`.** All maintainer-run,
  registered first. The Rarog arm is the **A.7 version-bumped binary**, which is the
  binary A.9 publishes, so the pool numbers describe the released engine
  rather than a predecessor of it.
    - **A.8.1 Reference pool refresh — DONE 2026-09-11, RAR-M45.** Colosseum
      rating tournament `5e539523`, `3+0.03`, 1T, 600 games per pair (the
      registration's floor is 400). 39,600 of 39,600 games in 9h48m, **zero
      forfeits and zero crashes**, every termination a natural chess result.
      Both frozen predictions scored in the ledger: 4 of 6 and 3 of 3 inside
      tolerance, the two misses both in the direction of Rarog being stronger,
      and the ±40 drift check passing on all eleven unchanged opponents.
      **Rarog 2.4.0 over 2.3.2 is +70 ± 20 at fixed size**, which is the
      release comparison RAR-E16 could only bound with a stopping-biased SPRT. Produces the head-to-head table the E.2
      gate is measured against and the first Houdini 3 number. Because 2.3.2
      stays in the pool, the run is also release evidence at the pool level.
      **Twelve engines** by the 2026-09-10 scope correction in RAR-M45, which
      records the pool, the drops and the reasoning: the 2.4.0 release binary,
      2.3.2, Basilisk 1.10.0 and 1.9.3, all four E.2 targets (Houdini 3,
      Critter 1.6a, Rybka 4, Fritz 16), Houdini 1.5a, Rybka 3, HIARCS 14 and
      Shredder 12. Fruit 2.1, Manta 1.0.0 and SaberTooth 0.3.0-alpha are
      dropped as near-saturated score columns. Pairs are independent, so the
      smaller pool changes no head-to-head; it changes only the fitted rating,
      which nothing downstream consumes. Rybka 4 is the sole rating anchor.
    - **A.8.2 Four-thread gauntlet — DONE 2026-09-11, RAR-M46.** Colosseum
      `dfb84c19`, `threads=4`, `concurrency=3`, hash 512 MB, adjudication off;
      Rarog 2.4.0 against the four targets, Basilisk 1.10.0, Rarog 2.3.2 and
      Rybka 3, 400 games each. 2,800 games in 2h30m, zero forfeits, one Houdini
      3 crash awarded as a loss. **All seven predictions missed, the four-target
      one in sign:** 4T deficits are *better* than 1T against three of four
      targets — Critter +75, Houdini 3 +55, Rybka 4 +26 — flat against Fritz 16,
      and **Rarog beats Basilisk 1.10.0 at 4T (+25) having lost at 1T (−23)**.
      Performance rating 3034 against a frozen 3003 and a predicted 2963–2993.
      NPS scaling passes: Rarog 5.80×, no opponent under 1.5×. **D.2's premise
      is contradicted — see below — and E.2's binding arm is 1T, not 4T.**

    - **A.8.3 Oracle deficit meter — DONE 2026-09-11, RAR-O03.** 3,000 paired
      equal-time games of the 2.4.0 release binary against the `hybrid` oracle
      (frozen Stockfish `9587eeeb` search driving `rarog_hce.dll` rebuilt from
      **this head's** evaluation), `3+0.03`, 1T, hash 64, no adjudication.
      **G(0) = −247.97 ± 10.89 Elo**, 19.35%, in 32m37s. The −235 ± 15
      prediction hit at its pessimistic edge. **Mean depth Rarog 19.68 against
      the oracle's 20.65 — 0.97 ply** — so at most about 60 of the 248 is
      depth and the rest is decision quality at near-equal depth, which
      corroborates the matched ablation's 272 ± 18 for LMR plus shallow
      pruning from a different direction. **This number supersedes 250.8 as
      B.9's baseline.** The old figure was adjudicated and this one is not;
      RAR-O01/O02 price that difference at about 74 Elo, so the two must never
      be differenced — see the calibration in RAR-O03.

    - **A.8.4 Speed baseline — DONE 2026-09-11, RAR-M48.** RAR-M41 protocol on
      the release head: two independent three-binary `pext` PGO pools at
      `5513573`, all six distinct and every copy verified at 7,601,220, run as
      a true 3v3 null. **Pooled median 3,189,100 / 3,190,438 n/s — about
      3.19 M — with the null at +0.04%, 95% bootstrap [−0.23%, +0.15%]**, so
      the instrument resolves about ±0.2% at three builds per arm. Per-build
      spread 0.42%, reproducing the documented ~0.4% PGO offset. The first
      attempt was discarded: it spanned 2.5% on a host running 12–16% CPU, and
      re-measuring the same binary idle moved it 2.3%. Not a regression against
      the recorded 3.22 M, which is a best-of; this run's best-of is 3,208,619.
- **A.9 Release 2.4.0 — DONE 2026-09-11.** The last leaf in Phase A, and the
  phase closes with it. **Documentation state:** PLAN, GUIDE, EXPERIMENTS,
  HISTORY, README and CHANGELOG carry the A.8 baselines as the current figures,
  every A.1–A.9 checkbox is ticked with its evidence, the checkpoint's released
  baseline is 2.4.0, and `check_guide.py` is green. **Release:** `dev` squashed
  into `master` as one `Version 2.4.0` commit, which ran the CI matrix for the
  first time on the 1.98.1 pin and **discharged that standing hold**; the tag
  then triggered the release CI, which built the per-tier PGO assets with
  `verify-isa` clean on each and the `bench 13` fingerprint reproduced, the
  ARM64 path having been cleared by RAR-P19. The agent never tagged, never
  published and never pushed to `master`. **What Phase A produced:** a
  consolidation release worth **+54.77 ± 17.04 Elo** over 2.3.2 (RAR-E16), a
  repository and document set that a checker keeps honest, and four baselines
  measured on the released binary — the 1T and 4T pool positions (RAR-M45,
  RAR-M46), the search deficit **−247.97 ± 10.89** with its 0.97-ply depth gap
  (RAR-O03), and the **3.19 MNPS** speed floor with a ±0.2% instrument
  (RAR-M48). Two of those changed later phases: D.2's premise is contradicted
  and E.2's binding arm is 1T.

## Phase B — Search programme

**Goal:** recover the measured 251-Elo equal-time search deficit, with the
evaluation frozen at the accepted `hce-v3` head, by rebuilding the search as
a Reckless-shaped architecture implemented in Rarog's own code, seeded,
fitted and gated cluster by cluster.

**Why this shape.** The matched ablation says the deficit is selectivity:
LMR and shallow-depth pruning explain 272 ± 18 Elo, the remaining mechanisms
about 30. Rarog already carries an LMR table with adjustments, LMP, futility,
SEE pruning, NMP, ProbCut, singular extensions, IIR, correction histories and
a staged picker, and orders moves better than the oracle by its own counters;
what differs is the population each mechanism admits and how the signals that
gate them are produced and combined. Previous attempts changed one mechanism
at a time against a co-adapted optimum (zero), or rewrote the core without
fitting it (lost). The donor's search is a working co-adapted point; adopting
its shape as a unit and fitting our constants is the cheapest way to move to a
different optimum.

**What "Reckless-shaped" means, concretely** (source of record: Reckless at
`31d9cd6`, `src/search.rs`, `history.rs`, `movepick.rs`, `transposition.rs`,
`time.rs`, `thread.rs`; read it, do not copy it):

- Node types as compile-time constants (`Root`, `PV`, `NonPV`), a per-ply
  stack entry holding the static eval, TT move, TT-pv flag, move count, the
  applied reduction, laterality, and pointers to the continuation-history and
  continuation-correction sub-tables selected by the move just made.
- One TT with 3-entry 32-byte clusters, 8-byte entries carrying a 16-bit key,
  move, score, raw static eval, depth, bound, tt-pv and a 5-bit age; static
  eval stored on every probe miss; a probe-aligned "estimated score" that
  tightens the static eval with the TT bound.
- Histories: quiet history indexed by side and by whether the from and to
  squares are threatened; noisy history by piece, to-square, captured type and
  to-square threat; pawn-structure history by pawn key bucket; continuation
  history at plies 1, 2, 4, 6 keyed by in-check and capture; bonus/malus
  formulas linear in depth with caps, gravity updates, and the moved-late malus
  scaled by index.
- Correction histories: pawn key, non-pawn key per colour, continuation
  correction at plies 2 and 4, all bucketed by the fifty-move clock; applied to
  the raw eval together with material-scaled optimism and rule-50 damping.
- Move picker with stages hash, generate-noisy, good-noisy by SEE threshold
  derived from the move's own score, quiet, bad-noisy; quiet scores from the
  four histories plus threat escape, checking-square, en-prise and offense
  terms.
- Selectivity: razoring on the estimated score, RFP with improvement and
  correction terms, NMP with adaptive reduction and verification, ProbCut with
  reduced-depth verification, singular extensions with double/triple margins,
  multi-cut and negative extensions, low-depth singular extension, hindsight
  reductions from the parent's eval delta; in the move loop LMP, futility with
  history and correction terms, bad-noisy futility, history pruning, SEE
  pruning with depth-quadratic thresholds; LMR in 1024ths with terms for
  log-depth, improvement, correction, TT bound, quiet history, PV window
  width, laterality, tt-pv, cut-node, check, cutoff count, singular margin, the
  parent's reduction and a per-node jitter; deeper/shallower re-search
  decisions from the reduced score; PVS on PV nodes.
- Quiescence: TT probe and cutoff, corrected stand-pat, interpolated
  fail-high scores, LMP at three moves when not in check, SEE pruning by a
  margin from alpha, TT write on every exit.
- Root: aspiration delta from eval and PV stability, optimism from the best
  score average, root move node accounting, forgotten-mate and aborted-loss
  guards, multi-PV structure.
- Time: soft and hard bounds from the clock and increment with a fullmove
  curve; a soft-limit multiplier from the best move's node fraction, score
  trend, PV and eval stability and best-move changes; hard check every 2,048
  nodes; a soft-stop vote across threads.
- Threads: lazy SMP with per-thread data, shared TT, shared correction
  histories, a shared best-stats word, and no depth skipping.

**Scale conversion (B.0).** Reckless's centipawn-like scale multiplies the
network output by material; Rarog's HCE is on its own refit scale. Every
cp-valued seed is converted by a measured ratio: the ratio of mean absolute
static evaluation over the same 40 bench positions plus 10,000 datagen
positions, Rarog HCE against Reckless's network. The ratio is recorded with
the B.0 handoff and used for every seed; it is a starting point that SPSA
moves, not a result.

**Gating shape for B.** Each cluster: (1) fingerprint changes, so no
neutrality claim; (2) fixed-node diagnostics against the oracle at sample
stride 1 (depth at 300k nodes, EBF, qsearch share, cutoff composition, LMR
re-search rate, NMP conversion), registered as explanation only; (3) a 2,000
game paired diagnostic run at seeds converted but unfitted, to detect a broken
port early (stop rule: worse than −40 Elo means a defect, not a tuning need);
(4) SPSA over the cluster's live coordinates, registered surface and horizon;
(5) the SPRT, `[0,10]` for the selectivity core and `[0,3]` or `[0,5]` for the
smaller clusters, sized from RAR-M10 at the expected value; (6) the ledger
row with calibration. Reject returns the cluster to `RESEARCH` with its
diagnostics; two rejections stop B.

- **B.0 Investigation: current search against the donor, cluster boundaries,
  seeds and instruments — `R3`.** Produce `analysis/search_programme_2026-xx.md`:
  the mechanism-by-mechanism map of Rarog's `negamax`/`quiescence`/picker/
  histories/TT/TM/SMP against Reckless (and Stockfish 19 where Reckless is
  silent), with each difference classified as adopt / keep ours with evidence
  / drop; the exact cluster contents below confirmed or changed; the scale
  ratio; the list of Rarog mechanisms with local evidence that must survive
  (no in-check extension: +30.75 Elo for removing it; root LMR relief; ProbCut
  move filter; typed TT provenance only if a consumer is named); the SPSA
  surface per cluster; the oracle-differential counter set re-mapped to the
  new mechanism names; the AblationMask disposition. Ends with frozen handoffs
  for B.1–B.3 and predictions for B.2. **No engine implementation.**
  Structural questions A.6 left to B.0 (`analysis/consolidation_2026-09-10.md`):
  whether `evidence.rs` (707 lines) has a consumer that changes a search
  decision, else B.1 deletes it with `tests/tt_provenance.rs` and the
  `store_kind_*` counters; the `NodeType` set; whether `Searcher` stays one
  struct or splits into per-thread state plus shared context; and the
  counter re-keying map from today's 286 `diag` names to the new modules.
  **Manta's record is a second worked example, not a donor** (frozen at
  1.1.0, 2026-09-13; `D:/code/manta/docs/adr/0070`, `0071`,
  `EXPERIMENTS.md` MAN-S36). It reached the same cluster cut independently
  after five isolated selectivity gates lost or stalled, and its core cut
  depth-12 nodes 8.5x with about +115 Elo in local matches. Two of its
  findings answer B.0 questions directly: typed TT provenance may govern
  the storage of speculative results (it stopped storing unverified null
  cutoffs) but must never govern the consumption of ordinary bounds
  (downgrading omitted-sibling fail-lows made its TT refuse most non-PV
  upper bounds, and was reverted) — so `evidence.rs` survives only as a
  diagnostics field, if at all; and a proposed history or correction relation
  is measured first as a **shadow producer** (trained, never read, admission
  profile counted under `diag`) before any consumer is written, which is how
  B.2.1 should introduce the continuation-correction tables.
- **B.1 Search restructure, behaviour-neutral — `I1`.** Split `search.rs`
  into the target modules; introduce the `NodeType` constants, the
  `StackEntry`, `PlyArray` and shared-context types; move params into
  `search/params.rs`; remove parameters classified dead in A.2.3; keep every
  mechanism exactly as it is. Done criteria: exact fingerprint
  7,601,220 / EBF 2.474 on magic and PEXT, debug and release suites, clippy,
  pooled-PGO NPS inside ±0.5% of A.8.4. This is the scaffold the clusters land
  on; it earns no strength credit. The move table is the B.1 handoff in
  `analysis/consolidation_2026-09-10.md`. Deletions owed here, all inert at
  default so fingerprint-neutral by construction: the 42 A.2.3 parameters;
  the **root-confidence subsystem** (`RootConfidence`, `root_confidence`,
  `tm_confidence_factor`, the `SharedSearchState` instability slots,
  `diag::RootConfidenceShadow` with the `rootconf_*`/`shadow_*` counters,
  ~210 lines of tests); **SMP iteration skipping** (`SMP_SKIP_*`,
  `helper_skips_iteration`, its test); the two test-only `move_ordering`
  index helpers. The `tm_*` helpers still in `search.rs` join
  `search/time.rs`. Tests move with their subjects; deleted subjects take
  their tests. Tooling commit: regenerate or delete `tools/spsa_configs`,
  re-run the oracle differential once under the new counter names and
  archive it, record RAR-S65–S69 as superseded.
- **B.2 Cluster 1 — the selectivity core — `I2`, then `V`.** One cluster:
  TT entry format with stored raw eval and tt-pv and the estimated-score rule;
  the correction histories and corrected-eval formula (including optimism
  and rule-50 damping in the search, replacing the evaluator's damping if B.0
  says so); the quiet/noisy/pawn/continuation histories with their update
  rules; the move picker stages and scoring; the move-loop pruning (LMP,
  futility, bad-noisy futility, history pruning, SEE pruning); the LMR formula
  and re-search rules; RFP and razoring on the estimated score; hindsight
  reductions; cutoff counting. NMP, ProbCut, singular and extensions keep
  Rarog's current forms in this cluster so that B.3 can measure them
  separately. Sub-steps:
    - **B.2.1** Implement to the B.0 handoff; unit tests for every table's
      bounds and gravity; picker exhaustiveness tests; TT store/probe tests
      including age and replacement; deterministic unwind tests. **Delivery
      shape (from Manta's 6.5.10):** the cluster lands as ordered tickets
      behind one umbrella switch, each ticket keeping the umbrella-off arm at
      the exact B.1 fingerprint; **canaries are anchored to the reference**,
      meaning a tactical position must be solved at the depth classical
      Stockfish `9587eeeb` solves it, not at whatever depth the current build
      manages, and a changed canary is recorded with its cause, never
      re-blessed; and a **decision trace** (`diag`-only, bounded to plies one
      and two under `searchmoves`, printing every prune, reduction and proof
      decision with its inputs) exists before the first ticket, because
      Manta's two seed defects — a static margin overriding a mate in one, a
      count-based skip dropping an unmade mating quiet — were found by that
      trace and are invisible to counters. Implementer and reviewer are
      separate roles; the reviewer's acceptance is recorded before B.2.2.
    - **B.2.2** Diagnostics: oracle differential at stride 1, depth at 300k
      nodes, EBF, tactical suite at fixed depth and equal nodes, 2,000-game
      unfitted paired run. Registered as explanation. **Screen thresholds are
      frozen at registration, before implementation**, so a weak candidate is
      turned back before it spends a maintainer SPRT: the reference-anchored
      geometric branching factor (`tools/branching_profile.ps1`, depths 4 to
      12, fresh process per depth, the phase-4 suite with ordinary and mate
      cohorts reported separately, against classical Stockfish `9587eeeb` on
      the same corpus) must reach a registered ceiling; nodes at a fixed
      depth a registered fraction of B.1's; pooled NPS at least a registered
      floor; the unfitted paired run at least a registered Elo. Below the
      floor the cluster is ablated by component switch once, in a registered
      order, then re-planned; between floor and target the review decides;
      above target B.2.3 proceeds. B.0 sets the numbers from its scale
      comparison; the shape of the ladder is fixed here.
    - **B.2.3** SPSA over the registered live coordinates (expected 40–70),
      `tools/spsa.ps1`, immutable horizon, staged stop. Maintainer-run.
    - **B.2.4** Gate: registered SPRT `[0,10]` against the B.1 head, cap
      sized from RAR-M10; then ledger row and calibration. Accepted head
      becomes the base for B.3. **Precondition: a null calibration on the
      adjudication-free harness.** RAR-M17 removed adjudication on
      2026-09-01, which PROCESS classes as a harness change owing an
      identical-binary null pair, and no calibration row follows it in the
      ledger; every gate since has run on an unvalidated boundary. One
      `tools/sprt.ps1 -Mode calibrate` run of the B.1 head against itself,
      maintainer-run, recorded as a RAR-M row, before this SPRT starts.
- **B.3 Cluster 2 — proof searches and extensions — `I2`, then `V`.** NMP
  with adaptive reduction and verification, ProbCut with reduced-depth
  verification and the TT-served shortcut, singular extensions with
  double/triple margins, multi-cut, negative extension, low-depth singular
  extension, IIR versus hindsight-depth policy. Rarog's own evidence rules:
  no in-check extension unless a new measurement says otherwise. Same
  sub-step shape as B.2 (implement, diagnose, SPSA if curvature justifies,
  SPRT `[0,5]`).
- **B.4 Cluster 3 — quiescence — `I2`, then `V`.** Reckless-shaped qsearch:
  TT cutoff, corrected stand-pat, fail-high interpolation, LMP at three
  moves, SEE pruning by margin, TT write on exit, check evasions only when
  in check. Target: Rarog's qsearch share (62% larger than the oracle's per
  interior node) without losing tactical suite results at equal nodes. SPRT
  `[0,3]`. **Dependency on B.2, recorded from Manta's MAN-S36 review:**
  count-based late-move skipping, low-depth unverified null cutoffs and
  zero-depth reduced probes all assume that a mate threat by a *quiet* move
  stays visible one ply later; a captures-only quiescence makes that false,
  and Manta's core was blind to WAC.001's mate in two through depth nine
  until direct quiet checks were generated at the first quiescence ply.
  Rarog's quiescence today generates captures only unless in check, and B.2
  makes pruning aggressive before B.4 touches quiescence. Therefore B.2's
  canaries include mate threats by quiet moves, B.4 may not remove any
  first-ply check generation B.2 turned out to rely on, and "check evasions
  only when in check" is measured against those canaries, not assumed from
  the donor.
- **B.5 Cluster 4 — root, aspiration, iterative deepening — `I2`, then `V`.**
  Aspiration delta from eval and PV stability, optimism, root move node
  accounting, forgotten-mate and aborted-loss guards, PV table, multi-PV.
  SPRT `[0,3]`. Root-only LMR relief keeps its accepted place unless B.2's
  formula subsumes it, which B.0 decides.
- **B.6 Search SPSA — `V`.** One joint SPSA over the coordinates the four
  clusters left live, only if B.0's curvature evidence and the cluster
  results justify it. Registered surface; PGO bake; SPRT `[0,3]`.
- **B.7 Search speed pass — `I1`, then `V`.** Behaviour-neutral throughput
  work on the new modules (allocation, layout, prefetch, inlining measured in
  search not on the bench), pooled-PGO NPS with a +0.5% floor per change,
  exact fingerprint. The 4.11b lesson stands: a bench-column win is a screen,
  not a result.
- **B.8 Cleanup — `I1`.** Remove dead parameters, unconsumed switches, the
  old `MovePicker`, evidence/provenance plumbing without a named consumer,
  and any diagnostic without an owner. Exact fingerprint; no game gate.
  Owner test for a diagnostic: a counter or probe is kept only if a tracked
  tool or analysis reads it by name (`tools/diag/bench_counters.py`,
  `phase4_differential.py`, or a B-phase ledger row); the rest of the 286
  `diag` names, the `correction_probe`, `lazy_probe` and `smp` submodules
  included, are deleted here unless B.2–B.5 named them. Every remaining
  `#[expect]` must still fire (the lint wall reports unfulfilled ones);
  `search_options.rs`'s single `#[allow]` keeps its written reason or goes.
- **B.9 Checkpoint — `V`.** Re-measure the deficit meters: RAR-O-series
  equal-time G(0) against the oracle, fixed-node depth and EBF, the
  reference-anchored geometric branching factor from B.2.2 (the durable
  tree-shape number; node ratios at one depth overstate the gap because
  nominal depths are not equal coverage across engines), pooled-PGO
  NPS, conversion instrument, and a pool gauntlet against the four target
  engines and Basilisk at 1T (400 games each). Record attributed Elo per
  accepted cluster from the SPRTs, and the checkpoint against the budget
  table. Remove the `ablate` feature afterwards; archive the oracle branches
  as tags (A.2.2) if not already done. **Freeze the search head for C.**

### Active workflow register

One row per open leaf in the active phases (A and B). The checker requires
the state and class here to match GUIDE's suffix. Later phases carry only a
class until they open.

| Leaf | Workflow state | Class | Current decision |
|---|---|---|---|
| B.0 | RESEARCH | R3 | Opens after A.6 (done 2026-09-10); ends with frozen handoffs for B.1–B.3 and the scale ratio |
| B.1 | RESEARCH | I1 | Waits for the B.0 handoff; exact fingerprint required |
| B.2.1 | RESEARCH | I2 | Waits for B.1 |
| B.2.2 | RESEARCH | V | Waits for B.2.1 |
| B.2.3 | RESEARCH | V | Waits for B.2.2; maintainer-run SPSA |
| B.2.4 | RESEARCH | V | Waits for B.2.3; SPRT `[0,10]` registered before games |
| B.3 | RESEARCH | I2 | Waits for the accepted B.2 head |
| B.4 | RESEARCH | I2 | Waits for B.3 |
| B.5 | RESEARCH | I2 | Waits for B.4 |
| B.6 | RESEARCH | V | Conditional on curvature evidence |
| B.7 | RESEARCH | I1 | After B.6 or its skip |
| B.8 | RESEARCH | I1 | After B.7 |
| B.9 | RESEARCH | V | Closes the programme; freezes the search head |

## Phase C — Evaluation programme

**Goal:** recover the measured 329-Elo same-search evaluation deficit, with
the search frozen at the B.9 head, by re-implementing the evaluation families
in Stockfish 11's classical shape where its conditioning is stronger, keeping
ours where the evidence says ours is better, refitting the whole surface
after every family cluster, and giving endgame handling its own bounded
cluster.

**Why Stockfish 11 here.** Reckless has no hand-crafted evaluation. Stockfish
11 is the last classical Stockfish and the reference the maturity record
already compares against (`analysis/hce_maturity_2026-08-25.md`). Its
families and their conditioning are the donor; its constants are seeds on a
different scale and ride the next Texel refit.

**Cluster shape for C.** Each family cluster: (1) the family's current terms
traced and their activation and residual measured on the fitting corpus by
cohort (king danger by attacker count, passers by rank and blocker, threats
by piece pair, endgames by material signature); (2) the donor's form of the
family read for its conditioning and populations; (3) a design that states
which terms are replaced, which stay, and which neighbouring families share
inputs (attack maps, mobility areas, pawn structure) so that the shared inputs
are computed once; (4) implementation with `EvalTrace` coverage for every new
slot and the reconstruction test; (5) a whole-surface Texel refit with the
existing toolchain, frozen test reported once; (6) a PGO bake and SPRT `[0,3]`
(`[3,10]` when the family's residual is large); (7) ledger row. Fit loss is a
screen and a falsifier, never acceptance (RAR-E03 lost 17 Elo with better
loss).

- **C.0 Investigation: family map, residuals and cluster order — `R3`.**
  Produce `analysis/eval_programme_2026-xx.md`: the six-family map from the
  maturity record refreshed on the B.9 head, per-family residual and
  activation evidence, the donor comparison of conditioning, the shared-input
  plan, the cluster order by expected value, the datagen and refit protocol
  for the programme (corpus name, size, splits, label policy from the label
  audit), and frozen handoffs for C.1 and the first family cluster. **No
  engine implementation.**
- **C.1 Evaluation restructure, behaviour-neutral — `I1`.** Split `eval.rs`
  into the target modules; one attack-map and mobility-area producer consumed
  by pieces, king, threats and space; `EvalTrace` unchanged in meaning. Exact
  fingerprint, suites, pooled NPS inside ±0.5%. The move table is the C.1
  handoff in `analysis/consolidation_2026-09-10.md`. Specifics owed here:
  `eval/attacks.rs` is cut out of the 531-line `eval_piece_activity`, which
  also yields pieces, threats and the king-safety inputs — legal only if the
  running `mg`/`eg` order is preserved, because the mop-up and the lazy gate
  read partial sums; the producer's `attacks_from_sq` reads get the
  `debug_assert!` the 2026-08-19 audit asked for; `eval_params!` with its
  137 entries and the `tune`/`texel` I/O move to `eval/params.rs` and
  `eval/trace.rs`; `src/kpk.rs` moves to `eval/endgame/`; `diag_lazy_dual`
  and the 21 `lazy_*` counters stay only if the lazy path stays, since
  `lazy_margin` is decided here. The `texel` trace-reconstruction test is
  part of the suite run, never a speed measurement.
- **C.2 Datagen and label contract for the programme — `V`.** Generate the
  programme's corpus with the B.9 search under the adjudication-off datagen
  profile; audit labels against tablebase truth (existing tool); freeze
  splits and manifests under a new corpus name. Records the label-contradiction
  rate and the corpus hash. Maintainer-run generation. Also produces the
  programme's **fitting manifest**: every evaluation coefficient named with a
  status of *free* (receives gradient), *fixed* (structure: phase divisors,
  caps, sentinels) or *excluded* (nonlinear blocks such as king danger, whose
  caps and truncation a linear model would misrepresent), each exclusion with
  its reason and its contribution carried as a fixed residual per sample. The
  tuner reads the manifest; the hand-kept frozen list in
  `tools/texel-tuner` is replaced by it. Adopted from Manta's
  `manta-hce-fit-v3` (1,109 free, 17 fixed, 103 excluded).
- **C.3 King safety cluster — `I2`, then `V`.** King danger in the donor's
  shape: attacker units and weights, safe and unsafe checks by piece type,
  weak squares in the king ring, king-flank attacks and defence, shelter and
  storm by file with the castling-destination alternative, queen-absent
  reduction, and the nonlinear danger-to-score map. Rarog's existing nonlinear
  danger table is the seed for the map. Refit, gate.
- **C.4 Threats and mobility cluster — `I2`, then `V`.** Mobility with a
  mobility area that excludes own king, queen, blocked pawns and pawn-attacked
  squares; threats: minor and rook attacks on weak enemies, hanging pieces,
  restricted squares, threat by pawn push, king threats, slider and knight
  attacks on the queen, weak queen protection. Shared attack maps from C.1.
  Refit, gate.
- **C.5 Endgame handling and winnability cluster — `R3` investigation with
  `I2`/`V` sub-steps.** The rescoped endgame section. Its goal is measured
  conversion and correct draw recognition where games actually go, not
  coverage of a function list.
    - **C.5.1 Classification and instruments — `R2`.** Adopt the registered
      family order (`tools/diag/endgame_ranking_v2.json`), confirm each family's
      kind (verdict, scale, conversion) against the code, and name the deciding
      instrument per family: theory truth (`endgame_truth.py`), drawn-cohort
      overclaim (`endgame_drawn.py`), conversion (`endgame_conversion.py`),
      floors, and the A.4 conversion audit at the game level.
    - **C.5.2 Generic winnability and scaling — `I2`.** The donor's scale
      factor logic in our form: pawn-count scaling for the stronger side,
      opposite-bishop scaling by non-pawn material and passers, rule-50
      scaling in the scale rather than only the global damping, and an
      initiative/complexity term conditioned on pawn count, king distance and
      both-flank pawns. This is what decides KRPPKRP (5.4% of games, no local
      7-man truth), KPsK (4.5%) and KBPsK (2.6%) generically. Refit, gate with
      an endgame-start cohort and STC.
    - **C.5.3 Conversion cluster: KXK, KBNK, KQKR — `I2`.** Mate drives and
      verdict families with the largest occurrence (KXK 37.8% of the set) and
      the largest measured conversion deficit (KQKR 23/13/3 at 60k/200k/600k
      nodes). Rule-50 damping interaction measured here, sign not assumed.
      Deciding instrument: conversion at bracketed budgets plus theory vetoes.
    - **C.5.4 Rook versus minor cluster: KRKN, KRKB, KRPKB — `I2`.** Three
      families with 100% or 99.6% drawn-cohort overclaim at +300 and the same
      over-representation in Rarog's games; one scaling mechanism, one gate.
    - **C.5.5 Rook and pawn cluster: KRPKR, KRKP, KPK, KPKP — `R2`.** Audit
      the existing scalers and the KPK bitbase integration; repair the 30.7%
      KRPKR overclaim if the drawn cohort supports it; close KPK/KPKP
      `NO_CHANGE` if their 4–5% overclaims do not select a mechanism.
    - **C.5.6 Measure-first families: KPsK, KBPsK, KBPPKB, KQKRPs — `R2`.**
      Measure coverage after C.5.2, decide whether any specific recogniser is
      still justified, otherwise close them as served generically.
    - **C.5.7 Theory sweep: KBPKB, KBPKN, KNNKP, KNNK, KQKP — `I1`.** Sub-1%
      families implemented or confirmed from one dispatcher with Syzygy tests
      and promotion-closure tests, no per-family research cards; `NO_CHANGE`
      where the evidence is clean (KNNK already measured clean).
    - **C.5.8 Endgame gate and closure — `V`.** One endgame-start cohort SPRT
      plus one STC SPRT for the whole C.5 cluster after its refit; floors and
      theory vetoes re-run; conversion instrument re-measured; KRPPKRP's 7-man
      hold recorded as an explicit exclusion unless independent truth appears.
- **C.6 Pawns and passers cluster — `I2`, then `V`.** Passed pawns with king
  proximity, blocker ownership and type, path safety and attack, unstoppable
  and unblocked conditions, rook behind; pawn structure conditionality
  (doubled, isolated, backward, connected by rank and phalanx, weak lever).
  Refit, gate.
- **C.7 Material, imbalance, phase and pieces cluster — `I2`, then `V`.**
  Imbalance in the donor's quadratic form seeded from current material terms,
  phase interpolation review, bishop pair and bishop-pawn colour terms,
  outposts, minor behind pawn, rook on open and semi-open files, trapped
  rook, weak queen, king protector distances. Refit, gate.
- **C.8 Refit cycles — `V`.** After the family clusters: regenerate data with
  the accepted head, refit the whole surface, gate; repeat while a cycle
  accepts, stop at the first that does not. Initialization control (neutral
  start against accepted start) in the first cycle. Each cycle records the
  C.2 manifest it fitted from, so every refit states what was and was not
  fitted; a coefficient's status changes only by a recorded decision, never
  by a cycle quietly widening the free set.
- **C.9 HCE SPSA of nonlinear residue — `V`.** Only the activated nonlinear or
  global terms the linear trace cannot fit; skipped with a written reason if
  the surface is flat.
- **C.10 Search re-fit after the new evaluation — `V`.** The search's
  cp-valued margins were fitted on the B-era scale. One joint SPSA over the
  registered cp coordinates, PGO, SPRT `[0,3]`.
- **C.11 Checkpoint — `V`.** Same-search deficit against Stockfish's classical
  HCE re-measured (a fresh hybrid build at the C head is required; the oracle
  package recipe is on the tagged `hybrid` branch), conversion instrument,
  pooled NPS, pool gauntlet at 1T. **Freeze the classical evaluation.**

## Phase D — Clock, threads, robustness

- **D.1 Time management — `R2` investigation, `I2`/`V` sub-steps.** Audit the
  current clock (budget, overhead, `smp_reserve`; the root-confidence
  consumers are gone since B.1 and are not rebuilt) against the Reckless
  shape; implement the soft/hard bound model with the
  node-fraction and stability multiplier if B.5 has not already; forfeit
  margin sized on a null pair; one registered SPRT `[0,3]` at STC and a
  direction check at `10+0.1`. **Audit checklist**, taken from Manta's
  ADR-0065, whose integrated clock passed inside a cumulative gate where
  Rarog's own root-confidence attempt at the same idea stayed inert: distinct
  optimum and immutable maximum budgets; the maximum rooted at `go` receipt
  and never extended by root or ponder evidence; ponder credit consumed once
  at a discounted rate; the optimum adjusted only by completed exact root
  iterations, from stability, best-move change, score trend and effort
  concentration as bounded factors; an easy root defined as stable
  concentrated effort that spends *less*; no early stop below a minimum
  completed depth; helper threads contributing at most a best-move-change
  count normalised by helper count, so worker count cannot multiply wall
  time. Each item is ticked as present, absent or different in Rarog before
  the donor shape is chosen; the list is a checklist, not a design to copy.
- **D.2 Lazy SMP quality — `R2` investigation, `I2`/`V` sub-steps.** 4T and
  8T scaling against 1T at equal wall time; helper diversity, TT sharing,
  shared correction histories, soft-stop voting, thread-safe counters. The
  helper depth-skip policy is designed fresh from the donor; B.1 deleted the
  inert `smp_iteration_skip` tables and they are not a seed. Competing
  hypothesis to carry into the investigation: Manta's main-authoritative lazy
  SMP (ADR-0064: worker zero alone owns the result, helpers contribute only TT
  evidence from staggered depths, no per-node shared atomics) against Rarog's
  current weighted helper voting. RAR-M46 found no 4T defect, so the question
  is which shape scales to G.1, not which repairs a deficit. Gate:
  4T SPRT `[0,5]` against the 1T-accepted head at 4T, no affinity, null pair
  first. High-thread and NUMA remain G.1.
  **PREMISE CONTRADICTED, RAR-M46, 2026-09-11 — re-scope before spending work
  here.** This leaf is written to recover strength from immature lazy SMP. At
  4T against the reference field there is no such deficit: Rarog's deficits are
  26 to 75 Elo *smaller* at 4T than at 1T against three of the four targets,
  its performance rating is +31 against its frozen 1T rating, and it beats
  Basilisk 1.10.0 at 4T (+25) having lost at 1T (−23). That does **not** show
  the SMP is good in absolute terms — it beats engines designed for two to four
  cores, which is a low bar, and says nothing about G.1's 8/16/32 threads where
  lazy SMP characteristically degrades. The cheap re-justification is a
  **self-scaling curve** — Rarog at 1T/2T/4T/8T against one fixed opponent —
  which measures Rarog's own scaling without needing a donor comparison. Until
  something like that establishes a defect, D.2 has no measured target.
  **Maintainer steer, 2026-09-11: compare against modern engines, not this
  field.** The RAR-M46 result only shows Rarog beating engines that never
  targeted SMP; the question D.2 actually asks is whether Rarog's lazy SMP is
  weak against engines that did. The clean instrument is **self-relative
  scaling**: for each engine play it against *itself* at 4T versus 1T and read
  the Elo it gains from the threads. That removes every cross-engine confound —
  strength, evaluation, NPS accounting — because each engine is its own
  control. Run it for Rarog, **Reckless** (`reckless-windows-avx2.exe`) and
  **Stockfish** (`stockfish-windows-x86-64-bmi2.exe`), all present in
  `D:/chess/engines/`, and compare the three gains. SMP value grows with time
  control, so the clock is a registration decision, not a default.
- **D.3 Engine lifecycle and protocol robustness — `R2`, then `I1`.** UCI
  parsing and dispatch, stop/ponder/infinite semantics, new-game resets,
  malformed input, panic reporting, Syzygy probe policy and thread safety.
  Deterministic tests; zero crashes over the pool tournaments. Owns the last
  move of the target layout: `engine.rs`, `engine_command.rs`,
  `uci_protocol.rs`, `search_options.rs` and `bench.rs`/`wac.rs` go under
  `src/uci/` in the same leaf, behaviour-neutral, exact fingerprint.
- **D.4 Tablebase policy — `R2`.** Root and interior probing depth and limits,
  WDL/DTZ use in conversion, interaction with the C.5 recognisers. Endgame-start
  cohort and conversion instrument decide.
  **Candidate, recorded 2026-09-10, not yet evaluated: replace vendored Fathom
  with `shakmaty-syzygy`.** A pure-Rust Syzygy prober (niklasf), v0.28.1,
  updated 2026-06-19, ~88k downloads, **GPL-3.0-or-later**, which matches
  Rarog's own licence. It would delete `vendor/fathom`, the `cc`
  build-dependency, `build.rs`'s C branch, the tier-dependent
  `TB_NO_HW_POP_COUNT` hazard that shipped 15 illegal `popcntq` in 2.3.0/2.3.1,
  the macOS `-fprofile-use` warning (RAR-P21) and the six `unsafe extern "C"`
  declarations in `syzygy.rs` — and it would put probe code inside Rust's LTO
  and PGO for the first time. Against that: it would be the crate's **first
  runtime dependency**, `[dependencies]` being empty today, and it pulls
  `shakmaty` itself, a move-generation library we do not need and would have to
  convert positions into. Evaluated here rather than sooner because the benefit
  is near-zero Elo while the risk lands on C.5's truth source, and D.4 must
  validate probe semantics anyway. **Two questions to answer before it is
  proposed, not after:** (1) does it expose **DTZ** as well as WDL, since
  `syzygy.rs` calls `tb_probe_root_dtz`; (2) what does the `shakmaty`
  dependency actually weigh, in tree size and in conversion cost per probe. A
  semantic difference in DTZ or rule-50 handling would corrupt endgame evidence
  silently rather than announce itself, so parity against the C path on the C.5
  fixtures is the acceptance condition. **Not a candidate:** `fathom-syzygy`
  (malu) — it is a binding to the same C library, v0.1.0 untouched since
  2023-01-02, so it keeps every C problem and adds a dependency.

## Phase E — Classical checkpoint and release

- **E.1 Attribution checkpoint — `V`.** Final head against 2.3.2 and against
  the B.9 and C.11 heads at STC, `10+0.1` and 4T; attributed Elo per programme
  from the accepted SPRTs; deficit meters; NPS; the maturity checklist
  (family map without unknown rows, every slot with a fitting instrument,
  every accepted representation reconstructing through `EvalTrace`).
- **E.2 Target gate — `V`.** The pool measurement defined in section 1, at 1T
  and 4T. Met, or not met with the measured shortfall per engine recorded.
- **E.3 Release — `M`/`V`.** Version, changelog, release notes, fmt, debug and
  release suites, clippy, feature builds, fingerprint, PGO assets, ISA
  verification, CI matrix, tag and publish on maintainer instruction. Version
  is 3.0.0 if E.2 is met, else 2.5.0. **Two workflow checks are added before
  this release, as tooling work that may land any time earlier:** (1) the
  release job fails when the tag does not equal the manifest version — today
  `build.yml` names the built file from `Cargo.toml` and the uploaded asset
  from the tag, so the two can disagree silently, which is the shape of the
  RAR-E16 baseline confusion; (2) every asset in the matrix runs `bench 13`
  and the job **asserts** one fingerprint across all of them, replacing the
  comment that tells the operator to read the node count in the log. Manta's
  release workflow already does both and rejects the tag otherwise.
## Phase F — NNUE

**Rules.** Own data only, generated by Rarog's classical head and later by its
NNUE heads. Reckless is the runtime and trainer-pipeline donor; the
architecture ladder is ours. The classical evaluation stays in the tree as the
datagen baseline and the fallback until F.9 replaces it in releases.

- **F.0 Investigation: runtime, data pipeline and first architecture — `R3`.**
  Board event interface and accumulator ownership (dirty pieces, per-thread
  per-ply accumulators, king buckets, refresh cache); trainer choice
  (`D:/code/net_trainer` against Bullet) with feature ordering, quantisation
  and export contracts; data format, deduplication and split policy; the first
  architecture (768×N perspective network with output buckets); the cost
  ledger inherited from the board audit. Frozen handoffs for F.1–F.4.
- **F.1 Board events and accumulator scaffolding — `I2`.** Behaviour-neutral
  for the HCE: factual move deltas, evaluator-owned stacks, validity and
  refresh semantics, randomized unwind tests, exact fingerprint, pooled NPS
  cost recorded.
- **F.2 Data generation at scale — `V`.** 30–60M unique positions from the
  classical head under the adjudication-off profile, by-game splits,
  manifests, tablebase and hard-position cohorts; hashes frozen. Maintainer-run.
- **F.3 Trainer hardening and baseline nets — `I2`, then `V`.** Deterministic
  pipeline, two seeds per configuration, validation selects, frozen test
  reports once.
- **F.4 Scalar integration — `I2`.** `quantised.bin` contract, integer-exact
  conformance against the trainer's reference evaluation, clean HCE fallback.
- **F.5 Incremental and SIMD — `I2`, then `V`.** Same-net incremental parity
  on every move type, SIMD tiers (AVX2, PEXT builds, ARM NEON), scalar
  reference retained, pooled-PGO NPS attribution.
- **F.6 Search re-fit for the network — `V`.** Score scale, correction
  histories, margins, qsearch and SEE thresholds re-fitted on the new
  evaluator (C.10's protocol).
- **F.7 Architecture ladder — `R3` with `I2`/`V` sub-steps.** Output buckets,
  king buckets with mirroring, then relation and threat inputs as in
  Reckless, one axis at a time; each net gated against the previous.
- **F.8 Data frontier — `V`.** On-policy refresh with the strongest net,
  deduplication, hard-position mining; repeat while a cycle accepts.
- **F.9 NNUE release — `M`/`V`.** Beat the classical release at STC, LTC
  and 4T; platform matrix; publish.
- **F.10 CCRL top-100 gate — `V`.** Submit; the list decides. Shortfall
  measured against the pool and fed back into F.7/F.8.

## Phase G — Scaling, platforms and the top 50

- **G.1 High-thread and NUMA — `R2`, then `I2`.** 8/16/32T scaling, TT and
  net placement, large pages, thread affinity policy.
- **G.2 Platform and product — `I1`.** Chess960 on demand, distributed
  testing when typical gains reach 1–3 Elo, and the **optional universal
  x86-64 binary**. The universal work is fully designed and deliberately
  unscheduled: `analysis/universal_binary_2026-09.md` carries the measured
  tier value, the link prototype's two findings (Rust symbols isolate under
  `-C metadata`; C symbols silently collapse to whichever copy links first),
  Stockfish's mechanism read from its source, the four candidate designs with
  their runtime costs, and the traps — chiefly that a `-C metadata` mismatch
  makes PGO apply **no profile at all**, without an error. It may never be
  done. The triggers that would revive it are recorded in that document, the
  strongest being NNUE in Phase F, which is what makes a wider tier ladder pay
  (196 of Stockfish's 249 ISA-specific lines are NNUE inference).
- **G.3 Frontier — `R3`.** Larger nets, data scaling, search fit at LTC; the
  top-50 gate is the CCRL list again.

## 3. Measurement protocols

| Meter | Protocol | Owner |
|---|---|---|
| Pool score | Colosseum, fixed pool with Houdini 3, `3+0.03`, UHO, no adjudication, 400 games per pair, 1T and 4T | A.5, B.9, C.11, E.2 |
| Search deficit G(0) | `tools/sprt.ps1` paired, head against the frozen `hybrid` oracle, 3,000 games, equal time, no adjudication | A.8.3, B.9 |
| Evaluation deficit | Hybrid at the current head against Stockfish-HCE hybrid, same search, 2,400 games | C.11 |
| Cluster acceptance | Registered final-PGO SPRT, brackets per rule 5, `[0,10]` for B.2 | every cluster |
| Neutral change | Exact fingerprint on magic and PEXT, debug and release suites, `nps_multibuild.ps1` pooled PGO | B.1, B.7, C.1, F.1 |
| Conversion | `tools/diag/conversion_audit.py` on the latest pool tournament | every checkpoint |
| Fixed-node shape | oracle differential at stride 1, depth at 300k nodes, EBF, tactical suite at fixed depth and equal nodes | B clusters |
| Endgame layers | theory truth, drawn overclaim, conversion at 60k/200k/600k, floors | C.5 |

Sizing every SPRT: `tools/spsa_convergence_model.py` and RAR-M10's drift
model at the expected value, before registration. Bracket, cap, book, clock
and adjudication never change after games are seen.

## 4. Release rules

- A release ships only from a head whose every accepted cluster has a ledger
  row and whose deficit meters are recorded at the checkpoint before it.
- 3.0.0 requires the E.2 gate met; 2.4.0 requires at least +40 Elo at STC over
  2.3.2 with the lower bound above +25, positive LTC and 4T lower bounds.
- NNUE releases require a win over the last classical release at STC, LTC and
  4T, and a clean platform matrix.
- Tag, push and publish only on maintainer instruction.

## 5. Documentation ownership

| File | Purpose |
|---|---|
| `GUIDE.md` | Operator guide, model mapping, prompts, the full checkbox board, checkpoint and next action |
| `PLAN.md` | This roadmap: objective, rules, phases, protocols |
| `EXPERIMENTS.md` | Frozen predictions, results, calibration, retry triggers, recipes |
| `PROCESS.md` | Research/handoff template and recurring build, fit, gate and release procedures |
| `HISTORY.md` | Completed work, retired numbering and the number map; never a source of the next step |
| `analysis/` | Per-leaf analyses and measurement records; raw artifacts stay local and ignored |
| `docs/archive/` | Verbatim archived roadmaps |

## 6. Number map

Every identifier from the archived roadmap is retired. Where a retired open
leaf continues here, this is the mapping; everything else is history.

| Retired | Continues as | Note |
|---|---|---|
| 4.12 (20 endgame functions) | C.5 (8 leaves) | rescoped from function coverage to conversion and generic scaling |
| 4.13, 4.14 (labels, refit cycles) | C.2, C.8 | inside the evaluation programme |
| 4.13a (HCE audit) | C.0 | |
| 4.15, 4.15a–c, 4.16, 4.18 (search audits, SPSA, cleanup) | B.0–B.9 | replaced by the search programme |
| 4.17 (time management) | D.1 | |
| 4.19, 4.20 (checkpoint, release) | E.1–E.3 | |
| A.3.4, A.5, A.6, A.7 (before 2026-09-10) | A.9, A.8, A.5, A.6 | Phase A reordered so the numbering matches execution: improvements, instrument and analysis, then the version bump, then baselines on the bumped binary, then the release. Only open leaves moved; A.1-A.3 keep their numbers |
| 4.21 (universal binary) | G.2 | investigated under A.4 and deferred 2026-09-10 as optional; `analysis/universal_binary_2026-09.md` holds the design and the revival triggers |
| Phase 5, 6, 7 (NNUE runway, baseline, frontier) | F | |
| Phase 8 (scaling) | G | |
| Phase 9 (classical fallback) | dropped | the classical evaluation stays as datagen baseline and fallback by construction |
