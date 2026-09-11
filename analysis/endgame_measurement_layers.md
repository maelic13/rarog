# The four measurement layers (PLAN 4.10.5)

Rarog's endgame work produces numbers of four different kinds. They answer
different questions, have different units, and move independently. Mixing them
is the failure this document exists to prevent, and it is not hypothetical:
4.9a.7 nearly concluded a scale function did nothing because it was read off a
conversion number, and RAR-E14 found a conversion baseline that was really a
count of aborted games.

Every instrument states which layer it reports. Every claim states its layer,
its instrument, its node budget and its position set.

## The layers

| # | Layer | Unit | Question | Instrument |
|---|---|---|---|---|
| 1 | **Theory truth** | one MOVE | did it throw a won position? | Syzygy WDL, `endgame_truth.py` (`first_discard_ply`) |
| 2 | **Move quality** | one MOVE | are the moves progressing? | DTZ progress and win-preservation, `endgame_truth.py` |
| 3 | **Conversion** | one POSITION | was the win finished inside the rules? | `endgame_truth.py` (`converted`), `endgame_conversion.py` |
| 4 | **Game strength** | one GAME PAIR | does it win more games? | SPRT at a real TC, `tools/sprt.ps1` |

Plus one that is not a layer but gates all of them:

| — | **Occurrence** | frequency | can layers 1-3 ever reach layer 4? | `endgame_search_occurrence.py`, RAR-M15 |

And one that is not a measurement at all:

| — | **Drawn-share bias** | one POSITION | does the evaluator claim won what is drawn? | `endgame_drawn.py` |

Drawn-share bias is layer-3-shaped -- per position, static -- but it measures
the *complement* of conversion. A SCALE function is validated here and is
invisible in conversion; a VERDICT function is the other way round. See
PLAN 4.12 for which of the twenty reference functions is which.

## Precedence

Each rule below has a case behind it.

1. **Truth is an absolute veto and outranks conversion.** A vector converting
   103/138 was rejected in favour of one converting 98/138, on ONE live
   discard. A position thrown away is not tradeable against positions won.
2. **Conversion NEVER establishes strength.** Conversion moving 95 -> 144 of
   198 produced **-1.40 +/- 4.07 Elo**. If a conversion gain is offered as
   evidence of strength, the answer is a registered gate, not an argument.
3. **Move quality and conversion can move in OPPOSITE directions on the same
   change**, and both readings can be correct: an engine can take longer routes
   while still mating, which is exactly what RAR-E12 did to KBN-K (dtz progress
   0.7260 -> 0.6753, conversion not significantly changed).
4. **Strength never overrides truth.** A positive SPRT does not license a
   clean-win discard or a rule-50 regression. RAR-E12 was adopted with its
   KBN-K breach WAIVED TO A NAMED OWNER, not dismissed.
5. **Occurrence prioritises; it is never evidence of value.** A family absent
   from middlegame trees can still occur inside four-man endgame trees, and a
   census that includes endgame roots measures its own suite.

## Aggregation

**Layers are never aggregated.** There is no exchange rate between a truth
failure and a conversion gain, so there is no summary number that combines
them. Weighted aggregate conversion is a layer-3 number and stays inside
layer 3.

**When two layers disagree, say which decides -- the disagreement is usually
the finding.** RAR-E14's whole content is a layer-3 number that was measuring
something else while layers 1 and 2 were fine.

## What belongs to no layer

**Bench identity is provenance.** `bench 13` identifies the SEARCH. It is
necessary to know which binary produced a number and it proves nothing about
play: 4.9a.4 moved KBN-K conversion from 19.4% to 96.9% with a byte-identical
bench, because the 40 bench positions never reach a minor-piece bare-king mate
within depth 13. Never read "bench unchanged" as "behaviour unchanged" for an
evaluation term with a narrow activation.

**Static fit loss belongs to no layer either.** It is the fitting objective,
not a measurement of play.

## Run conditions every report must carry

A number without these is not comparable with any other number:

- **node budget** per move (PLAN 4.10.6: bracket it, do not guess it)
- **position set**, by cohort digest (PLAN 4.10.2)
- engine binary, threads, hash
- ply limit, where a playout is involved
- for layer 4 additionally: TC, book, adjudication policy, concurrency,
  affinity

`endgame_truth.py`, `endgame_conversion.py` and `endgame_drawn.py` each stamp a
`layer` field naming their layer, so a report cannot be read as answering a
question it never asked.

## Worked example: how to read 4.9a.7

KRPKR is a SCALE function.

- Layer 1: unchanged. No new discards.
- Layer 2: win-preservation improved to 0.9928.
- Layer 3 (conversion): **the wrong instrument for this change**, and its
  number was contaminated besides. Reading the step here shows it doing
  nothing, correctly and uselessly.
- Drawn-share bias: **37.1% -> 25.8%**. This is the result.
- Layer 4: not run; the change rides 4.12.22's dependency-complete gate.
- Occurrence: 10.04% of games on the board, and bench-visible
  (8,044,078 -> 6,901,489), which says the family is reached constantly in the
  search tree.

The step succeeded. Reading only layer 3 would have called it a failure.
