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
| Fingerprint | `bench 13` **14,749,043 / EBF 2.564** at engine source `3a16fbb` (D.5.15: a draw verdict takes no correction; before it 13,987,223 / EBF 2.565 at `9fb525e`, C.10's search tune on the frozen evaluation; `rustc 1.99.1`) | RAR-S105, RAR-P36 |

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
   step" in every reading (`tools/spsa_theta_audit.py --movement-config`), and a run
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
     the cluster's SPSA. Flat means inside the sweep's noise floor, read
     before the next sweep from the same readings at the seed and one
     unit either side of it on each registered coordinate, where no real
     curvature can show: any change moves the tree, so without a floor
     no line reads flat (Basilisk's three sweeps: 0 of 15; maintainer
     decision 2026-11-02).
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
| D.3.10 | RESEARCH | I1 | Cause unread; read with D.3's tests, before the cut |
| D.2 | RESEARCH | R2 | **Night read 2026-11-03:** the slow state is Rarog's, not the host's (Rarog's four threads 2.62–2.66 times one thread for 9.5 hours, Reckless's 3.36–3.48 beside it); RAR-M81 UNRESOLVED (in that state Rarog's D 0.82 against 1.28 and 1.31), RAR-M80 NOT REPLICATED; parking, frequency and scheduling excluded. Next, zero-game on an idle host after Basilisk's tune: Rarog's four threads one to a physical core against both siblings in a rebuilt slot, Reckless the control. Earlier reads: the D.2 text below |
| D.4.1 | READY_FOR_IMPLEMENTATION | I1 | After D.3, in order of value: the table handle and the helper lifecycle first |
| D.4.2 | READY_FOR_IMPLEMENTATION | I1 | After D.3, in order of value; exact fingerprint, pooled NPS where hot |
| D.4.3 | READY_FOR_IMPLEMENTATION | I1 | After D.3, in order of value; exact fingerprint, pooled NPS where hot |
| D.4.4 | READY_FOR_IMPLEMENTATION | I1 | After D.3, in order of value; exact fingerprint, pooled NPS where hot |
| D.4.5 | READY_FOR_IMPLEMENTATION | I1 | After D.3, in order of value; exact fingerprint, pooled NPS where hot |
| D.4.6 | READY_FOR_IMPLEMENTATION | I1 | After D.3, in order of value; exact fingerprint, pooled NPS where hot |
| D.4.7 | READY_FOR_IMPLEMENTATION | I1 | After D.3, in order of value; exact fingerprint, pooled NPS where hot |
| D.4.8 | READY_FOR_IMPLEMENTATION | I1 | After D.3, in order of value; exact fingerprint, pooled NPS where hot |
| D.4.9 | READY_FOR_IMPLEMENTATION | I1 | After D.3, in order of value; exact fingerprint, pooled NPS where hot |
| D.4.10 | READY_FOR_IMPLEMENTATION | I1 | After D.3, in order of value; exact fingerprint, pooled NPS where hot |
| D.4.11 | READY_FOR_IMPLEMENTATION | I1 | After D.3, in order of value; exact fingerprint, pooled NPS where hot |
| D.4.12 | READY_FOR_IMPLEMENTATION | I1 | After D.3, in order of value; exact fingerprint, pooled NPS where hot |
| D.4.13 | READY_FOR_IMPLEMENTATION | I1 | After D.3, in order of value; exact fingerprint, pooled NPS where hot |
| D.4.14 | READY_FOR_IMPLEMENTATION | I1 | After D.3, in order of value; exact fingerprint, pooled NPS where hot |
| D.4.15 | READY_FOR_IMPLEMENTATION | I1 | After D.3, in order of value; exact fingerprint, pooled NPS where hot |
| D.4.16 | READY_FOR_IMPLEMENTATION | I1 | After D.3, in order of value; exact fingerprint, pooled NPS where hot |
| D.4.17 | READY_FOR_IMPLEMENTATION | I1 | After D.3, in order of value; exact fingerprint, pooled NPS where hot |
| D.4.18 | READY_FOR_IMPLEMENTATION | I1 | After D.3, in order of value; exact fingerprint, pooled NPS where hot |
| D.4.19 | READY_FOR_IMPLEMENTATION | I1 | Read 2026-11-02: the KPK verdicts agree with Syzygy and land as a test; won KPK and `krpkr_scale`'s wrong zeros went to D.5.21 and D.5.22; the KQKP, KRPKB and KBPK zeros unread |
| D.4.20 | READY_FOR_IMPLEMENTATION | I1 | Tooling; after D.3 |
| D.4.21 | READY_FOR_IMPLEMENTATION | I1 | After D.3; the behaviour-test sweep |
| D.5.1 | RESEARCH | R2 | Screen first among the cards (castling's classification) |
| D.5.2 | RESEARCH | R2 | Screen second (quiet queen promotions in quiescence) |
| D.5.3 | RESEARCH | R2 | Screen before any implementation; its gate queued behind D.2 |
| D.5.4 | RESEARCH | R2 | Screen before any implementation; its gate queued behind D.2 |
| D.5.5 | RESEARCH | R2 | Screen before any implementation; its gate queued behind D.2 |
| D.5.6 | RESEARCH | R2 | Screen before any implementation; its gate queued behind D.2 |
| D.5.7 | RESEARCH | R2 | Screen before any implementation; its gate queued behind D.2 |
| D.5.8 | RESEARCH | R2 | Screen before any implementation; its gate queued behind D.2 |
| D.5.9 | RESEARCH | R2 | Screen before any implementation; its gate queued behind D.2 |
| D.5.10 | RESEARCH | R2 | Screen before any implementation; its gate queued behind D.2 |
| D.5.11 | RESEARCH | R2 | Screen before any implementation; its gate queued behind D.2 |
| D.5.12 | RESEARCH | R2 | Screen before any implementation; its gate queued behind D.2 |
| D.5.13 | RESEARCH | R2 | Screen before any implementation; its gate queued behind D.2 |
| D.5.14 | RESEARCH | R2 | Screen before any implementation; its gate queued behind D.2 |
| D.5.16 | RESEARCH | R2 | Screen before any implementation; its gate queued behind D.2 |
| D.5.17 | RESEARCH | R2 | Screen before any implementation; its gate queued behind D.2 |
| D.5.18 | RESEARCH | R2 | Screen before any implementation; its gate queued behind D.2 |
| D.5.19 | RESEARCH | R2 | Screen before any implementation; its gate queued behind D.2 |
| D.5.20 | RESEARCH | R2 | Screen before any implementation; its gate queued behind D.2 |
| D.5.21 | RESEARCH | R2 | Screen: does a one- or two-ply search already read the 88 above 0; minutes |
| D.5.23 | RESEARCH | R2 | Screen: raw against corrected at the non-verdict leaves near the verdicts in the 63 fixture searches; minutes |
| D.5.22 | RESEARCH | R2 | Screen: a per-site scale that keeps the draws near 0 and the wins above it; minutes |
| D.3.9 | RESEARCH | R2 | Research cards; the `src/uci/` (planned) move with D.6.13 |
| D.8 | READY_FOR_IMPLEMENTATION | I1 | Linux transparent huge pages for the page allocations, behind `LargePages` (macOS stays Apple silicon only, by maintainer decision 2026-11-02); stands on its own: RAR-M79 (2026-11-02) read no large-page cost in a healthy host state, and their part in the slow state is unread (audit 2026-11-02); exact fingerprint, a Linux NPS read |
| D.6.1 | READY_FOR_IMPLEMENTATION | I1 | Can start now |
| D.6.2 | READY_FOR_IMPLEMENTATION | I1 | Can start now |
| D.6.3 | READY_FOR_IMPLEMENTATION | I1 | Can start now |
| D.6.4 | READY_FOR_IMPLEMENTATION | I1 | Own branch; exact fingerprint on both slider backends |
| D.6.5 | READY_FOR_IMPLEMENTATION | I1 | Own branch; exact fingerprint on both slider backends |
| D.6.6 | READY_FOR_IMPLEMENTATION | I1 | Own branch; exact fingerprint on both slider backends |
| D.6.7 | READY_FOR_IMPLEMENTATION | I1 | Own branch; exact fingerprint on both slider backends |
| D.6.8 | READY_FOR_IMPLEMENTATION | I1 | Own branch; exact fingerprint on both slider backends |
| D.6.9 | READY_FOR_IMPLEMENTATION | I1 | Own branch; exact fingerprint on both slider backends |
| D.6.10 | READY_FOR_IMPLEMENTATION | I1 | Own branch; exact fingerprint on both slider backends |
| D.6.11 | READY_FOR_IMPLEMENTATION | I1 | Own branch; exact fingerprint on both slider backends |
| D.6.12 | READY_FOR_IMPLEMENTATION | I1 | After D.2 closes |
| D.6.13 | READY_FOR_IMPLEMENTATION | I1 | Inside D.3 |
| D.6.14 | RESEARCH | R2 | After D.2 closes; `R2` first on how aspiration and MultiPV share the driver |
| D.6.15 | READY_FOR_IMPLEMENTATION | I1 | Own branch; exact fingerprint on both slider backends |
| D.6.16 | READY_FOR_IMPLEMENTATION | I1 | Own branch; exact fingerprint on both slider backends |
| D.6.17 | READY_FOR_IMPLEMENTATION | I1 | Own branch; exact fingerprint on both slider backends |
| D.6.18 | READY_FOR_IMPLEMENTATION | I1 | Own branch; exact fingerprint on both slider backends |
| D.6.19 | READY_FOR_IMPLEMENTATION | I1 | Own branch; exact fingerprint on both slider backends |
| D.6.20 | READY_FOR_IMPLEMENTATION | I1 | Its census can start now |
| D.7 | RESEARCH | R2 | Tablebase policy, with the review's probe-path items (PROTO-9 to 15, DRIVER-15, NODE-11, TT-11) and the cached empty `SyzygyPath` |

## Phase D — Clock, threads, robustness

Numbering: on 2026-11-01 the tablebase leaf D.4 became D.7, and D.4, D.5 and
D.6 were added from `analysis/engine_review_2026-11-01.md`; a record written
before that date that names D.4 means the tablebase leaf (HISTORY's number
map). On 2026-11-03 the numbered items of D.3 to D.6
became sub-steps with their numbers kept, so that the board shows each one:
item N of D.x is D.x.N (D.3's eight defects are D.3.1 to D.3.8; D.3.9 and
D.3.10 are new).

- **D.1 Time management: audit against the ADR-0065 checklist, soft/hard bounds with node-fraction multiplier, forfeit margin; SPRT `[0,3]` — `R2` investigation, `I2`/`V` sub-steps, DONE 2026-10-26.** **Maintainer decision 2026-10-26: closed, the residue accepted.** The stalls are solved for the vast majority and the rest is not harmful: the stop at twice the target ends every probed stall, and `d1`'s game residue is a third of 2.6.0's at `10+0.1` with no measured cost (RAR-R28 and its residue addendum). The single-window fix is held with its retry trigger, a measured cost. Audit the
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
    - **D.1.2 Time-management audit and bound model: the ADR-0065 checklist, soft and hard bounds with the node-fraction multiplier, the forfeit margin, the won-ending stalls (RAR-R13); SPRT `[0,3]` — `R2`, DONE 2026-10-26.**
      Closed 2026-10-26 with D.1.2.1, D.1.2.2 and D.1.2.3 done.
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
        - **D.1.2.1 Won-ending time stalls: rec1's cascade, rec2's missing stop after a proved mate, rec3's overrun (RAR-R13) — `R2`, DONE 2026-10-26.**
          **Maintainer decision 2026-10-26: closed, the residue accepted.** The stalls are solved for the vast majority and the rest is not harmful: the stop at twice the target ends every probed stall, and `d1`'s game residue is a third of 2.6.0's at `10+0.1` with no measured cost (RAR-R28 and its residue addendum). The single-window fix is held with its retry trigger, a measured cost.
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
        - **D.1.2.2 The forfeit margin against OS holds — `R2`, DONE 2026-10-26.**
          **Ticked 2026-10-26 by RAR-R28:**
          - no candidate time loss in 5,000 games;
          - the clock-floor median 429 ms at `3+0.03`;
          - no game under 100 ms, against the 2.6.0 engine's 680 of
            2,000.
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
    - **D.1.5 The clock gate, D.1's closing read after D.1.6: the clock pieces at their seeds against the 2.6.0 engine with the cascade fix; harm reads at `3+0.03` and `10+0.1`, a four-thread read, the probe and the profile — `V`, DONE 2026-10-26.**
      **Done 2026-10-26 (RAR-R28): no harm.**
      - `d1`'s head against the 2.6.0 engine read +1.2 ± 8.9 Elo at
        `3+0.03` and −0.2 ± 7.8 at `10+0.1` (2,000 games each), and
        −8.0 ± 11.3 at four threads. 0 faults, and 3–5% less time a
        move.
      - The probe passes, with rec2 in 5 ms where 2.6.0 runs to the
        maximum.
      - D.1's clock work is kept for 2.7.0. D.1.2.2 ticks.
      - D.1.2.1 misses its won share at ≥ 0.9 × the maximum (0.24%,
        0.85%; every game won) and returns to research.

      The history follows.
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
      **The clock pieces alone, registered 2026-10-24 as RAR-R24**, before
      any game:
      - **Why:** RAR-R22's addendum found that this read had never been
        run, and that `d1`'s clock reaches the hard maximum in about twice
        as many `3+0.03` games as 2.6.0's. Every D.1.6 arm carried that
        clock.
      - **The arms:** `rarog-d1head-pext-pgo.exe` (`3f4dcf4`, the `d1`
        defaults, bench 13,987,223) against `rarog-d15head-pext-pgo.exe`
        (`468ed7b`).
      - **The reads:** 2,000 games at `3+0.03` and 1,000 at `10+0.1`, harm
        rules −13 and −18 nElo, and `stall_scan2.py`'s maximum-hitting rate
        per side.
      - **Predictions:** 0 ± 14 and −3 ± 16 nElo.
      - **Command and order:** `run_r24.ps1 -Read stc|ltc|stalls`, dry-run
        clean. It informs this leaf and does not close it; the closing read
        above stays. Both arms are fixed binaries, so either order against
        RAR-R23 attributes cleanly; the maintainer ran this one first.
      **Read 2026-10-24, no harm at either control** (RAR-R24):
      - `3+0.03`: +8.0 ± 8.9 Elo (+13.7 ± 15.2 nElo).
      - `10+0.1`: +14.3 ± 11.2 Elo (+27.3 ± 21.5 nElo).
      - 0 faults and 0 time losses. d1head spends 3–4% less a move.
      - **Stalls:** d1head runs to the maximum in 38.0% of `3+0.03` games
        against 2.6.0's 20.5%, early and at balanced scores. That
        reproduces RAR-R22's addendum between PGO builds, with no measured
        cost.
      - **Disposition:** the clock pieces stay on `d1` at their seeds.
        RAR-R22's −7.4 is not the clock's. A harm read accepts no gain, so
        this leaf's closing read after D.1.6 stands.
      **The closing read, registered 2026-10-26 as RAR-R28**, before any
      game:
      - **Arms:** `d1`'s head `rarog-d1head3-pext-pgo.exe` (`6fcfac2`,
        bench 13,987,223: the clock pieces at their seeds and the stop at
        twice the target) against the 2.6.0 engine.
      - **Reads:** 2,000 games at `3+0.03` and 2,000 at `10+0.1` with −13
        nElo harm rules; 1,000 games at four threads as a direction read;
        the probe on rec1–rec3; `tm_profile_d15.py` at horizon 50.
      - **The ticks**, as D.1.2.1 and D.1.2.2 state them.
      - **Predictions:** +12 ± 14 and +20 ± 16 nElo; four threads −2 ± 15
        Elo.
      - **Command:** `run_r28.ps1 -Read probe|stc|ltc|4t|profile`, all
        three matches dry-run clean; maintainer-run.
    - **D.1.6 The root fail-high cascade in won endings: researched in the search, capped by the stop at twice the target — `R3` research, `I2`/`V` sub-steps, DONE 2026-10-26.**
      **Closed 2026-10-26.**
      - **The stalls end:** the fail-high stop at twice the soft target is
        `d1`'s default (`a79efe4`). It read no harm against off (RAR-R25:
        +4.3 ± 8.3 Elo at `3+0.03`, +4.9 ± 9.8 at `10+0.1`), and on the
        PGO build it ends every probed stall under the maximum (RAR-R27).
      - **No repair in the search or evaluation lands:** RAR-R22 H0,
        RAR-R23 harm at `10+0.1`, RAR-R27 H0 at −1.2 ± 3.0.
      - **The cause is recorded, not fixed:** an uncapped LMR correction
        term drives the won-ending tree's growth and most deeper searches
        everywhere (RAR-R26).
      - **Cleanup:** the research instruments are removed (`42b001f`),
        and the fingerprint stays 13,987,223.
      - **Retry trigger:** an LMR refit with the correction slope free.

      The history follows.
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
      **Where it stands (2026-10-24).** The four-part cluster (RAR-R19,
      RAR-R20), fitted by RAR-R21 (block 1: the freeze later, the clock's
      scale up 8%, the squared divisor up 44%, nothing else moved), ends
      every probed stall (eight of eight rows) but its gate accepted H0:
      −7.4 ± 7.5 Elo at `3+0.03`, +2.1 ± 11.6 at `10+0.1` (RAR-R22). What
      is known about the cost: the evaluation half reads even at equal
      speed at every size; the in-game NPS loss is the tree's composition,
      not speed; the arm still spends 5.5% less a move after the fit; parts
      1–3 at equal time read −9.7 ± 8.9. **The level above (2026-10-24,
      RAR-R22's addendum):** d1's clock runs to the hard maximum in 41% of
      `3+0.03` games before ply 40, double the 2.6.0 head's 15–18%, so the
      gate's −7.4 sums the ungated clock pieces (D.1.5) and the cluster;
      the window term and the budget charge are global changes (32 of 40
      fresh-process bench positions move), the offset is confined to
      bare-king trees (13 of 40, all 19 pieces or fewer); a clock stop
      conditioned on the chosen move (Basilisk's) would fire on 24% of
      searches here and is rejected. **The re-scoped plan:** the confined
      cluster {`MopupKnownWin`, a known-win window floor
      `AspKnownWinWindow` active only when the centre is in the known-win
      band}. It has no clock piece and no budget charge. Its evidence was
      to be the probe and a fresh-process differential for correctness, a
      2,000-game equal-speed harm read on one tune build, and the `10+0.1`
      read (AGENTS' repair cases 1 and 2). D.1.5's harm read of the clock
      pieces alone against 2.6.0 is recommended first.
      **The floor refuted, zero games (2026-10-24, RAR-R23):**
      - The floor (`3f4dcf4`, tune-only, 0 = off) changes nothing measured:
        0 of 40 fresh-process bench positions, 0 of 2 won endings, and
        every probe node count at 1000, 2000 and 3000.
      - Its premise was a jump of about 5,800 cp as the offset enters the
        horizon. What rec1 shows instead is a 12 cp window walking a score
        that climbs from 750 to 3,314 cp, below the band, five to fifteen
        windows an iteration. Only the global window term shortens that
        walk.
      - The offset alone keeps every probe row under the maximum and
        changes 13 of 40 positions, all with 19 pieces or fewer.
      - So **the repair registered is the offset alone** (RAR-R23), with
        the floor at 0. The window walk below the band is a search
        question outside D.1.6.
      - D.1.5's harm read of the clock pieces alone is registered as
        RAR-R24.
      **Both read 2026-10-24.**
      - **RAR-R24, no harm:** d1's clock against 2.6.0 read +8.0 ± 8.9 and
        +14.3 ± 11.2 Elo, so the clock is not RAR-R22's cost.
      - **RAR-R23 fails:** the offset alone reads −4.4 ± 15.2 nElo at
        `3+0.03` and **−25.0 ± 21.5 at `10+0.1`**, below the −18 harm
        threshold, with the wire live and time a move equal. Nothing lands,
        and the leaf is back in RESEARCH.
      - **Maintainer decisions 2026-10-25:**
        - Research the cause rather than close `NO_CHANGE`.
        - The fallback stop at twice the target is open to re-evaluation if
          it does not cost strength. It is a separate read, a cap and not a
          fix; the card below is the fix.
        - That read is registered as **RAR-R25**: one tune build, the stop
          at twice the target against off, the probe, 2,000 games at
          `3+0.03` and 1,000 at `10+0.1`, harm rules −13 and −18 nElo.
          Both matches dry-ran clean. A pass makes it the default as an
          interim cap, re-read when the search fix lands.
        - **RAR-R25 read 2026-10-25: PASS.**
          - The probe ended rec1 and rec3 at `180000+2000` in 19.7 s and
            15.7 s, against 43.2 s and the hard maximum off.
          - `3+0.03` read +4.3 ± 8.3 Elo (+8.0 nElo), and `10+0.1` +4.9 ±
            9.8 (+10.7 nElo). 0 faults.
          - The landing is the default `TmFailHighTarget` 20000 in its own
            engine commit, with the changelog entry, at an unchanged
            fingerprint. It is not yet made.
          - D.1.2.1's cascade half is met by the probe; its rec2 half stays
            with D.1.5's proved-mate stop.

      The fit arm's lesson stands: a refused SPSA step and a 2-second idle
      window are preflight matters, not results.
      **Research card (2026-10-25): the won-ending tree, not the window or
      the clock.**
      - **Mechanism.** In rec1 Rarog's tree grows far faster per ply than a
        search that converts the win:
        - Rarog at 200 M nodes reaches depth 16–20 and proves no mate
          (RAR-R23's traces).
        - Basilisk's current head, with its clock stop removed, proves mate
          in 25 at depth 26 in 73 M nodes. That is 0.56 × its maximum at
          `60000+600` and 0.20 × at `180000+2000`; its 2026-10-24 probe is
          in `D:\code\basilisk\tools\results\stall-probe-20261024\`.
        - Stockfish's search over Rarog's evaluation reaches depth 34 for
          about the same nodes (RAR-R19).
        - The window walk RAR-R23 found is the symptom: the root score keeps
          rising because each deeper tree finds more, and a 12 cp window
          chases it.
      - **The measured drivers** (RAR-R19, rec1 at depth 18, 19.1 M nodes
        and selective depth 89 at the defaults):
        - with LMR's correction term at 0: 4.1 M nodes, selective depth 39;
        - with no reduced move searched deeper than its nominal depth:
          3.2 M and 40;
        - with at most 2 plies deeper: 2.7 M and 51.
        - On bench, 443,080 of 4,216,025 reduced searches (10.5%) run
          deeper than nominal, by 1.7 plies on average, outside the line
          budget. The cap at 0 halves bench and the cap at 2 takes 21% off
          it, so the deeper ceiling is a global setting. Neither has been
          played in games.
      - **A difference from Basilisk, read in both sources.**
        - Basilisk caps its LMR correction term at 2,695, about 2.6 plies,
          "the donor's largest value of the term"
          (`search_kernel_core.cpp`).
        - Rarog's term is `lmr_correction * |correction| / 1024` with no
          cap (`late_move_reduction`, `src/search/node.rs`): about 0.33 ply
          a 100 cp of correction at the default 3330.
        - So a large correction turns a reduction negative, and the reduced
          move is searched up to 4 plies deeper, past the line budget.
          Basilisk allows 2 plus 1 at a PV node.
      - **Competing hypotheses.**
        - **H1:** the uncapped correction term in LMR drives the won-ending
          explosion. A cap would be confined to nodes with a large
          correction.
        - **H2:** the deeper-search ceiling (+4 against Basilisk's +2/+3)
          drives it, and is a global change.
        - **H3:** the window walk is a cause of its own. Weakened, because
          it shrinks wherever the tree does.
        - **H4:** the evaluation's flatness. Refuted as a net fix by
          RAR-R23.
      - **Interactions.**
        - C.10 fitted the LMR terms and their correction slope around the
          uncapped term and the +4 ceiling.
        - Correction history also feeds the singular margins
          (`sing_double_corr`, `sing_triple_corr`) and the static
          evaluation.
        - The line budget's invariant is stated in source and broken by the
          deeper plies (`CoreLmrDeeperBudget` restores it).
        - The aspiration constants were fitted around the present tree.
      - **Invariants.**
        - A cap leaves every node with a correction under it unchanged,
          checked by a fresh-process differential.
        - The fingerprint moves and is re-declared on landing.
        - No clock change is part of this card.
      - **Cheapest discriminating tests** (zero games; they need an idle
        host and wait for Basilisk's runs to finish):
        1. A diag histogram of the correction term's size at LMR nodes, on
           bench and on rec1/rec3. It says whether large terms are rare in
           ordinary positions, so whether a cap is confined.
        2. A tune-only cap on the term (0 = today), and node traces of rec1
           and rec3 at 200 M nodes for a few cap values, with and without
           the deeper ceiling at 2. Does rec1 reach a proved mate, as
           Basilisk's does?
        3. A fresh differential over bench for the chosen setting.
      - **Falsifier and stop rule.**
        - If no cap value ends rec1 and rec3 under the maximum in the probe
          without the window term, H1 is false. H2 is then read the same
          way.
        - If what ends them changes more than half of the bench positions,
          it is a global search change. Its read is an equal-speed
          2,000-game harm read first, and then the `[-5,5]` gate.
        - The line stops when that read is below −13 nElo or no setting
          ends the stalls.
      - **Why it could pay for itself.** If the escape is general, as 10.5%
        of bench's reduced searches suggests, a fix that keeps lines inside
        their budget could gain where the stalls themselves cost nothing.
        Only games can say.
      - **Read 2026-10-25, zero games** (instruments `c4da889`; evidence in
        `analysis/artifacts/d16-cascade-2026-10-23/tree-growth-2026-10-25/`,
        `RESULTS.md` against the frozen `PREDICTIONS.md`):
        - **The term at reductions.** It takes a ply or more off 19.2% of
          bench's reductions and 73.5% of rec1's. It drives 56.8% and
          91.4% of their deeper searches.
        - **Cap 1024** takes rec1 from depth 20 to 26 and rec3 from 21 to
          24 inside 200 M nodes. With the term at 0 they reach 35 and 28.
          Caps of 2048 and 3072 do not help, and neither does the deeper
          ceiling at 2.
        - **But cap 1024 changes 24 of 40 bench positions**, 32-piece
          openings among them (bench 11,480,295).
        - **No configuration proves a mate in the budget.** Each ends in
          the window walk as the win resolves, which the stop at twice the
          target now caps in play.
        - **Reading:** H1 is supported as the driver, but the term drives
          ordinary positions too, so the working cap is a global search
          change. H2 is refuted. Predictions 2 and 3 held; 1 and 4 missed,
          in mechanism.
        - **By the stop rule,** the cap is read at equal speed first:
          **RAR-R26**, registered and dry-run clean.
      - **RAR-R26 read 2026-10-25: no harm.**
        - `3+0.03` −1.4 ± 9.0 Elo, `10+0.1` +1.7 ± 11.5, 0 faults. The cap
          searched 0.09 and 0.24 ply deeper at equal time.
        - **The timed probe, run before any gate:**
          - The cap alone ends rec3's single long window at `180000+2000`
            (5.9 s, against the maximum uncapped). That is the case the
            stop cannot reach.
          - But it takes rec1 into the window walk at the win's resolution
            sooner, and that walk runs to the maximum without a stop.
          - With the stop at twice the target, every row ends at or under
            its optimum: rec1 0.78 × and rec3 0.42 × at `180000+2000`,
            against 1.30 × and 1.10 × for the stop alone.
        - **So the fix is the pair:** the cap for the tree's growth, the
          stop for the walk once the win resolves.
        - **Next:** the gate, a PGO arm with the cap baked against `d1`'s
          head (which has the stop), `[-5,5]` with its cap, the probe on
          the pair. Registered before any game; maintainer-run overnight.
        - **Registered 2026-10-25 as RAR-R27:**
          - **Arm:** `7703768` on the throwaway branch `d16-gate3`
            (recipe `analysis/arm_patches/7703768-rar-r27-d16gate3.patch`,
            `rarog-d16gate3-pext-pgo.exe`, bench 11,480,295).
          - **Head:** `rarog-d1head2-pext-pgo.exe` at `6253027`, bench
            13,987,223.
          - **Reads:** sprt-repair at `3+0.03`, cap 20,000 pairs, seed
            20261111; 1,000 games at `10+0.1`; the probe on both binaries.
          - **Prediction:** −1 ± 6 Elo.
          - **Dispositions:** H1 with the probe passing lands the cap and
            the cleanup; H0 closes D.1.6 on the stop.
        - **RAR-R27 read 2026-10-26: H0**, −1.2 ± 3.0 Elo; `10+0.1` +1.7 ±
          11.4; the probe PASS on both binaries. The cap does not land.
          D.1.6 closes on the stop at twice the target after D.1.6.2's
          cleanup. Retry trigger: an LMR refit with the correction slope
          free.
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
        - **D.1.6.2 Implementation: the cleanup, the unused D.1.6 instruments removed and the stop at twice the target kept — `I2`, DONE 2026-10-26.**
          **Done 2026-10-26 (`42b001f`):**
          - The 13 instrument commits from `027a604` to `c4da889` are
            reverted, `a79efe4` excepted: 680 lines removed, the stop
            untouched.
          - Bench is 13,987,223 / EBF 2.565 as before.
          - fmt is clean, clippy with all features and targets has 0
            warnings, and the tests pass (413 debug, 414 release, 0
            failed; the six removed were the instruments' own).
          - The `d16fix` SPSA group stays valid: `arms.txt` binds it to
            its own arm.
          **Re-scoped 2026-10-26 (RAR-R27, H0):** no repair lands. The step
          removes the D.1.6 instruments no default uses, as D.1.6.2's
          register row lists them. A coordinate whose default is today's
          behaviour returns to its constant, `TmFailHighTarget` stays at
          twice the target, and the fingerprint stays 13,987,223. It is
          verified as a behaviour-neutral change by that fingerprint plus
          the stop's test. The history follows.
          **Re-scoped 2026-10-24 to the offset alone (RAR-R23).** On its
          pass the landing on `d1` is:
          - `MopupKnownWin` 5826 as the default;
          - the fingerprint re-declared in GUIDE and the PLAN checkpoint:
            16,175,922 / EBF 2.580 on the tune build, with the landed
            production build's own bench recorded then;
          - `TmFailHighTarget` and its test deleted;
          - the unused instruments removed: `AspFailHighReduce`,
            `AspFailHighRecover`, `AspRecentre`, `QsFailSoft`,
            `CoreLmrDeeperCap`, `CoreExtensionBudgetEighths`,
            `AspDeltaSquareDiv`, `CoreLmrDeeperBudget`, `TmDepthFreeze` and
            `AspKnownWinWindow`. A coordinate whose default is today's
            behaviour returns to its constant;
          - the user-facing changelog entry: a fixed defect, the engine no
            longer runs a won ending against a bare king into its time
            limit.

          On a fail nothing lands; RAR-R23's dispositions name the two
          closes. The history follows.
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
        - **D.1.6.3 Gate: the repairs' reads and gates, the probe and the differential — `V`, DONE 2026-10-26.**
          **RAR-R27 read 2026-10-26: H0.**
          - The LMR correction cap at one ply against `d1`'s head, both with
            the stop: −1.2 ± 3.0 Elo (−2.2 ± 5.3 nElo) at 8,334 pairs, LLR
            −2.99; `10+0.1` +1.7 ± 11.4; 0 faults.
          - The probe passed on both binaries; on the PGO head the stop
            alone ends every row (rec1 at `180000+2000` 1.39 × its
            optimum).
          - The cap does not land, and D.1.6 closes on the stop after
            D.1.6.2's cleanup.
          **Registered 2026-10-24 as RAR-R23**, before any game:
          - **The read:** the offset alone (`MopupKnownWin=5826`) against
            the defaults on one tune build, `rarog-d16inst3-tune.exe`
            (`3f4dcf4`, bench 13,987,223), Colosseum `match
            -CategoricalTuneBuild`. 2,000 games at `3+0.03` (seed
            20261071) and 1,000 at `10+0.1` (seed 20261072).
          - **The harm rules:** −13 nElo at `3+0.03` and −18 nElo at
            `10+0.1`, AGENTS' repair case 2 with the maintainer's longer
            control.
          - **The correctness reads, done:** the probe has no row at the
            maximum; the differential confines the change to positions of
            19 pieces or fewer whose trees reach a bare king.
          - **Predictions:** −2 ± 10 nElo at `3+0.03` and 0 ± 14 at
            `10+0.1`.
          - **Command:** `run_r23.ps1 -Read stc|ltc`, dry-run clean.
          - **Read 2026-10-24, FAIL:** −2.3 ± 7.7 Elo (−4.4 nElo) at
            `3+0.03` passes; −11.1 ± 9.6 Elo (**−25.0 ± 21.5 nElo**) at
            `10+0.1` is below −18, which is harm. 0 faults. The `stc` run
            was interrupted at 79 games and resumed by the same command.
            Nothing lands.

          The earlier gate's history follows.
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
          fix lands. **Run 2026-10-24 as RAR-R22** on the arm `584c02d`
          (the fifteen fitted values and the budget baked, bench
          9,013,917): the probe eight of eight within the optimum, the
          wire check faultless, the SPRT **H0 at 1,377 pairs, −7.4 ± 7.5**,
          the `10+0.1` read +2.1 ± 11.6. Nothing lands; D.1.6 returns to
          research (above), and this gate is re-registered on the
          surviving cluster.
- **D.2 Lazy SMP quality at 4T/8T: diversity, shared TT and correction, soft-stop voting; 4T SPRT `[0,5]`; RAR-M78 read Rarog deficient at four threads (effective speedup 1.65 against Reckless 2.30 and Stockfish 2.48) — `R2` investigation, `I2`/`V` sub-steps.** 4T and
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
  **Target measured, RAR-P46, 2026-10-27.** For the C head, RAR-M46's
  contradiction no longer holds. Its own 4T/1T bench scaling is 3.28 against
  the B.9 head's 3.80, lost already at 2T. Under the games' conditions (three
  concurrent pinned 4T processes, 100 ms searches) it alone enters a
  system-level slow state, 7–10% under B.9 instead of 23% over. **RAR-P47,
  2026-10-27:** the `LargePages` option (`2b7bca9`) turns large pages off; the
  slow state appeared only with them on (2 of 16 passes against 0 of 16), weak
  evidence for the allocator. Off costs about 1–4% NPS in these cells.
  **RAR-M77, 2026-10-28:** in games under RAR-M74's four-thread conditions,
  large pages on are +2.7% NPS and +10.3 ± 5.9 Elo over off, so the default
  stays on. The slow state does not show in games: a match keeps its engine
  processes. The same head reads 8.08 M nps against itself but read 6.47 M
  against RAR-M74's field, so that −9.2% is a question about the opponents
  sharing the slot, not about large pages. Next for the agent: the 2T loss in
  shared writes; the self-relative scaling games come after.
  **RAR-P48 registered 2026-10-29.** A static read finds nothing threads
  share that changed since B.9; the speed pass that made C's nodes 22%
  cheaper was measured at one thread only. The read separates the loss
  into hardware (independent 1T processes on the same cores), the shared
  search, a search split across the two CCDs, and large pages, at 5 s
  with one thread per physical core. Its outcome picks D.2's next step.
  **RAR-P48 read 2026-10-29: Mixed.** Pinned one thread per physical
  core, the C head's E at 4T is 0.90 against B.9's 1.07, so the loss is
  real and is not RAR-P46's placement. It is not the CCD split, and not
  large pages. The split between hardware and shared search is
  unresolved: the instrument pooled two positions, and it read NPS from
  an iteration line that ends 2.3 to 5.0 s into the search. Per
  position, both meet the shared-search rule, and B.9's threads are
  faster sharing a search than apart while C's are not. Next for the
  agent: repair the instrument (per position, one fixed window) and
  register the read that decides it.
  **RAR-P49 registered 2026-10-29.** It reads every search's nodes at
  exactly 5,000 ms, interpolated between its info lines, judges each
  position separately, and keeps B.9 and the C head with large pages
  off. Shared search or both sends D.2 to a bisect of the speed pass at
  1T and 4T; hardware or not reproduced sends it to the scaling games.
  **RAR-P49 read 2026-10-30: shared search.** At 4T the two heads lose
  exactly the same to the shared hardware (H 0.956 against 0.956), and
  C's whole loss is in the shared search (Q 0.98 against 1.06, and 0.94
  against 1.00). Next for the agent: register a bisect from B.9 to C on
  this instrument, read on Q. Its first read asks whether the speed
  pass's parent already shows the loss; within the speed pass the
  prefetch sites go first.
  **RAR-P50 registered 2026-10-30.** A long-control match first was
  considered and set aside: RAR-M76 played C against B.9's search at 4T
  at `3+0.03`, where the loss does not exist, and a match long enough to
  see it would take days. The bisect reads five revisions, B.9,
  `52cc6de`, `6ade0e6`, `138d867` and the C head, each built with rustc
  1.99.1, and names the interval that holds at least half the loss in Q.
  **RAR-P50 read 2026-10-30: Mixed.** At the tightly measured position
  the loss splits between C's evaluation rebuilds (about 0.05 in ln Q)
  and the speed-pass squash (about 0.03). At the noisier one it
  accumulates over every step. No single change holds it, and both
  holders bought large gains, so the bisect stops; a mechanism study by
  shared-table counters waits for a hypothesis that predicts which rate
  differs. The engine target RAR-P46 named stays recorded and unfixed:
  about 6–7% per thread at 4T in searches of seconds, unseen in any
  match. Next for D.2: the self-relative scaling games, the maintainer's
  run.
  **RAR-M78 registered 2026-10-30.** Rarog, Reckless 0.9.0 and Stockfish 18
  each play themselves at four threads against one and, on one thread, at
  `20+0.2` against `10+0.1`. A raw four-thread gain favours a weaker
  engine, which gains more from any extra search. So each gain is divided
  by the engine's own Elo per doubling of time, giving an effective speedup
  comparable across engines (maintainer decision 2026-10-30). Rarog
  deficient opens a four-thread SMP research card; otherwise D.2's
  four-thread target stays closed and scaling beyond four threads stays
  with G.1.
  **Engine review input, 2026-11-01** (`analysis/engine_review_2026-11-01.md`,
  §3.3 and §3.4; the reviewers' reports and the profile logs are in
  `analysis/artifacts/engine-review-2026-11-01/`). The static read of what
  threads share is complete: on the node path only the transposition table and
  one 64-byte counter line are shared; histories, correction tables, the pawn
  and imbalance caches, the evaluator, the stack and the root records are per
  thread, every atomic is `Relaxed`, and the batched node counter with the
  stop flag costs an estimated 0.2–0.4% at four threads and table coherence
  1–2%, none of it the 6–7% loss. Three mechanisms can make a cheaper node
  share worse with no shared-memory change, and the review ranks their reads
  by cost. Every prediction below is frozen here, before any read.
  1. **The PGO profile covers one of three kernel copies.** `negamax`,
     `quiescence`, `check_stop` and `search_root_window` are generic over the
     poll closure, so the binary holds one copy of each for the engine's
     serial poll (`Engine::search`), one for `search_parallel`'s main thread
     (`main_poll`) and one for the helpers (`helper_poll`), about 15% of the
     binary; the training run sends `bench {depth}` at one thread
     (`xtask/src/main.rs:1156-1176`) and reaches only the first. In a replica
     of the recipe the eleven multi-thread instantiations had all-zero profile
     counters and were placed in `.text$unlikely`, the untrained
     `negamax<NonPv>` at 6,680 assembly lines against 15,568 for the trained
     copy; one Threads-2 training pass made every copy hot. Every thread of a
     `Threads > 1` search, the main one included, runs an untrained copy, and
     RAR-P49's Q compares exactly that against the trained copy in its 1T and
     4×1T cells, so any PGO-sensitive speed-up (the speed pass was tuned under
     PGO) lowers Q with no shared cause: the shape of RAR-P50's loss, spread
     over unrelated commits. Read: build the head with the poll taken as `&mut
     dyn FnMut() -> SearchEvent` at `search_impl`, `search_parallel` and
     `run_worker_job` (one copy, covered by the serial profile; the poll runs
     once per 2,048 nodes), or with a Threads-2 run added to the training, one
     cure at a time; `bench 13` must stay 13,987,223 / EBF 2.565; read Q at 4T
     and the 1T cell on RAR-P50's instrument against the current PGO build.
     Prediction: Q rises by 0.03 to 0.10 in ln with the 1T cell within 1%;
     under 0.02 kills it. A confirmed cure is a behaviour-neutral change
     qualified by the 4T NPS read, no game; it improves every shipped
     multi-thread asset and the 4T arm of every read since RAR-P46.
     Independent of RAR-M78, which prices four threads in Elo.
  2. **The shared backend's cost per operation, F16** (handed to D.2 by
     `analysis/architecture_review_2026-09.md` §4.6 and absent from this text
     until now). A shared probe is up to six (8-byte load, 2-byte load,
     `fold16`, compare) and a store scans up to six slots through `load_any`
     and `unpack_entry`; the local form does the same over three plain
     entries. Fitting RAR-P50's medians to a fixed per-node cost gives 47–61
     ns, about 12% of the head's node time, and a cheaper node makes a fixed
     cost a larger share, which is what the bisect's I2 step looks like. The
     1T fingerprint never sees the shared path, and 1T and 4T play different
     replacement geometry (3-way against 6-way). Read, no code change: `bench
     3` pooled at Threads 2 against Threads 1 (below `MIN_PARALLEL_DEPTH` the
     search stays serial but `configure` has made the table shared, so only
     the backend differs). Prediction: head ratio 0.87–0.91, B.9 0.90–0.93;
     refuted at 0.97 or above. Then, in cost order: the cure of item 1; a
     `replacement` that reads only occupancy, depth and age bits
     (behaviour-neutral); skipping byte-identical stores (item 4); and F16's
     answer, one atomic backend at every thread count, an `I2` with the 1T NPS
     pool at ±0.5% and gate case 3, because the geometry moves the
     fingerprint.
  3. **Q is an NPS ratio, and NPS mixes node cost with node mix.** A shared
     table turns searched nodes into cheap probe-and-return nodes, so Q above
     one is possible with nothing wasted (B.9: 1.051 at position 0) and Q can
     fall when the evaluation gets cheaper; RAR-R07/R08 showed the opposite
     failure of the proxy (3.95× NPS, lower depth). Read: RAR-R10's paired
     time-to-depth harness, 4T against 1T, B.9 and the head, zero games, about
     30 minutes. Prediction: the ratios differ by under 5%; six percent or
     more makes the loss real waste and keeps items 2 and 4 in scope.
  4. **The rest of the card.** The two pick-time prefetches (`1aaa39e`,
     `65f490e`) and the own-cluster prefetch are the only table-side change
     since B.9 (`tt.rs` is byte-identical to `24aefb4`) and fetch lines for
     moves then pruned, measured at one thread only: counters first
     (prefetches per searched move, predicted 1.6–2.2 against B.9's 1.0), then
     one arm per site on the instrument, Q +0.00 to +0.02 each kills the
     family. Byte-identical stores keep a line Modified and a probe miss
     stores twice: a counter (predicted 8–25%), compare before storing at 10%
     or more. `tb_hits` is a `fetch_add` per tablebase probe and sits with
     `stop_state`, `nodes` and the votes on one unpadded line: a thread-local
     flush on the 128-node boundary and `repr(align(64))`, fingerprint exact,
     a 5-man four-thread read as the activation read. Six slots of 16-bit tags
     have twice the local form's false-hit rate and 64 times Reckless's: a
     shadow full-key array in the instrument build measures it before any
     wider tag. Instrument rule: a four-thread rate study uses thread-local
     counters in `ThreadData` summed at join behind a feature; a `diag`
     build's process-global RMWs perturb what they measure and its NPS is
     never read. Donor divergences recorded so they are not rediscovered as
     fixes: the vote weight `+14` with a floor (donor `+10`), decisive results
     ranked by vote rather than by a win-or-loss rule, the quorum
     `floor(N/2)+1` without withdrawal (equal to the donor's at 2–4 and 6
     threads, different at 5, 7, 8 and 16), the ponder-move preference in the
     key, the SMP bundle of RAR-R03 (root rotation, pooled root scores, window
     re-centring) accepted whole and never decomposed, and correction history
     per thread where the donor shares it; each is priced, if at all, with
     RAR-M78's instrument under this leaf or G.1. The `Clone` derive and the
     helper handle the 2026-10-06 audit asked for are D.4's.
**RAR-M78 read 2026-11-01: Rarog deficient**, by the registered rule. Four
threads are worth 0.72 of a doubling of Rarog's own time (G4 +73.7 ± 11.5 over
G2 +101.8 ± 11.5 at `10+0.1`, effective speedup 1.65), against 1.20 (2.30) for
Reckless 0.9.0 and 1.31 (2.48) for Stockfish 18; Rarog's D minus theirs is
-0.48 ± 0.22 and -0.59 ± 0.23, both below zero. The runner's own statistics
say where to look: Rarog's four-thread side reached a lower mean depth than
its one-thread side (15.41 against 17.28) at 2.28× the nodes per second (the
bench read 3.28), where Reckless gained 1.3 plies at 3.45× and Stockfish 2.8
at 3.47×; and 28% of Rarog's four-thread moves were held over the 20 ms margin
outside its own clock (p99 175 ms) against 2% at one thread and under 1% for
the other engines. RAR-P50's recommendation against pursuing the four-thread
loss is withdrawn. **The four-thread SMP research card is open; its reads in
order:** (1) the PGO coverage of the multi-thread kernel copies, item 1 above,
one build (at four threads the main thread itself runs an untrained copy,
which a depth loss at higher NPS is consistent with); (2) time-to-depth four
threads against one on RAR-R10's harness, item 3 above; (3) the holds: where
the four-thread process spends the time between the clock's end and `bestmove`
(the join of every helper, `threads.rs:363–371`, the 1 ms wait loops, the
`info` and vote path), read from the games' `h=` comments and a timed `go
movetime` at Threads 1 and 4, then repaired as a clock-side defect under D.1's
rules; (4) the shared backend's cost, item 2 above; (5) the card's own
questions from the registration, what the helpers' nodes are worth: helper
depth, shared-table use and root diversity, with the donor divergences of item
4 priced on this instrument. Each read is registered with its prediction
before exposure; a repair that moves no fingerprint closes on the four-thread
NPS and time-to-depth reads, and anything else takes the leaf's 4T SPRT
`[0,5]`, whose premise this read restores.
  **The four-thread card, 2026-11-01** (research, zero games so far). The
  deficit is RAR-M78's: four threads are worth 0.72 doublings of Rarog's own
  time against 1.20 for Reckless and 1.31 for Stockfish, a gap of 0.48 to
  0.59 doublings. A speed-only mechanism is bounded: a gain of s in ln NPS is
  worth s / ln 2 doublings, so 7% buys 0.10. Established today, by file reads
  and builds:
  - **The kernel copies.** The head's PGO profile holds 17 kernel
    instantiations, the 11 multi-thread ones with all-zero counts, as the
    review found. `e057d84` takes the poll as a trait object: 6
    instantiations, all trained, `bench 13` exact. The table's shared-form
    branches inside the kernel stay untrained under any one-thread training.
  - **The holds are inside the clock.** `h=` is Colosseum's overhead after
    the last `info` line and lies within the charged time (RAR-R30), so
    RAR-M78's "outside its own clock" is corrected: no forfeit risk, but 9.0%
    of the four-thread time (1.2% at one thread) goes to search that is never
    reported. The code accounts for it: in a pool the soft stop is a vote, and
    the main thread past its target keeps iterating until three of four
    agree.
  - **The depth deficit is not the holds.** On moves held 20 ms or less the
    four-thread side still reports 15.25 plies against 17.30. Its main thread
    searched about a quarter of 1,392,558 nodes a move against the one-thread
    side's 622,252; at EBF 2.565 that costs 0.62 plies, not 1.87. About 1.25
    plies are unexplained by node count.

  Competing hypotheses, each with the shared signals it touches, its test and
  its falsifier:
  1. **H1, untrained kernel copies** raise every thread's node cost. Signals:
     none (code layout). Test: RAR-P52, Q at four threads. Falsifier: mean
     L(Q) under 0.02.
  2. **H2, the shared backend** (F16) costs each node, its branches cold in
     the shipped build. Signals: the table's geometry (six-way against
     three-way) and replacement. Test: RAR-P53, one binary, both backends,
     with and without PGO. Falsifier: R at 0.97 or above.
  3. **H3, the helpers slow the main thread's progress to depth.** Every
     thread searches the same depths (no skip; the old tables were inert,
     RAR-R10), same-depth helpers overwrite the main thread's entries, and the
     RAR-R03 bundle reorders and re-centres the main thread's root from the
     pool (`apply_shared_root_scores`, pooled root scores, window
     re-centring). Signals: table entries and replacement, root order,
     aspiration windows, and the best-move changes the clock reads. Test:
     RAR-R29's level X_ctl. Falsifier: X_ctl at 1.8 or above.
  4. **H4, the soft-stop vote** spends about 9% of four-thread time in a main
     iteration that never finishes. Signals: the soft target, the votes, the
     stop. Test: RAR-R30's timed read. Falsifier: NOT REPRODUCED, or JOIN
     (then a clock-side defect under D.1's rules).
  5. **H5, result selection.** The weighted vote (`+14` with a floor,
     decisive results by vote) can choose a helper's move, and the reported
     depth is then that helper's. Signals: the vote, the reported line, the
     ponder move. Test, not yet registered: the share of four-thread searches
     whose move differs from the main thread's, and their depths, counted in
     an instrument build. Falsifier: under 5% of moves differ.
  6. **H6, the table fills faster** at 2.28× the nodes. Signals: replacement.
     Test: RAR-R29's harness at Hash 1024, only if H3 survives. Falsifier: no
     change in X.

  **Interactions.** H1 and H2 multiply into per-thread NPS and share the cold
  shared branches, which a Threads-2 training run would cure together. H3 acts
  on depth per node and hides behind any NPS gain, so RAR-R29 reads levels as
  well as differences. H4 and H5 act on time and on the decision and are
  invisible to `go depth`. H2's answer, one atomic backend, would move the
  fingerprint and every table measurement H3 relies on, so it comes last.

  **The cheapest discriminating tests** are the four registered reads, about
  50 minutes of the maintainer's machine and no games: RAR-P52, RAR-R29,
  RAR-R30, RAR-P53. Then, by their outcome, H5's count and H3's ablations of
  the RAR-R03 members on RAR-R29's harness (tune-only switches), before any
  game.

  **Stop rule.** Each hypothesis stops at its falsifier. If H1 to H4 all fall
  and X_ctl is 1.8 or above, no zero-game read has found the deficit: the
  card registers H5's count and, if that falls too, closes `NO_CHANGE` with
  the deficit handed to G.1 rather than a 4T SPRT spent without a candidate.
  A repair that moves no fingerprint closes on the four-thread NPS and
  time-to-depth reads; anything else takes the leaf's 4T SPRT `[0,5]`, or a
  removal bracket for a RAR-R03 member.

  **Readiness: not `READY_FOR_IMPLEMENTATION`** for any playing change. The
  one change made, `e057d84`, is behaviour-neutral and was kept by RAR-P52.

  **The four reads, 2026-11-02** (zero games, the maintainer's runs):
  - **H1 holds, small.** RAR-P52: Q +0.031 in ln, in the band at its edge;
    four-thread NPS +2.4%, the one-thread cell about 0.9% slower, inside the
    registered 1%. `e057d84` stays and closes.
  - **H2 is refuted.** RAR-P53: the shared backend costs a one-thread node
    2.3% (R 0.977), and its cold PGO branches nothing (1.004). F16 leaves the
    card; cure (b), a Threads-2 PGO training run, is not registered.
  - **H3 holds, and it predates the C head.** RAR-R29: four threads reach a
    fixed depth more slowly than one, X 0.87 for the head and 0.96 for B.9,
    at about four times one thread's nodes. The main thread's own tree is no
    smaller than alone: the helpers buy it no depth. The registered
    comparison reads WASTE (d −0.099, 90% −0.235 to +0.014), about 0.06 of
    it the NPS ratio and 0.035 the nodes.
  - **H4 holds.** RAR-R30, VOTE: under the clock 25% of four-thread searches
    hold over 20 ms against 6% at one thread; the stop path is fast. A
    designed behaviour, priced only by games.
  - **The games' depth deficit is not reproduced.** At equal time X 0.87
    costs about 0.15 plies, against the games' 2.05 on unheld moves. The
    read searched depths 20 to 22 for about 2 s from a cleared table; the
    games search depths 15 to 17 in about 200 ms with the table kept.

  **Next, by RAR-R29's rule for X under 1.2:** what the helpers do to the
  main thread, each a zero-game read registered before exposure, ordered in
  that registration: RAR-R29's harness at the games' depth and time with the
  table kept across searches (it separates a short-search cost from a
  carried-table one); H3's ablations of the RAR-R03 members on the same
  harness; H5's count in an instrument build; H6 at Hash 1024, now in scope.
  H1 and H2 are closed and H4 waits for games. Still not ready for any
  playing change.

  **Registered 2026-11-02, the maintainer's, about two hours, no games:**
  - **RAR-R31,** the games replayed: twelve of RAR-M78's games searched again
    position by position with the games' clocks and the table kept, alone
    and under the games' conditions one at a time. It folds in H6 (a Hash
    1024 cell) and H5's count (searches that end on a line for a move no
    earlier line named), read from the output without an instrument build.
    Read from the games first: **in play four threads ran at 2.28 times one
    thread's NPS at every search length**, against 3.6 alone and pinned, and
    the same in every slot (H7, new: a condition of the games costs the
    four-thread side speed; Reckless keeps 3.44 under it).
  - **RAR-R32,** H3's screen: the pool's root order, the window centre, the
    helpers' rotation and the jitter offset switched off one at a time and
    together, on RAR-R29's harness.

  **Read 2026-11-02:**
  - **RAR-R31, NOT REPRODUCED.** Alone, with the games' positions, clocks
    and kept tables, four threads report the same depth as one (+0.08
    plies) at 3.78 times its NPS. On the same moves the games ran the
    four-thread side at about 60% of that speed and 1.38 plies shallower,
    the one-thread side at 91%. The slot's eight logical CPUs and three
    processes at once cost under 6%; H6 is refuted; H5 is active (+5.6
    points of closing lines) and held.
  - **RAR-R32, nothing resolves:** none of the four RAR-R03 members, nor all
    four together, holds the main thread back at about ±10%.

  **Compared with Reckless's code, 2026-11-02.** The same Lazy SMP skeleton
  (one shared table, every thread at every depth, a persistent blocking pool,
  the score-and-depth vote, a three-of-four soft stop), and six differences:
  Rarog's helper diversity (root rotation, pooled root order, window centre,
  jitter offset), which Reckless does without; per-thread correction history,
  where Reckless shares one; two table layouts by thread count against one;
  one node counter every thread adds to on the stop flag's cache line,
  against per-thread padded counters and a read-only stop line; a vote never
  withdrawn; and **large pages locked on Windows** for the table and every
  thread's big tables since `6ade0e6`, where Reckless allocates ordinarily on
  Windows and asks for transparent huge pages only on Linux. Rarog asks for
  none on Linux: D.8.

  **Read 2026-11-02 (the maintainer's run, 16:51 to 18:06):**
  - **RAR-M79, ABSENT:** in RAR-M78's own configuration (three games at
    once) the head's four threads ran at 3.44 times one thread in games,
    against RAR-M78's 2.48; `e057d84` covers 2.4% of that. Large pages off
    (3.53) and the B.9 head (3.28) carry no loss. **One game at a time**
    reads 2.69: on the same openings the four-thread side is 23% slower than
    with three games at once and the one-thread side 6% faster (an
    unregistered reading).
  - **RAR-R33, HITS WITHOUT SAVINGS:** at four threads the main thread's hit
    rate rises 5.6 points (90% +5.1 to +6.0) and its own probes do not fall
    (M 1.09, 90% 0.98 to 1.21): the helpers' entries reach it without
    cutting its tree. The next H3 read is the depth and bound of those hits
    against what the node needs.

  **H7, rewritten 2026-11-02 after RAR-M79.** RAR-M78's 2.48 was its
  sessions' host state, not a fixed condition of the games: the same
  configuration now reads 3.44. RAR-M78's threads leg is recorded as
  confounded and its effective speedup (1.65) is measured again before D.2
  reads anything from it. What stands is that the speed four threads keep in
  games depends on the host's state by up to a third, and that one game
  alone on the host, the nearest condition to a rating list that plays one
  game a machine, costs the four-thread side most. Whether Reckless loses
  the same way was never measured on one host state: its 3.44 in RAR-M78 was
  another session's. Next, each registered before exposure: one game at a
  time against three, Rarog and Reckless side by side, per-side in-game NPS;
  then Rarog's four-thread scaling measured again in the condition that read
  chooses. No engine change is in scope until then. RAR-R31's framing (40%
  lost in games, Reckless far less) stands as measured in RAR-M78's
  sessions.

  **Registered 2026-11-02, the maintainer's, about 80 minutes:**
  **RAR-M80**, one game at a time against three, Rarog and Reckless 0.9.0
  each against itself, one thread against four at `10+0.1`, 60 games a leg
  on the same openings and host state; per-side in-game NPS, no Elo read.
  Not replicated, Rarog's own, or shared decides the next read.
  **Basilisk cross-check input, 2026-11-02** (`analysis/basilisk_crosscheck_2026-11-02.md` §H, H26–H28), for H3
  and H5 once the condition is located: the root's first move is the shared
  table's (`node.rs:650-662`, as in Reckless; Stockfish 19 takes the
  thread's own previous best), so a helper's store at its own depth, or a
  fail-low root's least-bad move (D.5.4), can put a non-PV move first
  on the main thread; RAR-R32 switched off rotation, the pool order and the
  window centre but not this. Read, zero games: at four threads, the share
  of main-thread iterations whose first root move differs from that
  thread's last completed best, split fail-low and other; if large, one
  arm with the thread's own `root_moves[0]` first on RAR-R29's instrument.
  Stockfish 19 also shares its continuation, pawn and correction histories
  across threads through atomic entries, where Rarog keeps all of them per
  thread and Reckless shares correction only: a competing mechanism under
  H3, with RAR-R06's helper-history blend (−0.52) as the prior against.
  If a coherent table reopens (TT-9), Basilisk's costs at one thread are
  the prior: a `key16 ^ fold16` tag −3.91%, one packed 64-bit word with the
  move apart −1.22% [−1.72, −0.76] (BAS-C05).
  **Audit 2026-11-02** (`analysis/d2_audit_2026-11-02.md`, a re-read of
  every record and artifact above; nothing built, no game):
  - **The depth figures that steered the card are contaminated.** Every
    mean depth quoted from the runner (1.9 plies less at four threads,
    2.05 on unheld moves, Reckless +1.3, Stockfish +2.8, B.9 +0.30)
    averages mate-score moves in, where the head's four-thread side
    reports 7 to 10 plies and its one-thread side 23 to 27 (B.9 41 to 45
    on both sides, Reckless 62, Stockfish 125): a reporting difference in
    decided positions. Without them the four-thread side is 1.05 plies
    shallower in RAR-M78's games, 0.25 in RAR-M79's `on` leg (R 3.44),
    0.15 in `off`, 0.93 in `solo`, and B.9's is 0.51 shallower, not
    deeper; Reckless's is 0.72 deeper and Stockfish's 1.6. At R 3.44 the
    0.25 is RAR-R29's X 0.87 (0.15 plies) plus the shorter own time
    (0.12): on a healthy host no in-game depth deficit is unexplained.
    RAR-R30's "the holds do not explain the depth deficit", RAR-R31's
    premise and H5's motivation rest on the contaminated figure; each
    record carries its correction.
  - **RAR-M78's threads leg is confounded, as RAR-M79 found, and the
    deficit verdict falls with it:** "Rarog deficient" is not
    established, and the card's question is open again, not answered.
    Frozen prediction for a clean rerun at R about 3.4: G4 +122 ± 12
    (0.47 doublings more than at R 2.48, by the odds leg's 101.8 Elo a
    doubling), D 1.2, X 2.3, level with Reckless; G4 at about +85 or
    below reads deficient against Reckless again and makes the lead
    below the engine's own target.
  - **The host state is the blocker, and it is unidentified.**
    Four-thread processes alone lose a third of their speed, flat for
    hours, the head's so far and never a one-thread side: RAR-M74
    (2.60), RAR-P46 (5 of 12 passes), RAR-M78 (a whole leg), RAR-M79
    `solo` (2.69), against 3.44 to 3.8 when healthy. RAR-M79's `off` leg
    ran in the healthy state, so it cannot clear large pages for the
    slow one (D.8 corrected). In the slow state the main thread loses
    about twice the depth its NPS loss predicts (RAR-M78 1.05 plies
    against 0.6 expected, `solo` 0.93 against 0.5; RAR-R31's fidelity is
    +1.33 on non-mate moves), so the loss is not a uniform slowdown of
    every thread. Candidates that predict R near 2.5: two threads a
    physical core (the slot's eight logical CPUs, a parked core) and a
    parked or down-clocked core; the host runs a custom power plan with
    its parking settings at their defaults. The instrument is passive
    and costs nothing: `typeperf` at one second over the slot's logical
    CPUs (`% Processor Time`, `Parking Status`, `% of Maximum
    Frequency`, `Processor Frequency`) and `\Thread(rarog*)\% Processor
    Time`, beside every four-thread leg from now on, so the next slow
    episode names its mechanism; and each leg's per-side in-game R
    recorded as its validity check (a four-thread Rarog block under 3.0
    is not read).
  - **What stands as the engine-side lead:** four threads reach a depth
    with four times one thread's nodes and the helpers' entries do not
    cut the main thread's tree (RAR-R29, RAR-R33), where Reckless's and
    Stockfish's four-thread sides in games reach one thread's depth at
    1.3 to 1.6 times its nodes. It predates C (B.9 0.96) and is priced
    only by the rerun.
  - **Next, in order.** (1) RAR-M80 as registered, with the sampler
    beside it (an added observation; its legs do not change). (2)
    RAR-M81, the maintainer's night: Rarog's and Reckless's threads legs
    of RAR-M78 played again in alternating blocks of 250 games (three
    games at a time, the same binaries, Hash 256, `10+0.1`, the odds
    legs reused), the sampler running, each block's per-side R its
    validity check, read as RAR-M78 with the prediction above. (3) By
    its outcome: not deficient closes D.2's four-thread target
    `NO_CHANGE` with the host-state note carried to E.2's four-thread
    arm and every rated four-thread read; deficient makes RAR-R33's next
    read (the depth and bound of the main thread's hits against what its
    node needs) the card's first engine question, with helper depth
    staggering and the shared histories (H27) as the competing
    mechanisms. No engine change is in scope before (2). Until the night
    read is in, D.2 is held and the agent works D.5.15, D.4.19
    and D.3, in the register's order.
  **Registered 2026-11-02, after the audit:** RAR-M80 amended with the
  sampler (an added observation, before any game), and **RAR-M81**, the
  decision read above: Rarog's and Reckless's threads legs of RAR-M78
  played again, 1,000 games each, the odds legs reused, the sampler
  running, a leg with any fifth under R 3.0 (Reckless 3.2) recorded as
  slow state and not read; NOT DEFICIENT 0.60, UNRESOLVED 0.25, DEFICIENT
  0.15. One night, after RAR-M80 in one command
  (`analysis/artifacts/d2-night/run.ps1`), prepared 2026-11-02: self-test,
  dry run and sampler smoke recorded in RAR-M81's *Preparation*.
  **Night read 2026-11-03** (RAR-M80, RAR-M81; the sampler's reads in
  RAR-M81's *Result*): **the slow state is Rarog's, not the host's.** On one
  quiet host Rarog's four-thread side ran at 2.62–2.66 times its one-thread
  side in every leg and every fifth for 9.5 hours, Reckless's at 3.36–3.48
  beside it. RAR-M80 NOT REPLICATED: one game at a time costs nothing
  (RAR-M79's 23% was an episode). RAR-M81 UNRESOLVED by its rule (Rarog's
  leg SLOW STATE); in that state four threads are worth 0.82 doublings of
  Rarog's time against Reckless's 1.28 and Stockfish's 1.31, and G4 follows
  R. The sampler excludes core parking, frequency and scheduling: Rarog's
  threads get Reckless's CPU time and each does a third less than Rarog's
  single thread; its load sits more often on both SMT siblings of a core.
  The audit's host-state reading is corrected. Next, zero-game and on an
  idle host (after Basilisk's tune): the slot rebuilt outside the harness,
  Rarog's four-thread search confined to one logical CPU per physical core
  against both siblings, Reckless the control. Recommended before the
  registered replay of the slow leg, which would most likely repeat it.
- **D.3 Engine lifecycle and protocol robustness; `src/uci/` (planned) move; score normalisation research card (`analysis/uci_info_review_2026-09-16.md` item 6); zero crashes over pool tournaments — `I1` for the eight review-confirmed defects (2026-11-01), then `R2` for the cards.** UCI
  parsing and dispatch, stop/ponder/infinite semantics, new-game resets,
  malformed input, panic reporting, Syzygy probe policy and thread safety.
  **Found 2026-11-01 in D.2's checks:** in a full `cargo test` run,
  debug or release, 1 to 5 of `tests/uci_process.rs`'s 28 tests time out
  with no engine output at all (`seen: []`), a different set each run; on
  the unchanged head too (4 of 28, release). They pass 28 of 28 when the file
  runs alone. Cause unread; the start-up of a fresh process under the suite's
  load is the first candidate. A test that only fails under load is still a
  failure: D.3 owns it.
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
  **Engine review input, 2026-11-01** (`analysis/engine_review_2026-11-01.md`
  §3.6): eight shell defects, four reproduced in live sessions against the
  debug binary, each a one-function fix that leaves the fingerprint (bench
  sends no clock and no `setoption` during a search), repair case 1 with the
  `tests/uci_process.rs` case written first. They go before the 2.7.0 cut, in
  this order:
    - **D.3.1 A zero, negative or unparsable own clock searches without limit — `I1`, DONE 2026-11-03.** A zero, negative or unparsable own clock runs an unbounded search: clocks
     parse as `usize`, a failure becomes 0 with a notice, and
     `time.rs:145–149` reads 0 as no clock, so `go wtime 0 btime 0` and `go
     wtime -50 btime 1000` search until `stop` (replayed: past 3 s with no
     limit). Parse signed, keep presence as `Option` per side, clamp a present
     clock to at least 1 ms (the donor moves at once on a zero clock;
     Stockfish clamps); a unit test on `compute_runtime_limits`.
    - **D.3.2 `setoption` during a search wedges the input thread — `I1`, DONE 2026-11-03.** `setoption` during a search wedges the stdin thread for good:
     `set_option` calls `wait_for_search_finished`, which pushes a `Ready`
     behind the running `Go` and blocks; under `go infinite` the only line
     that could end the search is one that thread never reads (replayed:
     killed at 7.5 s with `stop` and `quit` unread). The wait is not needed,
     the FIFO already orders `Configure` after the `Go`: drop it or answer
     with a notice. The donor forwards only `isready`, `stop` and `quit` while
     searching.
    - **D.3.3 End of input never ends `go infinite` or a finished ponder search — `I1`, DONE 2026-11-03.** Stdin EOF never ends a `go infinite` or a finished ponder search: EOF
     pushes an ordinary FIFO `Quit` (deliberately, so a piped `bench`
     completes) and sets no flag the search polls, so the engine searches or
     sleeps forever when stdout stays open. An `eof` flag treated as `Stop` by
     `infinite` and `ponder` searches only.
    - **D.3.4 A closed stdout panics every `println!` — `I1`, DONE 2026-11-03.** Every library `println!` panics on a closed stdout, and under `panic =
     "abort"` the engine thread dies as a crash on a normal GUI shutdown
     order: `writeln!` on a locked handle with `request_quit` on `Err`, one
     GUI-gone signal that serves item 3 too.
    - **D.3.5 A non-UTF-8 input line reads as end of input — `I1`, DONE 2026-11-03.** A non-UTF-8 input line is read as EOF (`read_line`'s `Err` becomes 0
     bytes through `unwrap_or_default`) and the engine exits silently with
     status 0: `read_until` and `from_utf8_lossy`; only `Ok(0)` is EOF.
    - **D.3.6 `stop` during `bench` or `wac` prints an ordinary-looking summary — `I1`, DONE 2026-11-03.** `stop` during `bench` or `wac` ends the suite early and prints an
     ordinary-looking summary (`Nodes searched : 110427` against 1,679,897 for
     the same `bench 9`), the line the PGO trainer waits for: print that the
     bench was interrupted and no summary on `Stop`.
    - **D.3.7 A refused `Hash` is retried on every later `setoption` — `I1`, DONE 2026-11-03.** A refused `Hash` is retried and re-announced on every later `setoption`,
     because option state lives twice (the protocol's copy and
     `SearchShared.hash_mb`) and every `Configure` resends the whole struct:
     remember the last requested size, or configure only the option that
     changed.
    - **D.3.8 The forced-move shortcut fires for a one-move `searchmoves` — `I1`, DONE 2026-11-03.** The forced-move shortcut (stop after depth 2 with one legal move) fires
     for a one-move `searchmoves` under fixed depth or time, because the
     filtered root list feeds the count: exclude a non-empty `search_moves`.
    **Done 2026-11-03:** D.3.1 to D.3.8 (`46ddf56`); each defect has its
    `uci_process` case, written to fail on the code before (a present clock
    of 1 ms at least, `setoption` without the wait, the input-closed flag,
    `uci_println!`, lossy UTF-8 input, the interrupted-suite report, the
    remembered `Hash` request, the shortcut off under `searchmoves`);
    behaviour-neutral, the fingerprint 14,749,043 / EBF 2.564 reproduced;
    debug and release suites, fmt and clippy clean.
    - **D.3.9 The score-normalisation card, the `src/uci/` (planned) move and the protocol decisions — `R2`.**
      Decisions to record, not changes: a bad `position` ends the process with
      `process::exit(1)` from library code (both donors keep the previous
      position; the exit has one advantage, the message reaches the retained
      transcript); keep it until a logged game shows a GUI sending a recoverable
      bad `position`, move the `exit` to `main.rs`, and answer a pending `go` with
      `bestmove 0000` first. With the `src/uci/` (planned) move: one option table generating
      both the advertised string and the clamp, `OptionUpdate::{Applied, Rejected,
      Unknown}` so the protocol layer prints (15 `info_string!` sites leave the
      parser), one `run_suite` under `bench` and `wac`, the dead queued
      `PonderHit`, `#[ignore = "needs RAROG_SYZYGY_PATH"]` on `multipv_syzygy` so
      a skip is visible, and D.6's P12 (the option types out of the parser, a
      structured `InfoSink` record so score formatting leaves the search), which
      also removes the `search` and `search_options` cycle. `go mate N` as a
      fixed-depth search is noted, low priority. State: `READY_FOR_IMPLEMENTATION`
      for items 1–8 (`I1`); the score-normalisation card and the move stay `R2`.
      **Basilisk cross-check input, 2026-11-02** (`analysis/basilisk_crosscheck_2026-11-02.md` §H, H13–H16): for the
      score-normalisation card, `format_score` (`search/mod.rs:334-345`) maps
      the tablebase band to `cp` 19,873–20,000 but prints any ordinary score
      of 19,872 or more raw, the rule-50-downgraded mate (`TB_WIN_SCORE − 1`,
      `tt.rs:585`) among them, as `cp` above every tablebase win; Basilisk
      clamped ordinary scores to 19,871 after 111 such lines in 64 of 1,000
      games. A process-level test that panics a helper thread in a
      release-profile build (`panic = "abort"`) and requires the `info string`
      report and the exit status: Basilisk's caught a runtime keeping the
      handler per thread, and Rarog's test is in-process on the main thread
      (`crash_report.rs:167`). A mated or stalemated root under `go infinite`
      reports exactly once and gives `bestmove` only on `stop` (the existing
      case covers `go depth 5` and checks presence, not uniqueness). PROTO-6's
      decision also covers castling rights that do not match the rooks
      (`4k3/8/8/8/8/8/8/R3K3 w KQkq - 0 1`): Rarog rejects the FEN and ends
      the process, Basilisk keeps the rights the rooks support.
    - **D.3.10 `tests/uci_process.rs` times out under the full suite's load — `I1`.**
      Found 2026-11-01 in D.2's checks (D.3's text above): under a full
      `cargo test`, debug or release, 1 to 5 of the file's 28 tests time
      out with no engine output, a different set each run, on the unchanged
      head too; read again 2026-11-02 in D.4.19's verification (1 in debug,
      4 in release; 28 of 28 with the file alone, both profiles). A test
      that fails only under load is still a failure. Cause unread; the
      start-up of a fresh process under the suite's load first. On
      2026-11-03 four full runs (D.5.15's and D.3's, debug and release, the
      file holding 28 tests and then 36) timed out none: intermittent.
- **D.4 Behaviour-neutral repairs from the engine review: the table handle and the helper lifecycle, the fixed-depth vote, the `Hash` notice, dead code, test and CI gaps — `I1`.**
  Added 2026-11-01 from `analysis/engine_review_2026-11-01.md` (its §5 lists
  every item with its reviewer ID; the reports are in
  `analysis/artifacts/engine-review-2026-11-01/`). Every item reproduces
  `bench 13` 13,987,223 / EBF 2.565 on both slider backends, passes the debug
  and release suites, `cargo fmt --check` and `cargo clippy --all-features
  --all-targets` at zero warnings, and lands as its own commit; an item that
  touches hot code (6, 9, 10) adds the allocation guard and one pooled-NPS
  read by PROCESS's method. No strength gate: AGENTS' repair case 1 with the
  named activation or correctness read. In order of value:
    - **D.4.1 The table handle — `I1`.** (TT-1, TT-3, DRIVER-10, DRIVER-20; the 2026-10-06
     audit in D.2): delete the `Clone` derives on `TranspositionTable`,
     `TtStorage` and `LocalTable`; a `SharedTt(Arc<SharedTable>)` is the only
     thing a `WorkerJob` carries, and the worker drops it after sending its
     result, so an idle helper no longer pins the previous table through a
     `Hash` change (today a resize holds old plus new until each helper's next
     job, locked large-page memory included, and the comment that two are
     never held at once is true only on the page-backing path). Read: `Threads
     4`, `Hash 1024`, one search, `setoption Hash 2048`, the process's private
     bytes; a Threads-2 depth smoke completes.
    - **D.4.2 A `Hash` change at `Threads > 1` fills the table twice — `I1`.** (TT-2):
     `configure` builds a local table that `make_shared` then drops; build the
     shared form directly. The property that a failed resize keeps the old
     table is pinned by a test and is the maintainer's trade.
    - **D.4.3 The fixed-depth vote — `I1`.** (DRIVER-12): helpers drop the depth limit and
     the weighted vote runs for every single-line search, so `go depth N` at
     `Threads > 1` can report a deeper helper's result and print `info depth
     N+1`; skip the vote under a depth limit (the donor answers with thread
     0). A test at Threads 4.
    - **D.4.4 Helper start-up — `I1`.** (DRIVER-14, TT-8): each helper builds and discards a
     64 MiB table (`Searcher::default()` then `TranspositionTable::new(1)`),
     touches about 32 MB of tables, and `isready` does not wait for any of it;
     a helper constructor without the default table, and the worker
     acknowledges its init and `NewGame` so `readyok` means ready.
    - **D.4.5 The `Hash` rounding notice — `I1`.** (TT-7): `Hash` is floored to a power of
     two with no word (1000 → 512 MiB, 2000 → 1024, 3000 → 2048, 32000 →
     16384); an `info string` notice from `configure`, a README sentence, a
     test (`Hash 100` → 64). Sizing to any value (a multiply-shift index with
     a low-bit tag) changes the fingerprint at every size and is D.5's, only
     if the notice shows anyone sets such values.
    - **D.4.6 The dead quiescence type parameter — `I1`.** (NODE-6): `quiescence<NODE>` never
     reads `NODE` (the PV bit comes from the stored form since B.4), so `Pv`
     and `NonPv` are two identical 355-line copies of the hottest recursion;
     drop the parameter. Fingerprint exact; the pooled-NPS read and one
     `quiescence` body in the release binary.
    - **D.4.7 The root `searchmoves` filter — `I1`.** allocates a `Vec` per root call
     (NODE-13): filter in place.
    - **D.4.8 Dead Syzygy code — `I1`.** (PROTO-13): `syzygy::probe_root`, `RootProbe`, their
     constants and the dead `tb_init` branch (it returns `true` at both
     exits); one `unsafe` site fewer.
    - **D.4.9 The slider-table `unsafe` and the timer — `I1`.** (RUST-2): the four identical
     `get_unchecked` sites in `attacks.rs` become one `#[inline(always)]`
     helper (codegen `pext; add; mov` unchanged, checked in the assembly), and
     `timeBeginPeriod` is declared `pub safe fn` in the `unsafe extern` block
     (compile-checked on 1.99.1), removing the block in `main.rs`. With item 8
     the count falls from 23 to 18; the floor is the maintainer's decision and
     this leaf proposes it.
    - **D.4.10 Stack-built tables — `I1`.** (RUST-8): `HistoryTables::default` builds 64 KB
      and 21 KB tables in its frame and copies them to the heap, and
      `rank_root_moves` builds the 102 KB `TbRootMovesRaw` twice per tablebase
      root rank; build them zeroed on the heap, and the assembly scan shows
      the copies gone.
    - **D.4.11 The Fathom FFI layout guard — `I1`.** (RUST-5): `const` size asserts for
      `TbRootMoveRaw` (528 B) and `TbRootMovesRaw` (101,908 B, align 4) and a
      test that parses `TB_MAX_MOVES` and `TB_MAX_PLY` out of
      `vendor/fathom/src/tbprobe.h`; a drift there would be silent memory
      corruption.
    - **D.4.12 The PEXT tier in CI — `I1`.** (RUST-7, BOARD-5): `--cfg rarog_pext` without
      `+bmi2` compiles clean today, so add a `compile_error!` under
      `all(rarog_pext, target_arch = "x86_64", not(target_feature = "bmi2"))`,
      and a CI step with `RUSTFLAGS="--cfg rarog_pext -C
      target-feature=+bmi2"` for clippy and `cargo test --test
      slider_backends` (clean on this host).
    - **D.4.13 The `texel` and `tune` evaluation tests in CI — `I1`.** (EVAL-1): the
      integer-exact trace reconstruction gate runs only under `--features
      texel` and CI never runs it (37 pass today); one CI step per feature and
      a line in AGENTS' verification list for evaluation or trace changes.
    - **D.4.14 The KPK bitbase at compile time — `I1`.** (RUST-4): the one table built at run
      time, 524,288 bytes, exactly the large-page threshold, so it takes a 2
      MB locked page; a bit-packed `[u64; 1024]` (65 KB) built by `const`
      evaluation or `build.rs`, checked equal to `generate()` over every
      index, after which `initialize_tables` goes.
    - **D.4.15 `Move::from_uci` into `make_move` — `I1`.** (BOARD-4): a non-canonical move
      from the text constructor corrupts the board in release (a capture
      leaves the captured bit; `e1g1` castles without the rook); every caller
      canonicalises today. A `UciMove` newtype that only `legal_move` and
      `parse_move` turn into a `Move` makes the state unrepresentable.
    - **D.4.16 Names and comments that mislead — `I1`.** (EVAL-9, TT-13, DRIVER-13,
      BOARD-10; H12): seven ledger IDs and roadmap step numbers such as
      `mop_up.rs:129` and `144-149` remain in `src`, which AGENTS forbids
      (a census by script first); `LIGHT_SQUARES` holds the dark squares and its comment says
      otherwise (every use is relative, so no bug); `replacement`'s doc claims
      a bound clause the code lacks and `LocalCluster::holds` ignores
      occupancy where the shared one requires it; the aspiration doc says a
      fail-low halves toward the old window where the code halves toward the
      new alpha, and three `params.rs` notes describe pre-fit defaults;
      `piece_type_at_unchecked` is checked; roadmap numbers in `board.rs` and
      `zobrist.rs`.
    - **D.4.17 Tests the review found missing — `I1`.** (TT-12, NODE-14, DRIVER-21, BOARD-7,
      EVAL-2, PROTO-20): a deterministic torn-pair test and `false_hits <= 3`
      with a seeded set for the shared table (today's threshold cannot see an
      8-bit tag, and the eval-only test's `hashfull` assertion is a
      tautology), a four-thread hammer on one `SharedCluster`; the `cut_node`
      each recursive call receives; `is_quiet_move(mv) == !is_noisy(mv)` over
      legal moves; the estimate at an excluded node; deeper standard perft in
      release (start d5 4,865,609; position 3 d5 674,624 and d6 11,030,083;
      position 4 d5 15,833,292); a `gives_check_with` differential (0
      mismatches over 500,523 moves in the review's scratch run); the
      cross-null repetition characterisation of item 18; `tests/endgames.epd`
      and the recogniser FENs through `mirror_fen` (every one has White as the
      stronger side today).
    - **D.4.18 A record corrected — `I1`.** (BOARD-3): `analysis/draw_policy_2026-09-08.md`
      Policy 2 says a repetition scan crossing a null move can only be a false
      negative; two null moves by the same side while the opponent shuffles a
      piece out and back produce a false twofold through the public API
      (reproduced; 46 of 7.6 M nodes at depth 12 on the bench positions).
      Correct the sentence, pin the behaviour with a test, keep the retry
      trigger; a `plies_from_null` bound is a registered unit of its own, not
      bundled here.
  Added 2026-11-02 from the Basilisk cross-check (`analysis/basilisk_crosscheck_2026-11-02.md`,
  read against Basilisk's work since its 2026-10-06 review):
    - **D.4.19 The KPK evaluation and every hard draw verdict against Syzygy — `I1`.** (A3,
      widened by H7: Basilisk's port of the reference rook-ending rules
      called 18 of 3,000 random won KRPKR positions dead draws, BAS-E54;
      Rarog's `krpkr_scale` returns `Some(0)` at five sites,
      `eval/endgame/mod.rs:481-505`, guarded by three hand-picked won
      FENs, and the KQKP fortress, the wrong-corner KBP and the drawn KPK
      are hard zeros too. Read: every such zero over random or exhaustive
      KRPKR, KQKP, KRPKB and KBPK positions agrees with a WDL draw; a
      wrong zero is a D.5 repair card that discounts instead of zeroing.)
      The KPK part: a won KPK falls through
      to the generic evaluation (`eval/endgame/mod.rs:150-157`) and nothing
      tests its sign; `kpk.rs` holds four unit tests. Basilisk found 2,022
      of 662,704 positions where a Syzygy win reached its scalers at or
      below 0 for the pawn side (a second-rank pawn, the lone king
      adjacent). Read: every legal KPK position, both colours, both sides
      to move, the static evaluation's sign against the WDL verdict, and
      no win at or below 0; a bundled `KPvK` fixture makes it a test. No
      failure: the test lands here, case 1. Failures: a repair card in
      D.5 with Basilisk's evidence (cost in games about zero, a leaf
      error the search overrides from the root) as its prior.
      **Read 2026-11-02** (`analysis/draw_verdicts_2026-11-02.md`, §2–3).
      KPK: 662,704 legal positions; every drawn one reads exactly 0 and the
      bitbase agrees with Syzygy on all of them, so the verdicts land as a
      test (`tests/kpk_syzygy.rs`, case 1; `KPvK.rtbw` added to the
      fixture). 88 won positions read at or below 0 for the winner, from
      the generic evaluation (a pawn on its 2nd to 4th rank beside the lone
      king; Basilisk found 2,022): D.5.21, its acceptance test
      committed `#[ignore]`d. `krpkr_scale`: over 2,000,000 quiet random
      KRPKR positions its five zeros fired 65,865 times, 1,137 (1.7%) on
      wins for the pawn side, 1,128 of them at sites 1 and 5: D.5.22.
      The KQKP fortress, `krpkb_scale` and the wrong-corner KBPK zeros stay
      unread (4- and 5-man tables, minutes each).
    - **D.4.20 CI hygiene — `I1`.** (A5): no job in `ci.yml` or `release.yml` has
      `timeout-minutes`, so a wedged child process burns the runner's
      six hours; check which mnemonics `xtask`'s ISA verifier forbids at
      the baseline tier (Basilisk's portable set missed SSE4.1, AVX1 and
      BMI1's `andn` and `tzcnt`); check whether the macOS toolchain
      major is pinned (an image update otherwise changes codegen and NPS
      with no failing check). Tooling commits. Also (H1–H6): one
      aggregating job per workflow that always runs and fails unless every
      job it gathers succeeded, made the required check (GitHub reports a
      job skipped after a failed dependency as passing; `release.yml`'s
      `collect` has no `if: always()` and `ci.yml` has no aggregator); a
      documents job running `check_guide.py`, `check_citations.py` and the
      `tools/diag` tests, none of which CI runs today (Basilisk's found a
      guard test failing unnoticed), with a check that each ledger index
      row's disposition matches its entry's; a PGO use step that warns on
      functions with no profile (`-Cllvm-args=-pgo-warn-missing-function`)
      and fails when a kernel symbol is warned, so `e057d84`'s trained
      copies cannot silently go cold again; the Linux assets' glibc floor
      recorded and the runner labels pinned instead of `-latest` (a static
      build is G.2's, with an NPS read); Miri on the unit tests that reach
      the `unsafe` sites outside the FFI (TT cluster, slider tables), as a
      nightly job; a development build's `id name` carrying its commit and
      a dirty mark, as Stockfish's does, so a stale binary names itself.
    - **D.4.21 Basilisk's behaviour tests run against Rarog — `I1`.** (§D of the
      cross-check; about 20, among them shortest mate rather than the
      first mate, an injected illegal TT move never searched, every
      shipped FEN strictly legal, bare-king material monotone with the
      promotion order, a rook promotion only where the queen stalemates,
      a won ending that does not run the clock to the maximum, `Hash`
      setup finished before `readyok`, the ponder races, two positions
      from game incidents). One scratch run against the current binary
      first; a test that passes lands here as a regression anchor, one
      that fails becomes an item in the leaf that owns its mechanism.
      With them (H8–H11): the picker contract over a few thousand
      random-walk positions with randomised histories (every legal move
      once, a legal TT move first and never twice, a foreign TT move
      ignored, skip-quiets leaving exactly the noisy moves, ProbCut's list
      exactly the SEE-passing captures; today six fixed FENs with zeroed
      histories); a liveness test per search parameter (each value moves
      the tree on a named position; the SPSA coverage audit is static);
      floors that also fail when stale (WAC's `FLOOR = 122` was set at
      133/300 on 2026-07-19 and has no upper bound); fixtures that assert
      the score at a stated budget, not only the move (Basilisk's promotion
      fixture passed on a 4-cp tie held by root order).
    - **D.4.22 Measurement and gating methods from Basilisk — `I1`, DONE 2026-11-02.** (H38–H45; decided
      by the maintainer 2026-11-02). Adopted: a repair's registration
      states its measured exposure (AGENTS, *Gating*); a written analysis
      of the fitted constants before a bake, from
      `tools/spsa_theta_audit.py` (PROCESS, *SPSA go/no-go procedure*,
      step 9); the curvature sweep's noise floor (rule 8); `nps_read.py`
      warning when its arms search different trees; a second-step NPS
      accept below +1% on a change that moves code recorded as possibly
      layout (PROCESS, *Toolchain and harness notes*); a correct repair that fails its
      harm rule reverted by the gate with its defect left open and a
      rework registered (AGENTS, *Gating*). Declined for now, each with
      its retry trigger: the pinned bench, because the builds and not the
      cycles set the interval (retry when the disturbance rule repeats
      more than about one read in three); several pools in one read
      (retry at the first speed study with three or more arms); an
      ordered night-queue launcher (retry at the first chained run that
      started on a failed predecessor).
  Not in this leaf: anything that moves the fingerprint (D.5), the
  simplification programme (D.6), the shell defects (D.3), the tablebase path
  (D.7).
- **D.5 Strength candidates from the engine review: kernel and evaluation deviations from the donor, each behind a zero-game screen and a registered gate — `R2` cards, then `I1`/`I2`/`V`.**
  Added 2026-11-01 from `analysis/engine_review_2026-11-01.md` (§3.1, §3.2,
  §3.5). Each item is an unrecorded deviation from the donor's shape with a
  plausible mechanism; none is a measured strength defect, so each enters at
  `RESEARCH` and crosses `READY_FOR_IMPLEMENTATION` only with its screen read
  and its prediction frozen (AGENTS, *Classify the work*). Rules: no
  implementation before its screen; one candidate per gate; rule 6 stops the
  leaf after two rejections in a row; a screen that reads nothing closes the
  item `NO_CHANGE`. The fingerprint moves with every one, so none rides in D.4
  or D.6. Item 15 is a correctness repair, not a donor deviation, and goes
  first, before the 2.7.0 cut. Then the first two, classification repairs
  with the cheapest reads; items 3 and 4 are screened together and gated
  apart.
    - **D.5.1 Castling is neither quiet nor noisy — `R2`.** (NODE-3, BOARD-1):
     `is_quiet_move` is `flags <= DOUBLE_PUSH` while castling has flags 2 and
     3, and `is_noisy` is capture or promotion, so in the move loop castling
     skips late-move, futility and history pruning, takes the noisy SEE
     threshold and LMR base, reads noisy history, and when it fails receives
     `update_noisy` maluses into a slot the quiet picker never reads, while a
     successful castle is credited in the quiet tables; the noisy slot for the
     castling king drifts to its floor and adds about a ply of reduction. The
     predicate dates from the initial commit; the donor classes castling quiet
     with one predicate. Screen: the agreement test of D.4.17 and the
     four castling noisy slots after `bench 13` at stride 1 (expected strongly
     negative). Fix: define quiet once (`!is_capture() && !is_promo()`). Gate:
     case 2, a 2,000-game harm read, as a classification repair expected near
     zero.
    - **D.5.2 Quiescence cannot see a non-capturing queen promotion — `R2`.** (BOARD-2): pawn
     captures target enemy occupancy only, every non-capture promotion is
     generated with the quiets, and quiescence outside check generates
     captures only, so a safe promotion to a queen is invisible to the
     stand-pat decision (`8/P6k/8/8/8/8/8/K7 w - - 0 1` returns no captures).
     In the review's scratch counters over the bench positions at depth 10,
     1.53% of capture generations had a legal quiet queen promotion and 0.59%
     had it with an empty capture list; endgame-heavy books read higher. The
     donor generates queen promotions with the noisy moves and
     under-promotions with the quiets; B.4 exempted promotion captures from
     the count rule and never weighed this. Card: the horizon at the
     promotion; interactions with the quiescence count rule, delta pruning,
     the SEE floor, ProbCut's capture list (which it would also enter), the
     `has_pseudo_capture` pre-scan (it must admit a pawn on the seventh with a
     free square) and the staged picker contract; invariant: the generation
     partition test. Screen: the counter above on an endgame-rich suite; under
     about 0.5% unexamined promotions the item closes. Gate `[0,3]`
     (strength-seeking, a shared quiescence signal), `[-5,5]` if the
     maintainer prefers a symmetric read.
    - **D.5.3 Null-move and ProbCut children typed `cut_node = true` — `R2`.** (NODE-1): the
     donor passes `false` for the null child (`search.rs:606`) and both
     ProbCut searches (`664–684`); Rarog passes `true` at `node.rs:988`,
     `1203` and `1221`. The type feeds the child's TT-cutoff admission, the
     cut-node LMR terms, LDSE, the negative singular extension, the history
     bonus offsets and the fail-low parent reward, and flips every
     descendant's label. B.3's data put ProbCut's verification success at 98%
     (its child is an all node); the null child's right type depends on the
     NMP conversion rate (22% at the B.3 head, unrecorded since). No record
     mentions it, and the SPSA fits were made with it in place. Screen: `diag`
     counters of the fail-high rate by node type for null children, ProbCut
     children and LMR children on `bench 13` at stride 1; the donor's prior
     predicts at least 70% fail-high for nodes typed `true`; if ProbCut's
     children fail high at about 2% and the null children above 50%, split the
     fix. Then a unit test that pins the `cut_node` each recursive call
     receives. Gate: case 3 `[-5,5]` as a repair (TT cutoffs and reductions
     are shared signals); `[0,3]` only if the screen predicts a gain and the
     registration says so. A neutral fingerprint is not expected.
    - **D.5.4 A fail-low node stores its least-bad move as the TT move — `R2`.** (NODE-2):
     `best_move` is set on every new best score, so an `Upper` entry carries
     the refuted move and overwrites the position's earlier move; the donor
     sets it only when alpha rises and keeps the old move. Consumers at the
     next visit: IIR, the razoring guard (a stored quiet fail-low move blocks
     razoring at exactly its target nodes), the no-TT-move LMR term, the
     first-move slot, the fail-low parent-reward factor. Noted in
     `analysis/b21_review_2026-09-14.md` §4 and never isolated. Screen in a
     `diag` build storing `NULL` for `Upper`: `iir_applied`, razor counts, the
     no-TT-move LMR share, `bench 13` nodes, WAC at 100k and the canary rule.
     Gate: case 3, or `[0,3]` with a predicted gain.
    - **D.5.5 The exclusion search uses the table-refined estimate — `R2`.** (NODE-4):
     `eval_for_pruning` has no `excluded` guard; in a singular exclusion node
     the TT bound is Lower or Exact and the window sits below `tt_score`, so
     the refined estimate always exceeds `alpha + margin`, LDSE can never fire
     there and the LMR `alpha_gap` term is pinned at its clamp; the donor
     keeps the plain eval (`search.rs:471–482`). Screen: a counter of
     exclusion nodes where LDSE would fire; fix: a one-line guard. Gate: case
     2 under 2% of exclusion nodes, else `[0,3]`.
    - **D.5.6 Captures generated and scored before the TT move — `R2`.** (NODE-5):
     `fill_noisy` runs before the loop; the donor generates after the hash
     move fails to cut, and the TT move cuts at about 35% of loop nodes
     (`analysis/c12_speed_review_2026-10-18.md`: the one lead worth several
     percent). The capture scores then read a history the TT subtree has
     moved, so ordering changes. `I2` with a frozen prediction (speed +x%,
     games neutral), the pooled-PGO NPS read, then case 2.
    - **D.5.7 The ply cap — `R2`.** (NODE-8): `MAX_PLY = 128` is below the extension budget's
     bound (about twice the iteration depth plus 16); from about iteration 56
     the cap, not the budget, ends lines and the node returns a static eval
     with no in-check or draw handling (the donor: 240, a draw in check). The
     'revisit if seldepth reaches it' trigger of
     `analysis/search_programme_2026-09-13.md` fired (seldepth 127 at depth
     19–20 on bench position 11; D.1.1's rec2) without a decision. Screen: a
     counter of nodes entering the cap per game at `3+0.03`, `10+0.1` and
     `60+0.6` from the pool's positions, with the in-check share. Fix: raise
     the cap together with the band constants (`TB_WIN_SCORE`, `TB_VALUE`, the
     `i8` table depth, `reserve_history`, the `PlyArray` sizes). Gate: case 1
     if the count is zero at STC, else an LTC harm read.
    - **D.5.8 A depth-0 entry is counted as two nodes — `R2`.** (DRIVER-11): `check_stop` at
     the `negamax` entry and again at the `quiescence` entry it hands to, so
     reported nodes, `go nodes`, the bench count and the LMR spread's phase
     read the double count; the donor counts once per move made. Screen: the
     `diag` interior and quiescence counts against `td.nodes` on `bench 13`.
     Fix: test `depth <= 0` first and give the LMR spread its own counter if
     its phase is to be kept. A new fingerprint by construction: register it
     alone, case 2 at most, never folded into a strength change.
    - **D.5.9 The endgame scale's target — `R2`.** (EVAL-5): `evaluate` adds tempo to `mg`,
     tapers, then `scale_endgame` multiplies the whole; both donor commits
     scale the endgame half only (`v = mg·p + eg·(128−p)·sf/64`), so the forms
     differ for the opposite-bishop rule with pieces on the board and the
     partial recognisers at small phase. The constants were fitted inside
     Rarog's form (RAR-E26). Screen, offline: refit the three OCB constants
     with the scale on the endgame half on RAR-E26's cohort and compare
     held-out loss; then case 2 at most (OCB-with-pieces positions only).
     Under the evaluation freeze this is a registered repair.
    - **D.5.10 The pawnless default factor — `R2`.** (EVAL-6): `material.cpp:195–203` scales
      a side with no pawns and a small material edge to 0, 4 or 14 of 64;
      Rarog's pawnless branch covers at most one minor in total or one each,
      so K+B+N v K+B keeps two thirds of about a knight. Never a ranked family
      (the family list is SF11's endgame functions). Screen: the families'
      share of the game record; at 0.5% of games or more the Syzygy
      drawn-overclaim read of RAR-E36. Case 2.
    - **D.5.11 Piece-term definitions no experiment read — `R2`.** (EVAL-10): outposts use
      the current enemy pawn attacks rather than the attack span;
      minor-behind-pawn counts own pawns only; the long diagonal reads the
      enemy king zone rather than the two centre squares. Screen: an E62-style
      direction read, zero games; a term at the line enters a refit cluster
      (case 2 or 3 by what it moves).
    - **D.5.12 `Hash` at any size — `R2`.** (TT-7): the multiply-shift index with a low-bit
      tag, after D.4's notice shows a need; case 2.
    - **D.5.13 The aspiration fail-low window — `R2`.** (DRIVER-13): the code halves toward
      the new alpha; a counter of fail-lows whose re-search fails high
      (`asp_fail_high` straight after `asp_fail_low` at one depth); above 15%
      the old-alpha form is a `[0,3]` candidate.
    - **D.5.14 Low-value notes — `R2`.**: PV-node `force` and the mid-loop `Lower` store
      (NODE-7, TT-10: a counter of refused PV stores first; `force` is case
      2); the parent reward and history updates inside exclusion searches
      (NODE-10: shared with the donor; a counter, then case 3 only above 5% of
      rewards); quiet history captured at list generation rather than per move
      (NODE-12: nothing unless item 6 lands); the non-pawn correction key
      without kings and a fifty-move bucket in the TT key (BOARD-9: R2 only).
  Added 2026-11-02 from the Basilisk cross-check (`analysis/basilisk_crosscheck_2026-11-02.md`):
    - **D.5.15 The correction is added to the evaluator's draw verdicts — `I1`, DONE 2026-11-03.** (A1;
      Basilisk's B.14, accepted as BAS-S44 on 2026-11-01). The evaluator
      returns exactly 0 for the drawn KPK, the wrong-corner KBP-K,
      KK/KNK/KBK and minor against minor, and KNNK
      (`eval/endgame/mod.rs:52-58`, `148-209`); `corrected_eval_parts`
      adds the correction to every raw value (`search/correction.rs:173-188`),
      and the node-end training runs inside exclusion searches and on
      decisive scores (`search/node.rs:2087-2096`), the configuration in
      which Basilisk's shared structure keys moved its wrong-bishop draws
      off 0. A verdict is a classification with no error to correct.
      Screen: a debug probe of raw against corrected evaluation at those
      leaves after `bench 13` and after searches from Basilisk's
      wrong-bishop fixtures. If the corrected value leaves 0: the
      evaluator reports the verdict and the search adds no correction to
      it, with a test that fails on today's code; gate case 1 by a static
      differential (only nodes whose evaluation is a verdict change). If
      it never leaves 0, `NO_CHANGE` with the test kept. A cold probe is a
      false negative (H17): from cleared tables Basilisk's fixtures read 0;
      the defect appeared only after priming in one process (`ucinewgame`,
      `go depth 12` on `8/8/5K1P/8/8/8/k7/5B2 b - - 0 1` and
      `8/8/6KP/8/8/8/8/k4B2 b - - 0 1`, then the fixtures: 4 of 28 failed
      at +172), and organically on `5k2/8/6KP/8/8/8/8/5B2 b - - 0 1` (only
      Kg8 draws), +90 at depth 11 within 3,075 nodes: the exclusion search
      excludes the one drawing king move and its training lands on every
      structure table. A guard keyed on `raw == 0` moved 36–38 of 40 bench
      positions (most zeros are ordinary); keyed on the verdicts, 4–6 of
      40 searched alone, yet bench fell 28% through shared state, so the
      differential takes a fresh process per FEN. BAS-S44 read +7.4 ± 6.9
      (1,271 pairs); Basilisk also keeps a producer guard (no exclusion
      training at a raw 0), the alternative form.
      **Probe read 2026-11-02** (`analysis/draw_verdicts_2026-11-02.md`,
      §1; diag patch `analysis/d5_item15_verdict_probe.patch`): **the
      corrected value leaves 0.** In `bench 13` (fingerprint unchanged
      under the probe) 703 of 712 verdict leaves carry a correction, 434 by
      50 or more; in Basilisk's fixtures 98–99% do, by 77 to 117 on
      average, and the fixtures misread: 28 of 28 beyond ±75 from cleared
      tables (+123 to +252), 24 of 28 primed, the organic position −110 at
      depth 14 (0 at 8–13). Rarog needs no priming: the cold arm's first
      search, in a fresh process, read +123. **Repair card,
      `READY_FOR_IMPLEMENTATION` (`I1`).** *Mechanism:* a verdict is a
      classification; the search takes it as the static evaluation, 0,
      with no correction, at both static-evaluation sites (main search and
      quiescence), recognised by the verdict and never by `raw == 0`.
      *Interactions:* the table refinement of the static evaluation by a
      stored bound is search evidence and stays; the correction's training
      at a verdict node is unchanged (Basilisk's producer guard is a
      separate candidate, not this repair); a raw value read from the table
      is not re-evaluated, so the node recognises the verdict itself, only
      when the raw value is 0 and a recogniser can apply (five men or
      fewer, or a bare king; the probe's `draw_verdict_kind` is that
      test), one comparison elsewhere, and an NPS read by PROCESS's method
      only if a profile shows it. *Invariants and tests, failing today:* a
      verdict's corrected static evaluation is exactly 0 with trained
      tables; Basilisk's four fixtures read within ±75 at depths 8–14
      from cleared tables and primed (the cold `7k/8/6KP/8/8/8/8/5B2 w` at
      depth 8 reads +123 today). *Gate:* case 1 (AGENTS, *Gating*): the
      fingerprint moves, so its correctness read is a static differential
      with a fresh process per bench FEN (bench shares its tables across
      positions): every FEN whose baseline search reaches no verdict leaf
      (the probe's counters, per FEN) reproduces its node count exactly,
      and only FENs that reach one may move; the registration states its
      exposure from Colosseum's database. *Prediction, frozen 2026-11-02
      before any implementation:* no verdict leaf with a moved value in any
      arm (0.98); the cold fixtures all within ±75 (0.70), the primed all
      (0.65), the organic depth 14 (0.75); the per-FEN differential moves
      only FENs with a verdict leaf (0.85); in games about zero.
      *Falsifier and stop:* if 7 or more of the 28 cold fixtures still read
      beyond ±75 with no verdict value moved, the verdicts are not the
      fixtures' cause: the repair stays if its differential is clean (it is
      a correctness repair), and the fixtures' misread returns to
      `RESEARCH` (the training producer, H17's shared keys) as its own
      card.
      **Implemented and read 2026-11-03** (`3a16fbb`;
      `analysis/draw_verdicts_2026-11-02.md` §5): `corrected_eval_parts`
      returns 0 with no correction when the raw value is 0 and
      `is_draw_verdict` holds, a predicate that follows `scale_endgame`'s
      own order and is tested against its zeros. The fingerprint moves from
      13,987,223 / EBF 2.565 to **14,749,043 / EBF 2.564**. **Gate, case 1:**
      the per-FEN static differential (a fresh process per FEN, depth 13)
      is confined: FENs 3, 6, 37 and 40, each reaching verdict leaves,
      moved, four others that reach a few did not, and all 32 FENs without
      a verdict leaf reproduce their node counts exactly. Correctness read: in bench and in every fixture arm
      no verdict leaf keeps a moved value (703 of 712 in bench before).
      *Calibration:* no moved verdict (0.98) and a confined differential
      (0.85) held. The fixtures missed: from cleared tables 7 of 28 still
      read beyond ±75 (+83 to +103, against 28 of 28 at +123 to +252),
      after training 9 of 28 (24 before); the one-drawing-move position
      read −110 at depth 14 in the probe's process, unchanged, and within
      ±75 in a fresh searcher (predicted all within ±75 at 0.70, 0.65 and
      0.75). The miss is in coverage: the verdicts carried
      most of the misread, not all of it. **The falsifier fired** (7 of 28
      cold): by the stop rule the repair stays, its differential clean, and
      the remaining misread is D.5.23, whose acceptance test is committed
      `#[ignore]`d.
    - **D.5.16 Quiescence prunes by gain against a decisive material edge — `R2`.** (A2;
      Basilisk `d8d30e9`): futility (`node.rs:2355-2362`) and the SEE
      threshold (`node.rs:2392-2396`) read a capture's gain as the piece
      taken, which is false when the capture removes the opponent's
      mating material (Kxh8 in KBB-K is a dead draw); delta pruning is off
      at eight men or fewer and is not involved. Rarog has no known-win
      band, so the guard's definition is the research question. Screen:
      Basilisk's fixture, and a `diag` count of skipped captures that
      leave the opponent without mating material on the conversion
      families. `R2` card first.
    - **D.5.17 Underpromotions and Saavedra's study — `R2`.** (an investigation): Basilisk
      lost Saavedra's `c8=R` when it pruned underpromotions below the root,
      and its search picked a rook promotion over the queen in 59 of 228
      gate positions where a rook line merely transposed (BAS-S33,
      BAS-S34; a root rule now judges a rook or bishop promotion against
      the queen by more than 200 of exact score). Rarog generates every
      promotion and prunes none by piece, and has no record of either
      case. Read, zero games: Saavedra after 5...Rd4
      (`8/2P5/8/8/3r4/8/2K5/k7 w - - 0 6`, `c7c8r` with a mate score), the
      stalemate escape (`8/5P1k/5K2/8/8/8/8/8 w - - 0 1`), Basilisk's
      transposing rook position, and the rook and bishop promotions Rarog
      played in the archived games where a queen would not have
      stalemated, each checked against Syzygy or Stockfish. All correct:
      `NO_CHANGE` with the positions kept as tests. A wrong pick: a card
      with Basilisk's rule and its gates as the prior.
    - **D.5.18 Stockfish 19 mechanisms Basilisk adopted that Rarog lacks — `R2`.** (§C of
      the cross-check): upcoming-repetition detection (cuckoo; Basilisk
      found it must act as a cutoff at the draw window, not a draw score);
      seekMate (with a root score of 2000 or more from depth 16, RFP is
      capped at depth 6 and singular extensions are off); quiet pruning
      and IIR gated on the previous PV; the TT bound-mismatch penalty.
      Basilisk's donor is Stockfish 19 and Rarog's is Reckless, so its
      result is a prior only. Research cards, low priority; none crosses
      `READY_FOR_IMPLEMENTATION` without its mechanism shown active here.
      Basilisk also measured its check extension at −22 Elo at `3+0.03`
      and −5.4 ± 8.6 at `10+0.1`; Rarog has none. Basilisk's game reads
      since (H22): the upcoming-repetition test off 0.0 ± 9.3, so its
      priority falls; quiet pruning at PV nodes off the previous PV read
      +5.9 ± 9.9 with it off, and Rarog prunes quiets at every PV node as
      Reckless does, so the previous-PV gate would be a restriction;
      seekMate's depth is steep (depth 16 grew bench 10.7×, depth 8
      3.8M), its Elo unread. Also absent here and in PLAN: Stockfish 19's
      shuffling gate on singular extensions and non-zero history fills
      (Basilisk's fills read −0.5, null).
    - **D.5.19 Root and in-check reductions never read apart — `R2`.** (H23): LMR is gated
      `!NODE::ROOT && !in_check` (`node.rs:1759`, pinned by
      `lmr_never_reduces_at_the_root_or_in_check`); RAR-S74(e) read the two
      cases bundled in one switch at −3.8 ± 9.6; Reckless reduces in both;
      Basilisk's in-check selectivity read about +8 from unpaired runs
      (BAS-S41 items 1–2). `R2`, low: two counters, then one switch each.
    - **D.5.20 The fail-high stop's hole — `R2`.** (H24): the stop at twice the target is
      checked only when a window returns (`search/mod.rs:977-978`), so a
      window that never fails high runs to the maximum (RAR-R26's rec3,
      97.9 s at `180000+2000`); Basilisk adds the give-up window as a
      second form (2.79× and 4.07× the optimum) and registered a replay
      design (Basilisk's `b5_failhigh_stop_2026-10-25` packet, M1: the hole's
      rate by score band, the table carried). `R2` zero-game read;
      `NO_CHANGE` under 0.5% of searches. Long controls only; the hard
      maximum still bounds it.
    - **D.5.21 Won KPK positions read at or below 0 — `R2`.** (D.4.19's read,
      2026-11-02; Basilisk's A3): 88 of 445,128 won KPK positions read at
      or below 0 for the winner, 78 of the wrong sign and 10 exactly 0,
      all where the bitbase correctly passes a win to the generic
      evaluation: the pawn on its 2nd (68), 3rd (16) or 4th rank (4), the
      lone king beside it, the pawn side to move in 86
      (`analysis/draw_verdicts_2026-11-02.md`, §2). Prior (Basilisk, 2,022
      such): cost in games about zero, a leaf error the search overrides
      from the root. Screen: whether a one- or two-ply search already reads
      the 88 above 0; if it does, `NO_CHANGE` and the ignored test's
      reason says so. Otherwise the repair scores a bitbase win itself (by
      the pawn's rank, above a floor) and un-ignores
      `every_won_kpk_reads_above_zero_for_the_winner` in
      `tests/kpk_syzygy.rs`; gate case 1 by a static differential over KPK
      positions. `R2` read, minutes.
    - **D.5.22 `krpkr_scale`'s hard zeros on won positions — `R2`.** (D.4.19's read,
      2026-11-02; Basilisk's BAS-E54 found 18 of 3,000): over 2,000,000
      quiet random KRPKR positions the five `Some(0)` sites fired 65,865
      times, 97.6% on Syzygy draws; 1,137 (1.7%) are wins for the pawn
      side, 926 at site 1 (the third-rank defence, whose relaxation for a
      pawn on its 2nd or 3rd rank lets the defending rook stand anywhere)
      and 202 at site 5 (the king's blockade)
      (`analysis/draw_verdicts_2026-11-02.md`, §3). A zero on a defender's
      win (462) errs toward the truth. Repair form (D.4.19): discount
      instead of zeroing where the rule is approximate. Screen: per site,
      a scale that keeps the drawn positions near 0 and the wins above it,
      read on the same sample. KRPKR reaches about 10% of real games
      (`krpkr_scale`'s note) and the zero sites far fewer, so the
      registration states its measured exposure and the gate case follows
      (a static differential confines it, case 1; a refitted scale is a
      value change, case 2's fixed read). `R2` card.
    - **D.5.23 Wrong-bishop draws still misread through leaves the recogniser does not cover — `R2`.**
      D.5.15's falsifier (2026-11-03): with the correction kept off every
      verdict, Basilisk's four wrong-bishop KBP-K draws still read beyond
      ±75 in 7 of 28 searches from cleared tables (+83 to +103) and in 9 of
      28 after the two lost same-structure positions have trained the
      tables (up to +216), though no verdict leaf carries a correction
      (`analysis/draw_verdicts_2026-11-02.md` §5). `5k2/8/6KP/8/8/8/8/5B2 b`
      reads −110 at depth 14 in a process that searched depths 8 to 13
      first, with `ucinewgame` between, and within ±75 in a fresh searcher:
      some state survives `ucinewgame`, a second thing to read. Candidates: the
      correction on ordinary leaves just outside the recogniser's domain
      (the defending king off the corner square's neighbours, the pawn
      advanced), trained on the lost positions' shared structure keys
      (H17; Basilisk's producer guard, no training at a raw 0, is one
      form); the recogniser's domain itself; the generic evaluation of the
      near-corner positions. Screen: the probe extended to non-verdict
      leaves within a move of a verdict, raw against corrected, on the 63
      searches. Acceptance test: `wrong_bishop_draws_read_as_draws_with_cleared_and_trained_tables`
      in `tests/endgames.rs`, committed `#[ignore]`d. `R2` read, minutes.
  Evidence for items already here (H18–H21, H25): item 4 (NODE-2), the
  donor form read +3.3 ± 9.1 in Basilisk (2,000 games) and each of its
  consumers alone (IIR, the no-TT-move LMR term) threw a promotion fixture
  and a KR-K floor, so its screen includes an IIR-path fixture; item 3
  (NODE-1), a null move at cut nodes only read +5.0 ± 9.0 at `10+0.1` in
  Basilisk against Rarog's −10.4 at LTC (B-33), an interaction to watch if
  the null child is retyped; item 7 (the ply cap), bench position 11 is the
  same FEN as Basilisk's, whose 82× tree there is the position's
  conversion band (the root climbing 100–400 cp an iteration with
  full-depth fail-high re-searches), so check first whether seldepth 127
  is that band; item 8 (DRIVER-11), the time manager's effort share
  (`time.rs:337`) reads the node count too, and Basilisk's repair read
  +4.3 ± 5.9 bundled (time per move unchanged). For every tree-changing
  item, a deep-iteration cost screen with a per-depth watchdog (Basilisk's
  `deep_iteration_screen.py` caught a 23.5× tree that bench depth hid):
  positions chosen by a written rule, growth judged over the last three
  depths, the option wire proven live first, a late mate resolution not
  read as explosion.
  Evidence layer: counters and node counts choose and never accept; only the
  registered gate does (AGENTS, *Gating*).
- **D.6 Simplification programme from the engine review: tests into child files, the instrumentation behind probes, one score module, the board split, visibility, one list and one handle; about 860 source lines fewer and the largest file halved — `I1`, `R2` for the root driver.**
  Added 2026-11-01 from `analysis/engine_review_2026-11-01.md` §3.7 and §3.8
  (the B.2.0 recipe repeated: `src/` 29,190 lines, in-file tests 23%,
  instrumentation about 2,300 non-test lines and 437 of `negamax`'s 1,662; two
  dependency cycles; 657 public items, 77 unreferenced outside their file; the
  kernel within 1.3× the donor's after instrumentation, the root driver 630
  lines against 244). Every item is its own branch and squash, `I1` with the
  exact fingerprint on both slider backends, the suites, fmt and clippy;
  NPS-guarded means the allocation guard and one pooled-NPS read taken alone.
  Items that touch D.2's files (`tt.rs`, `shared.rs`, `threads.rs`,
  `thread.rs`) wait for D.2 to close; P12 runs inside D.3; the rest can run on
  their own branches while D.5's gates play. P0, P1 and P2 can start now.
    - **D.6.1 P0: the census as a tool — `I1`.** Commit the census as `tools/diag/arch_census.py` (planned) with a test
     (masked-source parser, `use` edge graph, public census, shape table,
     instrumentation attribution, duplicate finder); it reproduces the
     review's headline numbers at `a3dd950`, and E.1 re-measures with it.
    - **D.6.2 P1: in-file tests into child files — `I1`.** In-file tests into `<stem>/tests.rs` for the 13 files holding
     5,391 test lines (`node.rs` 3,609 → 2,462, `search/mod.rs` 2,977 → 1,979,
     `time.rs` 966 → 392, `king.rs` 924 → 521); test bodies untouched, counts
     equal before and after, release bytes identical. Neutral by construction.
    - **D.6.3 P2: one `score` module — `I1`.** One `score` module: the constants, one `MAX_PLY` (defined twice
     today), `MATE_BOUND`, `is_decisive`, `mate_in`; the nine inline threshold
     sites; ends the `eval` and `tt` cycle. Neutral.
    - **D.6.4 P3: geometry into one board module — `I1`.** Geometry into `board::geometry` (the `BETWEEN`/`LINE` tables and
     the six tables in `eval/pawns.rs`); one attacked-square predicate; one
     `ep_capture_is_legal`; one distance builder; a `TbPosition` method per
     Fathom entry point instead of eleven positional scalars;
     `legal_ep_capture_exists` returns `bool` (its always-`Ok` `Result` is an
     out-of-line call in `make_move`). NPS-guarded.
    - **D.6.5 P4: split `board/board.rs` — `I1`.** Split `board/board.rs` (one 2,066-line `impl`) into `fen`, `see`,
     `draw`, `checks`, `make`, before F.1 touches make/unmake. NPS-guarded.
    - **D.6.6 P5: delete `trace_decision!` — `I1`.** Delete `trace_decision!` (about 190 lines; no tool, document or
     analysis consumes it); retire condition: no open leaf registers a
     decision-trace read. Neutral.
    - **D.6.7 P6: the `diag` blocks into probe functions — `I1`.** Fold the 32 multi-line `diag` blocks into probe functions in
     `diag.rs` with empty `#[inline(always)]` twins (a trait object or a
     strategy type parameter was weighed and rejected); `negamax` 1,662 →
     about 1,270 with P5; the counter set equal at stride 1. NPS-guarded.
    - **D.6.8 P7: visibility follows use, second pass — `I1`.** Visibility follows use, second pass: the 32
     `pub(super)`/`pub(crate)` items that can be private and the 52 `pub`
     items referenced only inside `src/`; `pub(crate) mod diag`; `win32`
     narrowed. Neutral, compiler-checked.
    - **D.6.9 P8: delete the family masks — `I1`.** Delete the family masks (`eval/masks.rs`, the `Evaluator` fields,
     the `king.rs` hook and its tests, the two options; about 200 lines) once
     the maintainer confirms no open read sets `KingDangerMask` or
     `KingShelterMask`. `tune` builds only.
    - **D.6.10 P9: one `ArrayList<T, N>` — `I1`.** One `ArrayList<T, N>` under `MoveList` and `ScoredMoveList`; one
      `unsafe` fewer; keep the `MaybeUninit` init and its measured reason.
      NPS-guarded with the assembly `memcpy` check.
    - **D.6.11 P10: one `Compiled<T>` — `I1`.** One `Compiled<T>` in `infra`, a `runtime_params` cfg emitted by
      `build.rs` for `tune`, `texel` and test profiles, an `Evaluator` variant
      field; `texel` builds then read search parameters at run time (never
      measured for strength). NPS-guarded, read alone: C.12's folds were
      measured only inside bundles.
    - **D.6.12 P11: split `search/mod.rs`, after D.2 — `I1`.** Split `search/mod.rs` into `root`, `report`, `tb` after D.2
      closes. Neutral.
    - **D.6.13 P12: the adapter boundary, inside D.3 — `I1`.** The adapter boundary, inside D.3: the option types to
      `search/options.rs`, the parser and its notices to `src/uci/` (planned),
      `InfoSink::info(&InfoRecord)` so `format_score` and the bound and mate
      strings leave the search and the `info` line is built once; removes the
      `search` and `search_options` cycle. Neutral; `tests/uci_process.rs`
      pins the text.
    - **D.6.14 P13: one iterative-deepening driver — `R2`.** One iterative-deepening driver for single and MultiPV (53% of
      the MultiPV driver repeats the other): `R2` first on how aspiration
      state, `MateWatch`, the votes and the tablebase extension carry per
      line; `MultiPV 1` node-identical, `MultiPV > 1` judged by
      `tests/multipv.rs` and `multipv_syzygy.rs`; falsifier: a branch on the
      single-line path that costs NPS in the pooled read.
    - **D.6.15 P14: bench and WAC under one suite runner — `I1`.** Bench and WAC into `src/tools/` (planned) with one `run_suite`; delete
      `tests/board_performance.rs` (the third copy of the board workloads,
      gates nothing); `tests/common` (planned) for the two counting allocators and the
      repeated helpers; stale tool names (`AblationMask` in
      `fixed_budget_probe.py`, the legacy label in `feature_matrix.py`, the
      path in `conversion_audit.py`). Neutral.
    - **D.6.16 P15: the evaluation's shape — `I1`.** The evaluation's shape (EVAL-8): a `Score` accumulator and one
      `term!` macro emitting both halves and both trace sites, a per-colour
      context for the ten-argument term functions, `forward_one` and
      `pawn_attacks_of` used everywhere; fingerprint exact plus the `texel`
      reconstruction test.
    - **D.6.17 P16: comments — `I1`.** Comments (EVAL-7, TT-13, BOARD-10): the false seeded-zero notes
      on fitted terms, the lazy-gate rationale (the king map alone can
      subtract 637 cp against a 475 cp margin; the gate's support is
      RAR-E19/E20), the stale king-cap bound, the Apple cache-line paragraph,
      193 retired-number references. Comment-only commits.
    - **D.6.18 P17: lints — `I1`.** Lints (RUST-9, RUST-10): enable `cast_lossless`,
      `borrow_as_ptr`, `checked_conversions`, `manual_assert_eq` and
      `no_effect_underscore_binding` (65 mechanical edits, same code), the
      three plain `allow`s to `expect`, reasons on the twelve bare `expect`s,
      `clippy::allow_attributes` on; `suboptimal_flops` rejected because
      `mul_add` changes the clock's f64 results.
    - **D.6.19 P18: the node frame — `I1`.** The node frame (RUST-3, NODE-9): every `negamax` and
      `quiescence` copy has a 5.0–5.6 KB frame and calls `__chkstk` per node
      (the picker's 4,104-byte list and the ProbCut and quiescence lists); a
      design read first (per-thread ply-indexed list storage borrowed by slice
      splitting), stop under 0.5% NPS; a two-field `Copy` node-kind struct for
      the bool, `Move`, bool argument run; cold blocks outlined
      `#[inline(never)]`.
    - **D.6.20 P19: evaluation terms computed for a constant — `I1`.** Evaluation terms computed for a constant (added 2026-11-02
      from the Basilisk cross-check, §E): Basilisk stopped computing
      terms whose every weight was 0 for +2.35% NPS (BAS-P25). A census
      of Rarog's production terms whose fitted weight is 0 or folds to a
      constant (EVAL-11 names `hanging_rook` and `hanging_queen` at 1 and
      trapped bishops at 0.04% of rows) first, by reading; then the
      removals with the exact fingerprint, the `texel` reconstruction
      test and one pooled-NPS read. Terms E.4 or F.2.1 would refit stay.
  Held, with the reason: a stage split of `negamax` (R2 only if P5 and P6
  leave it unreadable); a unified legal move generator (about 300 lines, the
  highest NPS risk, no defect behind it); the table's local backend (D.2,
  F16); the controller thread and its epochs (B.3.5); any `inline(always)`
  pass (B.2.0 F24); the backup harness scripts (AGENTS); the three features
  themselves (all in use). Kept after review, with their records:
  `cpu_advice`, `crash_report`, `large_pages`, `wac`, the `diag` counters, the
  `tune` options and the `texel` trace in the engine crate.
- **D.7 Tablebase policy: probing depth/limits, WDL/DTZ in conversion, recogniser interaction — `R2`.** Root and interior probing depth and limits,
  WDL/DTZ use in conversion, interaction with the C.5 recognisers. Endgame-start
  cohort and conversion instrument decide.
  **Candidate, recorded 2026-09-10, not yet evaluated: replace vendored Fathom
  with `shakmaty-syzygy`.** A pure-Rust Syzygy prober (niklasf), v0.28.1,
  updated 2026-06-19, ~88k downloads, **GPL-3.0-or-later**, which matches
  Rarog's own licence. It would delete `vendor/fathom`, the `cc`
  build-dependency, `build.rs`'s C branch, the tier-dependent
  `TB_NO_HW_POP_COUNT` hazard that shipped 15 illegal `popcntq` in 2.3.0/2.3.1,
  the macOS `-fprofile-use` warning (RAR-P21) and the `unsafe extern "C"`
  block in `syzygy.rs` (four functions and one static) — and it would put probe code inside Rust's LTO
  and PGO for the first time. Against that: it would be the crate's **first
  runtime dependency**, `[dependencies]` being empty today, and it pulls
  `shakmaty` itself, a move-generation library we do not need and would have to
  convert positions into. Evaluated here rather than sooner because the benefit
  is near-zero Elo while the risk lands on C.5's truth source, and D.7 must
  validate probe semantics anyway. **Two questions to answer before it is
  proposed, not after:** (1) does it expose **DTZ** as well as WDL, since
  `syzygy.rs` ranks the root by DTZ through `tb_probe_root_impl`; (2) what does the `shakmaty`
  dependency actually weigh, in tree size and in conversion cost per probe. A
  semantic difference in DTZ or rule-50 handling would corrupt endgame evidence
  silently rather than announce itself, so parity against the C path on the C.5
  fixtures is the acceptance condition. **Not a candidate:** `fathom-syzygy`
  (malu) — it is a binding to the same C library, v0.1.0 untouched since
  2023-01-02, so it keeps every C problem and adds a dependency.
  **Engine review input, 2026-11-01** (`analysis/engine_review_2026-11-01.md`
  §3.6 and §3.4), each with its reviewer ID: the PV extension and
  `tb_root_board` gate on the loaded tables' size, not on `SyzygyProbeLimit`,
  which the root ranking and the in-search probes honour (PROTO-9; a 4-man
  fixture pair first, then `largest` from `shared.syzygy.largest`, case 1 with
  a static differential on the limited domain); the pre-search root ranking is
  one unabortable FFI call with no clock check and each PV-extension step is
  one more, unbounded with no clock (PROTO-10; rank per move with a deadline,
  `tb_probe_root_dtz` is declared and unused, or skip the ranking near the
  ceiling; case 2 if the ranking changes); Fathom maps tables lazily from any
  search thread and calls `exit(EXIT_FAILURE)` on a mapping failure, invisible
  to `crash_report` and with no `bestmove` (PROTO-11; a reason for the
  pure-Rust prober above); the file-count notice splits the path on `;` on
  every platform where Fathom uses `:` outside Windows, and the unit test pins
  the Windows form (PROTO-12: `std::env::split_paths`); `SyzygyProbeDepth`
  gates every probe where Stockfish gates only the largest loaded size and
  zeroes the depth when the limit exceeds the loaded tables (PROTO-14); root
  probes are documented not thread safe and run beside helpers' WDL probes in
  the no-clock extension (PROTO-15: run the extension after the helpers stop,
  or a threaded 4-man test); the join waits for a helper blocked in a table
  read (DRIVER-15: vote on the results received by a deadline); the in-search
  probe runs before the TT probe, so a probe is issued at nodes the table
  would have cut and `tbhits` is inflated (NODE-11: reorder,
  fingerprint-neutral without tables, an identity check with tables);
  tablebase-band scores skip the rule-50 downgrade mates get (TT-11); a
  non-ASCII `SyzygyPath` cannot load on Windows (PROTO-24). The dead
  `probe_root` and its `unsafe` site go in D.4.
  **Basilisk cross-check input, 2026-11-02** (`analysis/basilisk_crosscheck_2026-11-02.md` A4):
  `syzygy::initialize` returns early on an unchanged path (`syzygy.rs:157`)
  and caches a path whenever `tb_init` succeeds, also when it finds no
  files (`:191`), so re-sending the same `SyzygyPath` after the files are
  put in place loads nothing until the text changes. Cache only a path
  that loaded tables; a `uci_process` case with an empty directory and
  then a 3-man file; `I1`, fingerprint exact. Basilisk's in-search
  probe reorder (NODE-11 here) read +0.3 ± 3.2 at `10+0.1` with tables
  (8,000 games), a neutral repair, and cut table reads by 70% on six 7–8-man
  endings at depth 18, with 2,285 of 2,286 tablebase wins converted (H29).
- **D.8 Large pages on Linux: transparent huge pages for the big tables — `I1`, then `V`.**
  Added 2026-11-02 from the comparison with Reckless in D.2. `PageAllocator`
  (`src/large_pages.rs`) backs every allocation of at least `THRESHOLD`
  (512 KB) with 2 MB pages on Windows only: the transposition table and each
  search thread's correction, continuation and pawn tables. On Linux and
  macOS those allocations go to the system allocator. Linux backs anonymous
  memory with transparent huge pages unasked only when the kernel's mode is
  `always`; in the common `madvise` mode the tables stay on 4 KB pages, and
  the module's own case applies in full (about 100 MB read at random against
  a second-level TLB that maps 8 MB of 4 KB pages). Reckless maps its table
  and its big histories with `mmap` and `madvise(MADV_HUGEPAGE)` on Linux.
  - **Contract.** On Linux the page allocations are 2 MB aligned and advised
    with `madvise(MADV_HUGEPAGE)`, behind the same `LargePages` option;
    nothing is locked and no privilege is needed. `large_pages()` reports
    what the kernel allows (the mode in
    `/sys/kernel/mm/transparent_hugepage/enabled`, `never` reading as off).
    The tables, their contents and the search are unchanged: `bench 13`
    exact on every platform, and the CI's Linux job builds and tests it.
  - **Measurement.** The one-thread NPS read by PROCESS's method on a Linux
    host, with the option on and off in one binary and the kernel's mode
    recorded (WSL2 on the match host is the host available; its mode is part
    of the record), and a four-thread cell. A behaviour-neutral speed change
    closes on that read.
  - **Order.** On its own. RAR-M79 (2026-11-02) read large pages off 2.7%
    higher in R than on, inside noise, in a session where the four-thread
    speed loss was absent; whether they take part in the slow host state
    is unread (`analysis/d2_audit_2026-11-02.md`). Transparent huge pages
    lock nothing, so the Linux read stands whatever that answer is, and
    the scope stays every allocation the Windows path takes.
  - **macOS: not pursued.** Rarog ships macOS for Apple silicon only
    (maintainer decision 2026-11-02: no Intel macOS build, universal or
    separate). Apple silicon has no userspace superpage request (it fails
    on an M1, by a report on Apple's developer forums and OpenJDK's and
    MySQL's macOS notes; not an Apple document), and its base page is
    already 16 KB. The 2 MB superpages macOS offers exist on Intel Macs
    alone. On macOS `large_pages()` stays as it is.

## Phase E — Classical checkpoint and release

- **E.1 Attribution checkpoint: B.2.0 review re-run on the B.9/C.11 heads; STC, `10+0.1`, 4T against 2.3.2 and the B.9/C.11 heads; maturity checklist — `V`.** Re-run the B.2.0 architecture review
  on the B.9 and C.11 heads first. Final head against 2.3.2 and against
  the B.9 and C.11 heads at STC, `10+0.1` and 4T; attributed Elo per programme
  from the accepted SPRTs; deficit meters; NPS; the maturity checklist
  (family map without unknown rows, every slot with a fitting instrument,
  every accepted representation reconstructing through `EvalTrace`).
  **The architecture half of the re-run was done on 2026-11-01**
  (`analysis/engine_review_2026-11-01.md` §3.8, the B.2.0 recipe repeated on
  the D head; its census becomes `tools/diag/arch_census.py` (planned) under D.6 P0).
  E.1 keeps the attribution re-run and the maturity checklist and re-measures
  the architecture counts with that tool at the release head.
  Add a tablebase arm (H30): the head against the last release with Syzygy
  loaded, since the search has its own probes and CCRL testers use tables;
  Rarog's last games with tables predate D.1 (RAR-S97, 2026-10-01).
- **E.2 Target gate: ≥50% against Critter 1.6a, Houdini 3, Rybka 4.1 and Fritz 16 at 1T and 4T (Rybka 4.1 replaced Rybka 4, 2026-09-19); the binding arm is 1T — `V`.** The pool measurement defined in section 1, at 1T
  and 4T. Met, or not met with the measured shortfall per engine recorded.
  **The binding arm is 1T:** RAR-M46 measured 4T as the easier arm for three
  of the four targets, by 26 to 75 Elo, so Phases B and C are judged against
  the 1T column.
  **Contingency (C.13.7, 2026-10-16):** if the 1T gate is missed against one
  engine by 5 Elo or less, C.13.7's contempt card (form 1, a search draw
  offset) is the registered lever, measured in this gauntlet with and
  without it (`analysis/c137_contempt_card_2026-10-16.md`).
  Before the gauntlet (H31): `Assert-ColosseumRunFaults`
  (`tools/harness_common.ps1:960-1000`) voids a run on any engine's non-time
  fault, so one crash by Houdini or Rybka voids the pool; Basilisk keeps
  zero tolerance for its own build and accepts a foreign engine's faults
  up to 1% of that engine's games, quoted in the manifest.
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
  **First step, from the 2026-11-01 review (EVAL-3 in
  `analysis/engine_review_2026-11-01.md`):** the king-danger inputs are
  computed on plain attack maps, while the donor's bishop attacks see through
  queens, its rook attacks through queens and own rooks, and a pinned piece's
  attacks are cut to the king line (`evaluate.cpp:269–274`); C.3's fixture
  came from a donor patched to plain maps, so it proves nothing about the
  donor as played, and RAR-E72 put the king lead in the four inputs that read
  those maps. Zero games: stack `tools/diag/patches/sf9587_plain_maps.patch`
  on the sub-term patch and read RAR-E72's (B) with each bundle; frozen rule:
  x-ray and pin lines are the carrier if the plain bundle's (B) is at most
  half of +0.194; if so, the unit is an attack-map producer change that moves
  mobility, threats and king danger together (gate 3 with a refit), otherwise
  the lead is weights, cap or structure and the rebuild above is the owner.
  Also for the refit: the bishop pair's three representations fold exactly
  (EVAL-11), and a second-difference penalty on the mobility tables and the
  pawn PST (EVAL-13). If the rebuild moves the endgame king danger, read the
  phase with it (H32): with piece-count phase weights the middlegame share
  in simplified endings is far above the donor's (double rook 33% against
  10%, R+B against R+N 25% against 3%, Q against Q 33% against 13%).
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
  (`analysis/gyatso_read_2026-09-26.md`). The 30–60M is a guess (H33): size
  the corpus from effective independent rows and a learning curve on the
  first slice, as Basilisk's C.2.2 does, before the maintainer spends the
  hours.
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
      Before the fit (added 2026-11-02, `analysis/basilisk_crosscheck_2026-11-02.md` §E):
      `tools/texel-tuner` was ported from Basilisk, whose audit found five
      tuner defects; check each here by reading: Adam with no convergence
      criterion (travel bounded by learning rate × epochs); a bake that
      writes every differing slot with no provenance or completeness
      check (a stale dump reverted 42 members in Basilisk's dry run);
      hidden clamps outside any manifest; a linear stage that can break
      `safety_table`'s monotonicity, after which a later `clamp` has
      `lo > hi`; a coverage audit that credits a dead term. Also (H34, H35):
      `fit_k`'s ternary search stops silently at its [0.5, 2.5] bounds
      (`tools/texel-tuner/src/main.rs:1046-1060`), which deep labels can
      reach; and the passer's rank is carried by about seven collinear
      coordinates (the pawn PST, `passed_mg/eg`, the per-rank supported
      bonus and the four stop/path terms, some fitted near zero or
      negative), a collinear class for the fit's manifest.
- **F.3 Trainer hardening and baseline nets, two seeds per configuration — `I2`, then `V`.** Deterministic
  pipeline, two seeds per configuration, validation selects, frozen test
  reports once.
- **F.4 Scalar integration: `quantised.bin` contract, integer-exact conformance, HCE fallback — `I2`.** `quantised.bin` contract, integer-exact
  conformance against the trainer's reference evaluation, clean HCE fallback.
- **F.5 Incremental and SIMD: same-net parity on every move type, tiers, pooled NPS attribution — `I2`, then `V`.** Same-net incremental parity
  on every move type, SIMD tiers (AVX2, PEXT builds, ARM NEON), scalar
  reference retained, pooled-PGO NPS attribution. Priors against (H36,
  BAS-P26): slider tables compressed and expanded with `pdep` −0.98%
  [−1.19, −0.78] (the flat tables' hot lines stay in cache), `-mtune=znver3`
  −0.95%, FMA/F16C/MOVBE +0.02%.
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
  (196 of Stockfish's 249 ISA-specific lines are NNUE inference). Basilisk
  built and shipped the same design on 2026-10-29. It found that the relink
  costs nothing only once every tier function is 64-byte aligned (−0.58%
  unaligned, from placement alone), that symbols leak across tiers through
  LTO, and that renaming the kept post-LTO objects of each tier's ordinary
  PGO build avoids the profile trap. These are recorded as leads in that
  document's section 9, and they fire no trigger. Also from Basilisk (H37,
  H4): an SSE4.1-plus-POPCNT tier took most of AVX2's gain on pre-Haswell
  CPUs (+5.6% against +6.0%, indicative); its universal binary is checked
  under Intel SDE (each emulated CPU runs the right tier and reproduces
  bench, a forced tier it cannot run is refused); a statically linked
  Linux asset removes the glibc floor (Basilisk's +5% is one WSL read, so
  a pooled NPS read decides).
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
