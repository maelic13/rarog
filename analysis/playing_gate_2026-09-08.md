# Integrated board cluster playing gate — RAR-E15 / 4.11b.17

**Registered 2026-09-08, before any games.** Bounds, cap, book, clock,
adjudication and stop rule are fixed here and in `EXPERIMENTS.md`; none of them
may change once games are seen.

## Arms

| Arm | Engine | Git | Bench 13 | Binary SHA-256 |
|---|---|---|---:|---|
| Candidate | `rarog-411b-cand-pext-pgo.exe` | `b33d3ad` (head) | **7,601,220** | `8916725E...D958B617` |
| Baseline | `rarog-411b-base-pext-pgo.exe` | `fd21612` | **6,901,489** | `6F1592FF...3A5F92C6` |

`fd21612` is the 4.11b **section entry** — the last revision before any 4.11b
source change. Both are final-PGO pext builds from `tools/build_test.ps1`,
`rustc 1.97.1`, and both manifests record `git_dirty: false`.

## What is being gated

Every deliberate behaviour change in section 4.11b, as one dependency-complete
verdict:

- **4.11b.3** UCI and fullmove boundary repair.
- **4.11b.5 SEE king-legality, created-pin and recapture-promotion repair** —
  the dominant change, and the reason this gate exists.
- The behaviour-neutral throughput work of 4.11b.9, 4.11b.13 and 4.11b.14,
  which per the leaf does not require a separate SPRT and rides along here.

## Prior — genuinely two-sided, and the headwind is measured

This is not a candidate expected to win. Two measured facts point in opposite
directions:

**Against.** The candidate searches **+10.14% more nodes at fixed depth 13**
(6,901,489 -> 7,601,220). The repaired SEE prunes less, because the pre-repair
kernel was returning wrong verdicts that happened to prune more. AGENTS records
a measured **+7.36% tree change worth −1.49 ± 2.87 Elo**, so a +10% tree is
plausibly worth roughly −2 Elo.

**For.** RAR-M41 measured **+1.421% [+0.953%, +1.764%]** whole-search NPS, worth
about +2.8 Elo at the project's ~2 Elo per 1% NPS constant. **Caveat:** that was
measured over `1d720af..head` only, so the SEE repair's own throughput effect is
**unmeasured** and is not in that figure.

Net time-to-depth is about **8.6% worse** (1.1014 / 1.0142). Combining the two
calibrations gives a prior of roughly **−4 to +4 nElo, centred near zero**.

That is precisely an unknown-sign repair, so the bracket is **symmetric**, not
the `[0,3]` default and not a gainer bracket. RAR-S62 is the direct precedent: a
ProbCut correctness fix, gated at `[-5,5]`, which **cost** 5 Elo and accepted H0
at 4,436 games.

## Registered design

| Setting | Value |
|---|---|
| Bounds | **`[-5,5]` nElo** |
| Cap | **16,000 games** |
| Alpha / Beta | 0.05 / 0.05 |
| Time control | `3+0.03` |
| Threads / Hash | 1 / 64 MB |
| Book | paired `UHO_Lichess_4852_v1` |
| Adjudication | **none** (RAR-M17 default) |
| Concurrency | 14 with affinity |

**Sizing from RAR-M10**, `drift/game ≈ 8.3e-6 × width × (true − midpoint)`, at
width 10 and midpoint 0:

| True nElo | Games to a verdict |
|---:|---:|
| +4.5 | ~7,900 to H1 |
| +3 | ~11,800 to H1 |
| +2 | ~17,700 — beyond the cap |
| 0 | **never resolves** |
| −5 | ~7,100 to H0 |

The cap of 16,000 therefore resolves a true effect of about **2.2 nElo or
larger**. It is chosen over RAR-S62's 12,000 (which resolves 2.95 or larger)
because this prior carries real mass near zero, and the extra 4,000 games
materially reduce the chance of an uninformative stop. At RAR-M16's
no-adjudication throughput of 88.4 games/min the cap is about **3 hours**.

## Stop rule and what each outcome means

1. **H1 accepted.** The cluster is the accepted foundation for 4.12 and the
   development fingerprint updates to 7,601,220 with this verdict as evidence.
2. **H0 accepted.** The repaired pruning costs measurable strength. **This does
   not license reverting the repair.** Per the leaf: investigate the implicated
   search consumers and re-register a coherent repair; do not relax the oracle
   and do not proclaim a known-bug baseline correct. The 4.11b.5 defects are
   real and independently fixtured across 41 external cases — this gate asks
   what the *repaired* pruning costs, not whether the bug was a bug. The
   accepted fingerprint stays at the baseline's until a re-registered repair
   passes.
3. **Unresolved at the cap.** An accepted outcome and **not a pass**. It means
   the true effect is inside roughly ±2.2 nElo. RAR-S61 is the standing warning:
   a point estimate with a high LOS is not a result. RAR-S63 hit exactly this,
   stopping at +0.63 ± 6.22 over 12,000 games.

No bound, cap, book, clock or adjudication setting changes after games are seen,
and no success threshold is invented afterwards.

## Status at registration time

**Registered, not yet run** — superseded by the Result section below, which
records the run the maintainer then performed.

---

# Result — 2026-09-08, **H1 ACCEPTED**

`SPRT ([-5.00, 5.00]) completed - H1 was accepted` at **1,950 games**, 12.2% of
the registered 16,000 cap, in 21 minutes 47 seconds. The overnight Windows
restart happened afterwards and cost nothing: the run had already completed at
21:14:56Z.

| Quantity | Value |
|---|---|
| Elo | **+12.12 ± 10.17** |
| nElo | **+18.40 ± 15.42** |
| LLR | **2.96** of ±2.94 (100.4%) |
| LOS | 99.03% |
| W–D–L | 530–958–462 over 1,950 games, **51.74%** |
| Draw ratio | 41.74% |
| PairsRatio | 1.25 |
| Ptnml(0–2) | [45, 207, 407, 267, 49] |
| Timeouts / crashes | 1 / 0 |

Provenance verified against the registration: engine SHA-256 `8916725E...` and
`6F1592FF...`, `elo0=-5 elo1=5 alpha=0.05 beta=0.05 model=normalized`, budget
16,000, `3+0.03`, Hash 64, 1T, concurrency 14 on affinity CPUs
`0,2,4,...,26`, paired UHO book `7A7F6470...`, **adjudication none**, natural
termination. Nothing was changed after games were seen.

The single timeout is 1 in 1,950 = **0.051%**, at or below RAR-M14's documented
forfeit floor for this concurrency (0.077%, 0.135% and 0.172% in three
identical-binary null pairs). It is recorded, not treated as a defect.

## The prior was badly wrong, and in an instructive way

Registered prior: **−4 to +4 nElo, centred near zero**. Measured: **+18.40
nElo**. That is a miss in magnitude and in the confidence with which the sign
was called two-sided.

The error was treating the **+10.14% node increase as a tax**. It cited AGENTS'
measured "+7.36% tree change worth −1.49 ± 2.87 Elo" — but that calibration came
from a change that grew the tree *without* improving the decisions inside it.
Here the tree grew **because the search stopped pruning incorrectly**: the
pre-repair SEE returned wrong verdicts that cut lines it should have kept. The
extra nodes are the symptom of the repair working, not its cost, and the
accuracy plainly outweighed the ~8.6% time-to-depth penalty.

The second error was over-generalising from **RAR-S62**, where a ProbCut
correctness fix cost 5 Elo. That was one precedent with a different mechanism —
a desync that may have carried usable signal — and it was weighted as though it
established a general rule that correctness fixes cost strength.

**The lesson is specific: a node-count increase caused by removing wrong prunes
is not comparable to a node-count increase from widening a search.** They have
opposite expected signs and the calibration constant for one does not transfer
to the other.

## RAR-M10 was validated, and outside its stated range

RAR-M10 predicts `drift/game ≈ 8.3e-6 × width × (true − midpoint)`. At width 10,
midpoint 0 and a true +18.4 nElo that gives `1.527e-3` per game and **1,925
games** to ±2.94. The run took **1,950** — within **1.3%**.

RAR-M10 explicitly warned that predictions "outside roughly ±6 nElo, or under
different bounds" are extrapolation. This point is both: +18.4 nElo under
`[-5,5]` rather than `[3,10]`. It held anyway, which **extends** the fit's
validated range and is worth recording as a fourth calibration point.

## What is claimed

**Claimed:** the integrated 4.11b board cluster is accepted at **+12.12 ± 10.17
Elo**, **+18.40 ± 15.42 nElo**, H1 at 1,950 games under a symmetric `[-5,5]`
bracket registered before any games. The development fingerprint **7,601,220 /
EBF 2.474** now has its integrated verdict and becomes the accepted foundation
for 4.12.

**Not claimed — the magnitude is imprecise.** An SPRT decides; it does not
estimate. At 1,950 games the interval is ±10.17 Elo, so the honest reading is
"clearly positive, size poorly determined". Do not quote +12.12 as a settled
number.

**No subcomponent is credited.** The SEE repair, the boundary repair and the
throughput work were gated as one cluster under the cluster rule. Splitting the
gain between the repaired pruning and the +1.421% NPS requires an ablation that
has not been run, and RAR-S57's precedent is explicit that the honest
attribution until then is "the bundle".

## Evidence

`tools/results/sprt_411bCluster_vs_411bBase_20260908_225308.{log,pgn,manifest.txt}`
plus both engine manifests. PGN SHA-256 `F6F7295C...`, log SHA-256 `C02B1E36...`.
