# Rarog experiment ledger

This is the indexed maintainer record of measured experiments and the lessons
that may inform later work. It is not a roadmap: [`PLAN.md`](PLAN.md) owns what
will be done and in what order. [`CHANGELOG.md`](../CHANGELOG.md) remains the
user-facing release record.

**Layout (2026-10-05).** Each experiment's entry is its own file,
`docs/experiments/<ID>.md`; the tables below are the index, one line per
experiment with a short title and disposition cut from the entry and linked
to it. Search the index by ID or subsystem, then open the entry you need. An
entry and its index row change in the same commit; `check_guide.py` fails
when the index and the files disagree. Six entries whose rows had more cells
than their table's columns keep those cells as numbered parts.

Every lesson below is conditional. A result describes one engine state, test
protocol, time control, compiler and machine population; it does not establish
a universal chess-programming rule. An experiment from Basilisk is only a
prior for Rarog and never bypasses Rarog's own gates.

**Numbering note.** Rows cite three retired numbering schemes (legacy phases
such as `8.2(a)`, the Phase-4 roadmap before and after its 2026-09-04
renumbering) and the paths of their time; they are evidence and are not
rewritten. `HISTORY.md` resolves every scheme to its archived tracker and maps
retired open leaves onto the current lettered roadmap. Rows from RAR-M45 on
cite the current roadmap. Records too long for a row live in
`analysis/ledger_records_2026-09-14.md` or in a cited analysis packet.

**Commit note.** 71 commits the rows cite by hash are on no branch or tag
(the development line before the 2.4.0 squash, and deleted experiment
branches). `analysis/ledger_commits_2026-09-27.md` lists each with its date,
subject, citing rows and where its recipe lives, and names the local bundle
that holds every one of them.

## Contents

- [1. How to use this ledger](#1-how-to-use-this-ledger)
  - [Result and evidence vocabulary](#result-and-evidence-vocabulary)
  - [Recording contract](#recording-contract)
  - [Prediction freeze and calibration](#prediction-freeze-and-calibration)
- [2. Measurement, harness and tuning](#2-measurement-harness-and-tuning)
- [3. Search and selectivity](#3-search-and-selectivity)
  - [Search-accuracy decomposition](#search-accuracy-decomposition)
  - [Search-oracle observations](#search-oracle-observations)
  - [Registered, open](#registered-open)
  - [Accepted or retained](#accepted-or-retained)
  - [Rejected, neutral or deferred](#rejected-neutral-or-deferred)
- [4. Root search, time management and SMP](#4-root-search-time-management-and-smp)
- [5. Evaluation and data experiments](#5-evaluation-and-data-experiments)
- [6. Throughput, build and platforms](#6-throughput-build-and-platforms)
- [7. Correctness and protocol lessons](#7-correctness-and-protocol-lessons)
- [8. Cross-engine evidence imported from Basilisk](#8-cross-engine-evidence-imported-from-basilisk)
- [9. Open retry map](#9-open-retry-map)
- [10. Template for a new experiment](#10-template-for-a-new-experiment)

## 1. How to use this ledger

Search the contents by subsystem before proposing a mechanism, tune or retry.
Use the stable IDs in commit messages and `PLAN.md` when a prior result changes
a future decision. Do not copy the tables into `PLAN.md`.

### Result and evidence vocabulary

| Term | Meaning in this document |
|---|---|
| **Accepted** | Passed the registered gate and entered an accepted baseline. |
| **Retained** | Kept for correctness, infrastructure or structural value; any Elo figure may be unresolved. |
| **Rejected** | Failed its registered gate or had a clear adverse measurement and was reverted. |
| **Neutral/inconclusive** | Evidence did not distinguish a useful effect at the tested resolution. |
| **Observation** | Diagnostic evidence, not an acceptance verdict. |
| **No-change** | Research closed because the measured premise/opportunity did not justify an engine change. |
| **Deferred** | Not decided under present prerequisites; owner and objective resume condition are recorded. |
| **Imported prior** | Evidence from Basilisk; useful for ordering or designing a Rarog test, never for accepting it. |

Unless a row says otherwise, historical strength tests used paired games at
fast time control. Results before the pinned-harness repair of 2026-07-21 may
carry scheduler-placement bias. Fast-TC deltas are non-additive and may
compress or reverse at longer TC.

### Recording contract

Register before exposure, then update the same entry when accepting, reverting
or closing. Record the research question; baseline/candidate SHAs and any
dirty-diff identity; binary/compiler/PGO identity; hypothesis, competing
explanations and interacting consumers; cheapest prior falsifier; frozen
prediction and confidence; falsification and stop rules; full conditions;
diagnostics separately from the verdict; result/disposition; calibration and
postmortem; conditional lesson; objective retry trigger; and artifacts.

The prediction must state expected diagnostic movement, expected Elo sign or
range only when defensible, probability the candidate is positive/useful, and
the most likely failure mode. It is frozen once any deciding result is exposed.
Correct only a genuine clerical error, mark that correction explicitly, and
never fabricate a prediction for a historical entry that lacked one.

Use cautious language: “under these conditions this suggests …”, not “feature
X is good/bad”. If conditions or artifacts are unknown, say so.

### Prediction freeze and calibration

Keep **what was believed before exposure** separate from the postmortem. A good
retrospective explanation is not evidence that the result was predicted. After
a surprise ask which part of the original causal model was wrong, not why the
outcome now seems obvious.

At each phase checkpoint, review new prospectively registered experiments by
category (search/selectivity, HCE, endgame, TT/cache, SMP/time,
board/performance, tooling, data/tuning or NNUE): Was the sign right? Was the
magnitude systematically optimistic? Did high confidence predict reliability?
Which mechanism or interaction was missed? Did the instrument fail? Was any
rejected idea retried without its trigger? Record only repeated calibration
lessons; do not build a score or rewrite the frozen entries.

## 2. Measurement, harness and tuning

| ID | Experiment | Disposition |
|---|---|---|
| [RAR-M01](experiments/RAR-M01.md) | Early fixed-`movetime` gates were compared with the deployed clock path at `3+0.03` | Fixed movetime manufactured false negatives |
| [RAR-M02](experiments/RAR-M02.md) | Historical unpinned fastchess runs were audited under explicit physical-core placement on the Ryzen 9 5950X | Real affinity/topology defects were found |
| [RAR-M03](experiments/RAR-M03.md) | Identical-binary null testing after harness changes | The old symmetric `[-3,+3]` setup had zero expected LLR drift at equality |
| [RAR-M04](experiments/RAR-M04.md) | Opening-book migration to paired UHO games | Retained because it increased decisive-game signal and aligned SPRT, SPSA and gauntlet conditions |
| [RAR-M05](experiments/RAR-M05.md) | SPSA schedule audit: iteration/game units, PowerShell `$A`/`$a` collision and integer perturbation resolution | Several schedule defects were repaired |
| [RAR-M06](experiments/RAR-M06.md) | Resignation threshold replay against 69,350 historical games | `400/3` one-sided was too aggressive for Rarog's scale |
| [RAR-M07](experiments/RAR-M07.md) | Staged self-play gains were checked in an external engine cohort | The 2.2 cycle's roughly +316 staged result transferred as about +240 over 2.1.0 |
| [RAR-M08](experiments/RAR-M08.md) | The 36,400-game 2026-08-05 rating tournament used a 2.4-dev binary with interim values from an unfinished … | Closed observation |
| [RAR-M11](experiments/RAR-M11.md) | Phase-4 consolidated SPSA go/no-go review | Canceled before launch and removed in 2.3.2 |
| [RAR-M10](experiments/RAR-M10.md) | Calibrating this harness's LLR drift so a game budget can be derived rather than guessed | Retained method tool |
| [RAR-M09](experiments/RAR-M09.md) | Phase-4.1 normal versus diagnostic release builds on the Ryzen 9 5950X, non-PGO, `bench 13` plus four … | Retained infrastructure |
| [RAR-M12](experiments/RAR-M12.md) | Phase-4 step 4.0 — baseline and oracle freeze | Closed; baseline accepted |
| [RAR-M13](experiments/RAR-M13.md) | Adjudication unified on 600/3 two-sided, 2026-08-18 | Instrument change, no games |
| [RAR-M14](experiments/RAR-M14.md) | Time-forfeit floor at concurrency 14, Threads 1, `3+0.03`, measured from RAR-E06's 3,915-game PGN | Observation |
| [RAR-M15](experiments/RAR-M15.md) | How often the 20 reference endgames actually occur, and what adjudication does to them | Observation, decisive for 4.9a's gate design |
| [RAR-M16](experiments/RAR-M16.md) | What adjudication actually buys, measured from stored logs | Observation |
| [RAR-M17](experiments/RAR-M17.md) | Adjudication dropped as the harness default, 2026-09-01, by maintainer decision on RAR-M16 | Instrument change, no games |
| [RAR-M18](experiments/RAR-M18.md) | `datagen-v3`: Syzygy tablebase truth for labels, 2026-09-01 | Instrument addition, verified |
| [RAR-M19](experiments/RAR-M19.md) | Audit of the SEE / move-ordering piece-value scale, 2026-09-05 | `piece_value()` has not moved since the initial commit while the evaluator was refit four times underneath it |
| [RAR-M20](experiments/RAR-M20.md) | Board audit and native three-engine comparison, 2026-09-05 | Basilisk faster in all five comparable workloads |
| [RAR-M21](experiments/RAR-M21.md) | RAR-M21 — 4.11.7 budget transfer, registered 2026-09-05 | Recorded in full in the packet |
| [RAR-M22](experiments/RAR-M22.md) | RAR-M22 — 4.11.8 datagen label audit, COMPLETE 2026-09-06 | Recorded in full in the packet |
| [RAR-M23](experiments/RAR-M23.md) | RAR-M23 — 4.11.9 mate-drive promotion closure, COMPLETE 2026-09-06 | Recorded in full in the packet |
| [RAR-M24](experiments/RAR-M24.md) | RAR-M24 — 4.11.10 corrected conversion claims, COMPLETE 2026-09-06 | Recorded in full in the packet |
| [RAR-M25](experiments/RAR-M25.md) | RAR-M25 — 4.11b.2 board-v2 instrument and correctness corpus, COMPLETE | Recorded in full in the packet |
| [RAR-M26](experiments/RAR-M26.md) | RAR-M26 — 4.11b.3 parser and fullmove boundaries, COMPLETE 2026-09-06 | Recorded in full in the packet |
| [RAR-M27](experiments/RAR-M27.md) | RAR-M27 — 4.11b.4 SEE contracts and independent fixtures, COMPLETE | Recorded in full in the packet |
| [RAR-M28](experiments/RAR-M28.md) | RAR-M28 — 4.11b.5 SEE legality and promotion repair, COMPLETE 2026-09-07 | Recorded in full in the packet |
| [RAR-M29](experiments/RAR-M29.md) | RAR-M29 — 4.11b.6 neutral SEE injection and normalized timing, COMPLETE | Recorded in full in the packet |
| [RAR-M30](experiments/RAR-M30.md) | RAR-M30 — 4.11b.7 full-search board profile, COMPLETE 2026-09-07 | Recorded in full in the packet |
| [RAR-M31](experiments/RAR-M31.md) | RAR-M31 — 4.11b.8 pin discovery measurement, recorded 2026-09-07 | Recorded in full in the packet |
| [RAR-M32](experiments/RAR-M32.md) | RAR-M32 — 4.11b.9 fused ordinary relocation, 2026-09-07 | Recorded in full in the packet |
| [RAR-M33](experiments/RAR-M33.md) | RAR-M33 — 4.11b.9 fused ordinary relocation re-measured on a verified-idle | Recorded in full in the packet |
| [RAR-M34](experiments/RAR-M34.md) | RAR-M34 — 4.11b.10 shared pin/check information, COMPLETE 2026-09-08 | Recorded in full in the packet |
| [RAR-M35](experiments/RAR-M35.md) | RAR-M35 -- 4.11b.11 incremental SEE attacker maintenance, COMPLETE 2026-09-08 | Recorded in full in the packet |
| [RAR-M36](experiments/RAR-M36.md) | RAR-M36 — full-search board profile refreshed at head, COMPLETE 2026-09-08 | Recorded in full in the packet |
| [RAR-M37](experiments/RAR-M37.md) | RAR-M37 — 4.11b.12 king-square caching, COMPLETE 2026-09-08 | Recorded in full in the packet |
| [RAR-M38](experiments/RAR-M38.md) | RAR-M38 — 4.11b.13 history capacity and mutation contracts, COMPLETE | Recorded in full in the packet |
| [RAR-M39](experiments/RAR-M39.md) | RAR-M39 — 4.11b.14 larger board representation, COMPLETE 2026-09-08 | Recorded in full in the packet |
| [RAR-M40](experiments/RAR-M40.md) | RAR-M40 — 4.11b.15 draw-state policy boundary, COMPLETE 2026-09-08 | Recorded in full in the packet |
| [RAR-M41](experiments/RAR-M41.md) | RAR-M41 — 4.11b.16 integrated board cluster qualification, COMPLETE | Recorded in full in the packet |
| [RAR-M60](experiments/RAR-M60.md) | Colosseum CLI against fastchess on the same gate — the SPRT half of the 2026-09-17/18 harness parity … | All three accepted H1, and on the closest-sized pair the two instruments agree to within a tenth of an nElo |
| [RAR-M61](experiments/RAR-M61.md) | Colosseum CLI against fastchess on the same fixed match — the parity read no stopping rule biases … | The instruments agree, and the spread among Colosseum's own runs is as wide as the gap between the instruments |
| [RAR-M62](experiments/RAR-M62.md) | The Colosseum tune shape — 15 slots and 30 games per iteration, measured 2026-09-18, recorded 2026-09-21 from … | 15 x 30 runs about 1.65x the games per hour of 14 x 32 |

## 3. Search and selectivity

### Search-accuracy decomposition

Recovered from branch `spsa_impr` before it was retired; the tooling these rows
produced (`tools/sprt.ps1 -Nodes`, `tools/pgn_depth_at_nodes.py`,
`tools/diag_search_quality.ps1`, `diag.rs` `cutoff_first_move`) already shipped
in 2.3.2, but the results themselves had never been entered here. They predate
RAR-O01/O02 and reach the same conclusion from a different direction, which is
why they are recorded rather than superseded. Baselines are Rarog 2.3.1 versus
Basilisk 1.9.1, so magnitudes are not comparable to the 2.3.2-era oracle rows.

| ID | Experiment | Disposition |
|---|---|---|
| [RAR-S56](experiments/RAR-S56.md) | Phase-4 step 4.7a — null-move entry contract, PREPARED AND HELD | No games. Deliberately not gated |
| [RAR-S57](experiments/RAR-S57.md) | Phase-4 cluster 4.7 — ACCEPTED | PASSED, H1 accepted at 2,838 games — a fifth of the cap |
| [RAR-S58](experiments/RAR-S58.md) | Phase-4 cluster 4.7 ablation — COMPLETE | Arm C carries the whole result; arm A has no measured marginal contribution |
| [RAR-S59](experiments/RAR-S59.md) | Phase-4 step 4.5.3 — continuation-attribution asymmetry, measured with zero games | REJECTED on measurement |
| [RAR-S60](experiments/RAR-S60.md) | Phase-4 steps 4.5.3/4.5.4 — the six deferred per-ply fields, disposed with zero games | Two adopted, four rejected |
| [RAR-S61](experiments/RAR-S61.md) | Phase-4 cluster 4.5 (A) — REGISTERED, NOT YET RUN | UNRESOLVED at the 16,000-game cap — NOT promoted |
| [RAR-S62](experiments/RAR-S62.md) | Phase-4 cluster 4.5 ablation — REGISTERED, NOT YET RUN | H0 ACCEPTED at 4,436 games — the correctness fix COSTS strength |
| [RAR-S63](experiments/RAR-S63.md) | Phase-4 step 4.5.3 — ProbCut speculative-move contract, third variant. REGISTERED, NOT YET RUN | Dead tie, unresolved at the cap |
| [RAR-S64](experiments/RAR-S64.md) | Phase-4 cluster 4.5 (A) — RE-MEASUREMENT after the stale-reduction fix. REGISTERED, NOT YET RUN | H0 ACCEPTED at 8,088 games — Cluster A is worth NOTHING once the defect is removed |
| [RAR-S65](experiments/RAR-S65.md) | SUPERSEDED by B.2, never run (B.1, 2026-09-14): the switch or parameter this arm needed was removed as inert … | STOPPED at 12,522 games, not resolved, NOT promoted |
| [RAR-S66](experiments/RAR-S66.md) | SUPERSEDED by B.2, never run (B.1, 2026-09-14): the switch or parameter this arm needed was removed as inert … | STOPPED at 13,882 games, not resolved, NOT promoted |
| [RAR-S67](experiments/RAR-S67.md) | SUPERSEDED by B.2, never run (B.1, 2026-09-14): the switch or parameter this arm needed was removed as inert … | STOPPED early, tracking to H0 — NOT promoted |
| [RAR-S68](experiments/RAR-S68.md) | SUPERSEDED by B.2, never run (B.1, 2026-09-14): the switch or parameter this arm needed was removed as inert … | *Pending.* |
| [RAR-S69](experiments/RAR-S69.md) | SUPERSEDED by B.2, never run (B.1, 2026-09-14): the switch or parameter this arm needed was removed as inert … | *Pending.* |
| [RAR-E15](experiments/RAR-E15.md) | Phase-4 step 4.11b.17 — integrated board cluster playing gate. ACCEPTED, H1 at 1,950 games | Registered bounds `[-5,5]` nElo, cap 16,000 games, Alpha/Beta 0.05, fixed before any games |
| [RAR-M42](experiments/RAR-M42.md) | 4.11b.18 endgame evidence refresh after the accepted board head, COMPLETE 2026-09-09; section 4.11b CLOSED | Layer 1 clean, floors PASS both arms, 4.12 order verified UNCHANGED |
| [RAR-M43](experiments/RAR-M43.md) | SUPERSEDED 2026-09-09 by RAR-M44(d); raw session retained | The control did not reproduce RAR-M20, which governs how everything else may be read |
| [RAR-M44](experiments/RAR-M44.md) | 4.11b.19 research: move-list delivery probe, 2026-09-09; (a) and (b) IMPLEMENTED 2026-09-09 (`55e228a` … | Legal captures +40.52% |
| [RAR-M45](experiments/RAR-M45.md) | A.8.1 reference pool refresh with Houdini 3, 1T - REGISTERED, NOT YET RUN | Prediction, frozen 2026-09-09 |
| [RAR-M46](experiments/RAR-M46.md) | A.8.2 four-thread pool against the four targets and Basilisk - REGISTERED, NOT YET RUN | Prediction, frozen 2026-09-09 |
| [RAR-E16](experiments/RAR-E16.md) | A.3.2 consolidation release gate - REGISTERED, NOT YET RUN | Prediction, frozen 2026-09-09 |
| [RAR-S70](experiments/RAR-S70.md) | Root-only LMR relief, 1536/1024 ply — REGISTERED, NOT YET RUN | ACCEPTED — H1 at `[0,3]` nElo. Elo +2.33 +/- 1.85, nElo +3.58 +/- 2.85, LOS 99.30%, LLR 2.95 over 56,928 … |
| [RAR-S52](experiments/RAR-S52.md) | Search-quality ratio readout at the 2.3.1 head | Observation |
| [RAR-S53](experiments/RAR-S53.md) | Paired two-arm decomposition, Rarog 2.3.1 versus Basilisk 1.9.1: identical engines, book and seed, run once … | Observation, decisive for cycle direction |
| [RAR-S54](experiments/RAR-S54.md) | Blind uniform 15% shift of the whole selectivity surface toward **less** pruning, on a throwaway probe … | Positive, deliberately stopped at LLR 1.68 of 2.94 and recorded as a STOP, not an H1 |

### Closed 4.6 follow-ups

These two results existed only in ignored `tools/results` artifacts when the
SearchCore line was reverted. They are recorded here so the stopped samples
cannot be mistaken for pending work. Neither reached a registered SPRT
boundary; both are dispositions, not accepted H0 claims.

| ID | Experiment | Disposition |
|---|---|---|
| [RAR-S71](experiments/RAR-S71.md) | 4.6.2 SearchCore rewrite, registered before games at `43d5174` | Stopped manually before a boundary; rejected as the development route and reverted |
| [RAR-S72](experiments/RAR-S72.md) | 4.6.1 quiet SEE oracle screen | Stopped diagnostic null; candidate remains default-off |

### Search-oracle observations

These experiments identify a development target; they do not accept the hybrid
as Rarog code or assign Elo to an individual Stockfish mechanism. Pairwise Elo
below is the ordinary logistic estimate from score and is approximate, not the
project's paired-pentanomial SPRT estimator. They are the evidence base for the
Phase-4 programme in `PLAN.md` §4. They size Rarog's own targets and order
Rarog's own work; nothing here makes resembling Stockfish a goal.

| ID | Experiment | Disposition |
|---|---|---|
| [RAR-S55](experiments/RAR-S55.md) | Phase-4 step 4.2 — first differential reading | Observation |
| [RAR-O01](experiments/RAR-O01.md) | Stage-1 evaluator-isolation experiment | Observation, completed |
| [RAR-O02](experiments/RAR-O02.md) | No-adjudication confirmation of RAR-O01, stopped after 1,238/2,400 games because the architectural decision … | Observation, sufficient and deliberately stopped |
| [RAR-O03](experiments/RAR-O03.md) | A.8.3 oracle deficit meter G(0) on the release head - REGISTERED, NOT YET RUN | Prediction, frozen 2026-09-09 |
| [RAR-O04](experiments/RAR-O04.md) | B.9 oracle deficit meter G(0) on the search head — REGISTERED 2026-10-03, before any game; PLAYED 2026-10-03 … | Played 2026-10-03 18:00:29–18:34:15 UTC |
| [RAR-O05](experiments/RAR-O05.md) | C.0 evaluation meter at the start of Phase C: the same-search gap at equal nodes and at equal time — REGISTERED 2026-10-05, before any game; RUN 2026-10-05 | Observation: equal nodes +181.7 ± 19.0 (1,000 games), equal time +266.3 ± 19.9 (980 games, interrupted); the equal-node figure is Phase C's meter baseline |

### Registered, open

| ID | Experiment | Disposition |
|---|---|---|
| [RAR-S73](experiments/RAR-S73.md) | B.2 cluster 1, the selectivity core — REGISTERED 2026-09-14 before any game; B.2.4a passed 2026-09-16 and … | B.2.1 implementation record, zero games; diagnostics, not a verdict |
| [RAR-S74](experiments/RAR-S74.md) | B.2.2.2 categorical screen on the `b2core` arm — REGISTERED 2026-09-15, no games played | Runs (a)–(e) played 2026-09-15, 2,000 games each, 10,000 games in all, one time forfeit (run (d), round 133 … |
| [RAR-S75](experiments/RAR-S75.md) | B.2.3 SPSA on the `b2core` arm — REGISTERED 2026-09-15, before any game; nothing launched | Finished 2026-09-20: 5,000 of 5,000 iterations, 160,000 games |
| [RAR-S76](experiments/RAR-S76.md) | B.2.3 checkpoint peek, theta at iteration 3,900 against the unfitted head — REGISTERED 2026-09-19, before any … | Played 2026-09-19 |
| [RAR-S77](experiments/RAR-S77.md) | B.2.3 tail: theta at N = 5,000 against theta at 3,900, SPRT — REGISTERED 2026-09-19, before any game; the … | Played 2026-09-20 |
| [RAR-S78](experiments/RAR-S78.md) | Re-tune of the most-moved B.2.3 coordinates on Colosseum — DESIGN REGISTERED 2026-09-19. The full … | Tune finished 2026-09-24 |
| [RAR-S85](experiments/RAR-S85.md) | B.4 cluster 3, quiescence — REGISTERED 2026-09-28, before implementation; no game played | B.4.1 stopped at T1, 2026-09-28 |
| [RAR-S86](experiments/RAR-S86.md) | B.4.3, the quiescence cluster's tune — REGISTERED 2026-09-29, before any game | Block 1 played 2026-09-29/30 |
| [RAR-S87](experiments/RAR-S87.md) | B.4.3 categoricals, two 2,000-game reads — REGISTERED 2026-09-29, before any game | Played 2026-09-29 |
| [RAR-S88](experiments/RAR-S88.md) | B.4.4, the cluster 3 gate — REGISTERED 2026-09-30, before any game; binaries built and hashed | Played 2026-09-30 |
| [RAR-S89](experiments/RAR-S89.md) | B.5 cluster 4, the root: research and gate — REGISTERED 2026-09-30, before any game; binaries built and hashed | Played 2026-09-30 |
| [RAR-S90](experiments/RAR-S90.md) | B.5.5, the aspiration surface read by games — REGISTERED 2026-09-30, before any game | Played 2026-09-30 |
| [RAR-S91](experiments/RAR-S91.md) | B.5.1 card (b), draw-score randomisation, one 2,000-game categorical read — REGISTERED 2026-09-30, before any … | Played 2026-09-30 |
| [RAR-S92](experiments/RAR-S92.md) | B.5.4 optimism, one 2,000-game categorical read — REGISTERED 2026-10-01, before any game | Played 2026-10-01 |
| [RAR-S93](experiments/RAR-S93.md) | B.5.2.2 activation read, the tablebase work on every move — REGISTERED 2026-10-01, before any game | Played 2026-10-01 |
| [RAR-S94](experiments/RAR-S94.md) | B.5.2.2 gate, the tablebase repair with tables configured — REGISTERED 2026-10-01, before any game | Played 2026-10-01 |
| [RAR-S95](experiments/RAR-S95.md) | B.6 go/no-go from the tune journals — zero games, 2026-10-01. Rule and predictions were frozen in … | Read |
| [RAR-S96](experiments/RAR-S96.md) | `CoreIirMinDepth`, two 2,000-game categorical reads — REGISTERED 2026-10-01, before any game | Played 2026-10-01 |
| [RAR-S97](experiments/RAR-S97.md) | Tablebase PV extension start rule, activation and correctness read — REGISTERED 2026-10-01, before any game | Before any game, the paired behaviour read |
| [RAR-S98](experiments/RAR-S98.md) | B.7 speed pass in games — REGISTERED 2026-10-02, before any game | Played 2026-10-02 |
| [RAR-S79](experiments/RAR-S79.md) | B.3 cluster 2, proof searches and extensions — REGISTERED 2026-09-23, before implementation; no game played | Paired run played 2026-09-24 |
| [RAR-S80](experiments/RAR-S80.md) | B.3.2 categoricals, four 2,000-game runs — REGISTERED 2026-09-24, before any game | Runs 1–2 played 2026-09-24 |
| [RAR-S81](experiments/RAR-S81.md) | B.3.3 SPSA on the `b3proof` arm, in blocks — REGISTERED 2026-09-25, before the configuration is generated and … | Stopped 2026-09-25 at iteration 940 of 2,000 |
| [RAR-S82](experiments/RAR-S82.md) | B.3.3 on the corrected surface: two categorical reads, then the blocked SPSA — REGISTERED 2026-09-25, before … | Part 1 played 2026-09-25 |
| [RAR-S83](experiments/RAR-S83.md) | B.3.3, the tune resumed from the measured margin — REGISTERED 2026-09-26, before any game | Seed-set read played 2026-09-26 |
| [RAR-S84](experiments/RAR-S84.md) | B.3.4, the cluster 2 gate — REGISTERED 2026-09-27, before any game; binaries built and hashed | Played 2026-09-27 |

### Accepted or retained

| ID | Experiment | Disposition |
|---|---|---|
| [RAR-S01](experiments/RAR-S01.md) | Pruning/margin SPSA Group B in the early search state | Accepted, +6.17 ± 4.88 nElo |
| [RAR-S02](experiments/RAR-S02.md) | Qsearch TT-bound stand-pat refinement | Accepted, about +6.5 Elo |
| [RAR-S03](experiments/RAR-S03.md) | Per-move quiet futility pruning | Accepted, +7.98 ± 4.42 Elo |
| [RAR-S04](experiments/RAR-S04.md) | Joint pruning-family re-tune after the 2.2 HCE cycle | Accepted, +12.07 Elo |
| [RAR-S05](experiments/RAR-S05.md) | Split history bonus/malus semantics and consumers | Accepted, +22.13 Elo |
| [RAR-S06](experiments/RAR-S06.md) | Removal of the unconditional in-check extension from the then-current search | Accepted, +30.75 Elo |
| [RAR-S07](experiments/RAR-S07.md) | Broader history mechanism/tuning bundle | Accepted, +6.01 Elo |
| [RAR-S08](experiments/RAR-S08.md) | Broad selectivity refit after the search-accuracy decomposition | Accepted, +15.33 ± 7.34 nElo |
| [RAR-S09](experiments/RAR-S09.md) | Zero-reduction LMR floor | Accepted, +9.13 ± 5.45 nElo |
| [RAR-S10](experiments/RAR-S10.md) | Persistent `RootMove` records for mean, mean-square, PV, nodes and fail state | Retained infrastructure; isolated Elo unresolved |

### Rejected, neutral or deferred

| ID | Experiment | Disposition |
|---|---|---|
| [RAR-S11](experiments/RAR-S11.md) | Direct port of a more Stockfish-like ProbCut formula into the older search | Rejected, −24.5 ± 8.5 Elo; reverted |
| [RAR-S12](experiments/RAR-S12.md) | Removed/altered history aging before and after the bonus/malus split | Rejected twice |
| [RAR-S13](experiments/RAR-S13.md) | `cutoffCnt` plus full LMR-family SPSA | Rejected, −7.78 ± 8.00 |
| [RAR-S14](experiments/RAR-S14.md) | Post-LMR “do deeper” mechanism | Rejected, −7.29 Elo |
| [RAR-S15](experiments/RAR-S15.md) | Fail-soft qsearch against constants fitted around fail-hard bounds | Rejected, −5.96 Elo; reverted |
| [RAR-S16](experiments/RAR-S16.md) | Correction-history tune with `CorrGuardCapture=1` | Aggregate mechanism washed at +1.43 |
| [RAR-S17](experiments/RAR-S17.md) | Aspiration re-centering, verified mechanically against the tuned head | Rejected, −4.52 Elo |
| [RAR-S18](experiments/RAR-S18.md) | Full FIDE-like draw/repetition bundle, then a reduced null-clock/fence variant | Rejected, −7.21 ± 6.03 and −11.91 ± 7.67 |
| [RAR-S19](experiments/RAR-S19.md) | SEE pin-awareness verified against an independent legal-exchange oracle | Standalone rejected, −8.49 Elo |
| [RAR-S20](experiments/RAR-S20.md) | Half-run aspiration SPSA snapshot `ba3170b` (`15/148/149/9/20/8/0`) versus clean `p1043-base` | Rejected by acceptance rule after manual stop |
| [RAR-S21](experiments/RAR-S21.md) | Phase-4.1 diagnostic `bench 13`, 1T, deterministic sampled interaction map on the retained 6,502,902-node … | Observation |
| [RAR-S22](experiments/RAR-S22.md) | Phase-4.2 opening static audit of the TT producer/consumer graph at `f35bc09`, plus a re-run of RAR-S21's … | Observation |
| [RAR-S23](experiments/RAR-S23.md) | Phase-4.2 typed evidence refactor at `47f3ac6`: `OutcomeKind`/`NodeEvidence`/`MoveEvidence`, all 7 producers … | Retained infrastructure, behaviour-neutral |
| [RAR-S24](experiments/RAR-S24.md) | Phase-4.2b shadow test at `7815054`: what a confidence/depth penalty on window-contradicting inexact bounds … | Observation, and it contradicts the hypothesis that motivated it |
| [RAR-S25](experiments/RAR-S25.md) | Phase-4.3a provenance-hazard census at `d354d02`: can a consumer infer a producer from entry shape, given … | Observation; the absolute counts are exact but every PERCENTAGE below is provisional and biased low |
| [RAR-S26](experiments/RAR-S26.md) | Phase-4.3a arm sizing at `8acfd22`: four registered knobs A–D, one `tune` binary, 4 positions at fixed depth … | Diagnostic, not a verdict |
| [RAR-S27](experiments/RAR-S27.md) | Phase-4.3a **arm A**, `EvalPruneTtMinDepth=2` versus the seeded 0 | Rejected by the registered acceptance rule after a manual stop at 23,044 games |
| [RAR-S28](experiments/RAR-S28.md) | Owed 4.2 throughput check: does the typed-evidence refactor cost NPS? `47f3ac6` versus `1cf9c51`, **three … | Retained: no measurable throughput cost |
| [RAR-S29](experiments/RAR-S29.md) | Phase-4.3a **arm A**, `EvalPruneTtMinDepth=1` — denying depth-0 entries the right to refine the main-search … | Rejected at formal H0 |
| [RAR-S30](experiments/RAR-S30.md) | Phase-4.3a refinement shadow at `8822cf2`, sampled 1/1024 over `bench 13`, 1T diagnostic build, no behaviour … | Observation with important scope limits |
| [RAR-S31](experiments/RAR-S31.md) | Phase-4.3a **arm B**, `SingularTtDepthMargin=2` versus 3 | H1 reached on the tune binary |
| [RAR-S32](experiments/RAR-S32.md) | Build-transfer diagnostics for arm B on an idle 5950X: tune option and baked PGO fingerprints plus pooled NPS … | Both forms produced 6,100,099 nodes / EBF 2.437 |
| [RAR-S33](experiments/RAR-S33.md) | Phase-4.3c gate preparation and independent verification of the landed implementation | Implementation verified; the gate carries a measurable speed headwind |
| [RAR-S50](experiments/RAR-S50.md) | Phase-4.10a: rebuild the accumulated-bundle composition from MEASURED subsets, as RAR-S45 requires | Every recorded figure reproduces exactly, and the accumulation strategy fails anyway |
| [RAR-S51](experiments/RAR-S51.md) | NMP mate-clamp correctness repair: keep an unproven mate score from satisfying a null-move cutoff | Gate interrupted and formally unresolved |
| [RAR-S49](experiments/RAR-S49.md) | Phase-4.9e: size the carried-in retry PLAN 4.9 reserved — RAR-S27's surviving hypothesis that … | THE PREMISE IS FALSE: the tree is BIGGER, not smaller, so the hypothesis is void and no games are owed |
| [RAR-S48](experiments/RAR-S48.md) | Phase-4.9d: SIZE the in-check qsearch staging that 4.6c deferred here, before building it | Population real, payoff below this project's own measurement floor — NOT BUILT |
| [RAR-S47](experiments/RAR-S47.md) | Phase-4.7b: one `RootConfidence` snapshot per COMPLETED root iteration, consumed by time management and by … | Landed inert and bench-identical (6,502,902 / EBF 2.449 on normal, diag and tune); three of the model's own … |
| [RAR-S46](experiments/RAR-S46.md) | Phase-4.7a: cover the root abort/fallback path, which `bench` structurally cannot reach | Retained correctness infrastructure; no behaviour change |
| [RAR-S45](experiments/RAR-S45.md) | The two verifications owed after 4.6: (a) count the safe-versus-losing quiet-check population, since … | Both negative, and both changed a decision |
| [RAR-S44](experiments/RAR-S44.md) | Phase-4.6c: replace the flat `DIRECT_CHECK_BONUS = 32_000` with safe/losing check classes in quiet ordering … | Mixed, and one part is NOT verified |
| [RAR-S43](experiments/RAR-S43.md) | Phase-4.6b: derive LMP, futility and SEE pruning from the same PROSPECTIVE depth LMR will search the move at … | Diagnostic, and the largest cheap arm in Phase 4 |
| [RAR-S42](experiments/RAR-S42.md) | Phase-4.6a: resolve the documented late-evasion contradiction | Observation, and it resolved in favour of the CODE |
| [RAR-S41](experiments/RAR-S41.md) | Phase-4.5d: does any further correction CONTEXT carry usable signal? Exact residual buckets by halfmove clock … | Observation: no new context is justified, and for two different reasons |
| [RAR-S40](experiments/RAR-S40.md) | Phase-4.5c: is the correction-uncertainty term applied to an eval the correction is no longer part of? Exact … | Observation — a real mis-application, and a large one |
| [RAR-S39](experiments/RAR-S39.md) | Phase-4.5b: continuation correction extended from a single 1-ply `(piece, to)` slot to compact 2- and 4-ply … | Diagnostic |
| [RAR-S38](experiments/RAR-S38.md) | Phase-4.5: is a capture-caused correction residual actually noisier than a quiet-caused one? Exact per-class … | Observation, and it SUPPORTS the premise for the first time |
| [RAR-S37](experiments/RAR-S37.md) | Phase-4.4c: potential-singularity guard, tightenable NMP material floor, and the double-extension margin as a … | Diagnostic |
| [RAR-S36](experiments/RAR-S36.md) | Phase-4.4b guards landed inert and sized: NMP cut-node guard, NMP decisive-window guard, NMP … | Diagnostic, no strength claim |
| [RAR-S35](experiments/RAR-S35.md) | Phase-4.4a switch sizing: five mechanisms landed inert, then each measured alone on `bench 13` (deterministic … | Diagnostic, no strength claim |
| [RAR-S34](experiments/RAR-S34.md) | Phase-4.3c **gate result and cost attribution.** Gate: candidate `1dc4bc6` versus baseline `d00e1ac` … | Gate: not promoted — dead neutral |

## 4. Root search, time management and SMP

| ID | Experiment | Disposition |
|---|---|---|
| [RAR-R01](experiments/RAR-R01.md) | Early Stockfish-style clock management on the old harness | Accepted, reported +81 Elo |
| [RAR-R02](experiments/RAR-R02.md) | Clock safety reserved `2*MoveOverhead` | Retained |
| [RAR-R03](experiments/RAR-R03.md) | Five-change Lazy-SMP/root-result bundle versus the original 4T implementation | Accepted, +102.78 ± 16.38 at 4T |
| [RAR-R04](experiments/RAR-R04.md) | Symmetric early stop vote at 2T | Rejected, −15.85 Elo |
| [RAR-R05](experiments/RAR-R05.md) | Pool-view instability TM | Rejected, −5.54 Elo |
| [RAR-R06](experiments/RAR-R06.md) | Helper-history blending and additional ordering jitter | Neutral/rejected |
| [RAR-R07](experiments/RAR-R07.md) | Phase-4.9 opening profile of accepted semantics at 1/2/4/8/16T on an idle 5950X | Throughput scales almost perfectly and DEPTH DOES NOT MOVE |
| [RAR-R08](experiments/RAR-R08.md) | Phase-4.9b: measure the cutoff-USABLE share of TT hits against thread count, the lead RAR-R07 left | The hypothesis is REFUTED — TT hits get BETTER with threads, and depth still does not move |
| [RAR-R09](experiments/RAR-R09.md) | Phase-4.9c: implement per-thread ITERATION STAGGERING — the depth-diversity mechanism RAR-R08 showed is … | Landed inert and correct; LOCAL SIZING IS IMPOSSIBLE, and a null pair proves it rather than asserting it |
| [RAR-R10](experiments/RAR-R10.md) | Phase-4.9c-i: the powered sizing RAR-R09 said was owed | The harness now works and the answer is NO BENEFIT WHERE IT COUNTS |
| [RAR-R11](experiments/RAR-R11.md) | A.3.3 time-forfeit repair at `3+0.03` - RUN 2026-09-09/10, forfeit rate UNDECIDABLE (0 against 0), Elo bound … | Prediction, frozen 2026-09-09 |
| [RAR-R12](experiments/RAR-R12.md) | A.3.3 harness reserve: `Move Overhead` 40 against 10 on the same binary - RUN 2026-09-10, REJECTED at -80.85 … | Prediction, frozen 2026-09-09 |
| [RAR-R13](experiments/RAR-R13.md) | D.1.1: the head's hard maximum in won endings; mechanisms of rec1–rec3 and their frequency in games — NO_CHANGE 2026-10-06 | NO_CHANGE: rec1 a fail-high cascade, rec2 no stop after a proved mate, rec3 the soft stop one iteration late; every won-position maximum move in 5,400 games was in a win bar one ordinary low-clock draw (≤ 0.5 Elo); the fix is owned by D.1.2 |

## 5. Evaluation and data experiments

The historical HCE freeze ended with the 2026-08-30 plan reconciliation.
Current Phase-4 steps 4.7–4.10 qualify the data and every fitting instrument,
refit the complete existing surface, then add structure only where residuals
support it. These rows remain relevant to that analysis and to NNUE data,
teacher and measurement design, but they do not authorize retries unchanged or
make any historical parameter group exempt from the current audit and gate.

| ID | Experiment | Disposition |
|---|---|---|
| [RAR-M47](experiments/RAR-M47.md) | Comparative claim SUPERSEDED by RAR-M49, 2026-09-13: the 57/12-against-40/12 surplus over Basilisk is not … | FIDELITY PROVEN TWICE BEFORE THE NEW NUMBERS WERE TRUSTED |
| [RAR-M48](experiments/RAR-M48.md) | A.8.4 pooled-PGO NPS baseline of the 2.4.0 release head, 2026-09-11 | Baseline: pooled median 3,189,100 n/s (pool A) and 3,190,438 (pool B) - about 3.19 M n/s. Instrument … |
| [RAR-M49](experiments/RAR-M49.md) | A.5 conversion meter re-read on the 2.4.0 release binary, 2026-09-13, zero games | Rarog 2.4.0: 88 draws and 19 losses after a persistent piece-up in 3,600 games - 24.4 and 5.3 per 1,000 games … |
| [RAR-M50](experiments/RAR-M50.md) | B.0 search-programme measurements on the 2.4.0 head, 2026-09-13, zero games | Tree shape |
| [RAR-M51](experiments/RAR-M51.md) | B.2.0 architecture-review measurements on the B.1 head `a8b6640`, 2026-09-14, zero games, no source changed | 36 files / 21,916 lines in `src/` |
| [RAR-M52](experiments/RAR-M52.md) | B.2.0.1 repository and document census on `4e60e45`, 2026-09-14, zero games, nothing changed | 273 tracked files |
| [RAR-M53](experiments/RAR-M53.md) | B.2.0.1 repository and document restructure, COMPLETE 2026-09-14, documents only | PLAN 1,508 → 857 |
| [RAR-M54](experiments/RAR-M54.md) | Super Rating Tournament read, 2026-09-15: 42 engines, 172,200 games, zero games by this session | Rarog 2.4.0: 4636-1360-2204, 64.8%, 12th of 42 at Colosseum's 3001 (−23 from its prior); Rarog 2.3.2 … |
| [RAR-M55](experiments/RAR-M55.md) | Rybka 4.1 benchmark of the B.2.3 fit — REGISTERED 2026-09-19, before any counted game; an observation, never … | Played 2026-09-19 |
| [RAR-M56](experiments/RAR-M56.md) | The unfitted B.2 head against Rybka 4.1 — the control for RAR-M55 (PLAN B.2.3.2). Registered 2026-09-19 while … | Played 2026-09-19 |
| [RAR-M57](experiments/RAR-M57.md) | Pool gauntlet of Rarog 2.5.0-dev (the fit at 3,900) — maintainer-run 2026-09-19, recorded after the event … | Rating 3191 |
| [RAR-M58](experiments/RAR-M58.md) | Rarog 2.5.0-dev against Critter 1.6a — maintainer-run 2026-09-19, recorded after the event; not … | −5.0 Elo |
| [RAR-M59](experiments/RAR-M59.md) | First-search stalls: the Colosseum confirmation — REGISTERED 2026-09-19, before any game (PLAN B.2.8) | Not run, and it will not be |
| [RAR-M63](experiments/RAR-M63.md) | Pool gauntlet of Rarog 2.5.0-dev with cluster 2 (the B.3.4 head) — maintainer-run 2026-09-27/28, recorded … | Rating 3233 |
| [RAR-M64](experiments/RAR-M64.md) | B.9 pool gauntlet of the search head at 1T — REGISTERED 2026-10-03, before any game; PLAYED 2026-10-03 … | Played 2026-10-03 18:34:47–19:02:43 UTC |
| [RAR-M65](experiments/RAR-M65.md) | B.10 release read at the longer control: the B.9 head against the 2.4.0 release at `10+0.1` — REGISTERED … | Played 2026-10-03 21:25–21:59 UTC |
| [RAR-M66](experiments/RAR-M66.md) | B.10 release read at four threads: the B.9 head against the four E.2 targets, Basilisk 1.10.0 and Rarog 2.4.0 … | Played 2026-10-03 22:00–2026-10-04 00:10 UTC |
| [RAR-E01](experiments/RAR-E01.md) | Staged Texel fit over 2.19M self-play positions: king safety, threats, mobility, scalars, imbalance … | Every stage accepted |
| [RAR-E02](experiments/RAR-E02.md) | Lazy HCE shortcut after the evaluator expansion | Accepted, about +4.4 Elo |
| [RAR-E03](experiments/RAR-E03.md) | Stockfish-at-60k off-policy distillation with material scale pinned | Rejected, −17.11 Elo |
| [RAR-E04](experiments/RAR-E04.md) | 500k-game on-policy refresh yielding 2.18M unique positions | Rejected, −1.28 ± 2.79 over 26.8k games |
| [RAR-E05](experiments/RAR-E05.md) | Narrow L2-anchored refresh from a stronger label generator, moving 57/1,204 parameters mostly by 1 cp | Accepted, +11.56 ± 5.19 Elo |
| [RAR-E06](experiments/RAR-E06.md) | Complete 1,218-slot current-HCE WDL refit, including traced linear coefficients and nonlinear king danger … | ACCEPTED |
| [RAR-E07](experiments/RAR-E07.md) | 4.8a redundancy inventory on the accepted vector | Closed without a gate; nothing to remove |
| [RAR-E08](experiments/RAR-E08.md) | Self-play labels versus tablebase-corrected labels on <=6-man positions. ACCEPTED | Registered; screen complete |
| [RAR-E09](experiments/RAR-E09.md) | 4.9.1 post-fit residual audit of the accepted HCE | Closed: no 4.9 entry evidence found, and a label defect found instead |
| [RAR-E10](experiments/RAR-E10.md) | 4.9a.4 minor-piece mate drive. ACCEPTED 2026-09-01 on maintainer judgement, with NO game gate | KBN-K 19.4% -> 96.9%, KBB-K 78.0% -> 100.0% |
| [RAR-E11](experiments/RAR-E11.md) | SUPERSEDED IN FULL by RAR-E14/RAR-M24 | Stockfish does NOT convert everything: 90.2% weighted, not 100% |
| [RAR-E12](experiments/RAR-E12.md) | Complete HCE refit on `hce-v3-tb` | H1 ACCEPTED at 7,388 games: +11.81 +/- 5.33 Elo, +17.57 +/- 7.92 nElo |
| [RAR-E13](experiments/RAR-E13.md) | RAR-E13 — is the fitted king-safety table worth its tree cost? (registered 2026-09-03, before games) | Recorded in full in the packet |
| [RAR-E14](experiments/RAR-E14.md) | Audit of the endgame truth instrument, 2026-09-04, prompted by Basilisk BAS-E47/BAS-E50 and verified … | Three confirmed defects |
| [RAR-E17](experiments/RAR-E17.md) | C.0 donor-direction residual screen: what the classical Stockfish evaluation predicts that Rarog's does not — REGISTERED 2026-10-05, before the corpus was scored; RUN 2026-10-05 | Observation: Stockfish's total adds +3.59% held-out, its families +1.22%; king +0.66% and winnability +0.12% carry signal, mobility, pieces, material and space none |
| [RAR-E18](experiments/RAR-E18.md) | C.0 follow-up cuts of RAR-E17's rows: where the donor's total-level information sits — REGISTERED 2026-10-05, before the cuts were computed; RUN 2026-10-05 | Observation: at seven men or more the total adds +2.73% (magnitude 0.46, families 1.62); at six or fewer +29.7%, half of it magnitude |
| [RAR-E20](experiments/RAR-E20.md) | C.0.3: the full evaluation against the lazy one at equal nodes, per-node quality apart from tree and speed — REGISTERED 2026-10-06, before any game; PLAYED 2026-10-06 | Observation: −110.0 ± 11.5 Elo at equal nodes, equal to the equal-time loss; the whole cost is per node |
| [RAR-E19](experiments/RAR-E19.md) | C.0.3 lazy path: the played evaluation above `LazyMargin` against the full evaluation, one 2,000-game read with a harm rule — REGISTERED 2026-10-05, before any game; RUN 2026-10-05 | Harm: `LazyMargin` 2000 −104.5 ± 10.6 Elo (nElo −159.1 ± 15.2, 2,000 games); the shortcut stays at 600, C.0.3 back to research |
| [RAR-E21](experiments/RAR-E21.md) | C.0.4 king-safety sub-term attribution, Rarog's inactive danger inputs, and the donor's scale factor as a direction — REGISTERED 2026-10-06, before any sub-term was scored | (pending) |

## 6. Throughput, build and platforms

| ID | Experiment | Disposition |
|---|---|---|
| [RAR-P35](experiments/RAR-P35.md) | B.9 pooled NPS, the search head against the 2.4.0 release — REGISTERED 2026-10-03, before any read; READ … | Read 2026-10-03 19:56–20:00 local |
| [RAR-P34](experiments/RAR-P34.md) | B.8 cleanup, no-regression NPS read — REGISTERED 2026-10-03, before the pool is built; PASSED 2026-10-03 | Step 1, 2026-10-03 10:19–10:25 |
| [RAR-P33](experiments/RAR-P33.md) | B.7.3 SEE recapturer, accepted or rejected on NPS alone — REGISTERED 2026-10-03, before its pool is built … | Step 1, 2026-10-03 07:50–07:56 |
| [RAR-P32](experiments/RAR-P32.md) | The pooled-PGO NPS read, shortened — study COMPLETE 2026-10-02 | On the three quiet cycles: noise between readings 0.27% for best of 3 (0.34% best of 2, 0.65% one run, 1.32% … |
| [RAR-P31](experiments/RAR-P31.md) | B.7.2.14–B.7.2.20, acceptance reads of the audit's survivors — REGISTERED 2026-10-02, before any acceptance … | A (`a42fadc`), 2026-10-02 16:53–17:36, CPU 2.0% before and 4.6% after: +1.69% (95% CI +1.56% .. +1.89%) … |
| [RAR-P30](experiments/RAR-P30.md) | B.7.2.13 audit screen, four further exact candidates — REGISTERED 2026-10-02, before any pooled read; SCREEN … | Screen, 2026-10-02 08:39–12:30 |
| [RAR-P29](experiments/RAR-P29.md) | B.7.2.9 candidate 3 falsifier, the selection scan's local speedup, COMPLETE 2026-10-02 | All variants select the same entry on every tail |
| [RAR-P28](experiments/RAR-P28.md) | B.7.2 candidates 1 and 2, pooled-PGO NPS, and the re-profile, COMPLETE 2026-10-02 | Candidate 1 +8.85% |
| [RAR-P27](experiments/RAR-P27.md) | B.7.2 whole-search profile, COMPLETE 2026-10-01 | 256,177 samples, all engine samples resolved |
| [RAR-P26](experiments/RAR-P26.md) | B.2.0.2 MultiPV, behaviour-neutral at the default, COMPLETE 2026-09-15 | Fingerprints exact: 7,601,220 / EBF 2.474 and b2core 4,706,910 / EBF 2.391, 40/40 on magic and PEXT |
| [RAR-P25](experiments/RAR-P25.md) | B.2.0 architecture upgrades, behaviour-neutral, COMPLETE 2026-09-14 | Fingerprint exact at every engine commit: 7,601,220 / EBF 2.474, 40/40 positions identical |
| [RAR-P24](experiments/RAR-P24.md) | B.1 search restructure, behaviour-neutral, COMPLETE 2026-09-14 | Fingerprint exact at every commit: 7,601,220 / EBF 2.474, 40/40 positions identical |
| [RAR-P23](experiments/RAR-P23.md) | Windows ABI comparison for the shipped `pext` configuration - `msvc` versus `gnullvm` versus `gnu`. Ad-hoc … | NO MEANINGFUL DIFFERENCE; `msvc` retained, nothing changed |
| [RAR-P22](experiments/RAR-P22.md) | A.4.3 direct `pext` versus `base` NPS - REGISTERED 2026-09-10, prediction frozen while the run was in flight … | Prediction, frozen 2026-09-10 before exposure |
| [RAR-P21](experiments/RAR-P21.md) | A.4.1 `cc` 1.3.0 -> 1.4.5 and the Fathom build contract, COMPLETE 2026-09-10 (`1bf8171`) | GREEN on every checked surface, and one prior claim of mine is corrected below |
| [RAR-P20](experiments/RAR-P20.md) | A.4.1 per-tier NPS: what does the ISA tier actually buy? REGISTERED 2026-09-10, NOT YET RUN | Prediction, frozen 2026-09-10 before any run |
| [RAR-P19](experiments/RAR-P19.md) | A.9 (registered as A.3.4 before the 2026-09-10 Phase A reorder) ARM64 compatibility re-verification of the … | ALL GREEN; the `rust-lld` workaround still works on 1.98.1 and all three platforms agree at this head |
| [RAR-P18](experiments/RAR-P18.md) | A.3.1 toolchain bump 1.97.1 -> 1.98.1, behaviour-neutral qualification, COMPLETE 2026-09-09 (`ca8988a`) | NEUTRAL; no behaviour change and no resolvable speed change |
| [RAR-P01](experiments/RAR-P01.md) | Phase-9 clean-code/build program, each step bench-identical and spot-checked | End-to-end result was about −3.2% NPS, inferred around −2 to −3 Elo |
| [RAR-P02](experiments/RAR-P02.md) | Phase-10.3 bench-identical hot-path wave with two PGO builds/arm | Accepted, +10.35% NPS and +20.31 ± 7.13 Elo at `3+0.03` |
| [RAR-P03](experiments/RAR-P03.md) | Post-SMP duplicate-compute/index-hoist cleanup | Retained, +0.99% then +1.56% median NPS |
| [RAR-P04](experiments/RAR-P04.md) | Board-layer perft comparison with Basilisk | Rarog's board layer was not the main source of the remaining search-strength gap |
| [RAR-P05](experiments/RAR-P05.md) | Pawn-cache enlargement from the profile audit | A 128× larger table gained about 1.1 hit-rate points but lost 4.5% NPS |
| [RAR-P06](experiments/RAR-P06.md) | `origin/arm_fix` added AArch64 `PRFM PLDL1KEEP` and hoisted two HCE `LazyLock` accesses | Unverified when written; now CLOSED — both halves have had their target-native A/B |
| [RAR-P07](experiments/RAR-P07.md) | `origin/arm_fix` wrapped TT clusters in 128-byte Apple-oriented blocks | Unverified when written; now CLOSED and rejected — see RAR-P16 |
| [RAR-P08](experiments/RAR-P08.md) | Windows ARM64 PGO with pinned Rust used `rust-lld` to work around profile-link failure | Retained in 2.3.1 |
| [RAR-P16](experiments/RAR-P16.md) | Finish the two outstanding `origin/arm_fix` changes on current dev and measure them on Apple Silicon | NEITHER CHANGE IS MEASURABLE; `3ee4660` stays REJECTED, third time |
| [RAR-P17](experiments/RAR-P17.md) | Phase-4 step 4.5.1 — typed per-ply search context, pooled-PGO NPS | Behaviour-neutral and NPS-neutral — a clean null |
| [RAR-P15](experiments/RAR-P15.md) | Phase-4.8h: first full CI matrix dispatch carrying the 4.8 work — the `verify-isa` steps added in 4.8a and … | GREEN, 14/14 jobs, 4m 0s |
| [RAR-P14](experiments/RAR-P14.md) | Phase-4.8g: retest the Windows ARM64 PGO path on the pinned toolchain | PASSES — the last release-blocking unknown in 4.8 is cleared |
| [RAR-P13](experiments/RAR-P13.md) | Phase-4.8f: identical-binary calibration on macOS ARM64 — the null pair PLAN item 5 requires, run AFTER the … | NO ORDERING ARTIFACT; the +1.42% survives |
| [RAR-P12](experiments/RAR-P12.md) | Phase-4.8e: does the Apple 128-byte cache line cost anything? `SharedCluster` is `align(64)`/64 B, so on … | NO MATERIAL FALSE SHARING — question CLOSED with no code change |
| [RAR-P11](experiments/RAR-P11.md) | Phase-4.8c: the ARM64 verdict run PLAN 4.8 item 3 reserves, plus the Apple topology probe item 4 requires | PREFETCH ACCEPTED. +1.42% NPS |
| [RAR-P10](experiments/RAR-P10.md) | Phase-4.8b: port the AArch64 TT prefetch from `origin/arm_fix` onto current development and make its presence … | ACCEPTED on measurement: +1.42% NPS on an M4, and a silent three-release loss closed |
| [RAR-P09](experiments/RAR-P09.md) | Phase-4.8a: freeze the per-tier ISA contract and make it EXECUTE | Two shipped defects found, both invisible to every existing gate |

## 7. Correctness and protocol lessons

| ID | Experiment | Disposition |
|---|---|---|
| [RAR-C01](experiments/RAR-C01.md) | Self-consistency tests compared implementations sharing the same rule-50, repetition and SEE omissions | Independent legal-exchange and external perft oracles added/required |
| [RAR-C02](experiments/RAR-C02.md) | Rule-50 draw could override mate | Free mate-precedence fix retained |
| [RAR-C03](experiments/RAR-C03.md) | Multi-thread diagnostics reset/dumped inside helper-called root search and the diagnostic build had stopped … | Fixed |
| [RAR-C04](experiments/RAR-C04.md) | Aspiration mate-score re-search could fail to terminate | Fixed and regression-covered |

## 8. Cross-engine evidence imported from Basilisk

These are ideas, warnings or ordering priors already incorporated where useful
in Rarog's forward plan. No additional roadmap item is created merely by
listing them here.

| ID | Experiment | Disposition |
|---|---|---|
| [RAR-X01](experiments/RAR-X01.md) | TT-bound-aware pruning evaluation gained +7.18 Elo while preserving raw/corrected eval for learning | Strong prior for typed result evidence and producer/consumer capability separation, not for copying its TT … |
| [RAR-X02](experiments/RAR-X02.md) | Check-extension removal lost −10.17 ± 6.52 in Basilisk while Rarog's extension had gained +30.75 | Confirms that extensions and their LMR/pruning consumers co-adapt |
| [RAR-X03](experiments/RAR-X03.md) | Stockfish distillation gained +6.75 in Basilisk but lost −17.11 in Rarog | Teacher/corpus/representation fit dominates transfer |
| [RAR-X04](experiments/RAR-X04.md) | A 6-ply continuation-history channel lost −7.70 in Basilisk | Wider history distance can duplicate existing signals |
| [RAR-X05](experiments/RAR-X05.md) | Exact/PV reward-only history and surprise scaling jointly reverified at +3.06 ± 4.35 | Result kind and confidence can be useful training inputs when sibling maluses are not misapplied |
| [RAR-X06](experiments/RAR-X06.md) | Root instability TM reverified at +6.46 ± 4.12, while Rarog's raw pool-view variant lost −5.54 | Instability may help only when derived from a completed authoritative root snapshot and bounded with other … |
| [RAR-X07](experiments/RAR-X07.md) | Basilisk's +4.34% NPS wave measured +8.69 ± 6.63 Elo at STC | Speed-to-Elo direction is corroborated near this TC, but individual hot-path optimizations are profile- and … |
| [RAR-X08](experiments/RAR-X08.md) | Basilisk's `arm_fix` independently tried unmeasured TT over-alignment | Corroborates the need to measure topology and audit the executable asset contract, not the proposed wrapper |
| [RAR-X09](experiments/RAR-X09.md) | Basilisk's SMP safety bundle gained +30.42 ± 8.77 at 4T, smaller than Rarog's five-change +102.78 bundle | The different gains suggest different baseline defects |

The cross-review found no additional high-value Basilisk item missing from the
current Rarog plan. Items above are already covered, contradicted by local
evidence, or deliberately postponed to the NNUE/scaling phases.

## 9. Open retry map

| Prior IDs | Retry condition | PLAN destination |
|---|---|---|
| RAR-S11, RAR-S15, RAR-X01 | Phase-4 cluster B reaches these consumers, or NNUE scale freezes and they remain independently ablatable with material activation. | 4.6, else 7.3 |
| RAR-S13, RAR-S14, RAR-S19, RAR-X02 | Prospective depth or evidence architecture wins a categorical gate before its consumers are tuned, in Phase 4 or after NNUE. | 4.7–4.8, else 7.3 |
| RAR-S16, RAR-S38–S41, RAR-X04, RAR-X05 | Phase-4 cluster A reaches the correction/history contract, or NNUE residuals show a populated unique signal. Tune graded weights; never restore the rejected all-or-nothing capture guard. | 4.5, else 7.3 |
| RAR-S17, RAR-R04, RAR-R05, RAR-X06 | Phase-4 cluster E reaches root authority, or real-clock NNUE telemetry shows the completed root-confidence snapshot discriminates without moving total budget. The root gap stays excluded unless root searches produce comparable values. | 4.9, else 7.3 |
| RAR-R07–R10, RAR-X09 | Representative 4T/8T/16T hardware is available after NNUE; price depth diversity and retained SMP switches directly. | 8.0 |
| RAR-P01–RAR-P05, RAR-X07 | A new deployed profile identifies a material hotspot; use pooled same-target final-PGO A/B. | 8.0–8.1 |
| RAR-P06, RAR-P07, RAR-X08 | A new target-native profile identifies a topology/layout cost. The ARM prefetch itself is already accepted; do not retry rejected over-alignment from cache-line folklore. | 8.0–8.1 |
| RAR-S27, RAR-S29, RAR-S49 | **Closed for the flat TT-refinement depth-floor shape.** Reopen only through a materially different evidence model — a Phase-4 cluster-B contract or a post-NNUE fit — never the removed UCI coordinate. | 4.6, else 7.3 |
| RAR-S31 | Re-evaluate `SingularTtDepthMargin` inside Phase-4 cluster D or after NNUE, in final PGO only. The historical tune-binary H1 (+3.35 ± 2.44 Elo) did not meet the later material/final-PGO policy. | 4.8, else 7.3 |
| RAR-E03, RAR-E04, RAR-X03 | NNUE data/teacher experiment with changed representation and a frozen external holdout — not another HCE refit. The Phase-4 HCE track may study evaluator contracts but does not retry this distillation. | 4.12–4.16 for contracts; 5.0–7.2 for teacher/data |
| RAR-R13 | A game record shows the head failing to win, or forfeiting, after a won position, the loss preceded by a deep stall (≥ 0.9 × maximum at depth 40 or more, after a mate shown, or on a root lower bound); or a CCRL-blitz-or-longer record shows such moves costing results; or D.1.2 changes the bound model (rerun `tm_games.py`). | D.1.2 |

Anything not meeting its trigger stays closed. A retry is a new experiment with
a new ID and manifest; it does not overwrite the historical row.

## 10. Template for a new experiment

Use the research packet in `docs/PROCESS.md`. Register the experiment before
any games as `docs/experiments/<ID>.md`, in PROCESS's *Experiment
registration* form, and add its index row (ID linked to the file, short
title, disposition) to the section that owns it; update both when the result
lands. Anything longer than an entry goes in an `analysis/` packet the entry
cites.
