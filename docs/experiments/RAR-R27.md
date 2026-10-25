# RAR-R27 — D.1.6.3: the gate of the LMR correction cap at one ply, on top of `d1`'s stop at twice the target — REGISTERED 2026-10-25, before any game

Indexed under *4. Root search, time management and SMP* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-25 before any game. The gate, the
  `10+0.1` read and the probe are the maintainer's commands.
- **Kind:** D.1.6.3's gate, the frozen disposition of RAR-R26's pass. The
  cap changes shared search signals in the rated configuration, so the gate
  is AGENTS' repair case 3, the symmetric `[-5,5]` with its cap.
- **Hypothesis:** capping what the correction term takes off a late-move
  reduction at one ply is no weaker than `d1`'s head at `3+0.03`.
  - It stops reduced moves being searched past their budget where
    corrections are large.
  - With the stop at twice the target, it ends the won-ending stalls the
    probe measures: the cap ends rec3's single long window, and the stop
    ends rec1's walk once the win resolves (RAR-R26's probe).
- **Baseline:** `d1` at `6253027` (engine source `c4da889`):
  `tools/test_engines/rarog-d1head2-pext-pgo.exe`, a clean PGO build from a
  detached worktree, SHA-256 `619D7730603551B9…3555D649A4`, bench
  **13,987,223**, version `2.7.0-dev`. It carries the stop at twice the
  target (`a79efe4`) and `CoreLmrCorrectionCap` 0.
- **Candidate:** the throwaway branch `d16-gate3` at **`7703768`** on
  `6253027`.
  - It sets `CoreLmrCorrectionCap` 1024 as the default, version
    `2.7.0-dev+d16gate3`.
  - Recipe: `analysis/arm_patches/7703768-rar-r27-d16gate3.patch` (SHA-256
    `7cfd85b20549aec0…c7c84626ed`, three changed lines), baked by
    `bake_corrcap.py` in the evidence folder. `git apply --check` against
    `6253027` passes.
  - Binary: `tools/test_engines/rarog-d16gate3-pext-pgo.exe`, a clean PGO
    build from its own worktree, SHA-256 `304D1CF9B829BC77…ECEDDCBB02`,
    **bench 11,480,295**. That equals the diag build's bench with the
    option set (RAR-R26's evidence).
  - The arm's own unit test `the_correction_cap_bounds_the_term_and_is_off_by_default`
    asserts the default 0, so it fails on the arm by construction. The arm
    is a gate binary, not a landing.
- **What the gate can see and cannot.**
  - RAR-R26 read the same change at equal speed: −1.4 ± 9.0 Elo at
    `3+0.03`, +1.7 ± 11.5 at `10+0.1`, with the capped arm 0.09–0.24 ply
    deeper at equal time.
  - A PGO pair carries about ±1.5% NPS, about ±3 Elo at this clock.
  - `[-5,5]` excludes harm near −5 and accepts a change near even. The
    won-ending repair itself is carried by the probe.

## The gate (frozen)

- **SPRT:** Colosseum `sprt-repair` (`[-5,5]` nElo, pentanomial,
  `model=normalized`, α = β = 0.05), `3+0.03`, 20 ms margin, Hash 64, one
  thread, the wrapper's slot count, the UHO book in random order, no
  adjudication, **cap 20,000 pairs**, seed 20261111, run directory
  `rar-r27-gate`.
- **`ltc`:** `match-fixed-ltc`, 1,000 games at `10+0.1`, seed 20261112,
  `rar-r27-ltc`.
- **`probe`:** `tm_rec_probe_d16.py`, rec1 and rec3 at `60000+600` and
  `180000+2000`, both binaries, the probe's own idle check. **It passes
  when no candidate row reaches the maximum.**
- **Commands:** `run_r27.ps1 -Read probe|gate|ltc` in
  `analysis/artifacts/d16-cascade-2026-10-23/tree-growth-2026-10-25/`.
  Both matches dry-ran clean on 2026-10-25: host 3–5%, the pinned
  colosseum-cli 0.4.0, benches 11,480,295 and 13,987,223 matched, and the
  SPRT reads H0 nElo ≤ −5, H1 ≥ +5 with the 20,000-pair cap.
- **Order:** probe, then gate, then `ltc`.

## Dispositions

- **H1, with the probe passing:**
  - The cap lands on `d1` as the default in its own engine commit, and the
    unit test is updated for the new default.
  - The fingerprint is re-declared, 11,480,295 on the arm; the landed
    production build's own bench is recorded then.
  - `TmFailHighTarget` stays at twice the target. RAR-R25's retry trigger
    has fired, and RAR-R26's probe answers it: without the stop rec1 runs
    to the maximum.
  - Cleanup: the D.1.6 instruments the landing does not use are removed
    (`AspFailHighReduce`, `AspFailHighRecover`, `AspRecentre`,
    `QsFailSoft`, `CoreLmrDeeperCap`, `CoreExtensionBudgetEighths`,
    `AspDeltaSquareDiv`, `CoreLmrDeeperBudget`, `TmDepthFreeze`,
    `AspKnownWinWindow`, `MopupKnownWin`); a coordinate whose default is
    today's behaviour returns to its constant.
  - The changelog entry joins the stop's under Fixed.
  - D.1.6 closes, and D.1.2.1's cascade half ticks.
  - The `d16-gate3` branch is deleted once the recipe and this entry carry
    the arm.
- **H0:** the cap does not land.
  - The won-ending tree's growth is accepted, and the stop at twice the
    target remains the cap on the stalls.
  - D.1.6 closes on the stop, and the cap's instrument is removed in the
    cleanup.
  - Retry trigger: a refit of the LMR terms with the term's slope free (a
    search cluster or F.6).
- **The cap of 20,000 pairs, unresolved:** recorded as such. Nothing lands
  without a maintainer decision.
- The `ltc` read and the probe inform; they accept nothing on their own. A
  failed candidate probe row returns the cap to research.

## PRE-REGISTERED PREDICTION (frozen before exposure)

- **Gate:** **−1 ± 6 Elo** at `3+0.03` (−2 ± 10 nElo; 80% band −9 to +6
  Elo). That is RAR-R26's −1.4 at equal speed, with the PGO pair's ±3 and no
  speed change expected. P(H1) 0.35, P(cap) 0.35, P(H0) 0.30.
- **`ltc`:** the point estimate at or above the gate's, P 0.6.
- **Probe:** PASS, P 0.75. RAR-R26 measured the pair on the tune build with
  every row at or under its optimum. The PGO build crosses each iteration
  sooner, which moves which iteration meets the target; that is the most
  likely miss.
