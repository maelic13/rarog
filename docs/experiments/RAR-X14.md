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

**Its network path, net by net** (seven nets, 4.2 to 8.1). Each loss was
visible in advance:

- a first run −102 against the reference, for want of a factoriser, with a
  pure-evaluation target and an early stop;
- a self-play stage −84, for want of label strength (a second stage with
  stronger labels closed half the gap);
- a plateau that a λ cycle hid;
- checkpoint gains that summed to +71 where the direct gap closed by +28;
- network gains that vanished behind shared cores.

What paid: a pawn-pair block grafted at zero (+18), a passed-pawn block
(+7), one label scale from scratch (+23). Once saturated, only capacity paid:
material experts (+15).

Items TN-1 to TN-9, TD-1 to TD-16 and the net-by-net table in §12b of
`analysis/triumviratus_review_2026-11-03.md`.

## Possible Rarog implication

PLAN Phase F now opens with seven requirements, each owned by a leaf:

- a recipe checked against a reference;
- conformance gates;
- one label scale and labels deep enough;
- the stream's shape measured against the search;
- checkpoints judged at long controls against one anchor, on holdout loss at
  fixed λ;
- capacity, not epochs, once saturated;
- each step as CPU gates, proxies, then one full run.

F.0 to F.8 carry the details. The policy network is recorded so that it is
not proposed.

## Existing PLAN coverage

F.0, F.1, F.2, F.3, F.4, F.5, F.6, F.7, F.8
