# Full-search board profile — RAR-M30 / 4.11b.7

## Result

Board work is material but divided. Across 161,989 full-process ETW samples,
generation/legal delivery is **6.751%**, make/unmake **7.143%**, check queries
**5.177%**, and SEE **5.304%**. These mutually exclusive regions total
**24.375%**. A further **12.997%** is board storage/attack access attributed to
evaluation or other consumers and is not assigned to a board leaf merely
because its innermost helper lives in `board`.

The first implementation leaf remains **4.11b.8**. It owns a measured 6.751%
region, has the largest independent cross-engine prior, and does not depend on
the state-lifetime design required by 4.11b.10. The measured opportunity order
after that is SEE, relocation, shared geometry, king lookup, then history
allocation; dependency order still keeps shared geometry before SEE and king
lookup.

## Frozen protocol and identity

The source was `02420dcdecf5baff5c33859832c09e3e7f581828`; the 20-position
suite contains four roots each for opening, middlegame, check-heavy, promotion
and sparse endgame. Diagnostic counters used three repeats at 600,000 nodes;
ETW used five. The suite SHA-256 is
`0c8cefdfb4917295a2dfc26ad4acbde8fcd1c4e6e2c8760f11f57c01e6b153e3`.

The exact production and diagnostic executable hashes are respectively
`3c81ef95f3c1d4ab0f063eb814ed5f51b8ef57da7e3b5a579500cbb1d904dfd0`
and `aaeda61886789333503ab0984fb54a44ffea44758449995a84f07218425d42e1`.
All **60/60** fixed-node instrumentation-off comparisons matched depth,
seldepth, reported nodes, score type/value and best move. PV and ponder move
were not retained or compared. The remote runner also passed six harness tests,
debug and release Rust suites, formatting, and all-feature/all-target Clippy.

The received archive SHA-256 is
`2beb0f272df7eca6d89f800e1f663378395ce9ecaffad19335901159331e81cb`.
Its first xperf reports contained unresolved engine names because the PDB had
been archived as `rarog-production.pdb` rather than the embedded `rarog.pdb`.
Commit `952711f` fixes future reports and rejects this state. For this archive,
the matching PDB was restored under its embedded name and every engine RVA was
resolved with Visual Studio LLVM's symbolizer. All **151,142/151,142** engine
samples resolved. `tools/diag/summarize_board_search_etw.py` performs and
validates that recovery; the machine-readable result is
`analysis/artifacts/board-search-profile-20260907/summary.json`.

## Full-search time shares

Percentages use all process samples, not only resolved engine frames. Regions
are mutually exclusive. Each exclusive instruction address is classified with
its complete LLVM inline context, so an inlined shared attack helper is charged
to SEE, generation, make or check work when that consumer is visible.

| cohort | samples | generation / legality | make / unmake | check queries | SEE |
|---|---:|---:|---:|---:|---:|
| opening | 47,326 | 5.900% | 5.401% | 2.688% | 6.863% |
| middlegame | 31,291 | 7.184% | 7.980% | 5.430% | 6.206% |
| check-heavy | 37,174 | 6.776% | 7.002% | 4.425% | 6.706% |
| promotion | 23,823 | 7.921% | 8.366% | 8.080% | 2.972% |
| sparse endgame | 22,375 | 6.659% | 8.590% | 8.246% | 0.898% |
| **sample-weighted** | **161,989** | **6.751%** | **7.143%** | **5.177%** | **5.304%** |

The leaf-specific overlapping views are: add/remove relocation helpers
**2.998%**, `compute_pinned` **1.003%**, `check_info` **0.912%**,
`gives_check` **1.785%**, and `king_sq` **0.544%**. These are deliberately not
summed: a sampled inline path may contain more than one named mechanism.

Sampling error is small enough for leaf ordering, not for tiny optimization
claims. For example, a 5% share has a binomial standard error of about 0.054
percentage points over 161,989 samples; cohort and workload variation is the
larger uncertainty. Every candidate therefore still needs a controlled
within-Rarog full-search NPS measurement.

## Hot caller frequencies

The counters aggregate 30,604,224 reported nodes. One root legal-generation
call per search occurs before the diagnostic reset, so 60 such calls are
excluded from the atomic counts but included in ETW time.

| mechanism | calls | interpretation |
|---|---:|---|
| full generation | 3,395,724 | full-list paths, distinct from staged search delivery |
| standalone captures | 3,818,253 | capture-only callers; not the staged picker |
| staged captures / quiets | 3,547,548 / 2,083,335 | search picker shares its pin state across stages |
| `compute_pinned` | 9,474,168 | residual geometry work after existing staged sharing |
| `check_info` | 11,660,631 | check geometry construction |
| fast / full `gives_check` | 120,463,353 / 754,812 | 99.38% uses the hinted fast path |
| full / threshold SEE | 1,882,485 / 24,161,913 | 92.77% is threshold SEE |
| plain / checked real make | 2,688,294 / 21,787,521 | 89.02% uses `make_move_with_check` |
| real unmake | 24,475,815 | exactly matches all real makes |
| null make | 1,242,339 | separate null path |
| history push / growth | 25,718,154 / **0** | no hot allocation event in the measured searches |

This mechanically rejects two misleading microbench proxies. Plain perft make
represents only 10.98% of real search makes, and standalone capture generation
does not represent the shared-pin staged picker.

## Written time budget and priorities

The table gives the maximum whole-search speedup if the named measured region
could be made twice as fast with no tree or cache change. It is an Amdahl
ceiling, not a forecast.

| leaf | directly measured region | share | 2x-local whole-search ceiling | priority decision |
|---|---|---:|---:|---|
| 4.11b.8 legal generation/delivery | generation and legality | 6.751% | 3.49% | **next**; broad measured region and strongest prior |
| 4.11b.11 SEE kernel | SEE | 5.304% | 2.72% | high opportunity, after shared-geometry contract |
| 4.11b.9 fused relocation | add/remove subset | 2.998% | 1.52% | bounded candidate after 4.11b.8 |
| 4.11b.10 shared pin/check data | pinned 1.003%; check-info 0.912% | separate overlapping shares | measure each consumer | dependency before SEE/king decisions |
| 4.11b.12 king-square cache | `king_sq` | 0.544% | 0.27% | low; only after geometry work leaves it material |
| 4.11b.13 history capacity | allocation events | **0% observed** | 0% | contract/correctness work, not a speed candidate |

Make/unmake as a whole has a 3.70% twofold ceiling, but 4.11b.9 changes only
the measured 2.998% relocation subset. Check queries as a whole have a 2.66%
twofold ceiling, but 4.11b.10 must price cache maintenance and invalidation
against the smaller overlapping mechanisms rather than claim that entire
region. No Elo or strength claim is made and no games were run.

## Reproduction

Run the original one-command capture from an elevated PowerShell 7 prompt:

```powershell
pwsh -File tools/diag/run_board_search_profile_411b7.ps1
```

If a pre-`952711f` archive needs symbol recovery, put its exact production PE
and matching PDB together, preserve the PDB's embedded `rarog.pdb` name, then
run:

```powershell
python tools/diag/summarize_board_search_etw.py `
  --exe <archive>\etw\rarog-production.exe `
  --pdb <archive>\etw\rarog.pdb `
  --symbolizer <llvm-bin>\llvm-symbolizer.exe `
  --output <archive>\etw\resolved-summary.json `
  <archive>\etw\opening-butterfly.txt `
  <archive>\etw\middlegame-butterfly.txt `
  <archive>\etw\check-heavy-butterfly.txt `
  <archive>\etw\promotion-butterfly.txt `
  <archive>\etw\sparse-endgame-butterfly.txt
```

## Symbolization defect found 2026-09-08 (affects re-runs, not this record)

A refresh of this profile at `cf10a46` produced impossible region shares —
`make_unmake` **0.756%**, `see` **0.464%**, `king_square_lookup` **27.496%**,
with `core::num::trailing_zeros` as a 38% exclusive leaf. The numbers were
discarded and are not recorded anywhere as measurements.

**First diagnosis was WRONG and is retracted.** This document briefly blamed a
stale `rarog.pdb` shadowing the correct one in `dbghelp`'s search order. That is
not what happened: xperf symbolized correctly the whole time. Its own report
already named `rarog::eval::Evaluator::evaluate` with the same 18,503 exclusive
hits that the broken summary mislabelled `core::num::trailing_zeros`. Placing
the matching PDB beside the executable changed nothing about the reports, which
came back byte-identical. The PDB hygiene committed for that wrong reason is
harmless and kept, but it fixed nothing.

**Actual root cause: a report-schema mismatch.** xperf emits two different
exclusive-hits tables.

| mode | columns |
|---|---|
| without `-symbols` | `function, hits, percent, a, b, `**`address`** |
| with `-symbols` | `function name, exclusivehits, totalpercent, inclusivehits, `**`base`**`, limit, size` |

`summarize_board_search_etw.py` is built for the first: **one row per sampled
address**, which is what lets it recover a complete inline chain and charge a
hot inlined helper to its board *caller* rather than its leaf. It read a fixed
index 5, correct for that schema. The ETW runner had since been changed to pass
`-symbols`, which switches to the second schema, where index 5 is `limit` — the
byte one past the end of each function. Every lookup therefore resolved into the
next function or into padding, and the tool reported a complete-looking result
with 100% of samples "resolved".

**Reading `base` instead does not rescue it.** The symbolized schema has one row
per *function*, so board work inlined into a large search function is charged to
that function and never appears under its own region. Corrected to `base`, the
run read make/unmake at **3.59%**, against the **6.3%** that RAR-M33's measured
speedup independently requires. Still wrong, just less obviously.

**Why this record is unaffected.** RAR-M30 resolved **151,142 individual engine
samples** — per-address rows, the schema the tool supports. Today's capture
produced only ~562 function rows for the same workload. RAR-M30 also
cross-checks against an independent result: its **7.143%** make/unmake matches
the **6.3%** implied by RAR-M33's +0.876% whole-search gain from an ~18% local
speedup. **The shares in this document stand, and no refreshed shares have been
recorded from the 2026-09-08 capture.**

**Fix applied.** `summarize_board_search_etw.py` now resolves its columns by
header name and **refuses** the per-function schema outright, naming what it
needs, instead of emitting a plausible number from it. That is the durable part:
this tool reported "100% of engine samples resolved" while being completely
wrong, twice, and only a cross-check against an independently measured result
caught it. It can no longer fail that way silently. Covered by
`test_summarize_board_search_etw.py` (5/5).

**Attempted fix that was WRONG and is reverted.** Dropping xperf's `-symbols`
does **not** produce one row per sampled address. It produces one row per
MODULE: the entire Rarog image collapses to a single `***unknown***` row with
all 45,909 engine hits, base `0x0`, limit = image size. That is strictly less
attribution than the symbolized report, so `board_search_profile_etw.ps1` was
reverted to emitting the single `-symbols` report it had before.

**RESOLVED 2026-09-08 — the recipe was recovered.** RAR-M30's per-sample
attribution was a side effect of xperf being unable to find the PDB, which
`952711f` then "fixed". Denying xperf symbols deliberately — empty
`_NT_SYMBOL_PATH`, empty `_NT_SYMCACHE_PATH`, and no `rarog.pdb` beside the
executable — restores the per-address table this tool is built for. The profile
has been refreshed at head and reproduces this document's shares closely; see
`analysis/board_search_profile_2026-09-08.md` (RAR-M36). The shares here remain
valid for `02420dc` and are superseded only for the current head.
