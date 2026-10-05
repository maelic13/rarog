# RAR-M15 — How often the 20 reference endgames actually occur, and what adjudication does to them

Indexed under *2. Measurement, harness and tuning* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**How often the 20 reference endgames actually occur, and what adjudication does to them.** Replayed all 3,915 RAR-E06 games (no adjudication) and classified every position at <=6 pieces; then re-simulated the games under `strength-v2` (draw \|cp\|<=10 for 8 moves from move 40; resign \|cp\|>=600 for 3 moves, two-sided) from the PGN's own eval comments.

## Result / disposition

**Observation, decisive for 4.9a's gate design.** **52.7%** of games reach a <=6-piece position and 60.9% reach <=7, so endgames are not rare in aggregate -- but the per-family spread is three orders of magnitude. KXK **37.34%**, KRPKR **10.04%**, KPsK 4.19%, KPK 2.84%, KRKP 2.40%, KBPsK 1.92%; then KQKP 1.17%, KRPKB/KPKP 1.23%, KBPKB 0.89%, KBPPKB 0.66%, KRKN 0.61%, KRKB 0.51%; then **KBNK 0.28%** (11 games), KBPKN 0.28%, KNNKP 0.05%, KNNK 0.03%; and **KQKR, KQKRPs and KRPPKRP occurred exactly ZERO times**. Under simulated `strength-v2`, endgames reached fall from 52.7% to **24.9%** -- adjudication destroys **52.7% of all endgames before they are reached**.

## Conditional lesson and retry trigger

**One gating policy cannot cover this range, and a whole-match SPRT is structurally incapable for the tail.** A change confined to a class occurring in 0.28% of games cannot produce a detectable whole-match Elo at any budget this project has; three of the twenty never occur at all, so an endgame-start cohort for them must be CONSTRUCTED, not sampled. Conversely KXK and KRPKR are common enough that a normal no-adjudication STC SPRT can see them, so the pessimism is not uniform either. The adjudication figure is why rule 8 is not merely a preference for endgame work: adjudication halves the sample of the thing being measured. Simulation caveat: the rule is applied to the PGN's eval comments rather than reproducing fastchess's internal bookkeeping exactly, so treat 24.9% as approximate -- the halving is far larger than any plausible error in that approximation.

## Source

`tools/results/sprt_HCERefit_vs_HCEBase_20260901_072106.pgn`; 4.9a
