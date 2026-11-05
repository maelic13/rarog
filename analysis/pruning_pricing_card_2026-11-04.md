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
     with the searcher); a probe reads both tables and takes the deeper
     entry, the main one on a tie, so it reuses its own results. The main
     table is never written (eight write sites in `node.rs`: seven
     `store`, one `store_eval`), so the main search sees exactly the
     entries it would have seen. The scratch table persists across
     probes: its entries are honest results of searches of the same
     positions.
  2. **Histories and corrections:** frozen. Their update functions return
     without writing (a flag in `HistoryTables` and `CorrectionTables`,
     compiled only under `pricing`). The probe orders by the tables as they
     stand at the decision.
  3. **Stack, PV table, counters:** the stack entries and PV rows from the
     node's own ply upward (a move-level probe pushes its move at that
     ply, as the search does), `nodes`, `seldepth`, `nmp_min_ply` and the search
     statistics (`nmp_verifications`, `probcut_searches`, …) are saved before
     and restored after. The probe's nodes go to their own counter.
  4. **Nesting:** no probe starts inside a probe.
  5. **The decision reversed, nothing else:** a node-level probe re-enters
     `negamax` at the same ply with one rule switched off at that ply only
     (`td.pricing_off = Some((ply, rule))`, cleared when the probe
     returns); every rule stays on below it. A same-ply re-entry inside
     the probe sees the rule off too, which reverses nothing else: the
     singular exclusion search skips reverse futility, null move and
     ProbCut, and a null-move verification follows only a null move.
  6. **One thread:** probes run only at `Threads 1`, where the sample
     reproduces; `RAROG_PRICING` sets the rate shift (unset is off, `0`
     the rates below, `1` each halved) and `RAROG_PRICING_OUT` names the
     file the records are appended to.
  The evaluator's pawn and imbalance caches are pure functions of the
  position, so a probe that evicts an entry costs speed and no value.

  This is the form that can be proved isolated mechanically (V2 below): with
  sampling on, the main search's own node count must still be the
  fingerprint.

Per-node rule: the probe path allocates nothing per node. Records go into a
per-thread `Vec` reserved for the expected count (a push past it grows the
buffer, in the instrument build only) and are written at the existing
per-search dump point beside `diag::dump`; the default build compiles none
of it.

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

The sample is the decision itself hashed with the rule and the depth
(`key`, move, rule and depth through a 64-bit mix) against a per-rule
rate, so it does not depend on node order, reproduces exactly at one
thread, and a position revisited by the next iteration is drawn afresh. First
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
  on the reduction side: `CoreLmrQuiet` raised from 1,932 to 2,956 (one
  more ply on every reduced quiet) must raise the LMR fail-low **raises**
  share for quiets. The options go in on standard input before `bench`
  (a bare `bench` sets nothing, and `quit` aborts it).
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

## 11. Review, 2026-11-04, before implementation

Read against `node.rs`, `mod.rs`, `tt.rs`, `history.rs` and
`correction.rs`. The form stands. Corrected above: the TT read takes the
deeper entry; the restored stack starts at the node's own ply; same-ply
re-entries are named; the sample hash takes the depth; one thread and the
environment controls are stated; the LMR plant uses `CoreLmrQuiet`, an
existing option, instead of a new switch. Checked and unchanged: a TT probe
takes `&self` and writes nothing; the search's thread state is
`ThreadData`'s fields, all covered by section 3; node counting and the
2,048-node poll read `td.nodes`, which the probe restores, so the main
search reads the same counts after a probe.

## 12. Implementation and first read, 2026-11-04

Built in `7c2fec0` (the `pricing` feature: `src/search/pricing.rs`, the
table helpers and sixteen probe sites in `node.rs`, the freeze flags in
`history.rs` and `correction.rs`) and read with `tools/diag/pricing_table.py`
(`8a5fb6a`). Recipe: `cargo build --release --features pricing --target-dir
target/pricing`, then `python tools/diag/pricing_table.py run --exe
target/pricing/release/rarog.exe --out <file> --expect-nodes 14749043`; the
plants add `tune` and `--set`. Raw records and tables:
`analysis/artifacts/d44-pricing/r2/` (the honest read `v2.tsv`, SHA-256
`55138be08d104de7…`; pricing binary `a6d142b2adde681e…`, pricing and tune
`247c0158a745ae51…`). A record carries one field more than section 4 lists:
whether the priced move was quiet, which V4's LMR read needs.

| Check | Result |
|---|---|
| V1, sampling off | 14,749,043, pass |
| V2, sampling on | main search 14,749,043 with 317,670 probes of 1,338,960 nodes (0.09 times the main nodes, so the rates stand), pass |
| V3, two runs | byte-identical records, pass; every pending record closed |
| V5, default build | 14,749,043; clippy zero in the default, `pricing` and all-feature builds; debug, release and release `pricing` suites pass |
| V4, futility plant (`CoreFpBase` −100) | 0.13% [0.08, 0.20] to 1.95% [1.84, 2.07]: **separated**, pass |
| V4, LMR plant (`CoreLmrQuiet` 2,956), quiets as registered | 1.60% [1.35, 1.89] to 1.94% [1.64, 2.31]: **not separated**, fail; all moves alike |
| Beyond the registration, reverse-futility plant (`CoreRfpSquare` 40) | 6.09% [5.69, 6.53] to 6.04% [5.65, 6.46]: not separated |

Every label was printed once, in the per-rule summary that checked V2's
probe cost, before V3 and V4 ran; no rate, label or definition changed after.

**The contradiction.** V4 assumed a planted option reads higher in a rule's
all-depth share. Read by depth (after the fail, so a diagnosis, not a pass),
the reverse-futility plant is higher in every resolved band (depth 1 and
below 7.9% to 8.6%, 2: 4.8% to 6.3%, 3: 2.8% to 3.7%, 4–5: 0.9% to 2.3%),
but it cuts many more deep nodes (1,532 probes at depth 4–5 against 663, at
the same rate),
where wrong cuts are rarer, so the all-depth share stays flat. The LMR plant
is higher in every band too, inside the intervals. A global option also runs
inside the probe's own subtree, where it moves the probe's result as well as
the decision. The futility plant escapes both: its probes are mostly one ply
of quiescence (2.3 nodes on average) with no futility inside. So the premise
holds only for rules whose probes contain none of their own decisions. The
section 9 falsifier fires for the LMR label, which stays out of every screen;
the node-level verdicts have no passing plant either. Isolation and
reproducibility hold for every label; only futility's is plant-validated.

D.4.4 returns to `RESEARCH` with one question: a plant test that is valid
for every rule. The candidate, for the next card to judge before any run: a
plant confined to the measured search (active only outside probes, so a
probe keeps the honest rule), compared within depth bands and pooled
(Mantel–Haenszel), with the comparison and its threshold registered first.

**Calibration against section 7** (predictions unchanged above):

| Rule | Predicted | Read | Miss |
|---|---|---|---|
| LMR fail-lows | 1–4% | 1.63% | none (Triumviratus 1.7%) |
| LMP | 2–6% | 2.46% | none |
| Quiet futility | 1–4% | 0.13% | magnitude: the margin is far safer than assumed |
| SEE, quiet / noisy | 1–5% / 2–8% | 0.42% / 1.08% | magnitude, both low |
| Reverse futility | 1–4% | 6.09% | magnitude, high: 7.9% at depth 1 and below, 12.2% in all nodes |
| Null move | 0.5–3% | 1.99% | none overall; 18.8% in all nodes and 22.7% at depth 3 (both under 400 probes) |
| ProbCut | 0.5–3% | 0.34% | magnitude, low, as its 98% verification rate implied |
| Razoring | 3–10% | 7.03% | none |
| Quiescence move rules | 1–5% | 1.31–4.51% | none |
| Shares fall with depth | yes | reverse futility, null move, ProbCut, LMR above depth 3; LMP and history flat | partial |
| Result changes in under half of the raises | yes | 83–99% outside PV nodes, about half in them | sign and mechanism: pruned moves sit mostly in nodes that fail low, so a hidden raise would flip the node |

These are screens: the null move's all-node and the reverse futility's
shallow wrong rates are candidates for D.6.3's and D.6's cards, not findings
(section 9), and the node-level labels are not yet plant-validated.

## 13. A plant test for every rule, registered 2026-11-05 before any run

**Question.** Does each rule's label rise when that rule alone is made more
aggressive in the measured search? Section 12's V4 read the all-depth share
under a global option, and two things besides the label moved with it. The
plant reshaped the tree: the LMR plant cut the main search from 14,749,043
nodes to 8,871,834, the reverse-futility plant to 9,907,927, moving decisions
between depths and node types whose rates differ several times over. And
the option ran inside every probe too, so a probe searched with the same
planted rule below the priced ply.

**Competing explanations** for the LMR and reverse-futility nulls, and what
each predicts:

1. *Composition*: the label responds, but the depth and node-type mix hides
   it. A stratified statistic separates the global plants too.
2. *Contamination*: the planted rule inside the probe's subtree weakens the
   probe and hides raises. A confined plant separates where the global one
   did not, stratified alike.
3. *Insensitive label*: the probe does not measure the decision's cost (the
   wrong depth, window or ply for the rule's re-entry). An active, confined
   plant does not separate within strata.
4. *Inactive plant*: the option barely changes how often the rule fires.
   The activation check below fails.

**Design.**

- **Confinement.** With `RAROG_PRICING_CONFINE=1`, a `pricing` and `tune`
  build swaps the compiled-in parameter sets (search, core, proof and quiet)
  into the searcher when a probe opens and the set options back when it
  closes, so a planted option acts only in the measured search. One swap per
  probe, compiled only into `pricing` builds.
- **V6, the swap is neutral:** the honest arm is the same binary with
  confinement on and no option set, and must reproduce `v2.tsv` byte for
  byte.
- **V7, isolation under a plant:** probes leave the main search alone either
  way, so a confined plant's main search must equal its global run's in
  section 12: futility 12,149,055, LMR 8,871,834, reverse futility
  9,907,927 nodes.
- **V8, the wire is live:** a test sets a core parameter, opens a probe,
  sees the compiled-in value, closes it and sees the planted value again.
- **Plants:** one per rule, each the rule's own margin or count made more
  aggressive and nothing else:

| Rule (record name) | Option | Default to plant |
|---|---|---|
| Razoring (`razor`) | `CoreRazorBase` | 302 to 100 |
| Reverse futility (`rfp`) | `CoreRfpSquare` | 170 to 40 (section 12's) |
| Null move (`nmp`) | `CoreNmpRBase` | 4,255 to 6,300, two plies shallower |
| ProbCut (`probcut`) | `CoreProbcutBase` | 145 to 50 |
| Late-move pruning (`lmp`) | `CoreLmpBase` | 2,888 to 1,024 |
| Quiet futility (`futility`) | `CoreFpBase` | 150 to −100 (V4's) |
| Bad-noisy futility (`bnfp`) | `CoreBnfpBase` | 56 to −100 |
| History pruning (`history`) | `CoreHpSlope` | 1,019 to 300 |
| Quiet SEE (`see_quiet`) | `CoreSeeQuietSquare` | 7 to 0 |
| Noisy SEE (`see_noisy`) | `CoreSeeNoisySquare`, `CoreSeeNoisyLinear` | 5 and 12 to 0 and 0 |
| LMR fail-lows, quiets (`lmr`) | `CoreLmrQuiet` | 1,932 to 2,956 (V4's) |
| Quiescence delta (`q_delta`) | `QsDeltaMargin` | 308 to 0 |
| Quiescence futility (`q_futility`) | `QsFutilityMargin` | 257 to 50 |
| Quiescence count (`q_count`) | `QsCountLimit` | 3 to 2 |
| Quiescence SEE (`q_see`) | `QsSeeMargin` | 247 to 50 |
| Quiescence bad floor (`q_bad_floor`) | `QsSeeBadFloor` | −43 to 0 |

- **Activation:** the rule's probes per main-search node, planted over
  honest, at least 1.2; below it the plant is INACTIVE and the rule gets no
  verdict.
- **Statistic:** the Mantel–Haenszel odds ratio of the label, planted
  against honest, with strata of depth (exact, 12 and above pooled; the
  quiescence ply for the quiescence rules), node type and, for move rules,
  quiet or noisy; the 95% interval by Robins, Breslow and Greenland. LMR
  reads quiets only, as V4 registered.
- **Verdicts:** PASS when active and the lower bound exceeds 1; FAIL when
  active and the upper bound is below 1.25 (the odds of a wrong decision
  rose by less than a quarter while the plant added at least a fifth more
  decisions); UNRESOLVED otherwise. `pricing_table.py plant` prints them and
  exits 0, 1, 2 or 3 for PASS, FAIL, UNRESOLVED and INACTIVE.
- **Diagnostic, no verdict:** the same statistic on section 12's three
  global plants, which separates explanations 1 and 2.

**Predictions, frozen.** At least 13 of the 16 rules PASS. Futility, the
control, PASSes. LMR quiets and reverse futility PASS under confinement.
ProbCut is the likeliest UNRESOLVED (39 wrong verdicts in the honest read).
In the diagnostic, the global reverse-futility plant separates once
stratified and the global LMR plant does not. Low confidence on every
single rule; moderate on the count.

**Consequences.** PASS: the label is plant-validated and a screen may read
it. FAIL: the rule's probe is suspect (explanation 3); its label stays out
of every screen and D.4.4 stays in `RESEARCH` on that probe. UNRESOLVED or
INACTIVE: the label stays unvalidated and out of screens; a stronger plant
or a larger suite is a new registration, never a rerun under a changed
threshold. D.4.4 closes when no rule FAILs, its record listing every
label's status. Plants, strata and thresholds are fixed here and do not
change after the first planted run.

**Recipe.** `cargo build --release --features pricing,tune --target-dir
target/pricing-tune`, then per rule `python tools/diag/pricing_table.py run
--exe target/pricing-tune/release/rarog.exe --confine --set <Option>=<value>
--out <file>`, and `pricing_table.py plant <honest> <planted> --rule <name>`
(`--quiet quiet` for LMR).

## 14. The plant test read, 2026-11-05

Confinement in `6b95225`, the read in `ccff1a9` (`pricing_table.py run
--confine` and `plant`), the registration in `bd46336`, all run as section
13 registered with no plant, stratum or threshold changed. Binary: `pricing`
and `tune`, SHA-256 `dfdb31d53c3c2999…`; records, logs and `plants.md` in
`analysis/artifacts/d44-pricing/r3/`.

| Check | Result |
|---|---|
| V6, the swap is neutral | the confined honest arm is byte-identical to `v2.tsv` (`55138be08d104de7…`), pass |
| V7, isolation under a plant | confined main searches equal the global ones: futility 12,149,055, LMR 8,871,834, reverse futility 9,907,927, pass |
| V8, the wire is live | both swap tests pass; end to end, the LMR plant's probes searched 733,672 nodes confined against 705,283 global, so the binary swaps |
| V1, V2, V5 again | the default and `pricing` builds bench 14,749,043; the unconfined `pricing` run reproduces `v2.tsv`; clippy zero in the default, `pricing` and all-feature builds; suites pass (debug 444, release 445, release `pricing` 447) |

| Rule | Main nodes | Probes, honest to planted | Share, honest to planted | Activation | MH odds ratio [95%] | Verdict |
|---|---|---|---|---|---|---|
| `razor` | 12,854,694 | 12,737 to 22,300 | 7.03% to 5.60% | 2.009 | 0.765 [0.700, 0.837] | FAIL |
| `rfp` | 9,907,927 | 12,371 to 13,235 | 6.09% to 6.20% | 1.593 | 1.214 [1.093, 1.348] | PASS |
| `nmp` | 11,446,448 | 16,518 to 14,167 | 1.99% to 0.88% | 1.105 | - | INACTIVE |
| `probcut` | 14,121,083 | 11,447 to 26,614 | 0.34% to 1.13% | 2.428 | 3.254 [2.327, 4.551] | PASS |
| `lmp` | 16,893,682 | 20,096 to 29,102 | 2.46% to 3.83% | 1.264 | 1.601 [1.437, 1.784] | PASS |
| `futility` | 12,149,055 | 16,365 to 53,328 | 0.13% to 1.95% | 3.956 | 12.198 [7.938, 18.744] | PASS |
| `bnfp` | 12,134,765 | 20,332 to 31,109 | 0.59% to 1.30% | 1.860 | 2.068 [1.684, 2.538] | PASS |
| `history` | 13,918,222 | 73,878 to 86,576 | 3.02% to 3.85% | 1.242 | 1.174 [1.111, 1.242] | PASS |
| `see_quiet` | 9,685,155 | 13,452 to 11,360 | 0.42% to 0.86% | 1.286 | 2.009 [1.444, 2.794] | PASS |
| `see_noisy` | 15,239,293 | 9,383 to 11,602 | 1.08% to 1.83% | 1.197 | - | INACTIVE |
| `lmr` | 8,871,834 | 8,444 to 6,479 | 1.60% to 1.85% | 1.276 | 1.184 [0.921, 1.522] | UNRESOLVED |
| `q_delta` | 15,528,230 | 17,152 to 73,367 | 1.84% to 9.39% | 4.063 | 5.288 [4.714, 5.930] | PASS |
| `q_futility` | 16,875,099 | 32,107 to 67,813 | 3.39% to 8.88% | 1.846 | 2.779 [2.601, 2.969] | PASS |
| `q_count` | 12,753,118 | 7,096 to 14,251 | 1.31% to 1.91% | 2.323 | 1.468 [1.157, 1.864] | PASS |
| `q_see` | 13,778,462 | 42,691 to 52,407 | 1.90% to 5.48% | 1.314 | 3.042 [2.808, 3.295] | PASS |
| `q_bad_floor` | 13,208,448 | 2,884 to 4,468 | 4.51% to 9.27% | 1.730 | 2.158 [1.761, 2.643] | PASS |

**Verdicts.** Twelve labels PASS and are plant-validated: reverse futility,
ProbCut, late-move pruning, quiet futility, bad-noisy futility, history
pruning, quiet SEE and all five quiescence rules. Razoring FAILs. The LMR
label is UNRESOLVED. The null-move and noisy-SEE plants are INACTIVE (noisy
SEE at 1.197, just under the registered 1.2; its share rose, but no verdict
is drawn from an inactive plant). Per section 13 the razor, LMR, null-move
and noisy-SEE labels stay out of every screen, and D.4.4 stays in
`RESEARCH` on the razor probe.

**Diagnostic** (section 12's global plants, the same statistic): futility
12.15 [7.90, 18.67], reverse futility 1.19 [1.08, 1.33], LMR quiets 1.23
[0.96, 1.58]. Composition (explanation 1) alone explains section 12's
reverse-futility null. Contamination (explanation 2) is refuted for LMR:
confined, its ratio is 1.18 [0.92, 1.52], no closer to separating. The LMR
plant adds only 28% more decisions and moves the share from 1.60% to 1.85%,
an effect too small for about 120 raises per arm.

**Razoring's FAIL.** The wrong share falls from 6.94% to 5.24% at depth 1
and below and is flat at depth 2 (8.06% to 8.03%): the decisions a lower
margin adds are wrong less often than the honest ones. Two explanations
compete, and this read cannot separate them. (a) The razor probe is wrong
for its rule, which section 13's consequence assumes. (b) The plant is not
harmful: razoring's region lies where quiet futility already prunes every
quiet that does not give direct check (at depth 1 the futility value
`eval + 183 + history terms` sits below alpha whenever `eval < alpha − 284`,
up to the history term), so the extra razored nodes are ones the rest of the
search would resolve the same way, and the label, which is "razoring off,
everything else on", rightly finds them seldom wrong. The cheapest
discriminators, for the next card to register: a placebo read (probing
node-level rules at nodes where they did not fire must reproduce the node's
own side of alpha, testing the probe with no plant at all), and razor
records flagged by whether quiet futility covers every quiet at the node.
The null-move share halved (1.99% to 0.88%) under a plant that barely
changed how often it fires; recorded, not read.

**Calibration against section 13** (predictions unchanged above):

| Prediction | Read | Miss |
|---|---|---|
| At least 13 of 16 PASS | 12 | count, by one; razor and LMR missed, two plants inactive |
| Futility, the control, PASSes | PASS, 12.20 | none |
| LMR quiets PASS confined | UNRESOLVED, 1.18 | mechanism: contamination was not the cause; the plant's effect is small |
| Reverse futility PASSes confined | PASS, 1.21 | none |
| ProbCut likeliest UNRESOLVED | PASS, 3.25 | magnitude: its plant more than doubled the decisions |
| Global reverse futility separates stratified | 1.19 [1.08, 1.33] | none |
| Global LMR does not | 1.23 [0.96, 1.58] | none |
| No rule predicted to FAIL | razoring FAILs | the plant premise (a lower margin is a worse rule) may not hold where another rule covers the region |

What screens may read now: the twelve validated labels. D.6.3 needs the
null-move and ProbCut verdicts; ProbCut's is validated, the null move's is
not.

## 15. Re-entry, the table and stronger plants, registered 2026-11-05 before any run

**Question.** Why do two node-level rules move the wrong way under their
plants (razoring FAILs at 0.77; the null move's share halves, 1.99% to
0.88%, under a plant that barely changed how often it fires), and can the
LMR, null-move and noisy-SEE labels be resolved by plants that act?

**Competing explanations** for the reversed node-level reads:

1. *Re-entry defect.* A node-level probe re-enters the node with arguments
   that differ from the decision's (depth, window, node type, null-move
   permission, critical ply), so it prices a different node. Predicts that a
   probe with nothing switched off fails to reproduce the decision.
2. *Table contamination.* A probe reads the deeper of the main and scratch
   entries, and in a planted run the main table holds the planted search's
   bounds. Where the probe's subtree reaches positions the planted search
   stored, planted errors come back as agreement with the decision.
   Confinement swapped the parameters, not the table. Predicts that probes
   reading only their own scratch table (cold) lose the reversal, most where
   probes are deep (null move, 19.7 nodes on average) and least for razoring
   (4.0).
3. *A harmless plant.* A lower razoring margin adds nodes where quiet
   futility already prunes nearly every quiet, so the label, razoring off
   with everything else on, rightly finds them seldom wrong. Predicts that
   the identity probes reproduce the decision and that going cold changes
   nothing.

Not tested: the frozen histories and corrections were trained by the planted
search, so even a cold probe orders moves and corrects its estimates with the
planted search's tables. They move ordering and margins, never a stored
score.

**Design.**

- **T1, identity** (tests 1). With `RAROG_PRICING_IDENTITY=1` every sampled
  node-level decision (`razor`, `rfp`, `nmp`, `probcut`, `q_delta`) gets a
  second probe with nothing switched off. It reads the main table only and
  stores nothing, so it reads what the decision read and leaves the priced
  probe's scratch table alone. It is recorded as `<rule>_identity`, its label
  set when it fails to reproduce the decision (a cut: below beta; a returned
  score: on the other side of alpha). Run once, honest, warm. It holds for a
  rule when the 95% Wilson upper bound of that share is below 1%. **V9:** the
  run's priced records equal `v2.tsv`'s line for line.
- **T2, cold** (tests 2). With `RAROG_PRICING_COLD=1` probes read and write
  the scratch table alone. Arms: honest; and, confined, razoring
  (`CoreRazorBase` 100, section 13's), the null move (`CoreNmpBase` 0,
  below), reverse futility (section 13's, a node-level rule that passed warm)
  and futility (the control). Read by section 13's `plant`, its activation,
  strata and thresholds unchanged. **V10:** the honest cold run's main search
  is 14,749,043 nodes.
- **T3, stronger plants**, warm and confined as in section 13:

| Rule | Option | Default to plant |
|---|---|---|
| Null move (`nmp`) | `CoreNmpBase` | 159 to 0: the entry margin falls to its floor of 2, so the rule tries whenever the estimate clears beta |
| LMR fail-lows, quiets (`lmr`) | `CoreLmrQuiet` | 1,932 to 3,980: two plies on every reduced quiet |
| Noisy SEE (`see_noisy`) | `CoreSeeNoisySquare`, `CoreSeeNoisyLinear`, `CoreSeeNoisyHistory`, `CoreSeeNoisyConstant` | 5, 12, 33 and 33 to 0: threshold 0 at every depth, the most the formula's cap at 0 allows |

- **Diagnostic, no verdict:** each rule's honest share warm against cold,
  which is what the table contributes to its label.

**Inference rules.**

- A node-level rule whose identity holds and whose cold plant PASSes is
  validated for the warm honest read: the cold read shows that the label
  responds to the rule's harm, and an honest run's table holds no planted
  result.
- Razoring: if its identity holds and its cold plant does not PASS,
  explanation 3 stands. The label's mechanics are verified but its harm
  cannot be shown by its own margin; it stays out of every screen, recorded
  as such, and D.4.4 does not wait on it.
- The null move is validated by a warm PASS in T3, or by identity plus a
  cold PASS.
- LMR is validated by a PASS in T3; otherwise it stays out of every screen.
- Noisy SEE INACTIVE again: its own options cannot plant it harder, and the
  label stays out of every screen.
- A failed identity is a defect. It is found and repaired, and that rule's
  identity and plant reads are rerun as registered here before anything
  reads it.
- D.4.4 closes when every identity holds, after any repair; every label's
  status is then recorded, and unvalidated labels stay out of screens.

**Predictions, frozen, low confidence.** Every identity holds, each under
0.5% unreproduced. Cold: razoring still does not PASS (explanation 3), the
null move PASSes, reverse futility and futility PASS. Warm T3: the null move
at margin 0 does not PASS (contamination), LMR at two plies PASSes, noisy SEE
is INACTIVE. Warm against cold, honest: the node-level wrong shares rise
cold, and the move-level shares move by less than a fifth.

**Recipe.** As section 13, adding `--identity` or `--cold` to
`pricing_table.py run` (which set the variables above), `pricing_table.py
identity <records>` for T1 (exit 0 when every identity holds), and
`plant` for T2 and T3.
