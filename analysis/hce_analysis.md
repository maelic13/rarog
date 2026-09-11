# Rarog HCE analysis

> **Historical audit.** This report describes revision `ff21dc1` on
> 2026-07-13. Its four concrete activation defects (`attacked2`, enemy rook
> behind a passer, unstoppable passers, and phalanx detection) were fixed by
> `d5a6054` and are covered by current evaluator tests. Do not use their old
> priority or Elo estimate as current work. The current maturity disposition
> and pre-NNUE programme are in
> [`hce_maturity_2026-08-25.md`](hce_maturity_2026-08-25.md).

Status: working analysis, intended to be extended  
Analysis date: 2026-07-13  
Rarog revision: `ff21dc1` (`development`)  
Comparison baseline: Stockfish 18/current SFNNv13 development, PlentyChess 7/current development, other mid-2026 top engines where their implementation is public, Stockfish 11 as a mature classical reference, and Basilisk's HCE audit as a sibling-engine checklist

## Executive summary

Rarog's handcrafted evaluator is not primitive. It is a broad, approximately 1,200-parameter, tapered evaluator with material and PSTs, cached pawn structure, per-count mobility, a nonlinear king-danger table, attacker/victim threat tables, quadratic material imbalance, passer detail, several specialized endgames, rule-50 damping, and search correction history. The Phase-4 program produced a large measured gain, and the current evaluator is reasonably described as having a **Stockfish-11-class feature list**.

It is not, however, Stockfish-11-equivalent in the conditional semantics of those features, and a complete list of feature names is not the same as a complete evaluation function. Rarog's remaining evaluation loss comes from four different sources:

| Source of loss | Assessment |
|---|---|
| Incorrect feature activation | At least four concrete semantic defects are present; two are shared with Basilisk |
| Over-broad additive features | Many terms ask whether a pattern exists, not whether it is safe, usable, relevant, or convertible |
| Phase/endgame specialization | One phase scalar and a small endgame dispatcher cannot distinguish many materially different position classes |
| Representation and supervision ceiling | Current top engines learn king-, threat-, pawn- and material-conditioned interactions from vastly more densely labelled data |

The highest-confidence implementation findings are:

1. `attacked2` loses squares attacked by two pawns, although hanging, restricted-square, and king-safety code consumes it;
2. the enemy-rook-behind-passer term is nested under the friendly-rook loop, so it normally fails to activate and can double-count;
3. the "unstoppable passer" rule tests only the defending king and ignores every defending non-king piece;
4. the parameter documented as connected/phalanx recognizes pawn support, but not a same-rank phalanx; and
5. release search can serve a truncated lazy evaluation while Texel always trains and verifies the full evaluator.

There are also high-confidence misspecifications rather than binary bugs:

- opposite-coloured-bishop scaling is capped correctly, unlike Basilisk, but activates in positions containing arbitrary additional majors and minors;
- `queen_infiltration` means only "advanced and not pawn-attacked", so a queen attacked by a minor, rook, queen, or king is still rewarded;
- the fitted bishop-pair pawn correction has the opposite sign from its comment;
- whole-path passer coefficients are negative or nearly zero, suggesting aliasing or a feature whose broad activation mixes good and bad cases;
- king-safety fitting has left weak-ring, flank pressure, storms, missing shelter, and shelter-danger interaction at zero; and
- the initiative model always moves a nonzero endgame score farther from zero based on a very coarse complexity proxy.

The central conclusion is that Rarog is now losing much less from unoptimized constants than from **missing conditionality**. The evaluator sees "queen advanced", "safe pawn push", "rook behind passer", "space square", and "king attacker" as mostly separable facts. The best current evaluators represent exact piece-square, king-region, attacker-victim, pawn-pair, material-density, and search-context interactions. Constant tuning cannot recover information that the feature representation discarded.

There is still worthwhile HCE work. A disciplined correctness and conditional-feature campaign plausibly contains tens of Elo. Recovering the full top-engine gap with more additive HCE terms is unlikely. A competitive, king-conditioned and threat-aware NNUE remains the main route to the top cluster.

## Evidence classifications

This report separates implementation facts from strength estimates:

| Classification | Meaning |
|---|---|
| Verified defect | Source behavior contradicts the feature's stated or necessary semantics |
| Verified implementation property | Source behavior is clear, but may be an intentional speed/accuracy tradeoff |
| Measured behavior | A result recorded by the project, without assuming one causal mechanism |
| Architectural gap | A concrete representational difference from stronger evaluators |
| Strength hypothesis | A plausible Elo opportunity requiring controlled testing |

Elo ranges below are non-additive. Evaluation features overlap with one another and change search behavior, pruning reliability, correction history, and nodes per second.

## Scope

The audit covers:

- the main evaluator and parameters in [`src/eval.rs`](../src/eval.rs);
- evaluator use and correction history in [`src/search.rs`](../src/search.rs);
- linear and nonlinear fitting in [`tools/texel-tuner/src/main.rs`](../tools/texel-tuner/src/main.rs);
- self-play extraction in [`tools/texel/extract.py`](../tools/texel/extract.py);
- self-play adjudication in [`tools/datagen.ps1`](../tools/datagen.ps1);
- evaluator and endgame tests under [`tests/`](../tests/);
- the recorded experiment history and forward plan in [`PLAN.md`](../PLAN.md); and
- the sibling audit at `D:\code\basilisk\analysis\hce_analysis.md`, checked item by item rather than copied.

The focused existing suites passed during this audit:

```text
cargo test --release --test eval_invariants --test endgames --test eval_cache
```

Those tests establish useful invariants and current expected behavior. They do not cover several feature semantics identified below.

## Rarog's current evaluation pipeline

At a high level, [`Evaluator::evaluate()`](../src/eval.rs#L1027) computes:

```text
material + global PST
    + pawn structure and rank passers
    + immediate passer stop-square detail
    -> optional lazy exit
    + attack maps / attacked2
    + mobility
    + threats / hanging / restricted squares
    + nonlinear king danger and linear shelter/storm
    + rook/passer, blockade, x-ray, space, closedness and small terms
    + material imbalance
    + mop-up, tempo and initiative
    -> one MG/EG phase interpolation
    -> specialized endgame / OCB scaling
    -> rule-50 damping
    -> side-to-move score
```

This has real strengths:

- per-square material/PST tracing and a reconstruction verifier;
- separate per-count mobility tables rather than one mobility coefficient;
- per-attacker/per-victim threat tables and refined hanging detection;
- a nonlinear 40-entry king-safety conversion table;
- quadratic material imbalance;
- exact KPK, KBNK drive, wrong-bishop rook-pawn, KQKP fortress, conservative KRKP, KNNK and insufficient-material knowledge;
- a rule-50 curve matching the modern Stockfish-style `/199` shape rather than Basilisk's overly aggressive `/100` damping;
- by-game train/holdout splitting, FEN deduplication, phase reporting/balancing, and a true-quiet capture filter; and
- several correction-history contexts in search.

The audit therefore does not recommend replacing the HCE with a simpler one. It recommends repairing its semantics, measuring its residuals, and treating it as the datagen/search baseline for the eventual NNUE.

## Mid-2026 comparison baseline

The latest published CCRL 40/15 snapshot available for this audit is dated 2026-02-28. It lists Stockfish 18 at 3651, PlentyChess 7 at 3644, Torch 4 at 3638, Obsidian 16 at 3636, and Reckless 0.8 at 3634 on the four-CPU list. Rarog's project plan estimates roughly 3000 CCRL after the Phase-4 transfer. Those figures suggest a public-scale difference closer to 600 Elo than "a couple hundred", but they are not a controlled head-to-head measurement: versions, hardware, books, tablebases, sample sizes, and rating-pool anchoring differ. See the [CCRL 40/15 complete list](https://computerchess.org.uk/ccrl/4040/rating_list_all.html).

The useful comparison is architectural:

| Engine/reference | Evaluator capability relevant to Rarog | Public source |
|---|---|---|
| Stockfish 18/current | Threat-input NNUE; current SFNNv13 inherits compressed `FullThreats + HalfKAv2_hm`, a 1024-unit transformer and 32-unit second layer; direct PSQT and eight piece-count-selected outputs/layer stacks | [Stockfish 18 release](https://stockfishchess.org/blog/2026/stockfish-18/), [official NNUE architecture documentation](https://github.com/official-stockfish/nnue-pytorch/blob/master/docs/nnue.md) |
| PlentyChess | Threat-input NNUE trained on 15+ billion self-generated positions with partial self-distillation | [official repository](https://github.com/Yoshie2000/PlentyChess) |
| Stockfish 11 | Mature classical reference for conditioned space, king danger, passers, initiative/complexity and material-specific scaling | [classical evaluator](https://github.com/official-stockfish/Stockfish/blob/sf_11/src/evaluate.cpp) |
| Sirius 9 | Strong modern HCE reference with broad conditional threats, nonlinear king safety, complexity, scaling and specialized endgames | [official repository](https://github.com/mcthouacbb/Sirius) |

Torch is closed-source, so this report makes no internal claims about it. Ratings are whole-engine results, not evaluator ablations.

Stockfish 18 itself is a useful warning against attributing all strength to one network: its release reports up to 46 Elo over Stockfish 17 from a combined evaluation, correction-history, training, search, and hardware program. It also reports an automated training workflow able to consume more than 100 billion Lc0-evaluated positions. Rarog is competing against a better function class, denser supervision, much larger data, stronger residual correction, and co-adapted search.

## Basilisk finding cross-map

The sibling audit is highly relevant because the evaluators share much of their conceptual ancestry. It is not safe to assume that every Basilisk defect exists in Rarog. The following table records the actual mapping.

| Basilisk finding | Rarog applicability | Rarog evidence / distinction |
|---|---|---|
| OCB scaling can amplify | **Does not apply exactly** | Rarog caps `32 + 4*pawns + 4*passers` at 48, so it cannot amplify. Its activation scope is still over-broad; see section 5 |
| Enemy rook behind passer nested under friendly rook | **Applies exactly** | [`eval_rooks_behind_passers()`](../src/eval.rs#L2220) has the same dependency and duplicate-count risk |
| `attacked2` omits two-pawn overlap | **Applies exactly** | Pawn attacks are inserted as one union at [`eval.rs:1416`](../src/eval.rs#L1416), before overlap is accumulated |
| Winnability block is all zero and not tunable | **Does not apply exactly** | Rarog has a traced, fitted initiative weight of 2. The model is much shallower than a real winnability model and always increases magnitude |
| Lazy eval differs between tuner and release | **Applies exactly** | Texel forces `lazy = false`; production skips the positional block above the configured margin |
| Sparse endgame scaling | **Applies, but Rarog is stronger** | Rarog adds KPK, KQKP and KRKP handling and correct `/199` rule-50 damping, but still lacks broad material-signature specialization |
| Under-modelled / poorly identified king safety | **Applies strongly** | Rarog's nonlinear path is better and restores best holdout state, but many danger inputs and all storm/missing-shelter inputs are zero |
| Passer and pawn interactions too static | **Applies strongly** | Whole-path additions remain broad; unstoppable semantics are wrong; connected does not include phalanx |
| Threat aggregation loses exact context | **Applies strongly** | Rarog has richer attacker/victim tables than a flat HCE, but still reduces geometry to counts and piece types |
| Broad positional terms | **Applies strongly** | Space, bad bishop, queen infiltration, rook/queen alignment, trapped bishops and outposts retain broad activations |
| One MG/EG scalar is insufficient | **Applies exactly** | Every tapered term shares `phase` based on remaining material |
| Correction history is under-expressive | **Applies partially** | Rarog has fixed source scaling and only a previous-piece/destination continuation context; tactical updates are not filtered |
| Self-play data can preserve misconceptions | **Applies partially** | Rarog fixed by-game splitting and true-quiet filtering, and supports blended labels; self-referential adjudication and holdout reuse remain |
| Semantic test coverage is thin | **Applies** | Rarog has better symmetry/endgame/cache tests, but lacks the direct counterexamples identified here |
| Plain 768-input NNUE is only a baseline | **Strategically applies** | Rarog's current Phase-9 plan wisely avoids locking an exact final shape; king, threat and material conditioning should be explicit experiments |

## 1. Verified: pawn-pawn double attacks are missing from `attacked2`

The attack substrate starts at [`eval.rs:1413`](../src/eval.rs#L1413). For each colour, it does:

```rust
attacked_by[ci][Piece::Pawn as usize] = pawn_attacks[ci];
attacked[ci] |= pawn_attacks[ci];
```

The pawn bitboard is already the union of attacks from every pawn. If two pawns attack the same square, bitboard union represents the square once. `attacked2` is still empty, so their overlap is irretrievably lost. King and piece attacks subsequently update `attacked2` through `attacked & new_attacks`, but no code reconstructs overlap between the two pawn attack directions.

This is not a harmless comment mismatch. `attacked2` feeds:

- refined hanging classification at [`eval.rs:1675`](../src/eval.rs#L1675);
- strong protection for restricted squares at [`eval.rs:1736`](../src/eval.rs#L1736);
- weak king-ring classification at [`eval.rs:2058`](../src/eval.rs#L2058); and
- therefore the nonlinear king-danger bucket.

Consequences include:

- a target defended by two pawns can be considered defended only once;
- a target attacked by two pawns can fail the doubly-attacked branch;
- a square protected twice by pawns can be classified as restricted; and
- king-zone pressure from converging pawns is understated.

The correct construction needs the two directional pawn attack sets separately:

```text
left  = attacks from one pawn-capture direction
right = attacks from the other pawn-capture direction
attacked2 |= left & right
attacked2 |= attacked & (left | right)
attacked  |= left | right
```

Required tests:

1. two white pawns converging on one empty square put it in both `attacked` and `attacked2`;
2. mirror the position for Black;
3. place a victim on the square and verify refined hanging behavior;
4. use the square in a king zone and verify the danger input; and
5. verify a single pawn attack remains absent from `attacked2`.

This should be fixed before another global fit, because it changes the feature substrate consumed by several fitted families.

## 2. Verified: enemy rook behind a passer is conditioned on a friendly rook

[`eval_rooks_behind_passers()`](../src/eval.rs#L2220) loops over friendly rooks. It selects the friendly rook's file, finds friendly passers on that file, rewards the friendly rook if it is behind the passer, and then checks enemy rooks on the same file inside the same iteration.

The intended enemy-rook penalty therefore fires only if all three are true:

1. a friendly passer exists on the file;
2. an enemy rook is behind it; and
3. a friendly rook also happens to be on that file.

Condition 3 has nothing to do with whether the enemy rook blockades or attacks the passer. With no friendly rook on the file, the feature is invisible. With two friendly rooks on the file, the same enemy rook can be penalized twice.

The fitted values make the issue more revealing:

| Term | MG | EG |
|---|---:|---:|
| Friendly rook behind passer | 0 | 75 |
| Enemy rook behind passer | 13 | 0 |

The enemy endgame weight being zero is not reliable evidence that an enemy rook behind a passer has no endgame value: most natural activations never reached the trace. The tuner optimized the semantics of the buggy feature.

The loops should be independent. In addition, both own- and enemy-rook versions should decide whether a piece between rook and passer invalidates or weakens the feature. Current code checks rank ordering only, so a geometrically rearward but completely blocked rook receives the same activation as an active rook.

Required FEN tests:

- passer plus enemy rook behind, no friendly rook;
- friendly and enemy rook both behind;
- two friendly rooks on the file, ensuring no duplicate enemy penalty;
- an intervening blocker;
- rook in front of the passer; and
- colour-mirrored cases.

## 3. Verified: the "unstoppable passer" ignores defending pieces

The rule at [`eval.rs:1799`](../src/eval.rs#L1799) requires an empty path to promotion and compares only the enemy king's Chebyshev distance with the pawn's nominal move count. It never asks whether the defender owns a rook, queen, bishop, or knight that can stop the pawn.

That is the rule of the square only for pawn-versus-king geometry. In a general position, a distant rook on the back rank, a bishop controlling the promotion diagonal, a knight reaching the block square, or a queen checking from behind can make the passer completely stoppable while the feature awards `51` EG centipawns.

The term also misses important tempo details:

- promotion with check;
- side-to-move changes after a double pawn push;
- whether the own king can protect the advance;
- whether the defender can sacrifice a piece for the pawn;
- whether a rook can establish a rear attack after the pawn moves; and
- whether an apparently empty path opens a slider line.

Conservative repairs, in increasing order of complexity:

1. activate the current rule only when the defender has no non-king material;
2. replace the binary bonus with a material- and blocker-conditioned race term;
3. evaluate a short legal push sequence with attack-map updates; or
4. leave exact race valuation to an NNUE and retain only high-confidence HCE cases.

The first option is low risk and restores the name's semantics. More ambitious versions need a counterfactual/teacher-labelled passer suite rather than hand constants.

## 4. Verified: "connected/phalanx" recognizes only pawn support

The parameter declaration calls the rank table `connected/phalanx` at [`eval.rs:288`](../src/eval.rs#L288), while activation at [`eval.rs:1262`](../src/eval.rs#L1262) checks whether another friendly pawn attacks the pawn's square. That recognizes a pawn chain/support relation. It does not recognize adjacent friendly pawns on the same rank, which is the usual phalanx relation.

This matters because supported and phalanx pawns have different properties:

- support is already present but may be pinned or tactically removable;
- a phalanx can advance as a pair and create levers, but neither pawn currently defends the other;
- connected passers are much more valuable when both can advance safely; and
- rank dependence differs between a rear-supported pawn and a same-rank pair.

The fitted rank table is irregular, including a large MG value on relative rank 6. Combining two different intended concepts under one name while activating only one makes it difficult to interpret that shape.

Recommended structure:

```text
supported pawn: attacked by friendly pawn
phalanx pawn: friendly pawn on adjacent file, same rank
connected passer: passed pawn satisfying either relation
lever-capable pair: optional conditional term based on stop-square occupancy/attacks
```

Trace and fit these separately. A simple rename would fix documentation but not recover the missing phalanx information.

## 5. OCB scaling does not amplify, but its material scope is over-broad

This is an important difference from Basilisk. Rarog computes at [`eval.rs:2831`](../src/eval.rs#L2831):

```text
scale = min(32 + 4 * total_pawns + 4 * passers, 48)
score = score * scale / 48
```

The cap guarantees that OCB handling cannot increase `abs(score)`. Existing tests also cover relaxation by passed pawns. Basilisk's amplification defect therefore does **not** apply.

The activation test requires exactly one bishop per side on opposite colours, but it does not require bishops-only non-pawn material. Queens, rooks, and knights may remain. A queen-and-rook position with one opposite-coloured bishop each is not strategically equivalent to a pure OCB ending, yet it receives the same generic draw scaler.

This likely over-draws attacking positions where opposite bishops increase rather than reduce king danger. The safer hierarchy is:

1. very strong draw scaling for pure opposite-bishop endings;
2. milder scaling when additional minors remain;
3. little or no generic scaling with queens or multiple rooks; and
4. a factor based on the strong side's pawns/passers and the actual score sign, not only total pawn count.

Required tests should assert both non-amplification and scope:

- pure OCB with no passers;
- pure OCB with one or more passers;
- OCB plus queens;
- OCB plus rooks;
- same-coloured bishops; and
- sign preservation.

## 6. Lazy evaluation is a measured win with a train/serve mismatch

The production evaluator computes a cheap material/PST/pawn/pass-stop score and skips the complete activity/imbalance block when its absolute value exceeds `lazy_margin`, currently 600 cp. Under the `texel` feature, [`eval.rs:1104`](../src/eval.rs#L1104) forces `lazy = false`.

Production can therefore consume:

```text
cheap material + PST + pawn evaluation + mop-up + tempo + scaling
```

while fitting and trace verification always optimize:

```text
the complete evaluator, including mobility, threats, king safety,
space, passer relations, small terms and material imbalance
```

The project records lazy evaluation as a positive SPRT result, so it should not be removed on principle. The issue is that the original safety assertion in the comment — that no omitted positional term can flip a 600-cp margin — is not established after the evaluator grew. The omitted king-safety table alone reaches roughly 350 cp per side before interactions with mobility, threats, imbalance, passer detail, and OCB scaling.

The highest-risk cohort is not ordinary clean material advantage. It is materially unbalanced positions with exposed kings, trapped queens/rooks, promotion races, fortress geometry, or compensation. Those are precisely positions where cheap material and full positional evaluation can disagree sharply.

Recommended measurements:

1. add a diagnostic mode that computes both paths without changing the served result;
2. record `cheap`, `full`, delta, material signature, king-danger bucket and search outcome;
3. report sign flips and threshold crossings by cohort;
4. train/validate the function actually served, or prove the lazy/full residual is harmless above the threshold;
5. test a margin dependent on non-pawn material, king danger, or cheap/full uncertainty; and
6. reconsider lazy HCE after incremental NNUE changes the cost balance.

## 7. King safety: good framework, weak conditional signal

Rarog's king-safety framework is better than a flat attacker count. [`eval_king_safety()`](../src/eval.rs#L2018) combines attacker units, weak ring squares, safe checks by piece type, flank pressure, pawnless flank, queen relief, an optional shelter/storm input, and a nonlinear 40-entry safety table.

The fitted state nevertheless shows poor identification or insufficient semantics:

| Input | Current value / shape | Interpretation |
|---|---:|---|
| Minor / rook / queen attacker units | 2 / 2 / 5 | Nonzero, but only records whether an attack intersects the broad zone |
| Weak ring | 0 | No marginal value after the current substrate; also affected by the pawn `attacked2` defect |
| Safe checks N/B/R/Q | 4 / 6 / 4 / 16 | Useful signal, but "safe" means not attacked by the defender and not occupied by attacker material; pins and tactical legality are absent |
| Flank attack | 0 | Broad attack-minus-defense count did not survive fitting |
| Pawnless flank | 12 | Active, but not conditioned on attacker material beyond the common danger index |
| Missing file/adjacent shelter | 0 / 0 | Entire missing-shelter branch inert |
| Storm file/adjacent | 0 / 0 | Linear storm model inert |
| Shelter/storm danger interaction | 0 | The intended nonlinear exposure-by-pressure interaction remains inert |
| Safety table | long plateaus, 51 at index 0 | Large constant/plateau regions indicate bucket aliasing and correlated inputs |

The nonlinear king-safety tuner is a genuine re-evaluation path and, unlike the Basilisk audit's older issue, Rarog restores the best holdout vector. The remaining limitation is not simply a bad optimizer. The danger index compresses many qualitatively different attacks into one integer before the lookup. Once compressed, the table cannot distinguish:

- three minor attacks with no entry square from a queen/rook battery;
- a nominal safe check by a pinned attacker from a legal forcing check;
- an exposed king with no attacking queen from one with queen and rook coordination;
- weak ring squares with or without controlled escape squares;
- blocked versus unblocked pawn storms;
- castling to a safer wing versus the king's current shelter; and
- a checking square that can actually be occupied from one merely attacked geometrically.

The zero shelter/storm interaction after the Phase-6.2 refit is especially informative. It may mean the term is redundant, poorly scaled, weakly supported, or misspecified. It does not establish that shelter and storm have no value. A global coordinate fit can drive a broad feature to zero when good and bad activations cancel.

Recommended HCE experiment:

- fix `attacked2` first;
- instrument activation counts and danger histograms by queen presence and phase;
- distinguish legal/safe check access from attacked checking squares;
- separate blocked, unblocked and lever-supported storms;
- compare current shelter with reachable castling shelters when rights remain;
- add defender overload/pin inputs only where cheaply and correctly available;
- use a king-attack-enriched teacher corpus and paired counterfactual positions; and
- fit interacting danger inputs jointly, not one coordinate at a time only.

This is likely the best remaining classical feature family, but it is also where NNUE king conditioning has the clearest representational advantage.

## 8. Threats: richer than flat HCE, still lossy

Rarog deserves credit for going beyond generic hanging penalties. It has:

- pawn attacks by victim class;
- minor-attacker and rook-attacker tables indexed by victim type;
- refined hanging based on single/double attack and defense;
- safe pawn-push threats;
- weak pieces attacked by lower-valued material; and
- restricted-square counts.

The representation still aggregates away the relationships that decide whether a threat is real:

- exact attacker and victim squares;
- whether the attacker is pinned;
- whether executing the threat loses the attacker;
- whether the victim moves with tempo or gives check;
- discovered attacks and blockers;
- overloaded or pinned defenders;
- x-ray geometry;
- relation to the enemy king; and
- move order and side-to-move tactics.

There are also semantic simplifications in the current code:

- a safe pawn push means only that the destination is not attacked by an enemy pawn, not that the pawn survives minor/rook/queen attacks;
- restricted squares are counted globally even if no enemy piece can use the square;
- the same victim can contribute to overlapping base and v2 threat terms; and
- `attacked2` currently corrupts both hanging and restricted classification.

Stockfish's `Full_Threats` inputs encode a selected subset of exact attacking-piece/attacked-piece square pairs. The official documentation describes them as pair features added to the king-conditioned piece foundation. That is the key difference: the network receives the relationship before compression instead of asking a scalar count to summarize it.

A classical recovery path is possible but should be selective:

1. repair the attack substrate;
2. measure overlap and marginal value of base threat terms versus v2 tables;
3. condition safe pawn pushes on full attack/defense or SEE-like safety;
4. count restricted mobility per affected piece rather than unrelated board squares;
5. add only high-value pin/overload relations supported by residual analysis; and
6. avoid recreating a hand-written threat network one scalar at a time.

## 9. Pawn and passer evaluation needs future-state semantics

Rarog has a broad pawn package: passed rank, candidate, connected/support, doubled, isolated, backward, lever, islands, stop-square freedom/safety, path freedom/safety, blocker type, ideal knight blockader, rook support/blockade, king proximity, and an unstoppable flag.

The remaining weakness is mostly dynamic. Current attack maps describe the present board, while a passer's value depends on positions after one or more pushes:

- advancing vacates a square and opens slider lines;
- the blocker may move or be exchanged;
- rook-behind geometry can appear only after the push;
- king square-rule distance changes with tempo;
- promotion with check changes the race;
- connected passers can alternate pushes; and
- material determines whether a piece sacrifice for the pawn is favourable.

The new whole-path terms illustrate the fitting problem:

| Term | Current value |
|---|---:|
| Free path MG per rank | -1 |
| Free path EG per rank | -4 |
| Safe path EG per rank | 1 |

A clear path to promotion is not generally bad chess. Negative coefficients are more likely telling us that the feature is correlated with already-large passed-rank bonuses, activates in tactically misleading positions, or lacks the conditions needed to isolate useful path freedom. Treating those signs as a chess conclusion would be unsafe.

Other broad semantics include:

- backward-pawn detection cannot fully tell a fixed weakness from a pawn with a viable lever;
- pawn islands ignore whether breaks make the islands temporary;
- supported passers and phalanxes are conflated in documentation but not implementation;
- a candidate passer is not conditioned on the exchanges required to create it; and
- rook-behind terms ignore blockers.

Recommended work:

- fix the confirmed activation defects before retuning;
- add supported versus phalanx versus connected-passer features separately;
- distinguish blocker ownership/type and rear-rook line openness;
- construct a short-horizon passer-race diagnostic rather than more static path counts;
- bucket residuals by exact material signature and passer rank/file;
- enrich with tablebase WDL/DTZ for 5-7-piece races where available; and
- consider explicit pawn-pair inputs in NNUE if residuals remain dominated by chains, levers and rams.

## 10. Several positional terms ask an over-broad question

### Queen infiltration

[`eval.rs:1781`](../src/eval.rs#L1781) rewards a queen on relative rank 4 or higher if no enemy pawn attacks its square. The queen can still be attacked by a knight, bishop, rook, queen, or king. At `47/73` MG/EG, this can materially reward an advanced hanging or trapped queen.

The feature should use at least the full enemy attack map, preferably with defense/escape or SEE context. If full safety makes it too sparse, that is evidence that the broad term was averaging many false positives.

### Bad bishops

The bad-bishop term counts friendly pawns on the bishop's colour complex. It does not distinguish fixed central pawns from mobile flank pawns, pawns in front of versus behind the bishop, or whether the bishop has an active diagonal outside the chain. The MG coefficient fitted to zero while EG remains 17, consistent with an over-broad feature whose useful cases are phase- and structure-dependent.

### Bishop-pair pawn interaction

The comment says the bishop pair gains value with fewer pawns. The feature is `8 - total_pawns`, but the fitted weights are `-3/-5`. As pawns disappear, it reduces the bishop-pair bonus. Either the comment's intended chess prior is wrong in this evaluator, the term compensates for an oversized flat bishop-pair value, or correlated fitting inverted its marginal sign. This is not a binary code defect, but it is an identifiability warning.

### Space

Rarog improved the Basilisk-style broad space term by adding squares behind friendly pawns and piece-count weighting. The base set is still mainly centre-file squares not occupied by a friendly pawn and not attacked by an enemy pawn. It does not require general friendly control or exclude general enemy piece control. A square that neither side can use can contribute.

The current fitted values (`space_weight = 0`, `space_piece_mg = 1`, `space_behind_piece_mg = 0`) say that this representation has almost no marginal signal. A Stockfish-11-style space implementation used a more conditioned safe-area definition and nonlinearly weighted it by available pieces. More global space coefficients are unlikely to help unless square usability is repaired.

### Rooks and alignments

Rook seventh-rank values fitted to zero. A rank-only term ignores enemy king position, back-rank pawns, checking access, targets, and whether the rook is trapped. Rook connection and slider-on-queen/battery terms similarly need clear lines and useful direction; broad pawn-only x-rays can see through pieces that make the nominal relation strategically irrelevant.

### Trapped pieces and outposts

Trapped bishops are checked only below half phase and only at zero pseudo-mobility, missing common opening/middlegame traps and less absolute forms of restriction. Outposts require pawn support and absence of enemy pawn challenge, which is useful, but do not ask whether the piece is tactically stable, can be exchanged profitably, or has useful targets from the square.

The common pattern is feature misspecification: optimizing one weight averages genuinely good activations with irrelevant or harmful ones. The result is often zero, a surprising sign, or a small coefficient. That is not proof that the chess concept lacks value.

## 11. One phase scalar cannot specialize enough

Rarog computes a conventional remaining-material phase and uses it for every tapered term. Positions with the same scalar phase can nevertheless be strategically unrelated:

- queenless middlegame with four rooks;
- queen plus minors and no rooks;
- rook-and-minor ending;
- opposite bishops with queens;
- closed centre with undeveloped pieces; and
- open position after mass pawn exchanges.

They should not share exactly one interpolation rule for king safety, mobility, bishop pair, space, passers, and threats.

Modern Stockfish-style NNUE uses eight piece-count-selected PSQT outputs and eight downstream layer stacks. This is a small mixture of experts: the same input can be interpreted differently in dense and sparse positions. A classical analogue would be:

- material-signature-specific scale functions;
- a few piece-count/material buckets with separate downstream coefficients;
- queen-presence and major-piece-presence gates for king danger;
- pawn-count or openness gates for mobility/bishop terms; or
- king-bucketed PSTs as a transitional representation.

Adding more MG and EG constants without adding a discriminator does not solve this ambiguity.

## 12. Endgame conversion knowledge is useful but still sparse

Rarog's specialized endgames are a genuine strength over many small HCEs. The current dispatcher handles exact or narrow cases for KPK, wrong-bishop rook pawns, KQKP fortress, conservative KRKP, pawnless insufficient material, KNNK, and KBNK/KXK mop-up.

Large families still fall through to generic tapering and a coarse initiative term:

- rook and pawn versus rook beyond the one KRKP heuristic;
- multi-pawn rook endings;
- queen versus rook/minor fortress patterns;
- bishop-versus-knight endings with fixed pawn colours;
- wrong-rook-pawn patterns with additional defensive material;
- multi-pawn races;
- fortress-like material advantages; and
- pawnless imbalances that are technically winning but hard to convert.

The generic initiative at [`eval.rs:2531`](../src/eval.rs#L2531) uses total pawn count, king-file distance, and whether pawns exist on both flanks. It multiplies this complexity by the sign of the EG score and the fitted weight 2. Thus every nonzero endgame score is pushed farther from zero as "complexity" rises. It cannot reduce an unconvertible advantage, distinguish outflanking or infiltration, account for passers, recognize pure pawn endings, or preserve a calibrated winning margin. Calling it initiative is reasonable; treating it as winnability would not be.

Recommended direction:

1. build residual tables by exact material signature before adding rules;
2. use Syzygy WDL/DTZ as direct evidence for eligible positions;
3. implement only high-confidence material scalers with sign-preserving, non-amplifying behavior;
4. develop a real conversion/complexity model including strong-side pawns, passers, both flanks, king infiltration/outflanking, pure pawn endings and material family; and
5. validate drawn, won and cursed/blessed tablebase cohorts separately.

Do not force all conversion knowledge through one scalar. Material-specific scaling and learned material buckets are complementary.

## 13. Correction history helps, but cannot restore discarded information

Rarog's search combines pawn, minor, own non-pawn, opponent non-pawn, and one-ply continuation correction at [`search.rs:2230`](../src/search.rs#L2230):

```text
(pawn + minor + own_non_pawn + their_non_pawn + continuation/2) / 128
```

This is a strong feature for an HCE engine: search learns systematic residual error for recurring structures. Its current form is less expressive than leading implementations:

- source weights are fixed rather than jointly tuned to their predictive value;
- continuation correction is keyed only by the previous piece and destination, not a current/previous move pair;
- only one previous ply is represented;
- tactical best moves, captures, promotions, and in-check contexts are not consistently excluded from updates;
- correction magnitude is not yet fully used as an uncertainty signal for pruning; and
- hash collisions and support rates are not reported.

Correction history can learn that "this pawn hash is usually underestimated". It cannot reconstruct the exact bishop-attacks-knight relation or king-conditioned piece geometry that the static evaluator never encoded. It is best treated as online residual learning and confidence estimation, not as a substitute for evaluator capacity.

This overlaps with [`analysis/search_analysis.md`](search_analysis.md); HCE experiments should record raw HCE, corrected HCE, qsearch score, and depth-N score separately so the source of each gain is visible.

## 14. Tuning and data pipeline: significant strengths, remaining blind spots

### Improvements over the Basilisk checklist

Rarog's current extraction path already repairs several common mistakes:

- train/holdout split is by game, not individual position;
- FENs are deduplicated;
- a true-quiet proxy rejects positions with a winning capture available, rather than only positions where the played move was a capture;
- training can blend game result with the engine's search evaluation;
- phase mixture is reported and training can be balanced; and
- the nonlinear king-safety fitter re-evaluates positions and restores the best holdout state.

These are material process improvements and should be preserved.

### Self-referential adjudication

[`tools/datagen.ps1`](../tools/datagen.ps1) uses draw adjudication after an eight-move low-eval window and resignation after repeated 600-cp evaluations. The loop is:

```text
current Rarog evaluator/search
    -> positions reached and moves selected
    -> evaluator-based draw/resign adjudication
    -> final-result labels
    -> next Rarog evaluator
```

This is useful on-policy optimization. It is weak at correcting positions the current engine avoids, resigns prematurely, or calls drawn for the wrong reason. A refit can converge because the policy/data fixed point has been reached, not because objective residual error is small.

### The "holdout" is validation, not untouched test

The generic tuner fits sigmoid scale `K` on the holdout, selects the best epoch on the same set, restores it, and reports bucket losses there. The nonlinear king-safety path likewise fits `K` and chooses its best vector on the holdout. That is a valid validation workflow, but the set is not an unbiased final test set.

Rarog needs three roles:

| Split | Use |
|---|---|
| Train | Gradient/coordinate updates |
| Validation | `K`, blend lambda, early stopping, architecture and hyperparameter selection |
| Frozen test | One-time residual report; never used to choose weights or epochs |

### Label objective

Pure game-result WDL is noisy and sparse at the position level. Search-cp blending provides denser information, but the teacher is the same engine and its score contains the same HCE/search biases. The rejected Stockfish-distillation experiment is important evidence that lower off-policy MSE does not guarantee Rarog Elo. It does **not** imply that objective teacher diagnostics are useless; it implies that teacher fitting must be used to locate residuals and candidate representations, while self-play SPRT remains the deployment verdict.

The recent comparison between pure-WDL and blended targets must also be interpreted carefully: if lambda or candidates are selected on a pure-result validation objective, pure WDL has an objective-alignment advantage on that same set. A frozen external test plus SPRT is required to decide generalization and playing strength.

### Identifiability

Material/PST fitting has an inherent near-degeneracy: shifting every PST square for a piece and compensating its material value represents nearly the same function. L2-to-prior and material constraints mitigate drift, but do not centre PSTs explicitly. Other correlated families include:

- mobility versus trapped-piece terms;
- passed-rank versus stop/path safety;
- king attacker units versus safety-table shape;
- weak ring versus safe checks;
- flat bishop pair versus bishop-pair pawn correction;
- space versus mobility; and
- material versus quadratic imbalance.

Zero or negative fitted weights should trigger activation and covariance analysis, not immediate chess interpretation.

## 15. What the best engines do better beyond constants

### 15.1 They preserve relationships before compression

Rarog is approximately:

```text
evaluation = sum(feature_i(position) * weight_i), blended by one phase
```

Its nonlinear king table is an exception, but even there many relations are first compressed into one danger index.

A king-conditioned, threat-input NNUE can represent:

```text
bishop on this square
    with our king in this region
    and their king in that region
    attacking this particular victim
    through this pawn geometry
    at this material density
```

An HCE needs explicit cross-terms for each useful conjunction. Tuning constants cannot invent a cross-term that is absent from the trace.

### 15.2 They condition every piece on king location

Rarog's PST for a knight on e5 is global. Its king-safety package separately counts whether that knight attacks a zone. King-conditioned features give the knight a different first-layer vector depending on the king region. That naturally covers checking geometry, escape control, shelter interaction, passer races, and endgame king activity.

### 15.3 They encode exact threat pairs

Stockfish's current `Full_Threats` represents selected attacker-square/victim-square pairs. Rarog's tables know attacker type and victim type but mostly discard their squares and intervening geometry. Threat inputs let nonlinear layers combine the relationship with kings, other pieces and material.

### 15.4 They specialize by material density

Eight PSQT outputs and layer stacks selected by piece count allow different downstream functions for dense middlegames and sparse endings. Rarog uses one phase scalar for every term and a small hardcoded endgame dispatcher.

### 15.5 They retain a direct linear material/PSQT path

Modern Stockfish NNUE forwards several PSQT outputs directly from the feature transformer. This stabilizes large material imbalances and basic placement, leaving nonlinear capacity for interactions. Rarog's eventual NNUE should preserve this design option rather than force the hidden layers to relearn material from scratch.

### 15.6 They train on orders of magnitude more dense supervision

Rarog's multi-million-position on-policy data is useful for its scale, but Stockfish reports workflows beyond 100 billion Lc0-labelled positions and PlentyChess reports 15+ billion self-generated positions. The gap is not just quantity. Deep-search/cp/WDL labels supply information for every position, while one final result is shared across many positions and confounded by later play.

### 15.7 They co-adapt evaluation, residual correction and pruning confidence

Top search does not treat evaluation as a perfectly reliable scalar. Correction history adjusts recurrent residuals; correction magnitude and related context can influence pruning confidence. Stronger evaluation changes the safe operating point of reverse futility, null move, LMR, ProbCut and qsearch. Evaluator replacement must therefore be followed by search recalibration.

## 16. Diagnostic benchmark required before more HCE fitting

Create a frozen evaluator benchmark independent of optimizer epoch selection and Rarog adjudication. It should contain:

1. deep teacher cp/WDL labels from one or more strong sources;
2. Syzygy WDL/DTZ for eligible positions;
3. by-game or by-trajectory train/validation/test separation;
4. exact material-signature and phase labels;
5. king-danger and exposed-shelter cohorts;
6. passer races and rook-behind-passer positions;
7. fortresses and difficult conversions;
8. positions before losses, not only the tactical collapse;
9. paired counterfactuals changing exactly one intended feature; and
10. raw HCE, lazy HCE, corrected HCE, qsearch and depth-N outputs.

For every evaluator candidate report:

| Metric | Purpose |
|---|---|
| Global cp/WDL residual | Overall calibration |
| Residual by exact material signature | Phase/endgame failures |
| Residual by king-danger bucket | Attack under/overestimation |
| Residual by passer rank/file/blocker | Race and conversion failures |
| Full versus lazy delta/sign flips | Train/serve truncation risk |
| Raw versus corrected HCE | Correction-history contribution |
| HCE versus qsearch/depth-N | Static versus tactical error |
| Feature activation count/covariance | Sparse, aliased and redundant terms |
| Paired-position delta | Whether the feature has the intended semantics |

The benchmark is a diagnostic and experiment-selection tool, not a replacement for SPRT. The rejected distillation result proves that static loss alone is insufficient; it does not justify flying blind.

## 17. Recommended roadmap

### Phase A: correctness and semantic tests

1. Fix pawn contributions to `attacked2`.
2. Split friendly and enemy rook-behind-passer loops and define blocker semantics.
3. Restrict or rework unstoppable passers.
4. Separate pawn support and phalanx activation.
5. Narrow OCB scaling by material family.
6. Add direct tests for each case plus random colour/mirror symmetry.

These changes should be tested individually, then jointly refitted because the first two alter downstream trace activations.

### Phase B: measurement infrastructure

1. Build the frozen external diagnostic corpus.
2. Add cheap/full dual-evaluation logging.
3. Add activation, covariance and per-cohort residual reports.
4. Split train, validation and untouched test roles.
5. Keep self-play SPRT as the final accept/reject gate.

### Phase C: highest-value remaining HCE structure

Recommended order:

1. king danger after the attack-substrate fix;
2. real winnability/material-specific scaling;
3. passer race and rook/blocker semantics;
4. material/phase specialization;
5. safe queen infiltration, restricted mobility and a small number of conditional threat repairs;
6. correction-history weighting, tactical update filtering and uncertainty consumption; and
7. remove or retire terms that remain misspecified/zero after cohort analysis.

Do not launch another undifferentiated all-parameter fit before the confirmed activations are fixed. It would optimize around known semantic errors again.

### Phase D: staged NNUE

Use a simple network to prove accumulator correctness, serialization, quantization, embedding, SIMD inference and search integration. Do not freeze the baseline feature set as the final architecture.

| Stage | Candidate capability | Question answered |
|---:|---|---|
| A | Plain piece-square, two perspectives, one output | Is inference/update plumbing correct and fast enough? |
| B | King buckets / HalfKA-like input | How much king conditioning buys at acceptable refresh cost |
| C | Direct PSQT plus 8 material/output buckets | How much phase/material specialization buys |
| D | Explicit threat inputs | Whether tactical/positional threat residuals justify update bandwidth |
| E | Pawn-pair inputs | Whether chain/lever/ram residuals remain after king/threat inputs |

Compare NPS, accumulator refresh rate, static residuals, tactical/fortress cohorts, STC/LTC SPRT, and search interaction. Architecture should be versioned in the network file and inference interface from the first prototype.

### Phase E: retune search after evaluator stabilization

NNUE changes centipawn calibration, cost per node, residual distribution and pruning reliability. After the feature set stabilizes:

- retune correction-history weights;
- retune RFP, null move, futility, ProbCut and LMR margins;
- reconsider lazy evaluation;
- use correction/eval volatility as confidence signals; and
- validate at long time controls and multiple thread counts.

## 18. Non-additive planning estimates

These are priors for experiment ordering, not promises:

| Work package | Plausible scale | Confidence |
|---|---:|---|
| `attacked2`, rook/passer, unstoppable, phalanx correctness bundle | 5-20 Elo | High confidence in defects, low confidence in aggregate Elo |
| King-safety semantic rework | 10-35 Elo | Medium |
| Winnability/material scaling/endgames | 5-25 Elo | Medium |
| Passer/pawn/threat conditionality | 10-30 Elo | Medium-low because of overlap |
| Better correction-history consumption | 5-20 Elo | Medium |
| Independent diagnostic data and improved objectives | 10-30 Elo direct HCE potential, larger enabling value | Medium |
| Competitive king-conditioned NNUE | 100-250+ Elo | High confidence in direction, low confidence in exact range |

The HCE rows overlap heavily and should not be summed. A reasonable working prior is roughly **20-60 net Elo** from a disciplined HCE repair campaign. More is possible, but recovering several hundred Elo through additional additive terms is improbable.

## Priority list

| Priority | Item | Classification | Why now |
|---:|---|---|---|
| 1 | Fix pawn `attacked2` overlap | Verified defect | Corrupts threats and king safety downstream |
| 2 | Fix enemy rook-behind-passer loop | Verified defect | Natural activations are absent from both eval and tuning trace |
| 3 | Restrict unstoppable passers | Verified semantic defect | Awards race bonus despite trivial piece stops |
| 4 | Separate phalanx from support | Verified missing activation | Current feature name and intended concept exceed implementation |
| 5 | Add semantic/counterfactual tests | Process gap | Prevents the same class of silent feature error |
| 6 | Audit lazy/full disagreement | Train/serve gap | Current positive speed result does not prove evaluation safety |
| 7 | Build frozen diagnostic corpus | Process gap | Required to choose structural work rationally |
| 8 | Rework king-danger inputs | Architectural gap | Highest-value remaining classical interaction family |
| 9 | Add material/winnability specialization | Architectural gap | One phase scalar overstates/understates conversion |
| 10 | Prototype king-conditioned NNUE | Strategic | Main path beyond the HCE ceiling |

## Conclusion

Rarog's HCE is mature enough that another round of constant optimization is unlikely to change its competitive tier. The current evaluator already contains most classical feature names. Where it loses is in the **meaning and conditioning of activations**:

- two-pawn attacks are lost before threat and king analysis;
- an enemy rook relation depends on an unrelated friendly rook;
- an "unstoppable" pawn ignores defending pieces;
- a "phalanx" is not detected;
- advanced queens, space, outposts, passer paths and king attacks are judged by broad proxies; and
- all material classes share one tapered function until a small endgame dispatcher intervenes.

The best engines do not merely have better constants for these concepts. They retain exact piece, square, king, threat, pawn and material relationships, learn nonlinear interactions from dense supervision, and let search learn contextual residuals and uncertainty.

The immediate HCE program should therefore be:

1. repair the four verified activation defects;
2. add semantic counterexample tests;
3. establish a frozen residual benchmark and lazy/full diagnostics;
4. spend remaining HCE effort on king danger, conversion and material specialization; and
5. use that measurement infrastructure to design a king-conditioned, threat-aware NNUE rather than a permanently plain piece-square net.

The important future question is not only:

```text
Did the fit reduce Rarog's own WDL loss?
```

It is also:

```text
Did the representation become capable of distinguishing the positions
that deep chess evidence says have different value?
```

That distinction is where the remaining evaluation Elo is likely hiding.
