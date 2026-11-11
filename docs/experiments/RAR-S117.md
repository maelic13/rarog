# RAR-S117 — RAR-S116's read with its wire restated: D.6.4's consumers and the kept move's ordering on replayed gate games

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-11, after [RAR-S116](RAR-S116.md)'s
  wire failed and before any deciding count of either read was looked at. The
  read is the agent's (deterministic, no games).
- **Kind:** research and diagnosis after a rejected repair. No game, no gate.
- Disposition: open — registered, the deciding counts not looked at.
- **Baseline:** `d6` at `fbb9a67`, engine behaviour of `ee0b3e4`, bench
  13,038,236 / EBF 2.540.

## Why a new registration

RAR-S116's wire asked the replay of arm 0 to reproduce the games' mean depth
within 0.10. It read 0.146 under, so that read is void by its own rule:

| 60 games, moves without a mate score | replay | the games | under by |
|---|---:|---:|---:|
| arm 0 on the baseline's moves | 12.954 | 13.100 | 0.146 |
| arm 4 on the candidate's moves | 13.808 | 13.933 | 0.125 |
| the candidate above the baseline | 0.854 | 0.833 | |

- **The tolerance was the error.** It was set from a six-game smoke that
  happened to read 0.002.
- **The offset has a stated cause.** RAR-S116 said the game records the last
  `info` line's nodes and the engine searched on, so a replay's table is
  poorer than the game's. That lowers both arms alike.
- **The instrument is only ever read as a difference** between arms on the
  same positions and node counts, and the difference the games measured is
  reproduced within 0.02.

A level tolerance was the wrong wire. It is restated here, before any
deciding count is known, and nothing else changes.

## What is unchanged, by reference to RAR-S116

The question, the competing explanations, the arms, the patch
(`analysis/arm_patches/d64-rar-s116-screen.patch`), the executable (SHA-256
`ce249878…`), the replay script (SHA-256 `1a02dff6…`), the 60 games, the
deciding quantity *r*, the three rules, the quantities reported without
deciding, and predictions 2 to 8. Prediction 1, that the wire passes at 0.85,
was a miss and stays one.

## Wire (restated)

- The replay's depth difference, arm 4 on the candidate's moves less arm 0 on
  the baseline's, is within 0.10 of the games' own difference over the same
  moves. **Passed:** 0.854 against 0.833.
- Each arm's level is within 0.25 of the games'. **Passed:** 0.146 and 0.125
  under.
- Every search dumps each counter exactly once, and the arms search the same
  moves. The script asserts both.

## Exposure, stated

Seen: RAR-S116's exposure, and its wire log, which holds the six numbers
above and the share of moves replayed to the game's depth (34% and 33%) and
move (68% and 66%).

Not looked at: any replay count of the eight arms, the collect's log, and the
report. RAR-S116's run wrote them while this was being registered. The
searches are deterministic, so a rerun would write the same files, and this
read takes them as they are.

## Rules and predictions

RAR-S116's, unchanged: `READY_FOR_IMPLEMENTATION` if *r* ≤ 0.98 with the
first-move share at fail-low entries not lower; `NO_CHANGE` if *r* ≥ 0.99;
`RESEARCH` otherwise. Void if the restated wire fails.
