# PLAN closed leaves, verbatim, 2026-09-14

Moved from `PLAN.md` on 2026-09-14 (PLAN B.2.0.1), when each closed leaf there
was reduced to one line. The text below is exactly what PLAN held for Phase A,
B.0, B.1, B.2.0 and B.2.0.1 at that date, including their "original scope" paragraphs.
The dated records are in `HISTORY.md`; the evidence is in the ledger rows each
leaf cites. Paths inside may name files that later moved or were deleted.
Frozen: nothing here is current work.

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
  Rarog 57/12, Basilisk 40/12 on tournament `41768fe9`. **Re-read on the
  release games 2026-09-13 (RAR-M49, tournament `5e539523`): Rarog 88/19 in
  3,600, the same rate; Basilisk 94/12 — the surplus-over-Basilisk reading is
  retired, Rarog's own stable rate is what C.5 works from.** The tool is
  re-run at every programme checkpoint; it is a diagnostic layer, never an
  acceptance layer.
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

## Phase B — closed leaves

- **B.0 Investigation — DONE 2026-09-13, `NO_CHANGE` to source.**
  `analysis/search_programme_2026-09-13.md` (RAR-M50 for the artifacts).
  **Findings that change the programme:** (1) the tree-shape target was
  wrong — over depths 4–14 Rarog's geometric branching is **1.630** against
  the oracle's **1.736** and Reckless's **1.697**; its excess is a
  shallow-depth constant factor (3.5x the oracle's nodes at depth 4, 1.9x at
  depth 14; 1.02 quiescence nodes per interior node), so B.2.2's branching
  screen is a window, not a ceiling; (2) the deficit is decision quality at a
  fixed budget — on WAC at 100k nodes the oracle solves **242**, Reckless 224,
  Rarog **200** (400k: 267/247/237) while the three are level at nominal depth
  10; median depth at 300k nodes on the phase-4 suite 16 against the oracle's
  19; (3) the scale ratio is **0.457** (evaluation units, against the
  search-facing Reckless eval; 0.406 raw) and **0.75** (SEE units), but one
  scalar cannot seed margins sized for NNUE accuracy on an HCE whose
  search-minus-static residual averages 128 cp — seeds use a three-column
  rule (Reckless converted, the classical oracle converted at 0.485/0.40,
  Rarog fitted); (4) two contract shapes carry the gap and were invisible to
  every constant candidate: **46.7% of LMR reductions land in quiescence**
  (both donors floor the reduced depth at one ply) and **1.3% of reductions
  are re-searched** (oracle ~4%) at a mean reduction of 3.05 plies. **Decisions:**
  `NodeType {Root, PV, NonPV}` with runtime `cut_node`; `ThreadData` plus
  `SharedContext` without changing table ownership; `evidence.rs` deleted
  (no consumer changes a decision at default; both donors let singular read
  ProbCut-depth entries); killers, countermove and low-ply history dropped
  with B.2; IIR kept in B.2 beside hindsight, decided in B.3; NMP entry margin
  above beta, ProbCut's 4.7c filter shown to be the donor's own; the counter
  families to delete and keep are mapped; AblationMask bits re-declared in
  B.2.1. **Handoffs frozen** for B.1, B.2 and B.3; B.2's prediction is
  frozen (+35 Elo after fitting, 90% [+5, +70]; unfitted −10 ± 30). Original
  scope: Produce `analysis/search_programme_2026-xx.md`:
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
- **B.1 Search restructure, behaviour-neutral — `I1`. CLOSED 2026-09-14
  (RAR-P24).** Seven engine commits (`866cf9b`..`fcf8a2a`) plus a tooling
  commit (`c35260d`), every one at the exact **7,601,220 / EBF 2.474** on
  magic and PEXT with all 40 positions identical; fmt, clippy in every CI
  feature set plus `--all-features`, 284 release / 283 debug tests. Layout:
  `src/search/{mod,node,movepick,history,correction,stack,thread,threads,
  shared,time,params}.rs`; `evidence.rs`, `move_ordering.rs`,
  `search_threads.rs` and `time_manager.rs` are gone. **Deviations from the
  handoff, each recorded:** 44 parameters removed, not 42 (A.2.3's count of
  99 missed 11 declarations; the extra two, `TmConfHigh`/`TmConfLow`, fed only
  the root-confidence clock); `SharedContext` is the renamed
  `SharedSearchState`, and the TT handle stays on `Searcher` because moving it
  would change its access path, not its ownership; the sentinel stack is read
  through `PlyArray::back(ply, n)` rather than a signed index type, so `ply`
  stays `usize`; the node types are `Root`/`Pv`/`NonPv` (clippy's acronym
  lint); the oracle differential was re-run as a paired pre-/post-B.1 control
  because the oracle's DLL no longer reproduces the 47c file's oracle column
  (the two reports are identical); the MoveEvidence stage classification is
  inline in the `move_seen_*` census. **Speed is outside the ±0.5% window in
  the favourable direction: +6.30%, 95% [+5.78%, +6.84%]** against the
  RAR-M48 pool, interleaved on the same host — the per-move reduction
  estimate the prospective-depth switch computed for every move, the
  root-confidence snapshot and the per-node branches of the removed
  switches. The §11 baselines reproduced exactly (branching 1.630, WAC 200
  and 237, median depth 16, agreement 40/50). **B.2.2's NPS floor (0.90x)
  is read against the B.1 pool `tools/results/nps-b1-20260914/`, measured
  interleaved, not against 3.19 M**: this host read the RAR-M48 pool 3.6%
  slower on 2026-09-14 than on 2026-09-11. Original scope follows. **B.0 handoff frozen
  2026-09-13 (`analysis/search_programme_2026-09-13.md` §6, §13.1):
  `NodeType {Root, PV, NonPV}` with runtime `cut_node`; `ThreadData` plus
  `SharedContext` without changing table ownership; `Stack` with a sentinel
  entry and signed indexing; `evidence.rs`, `tests/tt_provenance.rs` and
  the dead counter families deleted (the freed TT flag bit stays unused);
  killers, countermove and low-ply history stay until B.2; the §11
  baselines re-measured on the B.1 binary.** Split `search.rs`
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
    - **B.2.0 Architecture and design review of the whole engine on the B.1
      head, then its accepted upgrades — `R3` for the review, `I2` for the
      upgrades. CLOSED 2026-09-14 (RAR-P25).** All twelve tickets landed in
      twelve commits (`687a8bf`..`0493283`), every engine commit at the exact
      **7,601,220 / EBF 2.474** on magic and PEXT with all 40 positions
      identical; fmt, clippy on default, diag, tune, ablate, texel,
      all-features and the workspace; 284 release / 283 debug tests. Pooled
      PGO NPS against the B.1 pool, interleaved: **+0.21%, 95% [−0.34%,
      +0.68%]**; T6 alone against the T5 head **+0.04% [−0.47%, +0.51%]**, so
      the table-policy refactor stays. The diag build at stride 1 reproduces
      the B.1 counter set except the three never-incremented counters T1
      deleted. Retired phase, step and ledger numbers in the owned `src/`
      files, the repository configuration and the tools: zero (316 remain in
      `eval.rs` and the five search mechanism files, owned by C.1 and
      B.2–B.5). **Deviations, each recorded in RAR-P25:** `benches/board.rs`
      imported four free generators the review missed, so it moved to the
      `Board` methods; 15 public types stay `pub` because the integration
      tests reach them through public fields and signatures; narrowing exposed
      13 dead items, deleted, and three feature-only items, gated; `go perft`
      reaches the handler as `GoRequest::Perft` and `setoption` reports an
      `OptionUpdate`; the helper-spawn notice moved to `Searcher::configure`;
      the tuner has its own lockfile and CI a `texel` clippy step;
      `engine_coverage.rs` split into four files, not three; the
      comment-only tickets T7, T8, T10 and T11 were verified on their combined
      state and then committed separately. `src/` shrank 379 lines, not the
      predicted 700–1,200, because rewritten comments replaced prose rather
      than deleting it and the command enum and go keyword table added
      lines. Found in passing: `tools/texel/fit_complete.ps1` pinned the
      stale fingerprint 6,901,489; since `b95dd8b` it reads GUIDE's
      Development head row instead.
      Original scope follows. Added 2026-09-14 by maintainer decision; runs before B.2.1.
      Numbered inside B.2 because the status board has two levels, not
      because it belongs to the cluster: it is behaviour-neutral and earns no
      strength credit. **Review** (`analysis/architecture_review_2026-09.md`):
      module ownership and public surfaces after B.1; the board → evaluation
      → search → UCI data flow and where state is duplicated or threaded
      through arguments; lifecycle, error and protocol contracts; allocation
      and layout choices that carry measured evidence and those that carry
      only history; test structure (in-file tests, `tests/`, fixtures) and
      the tooling that reads engine output; a comment audit — `src/` holds
      about 460 references to retired phase and step numbers. Every finding
      is classified keep / upgrade now / upgrade in its owner leaf, with the
      evidence and the owner named; behaviour-changing upgrades go to their
      B/C/D owner, never into this leaf. **Upgrades** (same leaf, state
      `READY_FOR_IMPLEMENTATION` → `IMPLEMENTED` → `LOCAL_QUALIFIED` →
      `CLOSED`, register class updated to `I2` at that transition): the
      accepted behaviour-neutral refactors and the comment hygiene in modules
      no B or C cluster rewrites — `board/`, `tt.rs`, `engine*.rs`,
      `uci_protocol.rs`, `search_options.rs`, `infra.rs`, `diag.rs`,
      `syzygy.rs`, `bench.rs`, `wac.rs`, `main.rs`, `tools/` — plus the
      structural (not mechanism) comments of the `search/` scaffold. Search
      mechanism comments are rewritten by the cluster that rewrites the
      mechanism (B.2–B.5); `eval.rs` comments by C.1. Comment rule, now in
      AGENTS: short, states the problem or invariant, no roadmap phase or
      step numbers, deleted when it only records history. Done criteria:
      exact fingerprint 7,601,220 / EBF 2.474 on magic and PEXT, debug and
      release suites, fmt, clippy, pooled-PGO NPS within ±0.5% of the B.1
      pool unless a speed change is registered on its own. E.1 re-runs the
      review on the B.9 and C.11 heads. **Review done 2026-09-14**
      (`analysis/architecture_review_2026-09.md`, RAR-M51): the layering is
      inward and sound and no structural rewrite is justified before the
      clusters; the weight is comments, surface and duplication. Twelve
      behaviour-neutral tickets are frozen in its §5: dead items and
      aliases; visibility follows use; one move-generation API; private
      `Board` fields; one definition each of the duplicated helpers and the
      LMR seed from the parameters; a tagged `EngineCommand` with a
      `ClearHash` command; options carry only options; a search output port
      instead of `println!` in `search/`; the TT policy factored once over
      its two backends with an NPS read of its own; the tuner out of the
      workspace; a tools index; comment hygiene in the owned files and,
      added to the scope, the repository configuration (`Cargo.toml`,
      `build.rs`, `.cargo/`, `rust-toolchain.toml`, `.github/`). The
      `Searcher` split into per-thread state, per-search configuration and
      engine-owned resources is designed there (§4.3) and is B.2.1's ticket
      0, not this leaf's. State `READY_FOR_IMPLEMENTATION`, class `I2`.
        - **B.2.0.1 Repository and document restructure, reformat and
          clean-up — `R3` review done 2026-09-14, `I2` for the work.**
          Added 2026-09-14 by maintainer decision: everything B.2.0 does
          not touch — the nine top-level documents, `analysis/`, `docs/`,
          the tool READMEs and the repository root. Standard: one
          document, one purpose, stated in its first paragraph; no content
          another document owns; nothing a reader can no longer act on;
          every tracked file reachable from an index or a citation.
          **Review** (`analysis/repository_review_2026-09.md`, RAR-M52):
          PLAN carries 734 lines of closed-leaf narrative; HISTORY holds a
          second copy of the archived Phase-4 tracker while its
          legacy-tracker section, which the resolution table points at, is
          empty; EXPERIMENTS holds essays and two row formats; PROCESS and
          AGENTS hold each other's material; README's test command is the
          one CI forbids; 44 dangling repository paths; seven of nine logo
          files unused against the storage policy; 13 superseded analyses
          beside live contracts with no index. **Nine tickets** in its §5:
          U1 HISTORY and the recovered legacy archive; U2 PLAN collapsed to
          open work, the number map to HISTORY, a dead-path check in
          `check_guide.py`; U3 one ledger row format; U4 AGENTS rule-first,
          drafted and shown before commit; U5 PROCESS procedures only; U6
          README; U7 the `analysis/` index and archive; U8 logo files and
          the storage policy; U9 GUIDE prose. Documents only, no engine
          input, every ticket its own commit; archive, never delete; no
          number, date, ID or verdict changes while it moves. Done criteria
          and the frozen size prediction are in the review. **Starts when
          B.2.0 is CLOSED**, because B.2.0's T7, T9 and T11 edit PROCESS,
          AGENTS, CI and `tools/`.
