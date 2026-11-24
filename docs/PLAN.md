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

**The pre-NNUE goal (maintainer decision 2026-11-18).** Before the NNUE
stage, the target is **Stockfish 16 with `Use NNUE=false`**, its last
hand-crafted evaluation: get as close to it as possible, and beat it if it
can be done. The meter is Rarog's gap to it, read head to head and in a
rated field. On 2026-11-18 Rarog 2.7.0 is **160 below** it in a 13-engine
field and **−164.8** [−187.1, −143.6] head to head at `3+0.03` 1T (RAR-M89).
RAR-M91 (2026-11-22) reads the merged kernel 100.2 below it in a twelve-engine
round robin, **−103.3** [−121.0, −86.0] head to head.
On 2026-11-21 the adopted kernel is **96.6 below** it in the same field,
pinned at RAR-M89's ratings, and **−100.6** [−119.4, −82.3] head to head
(RAR-M90).
The maintainer may change the target later. It sets the direction; E.2's
gate below is unchanged until the maintainer replaces it.

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
| Fingerprint | `bench 13` **3,694,298 / EBF 2.324** at engine source `64993ff`, on `master` as `b862256` since PR #21 (2026-11-23) (D.6.29: quiescence scores a stalemate in Stockfish's narrow form, kept by RAR-S139 on the maintainer's decision); before it 3,374,901 / EBF 2.316 at engine source `91bc561` since 2026-11-22 (D.6.7: the ply cap at the donor's 240, its score bands with it, kept by RAR-S138); before it 3,390,842 / EBF 2.317 at engine source `16f93cb`, on `master` as `94d7f72` since PR #20 (2026-11-22) (D.7.11.8: RAR-S135's centres at 2,000 baked into the kernel's seeds; before it 3,320,355 / EBF 2.317 at `4271031`, D.7.8.3: the root ordered by the kernel's own scorer, kept by RAR-S132; before it 3,348,508 / EBF 2.329 at `e363908`, D.7.8.2: quiescence and evaluation-only entries at the donor's depth, kept by RAR-S131; before it 3,278,226 / EBF 2.313 at `e63b652`, D.7.8.1: ProbCut's exchange bar at the donor's scale, RAR-S130; before it 3,488,485 / EBF 2.336 at `d23b9cf`, D.7.7: the classic kernel deleted, the compact kernel's fingerprint since D.7.6.2, `73966eb`, unchanged; before it the classic kernel's 9,067,595 / EBF 2.479 at `9d7b42f`, D.6.31: a count is a move made, the null move not counted, as Stockfish and Reckless count, RAR-S125; before it 8,935,848 / EBF 2.480 at `0814689`, D.6.30: a proven win kept against a later iteration, repair case 1, `analysis/d630_mate_screen_2026-11-13.md`; before it 9,077,016 / EBF 2.481 at `ba1d2cd`, D.6.8: each position counted once, RAR-S124; before it 11,477,040 / EBF 2.537 at `aee0b3b`, on `master` as `89b4a92`, PR #17, D.6.7: every line bounded at twice the iteration's depth, RAR-S122; before it 13,038,236 / EBF 2.540 at `ee0b3e4`, D.6.2: quiescence and ProbCut generate the quiet queen promotion, RAR-S108; before that 11,946,003 / EBF 2.547 at `72c4e60`, D.6.1: castling classed quiet, RAR-S107; before that 15,625,852 / EBF 2.563 at `1450340`, D.5.5: `Hash` at any size moved every slot, RAR-S106; before that 14,749,043 / EBF 2.564 at `3a16fbb`, D.6.15: a draw verdict takes no correction; before that 13,987,223 / EBF 2.565 at `9fb525e`, C.10's search tune on the frozen evaluation; `rustc 1.100.0` since 2026-11-13, RAR-P58) | RAR-S105, RAR-P58 |

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
| D.7 search shape | 22.5 ± 7.9 at equal clock against Phase B's donor as it is (RAR-O08, match 5); tree shape at equal work +3.5 ± 8.4 (RAR-O09), so most of it is not shape; the speed and clock shares are unread | 0–70 | conditional on D.7.2's reads or the maintainer's decision: the kernel in the donor's whole form, seeded from the hybrid's configuration and fitted. Equal strength is accepted (maintainer decision 2026-11-13), so the band starts at 0. **2026-11-19:** built and fitted once, the kernel reads +56.8 ± 9.5 over its donor's seeds (RAR-S128) and about +50 over the classic by a sum of two reads; repairs, clock use and a finishing fit follow, and D.7.13 is its one gate. **2026-11-21:** the gate accepts (RAR-S136, H1 at 520 pairs), with +67.0 ± 8.1 at `10+0.1` and +73.3 ± 8.1 at four threads against the classic; adopted the same day (D.7.14), the kernel reads 3,508.1, 96.6 below Stockfish 16 HCE and +63.5 over 2.7.0 in RAR-M89's field (RAR-M90) |
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
2. **Constants are seeds.** Only a GPL-compatible donor's constants are
   ported; an AGPL engine's never are (PROCESS, 2026-11-23). A ported constant
   sits on the donor's score scale and node population. It is converted through the measured scale ratio (B.0),
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
Then D.7, and only after it the open D.6 leaves, D.6.28 and D.6.29
among them, each asked first whether it is still needed (maintainer
decision 2026-11-13, after the Basilisk check and D.6.30).
The Basilisk check for new ports, research only, follows the cut and
precedes D.7 (maintainer decision 2026-11-13); done 2026-11-13
(`analysis/basilisk_check_2026-11-13.md`: D.6.29, D.6.30, evidence for
D.6.28, and inputs to D.7.0).
On 2026-11-19 (maintainer decisions, after the audit of D.7,
`analysis/d7_audit_2026-11-19.md`) the open steps of D.7 were renumbered
to run in ascending order (D.7.7 to D.7.15; HISTORY's number map), D.6.28
moved into them as D.7.9, and every step of D.6, closed or open, was
reopened for a review on the new kernel after D.7, in the next pull
request. D.7 closed 2026-11-21; the rows below are D.6's in number order.
On 2026-11-22 (maintainer decision) E.0, the research on where the
remaining strength is, moved ahead of D.8: it runs right after the last
D.6 review row, keeps its number, and its ranking may reorder D.8 to
D.15 and E.4.
On 2026-11-23 (maintainer decision on E.0's packet,
`analysis/e0_remaining_strength_2026-11-23.md`) the order of work leaves
the numbering again and every step keeps its number, a Phase E number
among Phase D's rows included. D.16 runs first, its profile (D.16.3.1) ahead of its
instrument steps; then E.4.0, opened by the donor's evaluation inside
the kernel (E.4.0.1). Then D.8 to D.17 in ascending
order, with D.12 cut to D.12.0 and its other leaves held in the last
rows; then Phase E as numbered. E.4's units stay after the release
(maintainer decision 2026-10-22).

| Leaf | Workflow state | Class | Current decision |
|---|---|---|---|
| E.4.0.1 | LOCAL_QUALIFIED | I2 | Added 2026-11-23 (maintainer decision on E.0's packet), E.4.0's first read; the bridge built and qualified 2026-11-24 (`tools/donor-eval`); the two equal-node reads done 2026-11-24 (RAR-O13): `sf_16` +80.3 ± 10.8, `sf_11` +74.4 ± 10.6, above the +25 rule, so a rebuild is not ruled out; the clock read with `sf_16` waits for its bridge price on an idle host |
| E.4.0 | RESEARCH | R3 | Moved ahead of D.8 2026-11-23 (maintainer decision on E.0's packet), research only: the donor and the form, decided on E.4.0.1's reads; E.4's units stay after the release |
| D.8.1 | READY_FOR_IMPLEMENTATION | I1 | After D.7 and the review of D.6: the census taken on the tree without the classic kernel, every later item read against it |
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
| D.8.21 | RESEARCH | V | Added 2026-11-20 from D.7.11, open since D.7.14's adoption (2026-11-21); a count on replayed gate games of how often each of eight seeds' terms binds, a seed deleted only if its term never does |
| D.9 | RESEARCH | R2 | Tablebase policy, with the review's probe-path items (PROTO-9 to 15, DRIVER-15, NODE-11, TT-11) and the cached empty `SyzygyPath` |
| D.10 | READY_FOR_IMPLEMENTATION | I1 | Linux transparent huge pages for the page allocations, behind `LargePages` (macOS stays Apple silicon only, by maintainer decision 2026-11-02); stands on its own: RAR-M79 (2026-11-02) read no large-page cost in a healthy host state, and their part in the slow state is unread (audit 2026-11-02); exact fingerprint, a Linux NPS read |
| D.11.1 | RESEARCH | R2 | The originality disclosure, moved from E.3.0, widened 2026-11-23 from Coda's read; its hold, lifted for 2.7.0 (maintainer decision 2026-11-13), stands for the release after it: the donor inventory by licence at the version read, over the source, tools and documents |
| D.11.2 | RESEARCH | R2 | After D.11.1: GPL-3.0 kept by maintainer decision 2026-11-23, Stockfish the primary donor and Reckless ideas only; the card's other questions (README's development line, public study material, the git history) open |
| D.11.3 | RESEARCH | I1 | After D.11.2: the decision carried out; registered experiments under the GPL route |
| D.11.4 | RESEARCH | M | After D.11.3: the disclosure on Coda's model; its notices ship in D.15.1's archives |
| D.12.0 | RESEARCH | R3 | Added 2026-11-14 (maintainer: Basilisk's findings are hints, not Rarog's best answer); not started: Rarog's own board interactions mapped and its options ranked before any of D.12's leaves runs |
| D.13 | RESEARCH | R2 | Moved from D.7.8.7 2026-11-20: opens after D.7.14; built only if one thread reproduces the fingerprint, then the NPS and four-thread reads |
| D.14 | RESEARCH | R2 | Added 2026-11-20 (maintainer request): the donor's soft-stop vote on the new clock, after the fit and D.7.14, beside D.13; a four-thread withdrawal screen at zero games first, 2% floor |
| D.15.1 | READY_FOR_IMPLEMENTATION | I1 | Added 2026-11-21 from Basilisk 1.11.0: each download a `.zip` or `.tar.gz` with the binary, the licences, the README and the changelog, where D.11's notices ship; before the next release |
| D.15.2 | READY_FOR_IMPLEMENTATION | I1 | Added 2026-11-21 from Basilisk 1.11.0: the release notes as the section's summary and highlights, with a link to the full changelog; before the next release |
| D.17 | RESEARCH | R2 | Added 2026-11-23 (maintainer request): not before the end of Phase D; a card on GitHub-hosted runners as a lane for self-play SPRTs and tunes, then a maintainer decision on the rules it changes |
| D.12.1 | RESEARCH | R2 | Held 2026-11-23 (maintainer decision on E.0's packet): the board's work a node is level with Stockfish 16's; D.16.3.1's profile showed no board function above the donor's (2026-11-23); reopens when D.12.0 shows one |
| D.12.1.1 | RESEARCH | I2 | Held 2026-11-23 with D.12.1 |
| D.12.1.2 | RESEARCH | V | Held 2026-11-23 with D.12.1 |
| D.12.2 | RESEARCH | R2 | Held 2026-11-23 with D.12.1 |
| D.12.2.1 | RESEARCH | R2 | Held 2026-11-23 with D.12.1 |
| D.12.2.2 | RESEARCH | I1 | Held 2026-11-23 with D.12.1 |
| D.12.3 | RESEARCH | I2 | Held 2026-11-23 with D.12.1 |
| D.12.4 | RESEARCH | V | Held 2026-11-23 with D.12.1 |

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
- **D.6 Strength candidates from the engine review: kernel and evaluation deviations from the donor, each behind a zero-game screen and a registered gate — `R2` cards, then `I1`/`I2`/`V`, DONE 2026-11-23.**
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
  here, not in the archive: the review after D.7 comes back to
  them (maintainer decision 2026-11-13).
  **Order, 2026-11-13 (maintainer decision):** D.6.8 runs first, then D.7.
  The other open items wait for D.7 and none is dropped. After D.7 each
  is asked first whether it is still an issue on the kernel D.7 leaves
  behind: an item the rebuild made moot closes with that reason, one whose
  defect remains keeps its card, and a form that lost its gate on the
  combined kernel (D.6.3, D.6.4, the fail-soft cluster of RAR-R19 to R21,
  one-retry aspiration) is re-opened or added where that review finds its
  problem still there.
  **Order, 2026-11-19 (maintainer decision, with D.7's rework).** After
  D.7, in the next pull request, every step of D.6 is reviewed on the new
  kernel in number order, the closed ones too: does the problem exist
  there? Zero games first (a test, a fixture, a replay). A problem the
  kernel does not have closes the step with that reason. One it has is
  discussed with the maintainer before any change, and only then gets
  its card. The closed steps are reopened for that review; their records
  below stay as written and describe the classic kernel, which D.7.7
  deleted. D.6.28, clock use, moved into D.7 as D.7.9, with its text.
    - **D.6.1 Castling is neither quiet nor noisy — `R2`, NO_CHANGE 2026-11-22 on the new kernel (reopened 2026-11-19; done 2026-11-10 on the classic kernel).** (NODE-3, BOARD-1):
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
     Review read 2026-11-22 on the compact kernel (zero games): the
     problem is not there. The kernel reads one partition, `is_quiet_move`
     (`src/search/compact/mod.rs`): any move but a capture or a queen
     promotion, so castling is quiet, as in Reckless's `is_quiet`
     (its `types/moves.rs`). Every reader takes it: the move loop's
     pruning, exchange threshold, reductions and history read, the tried
     moves' bonuses and maluses, the parent and fail-low rewards, the
     continuation key and the picker's table-move tier. Castling comes
     from the quiet generator and never enters a noisy table. The exchange
     test passes castling only at a threshold at or below zero, where
     Reckless's passes it always; no caller asks above zero, since every
     quiet threshold is capped at 0, the good-noisy bar reads noisy moves
     only and quiescence never generates a castle. Check: the kernel's
     seven picker tests (`compact::picker`, one asserting `e1g1` quiet)
     pass, exit 0. The board's own `Move::is_quiet`, which counts
     under-promotions noisy, is read only by tests; its comment claimed
     otherwise and was corrected (`f22bf1d`, comment only). Folding the
     two partitions into one is a cleanup for D.8's census (D.8.1).
    - **D.6.2 Quiescence cannot see a non-capturing queen promotion — `R2`, NO_CHANGE 2026-11-22 on the new kernel (reopened 2026-11-19; done 2026-11-10 on the classic kernel).** (BOARD-2): pawn
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
     Review read 2026-11-22 on the compact kernel (zero games): the
     problem is not there. The kernel's noisy stage
     (`generate_legal_noisy_staged_into`) emits the legal captures, then
     every legal non-capturing queen promotion through checks and pins, with
     no capture pre-scan to skip it; the quiet stage leaves queen pushes out,
     so none is handed over twice, and the rook, bishop and knight pushes
     stay quiet. Quiescence takes the picker with quiets skipped, so the push
     is generated and searched like a capture, its exchange test putting the
     promoted queen at risk; ProbCut takes the same picker and stops at the
     bad noisy moves, so a push that clears its bar is tried there too.
     Reckless generates promotions the same way (its queen push with the
     noisy moves, before the captures where Rarog's follows them; the picker
     orders by score, so only an exact tie could differ). Checks: the
     noisy-list test now runs through the staged generator and passes,
     against the legal moves on five positions walked two plies; a new
     quiescence test (`7fb73e4`) has a free push raise the score above the
     stand pat and head the line, and a rook-guarded one do neither, and it
     fails when the generator's queen pushes are switched off.
    - **D.6.3 Null-move and ProbCut children typed `cut_node = true` — `R2`, NO_CHANGE 2026-11-22 on the new kernel (reopened 2026-11-19; no change 2026-11-11 on the classic kernel).** (NODE-1): the
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
     Review read 2026-11-22 on the compact kernel (zero games): the
     problem is not there. Each of the kernel's nine recursive calls types
     its child as Reckless's matching call does: the null-move child, its
     verification and both ProbCut verification searches `false`, the PV
     search `false`, a reduced late move `true`, its re-search and the
     full-depth zero window the opposite of the parent, the exclusion
     search the parent's own type. RAR-S114's concern, a ProbCut margin
     fitted under the other typing, is gone too: D.7.11 fitted the
     kernel's ProbCut and cut-node terms with this typing in place, so the
     retry at the search re-fit has in effect run. NODE-14's test left
     with the classic kernel; its place is taken by
     `each_recursive_call_types_its_child_by_its_rule` (`9507abd`): test
     builds tag each call and count the type its child receives, the test
     holds the rule table and requires every call reached, and it fails on
     the null child typed `true` (8,125 children against the rule) and on a
     ProbCut child typed `true` (7).
    - **D.6.4 A fail-low node stores its least-bad move as the TT move — `R2`, NO_CHANGE 2026-11-22 on the new kernel (reopened 2026-11-19; no change 2026-11-11 on the classic kernel).** (NODE-2):
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
     Review read 2026-11-22 on the compact kernel (zero games): the
     problem is not there. The kernel's best move changes only when a
     score raises alpha, as Reckless's does, so a fail-low node stores no
     move, and the table keeps the entry's older move when a store for the
     same position brings none (`src/tt.rs`, pinned there by "a store
     without a move keeps the move"), as Reckless's `write` does. Neither
     kernel has IIR, the consumer that carried RAR-S115's loss; a missing
     TT move is read by the cut-node reduction terms and the singular
     search's need for a TT move, both the donor's and fitted with this
     store in place (D.7.11). The classic's own form, the best-scoring
     move kept, is not retried: its ordering evidence was 1.022 of the
     nodes (0.990 to 1.043) on another kernel, and the kernel's fit and
     gate were made with the donor's store. The classic's search-side test
     left with that kernel; no kernel test pins the fail-low store
     (maintainer decision 2026-11-22: closed without one).
    - **D.6.5 The exclusion search uses the table-refined estimate — `R2`, NO_CHANGE 2026-11-22 on the new kernel (reopened 2026-11-19; no change 2026-11-11 on the classic kernel).** (NODE-4):
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
     Review read 2026-11-22 on the compact kernel (zero games): the
     problem is not there. The kernel's `estimated_score` takes the stored
     score only out of check, outside an exclusion node, with a valid
     score whose bound tightens the evaluation, which is Reckless's rule;
     the node passes its exclusion flag, so an exclusion node prunes on the
     plain evaluation, the direction RAR-S119 found right. The re-entry
     sub-item matches the donor as well: an exclusion node reuses its
     ply's stored evaluation, and the singular search restores only
     `tt_pv`. The guard is pinned by
     `a_stored_score_replaces_the_evaluation_only_where_its_bound_tightens_it`,
     and the parity test replays the rule against the donor's decisions.
    - **D.6.6 Captures generated and scored before the TT move — `R2`, NO_CHANGE 2026-11-22 on the new kernel (reopened 2026-11-19; no change 2026-11-11 on the classic kernel).** (NODE-5):
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
     Review read 2026-11-22 on the compact kernel (zero games): the
     problem is not there. The staged picker's first stage hands over the
     TT move, and noisy moves are generated only in the next stage, so a
     TT move that cuts never pays for capture generation; Reckless's picker
     starts at its hash-move stage the same way. The stored move is checked
     by `legal_move`, which tests it directly and generates nothing.
     Quiescence and ProbCut build their pickers without a TT move, as the
     donor's do. The picker test that drains every stage asserts the TT
     move comes first; no further test pins the order, which changes cost
     alone, and RAR-S120 priced that cost below any resolvable read.
    - **D.6.7 The ply cap — `R2`, DONE 2026-11-22 on the new kernel (reopened 2026-11-19; done 2026-11-11 on the classic kernel).** (NODE-8): `MAX_PLY = 128` is below the extension budget's
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
     Review read 2026-11-22 on the compact kernel (RAR-S137, a replay of 60
     of RAR-M91's games, zero new games): the kernel ends a line at ply 127
     as the donor does, a draw by rule first and a draw in check, so the
     classic's static evaluation there is gone, and RAR-S108's race
     position no longer reaches the cap by depth 10 (seldepth 70). But the
     kernel's line bound is eight times the iteration's depth, so from
     iteration 16 the cap binds before it. No search reaches the cap at
     the games' node counts or at ten-thirds of them; at twenty times them
     0.511% do (21 of 4,107, in 14 of 60 games), all in endings at
     iterations 16 to 99, above RAR-S122's 0.1% trigger. **Open, for the
     maintainer's decision on the form, before any change.**
     Repaired 2026-11-22 (maintainer decision: the donor's cap, read at zero
     games and then by case 2): `91bc561` raises `MAX_PLY` to 240 in the
     search and the table, the mate band, the tablebase band and the
     decisive floor with it, as the donor's do; a test ties the table's cap
     to the search's, and the score display's test reads its band edges from
     the constants (`788c216`, its own commit). `bench 13` moves to
     3,374,901 / EBF 2.316, through the bands, not the cap (one bench
     position in forty). RAR-S138's part A, the 21 cap searches replayed:
     no correctness fault, 12 moves differ, Stockfish with Syzygy reads 10
     of them equal and 1 each way. Part B, 2,000 games at `3+0.03`: nElo
     −11.6 ± 15.2 against the −13 harm rule, **kept**. That rules out harm
     beyond roughly −20, not a loss near the line, and `3+0.03` cannot see
     the cap. Retry trigger: a loss read at a rated time control traced to
     the decisive band's lower floor.
    - **D.6.8 A depth-0 entry is counted as two nodes — `R2`, NO_CHANGE 2026-11-22 on the new kernel (reopened 2026-11-19; done 2026-11-13 on the classic kernel).** (DRIVER-11): `check_stop` at
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
     Review read 2026-11-22 on the compact kernel (zero games): the
     problem is not there. The kernel counts a node in three places, each
     right after a move is made: the main move loop, ProbCut and
     quiescence; nothing else adds to the count. The null move (D.6.31),
     the hand-off to quiescence, the exclusion search, re-searches and the
     null-move verification make no move and count nothing. Reckless counts
     the same way, once in its `make_move`, which those three call, and not
     for its null move. Pinned by the bench-positions test and the mate
     test (the count equals a test-only tally of moves made) and by
     `a_depth_one_search_counts_each_root_move_once` (20 from the start).
    - **D.6.9 The endgame scale's target — `R2`, CLOSED 2026-11-22: moved into E.4.2.** (EVAL-5): `evaluate` adds tempo to `mg`,
     tapers, then `scale_endgame` multiplies the whole; both donor commits
     scale the endgame half only (`v = mg·p + eg·(128−p)·sf/64`), so the forms
     differ for the opposite-bishop rule with pieces on the board and the
     partial recognisers at small phase. The constants were fitted inside
     Rarog's form (RAR-E26). Screen, offline: refit the three OCB constants
     with the scale on the endgame half on RAR-E26's cohort and compare
     held-out loss; then case 2 at most (OCB-with-pieces positions only).
     Under the evaluation freeze this is a registered repair.
     Review read 2026-11-22 on the compact kernel (zero games): the
     deviation is still there, since the kernel's wrapper adds correction
     and material scaling after `evaluate` and leaves the scale where it
     was. Its reach in the kernel's own games is small. Over 69,555
     positions the kernel moved in, from 1,000 of RAR-M91's games:
     - opposite bishops with other pieces, scaled (three pawns or fewer):
       0.05%, the forms about 5% of the score apart;
     - pure opposite-bishop endings: 1.5%, about 4% apart;
     - pure, under three pawns: 0.3%, about 2% apart;
     - the partial recognisers at small phase: not measured.

     The middlegame case the card names almost never arises: with pieces
     on the board the scale bites only at three pawns or fewer. **Closed
     here, moved into E.4's refit, now E.4.2** (maintainer decision
     2026-11-22): it fits both forms and lets held-out loss and its gate choose, so
     the constants stay consistent with the form. The count:
     `analysis/artifacts/d69-review-2026-11-22/ocb_exposure.py` (SHA-256
     `34c97b8d…`, its output `b967dfc7…`), on RAR-M91's PGN, seed
     20261122.
    - **D.6.10 The pawnless default factor — `R2`, CLOSED 2026-11-22: moved into E.4.2.** (EVAL-6): `material.cpp:195–203` scales
      a side with no pawns and a small material edge to 0, 4 or 14 of 64;
      Rarog's pawnless branch covers at most one minor in total or one each,
      so K+B+N v K+B keeps two thirds of about a knight. Never a ranked family
      (the family list is SF11's endgame functions). Screen: the families'
      share of the game record; at 0.5% of games or more the Syzygy
      drawn-overclaim read of RAR-E36. Case 2.
     Review read 2026-11-22 on the compact kernel (zero games): the
     gap is still there, an evaluation rule the kernel leaves as it was.
     Stockfish 11 scales a side with no pawns whose non-pawn material
     leads by at most a bishop (to a draw under a rook's worth, else 4 or
     14 of 64); Rarog's branch only scales a board with no pawns at all, no
     rooks or queens, and one minor or a minor each. The card's screen fires:
     1,320 of the kernel's 8,800 games in RAR-M91 (15.0%) reach a position
     the donor's rule scales and Rarog's does not, 4.2% of all positions,
     and fifteen families pass 0.5% of games each, KRN-KR, KB-KP, KR-KB,
     KR-KN and KRB-KR at 1.1% to 1.3%
     (`analysis/artifacts/d610-review-2026-11-22/pawnless_exposure.py`,
     SHA-256 `66f18f80…`, its output `0f841fb0…`). The same mechanism is
     D.6.26's. **Closed here, moved into E.4.2** (maintainer decision
     2026-11-22), where the card's next read, the Syzygy drawn-overclaim
     read, runs over every endgame family together.
    - **D.6.13 The aspiration fail-low window — `R2`, NO_CHANGE 2026-11-22 on the new kernel.** (DRIVER-13): the code halves toward
      the new alpha; a counter of fail-lows whose re-search fails high
      (`asp_fail_high` straight after `asp_fail_low` at one depth); above 15%
      the old-alpha form is a `[0,3]` candidate.
      Priors against neighbouring forms (TS-15 in
      `analysis/triumviratus_review_2026-11-03.md`): a decisive aspiration
      fail-high +0.7 ± 4.2, root depth recovery after a fail-high −1.82 ±
      4.54, a cap on root-move reductions −5.80 ± 5.93.
      Review read 2026-11-22 on the compact kernel (zero games): the
      problem is not there. The kernel's `window_fail_low` sets alpha to
      the failing score less the half-width and beta to that alpha plus
      the half-width, so beta falls to the failing score, not halfway, as
      Reckless's root loop does line for line; the fail-high side matches
      too. D.7.11 fitted both growth seeds, and the parity test replays
      both window rules against the donor's recorded decisions. The
      classic's halving form has no instance left, so the counter that was
      to choose between it and another form has no use.
  Added 2026-11-02 from the Basilisk cross-check (`analysis/basilisk_crosscheck_2026-11-02.md`):
    - **D.6.15 The correction is added to the evaluator's draw verdicts — `I1`, NO_CHANGE 2026-11-23 on the new kernel (reopened 2026-11-19; done 2026-11-03 on the classic kernel).**
      `3a16fbb`: the search no longer adds its correction to the
      evaluator's draw verdicts; the fingerprint moved from 13,987,223 to
      14,749,043, accepted by repair case 1 on a confined per-FEN
      differential, and no verdict leaf keeps a moved value. 7 of 28
      wrong-bishop draws still misread from cleared tables: D.6.23.
      Record: `docs/archive/PLAN-phaseD-done-2026-11-04.md`.
      Review read 2026-11-23 on the compact kernel (zero games): the
      problem is not there. `compact_eval` returns 0 with no correction and no
      optimism when the evaluator's raw score is a draw verdict;
      `a_draw_verdict_takes_no_correction` trains the tables on a wrong-bishop
      draw and requires exactly (0, 0) there, while an ordinary position takes
      its correction. Reckless has no verdicts to compare.
    - **D.6.16 Quiescence prunes by gain against a decisive material edge — `R2`, NO_CHANGE 2026-11-23 on the new kernel.** (A2;
      Basilisk `d8d30e9`): futility (`node.rs:2355-2362`) and the SEE
      threshold (`node.rs:2392-2396`) read a capture's gain as the piece
      taken, which is false when the capture removes the opponent's
      mating material (Kxh8 in KBB-K is a dead draw); delta pruning is off
      at eight men or fewer and is not involved. Rarog has no known-win
      band, so the guard's definition is the research question. Screen:
      Basilisk's fixture, and a `diag` count of skipped captures that
      leave the opponent without mating material on the conversion
      families. `R2` card first.
      Review read 2026-11-23 on the compact kernel (zero games): the
      problem is not there. The kernel's quiescence has no gain-based futility
      and no delta pruning, only the donor's exchange-threshold prune and its
      three-move stop. Basilisk's fixture (`d8d30e9`: four wrong-bishop draws,
      depths 8 to 14, 200,000 nodes, a fresh table) reads a draw in all 28
      searches; one, `7k/7P/6K1/8/8/3B4/8/8 w`, finishes depth 14 at 201,399
      nodes, just past the fixture's cap, and stays a draw to depth 16
      (`analysis/artifacts/d6-review-rest-2026-11-23/`).
    - **D.6.17 Underpromotions and Saavedra's study — `R2`, NO_CHANGE 2026-11-23 on the new kernel.** (an investigation): Basilisk
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
      that breaks it, so the review after D.7 asks first.
      Review read 2026-11-23 on the compact kernel (zero games): the
      wrong pick is not there. At 100,000 nodes the kernel plays `g7f8q` with a
      positive score in the transposing rook position: its picker orders the
      queen's capture-promotion ahead of the rook's.
      `queen_promotion_though_the_rook_was_searched_first` passes and runs again
      (`dfb593c`); Saavedra's `c7c8r` (a mate in 7) and the stalemate escape
      `f7f8r` (a mate in 2) are right and pinned by active tests. Basilisk's
      root rule is not needed.
    - **D.6.20 The fail-high stop's hole — `R2`, NO_CHANGE 2026-11-23 on the new kernel.** (H24): the stop at twice the target is
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
      the review after D.7 asks first.
      Review read 2026-11-23 on the compact kernel (zero games): the
      measured instance is gone. `won_ending_does_not_sink_the_clock` takes 2.17
      to 2.20 s in three runs on an idle host (limit 8 s; 9.46 to 9.82 s on the
      classic), keeps the win (`e3f4`, depth 20), and runs again (`dfb593c`). The
      hole itself, a window that never fails high running on to the hard limit,
      is the donor's shape: Reckless checks its soft limit only after an
      iteration and inside one only its hard bound, so Rarog's fail-high stop is
      an addition that closes part of it, and the hard maximum bounds the rest.
      The card's per-window read has no decision left to feed.
    - **D.6.21 Won KPK positions read at or below 0 — `R2`, CLOSED 2026-11-22: moved into E.4.2.** (D.5.19's read,
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
     **Closed here, moved into E.4.2** (maintainer decision 2026-11-22),
     before its review on the kernel; its screen runs there with the
     other endgame families'.
    - **D.6.22 `krpkr_scale`'s hard zeros on won positions — `R2`, CLOSED 2026-11-22: moved into E.4.2.** (D.5.19's read,
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
     **Closed here, moved into E.4.2** (maintainer decision 2026-11-22),
     before its review on the kernel; its screen runs there with the
     other endgame families'.
    - **D.6.23 Wrong-bishop draws still misread through leaves the recogniser does not cover — `R2`, CLOSED 2026-11-22: moved into E.4.2.**
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
     **Closed here, moved into E.4.2** (maintainer decision 2026-11-22),
     before its review on the kernel; its screen runs there with the
     other endgame families'.
    - **D.6.24 The KQKP fortress's hard zeros on won positions — `R2`, CLOSED 2026-11-22: moved into E.4.2.** (D.5.19's read,
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
     **Closed here, moved into E.4.2** (maintainer decision 2026-11-22),
     before its review on the kernel; its screen runs there with the
     other endgame families'.
    - **D.6.25 Two same-coloured bishops outscore a queen against a bare king — `R2`, CLOSED 2026-11-22: moved into E.4.2.** (D.5.21's
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
     **Closed here, moved into E.4.2** (maintainer decision 2026-11-22),
     before its review on the kernel; its screen runs there with the
     other endgame families'.
    - **D.6.26 Pawnless endings a minor ahead read as wins — `R2`, CLOSED 2026-11-22: moved into E.4.2.** (D.5.21's port of Basilisk's
      `pawnless_scale_truth_table`, 2026-11-08): Rarog has no scale for a
      pawnless side only a minor ahead, so Syzygy draws read KR-KB 277,
      KRB-KR 253 and KRN-KR 173, where the dead KB-KN draw reads 0 and three
      of the queen's classes read as wins. Screen: an enumeration of the
      classes against Syzygy, as D.5.19's, and their exposure in gate games;
      the repair form is a scale, as Stockfish 11's. Acceptance test:
      `pawnless_scale_truth_table` in `tests/behaviour_anchors.rs`, committed
      `#[ignore]`d (needs `RAROG_SYZYGY_PATH`). `R2` card.
     **Closed here, moved into E.4.2** (maintainer decision 2026-11-22),
     before its review on the kernel; its screen runs there with the
     other endgame families'.
    - **D.6.27 IIR has no node-type condition — `R2`, NO_CHANGE 2026-11-23 on the new kernel.** (found 2026-11-11,
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
      Review read 2026-11-23 on the compact kernel (zero games): the
      problem is not there. The kernel has no IIR, as Reckless has none; a
      missing TT move is read only by the cut-node reduction terms and by the
      singular search's need for one, both the donor's and fitted with the
      kernel (D.7.11).
    - **D.6.29 Quiescence never scores a stalemate — `R2`, DONE 2026-11-23 on the new kernel (RAR-S139, accepted by the maintainer's decision).** (found
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
      Review read 2026-11-23 on the compact kernel (zero games): **the
      problem is there.** The kernel's quiescence, like its donor's, never asks
      whether a side out of check with no capture has a legal move, so a
      stalemate reached through a capture still reads as its stand pat. The diag
      counters named above were the classic kernel's: the compact kernel's
      quiescence never fed them, and they were removed with the other dead
      counters on 2026-11-22 (`84f80de`), so the screen's re-read on the kernel
      needs them added to its quiescence first. **Open, for the maintainer's
      decision** on the form (Stockfish's narrow test, a departure from
      Reckless) and on whether to re-read the exposure first; the gate stays
      case 3, as above.
      Implemented 2026-11-23 (maintainer decision: Stockfish's form, gated)
      as `64993ff`: after quiescence's move loop, out of check with no
      move searched, a side with no free pawn push, no piece but pawns, a
      knight or more just taken and no legal move reads the draw. `bench 13`
      3,694,298 / EBF 2.324; fresh, only bench positions 15, 38, 39 and 40
      move. `a_stalemate_after_losing_the_last_piece_reads_as_a_draw` pins
      it both ways and fails with the rule off. Stockfish's own record: the
      test gains no Elo and passed as a no-regression change, against a
      tactical blind spot; Reckless, like most engines, has none.
      **Gate registered as RAR-S139**, before any game: its replay moved
      the score in 31.1% of searches, so SPRT `[-5,5]` at `3+0.03`, cap
      6,000 pairs, against `626d06b`; the binaries are built and the dry
      run passed; the maintainer's command.
      **Done 2026-11-23:** the maintainer stopped the gate at 6,046 games
      and accepted the candidate by decision (the SPRT unresolved, nElo
      +1.8 ± 8.8, LLR +0.90; harm beyond about −7 excluded, no gain
      shown). `64993ff` is kept; fingerprint 3,694,298 / EBF 2.324.
    - **D.6.30 A later iteration can overwrite a proven mate — `R2`, NO_CHANGE 2026-11-23 on the new kernel (reopened 2026-11-19; done 2026-11-13 on the classic kernel).**
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
      **Done 2026-11-13** (`0814689`): the repair as specified. `bench 13`
      moves to 8,935,848 / EBF 2.480, but every bench position in a fresh
      process is identical; the change enters at the one bench search the
      repair fires in (position 31, a proven win, under the table earlier
      positions left), so it is repair case 1. Activation read on the
      frozen replay: searches ending without a proven win 15 → 0, playing
      another move 3 → 0.
      Review read 2026-11-23 on the compact kernel (zero games): the
      repair is intact. The root driver keeps the last proving iteration's move,
      score and line after every completed iteration, whatever window or depth
      produced it, so the kernel's narrow windows and fail-high reduction reach
      it unchanged. A `diag` build's `bench 13` reads 13 proving iterations and
      none lost (`iter_mate_win`, `iter_mate_win_lost`): the counters are live,
      and bench does not exercise the keep. No test pins the rule; its
      activation read was the replay above.
    - **D.6.31 A null move adds a node to the count — `R2`, NO_CHANGE 2026-11-23 on the new kernel (reopened 2026-11-19; done 2026-11-15 on the classic kernel).**
      Added 2026-11-15 (maintainer: every engine compared here counts as
      Stockfish does). Stockfish counts in `do_move` and Reckless in its
      search `make_move`, once per move made, quiescence's and
      ProbCut's included, never for a null move; D.6.8's contract also
      counted the null move's position, about 1.7% of the count
      (RAR-O09), so every equal-node comparison with them needed a
      correction. The change: the null move's `count_node` goes, and
      every reader of the count (the LMR spread's phase, the effort
      share, the poll, the node limit) reads the new one, as in D.6.8. A
      test checks that a search making null moves reports exactly its
      moves made. The liveness probes of `CoreSingDoubleNotTtPv` and
      `CoreSingTripleNotTtPv` were re-pinned first, in their own commit
      (`947e5c5`), where both trees move. Implemented as `9d7b42f`,
      `bench 13` 9,067,595 / EBF 2.479. **Gate:** case 2, a 2,000-game
      read with the −13 harm rule, registered alone
      (`docs/experiments/RAR-S125.md`); the arms are built and the dry
      run passed; the maintainer's command.
      **Read 2026-11-15: kept**, nElo +5.9 ± 15.2 over 2,000 games
      against the −13 rule; nodes a move ×0.990 at the same depth and
      time. Rarog's head now counts as Stockfish, Reckless and Basilisk
      (`838a3c6`) do; fingerprint 9,067,595 / EBF 2.479.
      Review read 2026-11-23 on the compact kernel (zero games): the
      problem is not there. The kernel counts no null move; the bench-positions
      test and the mate test require the count to equal a test-only tally of
      moves made, in trees that make null moves (D.6.8's review).
    - **D.6.32 The shortest mate's depth moves with the seeds — `R2`, NO_CHANGE 2026-11-23 on the new kernel (added 2026-11-21 for the review after D.7).**
      (found by D.7.11.8's bake, RAR-S135, *The bake*; maintainer
      decision 2026-11-21 to investigate it here, after D.7, and not
      to hold the bake on it.) On the kernel's seeds before the bake,
      KQK's mate in three (`8/k7/8/8/3K4/8/8/1Q6 w - - 2 2`) is first
      reported at depth 8; on the baked seeds at depth 20, after a mate
      in five from depth 9 and in four from 16. KRK's mate in two
      (`k7/8/2K5/8/8/8/8/7R w - - 0 1`) moves from depth 5 to 6. A
      zero-game screen locates no family: any one of several unrelated
      families set alone moves KQK's first depth to 16 or more (the
      window, the pawn table's limit, the exchange values, the picker,
      the reward, RFP, LMP, the correction), and leaving out one of
      several others brings it back to 8 or 9
      (`analysis/artifacts/d711-fit-2026-11-20/mate_families.log`).
      RAR-S131 and RAR-S132 saw the same sensitivity: a table entry
      kept over a quiescence store can hold a longer mate, so how
      soon the shortest shows depends on the order the root searches
      its moves in. Any reported mate is its line's own length, so the
      cost in games is a later shortest mate, not a result. The mate
      tests no longer show it: at the bake their depths were measured
      again by their own rule (two plies past the first depth that
      finds the mate: KRK 8, KQK 22), and the warm-table test reads
      the table directly instead of comparing node counts.
      *Question:* why the shortest mate needs so much more depth than
      its length on the kernel, and whether a contract (a mate in n
      first reported by a fixed depth past 2n − 1) is worth a repair.
      *Screen, zero games:* the first depth of the shortest mate on the
      mate canaries and a KQK and KRK enumeration against Syzygy, on
      the kernel's seeds and with the table's mate entries not kept
      over quiescence stores; a repair is registered only if a
      mechanism is located. `R2` card.
      Review read 2026-11-23 on the compact kernel (zero games,
      `analysis/artifacts/d632-2026-11-23/mate_depths.py`, one search a
      position to depth 30, Hash 64, one thread): **the late shortest mate
      is the donor's, not Rarog's.** First depth of the shortest mate,
      Rarog (`64993ff`) / Reckless (`8d7d1dd`) / Stockfish 11 /
      Stockfish 18: KQK's mate in three 20 / 20 / 4 / 27; KQK's mate in
      four 9 / 16 / 16 / not by 30; KRK's mate in two 6 / 3 / 2 / 5; KRK's
      mate in five not by 30 (six reported) / 22 / not by 30 / 23. No
      engine holds a contract of a mate in n by a fixed depth past
      2n − 1; the reductions, pruning and mate entries of the shared shape
      make the shortest line late. The cost in games is nil: Rarog reaches
      depth 20 on KQK in 21,534 nodes, about 11 ms, within one move at
      `3+0.03`, and every reported mate is its line's. The arm with mate
      entries not kept over quiescence stores is not run: it needs a code
      change, and a repair it pointed to would depart from the donor for an
      effect with no game cost. **Closed `NO_CHANGE`** (maintainer decision
      2026-11-23).
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
- **D.7 Search shape: the kernel against its donor on Rarog's evaluation, rebuilt whole as one compact kernel if the reads or the maintainer say so — `R3` investigation, `I1`/`I2`/`V`/`R2` sub-steps, DONE 2026-11-21.**
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
  in about a hundred places (D.7.0) and was fitted around them, weaker than the
  donor's whole form on this evaluation? And do the problems the combined
  kernel has not solved (D.7.8's list) disappear in the donor's whole
  form, not in parts of it? The second is a reason for the step in its own
  right (maintainer, 2026-11-13).

  **Decision rule.** The data decides: D.7.2's reads open the rebuild at
  +10 Elo or more for the donor at equal nodes, the 95% interval excluding
  zero. Below that the agent reports the reads with a recommendation, and
  the maintainer decides after that consultation and may open the rebuild
  against the rule; the decision and its reason are recorded here.

  **Opened 2026-11-16 (maintainer decision, on RAR-O12 and D.7.2.3).**
  The rebuild goes ahead as one compact kernel in the donor's form, with
  three conditions that
  replace the 2026-11-13 acceptance (equal strength on the
  loss-permitting bracket):
  1. *Compact.* The kernel is the donor's search as it is: Rarog's
     additions on top of that shape (D.7.0's departures, its seventeen
     Rarog-only forms among them) are not carried into it. An addition
     is tried again only on the adopted kernel, one at a time, each with
     its own registration and gate. The aim is measured: the new kernel's
     search code against the donor's (`search.rs`, 1,488 lines at
     `0756957`), reported at D.7.9.
  2. *It must gain.* The fitted kernel is accepted only on a gain over
     the head (D.7.7, `[0,3]`), not on equal strength.
  3. *If it does not gain, Stockfish's form is next* (D.7.10), by the
     same method; the old kernel stays meanwhile.

  **Gated as a whole.** The kernel's parts are expected to pay only
  together, so no part is gated on Elo alone (rule 3: one
  dependency-complete cluster, its internal sub-steps no gates of their
  own). Each part is held instead to correctness: its invariants and
  tests, and parity of its decisions with the donor's (D.7.3, D.7.4). The
  whole kernel is then fitted as one unit (D.7.6) and gated once
  (D.7.7). The unfitted read (D.7.5) is a diagnostic: a loss there does
  not stop the step unless the diagnosis finds a defect, because an
  unfitted donor shape on Rarog's evaluation can lose and still gain once
  fitted (Phase B's core: +65.1 unfitted, then +138.6 from its fit,
  RAR-S73). The cost of this is that a part which harms the whole is
  found only by D.7.8's review or after adoption, not by its own gate.

  **Beside it.** Speed (D.12) and clock use (D.6.28) are levers the
  kernel does not touch (RAR-O12: speed and clock 51.6 ± 16.1 at the
  programmes' ends). D.12's behaviour-neutral board work can run beside
  D.7; a behaviour change such as D.6.28's does not land between D.7.6's
  fit and D.7.7's gate, which would move the baseline under the fit.
  Adoption is the maintainer's decision on D.7.7 and D.7.8.

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
  search-path perft; one count per move made, the null move not counted
  (D.6.31, the donor's own count); the acceptance tests of D.6.17 and
  D.6.20.

  **Stop rules.** Parity not reached: no games. D.7.7 at H0, or at its
  cap without H1: the old kernel stays and D.7.10 opens; this is one
  cluster, so its parts are not adopted on their own.

  **Independence.** A kernel nearer the donor's raises what D.11 must
  disclose and the weight of the licence question. PROCESS's boundary
  governs the implementation: architecture and mechanisms cross, constants
  cross as fitted seeds, the code is Rarog's own.
  *The threshold is not crossed* (maintainer, 2026-11-16: the compact
  kernel moves Rarog nearer Reckless's shape, never into copying it).
  - Each rule is written from the inventory's description and Rarog's
    own code: read the donor, close the file, design.
  - No donor code is transcribed except where an algorithm has one
    natural form, and each such place is named in its commit.
  - Rarog's structure stays: its node types, its rule functions as pure
    functions of input structs (D.8's target shape), its own names.
  - Constants enter as seeds converted by dimension and are refitted
    (D.7.4.1, D.7.6).
  - The parity instrument compares decisions on the same inputs, not
    code, and the donor's trace stays in ignored storage.
  - D.7.4's commits are checked against this list before D.7.5's games,
    and D.11's disclosure names the shape.
  **Decisions asked of the maintainer, and when** (2026-11-13). An agent
  stops and asks at each; none is taken by default.
  1. *Before RAR-O09's first game:* the runner (fastchess expected), and
     any preparation value outside the registration's stop bounds.
  2. *After RAR-O09 is read:* whether D.7.3 to D.7.9 open. At +10 or more
     at equal work the rule opens them; below it the agent reports with
     the recommendation the registration froze, and the maintainer may
     open the rebuild against it.
  3. *In D.7.4:* any donor rule that cannot be written without breaking
     an invariant, and any form the inventory marks kept or dropped that
     the implementer would treat otherwise.
  4. *When D.7.4.1 starts:* the exchange table, the donor's piece values
     kept or the SEE thresholds converted to Rarog's. *Answered
     2026-11-18:* the donor's values are kept.
  5. *After D.7.5:* a clearly negative unfitted run, before any diagnosis
     beyond the one PLAN allows.
  6. *Before D.7.6:* the tune's registration (surface, horizon,
     checkpoints), which the maintainer starts.
  7. *After D.7.7 and D.7.8:* adoption, and with it what D.11 must
     disclose before the release.

  Settled 2026-11-13: the parity check is a test of what was built, not a
  goal, and identical behaviour is not sought (the donor's forms, Rarog's
  own code); both parity layers are kept; the donor's trace stays in
  ignored storage; no unit translation stays in the engine.
  *Compactness relaxed (maintainer decision 2026-11-18):* the kernel stays
  as compact as it can, but more code is acceptable where a need is shown;
  the requirement was there to prevent another overengineered kernel and
  has served that purpose.
  **Reworked 2026-11-19 (maintainer decisions, on the audit of the kernel
  and of RAR-S128's tune, `analysis/d7_audit_2026-11-19.md`).** D.7.0 to
  D.7.6 are closed and stand as written. From D.7.7 on the steps are
  renumbered so that they run in ascending order, one at a time:
  1. *The tune stopped at its 2,000 checkpoint and its centres are the
     kernel's seeds* (D.7.6, D.7.6.2): +56.8 ± 9.5 Elo over the donor's
     seeds. This is the starting point for everything below.
  2. *No gate at this state.* The kernel is gated once, after the repairs
     and the finishing fit (D.7.13, on the gain bracket: condition 2 of
     2026-11-16 stands). The maintainer may ask for reads on the way; they
     decide nothing.
  3. *The classic kernel is deleted first* (D.7.7), before any repair,
     replacing "the old kernel beside it until adoption". If D.7.13 fails,
     the new kernel is discarded and the classic one restored from D.7.7's
     parent commit (D.7.15).
  4. *The repairs come from the audit* (D.7.8), not from a review of D.6's
     list. Every D.6 step, closed or open, is reviewed on the new kernel
     after D.7, in the next pull request (D.6, *Order, 2026-11-19*).
  5. *Clock use is inside D.7* (D.7.9, moved from D.6.28), before the fit,
     and the time manager's constants are coordinates of the fit's surface.
  6. *Rarog's own dropped forms are not retried in D.7*, replacing "tried
     again one at a time on the adopted kernel". Only a failed gate reopens
     them (D.7.15).
  7. *The kernel's scale stays free and the reported score is calibrated
     at the boundary* (D.7.12; the audit's finding A3). 2026-11-13's "no
     unit translation stays in the engine" covers the kernel's inside; the
     calibration is output only and never feeds a search or time decision.
  8. *Compactness is a secondary aim* (maintainer, 2026-11-19, repeating
     2026-11-18): it blocks nothing a need is shown for.

  *Numbers in records written before 2026-11-19:* D.7.7 was the gate (now
  D.7.13), D.7.8 the problem review (now D.7.8's repairs and the review of
  D.6 after D.7), D.7.9 adoption (now D.7.14; the classic kernel's deletion
  is D.7.7) and D.7.10 Stockfish's form (now inside D.7.15). The paragraphs
  above this one and the closed steps keep the old numbers and their dated
  decisions as written; where one disagrees with this block, this block
  holds. HISTORY's number map holds the same list.

  *How the open steps are worked.* One step at a time, in order. A step
  that changes play is part of the one cluster and has no gate of its own
  (rule 3): it is held to its contract and tests, to its zero-game read,
  and to one registered 2,000-game read at `3+0.03` against the build the
  step started from, under AGENTS' case-2 harm rule (below −13 nElo it is
  reverted and recorded) unless the step states another rule. Those reads
  are the maintainer's commands unless he delegates them in the session;
  the step prepares the binaries, the registration and the one command.
  Every build that plays is archived with its hash, and the kept state of
  one step is the next step's opponent.
    - **D.7.0 Investigation: the inventory of departures, the seeds and the registration of the reads — `R3`, DONE 2026-11-13.**
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
      **Done** (`analysis/search_shape_inventory_2026-11-13.md`): the
      kernel departs from the donor in about a hundred places, 69 of them
      in the node body, quiescence, the wrapper and the root window. Eight
      rest on an SPRT of their own, two that kept Rarog's form and six
      that rejected an alternative on the old kernel; about thirty-five
      rode inside a cluster's gate. Found beyond the list above: the
      donor's store on every alpha raise and its forced store at PV
      nodes; ProbCut at cut nodes only, at any depth and in check; a draw
      score of −2 to +2; no TT move in the donor's quiescence; the
      picker's promotion pushes and skip survivors; the correction's
      minor table and weights. Corrected: the donor does cut on a table
      hit at a PV quiescence node unless the score is decisive, and the
      queen push in quiescence (D.6.2) is the donor's form. The record
      behind the twofold draw rule is RAR-S18's four-part bundle, not the
      root-aware rule alone, so the new kernel takes the donor's rule.
      Units: the new kernel is built in the donor's units, the evaluation
      multiplied by 2536/1024 at the wrapper and scores divided back
      where they leave the kernel; Phase B converted the constants at
      ×0.457 and its seeds needed +138.6 of fitting. By maintainer
      decision the same day that multiplication is a scaffold only:
      D.7.4.1 folds the constants into Rarog's units and deletes it
      before any game. Parity: decision
      records from a tracing twin of the hybrid, reproduced exactly, and
      tree statistics on 500 positions against a measured noise floor;
      identical trees are not attainable, because the two engines hash
      their keys into the tables differently. RAR-O09 is registered with
      four matches and a frozen use; its prediction for the deciding
      match is 0 [−20, +20] and 0.20 that the rule fires, because
      RAR-O08's own reads put the donor about 1.1 to 1.2 times the
      head's positions a second with the odds and 8% ahead in clock
      spent. Verdict: `READY_FOR_IMPLEMENTATION` for D.7.1.1 and D.7.2;
      D.7.3 and D.7.4 are specified there and open with D.7.2's
      decision.
    - **D.7.1 The fixed-match script takes a node budget and a move time per engine — `I1`, DONE 2026-11-13.**
      `tools/sprt.ps1`: a node budget and a move time for each engine,
      beside `-TCA` and `-TCB`. Each is proved live by a smoke in which an
      absurd value moves the PGN and the score, and the combinations that
      make no sense are refused.
      **Done:** `-NodesA`/`-NodesB` (fastchess `nodes=`) and
      `-MoveTimeA`/`-MoveTimeB` (`st=`) beside `-TCA`/`-TCB`; an engine
      without its own limit takes the shared `-Nodes`, `-MoveTime` or
      `-TC`. Refused, each by `tools/diag/test_sprt_limits.ps1` (16/16,
      eleven refusals and five controls that reach the engine check): two
      limits for one engine, two kinds across the engines, a shared limit
      neither reads, a value of 0 or less given explicitly, a per-engine
      limit in a calibration, a malformed clock. Smokes
      (`analysis/artifacts/d71-smoke-2026-11-13/`, the D.6.8 arms, 20
      games each, `-Mode fixed`): a node budget of 1 against 20,000
      scored 0/20 and 20/20 with the sides swapped, the PGN showing depth
      1 at 0.000 s against depth 9 at about 24 ms; 5 ms against 200 ms a
      move scored 0.5/20 and 19.5/20, depth 6 at 5–8 ms against depth
      11–12 at 200 ms. Observed in passing: at a budget of 1 node the
      2.7.0 engine stopped inside the first iteration, printed its
      no-score marker as `score mate -0` and played the first legal move
      unsearched. Fixed in `112eb14` (maintainer decision 2026-11-13, both
      donors' forms): a node budget waits for the first iteration, as the
      clock already did and as Reckless does (`time.rs`, `check_time`);
      a move the search never scored, after a stop inside that
      iteration, prints `cp 0`, as Stockfish does (`search.cpp`, `pv`).
      Bench searches to a depth, so the fingerprint holds; repair case 1.
        - **D.7.1.1 The Colosseum wrapper takes a move time, a node budget per engine and a foreign arm — `I1`, DONE 2026-11-14.**
          Added 2026-11-13 by D.7.0, so RAR-O09 can run on Colosseum.
          Colosseum CLI 0.5.0's `match` already takes a node budget, a
          move time and a clock for each side; `tools/colosseum.ps1`
          does not pass them. `-MoveTimeMs` and `-MoveTimeMsB` with a
          run file `tools/colosseum/match-fixed-movetime.toml` (planned);
          `-NodesB`; `-ForeignB`, which declares an arm without a
          sidecar, records its hash and `id name`, requires A's sidecar
          and takes `-ExpectBench` for A alone, where today such an arm
          passes on warnings; without the switch a sidecar-less arm is
          refused. Each refusal and control in
          `tools/diag/test_colosseum_guards.ps1`; a smoke per limit in
          which an absurd value moves the PGN and the score. Then
          RAR-O09's four commands with their dry runs, written into its
          registration. Handoff:
          `analysis/search_shape_inventory_2026-11-13.md` section 9.
          Tooling only: no engine build. Not needed for RAR-O09 on
          fastchess, the maintainer's expected runner (2026-11-13), so
          D.7.2 does not wait for it.
          **Done 2026-11-14** (`10891ee`): `-MoveTimeMs` and
          `-MoveTimeMsB` with `tools/colosseum/match-fixed-movetime.toml`,
          `-NodesB`, and `-ForeignB`; without the switch an arm with no
          sidecar is now refused in a gate or a match, where it passed on
          a note before. `tools/diag/test_colosseum_guards.ps1`: 92 of 92
          cases, 23 of them new. Smokes on the pinned CLI 0.5.0, one
          binary on both sides, 8 games each: 20,000 against 200 nodes a
          move read 13,058 against 131 nodes a move and 8–0; 200 against
          2 ms a move read 190.2 against 1.9 ms and 8–0; a foreign
          Basilisk arm played 2 games, its SHA-256 and `id name` in the
          manifest. Not done: RAR-O09's four commands in Colosseum form.
          A registered experiment never changes its runner, and RAR-O09
          is on fastchess; the next match with a foreign arm on Colosseum
          writes its own.
    - **D.7.2 RAR-O09: the donor's search against the head at equal work, at equal move time and on the clock at two controls — `V`, DONE 2026-11-14.**
      Registered 2026-11-13 (`docs/experiments/RAR-O09.md`). Reckless's
      hybrid against the head, both rebuilt on the head, four fixed
      matches: 3,000 games at equal work a move (tree shape alone), 3,000
      at equal thinking time a move with the bridge's odds (shape and
      kernel speed), 3,000 at `3+0.03` with clocks (RAR-O08's match (5)
      again, on these binaries and this runner) and 2,000 at `10+0.1`.
      The differences between them are the shares of speed and clock
      use. Two instrument facts set the commands. Rarog counts a null
      move's child and the donor does not, so the donor's node budget is
      130,000 divided by one plus its null-move share, measured on a
      counting twin before any game. The donor stops 15 ms before its
      move time, so it is given 94 ms to Rarog's 70. Preparation: the
      two arms and the twin, the null-move share, the odds re-measured,
      a move-time smoke, then the four commands handed over. Runner:
      Colosseum once D.7.1.1 has landed and the pinned CLI keeps the
      book's move counters, else fastchess; chosen before the first
      game. Maintainer-run, about three hours.
      **Prepared 2026-11-14** (`docs/experiments/RAR-O09.md`,
      *Preparation*): runner fastchess (maintainer decision 2026-11-13);
      the three arms rebuilt on `1f29e04` (the head at fingerprint
      8,935,848 / EBF 2.480, the hybrid and its counting twin at bench
      4,158,758 both); null-move share 1.731% on 200 game positions
      (2.043% on bench), so the donor's budget is 127,800; odds ×1.1211
      on an idle host, so 70 against 93 ms a move and clocks of
      3.363+0.034 and 11.211+0.112. `sprt.ps1` now records each move's
      nodes in node-budget matches (`c487985`), which match (1)'s
      validity rule needs. Two rules were amended before any game
      (maintainer decisions 2026-11-14): the donor reports its budget
      plus two on about 1% of searches, because it tests its limit at
      node entry, so its tolerance in match (1) is plus two. The smoke's
      means fell below their bands through each engine's own early
      exits (one legal move, the iteration cap), which buy nothing under
      a move time, so the bands and match (2)'s ratio are read on the
      median, which sits on the commands (70.0 and 78 ms). The four
      commands are in the registration. Scope cut before any game
      (maintainer decision 2026-11-14): match (1) runs now, since only it
      feeds the rule and the rebuild's expectation; (2) to (4) stay
      prepared and run only when a decision or analysis names a use.
      **Read 2026-11-14:** `reckless-hce` **+3.47 ± 8.37** over the head
      at equal work (3,000 games, valid by a recorded deviation: 110
      donor moves reported 3 to 5 nodes over budget, against the
      amended plus two). PLAN's rule does not fire; the frozen
      recommendation is "unresolved for strength": a rebuild captures
      only the shape, worth about +3 before any fit. Decision 2 below:
      D.7.3 to D.7.9 are held by the maintainer (2026-11-14) until
      D.7.2.1's comparison with Basilisk and Basilisk's own evaluation
      programme are read. Reported, not Elo: at equal work the donor took
      91 against 60 ms a move, the opposite of the speed basis the
      registration used.
        - **D.7.2.1 RAR-O10: Rarog's and Basilisk's evaluations and moves against a reference — `R2`, DONE 2026-11-14.**
          Added 2026-11-14 (maintainer request, after RAR-O09). Basilisk's
          search programme gained about 100 Elo more than Rarog's from
          level starts while Basilisk stays about 35 to 80 behind, and its
          evaluation programme has not started. Without hybrids, which
          remove the fit each search was tuned around: each evaluation
          alone against Stockfish 19's expected score on 10,000 game
          positions, with Stockfish 11's evaluation and Rarog's Phase C
          start as anchors of known order; each whole engine's move at
          100 ms scored by the same reference, by position class. Zero
          games; a head-to-head match is optional. Registration and
          frozen use: `docs/experiments/RAR-O10.md`. Runs after Basilisk's
          running SPRT ends, before any further oracle read.
          **Read 2026-11-14:** the anchors pass. Rarog's head is more
          accurate than Basilisk's head (skill +0.0123 [+0.0016,
          +0.0235] against Stockfish 19's expected score); Basilisk's
          leads only with 10 to 24 pawns of material, and is below
          Rarog's Phase C start at the point. At 100 ms a move Basilisk's
          moves lose ×1.078 Rarog's (interval including zero), more with
          the most material, where its evaluation is furthest behind.
          By the frozen use H1 gains weight: Basilisk's evaluation
          programme has room, and if D.7 reopens for strength Basilisk's
          search programme, not Reckless's, is the candidate source. Both
          engines lose most with the most material (to D.7.8 and E.4); no
          named lead. Not Elo; an input to the maintainer's decision on
          D.7.3 to D.7.9.
        - **D.7.2.2 RAR-O11: the start points, per-node quality against speed, and evaluation cost — `R2`, DONE 2026-11-15.**
          Added 2026-11-14 (maintainer request, after RAR-O10). Rarog and
          Basilisk ran the same process from level starts and Basilisk's
          search programme gained more (+318 against +253, RAR-M84, and
          about +44 since, still open). Three explanations: Basilisk
          started with a weaker search and had more room; the donor's
          shape (Stockfish 19 against Reckless); the fit and the build.
          This read takes the first and the speed question: five engines
          (Rarog 2.4.0, 2.5.0 and head; Basilisk 1.10.2 and head) on
          RAR-O10's positions at 100 ms and at 150,000 nodes a move, scored
          by the same reference; each evaluation's cost a call; and
          Stockfish's classical evaluation against Rarog's in one search;
          then four fixed matches (the start pair and the programmes'
          ends, each at `3+0.03` and at 150,000 nodes), RAR-O10's match
          (c), and Stockfish dev's search on Rarog's evaluation at equal
          work (RAR-O09's form for RAR-O08's `sfdev-hce`). One night
          command, the maintainer's. Registration,
          frozen use and command: `docs/experiments/RAR-O11.md`.
          **Read 2026-11-15:** on the clock the start was level (Rarog
          2.4.0 −11.5 ± 11.7 against Basilisk 1.10.2) and Basilisk's
          search programme gained about 65 more than Rarog's (Rarog 2.5.0
          −76.3 ± 11.1 against Basilisk's head). The equal-node reads
          compared unequal node units: Rarog 2.4.0 and 2.5.0 predate
          D.6.8's count (about 21% high), and both Basilisks count
          re-searches, 1.10.2 also its hand-off to quiescence; so the
          per-node and speed readings are withdrawn until a
          work-equalised re-read (correction the same day, in the
          record). Unfitted on Rarog's evaluation
          at equal work, Stockfish dev's search is 34.3 ± 12.0 worse than
          Reckless's (+37.8 ± 8.6 for the head), so a Stockfish donor
          would rest on the fit, not a better shape. Rarog's evaluation
          is the cheapest per call of the four and the most accurate;
          Stockfish's classical evaluation is still lighter in one search
          and stronger. RAR-O10's match (c): Rarog's head +68.4 ± 9.0
          over Basilisk's head. D.7.2.3 is what separates the donor's
          shape as Basilisk built it from its fit and build.
        - **D.7.2.3 The two search programmes compared from their ledgers — `R2`, DONE 2026-11-16.**
          Added 2026-11-14 (maintainer request). Research only, read-only
          in Basilisk. For each programme, its accepted clusters in order
          with their gate results, split into what came from porting the
          donor's structure and what from refits and tunes; the
          mechanisms Basilisk took from Stockfish 19 that Rarog's search
          lacks, against D.7.0's inventory of Rarog's departures from
          Reckless; and what each programme measured at its start. It
          narrows explanations (2) and (3), which RAR-O11 cannot
          separate. Output: an analysis file in `analysis/`, and whether a
          Stockfish 19 hybrid read on Rarog's evaluation (the RAR-O08 and
          RAR-O09 instrument) is worth registering.
          **Comparison recorded 2026-11-15**
          (`analysis/d723_search_programmes_2026-11-15.md`; conclusions
          wait for D.7.2.4). Not in the selectivity core, where both end
          near +200 (Rarog's port +65 then its fit +139, Basilisk's +131
          then +59), but in the next clusters, unfitted as well as
          fitted: null move, ProbCut and singular (Rarog +50.5, Basilisk
          +92.8 and +17.6), quiescence (+4.4 against +31.7, +6.5 and
          +15.0), root (NO_CHANGE against +15.6), and a joint tune after
          the clusters (not run against +28.2), where Rarog kept its own
          forms and Basilisk ported Stockfish's whole. Basilisk is faster
          per position reached at every stage (+11.7% at the heads), with
          the dearer evaluation. The runner and the SPSA method are not
          the cause; a plain joint tune of the current search is expected
          at +5 to +15 and is worth running only if D.7 does not open.
          About half of D.6's open search leaves belong to the mixed
          kernel and would go with a full donor form; the clock (D.6.28),
          the quiescence stalemate (D.6.29) and the evaluation leaves
          would not.
          **Concluded 2026-11-16 with RAR-O12** (section 9 of the analysis):
          both levers are real. Basilisk's tree gained 55 more per move
          made, in the clusters where Rarog kept its own forms, and its
          speed and clock add about 50 at the programmes' ends. The
          recommendation: a full rebuild in one donor's form, ported whole
          and fitted as a unit, with the donor decided by the unfitted
          null-move, ProbCut and singular cluster read in each form; and
          Rarog's speed (D.12) and clock use (D.6.28) beside it, as they
          survive any kernel.
        - **D.7.2.4 RAR-O12: RAR-O11's equal-node reads again, at equal work in one node unit — `R2`, DONE 2026-11-16.**
          Added 2026-11-15 (maintainer request, after RAR-O11's
          correction). RAR-O11 compared engines at equal reported nodes,
          which were not equal work. Each engine's reported nodes over
          the positions its tree reaches by a move or a null move,
          measured on instrumented builds of its own source that
          reproduce its release's nodes and best move: Rarog 2.4.0
          ×1.417, 2.5.0 ×1.211, the head ×1.000, Basilisk 1.10.2 ×1.345,
          Basilisk's head ×1.037. At budgets in that unit, the moves on
          RAR-O10's positions and three matches (the start pair, the
          search programmes' ends, the heads). Registration, frozen use
          and command: `docs/experiments/RAR-O12.md`.
          **Read 2026-11-16** (in Stockfish's count, Basilisk's head
          `d17a62d`, Rarog's `9d7b42f`): at equal work the start was
          level (+0.5 ± 11.9 for Rarog 2.4.0); Basilisk's search
          programme gained 55.2 ± 16.6 more per move made (Rarog 2.5.0
          −54.7 ± 11.6 against Basilisk's head), so the tree question
          stands; speed and clock add 51.6 ± 16.1 at the ends and
          18.1 ± 15.6 at the heads for Basilisk (its head makes 15.8% more
          moves a second). Rarog's head leads +60.5 ± 11.5 at equal work
          and +42.4 ± 10.7 on the clock, by its evaluation programme.
    - **D.7.3 Parity instrument: Rarog and the hybrid on one position set — `I2`, DONE 2026-11-16.**
      Nodes to each depth and the rate at which each mechanism fires, on
      the same positions and the same evaluation; a planted difference
      must show.
      Designed by D.7.0
      (`analysis/search_shape_inventory_2026-11-13.md` section 7) in two
      layers. A tracing twin of the hybrid writes each decision's inputs
      and outputs, and every rule function of the new kernel must
      reproduce them exactly. Positions visited are now each engine's own
      count: since D.6.31 Rarog counts as the donor does, once per move
      made, so section 7's "the twin's nodes plus its null moves" is the
      twin's nodes. Tree statistics on 500 game positions are
      judged against a noise floor the twin measures on itself, with
      three planted differences that must fail. This leaf builds the
      twin, the trace format, the statistics tool and the floor; the
      rule tests land with D.7.4.
      **Done 2026-11-16** (`analysis/d73_parity_instrument_2026-11-16.md`):
      the twin (`tools/oracle-hybrids/reckless-0756957-rarog-hce-trace.patch`)
      reproduces the hybrid's `bench`, 4,158,758, with tracing off and
      on, and its salt and plants move it. On 500 positions of
      RAR-O08's match (5) (496 after four runaway searches), the floor
      is 0.021, 0.019 and 0.042 at depths 8, 11 and 14. The salted
      control passes. The `unreduced` and `q5` plants fail. The
      `lmr_check` plant passes (+0.084 at depth 14), which sets the
      instrument's stated limit: a change that size is for layer A.
      Layer A captured 250 positions with every one of 36 kinds at
      3,418 records or more from 210 positions or more; the trace stays
      in ignored storage with its hash.
    - **D.7.4 The kernel in the donor's form, behind one switch — `I2`, DONE 2026-11-18.**
      Own code in Rarog's structure, written in D.8's target shape (probes,
      not interleaved instrumentation). Parity by D.7.3 before any game;
      debug and release tests, fmt, clippy, the allocation guard and the
      search-path perft; the old kernel's fingerprint exact with the switch
      off.
      Handoff: `analysis/search_shape_inventory_2026-11-13.md` section 3
      (each departure with its proposal: the donor's form, kept, or
      dropped and asked again in D.7.8), section 5 (the donor's units and
      the seed table) and section 8 (scope, the order of eight commits
      and the invariants).
      **Done 2026-11-18** (`analysis/d74_compact_kernel_2026-11-18.md`;
      branch `d7-kernel` from `274adaa`, `ef1048d` to `042e2b9`). The
      kernel runs under the Cargo feature `compact-kernel`; the default
      build keeps the classic kernel, whose fingerprint stayed 9,067,595 /
      EBF 2.479 at every commit and whose pooled NPS reads −0.34% (95%
      −0.63 to −0.04), a pass by the no-regression rule (G1). Layer A: 47
      kinds, 3,269,262 records, no mismatch, 40 plants each failing its own
      kind; ten seeds no record detects, each named with its reason. Layer
      B, the deciding read on `a344748`: depth 8 −0.0098 ± 0.0257, 11
      +0.0121 ± 0.0227, 14 +0.0188 ± 0.0226, 36 of 36 rates, a pass. The
      compact fingerprint is **7,606,256 / EBF 2.433** (`a344748`), and the
      compact kernel is 11.54% faster a node than the classic (G2, at
      `cb7c79b`'s engine source).
      *Decisions, maintainer, 2026-11-17:*
      1. Inventory C-3: the compact kernel takes the donor's non-pawn
         correction key, kings included, because condition 1 of 2026-11-16
         carries none of Rarog's departures and the seeds were fitted with
         that key; not because it makes layer B pass.
      2. The frozen layer B rule alone decides, once, on a committed build;
         a failure stops with no games and no layer C without the
         maintainer's word.
      3. Independence: the node and quiescence following the donor's
         statement order is accepted, and the spans are named (below, the
         analysis and D.11).
      4. MultiPV under the compact kernel gets the donor's root window.
      5. D.4.4's pricing instrument is ported to the compact kernel.
      6. The twin's trace is extended and captured again, so layer A
         checks every rule and seed.
      7. The classic kernel's pooled NPS read precedes DONE; the other
         speed items are registered with an owner and a time.
      Later the same day: the compact kernel's SEE puts a promotion push
      through the exchange as the donor's does (`a344748`, after position
      305's diagnosis; the inventory's SEE-4 row was wrong); layer B is read
      again on `a344748` as the deciding read, declared before the result,
      `cb7c79b`'s pass kept as that build's; the three PV-only seeds are
      named as too sparsely sampled (`042e2b9`).
      *Departures from the inventory's commit order:* the picker landed
      with quiescence (`3fa2d96`), which cannot take the donor's form
      without it; the update seeds landed with the node body (`025ffc9`), so
      there is no commit 7; the probes are `1b8397d`. After the first layer
      B read: seven fixes (`856467e` to `62e9474`), the extended trace
      (`e84f4f9`, `635177b`, `474cdb7`), MultiPV (`cdda460`), pricing
      (`934213d`), a root-record fix (`e81227e`) and the SEE fix.
      *Recorded, not changed:* the singular search requires a table move,
      right here because a tablebase store carries none; the root's depth
      carries the reduced depth, which feeds only the first iteration's stop
      guards; the test that nodes equal moves made increments both counters
      at the same sites.
      *Open, for D.7.8:* position 305 diverges again from depth 3 (−866
      against the twin's −883) and stays 2.3 times the twin's largest
      salted tree, SEE-2 and SEE-3 the first candidates; the classic
      kernel shares the SEE shortcut on promotion pushes, recorded only.
      *Speed register:*
      - the board's `plies_from_null` kept in make, the null move and both
        unmakes, and the table store's force flag: G1, read 2026-11-17,
        no regression; closed;
      - the compact kernel against the classic head: G2, +11.54% a node,
        read 2026-11-17 before D.7.5; `a344748`'s SEE change is not
        re-read, and D.7.5's games carry it on the clock;
      - the king key computed at each correction read, the gate inputs
        built at the top of `compact_negamax` before any rule acts, and
        the root window's copies: done in D.7.8.9 (2026-11-20, +0.95%);
        closed;
      - the board's forms at large: D.12.
      *Squash paragraph, ready:* "The compact kernel is Rarog's own code in
      Reckless's search form. Two places are transcribed because the
      algorithm has one natural form: the interpolation toward beta (an
      `f32` `mul_add`, truncated) and the cuckoo tables of a repetition one
      move away (van Kervinck's method). The node's ProbCut loop, the three
      searches of its move loop and its tail, quiescence, and
      `Board::is_dead_material` follow the donor's statement order. The
      constants are the donor's, as seeds to be folded and refitted."
        - **D.7.4.1 The constants folded into Rarog's units and the scaffold deleted — `I2`, DONE 2026-11-18.**
          Added 2026-11-13 (maintainer decision: no unit translation
          stays in the engine, and the conversion is done properly this
          time). D.7.4 builds the kernel in the donor's units so parity
          can be read. This step converts each constant by its
          dimension (a margin times 1024/2536, a per-unit coefficient
          divided by it, ratios and counts unchanged), keeps sixteenths
          where a whole unit would move a value by more than 1%, and
          checks the result against the scaffold in one test build at
          zero games: every decision record reproduced except within one
          unit of its threshold, under 2% of any kind; positions visited
          within the noise floor plus 0.01; a constant converted by the
          wrong dimension must fail. Then the multiplication at the
          wrapper, the division at the boundary and the donor-unit
          constants are deleted, and the folded kernel's bench is its
          fingerprint. Rule:
          `analysis/search_shape_inventory_2026-11-13.md` section 5,
          *The fold*.
          *Exchange table (maintainer decision 2026-11-18):* the donor's
          piece values stay the compact kernel's exchange table (109, 403,
          435, 679, 1,242), so a threshold in exchange units is not
          converted; a term in evaluation units inside a SEE threshold is
          folded by its dimension like any other.
          *Done 2026-11-18* (`analysis/d741_fold_2026-11-18.md`): the
          fold `8489e46`, the guard's depth `7d476d3`, the compare's
          options `d7ca873`, MultiPV and the draw spread `5c33721`, the
          anchors and seldepth `2820eaa`. Layer A: 47 kinds without a
          mismatch, exceptions at most 1.44% of a kind, every plant and
          the three wrong-dimension plants fail theirs, 15 seeds that no
          record detects each with a checked reason. Check 2, registered
          before any result: positions visited pass at depths 8, 11 and
          14 (−0.0121, +0.0020, +0.0001); position 202 timed out. Compact
          fingerprint **7,543,485 / EBF 2.422** (was 7,606,256 / 2.433);
          the classic kernel's 9,067,595 / 2.479 unchanged.
          *Deviations:* the depth- and history-multiplied slopes are held
          in sixteenths and two per-unit steps in 1024ths; two premises of
          the inventory were false (the correction's update slope is a
          ratio, and its tables' saturation is in evaluation units, so the
          compact kernel has its own limits); check 1's "within one unit"
          is a bracket of the inputs moved one unit either way, and a
          record on a decisive score is a distance; futility's and
          bad-noisy futility's values had been returned in sixteenths, a
          defect the bench caught and the check did not. Four tests
          changed, each for a recorded reason: MultiPV lines search at the
          iteration's depth (an engine change; a line settled shallower
          outranked one settled at the depth), the three draw anchors
          accept the kernel's draw spread of one unit (maintainer decision
          2026-11-18), the seldepth test holds completed lines only (its
          premise was false: the failing line was a bound printed
          mid-iteration), and the allocation guard searches to depth 15
          (its precondition, its own commit).
    - **D.7.5 The unfitted paired run — `V`, DONE 2026-11-18.**
      2,000 games against the head (rule 8's screen), a diagnostic, not
      a stop (maintainer decision 2026-11-16). Clearly negative with
      parity passed is diagnosed once for a defect: units, scaling, a
      contract the port broke. A defect is fixed and the run repeated; no
      defect, and the step goes on to the fit, which is the system's
      completion.
      *Registered 2026-11-18 as RAR-S126:* both arms from `2820eaa`,
      the compact kernel against the classic, so the switch is the only
      difference; Colosseum, seed 20261118, the driver
      `analysis/artifacts/d75-unfitted-2026-11-18/run_s126.ps1`.
      *Read 2026-11-18:* nElo −11.1 ± 15.2 (Elo −6.9 ± 9.5), not clearly
      negative, so no diagnosis and the fit is next. The compact arm
      searched 1.2 plies deeper at the same time a move and converted
      winning scores better (98.1% against 96.2%); the classic arm reached
      a winning score in 52 more games. The prediction (+25) missed: the
      donor's lead at equal time (RAR-O08) was carried by its own clock,
      which this arm does not have.
    - **D.7.6 The fit — `V`, DONE 2026-11-19.**
      In Rarog's units, on the folded kernel (D.7.4.1), whose seeds are
      right to rounding. Curvature sweep, then one SPSA run under rule 7c, read at
      checkpoints and stopped when its gradients are gone. One joint run
      over every coordinate of the kernel, so the parts are fitted
      together. No gate of fitted over unfitted (maintainer decision
      2026-11-16): the tune's own gain read is recorded, and the one gate
      is D.7.7's.
      *Registered 2026-11-18 as RAR-S128* (surface
      `analysis/d76_surface_2026-11-18.md`): 246 seeds, 5 excluded by
      construction, 241 candidates by one range rule; an activation read
      on `bench 13` drops a candidate whose perturbations change nothing;
      the sweep on `lmr_log`, `rfp_square_16`, `nmp_r_base`,
      `fp_linear_16` and `singular_per_ply_1024`; then rule 7c's run to a
      ceiling of 6,000 iterations, seed 20261118. Time management stays
      carried over unchanged, as D.7 states.
      *Stopped 2026-11-19 at the 2,000 checkpoint (maintainer decision; rule
      7c's stop at a checkpoint, `STOP` in the run directory).* Theta is
      that checkpoint's centres (`tools/results/d76-fit/theta-2000.toml`,
      SHA-256 `d5b4998b…18dc9a`, 230 coordinates). The window was live (`sum
      z² − n` +30.4, `KLmrQuietHistory` over the line), so the stop is the
      maintainer's and not a quiet one. The interim read: the centres
      against the seeds **+56.8 ± 9.5 Elo** (nElo +93.1 ± 15.2 over 2,000
      games; predicted +20 nElo), the two wrapper coordinates alone
      +24.9 ± 9.0 (RAR-S129). The audit that followed is
      `analysis/d7_audit_2026-11-19.md`. What the tune had left goes to
      D.7.11's fit, on a repaired kernel and a completed surface. No gate
      at this state (maintainer decision 2026-11-19): D.7.13 is the gate.
        - **D.7.6.1 The compact kernel's seeds exposed to a tune build — `I1`, DONE 2026-11-18.**
          Added 2026-11-18 (RAR-S128's prerequisite): the seeds are a
          compile-time constant, so no build can change them. In the
          `tune` build only, each candidate becomes a UCI option by the
          surface's name and range, read by reference from the search's
          configuration (no lock, `Arc` or copy a node); the default and
          `compact-kernel` builds reproduce 9,067,595 and 7,543,485, the
          tune build at its defaults 7,543,485; the parity test keeps
          `Seeds::DONOR`. Then the activation read, the config files and
          the sweep, before any game.
          *Done 2026-11-18:* `a389c80` (engine: the options, the static
          table swapped at a search start when the options change, the root
          window given the search's seeds), `f7c89fb` (engine: a compact line
          held to eight times its iteration's depth, maintainer decision
          before any game), `89eeb41` (tooling: the coverage audit's
          `-TuneEngine`, class 6 read from a group's tune file), `2ca1ca1`
          (the surface files), `cff92ef` and `5671cd8` (tooling: the
          wrapper's dead-perturbation guard at rule 7c's graded
          perturbation, its suite's case moved to a surface that guard
          cannot resolve). Every fingerprint reproduces; the activation read
          keeps 231 of 241, and the surface is 230 with `KPickLateGood`
          dropped (`docs/experiments/RAR-S128.md`). RAR-S128 is amended
          before any game with PROCESS's pilot, which it had omitted, the
          line bound and the drop.
        - **D.7.6.2 The centres at 2,000 baked in as the kernel's seeds — `I1`, DONE 2026-11-19.**
          `Seeds::FITTED` beside `Seeds::DONOR` (`73966eb`): the kernel, the
          tuning options' defaults and ranges and the pricing plants read
          the fitted table; the parity test keeps the donor's, which only
          test builds compile. The compact fingerprint is
          **3,488,485 / EBF 2.336** (was 7,543,485 / 2.422): the tune
          binary's bench at theta, reproduced by the `compact-kernel` build
          and by the `tune` build at its defaults; the classic kernel's
          9,067,595 / EBF 2.479 is unchanged. One test's precondition was
          fixed first, in its own commit (`1a3420c`): the hanging-queen
          test asked for a gain of
          800 units and so pinned the wrapper's scale; it now states the
          swing without a scale and passes on the donor's seeds as before.
          The wrapper's and the window's form tests read the donor's table
          or the fitted numbers. Debug and release suites, fmt, clippy and
          the `tune` and `texel` library suites pass.
    - **D.7.7 The classic kernel deleted — `I2`, DONE 2026-11-19.**
      Maintainer decision 2026-11-19: the compact kernel is the engine's
      only kernel before any repair. If D.7.13's gate fails it is discarded
      and the classic kernel comes back from this step's parent commit.
      1. *First, the two baselines,* built from the commit this step starts
         on by `tools/build_test.ps1` (pext, PGO), archived in
         `tools/test_engines/` with their sidecars, their commit, bench and
         SHA-256 written here: `d7classic`, the default features (9,067,595
         / EBF 2.479), D.7.10's and D.7.13's opponent; and `d7baked`,
         `compact-kernel` (3,488,485 / EBF 2.336), the repairs' first
         opponent.
      2. *Deleted:* the classic node and quiescence with their rule code;
         the `Kernel` switch and the Cargo feature `compact-kernel` (the
         kernel is unconditional); the classic search coordinates of
         `SearchParams` that nothing reads any more (the time manager's and
         the root driver's stay); the classic sides of the pricing
         instrument and the probes; the tune groups under
         `tools/spsa_configs` that name deleted options. The eighteen tests
         that pin classic forms (`analysis/d74_compact_kernel_2026-11-18.md`,
         *Verification*) are retired or rewritten for the kernel's form,
         each named with its reason; no other test is relaxed. The root
         keeps its scorer in this step (`compact_root_picker`); D.7.8.3
         replaces it. Module and file names stay; renaming is D.8's.
      3. *Invariant:* `bench 13` reads 3,488,485 / EBF 2.336 before and
         after, exactly, on a fresh build. A count that moves stops the
         step until what the classic path still fed is found; nothing is
         kept "for now" to hold the count.
      4. *The fingerprint record moves* to 3,488,485 / EBF 2.336 in AGENTS'
         "currently", GUIDE's checkpoint and PLAN's checkpoint row, in this
         step's documentation commit; `check_guide.py` passes.
      5. *Checks:* debug and release suites, fmt, clippy over all features
         and targets, the `tune` and `texel` library suites, the allocation
         guard, the search-path perft, the parity test on the donor's
         table, xtask's PGO training check, and the tune build's option
         list against `tune.rs`'s rule.
      6. *Reported:* the kernel's search code in lines against the donor's
         1,488 (condition 1 of 2026-11-16), and what the deletion removed.
      *Done 2026-11-19.* **Baselines,** built from `29f32fb` (clean tree)
      by `tools/build_test.ps1`, pext PGO, `id name Rarog
      dev-20261119-29f32fbb`, in `tools/test_engines/` with their sidecars:
      `rarog-d7classic-pext-pgo.exe`, default features, 9,067,595 / EBF
      2.479, SHA-256
      `781797e99f40d5b118032c9d110e1cb2d291817015b068ecf5130994893926f4`;
      `rarog-d7baked-pext-pgo.exe`, `compact-kernel`, 3,488,485 / EBF
      2.336, SHA-256
      `7d49ba499b7bb980a89cd7b31bda9bdec26ba0c3ab34179c1238c45dc5a668b4`.
      Each copy's own `bench 13` read its count. **Commits:** `2a3d46b` and
      `5d6f737` (two test preconditions, below), `d23b9cf` (engine: the
      deletion, 31 files, 574 lines added and 7,442 removed), `1c175b2`
      (tooling: the tune groups), `0a885fe` (a test's name). A restored
      classic kernel starts from `5d6f737`, whose two tests pass on both
      kernels. **Deleted:** the classic node and quiescence with their
      rule helpers and test hooks; the `Kernel` switch and the feature
      `compact-kernel`; the classic aspiration window; `CoreParams`,
      `ProofParams` and `QuietParams` with their compiled-in handle, and
      the nine `SearchParams` coordinates only the classic kernel read
      (`AspMaxFails`, `LazyMargin` and the thirteen `Tm*` stay); the
      classic staged picker and its skip logic (the root's scorer stays
      for D.7.8.3, its `ScoredMove` without the two fields nothing read);
      the classic correction formula and history updates, and the minor
      and four-ply correction tables the kernel never read (about 4 MB a
      thread no longer cleared each game); the TT probe's classic
      readings; the classic sides of the pricing instrument (its
      parameter swap: the kernel confines through its seed table); the
      board's classic draw rule (`can_declare_draw_in_search`) and fast
      check test, with the discovered-check blockers only that test read;
      the decision trace only the classic node wrote; the diag
      correction-slot probe only it fed; the tune groups `b23core`,
      `b27all`, `b27core`, `b33`, `b43`, `c2margin`, `c3margin`,
      `c10pilot`, `c10block1`, `c10long` and `d16fix` (45 files), with
      the guard suite's tune cases moved to `d76fit` and its `Core*`
      guard deleted. The diag build's counters only the classic node fed
      stay and read zero (`board_gives_check_fast_calls` among them, which
      `tools/diag/board_search_profile.py` reads); they are D.8's to
      prune. **Invariant:** a fresh default release build reads
      3,488,485 / EBF 2.336, all forty positions' depth, score, nodes and
      EBF identical to `d7baked`'s; the tune build at its defaults reads
      the same. **Size** (condition 1 of 2026-11-16): the donor's
      `search.rs` at `0756957` is 1,490 lines by count (1,187 code lines,
      non-blank and not a comment); the kernel's node and quiescence
      (`compact/node.rs`, `compact/quiescence.rs`, tests excluded) are
      1,689 lines, 1,599 code lines, and 1,307 without the `pricing` and
      `probes` statements; with its picker, rules and both seed tables,
      window and module root, 4,773 lines, 4,061 code lines, 3,506
      without the instruments. **Tests.** The eighteen of D.7.4's record,
      at `29f32fb` with the compact kernel the default: rewritten for the
      kernel's form, each keeping its contract —
      `a_depth_one_search_counts_each_root_move_once` (one open root
      window at depth 1: twenty), `a_node_budget_waits_for_the_first_iteration`
      (the first iteration's nodes plus one),
      `a_clock_spent_before_the_search_began_still_returns_a_first_iteration_move`
      (every line at depth 1, the last exact naming the move),
      `search_writes_one_info_line_per_iteration_through_its_sink` and
      `a_proved_mate_ends_a_clock_search_after_three_iterations_but_not_analysis`
      (the kernel's window opens from depth 1, so they count exact lines
      and let bound lines pass), `every_search_parameter_moves_what_it_tunes`
      (the sixteen `SearchParams` left; the seeds are a tuning build's),
      `search_populates_persistent_root_move_records` (a line of at least
      the move: after a fail high the window searches one ply less),
      `a_stored_win_never_answers_a_position_repeating_the_game_history`
      (a third occurrence: N5's twofold is the donor's rule now),
      `bare_king_promotion_choice_in_search` (no minor promotion in the
      move or its line: at depths 6 and 7 the kernel plays `b8a7` with
      `f7f8q` next; MP-1's anchor stays D.6.17's question);
      retired with the forms they pinned —
      `a_fail_low_store_carries_its_best_scoring_move` (T8),
      `no_line_searches_past_twice_the_iteration_depth` (N6; the kernel's
      eight times is `a_line_keeps_eight_times_its_iteration_depth_before_quiescence`),
      `no_line_spends_more_than_the_iteration_depth_on_extensions` (LR4),
      and `a_null_move_adds_no_count`, `the_root_never_takes_a_null_move`,
      `the_verification_region_is_cleared_after_verifying_and_at_search_start`,
      `singular_decisions_run_in_a_middlegame_search` and
      `each_recursive_call_types_its_child_by_its_rule` (hooks only the
      classic node had; the kernel's rules are layer A's);
      `a_root_fail_high_past_the_soft_target_plays_the_move_with_its_lower_bound`
      passes unchanged. Two more failed on the compact kernel since the
      bake (`73966eb`), whose checks ran the default build only; each
      got its precondition fixed in its own commit:
      `kq_against_two_minors_is_the_queens_win_at_depth_1` (`2a3d46b`:
      it asked the search for 150 in evaluation units; the wrapper reads
      the three positions at 185, 232 and 149 against evaluations of 347,
      427 and 291, so it asks the evaluation for 150 and the search for a
      win) and `a_helper_searches_every_line` (`5d6f737`: it ranked upper
      bounds as scores, and the kernel's root publishes a fail-low against
      a mate as that mate; it counts at least three exact scores at the
      final depth, where one line leaves two). Tests of deleted code went
      with it: 32 of the classic `node.rs` (the eight above among them;
      the rest unit tests of its helpers, or classic forms called
      directly: a late direct check surviving the quiet skip, which the
      kernel skips with the rest; quiescence's PV-bit hand-off; LMR's
      root and in-check counters, which only the classic node fed; mate
      after a first-ply check in quiescence, covered by
      `a_hanging_queen_is_taken_and_a_mate_is_scored`), the classic
      correction's formula tests, two history updates, the fast check
      test, the TT probe's classic readings, the staged picker's, the
      decision trace's and xtask's two-kernel profile test. Three
      form-free contracts of the classic node moved onto the kernel (the
      cutoff reset at the ply cap, an exclusion search with only the
      excluded move scoring no mate, quiescence at the ply cap) and five
      of the classic correction's (the raw evaluation under any clock,
      training direction, continuation slots, draw verdicts, the
      tablebase band). The
      board's draw tests read the kernel's rule (`is_draw_at`): a single
      prior occurrence draws only inside the search, and a repetition
      across a null move is no draw
      (`a_repetition_across_null_moves_is_no_draw_in_search`, renamed in
      `0a885fe`). The zugzwang suite's material-guard arms set a deleted
      coordinate; its four soundness checks run once
      (`the_null_moves_soundness_properties_hold_together`). The
      search-path perft walks the kernel's picker and make. No other test
      was relaxed. **Checks** on the final state, each once, exit status
      read directly: `cargo fmt --check` 0; `cargo clippy --all-features
      --all-targets` 0, no warning (none under each feature alone either,
      read before the last comment-only edits);
      debug suite 506 passed, 13 ignored; release suite 508 passed, 12
      ignored (the allocation guard and both search-path perft tests
      among them); `tune` and `texel` library suites 266 passed each;
      xtask 57 passed; the parity test on the donor's trace
      (`analysis/artifacts/d74-parity-2026-11-17/trace-a`) 47 kinds, no
      mismatch, every plant failing its kind; xtask's PGO build: all
      copies of both kernel functions trained; the tune build lists the
      241 seed options by `tune.rs`'s rule and the sixteen `SearchParams`,
      and no other tuning option (a check that fails the D.7.6.1 tune
      build on 129 classic options); the Colosseum guard suite 92 of 92, the converter
      tests, the three groups' `--check` and the coverage audit with the
      tune build clean. Records and logs:
      `analysis/artifacts/d77/` (ignored).
    - **D.7.8 Repairs from the audit — `R2` cards, `I1`/`I2`/`V` sub-steps, DONE 2026-11-20.**
      The audit's findings (`analysis/d7_audit_2026-11-19.md`, A1 to A8),
      one sub-step each, in order, worked as D.7's *How the open steps are
      worked* states. The step closes after the last of them, with a table:
      each sub-step, what it changed, its read and its commit. On
      2026-11-20 (maintainer decisions) D.7.8.7 moved to D.13 with its
      card, its number retired, and D.7.8.11 was added from the question
      RAR-S132's read left: whether the donor's live re-scoring of the
      root is the better form. Later that day the step was reopened for
      D.7.8.12, a speed pass before the fit (maintainer decision: as much
      effective speed and NPS as can be had before the work continues,
      Reckless as the reference). The steps this one replaces left three
      records, kept here.
      *From D.7.4 (text of 2026-11-18):* position 305's second divergence
      from its donor is D.7.8.8. The pricing identity's null-move and
      razoring misses (0.70% and 0.42%, cause unknown) are found before any
      pricing read on the new kernel; no step of D.7 takes a pricing read.
      *From D.7.4.1, the downgraded-mate band, position 202's class (text of
      2026-11-18).* A mate the fifty-move clock could falsify reads back
      from the table as the highest ordinary score, as in the donor and
      Stockfish, but the compact kernel, before and after the fold, then
      grows trees one to two orders of magnitude past the twin's (202 at
      depth 14 did not finish in 590 s; the classic kernel reports the
      mates). The donor's band sits elsewhere (`MAX_PLY` 240, decisive from
      31,519) and it also downgrades tablebase scores. RAR-S126's games
      show no conversion cost: the band appears in 301 of the compact arm's
      games, 300 won.
      *Diagnosed 2026-11-18 at zero games (RAR-S127), on the maintainer's
      decision:* no port defect. Once a mate is in view, lines run to
      whatever ply cap exists (with the cap and band at the donor's, 240
      and 31,520, to 224 plies), as the donor's do (the twin needs 34.9
      million nodes on 202 at depth 18); ordinary lines are the donor's
      length (PV seldepth 23.08 against 22.96 on 100 parity positions);
      both named candidates are refuted. Rarog's line bound of twice the
      depth contains it on both positions but moves `bench 13` by −36%, a
      shape change across the kernel; it is a form of Rarog's own and is
      read again only if D.7.13 fails (D.7.15).
      *Corrected 2026-11-18 (RAR-S128's activation read):* a fifth of the
      fit's perturbations made bench 16, a won pawn ending, run to about a
      billion nodes, so the kernel carries a line bound of eight times the
      iteration's depth (`f7c89fb`, maintainer decision before any game; it
      binds no bench tree; four times failed the KBNK anchor). It stays.
      *Found by the tests rebuilt on the kernel (2026-11-19, `38528bc`),
      neither repaired in D.7:* (1) once late-move pruning or futility
      fires, the kernel's picker skips every quiet left, direct checks
      among them, as its donor's does; the classic kernel kept them. On
      the WAC positions and their continuations it plays a move that
      allows a mate in one in 1 of 123 positions where some move avoids
      one, at every depth to 7 (the classic kernel: none); with the skip
      off it does not. A dropped Rarog form, so not retried inside D.7
      (decision 6); `no_mate_in_one_is_allowed` is an ignored test until
      it is. (2) At equal nodes the kernel solves fewer WAC positions
      than the classic: 185 against 210 at 25,000 nodes a position, 201
      against 232 at 50,000, 219 against 243 at 100,000. Tactics, not
      Elo; the WAC band now reads 50,000 nodes and is calibrated at 201.
      *The table, final 2026-11-20* (the fingerprint after the last is
      3,320,355 / EBF 2.317):

      | Step | What changed | Read | Commits |
      |---|---|---|---|
      | D.7.8.1 | ProbCut's exchange bar at the donor's scale, 2,536 | RAR-S130, nElo −7.2 ± 15.2, kept | `ee1c79c`, `e63b652` |
      | D.7.8.2 | Quiescence and evaluation-only entries at the donor's depth, −1 | RAR-S131, +1.2 ± 15.2, kept | `3e27052`, `e363908` |
      | D.7.8.3 | The root in the staged picker's order, ties included; the classic scorer deleted | RAR-S132, +4.1 ± 15.2, no fault at four threads, kept | `2f47a78`, `4271031` |
      | D.7.8.4 | A failed window's bound line only past 10,000,000 nodes | none: play unchanged, info lines 16.0 against the donor's 15.8 | `1b153dd` |
      | D.7.8.5 | Nothing: 0 of 300 searches at `3000+30` discard a move | the frozen screen, `NO_CHANGE` | `927063c` (counters) |
      | D.7.8.6 | One per-thread reset at a new game, helpers included | none: one thread exact | `60424e5` |
      | D.7.8.7 | Moved to D.13 with its card (2026-11-20) | — | `31dede4` (card) |
      | D.7.8.8 | Nothing: the divergence is the pickers' tie rule, recorded | zero games | `66fd73b` |
      | D.7.8.9 | The king-keyed non-pawn keys kept, the gates asked lazily, the root list filled in place | pooled NPS +0.95% (+0.37 to +1.54) | `80eec20`, `e68d245`, `b88700a` |
      | D.7.8.10 | Nothing: a unit half the size | RAR-S133, −4.9 ± 15.2, `NO_CHANGE` | `f92abf0` (registration) |
      | D.7.8.11 | Nothing: the root's live re-scoring changed the next move in 0.96% of re-scores, under the frozen 1% | zero games, `NO_CHANGE` | recipe `1bbab9f-d7811-root-rescore.patch` |
      | D.7.8.12 | The child's table cluster asked for as soon as the move is picked; one fixed-capacity list for every move list | pooled NPS +2.71% (+2.11 to +3.31) | `1ff6522`, `80cef6f`, `0690de9` |

        - **D.7.8.1 ProbCut's exchange bar in exchange units — `I1`, DONE 2026-11-19.**
          (A1) ProbCut's picker holds captures to `probcut_beta − eval`,
          an evaluation-unit difference, tested against the donor's
          exchange values. The fold converted quiescence's bar
          (`q_see_scale`) and not this one, so the bar is 0.40 of the
          donor's and D.7.4.1's layer B read `probcut_depth` firing 27% more
          a position. A seed `probcut_see_scale`, in 1024ths, multiplies
          the bar: 1,024 is today's play and reproduces the fingerprint;
          2,536 is the donor's form at the fold's scale. The bar leaves its
          inline expression for a rule a test reads, a planted scale
          failing it.
          *Read, registered:* the tune build on both sides, 2,536 against
          1,024, 2,000 games at `3+0.03`; beside it at zero games the bench
          and ProbCut's attempts a node at both values. The donor's value
          becomes the seed unless its interval is wholly below zero; either
          way the scale is a coordinate of D.7.11's surface.
          *Implemented 2026-11-19 (`ee1c79c`):* `probcut_see_bar` in
          `rules.rs`, the fitted table at 1,024 and the donor's at 2,536;
          `bench 13` 3,488,485 / EBF 2.336 on all forty positions, and
          planted at 2,536 the rule's test fails and the bench reads
          3,278,226 / 2.313. `KProbcutSeeScale` takes a declared range,
          512 to 4,096: the range rule's half to twice of 1,024 does not
          reach 2,536, so as written the read could not be played.
          Checks: fmt, clippy over all features at zero warnings, debug
          535 and release 537 passed, the `tune` and `texel` libraries
          288 each, parity on the donor's trace without a mismatch (the
          seed named among those no record detects). At zero games the
          donor's bar lets 17% fewer captures a node into ProbCut (49.94
          against 41.48 a 1,000 nodes) with verifications almost level.
          The read is RAR-S130, registered before any game; its binary
          is `rarog-d781-tune.exe`. The step closes on the read.
          *Done 2026-11-19:* RAR-S130 read nElo −7.2 ± 15.2 over 2,000
          games, the interval [−22.4, +8.0] not wholly below zero, so the
          donor's 2,536 is the fitted seed (`e63b652`); `bench 13` 3,278,226 /
          EBF 2.313, the companion's bench reproduced by a fresh default
          build. Checks on that state: fmt, clippy over all features at
          zero warnings, the debug and release suites, the `tune` and
          `texel` libraries. The scale is a coordinate of D.7.11's surface.
          D.7.8.2 starts from this state.
        - **D.7.8.2 The table's depth for quiescence and evaluation-only entries — `R2`, DONE 2026-11-19.**
          (A5) Rarog stores a quiescence entry at depth 0 and an
          evaluation-only entry at −2; the donor stores both at −1. The
          keep rule (`depth + 4 + 2·pv <= entry.depth`) therefore lets a
          quiescence store overwrite a same-position entry of depth 3 (5
          on the PV) that the donor keeps, and the victim rank puts
          evaluation-only entries below quiescence ones where the donor
          ranks them level. TT-7 recorded the offset and corrected the
          null-move bound alone.
          *Card first:* every reader of the two depths (the table cut, the
          singular candidate's `tt_depth`, the reductions' `tt_depth <
          depth`, the null-move bound, the keep rule and the victim rank),
          and the encoding under which each reads as the donor's with
          TT-7's conversion deleted. *Then* one commit with the table's
          tests; the fingerprint moves; the 2,000-game read.
          *Card (2026-11-19):* the readers of a stored depth in the kernel
          are the table cut (`tt_depth > depth − (score < beta)`), the
          singular candidate (`tt_depth >= depth − 3`), the reductions'
          `tt_depth < depth` and `tt_depth >= depth`, the PV search's
          `tt_depth > 1` and the null-move bound (`depth − 2 <= tt_depth`).
          The first five ask about a node of depth 1 or more and read a
          quiescence or evaluation-only entry alike at 0, −1 or −2; the
          evaluation-only entry has no score, which each also requires.
          The null-move bound is the one that differs, and TT-7 converted
          it. The table's keep rule and victim rank read the depth
          itself. The encoding that reads as the donor's everywhere is
          both kinds at −1 (`TtDepth::SOME`), an evaluation-only entry
          told apart by its missing bound; the empty slot stays apart
          (depth 0, no bound).
          *Implemented 2026-11-19:* `3e27052` (tests, first: the
          shortest-mate check for KQK split out, below) and `e363908`
          (engine: `QUIESCENCE_DEPTH` and `EVAL_ONLY_DEPTH` at −1, TT-7's
          conversion deleted, the table's tests of the keep rule and the
          rank, a planted depth 0 failing the first). `bench 13` 3,348,508
          / EBF 2.329 (+2.1%). Checks: fmt, clippy over all features at
          zero warnings, debug 537 and release 539 passed, the `tune` and
          `texel` libraries 290 each, parity on the donor's trace without
          a mismatch (3,418 null-move bound records). *Found at zero
          games:* KQK's mate in four (`k7/8/8/8/8/2K5/8/1Q6 w`) is
          reported as a mate in six from depth 10 to 17 and as four only
          at 18, where it was four from depth 9: the quiescence half alone
          does it (built alone, the same bench and mates), a kept deeper
          entry holding the longer mate; Reckless's search reports five at
          every depth from 9 to 18. The mate is still found and played;
          the exact check is an ignored test naming this (maintainer
          decision 2026-11-19). The read is RAR-S131, registered before any
          game: `d782cand` against `d782base`, case 2. The step closes on
          the read.
          *Done 2026-11-19:* RAR-S131 read nElo +1.2 ± 15.2 over 2,000
          games, above the −13 harm line: kept. D.7.8.3 starts from
          `e363908`'s engine (`d782cand`).
          *Superseded 2026-11-19 (D.7.8.3):* with the root in the kernel's
          own order, ties included, KQK's mate in four is reported from
          depth 8 and its check runs again; the longer mate needed the
          classic root order with the donor's keep rule.
        - **D.7.8.3 The root's moves ordered by the kernel's own scorer — `I2`, DONE 2026-11-19.**
          (A8, inventory R6) The root's list is scored by the classic
          picker's scorer over the kernel's tables, the last user of that
          scorer and its constants once D.7.7 has run. The root takes the
          kernel's order (the table move, then `noisy_order` with the
          good-noisy bar, then `quiet_order`) over its whole list; the
          pool's shared order and the helper's rotation stay, which are
          Lazy SMP's; the classic scorer and its constants are deleted.
          The fingerprint moves. *Read:* the 2,000-game read, and 200
          games at four threads for faults alone.
          *Implemented 2026-11-19:* `2f47a78` (test precondition, first:
          the KRK mate-line check reads at depths 14 and 18, past the
          first mate, verified on `e363908`'s engine) and `4271031`
          (engine). The root's list is written in the staged picker's
          order, ties included: the moves come from its generators and
          its selection step orders them, the good-noisy bar decided in
          that order with its count of good moves so far; the whole list
          is ordered and then narrowed to the root's lines, as the
          donor's root skips a move outside its lines once handed over.
          The classic scorer, its constants and its two tests are gone;
          the tests that replace them show every legal move once, a
          narrowed list keeping the whole list's order, and the root's
          order equal to the staged picker's move for move on empty and
          filled histories (planted faults in the bar, its count and the
          table move's partition each fail it). `bench 13` 3,320,355 /
          EBF 2.317 (−0.8%). Checks: fmt; clippy over all features and
          targets at zero warnings (a planted lint showed it reports);
          debug 538 and release 540 passed with the KRK check failing
          before its precondition commit, the behaviour tests 8 of 8 in
          both after; the `tune` and `texel` libraries 291 each; parity
          on the donor's trace without a mismatch. *Found at zero
          games:* a first form at the same scores with the full
          generator's tie order read 4,408,144 / EBF 2.350 (+31.6%, one
          position ×5) and reported the queen mate in three as seven at
          depth 10; the root's tie order alone moves the tree by a third.
          With ties as the staged picker's, the mate in three shows from
          depth 8 and KQK's mate in four from depth 8 (D.7.8.2's ignored
          check runs again); KRK (`k7/8/8/8/2K5/8/8/7R w`, Stockfish's
          five) first shows a mate at depth 12, nine to depth 18, where
          `e363908` showed six from 10 and Reckless's search six from 12.
          The read is RAR-S132, registered before any game: `d783cand`
          against `d783base`, case 2, and the four-thread smoke for
          faults. The step closes on both.
          *Done 2026-11-19:* RAR-S132 read nElo +4.1 ± 15.2 over 2,000
          games, above the −13 harm line, and no fault in 200 games at
          four threads: kept. *Found after the games:* the donor's
          picker re-scores the root's remaining noisy moves after each
          good one it hands over and its remaining quiets before each,
          so a root move searched updates the order of those after it;
          this step scores the root once a node. Not in this step's
          text and not read; left to the maintainer (2026-11-20) whether
          it becomes a step of its own.
        - **D.7.8.4 Bound lines in short searches — `I1`, DONE 2026-11-19.**
          (A4) The kernel's narrow window fails often by design, and Rarog
          prints a bound line for each failure: 39 info lines a
          100,000-node search, 24 of them bounds, against the donor's 16
          and none. A failed root window's bound line is printed only once
          the search has counted 10,000,000 nodes, the donor's rule; exact
          lines are unchanged. The fingerprint is exact; the info-line
          tests state the rule; `CHANGELOG.md` gets the user's sentence.
          *Check:* info lines a 100,000-node search on the audit's five
          positions (`infolines.py`), near the donor's count. No game read:
          play is unchanged.
          *Done 2026-11-19* (`1b153dd`): a failed root window's bound line
          waits for 10,000,000 nodes on the search's count; the line a
          fail-high past the soft target stops on is still printed, as the
          report of the move played (the donor prints on a stop too). The
          floor is an `EngineOptions` field no UCI command sets, so a test
          can ask for every bound line; the stopped-fail-high anchor does,
          its scenario unchanged. Tests state the rule: no bound line
          under the floor with the exact lines unchanged, the stop line
          printed under it, a short process search printing one exact line
          an iteration; planted faults in the floor and the stop line each
          fail them. `bench 13` 3,320,355 / EBF 2.317, exact. *Check:*
          `infolines.py` at 100,000 nodes on the audit's five positions:
          16.0 lines a search, none a bound, against the donor's 15.8 (0.2
          bounds) and the step's start's 38.8 (22.8). Checks: fmt; clippy
          over all features at zero warnings; debug 540 and release 542
          passed; the `tune` and `texel` libraries 292 each; parity on the
          donor's trace without a mismatch. `CHANGELOG.md` has the user's
          sentence.
        - **D.7.8.5 The interrupted iteration — `R2`, then `I2` if its screen fires, NO_CHANGE 2026-11-19.**
          (A2) On a stop inside an iteration the root plays the last
          completed iteration's move; the donor plays `root_moves[0]` with
          the interrupted iteration's results, behind its forgotten-mate
          and aborted-loss guards (inventory R7, kept as Rarog's). The
          classic kernel stopped inside an iteration in 1 of 300 searches
          at `3+0.03` and discarded nothing
          (`analysis/b5_research_2026-09-30.md`); the compact kernel's
          iterations are several narrow windows each and were never read.
          *Screen, zero games, its rule frozen here:* that analysis's
          instrument on the kernel, 300 searches at each of its three
          clocks, counting searches stopped inside an iteration and, of
          those, a new best move or a failed-high move discarded. At 2% or
          more of the searches at `3+0.03` discarding one, the donor's rule
          is built; under it the step closes `NO_CHANGE` with the counts.
          *If built:* the root plays the interrupted iteration's best fully
          searched move when it beat the previous best, with the donor's
          two guards, D.6.30's proven-win rule kept; a test interrupts a
          search at swept budgets (`tests/root_abort.rs`). The 2,000-game
          read, and RAR-S129's match (P) again at 100,000 nodes a move
          against `reckless-hce`, the parity read this rule clouded.
          *NO_CHANGE 2026-11-19* (`analysis/d785_interrupt_2026-11-19.md`):
          B.5's counters on the kernel's root driver (`927063c`, diag
          only, the fingerprint exact), live under node budgets (90 of 90
          searches stopped inside an iteration). Of 300 searches, stopped
          inside an iteration and discarding a new best or failed-high
          move: `3000+30` 1 and **0 (0%)**, `1000+30` 4 and 1 (0.33%),
          `300+30` 24 and 4 (1.33%). Under the frozen 2%: the donor's rule
          is not built and RAR-S129's (P) is not re-run. Retry trigger: a
          time manager or a rated control that raises the share at its
          clock to 2%.
        - **D.7.8.6 A helper's previous score at a new game — `I1`, DONE 2026-11-19.**
          (A6) `reset_worker_state_for_new_game` leaves a helper's previous
          search score, so its first window of a new game opens round the
          last game's score. Reset with the main thread's; a test fails
          without it. The one-thread fingerprint is exact; no game read.
          *Done 2026-11-19* (`60424e5`): the main thread and every helper run
          one per-thread reset (histories, pawn cache, both previous
          scores), so the two cannot drift again; the test fails with the
          compact score left out. `bench 13` 3,320,355 / EBF 2.317,
          exact. Checks: fmt; clippy over all features at zero warnings;
          debug 541 and release 543 passed; the `tune` and `texel`
          libraries 293 each; parity without a mismatch. No changelog
          entry: a helper's first window is not something a user sees.
        - **D.7.8.8 Position 305's second divergence from the donor — `R2`, DONE 2026-11-19.**
          (from D.7.4) With the promotion push put through the exchange,
          position 305 equals the twin at depths 1 and 2 and diverges from
          depth 3 (−866 against −883), its depth-14 tree 2.3 times the
          twin's largest salted one. The first candidates are the kept SEE
          forms: SEE-2 (the donor's static pins against Rarog's exact
          legality) and SEE-3 (no promotion gain on a recapture). Zero
          games, one session: the first differing decision on the trace,
          named; a card if it is a departure worth taking, otherwise
          recorded with its reason. It changes no play by itself.
          *Done 2026-11-19, recorded, no card*
          (`analysis/d788_p305_2026-11-19.md`; the diagnostic
          `analysis/arm_patches/a344748-d788-p305-diag.patch`): the first
          differing decision is the pickers' tie rule. Among equal scores
          the donor takes the last entry and removes it with
          `swap_remove`, its table move leaving the list the same way;
          Rarog takes the first. With the donor's tie rule `a344748`
          equals the twin on 305 in nodes and every rule count through
          depth 6 (797, 1,149, 1,309, 2,009); a third divergence starts at
          depth 7. SEE-2, SEE-3, the king rule and castling are refuted
          (every donor SEE form together reads `a344748`'s 956 at depth
          3), and so is the donor's root re-scoring. At depth 14 the tie
          rule alone runs 305 to 184 million nodes and with the root's
          re-scoring to 530,420, inside the twin's salted range: tie order
          is chaotic, not a quality, so the 2.3 times D.7.4 recorded is an
          instance of it, and matching the donor's tie rule would buy
          parity on a diagnostic, not strength.
        - **D.7.8.9 The speed register — `I1`, DONE 2026-11-20.**
          (from D.7.4) Three items waited for adoption: the king key
          computed at each correction read, to be kept incrementally; the
          gate inputs built at the top of the node before any rule acts;
          and the root window's three copies of about 4 KB. Each its own
          commit, the fingerprint exact, no `memcpy` in the node's
          assembly. *Read:* the pooled NPS read by PROCESS's method against
          the build the step started from, prepared here. It comes before
          the fit so the fit runs at the kernel's final speed.
          *Done 2026-11-20.* Three engine commits, `bench 13` 3,320,355 /
          EBF 2.317 exact at each: `80eec20` (the non-pawn keys carry
          their kings, kept by the per-type masks as every piece moves;
          nothing read the key without the king); `e68d245` (the three
          gates: reverse futility and the null-move gate are asked only
          where the gate before did not end the node; the `probes` build
          still asks all three first, its rates byte-identical on a
          depth-12 search); `b88700a` (the root fills its picker's list in
          place: since D.7.7's 8-byte `ScoredMove` the copy was one of
          2,056 bytes a window, in `compact_root_picker`; the assembly now
          has no `memcpy` in the root's or the node's functions). Checks
          on `b88700a`: fmt; clippy over all features at zero warnings;
          debug 541 and release 543 passed; the `tune` and `texel`
          libraries 293 each; parity without a mismatch. *Read*
          (`analysis/artifacts/d78-night/nps/`, `tools/nps_read.py`): four
          pext PGO builds an arm from clean trees, `42e3073` against
          `b88700a`, idle host. Step 1, two cycles: +0.89% (−0.06% to
          +1.85%), between; step 2, six cycles: **+0.95% (+0.37% to
          +1.54%), accepted**, no disturbed cycle. An accept below +1% on
          code that moved is recorded as possibly code layout (PROCESS).
        - **D.7.8.10 A finer kernel unit: the screen — `V`, NO_CHANGE 2026-11-20.**
          (A3, the audit's decision 5) The fitted wrapper hands the kernel
          0.50 to 0.82 of the HCE's value in whole units, and the margins
          were folded to that unit; the donor works about 3.5 times finer.
          Emulated without a code change: through the tune build's options
          every seed whose dimension is evaluation units is doubled, every
          per-unit coefficient and `wrap_divisor` halved (the fold's
          dimension table, `analysis/d741_fold_2026-11-18.md`, and the
          parity test's classes say which is which), so the kernel plays
          the same rules on a unit half the size. *Read, registered:* that
          arm against the unchanged one, the tune build on both sides, at
          100,000 nodes a move each (one root driver, no clock, so the
          time manager's slopes do not enter), 2,000 games; the bench of
          both beside it. Interval wholly above zero: an `I1` step folds
          the finer unit in before D.7.9. Otherwise `NO_CHANGE`.
          *NO_CHANGE 2026-11-20* (RAR-S133): 65 options from the tune
          build's defaults, classed by the fold's factors (52 doubled, 13
          halved and `wrap_divisor` halved; `optimism_scale` kept, being
          in the raw evaluation's units; `q_see_correction_scale`, no
          option, the one seed not reached), `bench 13` 3,417,466 / EBF
          2.311 against 3,320,355 / 2.317. 2,000 games at 100,000 nodes a
          move: nElo −4.9 ± 15.2, the interval [−20.1, +10.3] not wholly
          above zero. The kernel keeps its fitted unit.
        - **D.7.8.11 The root's remaining moves re-scored as each is searched — `R2`, then `I2`, NO_CHANGE 2026-11-20.**
          (the question after RAR-S132, added 2026-11-20 by the maintainer's
          decision) **The defect or opportunity.** The donor's root picker
          re-scores the noisy moves still to come each time it hands over a
          good noisy move, and the quiets still to come before each quiet
          it hands over, from the histories the searched moves' subtrees
          have just updated; a noisy move is classed good or bad when it is
          picked, on the bar of its current score and the good moves so far
          (Reckless `movepick.rs`, the `NODE::ROOT` branches, at
          `0756957`). Rarog's root scores its whole list once at each root
          search (D.7.8.3) and classes every noisy move then. Stockfish does
          neither; it keeps a history for the first plies. Move ordering
          never reads the evaluation, so the donor's NNUE enters only
          through the histories it trained. **Mechanism:** every root move
          after the first is searched in a null window and reduced by its
          place in the list, so the order decides which moves get depth; a
          refutation found under one root move raises the histories of the
          moves that share it, and the donor's root reads that at once.
          **Evidence it is active here:** the root's tie order alone moved
          `bench 13` by a third (D.7.8.3's first form); on position 305 the
          donor's re-scoring changed nothing to depth 6 and changed the
          depth-14 tree from 184 million nodes to 530,420 (D.7.8.8), so it
          acts, with no sign. **Competing explanations of any effect:** tie
          order (chaotic, D.7.8.8), and the noisy class moving with the
          score rather than the ordering itself. **Interactions:** the
          good-noisy bar reads the score and the count of good moves, so
          the class of a capture can change; the root's reductions read the
          move's place; the pool's lifted moves and a helper's rotation
          (Rarog's Lazy SMP) stay above the re-scored tiers, untouched;
          MultiPV's later lines and `searchmoves` skip a move outside the
          root's lines once it is handed over, the donor's rule, so it still
          counts toward the bar; the histories themselves are not changed.
          **Invariants and their tests:** every legal move once, the table
          move first; a narrowed root hands over exactly the moves it
          admits, in the whole list's order; on histories that do not
          change between picks the root's order is the staged picker's,
          ties included (re-scoring on the same tables moves nothing);
          after a history changes between picks, the next pick is the
          staged picker's on the new tables over the moves left; root-only
          code, the node's unchanged. **The cheapest test,** zero games:
          two counters in the `diag` build over `bench 13`, the root's picks
          from the noisy and quiet stages that a re-score precedes, and
          those whose next move the re-score changes. **Falsifier and stop
          rule (frozen 2026-11-20, before the counters ran):** a share
          below 1% closes the step `NO_CHANGE`, the code reverted, no
          games; otherwise the fingerprint moves and the step takes a
          registered 2,000-game read against D.7.8.10's state, AGENTS'
          case 2 (reverted below −13 nElo), with 200 games at four threads
          read for faults alone, as RAR-S132. **Prediction (frozen
          2026-11-20, before the counters ran):** the share 20%, 80% band
          5% to 45%; `bench 13` moves, by under 15% with P 0.7; the read's
          prediction is frozen in its registration.
          *NO_CHANGE 2026-11-20* (`analysis/d7811_root_rescore_2026-11-20.md`;
          recipe `analysis/arm_patches/1bbab9f-d7811-root-rescore.patch`):
          the donor's root built in Rarog's root list, the table move, the
          pool's moves and a helper's rotation above the re-scored tiers;
          the root's order tests passed unchanged, and two new tests (a
          quiet or a capture whose history rose is picked next) passed and
          failed with the re-scoring planted off. `bench 13` 3,369,926 /
          EBF 2.326 (+1.5%). The screen: 319 of 33,313 re-scores changed
          the next move, **0.96%**, under the frozen 1%: no games, the code
          reverted. Calibration: a miss in magnitude by a factor of twenty;
          the histories trained over earlier iterations and the quiet
          order's static terms dominate what one root move's subtree adds.
          Retry trigger: a change that makes the root's histories move
          faster between root moves (a root-specific history, or D.7.11's
          fit raising the quiet order's history weights), then this screen
          again on the recipe.
        - **D.7.8.12 The speed pass before the fit — `R2` card, `I1` items, `V` reads, DONE 2026-11-20.**
          (maintainer decision 2026-11-20: as much effective speed and NPS
          as can be had before D.7.9 and the fit, Reckless as the
          reference) *The profile* (`analysis/d7812_profile_2026-11-20.md`,
          zero games): the shipped pext PGO build at 415 ns a node on
          `bench 13`, game play the same shape. Outside the evaluation the
          kernel is about as fast a node as Reckless; the clear gap is the
          table, 48 ns a node against Reckless's 23, 8.5% of all samples on
          the cluster's first load. Reckless prefetches the child's cluster
          before it makes the move, from the key the move will produce;
          Rarog prefetches only after, and `Board::key_after_hint` goes
          unused. **Mechanism:** memory latency the search does not hide,
          and work at every node that no node needs. **Interactions:** none
          with play for items 1 to 6, which keep `bench 13` exact; a
          prefetch competes for cache only, and code layout moves under PGO
          (PROCESS: an accept below +1% on code that moved is recorded as
          possibly layout). **Invariants:** the fingerprint exact at every
          commit; AGENTS' per-node rule (no allocation, no clone, no
          `memcpy`; the node's assembly checked); no new `unsafe` site (the
          floor is 17); the board's contract and perft. *The items,* in
          order, each its own engine commit:
          1. The table prefetch before the move, as Reckless: the hint's key
             prefetched before the push and the make in the node, in
             quiescence and in ProbCut, the prefetch after the make kept.
             Ceiling about 8.5%.
          2. The correction slots and the pawn table's entry prefetched once
             the make has set their keys, so they arrive while the table is
             probed (Rarog's own). Ceiling about 3.4%.
          3. `SearchedMoves` written only as moves are pushed, through one
             fixed-capacity list type shared with the other lists, so the
             `unsafe` sites do not grow. Ceiling about 0.7%.
          4. The board's undo stack as a fixed array, no `Vec` capacity
             checks in make and unmake. Ceiling about 0.65%. The stack
             alone; the board's forms stay D.12's.
          5. The evaluation's hot paths (`eval_piece_activity`,
             `eval_threats`, `AttackMaps::fill` and their slider lookups,
             21 to 23% of samples): drilled to the line, behaviour-neutral
             rewrites only, the evaluation's tests and the `texel` trace
             gate exact.
          6. Quiet scoring (4.4%, its multiply 2.6% more, waiting on the
             continuation histories): examined for a neutral form; none is
             presumed, as Reckless pays the same loads.
          Not in the pass: a different selection step or tie order, which
          moves the tree (D.7.8.3's tie order moved it by a third) and
          would need its own game read; the release profile, already
          Reckless's; the PGO training load, which the game profile
          matches. *Read and stop rule (frozen 2026-11-20):* each item by
          PROCESS's pooled NPS method against the commit before it, four
          pext PGO builds an arm, on an idle host: kept on accept; on
          *between*, kept only when it removes code or work without adding
          any, otherwise reverted; reverted on reject, its reason recorded.
          After the last, one pooled read of the whole pass against
          `63fa0e2`'s engine, and `bench 13` NPS beside it. No games: no
          item moves play. *Prediction (frozen 2026-11-20, before any
          item):* item 1 +3% (80% band +1 to +6), item 2 +0.8% (0 to +2),
          item 3 +0.4% (0 to +1), item 4 +0.3% (−0.2 to +0.8), item 5 +1%
          (0 to +3), item 6 0%; the pass as a whole +5% (80% band +2.5 to
          +9). At about 2 Elo a percent at `3+0.03` (three measurements)
          that is worth about 10 Elo, a conversion this step does not
          measure.
          *Done 2026-11-20* (`analysis/d7812_profile_2026-11-20.md`,
          *Result*; reads in `analysis/artifacts/d7812/nps/`). `bench 13`
          3,320,355 / EBF 2.317 at every commit; each item's checks fmt,
          clippy over all features at zero warnings, debug 541 and release
          543 passed. By PROCESS's pooled read against the commit before:
          item 1 (`1ff6522`, the hint before the make) **+1.09%**
          (+0.52 to +1.67), accepted; item 2 (`8f3c0f0`, the correction
          and pawn entries) +0.14% (−0.16 to +0.44), `NO_CHANGE`, reverted
          (`2aad651`), after two reads discarded as disturbed (four and one
          disturbed cycles, a remote-desktop session opened and closed
          during them); item 3 (`80cef6f`, one list type) +0.31% (+0.04 to
          +0.57), `NO_CHANGE` at step 2, kept by the frozen clause (30
          lines and the per-node writes removed, nothing added); item 4
          not built (the undo stack is a `Vec` because a game's history is
          unbounded, and a fixed array would drop repetition evidence);
          item 5 none (the evaluation's 23% is spread over about twenty
          terms, none over 0.5% on a line); item 6 none (the waiting is on
          the continuation histories' loads, the donor's own). Added after
          item 1's read, under the same rule, as item 1b: a profile of
          items 1 and 3 still put 5.5% of samples on the cluster's first
          load, so the hint moved up to where the move is picked, before
          the pruning rules (`0690de9`), **+1.12%** (+0.93 to +1.31),
          accepted. **The pass, `0690de9` against `b88700a`: +2.71%
          (+2.11 to +3.31), accepted at step 1.** The assembly of the node,
          quiescence, the picker, make and unmake and the evaluation holds
          no `memcpy` or `memset`. Calibration: predicted +5% (80% band
          +2.5 to +9), read +2.71%, at the band's low end; the table
          prefetch paid +2.2% against a predicted +3%, and the second
          prefetch nothing, which the profile's line share did not
          foresee: once the table's line is hidden, the correction and
          pawn loads were not on the critical path.
    - **D.7.9 Clock use on the new kernel — `R2`, DONE 2026-11-20.**
      Moved here from D.6.28 on 2026-11-19 (maintainer decision): it is the
      largest known lever beside the kernel (the donor's search led by 22.5
      at equal clock and 3.5 at equal work, RAR-O08 and RAR-O09), and a
      time manager changed after the fit would move the fit's baseline. Its
      constants are the donors' seeds, never fitted on Rarog (corrected
      2026-11-20: D.1.5 kept the clock pieces at their seeds, RAR-R28, and
      RAR-R15's block 1 was never baked), and read the kernel's score per
      unit (`tm_fall_slope`, `tm_fall_prev_slope`), which the fitted wrapper
      shrank (the audit's A3).
      1. *Screen, zero games,* on the repaired kernel's own games (D.7.8's
         reads): the unspent share of the budget by move number and game
         length, the clock at the game's end, and the share of undecided
         moves played under 500 ms, beside the figures below.
      2. *The form candidate is the low-clock horizon,* as the text below
         concluded: `TmHorizonPerSec` confined to where the forfeits were.
         Its card states the registered read against a field, the forfeit
         count its invariant and a `10+0.1` harm read beside it, and its
         cost in hours; the maintainer starts it. A form that is kept
         lands before D.7.10.
      3. *The constants go to the fit:* the fourteen `tm_*` coordinates of
         `SearchParams` join D.7.11.1's surface, a forfeit in the tune a
         fault that stops it.
      *D.6.28's text, as written:* (found
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
      *Screen and decision 2026-11-20* (`analysis/d79_clock_2026-11-20.md`, zero games; its
      instrument reproduces RAR-M87's end clocks exactly):
      1. On D.7.8's three reads the kernel spends 85.1–85.9% of its budget
         and ends on about 620 ms of clock, as the classic did head to head
         (84.3–84.5%, about 630), but it spends less early. In RAR-S126's
         games it spends 0.622 of the base in its first twenty moves against
         the classic's 0.696, and 83.8 ms a move against 96.0 above a second
         of clock; the time is still on the clock at move 40 (about 900 to
         1,040 ms against 600 to 660). Undecided moves under 500 ms fell to
         4.5–6.8% (the classic's 11.6–13.9% head to head, 18.8% against the
         field). On the kernel `h=`'s tail is search time after the last
         completed iteration under 300 ms of clock, not a hold: the journal's
         hold is the classic's.
      2. **The horizon's field read is not registered** (`NO_CHANGE`, low
         expected value). RAR-R17's direct read is −0.9 ± 8.0 Elo. The
         horizon acts only at `3+0.03`: at `10+0.1`, `60+0.6` and CCRL's
         `2'+1"` the optimum's cap binds first, and 40/15 sends
         `movestogo`. It is Stockfish's current rule, its domain halved on
         the kernel, and a ±5 Elo read costs about 8 hours. Retry trigger:
         a direct read of the horizon wholly below zero, or a rated control
         with an increment under about 50 ms.
      3. Thirteen `tm_*` coordinates join D.7.11.1's surface.
         `TmHorizonPerSec` stays at 50 and out of it: it is the forfeit
         guard, it acts only at `3+0.03`, and the fit stops on a forfeit.
         Their activation read is the clock's (`tm_activation.py`), since
         none changes a bench decision. *Superseded the same day by
         D.7.9.1:* the fit takes the clock coordinates the ported time
         manager leaves; the activation rule stands.
      Calibration: predictions 1–3 and 7 held; 4 and 5 missed low and 6 in
      sign, because the kernel keeps clock longer, which they did not
      foresee. The donor's whole time manager, which a trigger here would
      have opened after the fit, is D.7.9.1 by maintainer decision the same
      day.
        - **D.7.9.1 Reckless's time manager, whole — `R2` card, `I2` port, `V` reads, DONE 2026-11-20.**
          (maintainer decision 2026-11-20: one donor's time manager, not a
          hybrid, as the search showed; Reckless's.) Rarog's clock is a
          hybrid never fitted on Rarog. It has Stockfish's budget in an
          older form, without Stockfish's per-control scale, its cut when
          behind on the clock or `increaseDepth`. It has Stockfish-style
          falling, instability and effort terms, Reckless's stability
          factor, and Rarog's own fail-high stop. On the kernel's games 30%
          of its time is spent past the optimum, and 6.1% of its moves take
          three times the optimum or more (3.9–4.0% above a second of
          clock), past D.1's 3% trigger for `increaseDepth`
          (`analysis/artifacts/d79-clock-2026-11-20/over_optimum.py`).
          *Why Reckless:* its budget and all five factors were tuned
          together with its search parameters, in an "LTC" SPSA of 160,000
          games (Reckless pull request 956, 2026-05-01, verified by SPRTs at
          `8+0.08` and `40+0.4`) and a six-thread one of 200,000 (pull
          request 1072, 2026-06-06, verified at `40+0.4`). The pull requests
          do not state the tunes' own control (corrected 2026-11-20). Its
          search is the kernel's (D.7.4's parity), and every control it was
          checked at is longer than `3+0.03`. Stockfish's time manager is the more
          mature and the more widely tested, but it was tuned with
          Stockfish's search, depth rate and score scale; Stockfish's search
          on Rarog's evaluation trailed Reckless's by 88.7 ± 7.7 (RAR-O08,
          match 1).
          *The card, at zero games:*
          1. Every piece of `time.rs` and of the root driver's clock code is
             classed as either a guard or shape.
             - *Guards* are kept with their tests: the hard reserve under
               Move Overhead, the four-thread reserve, the proved-mate stop,
               the first-iteration guard, the tablebase extension's box, the
               single-move shortcut, and the `movetime`, `movestogo`, ponder,
               `nodes`, `depth` and `infinite` semantics.
             - *Shape* is replaced by the donor's: the budget and its
               horizon, the five factors, and the thread vote at 65% with
               withdrawal.
             - Whether Rarog's fail-high stop is a guard is decided here,
               from its measured cases on the kernel (RAR-R13's won endings
               running to the maximum, D.6.20's test).
          2. The donor's constants that read a score, the score trend's
             slope and the evaluation-stability window of 12, are in its
             network's units. They are folded into the kernel's units by the
             measured scale, as D.7.4.1 folded the search's.
          3. The donor's constants were tuned in a long-control session, and the fit runs at
             D.7.10's control. The card says which clock coordinates the fit
             takes, so that a `3+0.03` fit cannot move the long-control
             allocation unread.
          4. The reads, registered before any game: the kernel with the port
             against the kernel at `0690de9`, 2,000 games each at `3+0.03`
             and at `10+0.1`. The forfeit count is the invariant, with the
             clock profile (`clock_screen.py`) beside the reads. The port is
             kept unless an interval is wholly below zero, the rule D.7.8's
             reads used: a diagnostic inside D.7's one cluster, which D.7.13
             gates.
          It lands before D.7.10, which then reads the kernel with it. Not
          carried: Stockfish's per-control scale, its cut when behind and
          `increaseDepth`. Choosing one donor's form closes those gaps by not
          combining forms.
          *The card answered and the port built, 2026-11-20* (engine
          `38e947c`, tooling `c86bb60`; RAR-R34):
          1. *Guards kept with their tests:* the reserve under Move Overhead
             and its four-thread extra, the proved-mate stop, the
             first-iteration guard, the tablebase box, the single-move
             shortcut, and the `movetime`, `nodes`, `depth`, ponder and
             `infinite` semantics. The fail-high stop is a guard too: on the
             new clock it ends 10 of 300 searches at `3000+30`, against the
             old clock's 11 (D.7.8.5). The thread vote stays Rarog's strict
             majority, a measured multi-thread contract (two threads
             exempted cost −15.85 Elo) that the one-thread reads cannot
             test; the donor's vote is tried again on the fitted clock
             in D.14. *Shape replaced:* the budget and its horizon, and the
             five factors.
          2. *Units:* the score trend's slope is the donor's 0.0480 folded to
             0.1189 a unit. The score-stability count is the root window's
             own, already folded.
          3. *Fit coordinates:* the five factors' fifteen constants
             (`TmNodesBase` to `TmChangesScale`). The budget is constants in
             `time.rs` and is not fitted.
          4. *Checks:*
             - `bench 13` 3,320,355 / EBF 2.317, unchanged; tests in debug,
               release, `texel` and `tune`, clippy and `fmt` all clean.
             - The clock activation read: every factor moves and none is
               inert. The score trend is at a clamp on 98% of iterations;
               in a fresh game it reads the fall from zero.
             - The three reads are registered and dry-run (RAR-R34): 2,000
               games at `3+0.03`, 2,000 at `10+0.1`, and 200 at four
               threads for faults.
          *Read 2026-11-20 (RAR-R34): kept.*
          - **+30.0 ± 8.8 Elo** (nElo +51.9 ± 15.2) at `3+0.03` and
            **+18.6 ± 8.2** (nElo +34.6 ± 15.2) at `10+0.1`, against the
            hybrid; no fault or time loss in either, nor in 200 games at
            four threads.
          - The port spends 0.75 of the base in the first twenty moves
            against the hybrid's 0.60, at both controls, and plays the late
            game on a lower floor (a median of 226 against 436 ms at
            `3+0.03`).
          - Predicted +4 and +3: a miss in magnitude. The hybrid's early
            under-spend cost far more than RAR-R17's null horizon read
            suggested.
          - D.7.8.5's trigger is re-read and not fired: no search stopped
            inside an iteration at any of its three clocks.
          - Evidence layer: fixed reads inside D.7's cluster; D.7.13 gates
            it. The fifteen coordinates join D.7.11.1, and D.7.10 reads
            the kernel with the port. *Superseded 2026-11-20 by D.7.11's
            research:* the fifteen are held at the donor's values and
            are not fitted at `3+0.03`.
    - **D.7.10 The pre-fit read: the repaired kernel against the classic — `V`, DONE 2026-11-20.**
      One registration, two fixed matches of 2,000 games, the repaired
      kernel (pext, PGO) against `d7classic`: `3+0.03` and `10+0.1`. A
      diagnostic, not a gate (the gate is D.7.13). Its uses, frozen here:
      1. the first direct measurement of the new kernel against the old,
         which until now is a sum of two reads (about +50 ± 13);
      2. *the fit's time control.* The first fit halved the tree at a depth
         and was read at `3+0.03` only. If the `10+0.1` Elo is at least
         half the `3+0.03` Elo, the fit runs at `3+0.03`, rule 7c's shape
         unchanged. If it is less, the fit runs at `10+0.1` with a ceiling
         of 3,000 iterations, its registration giving the hours;
      3. a `3+0.03` interval wholly below zero stops D.7 for the
         maintainer before any fit.
      *Registered 2026-11-20 as RAR-S134*, before any game:
      - `d710kern` (RAR-R34's candidate, `38e947c`) against `d7classic`
        (`29f32fb`), both pext PGO builds. The kernel arm carries D.7.9.1's
        time manager and the classic arm its hybrid clock, so use 1
        measures the two together.
      - 2,000 games at each control, seeds 20261129 and 20261130, both
        dry-run.
      - One command runs both (`run_s134.ps1`), about 1 hour 45 minutes.
      - Predicted: +65 Elo at `3+0.03` and +45 at `10+0.1`, so the fit
        runs at `3+0.03` (0.70).
      *Read 2026-11-20* (RAR-S134):
      - +63.8 ± 9.1 Elo at `3+0.03` and +65.0 ± 8.3 at `10+0.1`, 2,000
        games each, no fault or time loss.
      - Use 2: the `10+0.1` Elo is 1.02 times the `3+0.03` Elo, so D.7.11
        runs at `3+0.03`. Use 3 does not fire.
      - At `3+0.03` 31% of the kernel's sides fall under 100 ms at some
        move (the classic's none), without a time loss; D.7.11.2's
        `--max-time-losses 0` stops a fit that forfeits. *Withdrawn
        2026-11-20 by D.7.11's research:* that limit ends a tune for
        good, so the fit runs on the runner's own allowance.
      - Predicted +45 at `10+0.1`: a miss in magnitude; the kernel's gain
        holds whole at the longer control.
    - **D.7.11 The finishing fit — `V`, DONE 2026-11-21.**
      *Result:* RAR-S135's tune stopped quiet at 2,000 iterations, its
      centres read +7.8 ± 9.1 Elo against the start (kept, no gain
      shown) and are baked (`16f93cb`); fingerprint 3,390,842 / EBF
      2.317.
      One rule 7c run on the repaired kernel from the baked centres, at
      `3+0.03` (D.7.10). Researched 2026-11-20 at zero games
      (`analysis/d711_fit_2026-11-20.md`): that record holds the evidence,
      the exact surface and the frozen prediction, and the sub-steps below
      run in order, one at a time. The curvature sweep is not repeated: the
      first fit stopped live (`sum z² − n` at +3.4 SD over its last 500
      iterations), which is the evidence rule 4 asks for.
      *Decided by the research, replacing this step's earlier text:*
      1. *The clock is not fitted here.* The fifteen soft-limit coordinates
         (`TmNodesBase` to `TmChangesScale`) stay at the donor's values.
         - Three are inert at `3+0.03`: no iteration of 4,200 sits at
           `TmNodesFloor`, `TmPvFloor` or `TmEvalFloor`, and `TmPvFloor`
           binds on 2.1% at `10+0.1`.
         - Six are one scale, the four bases and the trend's two clamps.
         - The scale's gradient belongs to the control: 31% of the
           kernel's sides fall under 100 ms at `3+0.03` and none at
           `10+0.1`.
         - Rarog's one clock fit (RAR-R15) found a gradient on the scale
           alone.
         - *Retry trigger:* D.7.13's `10+0.1` companion below its
           `3+0.03` reading by more than their combined intervals, or a
           rated control's games showing a factor at one end on more than
           90% of iterations. The fit is then its own registration at
           `10+0.1` or longer; until then H.3 holds clock refinements.
      2. *The surface is 225 coordinates:* 218 of the kernel's 242 seed
         options and seven new ones (D.7.11.1), each by the engine's
         range rule, step range/16, the full two steps (the least room
         is 5.0 steps). Revised twice the same day, before any reading:
         the kernel's exchange values joined when the maintainer asked
         what was still unfitted (the first scan had missed
         `compact/mod.rs`), and the units below left.
         - *Out, a seed a fit moves in whole units (16):*
           `singular_depth`, `singular_tt_depth`, `singular_exact_div`,
           `probcut_depth_base`, `tt_cut_typed_depth`,
           `tt_bonus_parent_moves`, `eval_bonus_depth`, `hp_depth`,
           `ldse_depth`, `fp_depth`, `bnfp_depth`, `nmp_verify_depth`,
           `q_move_limit`, `negative_extension`, `pick_late_good` and
           `see_noisy_square`. The first fit's gradients on three of them
           are banked in the bake (`singular_depth` 5 to 7,
           `tt_cut_typed_depth` and `hp_depth` 5 to 6); the next unit on
           those is what the exclusion gives up.
         - *Out, inactive on `bench 13` and on `bench 17` (8):*
           `fds_singular`, `fds_singular_cap`, `fds_singular_offset`,
           `nmp_min_16`, `rfp_min_16`, `pick_check_base`,
           `tt_cut_fifty`, `window_stability_cap`. `pick_check_piece`
           is inactive at 13 and active at 17, and is in. Whether any
           of the eight is removed is D.8.21's question, after D.7.14.
         - *Out, by construction (8):* the two thread spreads,
           `pick_escape[0]` and `[5]`, and four units. A history
           table's limit scales with its amounts and its consumers'
           weights, and the exchange values with every exchange margin
           and the wrapper's bases, without changing a decision; the
           rest of each family is in the surface. So the quiet, noisy
           and continuation limits and the pawn's exchange value are
           held. The pawn table's limit is in: that table is written
           with the quiet table's amounts, so its limit is free.
         - *Not coordinates:* `LazyMargin` (the frozen evaluation's own
           constant, rule 2), `AspMaxFails` and `TmFailHighTarget`
           (guards).
      3. *Rule 7c's shape is kept whole:* 30 games an iteration on 15
         slots, two steps, `r_end` 0.00155, ceiling 6,000, checkpoints at
         2,000 and every 1,000, window 1,000. About 13.5 hours to the
         first checkpoint, 6.8 for each further 1,000 iterations and 40.5
         to the ceiling, restated from the pilot's rate. Sixty games an
         iteration would play 10% more games an hour (the slots are busy
         79.3% of an iteration at 30): a candidate amendment to rule 7c
         before F.6's refit, not this run's.
      4. *No fixed fault limit.* On the pinned runner a tune that passes
         `--max-time-losses` ends on an invalid iteration and cannot be
         resumed. The new clock's lowest forfeit slack is 54 ms, and the
         first fit's journal holds 19 harness stalls longer. The tune
         runs on the runner's own allowance; a time loss reads FAULT at
         the next checkpoint and stops the tune there for the maintainer
         (rule 7c). A time loss whose `failed-games` table shows the
         losing search held longer than its slack is the host's and is
         accepted; one that overran its own deadline with no hold returns
         D.7 to the clock.
      5. *The tune never pauses for a read.* One interrupt in its
         terminal pauses it at a whole iteration and the same command
         resumes it, seed, schedule and checkpoints unchanged (PROCESS,
         *SPSA go/no-go*, item 10). A read plays the tune build on both
         sides, the rounded centres at a completed iteration against the
         start, with the tune paused or stopped, and changes nothing in
         the run. No forecast is made from a read, two reads never rank
         two checkpoints, and theta is never chosen by one.
      6. *What stays unfitted after this fit* is tabled in the record
         and noted at each owner: the clock at H.3, the thread spreads
         at G.1, `LazyMargin` at the next evaluation change (E.4, then
         F.4.1) through PROCESS's margin block, the whole-unit seeds at
         F.6, the depth thresholds at G.3, and the eight seeds no bench
         reaches at D.8.21. GUIDE's obligations hold one row for them.
      The step closes when D.7.11.8 bakes theta, or D.7.11.7's rule keeps
      the start.
        - **D.7.11.1 The tune build — `I1`, DONE 2026-11-20.**
          *Inputs:* the record's *In: seven new options*. Engine only.
          - `Seeds` gains `pawn_history_max` (8,192 in both tables), read
            by `history.rs` in place of its literal, and
            `singular_pv_per_ply_1024`, seeded at each table's
            `singular_per_ply_1024`, for the stored-PV term of
            `singular_margin`. The quiet, noisy and continuation limits
            stay literals.
          - The kernel's five exchange values (109, 403, 435, 679, 1,242:
            `KERNEL_SEE_VALUES` in `compact/mod.rs`, the donor's fitted
            piece values) become seeds, `exchange_value[0]` to `[4]`,
            read once a search and never rebuilt at a node; `[0]`, the
            pawn, joins `tune.rs`'s exclusions as their unit. In the same
            commit the test "a captured piece at least a knight"
            compares piece types, not values, so a perturbed bishop
            cannot flip it.
          - `q_see_correction_scale` leaves `tune.rs`'s exclusions: 254
            seed fields, 249 options.
          - Nothing else: no clock change, no range rule, no other seed.
          *Checks:* `bench 13` 3,320,355 / EBF 2.317 on the default build
          and on the tune build at its defaults; a test that each new
          option reaches its rule; the pawn table inside an `i16` at its
          largest limit (16,384); per-node code reads the seeds by
          reference as now; the parity test keeps `Seeds::DONOR`; debug
          and release tests, `fmt --check`, clippy with all features and
          targets, the `texel` and `tune` library suites.
          *Done:* its engine commit, every fingerprint reproduced.
          *Result (2026-11-20, `6843b16`):* 254 seed fields, 249 tuning
          options; `bench 13` 3,320,355 / EBF 2.317 on the default build
          and on the tune build at its defaults. The exchange values are
          built once a search in the tune build, at compile time in the
          default one; the pawn table's limit is read from the seeds once
          a search where a build can change them, a constant otherwise.
          Debug and release tests, `fmt --check`, clippy with all features
          and targets at zero warnings, the `texel` and `tune` library
          suites pass; the parity test on D.7.4's trace passes with the
          pawn limit and the five exchange values named among the seeds no
          record detects (no rule's arithmetic reads them), and it detects
          `singular_pv_per_ply_1024`.
        - **D.7.11.2 The surface files and the activation read — `V`, DONE 2026-11-20.**
          *Inputs:* D.7.11.1's commit, a clean tree.
          - `./tools/build_test.ps1 -Suffix d711fit -Tune`; its sidecar's
            bench 3,320,355, its SHA-256 recorded.
          - The activation read by RAR-S128's rule over all 249 options,
            with its guards (the default bench, a positive control, every
            `setoption` accepted). A seed that reproduces `bench 13` on
            both sides is read again at `bench 17` before it is dropped.
            Expected: eight inactive at both depths, `pick_check_piece`
            active at 17, the seven new active. A new option that reads
            inactive is dropped and named; any other difference from the
            record is reported before going on.
          - `tools/spsa_configs/config_d711fit.json` (planned), generated
            from the binary's own listing, never typed: each coordinate's default,
            the listing's range, `step` range/16 and at least 1;
            `fixed_d711fit.json` (Hash 64, Threads 1, MultiPV 1);
            `d711pilot`, the same surface. Converted `--rule7c --ceiling
            6000` and `--ceiling 64`, the room report empty.
          - `d76fit`, `d76pilot` and `d14block1` were listed in
            `historical.txt` on 2026-11-20: the first two were baked
            (D.7.6.2) and the third's options went with the old clock
            (D.7.9.1). That took the audit from 797 findings to clean;
            `d711fit` and its pilot are then the only live groups.
          *Checks:* `./tools/audit_spsa_coverage.ps1 -TuneEngine <the
          build>` ends `RESULT: clean`; `--check` passes on both groups;
          the surface's count equals the read's.
          *Done:* the files committed with the read's result.
          *Result (2026-11-20, files `66a1f14`):* the tune build
          `rarog-d711fit-tune.exe` from the clean tree at `3233da2`,
          SHA-256 `e88123b2…bddfc25`, sidecar bench 3,320,355, 249 seed
          options. The activation read
          (`analysis/artifacts/d711-fit-2026-11-20/activation3.py`) is the
          record's exactly: guards passed (default `bench 13` 3,320,355 and
          `bench 17` 20,004,610, `KLmrLog` plus 3,118,411, every `setoption`
          accepted, no timeout); nine inactive on `bench 13`, of which
          `KPickCheckPiece` moves `bench 17` and the eight named above
          move neither; the seven new options active on `bench 13`. The
          surface, generated by `gen_config.py` from the binary's listing
          and the read, is **225 coordinates**, the record's appendix name
          for name with the same seeds and ranges. The room report is empty
          at both ceilings, `--check` passes on both groups, and the audit
          ends `RESULT: clean`.
        - **D.7.11.3 The registration — `M`, DONE 2026-11-20.**
          RAR-S135, `docs/experiments/RAR-S135.md` (planned) with its
          index row, before any game, the pilot's included. It states: the hypothesis;
          the binary, its hash and fingerprint; the surface as the files
          hold it, with what is out and why; the shape and the hours; the
          stop by rule 7c; the fault rule of item 4; the pilot and its
          pass criteria (D.7.11.5); the read at the stop with its keep
          rule and the optional interim read (D.7.11.7); the seeds; and
          the record's prediction, copied as frozen.
          *Done:* committed, with `tools/spsa_configs/README.md`'s entry.
          *Result (2026-11-20):* `docs/experiments/RAR-S135.md`, before any
          game, with its index row and the README's entry; seeds
          2026113501 (pilot), 2026113502 (tune), 2026113503 (the read at
          the stop) and 2026113504 (an interim read), none used before.
        - **D.7.11.4 The drivers and their dry runs — `V`, DONE 2026-11-20.**
          In `analysis/artifacts/d711-fit-2026-11-20/`, each run from the
          machine's own terminal, showing the harness's live output:
          - the pilot's: `colosseum.ps1 -Mode spsa -ConfigGroup d711pilot
            -Ceiling 64 -AllowShortRun -FirstCheckpoint 64 -Window 64`;
          - the tune's: `-ConfigGroup d711fit -Ceiling 6000`, no fault
            limit; it starts or resumes, and retries only while the
            harness refuses a busy host before any game;
          - a read's: `-At <K>` and `-Games <N>`; the centres from `spsa
            history` with nothing written into the run directory, the
            count checked against the surface, `colosseum.ps1 -Mode match
            -CategoricalTuneBuild`, refused while a runner is alive.
          *Checks:* each driver's `-DryRun` path executed once; the
          wrapper's dry run verifies the surface at the ceiling and lists
          no rail warning.
          *Done:* the pilot's one command handed to the maintainer.
          *Result (2026-11-20):* `run_pilot.ps1`, `run_tune.ps1` and
          `run_read.ps1`, their SHA-256 in RAR-S135. Each dry path ran once
          through the wrapper's dry run with every guard passed: the
          surface verified at N = 64 and N = 6,000 with no rail warning, no
          fault limit passed, and the read's extraction, run on RAR-S128's
          stopped run, equal to that run's own theta line for line, its
          directory unchanged. The pilot's command handed over.
        - **D.7.11.5 The pilot — `V`, the maintainer's, DONE 2026-11-20.**
          64 iterations of 30 games, about 26 minutes, not PROCESS's
          default 128: its uses are the run proved end to end on the new
          build, the rate, the spread of an iteration's score at the first
          perturbations and the pause, and each is settled inside 64.
          The maintainer interrupts it once (Ctrl+C) and runs the command
          again.
          *Pass:* every option accepted; no first arm clamped; no fault;
          the reader's move check on the resumed run under 1e-6; the
          rate recorded and the registration's hours restated from it
          before the tune's first game.
          Its theta is never used. PROCESS's re-audit follows it.
          *Done:* the maintainer has the one command that starts the tune.
          *Result (2026-11-20, RAR-S135):* 64 iterations at 153.7 an
          hour, so 13.0 hours to the first checkpoint and 39.0 to the
          ceiling, restated before the tune's first game; every option
          accepted, no arm clamped, the re-audit without a rail. One time
          loss, the host's by the registered rule (a stall held all
          fifteen slots; the maintainer reports programs started on the
          host). The run was not interrupted, so the pause is tested in
          the tune's opening minutes by an amendment (maintainer decision
          the same day). The tune's command handed over.
        - **D.7.11.6 The run — `V`, the maintainer's, DONE 2026-11-21.**
          It stops only by its rules: quiet, a rail, a fault, the ceiling,
          or the maintainer's `STOP` at a checkpoint. Nothing waits for a
          read. While it runs the host is the tune's: file reads only.
          *Result (2026-11-21, RAR-S135):* quiet at its first checkpoint,
          2,000 iterations and 60,000 games, no fault: the largest |z|
          3.09 against the line of 3.69, no rail, no clamped arm. Theta is
          `theta-2000.txt`; the moves are small (median 0.26 of a step,
          largest 1.33). The pause tested in its opening minutes passed:
          the move check 0.0 on the paused run and 1.2e-12 on the resumed
          one.
        - **D.7.11.7 The read at the stop — `V`, DONE 2026-11-21.**
          Theta (`theta-<K>.toml`, the checkpoint the maintainer names)
          against the start, the tune build on both sides, 2,000 games at
          `3+0.03`, about 27 minutes, registered in RAR-S135.
          - *Its use:* the only measurement of this fit's own
            contribution. D.7.13's gate would pass on the pre-fit kernel
            (+63.8 Elo against the classic, about 600 pairs to H1), so a
            fit that cost 10 Elo would pass with it.
          - *The rule:* theta is baked unless the interval lies wholly
            below zero, D.7.8's keep rule. At 2,000 games that catches a
            true −15 Elo nine times in ten and a true −10 a little over
            half: "baked" means no harm beyond about −15 shown, not a
            gain.
          - *An interim read* is optional and the maintainer's: 1,000
            games, the tune paused or already stopped. Wholly below zero
            is reported, and the maintainer may then place `STOP`.
          *Result (2026-11-21, RAR-S135):* `theta-2000` against the
          start **+7.8 ± 9.1 Elo** (nElo +13.1 ± 15.2) over 2,000 games, no
          fault: not wholly below zero, so theta is baked; no gain shown.
          The quiet stop checked afterwards: no gradient persists across
          the tune's two windows (their z correlate at −0.03).
        - **D.7.11.8 The theta baked — `I1`, DONE 2026-11-21.**
          As D.7.6.2 baked the first: `tools/spsa_theta_audit.py <run>
          <config> --at <K>` first, each named coordinate given its line
          in the record; the values taken from the theta file by script;
          `Seeds::FITTED` and the tests that pin fitted numbers updated.
          *Checks:* the tune binary's `bench 13` with theta set, the tune
          build at its defaults and the default build are one count, the
          new fingerprint, recorded in GUIDE's checkpoint; debug and
          release tests, `fmt`, clippy, the `texel` and `tune` suites;
          `d711fit` and `d711pilot` listed in `historical.txt`.
          *Held (2026-11-21, RAR-S135):* the bake is prepared and its
          pinned numbers updated, but two behaviour tests fail on the
          baked seeds. KQK's mate in 3 first shows at depth 20, not 8,
          and KRK's mate in 2 at 6, not 5; no single family causes it.
          The warm-table test's node comparison flips. *Unblocks:* the
          maintainer's decision on those two tests.
          *Result (2026-11-21, RAR-S135):* the maintainer decided that
          the tests measure their depths on the seeds and that the
          sensitivity is investigated in D.6.32, after D.7. Three test
          commits first, each passing on the seeds before and after
          (`df2a6c1`, `da0665b`, `9ed0ca2`: the mate depths, the
          warm table read directly, the stored-win warm-up at 15), then
          the bake (`16f93cb`): 204 values, every suite passing, and
          `bench 13` **3,390,842 / EBF 2.317** on the tune binary with
          theta set, the tune build at its defaults and the default
          build.
    - **D.7.12 The reported score calibrated — `I1`, DONE 2026-11-21.**
      (A3; `analysis/d7_audit_2026-11-19.md`, *The score scale*) `score cp`
      has never been calibrated: the classic kernel won half its games at
      about +120 with nearly all material on the board and at +244 in
      endings, and the fit moved the kernel's scale again. D.3.9 closed
      this card `NO_CHANGE` on 2026-11-04 and moved the fit to F.6, because
      a network would replace the evaluation. It is reopened here for two
      reasons that are new: the fitted wrapper hands the kernel 0.50 to
      0.82 of the HCE, so the next release would print lower scores than
      2.7.0 for the same position, and the network is far (maintainer,
      2026-11-19). F.6 refits the same model per network.
      - *The form:* an ordinary score leaves as `100·s / a(m)`, `a` a
        function of the material on the board fitted on the fitted
        kernel's own games so that +100 is an even chance of winning
        (Stockfish's and the donor's convention). Mate and tablebase
        scores, and the band a downgraded mate reads in, pass unchanged in
        meaning: the map is monotone, the formatter stays
        `uci::info::format_score`, and an ordinary score still stops at
        `cp 19872`, below every tablebase win (D.3.9).
      - *The contract:* output only. No search or time decision reads a
        calibrated score; a test holds `bench 13`'s node count and every
        best move equal with the calibration replaced by the identity.
      - *The tool:* `tools/wdl_fit.py` (planned) fits `a(m)` from a
        Colosseum PGN and writes the constants; it runs on theta's side
        of D.7.11.7's read at the stop, or on a 2,000-game self-play
        match of the baked kernel, and again after any later fit. Not
        on the tune's own games: each arm there plays a perturbed
        wrapper, so its scores sit on another scale (corrected
        2026-11-20).
      - `CHANGELOG.md` gets the user's sentence; the tests that pin a
        printed score follow; Phase F reads its note (F.6).
      - *Added 2026-11-21* (maintainer decision): the same model's
        win, draw and loss rates on `info` lines behind
        `UCI_ShowWDL`, Stockfish's option and placement; Reckless has
        no such output.
      *Result (2026-11-21, `analysis/d712_score_2026-11-21.md`):*
      `tools/wdl_fit.py` (`78d4681`) fitted on the fitted side of
      RAR-S135's read at the stop, 97,741 positions: the internal
      score of an even chance is 83 to 84 with most material on the
      board and 136 at material 20; the even chance falls at a
      printed +100. `61c9c51` prints through it and adds
      `UCI_ShowWDL`; every suite passes and `bench 13` stays
      3,390,842 / EBF 2.317.
    - **D.7.13 The gate: the kernel against the classic — `V`, DONE 2026-11-21.**
      The fitted, repaired kernel (pext, PGO) against `d7classic` by SPRT
      on the gain bracket `[0,3]` at `3+0.03` (maintainer decision
      2026-11-16: it must gain; repeated 2026-11-19), its cap from RAR-M10
      at D.7.10's reading, registered before any game. Beside it,
      registered with it and deciding nothing alone: 2,000 games at
      `10+0.1`, and 2,000 at four threads with the explicit placement list
      (RAR-M82). H1 opens D.7.14. H0, or the cap without H1, opens D.7.15.
      At D.7.10's reading the SPRT reaches H1 in about 600 pairs and
      cannot see the finishing fit; that is D.7.11.7's read. The
      `10+0.1` companion is also the clock's check (D.7.11, item 1).
      *Registered 2026-11-21* (`docs/experiments/RAR-S136.md`, before
      any game): `d713kern`, a pext PGO build of `eb84cce` (bench
      3,390,842), against `d7classic`; cap 20,000 pairs, which by
      RAR-M10 resolves a true gain of about +4.5 nElo or more; seeds
      2026113601 to 2026113603. One driver runs the gate and both
      reads, and its dry run passed; about 3 hours 35 minutes on an idle
      host, the maintainer's.
      *Result (2026-11-21, RAR-S136):* **H1** at 520 pairs (1,040
      games, +79.9 ± 12.3 Elo at the stop); beside it **+67.0 ± 8.1**
      at `10+0.1` and **+73.3 ± 8.1** at four threads, 2,000 games
      each, no fault or time loss in any run. D.7.14 opens. The
      clock's check does not fire: the `10+0.1` read is 12.9 below the
      gate's, within their combined intervals, and above RAR-S134's
      `3+0.03` read, so the clock stays with H.3.
    - **D.7.14 Adoption and checkpoint — `V`, then `M`, DONE 2026-11-21.**
      The maintainer's decision on D.7.13. On adoption: the pre-NNUE meter
      read again (RAR-M89's field, the gap to Stockfish 16 with `Use
      NNUE=false`); pooled NPS and conversion recorded; `CHANGELOG.md`'s
      entry for the new search; the fingerprint record; D.11's disclosure
      names the kernel's shape, its transcribed places and its fitted
      constants; `d7-kernel` goes to `master` as one squash through the
      maintainer's pull request. An adopted kernel releases as 2.8.0 once
      D.11 allows. The review of D.6 on the new kernel then opens, in the
      next pull request.
      *On adoption, from D.7.11 (2026-11-20):* PROCESS's margin block is
      restated in the kernel's seeds before any evaluation change is
      registered: its fixed surface still names the classic kernel's
      options, deleted by D.7.7, and `LazyMargin` stays in it. GUIDE's
      obligation row for the constants not fitted on Rarog is reviewed,
      and D.8.21 opens.
      *Adopted 2026-11-21* (maintainer decision, on RAR-S136's H1). Done
      the same day, in one documentation commit:
      - the margin block restated in the kernel's options, 69
        coordinates (PROCESS, *Evaluation change under a fitted
        search*);
      - `CHANGELOG.md`'s entry for the new search, and the README's
        `UCI_ShowWDL` row and score scale, which D.7.12 left out;
      - D.11's facts for the kernel named (D.11, *Named at D.7.14*);
      - GUIDE's two obligation rows reviewed; D.8.21 open;
      - the fingerprint record stands: 3,390,842 / EBF 2.317, D.7.11.8's,
        in AGENTS, GUIDE and PLAN's checkpoint row;
      - the pre-NNUE meter and the conversion registered as RAR-M90, a
        7,800-game gauntlet against RAR-M89's field, the maintainer's
        run (about 1 hour 40 minutes). Speed is recorded from the
        games, RAR-S136's and RAR-M90's: Colosseum's nodes a second at
        equal time. A pooled bench read is not run, since bench's
        nodes are two different trees for the two kernels and no
        decision reads it; D.13 and D.14 each register their own
        baseline.
      *Done 2026-11-21, RAR-M90 read:* the kernel 3,508.1 ± 8.2 in
      RAR-M89's field pinned at its ratings, 96.6 below Stockfish 16
      HCE and −100.6 [−119.4, −82.3] head to head; +63.5 over 2.7.0 in
      the field and +78.3 [+60.6, +96.4] head to head; no fault or
      time loss in 7,800 games. Conversion 20.6 draws and 2.8 losses a
      piece up per 1,000 games against 2.7.0's 15.4 and 3.2, a count a
      game and not a chance, so unresolved; 1.92 M nodes a second in
      play against 2.7.0's 1.96 M, 2.2 plies deeper. GUIDE's
      checkpoint rows carry them. `d7-kernel` reaches `master` as one
      squash through the maintainer's pull request; the review of D.6
      on the new kernel opens in the next.
    - **D.7.15 If the kernel does not gain — `R3`, CLOSED 2026-11-21.**
      *Not opened:* D.7.13's H1 (RAR-S136) adopted the kernel.
      Opens only on D.7.13's H0 or its cap without H1 (maintainer decision
      2026-11-19). The new kernel is discarded and the classic kernel
      restored from D.7.7's parent commit (`5d6f737`). Then, in this order and each by
      its own registration: an investigation of why a kernel that read
      about +50 before its repairs did not gate; Rarog's own dropped forms
      retried on it one at a time (D.7.0's inventory, its seventeen
      Rarog-only forms and the line bound of twice the depth among them);
      and the kernel in Stockfish's form by D.7's method (added 2026-11-16:
      Stockfish's search as the donor, the version Basilisk follows, read
      in source; an inventory of Rarog's departures from it; the parity
      instrument; the kernel whole; one joint fit and one gate on the same
      gain bracket). Its evidence so far: Stockfish dev's search unfitted
      on Rarog's evaluation is 34.3 ± 12.0 worse than Reckless's at equal
      work (RAR-O11 (e)), and Basilisk's search, fitted on its own
      evaluation, gained 55.2 ± 16.6 more per move made than Rarog's
      (RAR-O12).
      Two items from D.7.4 wait here. The parity instrument's rate rule
      bounds its tail before it judges another kernel: its 3·SE limit has
      no ceiling, and one runaway tree widened its bands to 13 times a
      rate (`cb7c79b`). And the classic kernel answers a promotion push in
      SEE by its gain without the exchange, where the donor and the
      compact kernel play the exchange: a repair card for it, with its
      measured exposure, if the classic kernel is restored.
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
  D.7.7 deleted the classic kernel (2026-11-19), so D.8.1's census is
  taken on the tree that remains and every item is read against that
  census before it starts; the counts above are the 2026-11-01 tree's.
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
    - **D.8.21 P20: the kernel's seeds no search reaches — `V`.** Added
      2026-11-20 from D.7.11's research (`analysis/d711_fit_2026-11-20.md`,
      *The inactive seeds are not removed now*); opens after D.7.14.
      Eight seeds change no decision on `bench 13` or on `bench 17` at
      rule 7c's perturbation: `fds_singular`, `fds_singular_cap`,
      `fds_singular_offset`, `nmp_min_16`, `rfp_min_16`,
      `pick_check_base`, `tt_cut_fifty` and `window_stability_cap`. That
      is a statement about forty positions, not about games
      (`pick_check_piece` read inactive at depth 13 and active at 17), so
      nothing is removed on it.
      - *Screen, zero games:* a `probes` build over replayed gate games
        counts how often each term binds: the two floors against their
        margins, the cap against the two stability counts, the
        fifty-move test, the reduction's singular term, and whether the
        check offset ever changes an order.
      - *Rule, frozen here:* a term that never binds in the replay is
        deleted with its seed as a behaviour-neutral change, the
        fingerprint and the replay its checks; one that binds stays,
        its rate recorded. `tt_cut_fifty` stays whatever its rate: it
        is a rule of the fifty-move clock.
      - Not before D.7.14: a removal moves the fit's baseline, and the
        gain is a few lines, not Elo or speed (maintainer decision
        2026-11-19: compactness is secondary).
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
  compact kernel (D.7), whose shape follows Reckless's, makes it heavier for that
  release.
  *Named at D.7.14 (2026-11-21), on adoption:* the release after 2.7.0
  ships the new kernel, so the disclosure also names:
  - **The search's shape:** Reckless's (`0756957`), rebuilt from
    D.7.0's inventory in Rarog's own code and structure (D.7,
    *Independence*). Reckless's move ordering, history, correction and
    TT-cutoff updates, the second group above, are now part of that
    shape.
  - **The clock:** Reckless's time manager whole (D.7.9.1, `38e947c`),
    its constants the donor's, with Rarog's guards. Stockfish's time
    budget, the first group above, left the code with the classic
    kernel.
  - **The places transcribed,** each named in its commit: the
    interpolation toward beta in single precision with a fused
    multiply-add, truncated toward zero (`3bda2c3`), and the
    upcoming-repetition table by van Kervinck's method (`efbbb69`),
    which has one natural form.
  - **The constants:** the donor's seeds converted by dimension
    (D.7.4.1) and fitted on Rarog's own games twice (RAR-S128,
    RAR-S135). Those still at the donor's values are listed in
    `analysis/d711_fit_2026-11-20.md`, *What stays unfitted*: the
    clock's twenty-three, the two thread spreads and the eight seeds
    no bench reaches.
  - **The reported score:** Stockfish's win-rate form and its
    `UCI_ShowWDL` option, with Rarog's own constants (D.7.12).
  Step 1's rerun reads the release head with these in it.
  *Widened 2026-11-23* (maintainer request, from Coda's read,
  `analysis/coda_read_2026-11-23.md`; the maintainer wants Coda's
  disclosure as the model). The step is larger than drafted:
  - Reckless's licence by date: MIT from 2023-04-16, none from
    2025-02-22, AGPL-3.0 from 2026-01-26. Every Reckless commit Rarog
    cites for its code is later, so what crossed is AGPL material.
  - GPL-3.0 and AGPL-3.0 may be combined (section 13 of each), but the
    AGPL-derived parts keep AGPL terms and Reckless's notice, and AGPL's
    network clause covers the whole. Rarog's only licence statement is
    GPL-3.0-or-later and README names Reckless for inspiration only:
    that labelling gap is a defect whichever licence is chosen.
  - Tracked AGPL material: the three Reckless patches in
    `tools/oracle-hybrids/` carry Reckless's source in their context
    lines, unlabelled.
  - The dependence has grown since the step was written: D.7 rebuilt the
    kernel on Reckless's shape and took its clock whole (*Named at
    D.7.14*), and Phase F names Reckless a primary source for the network
    runtime and trainer pipeline (maintainer decision 2026-11-23). Coda's
    one finding of copied expression was in that area: threat-feature
    indexing and SIMD kernels modelled on Reckless's.
  - No release carries the compact kernel yet; its source is on public
    `master` since PR #20.
  Licence route decided 2026-11-23 (D.11.2): GPL-3.0 kept, Stockfish the
  primary donor, Reckless ideas only.
  The step's four items are carried by D.11.1 to D.11.4 below, item 4
  with D.15.1; the falsifier and stop rule above stand. Refitting donor
  constants stays outside this step under route 1 of D.11.2; under
  route 2 each refit is its own registered experiment, named by D.11.3.
  D.11 closes when D.11.1 to D.11.4 are done and D.15.1 ships the
  notices.
    - **D.11.1 The donor inventory by licence, on the release head — `R2`.**
      Item 1's rerun, widened from `src/` to everything tracked:
      - each donor's licence at each version Rarog read (Reckless's three
        periods above; Stockfish and Triumviratus GPL-3.0; Fathom MIT), in
        a table of Coda's form;
      - the Reckless-derived parts by kind: the kernel's shape, the clock
        and its constants, the constants still at the donor's values
        (*What stays unfitted*), the two transcribed places, and B.2's
        constants that 2.5.0 to 2.7.0 shipped;
      - the Stockfish-derived parts: Stockfish 11's evaluation design and
        the reported score's form;
      - tracked files holding donor text: the oracle patches, and the
        fifteen files that cite donor source by path (most with line
        numbers), read for quotation beyond a few words;
      - source comments that name a donor, checked against what the code
        is: Coda's own comments called reimplementations "verbatim" and
        "ports", which read as copies.

      Output: the inventory in the review (`analysis/`), which D.11.2
      decides on and D.11.4 publishes. Closes when it covers the source,
      the tools and the documents at the release head.
    - **D.11.2 The licence decision — `R2` card, then a maintainer decision.**
      After D.11.1. The card states, from the inventory, what each route
      costs (`analysis/coda_read_2026-11-23.md`, *What AGPL is*):
      1. *Relicense as AGPL-3.0-or-later.* `LICENSE`, `Cargo.toml`, README
         and the archives change. Rarog's duties are GPL's plus section 13:
         a modified Rarog that others play or query over a network offers
         its source, which the public repository and a link meet. Rating
         lists are unaffected, and Reckless stays readable at its latest
         commit as PROCESS has it. Releases up to 2.7.0 stay as published.
      2. *Stay GPL-3.0, Coda's route.* Each Reckless-derived part is
         reimplemented from ideas with constants derived from Rarog's own
         quantities, or removed, each a registered experiment; the patches
         are labelled AGPL or leave the tree; PROCESS's boundary becomes
         ideas only for an AGPL engine, with no constant crossing, and
         Phase F's sourcing decision of 2026-11-23 changes with it. The
         kernel's shape stays the one point where an argument can be made.
      3. *Ask Reckless's authors* (the maintainer sends): a courtesy with
         either route, and a written permission settles what it covers.

      The decision also settles whether README says how Rarog is
      developed, as Coda's opening does (AGENTS bars agents from adding AI
      attribution, so the line is the maintainer's); whether donor study
      material (the patches, any quoted donor code in `analysis/`) stays
      in the public repository; and whether the git history stays as it
      is and is said to. Not legal advice.
      *Recommendation of the 2026-11-23 read:* route 1 with route 3. It
      settles compliance for the kernel, the clock and the patches in one
      change and keeps D.7's design and PROCESS's donor. Route 2 only if
      Rarog's code must stay reusable under plain GPL-3.0.
      *Decided 2026-11-23 (maintainer):* route 2. Rarog stays GPL-3.0, and
      the attributions and the process are arranged so that Rarog never
      distributes Reckless's code with its own. From that day Stockfish is
      the primary donor and Reckless, like any AGPL or unlicensed engine,
      teaches ideas only: no constant port and no further AGPL material
      (PROCESS, *The independence boundary*). What is already in the tree
      is resolved here later, researched by the maintainer. The
      recommendation above is not taken. Still open in this card: the
      README line on how Rarog is developed, the study material in the
      public repository, and the git history, which keeps the patches in
      published commits.
    - **D.11.3 The decision carried out — `I1`, or registered experiments under route 2.**
      After D.11.2. Route 1: the licence files, `Cargo.toml`, Reckless's
      copyright notice where its material is, the patches labelled,
      PROCESS and AGENTS where they name the licence, and a `CHANGELOG.md`
      entry. Route 2: each part D.11.1 lists reimplemented or refitted as
      its own registered experiment (the clock's constants as a registered
      tune, read as Coda read its retune: values that return near the
      donor's are functional optima, not a failure), the patches out of
      the tree or labelled, and the boundary rewritten. Engine and
      documentation changes in separate commits.
      *Route 2 since 2026-11-23:* the boundary is already rewritten
      (PROCESS); the patches leave the tree, untracked with
      `git rm --cached` so the local bytes stay. Under the rule of that
      day a retune seeded from a Reckless value is still a port, so a
      refit starts from Rarog's own derivation; whether Coda's convergence
      argument changes that is this step's question.
    - **D.11.4 The disclosure, on Coda's model — `M`.**
      After D.11.3; item 3 in Coda's form (the read, *How Coda discloses
      its sources*):
      - the review in `analysis/`, dated and kept as a living record: why
        it exists, the principle applied (ideas and functional constants
        against expression), the licence table at the version read, one
        section per donor (what was taken, what was not, what was done,
        with the commits), its scope and what is still open, "not legal
        advice", and an invitation to open an issue;
      - a notices file at the root for derived and studied material
        (Reckless's notice where its material is, Fathom's MIT notice),
        separate from `LICENSE`, shipped in D.15.1's archives;
      - README's *Acknowledgements* and *License*, short and for users:
        the donors by licence with what each gave (Stockfish 11's
        evaluation design, Reckless's search shape, clock and some
        constants, Fathom's probing) and the licence's history in a
        sentence or two;
      - PROCESS's boundary saying what crossed and that it is disclosed.
- **D.12 Board review and rebuild: the contract audited, each component read against the donors' boards, rebuilt where another form measures better — `R2`, then `I2`/`V`.**
  Added 2026-11-14 (maintainer: the board implementation is revisited in
  depth), on the model of Basilisk's B.7 (audit and instruments, review,
  rebuild, speed read). Board work is about 22% of the time at
  `bench 13` in the last profile (generation 9.6%, SEE 7.1%, make/unmake
  4.9%; `analysis/c12_speed_review_2026-10-18.md`; corrected 2026-11-14
  from a 24% cited to RAR-M48, which holds no share), and the board is
  what D.7's kernel, should it land, and F.1's
  board events are built on; D.7 keeps Rarog's board, so this step does
  not wait for it. Rarog's form stays wherever it measures better.
  D.8.4 and D.8.5 (geometry into one module, the split of
  `board/board.rs`) move the same code: whichever lands first, the other
  rebases, and neither changes behaviour. Triumviratus's board leads
  (TB-4 to TB-8), deferred to H.4, are read here first; H.4 keeps what
  this step does not take. Not started; no work before the maintainer
  opens it. Basilisk's board findings for Rarog (imported 2026-11-14 from
  Basilisk's B.7, `analysis/basilisk_board_handoff_2026-11-14.md`, placed
  against Rarog's records in
  `analysis/d12_basilisk_board_placement_2026-11-14.md`) add four leaves,
  run in this order after D.12.0: D.12.1.1, D.12.1.2, D.12.2.1,
  D.12.2.2. Their figures are Basilisk's priors; nothing is adopted
  without Rarog's own registered read.
  **Held 2026-11-23 (maintainer decision on E.0's packet):** the board's
  work a node is level with Stockfish 16's (94 ns against 89 on game
  positions, `analysis/e0_remaining_strength_2026-11-23.md`), so the
  rebuild has no strength case against the engines above Rarog. D.12.0's
  research stays, for F.1's board events; D.12.1 to D.12.4 are held until
  D.16.3.1's profile or D.12.0 shows a board function that costs more
  than the donor's. D.16.3.1's profile did not (2026-11-23): once
  boundaries are matched, make with check information is 31.4 against
  32.9 ns a node, the exchange with its attacker query 26.3 against
  27.1, generation and legality 30.6 against 27.9 (0.6% of samples,
  within the cross-build uncertainty); D.12.0 alone can lift the hold.
    - **D.12.0 Rarog's own board interactions, and whether Basilisk's leads are Rarog's best options — `R3`.**
      Added 2026-11-14 (maintainer: what worked in Basilisk is a hint,
      not the best solution for Rarog). Research only, before any of
      D.12's leaves runs; nothing is built or timed. It starts from
      Rarog's source and records, not from the import.
      *The interaction map.* For each board component, its producers,
      consumers and shared signals inside Rarog, read in source: SEE's
      callers and thresholds (the picker's good-capture split and its
      divisor, the node's pruning thresholds, quiescence's scoring,
      threshold and bad-capture floor, ProbCut) and what they do with
      an answer; the per-node `CheckInfo`, `Threats` and pinned set and
      every reader of each (generation, staged king moves, quiet
      scoring, `gives_check_with`, make's check hint); the derived keys
      and their consumers (the pawn cache, the correction tables, the TT
      prefetch's key prediction); the undo record and the history
      stack; generation order as an input to the fingerprint, the
      picker's partition and the tree. Where a change to one component
      moves another's population, cost or answer, it says so, and which
      changes stay behaviour-neutral by construction.
      *The options.* For each component, Rarog's candidates ranked by
      an upper bound from Rarog's own profile and counts (C.12's head
      profile and the `diag` counters; a fresh profile, if the bounds
      need one, is prepared and handed to the maintainer), with Rarog's
      recorded results (RAR-M31 to RAR-M39, RAR-M43, RAR-M44, C.12's
      dropped list) and every donor's form (Stockfish, Reckless,
      Triumviratus, Basilisk) as candidates, none preferred. In
      particular: generation, the largest board share (9.6%) and
      Rarog's widest measured gap to Basilisk (29%), which none of the
      four leaves addresses; the check queries at about three
      `gives_check` calls a node; and whether board work is the right
      target at all against move ordering (20.7%) and the node body
      (17.2%).
      *Output.* The map and the ranked list; for each of D.12.1.1,
      D.12.1.2, D.12.2.1 and D.12.2.2 a decision (keep, reshape or
      drop) with its reason; any better Rarog option as a new leaf with
      the same card (mechanism, interactions, invariants, size,
      falsifier); and what the replay must record for the options kept.
      A leaf it reshapes gets its prediction rewritten here, before any
      exposure, the change recorded beside the old one.
      *The check hint first:* its precondition is counted before any
      option touching make, check information or `gives_check` is
      ranked (D.12.2, lead 2, the rule and the two counts).
      *Stop rule:* an option whose bound is under the pooled read's
      resolution (about 0.4% at step 1) is listed, not proposed. If
      nothing on the board clears it, D.12's speed leaves close
      `NO_CHANGE` and D.12 keeps only the contract work (D.12.1) and the
      rebuild's safety net.
    - **D.12.1 Board contract audit and instruments — `R2`.** The
      safety net D.12.3 rebuilds behind. Inventory first what exists and
      what each check covers: `tests/board_correctness.rs`,
      `board_differential.rs`, `board_v2.rs`, `see_contract.rs`,
      `see_pins.rs`, `slider_backends.rs`, `fuzz_lite.rs`, the board
      footprint assertions, `benches/board_v2.rs`, D.4's search-path
      perft and D.5.17's `gives_check` differential. Then close the gaps
      against Basilisk's B.7.1 checklist: an external oracle on real game
      positions (FEN, checkers, legal, capture and checking sets, perft
      and divide); make, unmake and null restoring every field over a
      walk; clones and long unwinds; history capacity; the Zobrist key
      set's GF(2) rank (TB-2); FEN robustness (TB-3); the clock outside
      the key; rule-50, repetition and mate precedence audited apart;
      SEE against the legal exchange tree. Every check is shown able to
      fail by a negative control. A semantic defect found here takes the
      correctness-repair path with its own gate (AGENTS, *Gating*). The
      timing instrument is Basilisk's contract cross-engine-board-v2
      (`D:/code/basilisk/tools/board_compare/CONTRACT.md`), reused rather
      than rebuilt: 512 positions of real games with a python-chess
      oracle line (`D:/code/basilisk/tests/data/board_micro_positions.fen`),
      columns `legal_gen`, `capture_gen`, `make_unmake`, `see_captures`
      (values 100/300/300/500/900), `gives_check` and `board_perft`.
      Rarog's adapter is a patch on `611307e`
      (`D:/code/basilisk/tools/board_compare/rarog/board_xv2.diff`: a new
      bench file and a `[[bench]]` entry, no Rarog source changed), built
      by `D:/code/basilisk/tools/board_compare/build.py`, beside adapters
      for Basilisk, Stockfish 19, classical Stockfish and Reckless; the
      runner is `D:/code/basilisk/tools/diag/board_micro_run.py`
      (alternating binaries, one pinned CPU, a host gate, every sample
      kept, semantic counts cross-checked). Added to it here: Rarog's
      own call mix a search node (makes, generations by stage, SEE,
      `gives_check`, check-information builds) counted by a throwaway
      build at its bench, and the search's fast `gives_check_with`,
      exposed under a bench-only feature.
        - **D.12.1.1 Search replay with a verify mode and a stub control, and the engine screen — `I2`.**
          The instrument of the import's §2, built for Rarog. A
          `board-trace` feature records every board operation the search
          makes on its own board, in order, with its result: make with
          its hint, unmake, the null move and its unmake, each generation
          stage with its moves in order, `see_ge` and
          `see_ge_quiet_aware` with their thresholds, `gives_check_with`,
          the check-information and threat builds, the TT-move legality
          test, the key prediction for the prefetch and the draw tests.
          A replayer, in its own crate under `tools/` with a path
          dependency so the engine binary is untouched, runs a trace on
          one board and nothing else. `--verify` compares every recorded
          result: the five keys and the checkers after each make and
          unmake, each SEE, check and legality answer, each list's moves
          in order. `--stub` runs the same records and dispatch with
          empty out-of-line calls: board cost is the timed run less the
          stub, and the two bracket it. An A/B of two boards on one trace
          needs no correction. The timing driver alternates binaries on
          one pinned CPU, refuses a busy host by the harness's measure
          (`tools/harness_common.ps1`) and archives hashes and every
          sample. The engine screen is C.12's `quickscreen.py` kept as a
          tracked tool: plain release builds, `bench 13` on one pinned
          CPU, 16 rounds with the order reversed on odd rounds, the
          per-round time ratio, the fingerprint asserted on every run.
          *Invariants:* the trace build reproduces the fingerprint, since
          it observes and changes nothing (`diag`'s contract); the
          recorder allocates, so it is never in a measured binary, and
          that exception to AGENTS' per-node rule is stated where it is
          built; verify fails at once on a planted wrong result in each
          record class; `--all-features` compiles it.
          *Size:* about ten records a node at Rarog's mix, near 90
          million for `bench 13`, so timing uses a reduced trace (about
          40 roots at a node cap) whose rates a node are checked against
          the full trace's; traces are regenerated on demand, never
          committed.
          *Calibration and falsifier:* the replay's rates a node must
          equal the `diag` counters' at the same revision, and its
          classes must place within C.12's profile shares (generation,
          SEE, make/unmake) as Basilisk's placed within its profile. The
          screen is read once on two of C.12's recorded pairs: `acd767e`
          with `09bb98c` (pooled +2.14%) and `081041c` (pooled +4.24%,
          which C.12's non-PGO screen read at +0.1%). A replay that cannot
          place its classes inside the profile prices nothing; a screen
          that misranks the first pair screens nothing, and a miss on the
          second records again that branch-layout changes need PGO.
          Replay and screen choose candidates; PROCESS's pooled read
          (`tools/nps_read.py`) decides, and nothing replaces it. Timed
          reads wait for an idle host the maintainer confirms; anything
          past a few minutes is handed over. *Retries nothing.*
        - **D.12.1.2 Two counts on the search's trace: SEE asks and capture generations repeated at a node — `V`.**
          The import's §5 on Rarog's own full `bench 13` trace
          (D.12.1.1); deterministic, no idle host. A node is the position
          key and ply between a make and its unmake. (1) For `see_ge` and
          `see_ge_quiet_aware` apart (they differ on quiet moves), and by
          call site (the picker's good-capture split, the node's pruning
          threshold, quiescence's scoring, threshold and bad-capture
          floor, ProbCut), the share of calls about a move already asked
          at the node with another threshold, and the share an earlier
          answer settles: `see_ge` is `see >= t` (C.12's `seecheck`,
          172,081 moves × 161 thresholds, and `tests/see_contract.rs`'s
          threshold walks), so a yes at `t` answers every threshold at or
          below it and a no every one at or above. (2) The share of noisy
          and capture generations repeated at the node (ProbCut's
          `generate_legal_noisy_into`, then the picker's capture stage).
          *Use:* whether a per-node SEE answer cache, or ProbCut's
          captures handed to the picker, earns a research card. The bound
          is the settled share of SEE's 7.1% of the profile and the
          repeated share of capture generation's part of 9.6%.
          *Prediction, frozen here:* repeated SEE asks 20% to 40%
          (quiescence can ask three times about one capture), settled 8%
          to 20%; repeated capture generations 3% to 10%.
          *Stop rule:* no card when a bound is under 0.5% of search time.
          A card is a picker change and waits for D.7.14: the kernel
          was rebuilt (decision of 2026-11-16) and these call sites are
          its picker's now. *Retries nothing:* C.12's one full SEE per
          tactical move answering every threshold (−0.24%) is another
          form, which a card would have to beat.
    - **D.12.2 Board review against Stockfish, Basilisk, Reckless and Triumviratus — `R2`.**
      A packet that reads each component against Stockfish 19's
      (`Position` and `StateInfo`), classical Stockfish's, Basilisk's,
      Reckless's (ideas only since 2026-11-23, PROCESS) and
      Triumviratus's measured ideas; measures Rarog's (its
      share in the profile at `bench 13`, D.12.1's microbenchmark, the
      cost a node at Rarog's own call mix); and decides keep, replace or
      adapt, each with its reason and expected cost. A departures table
      records every kept Rarog form; the frozen handoff is D.12.3's
      contract. Components: check information (computed at make as
      Stockfish's `StateInfo` holds it, or lazily per call; TB-4); make
      and unmake (what is copied, recomputed and branched; TB-5) and the
      per-ply dirty-piece record F.1 needs, its contract fixed here
      (TB-10); legality (the legal and pseudo-legal split, the TT and
      killer move test, `gives_check`); SEE (TB-6); attacks (TB-7);
      generation (TB-8; the side as a type parameter, staged delivery,
      captures not regenerated); repetition; Chess960 representable
      without a second rewrite (the feature is G.2's); the
      representation. A technique no source measured is a candidate to
      measure, not to adopt, and a measured loss counts as much as a
      gain.
      **Leads from Basilisk's board review** (recorded 2026-11-14 at the
      maintainer's request; Basilisk's B.7.2, 2026-11-14, packet
      `D:/code/basilisk/analysis/b72_board_review_2026-11-14.md` §1.1–1.3,
      on Basilisk's branch `b7-board-audit`; cite Basilisk's merged commit
      once its B.7 pull request lands). Rarog was measured at `611307e`.
      Every figure is Basilisk's measurement, a prior here and not a
      Rarog result. One session, five alternating rounds, idle Ryzen 9
      5950X, LLVM 23.1.5 for every arm, no PGO; median ns an operation,
      Rarog as a share of Basilisk; round-to-round spread 0.3–3.5% a
      column:
      - legal generation: Basilisk 48.1, Rarog 62.0 (129%), Reckless
        51.1, Stockfish 19 76.6, classical Stockfish 89.4;
      - make and unmake: Rarog **14.7**, the fastest of the five
        (Basilisk 17.3, classical Stockfish 35.5, Reckless 40.4,
        Stockfish 19 52.7);
      - SEE on captures: Rarog 13.8, Basilisk 15.3, both Stockfishes 6.6,
        Reckless 18.2;
      - `gives_check`: Rarog 7.6 (its public full form), Basilisk 6.5,
        both Stockfishes 3.2;
      - perft a leaf: Basilisk 3.35, Rarog 4.18 (125%), classical
        Stockfish 5.14, Reckless 5.41, Stockfish 19 6.19;
      - weighted by Basilisk's counted search mix (a node: 0.92
        make/unmake, 0.53 capture generations, 0.20 quiet generations,
        1.42 SEE, 1.44 `gives_check`), board cost a search node:
        Basilisk 65.8 ns, Rarog 67.0 (102%), classical Stockfish 71.7
        (109%), Stockfish 19 83.2 (126%).
      The leads:
      1. *Legal generation and perft* trail Basilisk by 29% and 25%. Read
         Basilisk's generator (`D:/code/basilisk/src/board.cpp`,
         `gen_legal_impl`: templated on colour, a check mask, pins from
         two x-ray slider lookups a line type, bulk pawn shifts) against
         Rarog's. RAR-M44(c) holds: a 12.7% generator gain on the bench
         measured −0.55% pooled-PGO, so only removed work counts and
         pooled NPS decides. Basilisk's replay read the generator's pins
         from aligned sliders at −0.5 ns a node, within noise: RAR-M31
         and C.12's −0.66% stand, and that form is not read again.
      2. *Make and unmake* is the fastest of the five: no lead. Whether
         the compact undo record should carry the derived keys is
         D.12.2.2. *The check hint* (corrected 2026-11-14 at Basilisk's
         request; the note it replaces over-read Basilisk's evidence as
         superseding the hint). Basilisk's evidence is for Basilisk's
         search, where `gives_check` is known before only 43% of makes
         (38.5% no, 4.5% yes), asked only where a pruning decision needs
         it: there a hinted make fed the known answers gained nothing on
         its replay of `bench 13` (+0.2 ns a node, +0.5 with no hint) and
         was not built into the engine. Its −0.74% is another mechanism,
         make deriving the checkers from the move with no answer from
         the search, and says nothing about a hint the search supplies.
         Its "about 4.0 ns a node uncharged" for Rarog priced Rarog's
         board under Basilisk's call mix, where a hinted board owes
         `gives_check` for the 57% of makes Basilisk does not ask about;
         it is not a cost Rarog's search pays. In Rarog (source at
         `a43443e`): the main search computes `gives_check` for every
         move it makes, just before the make, building the node's
         `CheckInfo` at the first, and hands it to
         `make_move_with_check`, which then skips `calculate_checkers`
         for a move that does not check. LMR reads the answer
         (`lmr_gives_check`), but only for moves after the first, from
         depth 2, never at the root or in check; for the rest, the
         answer, and at a node with no such move the `CheckInfo`, is
         paid only for the hint. ProbCut takes the plain make.
         Quiescence passes the hint only where its futility or
         count-limit test already asked, and the plain make otherwise.
         How many makes skip `calculate_checkers` is not counted.
         *The rule:* the hint is right exactly while its precondition
         holds, that the search already needs `gives_check`, and builds
         the node's `CheckInfo` for it, for nearly every move it makes;
         while that price is paid anyway, keep the hint. A search
         rebuild (D.7, if it lands) may change that, so D.12.0
         re-establishes the precondition on the search current when it
         runs, before anything about make is decided: over `bench 13`,
         the share of makes whose answer the search already computed for
         its own use, and the share of nodes that build `CheckInfo`
         only for the hint. If nearly every make has the answer, the
         hint stays. If not, the hint and the node's `CheckInfo` are a
         cost paid only for make, and the other forms are read: no hint,
         make computing the checkers with one `attackers_to` on the
         king (Basilisk's), check squares cached lazily per piece type
         in the picker, `gives_check` on demand per call, and the hybrid
         quiescence already uses (the hint where the answer is known,
         the plain make elsewhere). The checkers are identical in every
         form, so the fingerprint must not move; pooled-PGO NPS decides,
         not the microbenchmark. Basilisk's search replay
         (`D:/code/basilisk/tests/board_replay.cpp`,
         `D:/code/basilisk/tools/diag/board_replay_run.py`) and engine
         screen (`D:/code/basilisk/tools/diag/engine_ab.py`, branch
         `b7-board-audit` at `c6d27da`) show one cheap way to screen
         first. Neither engine's result transfers to the other.
      3. *The capture generator:* `generate_legal_noisy_into` yields 885
         of the 927 legal tactical moves; the 42 non-capturing rook,
         bishop and knight underpromotions go to the quiet stage, as in
         Reckless. Confirm that is intended everywhere the noisy
         generator is used (quiescence, ProbCut, evasions).
      4. *SEE policy:* like Stockfish, Rarog passes promotions and en
         passant without an exchange; on these 927 game moves that passes
         36 whose exchange loses (the legal exchange tree gives 325 moves
         at SEE ≥ 0, Rarog 361). Not a defect: decide the policy and
         record it. Basilisk scores these moves exactly (its BAS-C09
         departure).
      5. *`gives_check`:* the harness timed the public full form, since
         the search's fast form `gives_check_with(CheckInfo)` is
         `pub(crate)`; time the fast form (D.12.1) before comparing check
         cost.
      6. *Rarog's own weighting:* weight the columns by Rarog's call mix
         (D.12.1); per-operation times alone mislead. Stockfish halves
         `gives_check` and SEE with check information stored at every
         make but pays two to three times more a make; at Basilisk's mix
         the trade is a wash (cheaper queries save 17.0 ns a node, the
         dearer make costs 16.8; superseded 2026-11-14, first read as a
         loss from 65.8 against 71.7), and Rarog's mix may differ.
      7. *Stockfish's two boards as references:* Stockfish 19
         (`edb0d9db`), whose make writes the network's dirty-piece, threat
         and pawn-pair records, was 26% dearer a node than Basilisk;
         classical Stockfish (`9587eeeb`, its last HCE version) 9%. When
         Rarog adds network make records (F.1), price them with this
         harness against Stockfish 19's form.
        - **D.12.2.1 SEE's king-legality test from aligned sliders — `R2`.**
          After D.12.1.1 (its verify mode), not the rest of D.12.1.
          *Mechanism.* `see_recapturer` tests each selected candidate
          with a full `attackers_to_color` on its king under the exchange
          occupancy. For a candidate other than the king, every attacker
          that test can find off the target is a slider on one of the
          king's empty-board lines: a leaper's or the king's attack on it
          could come only from the target, which is excluded, or from a
          check the root move had to resolve. So the test equals: no
          enemy bishop, rook or queen in the exchange occupancy, off the
          target, on a line of its kind from the king, with nothing of
          that occupancy between them but the candidate. That covers the
          pin (the candidate alone between) and a check the previous
          capture uncovered (nothing between), which Basilisk's pin-only
          form does not refuse. Two 64-entry line tables and a
          between-squares mask a sniper; the king's own recapture keeps
          the full test. It reads the occupancy of the moment, as now, so
          the stale-pin defect 4.11b.5 repaired cannot return.
          *Touches:* `Board::see_recapturer` only; SEE's callers, values
          and exchange loop are unchanged.
          *Invariants:* every SEE answer equals the current form's: a
          debug check of the new against the old on every call (TM-10),
          D.12.1.1's verify on the whole trace, `tests/see_contract.rs`
          (the external fixtures and the threshold walks),
          `tests/see_pins.rs` (the legal-exchange oracle) and the exact
          fingerprint.
          *Fixtures first:* oracle-checked positions that separate the
          two contracts (a recapture refused because the previous capture
          uncovered a check; a root move that resolves a check by capture
          or interposition), shown able to fail: a pin-only form fails
          the first, the current form and the candidate pass all. A
          counterexample on any fixture or record returns the leaf to
          RESEARCH with it.
          *Size, frozen here:* the legality test cost 8.07% of the SEE
          microbenchmark when 4.11b.5 added it (RAR-M43) and SEE is 7.1%
          of the profile, so removing the test outright is worth about
          0.6% of search time, and its replacement is not free.
          Prediction: pooled-PGO NPS +0.1% to +0.6%. Basilisk's +1.56%
          (non-PGO) removed a pin array, its filter loop and a departure
          test, none of which Rarog has.
          *Falsifier and stop rule:* the replay A/B on the SEE records.
          A gain under 0.25% of the engine's time a node (the replay's ns
          a node against `bench 13`'s) stops it `NO_CHANGE` with no
          engine build; the floor sits under the pooled rule's +0.5%
          because the replay under-read both of Basilisk's engine gains
          (4.3 against 6.9 ns, 2.1 against 8.2). Above it, the screen,
          then PROCESS's two-step pooled read, handed to the maintainer,
          which decides.
          *Retries:* not RAR-M35 (the carried attacker set) but the
          target its lesson named, the per-candidate king-legality test;
          not C.12's superset pre-test (+0.08%), a form that kept the
          test. RAR-M34's reopening rule is not engaged, since nothing is
          shared. No recorded trigger forbids it.
        - **D.12.2.2 Unmake restoring the derived keys from its record — `I1`.**
          After D.12.1.1 (its verify mode).
          *Mechanism.* Unmake already restores the main key, castling,
          the en-passant square, both clocks and the checkers from
          `UnmakeInfo`, and discards no work. It recomputes the four
          derived keys (pawn, minor, both non-pawn) through `add_piece`,
          `remove_piece` and `move_piece`: a Zobrist load and the masked
          XORs per piece change, two or three a move. The candidate
          stores the four keys in the record at make (32 bytes, the
          record 24 → 56) and gives unmake board-only forms of the three
          helpers.
          *Touches:* `UnmakeInfo`, `make_move_inner`, `unmake_move` and
          the helpers' unmake forms; the null move is unchanged.
          *Invariants:* every derived key equal after each unmake
          (`tests/board_differential.rs` rebuilds every derived field;
          D.12.1.1's verify on the whole trace) and the exact
          fingerprint. The `UnmakeInfo <= 24` guard moves only in its own
          commit with this leaf's measurement; RAR-M39's argument was a
          footprint one, and a 128-ply stack of 56 bytes is 7 KiB, still
          in L1.
          *Size, frozen here:* make/unmake is 4.9% of the profile, and
          the derived keys are a part of unmake's part. Prediction:
          pooled-PGO NPS −0.2% to +0.6%. Basilisk's +1.86% (non-PGO)
          removed key updates its unmake overwrote afterwards, work
          Rarog does not do; Triumviratus's TB-5 copy (keys, fifty-move
          counter and occupancies, 64 bytes) read −0.61% of cycles there.
          *Falsifier and stop rule:* the replay A/B on the make and
          unmake records, by D.12.2.1's floor and path; make's added
          stores count against unmake's saving.
          *Retries:* RAR-M39's footprint decision, whose guard names
          growth as the condition for re-checking it, and TB-5, which
          D.12 reads before H.4. `acd767e`'s masked key toggles stay.
    - **D.12.3 Rebuild behind the board contract — `I2`.** D.12.2's
      accepted components, one commit each, each passing D.12.1's
      contract and D.4's search-path perft and keeping the exact
      `bench 13` fingerprint while the generation order is unchanged. No
      search, evaluation or parameter change rides along. Each replaced
      component keeps, behind a build switch, a check that runs the old
      form beside the new at every node and aborts on the first
      disagreement (TM-10). Per-node code obeys AGENTS' allocation and
      copy rule; the footprint assertions move only with a measured
      reason.
    - **D.12.4 Board speed read and gate — `V`.** Pooled-PGO NPS of the
      rebuilt board against the head by PROCESS's method, D.12.1's
      microbenchmark per component beside it, and each component's share
      in the profile; a component that does not pay its cost returns to
      Rarog's form. A component that changes the generation order, and
      so the tree, is registered and gated as AGENTS' *Gating* requires.
      Nanoseconds an operation are not NPS and not Elo.

- **D.13 Correction tables shared between threads — `R2` card, then `I1`/`V`.**
  Moved here from D.7.8.7 on 2026-11-20 with its card (maintainer
  decision): it is thread work, and one thread plays the same, so
  nothing in D.7's one-thread fit or gate waits for it. It opens after
  D.7.14 and is built only if `bench 13` reproduces the fingerprint at
  one thread; then the NPS read and the four-thread read the card
  prices. The card, as written on 2026-11-19:
  (A7, inventory C-6) The donor shares its pawn and non-pawn
  correction tables between threads; Rarog's are per thread, and
  four threads were worth 1.37 doublings of Rarog's time against
  Reckless's 1.49 (RAR-M83). *Card only:* what sharing needs (an
  atomic read and write a node is the exception AGENTS' per-node
  rule asks a reason and an NPS read for), its one-thread cost,
  and the four-thread read that would accept it with its cost in
  hours. The maintainer decides from the card whether it is built
  before D.7.13's four-thread read or left to the thread work
  after D.7. One thread plays the same either way, so the fit does
  not wait for it.
  *Card (2026-11-19).* **The donor's form:** `SharedCorrectionHistory`
  (Reckless `thread.rs`, at `0756957`) holds the pawn table and the
  two non-pawn tables, each `[16 buckets][2 sides][65,536]` of
  `AtomicI16`, one copy per NUMA node; a read is a relaxed load,
  an update a relaxed load and a relaxed store (no read-modify-
  write, so a racing update may be lost, never torn). The
  continuation correction stays per thread. Rarog's three tables
  have the same shape (`correction.rs`), per thread, 12 MB each.
  **What sharing needs:** the three tables move to the search's
  shared state, their entries `AtomicI16`; the node reads them
  through the reference it already holds to the shared state (no
  `Arc` touched per node) and trains them with the same gravity
  formula as a relaxed load and store. It is the exception AGENTS'
  per-node rule asks a reason and an NPS read for; the atomic
  signals one search's evaluation residuals to every thread of
  that search, and a new game clears it once rather than per
  thread. **One thread:** on x86-64 a relaxed 16-bit load or store
  is the same `mov` as today's, so one thread plays the same
  (`bench 13` exact) and the expected cost is an extra pointer
  load a node, within PROCESS's NPS read (four PGO builds an arm,
  step 1 about 6 minutes, ±0.4%, after about 15 minutes of
  builds). Memory: 12 MB less a helper. **Four threads:** RAR-M83
  read four threads worth 1.33 (1.16 to 1.49) doublings of the
  classic kernel's own time against Reckless's 1.51 and
  Stockfish's 1.55, NOT DEFICIENT; how much of the gap sharing
  closes is unmeasured. **The read that would accept it:** both
  arms at four threads, `3+0.03`, the explicit one-CPU-per-core
  placement, three games at a time (RAR-S132's smoke played 200 in
  10.6 minutes). A gain needs an SPRT: `[0,3]` at a true +4 nElo
  takes about 47,000 games, about 41 hours; `[0,5]`, widened for a
  prior of +5 to +10 at four threads, about 15,000 to 20,000,
  13 to 18 hours. A no-harm read only (case 2, 2,000 games) takes
  about 1 hour 45 minutes. **Recommendation: build it after D.7,
  in the thread work.** The fit and the gate are one-thread, which
  sharing does not change; built before D.7.13 it would sit in the
  four-thread read beside the gate with the kernel's other
  changes, and that read could not say which moved it; its own
  acceptance read costs half a day to two days of four-thread
  games. What would change the recommendation: D.7.13's
  four-thread read showing the kernel scaling worse than
  `d7classic`, which makes sharing its first candidate. **The
  maintainer decides** (asked 2026-11-20).
  *Decided 2026-11-20 (maintainer):* after D.7, as recommended, if
  one thread plays the same; a moved one-thread fingerprint returns
  the step to `RESEARCH` before any game.
  *Since 2026-11-23* Reckless is a reference for ideas only (PROCESS):
  sharing the tables is the idea under test, and the design is written
  from Rarog's own tables, with Stockfish's form, where it has one, the
  reference.
- **D.14 The donor's thread vote on the new clock — `R2` screen, then `I1`/`V`.**
  (maintainer request 2026-11-20) D.7.9.1 took Reckless's time manager
  whole except its soft-stop vote, which stayed Rarog's. Rarog's is a
  measured strict majority: exempting two threads cost −15.85 Elo
  (`shared.rs`, `votes_needed`). The one-thread reads cannot test a vote,
  but much has changed under it since that measurement: the kernel, and a
  soft limit whose factors move far more from one iteration to the next
  (the node-share factor alone runs from 0.55 to 3.18). So the donor's
  form may now work better.
  - *The difference:* the donor needs 65% of the threads, rounded up,
    and a thread whose soft limit stops being met withdraws its vote
    (Reckless `search.rs:264-279`). Rarog's thread votes once and keeps
    the vote. At one thread the two are the same rule; at two, three,
    four and six threads the thresholds are equal, so at the rated four
    threads only withdrawal differs. At five threads (4 against 3) and
    at eight and more the donor's threshold is higher.
  - *Why after the fit:* the fit and D.7.13's gate run at one thread,
    where the votes are identical, so neither can see this. The vote
    reads soft limits whose factors read the search the fit moves
    (the clock's own constants are held, D.7.11), so it is read
    after the fit. It opens after D.7.14, beside D.13; the two are read
    separately, so that each four-thread result names its cause.
  - *Screen, zero games, its rule frozen here:* a `diag` counter at
    four threads, on 300 book positions at `3000+30` with the explicit
    placement, counting the searches in which a thread that voted would
    have withdrawn (its soft limit no longer met at a later iteration)
    before the pool stopped. Under 2% of searches: `NO_CHANGE`, since
    withdrawal is too rare to move a result. At 2% or more, the
    donor's vote is built behind an option.
  - *The read if built:* both arms at four threads, `3+0.03`, the
    explicit placement, three games at a time. A no-harm read (case 2,
    2,000 games) takes about 1 hour 45 minutes. A gain needs an SPRT,
    priced in the card as D.13's was (`[0,5]` about 13 to 18 hours).
    One thread must reproduce the fingerprint and play the same.
  - *ARM ordering, if built:* a withdrawn vote is one thread's state read
    by the others, so it is published with release and read with acquire,
    as Coda requires of every reader-publish pattern (x86 hides the bug,
    ARM shows it; `analysis/coda_read_2026-11-23.md`, lead 7), and checked
    on the ARM compatibility host.
  - *Since 2026-11-23* Reckless is a reference for ideas only (PROCESS):
    withdrawal is the idea under test, and a built vote keeps Rarog's
    threshold or derives one from Rarog's own reads, never the donor's 65%.

- **D.15 Release packaging from Basilisk 1.11.0 — `I1`.**
  Added 2026-11-21 (maintainer request) from Basilisk's release work
  between `v1.10.2` and `v1.11.0`; it lands before the next release
  (2.8.0), which also waits for D.11. Basilisk's other release changes
  are already in Rarog or do not apply: the `<ver>-<os>-<x64|arm64>`
  asset names and the universal x86-64 binary (E.3.2, E.3.3), a
  development build's `id name` naming its commit, a patch tag cut from a
  branch, the release commit's fingerprint check, and Basilisk's Clang,
  Xcode and MSVC changes (Rarog pins its Rust toolchain). Its published
  `SHA256SUMS` is not taken: Rarog checks the sums between the
  workflow's jobs and does not publish them, since a sums file beside the
  assets proves nothing about them (`release.yml`).
    - **D.15.1 Each download an archive — `I1`.**
      Each release asset becomes a `.zip` (Windows) or a `.tar.gz` (Linux,
      macOS; the executable bit kept) holding one folder
      `rarog-<ver>-<os>-<arch>/` with the binary, `LICENSE`, Fathom's
      licence (`vendor/fathom/LICENSE`), D.11.4's notices file,
      `README.md` and `CHANGELOG.md`,
      as Basilisk's (`87de6fa`) and Stockfish's releases do. It is where
      D.11's notices ship with the binaries. The workflow's publish step,
      the README's download table and an upgrade note for the renamed
      downloads change with it; the smoke and fingerprint checks run on
      the binary taken out of each archive. Checked by a release-candidate
      run that lists each archive's contents.
    - **D.15.2 Release notes as a summary and highlights — `I1`.**
      The release notes become the changelog section's opening summary
      and a `### Highlights` list, each paragraph joined into one line
      (GitHub renders every newline as a break), with a link to the full
      changelog at the tag; the release check refuses a section with
      neither (Basilisk `1e68ec6`: 1.11.0's page carried the whole
      section, 4,300 words). `cargo xtask release-check --notes` writes
      them; its tests and a run on a dated section are the check. PROCESS
      *Release* and the changelog's convention change with it.
- **D.16 Speed read on game positions: the bench read checked against play, and one more speed pass on the compact kernel if they disagree — `I1`, then `V`; `R2` card for the pass, DONE 2026-11-24.**
  Added 2026-11-23 (maintainer request: more speed work on the compact
  kernel is coming) from Trinity's record
  (`analysis/trinity_read_2026-11-23.md`, *Measurement lessons*): a picker
  change there read +5.6% nodes per second on its bench, while its half
  that drops the quiet moves pruning has already skipped, the same search,
  ran 18% faster on 60 game positions at 300 ms and gained +18.6 ± 3.9 Elo
  where the bench priced about +3. Rarog's speed read is `bench 13 3`
  (PROCESS); D.7.8.12's profile found game play the same shape, which is a
  profile, not a paired delta, and a memory-bound engine's price for a
  change can move with the table size and the position mix. This step
  decides which instrument prices the coming speed work.
    - **D.16.1 The game-position speed read — `I1`, DONE 2026-11-23.**
      A frozen position set sampled from recent gate games (Rarog to move,
      spread over the phases, from Colosseum's database), each position
      searched in a fresh process (bench shares its table across
      positions) at a fixed node budget with the rated `Hash`; the arms
      are PROCESS's pooled PGO builds and the estimate and interval are
      the bench read's. Checked by a self pair that reads about 0.00% and
      a manifest of the set's hashes; it goes into PROCESS beside the
      bench read.
      *Done 2026-11-23* (`415ec4f`): `tools/nps_read.py --positions` on
      `tools/nps_positions_v1.jsonl`, 200 positions frozen by
      `tools/nps_positions.py` from RAR-S139's games (moves drawn by
      thinking time, one a game, no mate or tablebase score; phase mix
      41/23/25/11% against the games' 38/24/30/8% of time), budget
      186,000 nodes (the drawn moves' median), Hash 64, a fresh process a
      position with the clears done before the clock; the manifest holds
      the source's and the set's hashes and the set is stored byte-exact.
      Each reading's best moves and counts must match its arm's first,
      the tree check; it passed two passes of the head and refused 2.7.0.
      Self pair, two four-build pools of `415ec4f`: **−0.16% [−0.41%,
      +0.08%]** over two cycles, none disturbed, the trees identical, step
      1 in 6.6 minutes (2.385M nodes a second on these positions). The
      read's tests are `tools/diag/test_nps_read.py` (33 pass). It is in
      PROCESS beside the bench read; the bench read stays the speed read
      until D.16.2 decides.
    - **D.16.2 The bench read against play — `V`, DONE 2026-11-24.**
      Both reads on the same pools for at least two accepted speed
      changes of different kinds, rebuilt from their committed trees: a
      memory change (D.7.8.12's table prefetch) and a move-loop or
      evaluation change. Stop rule: when each change's two deltas agree
      within their combined intervals, the bench read stays the speed
      read, D.16.3 is not opened and D.16 closes; a change whose deltas
      differ beyond that makes the game-position read the speed read for
      its kind of change and opens D.16.3.
      *Done 2026-11-24* (`analysis/d162_bench_vs_play_2026-11-23.md`,
      registered `d7815f5` before any reading): three pools rebuilt from
      `42e3073`, `b88700a` and `1ff6522`, four pext PGO builds each at
      3,320,355, each read by step 1 of both reads, none disturbed, the
      games trees identical between arms. A, the table prefetch
      (`1ff6522` against `b88700a`): bench +0.27% (−0.08 to +0.62), games
      +0.70% (+0.44 to +0.95), gap 0.425 against the frozen band 0.432.
      B, the node work (`b88700a` against `42e3073`): bench +1.23% (+0.95
      to +1.51), games +1.29% (+0.99 to +1.58), gap 0.057 against 0.405.
      **Both agree, so the bench read stays the speed read**; D.16.3, open
      by the maintainer's decision, is read by it, and D.16 closes with
      D.16.3. Recorded for D.16.3's registration: on the memory change,
      the bench read did not exclude zero at step 1 where the games read
      did, and A's bench delta did not reproduce its acceptance read
      (+1.09%, 0.82 points away).
    - **D.16.3 One more speed pass on the compact kernel — `R2` card, `I1` items, `V` reads, NO_CHANGE 2026-11-24.**
      Opened by D.16.2's discrepancy or by the maintainer. A profile on
      game positions with Stockfish as the reference (Reckless's binary
      may be profiled beside it; its code is ideas only since 2026-11-23,
      PROCESS), items in
      D.7.8.12's form: the fingerprint exact at every commit, AGENTS'
      per-node rule, no new `unsafe` site (the floor is 17), each item
      priced by the read D.16.2 chose.
      *Handoff from D.16.3.1 (2026-11-23,* `analysis/d1631_node_profile_2026-11-23.md`*):* one item. In
      the quiet-scoring loop, prefetch the six history entries of the
      move a few places ahead before scoring the current one, through
      `tt.rs`'s `prefetch_ptr` (no new `unsafe` site, no buffer beyond
      the move list); the fingerprint exact. Ceiling 3.5% of samples,
      expected small. Stop rule: a read whose interval does not exclude
      zero reverts it, and D.16.3 then closes `NO_CHANGE`: the rest of
      the gap is the tree, its mechanisms and the evaluation's form.
      *NO_CHANGE 2026-11-24* (`analysis/d163_history_prefetch_2026-11-24.md`,
      registered `21793a2` before the build): the item built as
      `90c4460`, the fingerprint exact, checks passed, no `memcpy` in the
      node's functions; the bench read, deciding, **−4.74% (−6.49 to
      −2.99)**, `NO_CHANGE` by step 1, and the games read beside it
      −3.67% (−3.89 to −3.45). Reverted (`22b4310`, the source
      byte-identical to before). Predicted +0.3%: a miss in sign; the
      prefetch's own work and cache use cost more than the loads it
      hid, which a profile's stall share did not show. The rest of the
      gap is the tree, its mechanisms and the evaluation's form.
        - **D.16.3.1 The kernel's node beside Stockfish's, function by function — `R2`, DONE 2026-11-23.**
          Added 2026-11-23 (maintainer decision on E.0's packet) and run
          first of D.16. E.0 profiled Stockfish 16's classical build on 281
          game positions and set it beside D.7.8.12's profile of the
          kernel: 186 ns a node of search-side work (the node, move
          ordering, histories) against 108, the evaluation 95 against 77,
          the board 94 against 89; the kernel's shares were three days and
          two changes old. Zero games, no engine change: a fresh profile of
          the head on those positions, the two engines' functions matched
          one by one in the search-side block, the evaluation and the
          board, and D.7.8.5's counters read again on the head at `3+0.03`
          (4.0% of the kernel's time follows its last report). Output: a
          card of items in D.7.8.12's form, each behaviour-neutral, with
          its ceiling. Stop rules, frozen: with no neutral item of 1% of
          the samples or more D.16.3 closes `NO_CHANGE`, and what is left
          of the gap is the tree's own cost; D.7.8.5 reopens only at 2% of
          searches discarding a new best or failed-high move. E.0's band
          for the pass is 4% to 12% of node rate.
          *Done 2026-11-23* (`analysis/d1631_node_profile_2026-11-23.md`; zero games, no engine source
          changed). The head (`4ea670f`, pext PGO with its PDB archived,
          `bench 13` 3,694,298 / EBF 2.324) profiled on E.0's 281 positions
          at 150,000 nodes, 42,055 samples, beside E.0's Stockfish 16
          samples under one attribution rule. Today's gap is 412.7 against
          311.2 ns a node, +101.5: search side +76.7 (186.8 against 110.1),
          evaluation +31.5 (107.1 against 75.6), table −7.3, board +0.2.
          Quiescence nodes are 32.7% of the kernel's tree and 46.9% of
          Stockfish's; a full-width node's body costs 93.7 against 89.1 ns.
          The search side splits into the tree's shape (about 22 ns: more
          full-width nodes, 5.00 quiets scored a node against 3.27) and the
          kernel's own mechanisms Stockfish 16 lacks (about 54: correction
          14.3, threat maps every node 7.7, a quiet costing 9.73 against
          3.31 ns). One behaviour-neutral item reaches 1% of the samples:
          the quiet-scoring history loads prefetched a few moves ahead
          (3.5% of samples wait on them; D.7.8.12's analogous prefetch read
          +0.14%), so by the frozen rule D.16.3 does not close and keeps
          that one item. The band for the pass becomes 0% to 3.6% of node
          rate (0 to +6 Elo at RAR-M92's price, inferred), replacing E.0's
          4% to 12%. No board function costs clearly more than the donor's
          once boundaries are matched, so D.12.1 to D.12.4 stay held.
          D.7.8.5's counters on the head's diag build at `3000+30`: 0 of 300
          searches stopped inside an iteration, 0% against the frozen 2%,
          so D.7.8.5 stays closed; what E.0's 4.0% of time after the last
          report is stays unknown.
- **D.17 GitHub-hosted runners as a test lane for self-play SPRTs and tunes — `R2` card, then a maintainer decision.**
  Added 2026-11-23 (maintainer request), to open at the end of Phase D at
  the earliest, from Trinity's testing lane
  (`analysis/trinity_read_2026-11-23.md`, *The testing lane on GitHub
  Actions*). The use: more games for registered self-play SPRTs and tunes
  without the match host. GitHub's standard hosted runners are free for a
  public repository (4 vCPU, 16 GB; 6 hours a job, 20 jobs at once on the
  Free plan); the terms allow developing and testing the application and
  bar a burden disproportionate to its benefit, so the lane runs in bursts
  for registered work, never as a standing farm. Trinity's checks there:
  identical engines +0.15 over 9,000 games, and one candidate +16.3 ± 4.4
  against +19.1 ± 8.5 on its PC. The card answers, before anything is
  built:
  1. *The harness:* the pinned Colosseum CLI on a hosted runner (its
     Windows build, or its Linux build pinned by hash), the engine built
     on the runner from a committed tree, its fingerprint checked before
     any game.
  2. *The instrument:* a null pair of identical binaries that reads about
     zero, a registered pair read both there and on the match host that
     agrees within the intervals, and forfeits at `3+0.03` at the chosen
     concurrency no more often than on the host.
  3. *A tune longer than one job:* one registered run spread over jobs as
     large-batch steps (every job plays at the same parameters, a final
     job updates them, the next run continues from them), not independent
     chains averaged; PLAN rules 4 and 7c are unchanged.
  4. *The evidence:* each run's games and reports copied into
     `tools/results/` with a manifest before the artifacts expire.
  5. *The rules it changes:* AGENTS' idle host and pinned runner, and who
     starts a long run; speed and timing reads never run there.
  Falsifier and stop: a null pair away from zero, a cross-check outside
  the intervals or more forfeits than on the host rejects the lane, and
  it closes `NO_CHANGE`; the maintainer decides on adoption.

## Phase E — Classical checkpoint and release

- **E.0 Where the remaining strength is: speed, search, evaluation and how they fit each other, against the strongest hand-crafted engines — `R3`, DONE 2026-11-23.** Added
  2026-11-22 (maintainer request, after RAR-M91). Research only: zero
  games first, no code changed, nothing implemented. It ranks the levers;
  each one it proposes becomes a step of its own, numbered where it runs,
  and E.4 is ranked among them. It runs right after D.6's review and
  ahead of D.8 (maintainer decision 2026-11-22), so its ranking can reorder
  what remains of Phase D, D.12's board rebuild among it; its number
  stays E.0.
  **The question.** The merged kernel rates 100.2 below Stockfish 16 HCE
  in RAR-M91 (−103.3 [−121.0, −86.0] head to head) and 35 to 49 below
  Berserk 4.7.0 HCE and Stockfish 11. Which part of that gap can Rarog
  take, where, at what cost and in which order: the cost of a node, the
  tree the kernel searches, the evaluation, the fit of search and
  evaluation to each other, or elsewhere (clock, threads, tablebases, the
  board), alone or in combination.
  **What RAR-M91's games show** (its record, *Speed and depth by engine*):
  NPS and mean depth a move, Stockfish 16 HCE 2.16M and 25.2, Stockfish
  11 2.22M and 21.8, Berserk 2.41M and 19.8, Ethereal 14.40 HCE 2.99M and
  19.6, Basilisk 1.11.0 1.69M and 18.9, the kernel 1.58M and 15.0, Rarog
  2.7.0 1.52M and 12.7. These are leads, not measurements: each engine
  counts and reports in its own way, and Colosseum's depth includes moves
  with a mate score. Rarog is the slowest of the twelve. Since D.6.31 it
  counts nodes as Stockfish does, so Stockfish 16 HCE's 1.37 times its
  rate is one unit; Rarog's own slope at `3+0.03`, about 145 Elo a
  doubling (RAR-S98, RAR-S99), puts that ratio near +65 if all of it
  could be closed, which is not assumed, since the two evaluations cost
  differently. Depth does not order the field: Basilisk is 3.9 plies
  deeper than the kernel and 94.6 weaker, and the kernel is 2.3 plies
  deeper than 2.7.0 and 79 stronger. Depth is a symptom to explain, not a
  target.
  **Explanations to separate,** each with what already speaks to it:
  1. *The cost of a node:* evaluation, board and memory traffic. The last
     profile is D.7.8.12's (`analysis/d7812_profile_2026-11-20.md`); D.12
     owns the board's part.
  2. *The tree:* the kernel is Reckless's form, built for a network
     evaluation, its margins fitted on Rarog's evaluation at `3+0.03`
     (D.7.11). Against a search built for a hand-crafted evaluation doing
     better here: RAR-O08 put Stockfish classical's search on Rarog's
     evaluation 59.8 ± 8.1 below the head of the day.
  3. *The evaluation:* RAR-O07's same-search meter read 99 at C.11; RAR-O10
     read Rarog's evaluation as more accurate than Basilisk's against
     Stockfish 19, with no read against Stockfish 16 HCE's or Berserk's.
  4. *Search and evaluation together:* the evaluation's scale, noise and
     cost against the kernel's margins; `LazyMargin` not refitted on the
     kernel (*What stays unfitted*); the fit's labels from the B.9 head's
     games (RAR-M67; E.4's method question).
  5. *The instrument:* the node and depth conventions themselves.
  **The cheapest reads, zero games first:** NPS and depth by phase, mate
  moves out, from RAR-M91's database; the kernel's cost by component on
  the bench beside D.7.8.12's profile; RAR-O10's accuracy instrument on
  Stockfish 16 HCE's and Berserk's static evaluations beside Rarog's.
  Games only where they separate explanations, each registered with its
  use named first: the kernel against Stockfish 16 HCE at equal nodes
  (D.7.1.1's per-side budget, corrected for Rarog's driver discarding the
  interrupted iteration, RAR-O09) separates work a node from speed, and
  the same pair with Rarog's clock raised by the measured NPS ratio
  prices speed parity.
  **Leads from Coda** (added 2026-11-23;
  `analysis/coda_read_2026-11-23.md`, *Search and tooling leads*; its
  numbers are its own NNUE engine's short-control SPRTs and order tests
  only): a transition correction keyed by the hash of the last move in
  its position, which Rarog lacks (explanation 4); LMR adjusted by the
  child's static evaluation against alpha, +1.2 ± 1.0 there (2); two
  removals that gained, +1.9 ± 1.8 each, beside switches that record
  each feature's measured cost (2); each thread ordering its root by its
  own latest best move, +1.2 ± 1.4 at four threads (threads).
  **Output:** a short packet in `analysis/`, each lever with its evidence,
  its prior band frozen before any game, the cheapest test that kills it
  and the step that would own it; a lever without a discriminating read is
  listed as unknown, not ranked. The maintainer orders the steps it
  proposes before any is built.
  **The packet, 2026-11-23** (`analysis/e0_remaining_strength_2026-11-23.md`;
  zero games, no engine source changed). Read a move at a time, the leads
  above change: Stockfish 16 HCE searches 1.32 to 1.42 times the kernel's
  nodes a second (1.32 on game positions alone and with 14 running, so
  the gap is the node's own cost and not memory traffic), and 2.7 to 2.9
  plies deeper without mate scores, not ten. A profile of Stockfish 16's
  classical build beside D.7.8.12's puts two thirds of the 117 ns a node
  between them in search-side work (the node, move ordering, histories:
  186 ns against 108), 18 ns in the evaluation and nothing in the board
  (94 against 89). Stockfish 16's classical evaluation is statically no
  more accurate than Stockfish 11's on RAR-O10's instrument (−0.0044
  [−0.0087, +0.0005]); both lead Rarog's by 0.024 to 0.029. Inferred, not
  measured: the −100 is about −63 of node rate and −99 of evaluation
  (RAR-O07) against about +62 for the kernel's tree. The levers it ranks,
  bands frozen before any game: the evaluation in the donor's shape (+25
  to +70, killed by the donor's evaluation inside the kernel reading
  under +25), search-side work a node (+6 to +20), endgame knowledge (+3
  to +10), the fit's data (0 to +12), the evaluation's cost (0 to +8),
  the interrupted iteration (0 to +5: 4.0% of the kernel's time follows
  its last report, which reopens D.7.8.5's question), the search leads (0
  to +8) and the board rebuild (0 to +5, with no strength case left).
  **RAR-M92 is registered**, the one game read: the kernel on 1.35 times
  the clock against Stockfish 16 HCE, 1,600 games, the Elo price of the
  node rate. The equal-node match is not registered: under a node limit
  the kernel's last report covers 73% of its budget. E.0 stays open until
  RAR-M92 is read and the maintainer has ordered the steps the packet
  proposes: RAR-M92; D.16 with D.16.3's profile first; E.4.0 opened by
  the donor's evaluation inside the kernel; E.4's units; D.12 reduced to
  D.12.0.
  **Closed 2026-11-23.** RAR-M92 read the same day: the kernel on
  `4.050+0.041` −43.7 ± 11.8 Elo against Stockfish 16 HCE over 1,600
  games, no fault or time loss, a gain of +58.3 ± 17.3 over the
  baseline's −102.0 ± 12.7 (predicted +55), so 1% of node rate is worth
  about 1.67 Elo against the meter and the speed bands stand. The
  maintainer approved the packet's order the same day, every step keeping
  its number (the register's note of that date): D.16 with D.16.3.1
  first, E.4.0 with E.4.0.1 first, D.8 to D.11, D.12.0 alone with D.12.1
  to D.12.4 held, D.13 to D.17, then Phase E as numbered.
  **Superseded in part 2026-11-23 by D.16.3.1** (the head's fresh
  profile): the gap is 101.5 ns a node, three quarters search side, a
  third evaluation (107.1 against 75.6, not 18 ns), the table −7.3;
  lever 2's band, search-side work a node, is 0 to +6 (0% to 3.6% of
  rate), not +6 to +20: most of the search side is the tree's shape
  and the kernel's own mechanisms.
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
  stands for the release after it, and the review of D.6 after D.7 asks first whether
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
- **E.4 The hand-crafted evaluation in its donor's shape: the donor, a rebuild or repairs, endgame knowledge and the fit; the classical line's next release if it wins — `R3` card, then `I2`/`V`.**
  Reworked 2026-11-22 (maintainer decision, after D.6's review on the
  compact kernel): the review found Rarog's evaluation off its donor's
  shape in several places, most of them endgame knowledge, so the
  programme opens with research on the donor and the form, as D.7's
  research opened the search's rebuild, and the endgame items D.6 found
  are consolidated in it. Classical work resumes after Phases D and E's
  release (maintainer decision 2026-10-22), and E.0's ranking places this
  programme among the other levers. The sub-steps run in number order; the
  king-danger unit, E.4's whole content until this rework, is E.4.3.
  Every evaluation change here runs PROCESS's *Evaluation change under a
  fitted search*.
  *From D.7.11 (2026-11-20):* this is the first evaluation change under
  the fitted kernel, so its margin block (PROCESS, *Evaluation change
  under a fitted search*, restated at D.7.14) is where `LazyMargin` is
  refitted. It was last fitted with the classic search (RAR-S104, 423 to
  475, no gradient past noise), and D.7.11 held it at 475 with the
  kernel's margins fitted against it.
    - **E.4.0 The donor and the form — `R3`.** Research only, no engine
      change; added 2026-11-22 (maintainer decision). Its questions:
      1. *The donor.* Stockfish 11's classical evaluation, the design
         Rarog's follows today, or Stockfish 16's (`sf_16`), the last
         Stockfish shipped and Basilisk's donor, or `9587eeeb`'s
         (`SF_classical`), the last classical master. The evidence so far,
         Basilisk's included, is copied into
         `analysis/hce_donor_evidence_2026-11-22.md` so that it does not
         depend on Basilisk: in games `sf_16`'s and `9587eeeb`'s
         evaluations are equal within about 8 Elo in either search
         (BAS-O09), statically `9587eeeb`'s leads slightly (BAS-E58), and
         Basilisk kept `sf_16` by its frozen rule. Stockfish 11 was never
         measured against them, and Rarog's evaluation has not been read
         against either.
      2. *The form.* Rebuild Rarog's evaluation whole in the chosen donor's
         form, as D.7 rebuilt the search (one unit, parity with the donor by
         fixture before any fit, then one whole-surface fit and one gate),
         or repair it family by family. Decide by evidence in this engine:
         a static read of Rarog's evaluation against the donor's on the
         kernel's game positions by phase and by family (BAS-E58's
         method), and where Rarog's own terms carry information the
         donor's lack.
      3. *The interactions with the compact kernel:* its evaluation
         wrapper (the correction, the material scaling), `LazyMargin`, the
         draw verdicts, and the kernel's margins, fitted on today's
         evaluation (D.7.11).
      4. *The fitting method,* below, moved here from the refit.

      Output: a packet in `analysis/` whose decision is the maintainer's
      before anything is built; it adds the sub-steps the decision needs,
      and a rebuild takes E.4.2 and E.4.3 into its own units.
      **And the method itself (maintainer request, 2026-11-22).** Before the
      refit, research whether this pipeline is still the best way to fit
      Rarog's evaluation, and what would do better, by evidence in this
      engine: the labels (game results, or a blend with the kernel's own
      search scores; RAR-E03's Stockfish-distilled fit lost 17.11 Elo and
      stays the prior against distillation); the corpus (`hce-v4-tb`'s games
      were played by the B.9 head at 8,000 nodes, two searches ago, RAR-M67,
      against games of the kernel itself; RAR-E12 measured a corpus change
      alone at +11.8 ± 5.3 Elo); the optimiser (full-batch Adam on a fixed
      schedule against a converged second-order method); K by phase band; and
      fitting the evaluation together with the search margins it feeds
      (PROCESS's margin block) rather than after them. Held-out loss chooses
      candidates and accepts none (RAR-E03 improved it and lost); a method
      that changes the fit's result reaches E.4's gate, one that does not
      closes `NO_CHANGE`. E.0's ranking may reorder this work.
        - **E.4.0.1 The donor's evaluation inside the kernel — `I2`, then `V`.**
          Added 2026-11-23 (maintainer decision on E.0's packet), E.4.0's
          first read and RAR-O07's mirror: Stockfish's classical
          evaluation, `sf_11`'s and `sf_16`'s, called from the kernel
          through a diagnostic bridge that is never shipped, against the
          kernel on Rarog's evaluation. Both arms run Rarog's driver, so an
          equal-node read needs no correction. The bridge's price is
          measured as RAR-O08 measured its bridges', and each evaluation's
          scale is mapped by a rule frozen before any game. The reads,
          registered before any game: 2,000 games at equal nodes for each
          donor evaluation, then the better one on the clock with the
          bridge returned as time odds. Frozen in E.0's packet: under +25
          at equal nodes a rebuild is not indicated and E.4 is repairs;
          the programme's band is +25 to +70. It answers question 1 in
          play as well.
          *Built 2026-11-24* (`cf1c377` the `donor-eval` hook, `dbf265b`
          the bridge, `tools/donor-eval`; RAR-O13 registered before any
          game): both DLLs equal their tags' FEN setter on 10,039
          positions and the release binaries' `eval` to its printed
          precision; the hook idle keeps 3,694,298. The scale is K on
          results (684 and 717 in 1/1024): RAR-O08's mean rule, 0.28,
          was dominated by Stockfish's known-win scores. The kernel's
          trees on the donors' evaluations are 1.22 and 1.30 times its
          own at depth 14 on its game positions, with a tail in decided
          endings (seldepth past 90 where Stockfish's search reaches 20),
          a confound the read bundles.
          *Equal nodes read 2026-11-24* (RAR-O13): `sf_16`'s evaluation
          **+80.3 ± 10.8** Elo over the kernel on Rarog's, `sf_11`'s
          **+74.4 ± 10.6**, the two not separated; both above the +25
          rule, so a rebuild in the donor's form is not ruled out, at
          the top of the programme's band. Depth without mate scores was
          equal in the games. Waiting: the clock read with `sf_16`, its
          bridge price on an idle host; then E.4.0's decision.
    - **E.4.1 The tuner before any fit — `I1`.** The five checks below,
      added 2026-11-22 to the refit and moved here when E.4 became the
      programme, land before the programme's first fit.
      **The tuner before the refit (added 2026-11-22, from Basilisk's C.2.1.2,
      read on Rarog's tuner the same day).** Rarog's tuner already fits the
      played score: the lazy gate applies under `texel`, the base score is
      `evaluate()`'s own, `linear_delta_scale` calls the evaluator's scale
      rules in its order, and `--verify` compares against the evaluator's own
      sum, so Basilisk's defect is not here. Five gaps remain, each closed by
      a check that can fail:
      1. `fit_k` searches [0.5, 2.5] and says nothing at a bound: K at a bound
         fails the run.
      2. `clamp_weights` applies sign priors, passer ranks 0 and 7 at zero
         and a running maximum on the passer ranks to every coordinate after
         each step, and no manifest lists them: the run asserts that the
         clamps leave the seed unchanged, and the fitting manifest lists them.
      3. Rows the evaluator does not score linearly. A draw verdict scores 0
         with every coefficient 0, so it moves neither K nor a gradient and
         only adds a constant to the reported loss. The mop-up's rows (KBNK,
         KXK) carry large frozen terms and may move K, as Basilisk's override
         rows moved its K from 0.00606 to 0.00428: read K and the held-out
         loss with and without them on `hce-v4-tb`, and report the loss over
         the rows the fit can move.
      4. Adam moves a one-signed coordinate about `lr` an epoch, so 200 epochs
         at 0.3 bound its travel near 60 cp whatever the data say, and the
         tuner prints its best epoch but accepts a run whose best epoch is its
         last (Basilisk's T1): fail such a run, or one whose travel reaches 0.9
         of the bound.
      5. `extract.py` labels a game however it ended: reject by termination,
         as Basilisk's row contract does. No row of `hce-v4` changes (612,745
         normal endings, 2 adjudicated draws, no forfeit), so this binds only
         a new corpus.
    - **E.4.2 Endgame knowledge — `R2`, then `I2`/`V`.** Consolidated
      2026-11-22 (maintainer decision) from D.6's review on the compact
      kernel: D.6.9 (the endgame scale's target), D.6.10 (the pawnless
      default factor), D.6.21 (won KPK positions at or below 0), D.6.22
      (`krpkr_scale`'s hard zeros), D.6.23 (wrong-bishop draws through
      uncovered leaves), D.6.24 (the KQKP fortress's hard zeros), D.6.25
      (two same-coloured bishops above a queen) and D.6.26 (pawnless endings
      a minor ahead read as wins). Each keeps its card and screens in D.6.
      One Syzygy screen over all their families on the kernel's own game
      positions comes first (RAR-E36's drawn-overclaim method), D.6.10's
      families with it: 15% of the kernel's games reach a pawnless family
      Stockfish 11 scales and Rarog does not
      (`analysis/artifacts/d610-review-2026-11-22/`). Then repairs grouped by
      the families they touch, each gated as a repair (AGENTS, *Gating*).
      If E.4.0 decides a rebuild, these become its endgame cluster.
      Also for the refit: the endgame scale's target (D.6.9, moved here
      2026-11-22): fit both forms, the whole score scaled as now and the
      endgame half only as Stockfish 11 scales it, with the scale constants
      refitted under each, and let held-out loss and the gate choose; the
      forms differ mostly in pure opposite-bishop endings, 1.5% of the
      kernel's positions.
    - **E.4.3 The king-danger index rebuilt in the donor's structure — `R2` card, then `I2`/`V`.** The
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
      The bishop pair's three representations fold exactly
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
      If E.4.0 decides a rebuild, this unit folds into it.
## Phase F — NNUE (own data only)

**Rules.** Own data only, generated by Rarog's classical head and later by its
NNUE heads. Stockfish is the primary source for the runtime, the trainer
pipeline and the network's design, as for the search; Reckless, AGPL-3.0,
is read for ideas only (maintainer decisions 2026-11-23, the second
replacing the first's Reckless-and-Stockfish; PROCESS); Triumviratus, Gyatso and Trinity are
secondary inputs read as research. The architecture ladder is ours. The classical evaluation stays in the tree as the
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

**Inputs from Trinity's network path** (added 2026-11-23, maintainer
request; `analysis/trinity_read_2026-11-23.md`). Trinity holds CCRL 40/15
3522 at 1.0.0 with a plain search and a 768×8-king-bucket → 1024 → 8-output-bucket
SCReLU net trained with bullet on Leela-derived labels, so its data results
do not carry over as they stand. Its measurements are research inputs to
the leaves named, each to be confirmed here, and weigh below Stockfish:

1. run-to-run spread: three runs of one recipe that differed only in the
   seed read 0, −9.7 ± 4.9 and −0.2 ± 5.0 (a sample SD of about 5.5), and
   a pair read as one change had also changed the seed; F.3's two seeds per
   configuration stand, and the arms of a comparison share the seed and
   the data order (F.3, F.7);
2. held-out loss did not rank nets across recipes: 0.27% lower loss read
   −2.1 ± 2.9, 1.3% lower (L1 2048) −29.8 ± 4.2; whether holdout loss here
   ranks anything beyond the checkpoints of one run is a question F.3
   answers before loss chooses between recipes (F.3, F.6);
3. capacity carried to rated engines (king buckets +17, 512 → 1024 +50 in
   its calibration), more epochs or data on a converged net did not (about
   0; a restart at a high rate −13.8; checkpoint averages about 0): item 6
   above, confirmed independently (F.7);
4. L1 2048 cost 28% to 33% of nodes per second and lost at 3.65 billion
   positions; 1536 cost 19%: a first layer wider than 1024 starts with a
   negative prior at Rarog's corpus size (F.0, F.7);
5. a float-against-quantised read beside integer-exact conformance: mean
   |gap| 6.4 cp, and a bias of −4.1 to +5.2 cp by output bucket, so a
   capture across a bucket boundary moves about 7 cp (F.2);
6. length and the result weight interact: 900 superbatches beat 600 at a
   result weight of 0.5 (+5.7 ± 2.7), 0.25 beat 0.5 (+5.5 ± 3.8; λ = 0.75
   in this document's sense, the value Triumviratus's fix used), and length
   then added nothing (−1.0 ± 3.9); Rarog's λ and length come from its own
   curve (F.3);
7. self-play overstated network gains against rated engines by about 1.1×
   for capacity and up to 10× for more epochs (F.6);
8. a budget point: 600 superbatches of 16,384 × 6,104 positions took 5.0
   GPU-hours on an RTX 3080 (F.3, F.4).

**Licence notes from Coda** (added 2026-11-23;
`analysis/coda_read_2026-11-23.md`). Reckless is AGPL-3.0 and, since
2026-11-23, a reference for ideas only (PROCESS). Coda's one
finding of copied expression was in this phase's area: a threat-feature
index construction and SIMD kernels modelled on Reckless's. It
reimplemented both and kept the feature mapping, which its trained net
depends on, so a mapping a net is trained on is an interface chosen once
(F.5, F.7). The own-data rule keeps the data licence simple, where
Coda's LC0-trained nets owe ODbL attribution. Coda names its network
files by content hash with no architecture prefix, since the header
carries the architecture (F.6).

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
      *From D.7.11 (2026-11-20):* as an evaluation change it runs
      PROCESS's margin block on the kernel's seeds, `LazyMargin` in it.
      If E.4 did not change the evaluation, this is that constant's
      first refit on the kernel.
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
  *From D.7 (2026-11-19, `analysis/d7_audit_2026-11-19.md`):* D.7.12
  calibrates the printed score for the hand-crafted evaluation with the
  same model, so this step refits it and does not build it. The search's
  scale lives in the kernel's wrapper: its two coordinates are fitted
  first and alone on a new network, before the joint fit (on the HCE they
  were 44% of the first fit's gain, RAR-S129). Training targets record
  the kernel's score with the calibration in force, never the printed one.
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
  *From D.7.11 (2026-11-20; `analysis/d711_fit_2026-11-20.md`, What
  stays unfitted):* this is the next fit of the kernel's whole seed
  table, and it settles what D.7.11 left.
  - The sixteen whole-unit seeds (depths, counts and
    `see_noisy_square`) are owed a fit here: each read as a categorical
    step, or given a finer unit first. A perturbation of whole units on
    a seed of 3 to 7 is not repeated. The first fit found real
    gradients on three (`singular_depth` z +3.54).
  - The units stay held: three history limits and the pawn's exchange
    value scale with families the surface holds.
  - Rule 7c's shape is asked first whether 60 games an iteration serve
    it: the slots were busy 79.3% of an iteration at 30 and would be
    86.9% at 60, 10% more games an hour. Proven on a smoke before the
    registration.
  - `LazyMargin` has no successor on a network; with the HCE it is
    refitted by PROCESS's margin block (E.4, F.4.1).
- **F.7 Architecture ladder: output buckets, king buckets, relation/threat inputs; one axis at a time — `R3` with `I2`/`V` sub-steps.** Output buckets,
  king buckets with mirroring, then relation and threat inputs, one axis
  at a time (Stockfish's threat inputs the reference; Reckless's are ideas
  only, never its index construction or layouts, PROCESS 2026-11-23); each net gated against the previous.
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
    - **F.7.1 Two-timescale evaluator: a slow network sets the fast evaluator's later-layer weights; an optional rung beside the ladder — `R3`.** An
      optional research rung, added 2026-11-23 by maintainer decision: an
      alternative approach to test, not a replacement for F.7's ladder and
      not a condition of F.9. State `RESEARCH`; nothing is built until the
      zero-game screen below passes. *Mechanism.* Output buckets choose one
      of a few fixed sub-networks from a piece count. Here a deeper network
      reads the slowly varying part of the position (pawn structure,
      material, king placement) and outputs the weights or scales of the
      fast evaluator's later layers, so the depth an evaluator cannot afford
      at every node is paid once and shared across the tree. Two forms:
      keyed by a pawn-structure and material key and cached, which keeps the
      evaluation a function of the position; or computed once at the search
      root, which allows a far larger slow network but makes the evaluation
      depend on the root. A third leg, the last layer trained by gradient
      during the search, generalises `search/correction.rs` and waits on the
      first two. *Evidence and priors.* A prior-art search on 2026-11-23
      found close relatives only and no engine or paper doing either form
      (notes in `analysis/artifacts/two-timescale-eval-2026-11-23/`; Discord,
      Fishtest's test names and the shogi and xiangqi repositories were not
      searched, so this is "not found", not "untried"). Relatives: output
      buckets, Stockfish's dual net (retired in Stockfish 19), Ethereal's
      pawn-king network, and F.7's own Triumviratus rungs, the material-keyed
      experts and the cached pawn-king subnetwork. Against: the root's expert
      fixed for the search read −27.7 there, which is the root form in its
      crudest shape; a Stockfish developer reports that pawn-king add-ons
      have always failed in Stockfish; correction history's gain at the top
      is small. The leaf already sees the whole board, so the slow network
      adds capacity, not information. *Interactions.* It competes with
      output buckets and material experts for the same signal and is judged
      against a net that already has them; the root form changes stored
      evaluations between moves and so touches every consumer of a table
      score; a cache miss costs NPS, which F.5's pooled attribution prices.
      *Invariants.* Integer-exact conformance with the trainer (F.2); the
      cached form reproduces the uncached value on every position; own data
      only. *Falsifier, zero games.* Once F.3 and F.4 give a trainer and a
      corpus, train the F.7 net of the day and the same net with the cached
      conditioning on identical data and compare held-out loss. Stop rule:
      if the gain is not clearly larger than the output-bucket rung's own
      gain on the same split, close `NO_CHANGE` with no engine code written.
      A pass earns one registered SPRT against the same net without it, by
      F.7's rule; the root form is considered only after the cached form
      passes. Loss is not Elo: the screen can refuse, only the gate accepts.
      The prediction and the screen's threshold are frozen at that
      registration.
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
  *The thread spreads are owed a fit, on evidence* (from D.7.11,
  2026-11-20). `lmr_spread_thread` (27) and `fds_spread_thread` (26) are
  the donor's values. They multiply the thread index, so nothing at one
  thread reads them and no fit has held them. A four-thread tune plays
  about a fifth of the one-thread games an hour (three games at a time),
  so it is registered only if D.13's or D.14's four-thread reads, or
  RAR-M83's measure repeated on the adopted kernel, show a scaling
  deficit to chase: then these two coordinates, at four threads with the
  explicit placement (RAR-M82).
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
  *From D.7.11 (2026-11-20):* every fit so far ran at `3+0.03`. RAR-S134
  read the kernel's gain whole at `10+0.1` (1.02 of the `3+0.03` Elo), so
  no long-control refit is owed on that evidence. What a short control
  cannot fit is named, and is this step's first surface: the depth
  thresholds that act near its horizon (`fp_depth` 14, `bnfp_depth` 11,
  `nmp_verify_depth` 17, against a mean depth of 16.3 at `3+0.03` and
  20.2 at `10+0.1`), and the clock's floors (H.3).

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
  *The clock's constants are owed a fit* (from D.7.11, 2026-11-20;
  `analysis/d711_fit_2026-11-20.md`, section 1). Since D.7.9.1 the clock
  is Reckless's, and nothing in it was fitted on Rarog: the fifteen
  soft-limit constants (`TmNodesBase` to `TmChangesScale`), the budget's
  seven (`time.rs`, `TIME_OVERHEAD_MS` to `CYCLIC_HARD_SCALE`) and the
  fail-high stop's multiple. D.7.11 held them out of the `3+0.03` search
  fit. *Since 2026-11-23* they are AGPL constants that D.11.3
  resolves, and a fit seeded from them is still a port (PROCESS), so
  this fit follows D.11.3's treatment. The fit opens here, or earlier if D.7.13's `10+0.1` companion
  reads below its `3+0.03` reading by more than their combined
  intervals, or a rated control's games show a factor at one end on more
  than 90% of iterations. What its card starts from:
  - *Its own registration at `10+0.1` or longer, never inside a search
    fit.* The factors' population moves with the control: at the
    stopping iteration the product of the five mean factors is 1.66 at
    `3000+30` and 1.33 at `10000+100`, and 31% of sides live under
    100 ms at `3+0.03`, none at `10+0.1`. A tune there plays about 1,300
    games an hour, some 45 hours to a first checkpoint.
  - *The coordinates active at that control,* read again there by
    `tools/diag/tm_activation.py`. At `3+0.03` the three floors never
    bind; `TmPvFloor` binds on 2.1% of iterations at `10+0.1`.
  - *One scale, not six.* The four bases and the trend's two clamps
    each scale the same product, and Rarog's one clock fit (RAR-R15)
    found its only gradient on the scale.
  - *The trend factor's form is asked first.* On Rarog it is a switch:
    at a clamp on 79% of moves, its band 5.7 units wide against a
    median move-to-move change of 11. Fitting its slope fits a form
    that may not be the right one here.
  - *No fixed fault limit* (PROCESS, *SPSA go/no-go*, item 11): the
    clock's lowest forfeit slack is 54 ms, and an arm that spends more
    lives there longer.
- **H.4 Throughput at an identical tree on the network head — `R2`, then `I1`/`V`.**
  C.12's speed pass is closed and F.5 covers the network's inference. On the
  network head's profile, Triumviratus's board, SEE, picker, prefetch and
  build-layout leads (TB-4 to TB-8, TB-12, TB-14's layout flags; TB-16's
  negatives; its stack probes, TB-11, are D.8.19's) are read where the
  profile shows their cost, each with exact fingerprint and pooled-PGO NPS.
  The board's leads, TB-4 to TB-8, are read first by D.12 (added
  2026-11-14); this step keeps what D.12 does not take.
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
