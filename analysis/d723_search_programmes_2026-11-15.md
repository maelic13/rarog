# D.7.2.3: Rarog's and Basilisk's search programmes compared (2026-11-15)

Research only; nothing was built or timed for this note. Basilisk's records
were read without modification (D:/code/basilisk, its ledger IDs BAS-*).
Three read-only agents collected the cluster records; every figure below
that carries a ledger ID was checked in that ledger. **The conclusions wait
for RAR-O12** (D.7.2.4), which re-reads RAR-O11's withdrawn equal-node
results at equal work; section 8 states what each outcome would mean.
Statements marked *inference* are this note's reading, not a record's.

## 1. The question

Rarog 2.4.0 and Basilisk 1.10.2 were level on the clock (RAR-O11 (d1),
−11.5 ± 11.7 for Rarog). Both rebuilt their search in a donor's shape,
cluster by cluster, each cluster fitted and gated: Rarog in Reckless's,
Basilisk in Stockfish 19's, citing Rarog's clusters as its reason
(Basilisk PLAN, Phase B). Basilisk's programme gained about 65 more on the
clock (RAR-O11, (d3) − (d1), −64.8 ± 16.1). Where?

## 2. The programmes side by side

Gate results as their ledgers give them (Elo, 95% intervals; first-crossing
SPRT estimates are biased upward; cluster figures are not additive).

| Cluster | Rarog (Reckless's form) | Basilisk (Stockfish 19's form) |
|---|---|---|
| Start meter | oracle G(0) −248.0 ± 10.9 (RAR-O03) | oracle G(0) +312.6 ± 17.8 for the oracle (BAS-O05) |
| Selectivity core, unfitted port | +65.1 ± 23.3 (RAR-S73, B.2.4a) | +130.9 ± 11.4, fastchess +136.2 (BAS-S23) |
| Its fit | +138.6 ± 30.7 (RAR-S73, B.2.4b), then +13.1 ± 5.4 (RAR-S78) | +58.7 ± 20.9 (BAS-S23) |
| Null move, ProbCut, singular (IIR in Basilisk's) | unfitted +18.4 ± 9.4 (RAR-S79); gate +50.5 ± 10.9 (RAR-S84) | unfitted +67.5; gate +92.8 ± 14.1, its fit +17.6 ± 6.4 (BAS-S43) |
| Quiescence | unfitted +0.7 ± 9.3; gate +4.4 ± 2.9 (RAR-S88) | unfitted +29.3 ± 9.5; gate +31.7 ± 8.6, fit +6.5 ± 3.6, delta rule deleted +15.0 ± 8.0 (BAS-S52) |
| Root | donor forms lost (RAR-S89 to S92, NO_CHANGE); tablebase root +11.4 ± 8.5 (RAR-S94) | root records, centred aspiration, root LMR, 2× stop: +15.6 ± 5.9 (BAS-S55), fit +10.8 ± 5.3 (BAS-S56) |
| Joint tune after the clusters | not run: RAR-S95 read the finished journals as quiet, a restart worth +3 ± 3 | 170 coordinates under the new rule 7c: +28.2 ± 7.9 (BAS-S62) |
| Speed work inside the programme | +35.5 ± 9.0 (RAR-S98) | board rebuild +3.21% and +1.89% NPS (BAS-P30, P34) |
| The programme, direct | 2.5.0 +272 over 2.4.0 (RAR-M64) | +318.1 ± 24.5 over 1.10.2 (RAR-M84) |

- *Inference, where the gap sits.* It is not in the selectivity core:
  - Rarog's unfitted port was the weaker, +65 against +131. Its seeds were converted at ×0.457 against a measured ratio of 0.406 (RAR-M50, D.7.0's inventory §5).
  - Its fit then recovered more, +139 against +59, so both cores end near +200.
- The difference is in the next three clusters and the joint tune:
  - null move, ProbCut and singular, about +50 against about +110;
  - quiescence, about +4 against about +50;
  - root, Rarog's donor forms lost while Basilisk's +16 was kept;
  - the joint tune, not run against +28.

  Those sum to well over 65. They are not additive, and Rarog's in-programme speed pass offsets part.
- Basilisk's ports gained more **before** their fits as well: unfitted +67.5 against +18.4 and +29.3 against +0.7. So the difference in those clusters is not only fitting.

## 3. What each kept and ported in those clusters

- **Rarog kept its own forms where the donor's lost one at a time on its kernel** (D.7.0's inventory §3, §4, and the D.6 records):
  - quiescence fail-hard: fail-soft read −5.96 in RAR-S15, and RAR-R20 to R22 lost;
  - the delta rule, kept as a coordinate (Q5);
  - IIR beside hindsight, at any node of depth ≥ 4 (D.6.27);
  - ProbCut's SEE-gap filter (RAR-S57, S58);
  - its aspiration loop, after the donor's one-retry form and twice the width lost (RAR-S89, S90);
  - its node typing of null and ProbCut children (D.6.3).
- **Basilisk ported whole clusters in Stockfish's form.**
  - It deleted its delta rule (+15.0).
  - It took Stockfish's IIR (PV and cut nodes, depth ≥ 6), its quiescence and its root records.
  - It rejected a form only by a paired run against the fitted arm (BAS-S45, S53, S59, S60).
- *Inference.* In each of the three clusters where Basilisk gained more, Rarog's cluster was a mix: donor forms plus Rarog forms that had beaten their donor alternative one at a time. D.6's open search leaves sit in the same places (section 6). This is consistent with the records' own explanation, which both PLANs give: a coherent port fitted as a unit pays, while single mechanisms measured against a co-adapted optimum read zero or worse (Basilisk PLAN B; BAS-S16; Rarog's B.2 record). It is not yet shown that the mix is the cause. Basilisk's starting forms in those clusters may also have been weaker, which would give its ports more to gain.

## 4. Speed

Positions reached a second at 100 ms a move, one thread, 14 processes. These are RAR-O11's engine-reported rates divided by RAR-O12's counting ratios:

| Engine | Positions a second | Change over its search programme |
|---|---|---|
| Rarog 2.4.0 | 2.10 M | |
| Rarog 2.5.0 | 1.70 M | −19% |
| Rarog's head | 1.97 M | C.12's speed pass recovered part |
| Basilisk 1.10.2 | 2.64 M | |
| Basilisk's head | 2.20 M | −17% |

- Basilisk is faster per position than Rarog at every stage:
  - +25.6% at the start;
  - +29% at the search programmes' ends;
  - +11.7% at the heads.
- Its evaluation is not the reason: it is dearer per call than Rarog's (RAR-O11 (b), 452 against 416 ns).
- *Inference.* The speed is in board, generation and search overhead, where D.12 already records Rarog's widest measured board gap, generation at 29%. At about 2 Elo per 1% at `3+0.03` (Rarog's own rate), +11.7% would be worth about +23 at the heads; RAR-O12's (m3) against RAR-O10's (c) measures it.

## 5. Harness and SPSA

- **The runner is not the cause.**
  - Rarog moved its gates and tunes to Colosseum CLI on 2026-09-21 (B.2.6, PROCESS *Harness*). Basilisk used it from its first tune (A.3, 2026-09-29).
  - Each cross-check of the two runners agreed within its noise: RAR-M60, M61 for Rarog; B.2.2's fastchess replay for Basilisk.
  - Rarog's largest single gain came from a weather-factory tune (RAR-S75, 160,000 games, +138.6).
- **The method was the same family.** Both used blocks of 2,000 iterations, stopped when fewer than three coordinates moved a full step, until 2026-11-12/13. Basilisk then redesigned rule 7c (BAS-S57): one run to a ceiling, read at checkpoints by a gradient test. Rarog adopted it on 2026-11-13 and has not run a tune under it.
- **What differs is coverage.**
  - Basilisk ran one joint tune of 170 coordinates over its finished search (BAS-S61, S62), +28.2 ± 7.9 as a first crossing (its own read +22.9).
  - Rarog never ran one after its programme. RAR-S95 judged the finished journals quiet. The 2026-11-13 retrospective found a quiet read in that shape could not exclude about 11 Elo, and RAR-S78's restart found +13.1.
  - Rarog's last search refit, C.10.4 (RAR-S104, S105), was 82 coordinates after Phase C's evaluation changes: +4.5 ± 2.9.
- **Would a plain joint tune make Rarog level?**
  - *Inference.* Not on the evidence. The comparable reads are +4.5 (Rarog, two weeks after its evaluation changes) and +28.2 (Basilisk, overstated as a first crossing). That suggests +5 to +15, against a clock gap of about 65 at the programmes' ends (and Rarog's head is ahead today by its evaluation).
  - It is cheap next to a rebuild: at most 180,000 games, about a day and a half. It is independent of D.7's donor.
  - It would be wasted if D.7 rebuilds the kernel, since D.7.6 fits the new kernel anyway.
  - It belongs in the decision only on the branch where D.7 does not open (section 8).

## 6. A full donor form against the D.6 problems

From D.7.0's inventory (`analysis/search_shape_inventory_2026-11-13.md`)
and PLAN's D.6 records.

- **Would vanish or change with the kernel in one donor's form**, the inventory's own marks:
  - D.6.27, IIR without a node-type condition: the donor has none, and it closes if the tree does not miss it.
  - D.6.4, fail-low TT move: it closes with IIR gone (RAR-S117 showed the two entangled).
  - D.6.13, the aspiration fail-low window: the donor's aspiration form, probably moot.
  - D.6.16, quiescence gain pruning: the donor's futility and SEE form.
  - D.6.17, the promotion tie, broken by a root order the kernel replaces.
  - D.6.20, the fail-high stop's hole: the multi-cut window rule is dropped, the root driver kept.
  - D.6.3 and D.6.5, NO_CHANGE on the old kernel, arrive in the donor's form.
  - Seventeen Rarog forms the donor lacks are each to be asked again in D.7.8.
- **Would not change**:
  - D.6.28, clock use: Rarog spends 87 to 89% of its clock against Reckless's 95%. It is independent of the kernel and survives D.7 either way.
  - D.6.29, quiescence never scores a stalemate: Reckless lacks it too, Stockfish has it.
  - D.6.9, D.6.10 and D.6.21 to D.6.26: evaluation and recognisers.
  - D.6.30, the root driver.
- *Inference.* About half of D.6's open search leaves are symptoms of the mixed kernel: places where a donor form and a Rarog form, each fitted around the other, look defective only through each other (D.6.4 with IIR is the measured case). A full donor form removes that class wholesale; patching them one at a time has lost repeatedly (D.6.3, D.6.4, fail-soft, one-retry aspiration). This is the strongest argument for D.7 and does not depend on the donor's strength. It does not fix the clock, which RAR-O12 may show to matter.

## 7. Reckless or Stockfish as the donor

| Read | Reckless | Stockfish dev |
|---|---|---|
| Unfitted, on Rarog's evaluation, equal work, against the head (RAR-O09, RAR-O11 (e)) | +3.5 ± 8.4 for the donor | −37.8 ± 8.6 for the donor |
| Unfitted, on the clock, bridge returned (RAR-O08) | +22.5 ± 7.9 for the donor | −53.6 ± 8.2 for the donor |

- Fitted, the only evidence for Stockfish's form on a hand-written evaluation is Basilisk's: +318 from its start, and the larger cluster gains of section 2. That is on Basilisk's evaluation, with Basilisk's seeds, built by Basilisk.
- Unfitted reads measure transfer, not fitted potential (RAR-O08's note: Rarog's B.2 fit added +138.6, Basilisk's +58.7).
- Reckless is the nearer form: most of D.7.0's 104 departures are already the donor's or small, so its rebuild costs less.
- Reckless is AGPL-3.0 and Stockfish GPL-3.0; D.11 settles the licence either way.
- *Inference.* Neither donor is shown better for Rarog. Reckless transfers better unfitted and is closer. Stockfish is the one with a fitted HCE success, but in another engine.

## 8. What RAR-O12 decides here

The registration's frozen use: (m2) − (m1) is Basilisk's per-position gain over Rarog's.

- **(m2) − (m1) within ±15 of zero.** The two trees gained alike per position, and Basilisk's 65 on the clock is speed and clock use.
  - *Recommendation then:* Rarog's kernel speed (D.12's board and generation work, about +23 at the heads on section 4's estimate) and its clock use (D.6.28) come before any rebuild. They are cheaper and certain.
  - D.7, if opened, is for the D.6 class of section 6, with Reckless as the cheaper donor.
- **(m2) − (m1) below −25.** Basilisk's tree gained more per position.
  - *Recommendation then:* a full rebuild in one donor's form, ported whole cluster by cluster and fitted as a unit with a joint rule-7c tune at the end: Basilisk's process, not a patch.
  - The donor question stays open between Reckless (closer, better unfitted transfer) and Stockfish (Basilisk's fitted success). The cheapest discriminator is the unfitted cluster read, Basilisk's own first stage: port the null move, ProbCut and singular cluster in each form onto the head behind a switch, read each unfitted at 2,000 games, and compare with Basilisk's +67.5 and Rarog's +18.4.
- **Between.** Unresolved: both levers are open, and speed and the clock remain the cheaper first step.
- **Either way.**
  - A plain joint tune of the current search is worth doing only if D.7 does not open (section 5).
  - The clock-use leaf D.6.28 survives every branch.
