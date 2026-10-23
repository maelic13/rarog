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
| D.1.6 | READY_FOR_IMPLEMENTATION | R3 | The root fail-high cascade fixed in the search and the evaluation (maintainer decisions 2026-10-23): the research closed (RAR-R19) with a cluster of four parts that ends every probed stall within its optimum (RAR-R20); the gate next, then the landing; before D.1.5 so each gate attributes to one change |
| D.1.6.2 | READY_FOR_IMPLEMENTATION | I2 | The cluster under gate is the arm `6fd195e` (bench 8,996,837): `mopup_known_win` 6000, `AspDeltaSquareDiv` 10193, `CoreLmrDeeperBudget` 1, `TmDepthFreeze` 5000 as the defaults. On H1 it lands on `d1` with the fingerprint re-declared, `TmFailHighTarget` and the unused instruments removed, and the changelog entry |
| D.1.6.3 | GAME_GATE | V | The cluster verified (RAR-R20: seven of eight probe rows within the optimum, −52.5 ± 23.9 unfitted at PGO, even at equal speed per part). The road to the gate is RAR-R21, staged: three 2,000-game equal-speed reads decide whether SPSA block 1 over `d16fix` (fourteen coordinates, the fit arm `217125b`) runs; the gate (`[-5,5]`, cap 20,000 pairs, the probe, the `10+0.1` read) is re-registered on an arm rebuilt with the fitted values. Maintainer-run: `run_r21.ps1 -Read all4|nofreeze|freeze|fit` |
| D.1.5 | READY_FOR_IMPLEMENTATION | V | D.1's closing read after D.1.6: the clock pieces at their seeds against the 2.6.0 engine with the cascade fix, harm reads at both controls, a four-thread read, the probe and the profile; registered when D.1.6 lands. RAR-R16's SPRT ended unresolved |
| D.1.2.1 | READY_FOR_IMPLEMENTATION | R2 | Won-ending stalls: rec1 and rec3 are D.1.6's (the cascade fixed in the search); rec2 is the proved-mate stop; ticks on D.1.6's probe and D.1.5's read |
| D.1.2.2 | READY_FOR_IMPLEMENTATION | R2 | The forfeit margin is a horizon question: the sub-second contraction is a cluster item, the reserve stays; ticks on D.1.5's reads |
| D.1.2 | READY_FOR_IMPLEMENTATION | R2 | Research closed 2026-10-22 (RAR-R14); closes with D.1.5 |
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
      them), gated together by one SPRT, which D.1.5 now holds.
      **Research closed 2026-10-22, `READY_FOR_IMPLEMENTATION`** (RAR-R14,
      `analysis/d12_tm_research_2026-10-22.md`; re-opened in full by the
      maintainer's request of 2026-10-22; zero games). Read from 11,200 of
      the C head's games in six runs (RAR-M73–M76) and the formulas:
      - **The floor.** The budget assumes fifty moves to go at every clock,
        so at 500 ms it still counts 1,470 ms of future increments; a
        `3+0.03` game drifts to the clock where the optimum equals the
        increment, 155 ms. 22–41% of the head's games touch 100 ms and 21%
        of its moves against RAR-M75's field are played under 200 ms.
        Stockfish 11, Dragon 1 and Komodo 13.3 never go under 200, 790 and
        650 ms; Houdini 3 and 6.03 go as low as the head. Current
        Stockfish's sub-second horizon contraction (`mtg = time_ms / 20`)
        alone lifts the drift to 400 ms; Reckless's linear bounds sit at
        510 ms.
      - **Holds and forfeits.** OS holds of 80 ms or more hit 0.11–0.15%
        of moves at one thread and 0.4% at four (the move waits for every
        helper); four time losses in 226,180 games at the 20 ms margin
        since 2026-10-10, each a 69–129 ms hold at a clock of 70–163 ms in
        a lost game. No reserve covers a hold longer than the clock; a
        policy that never holds such a clock does. Elo value here under
        0.01; the cost is robustness on another harness.
      - **Spend.** Median 0.78–0.91 of the optimum, 99th percentile 3.7–4.5;
        21–31% of all head time is past the optimum; the front-loaded shape
        (44–47% of the game's time in the first forty plies) is shared with
        Stockfish 11 and Houdini 6, while Dragon and Komodo spread it and
        leave 1.2–1.6 s unused. No zero-game instrument reads the shape's
        Elo.
      - **Stalls.** RAR-R13 reproduces on the C head (won and mate moves at
        ≥ 0.9 × the maximum 0.30–0.44% and 0.24–0.53% at `3+0.03`, 1.44% and
        3.40% at `10+0.1`), all in wins bar three draws.
      - **Donors.** Reckless 0.10.0-dev: linear soft and hard bounds in the
        main clock, a continuous node-fraction multiplier (×0.55–2.5), pv
        and eval stability discounts, no mate stop, no in-iteration stop,
        a 65% vote. Stockfish master 2026-09-30: Rarog's shape plus
        `originalTimeAdjust` (a per-control scale, 0.76 at `3+0.03`, 1.53 at
        40/15), the sub-second contraction, a time-disadvantage scale, a
        stability discount (`timeReduction`), an effort cut that fires only
        above 90% concentration, a `mate_in(3)` stop, shallower iterations
        once half the target is spent. RAR-R13's "Stockfish has no clock-mode
        mate stop" is corrected there; rec2's `mate 5` lies outside its rule.
      The decision is one clock-only cluster (D.1.3), one fit (D.1.4) and
      one gate (D.1.5); the addenda's findings:
        - **D.1.2.1 Won-ending time stalls: rec1's cascade, rec2's missing stop after a proved mate, rec3's overrun (RAR-R13) — `R2`.**
          Research closed 2026-10-22: the fix is two cluster items, a
          proved-mate stop (a clock search ends after a completed iteration
          whose root mate for the side to move has held with the same move
          and distance for three iterations, or whose score is a loss in at
          most two plies) and a root fail-high stop past the soft target
          (a window that fails high once the elapsed time is at or past the
          soft target ends the search and plays the move that failed high,
          with its lower bound; that bound exceeds the previous best's
          exact score, so no best move is discarded and B.5's mid-iteration
          trigger is not reopened). Ticks when D.1.5's timed probe
          (`tm_rec_probe.py`, both clocks, idle host) shows no rec move at
          the hard maximum, rec2 under 2 s, and `tm_profile.py` on the
          gate's games reads the won and mate shares at ≥ 0.9 × the maximum
          below 0.2%.
          **Re-owned 2026-10-23 (maintainer decision; RAR-R18).** rec1's and
          rec3's cascades are a search property, full-depth re-searches
          after a root fail-high, and are fixed in the search by **D.1.6**;
          the root fail-high stop that capped them cost about 15 Elo at
          `3+0.03` and is retired (off by default, `a0f9755`). rec2's fix stays
          the proved-mate stop (RAR-P42 saw 2.6.0 still searching past a
          proved mate at `120+1`). Ticks when D.1.6's probe shows rec1 and
          rec3 under the hard maximum with no clock stop, and D.1.5's
          closing read shows rec2 under 2 s and the won and mate shares at
          ≥ 0.9 × the maximum below 0.2%.
        - **D.1.2.2 The forfeit margin against OS holds — `R2`.**
          Research closed 2026-10-22: the margin is a horizon question. The
          cluster contracts the horizon below one second of clock
          (`mtg = min(50, ⌊clock_s × TmHorizonPerSec⌋)`, seed 50, a fit
          coordinate); the engine's reserve (`2 × Move Overhead`, +30 ms
          above one thread) and the harness's 20 ms margin stay, with the
          arithmetic recorded in the packet. A deterministic test injects a
          late `go` stamp at a 150 ms clock and checks the reserve holds.
          Ticks when D.1.5's games show no candidate-arm time loss and
          `tm_profile.py` reads the candidate's clock-floor median above
          350 ms at `3+0.03` with under 3% of games touching 100 ms.
        - **D.1.2.3 The audit against the ADR-0065 checklist and the bound-model decision — `R2`, DONE 2026-10-22.**
          Ticked in the packet: present are distinct budgets, an immutable
          maximum rooted at `go` receipt, and completed-iteration
          adjustment by best-move change, score trend and effort; absent
          are a stability-over-iterations discount (both donors carry one),
          ponder credit (ponder is off in CCRL and Colosseum) and a
          first-iteration guard on the hard stop; different is the helper
          vote (D.2). The effort term has Stockfish's shape, a cut only for
          concentrated effort, which is why RAR-S47 read it inert; Reckless's
          is a continuous extender. The bound model: a clock search may end
          inside an iteration after a root fail-high past the soft target,
          and on a proved stable mate; a stability factor is added; the
          effort band's floor and endpoints become coordinates whose ranges
          reach both donors' shapes, so the fit decides the shape question.
          The SPRT moves to D.1.5.
    - **D.1.3 The clock cluster: RAR-R14's six clock-only mechanisms and the cross-move score trend — `I2`, DONE 2026-10-22.**
      From `analysis/d12_tm_research_2026-10-22.md`, *Implementation
      handoff*, in `src/search/time.rs` and the root loop of
      `src/search/mod.rs`: (1) the sub-second horizon contraction
      (`TmHorizonPerSec`, seed 50, range 20–100); (2) a stability factor
      `max(TmStabFloor, TmStabBase − TmStabSlope × k)`, `k` the completed
      iterations since the best move last changed, seeds 1.2881 / 0.0440 /
      0.716; (3) the effort band as coordinates, `TmEffortHigh` → `TmEffortLow`
      as the best move's share of the search's nodes rises from
      `TmEffortFloor` to 1.0, seeds 0.924 / 0.71 / 0.79, ranges 0.6–2.0 /
      0.4–1.0 / 0.0–0.95; (4) the proved-mate stop and (5) the root
      fail-high stop of D.1.2.1; (6) no hard stop before the first
      iteration completes. Invariants: `bench 13` unchanged (13,987,223 /
      EBF 2.565, by construction); `bestmove` is the move the last `info`
      line names and a fail-high stop's line carries `lowerbound`; the
      maximum is never raised; the DTZ root's `maximum = optimum` and the
      tablebase extension's box are unchanged; nothing per node changes.
      Tests: the first-iteration guard, the mate stop at a fixed node budget
      (fires after three stable mate iterations, not under `go infinite`),
      the fail-high stop on a constructed window, the horizon at 500 and
      1,000 ms, the injected late `go` stamp at 150 ms, SMP inertness at one
      thread. Instrumentation behind `diag`: counters for the two stops and
      the guard; a per-iteration log of `k`, the effort share and each
      factor for the activation read. Cheap qualification: the fingerprint;
      the timed probe (maintainer-run, idle host, about eight minutes).
      Non-goals: the aspiration loop (RAR-S89, RAR-S17), the thread vote
      (D.2), ponder, `nodestime`, the `movestogo` branch, the harness
      margin, Stockfish's `originalTimeAdjust` and time-disadvantage scale
      and `increaseDepth` (their triggers are in the packet).
      **Implemented 2026-10-22 on `d1`, `IMPLEMENTED`** (engine commit
      `5fec0d7`; `d829a8b` before it states the preconditions of two
      existing tests that the guard and the horizon change, and passes on
      the unchanged engine; tooling `2811a8a`). At the seeds, in
      `src/search/time.rs` (`sudden_death_horizon`, `MateWatch`,
      `soft_stop`) and both root loops of `src/search/mod.rs`. The five
      coordinates are in `SearchParams` with ranges `TmHorizonPerSec`
      20–100, `TmStabBase` 0.80–2.00, `TmStabSlope` 0–0.20, `TmStabFloor`
      0.40–1.00 (the packet set no stability ranges; these are the
      implementation's), `TmEffortFloor` 0–0.95, and `TmEffortHigh` widened
      to 0.6–2.0. What the handoff left open:
      - The two stops are the main thread's decision and end a pool's
        search directly; every thread's own soft target carries the new
        factors, and the vote is unchanged. D.1.5's 4T read covers this.
      - MultiPV takes the mate stop on line 1, not the fail-high stop, which
        would end a depth before its other lines are searched.
      - The guard holds the hard stop while the first iteration runs
        (`root_depth ≤ 1`).
      - The effort share is the best move's `RootMove.nodes` over the
        thread's nodes. The per-iteration counters went, with their one
        root-only store in `negamax`.
      - A fail-high after a fail-low can have a lower bound under the
        previous iteration's score. The move is still not a discard: the
        bound is above every move's bound from that fail-low.

      Verification on the final state, exit status read directly:
      `cargo fmt --check` 0; `cargo clippy --all-features --all-targets` 0,
      zero warnings; `cargo test --release` 0, 412 passed; `bench 13` on a
      fresh no-feature release build **13,987,223 / EBF 2.565**; the search
      tests under `diag` 0, 116 passed. `cargo test` in debug passed all
      411 on the same content before a line-ending-only change, then failed
      2 of 3 times on `first_kpk_search_in_a_fresh_process_costs_what_a_warm_one_does`
      (19 and 22 ms cold against 2 and 4 ms warm). That is a timing test of
      a `go depth 4` search the clock never reaches. The unchanged engine
      fails it at the same rate when `tt_sizing` runs just before it (1 of
      5 on each tree), and it passes 11 of 11 alone on the cluster: a
      load flake that predates the cluster. Each new test fails when its own
      mechanism is disabled (four mutations: fail-high, mate, guard,
      horizon). Instrumentation: `diag` counters `tm_mate_stop`,
      `tm_fail_high_stop` and `tm_first_iteration_guard`, and an
      `info string tm` line per completed iteration, read by
      `tools/diag/tm_activation.py` over B.5's 300 positions at `3000+30`.

      **Instrument findings for D.1.4 and D.1.5:**
      1. `TmOptScale`'s range was 0.5–2.0, so the wire check's 0.3 would have
         been clamped to 0.5 without a word. Colosseum's guard checks option
         names, not values. The lower bound is now 0.2.
      2. `tm_profile.py` profiles by PGN name, and a one-binary categorical
         match names both players alike, so it would pool the arms.
         `analysis/artifacts/d13-clock-cluster-2026-10-22/tm_wire_read.py`
         (SHA-256 `23ef9c7bcc360cda…`) relabels each game by arm from
         `games.jsonl`, checking each game's PGN hash, then runs
         `tm_profile.py` unchanged. Checked on `b32-cat-both`: 2,000 games,
         both arms 4.26 s a game.
      3. `tm_profile.py` and `tm_table.py` transcribe the fifty-move
         horizon, so they overstate the candidate's maximum on sub-second
         moves. D.1.5's "≥ 0.9 × the maximum" read must transcribe the
         contraction first. The probe's 60 s and 180 s clocks are
         unaffected.
      4. The registered fallback (mechanisms 1, 4, 5 and 6 without 2 and 3)
         cannot be set by options alone. Stability is neutral at
         `TmStabBase` = `TmStabFloor` = 1.0 with `TmStabSlope` 0, but moving
         the effort share to the whole search is code. The fallback arm
         needs the per-iteration share back, as a recipe patch.

      **`LOCAL_QUALIFIED` 2026-10-22, DONE.** The maintainer's PGO build,
      `tools/test_engines/rarog-d13cluster-pext-pgo.exe` (SHA-256
      `63d49fdcbc0824c6…`), was built from a clean tree at `da2df4e`, which
      has the same engine inputs as `5fec0d7`; it benches 13,987,223 / EBF
      2.565. The timed probe,
      `analysis/artifacts/d13-clock-cluster-2026-10-22/tm_rec_probe_d13.py`
      (SHA-256 `53a42b924714eea8…`), is the 2026-10-06 method pointed at
      that build. It ran on an idle host (3.5% CPU over 20 s) and wrote
      `tm_rec_probe_d13.txt` (SHA-256 `c33fef86716335a9…`). **Verdict PASS:**
      no rec move reached the maximum, and rec2 finished under 2 s.

      | Clock | rec1 | rec2 | rec3 |
      |---|---|---|---|
      | `60000+600` | 4,079 ms, 0.87 × optimum, d18, exact | 5 ms, d10, mate 6 | 2,061 ms, 0.47 ×, d21, lowerbound |
      | `180000+2000` | 8,585 ms, 0.57 ×, d19, lowerbound | 5 ms, d10, mate 6 | 12,821 ms, 0.90 ×, d21, lowerbound |

      On 2026-10-06 the B.9 head spent 32.3, 31.6 and 10.4 s at the short
      clock and 104.4, 102.2 and 10.4 s at the long one. Stockfish 19 spent
      1.5–45.6 s on the same three.
      - Three of the four non-mate moves ended on the fail-high stop's
        `lowerbound` line, and every one ended under its optimum.
      - The packet's "rec1 and rec3 within about 1.5 × their soft target"
        cannot be read from this output, which prints no soft target. It is
        D.1.5's to read at the fitted values.
      - rec2 stopped at depth 10 on a mate in 6 that held three iterations.
        Stockfish reports mate in 5. The stop plays a proved mate, not the
        shortest; each later move searches again from a mate at most one
        move shorter. The packet allowed this (`mate_in(3)`, the donor's
        rule, would not have stopped rec2 at all).

      This is the cheap qualification at the seeds. D.1.2.1 still ticks on
      D.1.5's probe at the fitted values and on its profile read.
      **Addendum 2026-10-22 (maintainer decision): a seventh mechanism, the
      cross-move score trend** both donors carry (RAR-R15, *Amendment*).
      Each thread keeps the root score its previous search in the game ended
      on, cleared by `ucinewgame`, and the falling eval adds `TmFallPrevSlope
      × (the previous search's score − this iteration's)` inside the same
      clamps; seed 0.0204 (Stockfish's raw slope, a foreign unit), range
      0–0.1. Zero-game screen on 1,600 wire-check games: the score moves
      18 cp at the median across a move pair, and the term lifts the factor
      off its clamp on 20% of moves. Engine `1f185cb`, clock-only;
      verification on the final state, exit status read directly:
      `cargo fmt --check` 0, `cargo clippy --all-features --all-targets` 0 at
      zero warnings, `cargo test` 0 and `cargo test --release` 0, `bench 13`
      on a fresh no-feature release build **13,987,223 / EBF 2.565**. Its
      wire check and the thirteenth coordinate are D.1.4's.
      **The fail-high stop retired 2026-10-23** (RAR-R18, maintainer
      decision): off by default (`a0f9755`, after `ec416d4` had moved it to
      twice the target); the cascades it capped are D.1.6's, in the search,
      and the spin goes when that fix lands. The other six mechanisms stay
      at their seeds for D.1.5's closing read.
    - **D.1.4 The clock fit: a wire check, an activation read, then SPSA blocks over the clock coordinates — `V`, DONE 2026-10-23.**
      Conditional (PLAN rule 4), with the zero-game sweep replaced by what
      a clock-only coordinate allows: (a) the wire check, `TmOptScale` at
      0.3 and each new coordinate at a range end in a 200-game match, each
      of which must move `tm_profile.py`'s time per game as the formula
      says (about −60% for the first); (b) the activation read from D.1.3's
      log on B.5's 300-position `3000+30` replay: a factor at one end on
      more than 90% of iterations drops its coordinate. Then one registered
      SPSA block of 60,000 games at `3+0.03`, Colosseum `spsa-tune`, over
      the twelve clock coordinates (the seven of 2.3.0 and the five new
      ones), seeded from the values above; the movement stop rule (rule 7c)
      ends it, and a second block runs only if block 1 moved a coordinate a
      full step and the rule did not fire. About ten hours a block.
      **(a) `TmOptScale` played 2026-10-22: the wire is live, but the
      registered size missed.** Maintainer-run: 200 games at `3+0.03`
      (`-CategoricalTuneBuild`, seed 20261022), on
      `tools/test_engines/rarog-d13cluster-tune.exe` (SHA-256
      `87c92774550c41d3…`, clean tree at `eff17c3`, bench 13,987,223). Arm A
      set `TmOptScale=3000`; arm B kept the seeds. The run is in
      `tools/results/d14-wire-optscale03`, with manifest
      `colosseum_match_d14-wire-optscale03_20261022_181558.manifest.txt`
      (SHA-256 `882daa78c8db131a…`): 0 faults, recount agrees. Read by
      `tm_wire_read.py`:
      - Time per game fell from 3.77 s to 2.92 s, **−22.5%**, against the
        registered "about −60%".
      - Time per move fell from 61 to 48 ms, −21%. In plies 0–19, where
        both arms' clocks are near 2.9 s, it fell from 86 to 39 ms, −55%.
      - From ply 80 arm A holds about twice the clock (2.3 s against
        1.1 s) and spends more per move than arm B. It ends draws with
        2.7 s left against 1.1 s.

      **Calibration:** the miss is in magnitude, through the mechanism. The
      budget is a share of the remaining clock, so time saved early buys
      time back later, and the expectation took the clock as fixed. The
      engine applies the value exactly (target over optimum 0.3000 of the
      seeds' at depth 1), and the harness and reader report the change move
      by move. So the instrument does what the check exists to prove. The
      plies 0–19 figure is a reading, not a pass rule written after the
      result. The coordinates still to check (each new one at a range end)
      state, before their games, whether their expectation is per game or
      per move at equal clock.

      The `−145.1 ± 31.5` Elo of arm A says only that a third of the budget
      costs a lot. On the arm at its seeds:
      - its clock floor had a median of 882 ms, p10 248;
      - no game went under 100 ms, and 16 of 200 went under 200 ms.

      That is a hint for D.1.5's floor prediction, not its read: the
      opponent was the same build at 0.3, and the sample is 200 games.

      Instrument notes:
      - `tm_profile.py` labels its 99.9th percentile as a second `p99`.
      - Its spent-over-optimum and ≥ 0.9 × maximum columns use the head's
        formula (D.1.3's finding 3).
      - The manifest's `harness_hold_ms:` reads p999 101 and max 878 ms,
        with 53 of 24,612 moves over the margin, both arms pooled. At these
        clocks no game was lost on time.

      **Registered 2026-10-22 as RAR-R15** (`docs/experiments/RAR-R15.md`),
      before any game of the remaining checks or of the block:
      - **(a) The five new coordinates' wire checks.** Each has a frozen
        formula expectation per move at equal clock, and a check is live
        past twice its SE. Two depart from "each at a range end in 200
        games", with reasons: `TmStabFloor` is checked at `TmStabSlope` 2000
        on both arms, because at the seed slope it binds on 5% of
        iterations and its check would be a null; `TmEffortFloor` plays
        800 games, because its −6% is 2.5 SE at 200. One batch runs them,
        `analysis/artifacts/d14-clock-fit-2026-10-22/run_wire_checks.ps1`.
      - **(b) The activation read, done:** 3,432 iterations, and no factor
        at one end on more than 90%. The effort factor is at `TmEffortHigh`
        on 87.5%, falling eval at its lower clamp on 66.9%, stability at
        k = 0 on 38.1% and never at its floor. All twelve coordinates stay.
        The rule's mapping from a pinned factor to "its coordinate" is
        undefined for multi-coordinate factors; it was not needed, and
        would have gone to the maintainer.
      - **(c) Block 1 of `d14block1`.** 2,000 × 30 games, seed 20261023;
        at most two blocks on rule 7c's count. The surface's ranges are
        the declared ones, except `TmOptScale` at 5000..20000. It runs
        only after all five wire checks read live.

      **Wire checks played 2026-10-22: four live, the horizon's not by its
      frozen rule** (RAR-R15, *Result of (a)*). The stability base, slope
      and floor and the effort floor moved time a move by +39%, −15%, +17%
      and −9%, each live and inside or past its band. The horizon moved the
      1000–2000 ms band −19% but the 500–1000 ms band only −2.3%, against
      "both bands ≥ 5% lower". The engine applies the coordinate exactly
      there (the optimum at 750 ms is 0.686 of the seed's, the formula's
      figure), and the clock floor rose (p10 146 → 242 ms, games under
      200 ms 35 → 11). Near 40 ms a move, time spent stops following the
      optimum. **Maintainer decision 2026-10-22: `TmHorizonPerSec` stays in
      the surface;** the miss is a calibration of the check's instrument
      assumption (a per-band read follows the optimum at every clock), not a
      pass, recorded in RAR-R15. An independent review the same day
      (RAR-R15, *Maintainer decision and review*) re-read D.1.3 from
      source, tests and artifacts and found no defect: the stops, the guard,
      the factors and the invariants are as recorded above, and the
      fail-high test fails under both mutations (the stop disabled, and the
      stop firing before the soft target). The surface stays at twelve: no
      clock constant outside the handoff becomes a coordinate, no search
      coordinate joins (an aspiration coordinate would move `bench 13` and
      D.1.5's attribution), and `TmStabFloor`, gradient-free at the seeds,
      stays with its noise walk quantified (0.41 steps SD a block) and a
      reading rule. **Block 1 is prepared, maintainer-run:** RAR-R15's
      command, dry-run through `colosseum.ps1` (12 coordinates, 2,000 × 30,
      `r-end` 0.0031, no rail warning, 15 slots, Hash 64, Threads 1,
      MultiPV 1), about 8–11 hours. The morning read is
      `python tools/spsa_block_rule.py d14block1 tools/results/d14-spsa-block1`
      and the manifest's `arm_clamps:` line. **Amended 2026-10-22
      (maintainer decision), before any game:** the cross-move score trend
      joins the cluster (D.1.3's addendum) and the surface, thirteen
      coordinates (`TmFallPrevSlope`, seed 204, range 0..1000, step 63),
      regenerated and audited; its wire check (`TmFallPrevSlope=1000` against
      the seeds, 200 games, live if plies 0–19 spend ≥ 5% more a move;
      formula ×1.36) ran before the block on the new tune build,
      `tools/test_engines/rarog-d13cluster7-tune.exe` (built clean at `2b9831f`, engine
      inputs `1f185cb`'s, bench 13,987,223), which the block's `-Engine`
      and `-ExpectRevision` name: **live, ×1.163 in plies 0–19** (about six
      SE; RAR-R15, *Result of the cross-move wire check*), 0 time losses.
      Block 1 at thirteen is dry-run and prepared, maintainer-run.
      **Block 1 played 2026-10-22/23** (60,000 games in 11 h 40 m, 0
      faults, no clamp; RAR-R15, *Result of (c)*).
      - Two of thirteen coordinates moved a full step: `TmOptScale` 10000 →
        11687 (+1.80 steps, level by iteration 1,500) and `TmStabSlope` 440
        → 302 (−1.10, still travelling). The rule says **STOP**, so theta is
        block 1's rounded centres.
      - Every move past noise is toward spending more time:
        `TmOptScale × TmEffortHigh` is ×1.233.
      - The chained launch started block 2 and stopped it after 90 games. A
        cut block never counts. Resuming it (11.6 h) is the maintainer's
        call; the agent recommends against.

      **Closed 2026-10-23 on the maintainer's decision.** Theta is block 1's
      rounded centres, and block 2 is abandoned. Another clock fit is
      registered only if D.1.5 accepts H1 at +10 Elo or more, from the gated
      head (RAR-R15, *Maintainer decision 2026-10-23*).
    - **D.1.5 The clock gate, D.1's closing read after D.1.6: the clock pieces at their seeds against the 2.6.0 engine with the cascade fix; harm reads at `3+0.03` and `10+0.1`, a four-thread read, the probe and the profile — `V`.**
      Registered as its own ledger entry before any game, with the packet's
      prediction re-frozen at the fitted values: the fitted cluster against
      the head, Colosseum `sprt-default`, `[0,3]` nElo, `3+0.03`, one
      thread, cap 20,000 pairs (about seven hours; a true +6 nElo accepts
      near 26,000 games); then 1,000 games at `10+0.1` and 1,000 at four
      threads as direction reads (about 0.6 and 0.8 hours). Acceptance adds
      D.1.2.1's probe and D.1.2.2's profile and forfeit reads. H1 bakes the
      fitted values, ticks D.1.2.1, D.1.2.2, D.1.2 and D.1, re-declares the
      fingerprint (unchanged) and writes the user-facing changelog entry.
      H0 returns the cluster to research with its estimate; the cap bakes
      nothing and records "unresolved". **Fallback, registered now:** on
      H0 or the cap, mechanisms 1, 4, 5 and 6 alone at their seeds are
      offered under AGENTS' repair case 2 (a 2,000-game read with a harm
      rule at −13 nElo), each expected near zero and bought for
      correctness; the maintainer decides whether to run it.
      **Registered 2026-10-23 as RAR-R16** (`docs/experiments/RAR-R16.md`),
      before any game:
      - **The arms.** The candidate is `rarog-d15fit-pext-pgo.exe`, block
        1's theta baked on the throwaway arm `d15-arm` (`082a780`, recipe
        `analysis/arm_patches/082a780-rar-r16-d15gate.patch`), version
        `2.7.0-dev+d15fit`. The head is `rarog-d15head-pext-pgo.exe` at
        `468ed7b`. Both are clean PGO builds and both bench 13,987,223.
      - **One batch runs everything,**
        `analysis/artifacts/d15-clock-gate-2026-10-23/run_d15.ps1`: the
        SPRT, the two direction reads, the probe, then the profile reads by
        `tm_profile_d15.py`. That reader is `tm_profile.py` with the
        candidate's horizon and p99.9 fixed (D.1.3's finding 3), checked
        against `tm_profile.txt`. All three matches are dry-run clean.
      - **The four-thread read** carries `--cores-per-game 4`. Without it
        the runner gave a game one physical core and no check refused it;
        `colosseum.ps1` refuses that now (`6c3a1a1`).
      - **The prediction, re-frozen:** +2 ± 4 Elo; H1 0.30, cap 0.50, H0
        0.20.

      **Before the SPRT, 2026-10-23** (RAR-R16, *Results before the
      SPRT*):
      - **The wire rule is corrected.** As registered it compared the
        candidate's time with the head's, but the fit's change is against
        the cluster's seeds; it was replaced before any SPRT game (RAR-R16,
        *Amendment*). The replacement check against a seeds PGO read
        **live, ×1.21 a move in plies 0–19**.
      - **Direction reads:** `10+0.1` −6.3 ± 11.7 Elo, four threads −10.4 ±
        11.6. They decide nothing; at four threads the clock-floor median
        rose from 151 to 353 ms and games under 100 ms fell from 39 to 0.
      - **Probe PASS.**
      - The SPRT is next, maintainer-run.

      **The SPRT is paused at 1,782 pairs (+0.2 ± 6.6 Elo, LLR −0.10,
      unresolved), and RAR-R17's decomposition has played, 2,000 games
      each:**
      - **The cluster at its seeds against the head: −14.9 ± 8.8 Elo.** It
        spends 7.4% less time a move at every stage and ends games with
        clock unused. The likeliest source is the fail-high stop cutting
        the head's overruns.
      - **The horizon at 100 against 53: −0.9 ± 8.0.** No late-game cost
        shown, and it keeps all 2,000 games above 100 ms of clock (443 at
        100 did not).
      - The fit bought back roughly what the seeds lost.
      - The research splits the mechanisms next, **the fail-high stop
        first** (a switch). The registered fallback should not run in its
        form, which keeps that stop at its seeds.
      - Maintainer requirement: any fallback shows no harm at `10+0.1` as
        well.

      **Research 2026-10-23, `MORE_RESEARCH` (RAR-R18,
      `docs/experiments/RAR-R18.md`).** Every read so far sits on the time
      rule (2 Elo per 1% of thinking time at `3+0.03`; RAR-S98, RAR-S99):
      the seeds' −14.9 against −14.8 for their −7.4% of time; the fit's
      +0.6 (the paused SPRT, 2,345 pairs) against −5.0 for −2.5%; the
      `10+0.1` and four-thread reads within one SE. Pooled, the cluster
      changes decision quality by **+1.6 ± 4.2 Elo**: it moves time. A
      replay on the switch build (`TmFailHighTarget`, engine `f3549ff`, a
      tune-only spin with play unchanged by default; bench 13 unchanged)
      shows the fail-high stop ending 29% of first-move searches at a
      median 1.18 × the target and never past 3 ×, removing about 10% of
      the spend: single fail-highs, not cascades. At twice the target it
      fires on under 1% and the spend is the no-stop level. The stability
      factor adds about 12%. Three 2,000-game categorical reads on that
      build (`stopoff`, `cascade` at 2 ×, `factorsoff` with the stop off on
      both arms) and a probe at 2 ×, frozen with the time rule as their
      null, decide the kept set: the horizon, the mate stop and the guard
      stay; the stop moves to the threshold the reads choose; the stability
      factor and the cross-move slope stay only if they decide better than
      their time. The kept set then reads as one repair (AGENTS' case 2) at
      `3+0.03` and `10+0.1` before anything lands. The research recommends
      recording RAR-R16's SPRT as unresolved, and against Reckless's shape
      or a respending budget now.
      **RAR-R18 read 2026-10-23** (2,000 games each, probe PASS): removing
      the stop **+15.5 ± 9.3** against the seeds, the stop at twice the
      target **+12.2 ± 8.8**, the factors **+6.9 ± 8.7** for keeping them.
      The stop at the target costs about 15 Elo at `3+0.03` and leaves its
      saving on the clock; at twice the target the cost is gone and the
      cascades still end (rec1 at `180000+2000` in 18.9 s against the
      head's 104 s). The time rule misread the reads: between two arms of
      one clock the whole-game time is conserved, so the rule saw
      redistributions as decisions (calibration in RAR-R18). Frozen
      dispositions: the stop to twice the target (landed on `d1` as
      `ec416d4`, its test precondition `7a344f5`; bench 13 unchanged); the
      stability factor and cross-move slope removed as insufficient; the kept set to one repair
      read at both controls. Additive over the reads the kept set is
      −9.6 ± 15.2 against the head without the factors and −2.7 ± 12.4
      with them, so their place in the repair arm is the maintainer's
      decision, recorded there.
      **Maintainer decision 2026-10-23: the correct fix, in the search.**
      The cascade is a search property (full-depth re-searches after a root
      fail-high) and is fixed there, by **D.1.6**, not capped by the clock:
      the fail-high stop is retired (off by default, `a0f9755`; the spin stays
      as D.1.6's control arm and goes with it). The other six mechanisms
      stay on `d1` at their seeds: the horizon, the mate stop and the guard
      as clock answers to clock problems; the stability factor, the effort
      band over the whole search and the cross-move trend on their lean
      (+6.9 ± 8.7, both donors), the fit's block 1 unused since it was
      fitted around the retired stop. **This leaf is now D.1's closing
      read, after D.1.6 lands:** `d1`'s head against the 2.6.0 engine plus
      the cascade fix, registered then under AGENTS' repair case 2 with no
      Elo claim: 2,000 games at `3+0.03` and 2,000 at `10+0.1`, harm rule
      −13 nElo on each; 1,000 games at four threads as a direction read;
      the probe and the profile reads that tick D.1.2.1 (rec2 under 2 s,
      the won and mate shares below 0.2%) and D.1.2.2 (no time loss, the
      clock floor). A pass lands the clock pieces in 2.7.0 as a
      correctness change; a fail lands none of D.1's clock work. RAR-R16's
      SPRT ended unresolved at 2,345 pairs (+0.6 ± 5.8 Elo, LLR −0.06):
      its arm, the fit over the stop at the target, no longer exists.
    - **D.1.6 The root fail-high cascade fixed in the search: depth-reduced re-searches, from D.1's investigation — `R3` research, `I2`/`V` sub-steps.**
      Rarog re-searches every root fail-high at full depth, so a cascade of
      k fail-highs costs k full iterations: rec1 ran nine re-searches to the
      hard maximum (32 s at `60000+600`, 104 s at `180000+2000`; RAR-R13),
      rec3 to about four times its target. Both donors make the re-searches
      cheaper instead of stopping them: Stockfish searches each re-search at
      the root depth less the count of fail-highs so far, and stops
      deepening once half the target is spent; Reckless re-searches one ply
      shallower. D.1.3's clock stop capped the symptom and cost about 15 Elo
      at `3+0.03` at the target (RAR-R18): time spent on fail-highs is well
      spent here, which is the evidence that makes this a gate, not a
      probe. This is not the aspiration window's growth, which RAR-S89 and
      RAR-S17 refused; it is a different device, untested here.
      Interactions: the aspiration constants were fitted in C.10 around
      full-depth re-searches and may owe a refit; the clock's factors read
      iteration completion; the TT carries a shallower re-search's results
      into the next iteration. It moves `bench 13`, so the fingerprint is
      re-declared when it lands, and it changes shared search signals in
      the rated configuration, so its gate is the symmetric `[-5,5]` with
      its cap (AGENTS' repair case 3; about 12,000 games) on the 2.6.0
      engine, with no clock stop in either arm. Acceptance adds the probe
      with no clock stop: rec1 and rec3 under the hard maximum at both
      clocks. Maintainer decision 2026-10-23: the correct fix is wanted
      ahead of NNUE, at its price. If the gate refutes it, the fallback is
      the clock stop at twice the target behind its spin, read as RAR-R18
      read it.
        - **D.1.6.1 Research: the cascade's mechanism in Rarog's loop, the donors' devices, the cheapest prototype and its zero-game reads, the registration — `R3`, DONE 2026-10-23.**
          The research card (mechanism, interactions, invariants, falsifier)
          per AGENTS, from the diag traces of rec1 and rec3 (RAR-R13's
          method) and the activation replay (29% of first-move searches
          fail high past the target). A prototype behind a tune switch (the
          re-search's depth reduction as a coordinate, 0 = today) is allowed
          for the zero-game reads: the probe with the clock stop off, the
          bench and counter differential against the head
          (`tools/diag/bench_counters.py`, `phase4_differential.py`) and the
          fail-high rate per iteration. Ends `READY_FOR_IMPLEMENTATION` with
          the device, its seeds, the gate's registration and frozen
          prediction, or `NO_CHANGE` with the evidence.
          **Read 2026-10-23, `MORE_RESEARCH` (RAR-R19,
          `docs/experiments/RAR-R19.md`).** On the 2.6.0 engine every root
          fail-high returns exactly its beta, so a cascade's length is set
          by the window schedule alone (deltas 12, 27, 52, 93, 161, 273,
          458, 763 …; re-centring on the failing score would save at most
          one re-search) and each re-search costs more than the last: rec1's
          depth 19 runs 7 fail-highs and 75 M nodes without completing,
          rec3's depth 21 five and 18.5 M. The device is one ply off a root
          window search per fail-high so far in the iteration, the exact
          window included, floor depth 1, reset each iteration, the nominal
          depth reported and sampled (`AspFailHighReduce`, tune-only,
          default 0, engine `027a604`; bench at 0 unchanged). Reads: bench
          13 at 1 is 8,510,754 (0.61 ×; the fail-high count unchanged, the
          fail-high nodes −45%, and the exact window after fail-highs,
          searched k plies shallower, −49%); rec1's depth 19 completes in
          1 M nodes; the idle-host probe with no clock stop ends rec3 at
          0.48 × its optimum at both clocks where the 2.6.0 engine runs it to
          the hard maximum (30.3 s, 97.9 s), rec1 at 0.83 × at `180000+2000`
          (head 2.51 ×) and 2.70 × at `60000+600` (head 0.91 ×, its cascade
          completing just past the target); at `3000+30` +0.68 nominal ply
          for +6.9% first-move spend, fail-highs an iteration unchanged.
          **The same probe on the PGO arm (`2d013c8`, 1.3 × faster) runs
          rec3 into the hard maximum at `180000+2000`:** it crosses the
          target one iteration later and depth 24's cascade costs more than
          the 90 s left. The frozen stop rule fires and the leaf stays in
          research. Why: a cascade's window count is the schedule's and
          unchanged (7, 7, 4, 4); in these endings a ply of depth saves about
          20% of a window, not the bench's 2.5 × (27 → 21 → 18 → 15 M nodes
          over four plies); and the next iteration's first window starts at
          full depth over a TT holding only the shallower result. The
          depth reduction is the search half of both donors' device; the
          other half is clock-coupled in both (Stockfish stops deepening at
          half the target and recovers depth over the next iterations).
          **Maintainer decision 2026-10-23:** complete the device, then gate
          it; the gate is not run on the partial device. **Node-indexed
          reads at 200 M nodes (RAR-R19):** with Stockfish's recovery as a
          second switch (`AspFailHighRecover`, engine commit named in
          RAR-R19) the costliest iteration is still over 150 M in both
          positions; with the window opened after three or four fail-highs
          (`AspMaxFails` 3, 4, alone and with the reduction) one open window
          costs 70–190 M. **Every root-loop device is refuted as the fix.**
          Depth for depth Rarog spends 7–57 × Stockfish 19's nodes on these
          positions: its root score rises with every ply (rec3 706 → 1149,
          rec1 799 → 2882) where Stockfish's is flat (484–573), and its
          lines reach the ply cap (seldepth 89 at depth 18 against 26). The
          cascade is the loop following a drifting score through a tree that
          grows to the cap; the cause is upstream of the root loop and is
          the re-scoped question: the static evaluation along the PV against
          the search score by depth, the extension and quiescence
          composition at stride 1, and Stockfish's classical evaluation in
          Rarog's search on these positions (RAR-O07's instrument). The gate
          (D.1.6.3) stays held; the research recommends the stop at twice
          the target (RAR-R18) as the clock's robustness piece meanwhile.
          **Maintainer decision 2026-10-23: the stop stays off; the
          research continues.** **Read 2026-10-23 (RAR-R19, zero games):**
          the oracle (Stockfish's search over Rarog's frozen evaluation)
          reaches depth 26 on rec1 in 3.1 M nodes with a flat score where
          Rarog spends 19 M on depth 18 and its score doubles, so the drift
          and the cost are the search, not the evaluation. In the cascade
          iteration a third of all reduced moves are searched deeper than
          their nominal depth (`reduced_depth`'s negative reductions, up to
          two plies above `new_depth` plus a PV node's two, outside the line
          budget; diag counters `d428c00`); zeroing the correction-history
          term alone shrinks rec1's depth 18 from 19 M to 4 M nodes,
          selective depth 89 to 39, score 1845 to 1184, while every other
          term moves the tree three to a hundred times at the ply scale.
          A tune-only cap on the deeper searches (`CoreLmrDeeperCap`, 4 =
          today, `ddbe1f2`) at 0 shrinks the
          fixed-depth trees three to six times and halves bench 13
          (6,639,253), but the explosion returns seven plies deeper when the
          search proves the win (rec3 depth 27: 100 M nodes). **Read one
          level further (RAR-R19, maintainer decision to continue):** the
          Rarog-against-oracle counter differential is 3.4 × on the
          standard suite and on these endings at depth 14 but 20.6 × at
          depth 18; the oracle to depth 34 proves rec1's win for the same
          order of nodes (100 M to score 2880 against Rarog's 154 M) in
          iterations that grow 1.3 × a ply, where Rarog's jump 26–245 × at
          the iteration that first resolves the win. A tune-only switch that
          charges deeper reduced searches to the line budget
          (`CoreLmrDeeperBudget`, `0366884`) restores the stated invariant
          (selective depth 127 → 52), reproduces the oracle's depth-18 score
          on rec1 at 7 × its nodes instead of 51 ×, and costs 30% of bench,
          but the lump remains: nine to eleven fail-high windows walking the
          schedule from 888 to 2818 in one iteration, because the root
          returns exactly beta on every fail-high (fail-hard in effect; the
          five fail-high interpolations toward beta are cleared, all at 0
          the root still returns the schedule's beta) where the oracle's
          fail-soft root and re-centring window climb the same range in one
          or two windows an iteration over eight iterations. **Read
          2026-10-23 (RAR-R19, maintainer decision to continue):** the
          fail-hardness is quiescence returning `alpha` on a fail-low (93%
          of bench's root fail-highs are exactly beta; ply-2 move-loop
          cutoffs two-thirds on the edge). Fail-soft quiescence
          (`QsFailSoft`) and re-centring (`AspRecentre`), the donors' two
          halves, together still walk eight or nine windows in the
          resolving iteration (the root's overshoot does not reach it;
          bench 1.32 ×, fail-soft alone 1.50 ×, re-centring alone 0.74 ×
          as a faster schedule). The line budget's size
          (`CoreExtensionBudgetEighths`) at half smooths rec3's score path
          (steps of 130 cp, largest iteration 86 M) but not rec1's, which
          finds its win later and still in one iteration (1274 → 7000,
          thirteen windows, over 180 M), at bench 0.50 ×. **Every
          single-device read is exhausted:** the stall is a won ending
          whose conversion is one forced line, seen all at once by the
          first iteration deep enough, and the window walking that jump in
          eight to thirteen bound-proofs. Open: a fit of the fail-high
          schedule under the bounded tree; the per-ply differential of the
          oracle's gradual score path against Rarog's step; the
          clock-coupled half. Strength candidates of their own for a
          `[-5,5]` gate: `CoreLmrDeeperBudget` (an invariant restored,
          bench 0.70 ×) and fail-soft quiescence (bench 1.50 ×). All
          switches on `d1` at their neutral defaults. **Read 2026-10-23
          (RAR-R19, maintainer decision to continue; the connected features
          against the donors, and the HCE):** the HCE values a won ending
          flat (K+R+B against K eight pawns; Stockfish 11's classical
          evaluation 55, a known win), so the search carries the win's value
          across the horizon and the score climbs only as deep lines reach
          the promotion or the mate; with Stockfish 11's own evaluation the
          oracle resolves rec1 at depth 21 for 0.7 M nodes. A known-win
          offset alone (`mopup_known_win`, an evaluation parameter, 0)
          makes rec1 worse (cascades of 7–16 windows, bench 1.29 ×) because
          the first window is 12 wide whatever the score; **with the
          donors' score-scaled first window** (`AspDeltaSquareDiv`,
          `ff14793`) **the cascade is gone:** every iteration closes in one
          or two windows and the score climbs gradually, the oracle's shape,
          at bench 0.73 ×. What remains is one window's cost at depth 20 and
          beyond in these endings (rec3's depth 21: a single exact window
          over 168 M nodes, 100 × the oracle's iteration), the tree's
          composition (less pruning per node, twice the singular attempts,
          effective depth twice nominal), a search cluster of its own. The
          candidate cluster for D.1.6.3: the known-win offset with the
          score-scaled window and the line budget, after a fit of the
          aspiration and deeper-search constants around them, gated `[-5,5]`
          with the probe; the evaluation half needs the maintainer's leave
          under the freeze. The clock-coupled half stays recommended for
          D.1.5 meanwhile. **Maintainer decision 2026-10-23: the whole
          fix, every part, now, verified.** The evaluation half is allowed;
          the fourth part is Stockfish's deepening freeze past half the
          target (`TmDepthFreeze`, `097ea5d`). The probe with no clock stop
          (RAR-R20): parts 1–3 end every probed stall under the maximum
          (rec3 at `180000+2000` 30.8 s against 98.0 s), all four end every
          move within its optimum (the PGO arm: rec3 2.5 s and 6.9 s, rec1
          2.3 s and 8.3 s, against the head's 30.3, 98.0, 4.4 and 37.5 s).
          **Done 2026-10-23, `READY_FOR_IMPLEMENTATION`:** the cluster, its
          arm `6fd195e` (bench 8,996,837), its probe, its wire check and its
          gate are RAR-R20's; the research recommends a fit of the
          aspiration and reduction constants around the cluster before the
          gate.
        - **D.1.6.2 Implementation: the device in the root loop, its tests, the fingerprint re-declared — `I2`.**
          `READY_FOR_IMPLEMENTATION` 2026-10-23. The cluster under gate is
          the arm `6fd195e` (recipe
          `analysis/arm_patches/6fd195e-rar-r20-d16fix4.patch`, bench
          8,996,837): the four switches of RAR-R20 baked on as the defaults
          (`mopup_known_win` 6000, `AspDeltaSquareDiv` 10193,
          `CoreLmrDeeperBudget` 1, `TmDepthFreeze` 5000), version
          `2.7.0-dev+d16fix4`. On H1 the landing on `d1` is: those four
          defaults (after the fit, if it ran, its values), the fingerprint
          re-declared in GUIDE and the PLAN checkpoint, `TmFailHighTarget`
          and its test deleted, the instruments the cluster does not use
          removed (`AspFailHighReduce`, `AspFailHighRecover`, `AspRecentre`,
          `QsFailSoft`, `CoreLmrDeeperCap`, `CoreExtensionBudgetEighths`,
          the arm `2d013c8` and its binary), the MultiPV loop given the same
          `Aspiration` form, and the user-facing changelog entry (a fixed
          defect: the engine no longer runs a won ending into its hard
          maximum). On H0 nothing lands and the fit is the next step.
        - **D.1.6.3 Gate: SPRT `[-5,5]` at `3+0.03` against the 2.6.0 engine, cap 20,000 pairs, the probe without a clock stop, a `10+0.1` direction read — `V`.**
          The first registration (RAR-R19, the reduction arm `2d013c8`,
          seed 20261040) is withdrawn: its arm was a partial device and its
          probe reached the maximum. **Registered 2026-10-23 as RAR-R20**,
          before any game: `rarog-d16fix4-pext-pgo.exe` (`6fd195e`, bench
          8,996,837, the four parts) against `rarog-d15head-pext-pgo.exe`
          (`468ed7b`, 13,987,223), Colosseum `sprt-repair`, seed 20261042,
          run directory `rar-r20-gate`; then `match-fixed-ltc` 1,000 games
          (seed 20261043) and the probe on both binaries;
          `run_r20.ps1 -Read wire|probe|gate|ltc`. Prediction, frozen,
          unfitted: −6 ± 8 Elo, H1 0.30, cap 0.25, H0 0.45; the research
          recommends the registered fit before the gate. **Verified
          2026-10-23 (RAR-R20):** the PGO arm ends seven of eight probed
          moves within its optimum and rec3 at `180000+2000` still reaches
          the maximum (one window at depth 24, the search-cluster half);
          the 200-game wire check reads −52.5 ± 23.9 unfitted, while at
          equal speed every part reads within noise of even and the PGO
          loss carries 5% less time a move and 5% lower in-game speed.
          **The road to the gate is RAR-R21, staged:** three 2,000-game
          equal-speed reads (the four parts, parts 1–3, the freeze alone)
          decide whether SPSA block 1 over the `d16fix` surface (fifteen
          coordinates, 60,000 games, the fit arm `217125b`) runs at all;
          the gate is re-registered on an arm rebuilt with the fitted
          values. The maintainer's commands are `run_r21.ps1 -Read
          all4|nofreeze|freeze|fit`.
          H1 lands it (harm near −5 excluded); H0 refutes it and the
          fallback is read. The fail-high stop's spin is deleted when the
          fix lands.
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
- **E.3 Release 2.7.0 through the tag-driven flow; 3.0.0 is reserved for the first NNUE release — `M`/`V`.** Version, changelog, release notes, fmt, debug and
  release suites, clippy, feature builds, fingerprint, PGO assets, ISA
  verification, CI matrix, tag and publish on maintainer instruction. Version
  is 2.7.0 whatever E.2 reads (maintainer decision 2026-10-22: Rarog 3 is
  the first NNUE Rarog, F.9; E.2's result is recorded, not a version number;
  2.5.0 was cut at B.10 and 2.6.0 from the C head ahead of E, maintainer
  decisions 2026-10-03 and 2026-10-22). Later classical gains (E.4, F.2.1)
  release as 2.8.0 and onward under Phase F's rule. The release is cut through the
  tag-driven flow of **E.3.1**, which also carries the two workflow checks
  this release owed (tag equals version; one fingerprint asserted across the
  matrix) and may land any time earlier.
    - **E.3.0 Originality disclosure before the release cut: the donor material named, the licence settled, the notices shipped with the binaries — `R2`, then `M`.** A
      small research step with its implementation. Added 2026-10-23 by
      maintainer decision. E.3.2 does not cut until it closes. Known
      evidence (`analysis/originality_review_2.6.0_2026-10-22.md`, on the
      2.6.0 source):
      - The code is Rarog's own. Raw shared token runs with Reckless cover
        0.34% of Rarog, against 96% between two Reckless versions.
      - The evaluation weights are Rarog's own fits, with no Stockfish 11
        table exact or scaled.
      - Two groups of donor material were copied unchanged:
        - Stockfish's time budget: about 17 constants in
          `src/search/time.rs`, which RAR-R14 and RAR-R15 kept as
          "Rarog's Stockfish-shaped budget".
        - Reckless's move ordering, history, correction and TT-cutoff
          updates: 37 of the 50 hard-coded search constants of 150 or
          more, with their control flow, adopted at B.2.
      - The evaluation design is Stockfish 11's.
      - Fathom (MIT) does the Syzygy probing and is not named.
      - The README says only "inspiration", and PROCESS's independence
        boundary says constants cross as refitted seeds. Both are out of
        step with the shipped code.
      - The maintainer's move-similarity screen (`tools/diag/sim_test.py`,
        2,000 positions at 100 ms, 2026-10-23) puts Rarog at 56.4%
        against Stockfish 11, 57.7% against Stockfish 10 and 54.2%
        against Reckless. Its row runs 50.6–57.7 across nine engines.
        The related controls read Stockfish 10–11 67.0% and Houdini 6–Fire
        68.8%. Rarog plays like an unrelated engine against both donors.
      - Venue rules:
        - CCRL, CEGT and SPCC do not judge originality.
        - TCEC sanctioned code copied *covertly* (Houdini and Fire, 2020).
        - ICGA rule 2 (2019) requires every code and data source to be
          named at registration and voids undeclared derivatives.

      The step:
      1. Rerun `tools/diag/originality_scan.py`,
         `originality_constants.py` and `originality_eval.py` on the
         release candidate's source, plus the screen if search or
         evaluation changed since 2.6.0, so that the disclosure describes
         2.7.0, not 2.6.0.
      2. Settle the licence with the maintainer. Reckless is AGPL-3.0,
         and the closely ported blocks are where a derivative-work
         argument could be made. The options:
         - credit Reckless and ask its authors (the maintainer sends);
         - relicense Rarog as AGPL-3.0-or-later;
         - rewrite and refit those blocks.

         The maintainer chooses; it is not legal advice.
      3. Write the disclosure where each fact belongs:
         - README's Acknowledgements, short and for users, naming
           Stockfish 11's evaluation design, Reckless's search, history
           and move ordering with some constants unchanged, Stockfish's
           time budget, and Fathom;
         - PROCESS's independence boundary, saying what crossed unchanged
           and that unchanged material is disclosed;
         - the review, updated for the release head, which also serves as
           the donor list for an ICGA-style registration.
      4. Ship the GPL text and Fathom's MIT notice with the release
         binaries (`release.yml` publishes bare executables today), with
         a `CHANGELOG.md` entry if the assets change.

      Falsifier and stop rule: a rerun that finds no group beyond the two
      above leaves the disclosure as drafted. A new group is added to it,
      not removed by tuning here. Refitting the copied constants is out of
      scope: that is a strength experiment that needs its own registration
      and gate. Closes when the README, PROCESS, the licence decision, the
      shipped notices and the updated review have landed.
    - **E.3.1 Tag-driven release flow — `I1`, DONE 2026-10-05.** `release.yml`
      and `cargo xtask release-check`; 2.5.0 was released through it, nine
      assets at 11,171,726 (run 37284706939). The procedure is
      `docs/PROCESS.md` *Release*; the leaf's text is in `docs/archive/PLAN-closed-2026-10-05.md`.
    - **E.3.2 Release cut: version 2.7.0, the `[Unreleased]` changelog reviewed and dated, suites, the release PR squash-merged, the `v` tag pushed on instruction through E.3.1's workflow — `M`.** E.3's own work, a leaf of its own since
      E.3.1 closed (2026-10-05): the version from `2.7.0-dev` to 2.7.0,
      behaviour-neutral with the fingerprint
      held; `CHANGELOG.md`'s `[Unreleased]`, kept as changes landed,
      reviewed and dated; fmt, clippy, debug and release suites, feature
      builds; GUIDE's checkpoint marks the release; the PR to `master`,
      squash-merged once `CI` and `Release` are green; the
      `vX.Y.Z` tag pushed on instruction through E.3.1's workflow, by
      PROCESS *Release*.
- **E.4 The king-danger index rebuilt in the donor's structure; the classical line's next release if it wins — `R2` card, then `I2`/`V`.** The
  one hand-crafted lead Phase C left (added 2026-10-22, maintainer
  decision: classical work resumes only after Phases D and E). Evidence:
  RAR-E72 read the donor's king signal in the middlegame band as
  conditioning, +0.28% static over the comparator; RAR-E73 refitted our 23
  danger coordinates from the donor's seeds and lost held-out loss, closing
  the lead as the index's structure, not its weights; RAR-E21 put 92% of the
  king residual in the danger map, its safe checks and weak ring foremost.
  Independent of Phase F's data, so it runs after E.3's release and before
  or beside F.0. Research card first: which structural difference carries
  the signal, the cheapest static falsifier, the unit's handoff. Then the
  unit in Rarog's own structure, fitted on `hce-v4-tb` by C.3's pipeline and
  gated by PROCESS's *Evaluation change under a fitted search* (gate 1
  `[0,3]`, the margin block if flagged, gate 2), with RAR-O07's same-search
  meter re-read at the close. Prior from Phase C's units: +10 to +25 Elo,
  frozen with its band at the registration. Accepted, it is released as the
  classical line's next version through E.3.1's flow and becomes the
  classical reference of Phase F's rule; `NO_CHANGE` closes the family work
  and leaves F.2.1 as the classical programme's last experiment.
## Phase F — NNUE (own data only)

**Rules.** Own data only, generated by Rarog's classical head and later by its
NNUE heads. Reckless is the runtime and trainer-pipeline donor; the
architecture ladder is ours. The classical evaluation stays in the tree as the
datagen baseline and the fallback until F.9 replaces it in releases.

**The classical evaluator in releases** (maintainer's rule, 2026-10-22; the
read and the reference are this document's proposal until a release applies
them): the classical evaluation ships as a selectable evaluator only while it
is the strongest classical Rarog measured. Each release whose search changed
reads its classical arm against the strongest classical release in a
2,000-game STC match; no measured loss keeps the option, a measured gain
makes that release the classical reference, and a loss whose interval
excludes zero drops the option from that release, so the classical line ends
at the last release that carried it, which stays published. The classical
arm gets no tuning of its own beyond one per-evaluator overlay of the
scale-sensitive search coordinates (margins, the lazy threshold, correction
scale, quiescence thresholds), fixed when F.6 refits the search; a search
change that strands that overlay is expected to end the line. One binary
with an evaluator option, never two release builds.

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
    - **F.2.1 Classical refit on search-visited data: the frozen HCE's whole surface refitted on F.2's dataset, gated as an evaluation change; the classical line's release if it wins — `V`.** The
      lever Phase C's review left (`analysis/phase_c_review_2026-10-19.md`,
      question 3; added 2026-10-22 by maintainer decision): the same-search
      meter's remaining 99 Elo (RAR-O07) is not in the families, which read
      at static parity with the donor on played positions (RAR-E70), but in
      how the evaluation scores the positions the search visits. F.2's data
      is those positions with deep labels, and `tools/texel` (C.2's
      protocol) reads it at no new instrument cost. The whole surface, E.4's
      king unit included, is refitted on F.2's data, screened statically on
      `hce-v4-tb`'s held-out rows and the frozen test, then gated by
      PROCESS's *Evaluation change under a fitted search* against the
      strongest classical release, with RAR-O07's meter re-read at the
      close. Prediction and band frozen at the registration; a fit that
      loses the screen or the gate closes `NO_CHANGE` and ends the classical
      programme, a win is released as the classical line's next version
      and becomes the reference of this phase's rule.
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
- **F.9 NNUE release, 3.0.0: beat the classical release at STC, LTC and 4T; platform matrix — `M`/`V`.** Beat the classical release at STC, LTC
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
- 3.0.0 is the first NNUE release (F.9; maintainer decision 2026-10-22,
  replacing "3.0.0 requires the E.2 gate met": the classical stage releases
  as 2.7.0 and onward, and E.2's result is recorded, not a version); 2.4.0 requires at least +40 Elo at STC over
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
