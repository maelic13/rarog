# Endgame evidence refresh after the accepted board head — RAR-M42 / 4.11b.18

## Decision

**Disposition, 2026-09-09: refreshed, floors PASS, and the registered 4.12 order
is verified UNCHANGED.** Section 4.11b closes. One non-blocking floor report is
handed to a named owner rather than absorbed.

Nothing was rerun reflexively: what could be reused was reused only after its
independence was *checked*, and one artifact that looked reusable turned out not
to be.

## Arms

Both binaries are the ones RAR-E15 gated, so no rebuild was needed:

| Arm | Engine | Git | Bench |
|---|---|---|---:|
| Accepted head | `rarog-411b-cand-pext-pgo.exe` | `b33d3ad` | 7,601,220 |
| 4.11 head | `rarog-411b-base-pext-pgo.exe` | `fd21612` | 6,901,489 |

Every instrument below is node-budgeted and seeded, so all of it is
deterministic and independent of host load.

## The reuse question, answered by checking rather than by reading

The registered 4.12 order (`endgame_ranking_v2.json`) consumes four inputs.
`endgame_ranking.py` itself runs no engine, but three of its inputs are
engine-measured:

| Input | Engine-measured? | Disposition |
|---|---|---|
| Colosseum board occurrence, 36,400 games | **No** — frozen PGN corpus classified by material | **Reused.** No engine can change it |
| `endgame_reference_results_v1.json` | Yes | Held constant across both arms so the census effect is isolated |
| `endgame_drawn_census_v1.json` | Yes | **Re-measured on both arms** |
| `endgame_tree_occurrence_v1.json` | Yes | Held constant across both arms |

**One assumption was wrong and checking caught it.** `analysis/endgame_measurement_layers.md`
describes drawn-share bias as "per position, static", which reads like an
evaluator-only measurement that an SEE repair cannot touch. It is not:
`endgame_drawn.py` takes `--engine` and searches every position at a fixed node
budget. Reusing it on the "static" reading would have been reuse on a false
premise.

## The mechanism, and why conversion alone would have misled

Over the frozen 83-position corpus `tests/endgames.epd`
(sha256 `e1043a6ace1193c8...`) at 60,000 nodes, 19 positions search differently
between the two heads. The split is exact:

| Cohort | Positions differing |
|---|---|
| Material on **both** sides | **19 / 55 = 34.5%** |
| Bare king | **0 / 28 = 0.0%** |

SEE only fires where captures exist, so a repair to it is invisible in bare-king
endings and visible everywhere else. Of the 19, ten differ only in node count;
eight changed score and PV; **one changed the best move** (KQ vs KRP,
`3Q4/8/K7/8/2p5/8/5r2/2k5 w - -`, `d8g5` at +552 becoming `d8d4` at +464).

This is why **conversion is the wrong instrument for this question**.
`endgame_conversion.py` covers KQ-K, KR-K, KBB-K and KBN-K — all bare-king — and
it came back **byte-identical** across both heads: same rates, same median mate
plies, same outcome buckets. Stopping there would have produced a confident
"nothing changed" that the contested families contradict.

## Layer 1 — the absolute veto is clean

`endgame_truth.py`, 19 families, 100 positions each, 60,000 nodes, seed
6200600. Both arms produce cohort digest `fe4866045506636f...`, matching the
digest recorded in `endgame_floors.json`, so the floor comparison is against the
same positions and is valid.

**Theory verdicts are identical on every one of the 19 families. No clean win is
newly discarded.** Under the precedence rule that truth outranks conversion and
is an absolute veto, that is the result that had to hold, and it holds.

## Floors — PASS on both arms

The 4.11 head reproduces the registered aggregate exactly, which validates the
instrument before the candidate is read:

| Arm | Weighted conversion | Reports below 2 SE |
|---|---|---|
| 4.11 head | 0.9300 -> **0.9300** (+0.0000) | 0 |
| Accepted head | 0.9300 -> **0.9336** (+0.0036, +0.4 SE) | 1, non-blocking |

Eleven of nineteen families moved, in both directions. The largest movers:

| Family | Metric | 4.11 head | Accepted head |
|---|---|---:|---:|
| KRP-KR | conversion | 0.9178 | **0.9726** |
| KQ-KR | conversion | 0.7700 | **0.8200** |
| KQ-KR | dtz progress | 0.3582 | **0.3990** (+2.7 SE, ratchet candidate) |
| KBP-KB | conversion | 0.9231 | 0.8846 |
| KBP-KN | conversion | 0.9649 | 0.9474 |
| KR-KN | win preserving | 0.9764 | 0.9598 |
| **KRP-KB** | **win preserving** | **0.9990** | **0.9949** (−2.2 SE) |

KRP-KR gaining 5.5 points of conversion is worth noting because it is the **top
family in the registered 4.12 order** at 10.04% board occurrence. The aggregate
is nonetheless +0.4 SE — not significant — and per the precedence rules a
conversion gain never establishes strength. RAR-E15 already supplied the layer-4
verdict for this cluster.

**The floors file was NOT updated.** KQ-KR's dtz progress qualifies as a ratchet
candidate, but KRP-KB's win-preserving rate fell 2.2 SE, and raising one floor
while lowering another in the same commit as the change that moved them is
exactly what the rule against relaxing a check alongside its own change forbids.
The floors pass as they stand.

## The 4.12 order is unchanged — rederived, not asserted

`endgame_ranking.py` was rerun with the same three held-constant inputs and each
arm's census:

- 4.11 head order **== registered `endgame_ranking_v2.json`**, all twenty
  families, exactly.
- Accepted head order **== 4.11 head order**, all twenty families, exactly.

The first equality is the load-bearing one: it proves the rederivation
reproduces the registered order before any conclusion is drawn from it.

**A correction worth recording.** The first rederivation used
`--occurrence-scope all` and produced a *different* order from the registered
v2. That was a wrong parameter, not a defect: `endgame_ranking_v2.json` records
its board-occurrence provenance as `Rating Tournament [engine], 10,000 games`,
so the matching scope is `engine`. With it, reproduction is exact. A rederivation
that fails to reproduce its own registered baseline is a broken instrument until
proven otherwise, and the difference was mine.

The registered 4.12 order therefore stands and needs no renumbering. 4.11.6's
retry trigger — *"if family occurrence is re-measured over real games at real
time control, this ranking is re-derived"* — is not fired by this work, because
occurrence was not re-measured; it comes from a frozen 36,400-game corpus.

## Versioned, not overwritten

New artifacts for the accepted head, with the 4.11-head originals left byte-identical:

- `tools/diag/endgame_drawn_census_v2.json`
- `tools/diag/endgame_truth_baseline_v2.json`

`endgame_drawn_census_v1.json`, `endgame_truth`'s v1 inputs, `endgame_floors.json`,
`endgame_ranking_v1.json` and `endgame_ranking_v2.json` are untouched.

## Owed obligation

**KRP-KB win-preserving rate, 0.9990 -> 0.9949 (−2.2 SE), needs an owner.** It
is reported and non-blocking, the floors pass, and it is not grounds to revert a
repair that RAR-E15 measured at +12.12 Elo. Owner: **4.12.6** (KRP-KB scale),
which already owns that family. Retry trigger: if a later change moves the same
rate below the 3 SE blocking threshold, it becomes blocking and must be
resolved before that change is accepted.

## Verification and limits

All instruments are node-budgeted and seeded, so none of this depends on host
load. Cohort digests match the registered floors. The 4.11 head reproduces the
registered census rates and the registered ranking exactly.

**Not done, and stated rather than implied:** `endgame_reference_results_v1.json`
and `endgame_tree_occurrence_v1.json` were held constant rather than
re-measured. They are engine-measured and therefore may also have moved. Holding
them constant is what isolates the census's effect on the order, and the order
is insensitive to a census shift many times larger than the one observed — but
this work does not establish that those two artifacts are themselves current.
Re-measuring them belongs with whichever leaf next depends on their absolute
values.
