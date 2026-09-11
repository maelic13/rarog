# Post-fit residual audit of the accepted HCE — 2026-09-01 (PLAN 4.9.1)

Entry evidence for 4.9. The question 4.9 asks is whether the accepted surface
is systematically wrong in a way **no coefficient value could fix**, because
only that licenses structural work.

## Instrument

`rarog-texel --report-endgames` over the accepted vector
(`hce-fit-20260831_095443/04-final.txt`, SHA `BAD51F3E…`) at the fit's pinned
`K = 1.37011`, on the **127,778 published-but-unused** positions from the
confirmation run (`hce-confirm-20260831_230548/dataset/train.csv`, recorded in
that run's manifest as `train_unused`). That set has never been fitted on,
never selected anything, and is not the frozen test — so it can be inspected
without burning a one-shot instrument. Global loss on it is **0.12241022**.

Artifact: `tools/results/hce-accepted/residual-endgames-accepted.csv`, 299
material classes.

## The finding: the largest residual is a LABEL defect, not an evaluation one

| Class | n | drawn n | evaluator predicts | actual label mean | bias |
|---|---:|---:|---:|---:|---:|
| **KR-K** | 379 | **284 (75%)** | **0.849** | 0.625 | **+0.339** |
| KRPP-KR | — | 111 | 0.786 | — | +0.286 |
| KR-KN | — | 108 | 0.733 | — | +0.233 |
| KBP-KN | — | 150 | 0.726 | — | +0.226 |
| KRB-KR | — | 198 | 0.723 | — | +0.223 |

**KR-K is K+R versus a bare king. It is a 100% theoretical win, and 75% of the
corpus's KR-K positions are labelled a draw.**

The evaluator predicts 0.849 on those positions. The truth is 1.0. The label
says 0.5. **The evaluator is closer to the truth than its own training data
is**, and the fit has been pulling it downward.

## Mechanism, confirmed directly

Rarog's own score for a won KR-K, by king placement:

| Position | Score |
|---|---:|
| KRK, defending king cornered | **+426** |
| KRK, defending king centralised | **+487** |
| KRK, from the audit's sample position | +698 |
| KBNK | +622 |
| KBBK, opposite-coloured bishops | +605 |
| KBBK, same-coloured bishops | 0 (correct — drawn) |

`datagen-v1`'s resign rule needs **both engines above 600 cp for three
consecutive moves**. Rarog evaluates a won rook ending at 426–698 depending on
where the kings stand, so across most KR-K positions the rule never fires. The
game plays on at 8,000 nodes/move, the engine fails to mate inside fifty moves,
and the label becomes **0.5**.

This is a self-reinforcing loop, and it is the one RAR-E08 was registered to
break: cannot convert → labelled a draw → evaluator learns draw → never steers
there → never learns to convert.

## Cohort residuals — 4.9's own named hypotheses

`--buckets` on the same unused set, same pinned K. Global loss **0.1224102**.

| Cohort | n | loss | vs global |
|---|---:|---:|---:|
| opening | 41,314 | 0.1728983 | +41% |
| middlegame | 45,266 | 0.1185061 | −3% |
| endgame | 41,198 | 0.0760696 | −38% |
| king-attack | 85,116 | 0.1270836 | **+3.8%** |
| threat | 119,282 | 0.1261994 | **+3.1%** |
| passer | 77,315 | 0.1149288 | **−6.1%** |
| no-queens | 68,244 | 0.0939508 | −23% |
| ocb | 8,631 | 0.0799537 | −35% |
| rook-ending | 11,705 | 0.0769564 | −37% |
| pawn-ending | 3,496 | 0.0990179 | −19% |

4.9 named king-safety and passer/threat conditionality as its two standing
hypotheses. King-attack and threat sit 3–4% above global on very broad
populations (67% and 93% of positions); passer sits 6% **below** it. None of
that licenses a structural cluster.

The opening cohort's +41% is not a defect signal either. Opening positions are
further from their result, so their outcome entropy is genuinely higher; a
cohort above global loss is not automatically mis-modelled. Distinguishing
"harder" from "wrongly modelled" would need an oracle's loss on the same
positions, which does not exist here.

**Instrument caveat, and it nearly invalidated this table.** The `passer`
bucket originally selected **127,777 of 127,778** positions and measured
nothing: `family_ranges()` used one min..max span per family, and `passed*`
is 13 non-contiguous fields whose span swept in 404 slots belonging to 113
other fields. Fixed before these numbers were taken; `threat` and
`king_safety_table` are contiguous and were always correct.

## What this means for 4.9 and 4.10

**4.9 closes: no evidence licenses a structural cluster.** 4.9 requires a
residual the existing surface cannot represent. This residual is fully representable — the surface would
price KR-K correctly if the labels said 1.0. Nothing here licenses structural
work, and the largest cohort residuals in the table are all of this shape:
sparse endgame classes whose labels were produced by a datagen engine that
could not convert them.

**It is strong prior evidence for RAR-E08's arm B.** The tablebase-corrected
label is exactly the correction KR-K needs, and RAR-M18's screen already found
13.27% of ≤6-man rows disagreeing with tablebase truth. This audit names the
mechanism behind that disagreement and shows it is not random: it concentrates
in classes the engine evaluates *below the resign threshold* while being
theoretically won.

**It qualifies the endgame audit's "drawn-subset overconfidence" finding.** On
KR-K at least, that overconfidence is the evaluator being right. Before
treating a drawn-subset bias as an evaluation defect, check whether the drawn
subset is theoretically drawn — for ≤6-man classes the tablebase can answer
directly.

## Open, and not answered here

The classes with a genuine label-versus-truth question above 6 men — KRPP-KR,
KRB-KR, KBPP-KNP and the rest of the table — cannot be checked against
tablebases at all. Whether their residual is the same label artifact or a real
evaluation gap is unresolved, and 7-man tablebases are not available locally.
Do not assume it generalizes from KR-K.
