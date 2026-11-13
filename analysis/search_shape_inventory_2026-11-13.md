# The kernel against its donor: inventory, seeds, parity and the reads (D.7.0), 2026-11-13

Kind of work: research (`R3`). No engine code changed.
Verdict: `READY_FOR_IMPLEMENTATION` for D.7.1.1 (the Colosseum wrapper) and
D.7.2 (RAR-O09's preparation and run). D.7.3 and D.7.4 are specified here and
open with D.7.2's decision, as PLAN's rule says.

Evidence layers: source reading (the inventory), one registered game result
(RAR-O08) and descriptive reads that are not Elo (speed, clock use, nodes to
depth).

## 1. Result

1. **The kernel departs from Reckless `0756957` in about a hundred places**
   (section 3 lists 104), not twelve. 69 are in the node body, quiescence,
   the wrapper and the root window. The rest are in the picker, the
   histories, the correction, SEE, the table and the limits. Eight rest on
   an SPRT of their own: two kept Rarog's form (ProbCut's capture filter,
   the line bound) and six rejected an alternative on the old kernel. About
   thirty-five rode inside a cluster's gate (B.2, B.3, B.4). The rest have a
   fixed read, a screen or nothing.
2. **Found here and not in the decision analysis' list:** the donor's store
   on every alpha raise and its forced store at PV nodes; ProbCut's
   population (cut nodes only, any depth, in check too); four null-move
   details; the donor's draw score of −2 to +2; the negative extension that
   may drop the TT move into quiescence; no TT move in the donor's
   quiescence; the picker's promotion pushes, skip survivors and root form;
   the correction's minor table and weights; SEE's values and legality; the
   table's key and depth conventions.
3. **Two earlier statements are corrected.** The donor does cut on a table
   hit at a PV quiescence node unless the score is decisive; "no cutoff at
   PV nodes" (Basilisk check, section 1) is Stockfish's form. The queen push
   in quiescence (D.6.2) is the donor's form, not one of Rarog's own.
4. **Units: the donor's units are a scaffold, not the kernel's final form**
   (section 5; amended the same day by the maintainer's decision). The
   kernel is first built with Rarog's evaluation multiplied by 2536/1024
   and the donor's constants unchanged, so parity can be read. Then the
   constants are folded into Rarog's units once, by their dimensions,
   checked against the scaffold in the same binary, and the multiplication
   is deleted. The fit and everything after it are in Rarog's units. Phase
   B converted blind at ×0.457, and its seeds needed +138.6 Elo of fitting.
5. **Parity is two layers** (section 7): decision records from a tracing twin
   of the hybrid, which each rule function must reproduce exactly, and tree
   statistics on 500 positions against a measured noise floor. Identical
   trees are not attainable: both engines hash their keys into the history,
   correction and transposition tables differently.
6. **RAR-O09 is registered with four matches**, not three (section 6). The
   two engines count nodes in different units, so "equal nodes" gets a
   measured correction; the donor stops 15 ms before its move time, so
   "equal time" gets unequal commands; and the clock match at `3+0.03` is
   replayed, because the decomposition needs one runner and one pair of
   binaries.
7. **My prediction for the deciding match is zero**, 80% band [−20, +20],
   and 0.20 that PLAN's rule fires. RAR-O08's own tree-shape read puts the
   donor's kernel at about 1.1 to 1.2 times the head's positions a second
   once the bridge is returned, and it spent 8% more of its clock. Those
   two can carry the whole 22.5. I recommend running the reads before any
   kernel code, as planned, and not opening D.7.3 on the second aim alone
   before they are in.
8. **Colosseum CLI 0.5.0 can play all four matches.** What is missing is on
   Rarog's side (D.7.1.1). Three small requests and one raised priority went
   to `analysis/colosseum_requests_2026-11-13.md` (items 2, 11, 12, 13).

## 2. Sources and method

- Rarog: engine source `112eb14` (the tree at `6bab3ff`), bench 8,935,848.
  Reckless: `0756957`, clean. The hybrid: that commit with
  `tools/oracle-hybrids/reckless-0756957-rarog-hce.patch`.
- Read in full by me: Reckless `src/search.rs`, `evaluation.rs`, `time.rs`,
  `types/score.rs`; Rarog `src/search/node.rs` lines 1 to 3023 (the non-test
  half), the root driver and aspiration in `mod.rs`, `params.rs`, the
  wrapper in `correction.rs`, the probe helpers in `tt.rs`, the hybrid patch.
- Read by three Opus agents with line citations, then spot-checked by me:
  the picker, histories, correction tables and SEE; the transposition table,
  score bands, node counting and limits; Colosseum CLI at the pin and
  Rarog's wrapper. Checked again by me: the donor's move predicates and
  piece values, its draw rules, both node counters, both move-time paths,
  and the three Colosseum findings in the pinned source and in run files.
- Line numbers are at these revisions. `node.rs` is Rarog's
  `src/search/node.rs`; `search.rs` is the donor's `src/search.rs`.
- Not checked: generation order position by position (MP-13), the equality
  of the two engines' setwise attack masks, and every path that differs only
  at more than one thread.

## 3. Inventory

Units: `eu` evaluation units, `su` SEE units, `hu` history units, `ply`,
`n` a count. Gate: **S** a registered SPRT of its own, **C** inside a
cluster's SPRT, **F** a fixed read, **R** a repair read, **–** none.
New kernel: **D** the donor's form, **K** kept as Rarog's (carried over or an
invariant PLAN names), **L** Rarog's own form, dropped for parity and asked
again in D.7.8.

### 3.1 Node entry and draws

| Id | Mechanism | Rarog | Reckless | Unit | Record, gate | New kernel |
|---|---|---|---|---|---|---|
| N1 | Order at entry | stop, ply cap, draw, mate distance, line bound, then quiescence (`node.rs:587-626`) | stop, quiescence at depth 0 first, then upcoming repetition, clock, draw, ply cap, mate distance (`search.rs:316-358`) | – | none, – | D |
| N2 | Ply cap | the corrected evaluation, before the draw test, in check too (`node.rs:590-592`, `:2560-2562`) | draw test first; a draw in check, else the raw evaluation (`search.rs:343-349`, `:1235-1241`) | – | D.6.7's screen (RAR-S121), – | D; `MAX_PLY` stays 128 (K) |
| N3 | Upcoming repetition | none | alpha raised to the draw score when a repetition is one move away, in both searches (`search.rs:325-331`, `:1214-1220`; `board.rs:298`) | – | B.0's map deferred it; never read in Rarog (Basilisk: the test off 0.0 ± 9.3), – | D; needs cuckoo tables in the board |
| N4 | Draw score | 0 (`node.rs:604`, `:2572`, `:2454`) | `nodes % 5 − 2` (`types/score.rs:20-22`) | eu | RAR-S91, `CoreDrawJitter=1` −1.2 ± 9.4, F | D |
| N5 | Draw rule | one earlier occurrence anywhere; Rarog's material rule (`board.rs:1234-1246`) | once after the root or twice before it (`board.rs:271-273`); its own material rule (`:242-267`) | – | RAR-S18: a four-part bundle that included the root-aware rule lost −7.21 ± 6.03, and −11.91 ± 7.67 without that part (two SPRTs `[-3,3]`); the comment at `board.rs:1243` credits the first figure to the root-aware rule alone | D, as a second predicate that takes the ply; the old one stays for the old kernel |
| N6 | Line bound, extension budget | both (`node.rs:300-338`, `:617-619`, `:924-926`, `:1585`, `:1636`, `:2075`) | neither | ply | the budget: B.3's cluster (RAR-S84), C; the line bound: D.6.7, RAR-S122, nElo −1.0 ± 6.2 at its cap, kept, S | L (PLAN: they stay only if the chains return) |
| N7 | `seldepth` | every node, `ply + 1` (`node.rs:600`, `:2568`) | PV nodes, `ply` (`search.rs:333-335`, `:1225-1228`) | ply | none, – | K (reporting) |
| N8 | Stop and clock polling | in `count_node`, every counted node (`mod.rs:1887-1948`) | at node entry, thread 0 (`search.rs:337-340`, `:1230-1233`) | – | D.1, D.3 | K |

### 3.2 Table use in the node

| Id | Mechanism | Rarog | Reckless | Unit | Record, gate | New kernel |
|---|---|---|---|---|---|---|
| T1 | Tablebase probe | before the table probe; store flagged by node type (`node.rs:645-667`) | after the table cutoff; store flagged `tt_pv` (`search.rs:411-445`) | – | B.5 cluster, RAR-S94, C | K (tablebases) |
| T2 | TT move legality | checked at the probe; an illegal move becomes none (`node.rs:751-754`, `:2633-2636`) | checked in the picker; the cutoff bonus trusts the raw move (`search.rs:396`) | – | H-7; none, – | K |
| T3 | IIR | depth −1 from depth 4 without a TT move, or off the PV with an entry more than 3 plies shallower (`node.rs:766-790`) | none | ply | RAR-S96 (two categorical reads); D.6.27 | D (none) |
| T4 | Evaluation in an exclusion search | recomputed from the stored raw value (`node.rs:794-810`) | the node's own stack value, raw none (`search.rs:456-458`) | eu | none, – | D |
| T5 | Estimate in an exclusion search | refined by the stored bound (`node.rs:854-858`) | the plain evaluation (`search.rs:471-482`) | eu | D.6.5, RAR-S118, RAR-S119: `NO_CHANGE` | D |
| T6 | Store on an alpha raise | none | a lower bound with the PV flag set, when the move is not the TT move (`search.rs:1069-1071`) | – | RAR-S73: "not adopted, recorded", – | D |
| T7 | Forced store at a PV node | none; a store can be refused by a deeper entry (`tt.rs:902-904`) | the final store at a PV node always overwrites (`search.rs:1191`, `transposition.rs:264`) | – | NODE-7 / TT-10 in PLAN's review list, – | D; needs one flag on the store |
| T8 | Fail-low store | the best-scoring move (`node.rs:2337-2340`, `:2517-2526`) | no move (`search.rs:1053-1055`, `:1191`) | – | D.6.4, RAR-S115 H0 (−14.7 ± 13.8), RAR-S117: `NO_CHANGE` | D |
| T9 | ProbCut store | the score less the margin it cleared (`node.rs:1493-1509`) | the raw score (`search.rs:697`) | eu | RAR-S34: the raw score +5.55% time to depth; not promoted, – | D |
| T10 | Root TT move under MultiPV | the line's own candidate stands in (`node.rs:755-764`) | none | – | D.3 | K (root driver) |

### 3.3 Static evaluation wrapper

| Id | Mechanism | Rarog | Reckless | Unit | Record, gate | New kernel |
|---|---|---|---|---|---|---|
| W1 | Material scaling | `eval·(1 + 15·(material − 8000)/(64·8000))`, 1.000 at the start, 0.766 with none (`correction.rs:185-189`) | `(raw·(21032 + material) + optimism·(1548 + material))/27015`, 1.160 at the start, 0.779 with none (`evaluation.rs:4-6`) | eu | B.2 cluster (RAR-S73); 15 from C.10 (RAR-S105), C | D |
| W2 | Optimism | none | `113·avg/(|avg| + 201)` for the side to move (`search.rs:120-124`) | eu | RAR-S92, `CoreOptimism=1` −5.6 ± 9.4, F | D |
| W3 | Rule-50 damping | `1 − min(clock, 100)/199` (`correction.rs:190-193`) | `(200 − clock)/200` (`evaluation.rs:8`) | – | B.2 cluster, C | D |
| W4 | Draw verdicts | 0, no correction (`correction.rs:180-182`) | no exemption | – | D.6.15, R | K (a repair; its domain is outside parity's positions) |
| W5 | Correction blend | six tables, weights 122, 56, 113, 115, 187, 163 in 128ths (`correction.rs:134-152`) | five tables, unweighted, `/64` (`search.rs:1403-1422`) | eu | C-1, C-2; C.10, C | D; the minor table at weight 0 |
| W6 | Correction update | `250·depth·diff/128` in [−1,692, 736] (`correction.rs:236-240`) | `148·depth·diff/128` in [−4,678, 2,496] (`search.rs:1428`) | table | C.10, C | D |
| W7 | Clamp | ±31,743 | ±31,518 | eu | follows the bands | K (Rarog's bands) |

### 3.4 Before the moves: pruning, null move, ProbCut

| Id | Mechanism | Rarog | Reckless | Unit | Record, gate | New kernel |
|---|---|---|---|---|---|---|
| P1 | Razoring's alpha limit | 936 (`node.rs:32`, `:937`) | 2048 (`search.rs:541`) | eu | 2048 × 0.457, B.0's seed rule; B.2 cluster, C | D |
| P2 | Singular candidate | depth ≥ 4, entry at most 2 plies shallower (`node.rs:1072-1075`) | depth ≥ 5 + `tt_pv`, at most 3 (`search.rs:519-523`) | ply | RAR-S80: the depth-4 floor +3.1 ± 9.1, the 2-ply margin +12.9 ± 9.4, F | D |
| NM1 | Null move's population | every non-root node without the PV flag, depth ≥ 3 (`node.rs:1087-1105`) | cut nodes only, any depth, the PV flag costs +110 (`search.rs:567-585`) | – | RAR-S80: the donor's population +3.1 ± 9.3 at `3+0.03`, −10.4 ± 12.0 at `10+0.1`, F | D |
| NM2 | Material guard | the mover has a non-pawn piece (`node.rs:225-238`) | all material on the board above 491 (`search.rs:579`) | eu | a legacy coordinate, never read in games; `tests/zugzwang.rs`, – | D; the zugzwang tests are re-read in D.7.8 |
| NM3 | Null after null | barred by `allow_null` (`node.rs:1088`, `:1148`, `:1228`) | not barred; only the verification region | – | none found, – | D |
| NM4 | Child's type | a cut node (`node.rs:1150`) | not a cut node (`search.rs:606`) | – | D.6.3: `NO_CHANGE` | D |
| NM5 | Cut value | beta for a win score, else `max(score, beta)` (`node.rs:102-104`) | the raw score, which can be below beta; no cut on a win or loss score (`search.rs:614-631`) | eu | B.3's implementation record; B.3 cluster (RAR-S84, +50.5 ± 10.9), C | D |
| NM6 | Verification inside a region | never nested; a fail-high below beta is dropped (`node.rs:1179-1274`) | verified again, and the region cleared (`search.rs:614-623`) | – | the same record, C | D |
| PC1 | ProbCut's population | not in check, not in an exclusion search, no PV flag, depth ≥ 4, any node type, a quiet TT move allowed (`node.rs:1289-1308`) | cut nodes only, any depth, in check and in exclusion searches, the TT move not quiet (`search.rs:638-641`) | – | B.3: the node-type switch kept on a zero-game screen, the depth floor kept; C | D |
| PC2 | Gate without a stored score | the estimate at beta (`node.rs:1302`) | the corrected evaluation at beta (`search.rs:640`) | eu | none, – | D |
| PC3 | Moves | all legal noisy, SEE ≥ max(0, 0.92·(probcut_beta − eval)), at most 5 (`node.rs:1315-1349`) | the picker's good-noisy stage at SEE ≥ probcut_beta − eval, no cap (`search.rs:643-648`) | su, n | RAR-S57, RAR-S58, S; 92 from C.10 | D |
| PC4 | Children's type | cut nodes (`node.rs:1416`, `:1438`) | not cut nodes (`search.rs:669`, `:681`) | – | D.6.3, RAR-S114 H0 (−3.7 ± 6.9): `NO_CHANGE` | D |

Razoring's other terms, reverse futility, the eval-difference history bonus,
hindsight and LDSE have the donor's form (`node.rs:817-825`, `:892-921`,
`:932-979`, `:993-1053`; `search.rs:492-564`, `:754`); only their fitted
values differ.

### 3.5 Singular search

| Id | Mechanism | Rarog | Reckless | Unit | Record, gate | New kernel |
|---|---|---|---|---|---|---|
| SE1 | Margin | 12 sixteenths a ply of span (`node.rs:282-285`) | 1 a ply; a quarter of the depth when exact (`search.rs:714-716`) | eu | fitted in B.3.3 (RAR-S81 to S83); B.3 cluster, C | D |
| SE2 | Double-extension base | −11 (`params.rs:485`) | 0 | eu | fitted, C | D |
| SE3 | Multi-cut | only with the singular beta at or above beta (`node.rs:1597-1612`) | whenever the exclusion search reaches beta (`search.rs:743-744`) | – | B.3 amendment 3: mandatory, from the KBNK test; C | D |
| SE4 | Demotion of the TT move | exclusion score above the stored one and not decisive (`node.rs:1613-1618`) | above the stored one and the exclusion search searched a move (`search.rs:745-746`) | – | B.3: recorded as a departure from the donor's letter; C | D |
| SE5 | Negative extension | floored: the TT move keeps a one-ply child (`node.rs:1624`) | −3, which can send it to quiescence (`search.rs:749-751`, `:857`) | ply | B.3 amendment 4, choice 8; C | D |
| SE6 | Only the excluded move is legal | returns alpha (`node.rs:2446-2449`) | returns −31,518, far below any singular beta (`search.rs:1085-1087`) | eu | B.3 amendment 4: the donor's value "the same defect class"; C | L |
| SE7 | PV first move | – | a TT move with a stored depth above 1 is searched at depth ≥ 1 (`search.rs:997-999`) | ply | – | D |

### 3.6 The move loop

| Id | Mechanism | Rarog | Reckless | Unit | Record, gate | New kernel |
|---|---|---|---|---|---|---|
| ML1 | Bad-noisy futility | adds the victim's value (`node.rs:1901-1905`) | no victim term (`search.rs:822`) | eu | B.0's seed table ("as in the oracle"); B.2 cluster, C | D |
| ML2 | History the rules read | a snapshot from when the quiets were scored (`node.rs:1743-1749`) | read at each move (`search.rs:786-791`) | hu | NODE-12 in the review, – | D |
| LR1 | LMR at the root and in check | never (`node.rs:2126`; test at `:3248`) | both (`search.rs:861`) | – | RAR-S74 (e): −3.8 ± 9.6, F | D |
| LR2 | Re-search depth | deeper only (`node.rs:2250-2253`) | deeper past +57, shallower below +9 (`search.rs:927-930`) | eu | RAR-S102: the shallower branch at 2 +0.5 ± 8.5, at 4 −12.9 ± 8.9, F | D |
| LR3 | Reductions on unreduced searches | none (`node.rs:2092-2122`, `:2278-2294`) | zero to two plies by a second formula, the first move off the PV included (`search.rs:939-993`) | 1024ths | adopted in B.0's map, never built; RAR-S74 (d): +1.7 ± 9.9 on an arm that reached only first moves off the PV, F | D |
| LR4 | Extension spending | tracked per line (`node.rs:2075`) | none | ply | N6 | L |
| ST1 | Node type in quiescence | untyped; stores keep only a stored PV bit (`node.rs:2606-2611`) | typed; a PV node's children are PV (`search.rs:1208`, `:1347`) | – | B.4 cluster (RAR-S88), C | D |

Late-move pruning, quiet futility, history pruning, both SEE allowances, the
LMR formula term for term, the cutoff count, the PV-flag propagation at a
fail low and the correction's admission rule have the donor's form
(`node.rs:1791-2064`, `:453-502`, `:2482-2496`; `search.rs:793-851`,
`:861-936`, `:1183-1200`); only values differ.

### 3.7 Quiescence

| Id | Mechanism | Rarog | Reckless | Unit | Record, gate | New kernel |
|---|---|---|---|---|---|---|
| Q1 | Table cutoff | any stored score, at PV nodes too (`node.rs:2630`) | not a decisive score at a PV node (`search.rs:1256-1265`) | – | B.4 research ("no change"); C | D |
| Q2 | TT move | ordered first (`node.rs:2633-2636`, `:2753-2764`) | the picker gets none (`search.rs:1319`) | – | B.4 research ("keep Rarog's"), C | D |
| Q3 | Stand-pat store | on every fail-high, the stored PV bit only (`node.rs:2676-2691`) | only without an entry; PV flag from the node (`search.rs:1300-1310`) | – | Phase 4.6: suppressing the store cut table cutoffs by 10.3%; B.4 "keep", C | D |
| Q4 | Depth cap | 16 plies of quiescence (`node.rs:2693-2695`) | none | ply | no record found, – | L |
| Q5 | Delta rule | above 8 men, stand pat + a queen + 308 below alpha ends the node (`node.rs:2699-2729`) | none | eu | B.4: kept as a coordinate; RAR-S88 (+4.4 ± 2.9), C; Basilisk deleted its own | D |
| Q6 | Generation | all legal noisy, or all legal evasions, scored at once (`node.rs:2732-2765`) | the staged picker; quiets in check only while the best score is a loss (`search.rs:1319-1323`) | – | B.4 cluster, C | D |
| Q7 | Capture pruning | futility by victim + 257 unless check; a limit of 3 unless recapture, check or promotion; SEE ≥ min(224, alpha − stand pat − 247); a bad-capture floor of −43 (`node.rs:2797-2940`) | stop at the third move unless a direct check; one SEE threshold `(alpha − eval)/8 − min(|corr|, 68) − 74 − history/48` (`search.rs:1332-1344`) | eu, su, hu | B.4 research; RAR-S100 for the limit (2 and 4 against 3: no direction); C | D |
| Q8 | Evasions | quiet evasions skipped once one evasion is not lost; no count limit (`node.rs:2779-2782`) | quiets skipped by the picker on the same test; the three-move stop applies (`search.rs:1321-1336`) | – | RAR-S87 (+7.8 ± 9.4, F), then RAR-S88, C | D |
| Q9 | Tail | fail-hard: alpha returned and stored, exact or upper (`node.rs:2993-3020`) | fail-soft: the best score, lower or upper, never exact (`search.rs:1389-1400`) | eu | RAR-S15: fail-soft −5.96, reverted; RAR-R20, RAR-R21: the four-part cluster on the old kernel −52.5 ± 23.9, then −6.9 ± 8.8; RAR-R22: H0, −7.4 ± 7.5, S | D |
| Q10 | Fail-high softening | 594 and 575 of 1024 toward beta (`params.rs:516-518`) | 0.8256 and 0.5072 (`search.rs:1302`, `:1390`) | – | fitted, C | D |
| Q11 | History on a cutoff | none | +100 to a noisy move that cuts (`search.rs:1377-1387`) | hu | RAR-S87: −4.7 ± 9.4, F | D |
| Q12 | Stalemate | not scored | not scored | – | D.6.29 | same; D.7.8 asks |

### 3.8 Root window

| Id | Mechanism | Rarog | Reckless | Unit | Record, gate | New kernel |
|---|---|---|---|---|---|---|
| R1 | Window | from depth 4, ±12 round the last score or the pool's deeper one (`mod.rs:332-352`, `:995-1003`) | from depth 1, `23 − min(stability, 7) + avg²/26394` round a running average (`search.rs:98-118`) | eu | RAR-S89: one retry H0, −18.3 ± 6.9, S; RAR-S90: twice the width −25.4 ± 9.2, F | D |
| R2 | On a fail low | delta ×1.64 + 8 from the centre; beta to the midpoint (`mod.rs:354-367`) | alpha from the failing score; beta to alpha + delta; delta ×(1 + 26/128) (`search.rs:142-146`) | eu | RAR-S17: re-centring on the failing score H0, −4.52 ± 4.80 over 8,604 games, S | D |
| R3 | On a fail high | delta ×1.65 + 8 from the centre; no reduction (`mod.rs:369-382`) | beta from the failing score; one ply less unless decisive; delta ×(1 + 60/128) (`search.rs:147-156`) | eu, ply | RAR-R19: every root-loop device refuted as the fix; RAR-R22 | D |
| R4 | Fails before the window opens | 20 (`params.rs:177`) | none; the delta grows until it covers the score | n | termination by construction | K as a guard only |
| R5 | Stack between windows | kept | reset with the cutoff counts (`search.rs:127-129`) | – | none | D |
| R6 | Root node | a full scored list, pool order, helper rotation (`node.rs:1659-1704`) | the staged picker, re-scored before each pick (`movepick.rs:72-74`, `:92-94`) | – | MP-6; RAR-R03's bundle | K (root driver and SMP) |
| R7 | Fail-high stop, proven win, mate watch | Rarog's (`mod.rs:1053-1120`) | the donor's forgotten-mate and aborted-loss rules (`search.rs:197-222`) | – | D.1, D.6.20, D.6.30 | K |

### 3.9 Picker, histories, correction tables, SEE

The agents' tables are kept in this session's record; each row below names
the side's line and the proposal.

| Id | Mechanism | Rarog | Reckless | Record, gate | New kernel |
|---|---|---|---|---|---|
| MP-1 | Promotion pushes | all four after the good captures, before every quiet, no SEE test (`movepick.rs:579-582`) | the queen push is noisy and SEE-tested; the others are quiets (`movegen.rs:135-141`) | RAR-S73 resolution (2), C | D |
| MP-2 | Is an under-promotion push noisy | yes (`moves.rs:172-174`) | no (`types/moves.rs:70-72`) | D.6.1, RAR-S107 (F); D.6.17 | D |
| MP-3 | Quiets after the skip | direct checks and promotions still come (`movepick.rs:137-157`) | none (`movepick.rs:80-91`) | RAR-S73 resolution (2), C | D |
| MP-5 | Captures generated | before the TT move (`node.rs:1708`) | after it (`movepick.rs:54-59`) | D.6.6, RAR-S120: `NO_CHANGE` | D |
| MP-7 | Quiescence split | SEE ≥ 0; promotions always good (`movepick.rs:704-711`) | `−score/47 + 116` (`movepick.rs:65`) | none found, – | D |
| MP-9 | ProbCut list | PC3 | PC3 | RAR-S57, S | D |
| MP-10, MP-11 | Good-noisy threshold, victim weight | 63 and 87; 18,976 on 100/320/330/500/900 (`movepick.rs:36-45`) | 47 and 116; 14,232 on 109/403/435/679/1,242 (`movepick.rs:65`, `:138`) | the ×0.75 seed rule, C | D |
| MP-12, MP-13 | Ties and generation order | the earliest maximum (`movepick.rs:124-128`) | the latest maximum (`movepick.rs:116-121`) | none, – | K (noise for parity) |
| H-1 | En passant in the noisy table | a pawn everywhere (`board.rs:866-870`) | a pawn in the picker, none in the updates (`search.rs:1107`, `:1130`) | none, – | K |
| H-2, H-3 | Update coefficients | fitted (`params.rs:404-417`) | `search.rs:1093-1160` | B.2.3 SPSA and later refits, C | D (seeds) |
| H-5 | Depth the updates read | after IIR | no IIR | T3 | D |
| C-1, C-2, C-5 | Minor table, weights, update | W5, W6 | W5, W6 | C.10, C | D |
| C-3 | Kings in the non-pawn key | left out (`board.rs:1339`) | included (`types/keys.rs:28-31`) | BOARD-9; Basilisk +11.6 ± 9.3 alone, – | K now; the first candidate after adoption |
| C-4 | Continuation correction | two tables (`correction.rs:58-59`) | one table read at plies 2 and 4 (`history.rs:197-219`) | none, – | D |
| C-6 | Sharing between threads | per thread | pawn and non-pawn shared | D.2, RAR-M83 | K |
| SEE-1 | Values | 100/320/330/500/900 (`board.rs:62`) | 109/403/435/679/1,242 (`types/piece.rs:138-148`) | RAR-M19, – | D for every threshold and the victim term |
| SEE-2, SEE-3, SEE-4 | Legality of recaptures, promotion on recapture, promotion pushes | exact (`board.rs:1462-1495`, `:1605-1612`) | static pins only; no promotion gain (`board/see.rs:18-71`) | RAR-M28, RAR-E15's cluster, C | K (board) |

The tables themselves are the same on both sides: the quiet table by side,
threat bits, from and to (8,192); the noisy table (12,800); the pawn table in
512 buckets (8,192); the continuation table (15,320); gravity; the stage
order; the quiet score's terms and weights; the correction's keys, buckets
and limits; which moves take a malus and when.

### 3.10 Table, scores, node count, limits

| Id | Mechanism | Rarog | Reckless | Record, gate | New kernel |
|---|---|---|---|---|---|
| TT-1 | Key check | 16 bits (`tt.rs:581-583`) | 21 bits (`transposition.rs:122`) | D.2 review item 4, open | K |
| TT-2 | Clock in the key | none (`board.rs:569-571`) | a bucket of 8 plies (`board.rs:79-84`) | BOARD-9; Basilisk −4.5 ± 8.4, – | K |
| TT-3, TT-4 | Tablebase band in the table | stored as is; only mates meet the rule-50 horizon (`tt.rs:601-629`) | moved by ply; the band meets the horizon too (`transposition.rs:269-271`, `:321-345`) | B.5.2.1; TT-11 open in D.9 | K |
| TT-7 | Depth of a quiescence entry | 0, evaluation-only −2 (`node.rs:2683`, `tt.rs:31`) | −1 for both (`search.rs:466`, `:1306`) | none, – | K; rules that read the stored depth are written against Rarog's values |
| TT-8, TT-9, TT-10 | Quiescence and ProbCut stores | Q3, Q9, T9 | Q3, Q9, T9 | – | D |
| SC-1 to SC-5 | `MAX_PLY`, bands, clamps | 128; mate from 31,872; win from 31,744 | 240; 31,760; 31,519 | follow `MAX_PLY` | K |
| SC-8 | Printed score | the internal score | rescaled by material (`types/score.rs:58-70`) | D.3.9: `NO_CHANGE`, to F.6 | K, after the division of section 5 |
| NC-1 | A null move's child | counted (`node.rs:1139`) | not counted (`search.rs:597`) | D.6.8's contract; RAR-S124 kept, R | K |
| LIM-3 | `go movetime T` | thinks T (`time.rs:132-147`) | thinks T − 15 ms (`time.rs:20`, `:55-56`) | RAR-R02 | K |
| LIM-5 | Several limits in one `go` | all apply (`uci/go.rs:83-120`) | the first token only (`uci.rs:446-456`) | none | K |
| LIM-6 | One legal move | answers after depth 2 (`mod.rs:1164-1170`) | searches on | D.3.8 | K |

### 3.11 Checked and the same

The table cutoff's rule and its rule-50 guard; the cutoff bonus; the
evaluation stored raw and at once on a miss; the improvement and its two
fallbacks; hindsight's two tests; the null search's bound and reduction
formula; the verification's depth and region; ProbCut's depth formula, raised
bound and second try; the singular depth, margin shape and extension terms;
the cutoff count and its reset two plies down; every LMR term and the node
count's spread; the reduced depth's clamp and the PV's two plies; the child's
type on each search; the quiet and noisy lists up to 32 moves; mate and
stalemate at the node's end; the table's index, ageing and replacement; mate
distance pruning; the first iteration always finished under a node or time
limit.

## 4. Rarog's own forms the donor lacks: keep or drop

PLAN's method puts the whole kernel in the donor's form and asks afterwards.
So a form stays only where PLAN names it carried over or an invariant, or
where dropping it brings back a defect a repair read closed.

| Form | Record | Proposal | D.7.8 asks |
|---|---|---|---|
| Twofold draw in search (N5) | RAR-S18, which does not carry the code comment's claim | drop for the donor's rule; draws stay draws, so PLAN's identity holds | repetition draws on the replayed games |
| Draw verdicts exempt from correction (W4) | D.6.15, repair case 1 | keep | whether the donor's optimism reaches a verdict (it must not) |
| One count per position, null moves counted (NC-1) | D.6.8, RAR-S124 | keep | no |
| TT move legality at the probe (T2) | correctness | keep | no |
| Exact SEE (SEE-2 to SEE-4) | RAR-M28, RAR-E15 | keep the algorithm, take the donor's values | no |
| Proven win kept, fail-high stop, mate watch (R7) | D.6.30, D.6.20, D.1 | keep: root driver | D.6.20's clock test |
| Line bound and extension budget (N6, LR4) | D.6.7, RAR-S122 | drop | the seldepth chains on RAR-S121's replay |
| Lone-move exclusion returns alpha (SE6) | code comment | drop | the same chains: this is what fed them |
| ProbCut's SEE-gap filter and cap of 5 (PC3) | RAR-S57, an SPRT | drop | ProbCut's conversion rate on the new kernel |
| Quiet checks and promotions survive the skip (MP-3) | RAR-S73 resolution (2) | drop | the late direct check test (`node.rs:3342`) |
| Promotion pushes ordered ahead of quiets (MP-1) | RAR-S73 | drop | D.6.17's root promotion anchor |
| Multi-cut's window test (SE3) | won-ending comment | drop | the won-ending clock test (D.6.20) |
| Null cut never returns a win (NM5) | none | drop | mate canaries |
| Quiescence depth cap (Q4) | B.4 | drop | the ply-cap count on the replay |
| IIR (T3) | RAR-S96 | drop | D.6.27 closes if the tree does not miss it |
| The correction's minor table (C-1) | C.10 | weight 0; a fit coordinate | the fit decides |
| Fail-low stores a move (T8) | D.6.4 | drop | closes with IIR gone |

The queen push in quiescence (D.6.2, RAR-S108, +12.2 ± 6.5) needs no row: it
is the donor's form.

## 5. Seeds and units

**Decision: the kernel is built in the donor's units and then folded into
Rarog's.** While it is built and compared with the hybrid, the wrapper
multiplies Rarog's raw evaluation by 2536/1024, as the hybrid does, and
every constant below crosses unchanged.

**Amended 2026-11-13 (maintainer decision):** no translation stays in the
engine. The multiplication is a scaffold for D.7.4. D.7.4.1 folds the
constants into Rarog's units and deletes it, before the unfitted run and
the fit; the rule is at the end of this section. The reasons below are why
the kernel starts in the donor's units, not why it would stay there.

Why, against converting the constants by 1024/2536:

1. *The hybrid is the one configuration of known strength.* Each converted
   and rounded constant moves away from it, and a parity failure could then
   be a rounding or an error with no way to tell.
2. *Resolution.* One Rarog unit is 2.48 of the donor's. The singular margin
   is 1 a ply, the re-search margins 9 and 57, LDSE 25, the stability test
   12. Rarog already needed sixteenths and quarters to hold such values
   (`singular_margin`, `nmp_depth`, `rfp_square`).
3. *Precedent.* Phase B converted at ×0.457 for evaluation units, a
   search-facing ratio, where the raw ratio it measured was 0.406 (RAR-M50)
   and the hybrid plays at 0.404. Its unfitted seeds stood 138.6 Elo below
   their fit (RAR-S73, B.2.4b).
4. *The conversion can then be checked.* With a kernel in the donor's
   units beside it in one binary, a converted constant is right or wrong
   at zero games. Phase B had nothing to check its conversion against.

What it costs, all at one boundary:

- **Scores leaving the kernel** are divided by 2536/1024, mate and tablebase
  bands untouched: the UCI score, the result's score, the root records the
  clock reads (`score_fall`, `cross_move_fall`, the remembered score), and
  any data output. Pool scores between threads stay in kernel units.
- **The printed score rises about 16% at full material**, because the
  donor's wrapper lifts where Rarog's is neutral there. D.3.9 left score
  normalisation to F.6.
- **SEE thresholds read the donor's piece values** (109, 403, 435, 679,
  1,242) through the value-set argument `see_ge_with_values` already takes.
  The exchange algorithm stays Rarog's.
- **Range.** A scaled evaluation reaches Rarog's win band at |raw| above
  12,817 and the table's 16-bit field at 13,231. The wrapper clamps to
  ±31,743 as now. Whether a legal position evaluates that high is not known;
  D.7.4 tests many-queen placements.
- **The table is cleared when the switch changes**, since stored scores
  change unit.
- PROCESS's boundary says a constant is "converted through the measured
  scale ratio". The fold is that conversion, so the sentence stands.
- These costs last only while the scaffold does. After the fold, scores,
  table entries and printed values are Rarog's again.

### The seed table

Donor values at `0756957`. `d` depth, `imp` improvement, `corr` the
correction's size, `h` history, `mc` move count. Units as in section 3.
"Rarog" gives today's fitted value in Rarog's units where the form is shared.

| Group | Donor's form and values | Unit | `search.rs` | Rarog today |
|---|---|---|---|---|
| Wrapper | `(raw·(21032 + mat) + opt·(1548 + mat))/27015`; `·(200 − clock)/200`; `+ corr`; raw = HCE·2536/1024 | eu | `evaluation.rs:3-13` | scale 15, damping 100 |
| Correction | sum of five tables `/64`; update `148·d·diff/128` in [−4678, 2496] | table | `:1403-1452` | 250, [−1692, 736], six weights |
| Optimism | `113·avg/(|avg| + 201)`, `avg` the mean of the shared and the own running average | eu | `:120-124` | none |
| Aspiration | delta `23 − min(eval_stab, pv_stab, 7) + avg²/26394`; low ×(1 + 26/128); high ×(1 + 60/128) and one ply less; stability counts a score within 12 of the average | eu | `:98-171`, `:185-195` | 12, 164%, 165%, +8 |
| Draw | `nodes % 5 − 2` | eu | `score.rs:20` | 0 |
| TT cutoff bonus | quiet `min(190d − 81, 1691)`; continuation `min(96d − 73, 1206)`; parent `mc < 4`; clock < 90 | hu | `:396-406` | cap 1727 |
| Eval-difference bonus | `clamp(812·(−(eval + parent eval))/128, −144, 324)`; `d < 6` or no entry | hu | `:492-503` | same text |
| Hindsight | +1 at parent reduction ≥ 2249 and delta < 0; −1 at delta > 57 | 1024ths, eu | `:506-517` | 1676, 34 |
| Razoring | `est < alpha − 237 − 254d²`, `alpha < 2048` | eu | `:538-546` | 302, 184, 936 |
| Reverse futility | `max(2, 1140d²/128 − 120·imp/1024 + 22d + 669·corr/1024 − 54·[no piece attacked] − 19)`; return 0.6945 toward beta | eu | `:549-564` | 170/16, 58, 9, 95, 12, −14; 586 |
| Null move | margin `max(2, −9d + 110·tt_pv − 94·imp/1024 − 21·[child cutoffs < 2] + 337)`; material > 491; `R = (4407 + 917·improving + 265d + 477·clamp(est − beta, 0, 1187)/128)/1024`; verify from depth 16 at `d/2` or `d − R`, region `ply + 3·depth/4` | eu, 1024ths | `:567-633` | 159, 14/4, 52, 16; 4255, 929, 277, 899, 475; 16 |
| ProbCut | beta `+ 254 − 85·improving`; base depth `d − 4 − improving`; one ply less per 319 cleared; +197 a ply saved; return 0.2695 toward beta | eu | `:636-705` | 145, 39; 124; 107; 253 |
| Singular | candidate `d ≥ 5 + tt_pv`, entry ≥ `d − 3`; margin `d` (exact: `⌈d/4⌉`) `+ d·[PV flag off the PV]`; depth `(d − 1)/2`; double `195·PV + 48·[new PV] − 16·quiet − 16·corr/128`; triple `230, 56, 19, 15, + 36`; multi-cut 0.4027; negative −3; LDSE `d ≤ 7`, 25 | eu, ply | `:519-523`, `:711-756` | 4, 2; 12/16; 95, 29, 10, 12, −11; 125, 23, 7, 6, 12; 379; 12 |
| Late-move pruning | `mc ≥ (2818 + 78·imp/16 + 1351d² + 74h/1024)/1024` | n | `:795-803` | 2888, 80, 402, 75 |
| Futility | `eval + 79d + 55h/1024 + 77·[eval ≥ beta] + 555·corr/1024 − 127`, `d < 14` | eu | `:806-819` | 33, 22, 35, 110, 150 |
| Bad-noisy futility | `eval + 84d + 82h/1024 + 24`, `d < 11` | eu | `:822-834` | 40, 46, 56, plus the victim |
| History pruning | `h < −948d`, `d < 5` | hu | `:837-839` | 1019 |
| SEE pruning | quiet `min(0, −12d² + 56d − 27h/1024 + 27)`; noisy `min(0, −7d² − 36d − 39h/1024 + 14)` | su | `:842-850` | 7, 31, 24, 17; 5, 12, 33, 33 |
| LMR | `269·log2 d − clamp(425·imp/128, −241, 1155) − 3417·corr/1024 + 1412·exact + 464·[tt ≤ alpha] + 326·[tt shallow] + 1024·[win beta]`; quiet `+2432 − 179h/1024 + 418·clamp(alpha − est, −65, 91)/128`; noisy `+1687 − 130h/1024`; `−128·min(plies since critical, 8)`; PV `−519 − 437·window/root window`, else `−96 + 32·laterality`; PV flag `−333 − 611·[tt > alpha] − 685·[tt deep]`, else cut node `+1852 + 2204·[no TT move]`; check −955; child cutoffs > 2 `+1151 + 400·[all node]`; singular `clamp(496·(gap − 185)/128, 0, 2021)`; parent `+136` past 414; spread `(nodes + 27·thread) % 128 − 59` | 1024ths | `:861-924` | 150; 333, −162, 1065; 3330; 1748; 526; 58; 1932, 316, 291, −65, 98; 830, 182; 171; 399, 306; 358, 24; 179, 571, 633; 2071, 2073; 1030; 1334, 253; 931, 89, 2084; 130 |
| Re-search | +1 past `best + 57`, −1 below `best + 9`, not at the root | eu | `:926-936` | 43; none |
| Unreduced searches | `207·log2 d − clamp(366·imp/128, −206, 1370) − 2255·corr/1024`; quiet `+1468 − 118h/1024`; noisy `+940 − 63h/1024`; PV flag `−844 − 1129·[tt deep]`, else cut node `+1260 + 2168·[no TT move]`; child cutoffs > 2 `+1394 + 258·[all node]`; singular `clamp(351·(gap − 188)/128, 0, 2167)`; TT move −3002; parent `+130` past 590; spread `(nodes + 26·thread) % 128 − 56`; one ply at 2621, two at 5579 | 1024ths | `:939-993` | none |
| Best-move updates | noisy bonus `min(96d, 885) − 43 − 87·cut`; noisy malus `min(175d, 1252) − 58 − 16n`; quiet bonus `min(184d, 1742) − 72 − 42·cut`; quiet malus `min(171d, 1099) − 46 − 31n`; continuation bonus `min(97d, 1098) − 74 − 48·cut`; continuation malus `min(414d, 949) − 49 − 17n`; fade `1024²/(1024 + 45i)²`; parent malus `min(93d − 52, 935)`; re-search bonus `min(233d − 86, 1550)` | hu | `:1092-1149` | 173, 1532; 197, 1405; 42; 1051; 894 |
| Fail-low reward | factor `88 + min(17·mc, 229) + 110·[TT move] + 144·[best ≤ eval − 97] + 306·[best ≤ −parent eval − 136]`; quiet `factor·min(180d − 37, 2414)/128`; grandparent `min(152d − 47, 1379)`; noisy `min(50d, 654)` | hu, eu | `:1151-1181` | 92; 44; 62 |
| Quiescence | stand pat 0.8256 toward beta; stop at the third move; SEE `(alpha − eval)/8 − min(corr, 68) − 74 − h/48`; cutoff bonus 100; cutoff 0.5072 toward beta | eu, su, hu | `:1299-1391` | 594; limit 3; 247, 224, −43; 575 |
| Picker | good noisy at SEE ≥ `−score/47 + 116`; victim `14232·value/1024`; queen promotion 4558; in check `200000 − 20000·piece`; quiet weights 1763, 1024, continuation 1614, 1066, 1086, 1051; escape 8854, 8170, 14051, 20357; check square 10723; threatened −8875; offense 3446; king wall −4494 | hu, su | `movepick.rs` | 63, 87; 18976; the rest equal |
| SEE values | 109, 403, 435, 679, 1242 | su | `piece.rs:138-148` | 100, 320, 330, 500, 900 |

The fit's surface (D.7.6) is these groups after the fold. Categoricals (the
population rules, the picker's forms) are switches, never coordinates.

### The fold into Rarog's units (D.7.4.1)

One mechanical step after parity, checked against the scaffold.

- **By dimension, not by a single factor.** With `k` = 1024/2536:
  - a value in evaluation units (a margin, a clamp on an evaluation
    difference, the aspiration delta, optimism's 113 and 201) is
    multiplied by `k`;
  - a coefficient that turns evaluation units into something else
    (reduction per unit of improvement, history bonus per unit of
    evaluation swing, the correction's update slope, depth per unit
    cleared in ProbCut) is divided by `k`;
  - a ratio of two evaluation quantities, a count, a ply, a history value
    and a reduction in 1024ths are unchanged;
  - a quadratic term (`avg²/26394`) is converted by `k` once for each unit
    it carries.
- **Resolution.** A converted value below 64 of the donor's units keeps
  sixteenths of a Rarog unit, as `singular_margin` does today. Nothing is
  rounded to a whole unit where that moves it by more than 1%.
- **Exchange values.** The donor's five piece values (109, 403, 435, 679,
  1,242) are a table the SEE thresholds and the victim term were fitted
  with, not a translation. My proposal is to keep them as the search's
  exchange table; the other way is to convert the SEE-unit thresholds to
  100/320/330/500/900 at ×0.75. The maintainer is asked when D.7.4.1
  starts.
- **Checks, at zero games, scaffold against folded in one test build:**
  1. Every decision record of layer A is reproduced by the folded rules,
     except where the deciding quantity lies within one Rarog unit of its
     threshold. The exceptions are counted and stay under 2% of any kind.
  2. Positions visited to depths 8, 11 and 14 on layer B's 500 positions:
     the mean log ratio within three standard errors of the noise floor
     plus 0.01.
  3. A plant: one constant converted by the wrong dimension must fail
     check 1 for its kind.
- **Then the scaffold is deleted:** the multiplication at the wrapper, the
  division at the boundary and the donor-unit copies of the constants. The
  folded kernel's bench is its fingerprint from there.
- **What the fit then does.** Its seeds are right to rounding, so SPSA
  starts at the measured configuration and needs no large movement. The
  curvature sweep (PLAN rule 4) still comes first, and a coordinate that
  runs to its bound is widened and registered again, not left there.

## 6. RAR-O09

Registered in `docs/experiments/RAR-O09.md`. What the design adds to PLAN's
three reads, and why:

- **A node is not the same thing in the two engines.** Rarog counts a null
  move's child (D.6.8's contract) and Reckless does not. A null-move child is
  a full node of work, so equal `go nodes` favours the donor by its null-move
  share, expected 1% to 3%, about 2 to 6 Elo at this control's 2 Elo per 1%.
  That is the size of the +10 bar's margin, so it is measured on a counting
  twin and returned in the donor's budget.
- **A move time is not the same either.** The donor stops 15 ms early. At 70
  ms that is 21%. The commands are 70 ms and 94 ms, checked in a smoke.
- **A fourth match replays RAR-O08's match (5).** The shares are differences
  between matches. They need the same binaries, and the head has moved
  (D.6.8, D.6.30, the first-iteration fix); they also need one runner.
- **The runner rule.** Colosseum resets the book's move counters; both time
  managers read them, the donor's much more (its first soft bound falls from
  93 to 51 ms). The clock legs replicate RAR-O08 only on fastchess or on a
  Colosseum that keeps the counters. The deciding match is unaffected.
- **The use is frozen, including my recommendation below PLAN's bar**, so it
  is not chosen after the result.

## 7. The parity instrument (D.7.3)

**Why not identical trees.** Even with every rule equal, the two engines
index their history, correction and transposition tables by different Zobrist
keys, so they alias differently; they break ordering ties differently; and
the node count that feeds the reduction spread and the draw score runs in a
different phase. The trees diverge within a few thousand nodes by
construction. Parity is therefore exact where it can be, on single decisions,
and statistical on whole trees.

**Layer A: decision records, exact.**

- *The twin.* `tools/oracle-hybrids/reckless-0756957-rarog-hce-trace.patch`
  (planned), on top of the hybrid's patch. With `RAROG_TRACE` set it writes,
  at every n-th node, one line for each decision that node takes: the
  decision's kind, its inputs and its outputs, as named integers. With the
  variable unset the twin's bench node count equals the hybrid's, and the
  count is also equal with tracing on; both are checked.
- *Kinds*, one per rule in section 5's table: `wrapper`, `tt_cut`,
  `estimate`, `eval_bonus`, `hindsight`, `razor`, `rfp`, `nmp_gate`, `nmp_r`,
  `nmp_result`, `probcut_gate`, `probcut_depth`, `singular_gate`,
  `singular_result`, `ldse`, `lmp`, `futility`, `bad_noisy`, `history_prune`,
  `see_margin`, `lmr`, `research`, `unreduced`, `updates`, `fail_low_reward`,
  `corr_update`, `q_tt_cut`, `q_stand_pat`, `q_see`, `q_tail`, `window`,
  `optimism`, `pick_noisy`, `pick_quiet`, `gravity`.
- *Rarog's side.* The new kernel holds every rule as a pure function of an
  input struct, as `late_move_reduction` and `nmp_margin` are today. An
  ignored test reads the trace named by `RAROG_DONOR_TRACE` and calls the
  function for each record.
- *Pass:* no mismatch in at least 2,000 records of each kind, 200 for kinds
  that fire rarely, from at least 200 positions.
- *Plant:* one constant changed in a test build must fail the kind it
  belongs to and no other.
- The trace stays in ignored storage with its hash recorded. It is the
  donor's behaviour as numbers, and it is not versioned (maintainer
  decision 2026-11-13), so the rule tests are a local check, not CI's.

**Layer B: tree statistics.**

- *Positions:* 500 from RAR-O08's match (5), the position after every 20th
  ply, a fresh process each, Hash 64.
- *Read:* positions visited to depths 8, 11 and 14 (the twin's nodes plus its
  null moves; Rarog's nodes), and how often each kind of layer A fires per
  1,000 positions, from the twin's counters and Rarog's probes.
- *Noise floor:* the twin with `RAROG_SPREAD_SALT` 1 to 4, a constant added
  to the node count inside the two spread terms and the draw score. It
  changes nothing but the phase, so the spread of the log ratio between
  salts is what two equal kernels show. It is measured before Rarog's kernel
  is compared.
- *Pass:* at each depth the mean log ratio of positions visited is within
  three standard errors of the floor plus 0.03, and each kind's rate is
  within three standard errors or 10%, whichever is larger. The 0.03 is an
  allowance for what is kept different on purpose (the table, the draw rule,
  ties, SEE's legality); it is frozen here and not widened after a read.
- *Plants:* the unreduced-search reductions removed, LMR off in check, and
  the quiescence stop at five moves must each fail. A plant that passes sets
  the instrument's stated limit.

**Layer C: one node replayed**, built only if A passes and B does not. The
twin records one node whole: its inputs, each child search it asked for and
what came back, and what it stored. Rarog's node function runs on the real
board with a test-only hook answering the child searches from the record,
and must ask for the same searches and store the same result. It finds a
wrong order of statements, which neither A nor B names.

**Stop rule (PLAN's):** parity not reached, no games.

## 8. The kernel (D.7.4)

**Contract.** One switch. With it off the old kernel's fingerprint is exact
(8,935,848) and a pooled NPS read shows no cost. The new code is Rarog's own,
written from this record and the donor's mechanisms, in D.8's target shape:
rules as pure functions, instrumentation as probes.

**Scope.** In: the node body, quiescence, the root window, the wrapper, the
picker's forms marked D, the seeds of the history and correction updates,
upcoming repetition. Out: the evaluation, board and move generation other
than the cuckoo tables, the table's structure, time management, Lazy SMP, the
root driver, the protocol, tablebases.

**Order, each step its own commit with the old fingerprint exact:**

1. The switch, the unit boundary of section 5, the wrapper, the donor's SEE
   values as a value set. Layer A: `wrapper`.
2. The rules as pure functions with their input structs. Layer A: every
   kind but the picker's.
3. Upcoming repetition in the board: cuckoo tables, behaviour-neutral for the
   old kernel. Perft and the repetition tests.
4. Quiescence in the donor's form, typed by node.
5. The node body: order at entry, table use, pruning, null move, ProbCut,
   singular search, the move loop with both reduction formulas, the stores.
6. The root window and optimism.
7. The picker's forms and the update seeds.
8. Probes for D.4's instruments. Layer B, then the allocation guard, the
   search-path perft, debug and release tests, fmt, clippy.

D.7.4.1 follows: the fold of section 5 and the scaffold's deletion, before
any game.

**Invariants** (PLAN's, with their tests): table bounds and the mate and
tablebase bands; the draw, repetition and rule-50 identities; no allocation
per node; no large value passed by copy (check the assembly for `memcpy`);
one count per position; D.6.17's and D.6.20's acceptance tests, which may
stay ignored until D.7.8.

**Not in this step:** tuning; any form marked L or K coming back; a change to
the old kernel; a change to the experiment after parity is read.

**A false premise** (a donor rule that cannot be written without breaking an
invariant) stops that rule, is recorded here, and the leaf returns to
`RESEARCH` for it.

## 9. The Colosseum wrapper (D.7.1.1)

Colosseum CLI 0.5.0's `match` takes `--a-nodes`, `--a-movetime-ms`,
`--a-base-ms`, `--a-increment-ms` and `--a-option`, and the same for B; dry
runs resolve all four of RAR-O09's legs. `tools/colosseum.ps1` does not:

1. **No move time.** Add `-MoveTimeMs` and `-MoveTimeMsB` for `-Mode match`,
   a run file `tools/colosseum/match-fixed-movetime.toml` (planned), and the
   resolved-configuration check for a per-move control on each side
   (`colosseum.ps1:516-517`, `:578-591`).
2. **One node budget for both sides.** Add `-NodesB`; the check at
   `colosseum.ps1:570-577` compares each side with its own value.
3. **A foreign arm passes on warnings.** Add `-ForeignB` for `-Mode match`:
   B has no sidecar, its SHA-256 and `id name` go in the manifest, A's
   sidecar is required, `-ExpectBench` takes A's value alone, and
   `-ExpectRevision` says in the manifest that it checked A only. Without
   the switch a sidecar-less arm in `sprt` or `match` is refused
   (`harness_common.ps1:651-657`, `:720-725`).
4. **Tests.** Each refusal and each control in
   `tools/diag/test_colosseum_guards.ps1`; a wire smoke per limit in which an
   absurd value moves the PGN and the score, as D.7.1 did for fastchess.
5. **RAR-O09's four commands**, with their dry runs, written into its
   registration's Colosseum form.

Requests for the Colosseum maintainer are in
`analysis/colosseum_requests_2026-11-13.md`: item 2 (the book's move
counters, needed by the clock legs), 11 (`winc` and `binc`), 12 (a match's
score in `results.txt` and the run record), 13 (node-budget compliance).

## 10. Not verified, and decisions that are the maintainer's

Not verified:

- The null-move share, the donor's speed in games and the bridge's price in
  games. RAR-O09's preparation and matches read them.
- Whether Rarog's evaluation ever exceeds 12,817.
- Generation order and tie order position by position.
- The 0.03 allowance of layer B. It is a judgment.
- Which refit set today's history coefficients (the C.10 patch sets the
  correction's).

Found in passing, not corrected here because it is engine source: the
comment at `src/board/board.rs:1243` says a root-aware repetition rule
"measured −7.21 ± 6.03 Elo". RAR-S18's record gives that figure to a
four-part bundle and −11.91 ± 7.67 to the bundle without the root-aware
part, and says the loss lies in the other parts. The comment is corrected in
D.7.4's first engine commit.

The maintainer's:

1. **The runner for RAR-O09**, by the registration's rule, before the first
   game. 2026-11-13: fastchess, unless Colosseum CLI is improved first; so
   D.7.1.1 does not hold D.7.2.
2. **Whether the trace is versioned.** 2026-11-13: it stays in ignored
   storage.
3. **If match (1) reads below +10:** whether the second aim alone opens the
   rebuild. My recommendation is fixed in the registration: against it at or
   below zero, unresolved between. 2026-11-13: the maintainer agrees with
   the assessment and may still open the rebuild against it, as PLAN's
   rule allows.

## 11. Models, by GUIDE's mapping

| Leaf | Model and mode | Why |
|---|---|---|
| D.7.1.1 | Claude Sonnet — Medium | a specified wrapper change with guard tests and smokes |
| D.7.2 | Claude Sonnet — High | builds, the counting twin, smokes and reading four matches by frozen rules |
| D.7.3 | Claude Opus — High | a tracing patch in the donor and a calibrated statistic |
| D.7.4 | Claude Opus — High | kernel implementation against a reference, per-node cost rules |
| D.7.4.1 | Claude Opus — High | a dimensional conversion of about a hundred constants with its own checks |
