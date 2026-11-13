# RAR-S118 — D.6.5's exclusion-node estimate and the same-ply re-entry rewrite: are they active, fresh and on replayed gate games

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-11, before any of its unexposed counts
  was read. The read is the agent's (deterministic, no games).
- **Kind:** a zero-game activation screen for one strength candidate and one
  stale-state defect. It can close either `NO_CHANGE`. It cannot make the
  estimate change ready: that needs a direction read registered after this
  one (*Readiness needs a direction*, the ledger's *Repeated lessons*).
- Disposition: observation — read 2026-11-11: LDSE would fire at 23.8% of exclusion nodes
  under the plain estimate with no singular outcome moving; a direction read is next.
  The re-entry rewrite closes `NO_CHANGE`.
- **Baseline:** `d6` at `9df630a`, engine behaviour of `ee0b3e4`, bench
  13,038,236 / EBF 2.540.

## The two questions

1. **The estimate (PLAN D.6.5, engine review NODE-4).** An exclusion node
   probes the same TT entry as its parent, whose bound is `Lower` or `Exact`
   (the singular candidacy requires it). The window is
   `singular_beta - 1, singular_beta`, below the stored score. The node's
   estimate is the table-refined eval. Where the refinement applies, it is
   at least the stored score, so the estimate sits above alpha. Three
   consumers read it inside the exclusion node:
   - LDSE (depth 7 or less, cut node, estimate 12 or more below alpha);
   - razoring (`Exact` bound only, since a `Lower` bound blocks it);
   - the quiet-move LMR alpha-gap term.

   The donor gives an exclusion node the plain eval as its estimate
   (Reckless `aec5cb2`, `search.rs:471–482`). The consumers' other guards
   (`excluded`, the bound) are identical to the donor's.
2. **The re-entry rewrite (Triumviratus review §4 item 10).** The exclusion
   search and the null-move verification re-enter the parent's ply. They
   rewrite the stack's `static_eval` and `tt_move`, and the parent restores
   only `tt_pv` and `laterality`. Children read `stack[ply].static_eval`
   (the eval swing, hindsight, improvement two plies on). If the correction
   tables moved between the parent's eval and the re-entry's, the children
   read an eval that is not the node's own.

**A premise corrected by derivation, not by a count.** PLAN and NODE-4 say the
LMR alpha-gap term is "pinned at its low clamp". It is not. Where the
refinement applies, `alpha - estimate = -(margin + 1)`, and the margin is
`0.75 * depth` (plus as much again at a non-PV `tt_pv` node) for a `Lower`
bound and less for `Exact`. That is far inside the clamp of −65 at every
reachable depth. The term is pinned at a small negative value, not at the
clamp.

## The four questions (AGENTS)

- **Mechanism.** For the estimate: LDSE and razoring can fire inside an
  exclusion search, and the LMR term follows the position's eval, as in the
  donor. What should change is the exclusion search's verdict, and so the
  singular, multicut, demotion and negative-extension outcomes. For the
  re-entry: the children of a re-entered node read its own eval.
- **Interactions.** The singular outcomes feed the extension budget, the
  TT move's slot, the multicut return and the LMR singular term. The
  constants of LDSE, razoring and the alpha-gap term were fitted with no
  firings at exclusion nodes.
- **Invariants.** Arm 0 is the source. The counters never steer. Bit 1
  changes only an exclusion node's estimate. Bit 2 changes only the two
  stack fields after the two re-entries. Bit 4 is a liveness probe only.
- **Falsifier.** If the decisions the plain estimate turns on are rare in a
  game-like table, nothing is left to gate. If the re-entries never leave
  another eval, the rewrite is harmless. The rules below say when.

## Instrument

- **Patch and binary.** `analysis/arm_patches/d65-rar-s118-screen.patch`
  (SHA-256 `1c782ac7…`) on `9df630a`. Its source tree is `d9c3b27f…`.
  It is built with `--features diag` only, as
  `analysis/artifacts/d65-items-2026-11-11/rarog-s118screen.exe`
  (`fc4a541a…`). `RAROG_S118_ARM`:
  - 1 is the candidate, the donor's plain estimate at an exclusion node;
  - 2 restores `static_eval` and `tt_move` after both re-entries;
  - 4 leaves the stack an eval 50 too high there, a liveness probe and
    never a candidate.
- **Counters,** at stride 1. Each is counted at the consumer's own site,
  after the depth the consumer reads:
  - at exclusion nodes: the bound, a refined estimate, LDSE and razoring as
    each estimate would decide, and the alpha-gap term under each estimate;
  - at each re-entry: whether the stack's eval and TT move differ from the
    parent's own, and, for the verification, whether the parent goes on.
- **Script.** `analysis/artifacts/d65-items-2026-11-11/read_d65.py`
  (SHA-256 `70cb36a6…`, frozen with this registration). It reuses
  RAR-S112's fresh form (`read_items.py`'s engine, the 40 bench positions,
  depth 13). It reuses RAR-S117's `replay.py` unchanged for the game-like
  state: the first 60 games of RAR-S115's gate, the baseline's moves,
  `go nodes` at each recorded count, Hash 64, one thread.
- **Runs.**
  - `fresh` and `replay` for arms 0, 1 and 2;
  - `tactics` (WAC at 100k and 400k nodes, and the depth-10 probe) for
    arm 1;
  - `report`.

  Arm 0's WAC files are not rebuilt. They are RAR-S113's, which ran the
  same source, and are copied beside the read. Arm 1 is read against them
  with RAR-M85's rule (`read_items.py`'s `tactic_read`).

## Wire

- **Arm 0 is the source:** each of the 40 fresh searches equals RAR-S111's
  arm-0 count.
- **The replay is RAR-S117's:** every replayed search of arm 0 equals
  RAR-S117's arm 0 in move, depth and nodes.
- **Each bit is live** (already seen, below): bit 1 and bit 4 move
  `bench 13`.

## Exposure, stated

Seen before registration: `bench 13` for arms 0 to 4 on the screen binary.

| arm | 0 | 1 | 2 | 3 | 4 |
|---|---:|---:|---:|---:|---:|
| `bench 13` | 13,038,236 | 12,940,354 | 13,038,236 | 12,940,354 | 12,375,925 |

The stack's eval after a re-entry is read: a wrong one moves the bench by
5.1%. Restoring it moves nothing on the bench. Every counter is unexposed, as
are every fresh and replayed search and arm 1's WAC.

## Decision (frozen)

**Void** if a wire condition fails or a search fails its dump count. The
game-like state decides. Fresh is reported.

- **The estimate.** Take A as the exclusion nodes where LDSE or razoring
  fires under the plain estimate, as a share of all exclusion nodes, in
  arm 0.
  - **`NO_CHANGE`** if A is under 1%, or if arm 1 shows gross tactical
    harm (a canary lost at 100k together with WAC under arm 0's count
    minus 6).
  - Otherwise the item **stays in `RESEARCH`**, for a direction read
    registered next. That read freezes one quantity the change should
    improve with its consumers held equal, which this read does not give:
    the exclusion verdict's agreement with a deeper exclusion search at the
    same node, under both estimates, in an isolated probe.
  - A gate, if one follows, keeps PLAN's rule: case 2 if LDSE fires at
    under 2% of exclusion nodes, else `[0,3]`.
- **The re-entry.** Take B as the re-entries after which the parent goes on
  with a stack eval other than its own. This counts every exclusion
  re-entry, and those null-move verifications that fail low. B is a share
  of those re-entries, in arm 0.
  - **`NO_CHANGE`** for this sub-item if B is under 1%.
  - Otherwise the restore is a repair of its own, gated by what its
    fingerprint shows (AGENTS, *Gating*).

**Reported and not deciding:**
- the bound shares and the refined share;
- the alpha-gap term's differing share and its size;
- arm 1's and arm 2's singular outcomes, interior nodes (fresh) and depth
  (replay) against arm 0;
- the `tt_move` differences;
- arm 1's WAC.

## PRE-REGISTERED PREDICTION (frozen before exposure)

1. The wire passes. 0.9.
2. At exclusion nodes out of check, `Lower` is 60% to 90%. 0.6. A bound
   other than `Lower` or `Exact` (the entry evicted between the probes)
   is under 1%. 0.8.
3. The estimate is refined at 50% to 90% of exclusion nodes out of check.
   0.6.
4. **The premise.** LDSE and razoring under the refined estimate fire at
   under 0.1% of exclusion nodes. 0.85.
5. LDSE under the plain estimate fires at 5% to 30% of exclusion nodes, in
   both states. 0.6. So the PLAN's gate class would be `[0,3]`. 0.7.
   Razoring under the plain estimate fires at under 1%. 0.7.
6. The alpha-gap term differs in 40% to 90% of quiet LMR computations at
   exclusion nodes. 0.6.
7. Arm 1 changes the share of singular attempts that extend by less than
   3 points in each state. 0.6. Its sign is not predicted.
8. Arm 1 against arm 0: fresh interior nodes 0.95 to 1.03 pooled (0.6), and
   replay depth within ±0.15 (0.6).
9. **The re-entry.** B is under 1% in both states (0.7), since restoring
   the eval leaves the bench unchanged. The sub-item closes `NO_CHANGE`.
   0.7.
10. Arm 1's WAC at 100k is within 4 of arm 0's 239. 0.7. No gross tactical
    harm. 0.9.
11. The estimate stays in `RESEARCH` for a direction read. 0.75.
12. **Most likely failure mode:** A passes on LDSE firings whose extra ply
    rarely changes the exclusion verdict. The count is then real activity
    with no evidence of benefit, which is why it cannot make the item
    ready.

## Result: the read (2026-11-11)

Read by `read_d65.py` (SHA-256 `70cb36a6…`, unchanged since registration) on
`rarog-s118screen.exe` (`fc4a541a…`). Logs and counts are beside it:
`collect.log`, `report.log`, `fresh_arm{0,1,2}.json`, `replay_arm{0,1,2}.json`
and arm 1's WAC files. Every search dumped each counter once.

**Wire: passed.**
- Arm 0's 40 fresh searches equal RAR-S111's arm-0 counts.
- All 3,670 replayed searches of arm 0 equal RAR-S117's arm 0 in move, depth
  and nodes.

| arm 0, the source | fresh, 40 searches | game-like, 3,670 replayed searches |
|---|---:|---:|
| exclusion nodes, of interior nodes | 4.69% | 4.40% |
| in check | 9.68% | 11.03% |
| `Lower` / `Exact` / other, out of check | 99.28 / 0.72 / 0% | 98.45 / 1.55 / 0% |
| estimate refined, out of check | 48.06% | 41.74% |
| LDSE under the refined estimate | 0 | 0 |
| **LDSE under the plain estimate** | **31.72%** | **23.82%** |
| razoring, plain / refined | 0.004% / 0 | 0.004% / 0 |
| quiet LMR whose alpha-gap term differs | 62.4%, by 163/1024 ply | 52.6%, by 117/1024 ply |
| **A**, decisions the plain estimate turns on | 31.72% | **23.82%** |
| exclusion re-entries leaving another eval | 0.004% | 0.001% |
| null verifications leaving another eval | 3.2% of 312 | 2.8% of 43,949 |
| verifications the parent goes on from | 5 | 1 |
| **B**, continued re-entries leaving another eval | 0.004% | **0.001%** |

| against arm 0 | arm 1, the plain estimate | arm 2, the restore |
|---|---:|---:|
| fresh interior nodes | 0.8849 | 0.9929 |
| replay depth, same move | −0.033, 65.4% | −0.031, 97.7% |
| singular attempts extended, game-like | 39.45% (39.53%) | 39.51% |
| multicut, game-like | 26.41% (26.45%) | 26.40% |
| TT move demoted, game-like | 10.52% (10.45%) | 10.47% |
| negative extension, game-like | 16.54% (16.67%) | 16.62% |
| WAC at 100k / 400k | 233 / 263 (239 / 264) | not read |

Arm 1 loses no canary at 100k; at 400k it loses WAC.170. Its later settles are
WAC.115 and WAC.199. Gross tactical harm: no (233 is the floor, and no canary
is lost at 100k).

**Decision, by the frozen rule.**
- **The estimate stays in `RESEARCH`.** A is 23.82% game-like, over 1%, with
  no gross tactical harm. The premise holds: under the refined estimate LDSE
  and razoring never fire at an exclusion node. Under the plain one, LDSE
  fires at a quarter of them. By PLAN's rule a gate would be `[0,3]`, since
  LDSE is over 2%. The next step is the direction read the rule names.
- **The re-entry closes `NO_CHANGE`.** B is 0.001% game-like. An exclusion
  search almost never leaves another eval. A null verification leaves one in
  3% of cases, but its parent goes on in only 1 of 43,949, so no child reads
  it. Restoring the fields moves the game-like depth by −0.031, which is
  noise: the move is the same in 97.7% of searches, and the rest are the
  chaotic divergence any change produces.

**What the counts say, and do not.**
- **Large activity, no aggregate shift.** A quarter of exclusion nodes
  extend their first move under the plain estimate. Yet every singular
  outcome's share moves by under 0.15 points game-like, and the replay's
  depth moves by −0.03. The registered failure mode is the likeliest reading
  so far: the extra ply rarely changes the exclusion verdict. Unchanged
  shares do not exclude verdicts that flip in both directions at the same
  rate. That is what a paired direction read measures.
- **The fresh tree shrinks by 11.5%; the game-like depth does not move.**
  This repeats D.6.4's lesson that the fresh size is no guide to a game's.

## Calibration (appended after the read)

1. The wire passed. Hit.
2. `Lower` 60% to 90%: missed in magnitude (98.5% to 99.3%). A
   null-window search cannot store `Exact`, so `Exact` entries come only
   from the PV searches, a small share of the tree. Other bounds under
   1%: hit (none).
3. Refined 50% to 90%: missed, just under (48% fresh, 42% game-like). A
   `Lower` entry's score is above the node's eval in fewer than half the
   cases, so the refinement applies less often than assumed.
4. The premise (under 0.1%): hit, exactly zero.
5. LDSE plain 5% to 30%: game-like 23.8% hit; fresh 31.7% missed in
   magnitude, just over. Gate class `[0,3]`: hit. Razoring under 1%: hit.
6. The alpha-gap term differs 40% to 90%: hit.
7. Singular extension share within 3 points: hit (under 0.1).
8. Fresh nodes 0.95 to 1.03: missed in magnitude (0.885). Replay depth
   within ±0.15: hit (−0.033). The miss is the instrument's state, not the
   mechanism: a fresh table exaggerates the size of a change.
9. B under 1%, re-entry `NO_CHANGE`: hit.
10. WAC at 100k within 4: missed (−6, at the floor). No gross harm: hit.
11. Stays in `RESEARCH`: hit.
12. The most likely failure mode is consistent with every aggregate: high
    activity, no outcome shift. It is not yet confirmed per node.
