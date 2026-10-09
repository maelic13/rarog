# RAR-S22 — Phase-4.2 opening static audit of the TT producer/consumer graph, plus a re-run of RAR-S21's …

Indexed under *3. Search and selectivity › Rejected, neutral or deferred* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Phase-4.2 opening static audit of the TT producer/consumer graph, plus a re-run of RAR-S21's reading on a freshly built diag binary.

## Result / disposition

**Observation.** The reading reproduced RAR-S21 digit-for-digit (fingerprint 6,502,902, EBF 2.449), so the sampled map is stable across a rebuild. Static findings: `TtEntry.flag_age` is **fully allocated** — 5 bits age (`0xF8`), 1 bit `is_pv`, 2 bits bound — so Plan 4.2's assumed "spare `flag_age` capacity" does not exist. 7 store sites and 13 read sites were enumerated. Sampled store mix: main 803 (7 exact / 508 lower / 288 upper), qsearch 1,673, ProbCut 14 — i.e. **67% of sampled stores are depth-0 qsearch entries and 37% are bare stand-pat**. ⚠ RAR-S23's exact census confirms the depth-0 and stand-pat shares (67.50% / 35.87%) but shows the ProbCut share here understated 2.4x; prefer the census. `singular_probcut_depth_match` was 32 of 101 sampled singular attempts, meaning a third of singular decisions read an entry at exactly ProbCut's `depth-3` + `Lower` signature, which cannot be attributed to a producer without provenance. `EvalPruneTtMinDepth` is seeded 0, so those depth-0 entries can refine pruning at any depth.

## Conditional lesson and retry trigger

Under this state the shortage is attribution, not counting: the coincidence rate is measurable while the producer is not, which is the argument for typed provenance rather than for tightening a depth threshold blind. Also recorded: `bench` shares one table across its 40 positions and ages it by 8 per position, wrapping after 31, so **any change to the age field's width is bench-visible and is a behaviour change, not a free refactor** — the cheapest 1-bit provenance slot (age 5→4 bits) therefore needs a strength gate, not a fingerprint check. Retry/extend when 4.3–4.4 need a persisted producer class.

## Source

`src/tt.rs`; `src/search.rs`; Plan 4.2
