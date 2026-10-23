# RAR-R21 — D.1.6.3: the staged road to the cascade cluster's gate: three 2,000-game equal-speed reads, then SPSA block 1 over the cluster and its constants, each stage with its rule — REGISTERED 2026-10-23, before any game

Indexed under *4. Root search, time management and SMP* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-23, before any game. Every stage is
  the maintainer's command; the agent prepared and dry-ran each.
- **Kind:** PLAN D.1.6.3, between the cluster's verification (RAR-R20) and
  its gate. The maintainer's requirement of 2026-10-23: be very sure the
  CPU time is not spent for nothing.
- **Why staged.** The cluster ends the stall (RAR-R20's probe) and reads
  −52.5 ± 23.9 Elo unfitted against the PGO head over 200 games, while at
  equal speed on one tune build every part reads within noise of even
  (RAR-R20's decompositions, ±27 to ±30). A 200-game read cannot separate
  "a −50 cluster", which no fit rescues, from "a −15 cluster that leaves
  time on the clock and runs 5% slower in games", which a fit and the
  clock's scale can. The 2,000-game reads resolve about ±9 Elo; they decide
  whether the ten-hour tune runs at all.
- **The binaries.** Stage 1 runs on `tools/test_engines/rarog-d16inst2-tune.exe`
  (`2f17467`, the `d1` head with every switch at its neutral default, bench
  13,987,223), both arms the same binary, arm A by UCI options, so speed is
  equal. Stage 2 runs on `tools/test_engines/rarog-d16fit-tune.exe`
  (`217125b` on `2f17467`, the throwaway branch `d16-fit`: the four parts
  baked as the search defaults, so the tune starts at the cluster's seeds as
  the harness requires; recipe
  `analysis/arm_patches/217125b-rar-r21-d16fit.patch`, SHA-256
  `e9fe1f76…42915c`, which reproduces the arm's tree; binary SHA-256
  `50323E4ED98C479B…`, bench 8,996,837). The runner is `run_r21.ps1` beside
  this entry; every stage dry-ran clean on 2026-10-23.

## Stage 1: three 2,000-game reads at `3+0.03`, equal speed (about 25 minutes each)

Colosseum `match`, `-CategoricalTuneBuild`, Hash 64, one thread, 14 slots,
the UHO book in random order, no adjudication, 2,000 games each:

1. **`all4`:** the four parts (`MopupKnownWin` 6000, `AspDeltaSquareDiv`
   10193, `CoreLmrDeeperBudget` 1, `TmDepthFreeze` 5000) against the
   defaults; seed 20261051, `tools/results/rar-r21-all4`.
2. **`nofreeze`:** parts 1–3 against the defaults; seed 20261052,
   `rar-r21-nofreeze`.
3. **`freeze`:** the freeze alone; seed 20261053, `rar-r21-freeze`.

**Readings:** each read's Elo with its interval from the manifest; each
arm's time a move and nodes a second from the manifest's `search:` lines
(the time rule: 2 Elo per 1% of thinking time at `3+0.03`, RAR-S98,
RAR-S99; between arms of one clock whole-game time is nearly conserved, so
the rule is read on time a move in plies 0–19 and on the clock left at the
end, as RAR-R18 learned).

**Rule for Stage 2 (frozen):**
- `all4` at **−15 Elo or better** (point estimate), or `nofreeze` at −10 or
  better with `freeze` carrying the rest: the loss is the size a fit and
  the clock's scale can recover; **Stage 2 runs**.
- `all4` below −15 with `nofreeze` also below −15: the search parts
  themselves cost decisions; Stage 2 does **not** run. The research then
  reads the three parts one at a time (2,000 games each, registered then),
  drops the one that costs, and re-registers.
- `all4` within ±5: the fit is optional; the gate may run directly on a
  rebuilt arm (RAR-R20's form) with the fit as its fallback on H0.

**Predictions (frozen):** `all4` **−18 ± 10 Elo** (band −30 to −6; the
−52.5 at PGO less about −20 of time left unused and in-game speed, less the
200-game lean); `nofreeze` **−8 ± 10**; `freeze` **−12 ± 10**, its time a
move 4–6% under the defaults'. P(Stage 2 runs) 0.6.

## Stage 2: SPSA block 1 over the cluster and its constants (about ten hours)

The surface `d16fix` (`tools/spsa_configs/config_d16fix.json`, generated
Colosseum files current at N = 2000): fourteen coordinates, the five
aspiration coordinates, `MopupKnownWin`, `TmDepthFreeze`, and the seven
reduction terms whose negative values the budget now charges; the b43
steps; `r-end` 0.0031; 2,000 iterations × 30 games = 60,000 games on the
fit arm; seed 20261050; `tools/results/rar-r21-fit`. Read by PLAN rule 7c:
a full-step mover in the rounded centres continues to block 2 with
`-SeedFrom`, none stops the fit at block 1.

**Predictions (frozen):** at least two full-step movers among
`MopupKnownWin`, `TmDepthFreeze`, `AspDeltaSquareDiv` and
`CoreLmrCorrection` (0.6); `TmDepthFreeze` moves up (later freezing, more
time spent), 0.6; the journal's mean z² above its sign-redraw null (0.6).

## Stage 3: the gate

RAR-R20's gate, re-registered then on an arm rebuilt with the fitted
values (a new recipe and binary, `[-5,5]` at `3+0.03`, cap 20,000 pairs,
the probe with no clock stop, the `10+0.1` read), with a fresh frozen
prediction from Stage 1's and Stage 2's numbers. A gate that passes lands
D.1.6 (D.1.6.2's landing list); one that fails returns the cluster to
research with the part-by-part reads.

**Stop rule for the whole road:** no stage starts before the previous
one's rule allows it; a stage that runs is never shortened after its
results are seen; a refused run or a fault stop halts the road.

**Evidence layer:** Stage 1 is Elo at ±9 and time use; Stage 2 is a tune's
movement, not Elo; only Stage 3 accepts anything.
