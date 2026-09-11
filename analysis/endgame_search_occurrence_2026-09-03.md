# Search-tree occurrence of the 20 reference endgame families — 2026-09-03

4.9a is ordered by **occurrence × defect**, where occurrence is RAR-M15's
measurement of how often a family appears **on the board** in real games. This
note measures the other quantity: how often each family is reached **in the
search tree**. They are close to inverse for the largest families, and the
ordering does not currently use the second one at all.

## Why the question came up

Two accepted steps bracket the range, and the bench fingerprint recorded it for
free:

- **4.9a.4** (KBN-K mate drive) moved KBN-K conversion from 19.4% to 96.9% and
  left `bench 13` **byte-identical**. No bench tree reaches a bare-king
  minor-piece mate at depth 13.
- **4.9a.7** (KRP-KR scale) moved `bench 13` by **14%**, 8,044,078 to
  6,901,489. Rook endings are reached constantly.

A change confined to positions the search never visits cannot pay, however bad
the family's conversion looks. So tree occurrence is a real term in the value
of a recognizer, and it was unmeasured.

## Instrument

Twenty exact counters plus a denominator, declared in `src/diag.rs` and
incremented from `evaluate()` under `--features diag`, so the production build
is untouched — the feature's own acceptance gate is *bench identical with diag
off*, and it holds: **6,901,489 / 2.458 both ways**.

Counted **before the whole-eval cache lookup**. A cache hit is still the search
reaching that family, and counting only misses would undercount exactly the
families the tree revisits most.

```bash
python tools/diag/bench_counters.py --exe target/release/rarog.exe --depth 13 --filter eg_
```

`bench` dumps counters once per position, so the 40 dumps **must be summed**;
`bench_counters.py` does that. Reading one dump shows zeros, which is what a
first attempt produced.

## Result

`bench 13`, 6,901,489 nodes, of which **80,589 evaluations (1.17%)** had five
or fewer non-king pieces and were classified.

| step | family | board occurrence | tree count | tree share |
|---|---|---:|---:|---:|
| 4.9a.24 | KRPPKRP | **0%** | **4,739** | 5.88% |
| 4.9a.23 | KQKRPs | **0%** | **3,554** | 4.41% |
| 4.9a.7 | KRPKR | 10.04% | 2,790 | 3.46% |
| 4.9a.9 | KPsK | 4.19% | 720 | 0.89% |
| 4.9a.14 | KQKP | 1.17% | 554 | 0.69% |
| 4.9a.11 | KRKP | 2.40% | 194 | 0.24% |
| 4.9a.10 | KPK | 2.84% | 186 | 0.23% |
| 4.9a.13 | KPKP | 1.23% | 185 | 0.23% |
| 4.9a.25 | KXK | **37.34%** | 179 | 0.22% |
| 4.9a.22 | KQKR | 0% | 39 | 0.05% |
| 4.9a.8 | KRPKB | 1.23% | 33 | 0.04% |
| 4.9a.12 | KBPsK | 1.92% | 27 | 0.03% |
| 4.9a.18 | KRKB | 0.51% | 18 | 0.02% |
| 4.9a.19 | KBPKN | 0.28% | 15 | 0.02% |
| 4.9a.15 | KBPKB | 0.89% | 13 | 0.02% |
| 4.9a.16 | KBPPKB | 0.66% | 12 | 0.01% |
| 4.9a.17 | KRKN | 0.61% | **0** | 0% |
| 4.9a.20 | KNNKP | 0.05% | **0** | 0% |
| 4.9a.21 | KNNK | 0.03% | **0** | 0% |
| 4.9a.26 | KBNK | 0.28% | **0** | 0% |

## What it says

**The two families the plan calls "never occurs naturally" are the two most
common in the search tree**, together 10.3% of classified evaluations. The
mechanism is not subtle: tree occurrence is dominated by the families with the
MOST men, because those are the ones a few captures away from a real position.
Board occurrence is dominated by the families games actually END in. The two
measures pull in opposite directions across the size range, and KXK is the
clearest case — 37.34% of games, 0.22% of the tree.

**KBN-K's zero is confirmed independently.** 4.9a.4's byte-identical bench was
not a coincidence of the fingerprint; the family is genuinely never evaluated in
this tree. The same holds for KNNK, KNNKP and KRKN.

## What it does NOT say

- **Tree occurrence is not Elo.** It is a screen, in the same family as bench
  node counts, and `AGENTS.md` already records a +7.36% tree change worth
  −1.49 ± 2.87 Elo. A leaf evaluation error only matters when it changes a root
  decision.
- **The bench suite is 40 positions and is not the game distribution.** Those
  positions were chosen for search characteristics and are middlegame-weighted.
  A tree rooted in a real endgame would reach small families far more often.
  This measures the bench tree, and says so.
- **It cannot re-rank on its own.** Occurrence × defect needs both terms, and
  the defect side for the two tree-frequent families is unmeasured.

## The one actionable collision

**KRPPKRP tops the tree list and cannot be verified locally at all.** It is
seven men and the tablebases stop at six, so neither a truth cohort nor a drawn
cohort can be built for it — which is already why PLAN defers it. This
measurement does not change that; it changes the *reason*. It is deferred
because it is unverifiable, not because it never occurs.

**KQKRPs is different and deserves promotion.** K+Q vs K+R+P is five men, so it
is fully verifiable, and it is second in the tree at 4.41% while sitting at
4.9a.23 near the end of the list on a 0% board occurrence. That is the one place
where this measurement and the existing ordering genuinely disagree about work
that can actually be done.

## Reproducing

```bash
cargo build --release -p rarog --bin rarog --features diag
python tools/diag/bench_counters.py --exe target/release/rarog.exe --depth 13 --filter eg_
cargo build --release -p rarog --bin rarog
```

The third line matters: `--features diag` leaves a diag binary in
`target/release/rarog.exe`, and `AGENTS.md` requires rebuilding with the exact
feature set before measuring anything else.

A note on the classifier: KPKP is the only symmetric family in the table, so it
matched in both orientations and read **370 instead of 185** until a guard was
added. Every other family is asymmetric and was unaffected.
