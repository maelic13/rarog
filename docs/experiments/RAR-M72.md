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
- **Result:** pending.
