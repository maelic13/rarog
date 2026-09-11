# Board comparison after 4.11b.19 — RAR-M44(d)

Supersedes the gap table in
[board_comparison_2026-09-09.md](board_comparison_2026-09-09.md) (RAR-M43).
Four arms, one session, zero games; board throughput only.

**Measured twice, and both sessions are kept.** The first ran on a head that
included the 4.11b.19(c) generator candidates. Their registered pooled-PGO run
then measured **−0.55% [−0.76%, −0.30%]** whole-search NPS and reverted them,
so that head no longer exists and its table is not the record. The current
table is the second session, on the (b) head. The first session is retained
below because the difference between the two is the most useful thing this
leaf measured.

## The control reproduced, which is what makes this session readable

RAR-M43's own control did **not** reproduce RAR-M20: the identical `ca03a46`
binary measured 0.7–6.1% faster on that day, so RAR-M43 had to be read
within-session only. Neither of today's sessions has that problem.

| Workload | `ca03a46` today | in RAR-M43 | drift | Basilisk drift |
|---|---:|---:|---:|---:|
| legal moves | 447.47 | 450.15 | −0.59% | −1.21% |
| legal captures | 99.31 | 99.87 | −0.55% | −0.61% |
| make/unmake | 43.70 | 44.28 | −1.32% | +0.14% |
| threshold SEE | 49.04 | 49.01 | +0.05% | −0.55% |
| perft(4) startpos | 289.84 | 290.49 | −0.23% | −0.41% |
| two-ply simulation | 371.59 | 364.97 | +1.81% | +0.37% |

Two independent binaries re-time inside about ±1.8%, so **this session and
RAR-M43's are comparable** and the Basilisk readings can be read across them.
Host busy stayed 5.1–5.8% against the recipe's 12% rejection threshold.

## All four arms, one session, medians of three cyclic rounds (M ops/s)

| Workload | Rarog head | Rarog `ca03a46` | Basilisk | Reckless |
|---|---:|---:|---:|---:|
| legal moves | **482.53** | 447.47 | 642.61 | 347.22 |
| legal captures | **126.96** | 99.31 | 120.03 | 62.41 |
| make/unmake | **51.87** | 43.70 | 58.24 | 25.04 |
| threshold SEE | 44.78 | 49.04 | 60.44 | 41.94 |
| perft(4) startpos | **316.53** | 289.84 | 402.64 | 184.09 |
| two-ply simulation | **403.12** | 371.59 | 536.67 | 267.84 |

## The gap to Basilisk: RAR-M43's table, superseded

How much faster Basilisk is. RAR-M43's "now" column becomes the "was" column.

| Workload | RAR-M20 (`ca03a46`) | RAR-M43 | **RAR-M44(d)** | closed since RAR-M43 |
|---|---:|---:|---:|---:|
| legal moves | 44.5% | 46.2% | **33.2%** | 13.0pp |
| legal captures | 20.9% | 26.2% | **−5.5%** | **31.7pp — Rarog is now 5.8% ahead** |
| two-ply simulation | 46.5% | 38.9% | **33.1%** | 5.8pp |
| perft(4) startpos | 39.2% | 38.2% | **27.2%** | 11.0pp |
| make/unmake | 31.3% | 11.8% | **12.3%** | −0.5pp |
| threshold SEE | — | 34.9% | **35.0%** | −0.1pp |

Every bit of that movement is 4.11b.19(a)+(b) removing a copy from BOTH the
harness and the search. Make/unmake and SEE were not touched by this leaf and
did not move, which is the internal consistency check on the table. Reckless
is unchanged and remains slowest in every comparable column.

## What 4.11b.19 did, measured against the same `ca03a46` control

| Workload | RAR-M43 (4.11b only) | **now (4.11b + 4.11b.19)** |
|---|---:|---:|
| legal moves | −1.15% | **+7.83%** |
| legal captures | −4.16% | **+27.84%** |
| make/unmake | +17.48% | +18.70% |
| perft(4) startpos | +0.67% | **+9.21%** |
| two-ply simulation | +5.44% | **+8.49%** |
| threshold SEE | −8.07% | −8.68% |

The threshold-SEE line is the 4.11b.5 repair's standing cost, which RAR-E15
has already paid for at +12.12 ± 10.17 Elo. It reproduces RAR-M43's figure
within 0.6pp, as it should, since nothing in this leaf touched SEE.

## RAR-M44's directional prediction, scored

RAR-M44 read its probe against RAR-M43's Basilisk row and predicted, for (b),
"capture generation moves from 26% behind to about 11% ahead and the
legal-moves gap from 46% to about 32%". Both were labelled directional and
cross-session. Measured here, in one session, on exactly the head that
prediction describes:

| | predicted | measured |
|---|---|---|
| legal-moves gap | about 32% | **33.2%** |
| capture generation | about 11% ahead | **5.8% ahead** |

The legal-moves call is essentially exact. The capture call was right in
direction and **overstated in magnitude** by about half. Basilisk re-times
0.6% lower here, which does not account for it; the probe's own capture column
was simply higher than the production build's.

## The superseded first session, kept because its difference is the finding

The first (d) session ran on the head that included the (c) candidates. Those
candidates are gone, so these are not current gap figures — they are the
record of what a 12.7% board-microbenchmark win looked like on the board
benchmark, immediately before the pooled-PGO run priced it at **−0.55%** of
whole-search NPS.

| Workload | (b) head, current | (c) head, reverted | (c)'s board effect |
|---|---:|---:|---:|
| legal moves | 482.53 | 546.01 | +13.2% |
| legal captures | 126.96 | 142.41 | +12.2% |
| make/unmake | 51.87 | 53.57 | +3.3% |
| threshold SEE | 44.78 | 46.68 | +4.2% |
| perft(4) startpos | 316.53 | 346.65 | +9.5% |
| two-ply simulation | 403.12 | 451.46 | +12.0% |

**Every board column improved by up to 13%, and the search got half a percent
slower.** That is the calibration this leaf leaves behind: on this codebase, a
board microbenchmark column is not a proxy for search speed, and the sign is
not even guaranteed to survive. Contrast (b), where +11% legal moves and +41%
legal captures did convert, to +2.48% of whole-search NPS. The difference
between the two is that (b) removed work, while (c) mostly moved work around
and doubled the generator's code.

## What the remaining gap is worth — corrected

RAR-M43 computed "about +2.9% whole-search NPS, ~+5.7 Elo" from closing
generation and make/unmake. **That arithmetic was incomplete in two ways**: it
priced only two of the four board regions, omitting SEE and the never-compared
check queries; and its generation gap was partly the harness, not the
generator. Both are corrected here.

RAR-M36's shares: generation and legality 6.556%, make/unmake 6.677%,
SEE 5.239%, check queries 5.179% — 23.65% of process time in total.

Closing the **remaining** generation (1.332x) and make/unmake (1.123x) gaps
entirely:

```
1 / [ 1 − 0.06556 − 0.06677 + 0.06556/1.332 + 0.06677/1.123 ] = 1.0242
```

**about +2.4% whole-search NPS.** Adding SEE, which this session measures at a
matched-value 1.350x:

```
1 / [ 1 − 0.06556 − 0.06677 − 0.05239
      + 0.06556/1.332 + 0.06677/1.123 + 0.05239/1.350 ] = 1.0387
```

**about +3.9%.** Check queries are 5.179% and have still never been compared,
so they are in neither number and the total is a floor, not a ceiling.

At the project's recorded ~2 Elo per 1% NPS at STC that is roughly 5 to 8 Elo —
**and no Elo is claimed here; that constant is a planning figure and this
instrument measures board throughput, not playing strength.** It is far below
the measured 250–355 Elo search deficit, so 4.11b's prioritisation stands.
**And (c) is direct evidence that this arithmetic is an upper bound on paper
only**: it closed a third of the generation gap on the bench and returned
negative search speed.

## Limits

- **Board throughput is not search speed and is not Elo.** The only
  whole-search numbers in this leaf are (b)'s +2.48% [+2.29%, +2.65%] and the
  (c) bundle's −0.55% [−0.76%, −0.30%].
- Threshold SEE is comparable across engines **only** because this harness
  injects the frozen 100/300/300/500/900/20000 vector (RAR-M29). Native-value
  SEE comparisons remain superseded.
- Round range reached 6.41% on Reckless's SEE column and 5.58% on the head's
  make/unmake column, so single-column differences below roughly 5% are not
  resolved. Every other head column sat at 0.17–1.89%.

## Evidence

`tools/results/board-compare-d2-20260909/` (ignored) is the current session:
`run_comparison.py` reused unchanged from RAR-M43 except for the head binary
path, `manifest.json` with all twelve runs and their host-busy readings,
twelve raw round files, `run.log`. Head binary SHA-256 `ecf4462d…`, built with
`RUSTFLAGS='-C target-cpu=native --cfg rarog_pext'`, no features.
`tools/results/board-compare-d-20260909/` is the superseded (c)-head session,
head binary `5aab250f…`. The three archived peer binaries hash-match the
RAR-M20 manifest: `ca03a46` `40f8fa53…`, Basilisk `7eeaff0c…`, Reckless
`449897a1…`. Recipe: `board_benchmark_recipe_2026-09-05.md`.
