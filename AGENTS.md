# Agent operating rules for Rarog

The rules an agent follows while working on Rarog. `GUIDE.md` says what to work
on; the relevant section of `PLAN.md` says why; `PROCESS.md` holds the
procedures these rules assume. Each rule is stated once, here. The incidents
that produced a rule are in the ledger rows it cites or in `HISTORY.md`.

## Classify the work

- Before substantial work, name its primary kind: research/diagnosis,
  experiment design, implementation, deterministic qualification, performance
  qualification, playing-strength gate, or documentation/provenance. A roadmap
  leaf is not automatically an instruction to write code. PLAN owns each
  leaf's workflow state and capability class; GUIDE maps classes to models.
- The workflow is `RESEARCH -> READY_FOR_IMPLEMENTATION -> IMPLEMENTED ->
  LOCAL_QUALIFIED -> GAME_GATE -> CLOSED`. Not every task needs every state:
  documentation can close without games, research can close `NO_CHANGE`,
  neutral performance work closes on deterministic and performance
  qualification. A playing-strength change does not bypass `GAME_GATE`.
- `READY_FOR_IMPLEMENTATION` is a hard boundary. A substantial playing change
  crosses it only with: the measured defect or opportunity and its evidence in
  this engine, credible competing explanations, interacting mechanisms, the
  cheapest test that can kill the hypothesis, its falsifier and stop rule, and
  the condition that justifies implementation. A plausible idea or a donor
  feature is not enough.

## Research and implementation

- Research owns the causal question, competing hypotheses, interaction map,
  prospective prediction, falsifiers, the meaning of each experiment and the
  readiness decision. Prefer cheap discriminating evidence to a sophisticated
  implementation of an uncertain idea.
- Implementation owns ordinary engineering: idiomatic Rust structure, necessary
  local refactoring, focused instrumentation and tests, compilation, debugging
  and cheap deterministic qualification. The maintainer need not prescribe it.
- Implementation does not replace the hypothesis, broaden the mechanism, add
  adjacent heuristics, tune unrelated constants, port extra donor behaviour,
  change the experiment after exposure, or rescue a weak candidate by changing
  its neighbours. If a material premise is false, keep useful instrumentation,
  record the contradiction and return the leaf to `RESEARCH`.
- Donor engines teach mechanisms, contracts, dependencies, failure modes and
  methods. What may cross is `PROCESS.md`, *The independence boundary*. Neither
  similarity nor a copied value is acceptance evidence.
- For nontrivial playing work, check shared signals and feedback: search
  changes move evaluation populations, evaluation changes move pruning,
  ordering evidence may also prune, TT semantics can mask a candidate, and
  rule-50, repetition and promotion closure can make a local feature non-local.
  Use a bounded baseline/A/B/A+B screen when it cheaply separates interaction;
  do not require a factorial for every small change.

## Judgment toward the CCRL goal

- The aim is CCRL top 100, ideally top 50: a direction for measured strength,
  reliability and prioritisation, not a promised ranking or a reason to
  accumulate features. Progress is resolved uncertainty and qualified results.
- Before a substantial engine change, answer in the PLAN research card or the
  experiment registration (link earlier answers): **mechanism** — what should
  improve and what evidence says it is active here; **interactions** — which
  producers, consumers and shared signals, and where it could duplicate,
  cancel or weaken an existing mechanism; **invariants** — which node, TT,
  score, history, board and protocol contracts must hold and how they are
  tested; **falsifier** — the cheapest refuting observation and the rule that
  stops further investment. Routine mechanical fixes need only the contract and
  a focused check.
- Disagree plainly when the evidence warrants, including with the maintainer
  or an earlier conclusion: say "I recommend against implementing this now",
  give the evidence, distinguish refutation from insufficient evidence, and
  name the cheapest alternative or the condition that would change the
  recommendation. A PLAN checkbox or a strong donor is not evidence. Do not
  manufacture objections; a supported positive result gets an equally clear
  recommendation.
- When a change would violate a known correctness contract or a registered
  experimental rule, stop that change, explain the conflict and offer a valid
  path. Ordinary engineering tradeoffs do not open a permission loop.
- Keep the design record in its owners: PLAN for contracts, decisions and
  dependencies; source and tests for executable invariants; EXPERIMENTS and
  linked analyses for predictions, failures and retry triggers; PROCESS for
  repeatable methods. Extend a missing contract there, not in a parallel
  summary.

## Expensive jobs and interruptions

- Long tournaments and SPRTs, large datagen, expensive tuning, large PGO
  campaigns and lengthy profiling belong to the maintainer unless explicitly
  delegated. Prepare and verify the command, inputs, configuration, artifacts
  and live wire, then hand over the runnable job. Cheap local qualification is
  the agent's job.
- When the user interrupts for a correction or scope change, finish and
  qualify that correction, report it and return control. Resume the earlier
  objective only when asked.

## Predictions and negative results

- Freeze predictions before exposure. Afterwards append a calibration: which
  assumption failed, and whether the miss was in sign, magnitude, mechanism,
  interaction, confidence or instrument. Never rewrite a prediction into
  retrospective certainty.
- `NO_CHANGE`, refuted, too sparse, low expected value, inappropriate
  interaction and retry-trigger-not-fired are successful outcomes. Do not
  create work to make the roadmap move, and do not retry a rejected idea until
  its trigger fires.
- State the evidence layer. Loss, nodes, NPS, depth, conversion, tactics and
  reference agreement are not Elo and have no implicit exchange rate to it.

## The one failure mode

Almost every agent mistake here has been a check that did not check what it
was thought to check: a stale binary measured, one record parsed instead of
forty, an exit code read from the wrong end of a pipe. **Verify mechanically,
never by eyeballing, and never by assuming a tool did what its name says.**

## Token-efficient execution

- Orient once per session: operating rules, GUIDE's current and held overview,
  the selected PLAN section. Follow up with `rg` and bounded excerpts; re-read
  only changed regions or to answer a concrete question.
- Batch independent reads and checks; send verbose output to logs and return
  exit status plus a short result. Run long jobs durably with logs and exit
  status, and back off unchanged polls: waiting on the CPU is not reasoning.
- Keep a compact working record: leaf, source and binary identity, evidence
  paths, completed checks, live process IDs, blocker, next action. After an
  interruption, inspect it and existing outputs before restarting anything.
- Define the smallest sufficient measurement before launching it, inside the
  leaf's registered scope; never shrink a registered run after seeing results.
  Reuse the harness's runner, parser and archive format. Before repeating any
  read, check or run, name what changed or what it answers; if nothing, skip
  it. Report confounds immediately; write documentation once results are
  ready, except registrations, new blockers and corrections to false claims.

## Measurement

- Colosseum CLI is the main harness for gates, fixed matches, tunes, null pairs
  and gauntlets; `tools/colosseum.ps1` drives it from the committed run files
  and carries every guard. fastchess and weather-factory (`tools/sprt.ps1`,
  `tools/spsa.ps1`) stay installed and working as the backup and the second
  opinion until at least release 2.5.0; retire nothing before then. PROCESS's
  *Harness* section names the cross-check triggers.
- Never measure a `--all-features` binary: it enables `texel`, which bypasses
  the eval and pawn caches. If a number you are not changing changes, check
  the binary.
- Measure only on an idle host: check CPU use and running engine or harness
  processes first, and if the machine is busy stop and ask rather than measure
  (RAR-M48's first pool was discarded for this). Keep builds, profiling and
  unrelated load off a match host while it plays. `colosseum.ps1` refuses a
  busy host; a waiver is recorded in the run's manifest, so it cannot be
  forgotten.
- Rebuild before measuring, with the exact feature set: `cargo test`,
  `clippy` and `bench` leave their own `target/release/rarog.exe`. For a
  multi-run study, build once, verify the fingerprint, hash and archive that
  executable, and measure the copy; rebuild only when source, features,
  toolchain or build settings change. Keep test builds separate.
- `bench` dumps counters once per position (40 lines per name for `bench 13`,
  47 for the oracle suite); sum them. Use `tools/diag/bench_counters.py` and
  `tools/diag/phase4_differential.py`; never hand-roll a counter parser.
  Counter ratios are valid only at `RAROG_DIAG_SAMPLE_STRIDE=1`
  (`analysis/phase4_counter_spec.md`).
- Before differencing two counters, confirm they are in the same unit (per
  node versus per move produced RAR-S25). A passing invariant does not prove
  comparability.
- A binary entered in a rated pool is a tagged release or carries its bench
  fingerprint or feature flag in its version string (`2.5.0-dev+b2core`), and
  its ledger row names the fingerprint. The harness binary is pinned the same
  way: `tools/colosseum/colosseum.pin.json` names its revision and SHA-256, and
  a runner that is not the pinned one is refused, not substituted.
- The current fingerprint is declared once, in GUIDE's checkpoint;
  `check_guide.py` fails when AGENTS' "currently" values or PLAN's checkpoint
  row disagree with it.

## Verification

- Scope checks to the change. Rust engine, test, build or dependency changes
  need debug and release tests, `cargo fmt --check` and
  `cargo clippy --all-features --all-targets` at zero warnings; release alone is
  not enough. Documentation-only changes need diff, link and status checks, and
  `check_guide.py` when GUIDE or PLAN structure changes, and no Cargo builds,
  engine tests or bench. Tooling changes need their tests and a meaningful
  smoke or negative check of the changed path, not a rebuilt engine.
- Run each required check once on the final relevant state, record command,
  exit status and coverage, and reuse the pass while its inputs are unchanged.
  Rerun after edits, failures or evidence of a gap; never call an old pass new.
- Suppress lints with `#[expect(...)]`, which warns when no longer needed; use
  `#[allow(...)]` only for a lint that fires in one feature configuration and
  not another, with its reason written.
- Check exit status directly: `cmd > out 2>&1; echo $?`, never through a pipe.
- Every scripted edit asserts its anchor matched, is unique and lands in
  executable code; re-read the region afterwards.
- Prove a harness wire is live before trusting a null from it: set an absurd
  value and require the numbers to move. Proving the engine responds is not
  proving the instrument reports it.
- A behaviour-neutral engine change reproduces the immediate development
  fingerprint (currently **7,435,006 / EBF 2.457**; the legacy search that
  `--no-default-features` still compiles reads 7,601,220 / EBF 2.474) plus
  targeted checks for
  behaviour the suite does not reach: an identical bench does not prove a
  narrow feature neutral (RAR-E10). Investigate a cross-platform mismatch
  (RAR-P14, RAR-P16). Docs-only work verifies the diff has no engine inputs.
  The comparison fingerprint is revision-specific: a deliberately integrated
  behaviour change updates the fingerprint record; never preserve a known
  defect to keep an obsolete count.
- Test constructs and behaviour, not words in a comment.

## Changes

- Engine changes and tooling or documentation changes go in separate commits.
  Commit after each finished and verified step. No `Co-Authored-By` trailers.
  Never relax a correctness test in the commit whose change made it fail; fix
  its precondition in its own commit, with the justifying measurement.
- Never push, tag, publish or merge to `master`; the maintainer does, on
  instruction. Do not amend or rewrite a commit that has left this machine.
- Most of the tree is CRLF. A scripted edit preserves the file's existing line
  endings, asserts each anchor is present exactly once, and re-reads the
  region afterwards; a mixed-ending file or a silently unmatched anchor is a
  failed edit.
- Counters explain a candidate; only a registered SPRT accepts one. Node counts
  are not Elo: a +7.36% tree change measured −1.49 ± 2.87 Elo.
- Comments explain the problem or the invariant, briefly: no roadmap step
  numbers, no ledger IDs as the explanation, no narration of earlier versions.
  A measured reason to keep a shape stays, in one sentence. Rewrite a
  mechanism's comments when you rewrite the mechanism.

## Evidence

- Development and raw evidence live on this machine; macOS and Windows on ARM
  are compatibility-test hosts. Git holds source, build and CI files, reusable
  tools, required fixtures and concise records; raw runs, logs, executables,
  traces and bundles stay in ignored `analysis/artifacts/` or `tools/results/`
  with their paths, recipes and hashes recorded (`analysis/README.md`). Small
  frozen datasets that tests or tools consume stay versioned. Compatibility
  checks never depend on this machine's private outputs. Never force-add
  evidence. Untracking uses `git rm --cached`, keeps the local bytes, and
  authorises neither deleting evidence nor rewriting history.
- A ledger row reproduces its artifact without the branch it came from: the
  recipe (exact values, or a small diff) plus a fingerprint proving a rebuild
  matched. A bare SHA is not evidence; keep recipes with the evidence, not
  only in branch history (RAR-S54).
- Before deleting a branch or tag, check what the ledger cites on it
  (`git branch -a --contains <sha>`; an empty answer means dangling).

## Gating

- The strength unit is one dependency-complete, locally fitted cluster;
  internal sub-steps get no gates of their own. Register it in `EXPERIMENTS.md`
  (hypothesis, baseline SHA, gate, cap, stop rule) before any games, and never
  change bounds, cap, book or adjudication after seeing games.
- `[0,3]` nElo is the default bracket. Widen only for a genuinely large prior
  and say why; a wide bracket resolves a large effect fast (RAR-S57, `[3,10]`,
  2,838 games). Compute the games at the expected value from RAR-M10 first.
- A removal or simplification uses a bracket that permits a small loss
  (`[-1.75, 0.25]`); a repair of unknown sign uses a symmetric one (RAR-S62,
  `[-5,5]`).
- Both harnesses run the pentanomial GSPRT (`model=normalized`); the gap to
  fishtest is bounds and budget, not the test. A registered experiment names
  its runner and never changes it mid-way.
- High bounds reject small gains: a true +4 nElo reaches H0 under `[0,10]` in
  about 35k games and is accepted by `[0,3]` in about 47k (RAR-M10). That is
  overnight compute: budget the games, do not widen the bounds or poll.
- Bench and counter screens choose candidates and never accept strength
  (RAR-S64). Do not invent an acceptance rule after seeing a result; register a
  narrower bracket prospectively if small gains must be bankable.
- An unresolved stop is not "probably fine": a high LOS on a point estimate is
  not evidence the mechanism works (RAR-S61, RAR-S64).
- SPSA is conditional (PLAN rule 4): first show activation, interaction and
  curvature with a zero-game sweep; a flat or monotone surface is evidence
  against the tune. A tune runs in registered blocks with a movement stop
  rule (PLAN rule 7c), never on a horizon chosen to fit the answer.

## Documents

- `GUIDE.md` and `PLAN.md` change in the same commit when roadmap status or
  requirements change; an AGENTS-only edit needs no PLAN or GUIDE churn.
- GUIDE carries status. Tick a step only when finished and verified, in the
  commit that finishes it; tick the parent when its last sub-step is ticked.
- Sub-steps indent by 4 spaces and addenda (`B.2.0.1`) by 8, never 6 (6
  renders as code); nothing goes deeper than three levels. Run
  `python tools/diag/check_guide.py` rather than reading the file.
- Keep GUIDE short: its operator contract, model mapping, two prompts, board
  and checkpoint. What a step involves goes in PLAN, a completed record in
  HISTORY, a procedure in PROCESS, evidence in EXPERIMENTS, a derivation in
  `analysis/`.
- `HISTORY.md` is history and resolves every retired numbering scheme; never
  take a next step from it or from `docs/archive/`. When documents disagree,
  source, defaults and reproducible artifacts outrank prose; fix the prose in
  the same change.

## Sequencing and holds

- Work one executable leaf at a time: verify proportionately, update PLAN and
  GUIDE in one documentation commit, report. In a multi-step session continue
  to the next eligible leaf without asking; otherwise stop at the requested
  scope. Intermediate commits never mark an unfinished cluster accepted.
- Read GUIDE's current and held overview and PLAN's register before selecting
  work. Keep a held leaf's ID, reason, unblock condition and latest completion
  point visible; review holds at each handoff and resume the earliest
  eligible one. Never silently skip, move or tick missing verification.
- `check_guide.py` checks structure; its open-item list is not a scheduler and
  resolves no hold or dependency. Always state the next executable step and
  any held obligation that matters.
- Report confounds when found; correct contradicted current claims where they
  live and preserve historical measurements with explicit supersession.

## Handing work back

- When maintainer action is needed, give runnable commands in their own fenced
  block and restate them rather than referring back. Routine internal checks do
  not become the maintainer's chores. Always name the next executable leaf.
- When reporting the next step, use its PLAN capability class and GUIDE's
  mapping to recommend one GPT model and one Claude model, each with its own
  thinking mode (`GPT: <model> — <mode>; Claude: <model> — <mode>`) and a brief
  task-specific reason. Prefer the least costly model judged sufficient and
  reserve deeper review for unresolved design, interaction or correctness
  questions; choose each mode independently at the lowest sufficient effort;
  never substitute newer models, and say so when a mode name is unknown rather
  than inventing one. A recommendation is a task judgment, not a guarantee, and
  changes no active setting.
- PLAN records capability classes, not vendor generations. Do not silently
  downgrade a class; if scope or uncertainty calls for escalation, say why and
  update PLAN and GUIDE together.
- Report what was actually measured, and say plainly when a step was skipped
  or a result is partial. For a multi-step session, summarise each completed
  leaf (ID, result, verification, commit) and name unfinished work separately.
