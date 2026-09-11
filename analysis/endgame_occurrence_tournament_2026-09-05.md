# Endgame occurrence over 36,400 rated games

PLAN 4.11.12. Discharges the retry trigger 4.11.6 registered when it froze the
4.12 order: *"if family occurrence is re-measured over real games at real time
control, this ranking is re-derived and 4.12 renumbered again."*

Tool: `tools/diag/endgame_board_occurrence.py`.
Artifacts, all tracked because the corpus behind them is not -- a 117 MB
Colosseum database in `AppData` is not evidence anyone else can reproduce:

* `tools/diag/endgame_board_occurrence_v1.json` -- the measurement
* `tools/diag/endgame_board_occurrence_m15_replay.json` -- the calibration
* `tools/diag/endgame_ranking_v2.json` -- the registered order

The whole-pool cross-check is not stored, because it is one flag away:

```
python tools/diag/endgame_ranking.py     --board-occurrence tools/diag/endgame_board_occurrence_v1.json     --occurrence-scope all
```

## What was measured

The Colosseum "Rating Tournament" of 2026-09-04/05: **36,400 games**, fourteen
engines, 3+0.03, of which **10,000 are Rarog 2.3.2's** against thirteen
opponents. Every position of every mainline is classified by material, and a
family counts once per game in which it ever appears.

This replaces two proxies at once. Board occurrence came from RAR-M15 — 3,915
games of one engine pair against **itself**, transcribed into
`endgame_ranking.py` as twenty constants with no artifact behind them. Tree
occurrence came from 40 bench roots, which 4.11.5 then measured as weak enough
that 4.11.6 refused to use it as a multiplier.

## The calibration came first, and it is the load-bearing part

A new tool producing new numbers proves nothing about whether the numbers moved
or the definition moved. So the tool was run first over **RAR-M15's own corpus**
(`tools/results/sprt_HCERefit_vs_HCEBase_20260901_072106.pgn`, retained, exactly
3,915 games) and compared against RAR-M15's published percentages.

**Thirteen of twenty agree to four decimals**, including KBN-K's published count
of exactly 11 games, and both aggregate figures reproduce: 52.69% of games reach
six men or fewer against a published 52.7%, and 60.87% reach seven against
60.9%. The per-position classifier is therefore the same classifier, and the
seven differences are differences of definition or of coverage — not of
measurement. Each was then measured rather than argued:

| family | this tool | RAR-M15 | cause, established by measurement |
|---|---:|---:|---|
| KRPKR | 0.0817 | 0.1004 | plural strong side, capped at 6 men: **0.1004** exactly |
| KRPKB | 0.0087 | 0.0123 | same variant: **0.0123** exactly |
| KBPsK | 0.0199 | 0.0192 | same 6-man cap: **0.0192** exactly |
| KXK | 0.3747 | 0.3734 | RAR-M15 also excluded lone minors; 5-game residual UNEXPLAINED |
| KRPPKRP | **0.1009** | 0.0000 | 7 men — outside RAR-M15's 6-man cap entirely |
| KQKR | **0.0115** | 0.0000 | 4 men, inside the cap. **Cause unknown.** |
| KQKRPs | **0.0041** | 0.0000 | — |

The 6-man cap accounts for four of the seven exactly. The KBPsK line is worth
its own sentence: the calibration gate **found** that one — it was not on the
exception list — which is the only reason to run a gate whose expected outcome
is a list of known differences.

## Finding 1 — two of RAR-M15's three zeros were never zero

`endgame_ranking.py` floors a zero-occurrence family at 3/n by the rule of
three, on the principle that a sample which fails to contain something bounds
its rate rather than annihilates it. That principle is right and it was applied
to the wrong problem. These were not thin samples. They are positions that were
there and were not looked at:

* **KRPPKRP** — `8/1r3p2/8/7P/8/4kPK1/1R6/8 b - - 0 66`, from RAR-M15's own
  games. 395 of its 3,915 games (10.09%); **1,808 of 36,400** in the tournament
  (4.97%), and **540 of Rarog's 10,000** (5.40%).
* **KQKR** — `8/8/R6K/8/8/7k/8/5q2 w - - 0 79`, also from RAR-M15's own games,
  and four men, so the six-man cap never explained this one. 45 of 3,915
  (1.15%); **63 of Rarog's 10,000** (0.63%).
* **KQKRPs** — 16 of 3,915 (0.41%); **42 of Rarog's 10,000** (0.42%).

`PLAN.md` said of the first of these that KQ-KR's 25-point conversion gap is
**"the largest gap and worth nothing"**, on that zero. That sentence is wrong
and is corrected here rather than left standing. KQ-KR carries the largest
conversion deficit in the whole set *and* occurs in real games; it moves from
4.12.15 to **4.12.10**.

## Finding 2 — the fourth most common family in the set cannot be measured

**KRPPKRP is 5.40% of Rarog's games** — more common than KP-K, KP-KP, KQ-KP,
KR-KP and every family below them — and it is seven men, so the local
tablebases cannot adjudicate a single position of it. Neither its conversion
nor its drawn share exists.

It stays ranked last, because a family whose evidence cannot be produced cannot
be worked. But the *reason* has inverted. The `UNVERIFIABLE` note used to read
"reachable neither by sampling play nor by verified construction"; it is
reached constantly. This is a **tooling gap**, not a rare ending, and 4.12.21 is
now to record it as one.

## Finding 3 — Rarog reaches rook-against-a-lone-minor at 1.6x the pool rate

| family | pool (36,400) | Rarog (10,000) | ratio |
|---|---:|---:|---:|
| KRKN | 0.32% | **0.51%** | 1.59x |
| KRKB | 0.32% | **0.46%** | 1.44x |

These are exactly the two families 4.11.4 measured as Rarog's worst
drawn-share defect: **796/796 dead-drawn KR-KN positions scored at +346**, and
KR-KB at +307. An engine that prices a dead draw as near-winning will steer
into it, so the occurrence and the defect may not be independent.

**Stated as a hypothesis, not a result.** The alternative reading is ordinary
sampling — 51 and 46 games are small, roughly two standard deviations from the
pool rate. It also means engine-scope occurrence is **endogenous**: it is
inflated exactly where Rarog's evaluation is wrong. That is arguably the right
gate — those are the families where the defect actually costs games — but it is
a confound and is named rather than buried. 4.12.4 owns testing it.

## Finding 4 — endgames are rarer in a mixed pool than in self-play

52.7% of RAR-M15's self-play games reached six men or fewer; **44.4%** of the
tournament's do, and 54.9% reach seven against 60.9%. A pool with large strength
gaps decides more games before the endgame. Every per-family share below moved
downward for the same reason, so the *ranking* is affected far less than the
absolute numbers are.

## What was registered

The order is re-derived from `--occurrence-scope engine` — Rarog's own 10,000
games, the distribution Rarog actually plays into:

```
python tools/diag/endgame_ranking.py     --board-occurrence tools/diag/endgame_board_occurrence_v1.json     --occurrence-scope engine --output tools/diag/endgame_ranking_v2.json
```

```
KRPKR KXK KRKN KRKB KRPKB KBPKB KRKP KBPKN KQKR KPK KPKP KQKP KBNK KNNKP KNNK
| KPsK KBPsK KBPPKB KQKRPs | KRPPKRP
```

Nine of the twenty moved. By RANK: KQKR **14 -> 9** (leaf 4.12.15 ->
4.12.10), KRKN **5 -> 3**, KRKB **7 -> 4**, KXK **3 -> 2**; KRPKB **2 -> 5**
and KRKP **4 -> 7** fall.

**The MEASURE FIRST group and the unverifiable tail are identical under all
three derivations** — RAR-M15's, the pool's and Rarog's — so ranks 16 to 20 do
not depend on this choice at all.

**Ranks 3 to 8 do.** Their priorities span 0.0015 to 0.0051 and the pool
derivation orders them differently (KRPKB, KBPKB, KRKP, KRKN, KRKB against
KRKN, KRKB, KRPKB, KBPKB, KRKP). Read that band as a band. The instrument
separates rank 1 from rank 9; it does not separate rank 4 from rank 6, and no
leaf in 4.12 should be argued on the strength of its position inside it.

## What this does not do

Occurrence gates; it does not score, and it is not an Elo estimate
(`analysis/endgame_measurement_layers.md`). Tree occurrence is still measured
over 40 bench roots and is still weak — 4.11.5's finding stands untouched, and
the two instruments still contradict each other on KQKRPs, which is 4.41% of
the tree and 0.42% of games.
