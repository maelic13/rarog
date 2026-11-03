# RAR-X13 — Triumviratus's measurement and tuning method, and its evaluation residual regression

Indexed under *8. Cross-engine evidence imported from other engines › From Triumviratus* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Triumviratus evidence

- **Early stops.** Seven of thirteen early-stopped tests peaked at +5 to +17
  and ended between −2 and +2. Its five "leans" together read −0.1 ± 6.7.
- **Its SPSA tool explored with half the intended perturbation** for a month.
- **Bake rule.** A lever enters a bake only when its net move is at least 2.5
  times the random-walk expectation: with 37 levers, about 11 look monotone
  by chance.
- **Regime.** A measurement is valid only in its regime: quiet checks in
  quiescence flipped sign once a cap was lifted.
- **Tree-size centring.** A fit on search data and a fixed-position node
  sweep do in minutes what SPSA does in hours.
- **Hardware counters** at an identical tree, with a byte-identical noise
  control.
- **Pricing what pruning hides.** Data builds re-search a sample of each
  rule's pruned moves and of reduced fail-lows at the denied depth. The label
  that matters is a reduced fail-low that beats alpha at full depth (1.7% of
  probed moves).
- **Shortcut builds** that run the original beside each identical-tree
  shortcut and abort on the first disagreement.
- **Concept residual regression** of a shallow score against a deep one
  found passers worth +11.1 cp per rank in endgames (t 10.3) on its network.
  Every fix outside the network then lost.

Items TM-1 to TM-9 and TE-1, TE-2 in `analysis/triumviratus_review_2026-11-03.md`.

## Rarog check

Colosseum's schedule ends exactly at c_end on the last iteration
(`tools/results/rar-r21-fit/run.log`, CLI 0.4.0: c 6.0000 at the start, 2.7634
at iteration 1,999 of 2,000). The pinned 0.5.0 does the same: the source at
the pin asserts it, and Basilisk's live 0.5.0 block on this host falls as
c₀/(k+1)^0.102 predicts.

Its "start ± 2.25c clear of a formula's change" rule fails at c₀ for 10 of
15 coordinates of RAR-R21's fit and 43 of 82 of RAR-S104's. It fails for 0
and 7 at c_end.

## Possible Rarog implication

Most of the method is already Rarog's: unresolved stops, regime-specific
reads, the sibling-free slot. Three instruments become D.4's leaves: the
schedule read (D.4.2, done), pruning and reduction pricing (D.4.4) and the
counter read (D.4.6). The shortcut builds go with H.4's speed items. The bake
rule, the range rule and tree-size centring go to F.6's registration, as a
decision for the maintainer. The residual regression is a screen for F.2.1's
classical refit.

## Existing PLAN coverage

D.4.2, D.4.4, D.4.6, F.2.1, F.6, F.7, G.3, H.1, H.4
