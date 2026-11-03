# RAR-X14 — Triumviratus's NNUE architecture, data and training findings

Indexed under *8. Cross-engine evidence imported from other engines › From Triumviratus* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Triumviratus evidence

Its own weights, trained from random initialisation on public Leela and
Stockfish data re-labelled with Leela BT4. Phase F's own-data rule excludes
that recipe; the architecture and the training method transfer as
mechanisms.

**Architecture.**

- Material experts on the input layer, four bands of total piece count
  trained as a shared base plus deltas: about +15 of 8.0's +27.3, at −7.5%
  NPS.
- Per-side experts with a pawn-king subnetwork: +10.3 ± 4.3 at fixed nodes.
- Input blocks: `PawnPair` +18.27 ± 9.94, `PassedPawns` +6.96 ± 6.56.
- A policy network inside alpha-beta lost in four forms, from −22.83 to −85.

**Training.**

- Watch the holdout loss at fixed λ=0 and λ=1: a λ cycle hid the plateau.
- Register checkpoints with Elo targets.
- Average the last epochs' weights.
- The data loader limits throughput.
- Network gains shrink at longer controls and hide behind shared cores.

**Conformance gates before training:** an integer evaluator against the
trainer; incremental against refresh at every node; a golden set of 2M
positions.

Items TN-1 to TN-9 and TD-1 to TD-8 in
`analysis/triumviratus_review_2026-11-03.md`.

## Possible Rarog implication

Priors for F.7's architecture ladder, with the graft screen as its cheap
method. F.3, F.4 and F.5 take the training and conformance instruments.
The policy network is recorded so that it is not proposed.

## Existing PLAN coverage

F.1, F.3, F.4, F.5, F.6, F.7, F.8
