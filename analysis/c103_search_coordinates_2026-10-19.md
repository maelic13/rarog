# C.10.3.1 — dead and switched-off search coordinates before the joint tune

**Status: analysis, 2026-10-19, on branch `c9` (engine sources `b7bc142`,
10,521,141 / EBF 2.521). Zero games. The maintainer stopped C.10.3's pilot at
28 of 128 iterations, before any journal was read, so that the search is
correct before it is fitted to the new evaluation; this note holds the
analysis of the two coordinates the relevance check found, a sweep of the
43 tunables the check did not cover, the recommended fixes, and the
registration of the one game read the question needs (RAR-S102).** Node
counts and firing rates are not Elo.

## 1. The question and what the data can and cannot show

C.10.3's relevance check (`analysis/artifacts/c10/bench_steps.py`) moved
each of the 98 candidate coordinates one step each way on the c103 tune
build and read `bench 13`. Ninety-six moved the tree both ways; two did
not: `CoreNmpTtPv` (inert) and `CoreLmrResearchShallower` (off at its
seed). The maintainer asked whether there are more, and whether the data
gathered is enough to say the search is fine.

What the check shows: a coordinate that moves the fixed-depth tree one
step each way is *live*, its consumer reaches it. What it cannot show:
that a live coordinate's mechanism is *correct* (the formula matches its
contract, the population it acts on is the one its design named), or that
a dead one is dead by mistake rather than by a measured decision. Section
3 sweeps the 43 tunables the check left out, so that "live at its seed" is
answered for every tunable in `params.rs`. Correctness beyond that is a
reading of each mechanism against its design record, which is what this
note does for the two found; a mechanism-by-mechanism audit of the whole
search in the form of `analysis/hce_audit_2026-10-15.md` is a separate
`R2` item and is not done here.

## 2. `CoreNmpTtPv`: dead by a measured decision, not by mistake

**What the code does.** The null-move margin (`nmp_margin`,
`src/search/node.rs`) adds `CoreNmpTtPv` when the node carries the
table's PV flag; the null move's entry condition requires `!tt_pv`. So the
term is added only where its multiplier is zero. `bench 13` is identical at
0, 24, 35, 61, 96 and 200 (the check's values), and the `tt_pv_veto`
counter (a non-PV, non-check node where the stored PV bit alone vetoes the
null move and ProbCut) reads 1.8% of interior nodes on both heads
(`analysis/artifacts/c10/rate_diff.txt`): that is the population the veto
removes from both mechanisms.

**How it came to be.** B.3's research card (`analysis/b3_research_2026-09-23.md`)
seeded the margin from Reckless's (`337 − 9·depth + 110·tt_pv − 94·improvement/1024
− 21·(child cutoffs < 2)`, converted) and made the null move's *population*
a switch, `CoreNmpNodes`: 0 = Rarog's measured population, `!tt_pv` at any
node type; 1 = the donors', cut nodes only, PV-flagged ones included. Under
population 0 the `tt_pv` term can never act, as the card's own entry rule
shows; it is live only under population 1. RAR-S80 read the switch in
games: `CoreNmpNodes=1` **+3.1 ± 9.3 Elo at `3+0.03`** and
**−10.4 ± 12.0 at `10+0.1`** (`analysis/b8_removed_2026-10-03.md`, row
197), no direction, so population 0 stayed and B.8 removed the switch,
leaving the margin term that only the other population reaches. The term
then rode RAR-S83's tune surface and C.10.1's, where it could only
random-walk.

**The donors.** Stockfish `9587eeeb` excludes the null move at true PV
nodes only (`!PvNode`) and carries `112 · ttPv` in its margin (`search.cpp`,
step 9). Reckless runs it at cut nodes (`cut_node`), whatever the flag,
with `110 · tt_pv` in the margin. Both admit PV-flagged non-PV nodes with a
higher bar; Rarog excludes them. The donors' population was read once
(RAR-S80, above) and lost at the longer control; the narrower change,
"keep any node type, drop the flag veto, let the term act at 48", was
never read.

**Options.**

| Option | What changes | Behaviour | Evidence it needs | Recommendation |
|---|---|---|---|---|
| A. Delete the term | `nmp_margin` loses `CoreNmpTtPv`; `params.rs` loses the coordinate; the `tt_pv` argument goes | none; the fingerprint reproduces | tests, fmt, clippy, the fingerprint; the SPSA audit's historical groups keep their recorded ranges (RAR-S83's surface named it) | **Do this, now.** It removes the contradiction between the margin's design note and the entry rule, costs the tune nothing, and is exact |
| B. Drop the veto, keep the term | the null move runs at PV-flagged non-PV nodes against `beta + m + 48` | changes 1.8% of interior nodes' pruning; the donors' shape | a registered game read or SPRT as a search change; a 2,000-game read resolves ±9 Elo, so a small effect is invisible and a `[0,3]` SPRT costs about 36,000 games at +3 | **Not now.** RAR-S80's reading of the wider population lost at `10+0.1`; the expected size is a few Elo at most on 1.8% of nodes; it would need its own gate before the tune and would delay C.10 by a day for an unmeasured prior. Retry trigger: a game record after C.10 showing null-move cuts failing where the stored PV bit is set, or a Phase D read of the donors' population at `10+0.1` |
| C. Leave it | the dead coordinate stays in `params.rs` | none | none | No: a dead coordinate on any later surface wastes a slot and misleads the next audit, as it did C.10.1's |

Option A is behaviour-neutral by construction and verified by the
fingerprint; AGENTS puts it in its own engine commit. It changes the
coordinate count from 142 to 141 and nothing in the pilot's 96.

## 3. `CoreLmrResearchShallower`: a mechanism the tunes switched off

**What the code does.** After a reduced move beats alpha, the re-search
goes one ply deeper when `score > best_score + CoreLmrResearchDeeper` and
one ply shallower when `score < best_score + CoreLmrResearchShallower`
(`node.rs`, the LMR re-search). The move has just beaten alpha and the
best score so far is at most alpha, so `score ≥ best_score + 1`, and the
shallower branch can only fire for a margin of 2 or more. At −1 it never
fires: `bench 13` is unchanged from −50 to +1 and moves at 2 (6,761,349),
4 (8,967,538) and 5 (7,722,019).

**How it came to be.** The donor's rule is `score < best_score + 9` in
Reckless's units, live; converted, Rarog seeded 4 (`config_b23core.json`).
RAR-S75's tune moved it to −3 and RAR-S78's restart to −1. The tunes
walked a continuous coordinate across the point where its mechanism turns
off, which SPSA is allowed to do, and every later surface carried it as a
slope that could no longer move the engine. That is the one error: a
coordinate that is a switch below 2 was treated as continuous afterwards.

**What decides it.** On or off is a categorical question, and PROCESS's
go/no-go (step 3) answers it with a game read, never by pinning it under a
tune. **RAR-S102** (registered 2026-10-19, before any game) reads 2 (the
smallest live value) and 4 (the donor's converted seed) against −1 (off),
2,000 games each on the c103 tune build, RAR-S80's direction rule. A
direction fixes the mechanism on, and the coordinate enters the pilot and
the blocks with its range floored at 2 so the tune can size it and never
switch it off again; no direction keeps it off and records the mechanism
for removal after the freeze, as a simplification under the loss-permitting
bracket. The frozen prediction is that neither read finds a direction
(0.72): the tunes' drift is weak evidence, but it is the only local
evidence, and a 36% smaller tree at depth 13 has pointed both ways before.

## 4. Are there more? The sweep of the 43 uncovered tunables

The relevance check covered the 83 evidence-selected coordinates and the
15 history-unit ones. The remaining 43 (`surface.tsv`'s `out` rows) were
swept the same way on the same build (`bench_steps.py --only`, one step
each way at the b43 step):

| Result | Coordinates |
|---|---|
| live, the bench moved both ways (34) | `CoreEvalRule50Damping`, `CoreRfpLerp`, `CoreHindsightDeepenReduction`, `CoreIirMinDepth`, `CoreLmpBase`, `CoreLmpSquare`, the four SEE depth terms, `CoreLmrLog`, both improvement clamps, `CoreLmrExact`, `CoreLmrTtScoreBelowAlpha`, `CoreLmrTtShallow`, `CoreLmrQuiet`, `CoreLmrNoisy`, `CoreLmrCriticalPly`, `CoreLmrPv`, `CoreLmrPvWindow`, `CoreLmrNonPv`, `CoreLmrLaterality`, `CoreLmrTtPv`, `CoreLmrTtPvScore`, `CoreLmrTtPvDepth`, `CoreLmrCutNode`, `CoreLmrCutNodeNoTtMove`, `CoreLmrGivesCheck`, `CoreLmrChildCutoffs`, `CoreLmrChildCutoffsAllNode`, `CoreLmrParent`, `CoreNmpVerifyDepth`, `CoreSingularTtDepthMargin` |
| inert at fixed depth, expected (8) | `TmOptScale`, `TmFallBase`, `TmFallSlope`, `TmInstabBase`, `TmInstabSlope`, `TmEffortHigh`, `TmEffortLow` (clock only); `AspMaxFails` (seeded never to fire first) |
| half, expected (1) | `NmpMinNonPawnPieces` at its floor of 1 (3 moves the bench to 22,527,865) |

The raw table is `analysis/artifacts/c103/bench_steps_out.tsv`.

Reading the flags: the seven `Tm*` multipliers act on the clock only and
`bench 13` runs to a fixed depth, so they cannot move it; they are D.1's
and were never candidates. `AspMaxFails` is seeded at 20 so it never fires
first (its design note), inert by construction and excluded for that
reason. `NmpMinNonPawnPieces` sits at its floor of 1, so one side is the
clamp; the other side moves the bench. Those flags are expected and name
no defect. **No further dead or switched-off coordinate exists at the seeds:** of the 141 tunables that act inside a fixed-depth search, the two the relevance check found are the only two.

## 5. What this answers and what it does not

- Every tunable in `params.rs` is now read at its seed: live, or dead for
  a stated reason. The two defects the maintainer saw are the only two
  dead-or-off coordinates among the 141 that the search can reach at
  fixed depth, and both have a fix: one exact deletion and one game read.
- "Live" is not "correct". The check cannot see a live coordinate whose
  formula disagrees with its contract. The only way to see that is a
  mechanism audit against the design records, the way C.13 audited the
  evaluation. That is an `R2` reading of about a day; it is the honest
  next step if "the search correct before the joint fit" means more than
  "no dead coordinate on the surface". It is proposed, not scheduled.
- The pilot's 28 stopped iterations are not read. A stopped run never
  counts (PLAN rule 7c); the pilot is relaunched on the amended surface.

## 6. The order of work from here

1. Delete `CoreNmpTtPv` (engine commit, exact; tests, fmt, clippy, the
   fingerprint). Rebuild the tune build as `rarog-c103b-tune.exe`, bench
   10,521,141; regenerate `surface.tsv` and the `c10pilot` group on 141
   tunables; `--check` and the coverage audit.
2. Run RAR-S102 (two reads, about 50 minutes, idle host).
3. Amend RAR-S101 with RAR-S102's outcome, regenerate the group once more
   if the coordinate enters, dry-run, relaunch the pilot (about 40
   minutes), run the frozen re-audit, decide the blocks' surface.
4. Register and run C.10.4's block 1.

Steps 2 to 4 are the maintainer's compute; if the night's work is
delegated to the review session, each step's command, inputs and exit
status are recorded as the registrations ask, and no registered run is
shortened or re-read.

## Evidence

- `analysis/artifacts/c10/`: `bench_steps.py`, `surface.tsv`,
  `c103_range_check.txt`, `rate_diff.txt` (`tt_pv_veto`), `bench_opt.py`.
- The sweep's table: `analysis/artifacts/c103/bench_steps_out.tsv`.
- `analysis/b3_research_2026-09-23.md` (the population switch and the
  margin's seeds), `docs/experiments/RAR-S80.md`,
  `analysis/b8_removed_2026-10-03.md` row 197.
- Donors: Stockfish `9587eeeb` `search.cpp` step 9 (tag `oracle/hybrid`);
  Reckless `src/search.rs` (`D:/code/reckless` at `31d9cd6`), the null
  move's entry and the re-search rule at 928–929.
- `docs/experiments/RAR-S101.md` (the pilot, amended), `RAR-S102.md`.
