# RAR-X11 — Triumviratus's speed work at an identical tree and its board correctness instrument

Indexed under *8. Cross-engine evidence imported from other engines › From Triumviratus* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Triumviratus evidence

Measured by hardware counters at an identical tree, on a dual Xeon Gold 6138
with AVX-512. Each board or picker change is worth 0.15% to 2.6% of cycles
or instructions:

- per-node state computed once;
- unmake from a 64-byte copy;
- an SEE attacker found by index;
- bishop and rook attacks computed together;
- branch-free selection and vector ranking;
- stack probes moved out of hot frames on Windows (−1.55%);
- prefetches of the child's TT and correction entries at make (+2.2% and
  +2.0%).

The network side moved more: inference on large pages +6.7%, an AVX-VNNI tier
+6% to +9%. Prefetching and reordering loads mostly did not pay; neither did
AVX-512 gathers. Its `tdperft` instrument recomputes every key and table from
scratch at every node of a perft through the search's own generator, and
re-checks each move against the TT-move legality test: 482M nodes, 0 errors.
Items TB-1 to TB-16 and TN-7, TN-8 in `analysis/triumviratus_review_2026-11-03.md`.

## Rarog check

Rarog's board is at measured parity with Basilisk and ahead of Reckless
(`analysis/board_comparison_411b19_2026-09-09.md`). C.12 already read the
quiet-SEE skip at −0.58% and the SEE fast path as a loss. Its
`tests/board_differential.rs` rebuilds every derived field on random walks,
but runs no exhaustive perft through the staged picker.

## Possible Rarog implication

No board rebuild is warranted. The speed items are leads for an
identical-tree pass on the network head (H.4), each read by pooled-PGO NPS.
The instrument is placed now, as D.9.3, before D.5.1 and D.5.2 change the
picker and D.6 moves the board. On AMD the same prefetch may cost instead of
pay.

## Existing PLAN coverage

D.9.3, F.1, F.5, G.1, G.2, H.4
