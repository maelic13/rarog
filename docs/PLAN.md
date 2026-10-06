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
| Evaluation deficit with the same search | Stockfish's classical HCE beats Rarog's current HCE inside Stockfish's search by **+181.7 ± 19.0 Elo at equal nodes** (150,000 a move), Phase C's meter baseline, and by +266.3 ± 19.9 at equal time (`3+0.03`), measured 2026-10-05. RAR-O02's earlier **about 329** measured the 2.3.2 evaluation in about 205 games with the Rarog-evaluation arm at 1.5 Mnps against the control's 2.3 | RAR-O05; RAR-O02 |
| Speed | **3.19 MNPS pooled median** at bench 13, PGO pext 1T, ±0.2% instrument resolution (best-of 3.21, which is the 3.22 previously recorded); Basilisk 3.71; board work 24% of time, evaluation 29%, search loop 23% | RAR-M48; RAR-M36, RAR-M44 |
| Conversion | **88 draws and 19 losses** after holding a piece-up advantage for 12+ plies, in 3,600 games against the six HCE-era engines on the **2.4.0 release** games — 24.4 and 5.3 per 1,000, unchanged from the 2026-09-04 pool's 57/12 in 2,400 (23.8 and 5.0). Basilisk 1.9.3 in the same tournament: 94 and 12. **RAR-M47's surplus-over-Basilisk reading is not reproduced and is retired**; the stable finding is Rarog's own rate, 80 of the 88 draws by fifty-move or repetition with material in hand; a third independent sample reads 24.2 and 3.3 per 1,000 (RAR-M54, 1,200 games against the same six) | RAR-M49 (release re-read, tournament `5e539523`); RAR-M54 (Super Rating Tournament, 42 engines); instrument RAR-M47 |
| Fingerprint | `bench 13` **11,171,726 / EBF 2.512**: cluster 3 (`b4quiet`: the stored PV bit on quiescence stores, fail-high interpolation, the count rule, margins as coordinates, evasion pruning) with RAR-S86's theta, accepted by RAR-S88's gate on 2026-09-30 at +4.4 ± 2.9 Elo and the default since `5a5c150`; before it cluster 2 (`b3proof`) with RAR-S83's theta read 12,897,901 / EBF 2.523 (`f53ca7d`, accepted 2026-09-27 at +50.5 ± 10.9, RAR-S84); before it the selectivity core with RAR-S78's theta read 7,435,006 / EBF 2.457 (`52c46df`, accepted 2026-09-25 at +13.1 ± 5.4, RAR-S78); before it the B.2.3-fitted core read 7,185,678 / EBF 2.444, the default since B.2.4b (2026-09-20, `a47e85b`). `--no-default-features` compiled the superseded B.1 search, which read 7,590,542 / EBF 2.473 since `5a5c150` (three quiescence thresholds shared with the core took their fitted values); before that it read 7,601,220 / EBF 2.474 — the 2.4.0 fingerprint, accepted by RAR-E15 and reproduced in A.7, A.8.3 and A.8.4 — until B.8 deleted it in `84de712` (2026-10-03); B.9 froze the search head at 11,171,726 / EBF 2.512 | GUIDE checkpoint; RAR-S73; RAR-M48 manifests |

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
| C evaluation | 182 same-search at equal nodes, 266 at equal time (RAR-O05; RAR-O02's 329 was the 2.3.2 evaluation) | 100–160 | six families, whole-surface refits, endgame conversion. **C.0 (2026-10-05):** C.0's screen supports two family clusters (king safety, winnability and scale), not six, and its prior for the family clusters together is 30–90 (`analysis/eval_programme_2026-10-05.md`). The band is corrected at C.11 by measurement |
| D clock, SMP, robustness | unmeasured | 15–40 | Reckless-shaped node-fraction TM; 4T quality |
| Speed inside B and C | — | 10–30 | per-node cost of the new search and evaluation modules. **B.9:** the B head is 26.2% slower than 2.4.0 in pooled bench NPS (RAR-P35), after B.7 won back about 20% inside B (+35.5 Elo, RAR-S98); B's gates already include that cost and that recovery, so this row recorded nothing of B's separately and now applies to C |

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

### Active workflow register

One row per open leaf in the active phase. The checker requires the state
and class here to match GUIDE's suffix. Phases A and B are closed; Phase C
is the active phase since C.0 opened it (2026-10-05), and `check_guide.py`'s
`ACTIVE_PREFIXES` names it. Later phases carry only a class until
they open.

| Leaf | Workflow state | Class | Current decision |
|---|---|---|---|
| C.1 | READY_FOR_IMPLEMENTATION | I1 | Handoff frozen by C.0 (`analysis/eval_programme_2026-10-05.md`, section 8); exact fingerprint required |
| C.2 | READY_FOR_IMPLEMENTATION | V | Protocol frozen by C.0 (section 9): corpus `hce-v4`; the baseline refit gated by PROCESS's shape (C.0 audit). C.0.3 is decided, so generation can run in parallel with C.1 once its command is prepared and wire-checked |
| C.3.1 | READY_FOR_IMPLEMENTATION | I2 | Handoff frozen by C.0.4 (`analysis/c04_king_unit_2026-10-06.md`); after C.1 |
| C.3.2 | READY_FOR_IMPLEMENTATION | I2 | Same handoff; the pawn-cache entry grows by 12 bytes |
| C.3.3 | READY_FOR_IMPLEMENTATION | I1 | Tuner changes the handoff names; no engine code |
| C.3.4 | READY_FOR_IMPLEMENTATION | V | Protocol frozen (cluster shape; the card's prediction); after C.3.1–C.3.3 and C.2's corpus |
| C.3.5 | RESEARCH | V | After gate 1 or a flagged tree read |
| C.3.6 | RESEARCH | V | After C.3.5 accepts; its reading decides whether C.4 to C.7 open |
| C.4 | RESEARCH | I2 | After C.5.2; opens with its own residual step (RAR-E17: threats +0.07%, mobility +0.01%) and closes `NO_CHANGE` if it finds none |
| C.5.1 | RESEARCH | R2 | The excess above six men was an artefact (RAR-E22); first the opposite-bishop refit candidate on the drawn-cohort instrument, then the family order by C.5's instruments |
| C.5.2 | RESEARCH | I2 | `NO_CHANGE` for the first unit (RAR-E21); open for what C.5.1's cut supports, gated by C.5's instruments |
| C.5.3 | RESEARCH | I2 | After C.5.2 |
| C.5.4 | RESEARCH | I2 | After C.5.2 |
| C.5.5 | RESEARCH | R2 | After C.5.2 |
| C.5.6 | RESEARCH | R2 | After C.5.2 |
| C.5.7 | RESEARCH | I1 | After C.5.2 |
| C.5.8 | RESEARCH | V | Closes C.5 |
| C.6 | RESEARCH | I2 | Opens with its own residual step (RAR-E17: passed +0.06%, pawns +0.04%); owns the unstoppable-passer tempo defect |
| C.7 | RESEARCH | I2 | Opens with its own residual step (RAR-E17: pieces and material nil) |
| C.8 | RESEARCH | V | After the family clusters |
| C.9 | RESEARCH | V | Conditional on a non-flat surface |
| C.10 | RESEARCH | V | After C.9 or its skip |
| C.11 | RESEARCH | V | Closes the programme; freezes the classical evaluation |

## Phase C — Evaluation programme (search frozen)

**Goal:** recover the measured same-search evaluation deficit (+181.7 ± 19.0
Elo at equal nodes, +266.3 ± 19.9 at equal time; RAR-O05), with
the search frozen at the B.9 head, by re-implementing the evaluation families
in Stockfish 11's classical shape where its conditioning is stronger, keeping
ours where the evidence says ours is better, refitting the whole surface
after every family cluster, and giving endgame handling its own bounded
cluster. **Meter caveat (C.0 audit, 2026-10-06; RAR-E22):** the +181.7 was
measured inside Stockfish's search, which is fitted to Stockfish's
evaluation shape, and a shape mismatch alone costs about 105 Elo per node
in a fitted search (RAR-E19, RAR-E20); with one Stockfish version the
donor's whole static edge above six men is 2.73% of held-out loss, king
0.75 of it. The recoverable share of the deficit is unknown, the budget
row's 100 to 160 and C.0's 30 to 90 are unsupported in either direction,
and the same-search deficit is re-read after the first accepted unit
(C.3.6), not only at C.11, before C.4 to C.7 open.

**What C.0 found (2026-10-05; `analysis/eval_programme_2026-10-05.md`).**
(1) The 329 is RAR-O02's figure for the **2.3.2** evaluation, about 205
games, with the Rarog-evaluation arm at 1.5 Mnps against the control's 2.3;
RAR-O05 (C.0.1) re-measured the gap on the current evaluation: **+181.7 ±
19.0 Elo at equal nodes** and +266.3 ± 19.9 at equal time (980 games; the
match was interrupted by an application update and is recorded, not
replayed). The equal-node figure is this phase's meter baseline; it is 150
or more, so the programme's premise stands and the order of work is
unchanged. (2) On 194,444 held-out positions Stockfish's total adds 3.59% to
Rarog's outcome prediction and its regular term families 1.22%; king safety
is the one family that stands out (+0.66%, +1.58% in the middlegame band),
winnability carries a small signal confined to low material, and mobility,
pieces, material and space carry none (RAR-E17, RAR-E18; static loss, which
ranks questions and accepts nothing). With one Stockfish version for
families and total the families read 1.85% over all rows and carry the whole
2.73% at seven men or more; the rest of the total's edge is exact endgame
knowledge and magnitude at six men or fewer (RAR-E22, 2026-10-06). (3) The
played evaluation omits
imbalance above its lazy gate, and the unstoppable-passer test is one tempo
generous. **Order of work:** C.0.1, C.0.3, C.0.4, C.1, C.2, the C.3 unit (with
C.5.1 and C.5.2), then C.4, the rest of C.5, C.6 and C.7, each of which
opens with its own residual step and closes `NO_CHANGE` if that step finds
none. C.0's
prior for the family clusters together is 30 to 90 Elo, below the budget
row's 100 to 160; the row is corrected at C.11 by measurement, not here.

**The search is frozen for the whole phase (B.9, 2026-10-03; amended
2026-10-06).** Phase C changes no search code. It changes no search
coordinate except (a) through C.10's joint tune and (b) through the
**margin block** attached to an evaluation unit's gate (cluster shape,
step 6): one registered SPSA block over the fixed surface of cp-valued
search coordinates named in PROCESS, *Evaluation change under a fitted
search*, evaluation weights fixed, seeded from the head. The argument rule
7b asks for is measured: a statically better evaluation lost −104.5 ± 10.6
Elo at equal time and −110.0 ± 11.5 at equal nodes through the search fitted
to the old one (RAR-E19, RAR-E20). A C-phase change that touches
`src/search/` returns to its owner leaf with an explicit reason recorded
there, and is gated as a search change. **Exception, maintainer decision
2026-10-06:** D.1.1's time-management repair lands before C.2's
baseline-refit gate and C.3's first gate, gated as a search change. It
moves only clock play (`bench 13` unchanged), and its accepted head
becomes this phase's frozen head, recorded here. **D.1.1 closed
`NO_CHANGE` on 2026-10-06 (RAR-R13): no repair lands, and the frozen head
stays at engine source `ee02ed1`.**

**Units, not families (maintainer decision 2026-10-06).** Evaluation terms
are made to work together by the whole-surface fit; what they interact
with is the search. The unit of implementation and acceptance is therefore
every term that shares inputs and moves the score's shape, built together:
the **first unit is C.3**, king safety in the donor's shape with
winnability and scaling (C.5.1 and C.5.2's content) and the shared inputs
they need, one fit, one margin block, one gate. Attribution inside a unit
uses family masks for zero-game tree reads or 2,000-game reads, never
gates. Threats, pawns and passers, and pieces (C.4, C.6, C.7) join a unit
only when their own evidence step finds a residual on the programme
corpus; otherwise they close `NO_CHANGE`. A wholesale port of the donor's
evaluation was considered and declined: the screen finds the donor's
information in two places, and broad transplants have lost more often than
won (Basilisk 5.9 −77.9; Manta MAN-E05/E07 −16.3/−7.0; the unit that won,
MAN-E19 +35.9, was an audited set with one constrained fit). **C.0.4
(2026-10-06):** the first unit is king safety alone; the scale factor and
the complexity term were measured as directions and not built (RAR-E21),
so C.5.1 and C.5.2 return to C.5's sequence.

**Time control (maintainer decision 2026-10-06).** Gate 2 at `3+0.03`,
`[0,3]`, accepts. Every accepted unit gets a `10+0.1` direction read
(1,000 games, `match-fixed-ltc.toml`); a negative read reopens the unit. A
unit that fails gate 2 inside the harm bound (above −5 Elo) and whose cost
is nodes rather than speed may be re-gated once at `10+0.1`, `[0,3]`,
registered before the read and never a second time. The frozen head: engine source `ee02ed1` (no engine input changed
through 2.5.0 and `d6998db`'s version bump; the behaviour-neutral source
changes since are listed under D.3's change log with their exact
fingerprints, latest `6433760`, and a registration names the revision it
builds from); the measured binary
`tools/test_engines/rarog-b9head-pext-pgo.exe`, built at `24aefb4` (clean,
`rustc 1.98.1`, pext PGO), SHA-256
`aac921141d78d202603d0810985389451c0969222e20874c3842b128905701ee`,
fingerprint **11,171,726 / EBF 2.512**. C.11's same-search deficit and every
C gate measure against this head or its accepted successors.

**Speed, a secondary requirement (maintainer decision 2026-10-03).**
Strength is primary and stays so; the gates are equal-time, so a family that
costs nodes pays for them in its own SPRT. What speed adds as a requirement:
(1) C.1's restructure keeps pooled NPS inside ±0.5% as written; (2) every
family cluster's design states its per-node cost (which inputs it shares,
what it recomputes) before implementation, and prefers the shared attack-map
and mobility producer to its own passes; (3) each accepted cluster's record
carries a pooled-PGO NPS reading beside its gate, so the evaluation's cost is
visible as it grows; (4) C.11 records the evaluation's share of search time
against B.9's 24% of samples, and a share above it is explained there, not
accepted silently. RAR-S98 measured about 2 Elo per 1% NPS at `3+0.03` on this
engine, which is what a node spent in evaluation costs.

**Why Stockfish 11 here.** Reckless has no hand-crafted evaluation. Stockfish
11 is the last classical Stockfish and the reference the maturity record
already compares against (`analysis/hce_maturity_2026-08-25.md`). Its
families and their conditioning are the donor; its constants are seeds on a
different scale and ride the next Texel refit.

**Cluster shape for C.** Each family cluster: (1) the family's current terms
traced and their activation and residual measured on the fitting corpus by
cohort (king danger by attacker count, passers by rank and blocker, threats
by piece pair, endgames by material signature); (2) the donor's form of the
family read for its conditioning and populations; (3) a design that states
which terms are replaced, which stay, and which neighbouring families share
inputs (attack maps, mobility areas, pawn structure) so that the shared inputs
are computed once; (4) implementation with `EvalTrace` coverage for every new
slot and the reconstruction test; (5) a whole-surface Texel refit with the
existing toolchain, frozen test reported once; (6) a fixed-depth tree read on game positions beside `bench 13`
(C.0.3: about 70 positions from the latest gate's games, depth 12, both
arms, the per-position ratio distribution and the correction-residual
counters; `bench 13` under-reads tree growth, 3.2% against 8.1% and 22%
in middlegames), then a PGO bake and **gate 1**, SPRT `[0,3]` at `3+0.03`
with the search unchanged; when gate 1 fails, or the tree read flagged the
candidate, the **margin block** (one rule-7c block, PROCESS's surface,
evaluation weights fixed) and then **gate 2**, the candidate with its
retuned margins against the head, SPRT `[0,3]`; gate 2 accepts. A unit
that fails both is a worse function; one that fails gate 1 and passes gate
2 is accepted with its margins, and the record says what they moved. An
equal-node companion read (`colosseum.ps1 -Mode match -Nodes 150000`,
2,000 games) may be registered to tell a loss of time from a loss per node
(RAR-E19 and RAR-E20 read −104.5 and −110.0); it accepts nothing. An
accepted unit then gets the `10+0.1` direction read
(`[3,10]` when the family's residual is large); (7) ledger row. Fit loss is a
screen and a falsifier, never acceptance (RAR-E03 lost 17 Elo with better
loss).

- **C.0 Investigation: family map, residuals, donor conditioning, shared inputs, cluster order, refit protocol — `R3`, DONE 2026-10-06.**
  Produce the evaluation programme document (C.0 names it): the six-family map from the
  maturity record refreshed on the B.9 head, per-family residual and
  activation evidence, the donor comparison of conditioning, the shared-input
  plan, the cluster order by expected value, the datagen and refit protocol
  for the programme (corpus name, size, splits, label policy from the label
  audit), and frozen handoffs for C.1 and the first family cluster. **No
  engine implementation.** The document is
  `analysis/eval_programme_2026-10-05.md`. Its verdict (2026-10-05): C.1 and
  C.2 `READY_FOR_IMPLEMENTATION`; king safety confirmed as the first family
  cluster with its handoff **not** frozen (`MORE_RESEARCH`, C.0.4). C.0
  closes when C.0.1 is played and C.0.4 has frozen C.3's handoff. **Closed
  2026-10-06:** C.0.1 played (RAR-O05) and C.0.4 froze C.3's handoff
  (`analysis/c04_king_unit_2026-10-06.md`, RAR-E21).
    - **C.0.1 Evaluation meter at the phase start: the same-search gap at equal nodes and at equal time (RAR-O05) — `V`, DONE 2026-10-05.** The oracle
      package on both sides, `Use Rarog HCE` false against true: 1,000 games
      at 150,000 nodes a move, then 1,000 at `3+0.03`, no adjudication.
      Maintainer-run from `analysis/artifacts/c0-meter/run_all.ps1`. The
      equal-node figure replaces "about 329" as this phase's meter baseline
      here and in GUIDE; under 100 Elo the family clusters are re-scoped
      with the maintainer before C.3 is built. **Read 2026-10-05:** +181.7 ±
      19.0 Elo at equal nodes, +266.3 ± 19.9 at equal time (980 games,
      interrupted, recorded and not replayed); the equal-node figure is the
      meter baseline above, and the premise stands.
    - **C.0.2 Donor-direction residual screen (RAR-E17, RAR-E18) — `V`, DONE 2026-10-05.** `tools/diag/donor_residual.py`: held-out outcome
      loss of Rarog's score with Stockfish's total or one of its term
      families added. Readings in the two entries and in the programme
      document, section 5.
    - **C.0.3 Lazy path: the played evaluation omits imbalance above its gate; measure, then remove, repair or keep — `R2`, DONE 2026-10-06.** Above
      `LazyMargin` (600) `evaluate` skips imbalance and the whole
      piece-activity block (mobility, threats, king safety, hanging pieces,
      the small terms, the bishop pair; confirmed in `src/eval.rs`
      2026-10-05), which RAR-E06 and RAR-E12 fitted to piece-value-sized
      amounts, while every fit runs with the shortcut off. Cheapest test, no code: the head
      built with `tune`, `LazyMargin=2000` against 600: `bench 13` and
      pooled NPS for the cost, then a registered 2,000-game fixed read
      (AGENTS' *Gating*, repair case 2). An evaluation change, so it is
      registered and gated like one; its outcome is in place before C.1
      moves the code and before C.2 generates. **Registered 2026-10-05 as
      RAR-E19** after the cost read on a PGO tune pool of the head:
      `LazyMargin` 2000 costs 7.52% NPS (95% 6.29% .. 8.74%) and grows
      `bench 13` by 3.23% nodes. The read is Colosseum, 2,000 games, harm
      at −13 nElo; harm keeps the shortcut and opens a speed-keeping repair,
      no harm removes the lazy path before C.1 (the entry holds the rule).
      **Read 2026-10-05: harm, −104.5 ± 10.6 Elo** (nElo −159.1 ± 15.2), far
      beyond the speed cost (−5.5% in the games); the full function grows the
      frozen search's tree (+63% nodes at depth 14 on a middlegame position)
      and it searches shallower. The shortcut stays at 600 and C.0.3 is
      research again: the open question is the mechanism, and whether C.2's
      fits should run with the lazy path on so that they describe the
      function the engine plays. **Research 2026-10-06**
      (`analysis/c03_lazy_research_2026-10-06.md`), zero games: over 72
      positions from RAR-E19's games the depth-12 tree grows **8.1%**
      (geometric mean 1.06; +22% in middlegames, −12% in endgames; the +63%
      was one position), interior nodes +9%, quiescence +6%, stand-pat cuts
      +1%, razoring +43%, quiescence delta pruning ×10.7, aspiration
      fail-lows +17%, correction residual per update +18%. Statically the
      full function is the **better** outcome predictor above the gate
      (loss 24% lower on those 16.6% of validation rows) and more extreme
      there by a median 282 cp: the harm is in how the frozen search
      consumes it, not in static quality. **Decision:** the played function
      is `NO_CHANGE` (the imbalance-before-the-gate repair adds magnitude
      where magnitude is punished and is not built); the fits describe the
      played function from C.2 on (the `texel` build honours `LazyMargin`;
      handoff in the card); the fixed-depth tree read on game positions is
      a standing screen (cluster shape, step 6). **RAR-E20** (equal nodes,
      2,000 games, maintainer-run) is registered to read the per-node share
      of the −104; the leaf closes on its record. **RAR-E20 read 2026-10-06:
      −110.0 ± 11.5 Elo at equal nodes**, equal to the equal-time loss
      within error: speed and tree size bought the lazy arm nothing; the
      whole cost is per node, the frozen search deciding worse with the
      full function's values. C.0.3 is closed: the played function
      unchanged, parity in the fits from C.2, the tree read and the
      equal-node companion as standing instruments (cluster shape, step 6).
      Still pending the maintainer: a bounded cp-margin retune before C.10
      if two clusters show the mismatch; neither read separates a search
      fitted to the old function from a function that is worse for this
      search, and only that retune can.
    - **C.0.4 Research card for the first evaluation unit: king safety with winnability and scaling; sub-term attribution, shared inputs, the magnitude contract; freezes C.3's handoff — `R3`, DONE 2026-10-06.** RAR-E17 says
      that Stockfish's king term carries a residual, not which part
      (shelter and storm, the danger index and its map, flank terms,
      king-to-pawn distance). The card attributes it (the oracle's trace
      extended with the sub-terms, or Rarog's own zero-weighted inputs
      tested as directions), explains why `ks_weak_ring`, `ks_flank_attack`
      and `ks_shelter_storm` were fitted to zero, then freezes the design,
      the shared inputs it needs with their per-node cost, the prediction
      (with the `bench 13` change) and the falsifier. Scope in the programme
      document, section 11. **Widened 2026-10-06** to the first unit: the
      card also freezes the winnability and scaling design (C.5.1 and
      C.5.2's content), the shared inputs the unit needs in C.1's producer
      and the pawn cache, the material table decision, the family masks for
      attribution, and the unit's **magnitude contract**: how its terms keep
      the score's distribution by |score| band where the frozen search
      expects it (RAR-E19/E20: a shape change in decided positions cost
      about 105 Elo per node), measured by the full-vs-played method before
      any game. **Done 2026-10-06 (RAR-E21,
      `analysis/c04_king_unit_2026-10-06.md`):** the danger map carries 92%
      of the king residual alone (+0.604 of +0.655%) and, inside it, the
      safe checks and the weak ring the most; shelter and storm 20%; the
      flank terms and the king-to-pawn distance nil. Rarog's three zeroed
      inputs are active and not collinear with the units: one unit of
      weight moves a king one bucket, onto the equal twin of a table
      fitted in pairs, so the zero is a resolution limit of the index,
      not evidence. The scale factor adds +0.09% at seven men or more and the
      complexity term +0.01%: C.5.2's generic scaling is `NO_CHANGE` for
      the first unit, and the donor's excess above six men was an artefact
      of RAR-E17's two Stockfish versions (RAR-E22). The magnitude contract, the
      frozen prediction and C.3's handoff are in the card; C.3 is
      `READY_FOR_IMPLEMENTATION` as king safety alone, after C.1 and C.2.
- **C.1 Evaluation restructure, behaviour-neutral: modules, one attack-map producer, `eval/params.rs`, `kpk` under `endgame/`; exact fingerprint — `I1`.** Split `eval.rs`
  into the target modules; one attack-map and mobility-area producer consumed
  by pieces, king, threats and space; `EvalTrace` unchanged in meaning. Exact
  fingerprint, suites, pooled NPS inside ±0.5%. The move table is the C.1
  handoff in `analysis/consolidation_2026-09-10.md`. Specifics owed here:
  `eval/attacks.rs` is cut out of the 531-line `eval_piece_activity`, which
  also yields pieces, threats and the king-safety inputs — legal only if the
  running `mg`/`eg` order is preserved, because the mop-up and the lazy gate
  read partial sums; the producer's `attacks_from_sq` reads get the
  `debug_assert!` the 2026-08-19 audit asked for; `eval_params!` with its
  137 entries and the `tune`/`texel` I/O move to `eval/params.rs` and
  `eval/trace.rs`; `src/kpk.rs` moves to `eval/endgame/`; B.8 deleted `diag_lazy_dual`
  and the 21 `lazy_*` counters (no owner), so if C.1 keeps a lazy path and
  wants that instrument to decide `lazy_margin`, it restores them from
  `10d0e83` (`analysis/b8_removed_2026-10-03.md`, entry 5). The `texel` trace-reconstruction test is
  part of the suite run, never a speed measurement. **Handoff frozen by
  C.0** (`analysis/eval_programme_2026-10-05.md`, section 8): its module
  table at `ee02ed1`'s line numbers supersedes the consolidation
  document's; it names the three running-sum reads to preserve (the lazy
  gate, the mop-up, the initiative term's sign) and the public paths that
  must stay; the fit tooling that patches `src/eval.rs` by path
  (`bake_params.py`, `fit_complete.ps1`, `confirm_hce_fit.ps1`) moves to
  `src/eval/params.rs` (planned) in a tooling commit of this step; and one targeted
  check joins the fingerprint, because `bench 13` barely reaches the lazy
  path and the recognisers: every static evaluation of
  `hce-v3-tb/validation.csv`, hashed before and after.
- **C.2 Datagen and label contract for the programme; corpus frozen under a new name; fitting manifest (free/fixed/excluded) — `V`.** Generate the
  programme's corpus with the B.9 search under the adjudication-off datagen
  profile; audit labels against tablebase truth (existing tool); freeze
  splits and manifests under a new corpus name. Records the label-contradiction
  rate and the corpus hash. Maintainer-run generation. Also produces the
  programme's **fitting manifest**: every evaluation coefficient named with a
  status of *free* (receives gradient), *fixed* (structure: phase divisors,
  caps, sentinels) or *excluded* (nonlinear blocks such as king danger, whose
  caps and truncation a linear model would misrepresent), each exclusion with
  its reason and its contribution carried as a fixed residual per sample. The
  tuner reads the manifest; the hand-kept frozen list in
  `tools/texel-tuner` is replaced by it. Adopted from Manta's
  `manta-hce-fit-v3` (1,109 free, 17 fixed, 103 excluded). **Protocol
  frozen by C.0** (programme document, section 9): corpus `hce-v4` and its
  relabelled `hce-v4-tb`; the accepted head as the datagen engine;
  `datagen-v2` at 8,000 nodes a move; `phase_book_v1.epd` from start 1, so
  every start keeps its split; 3,500,000 / 194,444 / 194,444 rows, the game
  count from the preflight; rows of six men or fewer relabelled by Syzygy;
  `datagen-v3` not adopted. **From C.2 on the fits describe the played
  function** (C.0.3, 2026-10-06): the `texel` build honours `LazyMargin`
  instead of forcing the gate off, a change in the `texel` path only,
  fingerprint-neutral by construction, with the tests the card names; its
  effect on the weights is measured inside the baseline gate below and
  stated there as conflated with the corpus. C.2 then **refits the unchanged surface on
  `hce-v4-tb` and gates it against the head by PROCESS's *Evaluation
  change under a fitted search*** (the static screens, the tree read,
  gate 1 `[0,3]`, the margin block when gate 1 fails or the tree read
  flags, gate 2; RAR-E12's refit alone grew the tree 12.3%, so a bare gate
  would count a search mismatch as a rejection; C.0 audit, 2026-10-06)
  before any structural
  cluster: the attribution baseline without which C.3's gate would conflate
  corpus, labelling search and structure (RAR-E12 measured +11.8 Elo from a
  corpus change alone). It also re-reads RAR-E17's screen on the new
  validation rows, registered before they are scored.
- **C.3 First evaluation unit: king safety (danger units, safe/unsafe checks, weak ring, flank, shelter/storm) with winnability and scaling (C.5.1, C.5.2); shared inputs; one refit; margin block; one gate — `I2`, then `V`.** King danger in the donor's
  shape: attacker units and weights, safe and unsafe checks by piece type,
  weak squares in the king ring, king-flank attacks and defence, shelter and
  storm by file with the castling-destination alternative, queen-absent
  reduction, and the nonlinear danger-to-score map. Rarog's existing nonlinear
  danger table is the seed for the map. Refit, gate. The list above is the
  donor's inventory, not the cluster's content: C.0.4's card selects from
  it by measured residual and freezes the handoff. RAR-E17 read this
  family's donor-direction residual at +0.66% of held-out loss (+1.58% in
  the middlegame band), the largest of any family. **Unit (2026-10-06):**
  C.3 builds king safety together with C.5.1's classification and C.5.2's
  winnability and scale factor, and the shared inputs both need, and gates
  them once by the cluster shape above (gate 1, margin block, gate 2, the
  `10+0.1` read); C.5.1 and C.5.2 keep their IDs as its parts and tick
  with it. **Handoff frozen by C.0.4 (2026-10-06; RAR-E21;
  `analysis/c04_king_unit_2026-10-06.md`):** the unit is king safety alone
  (C.5.2's generic scaling measured and not built, so C.5.1 and C.5.2 no
  longer tick with it): the donor's danger index in the donor's units
  (accumulated attackers, weak ring, safe checks by piece type with the
  exclusions, unsafe checks, pinned blockers, king-adjacent attacks, the
  mobility difference, the no-queen and knight-defender reductions, the
  shelter feedback) through a capped quadratic map traced as two linear
  scales; shelter and storm by file and rank with the castling
  destination in the pawn cache; the pawnless flank outside the index;
  the 40-bucket table, its inputs and the linear shelter and storm terms
  removed; the flank terms and the king-to-pawn distance not built. The
  shared inputs with their per-node cost, the family masks
  (`KingDangerMask`, `KingShelterMask`, `tune` builds only), the magnitude
  contract (`KS_INDEX_CAP` 1600 under a const assertion; the
  full-vs-played flag rule: a mean shift toward the sign above 20 cp in
  any band ≤ 600 or 2 points of change in the share above the lazy gate
  sends the candidate to the margin block before gate 1), the fixture
  test against the donor's printed components, the frozen prediction
  (gate 1 +4 Elo, 80% band [−8, +15], pass probability 0.35; gate 2
  0.55; depth-12 tree +3% to +12%; NPS within ±1%) and the falsifiers
  (the king family's residual against the candidate above +0.40% returns
  the leaf to `RESEARCH` before any game) are in the card. Depends on
  C.1 (the producer) and C.2 (the corpus and the fits describing the
  played function).
    - **C.3.1 King danger in the donor's shape: ring, accumulated attackers, weak ring, safe and unsafe checks, blockers, king-adjacent attacks, the reductions, the capped quadratic map; the old table and inputs removed; the fixture test — `I2`.** Semantics,
      invariants and tests in the card's handoff; the blockers are the one
      input the pooled NPS read may drop.
    - **C.3.2 Shelter and storm by file and rank with the castling destination in the pawn cache; the linear terms replaced; the pawnless flank outside the index — `I2`.** The
      entry grows by the king square, the castling rights and the shelter
      score per side (12 bytes) and stays `Copy`.
    - **C.3.3 Tuner: the nonlinear pass over the index coordinates in index units, the map scales and shelter tables in the linear groups, the two family masks, feature-support coverage — `I1`.**
    - **C.3.4 Refit on `hce-v4-tb`, static screens (the king family's residual re-read, the magnitude read), the tree read, PGO bake, gate-1 registration and handover — `V`.** The
      registration copies the card's prediction before the static screens
      are read.
    - **C.3.5 Margin block, gate 2, the `10+0.1` read, the ledger row — `V`.**
    - **C.3.6 Same-search deficit re-read on the accepted unit: the oracle package rebuilt with the C.3 evaluation in `rarog_hce.dll`, 1,000 games at 150,000 nodes a move against the Stockfish control; decides whether C.4 to C.7 open — `V`.** C.0.1's
      recipe (`analysis/artifacts/c0-meter/run_all.ps1`, the equal-node
      match) with the DLL rebuilt from the C.3 head by the `oracle/hybrid`
      tag's `build.ps1`; registered with a frozen prediction. The reading
      corrects the budget row and C.0's prior; the next unit opens only on
      its own residual evidence and on what this reading says the
      programme can still recover.
- **C.4 Threats and mobility cluster: mobility area, weak enemies, hanging, restricted, pawn push, queen threats; refit; gate — `I2`, then `V`.** Mobility with a
  mobility area that excludes own king, queen, blocked pawns and pawn-attacked
  squares; threats: minor and rook attacks on weak enemies, hanging pieces,
  restricted squares, threat by pawn push, king threats, slider and knight
  attacks on the queen, weak queen protection. Shared attack maps from C.1.
  Refit, gate. C.0 found no donor-direction residual for mobility (+0.01%;
  +0.11% in the middlegame band) and a small one for threats (+0.07%;
  +0.20% without queens): the cluster opens with its own residual step and
  closes `NO_CHANGE` if that step finds none.
- **C.5 Endgame handling and winnability cluster — `R3` investigation with
  `I2`/`V` sub-steps.** The rescoped endgame section. Its goal is measured
  conversion and correct draw recognition where games actually go, not
  coverage of a function list.
    - **C.5.1 Classification and deciding instrument per family — `R2`.** Adopt the registered
      family order (`tools/diag/endgame_ranking_v2.json`), confirm each family's
      kind (verdict, scale, conversion) against the code, and name the deciding
      instrument per family: theory truth (`endgame_truth.py`), drawn-cohort
      overclaim (`endgame_drawn.py`), conversion (`endgame_conversion.py`),
      floors, and the A.4 conversion audit at the game level. Starts from
      RAR-E18: at six men or fewer Stockfish's total predicts outcomes 29.7%
      better than Rarog's score, half of it the size it gives a win; above
      six men its total adds about 1.1 points of loss beyond its term
      families while the winnability term itself adds almost nothing, so
      the scale factor is the candidate and is unmeasured. The next cut is
      by material signature. **RAR-E21 and RAR-E22 (2026-10-06):** measured
      as a direction, the scale factor adds +0.09% at seven men or more,
      the complexity term +0.01% and the rule-50 damping +0.03%; and with
      one Stockfish version the donor's families carry exactly its total's
      gain there (+2.73% against +2.73%), so the 1.1 points were an
      artefact of RAR-E17's mixed versions and there is no total-level
      knowledge above six men to chase. The per-family gains there beyond
      king (+0.75%) are small: material and imbalance +0.12%, threats
      +0.10%, passed +0.06%, space +0.05%, mobility +0.04%, winnable
      +0.03%, pawns +0.03%. First step, zero games: the pure
      opposite-bishop cohort (Rarog's score scaled by the donor's factor,
      +2.13 ± 0.58% on 3,427 rows) as a candidate for refitting
      `opposite_bishop_scale`'s constants, read on the drawn-cohort
      instrument (`R2`). The one-queen cohort's king residual (+3.61%)
      belongs to C.3. At six men or fewer the donor's edge is exact
      knowledge and magnitude (its families +14.5% against its total
      +29.7%), much of which the search's tablebase probing covers in
      play; C.5's own instruments, not static loss, decide there.
    - **C.5.2 Generic winnability and scaling: pawn count, opposite bishops, rule-50 scale, complexity — `I2`.** The donor's scale
      factor logic in our form: pawn-count scaling for the stronger side,
      opposite-bishop scaling by non-pawn material and passers, rule-50
      scaling in the scale rather than only the global damping, and an
      initiative/complexity term conditioned on pawn count, king distance and
      both-flank pawns. This is what decides KRPPKRP (5.4% of games, no local
      7-man truth), KPsK (4.5%) and KBPsK (2.6%) generically. Refit, gate with
      an endgame-start cohort and STC. **RAR-E21 (2026-10-06): `NO_CHANGE`
      for the first unit.** The generic scale factor and the complexity
      term were measured as directions (+0.09% and +0.01% at seven men or
      more; their signal is at six men or fewer, +5.4% and +6.9%, and at
      phase < 32, +0.20% and +0.93%); C.3 builds neither. C.5.2 stays
      open for what C.5.1's material cut supports (the opposite-bishop
      refit first) and is gated by C.5's own instruments, not with C.3.
    - **C.5.3 Conversion cluster: KXK, KBNK, KQKR; rule-50 damping interaction measured — `I2`.** Mate drives and
      verdict families with the largest occurrence (KXK 37.8% of the set) and
      the largest measured conversion deficit (KQKR 23/13/3 at 60k/200k/600k
      nodes). Rule-50 damping interaction measured here, sign not assumed.
      Deciding instrument: conversion at bracketed budgets plus theory vetoes.
    - **C.5.4 Rook versus minor cluster: KRKN, KRKB, KRPKB — `I2`.** Three
      families with 100% or 99.6% drawn-cohort overclaim at +300 and the same
      over-representation in Rarog's games; one scaling mechanism, one gate.
    - **C.5.5 Rook and pawn cluster: KRPKR, KRKP, KPK, KPKP audit — `R2`.** Audit
      the existing scalers and the KPK bitbase integration; repair the 30.7%
      KRPKR overclaim if the drawn cohort supports it; close KPK/KPKP
      `NO_CHANGE` if their 4–5% overclaims do not select a mechanism.
    - **C.5.6 Measure-first families: KPsK, KBPsK, KBPPKB, KQKRPs — `R2`.**
      Measure coverage after C.5.2, decide whether any specific recogniser is
      still justified, otherwise close them as served generically.
    - **C.5.7 Theory sweep: KBPKB, KBPKN, KNNKP, KNNK, KQKP from one dispatcher — `I1`.** Sub-1%
      families implemented or confirmed from one dispatcher with Syzygy tests
      and promotion-closure tests, no per-family research cards; `NO_CHANGE`
      where the evidence is clean (KNNK already measured clean). Option
      on record for KBNK, from RBp4wn (tissatussa, Go, CC0, `github.com/tissatussa/RBp4wn`, read 2026-09-17):
      an in-memory distance-to-mate table generated at startup by
      retrograde analysis (about 33 million positions, folded by the four
      colour-preserving symmetries with the bishop normalised to one
      colour, about 5 MB resident, built asynchronously, probed like a
      tablebase with the fifty-move budget checked) makes the family exact
      without Syzygy files. Rarog's corner-drive term and the five-budget
      KBNK anchor stand until C.5's ranking says the family's 0.2% of
      games is worth 5 MB per process; if it is, this is the shape to
      build, behind the recogniser dispatcher.
    - **C.5.8 Endgame gate: endgame-start cohort SPRT plus STC SPRT; floors; conversion; 7-man exclusion — `V`.** One endgame-start cohort SPRT
      plus one STC SPRT for the whole C.5 cluster after its refit; floors and
      theory vetoes re-run; conversion instrument re-measured; KRPPKRP's 7-man
      hold recorded as an explicit exclusion unless independent truth appears.
- **C.6 Pawns and passers cluster; refit; gate — `I2`, then `V`.** Passed pawns with king
  proximity, blocker ownership and type, path safety and attack, unstoppable
  and unblocked conditions, rook behind; pawn structure conditionality
  (doubled, isolated, backward, connected by rank and phalanx, weak lever).
  Refit, gate. Opens with its own residual step (RAR-E17: passed +0.06%,
  pawns +0.04%) and closes `NO_CHANGE` if it finds none. Owns a defect C.0
  confirmed in the source: the unstoppable-passer test in `eval.rs`
  (`king_steps > pawn_steps − …`) is one tempo generous in both move
  orders; the repair is a definition change plus a refit, checked against
  tablebase-labelled pawn endings.
- **C.7 Material, imbalance, phase and pieces cluster; refit; gate — `I2`, then `V`.**
  Imbalance in the donor's quadratic form seeded from current material terms,
  phase interpolation review, bishop pair and bishop-pawn colour terms,
  outposts, minor behind pawn, rook on open and semi-open files, trapped
  rook, weak queen, king protector distances. Refit, gate. Opens with its
  own residual step (RAR-E17: pieces +0.00%, material and piece-square
  tables −0.02%, and +2.41% only at six men or fewer) and closes
  `NO_CHANGE` if it finds none.
- **C.8 Refit cycles: regenerate, refit, gate; stop at the first non-accepting cycle — `V`.** After the family clusters: regenerate data with
  the accepted head, refit the whole surface, gate; repeat while a cycle
  accepts, stop at the first that does not. Initialization control (neutral
  start against accepted start) in the first cycle. Each cycle records the
  C.2 manifest it fitted from, so every refit states what was and was not
  fitted; a coefficient's status changes only by a recorded decision, never
  by a cycle quietly widening the free set.
- **C.9 HCE SPSA of nonlinear residue, or a written skip — `V`.** Only the activated nonlinear or
  global terms the linear trace cannot fit; skipped with a written reason if
  the surface is flat.
- **C.10 Joint search SPSA after the new evaluation: cp margins plus every mechanism whose firing rate moved 10% or more (the whole-surface tune B.6 left for here); rule-7c blocks; SPRT `[0,3]` — `V`.** The search
  was fitted on the B-era evaluation: its cp-valued margins sit on that
  scale, and a new evaluation moves every node population the other
  coordinates were fitted on. This is the whole-surface search tune B.6
  left for here (maintainer decision 2026-10-01, RAR-S95): one joint SPSA
  on the frozen classical evaluation, search coordinates only (rule 2),
  registered by PROCESS's go/no-go procedure in rule-7c blocks, PGO bake,
  SPRT `[0,3]`.
  - **Surface:** every cp-valued margin, plus every mechanism whose firing
    rate per interior node has moved 10% or more against the B.9 head
    (stride-1 `bench 13` dumps of both heads), with its coordinates; the
    non-cp families (LMR, histories, corrections) are in by that test, not
    by default. B.6's surface of 123 coordinates is the starting list
    (`analysis/b6_research_2026-10-01.md`).
  - **Seeds:** the C head's defaults; a value a registered game read has
    shown better replaces its default.
  - **Carried from B.6:** `QsFutilityMargin` with a measured direction
    (rising from 178); `ProbCutSeeGapScale`, never fitted on this search;
    `CoreLmpSquare`'s range floor (256) widened first; `QsCountLimit` and
    any other discrete threshold read as a switch by games before block 1,
    never pinned.
  - **Skip rule:** none by a zero-game read. If the firing-rate check
    moves no mechanism by 10% and the cp scale ratio against the B era is
    within 5%, the leaf records that and asks the maintainer.
- **C.11 Checkpoint: same-search deficit, conversion, NPS, pool gauntlet; freeze the classical evaluation — `V`.** Same-search deficit against Stockfish's classical
  HCE re-measured (a fresh hybrid build at the C head is required; the oracle
  package recipe is on the tagged `hybrid` branch; the first re-read is
  C.3.6), conversion instrument,
  pooled NPS, pool gauntlet at 1T. **Freeze the classical evaluation.**

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
- **E.3 Release 3.0.0 (E.2 met) or 2.6.0 through the tag-driven flow — `M`/`V`.** Version, changelog, release notes, fmt, debug and
  release suites, clippy, feature builds, fingerprint, PGO assets, ISA
  verification, CI matrix, tag and publish on maintainer instruction. Version
  is 3.0.0 if E.2 is met, else 2.6.0 (2.5.0 is cut at B.10, maintainer
decision 2026-10-03). The release is cut through the
  tag-driven flow of **E.3.1**, which also carries the two workflow checks
  this release owed (tag equals version; one fingerprint asserted across the
  matrix) and may land any time earlier.
    - **E.3.1 Tag-driven release flow — `I1`, DONE 2026-10-05.** `release.yml`
      and `cargo xtask release-check`; 2.5.0 was released through it, nine
      assets at 11,171,726 (run 37284706939). The procedure is
      `docs/PROCESS.md` *Release*; the leaf's text is in `docs/archive/PLAN-closed-2026-10-05.md`.
    - **E.3.2 Release cut: version 3.0.0 (E.2 met) or 2.6.0, the `[Unreleased]` changelog reviewed and dated, suites, the PR merged with a merge commit, the `v` tag pushed on instruction through E.3.1's workflow — `M`.** E.3's own work, a leaf of its own since
      E.3.1 closed (2026-10-05): the version from `X.Y.Z-dev` to 3.0.0 if
      E.2 is met, otherwise 2.6.0, behaviour-neutral with the fingerprint
      held; `CHANGELOG.md`'s `[Unreleased]`, kept as changes landed,
      reviewed and dated; fmt, clippy, debug and release suites, feature
      builds; GUIDE's checkpoint marks the release; the PR to `master`,
      merged with a merge commit once `CI` and `Release` are green; the
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
