# Rarog development plan

This is the forward roadmap. It says what will be done, in what order, why,
and what decides each step. It does not record history: completed work lives
in [HISTORY.md](HISTORY.md), measured evidence in
[EXPERIMENTS.md](EXPERIMENTS.md), procedures in [PROCESS.md](PROCESS.md), and
the day-to-day status board with checkboxes in [GUIDE.md](../GUIDE.md). The
pre-rewrite roadmap is archived verbatim at
[docs/archive/PLAN-phase4-2026-09-09.md](archive/PLAN-phase4-2026-09-09.md);
every historical `4.x` reference in the ledger and analyses points there.

Rewritten 2026-09-09 as a battle plan with one objective and a measured
starting point. Phases are lettered (A–G) so that no new identifier collides
with any retired number cited in the ledger, the analyses or source comments.

## 1. Objective and gates

**Objective.** Make Rarog the strongest engine we can build, in two stages:

1. **Classical stage.** With the hand-crafted evaluation, beat the strongest
   HCE-era engines in the maintainer's own pool: **Critter 1.6a, Houdini 3,
   Rybka 4.1 and Fritz 16**. Rybka 4.1 replaced Rybka 4 by maintainer
   decision 2026-09-19: the same engine with bug fixes. The 2026-09-11
   baselines below measured Rybka 4; against Rybka 4.1, 2.4.0 scored −131 in
   the Super Rating Tournament (RAR-M54).
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
| Search deficit with Rarog's own evaluation | **−247.97 ± 10.89 Elo** at equal time on the 2.4.0 release head, evaluation proved constant, no adjudication (RAR-O03). Non-mate depth from the PGN 14.94 against 16.33 (the recorded 19.68 against 20.65 mis-attributed Black-to-move openings; corrected in RAR-O03, 2026-10-03). The superseded −250.8 was adjudicated and is not comparable | RAR-O03; `analysis/ablation_results.md` for the ablation |
| Where the search deficit lives | LMR plus shallow-depth pruning explain **272 ± 18** of it, near-additively; everything else about 30 | matched ablation, mask 160 |
| Evaluation deficit with the same search | Stockfish's classical HCE beats Rarog's frozen evaluation inside Stockfish's search by **+99.2 ± 16.6 Elo at equal nodes** (RAR-O07, 2026-10-21), from +181.7 ± 19.0 at Phase C's start (RAR-O05) | RAR-O05, RAR-O07 |
| Speed | **3.19 MNPS pooled median** at bench 13, PGO pext 1T, ±0.2% instrument resolution (best-of 3.21, which is the 3.22 previously recorded); Basilisk 3.71; board work 24% of time, evaluation 29%, search loop 23% | RAR-M48; RAR-M36, RAR-M44 |
| Conversion | **88 draws and 19 losses** after holding a piece-up advantage for 12+ plies, in 3,600 games against the six HCE-era engines on the **2.4.0 release** games — 24.4 and 5.3 per 1,000, unchanged from the 2026-09-04 pool's 57/12 in 2,400 (23.8 and 5.0). Basilisk 1.9.3 in the same tournament: 94 and 12. **RAR-M47's surplus-over-Basilisk reading is not reproduced and is retired**; the stable finding is Rarog's own rate, 80 of the 88 draws by fifty-move or repetition with material in hand; a third independent sample reads 24.2 and 3.3 per 1,000 (RAR-M54, 1,200 games against the same six) | RAR-M49 (release re-read, tournament `5e539523`); RAR-M54 (Super Rating Tournament, 42 engines); instrument RAR-M47 |
| Fingerprint | `bench 13` **13,987,223 / EBF 2.565** at engine source `9fb525e` (C.10's search tune on the frozen evaluation; `rustc 1.99.1`) | RAR-S105, RAR-P36 |

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
| B search | 251 equal-time (247.97, RAR-O03) | 120–200 | selectivity explains 272; a Reckless-shaped stack fitted locally. **Measured at B.9: 272 ± 14 recovered**, G(0) −247.97 → +24.24 ± 8.01 (RAR-O04): above the band, the head beats the oracle |
| C evaluation | 182 same-search at equal nodes, 266 at equal time (RAR-O05; RAR-O02's 329 was the 2.3.2 evaluation); 129 at equal nodes after C.3 (RAR-O06); **99 at C.11 (RAR-O07)** | 100–160 | six families, whole-surface refits, endgame conversion. **C.0 (2026-10-05):** C.0's screen supports two family clusters (king safety, winnability and scale), not six, and its prior for the family clusters together is 30–90 (`analysis/eval_programme_2026-10-05.md`). **C.3.6 (2026-10-09):** C.2's refit and C.3's king unit closed 52 ± 26 of the 182 in the same search (RAR-O06); one reading does not split them, and the refit is not a family cluster. The band is corrected at C.11 by RAR-O07's measurement (registered 2026-10-11), never by an estimate. **Measured at C.11 (RAR-O07, 2026-10-21): 82.5 ± 25.2 recovered at equal nodes**, +181.7 → +99.2 ± 16.6: below the planned 100–160 and in the upper half of C.0's 30–90 prior for the family clusters; Rarog's own strength gain over Phase C is read by C.11's pool gauntlet and E.1, not by this meter |
| D clock, SMP, robustness | unmeasured | 15–40 | Reckless-shaped node-fraction TM; 4T quality |
| Speed inside B and C | — | 10–30 | per-node cost of the new search and evaluation modules. **B.9:** the B head is 26.2% slower than 2.4.0 in pooled bench NPS (RAR-P35), after B.7 won back about 20% inside B (+35.5 Elo, RAR-S98); B's gates already include that cost and that recovery, so this row recorded nothing of B's separately and now applies to C. **C.12** owns the recovery (added 2026-10-13) |

**Corrected at B.9:** the search programme alone recovered more than its
band, and the B.9 head scores at least 50% against all four E.2 targets at 1T
(RAR-M64: Houdini 3 55.4%, rating 3,286.5 on the held scale); 4T is not
measured. The C and D bands stand until their checkpoints. As first written:
if those bands are right the classical head lands within reach of Rybka 4.1
and Fritz 16 and near Critter; Houdini 3 may only fall in the NNUE stage. Each
programme's checkpoint re-measures its deficit meter so a miss is seen as a
miss and the budget above is corrected rather than defended.

## 2. Operating rules

`AGENTS.md` is authoritative for measurement, verification, documents and
gating. The rules below decide order and acceptance in this roadmap.

1. **Donor architecture, own implementation.** Which engine donates what,
   what may cross and how the code is written: `docs/PROCESS.md`, *The
   independence boundary*. Similarity to a donor is never an acceptance
   criterion; games are.
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
   record.** Registration in `docs/EXPERIMENTS.md` before any games: hypothesis,
   baseline SHA, bracket, cap, stop rule and the frozen prediction. Bounds
   default to `[0,3]` nElo; a large prior uses `[0,10]` or `[3,10]` and says
   why; a removal uses a loss-permitting bracket, and a repair the gate of
   its case in AGENTS' *Gating*. Never change a gate after seeing games.
6. **Two rejected clusters in one programme stop it** and force a new evidence
   audit before a third is built.
7. **Every strength A/B runs with adjudication off**, at `3+0.03`, 1T, Hash 64,
   paired UHO, `fastchess -use-affinity`, concurrency 14, unless the
   registration states otherwise and why. Multi-thread gates drop affinity and
   calibrate a null pair first.
7b. **A cluster may be fitted before its gate, with an argument.** B.2 read
   +65.09 ± 23.26 Elo unfitted and +138.60 ± 30.66 fitted (RAR-S73), so donor
   constants can understate a mechanism by more than its whole margin and a
   good cluster can fail its gate for want of a fit. That is a reason to fit
   *some* clusters first, never a routine step: SPSA costs tens of hours
   (RAR-S75 took 62 active hours for 160,000 games), so a pre-gate tune is
   registered case by case with the argument for why this cluster needs it,
   what horizon, and what the gate would otherwise measure. Maintainer
   decision 2026-09-20. The whole-surface refit stays at B.6.

7c. **A tune runs in blocks with a movement stop rule, not on one horizon.**
   RAR-S75 ran 5,000 iterations and its last 1,100 were worth +4.43 ± 2.90;
   restarting the same 82 coordinates from its endpoint with a fresh gain
   schedule (RAR-S78, another 5,000) gained +13.1 ± 5.4, carried by
   coordinates the first run had barely moved. The limit was the decaying
   gain, not the games: a coordinate whose gradient appears only after
   others have moved receives it when the schedule is spent, and the right
   horizon cannot be predicted because it depends on how far the seeds are
   from the optimum. So a registered tune is a sequence of blocks, each
   2,000 iterations × 30 games on Colosseum (60,000 games, about 12.5 h),
   each started from the previous block's rounded centres with a fresh
   schedule, the same surface and the same steps. After each block, count
   the coordinates that moved at least one step from the block's seeds: at
   least three, and the next block runs; fewer, and the tune stops with the
   last block's rounded centres as theta; never more than three blocks
   without a new registration. Block size, count and ceiling are registered
   before the first game and never change after it. Nothing is baked
   between blocks; a later block starts through `colosseum.ps1 -SeedFrom`
   from the previous block's run directory, on the same config group and
   tune file, and the wrapper refuses an unfinished source, another binary
   or another surface. Maintainer decision 2026-09-25. **Amended 2026-09-26:**
   the count is a device for saving unattended compute, never a reason to
   end a tune the maintainer is watching: RAR-S82's block 1 ended with one
   coordinate two steps out and still travelling and eleven at half a step,
   and the count said stop. A chained launch may run every registered block
   unconditionally; theta stays the last completed block, and a block cut
   short never counts. A tune is judged by its gate, not by its movement.
   **Amended 2026-10-20 (maintainer decision, C.10.4):** the chain of
   2,000-iteration blocks is the shape for seeds far from their optimum.
   Where a pilot and an earlier block read the surface as near its optimum
   with small gradients, a tune may instead be registered as one longer run
   with a larger perturbation and a smaller gain ratio (RAR-S104: 4,000
   iterations, perturbations twice the b43 step, `r-end` 0.0016 in a
   committed `schedule_<group>.json`), because the signal-to-noise of each
   gradient estimate grows with the perturbation and a restarted schedule
   only re-inflates the random walk near an optimum; the registration
   derives the resolution it buys. The b43 step stays the unit of "a full
   step" in every reading (`pilot_reaudit.py --movement-config`), and a run
   cut short is still not theta.

8. **State the measurement layer.** Theory truth, move quality, conversion,
   fixed-node tree shape, NPS and game strength are different units with no
   exchange rate. Counters, node counts, EBF, tactical suites and fit loss
   explain or screen; only a registered final-PGO SPRT or the target gate
   accepts. **Cluster screens** (from B.2.2's review, 2026-09-15; every
   cluster from B.3 on registers its numbers against this ladder before
   implementation):
   - **The paired run governs.** The unfitted 2,000-game paired run against
     the last accepted head is the only screen that clears a cluster for its
     fit. Zero-game floors are diagnostics. A floor failure triggers the
     ablation sweep and a written cause in the cluster's analysis. It never
     holds a candidate the paired run cleared, and a floor passed never
     accepts one it failed.
   - **Ablation:** one sweep of the eight `AblationMask` bits in mechanism
     order (0 razoring, 1 reverse futility, 2 null move, 3 ProbCut, 4 IIR and
     hindsight, 5 move-loop pruning, 6 singular, 7 late-move reductions).
     Each bit is run alone on the fixed-node screens; there is no other order
     and no second sweep.
   - **Speed:** time-to-depth on `bench 13` replaces the pooled-NPS floor,
     because a tree-shape change makes nodes incomparable. Each arm's time is
     its bench nodes divided by its pooled-PGO median NPS from interleaved
     `nps_multibuild.ps1` runs. The candidate-to-baseline ratio is read
     against the floor the cluster registers, and pooled NPS is reported beside
     it as a diagnostic.
   - **Fixed-node quality:** WAC at 100k and 400k nodes, plus a positional
     screen at 100k nodes. That screen is the Strategic Test Suite once the
     maintainer places it as a tracked fixture `sts_v1.epd` under `tools/diag/`.
     Until then it is the phase-4 suite scored by agreement with the frozen
     oracle's best move at the same budget. Scoring convention for STS,
     the suite's own: each position's `c0` map gives the engine's move its
     listed points (0 if unlisted) against a maximum of 10, a position
     without a map scores 10 for a `bm` match and 0 otherwise, reported
     per theme as points, percentage and top-1 rate (as `sts_runner2.py`
     in RBp4wn (tissatussa, Go, CC0, `github.com/tissatussa/RBp4wn`, read 2026-09-17) does; the
     scoring is added to `fixed_budget_probe.py`, not a second runner).
   - **Canaries:** a regression rule. No oracle-anchored canary the baseline
     solves may be lost, where solving means stable at no more than the anchor
     plus two plies and solved at 100k nodes. New passes are recorded, not
     required.
   - **Curvature sweep before any SPSA:** a checklist item with its own
     evidence path, `analysis/<leaf>_sweep_<date>.md` with raw results under
     `tools/results/`. Five registered coordinates each run at 0.5x, 0.75x,
     1x, 1.5x and 2x, with `bench 13` and WAC at 100k per point. The
     classification is frozen before the first point, as in
     `analysis/b223_sweep_2026-09-15.md`. Flat or monotone on all five skips
     the cluster's SPSA.
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
| The classical evaluation is frozen (2026-10-22) at engine source `9fb525e`: it changes only by a registered repair or by Phase F's network | C.11 in `docs/archive/PLAN-closed-2026-10-22.md`; RAR-E73 |
| Per-node search code does not allocate | `tests/allocation_guard.rs`: whole searches at two depths, 1T and 4T, and the rule-50 boundary's mate test counted directly |
| Diagnostic counter units and sampling | `analysis/phase4_counter_spec.md` |
| Measurement layers for endgame work | `analysis/endgame_measurement_layers.md` |
| Texel data contract, splits, instrument coverage | `analysis/texel_fitting_handbook.md`, `analysis/hce_archive_audit_2026-08-31.md` |
| Behaviour-neutral change = exact fingerprint plus targeted checks plus pooled NPS | `AGENTS.md` |
| Cross-engine board benchmark parity | `benches/board.rs`, `analysis/board_comparison_411b19_2026-09-09.md` |

## Phase A — Reset: repository, instruments, baselines, consolidation release — CLOSED 2026-09-11

Documents, repository, toolchain (`rustc 1.98.1`) and CPU-tier assets were
reset, the conversion instrument built and the 2.4.0 baselines measured
(pool, 4T gauntlet, oracle deficit −248 Elo, 3.19 MNPS); 2.4.0 was released
at +54.8 ± 17.0 Elo over 2.3.2.

The phase's text is verbatim in `docs/archive/PLAN-closed-2026-10-05.md`; each leaf's
dated record is in `docs/HISTORY.md`.

## Phase B — Search programme (evaluation frozen) — CLOSED 2026-10-05

The search was rebuilt and fitted in gated clusters (selectivity core;
null-move, ProbCut and singular extensions; quiescence; tablebase root; a
speed pass and a cleanup), turning the oracle deficit from −248 into
+24 ± 8 Elo. It was released as 2.5.0, +272 Elo over 2.4.0 at `3+0.03`,
and is frozen at `ee02ed1` for Phase C.

The phase's text (B.0–B.10, with B.9's readings and attribution table
and B.10's release and ref review) is verbatim in
`docs/archive/PLAN-closed-2026-10-05.md`. The rule it left, the search freeze, is in Phase C.

## Phase C — Evaluation programme (search frozen) — CLOSED 2026-10-22

The classical evaluation was rebuilt on Stockfish 11's model in fitted
clusters (a refit, king safety, threats, pawns and passers, endgame rules,
two refit cycles), the engine made 29% faster in bench and its search
re-tuned on the new evaluation: 82 ± 25 of the 182-Elo equal-node
evaluation deficit closed, and +142 ± 25 Elo at 1T and +94 ± 26 at 4T over
the B.9 head against the pool. The evaluation is frozen at engine source
`9fb525e` (13,987,223 / EBF 2.565); the C head rates 3,479 in a wider
field, about 3,475 on CCRL's Blitz scale.

The phase's text (C.0–C.13: its rules, cluster shape, records and
readings) is verbatim in `docs/archive/PLAN-closed-2026-10-22.md`; the
readings are in the ledger rows each step names. The rule it left, the
frozen classical evaluation, is a standing contract.

### Active workflow register

One row per open leaf in the active phase. The checker requires the state
and class here to match GUIDE's suffix. Phases A, B and C are closed;
Phase D is the active phase since Phase C closed (2026-10-22), and
`check_guide.py`'s `ACTIVE_PREFIXES` names it. Later phases carry only a
class until they open. The rows are in the order of work, which may differ
from the numbering; the first open row is GUIDE's Now (`guide_board.py`).

| Leaf | Workflow state | Class | Current decision |
|---|---|---|---|
| D.1.2.1 | RESEARCH | R2 | Won-ending time stalls (RAR-R13): rec1's cascade, rec2's missing stop, rec3's overrun |
| D.1.2.2 | RESEARCH | R2 | The forfeit margin against OS holds |
| D.1.2.3 | RESEARCH | R2 | The audit and the bound model; one SPRT `[0,3]` for D.1.2 |
| D.1.2 | RESEARCH | R2 | Closes with D.1.2.3's SPRT, which gates the three addenda together |
| D.2 | RESEARCH | R2 | Lazy SMP at 4T/8T; RAR-M74's fallen thread scaling is new evidence for its premise |
| D.3 | RESEARCH | R2 | Engine lifecycle and protocol robustness |
| D.4 | RESEARCH | R2 | Tablebase policy |

## Phase D — Clock, threads, robustness

- **D.1 Time management: audit against the ADR-0065 checklist, soft/hard bounds with node-fraction multiplier, forfeit margin; SPRT `[0,3]` — `R2` investigation, `I2`/`V` sub-steps.** Audit the
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
  **Research input, measured 2026-10-01 by B.5.2**
  (`analysis/b52_research_2026-10-01.md`): in won endgames without
  tablebases, an aspiration cascade of fail-highs on a rising evaluation
  makes an iteration outlive the optimum, and the move ends only at the
  hard maximum. At `60000+600`, rec1 took 32,305 ms (maximum 32,304; the
  last `info` at 3,744 ms) and KRvK 11,251 ms (maximum 11,250, stalled at a
  downgraded mate value). Stockfish 19 took 5.6–6.4 s and about 1 s on the
  same positions. B.5.2 contains it at tablebase roots only; the general
  case is this leaf's.
    - **D.1.1 Time-management diagnosis, pulled forward: rec1's stalled re-search, rec2's search past a found mate, rec3's clock-independent stop; mechanism, frequency in games, the frozen fix and its gate — `R2`, NO_CHANGE 2026-10-06.**
      Pulled forward by maintainer decision 2026-10-06, ahead of C.2's
      baseline-refit gate and C.3's first gate (Phase C's freeze rule,
      its exception). Evidence: `analysis/basilisk_review_2026-10-06.md`, item 1.
      Without tablebases Rarog spends its hard maximum, 52–58% of the
      clock, on rec1 (a re-search stalled at `cp 1302 lowerbound`) and on
      rec2 (after finding `mate 5`, its iterations stall near depth 56) at
      both `60000+600` and `180000+2000`, where Stockfish 19 takes 9.4 and
      43.8 s on rec1 and 1.4 and 2.7 s on rec2; on rec3 it stops at 10.4 s
      and depth 21 at both clocks, so not by the clock. The maximum itself
      is the donor's and not the defect. Research first: the mechanism of
      each behaviour (is rec2 rec1's cascade or a missing stop after a
      proved mate; what ends rec3), how often each happens in game
      records, then the fix frozen with its prediction. Gate: a
      correctness repair takes AGENTS' repair case 2 (a registered
      2,000-game read with a harm rule); a change to the time formulas
      takes an SPRT `[0,3]`. The change touches only clock play (`bench
      13` unchanged); its accepted head becomes Phase C's frozen head.
      **Closed `NO_CHANGE` 2026-10-06 (RAR-R13;
      `analysis/d11_tm_diagnosis_2026-10-06.md`).** Untimed per-iteration
      traces give the three mechanisms. rec1 is a fail-high cascade at
      depth 32: nine re-searches centred on a stale `cp 534`, then a tenth
      window, [513, 1,691], that does not finish. rec2 is a proved mate
      (`mate 5` from depth 17) with no stop: there are no bound lines, so it
      is not the cascade, and depth 57 grows to the ply cap and does not
      finish. rec3 is the clock's soft stop one iteration late: both clocks'
      targets fall inside a depth-21 iteration that a six-step cascade
      stretched from 1.8 to 10.4 s. In 5,400 games of the head's search
      (RAR-M64, RAR-M65, RAR-E19), every move at ≥ 0.9 × the maximum in a
      won or mate position was in a game the head won, bar one `10+0.1`
      draw whose long move was an ordinary low-clock overrun. The upper
      bound on the effect is about 0.5 Elo, which no gate resolves. The
      repairs are either refuted territory (the aspiration loop: RAR-S89,
      RAR-S17; it also moves `bench 13`), D.1.2's bound model (a clock-only
      fail-high stop), or a time saving in won games only (a mate-proved
      stop). Nothing lands ahead of Phase C, and its frozen head is
      unchanged. The fix moves to D.1.2, which owns it, with these
      mechanisms and acceptance checks. RAR-R13's trigger pulls it forward
      again if a game record shows a cost.
    - **D.1.2 Time-management audit and bound model: the ADR-0065 checklist, soft and hard bounds with the node-fraction multiplier, the forfeit margin, the won-ending stalls (RAR-R13); SPRT `[0,3]` — `R2`.**
      The rest of D.1 as written above, after Phase C, starting from
      Phase C's head (D.1.1 changed nothing). **This step owns the fix of
      D.1.1's three behaviours** (RAR-R13,
      `analysis/d11_tm_diagnosis_2026-10-06.md`). They are real, and they
      had no measured cost in results at `3+0.03` and `10+0.1`, so they were
      not pulled ahead of Phase C, but they grow with the control.
      - **rec1:** a fail-high cascade whose late re-search never
        finishes, so the move ends only at the hard maximum.
      - **rec2:** a proved mate with no stop, whose next iteration grows to
        the ply cap and never finishes.
      - **rec3:** an iteration started under the soft target that a cascade
        stretches to about four times it.

      What the bound model must decide:

      - Whether a clock search may end inside an iteration (after a
        fail-high past the soft target, playing the fail-high move) or on a
        proved and stable mate. ADR-0065's "optimum adjusted only by
        completed exact iterations" is the item it interacts with.
      - How that interacts with B.5's retry trigger (mid-iteration stops
        above 10% reopen the discarded-best-move question).

      The aspiration loop itself is not the lever. RAR-S89 and RAR-S17
      rejected changing it, and it moves `bench 13`; the deep-iteration
      growth to the ply cap is a search property and stays out of scope.
      **Acceptance adds, beside the SPRT:**

      - a timed probe of rec1–rec3 at `60000+600` and `180000+2000`
        (`analysis/artifacts/basilisk-review-2026-10-06/tm_rec_probe.py`;
        maintainer-run, idle host) in which no move reaches the hard
        maximum;
      - `tm_games.py` (`analysis/artifacts/d11-tm-diagnosis-2026-10-06/`) on
        the gate's own games, reporting the share of won and mate moves at
        ≥ 0.9 × the maximum (0.48% and 0.24% at `3+0.03`, 1.77% and 2.59%
        at `10+0.1` on Phase C's head).

      The general overrun is input too: moves at ≥ 3 × the optimum are 4.7%
      of non-won moves at `3+0.03` and 1.8% at `10+0.1`; time past the
      optimum is 31.4% and 19.7% of all head time; and a started iteration
      always completes.

      D.1.2 is carried out in three addenda (added 2026-10-11 by maintainer
      request, so the two GUIDE holds it owns have steps that discharge
      them), gated together by D.1.2.3's SPRT:
        - **D.1.2.1 Won-ending time stalls: rec1's cascade, rec2's missing stop after a proved mate, rec3's overrun (RAR-R13) — `R2`.** First
          in D.1, after Phase C; earlier if RAR-R13's trigger fires (a game
          record shows a cost). Reproduce each with the timed probe
          (`tm_rec_probe.py`) on the head D.1 starts from, fix it inside the
          bound model (an end inside an iteration after a fail-high past the
          soft target; a stop on a proved, stable mate), and show the probe
          with no move at the hard maximum and `tm_games.py`'s won and mate
          shares at ≥ 0.9 × the maximum below their Phase C values (above).
        - **D.1.2.2 The forfeit margin against OS holds — `R2`.** The
          three gate time losses since 2026-10-10 (RAR-E29 round 3068,
          RAR-E32 rounds 1777 and 1855) each came on a short search whose
          process the OS held for 88.0, 82.4 and 69.4 ms with deadlines of
          89.2, 69.9 and 75.0 ms. Measure the hold distribution on this host
          from the gates' own `failed-games` and per-move timing records,
          then decide the engine's minimum move reserve at low clock (and
          whether the harness's 20 ms margin should change, a maintainer
          decision recorded in PROCESS) so that a hold of that size does not
          forfeit; a deterministic test with an injected delay checks it.
        - **D.1.2.3 The audit and the bound model: the ADR-0065 checklist, soft and hard bounds with the node-fraction multiplier; one SPRT `[0,3]` for D.1.2 — `R2`, then `V`.** The
          rest of D.1.2 as written above, with D.1.2.1 and D.1.2.2 inside
          the one gated change.
- **D.2 Lazy SMP quality at 4T/8T: diversity, shared TT and correction, soft-stop voting; 4T SPRT `[0,5]`; its premise is contradicted by RAR-M46, so re-scope first — `R2` investigation, `I2`/`V` sub-steps.** 4T and
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
  **Input, allocation and copy audit 2026-10-06:** `TranspositionTable`
  derives `Clone`, and a clone of the single-thread form copies the whole
  table. The one search-time clone, each helper's job in `search_parallel`,
  is safe only because `make_shared` runs first. Remove `Clone` and give
  helpers a handle only the shared form can produce, so the copy cannot be
  written; behaviour-neutral, exact fingerprint, no gate of its own.
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
- **D.3 Engine lifecycle and protocol robustness; `src/uci/` (planned) move; score normalisation research card (`analysis/uci_info_review_2026-09-16.md` item 6); zero crashes over pool tournaments — `R2`, then `I1`.** UCI
  parsing and dispatch, stop/ponder/infinite semantics, new-game resets,
  malformed input, panic reporting, Syzygy probe policy and thread safety.
  **Done out of band, 2026-10-01:**
  - `97ea52d`: the last `info` line now always describes `bestmove`. An
    iteration stopped after a window failed high or low on an unconfirmed
    move left that move's bound line last, serially under `go nodes` (now a
    deterministic test) and at Threads 8 under a clock (the flaky
    `threaded_bestmove…` test). Output only, both fingerprints exact.
  - `2326153`: the legacy search compiles again; `d754577`'s optimism call
    needed `b2core`, and CI builds that arm.
  - `b2c98c8`: the tablebase PV extension starts under a clock only when
    at least ten move overheads remain before the hard ceiling once the
    search has ended (100 ms at the default, about twice the largest
    blocked read measured, 54 ms). RAR-S94's forfeit came from the
    extension running at 58 ms; the old skip engaged only below about
    52 ms. The donors: Reckless has no extension; Stockfish gives it the
    same half-overhead box, checked after every probe, at each iteration's
    output inside the search's own time, with no guard against one blocked
    read. Rarog keeps its one extension after the search, the form RAR-S94
    measured, and adds the start rule. A unit test holds the rule at the
    forfeited move's numbers; on two 6-man roots at `100+30` the old binary
    extends and the fixed one does not, and at `3000+30` both print the
    same line. Both fingerprints exact. RAR-S97, its correctness read
    against the old binary on the endgame cohort, played 2026-10-01 with no
    defect: every clean win converted by both, no time loss, no fault.
  - `488d34a`: a tablebase root score prints without a bound (6 of 14 such
    lines carried one at a KQvK root, `go depth 8`; none now), as the donor
    prints it. Output only, both fingerprints exact.
  - `0827ab9` (2026-10-06, from the Basilisk review, `analysis/basilisk_review_2026-10-06.md`):
    `bench` floors its per-position and per-run nps at 1 ms like the
    `info` lines (a position solved inside the first millisecond printed
    its node count); a second `const` assertion keeps the mop-up below the
    decisive band; a regression test holds that a position repeating the
    game history scores a draw before any stored result. `bench 13` exact.
  - `6433760` (2026-10-06, from an allocation and copy audit): at a
    halfmove clock of 100 in check, the search's draw test built a heap
    `Vec` to ask whether the side to move is mated; it now fills a stack
    list in a cold helper. `tests/allocation_guard.rs` counts that test
    directly, because the growth bound cannot see a path this rare (it
    fails on the old code with one allocation). `bench 13` exact on default
    and pext; pooled-PGO no-regression read passed at −0.24% (−0.55% to
    +0.07%, six cycles; `analysis/artifacts/rule50-alloc/`).
  - C.12's speed pass (2026-10-17, on `c7`; `analysis/c12_speed_pass_2026-10-17.md`),
    the search, ordering, generation and board side, each `bench 13` exact
    at 10,521,141 / EBF 2.521: the per-node search parameter sets read as
    compile-time constants outside `tune` builds (`d61d4f6`; `d2ab515` keeps
    the root, clock and quiescence set runtime); pawn and knight attack sets
    from shifts (`58547cd`); quiescence's known check status passed to the
    make (`60f8b96`); the derived keys toggled by mask (`acd767e`); a
    one-kind generation pass's move flag known statically (`09bb98c`); the
    best-move selection by conditional moves (`081041c`); the child's TT
    cluster prefetched when the move is picked (`1aaa39e`, `65f490e`);
    staged king moves decided from the node's threat set (`9f388c5`); the
    attack tables built at compile time (`b52520d`).
  - C.12's third pass (2026-10-18, on `c7`; `analysis/c12_speed_review_2026-10-18.md`),
    kept by maintainer decision below the protocol's bar, each `bench 13`
    exact at 10,521,141 / EBF 2.521: the node's threat set computed only
    where a node reads it, the reverse-futility test in two steps and
    quiescence skipping it for an empty capture list (`842b586`); the node's
    check masks shared between the move loop and quiet scoring (`c01db68`).
    Together +0.41% (+0.28% .. +0.54%) pooled-PGO NPS with eight builds an
    arm, +0.08% (−0.11% .. +0.27%) with large pages on both arms.

  **Input from B.5.2.2:** a table read the page cache misses held the PV
  extension's DTZ root probe for about 54 ms and lost RAR-S94's round 745
  on time at 58 ms of clock (the failed-games record shows the extension's
  notice on that move; the same stall in an in-search probe is inferred,
  not observed). Probe policy under a short clock is this leaf's. Slow
  probes are reads the page cache misses (151 GB of tables against 128 GB
  of RAM); concurrent engines stretch the tail without raising its rate
  (`analysis/artifacts/b52-box-replay-2026-10-01/`).
  **Left from the 2026-10-01 audit of B.5.2.1** (the short-clock skip and
  the bound suffix are repaired above):
  - The extension gates on the loaded tables' size, not on
    `SyzygyProbeLimit`, which the root ranking and the in-search probes
    honour (read from `syzygy::rank_root_moves`, not reproduced; the
    committed 3-man fixture cannot show it, so it waits for a test that
    can).
  - A read blocked for longer than the start rule's margin plus the reserve
    (120 ms at the defaults) still loses on time; `go movetime` names no
    clock, so the rule does not apply there.
  Deterministic tests; zero crashes over the pool tournaments. Owns the last
  move of the target layout: `engine.rs`, `engine_command.rs`,
  `uci_protocol.rs`, `search_options.rs` and `bench.rs`/`wac.rs` go under
  `src/uci/` (planned) in the same leaf, behaviour-neutral, exact fingerprint.
  **Research card, added 2026-09-16 (`analysis/uci_info_review_2026-09-16.md` item 6):** the
  displayed score is raw internal units (startpos depth 1 reads `cp 143`;
  a tablebase win prints `cp 31744` because `TB_WIN_SCORE` sits below
  `format_score`'s mate cutoff). Stockfish and Reckless normalise to a
  win-probability scale. Fit a win-rate model on Rarog's own games (score
  against outcome by material phase), decide the `cp` mapping and a
  distinct tablebase-win band, and only then implement; display-only,
  gated by identity, no SPRT. **Noted, not adopted (2026-09-17):** an
  auto-sized hash as RBp4wn (tissatussa, Go, CC0, `github.com/tissatussa/RBp4wn`, read 2026-09-17)
  does it (probe at the ladder's maximum, skip move 1 whose empty-table
  search over-stores by about 35%, sample entries stored on moves 2–5,
  resize once to `peak / 1.4` on a log2 ladder, never again in the game;
  fixed `Hash` bypasses it). The GUI and every rated pool own `Hash`, and
  a mid-game resize discards warm entries, so it is at most a friendlier
  default for casual play; revisit only if the release checklist asks
  for one.
- **D.4 Tablebase policy: probing depth/limits, WDL/DTZ in conversion, recogniser interaction — `R2`.** Root and interior probing depth and limits,
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

- **E.1 Attribution checkpoint: B.2.0 review re-run on the B.9/C.11 heads; STC, `10+0.1`, 4T against 2.3.2 and the B.9/C.11 heads; maturity checklist — `V`.** Re-run the B.2.0 architecture review
  on the B.9 and C.11 heads first. Final head against 2.3.2 and against
  the B.9 and C.11 heads at STC, `10+0.1` and 4T; attributed Elo per programme
  from the accepted SPRTs; deficit meters; NPS; the maturity checklist
  (family map without unknown rows, every slot with a fitting instrument,
  every accepted representation reconstructing through `EvalTrace`).
- **E.2 Target gate: ≥50% against Critter 1.6a, Houdini 3, Rybka 4.1 and Fritz 16 at 1T and 4T (Rybka 4.1 replaced Rybka 4, 2026-09-19); the binding arm is 1T — `V`.** The pool measurement defined in section 1, at 1T
  and 4T. Met, or not met with the measured shortfall per engine recorded.
  **The binding arm is 1T:** RAR-M46 measured 4T as the easier arm for three
  of the four targets, by 26 to 75 Elo, so Phases B and C are judged against
  the 1T column.
  **Contingency (C.13.7, 2026-10-16):** if the 1T gate is missed against one
  engine by 5 Elo or less, C.13.7's contempt card (form 1, a search draw
  offset) is the registered lever, measured in this gauntlet with and
  without it (`analysis/c137_contempt_card_2026-10-16.md`).
- **E.3 Release 3.0.0 (E.2 met) or 2.7.0 through the tag-driven flow — `M`/`V`.** Version, changelog, release notes, fmt, debug and
  release suites, clippy, feature builds, fingerprint, PGO assets, ISA
  verification, CI matrix, tag and publish on maintainer instruction. Version
  is 3.0.0 if E.2 is met, else 2.7.0 (2.5.0 was cut at B.10 and 2.6.0 from
  the C head ahead of E, maintainer decisions 2026-10-03 and 2026-10-22). The release is cut through the
  tag-driven flow of **E.3.1**, which also carries the two workflow checks
  this release owed (tag equals version; one fingerprint asserted across the
  matrix) and may land any time earlier.
    - **E.3.1 Tag-driven release flow — `I1`, DONE 2026-10-05.** `release.yml`
      and `cargo xtask release-check`; 2.5.0 was released through it, nine
      assets at 11,171,726 (run 37284706939). The procedure is
      `docs/PROCESS.md` *Release*; the leaf's text is in `docs/archive/PLAN-closed-2026-10-05.md`.
    - **E.3.2 Release cut: version 3.0.0 (E.2 met) or 2.7.0, the `[Unreleased]` changelog reviewed and dated, suites, the release PR squash-merged, the `v` tag pushed on instruction through E.3.1's workflow — `M`.** E.3's own work, a leaf of its own since
      E.3.1 closed (2026-10-05): the version from `X.Y.Z-dev` to 3.0.0 if
      E.2 is met, otherwise 2.7.0, behaviour-neutral with the fingerprint
      held; `CHANGELOG.md`'s `[Unreleased]`, kept as changes landed,
      reviewed and dated; fmt, clippy, debug and release suites, feature
      builds; GUIDE's checkpoint marks the release; the PR to `master`,
      squash-merged once `CI` and `Release` are green; the
      `vX.Y.Z` tag pushed on instruction through E.3.1's workflow, by
      PROCESS *Release*.
## Phase F — NNUE (own data only)

**Rules.** Own data only, generated by Rarog's classical head and later by its
NNUE heads. Reckless is the runtime and trainer-pipeline donor; the
architecture ladder is ours. The classical evaluation stays in the tree as the
datagen baseline and the fallback until F.9 replaces it in releases.

- **F.0 Investigation: board events, accumulator ownership, trainer choice, data format, first architecture — `R3`.**
  Board event interface and accumulator ownership (dirty pieces, per-thread
  per-ply accumulators, king buckets, refresh cache); trainer choice
  (`D:/code/net_trainer` against Bullet) with feature ordering, quantisation
  and export contracts; data format, deduplication and split policy; the first
  architecture (768×N perspective network with output buckets); the cost
  ledger inherited from the board audit. Frozen handoffs for F.1–F.4.
  **Calibration point (2026-09-26, `analysis/gyatso_read_2026-09-26.md`):**
  a single-bucket 768×1024 perspective net with horizontal king mirroring,
  one output and squared clipped ReLU, trained on its author's own data and
  driven by a search simpler than Rarog's accepted core, holds official CCRL
  3258 at 40/15 and 3341 at blitz (Gyatso 1.5). The first architecture here
  is at least that; the ladder in F.7 starts above it. Gyatso's data
  pipeline (node-limited self-play with soft and hard caps, an opening book
  sampled by inverse use-count, viriformat output) is a compact reference
  for the data-format contract F.0 fixes for F.2.
- **F.1 Board events and accumulator scaffolding, behaviour-neutral for HCE; cost ledger — `I2`.** Behaviour-neutral
  for the HCE: factual move deltas, evaluator-owned stacks, validity and
  refresh semantics, randomized unwind tests, exact fingerprint, pooled NPS
  cost recorded.
- **F.2 Data generation at scale: 30–60M unique positions, splits, manifests, hashes — `V`.** 30–60M unique positions from the
  classical head under the adjudication-off profile, by-game splits,
  manifests, tablebase and hard-position cohorts; hashes frozen. Maintainer-run.
  The generator follows F.0's contract; node-limited play with soft and hard
  caps and inverse-use-count book sampling are the reference forms
  (`analysis/gyatso_read_2026-09-26.md`).
- **F.3 Trainer hardening and baseline nets, two seeds per configuration — `I2`, then `V`.** Deterministic
  pipeline, two seeds per configuration, validation selects, frozen test
  reports once.
- **F.4 Scalar integration: `quantised.bin` contract, integer-exact conformance, HCE fallback — `I2`.** `quantised.bin` contract, integer-exact
  conformance against the trainer's reference evaluation, clean HCE fallback.
- **F.5 Incremental and SIMD: same-net parity on every move type, tiers, pooled NPS attribution — `I2`, then `V`.** Same-net incremental parity
  on every move type, SIMD tiers (AVX2, PEXT builds, ARM NEON), scalar
  reference retained, pooled-PGO NPS attribution.
- **F.6 Search re-fit for the network — `V`.** Score scale, correction
  histories, margins, qsearch and SEE thresholds re-fitted on the new
  evaluator (C.10's protocol).
- **F.7 Architecture ladder: output buckets, king buckets, relation/threat inputs; one axis at a time — `R3` with `I2`/`V` sub-steps.** Output buckets,
  king buckets with mirroring, then relation and threat inputs as in
  Reckless, one axis at a time; each net gated against the previous.
- **F.8 Data frontier: on-policy refresh, deduplication, hard-position mining — `V`.** On-policy refresh with the strongest net,
  deduplication, hard-position mining; repeat while a cycle accepts.
- **F.9 NNUE release: beat the classical release at STC, LTC and 4T; platform matrix — `M`/`V`.** Beat the classical release at STC, LTC
  and 4T; platform matrix; publish.
- **F.10 CCRL top-100 gate — `V`.** Submit; the list decides. Shortfall
  measured against the pool and fed back into F.7/F.8.

## Phase G — Scaling, platforms and the top 50

- **G.1 High-thread and NUMA: 8/16/32T, TT and net placement, large pages, affinity policy — `R2`, then `I2`.** 8/16/32T scaling, TT and
  net placement, large pages, thread affinity policy.
- **G.2 Platform and product: Chess960 on demand, distributed testing, and the OPTIONAL universal binary (`analysis/universal_binary_2026-09.md`) — `I1`.** Chess960 on demand, distributed
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
- **G.3 Frontier: larger nets, data scaling, LTC search fit; CCRL top-50 gate — `R3`.** Larger nets, data scaling, search fit at LTC; the
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
| Fixed-node shape | oracle differential at stride 1, depth at 300k nodes, EBF, tactical suite at fixed depth and equal nodes, positional screen at 100k; diagnostics under rule 8's cluster ladder | B clusters |
| Endgame layers | theory truth, drawn overclaim, conversion at 60k/200k/600k, floors | C.5 |

Sizing every SPRT: `tools/spsa_convergence_model.py` and RAR-M10's drift
model at the expected value, before registration. Bracket, cap, book, clock
and adjudication never change after games are seen.

## 4. Release rules

- A release ships only from a head whose every accepted cluster has a ledger
  row and whose deficit meters are recorded at the checkpoint before it.
- 3.0.0 requires the E.2 gate met; 2.4.0 requires at least +40 Elo at STC over
  2.3.2 with the lower bound above +25, positive LTC and 4T lower bounds.
- 2.5.0 adopts 2.4.0's rule against 2.4.0 (maintainer decision 2026-10-04)
  and meets it: STC +272.4 (95% about +245 .. +308, RAR-M64), `10+0.1`
  +260.5 ± 16.0 (RAR-M65), 4T +322.7 (95% +289 .. +362, RAR-M66).
- 2.6.0, cut from the C head ahead of Phase E (maintainer decision
  2026-10-22), meets the same rule against 2.5.0 (RAR-M76): STC +153.4 ±
  10.7, `10+0.1` +149.3 ± 15.1, 4T +149.7 ± 14.7.
- NNUE releases require a win over the last classical release at STC, LTC and
  4T, and a clean platform matrix.
- Tag, push and publish only on maintainer instruction. From E.3.1 on, a
  release is cut by pushing a `vX.Y.Z` tag on `master`: the workflow
  validates tag, version, branch and changelog section, builds and
  fingerprint-checks every asset, and publishes only after all of them
  passed; the notes come from `CHANGELOG.md`, never from the GitHub form.

## 5. Documentation ownership

| File | Purpose |
|---|---|
| `GUIDE.md` | The checkbox board first, generated from this file's step heads by `tools/diag/guide_board.py` (closed phases as one- or two-sentence summaries), then the hand-kept checkpoint, holds, model mapping, prompts and operator guide |
| `docs/PLAN.md` | This roadmap: objective, rules, phases, protocols |
| `docs/EXPERIMENTS.md` | Frozen predictions, results, calibration, retry triggers, recipes |
| `docs/PROCESS.md` | Research/handoff template and recurring build, fit, gate and release procedures |
| `CHANGELOG.md` | User-facing changes: `[Unreleased]` collects them as they land; a release dates the section, whose body becomes the release notes |
| `docs/HISTORY.md` | Completed work, retired numbering and the number map; never a source of the next step |
| `analysis/` | Per-leaf analyses and measurement records; raw artifacts stay local and ignored |
| `docs/archive/` | Verbatim archived roadmaps and closed phases; a `PLAN A.x`, `B.x` or `E.3.1` citation resolves in `docs/archive/PLAN-closed-2026-10-05.md` |
