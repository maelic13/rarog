# Rarog development plan

This is the forward roadmap. It says what will be done, in what order, why,
and what decides each step. It does not record history: completed work lives
in [HISTORY.md](HISTORY.md), measured evidence in
[EXPERIMENTS.md](EXPERIMENTS.md), procedures in [PROCESS.md](PROCESS.md), and
the day-to-day status board with checkboxes in [GUIDE.md](GUIDE.md). The
pre-rewrite roadmap is archived verbatim at
[docs/archive/PLAN-phase4-2026-09-09.md](docs/archive/PLAN-phase4-2026-09-09.md);
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
| Search deficit with Rarog's own evaluation | **−247.97 ± 10.89 Elo** at equal time on the 2.4.0 release head, evaluation proved constant, no adjudication (RAR-O03). Depth 19.68 against 20.65. The superseded −250.8 was adjudicated and is not comparable | RAR-O03; `analysis/ablation_results.md` for the ablation |
| Where the search deficit lives | LMR plus shallow-depth pruning explain **272 ± 18** of it, near-additively; everything else about 30 | matched ablation, mask 160 |
| Evaluation deficit with the same search | Stockfish's classical HCE beats Rarog's HCE by **about 329 Elo** | RAR-O02 |
| Speed | **3.19 MNPS pooled median** at bench 13, PGO pext 1T, ±0.2% instrument resolution (best-of 3.21, which is the 3.22 previously recorded); Basilisk 3.71; board work 24% of time, evaluation 29%, search loop 23% | RAR-M48; RAR-M36, RAR-M44 |
| Conversion | **88 draws and 19 losses** after holding a piece-up advantage for 12+ plies, in 3,600 games against the six HCE-era engines on the **2.4.0 release** games — 24.4 and 5.3 per 1,000, unchanged from the 2026-09-04 pool's 57/12 in 2,400 (23.8 and 5.0). Basilisk 1.9.3 in the same tournament: 94 and 12. **RAR-M47's surplus-over-Basilisk reading is not reproduced and is retired**; the stable finding is Rarog's own rate, 80 of the 88 draws by fifty-move or repetition with material in hand; a third independent sample reads 24.2 and 3.3 per 1,000 (RAR-M54, 1,200 games against the same six) | RAR-M49 (release re-read, tournament `5e539523`); RAR-M54 (Super Rating Tournament, 42 engines); instrument RAR-M47 |
| Fingerprint | `bench 13` **7,185,678 / EBF 2.444**: the B.2.3-fitted selectivity core, the default build since B.2.4b accepted it (2026-09-20, `a47e85b`). `--no-default-features` compiles the superseded B.1 search at 7,601,220 / EBF 2.474 — the 2.4.0 fingerprint, accepted by RAR-E15 and reproduced in A.7, A.8.3 and A.8.4 — until B.8 deletes it | GUIDE checkpoint; RAR-S73; RAR-M48 manifests |

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
| B search | 251 equal-time | 120–200 | selectivity explains 272; a Reckless-shaped stack fitted locally |
| C evaluation | 329 same-search | 100–160 | six families, whole-surface refits, endgame conversion |
| D clock, SMP, robustness | unmeasured | 15–40 | Reckless-shaped node-fraction TM; 4T quality |
| Speed inside B and C | — | 10–30 | per-node cost of the new search and evaluation modules |

If those bands are right the classical head lands within reach of Rybka 4.1
and Fritz 16 and near Critter; Houdini 3 may only fall in the NNUE stage. Each
programme's checkpoint re-measures its deficit meter so a miss is seen as a
miss and the budget above is corrected rather than defended.

## 2. Operating rules

`AGENTS.md` is authoritative for measurement, verification, documents and
gating. The rules below decide order and acceptance in this roadmap.

1. **Donor architecture, own implementation.** Which engine donates what,
   what may cross and how the code is written: `PROCESS.md`, *The
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
   record.** Registration in `EXPERIMENTS.md` before any games: hypothesis,
   baseline SHA, bracket, cap, stop rule and the frozen prediction. Bounds
   default to `[0,3]` nElo; a large prior uses `[0,10]` or `[3,10]` and says
   why; a removal or unknown-sign repair uses a loss-permitting or symmetric
   bracket. Never change a gate after seeing games.
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
| Diagnostic counter units and sampling | `analysis/phase4_counter_spec.md` |
| Measurement layers for endgame work | `analysis/endgame_measurement_layers.md` |
| Texel data contract, splits, instrument coverage | `analysis/texel_fitting_handbook.md`, `analysis/hce_archive_audit_2026-08-31.md` |
| Behaviour-neutral change = exact fingerprint plus targeted checks plus pooled NPS | `AGENTS.md` |
| Cross-engine board benchmark parity | `benches/board.rs`, `analysis/board_comparison_411b19_2026-09-09.md` |

## Phase A — Reset: repository, instruments, baselines, consolidation release

**Closed 2026-09-11.** Each leaf's dated record is in `HISTORY.md`; the
text this section held is verbatim in
`docs/archive/PLAN-closed-leaves-2026-09-14.md`. The measured starting
figures are in section 1.

- **A.1 Document reset — CLOSED 2026-09-09.** New PLAN, GUIDE and HISTORY;
  the Phase-4 roadmap archived under `docs/archive/`; checker adapted to
  lettered phases.
- **A.2 Repository and branch cleanup — CLOSED 2026-09-09.** A.2.1 removed
  twelve one-off or superseded tracked files; A.2.2 tagged and deleted seven
  branches (ledger arms resolve through the `oracle/*` and `arm/*` tags),
  kept the oracle package in the untracked `hybrid/dist/` and realigned the
  local version tags to `origin`; A.2.3 inventoried features, options and
  parameters (`analysis/feature_inventory_2026-09-09.md`).
- **A.3 Release gate and pre-release repairs — CLOSED 2026-09-10.** A.3.1
  toolchain 1.97.1 → 1.98.1, behaviour-neutral (RAR-P18); A.3.2 release gate
  RAR-E16, H1 at 742 games, **+54.77 ± 17.04 Elo** over 2.3.2; A.3.3 the clock
  starts at `go` parse (RAR-R11), a larger Move Overhead rejected (RAR-R12).
- **A.4 Build, asset and CPU-selection improvements — CLOSED 2026-09-10.**
  Per-tier assets stay; the universal binary is optional under G.2. A.4.1
  `cc` 1.4.5 with the ISA contract verified (RAR-P21); A.4.2 startup CPU
  advisory (`src/cpu_advice.rs`); A.4.3 `pext` over `base` **+6.65%**
  (RAR-P22); A.4.4 README asset guidance; A.4.5 arguments run through the
  stdin dispatch.
- **A.5 Conversion instrument — DONE 2026-09-10 (RAR-M47).**
  `tools/diag/conversion_audit.py` over PGN exported by
  `tools/diag/export_tournament_pgn.py`.
- **A.6 Codebase consolidation analysis — DONE 2026-09-10, `NO_CHANGE`.**
  `analysis/consolidation_2026-09-10.md`: Phase A refactors nothing; the B.1
  and C.1 handoffs.
- **A.7 Version bump to 2.4.0 — DONE 2026-09-10 (`c6a548f`).**
- **A.8 Baselines on the release binary — CLOSED 2026-09-11.** A.8.1
  reference pool (RAR-M45); A.8.2 four-thread gauntlet (RAR-M46); A.8.3
  oracle deficit meter (RAR-O03); A.8.4 speed baseline (RAR-M48).
- **A.9 Release 2.4.0 — DONE 2026-09-11.** Squashed to `master`, CI green,
  tagged and published with per-tier PGO assets.

## Phase B — Search programme

**Goal:** recover the measured 251-Elo equal-time search deficit, with the
evaluation frozen at the accepted `hce-v3` head, by rebuilding the search as
a Reckless-shaped architecture implemented in Rarog's own code, seeded,
fitted and gated cluster by cluster.

**Why this shape.** The matched ablation says the deficit is selectivity:
LMR and shallow-depth pruning explain 272 ± 18 Elo, the remaining mechanisms
about 30. Rarog already carries an LMR table with adjustments, LMP, futility,
SEE pruning, NMP, ProbCut, singular extensions, IIR, correction histories and
a staged picker, and orders moves better than the oracle by its own counters;
what differs is the population each mechanism admits and how the signals that
gate them are produced and combined. Previous attempts changed one mechanism
at a time against a co-adapted optimum (zero), or rewrote the core without
fitting it (lost). The donor's search is a working co-adapted point; adopting
its shape as a unit and fitting our constants is the cheapest way to move to a
different optimum.

**What "Reckless-shaped" means, concretely** (source of record: Reckless at
`31d9cd6`, its search.rs, history.rs, movepick.rs, transposition.rs, time.rs
and thread.rs; read it, do not copy it):

- Node types as compile-time constants (`Root`, `PV`, `NonPV`), a per-ply
  stack entry holding the static eval, TT move, TT-pv flag, move count, the
  applied reduction, laterality, and pointers to the continuation-history and
  continuation-correction sub-tables selected by the move just made.
- One TT with 3-entry 32-byte clusters, 8-byte entries carrying a 16-bit key,
  move, score, raw static eval, depth, bound, tt-pv and a 5-bit age; static
  eval stored on every probe miss; a probe-aligned "estimated score" that
  tightens the static eval with the TT bound.
- Histories: quiet history indexed by side and by whether the from and to
  squares are threatened; noisy history by piece, to-square, captured type and
  to-square threat; pawn-structure history by pawn key bucket; continuation
  history at plies 1, 2, 4, 6 keyed by in-check and capture; bonus/malus
  formulas linear in depth with caps, gravity updates, and the moved-late malus
  scaled by index.
- Correction histories: pawn key, non-pawn key per colour, continuation
  correction at plies 2 and 4, all bucketed by the fifty-move clock; applied to
  the raw eval together with material-scaled optimism and rule-50 damping.
- Move picker with stages hash, generate-noisy, good-noisy by SEE threshold
  derived from the move's own score, quiet, bad-noisy; quiet scores from the
  four histories plus threat escape, checking-square, en-prise and offense
  terms.
- Selectivity: razoring on the estimated score, RFP with improvement and
  correction terms, NMP with adaptive reduction and verification, ProbCut with
  reduced-depth verification, singular extensions with double/triple margins,
  multi-cut and negative extensions, low-depth singular extension, hindsight
  reductions from the parent's eval delta; in the move loop LMP, futility with
  history and correction terms, bad-noisy futility, history pruning, SEE
  pruning with depth-quadratic thresholds; LMR in 1024ths with terms for
  log-depth, improvement, correction, TT bound, quiet history, PV window
  width, laterality, tt-pv, cut-node, check, cutoff count, singular margin, the
  parent's reduction and a per-node jitter; deeper/shallower re-search
  decisions from the reduced score; PVS on PV nodes.
- Quiescence: TT probe and cutoff, corrected stand-pat, interpolated
  fail-high scores, LMP at three moves when not in check, SEE pruning by a
  margin from alpha, TT write on every exit.
- Root: aspiration delta from eval and PV stability, optimism from the best
  score average, root move node accounting, forgotten-mate and aborted-loss
  guards, multi-PV structure.
- Time: soft and hard bounds from the clock and increment with a fullmove
  curve; a soft-limit multiplier from the best move's node fraction, score
  trend, PV and eval stability and best-move changes; hard check every 2,048
  nodes; a soft-stop vote across threads.
- Threads: lazy SMP with per-thread data, shared TT, shared correction
  histories, a shared best-stats word, and no depth skipping.

**Scale conversion (B.0).** Reckless's centipawn-like scale multiplies the
network output by material; Rarog's HCE is on its own refit scale. Every
cp-valued seed is converted by a measured ratio: the ratio of mean absolute
static evaluation over the same 40 bench positions plus 10,000 datagen
positions, Rarog HCE against Reckless's network. The ratio is recorded with
the B.0 handoff and used for every seed; it is a starting point that SPSA
moves, not a result.

**Gating shape for B.** Each cluster: (1) fingerprint changes, so no
neutrality claim; (2) fixed-node diagnostics against the oracle at sample
stride 1 (depth at 300k nodes, EBF, qsearch share, cutoff composition, LMR
re-search rate, NMP conversion), registered as explanation only; (3) a 2,000
game paired diagnostic run at seeds converted but unfitted, to detect a broken
port early (stop rule: worse than −40 Elo means a defect, not a tuning need);
(4) SPSA over the cluster's live coordinates, registered surface and horizon;
(5) the SPRT, `[0,10]` for the selectivity core and `[0,3]` or `[0,5]` for the
smaller clusters, sized from RAR-M10 at the expected value; (6) the ledger
row with calibration. Reject returns the cluster to `RESEARCH` with its
diagnostics; two rejections stop B.

- **B.0 Investigation — DONE 2026-09-13, `NO_CHANGE` (RAR-M50).**
  `analysis/search_programme_2026-09-13.md`: the deficit is decision quality
  at a fixed budget, not per-ply growth; the handoffs for B.1–B.3 and B.2's
  prediction are frozen there. Record in HISTORY; verbatim text in
  `docs/archive/PLAN-closed-leaves-2026-09-14.md`.
- **B.1 Search restructure, behaviour-neutral — CLOSED 2026-09-14
  (RAR-P24).** `src/search/` modules, `NodeType`, `ThreadData` and the
  sentinel stack; 44 inert parameters and TT provenance removed; exact
  fingerprint and pooled NPS **+6.30%**. B.2.2's NPS floor is read
  interleaved against the B.1 pool in `tools/results/nps-b1-20260914/`.
  Record in HISTORY.
- **B.2 Cluster 1 — the selectivity core — `I2`, then `V`.** **B.0 handoff
  frozen 2026-09-13 (analysis §3, §5, §8, §13.2). Changes to the list
  below: the LMR reduced depth is floored at one ply and may extend by up
  to two; killers, countermove and low-ply history are dropped; the history
  update policy (TT-cutoff bonus, eval-difference training, fail-low parent
  bonus, post-LMR bonus, index-scaled malus, no ageing) is part of the
  cluster; `cutoff_count`, `laterality`, `last_critical_ply` and the applied
  reduction are stack producers; IIR stays as is beside hindsight; move-loop
  pruning uses raw depth with quadratic terms as the donor does, not a
  prospective depth; the continuation-correction tables land as shadow
  producers first; a per-node `threats()` board producer is added; seeds
  follow the three-column rule of analysis §8.2 with 0.457 (evaluation
  units) and 0.75 (SEE units) as the Reckless conversion.** One cluster:
  TT entry format with stored raw eval and tt-pv and the estimated-score rule;
  the correction histories and corrected-eval formula (including optimism
  and rule-50 damping in the search, replacing the evaluator's damping if B.0
  says so); the quiet/noisy/pawn/continuation histories with their update
  rules; the move picker stages and scoring; the move-loop pruning (LMP,
  futility, bad-noisy futility, history pruning, SEE pruning); the LMR formula
  and re-search rules; RFP and razoring on the estimated score; hindsight
  reductions; cutoff counting. NMP, ProbCut, singular and extensions keep
  Rarog's current forms in this cluster so that B.3 can measure them
  separately. Sub-steps:
    - **B.2.0 Architecture review and upgrades — CLOSED 2026-09-14
      (RAR-P25).** Review in `analysis/architecture_review_2026-09.md`
      (RAR-M51); its twelve behaviour-neutral upgrades landed at the exact
      fingerprint and pooled NPS **+0.21%** against the B.1 pool. The
      `Searcher` split it designed is B.2.1's ticket 0. Record in HISTORY.
        - **B.2.0.1 Repository and document restructure — CLOSED 2026-09-14
          (RAR-M53).** Review in `analysis/repository_review_2026-09.md`
          (RAR-M52); all nine documents-only tickets landed: archived
          trackers, closed leaves to one line, one ledger row per experiment,
          procedures-only PROCESS, the `analysis/` index and archive, a
          dead-path check, a shorter GUIDE and the rule-first AGENTS. Logos
          stay tracked (U8's untracking was reverted). Record in HISTORY.
    - **B.2.1** Implement to the B.0 handoff; unit tests for every table's
      bounds and gravity; picker exhaustiveness tests; TT store/probe tests
      including age and replacement; deterministic unwind tests. **Ticket 0,
      from the B.2.0 review (`analysis/architecture_review_2026-09.md`
      §4.3):** split `Searcher` into per-thread `ThreadData` (tables,
      stack, evaluator, stop flags, output sink), per-search configuration
      (parameters, LMR table, limits) and engine-owned shared resources
      (table, Syzygy settings, `SharedContext`) before the first mechanism
      ticket, at the exact fingerprint and inside the NPS pool; `Threads =
      1` stays free of pool machinery by `thread_count == 1` rather than by
      the absence of a shared context; the table backend is untouched
      (D.2's). **Delivery
      shape (from Manta's 6.5.10):** the cluster lands as ordered tickets
      behind one umbrella switch, each ticket keeping the umbrella-off arm at
      the exact B.1 fingerprint; **canaries are anchored to the reference**,
      meaning a tactical position must be solved at the depth classical
      Stockfish `9587eeeb` solves it, not at whatever depth the current build
      manages, and a changed canary is recorded with its cause, never
      re-blessed; and a **decision trace** (`diag`-only, bounded to plies one
      and two under `searchmoves`, printing every prune, reduction and proof
      decision with its inputs) exists before the first ticket, because
      Manta's two seed defects — a static margin overriding a mate in one, a
      count-based skip dropping an unmade mating quiet — were found by that
      trace and are invisible to counters. Implementer and reviewer are
      separate roles; the reviewer's acceptance is recorded before B.2.2.
      **Implemented and reviewer-accepted 2026-09-14 (RAR-S73,
      `analysis/b21_review_2026-09-14.md`); CLOSED.** The umbrella is the `b2core` Cargo feature over
      `src/search/core/`; off, the engine is the B.1 behaviour exactly
      (7,601,220 / EBF 2.474 at every commit, pooled NPS −0.13% against the
      B.1 pool); on, it reads 4,706,910 / EBF 2.391 unfitted, with 80
      `CoreParams` coordinates. The review (class `R2`, a separate session)
      upheld the four resolutions RAR-S73 records: the root, in-check and
      first-move reduction invariant over §3.7's reduction scope; direct
      checks and promotions kept alive after the quiet skip; rule-50 damping
      moved from the evaluator into the search; the KBNK anchor's
      five-budget majority. No defect; both fingerprints reproduced from
      fresh builds. Recorded for their owners: fail-low nodes carry a TT
      move where the donor stores none (B.3's IIR decision), a dead PVS
      guard (B.8), per-node threat and check-mask work and a 31 MiB
      per-thread table footprint (B.7, D.2). **Speed warning for B.2.2:**
      single-build alternating benches read the candidate at about
      0.71x of the off arm; P5 predicted 0.93–0.97x and the pooled floor is
      0.90x, so B.2.2 runs the NPS pool first.
        - **B.2.0.2 MultiPV and the root-line contract — `I1`.** Added by
          maintainer decision 2026-09-14; **executes after B.2.1's reviewer
          acceptance and before B.2.2.** Rarog has no `MultiPV` option, and
          GUI analysis needs one. It is a protocol feature, not a strength
          change, so it is gated by identity at the default instead of by
          games. The root already keeps what it needs: `RootMove` records with
          score, previous score, PV and nodes, and a `td.root_moves`
          restriction that `searchmoves` uses. **Contract, frozen:**
          (1) `option name MultiPV type spin default 1 min 1 max 256`; the
          effective count is clamped to the root set the search actually uses
          after `searchmoves` and Syzygy root filtering. (2) **Identity at
          `MultiPV = 1`:** exact fingerprint on both arms (7,601,220 / EBF
          2.474 by default, the accepted `b2core` fingerprint with that
          feature); an identical `info` stream apart from `time`, `nps` and
          `hashfull` at fixed depth over the bench positions, so `multipv` is
          printed only when the count exceeds 1; pooled-PGO NPS inside the
          neutral-change noise, because root bookkeeping is layout-sensitive
          (`RootMove::record_search` is kept cold for a measured reason).
          (3) **Search for k > 1**, in `search_root` only: per depth, for line
          i = 0..k−1, search the root with the moves already ranked in this
          iteration excluded through the existing root restriction (verified
          in each arm, including the root's TT-move check), the aspiration
          window centred on that move's previous score; after each line,
          stable-sort the remaining records by score. Lines not yet searched
          at the current depth report their previous-depth score and PV;
          fail-high and fail-low lines carry `lowerbound` or `upperbound`.
          (4) `bestmove` and the ponder move come from line 1; best-move
          instability, effort and the soft-stop vote read line 1 only.
          (5) **Threads:** helpers keep searching at MultiPV 1 and contribute
          only through the table; with a count above 1 the reported lines and
          `bestmove` come from the main thread, and helper result voting is
          bypassed. (6) The single-legal-move shortcut, the decision trace
          under `searchmoves` and the search output port keep their
          behaviour. **Checks:** unit tests for k = 3 (distinct first moves,
          scores non-increasing at a completed depth, every PV legal through
          `assert_legal_pv`), k above the legal-move count, k combined with
          `searchmoves`, `bestmove` equal to line 1's first move, Threads 4
          with MultiPV 3 (distinct legal lines, no panic), and a Syzygy clamp
          test that skips when no tables are present; debug and release
          suites, `cargo fmt --check`, clippy at zero warnings; a 60-second
          `go infinite` session at MultiPV 4 ended by `stop`. No SPRT, because
          the default is unchanged. README option list and CHANGELOG updated
          in the same leaf. B.5 inherits this contract.
          **Implemented 2026-09-15 (RAR-P26); CLOSED.** Engine `1a27889`,
          README and CHANGELOG `e1b56f6`. Both fingerprints 40/40 on magic
          and PEXT; the depth-10 `info` stream over the 40 bench positions
          byte-identical to the parent on both arms; pooled PGO NPS +0.62%
          [−0.29%, +1.18%] against the B.2.1 head pool. Lines above 1 run in
          a cold root function beside `search_root`, which stays the one-line
          root unchanged. Two resolutions beyond the frozen text, each from
          the donors: lines after the first neither store the root TT entry
          nor read a TT-less root (their own candidate is the root TT move),
          because the excluded stored move otherwise let IIR search lines
          2..k a ply shallower than reported (Stockfish `pvIdx`, Reckless
          `pv_index`); and unsearched lines after a stop are the last
          completed depth's reported lines, never a root record, whose score
          keeps a fail-low bound (found by the 4-thread stop session, pinned
          by a node-stopped test that fails on the record fill).
          **Reviewed and accepted 2026-09-15** by a separate session (RAR-P26
          carries the checks): fingerprints and the fixed-depth `info` stream
          reproduced against the parent on both arms, suites and lint clean,
          stopped MultiPV 4 sessions on 1T and 4T as the contract requires.
    - **B.2.2** Diagnostics: oracle differential at stride 1, depth at 300k
      nodes, EBF, tactical suite at fixed depth and equal nodes, 2,000-game
      unfitted paired run. Registered as explanation. **Screen thresholds are
      frozen at registration, before implementation**, so a weak candidate is
      turned back before it spends a maintainer SPRT: the reference-anchored
      geometric branching factor (`tools/branching_profile.ps1`, depths 4 to
      12, fresh process per depth, the phase-4 suite with ordinary and mate
      cohorts reported separately, against classical Stockfish `9587eeeb` on
      the same corpus) must stay inside the registered window — B.0 found
      Rarog's 1.630 already below the oracle's 1.736; nodes at a fixed
      depth a registered fraction of B.1's; pooled NPS at least a registered
      floor; the unfitted paired run at least a registered Elo. Below the
      floor the cluster is ablated by component switch once, in a registered
      order, then re-planned; between floor and target the review decides;
      above target B.2.3 proceeds. **Numbers registered by B.0, 2026-09-13
      (analysis §11): branching window [1.55, 1.85]; WAC solved at 100k
      nodes floor 205 / target 220 (baseline 200, oracle 242) and at 400k
      floor 240 / target 250; oracle best-move agreement at 300k nodes floor
      38 of 50; pooled NPS floor 0.90x of the B.1 head; the unfitted paired
      run floor −40 Elo, target +10; 116 oracle-anchored WAC canaries at
      ≤ anchor + 2 plies and 100k nodes, 47 of them quiet key moves, WAC.001
      at ≤ depth 11. Instrument: `tools/diag/fixed_budget_probe.py`.** The
      shape of the ladder is fixed here.
      **Zero-game screens run 2026-09-15 (RAR-S73,
      `analysis/b22_screens_2026-09-15.md`); four floors fail.** Pooled NPS
      **0.683x** of the B.1 head [0.677, 0.687] (P5's falsifier fires); WAC at
      100k **204** (floor 205); oracle agreement **35/50** (floor 38);
      canaries **75/116** (B.1 67) and WAC.001 at depth 14. Branching 1.605
      is inside its window, WAC at 400k 245 is between floor and target, the
      median depth at 300k is unchanged at 16. P3 moved the wrong way
      (branching down) and P4 stopped short (2.56%, 35.3%). No ablation
      order had been registered, so the one ablation ran as an unordered
      sweep of all eight bits: removing razoring alone lifts the canaries to
      104, WAC at 100k to 217 and agreement to 39; ProbCut and singular are
      next and smaller. At a fixed depth the candidate searches more interior
      and fewer quiescence nodes than B.1. An equal-time WAC proxy (0.683 of
      each budget) reads 190 against B.1's 200 at 100k and 235 against 237 at
      400k. **Paired run, maintainer-run 2026-09-15: +52.16 ± 10.73 Elo**
      (nElo +75.14 ± 15.23; 745-808-447; pentanomial [34, 168, 364, 334,
      100]; 2,000 normal terminations; equal time per move and mean depth
      15.94 against 15.84), **above the +10 target** while the four floors
      fail, and P1 (−10 ± 30) missed in sign. **ETW profile:** `threats()` 1.6%
      and `check_info` 0.9% of samples, so the review's speed suspects cannot
      explain the NPS loss. The visible costs are evaluation (29.5% through
      inline chains), TT probing (7.4% exclusive), quiet scoring and selection
      in the picker, and correction lookups. There is no off-arm profile to
      compare. The pre-game recommendation against B.2.3 on this arm is
      withdrawn. **The registration has no rule for a run above target with
      floors failed. That, the screens' validity, razoring, speed order and
      the gate's bounds go to an in-depth review**
      (`analysis/b22_screens_2026-09-15.md` §9) before B.2.3.
      **Reviewed 2026-09-15** (`analysis/b22_review_2026-09-15.md`): the paired run
      governs and the floors are diagnostics; the NPS reading is mostly
      tree mix and small distributed per-node costs, not one producer;
      razoring lost the B.1 head's `!tt_pv` and depth guards; the
      correction trains on mate-range residuals and at singular-excluded
      nodes as the donors do, each shrinking the tree by a sixth. The
      review asks for three categorical paired runs, the eval-unit clamps
      converted, and the registered curvature sweep before B.2.3.
      **Maintainer decision 2026-09-15: the paired run governs; B.2.2's
      re-plan is the four sub-steps below, in order, and B.2.3 starts when
      B.2.2.3 has reported. B.7 stays where it is; nothing is pulled
      forward.** The five switches are categorical and are never SPSA
      coordinates (RAR-M13's lesson); a losing switch stays at its default
      until B.8 removes it. **CLOSED 2026-09-15 with B.2.2.4:** no switch
      adopted, the clamp conversion reverted to four coordinates, three
      curved coordinates so B.2.3 runs, and the screen ladder written into
      rule 8.
        - **B.2.2.1 Categorical switches and the seed-scale clamps — `I1`,
          CLOSED 2026-09-15 (RAR-S74).**
          Engine work on the `b2core` arm, one commit per item, each with a
          test that constructs its effect and the candidate fingerprint
          recorded in the commit message. (1) **Clamp conversion first**,
          because it changes the base every later arm is measured against:
          the two hard-coded clamps in `late_move_reduction` that are still
          in the donor's evaluation units become Rarog's (rule 8.2, ×0.457):
          the improvement term's `(-241, 1155)` → `(-110, 528)` and the
          `alpha − estimated` gap's `(-65, 91)` → `(-30, 42)`. New
          fingerprint, recorded in RAR-S73. (2) Five switches as `CoreParams`
          spins with range `0..=1`, defaults equal to today's behaviour so
          the fingerprint after (1) is unchanged by (2): `CoreRazorGuards`
          (0 = donor; 1 = the B.1 head's guards on the candidate's margins:
          no razoring on a `tt_pv` node and none above depth 3);
          `CoreCorrTrainDecisive` (1 = admit mate-range residuals; 0 =
          refuse `|score| >= TB_WIN_SCORE` at both training sites);
          `CoreCorrTrainExcluded` (1 = train at singular-excluded nodes; 0 =
          refuse when `excluded` is set, both sites); `CoreLmrFullDepth` (0 =
          off; 1 = the donor's full-depth-search branch for the moves LMR
          does not take at a non-root, non-check node: Reckless's second
          reduction formula, `207·ilog2(depth)`, improvement
          `(366·impr/128).clamp(-206, 1370)` with the clamp converted as in
          (1), `2255·|corr|/1024`, quiet `1468 − 118·history/1024`, noisy
          `940 − 63·history/1024`, `tt_pv` `−844 − 1129·(tt_depth ≥ depth)`,
          else cut node `+1260 + 2168·no TT move`, `cutoff_count > 2` `+1394
          + 258·all-node`, TT move `−3002`, parent reduction `> r + 590` →
          `+130`, jitter `(nodes + id·26) % 128 − 56`; the move is searched
          at `new_depth − (r ≥ 2621) − (r ≥ 5579)`, floored at one ply, and
          re-searched at `new_depth` on a null-window fail-high like a
          reduced move; its constants are seeds, not coordinates, until the
          switch wins); `CoreLmrCheckRoot` (0 = off; 1 = the donor's scope:
          late-move reductions also at the root and at in-check nodes, with
          the first move still unreduced and the one-ply floor kept; the
          §13.2 invariants this tests were a B.0 research decision and a
          registered switch is the way to overturn one). The two shapes
          were B.2's scope in §3.7 and were not built (B.2.1 review §3.1).
          (3) Tests: `CoreRazorGuards` — a `tt_pv` node far below alpha is
          not razored at 1 and is at 0; `CoreCorrTrainDecisive` — a
          mate-score residual leaves every table untouched at 0; 
          `CoreCorrTrainExcluded` — the singular search trains nothing at 0;
          `CoreLmrFullDepth` — a non-PV first move is searched below
          `new_depth` when its reduction reaches 2621 and at `new_depth`
          otherwise; `CoreLmrCheckRoot` — a root and an in-check node
          reduce a late move at 1 and never at 0; every switch keeps
          `stack_reductions_unwind_to_zero` and the PV-legality tests. (4)
          Verification: debug and release suites on both arms, fmt, clippy
          on all features and on `b2core,tune`; `bench 13` on the `b2core`
          arm at every commit; `AblationMask` bits still move the tree. (5)
          Hand over: one `b2core,tune` PGO build by
          `nps_build_pool.ps1 -Features b2core,tune -Builds 1`, hashed and
          fingerprinted, and the six `sprt.ps1` commands of B.2.2.2 with
          `-OptionsB`, then register RAR-S74 in `EXPERIMENTS.md` with the
          predictions of B.2.2.2 copied verbatim before any game.
          **Implemented 2026-09-15 (RAR-S73, RAR-S74).** Engine commits
          `bebed9f` (clamps), `0e0c526` `CoreRazorGuards`, `218036c`
          `CoreCorrTrainDecisive`, `dccf72c` `CoreCorrTrainExcluded`,
          `8d9c9da` `CoreLmrFullDepth`, `7632ae0` `CoreLmrCheckRoot`. The
          clamp conversion moves the candidate to **6,586,667 / EBF 2.433**
          (from 4,706,910 / 2.391). The switches leave it there at their
          defaults; flipped one at a time, the counts are 5,069,746, 5,504,856,
          5,425,272, 5,527,766 and 4,732,687. The off arm is 7,601,220 / EBF 2.474
          at every commit. Suites: release 295, debug 294, `b2core` release
          319, `b2core` debug 318, 0 failed. fmt and clippy are clean on every
          feature set, `b2core,tune` included. Every `AblationMask` bit moves
          the tree. Three readings of the frozen text: `CoreLmrFullDepth`
          reaches the first moves of non-PV nodes, since LMR takes every later
          move from depth 2 and the one-ply floor, which never deepens, leaves
          nothing at depth 1. It drops with LMR under ablation bit 7. It
          carries the listed terms only; the donor's singular-margin term is
          absent, as it is from the core's LMR. `CoreLmrCheckRoot` adds no
          in-check term, because the donor's `in_check` term is the core's
          gives-check term, and it leaves a root move's verification depth
          unmoved, as the donor does. Handover: the `b2core,tune` PGO
          binary `tools/results/b2221-core-tune-20260915/pext-1.exe`,
          sha256 `27B1AF0E…C053`, bench 6,586,667, source `7632ae0` clean.
        - **B.2.2.2 Six paired runs, the categorical screen — `V`,
          maintainer-run.** Registered as one bounded baseline/A/B/A+B screen
          (RAR-S74): the same `b2core,tune` PGO binary on both sides,
          `sprt.ps1 -Mode fixed -Games 2000 -NoAdjudication`, `3+0.03`, 1T,
          Hash 64, UHO, paired, concurrency 14 with affinity; A = defaults,
          B = one switch flipped by `-OptionsB`. Runs, in this order: (a)
          `CoreRazorGuards=1`; (b) `CoreCorrTrainDecisive=0`; (c)
          `CoreCorrTrainExcluded=0`; (d) `CoreLmrFullDepth=1`; (e)
          `CoreLmrCheckRoot=1`; (f) every switch that won in (a)–(e) together
          against the defaults, the interaction check. **Predictions, frozen
          2026-09-15 by the review:** (a) **+8 ± 11** (a fixed-node tactical
          loss of 29 canaries against a 28% node saving; the guard wins if
          the canaries are games); (b) **+3 ± 11** (mate residuals are a
          pruning signal, not an evaluation error, and B.1 refused them);
          (c) **0 ± 11** (donor-faithful either way); (d) **+5 ± 11** (the
          candidate searches 2.30x the oracle's interior nodes against B.1's
          1.80x and the absent branch is the likeliest reason); (e) **0 ±
          11**; (f) the sum of the winners' point estimates within ± 15, a
          larger shortfall meaning an interaction that B.2.3 must fit
          around. **Adoption rule, fixed before the games:** a switch is
          adopted at ≥ +5 Elo, rejected at ≤ −5, and left at its default
          between; (f) confirms the adopted set at ≥ +5 or returns the
          adopted switches to one-at-a-time confirmation. Adopted switches
          become the defaults in one engine commit with the new fingerprint
          recorded; the fixed-node diagnostics (WAC at 100k, agreement,
          canaries) are re-read on that arm as diagnostics only. Calibration
          of (a)–(f) appended to RAR-S74 after the games.
          **Runs (a)–(e) played 2026-09-15 (RAR-S74):** switch effects
          −2.6, **−15.1**, +2.3, +1.7, −3.8 (± 9.5–9.9 each); nothing
          adopted, (f) not applicable, one host forfeit. Refusing mate-range
          residuals costs about 15 Elo, so the donor's admission is right
          on this evaluation; the other four are noise. **Run (g), added by
          the review:** the converted arm (6,586,667) against the RAR-S73
          candidate (4,706,910, `tools/test_engines/rarog-b22core-pext-pgo.exe`),
          same conditions, prediction 0 ± 11; at or below −5 the clamp
          conversion is reverted and the clamps become SPSA coordinates.
          **Run (g) read 2026-09-15: −7.64 ± 9.72 for the converted arm; the
          rule fires.** Revert ticket, `I1`, one engine commit: restore the
          two clamps and expose them as four `CoreParams` spins seeded at
          the donor's values, `CoreLmrImprovementClampLo` −241 (range
          −1024..=0), `CoreLmrImprovementClampHi` 1155 (0..=4096),
          `CoreLmrAlphaGapLo` −65 (−512..=0), `CoreLmrAlphaGapHi` 91
          (0..=512); the clamp test asserts the seeds; the `b2core` bench
          must read 4,706,910 / EBF 2.391 again (the five switches are
          inert at their defaults); the off arm stays 7,601,220. The four
          are SPSA coordinates in B.2.3 (the five switches are not). Record
          the fingerprint in RAR-S73 and RAR-S74. B.2.2.2 closes on that
          commit; B.2.2.3 runs on the 4,706,910 arm.
          **CLOSED 2026-09-15 on `308abe9`.** The `b2core` bench reads
          4,706,910 / EBF 2.391 again, 40/40 per-position lines identical to
          the pre-conversion arm on magic and PEXT. The four coordinates set
          to the converted bounds over UCI reproduce 6,586,667. The off arm is
          7,601,220 / EBF 2.474. Suites: release 295, debug 294, `b2core`
          release 319, `b2core` debug 318, 0 failed; fmt and clippy clean on
          `--all-features` and `b2core,tune`. `CoreParams` now declares 89
          coordinates: the 80 of B.2.1, the four clamp bounds (SPSA
          coordinates in B.2.3) and the five categorical switches (never
          coordinates).
        - **B.2.2.3 Curvature sweep and the P6 profile — `V`.** On the arm
          B.2.2.2 leaves, the sweep §9 registered as SPSA's condition: the
          five coordinates (`CoreRfpLinear`, `CoreLmpSquare`, `CoreFpBase`,
          `CoreLmrQuiet`, `CoreCorrUpdateSlope`) at 0.5x, 0.75x, 1x, 1.5x
          and 2x of their defaults, set by UCI option on the `b2core,tune`
          build, each point reporting `bench 13` nodes and WAC solved at
          100k nodes (`fixed_budget_probe.py` with the option flags
          `02d2c09` added); a surface is *curved* when the WAC column has an
          interior maximum, *monotone* when it rises to an edge, *flat*
          when it moves by ≤ 2 solved. Rule: flat or monotone on all five
          sends B.2 to B.2.4 at the defaults; any curved coordinate sends it
          to B.2.3 over the registered coordinate set less the five
          categorical switches. Same sitting, for P6: `bench 13` with
          `CoreCorrWeightCont2=0 CoreCorrWeightCont4=0` against the
          defaults (P6 predicted < 3%), and the continuation-correction
          admission counts by remaining depth from a `diag` run, recorded
          once in RAR-S73. Frozen table in a short `analysis/` record.
          **CLOSED 2026-09-15** (`analysis/b223_sweep_2026-09-15.md`, rules
          frozen in `f7b766c` before any point; instrument `f975eae`). WAC at
          100k across 0.5x–2x on the 4,706,910 arm, which reads 204 at the
          defaults: `CoreRfpLinear` monotone (212 at 0.5x); `CoreLmpSquare`
          **curved** (211 at 0.75x, 202 and 196 at the edges); `CoreFpBase`
          monotone (209 at 0.5x and 0.75x); `CoreLmrQuiet` **curved** (205 at
          0.75x, weakest); `CoreCorrUpdateSlope` **curved** (207 at 0.75x).
          **Three curved: B.2 goes to B.2.3.** P6: zeroing both
          continuation weights moves `bench 13` by +5.59% (two-ply table alone
          +8.51%), above the predicted 3%, so the falsifier fires as worded;
          single coordinate steps move the bench 0.3–71%, so the reading does not
          establish a pruning-signal role alone, and re-admission is for
          review. Admissions concentrate at low remaining depth (70–72% at
          depth ≤ 2).
        - **B.2.2.4 Screen registrations for B.3–B.5 — `I1`, documents.**
          Write into PLAN's B.3, B.4 and B.5 screen text and into rule 8:
          the paired run governs and zero-game floors are diagnostics that
          trigger the ablation and a written cause, never a hold of a
          candidate the paired run cleared nor an acceptance of one it
          failed; the ablation order is the eight `AblationMask` bits in
          mechanism order, one sweep; the pooled-NPS floor is replaced by
          time-to-depth on `bench 13` (interleaved medians) with pooled NPS
          reported as a diagnostic, because a tree-shape change makes nodes
          incomparable; a positional fixed-node screen joins WAC — the
          Strategic Test Suite if the maintainer places it at
          a tracked fixture `sts_v1.epd` under `tools/diag/`, otherwise the phase-4
          suite at 100k nodes scored by oracle agreement; the canaries
          become a regression rule (no canary the baseline solves may be
          lost) instead of "all pass"; the curvature sweep is a checklist
          item with its own evidence path. GUIDE and PLAN in one commit;
          `check_guide.py` passes. **CLOSED 2026-09-15:** the ladder is rule
          8's *Cluster screens*, and B.3, B.4 and B.5 each point to it. The
          STS fixture is not placed, so the phase-4 suite scored by oracle
          agreement is the positional screen until it is. **With it, B.2.2
          closes.**
    - **B.2.3 SPSA on the `b2core` arm — preparation `I1`, tune `V`,
      maintainer-run.** Defined 2026-09-15 by the B.2.2 review after
      B.2.2.3 found three curved coordinates. **Arm:** `b2core` at
      4,706,910 / EBF 2.391 (engine `308abe9`), the five categorical
      switches at their RAR-S74 defaults. **Coordinates, 82:** every
      `CoreParams` spin except the five switches (`CoreRazorGuards`,
      `CoreCorrTrainDecisive`, `CoreCorrTrainExcluded`, `CoreLmrFullDepth`,
      `CoreLmrCheckRoot`, gated by games and never coordinates),
      `CoreIirMinDepth` (a discrete depth threshold; IIR is B.3's decision)
      and `CoreEvalRule50Damping` (too few games reach a high clock at
      `3+0.03` to carry a gradient; rule 6). The four LMR clamp bounds are
      in. `SearchParams` (null move, ProbCut, singular, aspiration, time)
      stay at their defaults: B.3, B.5 and B.6 own them, and search and
      evaluation coordinates are never mixed. **Config:**
      a new `config_b23core.json` under `tools/spsa_configs/`, one entry per coordinate
      with `value` = the engine default, `min_value`/`max_value` = the
      declared range, `step` = `max(2, round((max − min) / 16))` (about 6%
      of the range at iteration 1, decaying to 0.39 of that at N = 10,000;
      every integer coordinate keeps `step · c_t(N) ≥ 0.5`);
      `fixed_b23core.json` = Hash 64, Threads 1, MultiPV 1 and the five
      switches at their defaults (fixed because they were gated, not
      pinned). `./tools/audit_spsa_coverage.ps1` must read clean.
      **Tooling ticket first:** `tools/build_test.ps1 -Tune` refuses
      `-Features`, and `spsa.ps1` requires its bench-verified `*-tune`
      manifest, so today no `b2core` tune binary can enter the tuner.
      Let `-Tune -Features b2core` build `--features tune,b2core` with
      flavor `b2core-tune`, the manifest's fingerprint the arm's own
      (4,706,910), and `spsa.ps1` accept that flavor; a negative test that
      an off-arm tune binary is refused for a config naming `Core*`
      options. **Schedule** (`tools/spsa.ps1` defaults, validated by
      RAR-M05): 32 games per iteration, `r_end` 0.0031, `A = N/10`,
      `alpha` 0.601, `gamma` 0.102, concurrency 14 with the affinity patch.
      **Pilot** (PROCESS step 5): 128 iterations × 32 games, sensitivity
      only, never promoted or used as a seed; re-audit the surface after
      it. **Horizon:** the repo's convergence model puts 82 coordinates
      within a few percent of 30 (endpoint RMSE in step units 0.60 / 0.47
      / 0.42 at N = 5,000 / 10,000 / 15,000 for weak curvature; 0.26 /
      0.22 / 0.23 for moderate), so the second 5,000 iterations buy a modest sharpening of a tune
      predicted at about +12 Elo, below what 2,000 games resolve, for 35
      more hours. **N = 5,000** (160,000 games, about 35 hours at 82–97
      games per minute) is registered, maintainer decision 2026-09-15,
      run in sessions: the tuner saves state every 10 iterations,
      `-StopAfter` stages a stop and `-Resume` continues with the same
      `-Iterations`, which never changes after launch; the host must be
      idle while a session runs. Staged reviews at `-StopAfter 1250` and
      `2500` read state without changing the horizon. If the
      fitted-vs-unfitted run below is clearly positive and coordinates are
      still moving at N, B.6's joint tune starts from the fitted defaults. **Estimator:** the final theta at N from
      `tuner/state.json`, rounded to integers, no checkpoint selection;
      the maintainer pastes the final values. **Registration** as RAR-S75
      before launch: surface, fixed values, binary hash and fingerprint,
      horizon, gain, estimator. **After the tune:** bake theta as the
      `CoreParams` defaults in one engine commit (new `b2core`
      fingerprint), fresh PGO build; then B.2.4b, the fitted arm's own
      SPRT against the unfitted arm (prediction, frozen here: **+12 ±
      11** for the fitted arm, moderate; falsifier: H0 means the donor's
      seeds were already near the STC optimum, theta is not baked, and
      B.6's joint tune is deferred to the accepted head). **P6 ruling (B.2.2.3):** the continuation
      correction stays admitted; the +52 arm includes it, and a 5.6% bench
      move in a tree that moves 0.3–71% per coordinate step is not a
      pruning-signal finding. Calibration at B.2.4.
      **Preparation IMPLEMENTED 2026-09-15 (RAR-S75); the tune ran
      2026-09-15 to 2026-09-20, maintainer-run, and B.2.4b scored the
      prediction (+138.60 ± 30.66 against +12 ± 11).**
      - Tooling `2f2c43a`: `build_test.ps1 -Tune -Features b2core` builds
        flavor `b2core-tune`, and `spsa.ps1` refuses an off-arm tune binary
        for a `Core*` surface (checked with a real off-arm build).
      - Surface `b682cf8`, with the README at N = 5,000 in `fa54d5d`:
        `config_b23core.json` (82 coordinates, audit clean) and
        `fixed_b23core.json`.
      - Tune binary: `rarog-b23core-tune.exe`, sha256 `25467C63…FDA0E`,
        bench-verified at 4,706,910 / EBF 2.391.
      - `-SetupOnly -Iterations 5000` verified all 82 options, with A = 500
        and a = 0.09655.
      - Pilot done 2026-09-15 (4,096 games). **The tune finished 2026-09-20
        at 5,000 iterations and 160,000 games; the final theta is baked in
        `14a7079` (new `b2core` fingerprint 7,185,678 / EBF 2.444), after
        `54a8115` made two seed-pinning tests read their coordinates.**
        From 3,900 to 5,000 no coordinate moved a full step.
      - Sub-steps added 2026-09-19. None blocks B.2.4b, which needs only
        the baked final theta; B.2.3 closes when B.2.3.3 reports. The
        re-tune the maintainer asked for is B.2.7.
        - **B.2.3.1 Checkpoint peek at 3,900 — CLOSED 2026-09-19
          (RAR-S76).** Theta at 3,900 against the unfitted head:
          +118.72 ± 10.62 Elo in 2,000 games against a predicted
          +10 ± 11; bench 6,199,302 / EBF 2.421. Never baked.
        - **B.2.3.2 Rybka 4.1 benchmark — CLOSED 2026-09-19 (RAR-M55,
          RAR-M56).** Deep Rybka 4.1 SSE42 x64 with `Max CPUs=1` at the
          harness conditions, 2,000 games each: the B.2.3.1 binary
          **+97.69 ± 12.84 Elo**, the unfitted head **−7.12 ± 12.78**
          (predicted −15). The fit is worth +104.8 ± 18.1 against a foreign
          reference, about 88% of its self-play value. External reference
          points, never gates.
        - **B.2.3.4 Pool gauntlet of the fit — CLOSED 2026-09-19
          (RAR-M57).** Maintainer-run Colosseum gauntlet, 6,000 games at the
          Super Rating Tournament's conditions with the pool held at its
          ratings: Rarog 2.5.0-dev (native build of the B.2.3.1 values) rates
          **3191**, against 2.4.0's 3001. At 1T it scores 59.1% against
          Rybka 4.1, 52.3% against Fritz 16, 46.1% against Critter 1.6a and
          39.2% against Houdini 3. A follow-up match against Critter 1.6a,
          stopped at 1,050 games, reads −5.0 (95% −22 to +12; RAR-M58).
        - **B.2.3.3 The tail, theta at 5,000 against theta at 3,900 —
          CLOSED 2026-09-20 (RAR-S77).** `[0,3]` nElo, cap 40,000: **H1
          accepted in 20,806 games, +4.43 ± 2.90 Elo (+7.20 ± 4.72 nElo)**.
          The last 1,100 iterations were worth about 4 Elo, against a frozen
          prediction of an unresolved cap; the zero-game movement read (no
          coordinate moved a full step) described the tail correctly but
          carried no Elo information. **With it B.2.3 closes.**
    - **B.2.4** Gate, **two SPRTs (maintainer amendment 2026-09-15, made
      after the unfitted paired run was seen and before either SPRT ran;
      bounds, cap, book and adjudication unchanged from the
      registration):** **B.2.4a** the unfitted `b2core` arm against the
      off arm at the same revision, `[0,10]` nElo, cap 16,000 games,
      runs as soon as its two PGO binaries exist and does not wait for
      B.2.3; passing makes the unfitted arm the accepted head. Binaries
      built 2026-09-15 at `6e4fa8a`: `rarog-b24a-core-pext-pgo.exe`
      (4,706,910, sha256 `9206A598…71F8`) and `rarog-b24a-base-pext-pgo.exe`
      (7,601,220, sha256 `4EC72F0F…91D5`), recorded in RAR-S73. **Played 2026-09-16: H1 accepted, +65.09 ±
      23.26 Elo in 432 games (RAR-S73); the unfitted `b2core` arm is the
      accepted head of B.2 until B.2.4b.** **B.2.4b played 2026-09-20: H1
      accepted in 248 games, +138.60 ± 30.66 Elo (+217.71 nElo), so the
      FITTED arm (7,185,678 / EBF 2.444, `14a7079`) is the accepted head and
      the base for B.3 (RAR-S73).** The feature stayed a build flag until
      B.2.4b chose between the fitted and the unfitted defaults. **The flip
      landed 2026-09-20 (`a47e85b` engine, `58176a1` tooling), CLOSING the
      leaf:** `b2core` is a default Cargo feature, the default build is the
      accepted head at 7,185,678 / EBF 2.444 and reports `2.5.0-dev`, and
      `--no-default-features` still compiles the B.1 search at 7,601,220,
      reporting `2.5.0-dev+legacy`, until B.8 deletes that path. CI's default
      jobs cover the core and its `--no-default-features` jobs the legacy
      search; the feature matrix enumerates 32 subsets from a clean slate. **B.2.4b**
      the B.2.3-fitted arm against the unfitted arm, `[0,10]` nElo, cap
      16,000 games, after theta is baked; passing makes the fitted arm
      the accepted head, failing leaves the unfitted one. Each accepts
      one thing on its own evidence, so a weak tune cannot ride on the
      cluster's margin. Ledger row and calibration after each. Accepted
      head becomes the base for B.3. No null calibration precedes it: the 1T
      harness is calibrated and shared with Basilisk, and switching
      adjudication off symmetrically (RAR-M17) does not reopen it (RAR-M03;
      the RAR-E06 registration's calibration disposition, accepted by the
      maintainer 2026-09-01). **Correction 2026-09-14:** B.0 had added a
      "null calibration owed" precondition here claiming no calibration
      followed RAR-M17; that contradicted those two records and is withdrawn
      by maintainer decision.
    - **B.2.5 UCI `info` conformance — `I1`, CLOSED 2026-09-20.** All seven
      items landed: `da6d8f9` the pool's winner line, `5cba881` the mated-root
      line, `45779ee` the `nps` floor and `multipv 1` on every line, `8a76826`
      single-PV aspiration bounds, `5a8d58b` the seldepth convention,
      `1e725fe` the `<empty>` placeholder. Both fingerprints held at every
      commit (7,185,678 default, 7,601,220 legacy) and the depth-10 info
      stream over five positions stayed identical once each item's own change
      was masked. README's UCI notes and the changelog record the new output.
      Items 3 and 4 share a commit; the reason is in its message. Added
      2026-09-16 by
      maintainer decision from the implementer's review (`analysis/uci_info_review_2026-09-16.md`);
      runs after B.2.4 closes and before B.3, on the accepted head, both
      arms. Output-only, behaviour-neutral, no SPRT: gated by identity
      (exact fingerprints 7,185,678 / EBF 2.444 for the default build and
      7,601,220 / EBF 2.474 with `--no-default-features`) plus protocol tests, because the
      bench prints no `info` line and an identical bench proves nothing
      here. **Contract, one commit per item:** (1) after the pool's vote,
      when the reported line is not the winning thread's, print the
      winner's final line before `bestmove`, as Stockfish's
      `output_pv(*bestThread)` does; test: at Threads 4 over the review's
      positions the last `info` line's first PV move equals `bestmove`,
      always. (2) A root with no legal move prints `info depth 0 score
      mate 0` in check and `info depth 0 score cp 0` otherwise, then
      `bestmove 0000` unchanged. (3) `nps` divides by `max(1, elapsed
      ms)` in both senders; test: a line at `time 0` carries `nps` ≥
      `nodes · 1000`. (4) Single-PV lines carry `multipv 1`, so every line
      has one shape; the MultiPV test harness and `tests/multipv.rs`
      parsers are updated with it; the B.2.0.2 identity check is re-run
      as a diff against the previous head with the token stripped. (5)
      Aspiration fail-high and fail-low emit a line with `lowerbound` or
      `upperbound` in single-PV mode, as the MultiPV path already does,
      and a stopped iteration reports its last completed bound; no
      `currmove` output (optional, skipped by Reckless). (6) `seldepth`
      resets per iteration and reports the maximum ply reached plus one,
      Stockfish's convention; the diag and probe tools that read seldepth
      are checked for the off-by-one. (7) A string option set to the
      literal `<empty>`, the default Rarog advertises and GUIs such as
      CuteChess echo back, means an empty string, as Stockfish's
      `ucioption.cpp` treats it; today `SyzygyPath` tries to load a folder
      named `<empty>` and prints "loaded no usable tablebases"; test:
      `setoption name SyzygyPath value <empty>` prints no tablebase line
      and leaves probing off. Items (1) and (7) are the two defects in
      GitHub issue maelic13/rarog#1 (2026-09-15, Rarog 2.4.0 in CuteChess);
      its `ponder` question is not a defect (UCI allows a ponder move with
      `Ponder` off), and its 175 ms first line was the start-up stall B.2.8
      closed. **Verification:** both fingerprints
      exact at every commit; the depth-10 `info` stream over the bench
      positions diffed against the previous head with the new tokens
      stripped (identical apart from the changes the item names); debug
      and release suites on both arms; fmt; clippy on all features. Score
      normalisation (the review's item 6) is D.3's research card, not this
      leaf. Documentation: README's UCI notes and CHANGELOG.
    - **B.2.6 Adopt Colosseum CLI as the harness — `M`, tooling only.** Added
      2026-09-18 by maintainer decision. Colosseum CLI (`D:/code/colosseum`)
      replaces `sprt.ps1`'s runner, weather-factory, the gauntlet, NPS and
      PGN-replay scripts. **The harness is qualified in its own repository,
      not here:** its release acceptance (Colosseum PLAN Phase 10, the
      qualification item) runs the null pair, the scale check against
      fastchess, an SPRT replay, a fixed-field gauntlet and the SPSA
      recovery test, with Rarog as the validation engine. Rarog then trusts
      a released, qualified binary and repeats none of it; a new null pair
      is owed only on the trigger PROCESS already names, a runner, scheduler
      or topology change on this host. The leaf starts only after
      `cli-v0.1.0` is tagged, B.2.3's tune has finished on weather-factory
      and B.2.4b has been gated with `sprt.ps1`, so no registered
      experiment changes instrument mid-way. **B.2.6.1:** Rarog's policy
      as committed TOML run files (`3+0.03`, Hash 64, Threads 1, UHO, no
      adjudication, margin 20 ms, placement auto, 14 slots for gates and
      15 for tunes) and thin wrappers that keep every guard `sprt.ps1` and
      `spsa.ps1` enforce today; `setup_tools.ps1` stages the tagged release
      and pins its SHA-256. Check: the wrapper's dry run resolves to the
      conditions an `sprt.ps1` manifest records, field by field, and one
      short live run completes with the guards firing on a deliberately
      mismatched sidecar. **Landed 2026-09-21 by maintainer request, ahead
      of the tag and of B.2.4b:** `tools/colosseum/` holds the run files
      (`common`, the four brackets, `match-fixed`, `spsa-tune`,
      `calibrate-null`, `gauntlet`) with their README, and
      `tools/spsa_config_to_colosseum.py` converts a registered surface to
      a Colosseum tune file plus its run file, `c_end = step · N^−0.102`
      for the registered horizon, with `--check` refusing a file that has
      drifted from its JSON. Verified by a dry run of every file against
      the policy field by field (book, order, adjudication, placement,
      headroom, slots, time control, margin, brackets, tune width) and by
      the generated 82-coordinate surface matching the hand-made one used
      for the 2026-09-18 tune, parameter for parameter. Not yet done:
      wrappers with the `sprt.ps1`/`spsa.ps1` guards, `setup_tools.ps1`
      staging a tagged release with its hash, the manifest parity run and
      the short live run; `sprt.ps1` and `spsa.ps1` stay the gate and tune
      path until then. **Maintainer decision 2026-09-21, which supersedes
      the start condition and B.2.6.2's retirement above:** the leaf opens
      now. Colosseum CLI becomes the main tool for gates, fixed matches,
      tunes and gauntlets as soon as B.2.6.1's wrappers and parity pass.
      fastchess, weather-factory, `sprt.ps1`, `spsa.ps1` and everything
      they need stay installed, working and documented as the backup and
      the second opinion, at least until release 2.5.0; retirement is
      reviewed at that release and not before. Until `cli-v0.1.0` is
      published, `setup_tools.ps1` stages a Colosseum build pinned by its
      source revision and SHA-256, and **B.2.6.3** re-pins to the tagged
      archive and repeats the dry-run parity. **B.2.6.3 DONE 2026-09-22**, the
      day `cli-v0.1.0` was published at `40a15b1b`: the pin now names that
      revision, the executable hash `27FF817A…7F19` and the asset
      `colosseum-cli-0.1.0-windows-x64.zip` with GitHub's digest for it,
      `BE7D3A33…10F7`. **Two premises of this leaf were wrong and are
      corrected here.** There is no `SHA256SUMS` in the release; the
      per-asset digest GitHub serves is the source, which is the mechanism
      `setup_tools.ps1` already used for fastchess, and it is verified on the
      archive while the top-level hash is verified on the extracted
      executable. And the release line is squashed (`ba30829 Create CLI and
      infra`), so the superseded `0b78c29` is not an ancestor of the tag and
      the two revisions cannot be compared; only the hash distinguishes them,
      the release reporting the same `colosseum-cli 0.1.0` version string as
      the local build did. Staging ran end to end — the old binary refused
      against the new pin, archive digest verified, extracted, executable
      hash verified — after one fix: `Expand-Archive` validates the
      extension, so the temporary file keeps `.zip`. **Repeated on the
      released runner:** the dry run of the same gate resolves to a
      configuration identical to the superseded build's in **all 439 fields**
      including the config hash, so the adoption evidence carries over
      unchanged; parity **30 of 30**, the regenerated fixture byte-identical
      to the committed one; guard suite **22 of 22**; the release's own
      `self-test` 5 of 5. A registered experiment
      names its runner and never changes it mid-way. **Audit 2026-09-21** (`analysis/b2_audit_2026-09-21.md`):
      the nine run files dry-run with exit 0 on the staged CLI, a local
      build (sha256 `550CE5D0…DA11`) that is neither the qualified binary
      nor a release; the tag is the only outside dependency left, and the
      wrappers can be written before it. **B.2.6.1 landed 2026-09-21:**
      `tools/colosseum.ps1` runs the gate, the fixed match, the null pair,
      the tune and the gauntlet from the committed run files; every guard
      `sprt.ps1` and `spsa.ps1` enforce now has ONE implementation, in
      `tools/harness_common.ps1`, which both paths call, so they cannot
      drift; three guards the project had only in prose are now enforced —
      an idle host, the runner's SHA-256 pin, and `-ExpectBench` against the
      registered fingerprint. The resolved configuration is read back from
      the CLI's own dry run and refused when it is not Rarog's policy, and
      the run directory is read afterwards for faults. Checks, both
      committed: `tools/diag/colosseum_parity.py` matches **30 of 30**
      fields against `sprt_theta5000_vs_theta3900_20260920_082658`'s
      manifest, with a fixture test that mutates fourteen of them one at a
      time; `tools/diag/test_colosseum_guards.ps1` breaks one input per case
      and gets the refusal that names it, **22 of 22**, including four
      positive controls and both branches of the idle guard.
      `setup_tools.ps1` stages the runner from
      `tools/colosseum/colosseum.pin.json` (revision `0b78c29`, sha256
      `550CE5D0…DA11`) and refuses any other build; its four staging cases
      were smoke-tested with every touched file restored byte-identical.
      **B.2.6.1 DONE 2026-09-22, and with it B.2.6:** the 200-game
      `colosseum.ps1 -Mode match` of theta 5,000 vs theta 3,900 on
      `cli-v0.1.0` completed 2026-09-22 on an idle host (3% CPU), status
      `completed`, exit 0, 200 of 200 games, all terminations normal,
      zero faults, W-D-L 52-101-47, pentanomial [4, 25, 35, 34, 2], +8.7
      ± 31.1 Elo (a fixed match; it decides nothing);
      `colosseum_recount.py` reproduces every number from the PGN
      (`tools/results/colosseum-b261-live`, `run-record.json`
      `CBEB2F6D…AF94`). **Wrapper audit against the release, 2026-09-22
      (`506f4c7`), which corrects this record's first reading.** The
      runtime code the CLI runs is unchanged between the superseded
      `0b78c29` and `40a15b1` apart from refactors and a crash now
      reported with its exit status (still a `Disconnect` fault); every
      defect below was the wrapper's own, and was first entered here as
      a release change, which it was not. (1) The post-run fault regex
      matched no Colosseum build: the progress line changed shape at
      least five times during development, and both the superseded build
      and the release print `time: a-b; other: a-b; …`; an unreadable
      line only warned, so the zero-tolerance fault guard and the
      time-loss ceiling never ran. Faults now come from `colosseum-cli
      status --json`, the documented interface, and an unreadable view
      refuses the run. (2) Every non-zero exit was treated as a failure,
      but the codes are verdicts: an SPRT's H0 exits 1 and a cap stop 4,
      a calibration's inconclusive 4; such runs skipped every post-run
      check. (3) The recount oriented pairs by engine name, so a null
      pair was scored from White's side: +19.42 Elo on
      `colosseum-qual-symmetry`, whose runner record, −0.0 ± 2.0 with a
      symmetric pentanomial, is right; the null calibration stands. It
      now orients by the journal and checks a match against the
      checkpoint's pentanomial, the `match` record leaving it at zero by
      design. RAR-M60 and RAR-M61 used distinct names and are
      unaffected. (4) An absolute `-Dir` was joined onto the working
      directory. (5) A resume without `-Seed` drew a new seed, which the
      runner refuses on resume and a dry run does not check; the
      recorded seed is now carried over. Checked: guard suite **34 of
      34**, recount tests 7 of 7, parity tests pass; every local run in
      the released schema passes the new guard and its recount agrees;
      live through the wrapper, a 4-game match (exit 0), a 4-game
      one-binary calibrate (exit 4), a 2-pair SPRT (exit 4) and a
      one-iteration tune on a temporary N = 1 surface with an absolute
      `-Dir` (exit 0), in
      `tools/results/colosseum-smoke-{match,calibrate,sprt,spsa}` (the
      tune copied in from `%TEMP%`). Resume after Ctrl+C, run by the
      maintainer (`colosseum-resume-test`): stopped at 60 of 200, 61
      games kept, 139 played on resume at the carried seed, 200 unique
      journal games, recount equal to the checkpoint; the runner's
      "resuming 0 durable game(s)" counts only the journal after the
      last checkpoint, and its `run.log` records no stop or resume
      event, contrary to its docs, neither affecting results. The
      interrupted invocation's manifest simply ended, so it now records
      the interruption (`266bca8`). **Review 2026-09-22, a sixth
      defect:** `-Mode gauntlet` could never pass its own policy check.
      The dry run names the subcommand `tournament`, not `tournament run`,
      and resolves one `time_control` for the field rather than a clock
      per arm, so every gauntlet was refused as off policy; the mode had
      only been dry-run before the check existed. Fixed in the wrapper,
      which now also checks each participant's Hash against the policy
      and records a `-DryRun` invocation as such in its manifest, so a
      rehearsal is not read as an interrupted run. Two guard cases added
      (36 of 36); a two-game gauntlet through the wrapper on the released
      runner completed with status `completed`, exit 0, no fault
      (`colosseum_gauntlet_rarog-gauntlet-smoke_20260922_230525`). Every
      mode has now run live. Independent recount of the 200-game live
      run: [4, 25, 35, 34, 2], W-D-L 52-101-47, equal to the runner; the
      released archive and its executable re-hash to the pin.
      **B.2.6.2, DONE 2026-09-21:** nothing was retired. PROCESS gained a
      *Harness* section naming Colosseum the main path, the fastchess and
      weather-factory path the maintained backup until at least 2.5.0, and
      the three cross-check triggers — a runner, scheduler or topology
      change; a surprising result; a Colosseum re-pin. AGENTS,
      `tools/README.md` and `tools/colosseum/README.md` follow. The
      2026-09-17/18 parity runs are RAR-M60 (the gate replay: fastchess
      +94.00 ± 32.76 nElo against Colosseum's +94.08 ± 32.46 on the same
      arms) and RAR-M61 (the fixed match, where every interval overlaps and
      Colosseum's own spread is as wide as the gap between the instruments),
      both recounted from their PGNs. The next tune's shape is registered at
      15 slots and 30 games per iteration with the budget in games (RAR-M62,
      RAR-S78). Engine-specific tooling (builds, sidecars, bench
      fingerprints, profiling, counters, Texel) stays here by design.
    - **B.2.7 Re-tune of the most-moved B.2.3 coordinates on Colosseum —
      `V`, maintainer-run (RAR-S78).** Added 2026-09-19 by maintainer
      request. Surface: every coordinate at least one RAR-S75 step from
      its seed at N = 5,000, seeded at the rounded final theta, all others
      fixed there; steps sized to be detectable rather than range/16. It
      tests whether RAR-S75 was travel-limited, coupled, diluted by
      dimension or already at the optimum; the discriminating reads are in
      RAR-S78. The full registration (steps, horizon, slots and games per
      iteration as B.2.6.2 sets them, cap) comes before launch. Gated by an
      SPRT `[0,3]` against the head B.2.4b accepted on 2026-09-20. Waits for
      B.2.6. `spsa` was qualified on four coordinates at `r_end` 0.03, so
      this gate is also its first strength test at Rarog's scale, and
      weather-factory stays installed until it resolves
      (`analysis/b2_audit_2026-09-21.md`). **Registration completed
      2026-09-22 (RAR-S78), amended the same day before any game:** all
      82 coordinates restarted at theta_5000 with RAR-S75's steps, by
      maintainer decision, because the coordinates are coupled and a
      17-coordinate surface with 65 held fixed could only bound the
      available strength from below (the 17-coordinate design stays
      committed as `config_b27core`, not run); N = 5,000 × 30 games on 15 slots, `r_end` 0.0031, the
      same travel budget per coordinate as RAR-S75; `rarog-b27core-tune.exe`
      at 7,185,678; gate `[0,3]` against the B.2.4b head, cap 40,000
      pairs. Predictions frozen in the row. About 31 to 33 hours of tune
      and up to 15 of gate, both maintainer-run.
    - **B.2.8 First-search stalls — `I1`, CLOSED 2026-09-19
      (RAR-M59).** Added by maintainer decision after a harness lead. The
      KPK bitbase was built inside the first search that reached KPK (about
      34 ms on that search's clock), the hash table was converted to its
      shared form inside the first multi-threaded search, and helpers freed
      an unused 64 MiB table on it. Engine `9a7b663` and `c0e6ef7` build every
      table before input is read and do every conversion at `setoption`, as
      Stockfish and Reckless do; bench unchanged on both arms; a fresh-process
      test and a `configure` test fail with the fix disabled. Closed on that
      evidence by maintainer decision; the 10,000-game confirmation is
      Colosseum's qualification and runs there, its result appended to
      RAR-M59 when reported. **It was never played:** Colosseum closed its
      10.9m on 2026-09-19 without pursuing the target, and its qualification
      runs used binaries that predate the fix. RAR-M59 closes on 21,055
      post-fix fastchess games without a time loss (`analysis/b2_audit_2026-09-21.md`). The B.2.3 tune keeps its registered binary.
      The fixed pair for Colosseum: `rarog-startupfix-core-pext-pgo.exe` and
      `rarog-startupfix-base-pext-pgo.exe` at `c0e6ef7`.
- **B.3 Cluster 2 — proof searches and extensions — `I2`, then `V`.**
  **Researched 2026-09-23 on the fitted head (`analysis/b3_research_2026-09-23.md`,
  RAR-S79); B.0's handoff (analysis §3.4–3.5, §13.3) re-based.** What the
  head does today, `bench 13` at stride 1: NMP enters at 12.6% of interior
  nodes with a margin *below* beta and converts 22% (32% unfitted); its
  verification search disables the null move at one node only and fails 15
  times in 4,425; ProbCut's depth − 4 verification confirms 98.2% of qsearch
  passes; singular attempts 4.9% of interior nodes with a `4·d` margin where
  both donors use about `0.5·d`; IIR fires at 12.3% of interior nodes and
  Reckless has none. B.0's mechanism decisions stand; three things change on
  that evidence: NMP's population moves to cut nodes with a margin above
  beta, its verification becomes the donors' `nmp_min_ply` region (today's
  is inert) and the `cutoff_count` consumer lands; ProbCut keeps Rarog's
  measured SEE-gap filter and searched-move cap (RAR-S57/S58) and adopts the
  donor's margin-scaled verification depth, adjusted-beta re-search and lerp
  return, with the TT-served shortcut a switch; singular adopts the donor's
  graded double/triple extensions, soft multi-cut, TT-move demotion, −3
  negative extension, LDSE and the LMR margin term, seeded at Rarog's own
  `4·d` margin; IIR and the NMP and ProbCut populations become
  categoricals decided by zero-game screens, the populations defaulting
  to Rarog's measured `!tt_pv` any-type one (RAR-S57/S58). The RFP
  crossover check (packet) shows NMP's band is empty at depth 3 under
  the fitted RFP margin, as in the donor, so T1's yield is expected small.
  Seeds: shape from the donor, magnitude from Rarog's fitted value where one
  exists, else the donor's converted value (RAR-S75's lesson). **No in-check
  extension** (RAR-S06). **Gate bracket: the default `[0,3]`**, not B.0's
  `[0,5]`, fixed before any game: the prior is moderate and RAR-M10 shows
  `[0,5]` rejects a true +4 nElo the default accepts. Base: the accepted
  head at B.3.4's registration — the B.2.4b head, or the B.2.7 head if
  RAR-S78's gate accepts it first, in which case the branch is rebased and
  the screens and paired run repeat. Predictions frozen in RAR-S79: unfitted
  paired run +5 ± 12 Elo; fitted gate +8 ± 8 Elo (H1 0.60, cap 0.25, H0
  0.15); per-mechanism activation moves in the packet. Baselines measured
  on the head 2026-09-23: branching 1.700, WAC at 100k 221 and at 400k 260,
  agreement 41/50, canaries 87/116.
    - **B.3.1 Implement — `I2`, RESEARCH (returned a fourth time
      2026-09-23, after amendment 4's bounds).** (a1), `alpha` for a lone
      excluded move, and (a2), positive extensions only while `ply <
      2·root_depth`, are built; `CoreIirPolicy` is shrunk to its default
      (policies 0 and 2 still stop `bench 13` at position 15) and P8 is
      unfalsifiable. The arm reads 8,099,189 / EBF 2.466. The bounds do
      not bound seldepth: the default arm reaches the ply cap from depth
      17 on the forced-line positions, at up to 300× the head's nodes, and
      the head reaches it at depth 19 (packet Unknown 10). Third return,
      after T4, resolved by amendment 4: amendment 3 is applied: T3's activation
      gates pass at the default floor, KBNK passes on the arm, and the
      arm's `bench 13` is 10,396,945 / EBF 2.485. T4's `CoreIirPolicy` is
      built and neutral at its default. With IIR off or in the Stockfish
      form, singular extensions carry lines to the ply cap (seldepth 127)
      and `bench 13` does not finish. Only the accepted IIR bounds the
      arm's extensions (packet Unknown 9, *T4 record*). CI and the close-out
      wait for the amendment. Second return, after T3, resolved by
      amendment 3: T3 is built on the contract and amendment 2.
      Its three P5 activation gates fail, with every wire proven live
      (packet Unknown 8, *T3 record*): candidates are 3.24% of interior
      nodes because the donor's `depth >= 5 + tt_pv` floor replaces the
      head's `depth >= 4` (5.44% with it); three-ply extensions are 21% of
      attempts because the non-PV two-ply bar is at most zero; multi-cut is
      32% because the donor's rule is looser than Rarog's (19.3% under
      Rarog's). The same multi-cut rule makes the arm fail the KBNK
      regression test (`kbnk_positions_are_driven_to_mate`, 2 of 5
      budgets), which passes with Rarog's rule. The arm with T1–T3 reads
      6,841,250 / EBF 2.422. T4 waits for the amendment's choice among
      (a)–(d); recommended (c) plus (a), Rarog's multi-cut with the other
      legs as reads. First return, after T1 and T2, resolved by amendment 2:
      two premises failed on the stride-1 counters, with every wire proven
      live, and the rule for a false premise stopped the leaf before T3.
      First, ProbCut's donor verification is never deeper than the head's
      `depth − 4`. A third of the head's qsearch passes are at depth 4, where
      the verification repeats the qsearch, and among passes verified by a
      real search survival is 97.3% on the head and 97.9% on the arm, so P4's
      "below 95%" cannot be met. Second, the null-move verification region
      refuses no null move in `bench 13`, because reverse futility answers
      its shallow nodes first. The packet (Unknowns 6–7 and *B.3.1
      implementation record*) holds the evidence and three options. The
      research amendment picks one, re-registers P4's survival leg, then T3
      resumes on the contract below. Original handoff: Feature
      `b3proof`, off by default, the off arm exact at 7,185,678 / EBF 2.444
      on all 40 lines. Tickets in the packet's contract: T1 NMP (population
      switch, margin above beta with the `cutoff_count` term, donor
      reduction, TT-bound shortcut, `nmp_min_ply` verification region as
      per-thread state), T2 ProbCut (cut nodes, TT and quiet-TT-move gates,
      Rarog's filter and cap, margin-scaled depth with adjusted-beta
      re-search, lerp return, `CoreProbcutTtServed` switch), T3 singular
      (`potential_singularity`, graded extensions to +3, soft multi-cut,
      demotion, −3, LDSE, the LMR term), T4 `CoreIirPolicy` switch. Every
      constant a `Core*` spin at its seed; categoricals are switches.
      The packet's forward rules R1–R7 bind the implementation: one `est`
      accessor, cfg-gated blocks inside the node rather than a second
      node, per-thread state only, the root excluded, counter names
      kept for B.4 and B.9, decisive-score helpers on the TB bounds,
      the entry-then-per-move depth order.
      Invariants and their tests, counters and decision-trace lines as
      listed; `AblationMask` bits unchanged. Verified by the off-arm
      fingerprint, the test suites on both arms, fmt and clippy on every
      feature set, and the stride-1 counters read against P3–P5 before any
      screen runs.
    - **B.3.2 Diagnostics — `V`.** Rule 8's ladder at the packet's floors:
      stride-1 counters, the oracle differential, branching, WAC at 100k and
      400k, agreement, canaries (regression rule), time-to-depth; the bit
      sweep (2, 3, 4, 6); the five categoricals `CoreNmpNodes`,
      `CoreProbcutNodes`, `CoreProbcutTtServed`, `SingularTtDepthMargin`
      and `CoreSingularFloor` (amendment 3; `CoreIirPolicy` was shrunk to
      its default under amendment 4, so P8 has no screen) as baseline/A/B on
      the screens, zero games choosing the default and one 2,000-game
      categorical run only where the screens disagree; then the 2,000-game
      unfitted paired run against the head (maintainer-run), which governs.
      Deliverable `analysis/b32_screens_<date>.md` with a review.
    - **B.3.3 Sweep and SPSA — `V`.** Curvature sweep of `CoreNmpBase`,
      `CoreNmpREval`, `CoreProbcutBase`, `CoreSingularMargin` and
      `CoreLmrSingularOffset` (0.5x–2x, bench and WAC at 100k, classification
      frozen first); if curved, the 35-coordinate surface (the three LMR
      bases the new term lands on included) on Colosseum at
      15 × 30 from its own registration, maintainer-run; theta baked in one
      engine commit.
    - **B.3.4 Gate — `V`.** Fitted `b3proof` PGO build against the accepted
      head, Colosseum `sprt-default` `[0,3]` nElo, cap 20,000 pairs,
      registered with binaries and hashes before any game; H1 flips the
      default, H0 or the cap rejects the cluster as a unit (PLAN rule 6) and
      the bit sweep record says what B.6 may carry.
- **B.4 Cluster 3 — quiescence — `I2`, then `V`.** Reckless-shaped qsearch:
  TT cutoff, corrected stand-pat, fail-high interpolation, LMP at three
  moves, SEE pruning by margin, TT write on exit, check evasions only when
  in check. Target: Rarog's qsearch share (62% larger than the oracle's per
  interior node) without losing tactical suite results at equal nodes. SPRT
  `[0,3]`. **Dependency on B.2, recorded from Manta's MAN-S36 review:**
  count-based late-move skipping, low-depth unverified null cutoffs and
  zero-depth reduced probes all assume that a mate threat by a *quiet* move
  stays visible one ply later; a captures-only quiescence makes that false,
  and Manta's core was blind to WAC.001's mate in two through depth nine
  until direct quiet checks were generated at the first quiescence ply.
  Rarog's quiescence today generates captures only unless in check, and B.2
  makes pruning aggressive before B.4 touches quiescence. Therefore B.2's
  canaries include mate threats by quiet moves, B.4 may not remove any
  first-ply check generation B.2 turned out to rely on, and "check evasions
  only when in check" is measured against those canaries, not assumed from
  the donor. **Screens: rule 8's cluster ladder**, registered before
  implementation; under its canary regression rule a quiet mate-threat
  canary the baseline solves may not be lost. The paired run governs, and
  the curvature sweep precedes any SPSA. **Research card, added
  2026-09-17 (maintainer decision; RAR-M19): the search's piece-value
  scale.** The evaluator's material is Texel-fitted (middlegame
  88/394/418/537/1131, endgame 123/239/290/486/930, refit four times);
  the search carries a separate, never-fitted vector `PIECE_VALUES` =
  100/320/330/500/900 that feeds SEE (`PRODUCTION_SEE_VALUES`), capture
  and promotion ordering in both pickers, the quiescence delta margin
  (`stand_pat + queen + 200`), the bad-noisy futility's victim term, the
  corrected eval's material scale and the ProbCut SEE gap; B.2's SEE
  thresholds are seeded on it (donor ×0.75). Ordering consumers need only
  a self-consistent scale; the margin consumers compare it with
  evaluation units, where a 900-unit queen meets an 1131-unit one. B.4
  owns the decision because the delta margin and the SEE thresholds are
  its mechanisms: (1) audit each consumer for ordering-only against
  margin use; (2) decide whether the five values plus the delta margin
  join B.4's SPSA surface as coordinates (Stockfish fits its
  `PieceValue` by SPSA; Manta parameterises SEE), or whether the margin
  consumers switch to the evaluator's units with the ordering scale left
  fixed; (3) if fitted, the same scale must feed every consumer, and
  `CROSS_ENGINE_SEE_VALUES` stays frozen for the benchmark. The HCE
  vector stays Texel's (C.1–C.4). Zero-game evidence first: the SEE
  census and the qsearch delta-prune counters at stride 1 on the accepted
  head, then a registered A/B if a margin consumer changes units.
- **B.5 Cluster 4 — root, aspiration, iterative deepening — `I2`, then `V`.**
  Aspiration delta from eval and PV stability, optimism, root move node
  accounting, forgotten-mate and aborted-loss guards, PV table. Multi-PV is
  delivered earlier by B.2.0.2; B.5 keeps its contract (identity at
  `MultiPV = 1`, line semantics above 1).
  SPRT `[0,3]`. Root-only LMR relief keeps its accepted place unless B.2's
  formula subsumes it, which B.0 decides. **Screens: rule 8's cluster
  ladder**, registered before implementation. The paired run governs;
  time-to-depth replaces pooled NPS as the speed screen (aspiration and
  iterative-deepening changes move the tree); the curvature sweep precedes
  any SPSA.
- **B.6 Search SPSA — `V`.** One joint SPSA over the coordinates the four
  clusters left live, only if B.0's curvature evidence and the cluster
  results justify it. Registered surface; PGO bake; SPRT `[0,3]`.
- **B.7 Search speed pass — `I1`, then `V`.** Behaviour-neutral throughput
  work on the new modules (allocation, layout, prefetch, inlining measured in
  search not on the bench), pooled-PGO NPS with a +0.5% floor per change,
  exact fingerprint. The 4.11b lesson stands: a bench-column win is a screen,
  not a result.
- **B.8 Cleanup — `I1`.** Remove dead parameters, unconsumed switches, the
  old `MovePicker`, evidence/provenance plumbing without a named consumer,
  and any diagnostic without an owner. Exact fingerprint; no game gate.
  Owner test for a diagnostic: a counter or probe is kept only if a tracked
  tool or analysis reads it by name (`tools/diag/bench_counters.py`,
  `phase4_differential.py`, or a B-phase ledger row); the rest of the 286
  `diag` names, the `correction_probe`, `lazy_probe` and `smp` submodules
  included, are deleted here unless B.2–B.5 named them. Every remaining
  `#[expect]` must still fire (the lint wall reports unfulfilled ones);
  `search_options.rs`'s single `#[allow]` keeps its written reason or goes.
- **B.9 Checkpoint — `V`.** Re-measure the deficit meters: RAR-O-series
  equal-time G(0) against the oracle, fixed-node depth and EBF, the
  reference-anchored geometric branching factor from B.2.2 (the durable
  tree-shape number; node ratios at one depth overstate the gap because
  nominal depths are not equal coverage across engines), pooled-PGO
  NPS, conversion instrument, and a pool gauntlet against the four target
  engines and Basilisk at 1T (400 games each). Record attributed Elo per
  accepted cluster from the SPRTs, and the checkpoint against the budget
  table. Remove the `ablate` feature afterwards; archive the oracle branches
  as tags (A.2.2) if not already done. **Freeze the search head for C.**

### Active workflow register

One row per open leaf in the active phases (A and B). The checker requires
the state and class here to match GUIDE's suffix. Later phases carry only a
class until they open.

| Leaf | Workflow state | Class | Current decision |
|---|---|---|---|
| B.2.7 | IMPLEMENTED | V | **Fully registered 2026-09-22 (RAR-S78), amended the same day before any game to all 82 coordinates restarted at theta_5000 with RAR-S75's steps (coupling), N = 5,000 × 30 games, r_end 0.0031, binary and dry run in the row; the tune is the maintainer's to run.** Added 2026-09-19 (RAR-S78, design registered): Colosseum re-tune of the coordinates at least one step from their seeds at N = 5,000; full registration before launch; B.2.4b passed 2026-09-20 and B.2.6 closed 2026-09-22, so it is the next executable leaf: register, then hand over the tune |
| B.3.1 | READY_FOR_IMPLEMENTATION | I2 | **Resumed 2026-09-23 after amendment 5** (packet): bounded by cost, a per-line extension budget (sum of positive extensions ≤ iteration depth) replaces the ply bound, (a1) stays, a deep-iteration cost screen (≤ 3× the head's nodes at depths 16–20) joins B.3.2; then step 2, CI, close-out. Earlier: **returned a fourth time after amendment 4's bounds** (`90a7f87`: `alpha` for a lone excluded move, positive extensions only while `ply < 2·root_depth`, `CoreIirPolicy` shrunk to its default, P8 unfalsifiable; off arm exact; arm 8,099,189 / EBF 2.466). The bounds do not bound seldepth: the default arm reaches the ply cap at depth 17 on the forced-line positions, at up to 300× the head's nodes, and the head reaches it at depth 19, so the seldepth-based regression test's premise is false (packet Unknown 10, *Amendment 4 record*). Step 2, CI and close-out wait. Before that, **resumed 2026-09-23 after amendment 4** (packet): the extension chain is a defect, bounded by Stockfish's `alpha` return for a lone excluded move and a `ply < 2·root_depth` guard on positive extensions, with a forced-line test and a watchdog bench at every IIR policy; the IIR categorical stays if both then finish; then CI and close-out. Earlier: **returned a third time after T4** (`c1bec33` amendment 3 applied, with T3's gates and KBNK passing, arm 10,396,945 / EBF 2.485; `e90d8ba` T4, default tree unchanged; CI and close-out not done). Under `CoreIirPolicy` 0 or 2 the singular extensions run lines to the ply cap (seldepth 127 on bench positions 11 and 15) and `bench 13` does not finish, so IIR is load-bearing, not a free categorical (packet Unknown 9, *T4 record*: watchdog runs, head-vs-arm table, choice 8 = the −3 held to a one-ply child at depth 4). Recommended: (c), SF's alpha return at an exclusion node with no other move plus a shrunk IIR categorical. Before that, **resumed 2026-09-23 after amendment 3** (packet): Rarog's multi-cut rule (the donor's fails KBNK), Rarog's candidate floor as default with the donor's as `CoreSingularFloor`, bars kept, `CoreSingTripleBase` into the sweep, P5's three-ply and multi-cut legs reads; then T4, CI for the arm, close. Earlier: **returned a second time after T3** (`40dcd93`; off arm exact at 7,185,678 on all 40 lines; arm T1–T3 at 6,841,250 / EBF 2.422; T4 not built): P5's attempts, three-ply and multi-cut gates fail, with live wires, because of three contract terms (donor candidate floor, non-PV two-ply bar ≤ 0, the donor's looser multi-cut). The arm also fails `kbnk_positions_are_driven_to_mate` in `tests/endgames.rs` (2 of 5 budgets; T1+T2 passed), and only removing the multi-cut or restricting it to Rarog's rule clears it. Packet Unknown 8 and *T3 record* hold the counters, wire proofs, the attribution and options (a)–(d). Recommended: (c) plus (a), meaning Rarog's multi-cut rule, the other legs as reads, then T4. Before that, **resumed 2026-09-23 at T3 after amendment 2** (packet): option (a) for ProbCut, the region kept with a depth-20 wire check in B.3.2, choices 1–7 accepted, the demotion condition restated; P4's survival leg withdrawn, calibration in RAR-S79. Earlier the same day: **returned to RESEARCH after T1 and T2** (`cbc602e`, `2b9507b`; off arm exact at 7,185,678 on all 40 lines; T3 and T4 not built): two packet premises are contradicted by stride-1 counters with live wires (packet Unknowns 6–7). ProbCut's donor verification can never be deeper than the head's, so P4's survival leg is out of reach, and the region refuses no null move at bench depth. The packet's *B.3.1 implementation record* holds the counters, wire proofs, choices and options (a)–(c). Resume at T3 once the research amendment chooses. Recommended: (a) |
| B.3.2 | RESEARCH | V | Rule 8's ladder at RAR-S79's floors, the bit sweep, five categoricals on zero-game screens, the 2,000-game unfitted paired run (maintainer-run) |
| B.3.3 | RESEARCH | V | Curvature sweep of five coordinates, then the 35-coordinate SPSA on Colosseum if curved (maintainer-run), theta baked |
| B.3.4 | RESEARCH | V | SPRT `[0,3]` vs the accepted head, cap 20,000 pairs, registered with binaries before any game; H1 flips the default |
| B.4 | RESEARCH | I2 | Waits for B.3 |
| B.5 | RESEARCH | I2 | Waits for B.4 |
| B.6 | RESEARCH | V | Conditional on curvature evidence |
| B.7 | RESEARCH | I1 | After B.6 or its skip |
| B.8 | RESEARCH | I1 | After B.7 |
| B.9 | RESEARCH | V | Closes the programme; freezes the search head |

## Phase C — Evaluation programme

**Goal:** recover the measured 329-Elo same-search evaluation deficit, with
the search frozen at the B.9 head, by re-implementing the evaluation families
in Stockfish 11's classical shape where its conditioning is stronger, keeping
ours where the evidence says ours is better, refitting the whole surface
after every family cluster, and giving endgame handling its own bounded
cluster.

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
existing toolchain, frozen test reported once; (6) a PGO bake and SPRT `[0,3]`
(`[3,10]` when the family's residual is large); (7) ledger row. Fit loss is a
screen and a falsifier, never acceptance (RAR-E03 lost 17 Elo with better
loss).

- **C.0 Investigation: family map, residuals and cluster order — `R3`.**
  Produce the evaluation programme document (C.0 names it): the six-family map from the
  maturity record refreshed on the B.9 head, per-family residual and
  activation evidence, the donor comparison of conditioning, the shared-input
  plan, the cluster order by expected value, the datagen and refit protocol
  for the programme (corpus name, size, splits, label policy from the label
  audit), and frozen handoffs for C.1 and the first family cluster. **No
  engine implementation.**
- **C.1 Evaluation restructure, behaviour-neutral — `I1`.** Split `eval.rs`
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
  `eval/trace.rs`; `src/kpk.rs` moves to `eval/endgame/`; `diag_lazy_dual`
  and the 21 `lazy_*` counters stay only if the lazy path stays, since
  `lazy_margin` is decided here. The `texel` trace-reconstruction test is
  part of the suite run, never a speed measurement.
- **C.2 Datagen and label contract for the programme — `V`.** Generate the
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
  `manta-hce-fit-v3` (1,109 free, 17 fixed, 103 excluded).
- **C.3 King safety cluster — `I2`, then `V`.** King danger in the donor's
  shape: attacker units and weights, safe and unsafe checks by piece type,
  weak squares in the king ring, king-flank attacks and defence, shelter and
  storm by file with the castling-destination alternative, queen-absent
  reduction, and the nonlinear danger-to-score map. Rarog's existing nonlinear
  danger table is the seed for the map. Refit, gate.
- **C.4 Threats and mobility cluster — `I2`, then `V`.** Mobility with a
  mobility area that excludes own king, queen, blocked pawns and pawn-attacked
  squares; threats: minor and rook attacks on weak enemies, hanging pieces,
  restricted squares, threat by pawn push, king threats, slider and knight
  attacks on the queen, weak queen protection. Shared attack maps from C.1.
  Refit, gate.
- **C.5 Endgame handling and winnability cluster — `R3` investigation with
  `I2`/`V` sub-steps.** The rescoped endgame section. Its goal is measured
  conversion and correct draw recognition where games actually go, not
  coverage of a function list.
    - **C.5.1 Classification and instruments — `R2`.** Adopt the registered
      family order (`tools/diag/endgame_ranking_v2.json`), confirm each family's
      kind (verdict, scale, conversion) against the code, and name the deciding
      instrument per family: theory truth (`endgame_truth.py`), drawn-cohort
      overclaim (`endgame_drawn.py`), conversion (`endgame_conversion.py`),
      floors, and the A.4 conversion audit at the game level.
    - **C.5.2 Generic winnability and scaling — `I2`.** The donor's scale
      factor logic in our form: pawn-count scaling for the stronger side,
      opposite-bishop scaling by non-pawn material and passers, rule-50
      scaling in the scale rather than only the global damping, and an
      initiative/complexity term conditioned on pawn count, king distance and
      both-flank pawns. This is what decides KRPPKRP (5.4% of games, no local
      7-man truth), KPsK (4.5%) and KBPsK (2.6%) generically. Refit, gate with
      an endgame-start cohort and STC.
    - **C.5.3 Conversion cluster: KXK, KBNK, KQKR — `I2`.** Mate drives and
      verdict families with the largest occurrence (KXK 37.8% of the set) and
      the largest measured conversion deficit (KQKR 23/13/3 at 60k/200k/600k
      nodes). Rule-50 damping interaction measured here, sign not assumed.
      Deciding instrument: conversion at bracketed budgets plus theory vetoes.
    - **C.5.4 Rook versus minor cluster: KRKN, KRKB, KRPKB — `I2`.** Three
      families with 100% or 99.6% drawn-cohort overclaim at +300 and the same
      over-representation in Rarog's games; one scaling mechanism, one gate.
    - **C.5.5 Rook and pawn cluster: KRPKR, KRKP, KPK, KPKP — `R2`.** Audit
      the existing scalers and the KPK bitbase integration; repair the 30.7%
      KRPKR overclaim if the drawn cohort supports it; close KPK/KPKP
      `NO_CHANGE` if their 4–5% overclaims do not select a mechanism.
    - **C.5.6 Measure-first families: KPsK, KBPsK, KBPPKB, KQKRPs — `R2`.**
      Measure coverage after C.5.2, decide whether any specific recogniser is
      still justified, otherwise close them as served generically.
    - **C.5.7 Theory sweep: KBPKB, KBPKN, KNNKP, KNNK, KQKP — `I1`.** Sub-1%
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
    - **C.5.8 Endgame gate and closure — `V`.** One endgame-start cohort SPRT
      plus one STC SPRT for the whole C.5 cluster after its refit; floors and
      theory vetoes re-run; conversion instrument re-measured; KRPPKRP's 7-man
      hold recorded as an explicit exclusion unless independent truth appears.
- **C.6 Pawns and passers cluster — `I2`, then `V`.** Passed pawns with king
  proximity, blocker ownership and type, path safety and attack, unstoppable
  and unblocked conditions, rook behind; pawn structure conditionality
  (doubled, isolated, backward, connected by rank and phalanx, weak lever).
  Refit, gate.
- **C.7 Material, imbalance, phase and pieces cluster — `I2`, then `V`.**
  Imbalance in the donor's quadratic form seeded from current material terms,
  phase interpolation review, bishop pair and bishop-pawn colour terms,
  outposts, minor behind pawn, rook on open and semi-open files, trapped
  rook, weak queen, king protector distances. Refit, gate.
- **C.8 Refit cycles — `V`.** After the family clusters: regenerate data with
  the accepted head, refit the whole surface, gate; repeat while a cycle
  accepts, stop at the first that does not. Initialization control (neutral
  start against accepted start) in the first cycle. Each cycle records the
  C.2 manifest it fitted from, so every refit states what was and was not
  fitted; a coefficient's status changes only by a recorded decision, never
  by a cycle quietly widening the free set.
- **C.9 HCE SPSA of nonlinear residue — `V`.** Only the activated nonlinear or
  global terms the linear trace cannot fit; skipped with a written reason if
  the surface is flat.
- **C.10 Search re-fit after the new evaluation — `V`.** The search's
  cp-valued margins were fitted on the B-era scale. One joint SPSA over the
  registered cp coordinates, PGO, SPRT `[0,3]`.
- **C.11 Checkpoint — `V`.** Same-search deficit against Stockfish's classical
  HCE re-measured (a fresh hybrid build at the C head is required; the oracle
  package recipe is on the tagged `hybrid` branch), conversion instrument,
  pooled NPS, pool gauntlet at 1T. **Freeze the classical evaluation.**

## Phase D — Clock, threads, robustness

- **D.1 Time management — `R2` investigation, `I2`/`V` sub-steps.** Audit the
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
- **D.2 Lazy SMP quality — `R2` investigation, `I2`/`V` sub-steps.** 4T and
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
- **D.3 Engine lifecycle and protocol robustness — `R2`, then `I1`.** UCI
  parsing and dispatch, stop/ponder/infinite semantics, new-game resets,
  malformed input, panic reporting, Syzygy probe policy and thread safety.
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
- **D.4 Tablebase policy — `R2`.** Root and interior probing depth and limits,
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

- **E.1 Attribution checkpoint — `V`.** Re-run the B.2.0 architecture review
  on the B.9 and C.11 heads first. Final head against 2.3.2 and against
  the B.9 and C.11 heads at STC, `10+0.1` and 4T; attributed Elo per programme
  from the accepted SPRTs; deficit meters; NPS; the maturity checklist
  (family map without unknown rows, every slot with a fitting instrument,
  every accepted representation reconstructing through `EvalTrace`).
- **E.2 Target gate — `V`.** The pool measurement defined in section 1, at 1T
  and 4T. Met, or not met with the measured shortfall per engine recorded.
  **The binding arm is 1T:** RAR-M46 measured 4T as the easier arm for three
  of the four targets, by 26 to 75 Elo, so Phases B and C are judged against
  the 1T column.
- **E.3 Release — `M`/`V`.** Version, changelog, release notes, fmt, debug and
  release suites, clippy, feature builds, fingerprint, PGO assets, ISA
  verification, CI matrix, tag and publish on maintainer instruction. Version
  is 3.0.0 if E.2 is met, else 2.5.0. The release is cut through the
  tag-driven flow of **E.3.1**, which also carries the two workflow checks
  this release owed (tag equals version; one fingerprint asserted across the
  matrix) and may land any time earlier.
    - **E.3.1 Tag-driven release flow — `I1`, READY_FOR_IMPLEMENTATION,
      tooling only; may land any time, must land before E.3.** Added
      2026-09-22 by maintainer decision, modelled on Colosseum's release
      lanes (in `D:/code/colosseum`: the two release workflows, the
      `colosseum-release` tag validator and DEVELOPMENT's release lanes). It absorbs
      the two workflow checks E.3 already owed.
      **Defect it removes.** Today a release exists on GitHub before any
      binary is built: the maintainer creates the tag and the release by
      hand, types the notes into the form, and `build.yml` runs on
      `release: published` to build the nine-cell PGO matrix and attach the
      assets. A failed cell leaves a published release with a missing asset,
      which is what the repair-by-dispatch path exists for; nothing checks
      that the tag equals `Cargo.toml`'s version, that the tagged commit is
      on `master`, or that the notes match `CHANGELOG.md` (the shape of the
      RAR-E16 baseline confusion); and the smoke only checks that `bench`
      prints a positive number, so a stale or wrong source can ship with a
      plausible count.
      **Target.** The maintainer runs `git tag vX.Y.Z` on `master` and
      pushes the tag; nothing else. The workflow validates first (tag equals
      the manifest version, the tagged commit is an ancestor of
      `origin/master`, a `## [X.Y.Z]` section exists in `CHANGELOG.md`),
      then builds every cell read-only, runs `bench 13` on each asset and
      asserts one fingerprint across the matrix equal to the fingerprint the
      tagged source declares in GUIDE's checkpoint (a cross-platform
      mismatch is investigated, never waived: RAR-P14, RAR-P16), and only
      then one final job with `contents: write` creates the release with
      every asset and the notes extracted from that changelog section,
      marked latest. A `workflow_dispatch` candidate mode builds and checks
      every asset from a chosen ref and keeps them as workflow artifacts
      with no tag and no release: the rehearsal. Only
      `v<major>.<minor>.<patch>` tags trigger it, never the `arm/…` or
      `oracle/…` markers.
      **Frozen decisions.** Asset names stay exactly today's
      (`rarog-vX.Y.Z-<os>-<arch>[.exe]`), so README's `releases/latest` link
      and the per-tier guidance keep working. Release notes live in
      `CHANGELOG.md`, whose `[Unreleased]` section becomes the version's
      section at release time, as A.7 and 2.4.0 already did; the GitHub form
      is never typed into again. The repair dispatch is dropped: a failed tag
      run is repaired by deleting the tag, fixing `master` and tagging again,
      which is safe because nothing is published until every cell passed.
      `ci.yml` keeps running on pushes to `master` and the release workflow
      does not duplicate its matrix. The arm64 Windows linker workaround,
      `rust-toolchain.toml` and the `cargo xtask build --pgo` pipeline move
      over unchanged.
      **Compatibility, checked 2026-09-22:** no script, document or sibling
      repository consumes Rarog asset names or the `release: published`
      trigger (`D:/code/lichess-bot` and `D:/code/colosseum` searched);
      the version has one source, `Cargo.toml`, which `id name` prints, so
      tag-to-version is one check; `git tag` holds 23 `v*` release tags and
      the `arm/…` and `oracle/…` markers.
      **Work and exit.** The workflow; a local check that refuses a wrong
      tag, a commit off `master` and a missing changelog section before
      anything is pushed (`cargo xtask release-check vX.Y.Z`, as Colosseum's
      does); PROCESS's release procedure rewritten step by step with the
      exact commands; then one candidate run on the current head as the
      exit check: nine assets built, one fingerprint asserted, nothing
      published. Tooling commits only; no engine input changes and the
      fingerprint does not move. Tag, push and publish stay the
      maintainer's.
## Phase F — NNUE

**Rules.** Own data only, generated by Rarog's classical head and later by its
NNUE heads. Reckless is the runtime and trainer-pipeline donor; the
architecture ladder is ours. The classical evaluation stays in the tree as the
datagen baseline and the fallback until F.9 replaces it in releases.

- **F.0 Investigation: runtime, data pipeline and first architecture — `R3`.**
  Board event interface and accumulator ownership (dirty pieces, per-thread
  per-ply accumulators, king buckets, refresh cache); trainer choice
  (`D:/code/net_trainer` against Bullet) with feature ordering, quantisation
  and export contracts; data format, deduplication and split policy; the first
  architecture (768×N perspective network with output buckets); the cost
  ledger inherited from the board audit. Frozen handoffs for F.1–F.4.
- **F.1 Board events and accumulator scaffolding — `I2`.** Behaviour-neutral
  for the HCE: factual move deltas, evaluator-owned stacks, validity and
  refresh semantics, randomized unwind tests, exact fingerprint, pooled NPS
  cost recorded.
- **F.2 Data generation at scale — `V`.** 30–60M unique positions from the
  classical head under the adjudication-off profile, by-game splits,
  manifests, tablebase and hard-position cohorts; hashes frozen. Maintainer-run.
- **F.3 Trainer hardening and baseline nets — `I2`, then `V`.** Deterministic
  pipeline, two seeds per configuration, validation selects, frozen test
  reports once.
- **F.4 Scalar integration — `I2`.** `quantised.bin` contract, integer-exact
  conformance against the trainer's reference evaluation, clean HCE fallback.
- **F.5 Incremental and SIMD — `I2`, then `V`.** Same-net incremental parity
  on every move type, SIMD tiers (AVX2, PEXT builds, ARM NEON), scalar
  reference retained, pooled-PGO NPS attribution.
- **F.6 Search re-fit for the network — `V`.** Score scale, correction
  histories, margins, qsearch and SEE thresholds re-fitted on the new
  evaluator (C.10's protocol).
- **F.7 Architecture ladder — `R3` with `I2`/`V` sub-steps.** Output buckets,
  king buckets with mirroring, then relation and threat inputs as in
  Reckless, one axis at a time; each net gated against the previous.
- **F.8 Data frontier — `V`.** On-policy refresh with the strongest net,
  deduplication, hard-position mining; repeat while a cycle accepts.
- **F.9 NNUE release — `M`/`V`.** Beat the classical release at STC, LTC
  and 4T; platform matrix; publish.
- **F.10 CCRL top-100 gate — `V`.** Submit; the list decides. Shortfall
  measured against the pool and fed back into F.7/F.8.

## Phase G — Scaling, platforms and the top 50

- **G.1 High-thread and NUMA — `R2`, then `I2`.** 8/16/32T scaling, TT and
  net placement, large pages, thread affinity policy.
- **G.2 Platform and product — `I1`.** Chess960 on demand, distributed
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
- **G.3 Frontier — `R3`.** Larger nets, data scaling, search fit at LTC; the
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
| `GUIDE.md` | Operator guide, model mapping, prompts, the full checkbox board, checkpoint and next action |
| `PLAN.md` | This roadmap: objective, rules, phases, protocols |
| `EXPERIMENTS.md` | Frozen predictions, results, calibration, retry triggers, recipes |
| `PROCESS.md` | Research/handoff template and recurring build, fit, gate and release procedures |
| `HISTORY.md` | Completed work, retired numbering and the number map; never a source of the next step |
| `analysis/` | Per-leaf analyses and measurement records; raw artifacts stay local and ignored |
| `docs/archive/` | Verbatim archived roadmaps |
