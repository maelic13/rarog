# RAR-X10 — Triumviratus's search findings on a Stockfish-19-shaped search: single gains of +1 to +6, a +86 cluster rebuild

Indexed under *8. Cross-engine evidence imported from other engines › From Triumviratus* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Triumviratus evidence

Triumviratus 8.0–8.1 (`0c18907`, 2026-11-03), self-play on UHO 2024 on one
dual-Xeon host, mostly early-stopped SPRTs of 1,000–8,000 games. Every
single foreign rule ported into its old search lost or was flat; rebuilding
on Stockfish 19's base with a joint 51-parameter SPSA gave +85.8 ± 12.8 at
20+0.2. Its adopted single mechanisms read +0.9 to +6.3, among them:

- a per-line cap on extensions from negative reductions, +3.88 ± 3.26;
- causal reduction, +4.27 ± 4.13;
- the low-depth hash-move extension (Reckless-derived), +4.5 ± 3.3 at 10+0.1,
  then switched off after −9.4 ± 10.9 at 40+0.4;
- the upcoming-repetition filter, +5.62 ± 4.47.

Its negatives include its own reduction formula, learned reductions (−1.36
± 1.44 over 60,000), a quiet history in separate threat tables (−3.32 ±
1.93), fifty-move key bands on a fading evaluation (−3.7), and four forms of
root and aspiration change. The itemised list is TS-1 to TS-19 in
`analysis/triumviratus_review_2026-11-03.md`.

## Rarog check

Rarog lacks the upcoming-repetition detection altogether (D.6.18). Rarog has
LDSE inside a line extension budget, and no TT-cutoff damping. Its null-move
floor is a coordinate already. Its quiescence has no quiet checks.

## Possible Rarog implication

A prior for ordering and designing Rarog's own tests, never for accepting
one. The +86 rebuild against flat single ports corroborates PLAN rule 3.
Items at about +5 or less wait for Phase H by the maintainer's placement
rule. The exceptions are forms an open leaf must choose anyway: TS-1 for
D.6.7, the cuckoo filter for D.6.18, and the multiply-high index for D.6.12.

## Existing PLAN coverage

D.6.2, D.6.7, D.6.12, D.6.13, D.6.14, D.6.18, F.6, G.3, H.2
