# SMP / threading analysis — 2026-07-22

Question asked: does Rarog have a multi-thread deficit vs Basilisk and
Stockfish? Measured on the 5950X (16C/32T), 256 MB hash, quiet machine,
engines: `rarog-p103-gate` (10.3 head), Basilisk 1.9.0, Stockfish bmi2,
Reckless dev (`45ea6a9`, built locally, no syzygy).

## Verdict

**The feeling is mostly unfounded under game conditions.** At fixed movetime —
which is what games are — Rarog's NPS scaling at 16 threads is
**10.5× (66% efficiency), statistically the same as Stockfish's 10.9×**, with
Basilisk (12.2×) and Reckless (11.9×) modestly ahead. There is no dramatic
Rarog-specific deficit. There IS a consistent ~8–10 percentage-point
efficiency gap vs Basilisk at every thread count, and a *generational* design
gap vs Reckless/Stockfish in how helper threads are diversified — both real,
both modest, and the second one is only measurable in games.

Deployment note: **superseded 2026-07-25 — deployment is now 1T AND 4T**
(user decision). 1-thread remains a target — measured in Colosseum since Little Blitzer went out of scope 2026-07-25 — but 4T is a
first-class target (CCRL 4CPU conditions), so SMP quality is a direct
strength goal rather than an analysis-only nicety. The original text — "SMP
quality affects analysis use and CCRL-style multi-CPU lists, not the current
deployment condition. This bounds how much the wave should invest here" — no
longer applies and must not be used to defer SMP work.

## Measurements

### NPS scaling at movetime 5000 (game conditions, the number that counts)

| threads | Rarog NPS | eff | Basilisk NPS | eff | Reckless NPS | eff | SF NPS | eff |
|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| 1 | 2.10M | 100% | 2.18M | 100% | 0.92M | 100% | 1.04M | 100% |
| 2 | 3.10M | 74% | 3.52M | 81% | 1.38M | 75% | — | — |
| 4 | 5.89M | 70% | 6.89M | 79% | 2.85M | 77% | — | — |
| 8 | 11.5M | 69% | 13.9M | 80% | 5.64M | 76% | — | — |
| 16 | 22.1M | 66% | 26.5M | 76% | 11.0M | 74% | 11.4M | 68% |

Two reps × two middlegame positions, medians; ~±0.3M repeatability at 16T.

### RE-MEASURED 2026-07-25, after 8.13 + the TT repack — the deficit is GONE

Same protocol (movetime 5000, 256 MB, cold `ucinewgame`, 2 reps × 2 positions,
medians), idle box. Engine under test is HEAD `1cc4a85` (8.13 SMP rework +
10 B shared-TT slots + the 8.12(g2) hoist). **Basilisk 1.9.0 was re-measured
in the same session**, which is what makes the comparison valid — see the
caveat below.

All seven engines measured in the SAME session, on the same two positions.

| engine | 1T | 2T | 4T | 8T | 16T | 16T speedup | eff |
|---|--:|--:|--:|--:|--:|--:|--:|
| Stockfish bmi2 | 0.81M | 1.74M | 3.51M | 6.93M | 11.71M | **14.42×** | **90%** |
| Basilisk 1.9.1 | 2.36M | 4.80M | 9.58M | 19.10M | 31.73M | 13.44× | 84% |
| Reckless | 0.88M | 1.60M | 3.35M | 6.98M | 11.75M | 13.37× | 84% |
| Basilisk 1.9.0 | 2.28M | 4.55M | 9.28M | 18.48M | 30.36M | 13.29× | 83% |
| **Rarog HEAD** | 2.41M | 4.66M | 9.50M | 18.52M | 31.64M | **13.11×** | **82%** |
| SaberTooth | 3.95M | 7.81M | 15.37M | 29.98M | 48.45M | 12.27× | 77% |
| Rybka 4 | 0.29M | 0.48M | 0.73M | 1.18M | 1.44M | 4.91× | 31% |

**Verdict: the ~8–10 point gap vs Basilisk has closed to ~1–2 points.** Rarog
sits inside a tight modern cluster (82–84%) with both Basilisks and Reckless;
Stockfish leads at 90%; SaberTooth trails at 77%. Rybka 4 (2010 design) is the
control for what a real deficit looks like — already losing at 4 threads.

⚠ **Old and new absolute numbers are NOT comparable.** The *same* Basilisk
1.9.0 binary read 12.2× / 76% on 2026-07-22 and 13.29× / 83% here; Stockfish
read 10.9× / 68% then and 14.42× / 90% now. The 2026-07-22 run did not record
its FENs and the box was very likely not quiet — the most probable explanation
for a whole-table shift of that size. **Every engine here was therefore
re-measured in this session; nothing is carried over.** The FENs are pinned in
`tools/nps_scaling.ps1`, so this table is reproducible.

### Depth conversion at movetime 5000 (Kiwipete, 3 reps, Hash 256)

How each engine spends the extra nodes. Absolute depths are NOT comparable
across engines (pruning aggressiveness differs); only the 1T→16T **delta** is.

| engine | depth 1T | depth 16T | Δ | seldepth 1T | seldepth 16T | Δ |
|---|--:|--:|--:|--:|--:|--:|
| Reckless | 22 | 24 (23–24) | **+2** | 39 | 45 (41–50) | **+7** |
| Basilisk 1.9.1 | 22 | 23 (22–23) | +1 | 39 | 42 (40–45) | +3 |
| Stockfish | 26 | 25 (24–25) | **−1** | 57 | 62 (59–75) | **+8** |
| **Rarog HEAD** | 27 | 27 (26–27) | **0** | 40 | 42 (41–43) | **+2** |

**Losing nominal depth is not a defect** — Stockfish, the strongest engine
here, gives up an iteration at 16T and buys the largest width gain of anyone
(+8 seldepth, peaking at 75). That is a deliberate design choice.

**The real finding is that Rarog converts the least of the four, on BOTH
axes**: +0 depth and +2 seldepth, against Reckless's +2/+7, Basilisk's +1/+3
and Stockfish's −1/+8. Rarog's raw throughput scaling is now competitive
(82%), but the nodes those threads produce change the search less than in any
of the three reference engines. At Rarog's ~2.4 EBF, 13× the nodes is worth
≈2.9 iterations if spent purely on depth; it gained none of them, and did not
convert them to width either. **That gap — node throughput that does not
become search quality — is the remaining SMP question, and it is the one
9.7.5 is written to attack.**

⚠ SaberTooth exposes **no Hash option**, so its TT size is whatever it
defaults to — its column is a scaling shape, not a like-for-like comparison.

**Depth at movetime 5000 (Kiwipete, 2 reps):** Rarog reads d27/d27 at 1T and
d26/d26 at 16T, with d25–d27 spread at every count ≥2 — flat within noise, so
the extra nodes buy width, not depth (unchanged conclusion from the original
run). A single reading initially showed 27→25 and looked like a regression;
repeating it showed that was trap #5 (rep-to-rep spread), not signal. Never
call a depth delta from one reading.

**`hashfull` at 16T reaches 861–894 even at Hash=256**, so the TT is the
binding constraint at high thread counts — which is what the 10 B slot repack
addressed (it doubled shared capacity) and where any further multi-thread
work should look first.

Part of Rarog's gap is bookkeeping, not waste: at Threads=1 Rarog uses the
non-atomic `LocalTable` TT (`make_shared` switches storage only for SMP), so
its 1T baseline is a few percent faster than an always-atomic engine's — which
deflates the *ratio* without any thread being slower.

### Depth reached at movetime 10000 (how the extra nodes are spent)

| threads | Rarog avg depth | Reckless avg depth (seldepth) |
|--:|--:|--:|
| 1 | 26.0 | 26.7 (44.5) |
| 4 | 26.5 | 25.3 (45.2) |
| 16 | 28.2 | 26.2 (48.3) |

Rarog converts threads into +2.2 iterations of depth; Reckless deliberately
converts them into *width* (seldepth +3.8, depth flat). Which buys more Elo is
not decidable from any bench-style metric — only games can rank these.

## Measurement traps (all hit during this session; do not re-learn)

1. **Piping `go … quit` aborts the search** — the engine reads `quit`
   immediately. Drive a live process and read until `bestmove`
   (`scratchpad uci.ps1` pattern).
2. **TT persists between searches in one process.** Without `ucinewgame`
   before every `go`, repeat N is warmed by repeat N−1 (measured: depth-22
   TTD "faster" than depth-20). Cold-start every reading.
3. **Fixed-depth NPS/TTD under-measures Rarog SMP**: helpers inherit the main
   thread's depth limit (`WorkerJob.limits` is a straight clone;
   `search_root` iterates `1..=limits.depth`), so a helper finishing the
   target depth idles while the main thread completes. Fixed-depth 16T NPS
   read 16.5–18.8M vs 22.1M under movetime. Never quote a fixed-depth
   scaling number for Rarog; game path (clock/movetime) is unaffected.
4. **Fixed-depth TTD at reachable depths is useless for SMP quality anyway**:
   Lazy SMP spends threads on widening, so even Stockfish reads ~1.1× TTD at
   depth 23. All four engines look "broken" under that metric.
5. Lazy SMP run times are nondeterministic (2× spread rep-to-rep). Aggregate
   per-position ratios (median), never sums — a sum is owned by the slowest
   rep of the slowest position.

## Design comparison (code-level, verified in source)

Rarog (`search_threads.rs`, `search.rs::search_parallel`):
- Lazy SMP, persistent worker pool, shared lockless TT (xor-validated
  2×u64 entries, 3-entry 48 B clusters, 64 B aligned — sound).
- Helper diversification: **root-move rotation only** (`root_move_offset`
  promotes a different root move per helper at ply 0).
- Batched shared node counter (batch 128) — fine.
- Final answer: `select_parallel_result` weighted voting. Main thread stops
  helpers when IT finishes; helpers cannot stop the main thread.

Reckless (top-5, Rust, `threadpool.rs` / `search.rs` / `thread.rs`):
- **No root offsets.** Diversification is per-thread reduction jitter inside
  the tree: `reduction += ((nodes + id*27) % 128) - 59` — every thread
  searches a slightly different tree everywhere, not just at the root.
- **Cross-thread root-score sharing** (`best_stats` per-root-move atomics):
  helpers see the best score found anywhere and prune against it.
- **Majority soft-stop voting**: any thread's TM can vote to stop; a majority
  stops the search — the whole pool decides, not only the main thread.
- Sharded, cache-line-padded node counters (`#[repr(align(64))]`).

The Reckless items are candidate Phase items (post-8.4/8.5; each needs a
multi-thread SPRT, which costs real games at Threads=4):
1. Reduction jitter as helper diversification (replace or supplement root
   rotation). Cheapest to implement, standard in the current generation.
2. Cross-thread best-score sharing at the root.
3. Soft-stop voting (TM decision by pool majority).
4. Trivial, zero-game-risk: give helpers `MAX_DEPTH` instead of the main
   thread's depth limit (affects only `go depth` diagnostics; fixes trap 3).

## The question measurement cannot answer

Whether Rarog's root-rotation Lazy SMP loses Elo *per node* vs jittered
designs needs games. The standard experiment, when SMP becomes a priority:
self-play **Threads=4 vs Threads=1 at equal TC** for Rarog and for Basilisk —
the Elo-gain difference between the two pairs is the SMP-quality gap,
independent of NPS. ~2k games per pair at concurrency 3 with affinity.

