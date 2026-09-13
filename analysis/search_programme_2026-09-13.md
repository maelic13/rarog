# Search programme investigation — PLAN B.0

Revision `b9eef62` on `dev` (engine source identical to the 2.4.0 release,
`bench 13` 7,601,220 / EBF 2.474), 2026-09-13. Research leaf, class `R3`.
**No engine source changed.** This document is the B.0 deliverable PLAN
names: the mechanism map against the donor, the cluster contents, the scale
ratio, the survivor list, the SPSA surfaces, the counter re-keying, the
AblationMask disposition, the four structural questions A.6 left open, and
frozen handoffs for B.1, B.2 and B.3 with a frozen prediction for B.2.

Sources of record: Reckless `31d9cd6` (`src/search.rs`, `movepick.rs`,
`history.rs`, `transposition.rs`, `stack.rs`, `thread.rs`, `time.rs`,
`evaluation.rs`); Stockfish dev `59aae690` (2026-09-09) where Reckless is
silent; the classical Stockfish `9587eeeb` that the RAR-O03 oracle runs; and
Rarog's own ledger. Manta ADR-0070/0071 and MAN-S36 are a second worked
example, not a donor. Everything measured here was measured on this machine,
idle, on the release-identical binary `tools/test_engines/rarog-o03arm-pext-pgo.exe`
(fingerprint verified by its manifest) and archived under
`tools/results/b0-20260913/` (ignored storage, hashes in RAR-M50).

## 1. Verdict

**B.1 and B.2 are `READY_FOR_IMPLEMENTATION`; B.3's handoff is frozen and
its state is `READY_FOR_IMPLEMENTATION` contingent on the accepted B.2 head.**
The programme shape PLAN registered — one Reckless-shaped selectivity core
adopted as a unit, seeded, fitted and gated — survives the investigation,
with four corrections that change what B.2 builds and how B.2.2 screens it:

1. **The tree-shape target is wrong as written.** PLAN B.2.2 registers a
   *ceiling* on the reference-anchored geometric branching factor. Measured
   today over depths 4–14 on the phase-4 suite, Rarog's branching factor is
   **1.630**, the classical oracle's **1.736** and Reckless's **1.697**. Rarog
   already grows *slower per ply* than both references; what it does is
   start from a tree **3.5x** the oracle's at depth 4 and end **1.9x** at
   depth 14. Its excess nodes are a shallow-depth constant factor (quiescence
   1.02 qnodes per interior node against the oracle's ~0.6, in-check interior
   nodes 1.3x, quiets handed to the search 2.2x), not per-ply growth. A
   branching ceiling would reward exactly the over-selectivity RAR-S53 and
   RAR-S54 already measured. B.2.2's screen is re-registered below as a
   **window** on branching plus **fixed-node** quality screens.
2. **The deficit is decision quality at a fixed budget, and it is large in
   the tactical domain.** On the 300 WAC positions at **100,000 nodes** the
   oracle solves **242**, Reckless **224**, Rarog **200**; at 400,000 nodes
   267 / 247 / 237. At equal nominal depth 10 the three are level
   (188 / 200 / 190). That is the equal-node form of RAR-O03's "0.97 ply,
   248 Elo" finding, on a corpus with answers.
3. **Seeds cannot be converted by one scalar.** The measured scale ratio
   (PLAN's definition, mean absolute static evaluation over the 40 bench
   positions plus 10,000 hce-v3 positions) is **0.406** against Reckless's
   raw network output and **0.457** against the search-facing eval Reckless
   actually compares its margins to. But Reckless's margins are sized for an
   evaluation whose *error* is far smaller than Rarog's HCE's: Rarog's own
   search-minus-static residual averages **128 cp** on training nodes, a
   third of its mean absolute eval. Converting Reckless's razoring or
   reverse-futility margins by 0.457 would fire them at nearly every node.
   The classical oracle, by contrast, plays **248 Elo better with Rarog's
   own HCE** using its own margins scaled by a known factor (0.485). B.2's
   seed policy therefore uses **three columns** per constant — Reckless
   converted, the oracle's proven value converted, Rarog's fitted value — and
   a stated rule for which one seeds (section 8).
4. **Two co-adapted contract defects, both invisible to the counters that
   were read before, are named as the likely carriers of the decision-quality
   gap** and go into B.2 together: (a) **46.7% of Rarog's LMR reductions land
   in quiescence** (`lmr_qs_clamped` 723,977 of 1,549,665 at bench 13) —
   both donors clamp the reduced depth to at least one main-search ply, and
   Manta added the same floor after tracing a missed mate to a zero-depth
   probe; (b) Rarog **re-searches only 1.3%** of reduced moves (20,535 of
   1,549,665) against the oracle's ~4% at depth 8, with a mean reduction of
   **3.05 plies**; reduced moves that are never given the chance to fail
   high cannot correct the ordering that reduced them. Both are shapes, not
   constants, and neither was among the four LMR candidates RAR-S64-era work
   measured at zero.

The expected size of B.2 is **tens of Elo, not hundreds** (section 12). The
hundreds the maintainer wants are the sum of B.2–B.6 *and* the evaluation
programme: the same-search evaluation gap is 329 Elo and Rarog's search is
starved by evaluation noise it must compensate for with margins.

## 2. The question, the evidence layers, and what B.0 measured

**Question.** The 2.4.0 head loses 247.97 ± 10.89 Elo at equal time to the
classical Stockfish search running Rarog's own evaluation (RAR-O03), at a
depth deficit of only 0.97 ply. The matched ablation attributes 272 ± 18 of it
to LMR plus shallow-depth pruning (marginal value, not headroom). Which
mechanisms, in which shapes, carry the deficit, and how should the Reckless
architecture be cut into clusters for this engine?

**Evidence layers used, stated per PLAN rule 8.** Fixed-depth branching
profiles and fixed-node depth are tree-shape numbers. WAC at fixed nodes is
tactical move quality. Bench counters at stride 1 are activation and
conversion rates. The scale ratio is a units measurement. None of these is
Elo; the ledger's game results (RAR-O03, RAR-S54, RAR-S57/58, RAR-S64,
RAR-S66, RAR-S71, RAR-S06) are the only strength evidence cited.

### 2.1 Reference-anchored branching profile (new, `tools/branching_profile.ps1`)

Phase-4 suite `phase4_suite_v1.epd` (50 positions, five cohorts), Hash 64,
one thread, fresh process per depth, depths 4–14. UCI node counts (interior
plus quiescence, as each engine reports them).

| Engine | Geometric branching 4→14 | Per-position span, median | Nodes at d4 | Nodes at d8 | Nodes at d14 | ms at d14 |
|---|---:|---:|---:|---:|---:|---:|
| Rarog 2.4.0 | **1.630** | 1.599 | 67,579 | 769,010 | 8,964,846 | 3,515 |
| Oracle: Stockfish `9587eeeb` + Rarog HCE | **1.736** | 1.702 | 19,208 | 220,643 | 4,788,062 | 3,143 |
| Reckless 0.10-dev `31d9cd6`-era build | **1.697** | 1.694 | 28,146 | 275,009 | 5,582,351 | 4,750 |

Rarog/oracle node ratio: **3.52x at depth 4, 3.49x at depth 8, 1.87x at depth
14**. The gap shrinks with depth because Rarog's per-ply growth is the lowest
of the three. Nominal depths are not equal coverage across engines (Reckless
and Stockfish dev both extend singular moves by up to three plies and reduce
by negative extensions; the classical oracle extends checks), so the ratios
at one depth overstate or understate; the branching factor is the durable
statement and it says Rarog is not under-selective per ply.

### 2.2 Depth at a fixed budget (new, `tools/diag/fixed_budget_probe.py nodes 300000`)

Same suite, `go nodes 300000`, Hash 64. Means are inflated by positions where
a mate collapses the tree (Rarog's zugzwang cohort median is 99 because
several of those positions are found mates); medians are the readable number.

| Cohort (n) | Oracle median depth | Rarog | Reckless |
|---|---:|---:|---:|
| opening (12) | 16.0 | 13.5 | 18.0 |
| quiet middlegame (8) | 18.5 | 16.5 | 16.5 |
| tactical (12) | 19.0 | 15.0 | 17.0 |
| endgame (10) | 23.5 | 17.0 | 18.0 |
| all 50, median | **19** | **16** | **18** |

Throughput in the same run: Rarog **3.53 MNPS**, oracle **2.23**, Reckless
**1.06** (the oracle's FFI evaluation is cheaper on this suite than the 1.64
MNPS prior from the ablation runs; the Rarog/oracle speed ratio here is
**1.58x**, not 1.80x). Best-move agreement with the oracle at 300k nodes:
40 of 50; Reckless agrees with either on 31 of 50, as expected from a
different evaluation.

Read with RAR-S53: depth at equal nodes is double-edged. Rarog reached 2.5
more plies than Basilisk at equal nodes and still lost by 65 Elo, and here it
reaches three plies *fewer* than the oracle while solving fewer tactical
positions. Depth is registered as a monitored quantity, never a target.

### 2.3 Tactical quality at fixed nodes and the canary anchors (new)

`src/wac.epd`, 300 positions. Solved = the engine's `bestmove` is in the EPD's
`bm` set.

| Budget | Oracle | Rarog | Reckless |
|---|---:|---:|---:|
| 100,000 nodes | **242** | **200** | 224 |
| 400,000 nodes | **267** | **237** | 247 |
| depth 10, answer stable from some depth ≤ 10 | 188 | 190 | 200 |

At 100k nodes 185 positions are solved by both, **57 by the oracle only, 15 by
Rarog only**. The same engines are level at fixed depth 10, which is the
cleanest demonstration this project has that a nominal ply is not a unit of
work and that Rarog's nodes buy less than the oracle's.

The depth-10 run also records, per position and engine, the first move of the
PV at every completed depth. From it: the depth from which each engine's
answer stays correct ("stable depth") is the **reference anchor** B.2.1's
canaries need. 49 positions have an oracle stable depth ≤ 6 where Rarog is
three or more plies later or unsolved by 10; 16 of them Rarog does not solve
at all by depth 10 (WAC.017, 034, 068, 069, 077, 085, 098, 170, 182, 191,
195, 203, 228, 231, 233, 253). WAC.001 (`Qg6`, the quiet mate threat Manta's
core was blind to) is answered by this oracle from depth **9**, by Rarog
never through 10, and by Reckless never through 10 (it answers `Nh5` at 10).
The full table is `tools/results/b0-20260913/wac_anchor_d10.json`.

### 2.4 Scale ratio (new)

Corpus: the 40 `BENCH_FENS` plus 10,000 positions sampled with seed 20260913
from `tools/texel/data/hce-v3/test.csv` (the split the accepted evaluation
was tested on). Rarog: `Evaluator::evaluate` on the head crate (side-to-move
relative, rule-50 damped, lazy path live — what the search sees). Reckless:
the `eval` command's final line, internal units, White's POV; "search-facing"
applies Reckless's own `correct_eval` material scaling and rule-50 damping
with zero optimism and correction.

| Set | n | mean \|Rarog\| | mean \|Reckless raw\| | mean \|Reckless search-facing\| | ratio raw | ratio search-facing |
|---|---:|---:|---:|---:|---:|---:|
| all | 10,040 | 389.1 | 959.5 | 850.7 | **0.406** | **0.457** |
| bench 40 | 40 | 236.8 | 672.4 | 634.3 | 0.352 | 0.373 |
| datagen 10k | 10,000 | 389.7 | 960.7 | 851.5 | 0.406 | 0.458 |
| opening (material ≥ 8000) | 1,766 | 132.9 | 381.2 | 415.8 | 0.348 | 0.320 |
| middlegame | 3,505 | 265.6 | 786.3 | 773.6 | 0.338 | 0.343 |
| endgame (< 4000) | 4,729 | 577.6 | 1306.3 | 1072.0 | 0.442 | 0.539 |

Per-position ratio median **0.350** (IQR 0.24–0.52). With signs aligned to
side-to-move the two evaluations correlate at r = 0.883 and agree in sign on
85.3% of positions; the least-squares slope of Rarog on Reckless is 0.384.

Units the seeds must respect: Reckless's SEE and material values are
109/403/435/679/1242 against Rarog's 100/320/330/500/900, ratio **0.75**
(pawn 0.92, minors 0.79/0.76, rook 0.74, queen 0.72). The oracle feeds
Rarog's evaluation into Stockfish scaled by `PawnValueEg/100 = 2.06`
(`hybrid_eval.cpp`), so a classical Stockfish evaluation-unit constant is
**0.485** Rarog units, and its SEE thresholds, in `PieceValue[MG]` units
(124/781/825/1276/2538), are about **0.40** Rarog SEE units for pieces
(0.81 for pawns).

**Recorded ratio: 0.457 for evaluation-unit constants, 0.75 for
SEE/material-unit constants, both from Reckless; 0.485 and 0.40 for the
classical oracle's constants.** They are starting points SPSA moves.

### 2.5 Bench 13 counters at stride 1 on the head (new, `bench_counters.py`)

Diagnostic build of `b9eef62` (`--features diag`, separate target dir; it
reproduces 7,601,220). Interior nodes 2,766,067; quiescence nodes 2,820,172.

| Quantity | Value | Reading |
|---|---:|---|
| qnodes per interior node | 1.02 | oracle ~0.62 at depth 8 (RAR-S55 v4); Rarog's shallow tree is quiescence-heavy |
| in-check interior nodes | 11.4% | oracle 7.7% at depth 8 |
| `rfp_cut` / interior nodes | **41.9%** | oracle 32.5% at depth 8; `razor_drop` 4.3% |
| `nmp_cut` / `nmp_attempt` | **24.1%** | oracle 83% at depth 8; verification passes 946 of 952 |
| `probcut_cut` / `probcut_nodes` | 19.0% | 32,338 of 32,719 qsearch passes convert; the depth-4 verification refutes 1.2% |
| `singular_attempt` / nodes | 4.3% | extend 1: 25,903; extend 2: 26,967; multi-cut 21,111; negative 11,536; 28% of attempts seeded by a depth-3 Lower entry |
| `iir_applied` / nodes | 8.6% | 173,905 for a missing TT move, 63,063 for a shallow non-PV entry |
| `lmr_applied` | 1,549,665 | mean reduction **3.05 plies**; `lmr_zero_reduction` 125,671 |
| `lmr_qs_clamped` / `lmr_applied` | **46.7%** | reductions that reduce straight into quiescence |
| `lmr_research` / `lmr_applied` | **1.33%** | oracle 4.0% at depth 8 (RAR-S55 v4: 2,006 / 49,856) |
| `lmp_prune` / `move_seen_quiet` | 76% | 6,492,913 of 8,557,365 quiets are count- or history-pruned per move; `lmp_nodes` 10.3% of nodes |
| cutoff composition | 0.57 quiet per capture cutoff | oracle 1.37; first-move cutoff 88.1% (oracle 84%) |
| TT | hit 61.9%, cut 9.5% of nodes | `tt_bound_not_usable` 9.7% of hits; stores: full 984k, stand pat 1.11M, qsearch tail 635k, qsearch move 308k, ProbCut 32k |
| correction updates | 12.3% of nodes | 52% capture-caused; mean residual capture **178 cp**, quiet **73 cp**; overall **128 cp** |
| aspiration | 905 fails / 400 aspirated iterations | 2.3 re-searches per iteration from depth 4 |

## 3. Mechanism map: Rarog against the donor

Verdicts: **ADOPT** the donor's form in the named cluster; **KEEP** Rarog's
form, with the local evidence that earns it; **DROP** Rarog's mechanism
without replacement; **DEFER** to a later leaf. "SF" is Stockfish dev
`59aae690`; "oracle" is classical `9587eeeb`. Every ADOPT is a mechanism
written in Rarog's own structure; constants are seeds (section 8).

### 3.1 Node entry, draws, mate bounds

| Mechanism | Rarog | Reckless | Verdict |
|---|---|---|---|
| Node typing | `is_pv: bool`, `cut_node: bool` runtime | `NodeType {PV, ROOT}` compile-time, `cut_node` runtime | **ADOPT (B.1)**: `Root/PV/NonPV` consts, `cut_node` stays runtime (both donors keep it runtime because it flips per child). The A.6 wording "PV / Cut / All constants" is corrected. |
| Draw detection | `can_declare_draw_in_search` (rule-50 with mate precedence, repetition in history); no upcoming-repetition | `is_draw` + cuckoo `upcoming_repetition` raising alpha to the draw score | **DEFER**: `analysis/draw_policy_2026-09-08.md` disposed the draw policies `NO_CHANGE`; upcoming repetition is a D.3/D.4-adjacent contract with its own RAR-S18 history. Not in B. |
| Draw score | 0 | `nodes % 5 - 2` jitter | **DEFER** with the above (RAR-S18 bundled root-aware repetition and lost). |
| Mate-distance pruning | present | present | KEEP |
| Ply cap | `MAX_PLY = 128` | 240 | KEEP 128 (fits `i8` TT depth and the pv/stack arrays); revisit only if seldepth reaches it. |

### 3.2 Transposition table

| Mechanism | Rarog | Reckless | Verdict |
|---|---|---|---|
| Entry | 10 B: key16, score, static_eval, move, depth i8, flags (2 bound, 1 pv, **1 speculative**, 4 age) | 8 B: move, score, raw_eval, depth+1, flags (2 bound, 1 tt_pv, 5 age); 21-bit keys packed per cluster | **KEEP Rarog's layout** (local 3×10 B per 32 B, shared 6×10 B per 64 B, both settled by RAR-P11/P12/P16); **DROP the speculative bit**, age returns to 5 bits. |
| Store on probe miss | never (static eval travels only with search results) | writes `raw_eval` with `TtDepth::SOME` on every miss; SF the same | **ADOPT (B.2)** — this is the "static eval stored on every probe miss" PLAN names. Fingerprint-changing, so it belongs to the cluster, not to B.1. |
| Estimated score | `refine_eval` (bound-consistent TT score replaces the corrected eval; accepted RAR-S02 in qsearch) | identical rule (`estimated_score`) | KEEP; the mechanism is already the donor's. RAR-S27/S29's depth-floor variants stay closed. |
| Early cutoff | `depth >= d`, bound matches window, non-PV only | `tt_depth > depth - (tt_score < beta)`, `Upper` needs `!cut_node \|\| depth > 5`, `Lower` needs `cut_node \|\| depth > 5`; quiet TT move gets a history bonus on a `>= beta` cutoff if the parent searched < 4 moves; no cutoff at `fiftymove_clock >= 90` | **ADOPT (B.2)**: the node-type-conditioned admission and the TT-cutoff history feedback (Rarog's `tt_cutoff_bonus_pct` fitted to 0 was a fraction of the cutoff bonus, not the donor's `min(190d-81, 1691)` form); rule-50 guard adopted. |
| Replacement | `depth - age_delta/4`, entries with the same key overwrite unless shallower | same-key overwrite refused when `depth + 4 + 2*tt_pv <= old depth` and same age; else `depth - 4*relative_age` | **ADOPT** the refusal rule (B.2, cheap); the quality formula is already equivalent. |
| Typed provenance | `OutcomeKind` on every store; `NodeEvidence` decode; speculative bit consumed only behind dead switches | none | **DROP (B.1)** — section 6.1. |
| `tt_pv` propagation | `is_pv \|\| stored` gates RFP/razor/NMP/ProbCut jointly (`tt_pv_veto` 1.1% of nodes) | `tt_pv \|= entry.tt_pv`; fail-low propagates the parent's `tt_pv` (`bound == Upper && move_count > 2`); consumers: RFP, NMP margin, singular margin, LMR terms, replacement | **ADOPT (B.2)** the donor's per-consumer use; the joint veto goes. |

### 3.3 Static evaluation and correction

| Mechanism | Rarog | Reckless | Verdict |
|---|---|---|---|
| Corrected eval | `raw + Σ w·table / 16384` | `(raw·(21032+material) + optimism·(1548+material)) / 27015`, `·(200 - clock)/200`, `+ correction`, clamped | **ADOPT (B.2)** the *shape* with the material term and rule-50 damping moved into the search; Rarog's evaluator damps rule-50 itself today, so B.2 must remove one of the two (the evaluator's, behind the C.1 boundary, or apply the search's only when the evaluator's is off). The material coefficients are network-specific and are seeded at neutrality (`(21032+m)/27015 → 1`), left as SPSA coordinates. Optimism is a B.5 root product; B.2 lands it at 0. |
| Correction tables | pawn, minor, own non-pawn, their non-pawn, continuation (1 ply, `(piece,to)` slot), weights 135/80/104/160/152, no rule-50 bucket, gravity at 16384, update `(diff·depth).clamp(±1024)` at cutoffs and exact/fail-low nodes, **capture-caused included** (52% of updates, 2.3x noisier, RAR-S38) | pawn, non-pawn[White], non-pawn[Black], continuation at plies 2 and 4 keyed by `(in_check, capture, piece, to)` sub-table, **16 fifty-move buckets**, `/64`, update `(148·depth·diff/128).clamp(-4678, 2496)` only when `!in_check && !best_move.is_noisy()` and the bound direction agrees, at every completed node | **ADOPT (B.2)** the donor's tables, buckets, admission and update. The minor-key table is Rarog-only with no standalone evidence; keep it as a sixth table seeded at its fitted weight and let B.2.3 zero it. Per PLAN's Manta note, the continuation-correction tables are introduced as **shadow producers first** (trained, admission counted, not read) for one B.2.1 ticket, then consumed. RAR-S16's binary capture guard stays rejected; the donor's admission rule is the graded form RAR-S38 asked for. |
| `improving` | `static > stack[ply-2]`, false after a check; RAR-S66 (ply-4 fallback) stopped unresolved | `improvement = eval - eval[ply-2]` else `eval[ply-4]` else 0; `improving = improvement > 0`; the *magnitude* feeds RFP, NMP, LMR, LMP | **ADOPT (B.2)**: the continuous `improvement` with the ply-4 fallback lands as part of the cluster, which is how RAR-S66's question is answered without a standalone gate. |
| Opponent-worsening | absent (4.6 audit lead 5) | absent in Reckless; SF: `staticEval > -(ss-1)->staticEval`, feeds RFP and hindsight | **ADOPT via hindsight (B.2)**: Reckless's `eval_delta = eval + eval[ply-1]` is the same signal, consumed by the hindsight reductions. |
| Eval-difference history training | absent | at node entry, parent's quiet move gets `clamp(812·(-(eval + parent_eval))/128, -144, 324)` when `depth < 6 \|\| no TT entry` | **ADOPT (B.2)** with the quiet history. |

### 3.4 Node-level pruning and reductions

| Mechanism | Rarog | Reckless (SF dev / oracle where useful) | Verdict |
|---|---|---|---|
| Razoring | `!tt_pv`, depth ≤ 3, `eval + 274·d < alpha` → qsearch (4.3% of nodes) | `!PV`, `est < alpha - 237 - 254·d²`, `alpha < 2048`, TT move not quiet, bound not Lower, **no depth cap** (oracle: depth 1, 527 SF units) | **ADOPT (B.2)** the donor's shape; seed the margin from column rule 8.2. |
| Reverse futility | `!tt_pv`, depth ≤ 8, `eval_for_pruning - (52 + 51·not_improving)·d - 3·\|corr\|/128 >= beta` (41.9% of nodes) | `!tt_pv`, `est >= beta + max(2, 1140·d²/128 - 120·improvement/1024 + 22·d + 669·\|corr\|/1024 - 54·(own pieces unthreatened) - 19)`, `!is_loss(beta)`, `!is_win(est)`, **no depth cap**, returns `lerp(est, beta, 0.6945)` (oracle: `227·(d - improving)`, depth < 6) | **ADOPT (B.2)** the shape (quadratic depth, improvement magnitude, correction magnitude, threat term, lerp return). Rarog's margin is already 2–4x the donor's converted value at low depth and it still fires at 42% of nodes; B.2.2 must report `rfp_cut` per depth. |
| Null move | `allow_null`, depth ≥ 3, `eval_for_pruning >= beta - 12·d - 35·improving`, non-pawn material, `R = 4 + d/4 + clamp((eval-beta)/200, 0, 3)`, verify at depth ≥ 10 at the root only; **24% conversion** | `cut_node` only, `!potential_singularity`, `est >= beta + max(2, 337 - 9·d + 110·tt_pv - 94·improvement/1024 - 21·(cutoff_count[ply+1] < 2))`, material > 491, `ply >= nmp_min_ply`, no capture-Lower TT move; `R = (4407 + 917·improving + 265·d + 477·clamp(est-beta, 0, 1187)/128)/1024`; bound shortcut `tt_score` when Lower and `depth-2 <= tt_depth`; verification at depth ≥ 16 with the `nmp_min_ply` region (oracle: `staticEval >= beta + 311 - 33·d - 33·improving + 112·ttPv` and `eval >= beta`) | **ADOPT (B.3)**. Both donors demand a margin *above* beta (≈150 Rarog units); Rarog admits any eval above `beta - 12d`, which is the "entered too cheaply" shape 4.7c fixed for ProbCut. RAR-S58's 4.7a moved the entry to `eval >= beta` with the old 12d margin and measured nothing; the donor form is not that candidate. The `nmp_min_ply` region replaces the root-only verification (`nmp_nested_attempt` 76 today). |
| ProbCut | depth ≥ 4, `probcut_beta = beta + 180`, **4.7c filter**: SEE ≥ `probcut_beta - static_eval`, cap `2 + 2·cut_node` moves, qsearch then `depth - 4`, store Lower at `depth - 3`, return `score - (probcut_beta - beta)` | `cut_node`, `probcut_beta = beta + 254 - 85·improving`, picker threshold `probcut_beta - eval` (**the same filter**), qsearch then depth `base - (score - probcut_beta)/319`, adjusted-beta re-search, store at `probcut_depth + 1`, return `lerp(score, beta, 0.2695)` (SF dev adds the TT-served shortcut `beta + 428`; the oracle has `probcutBeta` TT-served at 0.89% of nodes) | **ADOPT (B.3)** the donor's depth and return shape; **4.7c is the donor's filter**, so "keep ours" and "adopt" coincide. The TT-served shortcut (oracle/SF, absent in Reckless) is a B.3 sub-candidate with its own switch. |
| IIR | depth ≥ 4, no TT move or shallow non-PV entry → `depth -= 1` (8.6% of nodes) | **none**; SF: `!followPV && !allNode && depth >= 6 && !ttMove` | **KEEP in B.2, DECIDE in B.3**: B.2 lands hindsight reductions alongside the existing IIR so B.3 can measure "IIR versus hindsight" as PLAN asks; the donor of record has none. |
| Hindsight reductions | absent | parent reduction ≥ 2249/1024 and `eval_delta < 0` → `depth += 1`; `!tt_pv && depth >= 2 && parent reduction > 0 && eval_delta > 57` → `depth -= 1` | **ADOPT (B.2)**. Requires `stack[ply].reduction` (the field RAR-S64 deleted, now with a live consumer). |
| Cutoff count | absent (RAR-S13 and RAR-S60 rejected `cutoffCnt` as a standalone reduction increase) | `cutoff_count[ply+2] = 0` at entry, `+= 1` on a beta cutoff; consumed by NMP margin and two LMR terms | **ADOPT (B.2)** as a producer with the LMR consumer; the NMP consumer is B.3. The RAR-S60 note stands: the grandchild-reset semantics are what make it non-inert. |

### 3.5 Singular and extensions

| Mechanism | Rarog | Reckless | Verdict |
|---|---|---|---|
| In-check extension | none (**RAR-S06 +30.75 for removing it**) | none (SF dev: none; oracle: yes) | **KEEP**. |
| Singular | `mv == tt_move`, depth ≥ 4, `allows_singular` (Lower/Exact within 3 plies, non-mate), `singular_beta = tt_score - 4·d`, depth `(d-1)/2`; extend 1, or 2 if `!is_pv` and below `beta - 20`; multi-cut returns `singular_beta`; negative −1 if `tt_score >= beta` | `potential_singularity` (depth ≥ 5 + tt_pv, `tt_depth >= depth - 3`, bound ≠ Upper, non-decisive); margin `depth` (or `⌈d/4⌉` on Exact) `+ depth·(tt_pv && !PV)`; double/triple margins with PV, quiet-TT-move and correction terms; multi-cut returns `lerp(singular, beta, 0.4027)`; `singular_score > tt_score` clears the TT move; negative **−3**; LDSE `depth <= 7 && cut_node && est <= alpha - 25 → +1`; `tt_move_score - singular_score` feeds LMR | **ADOPT (B.3)**; B.2 keeps Rarog's form. The 3-ply depth margin, RAR-S31's parked margin 2, and the ProbCut-seeded singular population (28% of attempts) are all inside B.3's SPSA/diagnostics, not standalone gates. |
| Extension cap | +2 | +3 and −3 | with B.3. |

### 3.6 Move picker and histories

| Mechanism | Rarog | Reckless | Verdict |
|---|---|---|---|
| Stages | TT, good captures (SEE ≥ 0), generate quiets, quiets, bad captures; full list at root/in check/excluded | Hash, GenerateNoisy, GoodNoisy by SEE threshold `-score/47 + 116` (or 1 when the TT move is quiet and > 2 noisy already picked), Quiet, BadNoisy; `skip_quiets` moves straight to BadNoisy; root re-scores each pick | **ADOPT (B.2)**. |
| Noisy scoring | `20M + 16·(victim + promo) - attacker + cap_history` (good), `-2M + …` (bad) | `14232·captured/1024 + noisy_history[piece][to][captured][to_threatened] + 4558·queen_promo + (200000 - 20000·pt)·in_check` | **ADOPT (B.2)**. |
| Quiet scoring | `2·main + pawn + low_ply/(1+ply) + cont(1,2,4,6) + 32000·direct_check`; killers 16M/15.9M, countermove 15.8M tiers | `1763·quiet/1024 + pawn + 1614·cont1 + 1066·cont2 + 1086·cont4 + 1051·cont6 (all /1024) + escape[pt]·threatened_from + 10723·checking_square - 8875·threatened_to + 3446·offense - 4494·king_wall_pawn`; **no killers, no countermove, no low-ply history** (SF dev: low-ply yes, killers no) | **ADOPT (B.2)** and **DROP** killers, countermove and low-ply history with the cluster (rule 3: no standalone gate). RAR-S65 (killer travel) is superseded, as A.2.3 already records. |
| Quiet history | `[color][from][to]` | `[stm][from_threatened][to_threatened][from][to]`, max 8192 | **ADOPT (B.2)**; needs a per-position `all_threats` bitboard (section 6.5). |
| Noisy history | `[attacker][to][victim]` | `[piece][to][captured_type][to_threatened]`, max 12800 | **ADOPT (B.2)**. |
| Pawn history | 4096 buckets × (piece,to) | 512 buckets × (piece,to), max 8192 | **ADOPT** the donor's size as a seed; a coordinate. |
| Continuation history | four flat tables keyed `(prev piece,to) × (piece,to)` at plies 1,2,4,6, divisors 1/1/2/3 | sub-tables keyed `(in_check, capture, piece, to)` at plies 1,2,4,6, pointer stored on the stack, max 15320 | **ADOPT (B.2)**: the check/capture context is the "check/capture context in history indexing" 4.9b never executed. |
| Bonus/malus | `min(174·d - 264, 2491)`, malus `min(210·d, 1877)`; surprise scaling 119%; exact-node quiet bonus 31%; cross-category capture malus 25% | quiet `min(184·d, 1742) - 72 - 42·cut_node`; quiet malus `min(171·d, 1099) - 46 - 31·n_quiets`, scaled by `1024²/(1024 + 45·i)²` by search index; noisy `min(96·d, 885) - 43 - 87·cut_node`; cont bonus `min(97·d, 1098) - 74 - 48·cut_node`; fail-low bonus to the parent's quiet move with a five-term factor; post-LMR cont bonus `min(233·d - 86, 1550)` when re-searched and cut; quiet-check TT bonus on TT cutoffs; **applied at exact nodes too** (`best_move.is_present()`), not only at cutoffs | **ADOPT (B.2)** the whole update policy. Rarog's 8.4 coverage knobs (exact bonus, capture malus, surprise) are subsumed; RAR-S59's rejected continuation malus returns inside the donor's index-scaled form and is not a standalone candidate. |
| Ageing | halve every table per search | no ageing; gravity only (SF dev decays main history by 729/1024 per search) | **ADOPT (B.2)** no ageing for the new tables, as a coordinate `history_decay_pct` seeded at 100 (none). |

### 3.7 Move-loop pruning and LMR

| Mechanism | Rarog | Reckless | Verdict |
|---|---|---|---|
| Index | `searched` (moves actually searched); pruned moves do not advance it; `SelectivityCountConsidered` measured 0 | `move_count` counts every move handed by the picker (pruned included), and `stack[ply].move_count` is read by the child | **ADOPT (B.2)** — free, inside the cluster. |
| LMP | disjunction of eval-margin (d ≤ 3), count `> 1 + 2d²/3 (+d improving)` (d ≤ 8), two history thresholds; **pruned move per move**, `skip_quiets` switch off | `!in_check && !is_direct_check && quiet && !is_win(beta) && move_count >= (2818 + 78·improvement/16 + 1351·d² + 74·history/1024)/1024` → `skip_quiets` (picker stops generating quiets; **direct checks exempt** before make) | **ADOPT (B.2)**; the Manta amendment (direct quiet checks survive the skip) is the donor's `!is_direct_check`. |
| Quiet futility | d ≤ 8, `eval_for_pruning + 211 + 135·d + 3·\|corr\|/128 <= alpha`, `!gives_check` | `depth < 14`, `eval + 79·d + 55·history/1024 + 77·(eval >= beta) + 555·\|corr\|/1024 - 127 <= alpha`, raises `best_score` to the futility value (fail-soft), sets `skip_quiets` | **ADOPT (B.2)**, fail-soft included. RAR-S15's fail-soft loss was a de-tuning of fail-hard-fitted constants; inside a refit cluster the objection lapses. |
| Bad-noisy futility | none | `depth < 11`, picker stage BadNoisy, `eval + 84·d + 82·history/1024 + 24 <= alpha` → break | **ADOPT (B.2)**. |
| History pruning | inside LMP's disjunction (`quiet_hist < -5617·d`, d ≤ 7; `< -10000`, d ≤ 4) | `depth < 5 && history < -948·d` (history = quiet + cont1 + cont2) | **ADOPT (B.2)**. |
| SEE pruning | captures only: `see_ge(mv, max(-66·d - cap_hist/8, -955))`, d ≤ 8; quiet SEE prune switch off (4.6.4 registered, never run) | quiets `(-12·d² + 56·d - 27·history/1024 + 27).min(0)`, noisy `(-7·d² - 36·d - 39·history/1024 + 14).min(0)`, both `!in_check`, no depth cap | **ADOPT (B.2)** both branches; the quiet branch is the 4.6.4 mechanism landing inside its cluster. Rarog's `see_ge_quiet_aware` already prices quiet moves (Manta had to add this). |
| Prospective depth | `SelectivityProspectiveDepth` switch off (measured 0) | **none** — Reckless prunes on raw depth with quadratic terms; SF dev uses `lmrDepth = newDepth - r/1024` | **Follow the donor of record (B.2)**: raw depth, quadratic. The A.6 audit's "one prospective depth" premise is not in Reckless and measured zero here. |
| LMR eligibility | depth ≥ 3, index ≥ 2, quiet or bad capture, not promo, not in check, not a checking move; reduction **may reach the child's whole depth** (46.7% do) | `depth >= 2 && move_count >= 2`, every move class; `reduced_depth = clamp(new_depth - r/1024, 1, new_depth + 2) + 2·PV` — **floor of one ply**, negative reductions allowed; separate "full-depth" branch reduces by 1–2 plies from its own `r` thresholds | **ADOPT (B.2)** including the floor and the extension-by-reduction. The floor is the single most important shape change in the cluster (section 1, item 4a). `LmrMinReducedDepth` measured −12.7 ± 26.5 alone (below resolution) and is not evidence against it. |
| LMR terms | table `646/1024 + ln·ln/(2335/1024)`; `tt_pv` −887, quiet +1024, improving −1024, exact +109, TT move present & index ≥ 4 +656, cut node +780, bad capture +1024, good history −1024, `-quiet_hist·1024/8395`, `-corr·27/128`, root −1536 | `269·ilog2(depth)`; `-(425·improvement/128).clamp(-241, 1155)`; `-3417·\|corr\|/1024`; exact +1412; TT score ≤ alpha +464; TT shallower +326; `is_win(beta)` +1024; quiet +2432 − 179·history/1024 + `418·clamp(alpha - est, -65, 91)/128`, noisy +1687 − 130·history/1024; `-128·min(ply - last_critical_ply, 8)`; PV `-519 - 437·(beta-alpha)/root_delta`, else `-96 + 32·laterality`; tt_pv −333 −611·(tt > alpha) −685·(tt_depth ≥ depth), else cut node +1852 +2204·no TT move; child in check (the move gives check) −955; `cutoff_count[ply+1] > 2` +1151 (+400 all-node); singular margin `clamp(496·(margin - 185)/128, 0, 2021)`; parent reduction > this + 414 → +136; jitter `(nodes + id·27) % 128 - 59` | **ADOPT (B.2)** the formula as a whole; the singular-margin term is populated by B.3 (reads 0 until then). Root relief (RAR-S70, +2.33 ± 1.85) is **subsumed**: the donor's PV term (`-519 - 437·window/root_delta`) plus the Root node type give the root its own reduction; B.5 confirms with the root cluster. The 1T jitter is the donor's `nodes % 128` term (the 4.10 B1 lead, landing inside the cluster rather than as RAR-S67). |
| Re-search | `reduction > 0 && score > alpha` → full depth null window; PVS full window if inside | `score > alpha` → `new_depth += (score > best + 57) - (score < best + 9)`; null-window at `new_depth` if deeper than the probe; post-LMR continuation bonus | **ADOPT (B.2)**. Rarog's own rejected do-deeper (Phase 2.8) was a standalone constant-fit; this is the donor's shape inside the cluster. |
| Laterality | absent | `stack[ply].laterality = parent + max(ilog2(move_count) - 1, 0)`, consumed by the non-PV LMR term | **ADOPT (B.2)** (part of the formula). |
| `last_critical_ply` | absent | the ply of the last non-reduced/first-move search, consumed by LMR | **ADOPT (B.2)**. |

### 3.8 Quiescence — B.4, recorded here for the boundary

| Mechanism | Rarog | Reckless | Verdict |
|---|---|---|---|
| PV concept | none (4.6 audit lead 2) | `NODE::PV` typed; TT cutoff and refinement refused for decisive scores at PV | **ADOPT (B.4)**. |
| TT | probe, cutoff at any depth ≥ 0, `refine_eval_bound_only` stand-pat, stores stand pat / move / tail | probe, cutoff, best_score refined by TT bound, stand pat `lerp(best, beta, 0.8256)` stored only on a miss, tail write on every exit | **ADOPT (B.4)**. |
| Moves | captures (all legal when in check), delta prune `> 8 pieces && stand_pat + Q + 200 < alpha`, SEE margin `alpha - stand_pat - 265` clamped, bad floor −55, tactical count > 6 prune; `MAX_QPLY 16` | noisy moves only (all when in check), **LMP at 3 moves** unless direct check, SEE threshold `(alpha - eval)/8 - min(\|corr\|, 68) - 74 - history/48`, no ply cap; fail-high `lerp(best, beta, 0.5072)`; noisy-history bonus 100 on a fail-high | **ADOPT (B.4)** subject to B.2's canaries (PLAN's Manta dependency). Neither Reckless nor Rarog generates quiet checks at the first quiescence ply; the classical oracle does, and it is the engine that answers WAC.001 from depth 9 while both Rarog and Reckless miss it through 10. First-ply checks are therefore a **B.4 candidate switch measured against the canaries**, not a default. |

### 3.9 Root, aspiration, time — B.5 and D.1

| Mechanism | Rarog | Reckless | Verdict |
|---|---|---|---|
| Aspiration | from depth 4, `delta = 21`, grow ×1.5 + 5 per fail, `beta = (alpha+beta)/2` on fail-low, open after 20 fails or a mate score | `delta = 23 - min(eval_stability, pv_stability, 7) + avg²/26394`, fail-low re-centres on the score, fail-high raises beta and **reduces the iteration's depth by 1** (`reduction += 1`, capped at 1 for decisive scores), growth 26/128 (low) and 60/128 (high) | **ADOPT (B.5)**. RAR-S17's re-centring loss (−4.52) was standalone against fitted constants; the donor's form lands with the root cluster and the refit. |
| Root move records | `RootMove` with score/depth/nodes/PV, pool scores | `RootMove` with score, previous, display, bounds, seldepth, nodes, PV, tb rank; `best_stats` shared word | **ADOPT (B.5)**. |
| Optimism | none | `113·best_avg/(\|best_avg\| + 201)`, applied in the corrected eval | **B.5** (producer) with B.2's consumer slot. |
| Forgotten-mate / aborted-loss guards | none | present | **ADOPT (B.5)**. |
| Time management | SF-2.2-shaped optimum/maximum with fall/instability/effort multipliers; instability slots and root-confidence consumers dead | soft/hard bounds from clock and increment with a fullmove curve; multiplier from node fraction, score trend, PV and eval stability, best-move changes; hard check every 2048 nodes; 65% soft-stop vote | **D.1** owns it; B.5 may land the multiplier producers if they come with aspiration. |
| SMP | root rotation, weighted vote merge, pool root scores, private histories | no rotation, no vote (thread 0 reports), shared TT and NUMA-replicated shared correction history, `best_stats` word | **D.2** (premise contradicted by RAR-M46; not touched in B). B.1 deletes only the dead instability slots and iteration skipping. |

## 4. Interactions and shared signals

The signals that at least two B clusters produce or consume; every handoff
names them.

| Signal | Producer | Consumers |
|---|---|---|
| `estimated_score` (TT-refined corrected eval) | B.2 (TT + correction) | razoring, RFP, NMP margin and reduction (B.3), LDSE (B.3), LMR `alpha - est` term, ProbCut entry (B.3), quiescence stand-pat (B.4) |
| `improvement` / `improving` | B.2 | RFP, NMP (B.3), ProbCut beta (B.3), LMP, LMR |
| `correction_value` magnitude | B.2 | RFP, quiet futility, LMR, singular margins (B.3), quiescence SEE (B.4) |
| `tt_pv` | B.2 | RFP, NMP margin (B.3), singular margins (B.3), LMR, replacement, hindsight |
| `cutoff_count[ply+1]` | B.2 | LMR; NMP margin (B.3) |
| `stack[ply].reduction` | B.2 (LMR) | hindsight reductions (B.2), LMR parent term |
| `stack[ply].move_count`, `laterality`, `last_critical_ply` | B.2 | LMR, fail-low history bonus, TT-cutoff bonus |
| quiet/noisy/pawn/continuation history | B.2 | picker order, LMP, futility, history pruning, SEE thresholds, LMR; quiescence SEE (B.4) |
| `singular_score`, `tt_move_score` | B.3 | LMR margin term (slot in B.2, 0 until B.3) |
| `all_threats` bitboard | board (B.2 adds the producer) | quiet/noisy history indexing, picker scoring, RFP threat term |
| `optimism` | B.5 | corrected eval (slot in B.2, 0 until B.5) |

Feedback loops to name in every registration: ordering evidence prunes
(history feeds LMP, futility, history pruning and SEE thresholds, so a
history change is a pruning change); evaluation error widens margins
(correction magnitude); TT admission masks candidates (the TT-cutoff history
bonus and the node-type-conditioned cutoff both change which nodes are ever
searched); the re-search rule trains history (post-LMR bonus) which orders
the next iteration. The B.2.2 diagnostics must report per-mechanism
activation *and* the cutoff composition, because RAR-S59 showed a
"hygiene" history change can be a disguised selectivity increase.

## 5. Cluster contents, confirmed and changed

**B.1 (behaviour-neutral restructure)** — as A.6's table, plus the decisions
in section 6. Exact fingerprint 7,601,220 / EBF 2.474.

**B.2 (selectivity core)** — confirmed as PLAN lists, with these changes:
- adds the eval-difference quiet-history training, the TT-cutoff history
  bonus, the fail-low parent bonus and the post-LMR continuation bonus (all
  history *update policy*, which PLAN's list called "update rules");
- adds the LMR depth floor and the negative-reduction allowance;
- adds `laterality`, `last_critical_ply`, `cutoff_count` and
  `stack[ply].reduction` as stack producers;
- **drops** killers, countermove and low-ply history;
- keeps Rarog's IIR unchanged and lands hindsight beside it;
- keeps Rarog's NMP, ProbCut (4.7c) and singular forms unchanged, but wires
  them to the new signals where the old ones disappear (`eval_for_pruning`
  becomes `estimated_score`, `improving` becomes `improvement > 0`, the
  joint `tt_pv` veto becomes per-consumer flags at their current values);
- lands the continuation-correction tables as shadow producers first.

**B.3 (proof searches and extensions)** — NMP with the entry margin, adaptive
reduction, bound shortcut and `nmp_min_ply` verification region; ProbCut in
the donor's depth/return shape keeping the 4.7c filter, with the TT-served
shortcut as a switch; singular with double/triple margins, multi-cut lerp,
negative −3, LDSE, the TT-move-clearing rule and the LMR margin term; the
IIR-versus-hindsight decision; the `cutoff_count` NMP consumer. Bracket
`[0,5]` as registered.

**B.4 (quiescence)** — as PLAN, with first-ply quiet checks as a measured
switch (section 3.8). **B.5** — as PLAN, with root LMR relief retired in
favour of the donor's PV/Root reduction terms and confirmed there.

## 6. The four questions A.6 left to B.0, and the board producer B.2 needs

### 6.1 `evidence.rs`: delete in B.1

Consumers of the producer class that change a search decision at the shipped
defaults: **none**. `allows_singular(.., reject_speculative)` reads the
speculative bit only when `singular_reject_speculative = 1` (dead, A.2.3);
`probcut_store_actual_score` is dead; every other use is the debug store
contract, the `store_kind_*` census and `MoveClass` for diagnostics. Both
donors store ProbCut results at `probcut_depth + 1` and let singular
verification read them (Reckless `tt_depth >= depth - 3 && bound != Upper`),
which is exactly the coincidence RAR-S22 built the taxonomy to refuse; the
donors' strength says the refusal is not load-bearing. Manta's lesson agrees:
provenance may govern *storage* of speculative results, never consumption of
ordinary bounds. B.1 removes `src/evidence.rs`, `tests/tt_provenance.rs`,
the `SPECULATIVE_BIT` (B.1 leaves the freed bit unused so the 4-bit age
arithmetic and the fingerprint are untouched; widening the age to 5 bits is
B.2's, with the replacement rule), the `kind` field
of `TtStore`, `debug_assert_outcome`, and the `store_kind_*`,
`tt_move_inherited*`, `tt_horizon_*`, `store_committed_*`,
`store_skipped_depth_rule` counters. `NodeEvidence`'s one-decode-per-node
discipline is kept as a plain `TtProbe` struct in `tt.rs` without a producer
field; `MoveEvidence` becomes the picker's stage enum.

### 6.2 `NodeType`

`trait NodeType { const PV: bool; const ROOT: bool; }` with `Root`, `PV`,
`NonPV`, monomorphising `search::<NODE>` and `qsearch::<NODE>`; `cut_node:
bool` stays a runtime argument (`all_node = !PV && !cut_node`). This is the
form of both donors and it is what lets B.4 give quiescence a PV concept
without a new parameter.

### 6.3 `Searcher` split

`search/thread.rs::ThreadData` (board, stack, per-thread histories and
correction tables, root moves, PV table, counters, `nmp_min_ply`, optimism,
jitter state) and `search/shared.rs::SharedContext` (TT, stop state, node and
TB counters, root scores, stop votes, `best_stats`). B.1 moves today's fields
into these two types **without changing ownership**: correction tables stay
per thread. Sharing them (Reckless replicates per NUMA node) is a D.2
decision on the accepted B head. `Stack` with a sentinel entry and signed
indexing from −8 (Reckless `PlyArray`) replaces the four `[T; MAX_PLY]`
arrays and the `ply >= 2` guards.

### 6.4 Counter re-keying map

Owner test (PLAN B.8): a counter survives only if `phase4_differential.py`
reads it by name, or a B-phase registration names it. The COMPARABLE core
(52 names) and the EXCLUDED-but-reported names in that tool **keep their
names** — they are the cross-engine contract of `phase4_counter_spec.md`, and
renaming them breaks the join with the frozen `hybrid-diag` oracle build.
Rarog-only families move as follows:

| Family (today) | Disposition |
|---|---|
| `rootconf_*` (30), `root_gap_sum`, `root_deviation_sum`, `root_effort_ppm_sum`, `shadow_4_*` (6), `contradict_*` (15), `refine_*` (10), `agree_*` (2), `tt_pv_veto_*` (4), `corr_applied_to_replaced_eval`, `store_kind_*` (7), `tt_move_inherited*` (2), `tt_horizon_overwrote_searched`, `store_committed_*` (3), `store_skipped_depth_rule`, `tt_store_same_key`, `tt_store_fresh`, `iir_extension_debt`, `singular_probcut_depth_match`, `singular_speculative_seed_blocked`, `nmp_eval_*` (3), `nmp_nested_attempt`, `nmp_decisive_population`, `check_order_*` (2), `lmr_floor_clamped`, `lmr_root_*` (2), `best_move_rank_*` (4, duplicates `best_rank_*`), `best_stage_*` (4), `best_was_reduced`, `thread_depth_*` | **delete in B.1** with their mechanisms, or in B.8 if a B.2–B.5 registration has not named them by then. |
| `lmr_qs_clamped`, `lmr_zero_reduction`, `nmp_cut_unproven_mate`, `nmp_verify_*`, `probcut_qpass`, `probcut_tt_store`, `skip_quiets_nodes`, `quiet_see_prune`, `correction_resid_*` (10), `correction_sample_*` (2), `correction_slot_*` (4), `correction_on_capture`, `tt_eval_refined`, `tt_eval_delta_sum`, `tt_reject_*` (4), `tt_sample_*` (2), `sampled_*` (2), `q_check_*` (3), `q_*_store` (4), `prospective_depth_sum` | **keep through B.2.2**, re-keyed by module prefix where the mechanism moves (`lmr_*` → `node_lmr_*`, `correction_*` → `corr_*`, `q_*` unchanged, `tt_*` unchanged); B.2.2's registration names the ones it reads and B.8 deletes the rest. |
| `lazy_*` (21) | C.1 owns (A.6 item 4). |
| `eg_*` (21), `board_*` (24) | evaluation and board instruments; untouched by B. |
| `worker_*` (3) | D.2. |

New counters B.2.1 must add for B.2.2, with their denominators: `rfp_cut` by
depth bucket (1–3, 4–7, 8+); `lmr_floor_hits` (reduction clamped at one ply)
and `lmr_extended` (negative reduction); `lmr_research_deeper` /
`lmr_research_shallower`; `hindsight_up` / `hindsight_down`; `skip_quiets_nodes`
(kept); `corr_cont2_admitted` / `corr_cont4_admitted` (the shadow-producer
admission profile); `tt_cutoff_quiet_bonus`; `history_pruned`,
`bad_noisy_futility`, `quiet_see_prune` (per move, with `prune_shadow_moves`
as denominator).

### 6.5 The board producer: `all_threats`

Every threat-indexed history, the picker's escape/threatened/offense terms,
the RFP "own pieces unthreatened" term and `is_direct_check` need per-position
attack sets of the side not to move (pawn, knight, bishop, rook, queen, king
attack unions with the friendly king removed from the occupancy) and the
enemy's checking squares per piece type. Rarog's evaluator already computes
attack maps per evaluation (`attacked_by[2][6]`, C.1's `eval/attacks.rs`),
but the search must have them at every node whether or not the evaluator
runs (TT-hit nodes skip evaluation). Reckless computes them in `make_move`
(`update_threats`) and stores them on the state stack. B.2.1 adds a
`Board::threats()` producer computed lazily once per node from the existing
`attacks.rs` primitives (no incremental state, no board-footprint change;
`Board <= 264` bytes stands) and measures its NPS cost in the B.2.2 pooled run;
moving it into `make_move` is a B.7 option.

## 7. Survivors with local evidence

| Mechanism | Evidence | Disposition |
|---|---|---|
| No in-check extension | RAR-S06 +30.75; RAR-X02 (Basilisk −10.17 the other way) | keep; matches both modern donors |
| ProbCut move filter (4.7c) | RAR-S57/S58 +15.56 ± 10.02 | keep; it *is* the donor's `probcut_beta - eval` threshold |
| Root LMR relief (RAR-S70) | +2.33 ± 1.85 | retired into the donor's Root/PV reduction terms; B.5 confirms |
| Zero-reduction floor (`lmr_zero_reduction`) | +9.13 nElo in 2.3.2 | subsumed by the donor's clamp `[1, new_depth + 2]` |
| Qsearch TT stand-pat refinement (RAR-S02) | +6.5 | kept; the donor has the same rule |
| Stand-pat stores | 4.6(1): suppressing them worsens cutoffs | kept in B.2; B.4 adopts the donor's miss-only store as a measured change |
| Fifty-move mate precedence, draw policies | `draw_policy_2026-09-08.md` | untouched |
| Typed provenance | no consumer at default (6.1) | deleted |
| Killers, countermove, low-ply history | no standalone evidence; absent in the donor | dropped with the cluster |
| `SelectivityCountConsidered`, `SelectivityProspectiveDepth`, `LmrMinReducedDepth`, quiet SEE prune switch, `ImprovingPly4Fallback` | measured flat alone (four RAR-S64-era nulls at ±18–26 Elo resolution) | not evidence against the cluster; each lands inside B.2 in the donor's form |

## 8. Seeds and the scale conversion

### 8.1 Conversion factors

| Constant class | Reckless → Rarog | Oracle (`9587eeeb`) → Rarog | Notes |
|---|---:|---:|---|
| evaluation units (margins, deltas, correction bounds) | **0.457** | **0.485** | section 2.4; phase-dependent 0.32–0.54 |
| SEE / material units | **0.75** | **0.40** (0.81 for pawn-only) | Rarog SEE values 100/320/330/500/900 |
| depth, plies, 1024ths of a ply, move counts | 1 | 1 | |
| history units | 1 (Rarog `HISTORY_MAX` 16384 vs donor 8192–16418) | — | the donor's per-table maxima are adopted with the tables |
| time multipliers, lerp weights, growth ratios | 1 | 1 | |

### 8.2 Seed rule

For every constant B.2.1 lands, the handoff table carries three columns:
Reckless converted, oracle converted (where the mechanism exists there),
Rarog fitted (where the mechanism exists here). The seed is:

1. the **Reckless converted value** when the constant is in plies, counts,
   history units or 1024ths, or when neither of the other columns exists;
2. otherwise the **geometric mean of the oracle-converted and Rarog-fitted
   values** when the Reckless-converted value lies more than 2x outside their
   range — this is the case for razoring, RFP, quiet futility and the NMP
   entry margin, where Reckless's small margins encode NNUE accuracy the HCE
   does not have;
3. otherwise the Reckless converted value.

Column values are recorded in the handoff so B.2.3's SPSA ranges can be set
to cover all three. The rule is a research decision recorded here; it does
not authorise changing a default after B.2.2's screen without recording it.

### 8.3 Principal seeds (evaluation-unit and SEE-unit constants)

| Constant | Reckless | ×0.457 / ×0.75 | Oracle | ×0.485 / ×0.40 | Rarog fitted | Seed (rule) |
|---|---:|---:|---:|---:|---:|---:|
| Razoring base | 237 + 254·d² | 108 + 116·d² | 527 at d = 1 | 256 | 274·d (d ≤ 3) | 260 + 116·d² (2, then shape) |
| RFP per-ply term | 1140·d²/128 + 22·d − 19 | 4.1·d² + 10·d − 9 | 227·(d − improving) | 110·(d − improving) | (52 + 51·not_impr)·d | 6·d² + 50·d (2) |
| RFP correction term | 669·\|corr\|/1024 | 0.30·\|corr\| | — | — | 3·\|corr\|/128 | 0.30·\|corr\| (1) |
| RFP threat term | 54 | 25 | — | — | — | 25 (1) |
| NMP entry margin | 337 − 9·d + 110·tt_pv | 154 − 4·d + 50·tt_pv | 311 − 33·d + 112·ttPv (+ eval ≥ beta) | 151 − 16·d + 54·ttPv | −12·d (below beta) | 150 − 10·d + 50·tt_pv (2) |
| NMP eval-margin reduction | 477·clamp(est − β, 0, 1187)/128 /1024 | clamp to 542 | min((eval − β)/192, 3) | /93, cap 3 | clamp((eval − β)/200, 0, 3) | (est − β)/150 cap 3 (2) |
| ProbCut beta margin | 254 − 85·improving | 116 − 39·improving | 176 − 49·improving | 85 − 24·improving | 180 | 120 − 35·improving (3) |
| Quiet futility | 79·d + 55·hist/1024 + 77·(eval ≥ β) + 555·\|corr\|/1024 − 127 | 36·d + 25·hist/1024 + 35 + 0.25·\|corr\| − 58 | 284 + 188·lmrDepth | 138 + 91·lmrDepth | 211 + 135·d | 150 + 90·d + 25·hist/1024 (2) |
| Bad-noisy futility | 84·d + 82·hist/1024 + 24 | 38·d + 37·hist/1024 + 11 | 267 + 391·lmrDepth + victim | 130 + 190·lmrDepth | — | 60·d + 37·hist/1024 + 40 (2, with the victim value added as in the oracle) |
| LMP count | (2818 + 78·impr/16 + 1351·d² + 74·hist/1024)/1024 | same (counts) | (3 + d²)/(2 − improving) | same | 1 + 2d²/3 (+d) | Reckless (1) |
| History pruning | hist < −948·d | same | contHist < −4136·d (SF dev) | — | −5617·d (composite) | −948·d on the donor composite (1) |
| SEE quiet | −12·d² + 56·d − 27·hist/1024 + 27 | ×0.75 | −(29 − min(l,17))·l² | ×0.40 | none | Reckless ×0.75 (1) |
| SEE noisy | −7·d² − 36·d − 39·hist/1024 + 14 | ×0.75 | −202·d | −81·d | −66·d − cap/8, floor −955 | Reckless ×0.75 (1) |
| Aspiration delta | 23 − stability + avg²/26394 | 11 + avg²/12100 | 19 | 9 | 21 | B.5 |
| Correction update clamp | (148·d·diff/128).clamp(−4678, 2496) | ×0.457 on the bounds | — | — | ±1024 | (−2138, 1141) (1) |
| Correction scale | /64 | /64 | — | — | /16384 with weights | /64 (1) |
| Singular margins | depth (or ⌈d/4⌉ Exact) + depth·(tt_pv && !PV); double 195/48/16, triple 230/56/19/36 | ×0.457 | (formerPv + 4)·d/2 | ×0.485 | 4·d; double 20 | B.3 |
| LDSE | est ≤ α − 25 | −11 | — | — | — | B.3 |

All LMR terms, history bonus/malus formulas, picker weights, table maxima and
lerp weights are seeded at the donor's values unchanged (rule 1).

## 9. SPSA surfaces

**B.2.3** (expected 55 live coordinates, registered before B.2.1 lands the
umbrella; ranges span the three seed columns):

- node margins (11): razoring base and square term; RFP square, linear,
  improvement, correction, threat and constant terms; the RFP lerp weight;
  IIR depth floor (kept mechanism).
- move-loop pruning (16): LMP four coefficients; quiet futility five; bad-noisy
  futility three; history pruning slope; SEE quiet four; SEE noisy four.
- LMR (18): `ilog2` coefficient; improvement scale and clamps; correction scale;
  exact, TT-score, TT-depth, quiet base, quiet history, `alpha − est` scale,
  noisy base and history, critical-ply, PV base and window, non-PV base and
  laterality, tt_pv three, cut-node two, in-check, cutoff-count two, parent
  term; re-search deeper/shallower thresholds.
- histories (6): quiet bonus cap and slope; quiet malus cap, slope and index
  scale; continuation bonus cap; noisy bonus cap; TT-cutoff bonus cap;
  fail-low factor base.
- correction (6): update slope and two clamps; the six table weights
  (pawn, np-white, np-black, cont2, cont4, minor) as five ratios; bucket count
  fixed at 16.
- eval formula (2): material coefficient, rule-50 damping.

Excluded from SPSA (categorical, per RAR-S13's lesson): the LMR floor, the
skip-quiets exemption, the TT-cutoff admission rule, the correction
admission rule, no-ageing. **SPSA is conditional** (PLAN rule 4): B.2.2 runs a
zero-game sweep of five coordinates (RFP linear term, LMP square term, quiet
futility base, LMR quiet base, correction update slope) across 0.5x–2x on the
bench and reports whether the tree and WAC-at-100k surfaces are curved; a
flat or monotone surface on all five sends B.2 straight to B.2.4 at seeds.

**B.3**: NMP entry margin (3), reduction (4), verification depth threshold;
ProbCut margin (2), depth divisor (319), adjusted-beta slope (197), cap;
singular margins (8), LDSE margin; IIR switch is categorical. **B.4**: SEE
threshold (4), stand-pat and fail-high lerp weights, LMP count. **B.5**:
aspiration (5), optimism (2), TM multipliers (10, shared with D.1).

## 10. The differential counter set and AblationMask

The COMPARABLE core keeps its names (6.4) and gains the B.2.1 counters listed
there on the Rarog side only (they stay Rarog-only until the oracle build is
extended, which nobody is asked to do). The oracle differential is re-run
once at B.1 under the deleted-family map (the join must reproduce
`phase4_differential_47c_depth8.txt`'s oracle column, which is deterministic
for the frozen build plus the 2.4.0 DLL) and once at B.2.2.

`AblationMask` stays through B.9 as PLAN records. B.2.1 **re-declares the eight
bits** on the new mechanisms so `G(mask)` remains runnable against the
oracle's fixed bits: 0 razoring, 1 RFP, 2 NMP, 3 ProbCut, 4 IIR/hindsight,
5 move-loop pruning (LMP, futility, history, SEE), 6 singular and extensions,
7 LMR. Bit 6 stays asymmetric (the oracle also drops check extensions).

## 11. B.2.2 screens, canaries and the decision trace — registered numbers

Baselines are the 2.4.0 head numbers in section 2 and are re-measured on the
B.1 head before B.2.1 starts (B.1 must reproduce them exactly except NPS).
Thresholds are frozen here; they are screens, never acceptance (PLAN rule 8).

| Screen | Instrument | Baseline (2.4.0) | Reference | Floor (ablate below) | Target (proceed above) |
|---|---|---:|---:|---:|---:|
| Geometric branching 4–14, phase-4 suite | `branching_profile.ps1`, Hash 64, fresh process | 1.630 | oracle 1.736 | **window [1.55, 1.85]** | inside the window; outside either edge is a defect flag, not a tuning need |
| Nodes at depth 14 relative to the oracle | same run | 1.87x | 1.0 | ≤ 2.2x | ≤ 1.5x (monitored; nominal depth semantics change with the cluster) |
| WAC solved at 100k nodes | `fixed_budget_probe.py nodes 100000` | 200 | 242 | **≥ 205** | **≥ 220** |
| WAC solved at 400k nodes | same | 237 | 267 | ≥ 240 | ≥ 250 |
| Median depth at 300k, phase-4 suite | `fixed_budget_probe.py nodes 300000` | 16 | 19 | monitored | monitored; must not rise while WAC-at-nodes falls |
| Oracle best-move agreement at 300k | same run | 40/50 | — | ≥ 38 | ≥ 43 |
| Pooled-PGO NPS, RAR-M41 protocol | `nps_build_pool.ps1` + `nps_multibuild.ps1` | 3.19 MNPS | — | **≥ 0.90x** | ≥ 0.95x |
| 2,000-game paired run vs the B.1 head, seeds unfitted | `sprt.ps1 -Mode fixed -Games 2000 -NoAdjudication` | 0 | — | **> −40 Elo** (below is a defect) | ≥ +10 Elo proceeds to B.2.3 directly; −40..+10 the review decides after the component ablation |
| Canaries | section below | — | oracle anchors | all pass | all pass |

**Canaries, anchored to the oracle.** From `wac_anchor_d10.json`: every WAC
position whose oracle stable depth is ≤ 6 must be solved, stable, by the
full B.2 arm at ≤ oracle depth + 2 **and** at 100,000 nodes. That is 116
positions today; Rarog fails 16 of them outright and is three or more plies
late on 33 more. WAC.001 `g3g6` is required at ≤ depth 11 (oracle 9). The
quiet-mate-threat cohort PLAN's Manta note requires is the subset whose key
move is a non-capturing, non-checking quiet move; 47 of the anchored
positions qualify (WAC.003, 006, 008, 016, 017, 018, 019, 024, 026, 029, 034,
038, 039, 042, 048, 053, 059, 069, 072, 077, 085, 107, 109, 110, 115, 117,
133, 140, 142, 144, 150, 162, 175, 182, 191, 195, 201, 203, 206, 224, 231,
233, 235, 249, 259, 274, 294). A changed canary is recorded with its cause, never
re-blessed; a canary that the oracle itself fails at 100k nodes is excluded
by that fact, not by editing.

**Decision trace** (B.2.1 ticket 0, `diag`-only): under `searchmoves`, at plies
one and two, print every prune, reduction, extension and proof decision with
its inputs (`estimated_score`, `improvement`, correction, history composite,
`move_count`, `reduction`, `cutoff_count`, the window). Manta's two seed
defects were found by exactly this and are invisible to counters.

## 12. Frozen predictions for B.2 (before any exposure)

Written before B.1 exists. Confidence is stated; the calibration is appended
at B.2.4, never rewritten.

| # | Prediction | Confidence | Falsifier |
|---|---|---|---|
| P1 | The unfitted paired run (2,000 games, converted seeds) measures **−10 ± 30 Elo**; probability of the −40 defect stop 15% | moderate | a result below −40 means a defect or a wrong seed column, not a tuning need |
| P2 | After B.2.3 and the `[0,10]` gate, B.2 measures **+35 Elo at STC**, 90% interval **[+5, +70]**; H1 probability 65% | moderate | H0 or a stop below +10 refutes "coherent core recovers a large share"; the residual then lies in B.4 (quiescence shape) and the evaluation, not in more selectivity |
| P3 | Branching factor moves **up** into [1.66, 1.78] (fewer reductions into quiescence, quiets skipped by count rather than pruned one by one) while WAC at 100k rises to **≥ 215** | moderate | branching falling below 1.60 with WAC flat says the seeds are over-selective on this evaluation and the seed rule 8.2 was applied wrongly |
| P4 | `lmr_research / lmr_applied` rises from 1.3% to **3–5%** and `rfp_cut` falls below **35%** of interior nodes | high | either not moving means the floor or the estimated-score wiring did not land |
| P5 | NPS lands at **0.93–0.97x** of the B.1 head (threat sets per node, larger histories) | moderate | below 0.90 sends the `threats()` producer to B.7 before the gate |
| P6 | The continuation-correction shadow admission profile is concentrated at low remaining depth, as Manta measured; enabling the consumer changes bench by **< 3%** | moderate | a large tree change from a correction consumer means it is acting as a pruning signal and must be re-admitted |

Stop rule for the programme: PLAN rule 6 (two rejected clusters stop B). B.0's
own stop rule: if P2 fails *and* P4 held, the coherent-core hypothesis is
refuted for this evaluation and B.3–B.5 are re-scoped to the smallest
donor-shaped changes with measured activation; if P2 fails *and* P4 failed,
B.2 returns to `IMPLEMENTED` for a defect hunt, not to `RESEARCH`.

## 13. Handoffs

### 13.1 B.1 — READY_FOR_IMPLEMENTATION (`I1`)

Scope: A.6's move table plus sections 6.1–6.4: `NodeType`, `ThreadData` /
`SharedContext`, `Stack` with sentinel, deletion of `evidence.rs` and the
dead families, the 42 inert parameters, the root-confidence subsystem, SMP
iteration skipping, the two test-only index helpers; counters re-keyed per
6.4; `tm_*` helpers to `search/time.rs`. Done criteria as PLAN: exact
7,601,220 / EBF 2.474 on magic and PEXT, debug and release suites, fmt,
clippy, pooled-PGO NPS within ±0.5% of RAR-M48. Tooling commit: regenerate
or delete `tools/spsa_configs`, re-run the oracle differential once under the
new names, record RAR-S65–S69 superseded, re-measure the section 11 baselines
on the B.1 binary (they must match section 2 exactly except NPS).

### 13.2 B.2 — READY_FOR_IMPLEMENTATION (`I2`), waits for B.1

Semantics frozen by sections 3.2–3.4, 3.6–3.7 and 5; seeds by section 8;
invariants: legal PV and best move; terminal and draw precedence; mate
distance semantics; the umbrella-off arm reproduces the B.1 fingerprint at
every ticket; nothing reduced or omitted at the root, in check, or before one
move is searched; the reduced depth is at least one ply; direct quiet checks
survive the count skip; correction never trains from in-check, capture-best
or bound-disagreeing nodes; the TT stores the raw static eval only. Ticket
order: 0 decision trace; 1 board `threats()` and the stack producers; 2 TT
miss-store, node-typed cutoff, replacement refusal, per-consumer `tt_pv`; 3
histories and picker (shadow continuation-correction producers here); 4
correction consumer and corrected-eval formula; 5 node-level razoring, RFP,
hindsight, cutoff count; 6 move-loop pruning; 7 LMR and re-search; 8 history
update policy. Each ticket keeps the umbrella-off fingerprint; B.2.2's
screens run on the full arm; the reviewer's acceptance is recorded before
B.2.2. Register B.2.3 and B.2.4 in `EXPERIMENTS.md` before the first game
(P1–P6 copied verbatim).

### 13.3 B.3 — READY_FOR_IMPLEMENTATION (`I2`), contingent on the accepted B.2 head

Semantics frozen by sections 3.4 (NMP, ProbCut, IIR) and 3.5; seeds by 8.3
and 9; the `[0,5]` bracket as registered. Before implementing, re-read
B.2.2's diagnostics for `nmp_cut/nmp_attempt`, `probcut_*` and
`singular_*`; the mechanism decisions stand, the activation numbers are
re-based.

## 14. The four AGENTS questions, answered for the programme

**Mechanism.** Rarog's search reaches nearly the oracle's depth at equal time
(0.97 ply) and far less at equal nodes (three plies median), spends a third
of its nodes in quiescence against the oracle's fifth, reduces 47% of its
late moves straight into quiescence, verifies 1.3% of reductions, converts
24% of null-move attempts, and orders better than the oracle by first-move
cutoff. The active defect is not *how much* it prunes but *what it prunes on
and what it does with a reduced result*: shape, not constants, which is why
five constant candidates measured zero and the one shape change (4.7c)
paid. **Interactions.** Section 4; the feedback loops named there are why
the core is one cluster. **Invariants.** Sections 11 and 13.2; tests per PLAN
B.2.1. **Falsifier.** P1–P6, the −40 stop rule, and the two-cluster
programme stop.

## 15. Not done, and why

- No oracle differential was re-run: the `hybrid-diag` binary is not on
  disk, and the archived 47c reading with the counters in 2.5 answers B.0's
  questions; B.1 re-runs it under the new names.
- No G(mask) ablation was re-run; the mask bits are re-declared in B.2.1.
- The scale ratio used Reckless's `eval` command output, which is the raw
  network; the search-facing column applies the formula from
  `evaluation.rs` by hand with zero optimism and correction. Both are
  recorded.
- Stockfish dev was read for the pruning, singular, LMR, history and TM
  regions only; its NNUE-side mechanisms are F.0's.
- Nothing here is Elo. RAR-M50 records the artifacts and hashes.
