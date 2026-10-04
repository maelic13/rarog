# 8.12(c) Profile pass — where Rarog's search time actually goes

**Date:** 2026-07-23 · **Binary:** non-PGO release at `4b83e5a` (bench
5,480,624) · **Machine:** idle Zen 3, SPSA stopped · **Harness:**
`tools/profile_probe.py` + `tools/profile_attrib.ps1`

## Method: duplicate-work attribution (no profiler required)

`samply`/`perf` are unavailable on this box and ETW (`xperf`/`wpr`) needs
elevation, which this shell does not have. Instead each region was measured by
**running it twice** and discarding the second result behind `black_box`:

> if region R is fraction `f` of runtime, doubling it gives
> `NPS_base / NPS_dup = 1 + f`, so **`f = NPS_base/NPS_dup − 1`**

Properties that make this trustworthy:

- **Every probe binary was verified bench-identical** (5,480,624). The
  duplicate's result is thrown away, so behaviour — and therefore the node
  workload — cannot change. A probe that moved the fingerprint would be
  rejected as non-behaviour-preserving; none did.
- Effects are large by construction (doubling a 15% region costs 13% NPS), so
  they clear the noise floor that made the 10.3 sub-1% measurements so
  painful.
- 5 rounds × best-of-3 bench per binary, base re-measured between every probe
  and compared on medians.

**Caveat, stated honestly:** doubling a region also doubles its cache
footprint, and the second run reads warm caches. Both effects are small but
opposite, so treat these as accurate to roughly ±20% *relative* — good enough
to rank levers, not to quote as exact percentages.

## Results (share of total search runtime)

| Region | share | verdict |
|---|---|---|
| **`eval_piece_activity`** | **15.3%** | **the dominant lever** |
| `append_scored_moves` (quiet scoring) | 5.4% | new candidate, was not on the list |
| `make_move` (+unmake) | 4.3% | board mutation in search conditions |
| `eval_imbalance` | 2.7% | |
| `generate_legal_quiets_pinned` | 2.3% | |
| material+PST+phase walk | 2.2% | corroborates the 8.12(a) rejection |
| `eval_pawns` (incl. its cache) | 1.7% | |
| `generate_legal_captures_pinned` | 1.1% | |
| `move_gives_check` | 0.6% | |
| `tt.probe` (both sites) | 0.6% | |
| whole `evaluate()` re-call | 1.4% | *= cost of a cache HIT, see below* |

## The four questions 8.12(c) was posed — three answered NO

1. **`eval_piece_activity` share → 15.3%. CONFIRMED as the target.** It is
   3× the next-largest region and larger than all move generation, TT probing
   and check detection combined.
2. **Staged movegen remainder → 3.4% total** (quiets 2.3% + captures 1.1%).
   **Killed as a lever**, and this also retires the thread opened by
   `board_perft_compare.md`: Basilisk leads us by ~14% on sparse positions,
   but since all movegen is 3.4% of search time, closing that entire gap would
   buy ≈0.2%. The board is not where our time goes.
3. **TT-probe / prefetch coverage → 0.6%. Killed.** The 10.3 prefetch work
   left nothing on the table.
4. **Delayed direct-check detection in LMR → `move_gives_check` is 0.6%
   total. Killed.** 10.3(2)+(3) already reduced this to noise; there is no
   remaining prize behind a *delayed* variant.

## The eval cache: measured, and it is NOT a capacity problem

Instrumented over a full bench:

- **hit rate 11.6%** (219,098 hits / 1,670,283 misses)
- **98.0% of misses land on an occupied slot with a different key**

That looks exactly like a table too small — so it was swept:

| entries | memory/thread | hit rate | NPS |
|---|---|---|---|
| 32,768 (current) | 768 KB | 11.6% | **3,113,990** |
| 262,144 | 6 MB | 12.5% | 3,032,996 |
| 1,048,576 | 24 MB | 12.7% | 2,989,974 |
| 4,194,304 | 96 MB | 12.7% | 2,973,751 |

**A 128× bigger table buys 1.1 points of hit rate and LOSES 4.5% NPS.** The
collisions are not capacity pressure — positions genuinely do not repeat
inside the search, so the reuse ceiling is ~13%. **This kills the eval-cache
half of 8.12(d) outright**; do not revisit sizing without new evidence.

The cache still earns its place at the current size: it converts ~219k full
evals into hits (~2.9% of runtime saved) for ~1.4% of lookup overhead, so
roughly **+1.5% net**. Keep exactly as is.

## Lazy eval has headroom

**17.0% of eval-cache misses take the lazy path** (284,488 lazy vs 1,385,795
full). So 83% of evaluated nodes pay the full `eval_piece_activity` price.
`LazyMargin` is already a UCI-exposed, SPSA-tunable knob — widening it trades
eval accuracy for the 15.3% region directly, and unlike everything else here
that is a *strength* trade, so it needs an SPRT rather than an NPS gate.

## What becomes new sub-steps

- **(f) attack `eval_piece_activity` — 15.3%.** Two independent routes:
  (i) make it cheaper (mobility/king-danger loops, early exits), a pure speed
  change gated on NPS; (ii) run it less often via a re-tuned `LazyMargin`,
  a strength trade gated by SPRT. Route (ii) is nearly free to try since the
  knob already exists.
- **(g) lazy/staged quiet scoring — 5.4%.** `append_scored_moves` scores
  *every* quiet up-front, but the move picker frequently cuts off after a
  handful. Score on demand, or in cheap batches.
- **(d) eval-cache sizing — CLOSED, measured negative** (table above).

Deliberately NOT pursued: movegen, TT probe, check detection, material/PST
walk — all measured at or below 3.4% with no plausible large win.

## ETW sampling profile — cross-check (added same day)

The user ran `tools/profile_etw.ps1` from an elevated shell: xperf kernel
sampling at **8.2 kHz** over `bench 13 5`, **79,764 samples** in `rarog.exe`,
symbols from a local PDB (release build with `debug=2`; debug info lives in the
PDB and does not change codegen — verified bench-identical). Report generated
with `xperf -a stack -butterfly -process rarog.exe`.

**Flat profile, exclusive hits (top of 79,764):**

| symbol | excl. | note |
|---|---|---|
| `Evaluator::evaluate` | **29.7%** | whole eval, inlined by fat LTO |
| `Searcher::negamax` | 15.1% | search control flow itself |
| `Searcher::quiescence` | 8.7% | |
| `Searcher::append_scored_moves` | **7.4%** | = sub-step (g) |
| `enum2$::next` (`MovePicker::next`) | **6.6%** | **new — not on any list** |
| `Searcher::score_tactical_move` | 4.4% | per-capture scoring incl. SEE |
| `Board::make_move_inner` | 3.8% | |
| `movegen::gen_captures_with_pin` | 1.9% | |
| `Searcher::corrected_eval_from_raw` | 1.7% | correction-history lookups |
| `Board::see_pins` | **1.6%** | **sizes sub-step (b)** |
| `Searcher::store_tt` | 1.4% | |
| `Board::is_attacked_with_occ` | 1.1% | |
| `movegen::compute_pinned` | 0.7% | |
| `Board::check_info` | 0.7% | |
| `vcruntime140` (unresolved CRT) | ~6.6% | memcpy/memset, see below |

### The two methods agree on ranking, and disagree on magnitude predictably

| region | duplicate-work | ETW exclusive |
|---|---|---|
| eval (all) | ~21.9% (sum of parts) | 29.7% |
| quiet scoring | 5.4% | 7.4% |
| make_move | 4.3% | 3.8% |
| capture generation | 1.1% | 1.9% |

Same ordering, but duplicate-work **understates eval by ~25% relative**. That
is exactly the warm-cache bias flagged in the caveat above: the duplicated run
reads tables the first run just pulled into L1/L2, so its *marginal* cost is
below the true first-run cost. Rule of thumb going forward: **duplicate-work
is a reliable ranker and a conservative lower bound on share**, not an exact
measure. Where the two disagree, prefer ETW.

### What ETW found that duplicate-work could not

- **`MovePicker::next` = 6.6%** — the picker's own staging/selection logic,
  *after* 10.3(8d) already optimised `pick_next`. Nothing probed this because
  nobody suspected it. Now the #3 region in the engine.
- **`score_tactical_move` = 4.4%** and **`see_pins` = 1.6%** — together these
  size sub-step **(b)** honestly: the SEE pin cache can recover at most the
  repeated part of 1.6%, so **≈0.5–1.0%**, matching the original estimate
  rather than beating it.
- **`corrected_eval_from_raw` = 1.7%** — correction-history lookups, never on
  any candidate list.
- **`negamax` + `quiescence` exclusive = 23.8%** — the search's own control
  flow (pruning conditions, bookkeeping, stack updates). Large but diffuse;
  no single hot spot to attack.
- **~6.6% in unresolved `vcruntime140`** (memcpy/memset). **Not attributed**:
  the hits are spread across dozens of call sites, the largest being 60
  samples (0.075%), so there is no single owner. Plausible source is
  by-value returns of the big move lists (`MoveList` ~1 KB,
  `ScoredMoveList` ~4 KB), which would make it (g)-adjacent — but that is a
  hypothesis, not a finding. Do not act on it without measuring.

### Build caveat (applies to BOTH methods)

Both the duplicate-work probes and this ETW trace ran on the **plain
`cargo build --release`** binary — which on this host is the *generic/magic*
slider build with no PGO (2.88 M nps here vs ~3.3 M for shipped pext-PGO).
Shares on the shipped binary will differ somewhat, most likely *against*
eval's favour: `eval_piece_activity` leans on slider attacks for mobility, and
PEXT makes those cheaper. The ranking is safe; treat the absolute percentages
as ±few points when reasoning about the pext-PGO binary.

## Reproducing

```
python tools/profile_probe.py list          # regions
python tools/profile_probe.py <region>      # apply one probe
cargo build --release                       # verify bench is UNCHANGED
python tools/profile_probe.py revert
pwsh tools/profile_attrib.ps1 -Reps 3 -Rounds 5
```

Add a region by appending an anchor/replacement pair to `PROBES`. The anchor
must appear exactly once, and the duplicated work must be side-effect-free —
bench identity is the check that it was.
