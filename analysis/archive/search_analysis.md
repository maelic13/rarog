# Rarog search analysis

Status: working document  
Analysis date: 2026-07-13  
Rarog revision: `ff21dc1` (`development`)  
Related review: [`D:/code/basilisk/analysis/archive/search_analysis.md`](../../basilisk/analysis/archive/search_analysis.md)

## Executive summary

Rarog is not mainly losing because one pruning coefficient is wrong. Its search is already broad and recognizably modern: PVS, aspiration windows, TT cutoffs with a PV bit, null move with verification, ProbCut, singular extensions, several history families, correction history, SEE pruning, qsearch TT storage, and score/depth-weighted Lazy-SMP voting are all present.

The larger gap is that these mechanisms are composed more categorically than in the strongest public engines. Rarog still treats checks, checked nodes, TT-PV ancestry, captures, promotions, and history updates as mostly separate cases. Stockfish and the strongest open engines increasingly make the same decisions from a richer confidence model: node type, move count, history, correction magnitude, TT quality, threats, expected cut-node status, and the result of the reduced search all influence how much work a move receives.

The most important findings are:

| Priority | Finding | Assessment |
|---|---|---|
| P0 | The 50-move draw is tested before checkmate at non-root nodes | **Verified correctness defect**; reproduced locally |
| P1 | Checks and checked nodes are overprotected | Unconditional check extension, no LMR in check, no LMR for checking moves, and pruning exemptions stack together |
| P1 | A historical TT-PV bit disables too much pruning | `tt_pv` gates RFP, razoring, NMP, ProbCut and move pruning, even when the current node is not PV |
| P1 | History is structurally rich but trained on too few events | Most learning occurs only on beta cutoffs; exact/PV best moves and TT cutoffs provide little or no feedback |
| P1 | LMR eligibility is too categorical | Good captures, promotions, checks and all moves in checked nodes are never reduced; reductions cannot become zero or negative once eligible |
| P1 | Root search throws away uncertainty and per-move work | No persistent root-move score/PV/variance/effort record; this limits aspiration, time management and SMP coordination |
| P2 | Correction history is updated by tactical outcomes and has a shallow continuation key | This can teach the static evaluator to absorb search tactics and then feed that noise back into pruning |
| P2 | Several prunes discard fail-soft information | Plain `continue` paths lose a useful lower estimate for `best_score`, TT bounds and later decisions |
| P2 | Null-move verification only suppresses null move at the verification root | Descendants can use null move again; modern implementations suppress it over a verified subtree/ply region |
| P2 | SMP diversity is weaker than it appears | Root score rotation does not displace a legal TT move, so helpers often start with the same root move |

These are architectural hypotheses, not guaranteed Elo. Each search change must be measured by paired games and diagnostics. Rarog's current documented `bench 13` result (13,541,282 nodes, geometric-mean EBF 2.548) shows substantially more tree growth than the approximately 1.8-2.0 range often seen in current Stockfish-style search, but EBF is position-, evaluator- and implementation-dependent. It is a locator for inefficient selectivity, not an Elo metric.

## Evidence classes

This document uses four labels:

| Label | Meaning |
|---|---|
| **Verified** | Directly reproduced or unambiguous from Rarog's code |
| **Strong** | Clear code-level difference from multiple leading public engines and a plausible mechanism |
| **Medium** | Plausible transfer from Basilisk/top-engine practice, but interaction or expected size is uncertain |
| **Do not transfer** | Basilisk's finding is absent, already handled, or materially different in Rarog |

Ranking does not establish causality. The mid-2026 baseline merely identifies relevant comparison engines. The [CCRL 40/15 list](https://computerchess.org.uk/ccrl/4040/) places Stockfish 18, Reckless 0.9, PlentyChess 7, Torch v4d and Obsidian 16 at the top; the [CCRL FRC list](https://computerchess.org.uk/404FRC/) has Reckless and Stockfish first and second. Torch's current source is not public, so detailed comparisons below use public snapshots of Stockfish, Reckless, PlentyChess and Obsidian. Stockfish development is also a moving target: the official [progression tests](https://official-stockfish.github.io/docs/stockfish-wiki/Regression-Tests.html) report large cumulative gains over Stockfish 18 during 2026, from both search and NNUE changes.

## Applicability of the Basilisk findings

| Basilisk section | Applies to Rarog? | Rarog-specific conclusion |
|---|---|---|
| 1. Check extension/checking moves | **Yes - strong** | The same compound overprotection exists and is one of the main search targets |
| 2. Unified LMR/pruning confidence | **Yes - strong** | Rarog has more LMR terms than Basilisk, but eligibility and pruning remain categorical and disconnected |
| 3. Sparse/context-poor history | **Yes - strong** | Rarog has richer tables, yet update coverage is still concentrated at beta cutoffs and lacks threat context |
| 4. TT semantics/density | **Partial** | Rarog already stores a PV bit and local TT density is good; rule-50 safety and shared-TT/helper policy remain relevant |
| 5. Root information loss | **Yes - strong** | Root state is essentially a move list plus the current best result, not a persistent `RootMove` model |
| 6. Lazy SMP diversity/merge | **Partial** | Diversity is weak, but the "weak merge" criticism does not apply: Rarog already uses score/depth-weighted voting |
| 7. Qsearch evasions/cap/storage | **Mostly no** | Rarog correctly searches all legal evasions, orders them, stores qsearch TT results and does not cap checked qsearch at `MAX_QPLY` |
| 8. Correction semantics | **Yes - strong** | Tactical best moves are not excluded; continuation correction is only keyed by the previous piece/destination |
| 9. Upcoming repetition | **Yes - medium** | Rarog detects positions already repeated in its history, not cycles that can be closed by one legal move |
| 10. Rule-50/checkmate defect | **Yes - verified** | Exact Basilisk reproduction also fails in Rarog |
| 11. Bound shaping/fail-soft | **Yes - medium** | Several move-pruning and qsearch exits return alpha or skip without retaining a fail-soft estimate |
| 12. ProbCut/null verification | **Partial** | Both exist and are stronger than Basilisk's baseline, but TT veto/context and subtree-wide NMP suppression are missing |
| 13. IIR/node type | **Partial** | IIR is broad and only weakly node-type aware |
| 14. Regression gates | **No, as stated** | Rarog's plan correctly treats SPRT as the strength verdict; fixed-depth tests mainly cover correctness/repeatability |

## 1. Checks and checked nodes are overprotected

**Evidence: strong.**

At every checked node Rarog increments depth unconditionally ([`search.rs:958`](../src/search.rs#L958)). Later it disables the entire forward-pruning block when in check ([`search.rs:1058`](../src/search.rs#L1058)), disables move pruning in check ([`search.rs:1251`](../src/search.rs#L1251)), and excludes checked nodes and checking moves from LMR ([`search.rs:1366`](../src/search.rs#L1366)). Checking moves are also exempt from quiet/history/futility and capture-SEE pruning ([`search.rs:1267`](../src/search.rs#L1267), [`search.rs:1277`](../src/search.rs#L1277), [`search.rs:1290`](../src/search.rs#L1290)).

The effects compound:

1. A quiet check is not move-pruned.
2. It is not late-move reduced.
3. Its child is in check, so the child receives another ply.
4. The checked child cannot use the normal forward-pruning block.
5. Every reply at that checked child is also searched without LMR.

This gives a late, dubious quiet check several plies more effective work than a quiet positional move with excellent history. It also makes check-heavy lines a likely source of Rarog's high EBF.

Current Stockfish/Reckless-style search does not use "tactical category = full depth" as a blanket rule. Checked nodes can still reduce late evasions, and checking moves can be reduced when their ordering/history/SEE context is weak. Stockfish's move-ordering check bonus is also conditional on SEE rather than treating every check as equally forcing.

Recommended experiment sequence:

| Step | Change | Why isolate it |
|---|---|---|
| 1 | Remove the unconditional node-level check extension | It is the largest multiplicative source of extra work |
| 2 | Permit LMR in checked nodes, initially with a modest reduction discount | Separates evasion selectivity from checking-move handling |
| 3 | Permit LMR for late checking moves, with SEE/history protection | Avoid a simultaneous aggressive pruning change |
| 4 | Replace unconditional pruning exemptions with a forcing-confidence adjustment | A sound check can still receive protection without protecting every check |

Instrument `nodes_in_check`, `check_children`, `reduced_evasions`, `reduced_checks`, re-search rate and tactical-suite failures. A raw node reduction without stable game strength is not enough.

## 2. `tt_pv` is used as a global no-prune flag

**Evidence: strong; Rarog-specific.**

Rarog defines:

```text
tt_pv = current node is PV OR the probed TT entry was stored from a PV node
```

The historical TT bit is useful information, but Rarog promotes it into a broad safety classification. `!tt_pv` is required for RFP, razoring, null move and ProbCut ([`search.rs:997`](../src/search.rs#L997), [`search.rs:1058`](../src/search.rs#L1058)), and for all late move/futility/history/SEE pruning ([`search.rs:1251`](../src/search.rs#L1251)). A collision-free, legally matching entry can therefore make a present non-PV node much more expensive merely because that position was PV in another path or iteration.

Leading search uses TT-PV status selectively: it can reduce LMR aggressiveness, affect singular-extension logic, change correction/history weights or participate in a node-confidence score. It is not normally a universal prohibition on forward pruning.

Recommended redesign:

- Keep `is_pv` as the hard semantic gate where correctness/search-window assumptions require it.
- Treat stored TT-PV as one confidence input: smaller LMR, larger futility margin, stricter ProbCut, or higher move-count threshold.
- Require current evidence as well: TT depth, bound quality, TT score relative to the window, move legality, correction magnitude, and node type.
- Measure how many nodes enter each prune with `is_pv == false && stored_tt_pv == true`; this reveals the actual opportunity before changing policy.

This likely interacts strongly with check overprotection: a checked node with a TT-PV entry currently triggers both protection systems.

## 3. LMR and forward pruning need a shared confidence model

**Evidence: strong.**

Rarog's reduction amount is already more nuanced than Basilisk's. It includes a depth/move-count table, TT-PV, improving, exact-bound, nominal shallow-TT, cut-node, bad-capture and quiet-history terms ([`search.rs:1378`](../src/search.rs#L1378)). That is good infrastructure.

The problem is the boundary around that calculation:

- only moves at depth at least 3 and move index at least 3 are eligible;
- all moves in check are excluded;
- good/equal captures are excluded;
- promotions and checking moves are excluded;
- once eligible, reduction is clamped to at least one ply ([`search.rs:1407`](../src/search.rs#L1407)); exceptionally strong late moves cannot receive zero reduction or a fractional extension from the same model;
- RFP/razoring/NMP/ProbCut happen before the per-move LMR calculation and use a different, much smaller context set;
- quiet move pruning has a separate collection of thresholds and does not use a prospective reduced depth.

There is also a likely naming/implementation discrepancy: the `lmr_shallow_tt` parameter is documented as a shallow/absent-TT term, but the live condition is `!tt_move.is_null() && searched >= 4` and never compares TT depth ([`search.rs:1392`](../src/search.rs#L1392)). This may be intentional inherited behavior, so it should be resolved by renaming or a dedicated A/B test rather than silently "fixed."

A stronger design computes a provisional `lmr_depth` or confidence score once, then uses it consistently:

```text
base reduction(depth, move count)
  + expected cut-node pressure
  + weak/absent/shallow TT evidence
  + bad capture or poor SEE
  + correction-history uncertainty
  - PV / TT-PV confidence
  - strong main + continuation + pawn history
  - forcing/threat evidence
  - exact/deep TT evidence
```

That prospective depth can drive LMP, futility, SEE pruning and the actual reduced search. It avoids contradictory outcomes such as "too tactical to reduce" but "too quiet to update" and makes feature experiments composable.

Missing high-value inputs compared with the current leading style include cutoff count, all-node/cut-node confidence, correction magnitude, TT score quality rather than only bound kind, threat context, and reduced-search result feedback. Rarog previously tested a do-deeper variant and rejected it (-1.38 Elo with roughly 4% more nodes, documented in the source); do not simply re-enable it. First improve eligibility/context, then test post-LMR depth adjustment as part of that coherent model.

## 4. History tables are rich, but learning coverage is sparse

**Evidence: strong.**

Rarog has main history, low-ply history, pawn history, capture history, continuation histories at offsets 1/2/4/6, killers and countermoves. The storage is not the weakness. The update policy is.

The main reward path is entered after a beta cutoff ([`search.rs:1498`](../src/search.rs#L1498)). Quiet cutoffs reward the best quiet and penalize earlier quiets/captures. Capture cutoffs reward the best capture and penalize only an earlier capture subset. In contrast:

- an exact/PV best move that raises alpha but does not fail high receives no comparable general reward;
- a TT lower-bound cutoff returns before history feedback ([`search.rs:998`](../src/search.rs#L998));
- fail-low nodes provide little contextual negative evidence;
- static-eval changes across a move are not learned as a separate history signal;
- post-LMR outcomes do not update a dedicated reduction-confidence history;
- reward and malus share the same symmetric `depth^2 + 2*depth`, capped at 1200 ([`move_ordering.rs:127`](../src/move_ordering.rs#L127)); the strongest engines tune these as separate functions;
- capture-cutoff cross-category maluses are asymmetric, so a winning capture does not teach that previously searched quiets/bad captures were poor in the same way as a winning quiet cutoff.

Context is also missing. Current public leaders use threats to distinguish, for example, a quiet move that saves an attacked minor piece from the same from/to move in a calm position. Rarog's main quiet history is `[side][from][to]`; pawn and continuation tables add useful context, but there is no attacked-from/attacked-to or threat-indexed history.

Recommended changes, in order:

1. Add feedback on TT cutoffs. Reward a legal TT move on a valid lower-bound cutoff; consider a bounded negative update on upper-bound fail-low evidence.
2. Reward quiet exact/PV best moves, with a smaller bonus than beta cutoffs.
3. Split bonus and malus curves. This is already listed in `PLAN.md` and should precede table proliferation.
4. Normalize cross-category maluses so quiet and capture cutoffs teach all previously searched alternatives deliberately.
5. Add one threat-context bit only after the update-coverage changes are measured.
6. Audit killers/countermoves. They are cheap, but a persistent unaged countermove can become stale while the numeric histories are halved each search.

Useful counters are updates per node by event type, saturation percentage per table, average absolute entry, TT-cutoff update count, exact-node update count, and ordering rank of the eventual best move.

## 5. Correction history can learn tactics as evaluator bias

**Evidence: strong.**

Rarog combines pawn, minor, own non-pawn, opponent non-pawn and continuation correction into corrected evaluation ([`search.rs:2230`](../src/search.rs#L2230)). This is modern and potentially valuable. Two semantic problems remain.

First, updates are not guarded against tactical best moves. A capture or other tactical beta cutoff can update correction ([`search.rs:1539`](../src/search.rs#L1539)); exact/fail-low end-of-node updates can do the same ([`search.rs:1582`](../src/search.rs#L1582)). Correction history should model systematic static-evaluation error in quiet positions, not memorize the search gain from a tactic. A capture best move, promotion, or in-check node is poor training data unless carefully filtered.

Second, `continuation_correction_history` has only `6 * 64 = 384` entries and is keyed solely by the previous move's piece and destination ([`search.rs:2243`](../src/search.rs#L2243)). It is not a continuation pair. Many completely different current positions and candidate moves collapse onto the same previous-move bucket. Leading engines use genuinely paired continuation-correction contexts, often at more than one previous-ply offset.

Recommended semantics:

- update only when static evaluation is valid and the node is quiet;
- reject or strongly down-weight updates when the best move is a capture, promotion, check or tactical refutation;
- respect bound direction: do not train from a lower/upper bound as though it were an exact search score;
- clamp by both depth and score difference, and track saturation;
- replace the 384-entry continuation term with a true `(previous piece,to) -> (current piece,to)` relation, initially at offsets 2 and 4 or whichever pair tests best;
- feed absolute correction magnitude into reduction/futility confidence: a position the evaluator often misjudges should be searched more cautiously.

This work should be tested after history update coverage, because correction and history both affect move ordering and pruning.

## 6. Root search, aspiration and time management discard information

**Evidence: strong.**

Rarog persists the legal root move list, but not a full root-move record. The search result primarily carries the selected best move, score, depth, PV and nodes. It does not retain, for every root move across iterations:

- current and previous score;
- average score and mean-square score/variance;
- complete PV;
- nodes or effort spent on that move;
- fail-low/fail-high count;
- stability age and last completed depth.

That information is useful even before SMP. It supports root reordering, dynamic aspiration width, time extensions for unstable evaluations, early termination when one move owns most effort and remains stable, and better fallback behavior after an interrupted iteration.

There is also a concrete time-manager ordering issue. `prev_avg_score` is updated from the newly completed score ([`search.rs:720`](../src/search.rs#L720)) before `falling_eval` uses it ([`search.rs:745`](../src/search.rs#L745)), although the comment says it feeds the next iteration. This attenuates the apparent score fall. Preserve `previous_average` for the current decision, then update the EWMA for the next iteration.

Recommended `RootMove` minimum:

```text
move, score, previous_score, average_score, mean_squared_score,
pv, nodes, seldepth, fail_highs, fail_lows, last_best_depth
```

This is not just presentation infrastructure. It is the substrate for stronger aspiration, time management and SMP.

## 7. Lazy SMP: diversity is limited, merge policy is already good

**Evidence: mixed.**

The Basilisk conclusion that final thread selection is weak does **not** transfer. Rarog already aggregates votes per best move, weights each result by relative score and completed depth, and uses a sensible decisive-score/depth/main-thread tie break ([`search.rs:2545`](../src/search.rs#L2545)). Keep this unless games show otherwise.

The diversity criticism does transfer. Helpers receive a root score offset, but the move picker emits a legal TT move before ordinary scored moves. Therefore score rotation usually affects the second move, while all threads still search the same TT root move first. Helpers also keep private history tables, so they duplicate early work without sharing newly learned ordering. Shared-TT writes are filtered for helpers (Exact depth 3, Lower 5, Upper 7; [`search.rs:2347`](../src/search.rs#L2347)), which reduces pollution but may also delay useful cross-thread information.

Recommended order:

1. Build persistent root-move state first.
2. Diversify the actual first root move for selected helpers, not only its score after TT precedence.
3. Use deterministic depth/thread schedules so results are reproducible enough to diagnose.
4. Measure overlap: percentage of root and depth-2 nodes visited by multiple threads, unique TT stores, helper cutoff contribution and speedup at 2/4/8 threads.
5. Only then tune helper write thresholds or limited history seeding/sharing.

Do not replace Rarog's voting with “deepest thread wins”; that would be a regression relative to its current merge logic.

## 8. Quiescence: most Basilisk defects do not transfer

**Evidence: do not transfer, except fail-soft details.**

Rarog's qsearch is materially stronger than the Basilisk implementation reviewed:

- in check it generates all legal evasions and detects mate;
- evasions are ordered through the normal scoring path;
- qsearch probes and stores TT entries;
- `MAX_QPLY = 16` is applied to non-check stand-pat search, not used to truncate a checked evasion chain;
- outside check it searches captures, matching current Stockfish's broad qsearch move class.

Therefore adding quiet qchecks should not be a default priority. It is not current public-engine consensus and can expand the tree substantially. Test it only as a narrow SEE/history-gated experiment.

The transferable issue is bound quality. Delta/SEE prunes and some terminal exits return `alpha` or skip a move without retaining a fail-soft estimate. Per-move quiet futility in the main search explicitly documents a plain skip ([`search.rs:1270`](../src/search.rs#L1270)). This weakens the information stored in the TT and can destabilize aspiration/re-search behavior. Stockfish's 2026 progression also records a gain from not storing stand-pat return values in TT, so Rarog's qsearch stand-pat TT stores deserve an isolated test; this is not enough evidence to remove them blindly.

## 9. ProbCut, null move and IIR

### ProbCut

**Evidence: medium.**

Rarog searches at most eight SEE-nonnegative captures at `beta + margin`, first in qsearch and then at reduced depth ([`search.rs:1135`](../src/search.rs#L1135)). Compared with current leading practice, missing context includes a TT veto when an adequate upper bound already argues against ProbCut, richer capture-history/SEE thresholding, and tighter node-type control.

Rarog's earlier “ProbCut port” experiment lost about 24.5 Elo according to `PLAN.md`. That is a warning against copying a current Stockfish formula into a different evaluator/search. The worthwhile experiment is not “more ProbCut”; it is measuring false positives by TT quality, correction magnitude, capture history and reduced-search revalidation.

### Null move

**Evidence: medium-strong.**

At depth at least 10, Rarog verifies a null cutoff with `allow_null = false` at the same node ([`search.rs:1106`](../src/search.rs#L1106)). The recursive move searches below that verification call pass `allow_null = true` again. Thus null move is suppressed only at the verification root, not throughout a minimum-ply verification region. That is weaker zugzwang protection than the modern `nmpMinPly` pattern.

Introduce a stack/thread field such as `nmp_min_ply` or an explicit verification mode. Suppress null move throughout the verification subtree until the threshold is crossed. Test endgames and low-material positions separately; node cost alone can hide a correctness-strength tradeoff.

### IIR

**Evidence: medium.**

Rarog reduces depth whenever depth is at least 4 and the TT move is absent, or a non-PV TT entry is more than three plies shallow ([`search.rs:1019`](../src/search.rs#L1019)). This also reduces a PV node with no TT move. A modern node-type-aware policy can distinguish PV, cut/all expectation, TT-bound quality and whether a singular search is in progress. Instrument how often IIR fires at PV nodes and whether the resulting first move later fails high before making it more aggressive.

## 10. Repetition, graph history and a verified rule-50 defect

### Upcoming repetition

**Evidence: medium.**

`Board::is_repetition` scans same-side hashes already present in history and Rarog returns an exact zero for a repeated search position ([`board.rs:1620`](../src/board/board.rs#L1620)). It does not detect an *upcoming* repetition: a legal move that closes a cycle whose matching position is in the game/search path. Strong engines use cuckoo-style reversible-move lookup or equivalent graph-history logic to recognize these cycles earlier and handle alpha-dependent draw values.

This is likely smaller than LMR/history/root work, but it matters in perpetuals and endgames. Add it after the P1 selectivity changes, with repetition-specific tests and contempt/draw-value semantics made explicit.

### Verified: checkmate at halfmove 100 is scored as draw

The exact Basilisk reproduction also fails in Rarog:

```text
position fen 7k/5K2/6Q1/8/8/8/8/8 w - - 99 50
go depth 4
```

Observed with `target/release/rarog.exe` built from the analyzed revision:

```text
info depth 1 ... score cp 0 ... pv g6h5
info depth 4 ... score cp 0 ... pv g6h5
bestmove g6h5
```

The mate-in-one is missed at every depth. At a non-root child, `negamax` calls `can_declare_draw_in_search()` before computing `in_check` or generating legal moves ([`search.rs:954`](../src/search.rs#L954)); qsearch does the same ([`search.rs:1620`](../src/search.rs#L1620)). A quiet mating move increments the halfmove clock from 99 to 100, so the child returns draw before recognizing checkmate.

Required correction:

- terminal no-legal-move handling must take precedence over a claimable 50-move draw when the side to move is checkmated;
- keep a fast rule-50 path for non-check positions if desired;
- in check at `halfmove_clock >= 100`, verify whether at least one legal evasion exists before returning draw;
- apply the same ordering to negamax and qsearch;
- add a regression at depths 1 and greater, plus a stalemate-at-100 and legal-evasion-at-100 case.

This is P0 because it is a rules/search correctness defect, even though it is rare and its Elo effect is probably small.

## 11. TT semantics and storage

**Evidence: mostly do not transfer from Basilisk.**

Rarog already stores exact/lower/upper bounds, static eval, move, depth, age and a persistent PV bit ([`tt.rs:31`](../src/tt.rs#L31)). The local TT packs three 10-byte entries into a 32-byte aligned cluster ([`tt.rs:63`](../src/tt.rs#L63)), so Basilisk's low local entry-density criticism does not apply.

The shared atomic TT uses three larger atomic entries per 64-byte-scale cluster, so multi-thread memory density and bandwidth are less favorable than local mode. That is an optimization issue, not the main search-design gap. More important semantic issues are:

- overuse of the PV bit as a pruning veto (section 2);
- TT early cutoffs do not feed history (section 4);
- rule-50 mate ordering is unsafe (section 10);
- `score_from_tt` adjusts mate usability against the halfmove clock, but Rarog lacks the more complete graph/rule-50 cutoff safeguards used by leading engines near 100 plies;
- helper write filtering may trade too much timely information for lower pollution and needs scaling data, not intuition.

Do not redesign the TT layout before fixing the semantic consumers. A denser table cannot compensate for a policy that turns stale TT-PV ancestry into a global no-prune state.

## 12. Search regression methodology

**Evidence: Basilisk finding does not transfer as stated.**

Rarog's `PLAN.md` explicitly says SPRT is the only strength verdict and uses bench identity only for behavior-preserving/inert work. That is the correct separation. `tests/search_strength.rs` contains a few fixed-depth best-move assertions, but they cover mate, a hanging queen, a known tactical line, repeatability and thread-count handling; they are not presented as an Elo gate.

Still, structural search work needs more observability than a total node fingerprint. Add a stable diagnostics mode with per-feature counters and a JSON/CSV output option. For every candidate, compare:

| Dimension | Minimum measurement |
|---|---|
| Correctness | unit tests, mate/stalemate/rule-50/repetition cases, legal PV |
| Tree shape | total nodes, EBF, qnodes, check nodes, prune counts, LMR counts and re-searches |
| Ordering | TT-first success, best-move rank, quiet/capture history distributions |
| Bounds | fail-high/fail-low counts, aspiration retries, TT bound mix |
| SMP | overlap, unique work, NPS and Elo at 1/2/4/8 threads |
| Strength | paired SPRT at primary TC, then a longer-TC and boundary/endgame rider for risky changes |

Bench identity should continue to gate only supposedly behavior-preserving changes. For an intentional search change, changed nodes are expected.

## Where the leading engines are structurally better

The following is the answer to “apart from more optimization of constants”:

| Area | Rarog today | Leading public-engine pattern |
|---|---|---|
| Tactical selectivity | Hard exemptions for check/in-check/promotion/good capture | Most moves are reducible; context determines protection |
| Node confidence | `is_pv`, `tt_pv`, `in_check`, `improving` mostly act as booleans | Multiple weak signals combine into depth/reduction/margin decisions |
| History learning | Rich tables, cutoff-centric updates | Feedback from cutoffs, exact best moves, TT cutoffs, fail-lows, eval changes and LMR outcomes |
| Threat handling | Checks protected categorically; no threat-indexed quiet history | SEE-qualified forcing bonuses and threatened-from/to context |
| Correction | Several material/hash contexts, but tactical pollution | Quiet/bound-correct updates, true continuation pairs, correction magnitude feeds selectivity |
| Root model | Best result plus root move list | Persistent score/PV/variance/effort per root move |
| Time management | Best-move/score stability with attenuated previous-average signal | Root variance, effort distribution, fail history and score trend |
| SMP | Shared TT, private histories, score rotation behind TT move | Deliberate root/depth diversity and persistent root-state cooperation |
| Graph history | Already-seen repetition only | Upcoming repetition/cycle detection and rule-50-aware TT safety |
| Post-search feedback | Reduced search either re-searches at fixed full depth or not | Search result can adjust subsequent depth and train confidence histories |

## What not to prioritize

| Idea | Reason |
|---|---|
| Retune all LMR/RFP/futility constants first | It optimizes the current categorical structure and is unlikely to close the architectural gap |
| Add quiet qsearch checks broadly | Current Stockfish also uses captures/evasions; tree cost is high and evidence is weak |
| Redesign local TT density | Rarog's local 3-entry/32-byte layout is already reasonable |
| Add a TT PV bit | It already exists; the problem is its over-broad use |
| Replace SMP selection with deepest/best-score thread | Rarog's existing score/depth voting is better than that |
| Re-enable previously failed patches unchanged | ProbCut port, no-aging history, double-extension cap, do-deeper and broad LMR retunes already failed; new tests need a new mechanism |
| Treat EBF/NPS as a strength verdict | They explain cost, not move quality; SPRT remains decisive |

## Recommended implementation roadmap

### Phase 0 - correctness and counters

1. Fix rule-50/checkmate ordering in both negamax and qsearch.
2. Add regression tests for mate/stalemate/legal evasion at halfmove 100.
3. Add search counters for checks, TT-PV gating, prune attempts/successes, LMR eligibility/re-searches, history update event, correction updates and SMP overlap.
4. Correct the `prev_avg_score` update ordering or cover it with a dedicated time-manager experiment.

### Phase 1 - remove compound tactical overprotection

1. Remove unconditional in-check extension.
2. Permit reduced late evasions with a conservative discount.
3. Permit reduction of weak late checks using SEE/history.
4. Narrow checking-move pruning exemptions.

Run these as separate SPRTs. The combined patch has the largest theoretical gain but would be impossible to diagnose if tactical strength regressed.

### Phase 2 - TT-PV and unified selectivity

1. Split `is_pv` from stored TT-PV in pruning gates.
2. Convert stored TT-PV into margin/reduction adjustments.
3. Compute a prospective reduced depth shared by LMR, LMP, futility and SEE decisions.
4. Resolve the `lmr_shallow_tt` naming/condition mismatch with a controlled test.
5. Add correction magnitude and TT score quality as confidence inputs.

### Phase 3 - information retention and learning

1. Add TT-cutoff and exact-best-move history feedback.
2. Split history bonus and malus functions.
3. Make cross-category penalties explicit and symmetric by intent.
4. Guard correction updates against tactical best moves and invalid bound direction.
5. Replace shallow continuation correction with true move-pair contexts.
6. Add one threat-aware history dimension if earlier steps succeed.

### Phase 4 - root, time and SMP

1. Introduce persistent `RootMove` records.
2. Use root variance/effort/stability in aspiration and time management.
3. Make helper first-move diversity override root TT precedence deliberately.
4. Measure and tune helper TT write policy at 2/4/8 threads.

### Phase 5 - graph/tactical refinements

1. Add upcoming repetition detection.
2. Add subtree-wide null-move verification suppression.
3. Revisit ProbCut with TT/correction/history context.
4. Make IIR node-type aware.
5. Revisit fail-soft bound shaping and qsearch stand-pat TT storage.

## Elo hypotheses and risk

These ranges are prioritization estimates, not additive predictions.

| Work package | Expected upside | Risk | Main failure mode |
|---|---:|---:|---|
| Check/in-check selectivity redesign | High | High | Tactical blindness if exemptions are removed too aggressively |
| Narrow TT-PV gating | Medium-high | Medium | Pruning a genuinely unstable former-PV node |
| History update coverage | Medium-high | Medium | Feedback loops/saturation from over-rewarding TT moves |
| Persistent root state + time use | Medium | Low-medium | More complexity without benefit at fixed-depth testing |
| Correction semantic cleanup | Medium | Medium | Removing useful accidental tactical bias before replacement context exists |
| Unified prospective LMR depth | High | High | Large interacting tree change; difficult attribution |
| SMP diversity | Medium at multi-thread | Medium | TT pollution or duplicated re-searches |
| Null verification region | Low-medium | Low-medium | Extra nodes in positions where current null cutoff is safe |
| Upcoming repetition | Low-medium | Medium | Incorrect draw semantics/false cycle matches |
| Rule-50 mate fix | Correctness, small Elo | Low | Extra legal-evasion generation near the threshold |

## NNUE interaction

Rarog's evaluator is still a major part of the total gap to Stockfish-class engines. Search and evaluation are not separable: stronger NNUE changes static-eval reliability, correction distributions, improving detection, null-move confidence, futility safety and the value of deeper tactical verification. Therefore:

- do not copy Stockfish margins into the current HCE and expect transfer;
- prefer structural changes whose semantics remain valid across evaluators;
- record correction magnitude, eval volatility and pruning error so the same search can be recalibrated after NNUE work;
- rerun the best structural candidates when the evaluator changes materially.

## Immediate candidate sequence

If work starts now, the most informative order is:

1. Rule-50/checkmate fix and diagnostics.
2. Remove blanket in-check extension only.
3. Reduce late evasions only.
4. Replace `tt_pv` hard gates with conservative margin adjustments.
5. Add history feedback for exact best moves and TT cutoffs.
6. Add persistent root-move records and fix previous-average timing.
7. Guard correction updates against tactical best moves.
8. Generalize LMR eligibility and shared prospective depth.
9. Make helper first-root-move diversity real.
10. Add null-verification region and upcoming repetition.

This sequence first fixes correctness, then attacks the largest source of unnecessary nodes, then improves the learning and root information needed to make later selectivity changes safe.

## Open questions for continued analysis

- What percentage of non-PV nodes are protected solely by a stored TT-PV bit?
- How much of bench EBF is attributable to checked nodes and checking branches?
- What is the LMR re-search rate by quiet/check/capture/history bucket?
- How often does the current `lmr_shallow_tt` condition fire with a genuinely shallow entry versus a deep entry?
- What fraction of correction updates have a tactical best move?
- How often does a TT cutoff occur without any corresponding history update?
- At 2/4/8 threads, how often do all workers search the same first and second root move?
- Does helper TT filtering improve game strength or only reduce hash churn?
- How much time-manager behavior changes when `falling_eval` uses the truly previous average?
- Which failed historical search experiments become materially different after the eligibility/history/root redesign?

## Source baseline

- Rarog: local `development` revision `ff21dc1`, analyzed directly.
- Basilisk: local [`search_analysis.md`](../../basilisk/analysis/archive/search_analysis.md), used as a checklist; findings were not copied when Rarog already handles them.
- Stockfish: public master snapshot inspected 2026-07-13; [official repository](https://github.com/official-stockfish/Stockfish).
- Reckless: public master snapshot inspected 2026-07-13; [repository](https://github.com/codedeliveryservice/Reckless).
- PlentyChess: public master snapshot inspected 2026-07-13; [repository](https://github.com/Yoshie2000/PlentyChess).
- Obsidian: public source snapshot inspected for a secondary implementation comparison; [repository](https://github.com/gab8192/Obsidian).

