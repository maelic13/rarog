# Gating strength

A module of `AGENTS.md`, as binding as the core. Read it in full when any
line below applies; when unsure, read it.

- Scope: registering a strength experiment and deciding what accepts a change.
- Kind: playing-strength gate; a leaf at or entering `GAME_GATE`; any change to how the engine plays, a repair included, to settle what gates it.
- Paths: `src/`, `docs/experiments/`, `docs/EXPERIMENTS.md`.
- Before: choosing a bracket, cap, harm rule or tune; preparing a gate, fixed-read or SPSA command; writing that a change is accepted, kept, rejected or stronger.

## Gating

- The strength unit is one dependency-complete, locally fitted cluster;
  internal sub-steps get no gates of their own. Register it before any games
  as a ledger entry, `docs/experiments/<ID>.md` with its index row in
  `docs/EXPERIMENTS.md` (hypothesis, baseline SHA, gate, cap, stop rule), and
  never change bounds, cap, book or adjudication after seeing games.
- `[0,3]` nElo is the default bracket. Widen only for a genuinely large prior
  and say why; a wide bracket resolves a large effect fast (RAR-S57, `[3,10]`,
  2,838 games). Compute the games at the expected value from RAR-M10 first.
- A removal or simplification uses a bracket that permits a small loss
  (`[-1.75, 0.25]`). A repair is gated by what it can change:
  1. One that reproduces the fingerprint in the rated configuration needs no
     strength gate; it gets an activation and correctness read where it is
     active. So does one that moves the fingerprint only through positions
     a static differential confines to the repair's domain, with that
     differential as its correctness read (maintainer decision 2026-10-15).
  2. One expected near zero, where a small loss is acceptable for
     correctness, takes a pre-registered 2,000-game fixed read with a harm
     rule; the registration states the threshold and its power. Scaled from
     RAR-S94's interval (±17.3 nElo at 1,552 games) the standard error is
     about 7.8 nElo; a 5% false alarm at true zero puts the threshold near
     −13 nElo, which catches a true −5 with probability 0.16, −10 with 0.36,
     −20 with 0.82 and −25 with 0.94. "Kept" therefore means no harm beyond
     roughly −20 nElo.
  3. One that changes shared search or evaluation signals in the rated
     configuration, or could be large either way, keeps the symmetric
     `[-5,5]` with its cap (RAR-S62; about 12,000 games), which excludes
     harm near −5.
- A repair's registration states its measured exposure: the share of
  recent gate games that reach the repair's domain, from Colosseum's
  database, so its cap and harm rule are read against how often it can
  matter (maintainer decision 2026-11-02).
- A correct repair that fails its harm rule leaves `master` by the gate,
  but the revert never closes its defect: the defect stays open in its
  owner leaf, and a rework is registered as a new experiment, the repair
  with the named constants that were tuned around the defect refitted.
  That is a new registration, not a rescue inside the failed one
  (maintainer decision 2026-11-02).
- Both harnesses run the pentanomial GSPRT (`model=normalized`); the gap to
  fishtest is bounds and budget, not the test. A registered experiment names
  its runner and never changes it mid-way.
- High bounds reject small gains: a true +4 nElo reaches H0 under `[0,10]` in
  about 35k games and is accepted by `[0,3]` in about 47k (RAR-M10). That is
  overnight compute: budget the games, do not widen the bounds or poll.
- Bench and counter screens choose and explain candidates and never accept
  strength: only a registered SPRT accepts one (RAR-S64). Do not invent an acceptance rule after seeing a result; register a
  narrower bracket prospectively if small gains must be bankable.
- Node counts
  are not Elo: a +7.36% tree change measured −1.49 ± 2.87 Elo.
- An unresolved stop is not "probably fine": a high LOS on a point estimate is
  not evidence the mechanism works (RAR-S61, RAR-S64).
- SPSA is conditional (PLAN rule 4): first show activation, interaction and
  curvature with a zero-game sweep; a flat or monotone surface is evidence
  against the tune. A tune is one registered run read at checkpoints by
  its gradient test and stopped when quiet (PLAN rule 7c), never on a
  horizon chosen to fit the answer.
