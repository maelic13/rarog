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
starting point. Phases are lettered (A–H) so that no new identifier collides
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
| Fingerprint | `bench 13` **9,077,016 / EBF 2.481** at engine source `ba1d2cd` (D.6.8: each position counted once, RAR-S124; before it 11,477,040 / EBF 2.537 at `aee0b3b`, on `master` as `89b4a92`, PR #17, D.6.7: every line bounded at twice the iteration's depth, RAR-S122; before it 13,038,236 / EBF 2.540 at `ee0b3e4`, D.6.2: quiescence and ProbCut generate the quiet queen promotion, RAR-S108; before that 11,946,003 / EBF 2.547 at `72c4e60`, D.6.1: castling classed quiet, RAR-S107; before that 15,625,852 / EBF 2.563 at `1450340`, D.5.5: `Hash` at any size moved every slot, RAR-S106; before that 14,749,043 / EBF 2.564 at `3a16fbb`, D.6.15: a draw verdict takes no correction; before that 13,987,223 / EBF 2.565 at `9fb525e`, C.10's search tune on the frozen evaluation; `rustc 1.100.0` since 2026-11-13, RAR-P58) | RAR-S105, RAR-P58 |

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
| D.7 search shape | 22.5 ± 7.9 at equal clock against Phase B's donor as it is (RAR-O08, match 5); the shares of tree shape, clock use and kernel speed are unread | 0–70 | conditional on D.7.2's reads or the maintainer's decision: the kernel in the donor's whole form, seeded from the hybrid's configuration and fitted. Equal strength is accepted (maintainer decision 2026-11-13), so the band starts at 0 |
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

7c. **A tune is one run, read at checkpoints, and stopped when its
   gradients are gone.** Adopted 2026-11-13 (maintainer decision) from
   Basilisk's B.5.6 and B.5.8, replacing the blocks this rule held from
   2026-09-25 (RAR-S75, RAR-S78, RAR-S82–S83) and its 2026-10-20 shape
   (RAR-S104); the record is `analysis/spsa_method_adoption_2026-11-13.md`.
   - *Shape.* 30 games an iteration on 15 slots, with a registered ceiling:
     6,000 iterations (180,000 games) unless the registration argues
     another. The perturbation is two steps (the config's `step`, the b43
     step) where the seed has two steps of room inside its range,
     otherwise the largest from one to two steps the room allows; a
     coordinate with under one step of room keeps one step and is named at
     registration. `r_end` is 0.00155. `tools/spsa_config_to_colosseum.py
     <group> --rule7c --ceiling N` writes the files.
   - *Checkpoints.* Iteration 2,000, then every 1,000.
     `tools/colosseum.ps1 -Mode spsa -Ceiling N` plays to each with the
     runner's `--stop-after-iteration`, and `tools/spsa_read.py` reads the
     last 1,000 iterations there.
   - *The test.* For each coordinate, `z = Σ δ·D / √(Σ D²)`: δ is its
     perturbation's sign at an iteration, regenerated from the run's seed,
     and D the plus side's wins less its losses. With no gradient z has
     mean 0 and variance 1 exactly, because the tuner drew the signs. The
     window is **live** if any |z| passes the familywise 5% line for the
     surface's coordinate count, or if `Σ z² − n` is at least `2·√(2n)`;
     otherwise it is **quiet**.
   - *The rule.* Quiet: the tune stops, and theta is that checkpoint's
     rounded centres. Live: the next 1,000 iterations run. The maintainer
     may stop at any checkpoint (a file named `STOP` in the run directory),
     and theta is then that checkpoint's centres. A run stopped between
     checkpoints is not theta; it resumes. A run with engine, time or
     infrastructure faults stops for the maintainer, who may accept them
     once analysed.
   - *Rails.* A centre within one perturbation of a bound, its z pointing
     outward past the familywise line, stops the tune for a new
     registration of the range. A weaker outward push (z of 2 or more) is
     reported and stops nothing.
   - *The record.* Each read appends every coordinate's z, the tune's own
     read of its gain, the stop's floor in Elo and a frozen forecast for
     the next 2,000 iterations. The forecast decides nothing until three
     scored forecasts have each fallen within twice the combined SD of the
     forecast and of the gain read it is scored against.

   Quiet means no gradient above the stated floor, not convergence. A tune
   is judged by its gate, not by its read, and a stop is never read from
   net travel or a count of movers. The 2,000-iteration minimum is there
   because a gradient can appear late (Basilisk's B.4.6 read quiet over its
   first 1,000 iterations and live over its second). In the blocks' shape a
   quiet read at 82 coordinates could not exclude about 11 Elo, which is
   what RAR-S78's restart found; the doubled perturbation exists to lower
   that floor.

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
   - **Canaries:** a regression diagnostic, with the standing of every
     zero-game floor above. The floor is that no oracle-anchored canary the
     baseline passes is lost, where passing means stable at no more than
     the anchor plus two plies and solved at 100k nodes. New passes are
     recorded beside the losses. A later settle with the position still
     solved at 100k is a stability shift: it gets a written cause from the
     per-depth moves and never holds a candidate from its gate. A lost
     tactic, a canary the baseline solves at 100k and the candidate does
     not, stops a candidate before any game only together with WAC at 100k
     under its floor (RAR-M85; maintainer decision 2026-11-11, delegated
     to the agent's review on condition of a stated and valid reason).
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
    equal-time gap against the frozen Stockfish-search oracle (B) and, from
    RAR-O08 on, against Reckless's search on Rarog's evaluation, the
    stronger reference (the head +59.8 ± 8.1 over the first and −22.5 ± 7.9
    against the second, 2026-11-13), the
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
class until they open. The rows are in ascending order, which is the order
of work (maintainer decision 2026-11-03): a leaf added later takes the next
free number of the step it belongs to, and one that must run earlier is
placed by renumbering, never by moving its row. The first open row is
GUIDE's Now (`guide_board.py`). E.3.2 and E.3.3 ran ahead of Phase D by
maintainer decision 2026-11-04; both are done. On 2026-11-05 (maintainer
request) the roadmap was checked so that the numbering is the order of work
from here to the end: no step waits for a later one, and no work is listed
twice (HISTORY's number map holds what moved).
On 2026-11-13 (maintainer decision, after RAR-O08) the order of work left
the numbering in two places, and the rows below are in the order of work.
D.6.8 ran first (done 2026-11-13), because D.7.2's equal-node match needs
one count per position. D.7, the search-shape step, then runs ahead of every other open
D.6 leaf; those leaves keep their numbers, which tests and records cite,
and each is asked first whether it is still an issue on the kernel D.7
leaves behind. 2.7.0, an unscheduled release with no step of its own, was
cut 2026-11-13, before D.7, on a registered read against 2.6.0 (E.3).
The Basilisk check for new ports, research only, follows the cut and
precedes D.7 (maintainer decision 2026-11-13); done 2026-11-13
(`analysis/basilisk_check_2026-11-13.md`: D.6.29, D.6.30, evidence for
D.6.28, and inputs to D.7.0).

| Leaf | Workflow state | Class | Current decision |
|---|---|---|---|
| D.7.0 | RESEARCH | R3 | The inventory of every departure from Reckless `0756957`, the seed table from the hybrid's configuration, the parity design, RAR-O09's registration |
| D.7.1 | READY_FOR_IMPLEMENTATION | I1 | `sprt.ps1` takes a node budget and a move time per engine, each with a wire smoke |
| D.7.2 | RESEARCH | V | RAR-O09's three reads after D.6.8; the rebuild opens at +10 or more at equal nodes, otherwise the maintainer decides after consultation |
| D.7.3 | RESEARCH | I2 | Opens with D.7.2's decision |
| D.7.4 | RESEARCH | I2 | Opens with D.7.2's decision; parity before any game |
| D.7.5 | RESEARCH | V | The unfitted paired run of 2,000 games |
| D.7.6 | RESEARCH | V | The fit as one rule 7c run read at checkpoints; kept only if fitted over unfitted passes the gate PROCESS's SPSA procedure chooses |
| D.7.7 | RESEARCH | V | The kernel against the head on the loss-permitting bracket; `10+0.1` and 4T reads |
| D.7.8 | RESEARCH | R2 | Each known problem asked "still an issue?" on the new kernel |
| D.7.9 | RESEARCH | V | Adoption by maintainer decision on D.7.7 and D.7.8; meters; the old kernel deleted |
| D.6.9 | RESEARCH | R2 | Screen before any implementation |
| D.6.10 | RESEARCH | R2 | Screen before any implementation |
| D.6.13 | RESEARCH | R2 | After D.7: the donor's aspiration form is part of a rebuilt kernel; re-evaluate first, close if moot |
| D.6.16 | RESEARCH | R2 | After D.7: quiescence is rebuilt in the donor's form; re-evaluate the gain pruning on it first |
| D.6.17 | RESEARCH | R2 | Card: the wrong-pick branch fired on D.6.3's candidate (a transposing rook kept by root order); Basilisk's root rule is the prior, the ignored transposing-rook anchor the acceptance test; after D.7, asked first by D.7.8 (its hold on 2.7.0 lifted 2026-11-13) |
| D.6.20 | RESEARCH | R2 | Screen before any implementation; a measured instance: the won-ending clock test is red since `ee0b3e4` (9.5 s against 8 s), ignored as its acceptance test; after D.7, asked first by D.7.8 (its hold on 2.7.0 lifted 2026-11-13) |
| D.6.21 | RESEARCH | R2 | Screen: does a one- or two-ply search already read the 88 above 0; minutes |
| D.6.22 | RESEARCH | R2 | Screen: a per-site scale that keeps the draws near 0 and the wins above it; minutes |
| D.6.23 | RESEARCH | R2 | Screen: raw against corrected at the non-verdict leaves near the verdicts in the 63 fixture searches; minutes |
| D.6.24 | RESEARCH | R2 | Screen: a narrowed or discounted fortress on the same enumeration, draws kept at 0 and wins lifted; minutes |
| D.6.25 | RESEARCH | R2 | Screen: does any search at the gate's budget pick the bishop promotion, or score the same-coloured pair above a draw at the root; minutes |
| D.6.26 | RESEARCH | R2 | Screen: the pawnless one-minor-ahead classes against Syzygy over an enumeration, and their exposure in gate games; minutes |
| D.6.27 | RESEARCH | R2 | After D.7: the donor has no IIR; re-evaluate first, close if moot |
| D.6.28 | RESEARCH | R2 | Screened 2026-11-13 on RAR-M87's games: the field gap sits in the low-clock band the horizon rule shapes; next, the horizon against a field, registered, forfeits its invariant |
| D.6.29 | READY_FOR_IMPLEMENTATION | I1 | Screened 2026-11-13: 19.2% of replayed searches hold a misread stalemate; the narrow repair, a root-change replay, then case 3 `[-5,5]` |
| D.6.30 | READY_FOR_IMPLEMENTATION | I1 | Screened 2026-11-13: 3 of 12,520 replayed searches play another move after a proven win (all games won); keep the proven win, case 1 if the fingerprint holds |
| D.8.1 | READY_FOR_IMPLEMENTATION | I1 | No dependency |
| D.8.2 | READY_FOR_IMPLEMENTATION | I1 | No dependency |
| D.8.3 | READY_FOR_IMPLEMENTATION | I1 | No dependency |
| D.8.4 | READY_FOR_IMPLEMENTATION | I1 | Own branch; exact fingerprint on both slider backends |
| D.8.5 | READY_FOR_IMPLEMENTATION | I1 | Own branch; exact fingerprint on both slider backends |
| D.8.6 | READY_FOR_IMPLEMENTATION | I1 | Own branch; exact fingerprint on both slider backends |
| D.8.7 | READY_FOR_IMPLEMENTATION | I1 | Own branch; exact fingerprint on both slider backends |
| D.8.8 | READY_FOR_IMPLEMENTATION | I1 | Own branch; exact fingerprint on both slider backends |
| D.8.9 | READY_FOR_IMPLEMENTATION | I1 | Own branch; exact fingerprint on both slider backends |
| D.8.10 | READY_FOR_IMPLEMENTATION | I1 | Own branch; exact fingerprint on both slider backends |
| D.8.11 | READY_FOR_IMPLEMENTATION | I1 | Own branch; exact fingerprint on both slider backends |
| D.8.12 | READY_FOR_IMPLEMENTATION | I1 | Own branch; exact fingerprint on both slider backends |
| D.8.14 | RESEARCH | R2 | `R2` first on how aspiration and MultiPV share the driver |
| D.8.15 | READY_FOR_IMPLEMENTATION | I1 | Own branch; exact fingerprint on both slider backends |
| D.8.16 | READY_FOR_IMPLEMENTATION | I1 | Own branch; exact fingerprint on both slider backends |
| D.8.18 | READY_FOR_IMPLEMENTATION | I1 | Own branch; exact fingerprint on both slider backends |
| D.8.19 | READY_FOR_IMPLEMENTATION | I1 | Own branch; exact fingerprint on both slider backends |
| D.8.20 | READY_FOR_IMPLEMENTATION | I1 | Its census has no dependency |
| D.9 | RESEARCH | R2 | Tablebase policy, with the review's probe-path items (PROTO-9 to 15, DRIVER-15, NODE-11, TT-11) and the cached empty `SyzygyPath` |
| D.10 | READY_FOR_IMPLEMENTATION | I1 | Linux transparent huge pages for the page allocations, behind `LargePages` (macOS stays Apple silicon only, by maintainer decision 2026-11-02); stands on its own: RAR-M79 (2026-11-02) read no large-page cost in a healthy host state, and their part in the slow state is unread (audit 2026-11-02); exact fingerprint, a Linux NPS read |
| D.11 | RESEARCH | R2 | The originality disclosure, moved from E.3.0; its hold is lifted for 2.7.0 (maintainer decision 2026-11-13) and stands for the release after it |

## Phase D — Clock, threads, robustness

Numbering: on 2026-11-01 the tablebase leaf D.4 became D.7, and D.4, D.5 and
D.6 were added from `analysis/engine_review_2026-11-01.md`. On 2026-11-03
the numbered items of D.3 to D.6 became sub-steps with their numbers kept:
item N of D.x is D.x.N (D.3's eight defects are D.3.1 to D.3.8; D.3.9 to
D.3.11 are new). Later on 2026-11-03 (maintainer decision) the steps from
D.4 on were renumbered so that the numbering is the order of work, every
sub-step number kept: the instruments D.9 became D.4, the repairs D.4
became D.5, the strength candidates D.5 became D.6, Linux large pages D.8
became D.7, the simplification programme D.6 became D.8 and the tablebase
policy D.7 became D.9. The tracked documents carry the new numbers;
HISTORY's number map resolves the old ones in commit messages and in
records from before 2026-11-01, where D.4 is the tablebase leaf. On
2026-11-05 five strength candidates left D.6 (D.6.11 to E.4, D.6.12 to G.2,
D.6.14, D.6.18 and D.6.19 to H.2) and D.8.17 merged into D.5.16; their
numbers are retired, not reused.
On 2026-11-13 (maintainer decision) the search-shape step took D.7, large
pages on Linux moved from D.7 to D.10 and the originality disclosure
moved from E.3.0 to D.11; D.7 runs before the open D.6 leaves, which keep
their numbers.

A finished step keeps its heading over a short summary; its full
text, and a finished parent's sub-steps, are in `docs/archive/PLAN-phaseD-done-2026-11-04.md`.

- **D.1 Time management: audit against the ADR-0065 checklist, soft/hard bounds with node-fraction multiplier, forfeit margin; SPRT `[0,3]` — `R2` investigation, `I2`/`V` sub-steps, DONE 2026-10-26.**
  Landed as `703fe78` (PR #12, "Rework time management"): the clock
  audited against Manta's ADR-0065 checklist and reworked, so the engine
  stops at a proved mate, keeps more reserve when its clock runs low and
  weighs how settled the best move is; the root fail-high cascade in won
  endings is ended by a stop at twice the target, which ends every probed
  stall. Closed by maintainer decision 2026-10-26 with the residue
  accepted: `d1`'s game residue is a third of 2.6.0's at `10+0.1` with no
  measured cost (RAR-R28); the single-window fix is held with its retry
  trigger, a measured cost. Full record, D.1.1 to D.1.6: `docs/archive/PLAN-phaseD-done-2026-11-04.md`.
- **D.2 Lazy SMP quality at 4T/8T: diversity, shared TT and correction, soft-stop voting; 4T SPRT `[0,5]`; RAR-M78's deficit verdict was measured in a packed slot and is superseded (RAR-M82), the sibling-free comparison is RAR-M83 — `R2` investigation, `I2`/`V` sub-steps, DONE 2026-11-04.**
  Landed in `c69761d` (PR #13). The in-game four-thread slow state (four
  threads at 2.5 to 2.7 times one) was Windows packing each four-thread
  side on SMT siblings inside Colosseum's auto slot, not Rarog's code: any
  engine keeps about 0.63 of a thread beside its sibling (RAR-P54), and
  with one logical CPU per physical core Rarog's four threads run at 4.03
  times one (RAR-M82); that placement is the policy for every rated
  multi-thread read (PROCESS, *Harness*). RAR-M83 read NOT DEFICIENT:
  four threads are worth 1.37 doublings of Rarog's own time against
  Reckless's 1.49 and Stockfish's 1.54, so the four-thread target closed
  `NO_CHANGE`, its open items G.1's priors (the four-thread holds, H5's
  result selection, the 2T to 4T shared-search loss of RAR-P49 and
  RAR-P50, helper depth staggering, an engine-side placement hint).
  `e057d84` put the multi-thread kernel under PGO training. Full record,
  D.2.1 to D.2.7: `docs/archive/PLAN-phaseD-done-2026-11-04.md`.
- **D.3 Engine lifecycle and protocol robustness; `src/uci/` (planned) move; score normalisation research card (`analysis/uci_info_review_2026-09-16.md` item 6); zero crashes over pool tournaments — `I1` for the eight review-confirmed defects (2026-11-01), then `R2` for the cards, DONE 2026-11-04.**
  Landed in `c69761d` (PR #13). The review's eight defects are repaired,
  each with its process test (D.3.1 to D.3.8): a zero, negative or
  unreadable clock moves at once; `setoption` no longer blocks `stop`;
  the end of input ends an infinite or ponder search; a closed stdout or
  a line that is not UTF-8 no longer ends the engine; `stop` during
  `bench` or `wac` says so; a refused `Hash` is reported (on every `Hash`
  sent since D.5.24); a one-move `searchmoves` searches to its depth.
  D.3.9: ordinary scores print below every tablebase win; a `position`
  that cannot be read lets a running search answer before the critical
  exit; a helper's panic and a terminal root under `go infinite` are
  tested; the protocol moved to `src/uci/` behind one option table, the
  search handing it records (D.8.13); score normalisation closed
  `NO_CHANGE`, to be fitted per network in F.6. The load flakes are
  repaired (D.3.10, the `isready` handshake; D.3.11, the fastest of three
  cold searches). Full record, D.3.1 to D.3.11: `docs/archive/PLAN-phaseD-done-2026-11-04.md`.
- **D.4 Instruments from the Triumviratus review: the measurement-integrity reads, a search-path perft, pruning and reduction pricing, the loss table by cell, an identical-tree counter read — `R2`, then `I1`/`I2`/`V`, DONE 2026-11-06.**
  Landed in `7d5fc06` (PR #15). The instruments the repairs and candidates
  use, each validated on a
  planted case; none changes the tree or a gate, and their readings screen,
  never accept. D.4.1: wall-clock overhead is clean (`go movetime` overshoots
  by under 1 ms). D.4.2: Colosseum's SPSA schedule is clean. D.4.3: a
  search-path perft through the staged picker and the search's make and
  unmake runs in `cargo test --release` (`src/search/path_perft.rs`), and five
  planted faults fail it. D.4.4: pruning and reduction pricing, the `pricing`
  feature read by `tools/diag/pricing_table.py`, with the main search's nodes
  unchanged and fifteen of sixteen labels plant-validated; razoring's label
  stays out of every screen until a plant harmful on its own passes, read
  first by the step that wants it
  (`analysis/pruning_pricing_card_2026-11-04.md`). D.4.5: the loss-table tool
  `tools/diag/loss_table.py`, its run on the C head withdrawn before any
  reading and the read moved to H.3
  (`analysis/loss_table_card_2026-11-05.md`). D.4.6: closed `NO_CHANGE`, no
  hardware counter beats the pooled builds, so the pooled read stays the
  speed check, and an unexplained two-state split between processes is H.4's
  lead (`analysis/identical_tree_counters_card_2026-11-05.md`). Full record,
  D.4.1 to D.4.6: `docs/archive/PLAN-phaseD-done-2026-11-04.md`.
- **D.5 Behaviour-neutral repairs from the engine review: the table handle and the helper lifecycle, the fixed-depth vote, `Hash` at any size, dead code, test and CI gaps — `I1`, DONE 2026-11-08.**
  Landed in `b202174` (PR #16, "Repair the engine review's findings
  without changing play"). Every repair reproduces the fingerprint on both
  slider backends except D.5.5, which moved it by design (RAR-S106).
  D.5.1 to D.5.5: helpers hold a table handle they release after each job
  and `readyok` waits for their start-up; a `Hash` change at more than one
  thread fills the table once; a depth limit and `go mate` are answered by
  the main thread's rule; `Hash` is the size set. D.5.6 to D.5.18:
  quiescence has one body and the root narrows its move list in place;
  dead Syzygy code is gone; the `unsafe` sites fall from 24 to 17, the
  floor since (maintainer decision 2026-11-08); the history tables and
  Fathom's root buffer are zeroed on the heap; Fathom's FFI layout is
  checked; the PEXT tier is refused without BMI2 and tested in CI, as are
  the texel and tune tests; the KPK bitbase is built at compile time; UCI
  text is a `UciMove` only the board turns into a `Move`; misleading names
  and comments are fixed, the review's missing tests written and a draw
  record corrected. D.5.19 read every hard draw verdict against Syzygy.
  D.5.20: every CI job has a timeout, the runners are pinned, each
  workflow has one aggregating check, a PGO build whose kernel went
  untrained fails, the `unsafe` preconditions are checked on stable in
  place of a nightly Miri job, and a new ledger entry and its index row
  state the same disposition. D.5.21: Basilisk's behaviour tests run
  against Rarog, with the picker's contract under random histories, every
  search parameter shown to move what it tunes, exact test floors and the
  promotion fixtures' scores. D.5.22 adopted Basilisk's measurement
  methods (three declined with retry triggers), D.5.23 clamps the
  correction bonus, D.5.24 refuses a `Hash` above physical memory. What
  the tests found wrong in play is D.6.24 to D.6.26. Full record, D.5.1 to
  D.5.24: `docs/archive/PLAN-phaseD-done-2026-11-04.md`.
- **D.6 Strength candidates from the engine review: kernel and evaluation deviations from the donor, each behind a zero-game screen and a registered gate — `R2` cards, then `I1`/`I2`/`V`.**
  Added 2026-11-01 from `analysis/engine_review_2026-11-01.md` (§3.1, §3.2,
  §3.5). Each item is an unrecorded deviation from the donor's shape with a
  plausible mechanism; none is a measured strength defect, so each enters at
  `RESEARCH` and crosses `READY_FOR_IMPLEMENTATION` only with its screen read
  and its prediction frozen (AGENTS, *Classify the work*). Rules: no
  implementation before its screen; one candidate per gate; rule 6 stops the
  leaf after two rejections in a row; a screen that reads nothing closes the
  item `NO_CHANGE`. The fingerprint moves with every one, so none rides in D.5
  or D.8. Item 15, a correctness repair, is done. The first two are
  classification repairs with the cheapest reads; items 3 and 4 are
  screened together and gated apart. Items 11, 12, 14, 18 and 19 left on
  2026-11-05: the piece-term definitions to E.4's refit, `Hash` at any size
  to G.2, and the low-priority notes, Stockfish 19's mechanisms and the
  root and in-check reductions to H.2 under Phase H's rule.
  D.6.1 to D.6.7 landed in `89b4a92` (PR #17, "Fix castling and queen
  promotions in the search and bound long lines"). Their full text stays
  here, not in the archive: D.7.8 and the items after D.7 come back to
  them (maintainer decision 2026-11-13).
  **Order, 2026-11-13 (maintainer decision):** D.6.8 runs first, then D.7.
  The other open items wait for D.7 and none is dropped. After D.7.8 each
  is asked first whether it is still an issue on the kernel D.7 leaves
  behind: an item the rebuild made moot closes with that reason, one whose
  defect remains keeps its card, and a form that lost its gate on the
  combined kernel (D.6.3, D.6.4, the fail-soft cluster of RAR-R19 to R21,
  one-retry aspiration) is re-opened or added where D.7.8 finds its
  problem still there.
    - **D.6.1 Castling is neither quiet nor noisy — `R2`, DONE 2026-11-10.** (NODE-3, BOARD-1):
     `is_quiet_move` is `flags <= DOUBLE_PUSH` while castling has flags 2 and
     3, and `is_noisy` is capture or promotion, so in the move loop castling
     skips late-move, futility and history pruning, takes the noisy SEE
     threshold and LMR base, reads noisy history, and when it fails receives
     `update_noisy` maluses into a slot the quiet picker never reads, while a
     successful castle is credited in the quiet tables; the noisy slot for the
     castling king drifts to its floor and adds about a ply of reduction. The
     predicate dates from the initial commit; the donor classes castling quiet
     with one predicate. Screen: an agreement test, `is_quiet_move(mv) ==
     !is_noisy(mv)` over legal moves (NODE-14; it fails on castling today and
     lands with the fix), and the four castling noisy slots after `bench 13`
     at stride 1 (expected strongly negative). Fix: define quiet once (`!is_capture() && !is_promo()`). Gate:
     case 2, a 2,000-game harm read, as a classification repair expected near
     zero. Screen read 2026-11-09 (RAR-S107): the slots at or near their floor,
     1 in 381 moves in the loop, between 49% and 71% of them decided differently,
     castling legal in 95% of gate games, so no closing rule fired; the repair is
     `72c4e60` on `d6`, kept by its harm read on 2026-11-10 at nElo −4.9 ± 15.2.
    - **D.6.2 Quiescence cannot see a non-capturing queen promotion — `R2`, DONE 2026-11-10.** (BOARD-2): pawn
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
     Prior (2026-11-03, TS-14 in
     `analysis/triumviratus_review_2026-11-03.md`): Triumviratus's passed-pawn
     push to the seventh searched in quiescence read −0.77, +1.1, −1.8 and
     +0.23 in four runs, and its quiet hash move in quiescence −1.05 ± 3.92.
     Screen read 2026-11-10 (RAR-S108): in endings 5.9% of quiescence capture
     generations had a quiet queen promotion and 3.4% a safe one that could
     raise alpha, legal in 54% of gate games, so no closing rule fired; the
     push is `ee0b3e4` on `d6` (bench 13,038,236), accepted by its SPRT
     `[0,3]` on 2026-11-10 at nElo +12.2 ± 6.5 over 11,108 games.
    - **D.6.3 Null-move and ProbCut children typed `cut_node = true` — `R2`, NO_CHANGE 2026-11-11.** (NODE-1): the
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
     fix. With the fix, a unit test that pins the `cut_node` each recursive
     call receives (NODE-14). Gate: case 3 `[-5,5]` as a repair (TT cutoffs and reductions
     are shared signals); `[0,3]` only if the screen predicts a gain and the
     registration says so. A neutral fingerprint is not expected.
     Screen read 2026-11-10 with D.6.4 (RAR-S109): ProbCut children fail
     high 2.2% and null children 31%, so both are mostly all nodes. The
     null half's sparseness leg fired (0.52% of interior nodes), but on a
     measure the screen's own arm refutes: typing the null child alone
     moves `bench 13` −5.0% and the null-move cut rate from 68.7% to
     74.8%, because the child's result decides whether its parent searches
     at all. That half is neither ready nor closed; the same measure is
     void for the ProbCut half's pass. Each half's arm loses canaries by a
     later settle at depth 10, every one still solved at 100k, so the item
     stays in research. With D.6.4 the tree shrinks 30%, 34% below the two
     separate effects: the next registration treats D.6.3, null child
     included, and D.6.4 as one cluster. It first explains that coupling
     with arms 5 and 6, and measures reach by the null-move cut rate per
     arm and the tree beneath parents whose null move did not cut, not by
     the child's interior subtree.
     Cluster screen read 2026-11-10 (RAR-S110): the null half fails its
     reach floor (+4.84 points against 5.36) and leaves the cluster,
     retried on a change to the null move's margin or reduction or the
     cut-node consumers. The ProbCut half stays by its children's history
     rewards (2.1% of noisy updates), so the cluster arm is ProbCut typed
     `false` with D.6.4. The no-TT-move LMR term, the null decisions and
     the TT cutoffs all carry part of the coupling, but three bench
     positions carry 56.5% of it, so the item is back in research. Next is
     a fresh-process per-position read, registered first. The canary rule's
     treatment of a later settle is put to the maintainer (RAR-S110).
     Fresh-process read 2026-11-10 (RAR-S111): each bench position in its
     own process, nine arms. The coupling is not general: without shared
     state the pooled index is 1.17 and the median 1.10, below 1 in 17 of
     39 positions. The bench's coupling was position 11 in D.6.4's arm,
     which the shared table held at 2.55M interior nodes against 0.41M
     fresh. ProbCut typed `false` alone searches 0.958 of the source's
     nodes. The cluster's reason to be one unit falls, and the next
     registration reads this half and D.6.4 as separable repairs on
     fresh-process counts, each with its canary outcome. RAR-S110's
     reach floors were bench reads; the null half's closure stands.
     Readiness read 2026-11-11 (RAR-S112), as its own repair: ProbCut's
     children fail high 2.2% in fresh searches and 2.9% in game-like
     ones, and their fail-low rewards are 1.9% and 1.6% of the
     noisy-history updates. Typing them `false` removes those rewards
     and makes ProbCut cut 4.4 points less often per node game-like;
     the tree has no one direction (0.96 fresh, 1.01 game-like). WAC is
     240 against 239 at 100k and 266 against 264 at 400k, with no lost
     tactic at 100k. The old rule's two losses are a one-iteration
     flicker and a move found one iteration later in fewer nodes; a
     later settle is a diagnostic (RAR-M85). **Ready for
     implementation:** the two ProbCut verification calls pass `false`,
     the null child stays `true`, `bench 13` must read 13,148,505, with
     NODE-14's unit test; then its gate, case 3, registered before any
     game. If the gate fails, the rework is the typing with the reward
     kept.
     Implemented 2026-11-11 as `d15050a`: the two calls pass `false`,
     nothing else changes, `bench 13` 13,148,505 / EBF 2.559, and
     NODE-14's test `each_recursive_call_types_its_child_by_its_rule`
     pins every recursive call's type (with the old typing it fails on
     ProbCut's children). Three tests it moved were settled first, each
     in its own commit: the transposing-rook anchor split (D.6.17), the
     singular margins' liveness probes re-pinned, and the won-ending
     clock test, red since `ee0b3e4`, held (D.6.20). **Gate registered
     as RAR-S114,** before any game: case 3, SPRT `[-5,5]`, cap 6,000
     pairs, against `63813d8`; every recent gate game reaches
     ProbCut's domain. H1 or the cap without H0 keeps it; H0 reverts
     it and registers the typing with the reward kept.
     **Gate read 2026-11-11 (RAR-S114): H0** at 9,662 games, nElo
     −3.7 ± 6.9, LLR −2.97, no fault. In the games the arms searched
     trees of the same size (157,625 against 157,816 nodes a move,
     depth 13.23 against 13.20), so the cost lies in what the search
     decides. The typing is reverted (`91265f2`), keeping NODE-14's test
     with ProbCut's children pinned `true`; `bench 13` is 13,038,236
     again. The defect stays open. **Ready for implementation, as
     registered:** the two calls pass `false` and the ProbCut child
     keeps the fail-low reward to the capture's noisy history, which
     separates the reward from the type's other consumers; its own
     registration and gate, case 3.
     Research after the gate 2026-11-11 (RAR-M86, with RAR-S114's
     games reread): **closed `NO_CHANGE`; the registered rework is
     not gated** (maintainer decision 2026-11-11, on this research's
     recommendation; it replaces RAR-S114's line that the defect
     stays open). The gate's H0 is weak evidence: the case-3 gate
     reverts 34% of neutral repairs, and an H0 as late as RAR-S114's
     is 2.6 times as likely at −5 as at zero and twice as likely at
     zero as at +2. It speaks against a gain of 3 or more and shows
     no loss of 5. Nothing in its games differs between the arms:
     nodes, depth, swings in a side's own score, which side sees a
     result first, conversion. No mechanism points to a gain: the
     typing makes ProbCut's verification stricter under a margin
     fitted with the cheaper one, and the reward it drops is 1.6% of
     noisy-history updates, which the rework would only restore. The
     donor's label is not evidence (rule 1). Retry at the search
     re-fit (F.6) with ProbCut's margin and the cut-node terms in the
     same fit, or on a change to ProbCut's margin, depth or
     verification. NODE-14's test keeps every recursive call's type
     pinned, ProbCut's children `true`.
    - **D.6.4 A fail-low node stores its least-bad move as the TT move — `R2`, NO_CHANGE 2026-11-11.** (NODE-2):
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
     Screen read 2026-11-10 with D.6.3 (RAR-S109): 66% of fail-low stores
     change the stored move, and a move from an `Upper` entry is at 10% of
     interior nodes, keeping IIR off at 338k of them. Its arm loses WAC.069
     and WAC.182 to a later settle at depth 10, so the item stays in
     research, in one cluster with D.6.3.
     It stays in that cluster after RAR-S110 and is read with D.6.3.
     Fresh-process read 2026-11-10 (RAR-S111): alone it searches 44% fewer
     interior nodes than the source over the 40 bench positions, 25%
     fewer at the median, and 32 of 39 positions shrink. RAR-S109's
     −1.94% was the bench's, held up by one position on shared state, and
     is superseded as the size of this change. It is separable from
     D.6.3 and is read as its own repair next; that registration says
     which table state its reads use.
     Readiness read 2026-11-11 (RAR-S113), as its own repair, in fresh
     searches and in game-like ones (the table and histories a game's
     earlier moves leave): 64% and 53% of the source's `Upper` stores
     change the stored move, and a TT move from an `Upper` entry is at
     10% and 13% of interior nodes. The size depends on the table: the
     change searches 0.559 of the source's nodes fresh and 0.870
     game-like (0.905 at the median). LMR without a TT move goes from
     20% to 57% of its computations and IIR from 12% to 15% of nodes,
     populations their constants were not fitted on. WAC is 237 against
     239 at 100k and 265 against 264 at 400k, with no lost tactic. **Ready
     for implementation, after D.6.3's gate is read:** an `Upper` store
     passes no move; `bench 13` must read 12,785,308 on the source or
     9,892,346 on top of D.6.3's typing; gate case 3. If the gate fails,
     the rework refits IIR's depth and the no-TT-move LMR terms.
     D.6.3's gate was read 2026-11-11 (RAR-S114, H0, reverted), so the
     tree is the source's: `bench 13` must read 12,785,308. It runs
     before D.6.3's rework (maintainer decision 2026-11-11).
     Implemented 2026-11-11 as `45adef3`: the main search's store
     passes no move for an `Upper` bound, nothing else changes,
     `bench 13` 12,785,308 / EBF 2.513, and
     `a_fail_low_store_keeps_the_stored_move` pins it (with the old
     store a fresh table's fail low carries the first move searched).
     Two tests it moved were settled first, each in its own commit:
     the singular margins' liveness probes re-pinned on `SEMI_SLAV`
     (`9210e5e`), and the MultiPV helper test asserting its stated
     invariant, the three best root moves exact, not a count
     (`7a3087a`). **Gate registered as RAR-S115,** before any game:
     case 3, SPRT `[-5,5]`, cap 6,000 pairs, against `7a3087a`; every
     recent gate game reaches the repair's domain. H1 or the cap
     without H0 keeps it; H0 reverts it and registers the store
     change with IIR's depth and the no-TT-move LMR terms refitted.
     **Gate read 2026-11-11 (RAR-S115): H0** at 2,452 games, nElo
     −14.7 ± 13.8, LLR −2.97, no fault. In the games the candidate
     reached mean depth 14.01 against 13.12 on 162,331 against
     157,539 nodes a move: it searched deeper and played weaker.
     The store change is reverted (`5d60017`), and its unit test
     now states the restored contract, that a fail low stores its
     best-scoring move; `bench 13` is 13,038,236 again. The defect
     stays open. The registered rework is the store change with
     IIR's depth and the no-TT-move LMR terms refitted. It has no
     readiness card, so the item is in research: which consumer of
     the stored move carries the loss, and whether a refit is the
     right rework.
     Research after the gate 2026-11-11 (RAR-S117, a replay of 60 of
     the gate's games under eight arms; RAR-S116 was void by its
     wire): **closed `NO_CHANGE`.** The store change's smaller tree
     is IIR and nothing else. It searched 0.81 of the source's nodes
     to the same depth and gained 0.66 of depth at the games' node
     counts; with IIR alone reading a fail-low entry as carrying a
     move, the gain is 0.07. Holding the cut-node LMR term or the
     razoring guard changes nothing. RAR-S113's 20% to 57% counted
     every LMR computation; the term applies at cut nodes off the PV
     line, where the share moved from 10% to 14%. So the gate
     measured IIR taking a ply from revisited fail-low nodes: 0.66
     of nominal depth for a loss of 14.7 ± 13.8 nElo. The repair's
     own premise fails as well. With the three consumers held,
     keeping the entry's older move needs 1.022 of the nodes the
     best-scoring move does (0.990 to 1.043), and taking the
     best-scoring move from the picker costs 23% more. The stored
     move is useful ordering here, not a defect. The registered
     refit is not the right rework: IIR's depth threshold cannot
     pick out a node class, the LMR term has nothing to refit, and
     neither would give the change a gain to gate. Retry only once
     IIR no longer reduces a node for a fail-low entry without a
     move (D.6.27, or the search re-fit, F.6), and then as an
     ordering change with a predicted node saving at equal
     consumers. The store's contract is pinned by
     `a_fail_low_store_carries_its_best_scoring_move`.
    - **D.6.5 The exclusion search uses the table-refined estimate — `R2`, NO_CHANGE 2026-11-11.** (NODE-4):
     `eval_for_pruning` has no `excluded` guard; in a singular exclusion node
     the TT bound is Lower or Exact and the window sits below `tt_score`, so
     the refined estimate always exceeds `alpha + margin`, LDSE can never fire
     there and the LMR `alpha_gap` term is pinned at its clamp; the donor
     keeps the plain eval (`search.rs:471–482`). Screen: a counter of
     exclusion nodes where LDSE would fire; fix: a one-line guard, with a
     test of the estimate at an excluded node (NODE-14). Gate: case
     2 under 2% of exclusion nodes, else `[0,3]`.
     Also (2026-11-03, `analysis/triumviratus_review_2026-11-03.md` §4 item
     10): the exclusion search and the null-move verification re-enter the
     same ply and rewrite the stack's `static_eval` and `tt_move`
     (`node.rs:692-712`), restoring only `tt_pv` and `laterality`; if the
     correction tables changed in between, the stack's eval differs from the
     node's own. A counter of re-entries where the two differ goes with this
     item's screen.
     Screen read 2026-11-11 (RAR-S118, fresh and on RAR-S117's 60
     replayed gate games): the premise holds. Under the refined estimate
     LDSE and razoring never fire at an exclusion node; under the plain
     one LDSE fires at 23.8% of them game-like (31.7% fresh), razoring at
     0.004%. The alpha-gap term is not pinned at its clamp as stated
     above: where the refinement applies it is fixed at
     `-(margin + 1)`, about 0.75 of a unit per ply, far inside −65; it
     differs between the two estimates in 53% of the quiet reductions.
     The change moves no aggregate: every singular outcome's share moves
     by under 0.15 points game-like and the replay's depth by −0.03
     (fresh nodes 0.885); WAC is 233 against 239 at 100k, at its floor,
     with no canary lost. **The item stays in research** for a paired
     direction read: in an isolated probe at sampled singular attempts,
     the exclusion verdict under each estimate against a deeper
     exclusion search at the same node. That read decides readiness; a
     gate would be `[0,3]`, since LDSE fires at over 2%. The re-entry
     rewrite **closes `NO_CHANGE`**: an exclusion search leaves another
     eval at 0.001% of re-entries, and a null verification that does is
     followed by its parent's own search in 1 of 43,949.
     Direction read 2026-11-11 (RAR-S119, a paired probe at 65,328
     sampled singular attempts of the same replayed games): **closed
     `NO_CHANGE`** by its frozen first rule. The two estimates' decisions
     differ at 1.96% of attempts, under 2%. Where they differ, the plain
     estimate matches a deeper exclusion search 747 times against 273,
     and its exclusion searches cost 0.943 of the refined ones. The
     direction is the donor's, but the net gain is 0.74 points of
     decisions at 4.4% of nodes, which no gate can resolve. Retry only
     after a change to LDSE's depth or margin, razoring's margins, the
     singular margins or the alpha-gap term, or at the search re-fit
     (F.6), and only if the probe then reads 2% or more.
    - **D.6.6 Captures generated and scored before the TT move — `R2`, NO_CHANGE 2026-11-11.** (NODE-5):
     `fill_noisy` runs before the loop; the donor generates after the hash
     move fails to cut, and the TT move cuts at about 35% of loop nodes
     (`analysis/c12_speed_review_2026-10-18.md`: the one lead worth several
     percent). The capture scores then read a history the TT subtree has
     moved, so ordering changes. `I2` with a frozen prediction (speed +x%,
     games neutral), the pooled-PGO NPS read, then case 2.
     Screen read 2026-11-11 (RAR-S120, cycle counts at an identical tree
     on RAR-S117's 60 replayed gate games, idle host): **closed
     `NO_CHANGE`.** Generating and scoring captures costs 3.95% of search
     cycles, and 22% of fills are at nodes that cut before any capture is
     looked at. So generating after the TT move could save at most 0.82%
     (0.93% on the bench), under the frozen 1%. By about 2 Elo per 1% NPS
     that is under 2 Elo, below what a speed read and a case-2 read can
     bank. The playing change would be negligible: the TT move's subtree
     changes the top capture at 0.11% of the nodes where order matters.
     C.12's "several percent" is superseded. Retry only if capture
     generation or scoring becomes materially dearer, or a later profile
     puts it at 8% or more of search cycles (H.4).
    - **D.6.7 The ply cap — `R2`, DONE 2026-11-11.** (NODE-8): `MAX_PLY = 128` is below the extension budget's
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
     A form to choose with the cap (2026-11-03, TS-1 in
     `analysis/triumviratus_review_2026-11-03.md`): bound the lines instead of
     only raising the cap. Triumviratus counts the extensions that negative
     reductions produce, per line, and allows one (−6.6% nodes and −0.55 plies
     of selective depth at fixed depth; +3.88 ± 3.26 over 11,384); Stockfish
     after 19 bounds the depth by ply against root depth. D.1.6 measured the
     same growth here (RAR-R19: a third of reduced moves in the cascade
     iteration run deeper than nominal, outside the line budget).
     A far shallower case (2026-11-10, RAR-S108): in a promotion race,
     `8/2p2P1P/2n2P2/1k6/p7/6N1/P5K1/8 b - - 0 44`, seldepth reaches 127 by
     depth 10 and depth 11 is not finished in 60 s (about 200M nodes, the TT
     full), on `72c4e60` and with D.6.2's push alike; each of the other 1,483
     first positions with ten men or fewer from RAR-S107's games finished depth
     12 in 3.1 s at most. The screen asks first which mechanism drives the line
     to the cap there: extensions, quiescence or the conversion band.
     Screen read 2026-11-11 (RAR-S121: cap counters on RAR-S117's 60
     replayed gate games at the recorded node counts times 1, 10/3 and 20,
     and on the race position): **ready for implementation, the form
     "bound the lines".** No search reaches the cap at `3+0.03`'s node
     counts. At ten-thirds of them 0.109% of searches reach it; at twenty
     times, 1.962%, in 22 of the 60 games. There, 22% of cap nodes are in
     check and 15% are draws by rule, and each returns a static eval. At a
     cap node the iteration is at depth 15 and the budget has spent 13 of
     it, yet the line holds about 100 plies more: depth gained outside the
     budget carries 88% of the plies beyond the iteration's depth (93% in
     the race position). Quiescence carries none. Raising the cap would
     move the symptom, not the cause. The implementation first splits that
     gain by source, before choosing the bound:
     - the late-move reduction's room above nominal, two plies and two more
       on a PV line;
     - the re-search ply;
     - hindsight's ply.

     TS-1's per-line count is the prior form. **Gate:** case 3, `[-5,5]`
     with its cap, since bounding the lines changes reductions across the
     tree. That corrects the case 1 RAR-S121 copied from this card, whose
     clause was written for raising the cap. Case 1 stays open only to a
     form confined to lines that reach the cap, with a static differential
     as its correctness read. The registration states the exposure (zero
     at STC, 1.96% of searches at `60+0.6`'s node counts) and chooses its
     time control knowingly: a gate at `3+0.03` cannot see the cap.
     Implemented 2026-11-11 as `aee0b3b`: a node's depth is held to
     `2 × iteration − ply`, before quiescence and again after hindsight,
     which keeps the contract the extension budget's comment states.
     The bound holds every out-of-budget gain to the room the extensions
     left, so the source split was not needed to choose it. `bench 13`
     11,477,040 / EBF 2.537. `no_line_searches_past_twice_the_iteration_depth`
     pins it, and fails with the clamp disabled. Three test preconditions
     were fixed first, each in its own commit and read on both trees:
     - the node probes' iteration depth (`bd37b17`);
     - two singular liveness pins (`61a7cb4`);
     - the helper's ranking without an unconfirmed fail-high (`5f21606`).

     On the candidate, the race position's selective depth at depth 10
     falls from 127 to 30. At the games' node counts the bound cuts 0.49%
     of interior nodes, in 94% of searches and in all 60 games. At twenty
     times them the cap is reached in 2 searches instead of 72, both deep
     endgame iterations (63 and 84) where twice the iteration reaches it by
     itself; that residual is outside this repair. WAC is 238 against 239
     at 100k and 260 against 264 at 400k, with no canary lost. **Gate
     registered as RAR-S122,** before any game: case 3, SPRT `[-5,5]`, cap
     6,000 pairs, against `5f21606`. A `3+0.03` gate reads the bound's
     effect on the tree, not the cap. H0 reverts it and the defect stays
     open here.
     **Gate read 2026-11-11 (RAR-S122): the cap without a bound,** nElo
     −1.0 ± 6.2 over 12,000 games, so the bound is kept and the fingerprint
     moves to 11,477,040 / EBF 2.537. That is no harm beyond about −5, not a
     gain. The residual is the cap from iteration 64, where twice the
     iteration reaches it by itself: 0.05% of replayed searches at twenty
     times the games' node counts. Retry, by raising the cap with the band
     constants or by the donor's in-check and draw handling at the cap, if a
     read at a rated time control finds more than 0.1% of searches reaching
     it.
    - **D.6.8 A depth-0 entry is counted as two nodes — `R2`, DONE 2026-11-13.** (DRIVER-11): `check_stop` at
     the `negamax` entry and again at the `quiescence` entry it hands to, so
     reported nodes, `go nodes`, the bench count and the LMR spread's phase
     read the double count; the donor counts once per move made. Screen: the
     `diag` interior and quiescence counts against `td.nodes` on `bench 13`.
     Fix: test `depth <= 0` first and give the LMR spread its own counter if
     its phase is to be kept. A new fingerprint by construction: register it
     alone, case 2 at most, never folded into a strength change.
     Screen read 2026-11-12 (RAR-S123, `bench 13` at stride 1): **ready
     for implementation.** 16.1% of `td.nodes` counts a position the search
     already counted:
     - the hand-off to quiescence, 12.3%;
     - the singular exclusion search, 2.9%;
     - razoring's quiescence search, 0.9%;
     - null-move verification, nearly none.

     So reported `nps` reads about 19% high, and `go nodes N` searches
     about 0.84 N positions. The fix counts each position once and every
     consumer reads that count: the LMR spread, the effort share, the poll
     cadence and the node limit. No old clock is kept. **Gate:** case 2,
     a 2,000-game read with the −13 harm rule, registered alone. One
     contract question is open before the gate's registration. LMR's
     full-depth re-search and ProbCut's verification re-enter a child
     without a new move, at least 1.2% more. The donor's one count per
     move made covers them; the screen's four paths do not.
     **Contract decided 2026-11-13:** a position is counted once, when a
     move or a null move reaches it, as RAR-S123's frozen rule says; a
     re-search enters a position a move already reached and counts
     nothing, the donor's one count per move made, which is what D.7.2's
     equal-node match needs. The null move is counted, by the frozen
     rule, where the donor does not count it.
     Implemented 2026-11-13 as `ba1d2cd`: `count_node` is called once
     after each move the main loop, ProbCut and quiescence make and after
     the null move, and the node entries only read the stop flag.
     `bench 13` 9,077,016 / EBF 2.481. On the old tree (an uncommitted
     arm keeping the old clock for the LMR spread, which read exactly
     11,477,040) the count reads 9,475,302, inside RAR-S123's bound of
     9,489,974; the rest is the spread's new phase.
     `a_depth_one_search_counts_each_root_move_once` pins it: 20 at depth
     1 from the start position, 51 on the old count. Two test
     preconditions were fixed first, each in its own commit and read on
     both trees: the transposing promotion anchor pins the transposition,
     not a margin between two searches of a tie (`56da869`), and the
     double singular liveness probe moves to the knight-up position at
     depth 10 (`10059f9`). **Gate registered as RAR-S124,** before any
     game: case 2, 2,000 games at `3+0.03` against `10059f9`, the −13
     harm rule; every search of every game reads the count.
     **Gate read 2026-11-13 (RAR-S124): kept,** nElo −6.2 ± 15.2 against
     the −13 rule, no fault; the same depth and time a move, nodes a move
     ×0.826, the count's own ratio on the bench's fixed tree. The
     fingerprint moves to 9,077,016 / EBF 2.481. Kept means no harm
     beyond roughly −20 nElo.
    - **D.6.9 The endgame scale's target — `R2`.** (EVAL-5): `evaluate` adds tempo to `mg`,
     tapers, then `scale_endgame` multiplies the whole; both donor commits
     scale the endgame half only (`v = mg·p + eg·(128−p)·sf/64`), so the forms
     differ for the opposite-bishop rule with pieces on the board and the
     partial recognisers at small phase. The constants were fitted inside
     Rarog's form (RAR-E26). Screen, offline: refit the three OCB constants
     with the scale on the endgame half on RAR-E26's cohort and compare
     held-out loss; then case 2 at most (OCB-with-pieces positions only).
     Under the evaluation freeze this is a registered repair.
    - **D.6.10 The pawnless default factor — `R2`.** (EVAL-6): `material.cpp:195–203` scales
      a side with no pawns and a small material edge to 0, 4 or 14 of 64;
      Rarog's pawnless branch covers at most one minor in total or one each,
      so K+B+N v K+B keeps two thirds of about a knight. Never a ranked family
      (the family list is SF11's endgame functions). Screen: the families'
      share of the game record; at 0.5% of games or more the Syzygy
      drawn-overclaim read of RAR-E36. Case 2.
    - **D.6.13 The aspiration fail-low window — `R2`.** (DRIVER-13): the code halves toward
      the new alpha; a counter of fail-lows whose re-search fails high
      (`asp_fail_high` straight after `asp_fail_low` at one depth); above 15%
      the old-alpha form is a `[0,3]` candidate.
      Priors against neighbouring forms (TS-15 in
      `analysis/triumviratus_review_2026-11-03.md`): a decisive aspiration
      fail-high +0.7 ± 4.2, root depth recovery after a fail-high −1.82 ±
      4.54, a cap on root-move reductions −5.80 ± 5.93.
  Added 2026-11-02 from the Basilisk cross-check (`analysis/basilisk_crosscheck_2026-11-02.md`):
    - **D.6.15 The correction is added to the evaluator's draw verdicts — `I1`, DONE 2026-11-03.**
      `3a16fbb`: the search no longer adds its correction to the
      evaluator's draw verdicts; the fingerprint moved from 13,987,223 to
      14,749,043, accepted by repair case 1 on a confined per-FEN
      differential, and no verdict leaf keeps a moved value. 7 of 28
      wrong-bishop draws still misread from cleared tables: D.6.23.
      Record: `docs/archive/PLAN-phaseD-done-2026-11-04.md`.
    - **D.6.16 Quiescence prunes by gain against a decisive material edge — `R2`.** (A2;
      Basilisk `d8d30e9`): futility (`node.rs:2355-2362`) and the SEE
      threshold (`node.rs:2392-2396`) read a capture's gain as the piece
      taken, which is false when the capture removes the opponent's
      mating material (Kxh8 in KBB-K is a dead draw); delta pruning is off
      at eight men or fewer and is not involved. Rarog has no known-win
      band, so the guard's definition is the research question. Screen:
      Basilisk's fixture, and a `diag` count of skipped captures that
      leave the opponent without mating material on the conversion
      families. `R2` card first.
    - **D.6.17 Underpromotions and Saavedra's study — `R2`.** (an investigation): Basilisk
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
      **The wrong-pick branch fired 2026-11-11, on D.6.3's candidate**
      (ProbCut's children typed `false`), in Basilisk's transposing rook
      position (`2k2rr1/6P1/2P5/7p/1B2p3/4P3/5K2/8 w - - 0 46`): after
      `gxf8` with either piece, `Rxf8+ Bxf8` reaches the same position,
      a true tie. At 100,000 nodes the candidate plays `g7f8r` (528 at
      depth 14), the rook it searched first, from depth 12 on; the
      source at `79f761d` plays the queen by its tree's shape. Searched
      apart at depth 14 the queen reads 896 and the rook 768, the
      noise of two searches the anchor had read as a margin. Nothing in
      Rarog's search breaks the tie. The item is now a card with
      Basilisk's rule as its prior (a rook or bishop promotion is played
      over the queen only by more than 200 of exact score, BAS-S33,
      BAS-S34). Its acceptance test is
      `queen_promotion_though_the_rook_was_searched_first`, ignored
      until the rule lands (`d41bae3`);
      `promotion_on_f8_wins_where_the_rook_transposes` stays active. It
      was to land before the 2.7.0 cut (maintainer decision 2026-11-11);
      that hold was lifted 2026-11-13 (maintainer decision): the rook is a
      true tie that changes no result, and D.7 replaces the root order
      that breaks it, so D.7.8 asks first.
    - **D.6.20 The fail-high stop's hole — `R2`.** (H24): the stop at twice the target is
      checked only when a window returns (`search/mod.rs:977-978`), so a
      window that never fails high runs to the maximum (RAR-R26's rec3,
      97.9 s at `180000+2000`); Basilisk adds the give-up window as a
      second form (2.79× and 4.07× the optimum) and registered a replay
      design (Basilisk's `b5_failhigh_stop_2026-10-25` packet, M1: the hole's
      rate by score band, the table carried). `R2` zero-game read;
      `NO_CHANGE` under 0.5% of searches. Long controls only; the hard
      maximum still bounds it.
      **A measured instance, 2026-11-11,** at `30000+300`:
      `won_ending_does_not_sink_the_clock`
      (`7r/5R2/8/2k1PB2/8/4K3/8/8 w - - 0 86`, limit 8 s) is red since
      `ee0b3e4`, the quiescence queen push. Read alone on an idle host,
      three runs a commit: 2.42 to 2.53 s at `b202174` and `72c4e60`
      (9,838,664 nodes, depth 18, exact); 9.46 to 9.82 s at `ee0b3e4`
      and `79f761d`, the same engine source (37,767,960 nodes, the last
      line a depth-16 lowerbound); 12.91 s with D.6.3's candidate (a
      depth-19 lowerbound). The node counts repeat exactly, so it is a
      failing test and not a flaky one. The control is not long, and
      the per-window timing that would show whether this is the hole
      above or another cause is not yet read: that is the leaf's first
      read. The test is ignored as this leaf's acceptance test
      (`b6914f0`). Its hold on the 2.7.0 cut was lifted 2026-11-13
      (maintainer decision): the hole is older than 2.6.0, the hard
      maximum bounds it, and the stop sits in the loop D.7 replaces, so
      D.7.8 asks first.
    - **D.6.21 Won KPK positions read at or below 0 — `R2`.** (D.5.19's read,
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
    - **D.6.22 `krpkr_scale`'s hard zeros on won positions — `R2`.** (D.5.19's read,
      2026-11-02; Basilisk's BAS-E54 found 18 of 3,000): over 2,000,000
      quiet random KRPKR positions the five `Some(0)` sites fired 65,865
      times, 97.6% on Syzygy draws; 1,137 (1.7%) are wins for the pawn
      side, 926 at site 1 (the third-rank defence, whose relaxation for a
      pawn on its 2nd or 3rd rank lets the defending rook stand anywhere)
      and 202 at site 5 (the king's blockade)
      (`analysis/draw_verdicts_2026-11-02.md`, §3). A zero on a defender's
      win (462) errs toward the truth. Repair form (D.5.19): discount
      instead of zeroing where the rule is approximate. Screen: per site,
      a scale that keeps the drawn positions near 0 and the wins above it,
      read on the same sample. KRPKR reaches about 10% of real games
      (`krpkr_scale`'s note) and the zero sites far fewer, so the
      registration states its measured exposure and the gate case follows
      (a static differential confines it, case 1; a refitted scale is a
      value change, case 2's fixed read). `R2` card.
    - **D.6.23 Wrong-bishop draws still misread through leaves the recogniser does not cover — `R2`.**
      D.6.15's falsifier (2026-11-03): with the correction kept off every
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
    - **D.6.24 The KQKP fortress's hard zeros on won positions — `R2`.** (D.5.19's read,
      2026-11-08): over every legal KQvKP position with the pawn on the
      defender's seventh, 5,552,520 of them, `kqkp_fortress_scale` returns 0
      at 172,256: 131,256 Syzygy draws, 32,360 wins for the queen side and
      8,640 for the pawn side, which err toward the truth. Of the 69,760
      quiet zeros 18,628 (26.7%) are queen wins, where the rule's test of
      the queen's king (more than two squares from the queening square) is
      loosest: at three or four squares, 12,552 of the 14,760 with the queen
      side to move and 3,860 of the 3,868 with the pawn side to move; from
      five to seven only the queen side to move remains, 1,996 of its 2,208
      on rook pawns (`analysis/draw_verdicts_2026-11-02.md`, §7). Repair
      form (D.5.19): discount instead of zeroing, or narrow the zone to what
      the tables call drawn (distance, side to move, file). Screen: the same
      enumeration with the candidate rule, the draws kept at 0 and the wins
      lifted. The registration states measured exposure (gate games
      reaching KQKP with a seventh-rank rook or bishop pawn); a narrowed zone
      that a static differential confines is case 1. `R2` card, minutes.
    - **D.6.25 Two same-coloured bishops outscore a queen against a bare king — `R2`.** (D.5.21's
      port of Basilisk's `bare_king_material_is_monotone`, 2026-11-08): from
      `1B6/5P2/8/2k2K2/8/8/8/8 w` the promotions evaluate queen 1433, rook
      990, knight 1097 and bishop 1537, and the bishop's position, two
      bishops on one colour, is a dead draw; `endgames.epd`'s same-coloured
      KBB-K draws (lines 62–63) read 1149 and 2550. The search is not misled
      here (it promotes to a queen at every depth 1–12), so the cost is
      where a static score decides: a choice between a draw and a win the
      search cannot resolve. Screen: whether any search at the gate's budget
      chooses the drawn material, and the exposure of KBB-K with one colour
      in gate games. Acceptance test: `bare_king_promotion_order` in
      `tests/behaviour_anchors.rs`, committed `#[ignore]`d. `R2` card.
    - **D.6.26 Pawnless endings a minor ahead read as wins — `R2`.** (D.5.21's port of Basilisk's
      `pawnless_scale_truth_table`, 2026-11-08): Rarog has no scale for a
      pawnless side only a minor ahead, so Syzygy draws read KR-KB 277,
      KRB-KR 253 and KRN-KR 173, where the dead KB-KN draw reads 0 and three
      of the queen's classes read as wins. Screen: an enumeration of the
      classes against Syzygy, as D.5.19's, and their exposure in gate games;
      the repair form is a scale, as Stockfish 11's. Acceptance test:
      `pawnless_scale_truth_table` in `tests/behaviour_anchors.rs`, committed
      `#[ignore]`d (needs `RAROG_SYZYGY_PATH`). `R2` card.
    - **D.6.27 IIR has no node-type condition — `R2`.** (found 2026-11-11,
      RAR-S117): IIR reduces any node of depth 4 or more that has no table
      move. Stockfish 19 reduces only PV and cut nodes, at depth 6 or more,
      and Reckless at its head of 2026-10-29 has no IIR. Local evidence:
      D.6.4's store change left revisited fail-low nodes without a move,
      IIR for want of a move rose from 9.9% to 13.5% of interior nodes, that
      alone bought 0.66 of depth at the games' node counts, and the gate
      lost 14.7 ± 13.8 nElo (RAR-S115). A ply taken from those nodes was
      worth more than the ply the clock gave back. Whether the source's own
      IIR at expected all nodes costs the same way is not measured. Screen,
      zero games, on RAR-S117's replay: IIR applications by node type and
      by whether the node then fails low; under 10% of them at expected all
      nodes closes the item `NO_CHANGE`. Candidate: IIR at PV and cut nodes
      only. Its tree grows and its nominal depth falls, so the card freezes
      a prediction for both before any game. Interactions: IIR's depth
      threshold, the too-shallow-entry half of its predicate, the cut-node
      LMR term, and D.6.4's store, which is an ordering change alone once
      IIR ignores those nodes. Gate: `[0,3]` if the card predicts a gain,
      else case 3. `R2` card.
    - **D.6.28 Clock use: a tenth of the budget is left unspent — `R2`.** (found
      2026-11-13 in RAR-O08's games): against a budget of the base plus the
      increment times its own moves, Rarog's head spent 87.1 to 88.9% of
      its clock where Reckless's time manager spent 94.9 to 95.1% and
      Stockfish classical's 93.5 to 94.1%; in the first twenty moves Rarog
      spent 0.76 to 0.77 of the base and Reckless 0.84 to 0.87. Rarog's
      budget constants are Stockfish's, and Stockfish dev reads 88.7 to
      89.7%. The layer is clock use, not Elo: D.1 made the engine keep more
      reserve when its clock runs low, and whether the total left unspent
      costs strength was not read. Independent of the kernel, so it
      survives D.7 either way, and D.7.2's equal-move-time read sizes it.
      Screen, zero games: the unspent share by move number and by game
      length on Colosseum's gate games, and where the reserve sits (the
      low-clock rule, the hard bound, the forfeit margin). Candidate: one
      knob, `tm_opt_scale`, registered as an SPRT `[0,3]` with a `10+0.1`
      harm read and the forfeit count as its invariant. `R2` card.
      **RAR-M87's games, 2026-11-13** (`analysis/basilisk_check_2026-11-13.md` §3): the low-clock
      rule is the mechanism, and the field gap sits where it acts. Below
      one second `sudden_death_horizon` (`time.rs:101`) sets the horizon
      to `clock_s × TmHorizonPerSec` (50), where 2.6.0 kept 50 moves at
      every clock: the optimum falls to 0.65 of 2.6.0's at 500 ms and
      0.43 at 300 ms, the clock stops falling near 420–575 ms against
      154 ms, and the 2.7.0 head ended its games with 616 ms left against
      379. Split by whether Rarog's clock fell under 500 ms while the game
      was undecided (|s| < 150), the head's gap to 2.6.0 against the field
      is −33.8 ± 10.1 Elo inside the band and +9.0 ± 10.7 outside
      (−18.7 ± 8.5 and +28.4 ± 14.5 at 1,000 ms): a biased split, but the
      obvious bias favours the head in both buckets. The rule answered
      forfeits RAR-R14 sized at about 0.01 Elo, and RAR-R17 read the
      horizon at 100 against 53 head to head only (−0.9 ± 8.0). So the
      narrower candidate is the horizon, not `tm_opt_scale`: confine the
      contraction to where the forfeits were (`TmHorizonPerSec` near 167,
      its range widened past 100), read against a field, not head to
      head, the forfeit count its invariant. Evidence layer: a post-hoc
      split of game outcomes, which chooses and does not accept.
    - **D.6.29 Quiescence never scores a stalemate — `R2`.** (found
      2026-11-13, the Basilisk check, `analysis/basilisk_check_2026-11-13.md` §1): out of
      check with no capture to search, quiescence never asks whether the
      side to move has a legal move, so a stalemate reached through a
      capture reads as its stand-pat. Basilisk's rebuilt quiescence tests
      the narrow case, the mover just left with king and pawns, inside a
      cluster gated as a whole (B.4, +31.7), so the piece has no read of
      its own; Reckless lacks it too, so D.7 does not supply it. Screen,
      zero games: a `diag` counter of quiescence nodes out of check with
      no capture, the mover left with king and pawns and no legal move,
      over `bench 13` and replayed gate games; near zero closes it
      `NO_CHANGE`. `R2` card.
      **Screened 2026-11-13** (`analysis/d629_stalemate_screen_2026-11-13.md`,
      rule frozen before the replay): on 200 replayed head-to-head games of
      RAR-M87, 19.2% of searches (2,403 of 12,520, in 164 games) hold at
      least one stalemate quiescence reads 100 cp or more from the draw;
      59,053 stalemates in 17.9M king-and-pawns nodes. Above the 1% rule,
      so a repair. **The repair:** out of check, a side to move with only
      king and pawns and no legal move scores the draw, before the stand
      pat, as the main search scores a stalemate. **Form:** Reckless
      `0756957` has none (its quiescence returns a mate in check and
      otherwise stands pat); Stockfish dev's quiescence tests it only when
      no move was searched, the side to move has no piece, the last move
      captured a knight or more and no pawn can push, and Basilisk follows
      it without the push filter. Stockfish's is the form, not widened to
      positions with pieces; it departs from the primary donor, so taking
      it is the maintainer's decision (2026-11-06 rule). The screen
      counted every king-and-pawns node, a superset of the form's trigger,
      so the root-change replay, run with the form, gives its own reach.
      **Contract:** per-node code,
      so an existence test, not a move list (no allocation, no large copy):
      king steps first, then pawn pushes and captures, stopping at the
      first legal move; only king-and-pawns nodes pay, 4.3% of quiescence
      nodes. **Competing explanations of no effect:** the misreads sit in
      lines already refuted, and the main search scores the stalemate one
      ply up whenever the capture is searched there. **Interactions:** the
      depth-0 TT store of such a node (now a draw), the stand-pat refine
      and fail-high blend it bypasses, and the draw score (no contempt).
      **Cheapest kill before games:** the same replay with the repair,
      counting searches whose root move or score differs; none moved
      closes it `NO_CHANGE`. **Gate:** it moves the fingerprint and shared
      quiescence scores in about a fifth of searches, and could matter
      either way, so case 3, `[-5,5]` with its cap, its exposure read
      from Colosseum's database at registration. The diag counters
      (`q_kp_nodes`, `q_kp_stalemate`, `q_kp_stalemate_misread`) stay as
      its activation check.
    - **D.6.30 A later iteration can overwrite a proven mate — `R2`.**
      (found 2026-11-13, the Basilisk check, `analysis/basilisk_check_2026-11-13.md` §1):
      each completed iteration replaces the root's score and move
      (`search/mod.rs:1074-1080`); the mate watch stops the search and
      keeps nothing, so an iteration that loses a mate to pruning or a
      table artefact replaces it. Basilisk keeps a proven mate against
      such an iteration. The root driver outlives D.7. Screen, zero
      games: on the mate canaries and replayed gate games, count
      completed iterations whose score leaves the mate band after an
      earlier one proved a mate; zero closes it `NO_CHANGE`. `R2` card.
      **Screened 2026-11-13** (`analysis/d630_mate_screen_2026-11-13.md`,
      rule frozen before the replay): on the same 200 replayed games, 267
      searches prove a win; a later iteration loses it in 35, 15 end
      without it and 3 play a different move. All twelve games were won,
      so the cost is a delayed mate, not a result. By the rule a repair:
      **the repair** keeps the last proving iteration's move and score
      when a later completed iteration no longer proves a win, as
      Basilisk does; a deeper iteration proving a shorter win still
      replaces it. **Contract:** root driver only, once per iteration;
      MultiPV keeps its own lines. **Interactions:** the soft stop's
      stability and fall factors and the mate watch read the root score
      and move, so the kept win also holds the clock decisions on it.
      **Gate:** if `bench 13` reproduces the fingerprint (no bench search
      ends without its win), repair case 1: an activation and correctness
      read on the replay (searches ending without a proven win go to
      zero), no strength gate. Expected value small; it may shorten the
      won-ending thinks D.6.20 holds.
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
- **D.7 Search shape: the kernel against its donor on Rarog's evaluation, rebuilt whole in Reckless's form if the reads or the maintainer say so — `R3` investigation, `I1`/`I2`/`V`/`R2` sub-steps.**
  Added 2026-11-13 from RAR-O08 and
  `analysis/search_shape_decision_2026-11-13.md` (maintainer decision the
  same day). Reckless's search, unchanged, on Rarog's evaluation is
  22.5 ± 7.9 Elo ahead of Rarog's head at equal clock, and it is Phase B's
  own donor: its search source at `0756957` differs from B.0's snapshot
  in two lines, neither behaviour. Stockfish dev's search is 88.7 ± 7.7
  behind Reckless's and level with Stockfish classical's, so the donor
  stays Reckless and no separate HCE shape is indicated: a kernel in the
  donor's form is not redone for the network, and its constants are F.6's
  seeds. The 22.5 bundles tree shape, clock use and the kernel's speed per
  node; nothing yet says which.

  **Aim.** Two questions. Is Rarog's kernel, which departs from its donor
  in at least twelve forms and was fitted around them, weaker than the
  donor's whole form on this evaluation? And do the problems the combined
  kernel has not solved (D.7.8's list) disappear in the donor's whole
  form, not in parts of it? The second is a reason for the step in its own
  right (maintainer, 2026-11-13).

  **Decision rule.** The data decides: D.7.2's reads open the rebuild at
  +10 Elo or more for the donor at equal nodes, the 95% interval excluding
  zero. Below that the agent reports the reads with a recommendation, and
  the maintainer decides after that consultation and may open the rebuild
  against the rule; the decision and its reason are recorded here.

  **Acceptance.** Equal strength is accepted (maintainer decision
  2026-11-13): the rebuilt kernel is gated against the head on the
  loss-permitting bracket, never adopted ungated, and D.7.8's review is
  the other half of the judgment. Adoption is the maintainer's decision on
  both.

  **Method.** One cluster (rule 3): the node body, quiescence, the root
  window and the evaluation wrapper move together behind one switch, the
  old kernel beside it until adoption. The seeds are the hybrid's own
  configuration, the one point of known strength. Parity with the hybrid
  is checked at zero games before any game is played. Donor forms are not
  tried one at a time on the old kernel: D.6.3, D.6.4, the fail-soft
  cluster (RAR-R20, RAR-R21) and one-retry aspiration (RAR-S89) lost
  there. Rarog's own gate-accepted forms that the donor lacks are listed
  by D.7.0 with a keep-or-drop proposal each and re-read in D.7.8.

  **Carried over unchanged.** The frozen evaluation, board and move
  generation, the table, time management, Lazy SMP, the root driver, the
  protocol, tablebases, D.1 to D.5's repairs and D.4's instruments.

  **Invariants.** Table bounds and the mate and tablebase bands; the draw,
  repetition and rule-50 identities; no allocation per node; the
  search-path perft; one count per position (D.6.8); the acceptance tests
  of D.6.17 and D.6.20.

  **Stop rules.** D.7.2 below its bar and no maintainer decision to
  proceed: D.7.3 to D.7.9 close `NO_CHANGE`. Parity not reached: no games.
  The gate at H0: the old kernel stays; this is one cluster, so one
  rejection stops the step.

  **Independence.** A kernel nearer the donor's raises what D.11 must
  disclose and the weight of the licence question. PROCESS's boundary
  governs the implementation: architecture and mechanisms cross, constants
  cross as fitted seeds, the code is Rarog's own.
    - **D.7.0 Investigation: the inventory of departures, the seeds and the registration of the reads — `R3`.**
      Every place Rarog's kernel departs from Reckless `0756957`, with file
      and line on both sides, its unit, the record that chose it and
      whether a gate accepted it. The verified start is section 5 of the
      analysis: quiescence's fail-low return and its shape, the null move's
      population, LMR at the root and in check, the re-search depth, the
      reductions on unreduced searches, IIR, multi-cut, the extension
      budget and line bound, aspiration, optimism, upcoming repetition.
      The seed table from the hybrid's configuration: the evaluation scaled
      by 2536/1024 into the donor's units, or the constants converted the
      other way; the investigation chooses and says why. The parity
      instrument's design. RAR-O09's registration with its predictions.
      Ends `READY_FOR_IMPLEMENTATION` or `NO_CHANGE`.
      Inputs from the Basilisk check (2026-11-13, `analysis/basilisk_check_2026-11-13.md` §1–2):
      Basilisk's rebuilt quiescence and root were gated as clusters, so no
      single piece carries a read here. The departures they name: the
      delta rule (Basilisk deleted it; Reckless has none); the quiescence
      TT cutoff at PV nodes and its fail-hard tail; recapture and evasion
      pruning; the aspiration centre (Rarog computes an effort-weighted
      average and mean square per root move that only tests read); the
      ply cap before the draw test, which returns the evaluation even in
      check (Reckless tests the draw first); kings in the non-pawn
      correction key (Reckless includes them); and the cutoff count
      raised by an exclusion search's fail-high (as Reckless).
    - **D.7.1 The fixed-match script takes a node budget and a move time per engine — `I1`.**
      `tools/sprt.ps1`: a node budget and a move time for each engine,
      beside `-TCA` and `-TCB`. Each is proved live by a smoke in which an
      absurd value moves the PGN and the score, and the combinations that
      make no sense are refused.
    - **D.7.2 RAR-O09: the donor's search against the head at equal nodes, at equal move time and at a long control — `V`.**
      Reckless's hybrid against the head, both rebuilt on the head after
      D.6.8: 3,000 games at equal nodes a move (tree shape alone), 3,000 at
      equal time a move with the bridge's odds (shape and kernel speed),
      2,000 at `10+0.1` with clocks (whether the surplus holds at a long
      control). The differences to RAR-O08's match (5) are the shares of
      clock use and speed. Maintainer-run, about three hours.
    - **D.7.3 Parity instrument: Rarog and the hybrid on one position set — `I2`.**
      Nodes to each depth and the rate at which each mechanism fires, on
      the same positions and the same evaluation; a planted difference
      must show.
    - **D.7.4 The kernel in the donor's form, behind one switch — `I2`.**
      Own code in Rarog's structure, written in D.8's target shape (probes,
      not interleaved instrumentation). Parity by D.7.3 before any game;
      debug and release tests, fmt, clippy, the allocation guard and the
      search-path perft; the old kernel's fingerprint exact with the switch
      off.
    - **D.7.5 The unfitted paired run — `V`.**
      2,000 games against the head (rule 8's screen). Clearly negative with
      parity passed puts the difference outside the kernel: one diagnosis,
      then stop.
    - **D.7.6 The fit — `V`.**
      Curvature sweep, then one SPSA run under rule 7c, read at
      checkpoints and stopped when its gradients are gone. The fitted
      constants are kept only if fitted over unfitted passes the gate
      PROCESS's SPSA procedure chooses from the tune's own gain read before
      any gate game: a fixed 6,000 games where the read is +10 Elo or more,
      otherwise SPRT `[0,3]`.
    - **D.7.7 The gate: the kernel against the head — `V`.**
      The loss-permitting bracket `[-1.75, 0.25]`, with a `10+0.1` read and
      a 4T read registered beside it.
    - **D.7.8 Problem review on the new kernel — `R2`.**
      Each problem the combined kernel left is asked "is this still an
      issue?", at zero games where a replay, a fixture or a test answers
      it: the seldepth chains to the ply cap (RAR-S121's replay; the line
      bound and the extension budget stay only if the chains return), the
      root fail-high cascade (RAR-R19 to R21), the won-ending clock test
      (D.6.20), the root promotion rule (D.6.17), the forms that lost on
      the combined kernel, and Rarog's own accepted forms from D.7.0's
      list. The result is a table: gone, still there, or new. It feeds the
      adoption decision and says which D.6 leaves close, stay or re-open.
    - **D.7.9 Adoption and checkpoint — `V`, then `I1`.**
      The maintainer's decision on D.7.7 and D.7.8. On adoption: RAR-O08's
      matches (2) and (5) re-read on the new head, pooled NPS, conversion,
      the old kernel and the switch deleted, the fingerprint record
      updated. An adopted kernel releases as 2.8.0.
- **D.8 Simplification programme from the engine review: tests into child files, the instrumentation behind probes, one score module, the board split, visibility, one list and one handle; about 860 source lines fewer and the largest file halved — `I1`, `R2` for the root driver.**
  Added 2026-11-01 from `analysis/engine_review_2026-11-01.md` §3.7 and §3.8
  (the B.2.0 recipe repeated: `src/` 29,190 lines, in-file tests 23%,
  instrumentation about 2,300 non-test lines and 437 of `negamax`'s 1,662; two
  dependency cycles; 657 public items, 77 unreferenced outside their file; the
  kernel within 1.3× the donor's after instrumentation, the root driver 630
  lines against 244). Every item is its own branch and squash, `I1` with the
  exact fingerprint on both slider backends, the suites, fmt and clippy;
  NPS-guarded means the allocation guard and one pooled-NPS read taken alone.
  D.2 and D.3 are closed, so nothing here waits: the items run in order, each
  on its own branch. P16, the comment pass, merged into D.5.16 on
  2026-11-05.
  D.8.6, D.8.7 and D.8.19 are read again after D.7 (2026-11-13): a kernel
  rebuilt there is written in their target shape.
    - **D.8.1 P0: the census as a tool — `I1`.** Commit the census as `tools/diag/arch_census.py` (planned) with a test
     (masked-source parser, `use` edge graph, public census, shape table,
     instrumentation attribution, duplicate finder); it reproduces the
     review's headline numbers at `a3dd950`, and E.1 re-measures with it.
    - **D.8.2 P1: in-file tests into child files — `I1`.** In-file tests into `<stem>/tests.rs` for the 13 files holding
     5,391 test lines (`node.rs` 3,609 → 2,462, `search/mod.rs` 2,977 → 1,979,
     `time.rs` 966 → 392, `king.rs` 924 → 521); test bodies untouched, counts
     equal before and after, release bytes identical. Neutral by construction.
    - **D.8.3 P2: one `score` module — `I1`.** One `score` module: the constants, one `MAX_PLY` (defined twice
     today), `MATE_BOUND`, `is_decisive`, `mate_in`; the nine inline threshold
     sites; ends the `eval` and `tt` cycle. Neutral.
    - **D.8.4 P3: geometry into one board module — `I1`.** Geometry into `board::geometry` (the `BETWEEN`/`LINE` tables and
     the six tables in `eval/pawns.rs`); one attacked-square predicate; one
     `ep_capture_is_legal`; one distance builder; a `TbPosition` method per
     Fathom entry point instead of eleven positional scalars;
     `legal_ep_capture_exists` returns `bool` (its always-`Ok` `Result` is an
     out-of-line call in `make_move`). NPS-guarded.
    - **D.8.5 P4: split `board/board.rs` — `I1`.** Split `board/board.rs` (one 2,066-line `impl`) into `fen`, `see`,
     `draw`, `checks`, `make`, before F.1 touches make/unmake. NPS-guarded.
    - **D.8.6 P5: delete `trace_decision!` — `I1`.** Delete `trace_decision!` (about 190 lines; no tool, document or
     analysis consumes it); retire condition: no open leaf registers a
     decision-trace read. Neutral.
    - **D.8.7 P6: the `diag` blocks into probe functions — `I1`.** Fold the 32 multi-line `diag` blocks into probe functions in
     `diag.rs` with empty `#[inline(always)]` twins (a trait object or a
     strategy type parameter was weighed and rejected); `negamax` 1,662 →
     about 1,270 with P5; the counter set equal at stride 1. NPS-guarded.
    - **D.8.8 P7: visibility follows use, second pass — `I1`.** Visibility follows use, second pass: the 32
     `pub(super)`/`pub(crate)` items that can be private and the 52 `pub`
     items referenced only inside `src/`; `pub(crate) mod diag`; `win32`
     narrowed. Neutral, compiler-checked.
    - **D.8.9 P8: delete the family masks — `I1`.** Delete the family masks (`eval/masks.rs`, the `Evaluator` fields,
     the `king.rs` hook and its tests, the two options; about 200 lines) once
     the maintainer confirms no open read sets `KingDangerMask` or
     `KingShelterMask`. `tune` builds only.
    - **D.8.10 P9: one `ArrayList<T, N>` — `I1`.** One `ArrayList<T, N>` under `MoveList` and `ScoredMoveList`; one
      `unsafe` fewer; keep the `MaybeUninit` init and its measured reason.
      NPS-guarded with the assembly `memcpy` check.
    - **D.8.11 P10: one `Compiled<T>` — `I1`.** One `Compiled<T>` in `infra`, a `runtime_params` cfg emitted by
      `build.rs` for `tune`, `texel` and test profiles, an `Evaluator` variant
      field; `texel` builds then read search parameters at run time (never
      measured for strength). NPS-guarded, read alone: C.12's folds were
      measured only inside bundles.
      With it (2026-11-03, `analysis/triumviratus_review_2026-11-03.md` §4
      item 9): the six `SearchParams` values still read from memory on hot
      paths outside `tune` (`qs_see_margin`, `qs_see_clamp_hi`,
      `qs_see_bad_floor`, `nmp_min_non_pawn_pieces`, `probcut_see_gap_scale`,
      `lazy_margin`).
    - **D.8.12 P11: split `search/mod.rs` — `I1`.** Split `search/mod.rs` into `root`, `report`, `tb`.
      Neutral.
    - **D.8.13 P12: the adapter boundary, inside D.3 — `I1`, DONE 2026-11-04.**
      Done inside D.3.9 (`cef6e97`, `d0759b9`). Record: `docs/archive/PLAN-phaseD-done-2026-11-04.md`.
    - **D.8.14 P13: one iterative-deepening driver — `R2`.** One iterative-deepening driver for single and MultiPV (53% of
      the MultiPV driver repeats the other): `R2` first on how aspiration
      state, `MateWatch`, the votes and the tablebase extension carry per
      line; `MultiPV 1` node-identical, `MultiPV > 1` judged by
      `tests/multipv.rs` and `multipv_syzygy.rs`; falsifier: a branch on the
      single-line path that costs NPS in the pooled read.
    - **D.8.15 P14: bench and WAC under one suite runner — `I1`.** Bench and WAC into `src/tools/` (planned) with one `run_suite`; delete
      `tests/board_performance.rs` (the third copy of the board workloads,
      gates nothing); `tests/common` (planned) for the two counting allocators and the
      repeated helpers; stale tool names (`AblationMask` in
      `fixed_budget_probe.py`, the legacy label in `feature_matrix.py`, the
      path in `conversion_audit.py`). Neutral. One `run_suite` already serves `bench`
      and `wac` in `engine.rs` (`27cd37e`, with D.3.9); the move to
      `src/tools/` (planned) and the rest remain.
    - **D.8.16 P15: the evaluation's shape — `I1`.** The evaluation's shape (EVAL-8): a `Score` accumulator and one
      `term!` macro emitting both halves and both trace sites, a per-colour
      context for the ten-argument term functions, `forward_one` and
      `pawn_attacks_of` used everywhere; fingerprint exact plus the `texel`
      reconstruction test.
    - **D.8.18 P17: lints — `I1`.** Lints (RUST-9, RUST-10): enable `cast_lossless`,
      `borrow_as_ptr`, `checked_conversions`, `manual_assert_eq` and
      `no_effect_underscore_binding` (65 mechanical edits, same code), the
      three plain `allow`s to `expect`, reasons on the twelve bare `expect`s,
      `clippy::allow_attributes` on; `suboptimal_flops` rejected because
      `mul_add` changes the clock's f64 results.
    - **D.8.19 P18: the node frame — `I1`.** The node frame (RUST-3, NODE-9): every `negamax` and
      `quiescence` copy has a 5.0–5.6 KB frame and calls `__chkstk` per node
      (the picker's 4,104-byte list and the ProbCut and quiescence lists); a
      design read first (per-thread ply-indexed list storage borrowed by slice
      splitting), stop under 0.5% NPS; a two-field `Copy` node-kind struct for
      the bool, `Move`, bool argument run; cold blocks outlined
      `#[inline(never)]`. Triumviratus's prior (2026-11-03, TB-11 in
      `analysis/triumviratus_review_2026-11-03.md`): moving its list fills
      out of line and its evaluation lists off the stack, so that no hot
      frame passed 4 KB, saved 1.55% of cycles on Windows.
    - **D.8.20 P19: evaluation terms computed for a constant — `I1`.** Evaluation terms computed for a constant (added 2026-11-02
      from the Basilisk cross-check, §E): Basilisk stopped computing
      terms whose every weight was 0 for +2.35% NPS (BAS-P25). A census
      of Rarog's production terms whose fitted weight is 0 or folds to a
      constant (EVAL-11 names `hanging_rook` and `hanging_queen` at 1 and
      trapped bishops at 0.04% of rows) first, by reading; then the
      removals with the exact fingerprint, the `texel` reconstruction
      test and one pooled-NPS read. Terms E.4 or F.4.1 would refit stay.
  Held, with the reason: a stage split of `negamax` (R2 only if P5 and P6
  leave it unreadable); a unified legal move generator (about 300 lines, the
  highest NPS risk, no defect behind it); the table's local backend (D.2,
  F16); the controller thread and its epochs (B.3.5); any `inline(always)`
  pass (B.2.0 F24); the backup harness scripts (AGENTS); the three features
  themselves (all in use). Kept after review, with their records:
  `cpu_advice`, `crash_report`, `large_pages`, `wac`, the `diag` counters, the
  `tune` options and the `texel` trace in the engine crate.
- **D.9 Tablebase policy: probing depth/limits, WDL/DTZ in conversion, recogniser interaction — `R2`.** Root and interior probing depth and limits,
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
  is near-zero Elo while the risk lands on C.5's truth source, and D.9 must
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
  `probe_root` and its `unsafe` site go in D.5.
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
- **D.10 Large pages on Linux: transparent huge pages for the big tables — `I1`, then `V`.**
  Renumbered from D.7 on 2026-11-13 (maintainer decision), when the
  search-shape step took that number.
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
- **D.11 Originality disclosure before the release cut: the donor material named, the licence settled, the notices shipped with the binaries — `R2`, then `M`.** A
  small research step with its implementation. Added 2026-10-23 by
  maintainer decision. It held the 2.7.0 cut until 2026-11-13, when the
  maintainer lifted the hold for 2.7.0; the release after 2.7.0 does not
  cut until it closes. Known
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
  Added 2026-11-03: Triumviratus (GPL-3.0) is a third donor, for
  mechanisms only (PROCESS, *The independence boundary*); nothing from it
  is in the code on that date. A mechanism from it that ships before the
  cut is named in the disclosure.
  Moved from E.3.0 on 2026-11-13 (maintainer decision): the disclosure is
  a Phase D leaf. The 2.7.0 cut runs ahead of D.7, and the hold is lifted
  for it (maintainer decision 2026-11-13): 2.7.0 ships with 2.6.0's
  notices, and the disclosure closes before the release after it. A
  kernel rebuilt in Reckless's form (D.7) makes it heavier for that
  release.

## Phase E — Classical checkpoint and release

- **E.1 Attribution checkpoint: B.2.0 review re-run on the B.9/C.11 heads; STC, `10+0.1`, 4T against 2.3.2 and the B.9/C.11 heads; maturity checklist — `V`.** Re-run the B.2.0 architecture review
  on the B.9 and C.11 heads first. Final head against 2.3.2 and against
  the B.9 and C.11 heads at STC, `10+0.1` and 4T; attributed Elo per programme
  from the accepted SPRTs; deficit meters; NPS; the maturity checklist
  (family map without unknown rows, every slot with a fitting instrument,
  every accepted representation reconstructing through `EvalTrace`).
  **The architecture half of the re-run was done on 2026-11-01**
  (`analysis/engine_review_2026-11-01.md` §3.8, the B.2.0 recipe repeated on
  the D head; its census becomes `tools/diag/arch_census.py` (planned) under D.8 P0).
  E.1 keeps the attribution re-run and the maturity checklist and re-measures
  the architecture counts with that tool at the release head.
  Add a tablebase arm (H30): the head against the last release with Syzygy
  loaded, since the search has its own probes and CCRL testers use tables;
  Rarog's last games with tables predate D.1 (RAR-S97, 2026-10-01).
  The loss table by cell (TT-1, D.4.5's tool) is read in H.3, on the
  network head's games (moved 2026-11-05: a classical head's games describe
  an engine H.3 will not tune).
  The wider round robin adds Stockfish 15.1 with `Use NNUE` off beside
  Stockfish 11 (3,568 against the C head's 3,479, RAR-M75): the classical
  evaluation under a later search, never rated in this pool (2026-11-13).
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
- **E.3 Release through the tag-driven flow; 3.0.0 is reserved for the first NNUE release — `M`/`V`.** Version, changelog, release notes, fmt, debug and
  release suites, clippy, feature builds, fingerprint, PGO assets, ISA
  verification, CI matrix, tag and publish on maintainer instruction. E.2's
  result sets no version (maintainer decision 2026-10-22: Rarog 3 is
  the first NNUE Rarog, F.9; E.2's result is recorded, not a version number;
  2.5.0 was cut at B.10 and 2.6.0 from the C head ahead of E, maintainer
  decisions 2026-10-03 and 2026-10-22). Later classical gains (E.4, F.4.1)
  release after it under Phase F's rule. The release is cut through the
  tag-driven flow of **E.3.1**, which also carries the two workflow checks
  this release owed (tag equals version; one fingerprint asserted across the
  matrix) and may land any time earlier.
  **Decision, 2026-11-13 (maintainer):** 2.7.0 is an unscheduled
  release, cut from the current head before D.7 by PROCESS *Release*
  with no roadmap step of its own; E.3.4 stays in place for E.3's
  release, its version set when it is cut. 2.7.0 brings the fixes in
  `[Unreleased]` and the new executables to users and testers now; a
  kernel D.7 adopts releases as 2.8.0. The condition is no loss of playing strength against 2.6.0,
  read by a registered match before the cut: the last read (RAR-M84,
  2026-11-08, unregistered) had that day's head level with 2.6.0 head to
  head and 13.6 ± 9.9 Elo lower against the shared field, before D.6.2's
  nElo +12.2 ± 6.5. The holds of D.11, D.6.17 and D.6.20 on the cut
  are lifted for 2.7.0 (maintainer decisions 2026-11-13): D.11's
  stands for the release after it, and D.7.8 asks first whether
  D.6.17's tie and D.6.20's clock test survive the new kernel.
  **The read, 2026-11-13 (RAR-M87): a loss.** Against a five-engine
  field the head is G −7.6 Elo [−14.4, −0.7] below 2.6.0; head to head
  it is +10.1 ± 8.7. The maintainer decided after the result, as an
  exception for 2.7.0 and not a release rule, to ship it as it is for its
  fixes. The head leaves about 5% of its clock unspent, a lead for a
  research card, untested. **Released 2026-11-13** as 2.7.0 from the
  `v2.7.0` tag, fingerprint 9,077,016 / EBF 2.481.
    - **E.3.1 Tag-driven release flow — `I1`, DONE 2026-10-05.** `release.yml`
      and `cargo xtask release-check`; 2.5.0 was released through it, nine
      assets at 11,171,726 (run 37284706939). The procedure is
      `docs/PROCESS.md` *Release*; the leaf's text is in `docs/archive/PLAN-closed-2026-10-05.md`.
    - **E.3.2 Build names: one scheme for `id name`, the start-up line, the files and the build command, and an `sse42` tier — `I1`, DONE 2026-11-04.** Added
      2026-11-04 by maintainer decision, with E.3.3, ahead of Phase D's
      remaining work. The form is Basilisk's
      (`D:/code/basilisk/cmake/git-version.cmake`,
      `D:/code/basilisk/src/version.cpp`,
      `D:/code/basilisk/cmake/dist-copy.cmake`), so the maintainer's engines name their
      builds alike. The contract:
      - `id name` is `Rarog X.Y.Z` at a release and, while `Cargo.toml`
        holds a `-dev` version, `Rarog dev-<commit date, yyyymmdd>-<8 hash
        characters>`, with `-m` before the hash when the tree differs from
        its commit (Stockfish's form). `RAROG_DEV_ID=<date>-<hash>` fixes
        that part for a byte-identity comparison, since the name is in the
        binary. `id name` never names a tier, a kind or a feature: rating
        lists key on it, and of the library's engines read on 2026-11-04
        Stockfish 12 onward, Reckless, Beast and Basilisk name none of
        them (Ethereal 13.07, Fire 8.3 and Stockfish 10 and 11 named the
        tier).
      - The start-up line names what `id name` does not: the tier and any
        kind or feature (`(pext)`, `(pext, nopgo)`, `(native)`,
        `(pext, tune)`).
      - A file is `rarog-<version>-<os>-<arch>[-<tier>][-<kind>]`: the
        version as `id name` gives it, without a `v` (the git tag keeps
        `vX.Y.Z`); `windows`, `linux` or `macos`; `x64` or `arm64`; on a
        single-tier x64 build its tier (`base`, `sse42`, `avx2`, `pext`,
        or `native` in its place), and no tier on arm64 or the universal
        binary; then the kinds, on anything but a PGO build with the
        default features: `nopgo`, `debug`, the engine's features `tune`,
        `diag` and `texel`, then a candidate arm's own feature, sorted. The release's
        `rarog-v2.6.0-windows-pext.exe` becomes
        `rarog-2.7.0-windows-x64-pext.exe`, and today's
        `rarog-v2.7.0-dev-windows-pext-pgo.exe` becomes
        `rarog-dev-20261104-209b555a-windows-x64-pext.exe`.
      - The tiers are the x86-64 levels: `base` x86-64; `sse42` x86-64-v2,
        new (SSE4.2, SSSE3 and POPCNT: Nehalem to Ivy Bridge and AMD
        Bulldozer to Steamroller, the CPUs Basilisk's `sse41-popcnt` tier
        serves); `avx2` x86-64-v3; `pext` x86-64-v3 with the PEXT slider
        path. `sse42` is a single-tier build and asset and a tier of
        E.3.3's universal binary (maintainer decision 2026-11-04), with its
        own `verify-isa` contract. Until E.3.3 ships, the release carries
        each x64 tier for Windows and Linux and the three arm64 assets.
      - `cargo xtask build`, the same as `cargo xtask build --release`,
        builds the best tier this host runs, with PGO and that tier's
        portable code generation, never `native` (RAR-P51: −3.8% here).
        `--tier` picks another tier the host runs (PGO trains by running
        the build, so a tier above the host is refused); `--nopgo`,
        `--native` and `--features` make the other kinds; `--universal` is
        E.3.3's. A plain `cargo build` stays the unnamed development and
        test build.
      - The callers follow: `tools/build_test.ps1`, `release.yml` and
        `release-check` (which assert the name), PROCESS *Build* and
        *Release* step 6, the README's download table, a CHANGELOG entry
        for the new file names, and AGENTS' pool-identity rule (the dev
        name identifies the commit; a feature arm is named in its file
        name and its sidecar). Testing a dirty build stays refused
        (`harness_common.ps1`).
      Verification: every tier benches the declared fingerprint, so the
      change is behaviour-neutral; unit tests of the name for a release, a
      development build, a dirty tree and a fixed id; `verify-isa` fails a
      planted `popcnt` in `base` and an AVX instruction in `sse42`; one
      pooled NPS read of `sse42` over `base` on the 5950X, indicative of
      what older CPUs gain. Not in this leaf: whole-number releases
      (`Rarog 3`, `Rarog 3.1`), perhaps by dropping trailing zeros from the
      shown version, an idea for F.9 still to be discussed.
      Implemented 2026-11-04 on `e32` in three commits (engine `ef8d60e`,
      tooling `b4cbb13`, documents `d5f23da`): `src/build_name.rs` and
      `src/cpu_tier.rs` hold the rules once and are compiled into
      `build.rs`, the engine and `xtask`; `xtask` runs every build it makes
      and refuses one whose `id name` or start-up line contradicts its file
      name (a planted wrong label was refused and its file removed). Every
      tier (`base`, `sse42`, `avx2`, `pext`) and the `tune` and `native`
      builds bench 14,749,043 on this host and pass their ISA contracts;
      `verify-isa` refuses the `sse42` build as `base` (237 `popcnt`) and
      the `avx2` build as `sse42`. The staged test-engine names
      (`tools/test_engines/rarog-<suffix>-pext-pgo.exe`) are unchanged, so
      the harness scripts and recorded recipes that name them still work.
      The pooled NPS read (RAR-P55): `sse42` **+8.06%** [+7.57%, +8.55%]
      over `base` on the 5950X, indicative of what older CPUs gain;
      `nps_read.py` now reads a pool of any tier (`145ab14`).
    - **E.3.3 The universal x86-64 binary: one release file per OS that picks its tier at start-up, at the single-tier build's speed — `R2` card, then `I2`/`V`, DONE 2026-11-04.** Moved
      from G.2 on 2026-11-04 by maintainer decision. Its record
      (`analysis/universal_binary_2026-09.md`) deferred it at about zero
      Elo against a correctly chosen asset; its third trigger is taken as
      fired: unattended testers pick an asset without reading start-up
      output, a `base` asset on a `pext` CPU gives up 6.65% (RAR-P22,
      about 13 Elo at RAR-M20's rate), `pext` on Zen 1 or 2 is the slowest
      engine, and 2.7.0 is the release CCRL will test. The design is the
      record's section 5 (a): every tier its own complete PGO build, linked
      behind a dispatcher in baseline code. Basilisk built and shipped it
      (the record's section 9). The card settles the Rust route with the
      cheapest checks first, in this order, each able to stop it:
      1. rustc keeps the post-LTO object of a tier's ordinary PGO build
         (`-C save-temps`), and that object linked alone reproduces the
         fingerprint with its hot functions byte for byte the tier
         build's. This is Basilisk's route: symbols are renamed after PGO,
         so no profile is matched against renamed symbols, and section 7's
         metadata trap does not arise.
      2. Every symbol a tier defines is renamed or made local except its
         entry, what fat LTO pulls in from `std` included, and no defined
         symbol is shared between tiers (`llvm-nm`, intersected).
      3. `std`'s own initialisers in each tier (the Windows TLS callback,
         the Linux `.init_array` argument capture) are listed, and each is
         shown harmless when every tier runs it, or removed.
      4. `-C llvm-args=-align-all-functions=6` reaches LTO code generation,
         so every tier function starts on a 64-byte line (Basilisk's
         unaligned relink read −0.58%).
      If check 1 fails, the fallback is four `staticlib` tiers trained
      inside one instrumented universal build, each through the
      forced-tier override; if that fails too, the leaf closes `NO_CHANGE`
      and E.3.2's per-tier assets ship. The build then: the dispatcher from
      `cpu_advice` (feature bits with the OS's YMM state; AMD family 15h
      and 17h kept off `pext`); `RAROG_ISA` forcing a tier and refusing one
      the CPU cannot run (exit 1 with an `info string`); the start-up line
      naming the chosen tier, `id name` unchanged; Fathom linked once in
      baseline code; `cargo xtask build --universal`; every trap of the
      record's sections 3, 7 and 9 checked and seen to fail on a planted
      fault; every forced tier benched for the fingerprint; an Intel SDE
      selection matrix in `release.yml`. The gate, its threshold frozen in
      the card before the read: a pooled NPS read of the universal binary
      against the single-tier `pext` build, no regression, with Basilisk's
      separate placement and alignment contrasts. On a pass the x64
      release is `rarog-<version>-windows-x64.exe` and
      `rarog-<version>-linux-x64`, with no per-tier x64 asset (maintainer
      decision 2026-11-04): five assets with the three arm64. Measurements
      stay on the single-tier `pext` build.
      Card 2026-11-04 (`analysis/universal_binary_2026-09.md` section 11):
      the four checks pass on Windows. The kept post-LTO object relinks to
      the build's text byte for byte; fat LTO leaves three globals a tier,
      renamed after PGO; the tier objects carry no initialiser; the
      alignment reaches LTO. Built by `cargo xtask build --universal`, each
      tier and the whole image held to their ISA contracts, every tier
      forced at 14,749,043. Its speed gate is RAR-P56, registered before the
      read: **PASS**, the universal binary **+0.45%** [+0.17%, +0.72%] over
      the single-tier `pext` build. By the maintainer's decision it is now
      the x64 release asset: `release.yml` builds it for Linux and Windows,
      benches every forced tier on the runner and checks the selection under
      Intel SDE (`tools/diag/check_universal.py`), five assets in all. The
      Linux form, built in WSL (Ubuntu 24.04, a clean clone): every check
      passes, five globals a tier (std's argument-capture static among
      them), its one `.init_array` entry harmless (each forced tier read its
      own `bench 13` argument), glibc 2.34; the link now strips debug
      information as cargo's does, 8,346,928 bytes from 14,083,904
      (`d1bbb60`). By maintainer decision 2026-11-04 every `cargo xtask`
      build for Windows and Linux links its C runtime statically, so a user
      needs nothing installed (macOS cannot), and xtask refuses one that
      does not (RAR-P57: Windows +0.13%, a pass; the WSL read of Linux was
      too noisy to resolve, its speed question open for a native host).
      Both universal binaries are static and pass every check (Windows
      `dac14c5`; Linux `dc66f27`, 9,435,472 bytes, no loader): the first
      static Linux link crashed before `main` because it passed the C
      libraries again after rustc's own static group, and the universal
      check caught it. The first `Release` run (41896823228, on PR #14's
      head `2fb6df1`, Linux built on ubuntu-24.04) passed the SDE matrix:
      every emulated CPU chose its expected tier and benched 14,749,043,
      eight on Linux and six on Windows, and `pext` forced on Sandy Bridge
      was refused. SDE emulates only Intel parts, so the `avx2` tier's
      selection (AMD Zen 1 and 2) is not exercised there; the forced-tier
      benches run that tier. Landed with E.3.2 as `5efe16f` (PR #14). The
      static Linux runtime's speed stays open for a native host (RAR-P57).
    - **E.3.4 Release cut: the version, the `[Unreleased]` changelog reviewed and dated, suites, the release PR squash-merged, the `v` tag pushed on instruction through E.3.1's workflow — `M`.** E.3's own work, a leaf of its own since
      E.3.1 closed (2026-10-05), numbered E.3.2 until 2026-11-04: the version from its `-dev` form to the release's,
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
  F.0; a win also strengthens F.4's labels, which the classical head
  generates. Research card first: which structural difference carries
  the signal, the cheapest static falsifier, the unit's handoff. Then the
  unit in Rarog's own structure, fitted on `hce-v4-tb` by C.3's pipeline and
  gated by PROCESS's *Evaluation change under a fitted search* (gate 1
  `[0,3]`, the margin block if flagged, gate 2), with RAR-O07's same-search
  meter re-read at the close. Prior from Phase C's units: +10 to +25 Elo,
  frozen with its band at the registration. Accepted, it is released as the
  classical line's next version through E.3.1's flow and becomes the
  classical reference of Phase F's rule; `NO_CHANGE` closes the family work
  and leaves F.4.1 as the classical programme's last experiment.
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
  pawn PST (EVAL-13). And the piece-term definitions no experiment read
  (EVAL-10; D.6.11 until 2026-11-05): outposts use the current enemy pawn
  attacks rather than the attack span; minor-behind-pawn counts own pawns
  only; the long diagonal reads the enemy king zone rather than the two
  centre squares. Screen before the fit: an E62-style direction read, zero
  games; a term at the line enters this unit's refit. If the rebuild
  moves the endgame king danger, read the
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

**Lessons from Triumviratus's network path** (added 2026-11-03;
`analysis/triumviratus_review_2026-11-03.md` §11, §12, §12b; IDs are its
items). Triumviratus trained its own weights from random initialisation, but
on public Leela and Stockfish data; this phase's rule excludes that recipe,
and its architecture, inference, training method and failure list transfer.
Its seven networks lost it Elo in ways each visible in advance: a first run
102 below the reference for want of a factoriser, with a pure-evaluation
target and an early stop; a self-play stage 84 below for want of label strength; a
plateau a λ cycle hid; gains that vanished behind shared cores. In the order
this phase meets them, each is now a requirement of the leaf named:

1. conformance gates before any training run (F.2; TN-9);
2. a recipe checked against a reference recipe before the first paid GPU
   hour (F.3; TD-11);
3. one label scale per run, and labels deep enough, sized by a learning
   curve (F.3 reads the curve, F.4 generates to it; TD-9);
4. the training stream's shape measured against the positions the search
   evaluates (F.4; TD-10);
5. runs watched on holdout loss at fixed λ=0 and λ=1, with λ fixed early
   (F.3), and nets judged at `20+0.2` or longer against one fixed anchor
   (F.6; TD-1, TD-12);
6. capacity or new inputs, not more epochs, once a net saturates (F.7;
   TD-13);
7. each network step as CPU-only gates, then proxy trainings each judged by
   an SPRT, then one full run, close calls taking the conservative choice
   (F.0, F.7; TD-16).

The steps run in that order (reordered 2026-11-05 so that no step waits for
a later one): the engine reads a network exactly before any is trained (F.1,
F.2), the trainer is proved on a pilot slice and sizes the corpus (F.3)
before the corpus is generated (F.4), and the first nets are judged in games
only once inference runs at speed (F.5, F.6).

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
  for the data-format contract F.0 fixes for F.4.
  Triumviratus inputs (2026-11-03; §12b, TN-1, TN-3, TN-6, TD-16): its network
  path, net by net, read before the first architecture and recipe are chosen;
  its cost-ledger points (L1 2048 −20% NPS, L1 1536 −11.2%, four input-layer
  experts at L1 1024 −7.5%, a mobility block −16.2% untrained, the network on
  large pages +6.7% at 250 MB); a refusal prior, a policy network inside
  alpha-beta, which lost in four forms (−22.8 to −85) because ordering already
  found the cutoff move first 79.2% of the time without a hash move and a
  static prior from the parent loses to histories that adapt to the subtree;
  and the state of Stockfish's networks in October 2026 as F.7's survey (PSQT
  removed in SFNNv17; int8 first-layer weights trained by clamping; king ×
  opponent-queen buckets with 32 layer stacks in SFNNv18, about +4 together
  with every single change failing alone; row pairing in the update, +5% to
  +7% NPS; learned routers for the layer stacks; pawn input blocks in test).
- **F.1 Board events and accumulator scaffolding, behaviour-neutral for HCE; cost ledger — `I2`.** Behaviour-neutral
  for the HCE: factual move deltas, evaluator-owned stacks, validity and
  refresh semantics, randomized unwind tests, exact fingerprint, pooled NPS
  cost recorded.
  Triumviratus input (TB-10, `analysis/triumviratus_review_2026-11-03.md` §6):
  one board for search and network, make recording only the moved piece and
  the pending moves replayed from the current board before an evaluation.
- **F.2 Scalar integration: `quantised.bin` contract, integer-exact conformance, HCE fallback — `I2`.** `quantised.bin` contract, integer-exact
  conformance against the trainer's reference evaluation, clean HCE fallback.
  Proved on nets F.0's trainer exports untrained, random and with perturbed
  parts, so it needs no trained net and comes before every training run.
  Triumviratus's conformance gates, required before any training run
  (2026-11-03; TN-9): an integer evaluator matching the trainer on exported
  nets; loader indices against an independent reader (0 of 7,025 wrong there);
  a random net with perturbed per-bucket parts, since zero deltas hide a wrong
  bucket index; the engine against the evaluator move by move across bucket
  changes; a golden position set bit-identical after any rewrite (2,000,124
  positions there).
- **F.3 Trainer hardening: a deterministic pipeline checked against a reference recipe on a pilot slice, and the learning curve that sizes F.4 — `I2`, then `V`.** Deterministic
  pipeline, two seeds per configuration, validation selects, frozen test
  reports once. It trains on a pilot slice generated to F.0's data contract
  (maintainer-run, hours), checks the recipe against a reference recipe
  there, and reads the learning curve (effective independent rows, label
  depth) that sizes F.4's corpus (H33; Basilisk's C.2.2 is the form). Its
  runs are judged by holdout loss; nets are first judged in games in F.6,
  once F.2's conformance and F.5's speed stand, and the judging lessons
  below apply there.
  Triumviratus inputs (2026-11-03; TD-1–TD-4, TD-7, TD-11, TD-12, TD-15).
  Check every recipe against a reference before paying for a run: its first
  net (no factoriser, λ 1.0 only, a stop on a flat validation loss while the
  rate was still 38% of its start) read −102, and −39 once fixed. Watch
  holdout loss at fixed λ=0 and λ=1, never the training loss at a moving λ: a
  λ cycle climbing back to 1.0 made its net imitate the labels while the
  result loss stopped where Elo flattened, and a fixed λ of 0.75 restored the
  gain. Register checkpoints with Elo targets (an early logarithmic fit
  predicted −32 at the plateau; −70 was measured). Judge checkpoints at
  `20+0.2` or longer against one fixed anchor (one net read −27 at 8+0.08 and
  −84 at 20+0.2; checkpoint-to-checkpoint gains summed to +71 where the direct
  gap closed by +28), never by a validation loss at a moving λ; the first
  epochs after a restart at a higher rate are recovery, not progress. A
  restart at a fifth of the peak rate moved a plateaued run, and the final net
  is the average of the last few epochs' weights (+3.8 ± 7.7, kept for its
  lower variance). The loader, not the GPU, was the limit (one feature thread
  per GPU starved it: 381k to 1.9M positions a second with more). Nets are
  compared in a sibling-free slot: one net read +0.8 at 75 concurrent games
  and +23.4 at 38.
- **F.4 Data generation at scale: the corpus F.3's learning curve sizes, splits, manifests, hashes — `V`.** The corpus F.3's learning curve sizes, from the
  classical head under the adjudication-off profile, by-game splits,
  manifests, tablebase and hard-position cohorts; hashes frozen. Maintainer-run.
  The generator follows F.0's contract; node-limited play with soft and hard
  caps and inverse-use-count book sampling are the reference forms
  (`analysis/gyatso_read_2026-09-26.md`). 30–60M unique positions was the
  first guess (H33); F.3's curve replaces it before the maintainer spends
  the hours.
  Triumviratus inputs (2026-11-03; TD-9, TD-10), read for self-play. One label
  scale per run: data from successive heads is re-scored by one search before
  it is mixed, because two target distributions in one run train toward
  neither. Label strength over volume: its net trained on 5,000-node self-play
  alone ended −84 against the reference, and a second stage with stronger
  labels closed half the gap, so the node budget per label is sized by the
  learning curve, not by throughput. The stream's shape measured, not assumed:
  files drawn uniformly over-read small files; sampling by piece count feeds
  the endgame buckets a material key otherwise starves; early skipping soft
  rather than hard; the positions the search evaluates compared with the
  training stream bucket by bucket (on Triumviratus the search held 2–3.5×
  more unbalanced material pairs than its data). Game positions at every ply
  were the right source; book exits and endgame books were not.
    - **F.4.1 Classical refit on search-visited data: the frozen HCE's whole surface refitted on F.4's dataset, gated as an evaluation change; the classical line's release if it wins — `V`.** The
      lever Phase C's review left (`analysis/phase_c_review_2026-10-19.md`,
      question 3; added 2026-10-22 by maintainer decision): the same-search
      meter's remaining 99 Elo (RAR-O07) is not in the families, which read
      at static parity with the donor on played positions (RAR-E70), but in
      how the evaluation scores the positions the search visits. F.4's data
      is those positions with deep labels, and `tools/texel` (C.2's
      protocol) reads it at no new instrument cost. The whole surface, E.4's
      king unit included, is refitted on F.4's data, screened statically on
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
      Also (2026-11-03, TE-1 in `analysis/triumviratus_review_2026-11-03.md`):
      Triumviratus's concept residual regression (a shallow score against a
      deep one, about thirty concepts regressed with score, material and piece
      count held fixed, |t| > 3 a signal) as a static screen on F.4's
      positions before the fit.
- **F.5 Incremental and SIMD: same-net parity on every move type, tiers, pooled NPS attribution — `I2`, then `V`.** Same-net incremental parity
  on every move type, SIMD tiers (AVX2, PEXT builds, ARM NEON), scalar
  reference retained, pooled-PGO NPS attribution. Priors against (H36,
  BAS-P26): slider tables compressed and expanded with `pdep` −0.98%
  [−1.19, −0.78] (the flat tables' hot lines stay in cache), `-mtune=znver3`
  −0.95%, FMA/F16C/MOVBE +0.02%.
  Triumviratus inputs (2026-11-03; TN-5, TN-7, TN-8, TD-15, TB-14, TB-15):
  incremental against refresh at every node (its `nnperft`); measured there: a
  refresh cache per bucket, a pawn-block refresh cache keyed by the pawn
  bitboards (+2.8%), the hybrid update on king moves (Stockfish `db98633b`,
  +3.4%), both perspectives in one dirty-list pass (+1.5%), first-layer
  neurons permuted at export over all input blocks so zero activations fall in
  skipped blocks (+1.6% to +2.5%, bit-identical; permuting only some blocks
  makes the net wrong, not slow), no feature opened and closed inside one
  capture, the network on large pages (+6.7% at 250 MB), an AVX-VNNI tier (+6%
  to +9%). Memory is the cost: 47% of last-level misses fell in the
  incremental update and 11% in refreshes; a split accumulator cost +12%
  cycles, and 8-bit weights quantised after training were slower. Whether the
  evaluation cache still pays once the TT stores the network's evaluation (12
  hits in 2.5M lookups there).
- **F.6 The first nets and the search re-fit: baseline nets judged against one fixed anchor, then the search refitted to the network — `V`.** Baseline
  nets first: two seeds per configuration on F.4's corpus, validation
  selects, the frozen test reports once, and each net is judged at
  `20+0.2` or longer against one fixed anchor in a sibling-free slot (F.3's
  lessons). Then the search: score scale, correction
  histories, margins, qsearch and SEE thresholds re-fitted on the new
  evaluator (C.10's protocol). The printed score's scale is fitted here
  too, per network, so that a stated score reads as a stated chance to win
  (D.3.9's normalisation card, closed `NO_CHANGE` 2026-11-04 for this
  phase; the formatter is `uci::info::format_score`).
  Triumviratus inputs, to settle before this tune's registration (2026-11-03;
  TD-8, TM-3, TM-5, TM-8, the review's check 15). The scale per piece-count
  bucket recalibrated for each net by games (matching the raw ratio to the
  previous net lost), the scale-dependent margins retuned on the final net
  (+8.6 there), blend constants inherited from another net tuned on this one
  (+3.75). A lever bakes only when its net move is at least 2.5 times the
  random-walk expectation (with 37 levers about 11 look monotone by chance).
  Start ± 2.25c clear of a formula's change of nature (at c₀ this fails for 43
  of RAR-S104's 82 coordinates). Tree size centred by a fixed-position node
  sweep before the SPSA. A tune taken at the control it is used at. The method
  changes are the maintainer's decision.
- **F.7 Architecture ladder: output buckets, king buckets, relation/threat inputs; one axis at a time — `R3` with `I2`/`V` sub-steps.** Output buckets,
  king buckets with mirroring, then relation and threat inputs as in
  Reckless, one axis at a time; each net gated against the previous.
  Triumviratus inputs (2026-11-03; TN-1–TN-6, TE-2, TD-6, TD-13, TD-16). The
  rungs it climbed and what each bought: threat inputs over plain
  king-relative inputs (+50 for the engine), a pawn-pair block grafted at zero
  (+18), a passed-pawn block trained alone on the frozen base (+7 in four
  epochs), the whole net retrained from scratch on one label scale (+23); then
  saturation, where more epochs, an annealing tail and fine-tunes all read
  zero and only capacity paid: input-layer experts by total material as a
  shared base plus zero-started deltas (about +15, at −7.5% NPS), and per-side
  experts by the opponent's non-pawn material with int8 king-relative weights
  and a cached pawn-king subnetwork (+10.3 ± 4.3 at fixed nodes). Bucket keys
  chosen by measurement: its net's own regimes followed material × queens
  (normalised mutual information 0.43). The zero-initialised graft as the
  cheap screen, with its limit: frozen later layers may not use the new
  information, and a block started from existing rows needs a lower rate.
  Negative priors: averaging two experts near a band edge (−24 at time), a
  split accumulator, weights quantised after training, the root's expert fixed
  for the search (−27.7). Unmeasured proposals: overlapping material bands,
  shared and private first-layer lanes per expert. Each step as CPU gates,
  proxy trainings each judged by an SPRT, then one full run.
- **F.8 Data frontier: on-policy refresh, deduplication, hard-position mining — `V`.** On-policy refresh with the strongest net,
  deduplication, hard-position mining; repeat while a cycle accepts.
  Triumviratus inputs (2026-11-03; TD-5, TD-6, TD-14). A refresh adds new
  information, not more of the old: more epochs on data of the same family
  moved a converged net by 1% in scale and 0 Elo, per-expert fine-tunes +0.7,
  and a stream filtered to passed-pawn positions added nothing to a converged
  block. Its one refine with its own self-play (6.5% of the mix, listed ten
  times) rode with new data and a new block, +18 together, so its own share is
  unknown. Hard examples (positions lost, or where a deep search contradicts
  the static score) need a dedicated generation at scale. Each refresh is
  gated against the previous net on the fixed anchor.
- **F.9 NNUE release, 3.0.0: beat the classical release at STC, LTC and 4T; platform matrix — `M`/`V`.** Beat the classical release at STC, LTC
  and 4T; platform matrix; publish.
- **F.10 CCRL top-100 gate — `V`.** Submit; the list decides. Shortfall
  measured against the pool and fed back into F.7/F.8.

## Phase G — Scaling, platforms and the top 50

- **G.1 High-thread and NUMA: 8/16/32T, TT and net placement, large pages, affinity policy — `R2`, then `I2`.** 8/16/32T scaling, TT and
  net placement, large pages, thread affinity policy.
  Triumviratus inputs (TB-13, TP-6, and TP-4's SMP levers, since D.2 closed
  with RAR-M83 NOT DEFICIENT; `analysis/triumviratus_review_2026-11-03.md`
  §6, §8): search state on
  large pages beyond the TT, NUMA placement with one network and one history
  set per node, and its SMP levers.
  From D.2 (RAR-M82, 2026-11-03): whether the engine should place its own
  threads waits on how CCRL's testers place a multi-thread engine. In a slot
  of one logical CPU per physical core Rarog's four threads keep 4.03 times
  one; hard affinity inside an engine fights the tester's mask, and no donor
  sets it.
- **G.2 Platform and product: Chess960 on demand and distributed testing — `I1`.** Chess960 on demand, and distributed
  testing when typical gains reach 1–3 Elo. The universal x86-64 binary
  was G.2's until 2026-11-04, when the maintainer moved it to E.3.3
  (`analysis/universal_binary_2026-09.md`). Basilisk's statically linked
  Linux asset (H4) is E.3.3's too since 2026-11-04 (RAR-P57).
  Triumviratus inputs (TB-9, TB-14, TA-1, TA-3;
  `analysis/triumviratus_review_2026-11-03.md` §6, §13): its Chess960 contract
  and FRC perft suite, per-variant profiles merged into one universal build,
  an analysis build compiled as a second search copy, and strength levels
  calibrated in games.
- **G.3 Frontier: larger nets, data scaling, LTC search fit; CCRL top-50 gate — `R3`.** Larger nets, data scaling, search fit at LTC; the
  top-50 gate is the CCRL list again.
  Triumviratus inputs (TS-2, TS-19, TM-5;
  `analysis/triumviratus_review_2026-11-03.md` §5, §9): Stockfish's tree at
  depth 12 and more reduces nearly twice as much as Triumviratus's, and no
  single transplant shortened it; depth-dependent levers need a long-control
  fit, since short-control tunes never see depth beyond 15; its low-depth
  hash-move extension, which Rarog shares, lost at 40+0.4.

## Phase H — Refinements after the big gains

**Rule** (maintainer, 2026-11-03, stated for Basilisk and applied here;
`analysis/triumviratus_review_2026-11-03.md` §2): "I do not wish to chase 5
Elo if we are 400 Elo behind." A mechanism measured elsewhere at about +5 Elo
or less waits here until the network has replaced the classical evaluation
(F.9) and the larger gains (F.7, F.8, G.3) are exhausted. An item that belongs
to a defect a leaf already repairs, or that is a form a leaf must choose
anyway, stays with that leaf. Instruments, correctness checks and
measurement-integrity reads are never small gains: they go to the leaf that
consumes them. Every item here is a prior, never acceptance: it enters at
`RESEARCH`, crosses `READY_FOR_IMPLEMENTATION` only on a local screen, and is
gated as AGENTS' *Gating* requires, clustered where it shares a signal (rule
3). The step numbers H.1–H.4 are unrelated to the Basilisk cross-check's item
labels H1–H45.

- **H.1 Triage: every deferred small gain re-ranked on the network head — `R2`.**
  Opens when F.10 and G.3 have closed, or earlier by maintainer decision once
  two consecutive programme steps have each read under +5 Elo (the trigger
  Basilisk set). Re-ranks everything below against the then-current head and
  its profile: drops what the network made moot (TS-12 without input experts,
  TT-10's expert rules), groups what shares a signal into clusters, and orders
  by local activation, not by the donor's Elo; the order is registered before
  any game. With it, the ablation of features accepted on weak evidence, each
  by a simplification bracket (Triumviratus freed two that way). Sources:
  `analysis/triumviratus_review_2026-11-03.md` §14, and the engine review's
  and the Basilisk cross-check's low-priority items in H.2; later reviews
  add their own.
- **H.2 Search refinements — `R2` cards, then `I2`/`V`.**
  Each item alone against the head on one binary, its SPRT run to a decision
  with no early stop on a lean (Triumviratus's stopped leans added up to
  nothing, TM-4); levers that tune together enter one SPSA instead.
  Triumviratus's single search gains, each measured on a Stockfish-19-shaped
  search with material experts: causal reduction (TS-3, +4.27 ± 4.13); the TT
  cutoff value damped toward beta (TS-4, +2.2); HistGrad (TS-5, +0.9); a
  discovered-check ordering bonus (TS-6, +0.8); the null-move non-pawn floor
  as a fit of `NmpMinNonPawnPieces` (TS-9, +1.3); passer pushes reduced less
  and never pruned with little material (TS-10, +4.4 ± 6.8); a continuation
  correction six plies back (TS-11, −0.8 alone; RAR-X04 is the local caution);
  corrections and margins per expert if F.7 adopts input experts (TS-12); a
  minor each with no king in a corner as a search draw (the review's check 8,
  +1.1). Chess-knowledge ordering terms only as SPSA coordinates beside the
  histories they compete with (TS-7: flat at hand values there, and removing
  them gained +4.21 on its next net). TS-8's continuous improvement in the
  move count is Rarog's form already.
  From the engine review and the Basilisk cross-check, moved here on
  2026-11-05 under this phase's rule (each was D.6's, and its own record
  ranked it low; HISTORY's number map):
  *The engine review's low-value notes* (D.6.14): PV-node `force` and the
  mid-loop `Lower` store (NODE-7, TT-10: a counter of refused PV stores
  first; `force` is case 2); the parent reward and history updates inside
  exclusion searches (NODE-10: shared with the donor; a counter, then case 3
  only above 5% of rewards); quiet history captured at list generation rather
  than per move (NODE-12: nothing unless D.6.6 lands); the non-pawn
  correction key without kings and a fifty-move bucket in the TT key
  (BOARD-9: R2 only). For BOARD-9 (2026-11-03, TS-18b in
  `analysis/triumviratus_review_2026-11-03.md`): on an evaluation that
  already fades with the clock, as Rarog's does (`eval_rule50_damping`),
  Stockfish 19's fifty-move bands in the TT key read −3.7 in Triumviratus,
  and fading the correction with the counter −2.7.
  Basilisk's paired reads (BAS-S59, BAS-S60, 2026-11-13): kings in the
  non-pawn key +11.6 ± 9.3 alone, undecided together with razoring
  (+2.2 ± 4.2 over 10,000 games); the fifty-move key −4.5 ± 8.4; neither
  adopted.
  *Stockfish 19 mechanisms Basilisk adopted that Rarog lacks* (D.6.18; §C
  of the cross-check): upcoming-repetition detection (cuckoo; Basilisk found
  it must act as a cutoff at the draw window, not a draw score); seekMate
  (with a root score of 2000 or more from depth 16, RFP is capped at depth 6
  and singular extensions are off); quiet pruning and IIR gated on the
  previous PV; the TT bound-mismatch penalty. Basilisk's donor is Stockfish
  19 and Rarog's is Reckless, so its result is a prior only. Research cards,
  low priority; none crosses `READY_FOR_IMPLEMENTATION` without its mechanism
  shown active here. Basilisk also measured its check extension at −22 Elo at
  `3+0.03` and −5.4 ± 8.6 at `10+0.1`; Rarog has none. Basilisk's game reads
  since (H22): the upcoming-repetition test off 0.0 ± 9.3, so its priority
  falls; quiet pruning at PV nodes off the previous PV read +5.9 ± 9.9 with
  it off, and Rarog prunes quiets at every PV node as Reckless does, so the
  previous-PV gate would be a restriction; seekMate's depth is steep (depth
  16 grew bench 10.7×, depth 8 3.8M), its Elo unread. Also absent here and in
  PLAN: Stockfish 19's shuffling gate on singular extensions and non-zero
  history fills (Basilisk's fills read −0.5, null). Triumviratus input,
  2026-11-03 (`analysis/triumviratus_review_2026-11-03.md` §4 item 2, TS-16,
  TS-17). The upcoming-repetition detection takes Stockfish 19's filter: a
  cycle counts only when the opponent's key changes inside the window cancel.
  Adding the filter to a port without it read +5.62 ± 4.47 over 6,500.
  Negative or null priors: linear razoring at all-nodes −2.32 ± 2.52; a null
  threshold after sibling null fail-highs +0.46; clean singular-verification
  nodes, a return to the root counted as a draw, the move count restored
  after the singular search (Reckless-derived) and no null move under a hash
  upper bound, each about ±0.5.
  *Root and in-check reductions never read apart* (D.6.19; H23): LMR is
  gated `!NODE::ROOT && !in_check` (`node.rs:1759`, pinned by
  `lmr_never_reduces_at_the_root_or_in_check`); RAR-S74(e) read the two cases
  bundled in one switch at −3.8 ± 9.6; Reckless reduces in both; Basilisk's
  in-check selectivity read about +8 from unpaired runs (BAS-S41 items 1–2).
  `R2`, low: two counters, then one switch each.
- **H.3 Time-management refinements — `R2`, then `I2`/`V`.**
  First the loss table by cell (TT-1) on the network head's own games:
  `tools/diag/loss_table.py` (D.4.5) with the design of
  `analysis/loss_table_card_2026-11-05.md`, registered afresh with its own
  predictions and its smoke timed at the run's worker count. It ranks the
  levers that follow: the surprise rule
  (TT-7, +6.2 ± 6.8 at 40+0.4); less time when behind on the clock (TT-8);
  time in defence and early saving (TT-9, +7.3 ± 7.1 at 40+0.4 together); a
  moves-left horizon fitted on Rarog's games (TT-6); the smaller rules
  (TT-10). Each lever is read at the control it acts at, against an external
  engine (TT-3); TT-5 holds the negatives. D.1's record and retry triggers
  stay the base.
- **H.4 Throughput at an identical tree on the network head — `R2`, then `I1`/`V`.**
  C.12's speed pass is closed and F.5 covers the network's inference. On the
  network head's profile, Triumviratus's board, SEE, picker, prefetch and
  build-layout leads (TB-4 to TB-8, TB-12, TB-14's layout flags; TB-16's
  negatives; its stack probes, TB-11, are D.8.19's) are read where the
  profile shows their cost, each with exact fingerprint and pooled-PGO NPS.
  D.4.6 left one more lead: a process falls at random into one of two states
  on the same tree, 0.40% apart in NPS, cause unknown
  (`analysis/identical_tree_counters_card_2026-11-05.md`).
  Each shortcut ships with a build switch that runs the original beside it at
  every node and aborts on the first disagreement (TM-10, as Triumviratus
  checked every identical-tree patch).
  The Basilisk check (2026-11-13) adds one lead: Rarog builds every
  quiet-stage threat map eagerly, slider loops included
  (`movepick.rs:500-516`), where Basilisk builds them lazily (part of
  BAS-P29's +4%). First count quiet stages that build a slider map no
  quiet move uses.

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
- 2.7.0, an unscheduled release cut before D.7 (maintainer decision
  2026-11-13), was read against 2.6.0 (RAR-M87): G −7.6 Elo [−14.4, −0.7]
  against a five-engine field, a loss by its rule, and +10.1 ± 8.7 head to
  head. The maintainer shipped it for its fixes after seeing the result,
  an exception for 2.7.0 and not a rule.
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
