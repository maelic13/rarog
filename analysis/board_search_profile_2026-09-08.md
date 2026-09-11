# Refreshed full-search board profile — RAR-M36 / recipe recovery

Supersedes the shares in `board_search_profile_2026-09-07.md` (RAR-M30) for the
current head. RAR-M30 is not withdrawn: it measured `02420dc` correctly, and the
two agree closely everywhere except the region 4.11b.9 changed.

## The recipe, recovered

RAR-M30's per-sample attribution was not a setting — it was a side effect of a
bug, and fixing that bug silently broke the instrument.

Before `952711f`, `_NT_SYMBOL_PATH` pointed at the directory holding
`rarog-production.pdb`. rustc embeds the name `rarog.pdb` in the executable, so
xperf could not discover it and emitted **one row per sampled address**, named
`***unknown***`. `summarize_board_search_etw.py` (`ec9e0de`) exists precisely to
recover those addresses with llvm-symbolizer, which yields the **complete inline
chain** and lets a hot inlined helper be charged to its board *caller* rather
than its leaf.

`952711f` "Fix ETW profile symbol resolution" made xperf resolve symbols. xperf
then aggregated **per function**, board work inlined into `negamax` or
`evaluate` was charged to those functions, and the summarizer — reading a fixed
column index that had been correct for the per-address table — began resolving
`limit`, the byte one past the end of each function. It reported "100% of engine
samples resolved" while being entirely wrong.

**The working recipe is to deny xperf symbols on purpose:**

```
_NT_SYMBOL_PATH=<empty dir>  _NT_SYMCACHE_PATH=<empty dir>  and no rarog.pdb
beside the executable
  -> xperf -i <etl> -o <report> -symbols -a stack -butterfly 100 -process <exe>
  -> 6,224 rows for one cohort, base == limit, size 0, all ***unknown***
  -> summarize_board_search_etw.py recovers them with llvm-symbolizer
```

An empty `_NT_SYMBOL_PATH` alone is **not** enough: xperf reuses its symcache,
and a PDB beside the executable is found by dbghelp before either. All three
must be denied. Afterwards the PDB must be restored beside the executable under
its embedded name, because llvm-symbolizer resolves by that name, not by `--pdb`.

Both failure directions are now guarded and detected from the data, not the
header: `base == limit, size == 0` is per-address and accepted; any hits in rows
with non-zero size are per-function and **refused** with the regeneration
recipe in the message.

## Refreshed shares at `2d621ff`

162,846 process samples, five cohorts, 600,000 nodes, 5 repeats, production
`a3cca8dc...`, PDB `c61e93e3...`.

| Region | RAR-M30 (`02420dc`) | **RAR-M36 (head)** |
|---|---:|---:|
| generation and legality | 6.751% | **6.556%** |
| make/unmake | 7.143% | **6.677%** |
| SEE | 5.304% | **5.239%** |
| check queries | 5.177% | **5.179%** |

| Mechanism | RAR-M30 | **RAR-M36** |
|---|---:|---:|
| piece relocation helpers | 2.998% | **2.752%** |
| gives_check | — | **1.654%** |
| check_info | 0.912% | **1.026%** |
| compute_pinned | 1.003% | **0.979%** |
| king square lookup | 0.544% | **0.502%** |

**The instrument validates against an independent result.** RAR-M33 measured
+0.876% whole-search from an ~18% local make/unmake gain, which requires that
region to be about **6.3%**. This profile reads **6.677%**, down from RAR-M30's
7.143% — the drop is 4.11b.9, measured by a second instrument that knew nothing
about it. `check_queries` reproduces to within 0.002 percentage points.

**One stale marker was found and fixed.** `piece_relocation_helpers` keyed only
on `::remove_piece` and `::add_piece`, so after 4.11b.9 fused the QUIET path
into `Board::move_piece` it under-read the region at 1.419%. With `::move_piece`
added it reads 2.752%, consistent with RAR-M30's 2.998% reduced by the accepted
speedup. A marker list is a silent liability whenever a mechanism is renamed.

## Per-function view — where SEE cost actually sits

From the symbolized report (exclusive samples, 151,826 engine samples across
five cohorts). This view aggregates by function and so under-attributes inlined
work; it is recorded for the one thing it shows clearly.

| exclusive | function |
|---:|---|
| 29.49% | `rarog::eval::Evaluator::evaluate` |
| 23.07% | `rarog::search::Searcher::negamax` |
| 6.54% | `rarog::search::Searcher::append_scored_moves` |
| 6.22% | `rarog::search::Searcher::quiescence` |
| **4.35%** | **`rarog::board::board::Board::see_recapturer`** |
| 3.76% | `rarog::board::board::Board::make_move_inner` |
| 1.67% | `rarog::board::board::Board::is_attacked_with_occ` |
| 1.10% | `rarog::board::board::Board::check_info` |
| 1.05% | `rarog::board::movegen::compute_pinned` |
| **0.87%** | **`rarog::board::board::Board::see_ge_impl`** |

**`see_recapturer` outweighs `see_ge_impl` five to one.** `see_recapturer` is
where the per-candidate selected-king legality test lives. This is independent
measured support for RAR-M35's conclusion, which was reached from the code and
from a failed candidate: SEE's cost is in the legality test, not in the attacker
set that RAR-M35 tried and failed to make incremental.

It does **not** split the two `attackers_to_color` calls inside
`see_recapturer` — both inline into it, so no profile at this granularity can.
That still needs a counter or an `#[inline(never)]` probe.

## What this settles

- **4.11b.12** king-square caching: **0.502%**, a 2x-local ceiling of 0.25%.
  Down from 0.544%, and geometry work did not make it material.
- **Future SEE work**: 5.239% region, dominated by `see_recapturer`.
- **No leaf is blocked** on further profiling.
