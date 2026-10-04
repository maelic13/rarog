# Board comparison after 4.11b — RAR-M43

> **SUPERSEDED 2026-09-09 by 4.11b.19(d) —
> [board_comparison_411b19_2026-09-09.md](board_comparison_411b19_2026-09-09.md).**
> Two specific findings below are wrong and the raw numbers are kept anyway.
> **(1) The generation gap was partly the HARNESS, not the generator.** This
> document's "legal moves" and "legal captures" columns timed a 520-byte
> `MoveList` return copy inside the timed region on the Rarog side and not on
> Basilisk's, whose harness had removed it; the two columns never measured the
> same work. RAR-M44 found the copy in the emitted assembly and 4.11b.19(a)
> fixed the harness. **(2) The Elo arithmetic in "What the remaining gap is
> worth" priced only two of the four board regions**, omitting SEE (5.239%)
> and check queries (5.179%). Corrected in the new document, where closing the
> remaining generation and make/unmake gaps is worth about +1.5% rather than
> +2.9%, and about +2.7% with SEE included. Nothing below is deleted: the
> session, the control argument and every raw figure stand as measured.

Re-measurement of RAR-M20's board benchmark against Basilisk, on the accepted
board head. Zero games; this is board throughput only.

## Why the control is the load-bearing part

Four arms were measured **in one session**, not three:

| Arm | Binary | SHA-256 |
|---|---|---|
| `rarog-head` | built 2026-09-09 from `c1a7713` with the RAR-M20 recipe flags | `fd4c83af...` |
| `rarog-ca03a46` | **the exact binary RAR-M20 measured** | `40f8fa53...` |
| `basilisk` | **the exact binary RAR-M20 measured** (`d734766`) | `7eeaff0c...` |
| `reckless` | **the exact binary RAR-M20 measured** (`91b56c2`) | `449897a1...` |

Basilisk and Reckless were not rebuilt: their archived binaries hash-match the
RAR-M20 manifest, so re-timing them is strictly better than reusing their old
numbers, and it keeps every arm mutually comparable.

**The control did not reproduce RAR-M20, and that is the finding that governs
how everything else may be read.** The identical `ca03a46` binary measured
faster today than it did on 2026-09-05:

| Workload | RAR-M20 | Today | Offset |
|---|---:|---:|---:|
| legal moves | 447.131 | 450.149 | +0.7% |
| legal captures | 98.204 | 99.869 | +1.7% |
| make/unmake | 42.521 | 44.284 | +4.1% |
| threshold SEE | 46.676 | 49.012 | +5.0% |
| perft(4) startpos | 273.741 | 290.493 | +6.1% |
| two-ply simulation | 351.809 | 364.970 | +3.7% |

A session-level offset of up to 6.1% on unchanged code. **Comparing today's
Rarog against RAR-M20's recorded Basilisk figure would have attributed that
offset to 4.11b.** Every number below is therefore within-session only.

Also worth noting: `cargo bench` produced an executable with the *same
filename* as the original build, `board-12ca175dd86ea15e.exe`. Only the hash
distinguishes them, exactly as the recipe warns.

## All four arms, same session

Median M ops/s of three round medians, affinity mask 4, 150 ms warmup plus
eleven 150 ms samples per workload, three cyclic orders, host busy 5.01–6.25%
against the runner's 12% rejection threshold.

| Workload | ca03a46 | **head** | basilisk | reckless |
|---|---:|---:|---:|---:|
| legal moves | 450.15 | **444.99** | 650.47 | 348.27 |
| legal captures | 99.87 | **95.72** | 120.77 | 62.46 |
| make/unmake | 44.28 | **52.02** | 58.16 | 24.59 |
| threshold SEE | 49.01 | **45.05** | 60.77 | 42.37 |
| perft(4) startpos | 290.49 | **292.45** | 404.29 | 184.68 |
| two-ply simulation | 364.97 | **384.81** | 534.69 | 267.51 |

Threshold SEE remains **not comparable across engines** — different value
vectors, per RAR-M19 and RAR-M29. It is comparable between the two Rarog arms.

## What 4.11b actually did to the board

| Workload | head vs ca03a46 |
|---|---:|
| make/unmake | **+17.48%** |
| two-ply simulation | +5.44% |
| perft(4) startpos | +0.67% |
| legal moves | −1.15% |
| legal captures | −4.16% |
| threshold SEE | **−8.07%** |

**make/unmake +17.48%** is the accepted fused relocation, and it reproduces the
+17.97% that 4.11b.9's own board-v2 instrument measured — two independent
harnesses agreeing.

**Threshold SEE is 8.07% slower, and that is the repair, not noise.** The
4.11b.5 kernel added a per-candidate selected-king legality test that the old
one did not perform. 4.11b.9 saw this column down 1–2% and attributed it to code
layout; against the pre-repair binary the true cost is visible. It is bought and
paid for: RAR-E15 gated the whole package at **+12.12 ± 10.17 Elo**.

Legal moves −1.15% sits inside the round-to-round spread (2.81% for head, 5.80%
for ca03a46). Legal captures −4.16% is marginally outside it and was not a
target of any 4.11b leaf.

## The gap to Basilisk, then and now — both measured today

**SUPERSEDED — see the banner at the top.** The Rarog side of the two
generation rows includes a harness copy that Basilisk's harness did not pay.
The current table is in
[board_comparison_411b19_2026-09-09.md](board_comparison_411b19_2026-09-09.md).

How much faster Basilisk is:

| Workload | was (ca03a46) | **now (head)** | change |
|---|---:|---:|---:|
| make/unmake | 31.3% | **11.8%** | **19.5pp closed** |
| two-ply simulation | 46.5% | **38.9%** | 7.6pp closed |
| perft(4) startpos | 39.2% | **38.2%** | 0.9pp closed |
| legal moves | 44.5% | **46.2%** | 1.7pp wider |
| legal captures | 20.9% | **26.2%** | 5.2pp wider |

4.11b closed most of the make/unmake gap and a third of the two-ply gap. It did
nothing for generation, which is where the largest gap always was — 4.11b.8's
pin candidate was withdrawn and 4.11b.10/4.11b.11 both closed `NO_CHANGE`, so
that was the expected outcome, not a shortfall against a target.

Reckless remains slowest in every comparable column and is unchanged; it is
included because re-timing its archived binary costs nothing and keeps the
session self-consistent.

## What the remaining gap is worth

**SUPERSEDED — this arithmetic omitted SEE and check queries, and its
generation gap was partly the harness.** Corrected in
[board_comparison_411b19_2026-09-09.md](board_comparison_411b19_2026-09-09.md).

Board throughput is not search speed. RAR-M36 puts board work at **23.65%** of
process time: generation and legality 6.556%, make/unmake 6.677%, SEE 5.239%,
check queries 5.179%.

Closing the **remaining** generation and make/unmake gaps to Basilisk entirely —
generation 1.462x faster, make/unmake 1.118x — gives:

```
1 / [ 1 − 0.06556 − 0.06677 + 0.06556/1.462 + 0.06677/1.118 ] = 1.0286
```

**about +2.9% whole-search NPS, ~+5.7 Elo** at the project's ~2 Elo per 1% NPS
constant. Before 4.11b the same arithmetic gave roughly +4.4%. So the board work
still on the table against Basilisk is worth single-digit Elo, and 4.11b has
already banked **+1.421% [+0.953%, +1.764%]** of it (RAR-M41), about +2.8 Elo.

That is the honest scale, and it is why 4.11b's bounded optimization found so
little to accept: no individual board region is large enough for a big win, and
two of the three candidates tried measured negative or unresolvable.

## Limits

- **Within-session only.** The control shifted up to 6.1% on unchanged code, so
  none of these absolute figures should be differenced against RAR-M20's.
- **Non-PGO, native, single-threaded microbenchmark.** It measures board
  primitives, not playing strength, and no Elo is claimed from it.
- **Threshold SEE is intra-Rarog only** across the two Rarog arms.
- Round-to-round spread reached 9.41% for Reckless and 5.80% for ca03a46, so
  differences below roughly 5% on a single column are not resolved by this
  instrument.

## Evidence

`tools/results/board-compare-20260909/` (ignored, local): `run_comparison.py`
with the four-arm `bins` map, `manifest.json` with all twelve runs and binary
hashes, twelve raw round outputs, `run.log`. Recipe and archived binaries:
`analysis/board_benchmark_recipe_2026-09-05.md` and
`D:/chess/results/board-audit-20260905/binaries/`.
