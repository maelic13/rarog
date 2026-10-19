# RAR-M72 — verification: the C.13 head against the pre-C.13 head, a 2,000-game harm read

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-15, before any game. The run is the maintainer's overnight command, at the maintainer's request (a check that C.13 damaged nothing).
- **Kind:** a measurement with a harm rule (AGENTS' second repair case). It accepts nothing.
- **Arms:**
  - *C.13 head:* `rarog-c13best-pext-pgo.exe` (`3C92DC47…`, `931dcca`, **10,521,141 / EBF 2.521**). This is C.13.1's connected-rooks repair and the zero-term removal; C.13.2 to C.13.6 left the engine unchanged.
  - *Pre-C.13 head:* `rarog-c8cycle2-pext-pgo.exe` (`A3A11321…`, `75c3db5`, **14,126,183 / EBF 2.557**), RAR-E61's accepted arm.
- **What can differ:**
  - The repair changes play only where a side has three or more rooks: 5 of 194,444 held-out rows in the static differential.
  - The removal is behaviour-neutral (the fingerprint reproduced) and may run slightly faster.
  - Expected: no difference.
- **Read (frozen):** Colosseum `-Mode match` (`match-fixed.toml`): **2,000 games**, `3+0.03`, one thread, Hash 64, the UHO book in random order, seed 4209, no adjudication.
- **Harm rule (frozen):** a point estimate below **−13 nElo** flags harm. Before any other work, C.13.1's two engine commits are then re-examined: the differential, the removal's neutrality, and NPS.
  - Power, from AGENTS: the standard error is about 7.8 nElo, so a true −10 nElo is caught with probability 0.36, −20 with 0.82 and −25 with 0.94. Passing means no harm beyond roughly −20 nElo; it cannot see a small loss.
- **PRE-REGISTERED PREDICTION (freeze before exposure):** **+1 nElo**, 80% band [−10, +12]; probability of a harm flag 0.05.
- **Run attempt 2026-10-15 23:44 (overnight, after RAR-E68):** **not played.** The harness's
  preflight measured the host at 24% CPU right after RAR-E68 finished and refused to start, as
  it must; no game was played, so nothing was read. The overnight script started it
  immediately, without waiting for the host to settle. The read stays registered as frozen
  and runs once on an idle host.
- **Result (played 2026-10-16, 08:33–08:54 local, by the maintainer's command; `tools/results/rar-m72`, manifest `colosseum_match_rar-m72_20261016_083320.manifest.txt`):**
  - *Conditions, checked from the manifest:*
    - the registered arms and fingerprints (`3C92DC47…` at `931dcca`, 10,521,141; `A3A11321…` at `75c3db5`, 14,126,183);
    - the pinned runner (colosseum-cli 0.3.0), UHO seed 4209, no adjudication;
    - the host 2.2% busy with no waiver, and the repository clean at `af76db6`.
  - **+1.9 ± 8.5 Elo, +3.4 ± 15.2 nElo** over 2,000 games, for the C.13 head: W/D/L 501 / 1,009 / 490, pentanomial [24, 213, 511, 232, 20]. No faults, every termination normal. `colosseum_recount.py` agrees exactly.
- **Disposition:** +3.4 nElo is above the −13 nElo line: **no harm flag.** C.13.1 is not re-examined, and C.13's work may reach `master`. As registered, this excludes harm beyond roughly −20 nElo and cannot see a small loss.
- **PREDICTION CALIBRATION (appended after exposure):**
  - Original prediction (not rewritten): +1 nElo, 80% band [−10, +12]; probability of a harm flag 0.05.
  - Observed: +3.4 nElo, inside the band and 2.4 above its centre, well within one standard error. No miss: the expectation of no difference held, and the read had the power it was registered with.
- **Artifacts:** `tools/results/rar-m72`, `analysis/artifacts/c136/recount-rar-m72.log`.
