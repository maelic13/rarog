# Agent operating rules for Rarog

Read `GUIDE.md` for what to work on and the relevant part of `PLAN.md` for why.
These rules protect correctness and the maintainer's time and token budget.

## Classify the work before starting

Before substantial work, name its primary kind: research/diagnosis, experiment
design, implementation, deterministic qualification, performance
qualification, playing-strength gate, or documentation/provenance. Do not read
every roadmap leaf as an instruction to write code. `PLAN.md` owns the workflow
state and capability class; `GUIDE.md` owns the maintainer-editable mapping from
capability classes to current models.

The prospective workflow is `RESEARCH -> READY_FOR_IMPLEMENTATION ->
IMPLEMENTED -> LOCAL_QUALIFIED -> GAME_GATE -> CLOSED`. Not every task needs
every state. Documentation can close without games; research can close with a
justified `NO_CHANGE`; neutral performance work uses deterministic/performance
qualification. A playing-strength change normally cannot bypass `GAME_GATE`.

`READY_FOR_IMPLEMENTATION` is a hard semantic boundary. Before promoting a
substantial playing change, establish the measured defect or opportunity, the
evidence for it in this engine, credible competing explanations, interacting
mechanisms, the cheapest test that can kill the hypothesis, its falsifier and
stop rule, and the exact condition that makes implementation justified. A
plausible chess-programming idea or donor-engine feature is not enough.

## Research and implementation ownership

- Research owns the causal question, competing hypotheses, interaction map,
  prospective prediction, falsifiers, experiment meaning and readiness
  decision. It should prefer cheap discriminating evidence over a sophisticated
  implementation of an uncertain idea.
- Implementation owns normal local engineering: idiomatic Rust structure,
  necessary local refactoring, focused instrumentation/tests, compilation,
  debugging and cheap deterministic qualification. The maintainer should not
  need to prescribe those ordinary steps.
- Implementation must not silently replace the hypothesis, broaden the chess
  mechanism, add adjacent heuristics, tune unrelated constants, port extra
  donor behavior, change the experiment after exposure or rescue a weak
  candidate by modifying neighbouring mechanisms. If a material premise is
  false, preserve useful instrumentation, record the contradiction and return
  the leaf to `RESEARCH`.
- Reference engines teach mechanisms, contracts, dependencies, failure modes
  and experimental methods. Their constants may be labelled seed values under
  PLAN's independence boundary; neither similarity nor a copied value is
  acceptance evidence.

For nontrivial playing work, explicitly check for shared signals and feedback:
search changes alter evaluation populations, evaluation changes alter pruning,
ordering evidence may also prune, TT semantics can mask a candidate, and
rule-50/repetition/promotion closure can make a local-looking feature non-local.
Use a bounded baseline/A/B/A+B screen when it cheaply distinguishes interaction;
do not require a factorial for every small change.

## Engineering judgment toward the CCRL goal

The maintainer aims for CCRL top 100, ideally top 50. Treat that as a direction
for measured strength, reliability and research prioritization, not a promised
ranking or a reason to accumulate familiar features. Judge progress by resolved
uncertainty and qualified results, not code volume or willingness to implement.

Before a substantial engine change, answer these four questions briefly in the
existing PLAN research card or experiment registration; link prior answers
instead of repeating them. Routine mechanical fixes need only the applicable
contract and focused check, not a new research document.

1. **Mechanism:** what causal mechanism should improve strength, correctness
   or useful speed, and what evidence says it is active in Rarog?
2. **Interactions:** which producers, consumers and shared signals interact
   with it; where could it duplicate, cancel or weaken an existing mechanism?
3. **Invariants:** which node/depth, TT, score/evaluation, history, board and
   protocol contracts must remain true, and how will the relevant ones be tested?
4. **Falsifier:** what cheapest observation would refute the explanation or
   reject the candidate, and what prospective rule stops further investment?

**Disagree plainly when warranted, including with the maintainer or an earlier
agent conclusion.** Say "I recommend against implementing this now" when the
evidence shows a conflicting contract, redundant mechanism, absent activation,
unfavorable cost or unresolved premise. Explain the specific evidence, distinguish
refutation from insufficient evidence, and give the cheapest alternative or
objective condition that would change the recommendation. An existing PLAN
checkbox or a strong donor engine is not evidence that the idea is worthwhile.
Do not manufacture objections or rejection quotas; supported positive results
deserve equally clear recommendations.

When a proposed change would violate a known correctness contract or registered
experimental rule, stop that change, explain the conflict and offer a valid
path. Do not silently implement it, relax the check or disguise uncertainty as
success. Ordinary engineering tradeoffs do not create a new permission loop:
make the recommendation and continue already-authorized independent work.

Keep the engine-specific design record in its existing owners: PLAN for current
contracts, decisions and dependencies; source/tests for executable invariants;
EXPERIMENTS and linked analysis for predictions, failures and retry triggers;
PROCESS for repeatable methods. Extend a missing contract there. Do not create
a parallel design summary that can drift, or reread the entire history per leaf.

## Expensive jobs and interruptions

Long tournaments/SPRTs, large datagen, expensive tuning, large PGO campaigns,
lengthy profiling and other machine-occupying jobs belong to the maintainer
unless the repository or user explicitly delegates them. The agent prepares
and verifies the command, inputs, configuration, artifacts and live wire, then
hands off the runnable job. Cheap local qualification remains the agent's job.

If the user interrupts work for a correction or scope change, finish and
qualify that requested correction, report it and return control. Do not resume
the interrupted objective unless the user explicitly asks.

## Predictions, negative results and research calibration

Freeze prospective predictions before result exposure. Afterwards append a
calibration/postmortem; never rewrite the prediction into retrospective
certainty. A persuasive after-the-fact explanation does not prove it was
predicted. Record which original assumption failed and whether the miss was in
sign, magnitude, mechanism, interaction, confidence or instrument.

`NO_CHANGE`, refuted, too sparse, low expected value, inappropriate interaction
and retry-trigger-not-fired are successful research outcomes. Do not create
implementation work to make the roadmap move, and do not retry a rejected idea
until its objective trigger fires. State the evidence layer: better loss,
nodes, NPS, depth, conversion, tactics or reference agreement is not
automatically Elo and has no implicit exchange rate to it.

## The one failure mode

Almost every mistake made in this repo by an agent has the same shape: *the
check that was run did not check what it was thought to check*. A stale binary
was measured, a parser silently read one record instead of forty, an exit code
came from the wrong end of a pipe. The engine was never the problem.

So: **verify mechanically, never by eyeballing, and never by assuming the tool
did what its name suggests.**

## Token-efficient execution

- **One orientation per session, not per tool call or leaf.** Read operating
  rules once, GUIDE's current/held overview, and the selected PLAN section.
  Use `rg` and bounded excerpts for follow-up. Re-read only changed regions
  or to answer a concrete unresolved question; do not repeatedly dump documents.
- Batch independent reads/checks. Send verbose output to logs and return exit
  status plus a short result; read full output only to investigate a failure.
- Keep a compact working record: leaf, accepted source/binary identity,
  evidence paths, completed checks, live process/session ID, blocker and next
  action. After interruption, inspect that record and existing outputs before
  restarting work. A usage interruption does not imply the process stopped.
- **Define the smallest sufficient measurement before launching it.** Follow
  the leaf's registered scope; do not expand decisive cases to every family,
  feature or engine by default. Record why any expansion is necessary before
  collecting results. Never shrink a registered run after seeing its results.
- **Reuse the harness.** Prefer its existing runner, parser and archive format.
  A one-off invocation does not need a new orchestration framework. Add a
  durable helper only for a missing correctness check or repeated workflow;
  keep it small. Do not create separate run/summary/archive tools by default.
- Run long jobs in a durable process with logs, output paths and exit status.
  Prefer completion notifications or bounded completion waits. After confirming
  startup, back off unchanged polls within host/tool limits; inspect logs at
  milestones, completion, errors or a credible stall. Do not repeatedly query
  both process lists and unchanged log tails. CPU waiting is not useful reasoning.
- Give milestone updates and report confounds immediately. If the host requires
  more frequent updates, keep them brief; do not perform extra inspections or
  repeat the analysis merely to manufacture something to say.
- **Write documentation once results are ready.** Prospective registration,
  newly discovered blockers and corrections to false claims cannot wait;
  routine "running" prose and partial result tables can. Keep raw progress in
  the job log, then update final status and evidence together.
- Before repeating a read, check, run or helper implementation, identify what
  changed or what unanswered question it resolves. If neither exists, skip it.
  Once the leaf's acceptance checks pass, commit and follow the authorized
  sequence; do not invent extra audits to fill elapsed compute time.

## Measurement

- **`--all-features` enables `texel`, which must never be measured.** The
  manifest says it bypasses the eval and pawn caches. `cargo test --release
  --all-features` leaves that binary in `target/release/rarog.exe`, and a
  depth sweep run on it produced a confident, wrong conclusion — reversed
  once rebuilt. The tell was that the BASELINE moved between two sweeps; if a
  number you are not changing changes, stop and check the binary.
- **Rebuild before measuring, with the exact feature set.**
  `cargo test`, `cargo clippy` and `cargo bench` all build the `rarog` binary
  too, with *their* features, and leave it in `target/release/rarog.exe`. A
  differential run was voided this way. There is no such thing as "the binary
  is probably still right". For a multi-run study, build once, verify its
  fingerprint, archive/hash that executable and measure that immutable copy.
  Rebuild if source, features, toolchain or build settings change; a hash-verified
  copy does not need rebuilding for every budget. Keep test builds separate.
- `bench` dumps diagnostic counters **once per position** — 40 lines per name
  for `bench 13`, 47 for the oracle. They must be **summed**. Reading the last
  one gives a single position's numbers that look plausible and are wrong.
- **Never hand-roll a counter parser.** Use `tools/diag/bench_counters.py` for
  bench and `tools/diag/phase4_differential.py` for the suite. Both aggregate
  correctly.
- Counter ratios are only valid at `RAROG_DIAG_SAMPLE_STRIDE=1`. Half the core
  counters are sampled and half are exact, deliberately; see
  `analysis/phase4_counter_spec.md`.
- Before differencing two counters, check they are in the **same unit**. Per
  node vs per move has produced three false findings in this project
  (RAR-S25, and twice inside the matched-ablation instrumentation itself). A passing
  invariant does not prove comparability — `probcut_cut <= probcut_attempt`
  held for two phases while the two counters counted different things.

## Verification

- **Scope checks to the change.** Rust engine/test/build/dependency changes
  require debug AND release tests, `cargo fmt --check`, and
  `cargo clippy --all-features --all-targets`, with zero warnings. Debug-only
  failures have occurred; release alone is insufficient.
- Documentation-only changes need diff/link/status consistency checks, not
  Cargo builds, engine tests or bench. Run `check_guide.py` when GUIDE/PLAN
  structure or status changes. Python/tooling changes need affected tooling
  tests and a meaningful smoke/negative check of changed measurement paths;
  they do not automatically require rebuilding an unchanged engine.
- **Run each required check once on the final relevant state.** Record its
  command, exit status and covered source/configuration. Reuse that pass while
  those inputs are unchanged, including across a following docs-only commit.
  Rerun affected checks after edits, failures or new evidence of a gap; do not
  rerun the whole suite after prose edits. Never call a prior pass newly run.
- **Suppress lints with `#[expect(...)]`, not `#[allow(...)]`.** An
  expectation warns when it stops being needed, so the suppression list
  cleans itself; an `allow` sits there forever. Converting the crate's 25
  found six that had been dead for some time. Use `allow` only when the
  lint fires in one feature configuration and not another — there is
  exactly one such site, in `search_options.rs`, and it says so. Every
  suppression still needs a written reason.
- **Check exit status directly**, never through a pipe: `cmd > out 2>&1; echo
  $?` and then read `out`. `cmd | tail` reports `tail`'s status, which is
  always 0.
- **Every scripted edit must assert its anchor matched.** A `str.replace` that
  finds nothing changes nothing and reports success. If you edit with a script,
  `assert old in text` before writing, and re-read the region after. Assert the anchor is **unique** and lands in executable code:
  a `--rset` block anchored on a line that also appears in the module
  docstring was inserted as prose, parsed fine, and silently measured
  default parameters in every run for two screens.
- **Prove a harness wire is live before trusting a null from it.** Set a
  deliberately absurd value and require the numbers to move. Two candidates
  were recorded as null results by a dead `--rset`; one of them, re-measured,
  moved oracle agreement 66% -> 78%. Verifying the ENGINE responds is not the
  same check -- a standalone probe confirmed the option worked while the
  instrument reporting on it did not.
- A behavior-neutral **engine change** must reproduce the immediate development
  production `bench 13` fingerprint (currently **7,601,220 / EBF 2.474**),
  plus targeted checks for changed behavior the suite does not reach. The
  fingerprint includes the RAR-M28 SEE repair and was accepted by RAR-E15. The
  RAR-E12-era mate drive changed KBN-K conversion 19.4% -> 96.9% with an
  identical bench: fingerprint equality alone is not proof of narrow-feature
  neutrality.
  RAR-P14/P16 establish cross-platform fingerprint agreement; investigate a
  mismatch, do not dismiss it as platform noise. For docs-only work, verify
  the diff contains no engine inputs rather than running a neutrality bench.

## Changes

- Engine changes and tooling/doc changes go in **separate commits**.
- Commit after each finished **and verified** step, not after each edit.
- **No `Co-Authored-By` trailers.**
- A correctness test is never relaxed in the same commit as the change that
  made it fail. Fix its precondition, in its own commit, with the measurement
  that justifies it.
- Counters explain a candidate; only a registered SPRT accepts one. Node counts
  are not Elo: a measured +7.36% tree change was worth −1.49 ± 2.87 Elo.

## Evidence

- **Development and raw evidence live on this machine.** macOS and Windows
  on ARM are compatibility-test hosts, not separate development workspaces.
  Keep source, build/CI files, reusable tools, required test fixtures, and
  concise design/result records in Git. Keep raw runs, logs, executables,
  profiling traces, bundles and scratch outputs in ignored `analysis/artifacts/`
  or `tools/results/`; never force-add them merely to preserve evidence.
  Record local paths, recipes and hashes in the tracked analysis/ledger.
  Small frozen datasets actually consumed by tests or tools remain versioned.
  Compatibility checks must not depend on this machine's private run outputs.
  See `analysis/README.md` for storage boundaries. Untracking uses `git rm
  --cached` and preserves local bytes; it does not authorize deleting evidence
  or rewriting Git history.

- **A ledger row must reproduce its artifact without the branch it came from.**
  Record the recipe — exact parameter values, or the diff when it is small —
  plus a fingerprint that proves a rebuild matched. A bare SHA is not evidence;
  it is a promise that someone else is still storing your evidence.
- Before deleting any branch or tag, check what the ledger cites on it. A SHA
  with no output from `git branch -a --contains <sha>` is **dangling** and will
  disappear at the next `gc`.
- RAR-S54 cited a docs-only commit while its real source was dangling on a
  deleted branch. Its twelve parameter values now live in `EXPERIMENTS.md`;
  retain such recipes with the evidence, not only in branch history.

## Gating

- The strength unit is one dependency-complete, locally fitted **cluster**.
  Internal substeps are not expected to win standalone and do not get their own
  gates.
- Register in `EXPERIMENTS.md` — hypothesis, baseline SHA, gate, cap, stop rule
  — **before any games**. Never change bounds, cap, book or adjudication after
  seeing games.
- **`[0,3]` nElo is the DEFAULT bracket.** Widen only when the prior is
  genuinely large, and say why in the registration. This is not "narrow is
  better": the ProbCut cluster (RAR-S57) had a 25–60 nElo prior, measured +24.90, and `[3,10]` resolved
  it in **2,838 games** — a wide bracket is the right instrument for a large
  effect. The error is using one for a small candidate. Compute the games at
  the EXPECTED value from RAR-M10 before choosing, every time.
- **A removal or simplification needs a bracket that permits a small loss**,
  fishtest-style (`[-1.75, 0.25]`), not `[0,3]`. A repair of unknown sign wants
  a symmetric bracket that can detect harm — RAR-S62 used `[-5,5]` and resolved
  in 4,436 games.
- **The harness already runs GSPRT; nothing to change.** `tools/sprt.ps1`
  passes `model=normalized` to fastchess and the output carries `Ptnml(0-2)`,
  so it is the pentanomial GSPRT — the same mathematics fishtest uses, with
  nuisance parameters replaced by maximum-likelihood estimates. The gap between
  this project and fishtest is bounds and budget, never the test.
- **High bounds reject small gains, not merely slowly resolve them.** RAR-M10
  estimates a true +4 nElo reaches H0 under `[0,10]` in ~35k games and `[3,10]`
  in ~20k; `[0,3]` accepts it in ~47k. At the measured ~98 games/min this is
  overnight compute, not an excuse to widen bounds or consume tokens polling.
  Bench/counter screening chooses which candidates earn a gate; it never
  accepts strength (RAR-S64's clean bench signal measured zero in games).
- **Do not invent an acceptance rule after seeing a result.** A threshold like
  "accept if the CI excludes zero at 20,000 games" is arbitrary and is the same
  act as moving the bounds. If small gains need to be bankable, register the
  narrower bracket PROSPECTIVELY.
- **An unresolved stop is not "probably fine".** RAR-S61 measured
  +4.50 ± 3.50 at LOS 99.41% and the entire effect turned out to be a stale-read
  bug (RAR-S64 re-measured it at +0.39 once fixed). A high LOS on a point
  estimate is not evidence that a mechanism works.
- **SPSA is conditional, not owed.** PLAN rule 4 says "only when activation,
  interaction and curvature justify the cost". Establish that first with a
  zero-game sweep over the suite or bench; a flat or monotone surface is
  evidence *against* spending it.

## Documents

- **`GUIDE.md` and `PLAN.md` are updated in the SAME commit.** GUIDE is the
  short operator contract/current-model mapping plus the overview of PLAN's
  current state and ordered steps. A
  GUIDE that disagrees with PLAN is worse than no GUIDE, because it is the
  file that says what to do next and it will be believed. This applies when
  roadmap status or requirements change; an AGENTS-only operating-rule edit
  does not require unrelated PLAN/GUIDE churn.
- **GUIDE carries STATUS, not just a list.** Its phase checkboxes are how
  the maintainer sees what is done. Tick one only when the step is finished
  AND verified, in the commit that finishes it — never in advance.
- **Tick the PARENT when its last sub-step is ticked.** A step whose sub-steps
  are all done is done; leaving it open makes finished work look outstanding.
- **Sub-items indent by 4 spaces, never 6.** Under a `- ` parent the content
  column is 2, so 6 spaces is the indented-code threshold and the sub-list
  silently renders as a code block. Both rules are checked mechanically —
  run it rather than reading the file:

  ```bash
  python tools/diag/check_guide.py
  ```
- **Keep GUIDE short.** Outside its operator contract, model mapping and two
  reusable prompts, a change that runs past a few lines belongs somewhere
  else: what a step INVOLVES goes in `PLAN.md`; a completed step's
  record goes in `HISTORY.md`; a repeatable procedure goes in `PROCESS.md`;
  durable evidence goes in `EXPERIMENTS.md`; a measurement's derivation goes in
  `analysis/`. GUIDE grew to 898 lines by absorbing all five and stopped being
  readable as an overview.
- `HISTORY.md` is HISTORY. Every numbering scheme in it is retired; the
  current roadmap uses lettered phases (`A.2.1`) and PLAN section 6 maps the
  retired Phase-4 identifiers onto it. Never take a next step from HISTORY or
  from `docs/archive/`.
- When two documents disagree, source, defaults and reproducible artifacts
  outrank prose. Fix the prose in the same change.

## Step sequencing and explicit holds

- Work one executable leaf at a time: verify proportionately, update PLAN and
  GUIDE in the same documentation commit, and report. If the maintainer
  authorized a multi-step session, continue to the next eligible leaf without
  asking again; otherwise stop at the requested scope. Engine and
  tooling/doc changes still go in separate commits; intermediate commits do
  not falsely mark an unfinished cluster accepted.
- Read GUIDE's current/held overview and PLAN's dependency register before
  selecting work. The earliest open leaf may be held. Keep its checkbox/ID,
  reason, unblock condition and latest required completion point visible.
  Review holds each handoff; resume the earliest eligible one. Never silently
  skip, move or tick missing verification.
- The agent may use check_guide.py internally for structural consistency.
  Its raw open-item list is not a scheduler and does not resolve holds or
  dependencies. The maintainer need not run it; always state the next
  executable step and any held obligation that matters.
- Report confounds when found. Correct contradicted current claims in
  PLAN/GUIDE/analysis/EXPERIMENTS where applicable; preserve historical
  measurements with explicit supersession rather than silently deleting them.
- Test constructs and behavior, not a word in a comment/disclaimer. Check
  each command's actual exit status and require every intended check to have
  run successfully before committing; do not rely on a chained command's
  final status as proof of earlier checks.
- The comparison fingerprint is revision-specific. For a neutral change compare
  to the exact immediate development baseline (currently 7,601,220 / EBF 2.474)
  and targeted cases. A deliberately integrated behavior change updates the
  fingerprint record; never preserve a known defect to force an obsolete count.

## Handing work back

- When maintainer action is needed, give runnable commands in their own fenced
  block and restate them rather than referring back. Routine internal checks
  need not become user chores. Always name the next executable leaf.
- **Whenever reporting the next step, use its PLAN capability class and the
  current GUIDE mapping to recommend one GPT model AND one Claude model**, with
  a brief task-specific reason. GUIDE's table is the single maintainer-edited
  source for names and versions; do not automatically substitute newer models.
  Prefer the least costly model judged sufficient for the defined task;
  reserve deeper review for unresolved design, interaction or correctness
  questions. Recommendations are task judgments, not guarantees or claims of
  measured model superiority. Do not change the active model automatically.
- **PLAN records stable capability classes, not vendor generations.** Do not
  silently downgrade a recorded class. If changed scope or new
  correctness/design uncertainty calls for escalation, state why and update
  PLAN and GUIDE together. The current mapping is a recommendation, not an
  automatic model change.
- **Pair EACH recommended model with its own thinking mode**: Medium, High,
  Extra High, or a stronger mode using that model's actual supported name.
  Choose effort independently for each model and the specific next leaf;
  do not automatically give both models the same mode or assume their effort
  labels are equivalent. Recommend the lowest effort judged sufficient for
  the task's ambiguity and verification demands, not its compute duration.
  Use `GPT: <model> — <mode>; Claude: <model> — <mode>` plus a brief reason.
  If support or the exact mode name is unknown, say so rather than inventing
  a setting. A recommendation does not authorize changing active settings.
- Report what was actually measured. If a step was skipped or a result is
  partial, say so plainly.
- For a multi-step session, give a short summary per completed leaf: ID,
  result/change, verification, commit. Name unfinished work separately. Do not
  recount every tool call or imply planned work was implemented.
