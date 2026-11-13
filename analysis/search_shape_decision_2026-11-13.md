# Should Rarog redo its search programme? Decision analysis, 2026-11-13

Kind of work: research and documentation (`R3`). No engine code changed.
Status: decided 2026-11-13. Section 12 holds the maintainer's decisions;
where they differ from sections 1, 7 and 9, section 12 and PLAN govern.

Evidence layers used here: game strength (RAR-O08, RAR-M84), and three
descriptive layers that are not Elo and have no exchange rate to it (clock
use, reported depth, nodes to depth).

## 1. Recommendation

1. **Do not redo Phase B in Stockfish's shape.** I recommend against it.
   Stockfish dev's search, as it is, on Rarog's evaluation is 88.7 ± 7.7 Elo
   behind Reckless's and 53.6 ± 8.2 behind Rarog's head (RAR-O08). The
   largest fit Basilisk recorded on that shape is +58.7. A redo would spend a
   phase to arrive about where the head already is.
2. **Do not start a full Phase B redo in Reckless's shape now either.** The
   one result that points at a deficit is match (5): Phase B's own donor,
   unfitted, is 22.5 ± 7.9 ahead of Rarog's fitted head at equal clock. That
   number bundles three things the read cannot separate: the search's shape,
   time management, and the kernel's speed per node. Reckless spent 95% of
   its clock in those games and Rarog 88%, which alone could be a large part
   of it.
3. **Spend one night separating them, then decide.** Three matches (equal
   nodes, equal move time, and the clock match at `10+0.1`) cost about three
   hours and can end the question before any engine code is written. My
   estimate is that a kernel rebuild is somewhat less likely than not to be
   warranted (about 0.4).
4. **If the equal-node read holds, rebuild the kernel whole in the donor's
   form**, not departure by departure: one cluster, seeded from the hybrid's
   exact configuration, compared with the hybrid at zero games, fitted, gated
   against the head. That is the supported positive case, and in it I
   recommend the rebuild without reservation.
5. **Open a clock-use leaf whatever the night says.** It is independent of
   the kernel and is the cheapest candidate on the table.
6. **Rust is not the cause.** The strongest search on this evaluation is
   Rust code, unchanged.

The next executable leaf stays **D.6.8** (count each position once). It is
already `READY_FOR_IMPLEMENTATION`, and the equal-node match needs it: Rarog
counts 16.1% of `bench 13` twice (RAR-S123), so "equal nodes" is not equal
until it lands.

## 2. What RAR-O08 measured

Six matches of 3,000 games, all complete, every termination normal, scored
from the PGNs (`docs/experiments/RAR-O08.md` holds the record, the frozen
rules and the calibration).

| # | First-named arm over second | Predicted | Measured |
|---|---|---|---|
| 1 | Stockfish dev over Reckless | +40 | **−88.7 ± 7.7** |
| 2 | Rarog's head over Stockfish classical | +10 | **+59.8 ± 8.1** |
| 3 | Stockfish dev over Stockfish classical | −60 | **+4.4 ± 8.1** |
| 4 | Rarog's head over Stockfish dev | +70 | +53.6 ± 8.2 |
| 5 | Rarog's head over Reckless | +110 | **−22.5 ± 7.9** |
| 6 | Reckless over Stockfish classical | −100 | +89.5 ± 8.1 |

All four arms search Rarog's current evaluation; the donors get their bridge
cost back as time odds. A least-squares fit puts Rarog's head at 0, Reckless
at +27.5, Stockfish dev at −57.2 and Stockfish classical at −61.1, with no
match further than 5.0 from the fit.

The three frozen questions, by the frozen rules:

- **(a) A search deficit against the best HCE search?** The rule read it
  against Stockfish classical: +59.8 for Rarog, so **not indicated**. The
  rule's premise was wrong, though: Stockfish classical is the weakest arm
  here, not the strongest. Against the strongest arm Rarog is behind by 22.5
  at equal clock, a result no frozen rule reads and which therefore decides
  nothing by itself.
- **(b) Which donor shape?** **Reckless**, far past the rule's bar.
- **(c) Does an HCE want a different shape from the NNUE one?** **Not
  indicated.** Stockfish's NNUE-era search is level with its classical-era
  search on this evaluation, and the strongest arm is an NNUE-era search.

Refuted, and not merely unsupported:

- "Rarog's search is behind Stockfish's classical search on its own
  evaluation." It is 59.8 ahead.
- "Stockfish's current shape transfers to this HCE better than Reckless's."
  It is 88.7 behind, at `3+0.03`.

Insufficient evidence either way:

- That Rarog's search *shape* is worse than its donor's (section 4).
- What either donor's shape would reach after a fit to this evaluation.
- Whether (b) holds at long controls. Stockfish's search is tuned for much
  deeper trees; one `10+0.1` match would say.

## 3. What this changes in yesterday's working conclusions

- **"The cause is Stockfish's shape taken whole against a Reckless hybrid."**
  Half of this survives. Stockfish's shape is not the better one here. What
  may still hold is "whole against partial": Basilisk's first cluster took a
  whole donor subsystem behind one switch and read +131 to +136 unfitted;
  Rarog's B.2 kept its own IIR, null-move population, ProbCut filter,
  quiescence and aspiration and read +52 to +65 unfitted (RAR-S73).
- **"Basilisk used a three-column seed rule and Rarog did not."** Rarog used
  the same rule (`analysis/search_programme_2026-09-13.md`, section 8.2). It
  is not a difference between the two programmes.
- **The donor did not move.** Reckless's `src/search.rs` at `0756957` differs
  from B.0's snapshot `31d9cd6` in two lines, neither of them behaviour.
  Match (5) is Rarog's fitted search against the unfitted search it was
  derived from. So the size of Rarog's fit (+138.6) never showed Rarog above
  its donor; it showed Rarog's seeds below it.
- **"Corrected for speed, Rarog's starting deficit was about 290 to 330, the
  same as Basilisk's."** Weaker than stated. RAR-O05's +84.6 ± 27.5 is the
  difference between equal time and equal nodes for two evaluations inside
  Stockfish's search; it is not the old bridge's price, which was never
  measured (RAR-O03 did not record NPS). Basilisk's oracle was handicapped
  too: BAS-O05 ran at 2.83 against 3.99 Mnps, not PGO-built. The level start
  rests on RAR-M84's direct ratings (3,001 and 2,995 held), which stand.
- **"Basilisk's search programme gained 50 to 70 more."** RAR-M84's figures
  are confirmed (+318.1 ± 24.5 against +252.6 ± 20.4 from a level start, on
  an unregistered run with no idle record). Taking off Basilisk's evaluation
  repair (+18.3) and draw verdicts (+7.4) leaves a difference nearer 40,
  with an interval of about ±32, and Rarog's side still counts its speed
  pass. Basilisk's own deficit meter has not been re-read since
  its clusters landed, so the comparison has no second witness. Last night's
  read neither confirms nor refutes it.

## 4. What match (5) bundles

Reckless's whole engine except its evaluation, against Rarog's whole engine,
on one evaluation. Three unregistered descriptive reads from the same games
and binaries (`analysis/artifacts/oracle-reads-2026-11-12/reads/` and
`tree_shape/`):

| Read | Rarog's head | Reckless | Stockfish dev | Stockfish classical |
|---|---|---|---|---|
| Share of its clock budget spent | 87.1–88.9% | 94.9–95.1% | 88.7–89.7% | 93.5–94.1% |
| Time in the first 20 moves, as a share of the base | 0.76–0.77 | 0.84–0.87 | 0.77–0.79 | 0.82–0.85 |
| Mean reported depth per move | 13.9–14.5 | 19.9–20.8 | 21.0–21.6 | 21.3–21.8 |
| Nodes to depth 13, 40 bench positions | 13,969,593 | 6,189,302 | 994,007 | 3,454,215 |
| Nodes to depth 18 | 151,105,667 | 111,625,270 | 10,079,352 | 27,660,419 |

Three competing explanations for the 22.5, and what each predicts:

1. **Shape.** The donor's tree is better per node. Predicts a surplus at
   equal nodes. The tree is certainly different: the donor reaches depth 13
   on 44% of Rarog's count (about 52% once Rarog's double count is removed).
   But tree size does not rank the arms: the smallest tree belongs to the
   third-strongest.
2. **Time management.** Reckless spends about 7% more of its budget and
   about 10% more in the first twenty moves. Rarog's budget constants are
   Stockfish's, and Stockfish dev shows the same 89%. PROCESS's speed rule
   for this control is about 2 Elo per 1% of thinking time, near 145 Elo a
   doubling (RAR-S98, RAR-S99), so a 7% difference in time spent is of the
   order of 10 Elo if nothing else differs. Predicts that the surplus
   shrinks at equal move time.
3. **Kernel speed.** The odds return the bridge only. If Reckless's move
   generation, picker and make/unmake cost less per node than Rarog's, the
   hybrid keeps that. Predicts a surplus at equal move time that is absent
   at equal nodes.

The cheapest test that separates them is three matches between the same two
binaries: equal nodes per move (shape alone), equal time per move (shape and
speed), and the existing clock match (all three). The differences are the
shares.

## 5. Where Rarog's kernel departs from its donor

Verified in source at Rarog `35e9d69`, Reckless `0756957` and Stockfish
`def601cc`. This list is what I checked, not a complete census; the complete
one is the first product of the investigation in section 9.

| Mechanism | Rarog (`src/search/`) | Reckless (`src/search.rs`) |
|---|---|---|
| Quiescence on a fail low | returns and stores `alpha` (`node.rs:2973–3000`) | fails soft: the best score is returned and stored (`:1285–1400`) |
| Quiescence's shape | legal generation up front, delta pruning by a queen, a ply cap, late captures skipped unless a recapture or check (`node.rs:2674–2718`, `:2818`) | staged picker, stop at the third move unless a direct check, one SEE threshold (`:1323–1344`) |
| Null move's population | every node without the table's PV flag, depth 3 up (`node.rs:1087–1104`) | cut nodes only, no depth floor (`:567–585`). Stockfish: cut nodes (`search.cpp:1038`) |
| LMR at the root and in check | never (`node.rs:2123`, pinned by a test at `:3228`) | both (`:861`) |
| Re-search depth after a reduced move beats alpha | deeper only (`node.rs:2240–2249`) | deeper or shallower (`:927–930`) |
| Reductions on unreduced searches | none: the first move and unreduced moves search at full depth (`node.rs:2089–2119`, `:2275–2291`) | zero to two plies by a second formula, heaviest at a cut node without a TT move (`:938–993`) |
| IIR | every node type from depth 4 (`node.rs:766–790`) | none anywhere in `src/`. Stockfish: PV and cut nodes off the previous PV, depth 6 up (`search.cpp:1087`) |
| Multi-cut | only when the singular beta is at or above beta (`node.rs:1595`) | whenever the exclusion search reaches beta (`:743`) |
| Extension budget and line bound | both (`node.rs:300–338`) | neither |
| Aspiration | fixed ±12 re-centred on the previous score, beta to the midpoint on a fail low, no depth reduction on a fail high (`mod.rs:311–372`) | width from stability and the mean square, widened from the failing score, a ply less per fail high (`:98–171`) |
| Optimism | none in `src/` | from the running average into the evaluation wrapper (`:120–124`, `evaluation.rs:3–13`) |
| Upcoming repetition | none in `src/` | main search and quiescence (`:326`, `:1215`) |
| LMP's coefficient on depth squared | 402 (`params.rs:334`) | 1351 (`:799`) |
| LMR in 1024ths: log, quiet, noisy | 150, 1,932, 830 (`params.rs:364–382`) | 269, 2,432, 1,687 (`:862–878`) |

Corrections to yesterday's list:

- The IIR item is stronger than recorded: the donor has no IIR at all. Its
  place is taken by the reduction on unreduced searches, which Rarog lacks.
  D.6.27 as written (IIR by node type) would move Rarog toward Stockfish's
  form, not its donor's.
- The reduction on unreduced searches and the upcoming-repetition test were
  missing from the list.
- LMP and the LMR terms are scale-free, so those gaps are the fit's
  movement, not a unit conversion. The fit tightened LMP by a factor of
  three and lightened LMR, which is what a fit does when it compensates for
  missing forms.
- The Stockfish-only mechanisms I confirmed in source are TT-move history,
  low-ply history, the `statScore` chain, opponent-worsening and the depth
  cut after an alpha raise. I did not pursue the rest: result (b) makes them
  moot for a shape decision.

## 6. Options

**A. Full Phase B redo in Stockfish's shape.** Against, for the reason in
section 1. The condition that would change this: Stockfish dev's hybrid
level with or ahead of Reckless's at `10+0.1`.

**B. Donor forms one at a time on the current kernel** (fail-soft
quiescence as a rework, the underpowered departures re-read). Against. The
record says such steps measure zero or lose at a co-adapted optimum: D.6.3
and D.6.4 lost their gates, D.6.5 and D.6.6 closed `NO_CHANGE`, the
root-and-check switch read −3.8 ± 9.6, the null-move population +3.1 ± 9.3,
one-retry aspiration −18.3 ± 6.9, and the four-part cluster built around
fail-soft quiescence inside the current kernel −52.5 ± 23.9 unfitted and
−6.9 ± 8.8 after a small tune (RAR-R20, RAR-R21). RAR-S15's retry trigger
(a joint refit) has not fired.
Re-reading each null to ±3 costs about four hours apiece for effects the
same argument puts near zero. PLAN's own founding statement says the same:
increments to a co-adapted optimum measure zero.

**C. The kernel rebuilt whole in the donor's form, behind a killing first
step.** For, conditionally. Three things make it different from Phase B:

- *There is a reference.* The hybrid is the donor's shape on this
  evaluation, with a measured strength. Phase B had only a donor on its own
  network.
- *The seed is known to play.* The hybrid's configuration (the evaluation
  scaled by 2536/1024 into the donor's units, the donor's constants
  unchanged, the donor's wrapper) is a point of known strength. Seeding from
  it removes the unit-conversion step, which cost Basilisk three amendments
  to its seed rule.
- *Parity can be checked at zero games.* With the same evaluation on both
  sides, Rarog's new kernel and the hybrid can be compared on the same
  positions: nodes to each depth and the rate at which each mechanism fires.
  A kernel that has not converged on the reference is not the donor's shape
  and plays no games. This is the check Phase B could not make, and its
  absence is how a "Reckless-shaped" kernel came to differ in at least
  twelve forms.

What carries over unchanged: the evaluation (frozen), board and move
generation, the table, the histories and correction tables where they
already match, time management, Lazy SMP, the root driver, the protocol,
tablebases, D.1 to D.5's repairs and D.4's instruments. What is rebuilt: the
node body, quiescence, the root window and the evaluation wrapper.

Rarog's own gate-accepted forms (the queen push in quiescence, the evasion
prune, the line bound, the draw verdicts) come back one at a time *after*
the new kernel is accepted. Single forms can be read on a donor-shaped
base: Basilisk's reconciliation cleared two of five rows last night
(+10.6 ± 9.4 and +11.6 ± 9.3, both still owing a joint SPRT).

## 7. Judgment card for option C

- **Mechanism.** The donor's selectivity as one co-adapted unit. Evidence it
  is active here: the hybrid reaches depth 13 on about half of Rarog's nodes
  and is 22.5 ± 7.9 ahead at equal clock. Evidence still missing: that any
  of the 22.5 is the tree and not the clock or the kernel's speed.
- **Interactions.** Fail-soft quiescence scores feed the table's bounds,
  which feed null move's bound, the TT cutoffs and the singular margin. The
  null-move population, the missing IIR and the reductions on unreduced
  searches all act on cut nodes without a TT move and must move together.
  Optimism enters every evaluation the margins read. The extension budget
  and the line bound exist because of the seldepth chains; without them the
  ply-cap count on replayed games is the check that the chains do not
  return. Lazy SMP reads the per-thread spread term in both reduction
  formulas.
- **Invariants.** Table bounds and mate and tablebase bands; the draw,
  repetition and rule-50 identities; no allocation per node (the allocation
  guard); the search-path perft; one count per position (D.6.8); the root
  promotion rule and the won-ending clock test (D.6.17 and D.6.20's
  acceptance tests); the evaluation's freeze.
- **Falsifiers and stop rules**, in order of cost:
  1. Equal nodes: Reckless's hybrid at or below Rarog's head. The kernel
     work stops before it starts and its leaves close `NO_CHANGE`.
  2. Parity: the new kernel's tree does not converge on the hybrid's. No
     games are played.
  3. The unfitted paired run of 2,000 games against the head: clearly
     negative with parity passed means the difference lies outside the
     kernel. One diagnosis, then stop.
  4. The gate: H0. The old kernel stays. This is one cluster, so rule 6's
     second rejection is not available: one rejection stops the programme.
- **Prior, stated so it can be wrong.** Equal-node surplus +10, 80% band
  [−15, +35]. Given the rule fires: unfitted kernel against the head +10
  [−10, +30]; fitted +35 [+10, +70].
- **Cost.** The night of reads: about three hours. If it proceeds: a paired
  run (25 minutes), one to three SPSA blocks of 12.5 hours, two SPRTs of an
  hour or so each at these effect sizes, and long-control and 4T reads of
  about three hours. Twenty-five to fifty-five compute hours, three to five
  nights, and roughly eight to twelve agent sessions. Phase B took three and
  a half weeks.

## 8. Rust against C++

My judgment: **no, the record does not show Rust making agents less
effective than C++ at this work**, and last night's read is direct evidence
against a language explanation for the gap.

- The strongest search on this evaluation is Rust code, unchanged. The
  language is not a ceiling on the engine.
- Both donors are in their adopter's language: Basilisk took a C++ donor
  into C++, Rarog a Rust donor into Rust. A same-language shortcut was
  available to both. Rarog's departures were each chosen and each has a
  record; none was forced by the borrow checker.
- Rarog's ledger holds about eight Rust-attributable rows: move lists
  returned by value (a 520-byte copy per call, and six 4,104-byte copies a
  node at 2.9%), toolchain regressions of 1.5% to 3.6%, the `unsafe` floor.
  The 2026-10-06 audit found no borrow settled by a clone. None cost a gate.
- Basilisk's ledger holds about four C++-attributable rows: a signed
  overflow found by a sanitizer read and never hit in play, a torn table
  entry under SMP accepted as a risk, a truncated buffer, and MSVC 14.4%
  slower than Clang. None cost a gate.
- The two sets differ in kind. Rust's costs showed in profiles as
  throughput and were fixed; C++'s were latent correctness found by audits.
  Both are a few percent or a few audit hours, an order of magnitude below
  the 40 to 70 Elo in question.
- What does tax an agent in Rarog is the kernel's house shape, not its
  language: `node.rs` is 185 KB against the donor's 51 KB `search.rs` for
  the same mechanisms, because counters, pricing, traces and test hooks are
  interleaved with the search. The donor is Rust too, and compact. D.8
  exists to fix this.

Where I differ from yesterday's judgment: it named the search form as the
cause and meant Stockfish's against Reckless's. The form that mattered, if
any did, is whole against partial. Limit of this judgment: the ledgers
cannot give an error rate by language; I am reading the absence of
gate-costing language incidents and one decisive counterexample.

## 9. Proposed PLAN changes (not applied)

Numbering. Phase E holds completed IDs (E.3.1 to E.3.3), so no phase can be
inserted before it, and D.8 holds one (D.8.13). The open, childless D.7
(large pages on Linux) can be renumbered. Proposal: the search-shape step
becomes **D.7**, large pages on Linux becomes **D.10**, with the map in
HISTORY.

Order. D.7.0 to D.7.2 cost one short agent session and one night and decide
whether D.6.13, D.6.16 and D.6.27 are worth doing. I propose they run ahead
of the remaining D.6 leaves by maintainer decision, as E.3.2 and E.3.3 ran
ahead of Phase D. D.6.8 comes first because the equal-node match needs it.

| Leaf | What | Class | State | Depends on | Gate or rule |
|---|---|---|---|---|---|
| D.6.8 | Count each position once (existing) | `R2` | READY_FOR_IMPLEMENTATION | — | case 2, as registered |
| D.6.13, D.6.16, D.6.27 | Aspiration window, quiescence gain pruning, IIR by node type (existing) | `R2` | RESEARCH, **held** | D.7.2's read | absorbed by D.7.4 if the kernel is rebuilt; back to their own cards if not. D.6.27's candidate form is re-read against the donor, which has no IIR |
| D.6.28 | Clock use: 11 to 13% of the budget unspent where the donor leaves 5% (new) | `R2` | RESEARCH | — | screen: unspent share by move number on Colosseum's gate games; then one knob (`tm_opt_scale`) as a registered `[0,3]` SPRT with a `10+0.1` harm read |
| D.7 | Search shape: the kernel against its donor on Rarog's evaluation (new) | `R3`, then `I2`/`V` | — | — | — |
| D.7.0 | Investigation: the complete departure table from the differential, the seed table from the hybrid's configuration, the parity instrument's design, RAR-O09's registration | `R3` | RESEARCH | RAR-O08 | ends `READY_FOR_IMPLEMENTATION` or `NO_CHANGE` |
| D.7.1 | `sprt.ps1`: per-engine node and move-time budgets, each with a wire smoke | `I1` | READY_FOR_IMPLEMENTATION | — | tooling tests and a negative check |
| D.7.2 | RAR-O09: Reckless's hybrid against the head at equal nodes, at equal move time, and at `10+0.1` with clocks; 3,000, 3,000 and 2,000 games | `V` | RESEARCH | D.6.8, D.7.0, D.7.1 | frozen in the registration. Proposed: the kernel work opens at +10 or more at equal nodes with the interval excluding zero; closes `NO_CHANGE` at 0 or below; between, the maintainer decides on cost |
| D.7.3 | Parity instrument: Rarog and the hybrid on one position set, nodes to depth and mechanism rates | `I2` | RESEARCH | D.7.2's rule | a planted difference must show |
| D.7.4 | The kernel in the donor's form behind one switch: node body, quiescence, root window, wrapper; donor constants as seeds | `I2` | RESEARCH | D.7.3 | parity before any game; debug and release tests, the allocation guard, the search-path perft |
| D.7.5 | Unfitted: the 2,000-game paired run, then gate (a) against the head | `V` | RESEARCH | D.7.4 | SPRT `[0,3]`; a `10+0.1` harm read |
| D.7.6 | The fit: curvature sweep, then SPSA in registered blocks | `V` | RESEARCH | D.7.5 | rule 7c; at most three blocks |
| D.7.7 | Gate (b), fitted against unfitted; `10+0.1` and 4T reads | `V` | RESEARCH | D.7.6 | SPRT `[0,3]` |
| D.7.8 | Rarog's accepted forms re-read one at a time on the new kernel | `R2` | RESEARCH | D.7.7 | each by its repair case |
| D.7.9 | Checkpoint: RAR-O08's matches (2) and (5) on the new head, pooled NPS, conversion; the old kernel and the switch deleted | `V`, then `I1` | RESEARCH | D.7.8 | rule 10 |
| D.10 | Large pages on Linux (was D.7) | `I1` | READY_FOR_IMPLEMENTATION | — | unchanged |

Other edits in the same documentation commit:

- **Section 1.** The search-deficit row takes RAR-O08's two figures. The
  Elo budget gains a D.7 row: 22.5 ± 7.9 at equal clock against the donor
  as it is, composition unread, planned recovery 0 to 70, conditional on
  D.7.2. If the maintainer wants it stated: the classical aim "competitive
  with Stockfish 11" as a direction beside E.2's gate.
- **Rule 10.** The search meter becomes the gap to Reckless's search on
  Rarog's evaluation, with the Stockfish classical oracle kept beside it.
- **D.8.** A note that D.8.6, D.8.7 and D.8.19 are read again after D.7: a
  rebuilt kernel is written in their target shape.
- **E.1.** The wider round robin adds **Stockfish 15.1 with `Use NNUE` off**
  beside Stockfish 11 (3,568 against the C head's 3,479, RAR-M75). It runs
  the classical evaluation under a search three years later than Stockfish
  11's, so it may be the stronger HCE target, and it has never been rated
  in this pool.
- **F.6.** A note that (c) was not indicated, so a kernel in the donor's
  form is not redone for the network, and that its donor constants are
  F.6's seeds.
- **GUIDE.** The regenerated board, the checkpoint's search-deficit and
  active-experiment rows, and the holds table (D.6.13, D.6.16, D.6.27).

## 10. Models, by GUIDE's mapping

| Leaf | Model and mode | Why |
|---|---|---|
| D.6.8 | Claude Opus 5 — High | a counting contract with re-search cases to decide |
| D.6.28 | Claude Opus 5 — High | a bounded screen and one knob |
| D.7.0 | Claude Fable 5.1 — High | the interaction map and the experiment design are the open questions |
| D.7.1 | Claude Sonnet 5 — Medium | a specified harness option with smokes |
| D.7.2, D.7.5, D.7.6, D.7.7, D.7.9 | Claude Sonnet 5 — High | preparation, verification and reading of registered runs |
| D.7.3, D.7.4 | Claude Opus 5 — High | kernel implementation against a reference |
| D.7.8 | Claude Opus 5 — High | one bounded question per form |

## 11. Not verified, and decisions that are the maintainer's

Not verified:

- The composition of match (5). That is D.7.2.
- The bridge's share in games; the odds come from bench.
- The rest of yesterday's Stockfish-only list (the two-ply TT cutoff check,
  the whole-node singular depth, continuation ply 3, the six-ply
  correction).
- RAR-S15's games, interval and revision: its record holds only the figure.
- Basilisk's deficit after its clusters: never re-read.

The maintainer's decisions:

1. Whether D.7.0 to D.7.2 run ahead of the remaining D.6 leaves.
2. The renumbering of large pages on Linux to D.10.
3. Whether the search-shape step sits before the 2.7.0 release (as D.7) or
   after it as the classical line's next release, as E.4 does.
4. A kernel closer to Reckless's raises the weight of E.3.0's originality
   disclosure and of the licence question (Reckless is AGPL-3.0).
5. How `oracle-reads` reaches `master`. It sits on `d6`; I suggest its own
   PR after `d6`'s, since its four commits do not depend on D.6's source.

## 12. Maintainer decisions, 2026-11-13

1. The rebuild stays conditional on the data (D.7.2's reads), and the
   maintainer may open it against the rule after consultation.
2. Equal strength is accepted: the rebuilt kernel is gated on the
   loss-permitting bracket. The step's second aim is to see whether the
   problems the combined kernel has not solved disappear in the donor's
   whole form, so a problem review on the new kernel (D.7.8) is part of
   the adoption judgment. This replaces section 9's gain-only gates.
3. D.7 runs before the open D.6 leaves, D.6.8 excepted. Nothing in D.6 is
   dropped: after D.7 each leaf is asked first whether it is still an
   issue, and forms that lost on the combined kernel are re-opened or
   added where their problem remains. This replaces section 9's holds on
   D.6.13, D.6.16 and D.6.27.
4. Large pages on Linux is renumbered D.10. The originality disclosure
   moves from E.3.0 to D.11, Phase D's last leaf.
5. 2.7.0 is cut from the current head before D.7, if a registered read
   shows no loss of strength against 2.6.0.
6. `d6` and `oracle-reads` reach `master` as one PR.
