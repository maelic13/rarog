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
   Rybka 4 and Fritz 16**.
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
| Fingerprint | `bench 13` **7,601,220 / EBF 2.474**; engine source unchanged since `c80df74`, accepted by RAR-E15, and reproduced by every 2.4.0 build in A.7, A.8.3 and A.8.4 | GUIDE checkpoint; RAR-M48 manifests |

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

If those bands are right the classical head lands within reach of Rybka 4 and
Fritz 16 and near Critter; Houdini 3 may only fall in the NNUE stage. Each
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
8. **State the measurement layer.** Theory truth, move quality, conversion,
   fixed-node tree shape, NPS and game strength are different units with no
   exchange rate. Counters, node counts, EBF, tactical suites and fit loss
   explain or screen; only a registered final-PGO SPRT or the target gate
   accepts.
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
      400k. **Not played:** the registered 2,000-game paired run (binaries
      built, hash-bound and smoke-tested) and the ETW profile the review asked
      for before it (binary staged; needs an elevated shell). P1 assumed
      0.93–0.97x speed, so at 0.683x the −40 floor cannot separate a defect
      from the speed loss. **Decision owed by the maintainer:** re-plan B.2
      (razoring first, then ProbCut and singular; B.7 for the per-node cost)
      and whether the paired run is played on this arm now.
    - **B.2.3** SPSA over the registered live coordinates (expected 40–70),
      `tools/spsa.ps1`, immutable horizon, staged stop. Maintainer-run.
    - **B.2.4** Gate: registered SPRT `[0,10]` against the B.1 head, cap
      sized from RAR-M10; then ledger row and calibration. Accepted head
      becomes the base for B.3. No null calibration precedes it: the 1T
      harness is calibrated and shared with Basilisk, and switching
      adjudication off symmetrically (RAR-M17) does not reopen it (RAR-M03;
      the RAR-E06 registration's calibration disposition, accepted by the
      maintainer 2026-09-01). **Correction 2026-09-14:** B.0 had added a
      "null calibration owed" precondition here claiming no calibration
      followed RAR-M17; that contradicted those two records and is withdrawn
      by maintainer decision.
- **B.3 Cluster 2 — proof searches and extensions — `I2`, then `V`.** **B.0
  handoff frozen 2026-09-13 (analysis §3.4–3.5, §13.3): NMP adopts the
  donor's entry margin above beta (both donors demand about 150 Rarog
  units; Rarog demands none and converts 24% of attempts), the adaptive
  reduction, the TT-bound shortcut and the `nmp_min_ply` verification
  region; ProbCut keeps the 4.7c filter, which is the donor's own
  `probcut_beta − eval` threshold, and adopts the donor's depth and return
  shape with the oracle's TT-served shortcut as a measured switch; singular
  adopts the double/triple margins, multi-cut lerp, −3 negative extension,
  LDSE and the LMR margin term; IIR versus hindsight is decided here on
  B.2.2's numbers.** NMP
  with adaptive reduction and verification, ProbCut with reduced-depth
  verification and the TT-served shortcut, singular extensions with
  double/triple margins, multi-cut, negative extension, low-depth singular
  extension, IIR versus hindsight-depth policy. Rarog's own evidence rules:
  no in-check extension unless a new measurement says otherwise. Same
  sub-step shape as B.2 (implement, diagnose, SPSA if curvature justifies,
  SPRT `[0,5]`).
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
  the donor.
- **B.5 Cluster 4 — root, aspiration, iterative deepening — `I2`, then `V`.**
  Aspiration delta from eval and PV stability, optimism, root move node
  accounting, forgotten-mate and aborted-loss guards, PV table. Multi-PV is
  delivered earlier by B.2.0.2; B.5 keeps its contract (identity at
  `MultiPV = 1`, line semantics above 1).
  SPRT `[0,3]`. Root-only LMR relief keeps its accepted place unless B.2's
  formula subsumes it, which B.0 decides.
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
| B.2.2 | READY_FOR_IMPLEMENTATION | V | Zero-game screens run 2026-09-15: NPS 0.683x, WAC 100k 204, agreement 35, canaries 75/116 below floors; one ablation sweep points at razoring (`analysis/b22_screens_2026-09-15.md`). The paired run and the ETW profile are prepared, maintainer-run; the re-plan is the maintainer's decision |
| B.2.3 | RESEARCH | V | Waits for B.2.2; maintainer-run SPSA |
| B.2.4 | RESEARCH | V | Waits for B.2.3; SPRT `[0,10]` registered before games |
| B.3 | READY_FOR_IMPLEMENTATION | I2 | Handoff frozen by B.0; waits for the accepted B.2 head |
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
      where the evidence is clean (KNNK already measured clean).
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
  is 3.0.0 if E.2 is met, else 2.5.0. **Two workflow checks are added before
  this release, as tooling work that may land any time earlier:** (1) the
  release job fails when the tag does not equal the manifest version — today
  `build.yml` names the built file from `Cargo.toml` and the uploaded asset
  from the tag, so the two can disagree silently, which is the shape of the
  RAR-E16 baseline confusion; (2) every asset in the matrix runs `bench 13`
  and the job **asserts** one fingerprint across all of them, replacing the
  comment that tells the operator to read the node count in the log. Manta's
  release workflow already does both and rejects the tag otherwise.
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
| Fixed-node shape | oracle differential at stride 1, depth at 300k nodes, EBF, tactical suite at fixed depth and equal nodes | B clusters |
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
- Tag, push and publish only on maintainer instruction.

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
