# RAR-E08 — Self-play labels versus tablebase-corrected labels on <=6-man positions. ACCEPTED

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**Self-play labels versus tablebase-corrected labels on <=6-man positions. ACCEPTED.** H1 at 13,432 games: **+6.73 +/- 3.82 Elo, +10.34 +/- 5.88 nElo**, LOS 99.97%, LLR 2.95, 2h30m, **zero time forfeits**. One game set, two label sets: arm A keeps the literal self-play WDL everywhere; arm B rewrites the label of every position with 6 men or fewer to its Syzygy value, fifty-move rule kept, and changes nothing else.

## Result / disposition

**Registered; screen complete.** `hce-v2` carries **233,143 of 2,300,000 train rows at <=6 men (10.14%)**, of which **70.9% are labelled draw**. On a 20,953-row sample the tablebase disagrees with the self-play label on **13.27%** of them, so **about 1.35% of all training rows would be rewritten**. The disagreement is not one-directional: 1,506 sampled rows drew in self-play but are theoretically decisive, 1,253 were decisive but are theoretically drawn, and 22 are outright win/loss reversals.

## Conditional lesson and retry trigger

**Which label is 'correct' is genuinely open, which is why this is an experiment and not a decision.** Texel fits the value realizable by the CONSUMING SEARCH -- Basilisk priced borrowed Stockfish labels at **-7.30 +/- 4.76**, the worst arm it ran -- and under that principle a KBN-K position Rarog converts 7% of the time really is a draw, so the self-play label is right and the tablebase one teaches the engine to enter endgames it cannot win. Against that, self-play labels are self-reinforcing: cannot convert -> labelled draw -> evaluator learns draw -> never steers there -> never learns to convert. **The arms cannot be compared by offline loss**, because their targets differ and a loss measured against different targets is not a comparison; only a head-to-head game result decides.

## Source

this registration; RAR-M18; `analysis/archive/basilisk_audit_2026-08-30.md`; 4.10; record: `analysis/ledger_records_2026-09-14.md`, RAR-E08 (Evaluation and data experiments)
