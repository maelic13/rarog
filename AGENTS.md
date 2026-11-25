# Agent operating rules for Rarog

The rules an agent follows while working on Rarog. `docs/PLAN.md` says what to
work on and why; `docs/PROCESS.md` holds the procedures these rules assume.
`GUIDE.md` is the maintainer's board: agents keep it current and never orient
or take a step from it (maintainer decision 2026-11-25). Each rule is stated
once: here, or in the module the index below names. The incidents
that produced a rule are in the ledger rows it cites or in `docs/HISTORY.md`.
Read this file in full from disk before work: some agent tools load rule
files only up to a size limit, and the last sections bind as much as the
first.

## Rule modules

This file is the core: every tool loads it whole, so it holds only what binds
on every task. The rules for particular work are in the modules below, and a
module binds exactly as this file does. Read a module in full from disk as
soon as one of its lines applies; when unsure whether one applies, read it.
A module is skipped only because no trigger fired, never to save time.

### `docs/rules/research.md`
- Scope: research, experiment design and the readiness of a playing change.
- Kind: research/diagnosis or experiment design; any change meant to alter how the engine plays, a repair included.
- Paths: `src/`, `analysis/`, `docs/experiments/`.
- Before: writing a research card, packet, prediction or experiment registration; moving a leaf to or from `READY_FOR_IMPLEMENTATION`; opening another engine's source or taking a mechanism, value, table or patch from one.

### `docs/rules/engine.md`
- Scope: implementing and qualifying a change to the engine's code or build.
- Kind: implementation or deterministic qualification.
- Paths: `src/`, `tests/`, `benches/`, `xtask/`, `vendor/`, `.cargo/`, `build.rs`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`.
- Before: suppressing a lint; calling a change behaviour-neutral.

### `docs/rules/measurement.md`
- Scope: producing or reading a number: the binary, the host, the harness, the counters.
- Kind: performance qualification; any step whose result is a measured number.
- Paths: `tools/`, `analysis/`, `docs/experiments/`, `docs/EXPERIMENTS.md`.
- Before: running, or preparing for the maintainer, a bench, NPS read, profile, match, gauntlet, SPRT, tune or datagen; building a binary that will be measured; summing, differencing or quoting a counter, timing or size.

### `docs/rules/gating.md`
- Scope: registering a strength experiment and deciding what accepts a change.
- Kind: playing-strength gate; a leaf at or entering `GAME_GATE`; any change to how the engine plays, a repair included, to settle what gates it.
- Paths: `src/`, `docs/experiments/`, `docs/EXPERIMENTS.md`.
- Before: choosing a bracket, cap, harm rule or tune; preparing a gate, fixed-read or SPSA command; writing that a change is accepted, kept, rejected or stronger.

### `docs/rules/documents.md`
- Scope: the roadmap documents, the owners of the record, the changelog and the README.
- Kind: documentation/provenance; recording or closing any step.
- Paths: `GUIDE.md`, `docs/PLAN.md`, `docs/HISTORY.md`, `docs/PROCESS.md`, `docs/archive/`, `CHANGELOG.md`, `README.md`.
- Before: marking a step `DONE`, `CLOSED` or `NO_CHANGE`; a leaf's documentation commit; landing a change a user of the engine would notice; writing release notes.

### `docs/rules/evidence.md`
- Scope: where evidence lives, how a commit is cited and preserved, branches and tags.
- Kind: provenance work; a phase close.
- Paths: `analysis/`, `tools/results/`, `tools/diag/citation_foreign.tsv`, `.gitignore`, `.gitattributes`.
- Before: writing a commit hash, branch or tag into a document; deleting or keeping a branch; deleting a tag; `git add -f`, `git rm` or untracking a file; archiving a binary, log or run output; asking for a pull request.

## Classify the work

- Before substantial work, name its primary kind: research/diagnosis,
  experiment design, implementation, deterministic qualification, performance
  qualification, playing-strength gate, or documentation/provenance. A roadmap
  leaf is not automatically an instruction to write code. PLAN owns each
  leaf's workflow state and capability class; *Handing work back* maps
  classes to models.
- The workflow is `RESEARCH -> READY_FOR_IMPLEMENTATION -> IMPLEMENTED ->
  LOCAL_QUALIFIED -> GAME_GATE -> CLOSED`. Not every task needs every state:
  documentation can close without games, research can close `NO_CHANGE`,
  neutral performance work closes on deterministic and performance
  qualification. A playing-strength change does not bypass `GAME_GATE`.

## The one failure mode

Almost every agent mistake here has been a check that did not check what it
was thought to check: a stale binary measured, one record parsed instead of
forty, an exit code read from the wrong end of a pipe. **Verify mechanically,
never by eyeballing, and never by assuming a tool did what its name says.**

## Verification

- Scope checks to the change. Documentation-only changes need diff, link and status checks, and
  `check_guide.py` when GUIDE or PLAN structure changes, and no Cargo builds,
  engine tests or bench. Tooling changes need their tests and a meaningful
  smoke or negative check of the changed path, not a rebuilt engine.
  Docs-only work verifies the diff has no engine inputs.
- Run each required check once on the final relevant state, record command,
  exit status and coverage, and reuse the pass while its inputs are unchanged.
  Rerun after edits, failures or evidence of a gap; never call an old pass new.
- Check exit status directly: `cmd > out 2>&1; echo $?`, never through a pipe.
- Test constructs and behaviour, not words in a comment.

## Changes

- Engine changes and tooling or documentation changes go in separate commits.
  Commit after each finished and verified step. Never relax a correctness
  test in the commit whose change made it fail; fix its precondition in its
  own commit, with the justifying measurement.
- No attribution to Claude or any other AI tool, anywhere: no
  `Co-Authored-By` trailer, no "Generated with" line, in commits, PR titles
  and descriptions, squash messages or files. The maintainer adds
  attribution where they want it (maintainer decision 2026-10-09;
  `.claude/settings.json` turns Claude Code's own attribution off).
- Never push, create a tag, publish or merge to `master`; the maintainer
  does, on instruction. Do not amend or rewrite a commit that has
  left this machine. Archive tags follow *Evidence*, in
  `docs/rules/evidence.md`.
- `master` is the trunk. Each coherent piece of work (a PLAN unit such as
  C.3, or one between-units change: tooling, documentation, a research
  close) lives on its own branch and reaches `master` as one squash commit
  through a PR the maintainer merges; `master` may run ahead of the latest
  release, which is a tag on it (maintainer decision 2026-10-09, replacing
  the merge commits of 2026-10-05; the procedure is PROCESS, *Branches and
  pull requests* and *Release*). The small, separate commits above stay on
  the branch.
- Never force-add evidence. Untracking uses `git rm --cached`, keeps the
  local bytes, and authorises neither deleting evidence nor rewriting history.
- `dragon_nnue` is kept, locally and on GitHub: never delete it, whatever a
  review finds (maintainer decision 2026-11-18). Deleting a remote tag or
  branch is the maintainer's command.
- Most of the tree is CRLF. Every scripted edit preserves the file's existing
  line endings, asserts each anchor is present exactly once and lands in
  executable code, and re-reads the region afterwards; a mixed-ending file or
  a silently unmatched anchor is a failed edit.

## Judgment toward the CCRL goal

- The aim is CCRL top 100, ideally top 50: a direction for measured strength,
  reliability and prioritisation, not a promised ranking or a reason to
  accumulate features. Progress is resolved uncertainty and qualified results.
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
- `NO_CHANGE`, refuted, too sparse, low expected value, inappropriate
  interaction and retry-trigger-not-fired are successful outcomes. Do not
  create work to make the roadmap move, and do not retry a rejected idea until
  its trigger fires.

## Expensive jobs and interruptions

- Long tournaments and SPRTs, large datagen, expensive tuning, large PGO
  campaigns and lengthy profiling belong to the maintainer unless explicitly
  delegated. Prepare and verify the command, inputs, configuration, artifacts
  and live wire, then hand over the runnable job. Cheap local qualification is
  the agent's job.
- When the user interrupts for a correction or scope change, finish and
  qualify that correction, report it and return control. Resume the earlier
  objective only when asked.

## Token-efficient execution

- Orient once per session, before selecting work: this file and each module
  the task triggers, then PLAN's register, whose rows are the order of work
  and whose first open row is the current leaf, and the selected leaf's
  section; `python tools/diag/plan_leaf.py` prints both with their lines.
  Follow up with `rg` and bounded
  excerpts; re-read only changed regions or to answer a concrete question.
- Batch independent reads and checks; send verbose output to logs and return
  exit status plus a short result. Run long jobs durably with logs and exit
  status, and back off unchanged polls: waiting on the CPU is not reasoning.
- Keep a compact working record: leaf, source and binary identity, evidence
  paths, completed checks, live process IDs, blocker, next action. After an
  interruption, inspect it and existing outputs before restarting anything.
- Before repeating any read, check or run, name what changed or what it
  answers; if nothing, skip it. Write documentation once results are ready,
  except registrations, new blockers and corrections to false claims.

## Sequencing and holds

- Work one executable leaf at a time: verify proportionately, update PLAN and
  GUIDE in one documentation commit, report. In a multi-step session continue
  to the next eligible leaf without asking; otherwise stop at the requested
  scope. Intermediate commits never mark an unfinished cluster accepted.
- Keep a held leaf's ID, reason, unblock condition and latest completion
  point visible; review holds at each handoff and resume the earliest
  eligible one. Never silently skip, move or tick missing verification.
- `check_guide.py` checks structure; its open-item list is not a scheduler and
  resolves no hold or dependency. Always state the next executable step and
  any held obligation that matters.
- `docs/HISTORY.md` is history and resolves every retired numbering scheme; never
  take a next step from it or from `docs/archive/`. When documents disagree,
  source, defaults and reproducible artifacts outrank prose; fix the prose in
  the same change.
- Report confounds immediately; correct contradicted current claims where they
  live and preserve historical measurements with explicit supersession.

## Handing work back

- When maintainer action is needed, give runnable commands in their own fenced
  block and restate them rather than referring back. Routine internal checks do
  not become the maintainer's chores.
- When reporting the next step, use its PLAN capability class and the mapping
  below to recommend one Claude model family (Fable, Opus or Sonnet)
  with its thinking mode (`Claude: <family> — <mode>`) and a brief
  task-specific reason; Claude models only, by maintainer decision
  2026-09-25, and the family without a version, by maintainer decision
  2026-11-13: versions move too fast to store. Prefer the least costly family judged sufficient and
  reserve deeper review for unresolved design, interaction or correctness
  questions; choose each mode independently at the lowest sufficient effort,
  and say so when a mode name is unknown rather
  than inventing one. A recommendation is a task judgment, not a guarantee, and
  changes no active setting.
- The mapping, which GUIDE's *Model mapping* shows the maintainer: `R3`
  Fable — High; `R2` Opus — High; `I2` Opus — High; `I1` Sonnet — Medium;
  `M` Sonnet — Medium; `V` Sonnet — High.
- Report what was actually measured, and say plainly when a step was skipped
  or a result is partial. For a multi-step session, summarise each completed
  leaf (ID, result, verification, commit) and name unfinished work separately.
- State the evidence layer. Loss, nodes, NPS, depth, conversion, tactics and
  reference agreement are not Elo and have no implicit exchange rate to it.

## Before reporting work as done

1. List the files changed, the commands run and the actions taken: commits,
   branches, tags, measurements, records, anything handed to the maintainer.
   `python tools/diag/check_rules.py --triggered` names the modules the
   changed paths require.
2. Hold that list against every `Kind`, `Paths` and `Before` line above.
3. Read in full each module that was triggered and not read, recheck the
   work against it, and repair what it finds.
4. Name the modules read in the report, so that a skipped one shows.

## Keeping these rules small

- A new rule goes into the module its trigger belongs to; only an invariant,
  a rule whose breach does damage on any task, enters this file.
- This file with everything a tool loads beside it stays within 18,000
  bytes, and each module within 18,000: three quarters of the smallest
  documented limit, 24,000 bytes a rule file (2026-11-25). Over a budget, a
  rule moves or is compressed first; raising one is a maintainer decision,
  recorded here with its reason.
- A tool stub such as `CLAUDE.md` imports this file and nothing else; a
  module is named by its path, never imported, or it loads on every task.
- A rule keeps at most a short citation of its incident, a maintainer
  decision its date and substance; the story goes to the ledger row or
  `docs/HISTORY.md`.
- `python tools/diag/check_rules.py` enforces the budgets, the index and the
  module headers; run it after any edit to a rule file.
