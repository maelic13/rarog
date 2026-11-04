# D.4.4 research card: pricing what pruning and reductions hide

Date: 2026-11-04. Leaf: PLAN D.4.4 (`R2`, then `I2`). Source:
`analysis/triumviratus_review_2026-11-03.md` TM-9. Revision read:
`d4` at `7e8a972` (engine source as `5efe16f`), fingerprint 14,749,043.

## 1. The question

Rarog counts how often each rule fires (`diag`) and could price a whole
mechanism by switching it off, but nothing says how often a single decision
was wrong: how often a pruned move would have raised alpha at the depth the
rule denied it, or a reduced move that failed low would have beaten alpha at
full depth. D.6.3, D.6.16 and F.6's fit need that price per rule, depth,
node type and move number; D.6.5 and D.6.19 need only counters and are not
this instrument's users after all (section 8).

The instrument re-searches a deterministic sample of denied decisions inside
the running search. The card decides how a re-search can run without
changing the tree it is measuring, what each label means, what the sample
costs, and what would make the instrument untrustworthy.

## 2. Decisions priced, read at `d4` (`src/search/node.rs`)

Decision counts are from the C head's stride-1 dump of `bench 13`
(`analysis/artifacts/c10/counters_c_stride1.txt`, 10,521,141 nodes, so an
order of magnitude for the current 14,749,043, not this head's count).

| Rule | Kind | What it denies | Decisions in `bench 13` |
|---|---|---|---|
| Razoring | node | the node's full search (it returns quiescence) | 86,836 |
| Reverse futility | node | the node's search (it returns a cut) | 2,521,833 |
| Null move | node | the node's search (a null-move fail-high) | 84,157 |
| ProbCut | node | the node's search (a verified capture fail-high) | 65,755 |
| Late-move pruning | move | this quiet and every later one (`skip_quiets`) | 1,039,976 |
| Quiet futility | move | this quiet and every later one (`skip_quiets`) | 267,121 |
| Bad-noisy futility | move | this capture and every later move (`break`) | no counter |
| History pruning | move | this quiet | no counter |
| SEE, quiet and noisy | move | this move | 1,359,035 and 1,125,722 |
| LMR, failed low | move | the full-depth search of the move | 3,142,387 − 104,500 re-searched |
| Quiescence delta | node | the node's captures | 12,630 |
| Quiescence futility, count, SEE, bad floor | move | this capture | 359,804; 84,502; 447,885; 62,177 |

Move-loop pruning runs only outside the root, outside check and after a
non-losing score; LMR only outside the root and check at depth 2 or more.
Exclusion (singular) searches are not priced: their window is set by the TT
score, not by the node.

## 3. The form: an isolated probe inside the search, its own feature

Three forms were weighed.

- **Inside `diag`.** Refused by `Cargo.toml`'s own contract: the `diag`
  feature may not change nodes, and a probe adds nodes and would be counted
  by every `diag` counter it passes through.
- **A scratch `Searcher` per probe.** It isolates by construction, but a
  copied thread state costs about 15 MB per probe (the continuation history
  and the two continuation corrections are 4.7 MB each) and an empty table
  makes the probe a search no real re-search resembles.
- **A probe in the running search, isolated by rule (chosen).** A new cargo
  feature, `pricing`, measured alone and never combined with `diag` for a
  read. A sampled decision calls one probe function. While it runs:
  1. **TT:** stores go to a per-thread scratch table (16 MB, allocated once
     at search start); probes read the scratch table first and the main
     table after it. The main table is never written, so the main search
     sees exactly the entries it would have seen. The scratch table persists
     across probes: its entries are honest results of searches of the same
     positions.
  2. **Histories and corrections:** frozen. Their update functions return
     without writing (a flag in `HistoryTables` and `CorrectionTables`,
     compiled only under `pricing`). The probe orders by the tables as they
     stand at the decision.
  3. **Stack, PV table, counters:** the stack entries and PV rows from the
     probe's ply upward, `nodes`, `seldepth`, `nmp_min_ply` and the search
     statistics (`nmp_verifications`, `probcut_searches`, …) are saved before
     and restored after. The probe's nodes go to their own counter.
  4. **Nesting:** no probe starts inside a probe.
  5. **The decision reversed, nothing else:** a node-level probe re-enters
     `negamax` at the same ply with one rule switched off at that ply only
     (`td.pricing_off = Some((ply, rule))`, consumed at the node); every rule
     stays on below it.
  The evaluator's pawn and imbalance caches are pure functions of the
  position, so a probe that evicts an entry costs speed and no value.

  This is the form that can be proved isolated mechanically (V2 below): with
  sampling on, the main search's own node count must still be the
  fingerprint.

Per-node rule: the probe path allocates nothing per node. Records go into a
per-thread `Vec` reserved at search start and are written at the existing
per-position dump point; the default build compiles none of it.

## 4. Labels

The window is the node's at the moment of the decision; `alpha` is the
current best, raised or not.

- **Move-level prune** (LMP, quiet futility, bad-noisy futility, history,
  SEE): the move is made and searched at `new_depth = depth - 1` with the
  null window `(alpha, alpha + 1)` and the child's node type as the search
  would give it. Label **raises**: the score beats alpha. The probe prices
  the move at the full depth, not at the reduced depth LMR would then have
  given it: LMR's own errors are priced separately. For LMP, quiet futility
  and bad-noisy futility the decision also removes the moves behind it; the
  record carries that tail's length (the picker's unemitted quiets, or the
  unemitted moves after a `break`) and the probe prices only the first. A
  per-decision price is therefore at least the first move's.
- **LMR, failed low** (the reduced search scored at or below alpha and was
  not re-searched): searched again at `new_depth` with `(alpha, alpha + 1)`,
  as the re-search would be. Label **raises**: beats alpha. A move whose
  reduced search already beat alpha is re-searched by the engine and is
  never probed: Triumviratus's first label, beating alpha at the reduced
  depth, spent its nodes on moves already re-searched (−9.65 ± 10.36).
- **Node-level cut** (reverse futility, null move, ProbCut): the node is
  searched at its depth and window with that rule off. Label **wrong**: the
  probe scores below beta, so the returned cut was false.
- **Razoring and quiescence delta:** the node with the rule off. Label
  **wrong**: the probe and the returned quiescence score fall on different
  sides of alpha.
- **Quiescence move rules:** the capture searched as quiescence searches
  it, `-quiescence(-beta, -alpha)`. Label **raises**: beats alpha.
- **Result changed**, for every move-level label that raises: at the node's
  exit, whether the node failed low (its stored bound is an upper bound).
  Then the hidden move would have changed the node's result; otherwise the
  node found its raise or cut by another move and the hidden one would have
  changed only the best move or the cut's position. Pending labels of a node
  are held in a per-ply slot of at most eight and closed at its exit.

Each record: rule, depth, ply, node type (PV, cut, all), move number, the
tail length, the probe's nodes, the label and the result-changed flag.

## 5. Sample and cost

The sample is the decision itself hashed with the rule
(`key ^ move ^ rule` through a 64-bit mix) against a per-rule rate, so it
does not depend on node order and reproduces exactly at one thread. First
rates, aiming at about 10,000 probes per rule on `bench 13`: LMR fail-lows
and reverse futility 1/256, SEE 1/128, LMP 1/64, quiet futility and the
quiescence rules 1/16 (delta 1), razoring, null move and ProbCut 1/8;
bad-noisy futility and history pruning take 1/16 until their counts are
known. At 2% and 10,000 probes a rule's share has a standard error near
0.14 points; its cells by depth and move number are thinner, and a cell is
read only with at least 400 probes.

The rates are an instrument setting fixed from probe cost alone before any
label is read: if the probe nodes of one `bench 13` exceed 20 times its
main nodes, every rate is halved, once, before the first read. Rates never
move after labels are seen.

The first read is `bench 13`'s forty positions, single thread, the same run
that validates the instrument. A larger position suite is registered only
when a user's cell has fewer than 400 probes there.

## 6. Validation before any reading is trusted

- **V1, off is exact:** a `pricing` build with sampling off benches
  14,749,043.
- **V2, on is isolated:** with sampling on, the main search's node count is
  still 14,749,043, and the probe nodes are reported apart. Any state the
  probe leaks (a TT store, a history write, a stack entry) moves this count;
  it is the instrument's isolation proof.
- **V3, reproducible:** two runs write byte-identical records.
- **V4, the label responds:** a planted over-pruning rule reads higher than
  its honest form. In a `pricing` and `tune` build, `CoreFpBase` lowered from
  150 to −100 prunes more quiets by futility; its **raises** share must
  exceed the default's with disjoint 95% Wilson intervals. The same plant
  on the reduction side: one more ply of reduction on every reduced move (a
  test switch under `pricing`) must raise the LMR fail-low **raises** share.
- **V5, the shipped build is untouched:** the default build benches
  14,749,043 and compiles no `pricing` code; every guard is behind
  `cfg(feature = "pricing")`.
- `cargo clippy --all-features --all-targets` stays at zero warnings with the
  new feature in the set, and the debug and release suites pass.

## 7. Predictions, frozen before any probe has run

For the first `bench 13` read, as shares of probed decisions, with low
confidence; they exist to be calibrated:

| Rule | Predicted share |
|---|---|
| LMR fail-lows that raise at full depth | 1–4% (Triumviratus: 1.7%) |
| LMP, first denied quiet | 2–6% |
| Quiet futility | 1–4% |
| SEE, quiet / noisy | 1–5% / 2–8% |
| Reverse futility wrong | 1–4% |
| Null move wrong | 0.5–3% |
| ProbCut wrong | 0.5–3% (its verification passes at 98%, B.3) |
| Razoring wrong | 3–10% |
| Quiescence move rules | 1–5% |

Shares fall as depth rises, because the margins grow with depth faster
than the errors they make; the result changes in under half of the raises,
because most pruned moves sit in nodes that cut by another move.

## 8. Users

- **D.6.3** (null-move and ProbCut children typed cut): the null-move and
  ProbCut wrong rates by node type and depth say whether the type matters
  where those rules cut; its own screen stays the fail-high rate by node
  type.
- **D.6.16** (quiescence pruning against a decisive material edge): the
  quiescence futility and SEE labels on the conversion positions.
- **F.6** (the search re-fit for the network): every rule's price on the
  network head, before and after the re-fit.
- **D.6.5** (the exclusion search's refined estimate) and **D.6.19** (root
  and in-check reductions) need counters, not prices: exclusion searches
  are not priced (section 2), and a reduction the engine never makes has no
  decision to re-search. PLAN keeps them off this instrument's user list.

## 9. Falsifier and stop rule

The hypothesis is that an in-search probe isolated by section 3 prices each
rule reproducibly at a bearable cost.

- If V2 fails and the leaked state is not found and closed inside the
  implementation step, the instrument is not trusted and D.4.4 returns to
  `RESEARCH` with the scratch-`Searcher` form as the next candidate.
- If V4's plant does not separate from the honest rule, the label cannot
  discriminate at this sample: the instrument is held, unused by any
  screen, until a larger registered suite gives the planted case 95%
  separation.
- If halved rates still leave probe nodes above 20 times the main nodes,
  the rare-rule rates stay and the frequent rules are priced on a sample of
  positions rather than of decisions.

A price is a screen, never acceptance: a high wrong rate makes a rule a
candidate for a registered gate; it does not show that loosening the rule
gains strength (RAR-S64).

## 10. Readiness

READY_FOR_IMPLEMENTATION, `I2`. The implementation owns the `pricing`
feature, the scratch table, the freeze flags, the save-and-restore, the probe
sites listed in section 2, the record format with a parser beside
`tools/diag/bench_counters.py`, and V1–V5. The first `bench 13` read and its
calibration against section 7 close D.4.4.
