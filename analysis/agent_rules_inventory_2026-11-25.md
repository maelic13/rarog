# Agent rule inventory, 2026-11-25

Base: `ab6e0d3` is the commit whose `AGENTS.md` this inventory cuts: one file
of 29,399 bytes and 459 lines, read with the working tree's CRLF endings.

On 2026-11-25 the single rulebook was split into a core that every agent
tool loads whole and six modules that an agent reads in full when a trigger
fires. This record holds the limits and the budget the split was sized to,
the proof that no statement was lost, the maintainer's decisions of the same
day, and the test of the triggers on eight past tasks. It describes one
change; the rules move on without it.

## Load limits and the budget

| Tool | What it loads unasked | Documented limit | Source, read 2026-11-25 |
|---|---|---|---|
| Antigravity | `AGENTS.md` (always on), `GEMINI.md`, `.agents/rules/*.md` | 24,000 bytes a rule file, measured after its includes expand, the rest cut; 20,000 tokens across all always-on rules, past which the largest file becomes a pointer | `antigravity.google/docs/rules`, *Size limits and token budgets* |
| Codex | every `AGENTS.md` from the repository root down to the working directory | 32 KiB for those files together (`project_doc_max_bytes`); it stops adding at the limit | `developers.openai.com/codex/guides/agents-md` |
| Claude Code | `CLAUDE.md` with its `@` imports, here `AGENTS.md`; the auto-memory index `MEMORY.md` | No byte limit on `CLAUDE.md` or an import is documented; imports nest four deep; the advice is under 200 lines a file. `MEMORY.md`: its first 200 lines or 25 KB | `code.claude.com/docs/en/memory` |

- The reported cut at 24,000 bytes is Antigravity's documented limit. Byte
  24,000 of the old file is on line 375, inside *Gating*: the end of
  *Gating*, then *Documents*, *Sequencing and holds* and *Handing work back*
  were never loaded.
- The repository configures no other tool: `CLAUDE.md` is the only stub.
  The global rule files on the development machine add nothing to a set
  (Codex's is empty, Antigravity's absent).
- **The core budget is 18,000 bytes**: three quarters of the smallest limit.
  It counts the core and every file a tool loads with it, in UTF-8 bytes
  with CRLF endings, as a Windows working tree holds them. A module has the
  same budget, so that any one is read in a single read.
- Bytes bound tokens from above (a token is at least one byte), so the
  core is inside Antigravity's 20,000-token budget without a tokenizer; its
  size in tokens is unmeasured.

## Sizes

| Auto-loaded set | Before | After | Limit or budget |
|---|---|---|---|
| `AGENTS.md` alone (Codex, Antigravity) | 29,399 bytes, 459 lines | 14,994 bytes, 243 lines | budget 18,000; limits 24,000 and 32,768 |
| `CLAUDE.md` + `AGENTS.md` (Claude Code) | 29,653 bytes | 15,285 bytes | budget 18,000 |
| Claude Code's `MEMORY.md` index, outside the repository | 8,491 bytes, 52 lines | unchanged | 25 KB or 200 lines |

| Module | Bytes | Lines |
|---|---|---|
| `docs/rules/research.md` | 3,472 | 60 |
| `docs/rules/engine.md` | 3,370 | 61 |
| `docs/rules/measurement.md` | 4,486 | 73 |
| `docs/rules/gating.md` | 4,432 | 67 |
| `docs/rules/documents.md` | 3,473 | 57 |
| `docs/rules/evidence.md` | 3,660 | 58 |

## Orientation: PLAN, not GUIDE

Maintainer decision 2026-11-25: GUIDE is the maintainer's board, and an
agent does not orient from it. The old rulebook sent an agent to GUIDE for
four things. Each now has a home an agent reads without it:

| What an agent took from GUIDE | Where it reads it now |
|---|---|
| The current leaf (the board's *Now*) | The first open row of PLAN's *Active workflow register*, whose rows are the order of work. `python tools/diag/plan_leaf.py` prints the register and the lines of the leaf's text; it reads PLAN by the board generator's grammar, and a test holds its current leaf equal to the board's *Now* |
| Holds and the order of work | The same register: a held leaf says so in its row, and the decisions that set the order are in the register's opening text |
| The fingerprint | `docs/rules/engine.md`, *Verification* ("currently"), which `check_guide.py` holds equal to GUIDE's checkpoint |
| The mapping of classes to models | The core, *Handing work back*, which `check_guide.py` holds equal to GUIDE's *Model mapping* |

PLAN is 465 KB, so nothing in it is read whole. The default read is the
register, 7,203 bytes, which `check_rules.py` holds to 18,000, and the
selected leaf's text at the lines `plan_leaf.py <ID>` gives (D.9.3, the
current leaf on this day, is 951 bytes; its step D.9 10,327). PLAN's
operating rules and standing contracts are read by section when a rule
cites them; `plan_leaf.py` prints their lines too. Agents still write GUIDE:
they regenerate its board and keep its checkpoint and holds current
(`docs/rules/documents.md`).

## Method and totals

The old file is cut mechanically: each paragraph or top-level bullet is a
block, each sentence of a block a statement, with the ID
`<section>-<block><letter>`. That gives 210 statements. Each has a kind:

- **invariant**: binds on every task, so it is in the core;
- **task**: binds on certain work, so it is in the module that work triggers;
- **duplicate**: said by another statement, which is named;
- **history**: an incident or a review, not a rule.

`python tools/diag/rules_inventory.py --verify` cuts the base file again,
requires this table to carry exactly those 210 IDs with their own text, and
requires each statement to be found, whitespace aside, in the file named
for it: its old wording, or the wording under *Rewordings*. Run on the tree
of the commit that added this file, exit status 0:

| Kind | Where | Statements |
|---|---|---|
| invariant | `AGENTS.md` | 79 |
| task | `docs/rules/research.md` | 17 |
| task | `docs/rules/engine.md` | 19 |
| task | `docs/rules/measurement.md` | 27 |
| task | `docs/rules/gating.md` | 26 |
| task | `docs/rules/documents.md` | 19 |
| task | `docs/rules/evidence.md` | 17 |
| duplicate | merged into the statement named | 5 |
| history | `docs/HISTORY.md` | 1 |
| | | 210 |

Twenty statements were reworded and four incident notes moved to
`docs/HISTORY.md`; both lists are under *Rewordings*. Every other statement
arrived word for word.

The five merges, made on the maintainer's word that duplicates which make
sense to merge may be merged: `VER-6a` into `CHG-8b` (one rule for a
scripted edit), `HND-1c` into `SEQ-3b` (name the next step), `SEQ-2a` into
`TOK-1a` (read PLAN's register before selecting work), `CHG-9a` into
`GAT-8a` (a screen never accepts strength), `DOC-3c` into `JDG-6a` (the
owners of the record). Two more pairs each said one thing twice and now say
it once without either statement going: `TOK-4d` and `SEQ-4a` (report
confounds), `CHG-3a` and `EVI-3b` (an agent creates no tag).

## Old section to new home

Every reference to an old section by name, in PLAN, PROCESS, the tools'
README, the tools' comments and one test's header, was rewritten to the new
home in the same change. The ledger, HISTORY, `analysis/` and the archive
keep the old names: they record what was true when written, and this table
resolves them.

| Old section of `AGENTS.md` | Now |
|---|---|
| *Classify the work* | core, except the readiness boundary: `docs/rules/research.md`, *Readiness* |
| *Research and implementation* | `docs/rules/research.md` (research, donors, shared signals); `docs/rules/engine.md` (implementation, per-node code, neutral repairs) |
| *Judgment toward the CCRL goal* | core; the CCRL-configuration check and the four answers in `docs/rules/research.md`; the record's owners in `docs/rules/documents.md` |
| *Expensive jobs and interruptions* | core; information for a use in `docs/rules/measurement.md`, *What to measure* |
| *Predictions and negative results* | core (*Judgment*, *Handing work back*); frozen predictions in `docs/rules/research.md`, *Predictions* |
| *The one failure mode* | core |
| *Token-efficient execution* | core; the smallest sufficient measurement in `docs/rules/measurement.md` |
| *Measurement* | `docs/rules/measurement.md` |
| *Verification* | core; the Rust checks, the feature tests, lint suppression and the fingerprint in `docs/rules/engine.md`, *Verification*; the live wire in `docs/rules/measurement.md` |
| *Changes* | core; cited commits in `docs/rules/evidence.md`; the changelog and README in `docs/rules/documents.md`; counters and Elo in `docs/rules/gating.md`; comments in `docs/rules/engine.md`, *Comments* |
| *Evidence* | `docs/rules/evidence.md`; force-adding, untracking, `dragon_nnue` and remote deletion in the core, *Changes* |
| *Gating* | `docs/rules/gating.md` |
| *Documents* | `docs/rules/documents.md`; HISTORY as history and source over prose in the core, *Sequencing and holds* |
| *Sequencing and holds* | core |
| *Handing work back* | core; class escalation in `docs/rules/documents.md` |

## Statements

| ID | Lines | Kind | Destination | Statement, as the old rulebook had it |
|---|---|---|---|---|
| `INT-1a` | 3-9 | invariant | `AGENTS.md` | The rules an agent follows while working on Rarog. |
| `INT-1b` | 3-9 | invariant | `AGENTS.md` | `GUIDE.md` says what to work on; the relevant section of `docs/PLAN.md` says why; `docs/PROCESS.md` holds the procedures these rules assume. |
| `INT-1c` | 3-9 | invariant | `AGENTS.md` | Each rule is stated once, here. |
| `INT-1d` | 3-9 | invariant | `AGENTS.md` | The incidents that produced a rule are in the ledger rows it cites or in `docs/HISTORY.md`. |
| `INT-1e` | 3-9 | invariant | `AGENTS.md` | Read this file in full from disk before work: some agent tools load rule files only up to a size limit, and the last sections bind as much as the first. |
| `CLS-1a` | 13-17 | invariant | `AGENTS.md` | Before substantial work, name its primary kind: research/diagnosis, experiment design, implementation, deterministic qualification, performance qualification, playing-strength gate, or documentation/provenance. |
| `CLS-1b` | 13-17 | invariant | `AGENTS.md` | A roadmap leaf is not automatically an instruction to write code. |
| `CLS-1c` | 13-17 | invariant | `AGENTS.md` | PLAN owns each leaf's workflow state and capability class; GUIDE maps classes to models. |
| `CLS-2a` | 18-22 | invariant | `AGENTS.md` | The workflow is `RESEARCH -> READY_FOR_IMPLEMENTATION -> IMPLEMENTED -> LOCAL_QUALIFIED -> GAME_GATE -> CLOSED`. |
| `CLS-2b` | 18-22 | invariant | `AGENTS.md` | Not every task needs every state: documentation can close without games, research can close `NO_CHANGE`, neutral performance work closes on deterministic and performance qualification. |
| `CLS-2c` | 18-22 | invariant | `AGENTS.md` | A playing-strength change does not bypass `GAME_GATE`. |
| `CLS-3a` | 23-28 | task | `docs/rules/research.md` | `READY_FOR_IMPLEMENTATION` is a hard boundary. |
| `CLS-3b` | 23-28 | task | `docs/rules/research.md` | A substantial playing change crosses it only with: the measured defect or opportunity and its evidence in this engine, credible competing explanations, interacting mechanisms, the cheapest test that can kill the hypothesis, its falsifier and stop rule, and the condition that justifies implementation. |
| `CLS-3c` | 23-28 | task | `docs/rules/research.md` | A plausible idea or a donor feature is not enough. |
| `RES-1a` | 32-35 | task | `docs/rules/research.md` | Research owns the causal question, competing hypotheses, interaction map, prospective prediction, falsifiers, the meaning of each experiment and the readiness decision. |
| `RES-1b` | 32-35 | task | `docs/rules/research.md` | Prefer cheap discriminating evidence to a sophisticated implementation of an uncertain idea. |
| `RES-2a` | 36-38 | task | `docs/rules/engine.md` | Implementation owns ordinary engineering: idiomatic Rust structure, necessary local refactoring, focused instrumentation and tests, compilation, debugging and cheap deterministic qualification. |
| `RES-2b` | 36-38 | task | `docs/rules/engine.md` | The maintainer need not prescribe it. |
| `RES-3a` | 39-43 | task | `docs/rules/engine.md` | Implementation does not replace the hypothesis, broaden the mechanism, add adjacent heuristics, tune unrelated constants, port extra donor behaviour, change the experiment after exposure, or rescue a weak candidate by changing its neighbours. |
| `RES-3b` | 39-43 | task | `docs/rules/engine.md` | If a material premise is false, keep useful instrumentation, record the contradiction and return the leaf to `RESEARCH`. |
| `RES-4a` | 44-51 | task | `docs/rules/engine.md` | Per-node code (negamax, qsearch, move picking, movegen, make/unmake, eval, SEE, TT probes) does not allocate, clone a `Board`, take a lock, touch an `Arc`, or pass or return a large value by copy (check the assembly for `memcpy`). |
| `RES-4b` | 44-51 | task | `docs/rules/engine.md` | Ownership is solved with `&mut` borrows, make/unmake and per-thread arrays, never by a clone that satisfies the borrow checker. |
| `RES-4c` | 44-51 | task | `docs/rules/engine.md` | An exception states its reason in one sentence and carries an NPS measurement by PROCESS's method. |
| `RES-4d` | 44-51 | task | `docs/rules/engine.md` | A shared atomic states what it signals and which search it belongs to. |
| `RES-4e` | 44-51 | task | `docs/rules/engine.md` | B.7.1's allocation test checks the first clause. |
| `RES-5a` | 52-57 | task | `docs/rules/research.md` | Donor engines teach mechanisms, contracts, dependencies, failure modes and methods. |
| `RES-5b` | 52-57 | task | `docs/rules/research.md` | What may cross is `docs/PROCESS.md`, *The independence boundary*. |
| `RES-5c` | 52-57 | task | `docs/rules/research.md` | Neither similarity nor a copied value is acceptance evidence. |
| `RES-5d` | 52-57 | task | `docs/rules/research.md` | Stockfish is the primary donor; Reckless, an AGPL engine, and any other AGPL or unlicensed engine teach ideas only: no code, constant, table or patch of their source enters the tree (maintainer decision 2026-11-23). |
| `RES-6a` | 58-63 | task | `docs/rules/research.md` | For nontrivial playing work, check shared signals and feedback: search changes move evaluation populations, evaluation changes move pruning, ordering evidence may also prune, TT semantics can mask a candidate, and rule-50, repetition and promotion closure can make a local feature non-local. |
| `RES-6b` | 58-63 | task | `docs/rules/research.md` | Use a bounded baseline/A/B/A+B screen when it cheaply separates interaction; do not require a factorial for every small change. |
| `RES-7a` | 64-65 | task | `docs/rules/engine.md` | Keep default-neutral repairs in a separate sub-step and commit cluster from behaviour-changing ones so each read isolates one cause. |
| `JDG-1a` | 69-71 | invariant | `AGENTS.md` | The aim is CCRL top 100, ideally top 50: a direction for measured strength, reliability and prioritisation, not a promised ranking or a reason to accumulate features. |
| `JDG-1b` | 69-71 | invariant | `AGENTS.md` | Progress is resolved uncertainty and qualified results. |
| `JDG-2a` | 72-75 | task | `docs/rules/research.md` | When evaluating a defect or repair, check whether it is active in the CCRL rated configuration (such as tablebases, clocks, or threads) versus Rarog's own pools and gates, because a feature absent from local pools can still matter to CCRL and vice versa. |
| `JDG-3a` | 76-84 | task | `docs/rules/research.md` | Before a substantial engine change, answer in the PLAN research card or the experiment registration (link earlier answers): **mechanism** — what should improve and what evidence says it is active here; **interactions** — which producers, consumers and shared signals, and where it could duplicate, cancel or weaken an existing mechanism; **invariants** — which node, TT, score, history, board and protocol contracts must hold and how they are tested; **falsifier** — the cheapest refuting observation and the rule that stops further investment. |
| `JDG-3b` | 76-84 | task | `docs/rules/research.md` | Routine mechanical fixes need only the contract and a focused check. |
| `JDG-4a` | 85-91 | invariant | `AGENTS.md` | Disagree plainly when the evidence warrants, including with the maintainer or an earlier conclusion: say "I recommend against implementing this now", give the evidence, distinguish refutation from insufficient evidence, and name the cheapest alternative or the condition that would change the recommendation. |
| `JDG-4b` | 85-91 | invariant | `AGENTS.md` | A PLAN checkbox or a strong donor is not evidence. |
| `JDG-4c` | 85-91 | invariant | `AGENTS.md` | Do not manufacture objections; a supported positive result gets an equally clear recommendation. |
| `JDG-5a` | 92-94 | invariant | `AGENTS.md` | When a change would violate a known correctness contract or a registered experimental rule, stop that change, explain the conflict and offer a valid path. |
| `JDG-5b` | 92-94 | invariant | `AGENTS.md` | Ordinary engineering tradeoffs do not open a permission loop. |
| `JDG-6a` | 95-99 | task | `docs/rules/documents.md` | Keep the design record in its owners: PLAN for contracts, decisions and dependencies; source and tests for executable invariants; EXPERIMENTS and linked analyses for predictions, failures and retry triggers; PROCESS for repeatable methods. |
| `JDG-6b` | 95-99 | task | `docs/rules/documents.md` | Extend a missing contract there, not in a parallel summary. |
| `EXP-1a` | 103-107 | invariant | `AGENTS.md` | Long tournaments and SPRTs, large datagen, expensive tuning, large PGO campaigns and lengthy profiling belong to the maintainer unless explicitly delegated. |
| `EXP-1b` | 103-107 | invariant | `AGENTS.md` | Prepare and verify the command, inputs, configuration, artifacts and live wire, then hand over the runnable job. |
| `EXP-1c` | 103-107 | invariant | `AGENTS.md` | Cheap local qualification is the agent's job. |
| `EXP-2a` | 108-114 | task | `docs/rules/measurement.md` | Get the information a use needs and no more. |
| `EXP-2b` | 108-114 | task | `docs/rules/measurement.md` | Before registering or running a measurement, name what it is for: a decision, an analysis or understanding a question. |
| `EXP-2c` | 108-114 | task | `docs/rules/measurement.md` | A read that feeds nothing is prepared or left out, not run, and runs when a use is named; a decision is never taken without the information it needs either (maintainer decision 2026-11-14, after RAR-O09's four matches were cut to the one PLAN's rule reads). |
| `EXP-3a` | 115-117 | invariant | `AGENTS.md` | When the user interrupts for a correction or scope change, finish and qualify that correction, report it and return control. |
| `EXP-3b` | 115-117 | invariant | `AGENTS.md` | Resume the earlier objective only when asked. |
| `PRD-1a` | 121-124 | task | `docs/rules/research.md` | Freeze predictions before exposure. |
| `PRD-1b` | 121-124 | task | `docs/rules/research.md` | Afterwards append a calibration: which assumption failed, and whether the miss was in sign, magnitude, mechanism, interaction, confidence or instrument. |
| `PRD-1c` | 121-124 | task | `docs/rules/research.md` | Never rewrite a prediction into retrospective certainty. |
| `PRD-2a` | 125-128 | invariant | `AGENTS.md` | `NO_CHANGE`, refuted, too sparse, low expected value, inappropriate interaction and retry-trigger-not-fired are successful outcomes. |
| `PRD-2b` | 125-128 | invariant | `AGENTS.md` | Do not create work to make the roadmap move, and do not retry a rejected idea until its trigger fires. |
| `PRD-3a` | 129-130 | invariant | `AGENTS.md` | State the evidence layer. |
| `PRD-3b` | 129-130 | invariant | `AGENTS.md` | Loss, nodes, NPS, depth, conversion, tactics and reference agreement are not Elo and have no implicit exchange rate to it. |
| `FM-1a` | 134-137 | invariant | `AGENTS.md` | Almost every agent mistake here has been a check that did not check what it was thought to check: a stale binary measured, one record parsed instead of forty, an exit code read from the wrong end of a pipe. |
| `FM-1b` | 134-137 | invariant | `AGENTS.md` | **Verify mechanically, never by eyeballing, and never by assuming a tool did what its name says.** |
| `TOK-1a` | 141-143 | invariant | `AGENTS.md` | Orient once per session: operating rules, GUIDE's current and held overview, the selected PLAN section. |
| `TOK-1b` | 141-143 | invariant | `AGENTS.md` | Follow up with `rg` and bounded excerpts; re-read only changed regions or to answer a concrete question. |
| `TOK-2a` | 144-146 | invariant | `AGENTS.md` | Batch independent reads and checks; send verbose output to logs and return exit status plus a short result. |
| `TOK-2b` | 144-146 | invariant | `AGENTS.md` | Run long jobs durably with logs and exit status, and back off unchanged polls: waiting on the CPU is not reasoning. |
| `TOK-3a` | 147-149 | invariant | `AGENTS.md` | Keep a compact working record: leaf, source and binary identity, evidence paths, completed checks, live process IDs, blocker, next action. |
| `TOK-3b` | 147-149 | invariant | `AGENTS.md` | After an interruption, inspect it and existing outputs before restarting anything. |
| `TOK-4a` | 150-155 | task | `docs/rules/measurement.md` | Define the smallest sufficient measurement before launching it, inside the leaf's registered scope; never shrink a registered run after seeing results. |
| `TOK-4b` | 150-155 | task | `docs/rules/measurement.md` | Reuse the harness's runner, parser and archive format. |
| `TOK-4c` | 150-155 | invariant | `AGENTS.md` | Before repeating any read, check or run, name what changed or what it answers; if nothing, skip it. |
| `TOK-4d` | 150-155 | invariant | `AGENTS.md` | Report confounds immediately; write documentation once results are ready, except registrations, new blockers and corrections to false claims. |
| `MEA-1a` | 159-167 | task | `docs/rules/measurement.md` | Colosseum CLI is the main harness for gates, fixed matches, tunes, null pairs, gauntlets and datagen; `tools/colosseum.ps1` and `tools/datagen.ps1` drive it and carry every guard. fastchess and weather-factory (`tools/sprt.ps1`, `tools/spsa.ps1`, `tools/gauntlet.ps1`, `datagen.ps1 -Runner fastchess`) stay installed and working as backup only, the second opinion. |
| `MEA-1b` | 159-167 | history | `docs/HISTORY.md` | They were reviewed at 2.5.0 on 2026-10-04 and kept, and kept again as backup only on 2026-10-14 when datagen moved to Colosseum. |
| `MEA-1c` | 159-167 | task | `docs/rules/measurement.md` | They retire only on a maintainer decision recorded here. |
| `MEA-1d` | 159-167 | task | `docs/rules/measurement.md` | PROCESS's *Harness* section names the cross-check triggers. |
| `MEA-2a` | 168-170 | task | `docs/rules/measurement.md` | Never measure a `--all-features` binary: it enables `texel`, which bypasses the eval and pawn caches. |
| `MEA-2b` | 168-170 | task | `docs/rules/measurement.md` | If a number you are not changing changes, check the binary. |
| `MEA-3a` | 171-176 | task | `docs/rules/measurement.md` | Measure only on an idle host: check CPU use and running engine or harness processes first, and if the machine is busy stop and ask rather than measure (RAR-M48's first pool was discarded for this). |
| `MEA-3b` | 171-176 | task | `docs/rules/measurement.md` | Keep builds, profiling and unrelated load off a match host while it plays. |
| `MEA-3c` | 171-176 | task | `docs/rules/measurement.md` | `colosseum.ps1` refuses a busy host; a waiver is recorded in the run's manifest, so it cannot be forgotten. |
| `MEA-4a` | 177-181 | task | `docs/rules/measurement.md` | Rebuild before measuring, with the exact feature set: `cargo test`, `clippy` and `bench` leave their own `target/release/rarog.exe`. |
| `MEA-4b` | 177-181 | task | `docs/rules/measurement.md` | For a multi-run study, build once, verify the fingerprint, hash and archive that executable, and measure the copy; rebuild only when source, features, toolchain or build settings change. |
| `MEA-4c` | 177-181 | task | `docs/rules/measurement.md` | Keep test builds separate. |
| `MEA-5a` | 182-187 | task | `docs/rules/measurement.md` | A `diag` build's `bench` dumps counters once per position (40 lines per name for `bench 13`); sum them with `tools/diag/bench_counters.py`, never a hand-rolled parser. |
| `MEA-5b` | 182-187 | task | `docs/rules/measurement.md` | The search's rule counts are the kernel's rate probes (`--features probes`), read by `tools/oracle-hybrids/tree_stats.py`; `tools/diag/phase4_differential.py` reads node counters that only builds before the compact kernel emit. |
| `MEA-6a` | 188-190 | task | `docs/rules/measurement.md` | Before differencing two counters, confirm they are in the same unit (per node versus per move produced RAR-S25). |
| `MEA-6b` | 188-190 | task | `docs/rules/measurement.md` | A passing invariant does not prove comparability. |
| `MEA-7a` | 191-195 | task | `docs/rules/measurement.md` | A counter from another engine or another tree (nodes, hits per thousand nodes, rule rates) is at most a prior for a prediction, never evidence about Rarog: the evidence is Rarog's own before and after. |
| `MEA-7b` | 191-195 | task | `docs/rules/measurement.md` | A timing or size that was not measured is marked unmeasured, never stated as a number. |
| `MEA-8a` | 196-202 | task | `docs/rules/measurement.md` | A binary entered in a rated pool is a tagged release or a development build of a committed tree, whose `id name` names its commit (`dev-20261104-209b555a`; a dirty tree's `-m` is refused for testing); a feature arm is named in its file and sidecar, and its ledger row names the fingerprint. |
| `MEA-8b` | 196-202 | task | `docs/rules/measurement.md` | The harness binary is pinned the same way: `tools/colosseum/colosseum.pin.json` names its revision and SHA-256, and a runner that is not the pinned one is refused, not substituted. |
| `MEA-9a` | 203-205 | task | `docs/rules/measurement.md` | The current fingerprint is declared once, in GUIDE's checkpoint; `check_guide.py` fails when AGENTS' "currently" values or PLAN's checkpoint row disagree with it. |
| `VER-1a` | 209-215 | invariant | `AGENTS.md` | Scope checks to the change. |
| `VER-1b` | 209-215 | task | `docs/rules/engine.md` | Rust engine, test, build or dependency changes need debug and release tests, `cargo fmt --check` and `cargo clippy --all-features --all-targets` at zero warnings; release alone is not enough. |
| `VER-1c` | 209-215 | invariant | `AGENTS.md` | Documentation-only changes need diff, link and status checks, and `check_guide.py` when GUIDE or PLAN structure changes, and no Cargo builds, engine tests or bench. |
| `VER-1d` | 209-215 | invariant | `AGENTS.md` | Tooling changes need their tests and a meaningful smoke or negative check of the changed path, not a rebuilt engine. |
| `VER-2a` | 216-219 | task | `docs/rules/engine.md` | An evaluation, trace or parameter change also runs `cargo test --release --features texel --lib` and `cargo test --release --features tune --lib`: the trace reconstruction gate and the parameter tests compile only there. |
| `VER-3a` | 220-222 | invariant | `AGENTS.md` | Run each required check once on the final relevant state, record command, exit status and coverage, and reuse the pass while its inputs are unchanged. |
| `VER-3b` | 220-222 | invariant | `AGENTS.md` | Rerun after edits, failures or evidence of a gap; never call an old pass new. |
| `VER-4a` | 223-225 | task | `docs/rules/engine.md` | Suppress lints with `#[expect(...)]`, which warns when no longer needed; use `#[allow(...)]` only for a lint that fires in one feature configuration and not another, with its reason written. |
| `VER-5a` | 226-226 | invariant | `AGENTS.md` | Check exit status directly: `cmd > out 2>&1; echo $?`, never through a pipe. |
| `VER-6a` | 227-228 | duplicate | `CHG-8b` | Every scripted edit asserts its anchor matched, is unique and lands in executable code; re-read the region afterwards. |
| `VER-7a` | 229-231 | task | `docs/rules/measurement.md` | Prove a harness wire is live before trusting a null from it: set an absurd value and require the numbers to move. |
| `VER-7b` | 229-231 | task | `docs/rules/measurement.md` | Proving the engine responds is not proving the instrument reports it. |
| `VER-8a` | 232-239 | task | `docs/rules/engine.md` | A behaviour-neutral engine change reproduces the immediate development fingerprint (currently **3,694,298 / EBF 2.324**) plus targeted checks for behaviour the suite does not reach: an identical bench does not prove a narrow feature neutral (RAR-E10). |
| `VER-8b` | 232-239 | task | `docs/rules/engine.md` | Investigate a cross-platform mismatch (RAR-P14, RAR-P16). |
| `VER-8c` | 232-239 | invariant | `AGENTS.md` | Docs-only work verifies the diff has no engine inputs. |
| `VER-8d` | 232-239 | task | `docs/rules/engine.md` | The comparison fingerprint is revision-specific: a deliberately integrated behaviour change updates the fingerprint record; never preserve a known defect to keep an obsolete count. |
| `VER-9a` | 240-240 | invariant | `AGENTS.md` | Test constructs and behaviour, not words in a comment. |
| `CHG-1a` | 244-247 | invariant | `AGENTS.md` | Engine changes and tooling or documentation changes go in separate commits. |
| `CHG-1b` | 244-247 | invariant | `AGENTS.md` | Commit after each finished and verified step. |
| `CHG-1c` | 244-247 | invariant | `AGENTS.md` | Never relax a correctness test in the commit whose change made it fail; fix its precondition in its own commit, with the justifying measurement. |
| `CHG-2a` | 248-252 | invariant | `AGENTS.md` | No attribution to Claude or any other AI tool, anywhere: no `Co-Authored-By` trailer, no "Generated with" line, in commits, PR titles and descriptions, squash messages or files. |
| `CHG-2b` | 248-252 | invariant | `AGENTS.md` | The maintainer adds attribution where they want it (maintainer decision 2026-10-09; `.claude/settings.json` turns Claude Code's own attribution off). |
| `CHG-3a` | 253-255 | invariant | `AGENTS.md` | Never push, create a release tag, publish or merge to `master`; the maintainer does, on instruction. |
| `CHG-3b` | 253-255 | invariant | `AGENTS.md` | Do not amend or rewrite a commit that has left this machine. |
| `CHG-3c` | 253-255 | invariant | `AGENTS.md` | Archive tags follow *Evidence*. |
| `CHG-4a` | 256-263 | invariant | `AGENTS.md` | `master` is the trunk. |
| `CHG-4b` | 256-263 | invariant | `AGENTS.md` | Each coherent piece of work (a PLAN unit such as C.3, or one between-units change: tooling, documentation, a research close) lives on its own branch and reaches `master` as one squash commit through a PR the maintainer merges; `master` may run ahead of the latest release, which is a tag on it (maintainer decision 2026-10-09, replacing the merge commits of 2026-10-05; the procedure is PROCESS, *Branches and pull requests* and *Release*). |
| `CHG-4c` | 256-263 | invariant | `AGENTS.md` | The small, separate commits above stay on the branch. |
| `CHG-5a` | 264-279 | task | `docs/rules/evidence.md` | A cited commit stays reachable: on `master`, under a PR (GitHub keeps every PR's commits as `refs/pull/<n>/head`), on a kept tag, or on the local branch of work in flight. |
| `CHG-5b` | 264-279 | task | `docs/rules/evidence.md` | The landed state is cited by its squash commit on `master`; a branch's own commits stay citable for the step each made. |
| `CHG-5c` | 264-279 | task | `docs/rules/evidence.md` | A gate arm is cited by its recipe file once its branch is deleted. |
| `CHG-5d` | 264-279 | task | `docs/rules/evidence.md` | `python tools/diag/check_citations.py` fails on any other commit, and on hex that is neither a commit here nor listed in `tools/diag/citation_foreign.tsv`; it needs the PR refs fetched once per clone (`git fetch origin "+refs/pull/*/head:refs/remotes/origin/pr/*"`) (maintainer decisions 2026-10-09). |
| `CHG-5e` | 264-279 | task | `docs/rules/evidence.md` | A one-off fetch does not keep them: `refs/remotes/origin/pr/*` lies under the branch refspec's destination, so the next `git fetch --prune` deletes them as branches the remote lacks. |
| `CHG-5f` | 264-279 | task | `docs/rules/evidence.md` | Add the refspec to the remote once per clone instead, `git config --add remote.origin.fetch "+refs/pull/*/head:refs/remotes/origin/pr/*"`, and every fetch, pruning or not, keeps them (maintainer decision 2026-10-11; on 2026-10-11 this clone held none and the check failed). |
| `CHG-6a` | 280-285 | task | `docs/rules/documents.md` | `CHANGELOG.md`'s `[Unreleased]` grows as the work lands: a change a user of the engine would notice (a gate-accepted strength change; a UCI option, output or default; a fixed defect; a removal; a build or asset change) gets its entry, written for users, in the commit that records it. |
| `CHG-6b` | 280-285 | task | `docs/rules/documents.md` | Experiments, refutations and internal refactors get none. |
| `CHG-6c` | 280-285 | task | `docs/rules/documents.md` | A release dates the section and never reconstructs it. |
| `CHG-7a` | 286-292 | task | `docs/rules/documents.md` | The README, the changelog and the release notes cut from it are for users, not for the record: short, simple and to the point. |
| `CHG-7b` | 286-292 | task | `docs/rules/documents.md` | An entry says what changed for the user in a sentence or two, not how it was done or measured: no game counts, error bars, node counts, parameter counts or internal names. |
| `CHG-7c` | 286-292 | task | `docs/rules/documents.md` | A number appears only for a strength gain (rounded, once, in the release's opening line) or when the user acts on it, such as an option's range. |
| `CHG-7d` | 286-292 | task | `docs/rules/documents.md` | The evidence lives in EXPERIMENTS and the ledger, never here. |
| `CHG-8a` | 293-296 | invariant | `AGENTS.md` | Most of the tree is CRLF. |
| `CHG-8b` | 293-296 | invariant | `AGENTS.md` | A scripted edit preserves the file's existing line endings, asserts each anchor is present exactly once, and re-reads the region afterwards; a mixed-ending file or a silently unmatched anchor is a failed edit. |
| `CHG-9a` | 297-298 | duplicate | `GAT-8a` | Counters explain a candidate; only a registered SPRT accepts one. |
| `CHG-9b` | 297-298 | task | `docs/rules/gating.md` | Node counts are not Elo: a +7.36% tree change measured −1.49 ± 2.87 Elo. |
| `CHG-10a` | 299-302 | task | `docs/rules/engine.md` | Comments explain the problem or the invariant, briefly: no roadmap step numbers, no ledger IDs as the explanation, no narration of earlier versions. |
| `CHG-10b` | 299-302 | task | `docs/rules/engine.md` | A measured reason to keep a shape stays, in one sentence. |
| `CHG-10c` | 299-302 | task | `docs/rules/engine.md` | Rewrite a mechanism's comments when you rewrite the mechanism. |
| `EVI-1a` | 306-314 | task | `docs/rules/evidence.md` | Development and raw evidence live on this machine; macOS and Windows on ARM are compatibility-test hosts. |
| `EVI-1b` | 306-314 | task | `docs/rules/evidence.md` | Git holds source, build and CI files, reusable tools, required fixtures and concise records; raw runs, logs, executables, traces and bundles stay in ignored `analysis/artifacts/` or `tools/results/` with their paths, recipes and hashes recorded (`analysis/README.md`). |
| `EVI-1c` | 306-314 | task | `docs/rules/evidence.md` | Small frozen datasets that tests or tools consume stay versioned. |
| `EVI-1d` | 306-314 | task | `docs/rules/evidence.md` | Compatibility checks never depend on this machine's private outputs. |
| `EVI-1e` | 306-314 | invariant | `AGENTS.md` | Never force-add evidence. |
| `EVI-1f` | 306-314 | invariant | `AGENTS.md` | Untracking uses `git rm --cached`, keeps the local bytes, and authorises neither deleting evidence nor rewriting history. |
| `EVI-2a` | 315-321 | task | `docs/rules/evidence.md` | Information lives in the tracked documents, not in refs. |
| `EVI-2b` | 315-321 | task | `docs/rules/evidence.md` | A ledger row, PLAN record or analysis states what changed, why, the result and the decision, and carries the exact recipe (values inline, or a small patch file in Git) plus a fingerprint proving a rebuild matched. |
| `EVI-2c` | 315-321 | task | `docs/rules/evidence.md` | A hash, branch or tag is at most a pointer beside that information, never its only carrier (RAR-S54; on 2026-09-27, 71 cited commits were on no ref, `analysis/ledger_commits_2026-09-27.md`). |
| `EVI-3a` | 322-329 | task | `docs/rules/evidence.md` | A commit is preserved by its recipe in Git, never by a kept branch or a new tag: a few lines of values in the citing record, or a small patch file in `analysis/arm_patches/` proved to reproduce the commit's tree from a base on a kept ref; the citing document then names the recipe, not the hash (maintainer decision 2026-11-04, replacing the annotated tags of 2026-10-09). |
| `EVI-3b` | 322-329 | task | `docs/rules/evidence.md` | The tags kept are the releases (`v*`) and the oracle's three (`oracle/*`, a vendored Stockfish tree too large for a patch), which retire by the conditions their records state; an agent creates no tag. |
| `EVI-4a` | 330-331 | invariant | `AGENTS.md` | `dragon_nnue` is kept, locally and on GitHub: never delete it, whatever a review finds (maintainer decision 2026-11-18). |
| `EVI-5a` | 332-337 | task | `docs/rules/evidence.md` | No mess: a throwaway work branch is deleted once its information is recorded, and at each phase close tags and branches are reviewed and any whose retirement condition fired are removed. |
| `EVI-5b` | 332-337 | task | `docs/rules/evidence.md` | Before removing one, check what still cites it and whether anything else holds its commits (`git for-each-ref --contains <sha>`; empty means it would dangle). |
| `EVI-5c` | 332-337 | invariant | `AGENTS.md` | Deleting a remote tag or branch is the maintainer's command. |
| `GAT-1a` | 341-345 | task | `docs/rules/gating.md` | The strength unit is one dependency-complete, locally fitted cluster; internal sub-steps get no gates of their own. |
| `GAT-1b` | 341-345 | task | `docs/rules/gating.md` | Register it before any games as a ledger entry, `docs/experiments/<ID>.md` with its index row in `docs/EXPERIMENTS.md` (hypothesis, baseline SHA, gate, cap, stop rule), and never change bounds, cap, book or adjudication after seeing games. |
| `GAT-2a` | 346-348 | task | `docs/rules/gating.md` | `[0,3]` nElo is the default bracket. |
| `GAT-2b` | 346-348 | task | `docs/rules/gating.md` | Widen only for a genuinely large prior and say why; a wide bracket resolves a large effect fast (RAR-S57, `[3,10]`, 2,838 games). |
| `GAT-2c` | 346-348 | task | `docs/rules/gating.md` | Compute the games at the expected value from RAR-M10 first. |
| `GAT-3a` | 349-367 | task | `docs/rules/gating.md` | A removal or simplification uses a bracket that permits a small loss (`[-1.75, 0.25]`). |
| `GAT-3b` | 349-367 | task | `docs/rules/gating.md` | A repair is gated by what it can change: |
| `GAT-3c` | 349-367 | task | `docs/rules/gating.md` | One that reproduces the fingerprint in the rated configuration needs no strength gate; it gets an activation and correctness read where it is active. |
| `GAT-3d` | 349-367 | task | `docs/rules/gating.md` | So does one that moves the fingerprint only through positions a static differential confines to the repair's domain, with that differential as its correctness read (maintainer decision 2026-10-15). |
| `GAT-3e` | 349-367 | task | `docs/rules/gating.md` | One expected near zero, where a small loss is acceptable for correctness, takes a pre-registered 2,000-game fixed read with a harm rule; the registration states the threshold and its power. |
| `GAT-3f` | 349-367 | task | `docs/rules/gating.md` | Scaled from RAR-S94's interval (±17.3 nElo at 1,552 games) the standard error is about 7.8 nElo; a 5% false alarm at true zero puts the threshold near −13 nElo, which catches a true −5 with probability 0.16, −10 with 0.36, −20 with 0.82 and −25 with 0.94. |
| `GAT-3g` | 349-367 | task | `docs/rules/gating.md` | "Kept" therefore means no harm beyond roughly −20 nElo. |
| `GAT-3h` | 349-367 | task | `docs/rules/gating.md` | One that changes shared search or evaluation signals in the rated configuration, or could be large either way, keeps the symmetric `[-5,5]` with its cap (RAR-S62; about 12,000 games), which excludes harm near −5. |
| `GAT-4a` | 368-371 | task | `docs/rules/gating.md` | A repair's registration states its measured exposure: the share of recent gate games that reach the repair's domain, from Colosseum's database, so its cap and harm rule are read against how often it can matter (maintainer decision 2026-11-02). |
| `GAT-5a` | 372-377 | task | `docs/rules/gating.md` | A correct repair that fails its harm rule leaves `master` by the gate, but the revert never closes its defect: the defect stays open in its owner leaf, and a rework is registered as a new experiment, the repair with the named constants that were tuned around the defect refitted. |
| `GAT-5b` | 372-377 | task | `docs/rules/gating.md` | That is a new registration, not a rescue inside the failed one (maintainer decision 2026-11-02). |
| `GAT-6a` | 378-380 | task | `docs/rules/gating.md` | Both harnesses run the pentanomial GSPRT (`model=normalized`); the gap to fishtest is bounds and budget, not the test. |
| `GAT-6b` | 378-380 | task | `docs/rules/gating.md` | A registered experiment names its runner and never changes it mid-way. |
| `GAT-7a` | 381-383 | task | `docs/rules/gating.md` | High bounds reject small gains: a true +4 nElo reaches H0 under `[0,10]` in about 35k games and is accepted by `[0,3]` in about 47k (RAR-M10). |
| `GAT-7b` | 381-383 | task | `docs/rules/gating.md` | That is overnight compute: budget the games, do not widen the bounds or poll. |
| `GAT-8a` | 384-386 | task | `docs/rules/gating.md` | Bench and counter screens choose candidates and never accept strength (RAR-S64). |
| `GAT-8b` | 384-386 | task | `docs/rules/gating.md` | Do not invent an acceptance rule after seeing a result; register a narrower bracket prospectively if small gains must be bankable. |
| `GAT-9a` | 387-388 | task | `docs/rules/gating.md` | An unresolved stop is not "probably fine": a high LOS on a point estimate is not evidence the mechanism works (RAR-S61, RAR-S64). |
| `GAT-10a` | 389-393 | task | `docs/rules/gating.md` | SPSA is conditional (PLAN rule 4): first show activation, interaction and curvature with a zero-game sweep; a flat or monotone surface is evidence against the tune. |
| `GAT-10b` | 389-393 | task | `docs/rules/gating.md` | A tune is one registered run read at checkpoints by its gradient test and stopped when quiet (PLAN rule 7c), never on a horizon chosen to fit the answer. |
| `DOC-1a` | 397-405 | task | `docs/rules/documents.md` | GUIDE's board is generated from `docs/PLAN.md` by `python tools/diag/guide_board.py`; never edit it by hand. |
| `DOC-1b` | 397-405 | task | `docs/rules/documents.md` | A PLAN step head reads ``- **<ID> <title> — `<class>`….**``: the title is what the board shows, the first backticked class is the step's class, and `DONE <date>` (or CLOSED, NO_CHANGE) in the field ticks it. |
| `DOC-1c` | 397-405 | task | `docs/rules/documents.md` | Mark a step DONE only when finished and verified, in the commit that finishes it, and a parent when its last sub-step is; then regenerate. |
| `DOC-1d` | 397-405 | task | `docs/rules/documents.md` | Run `python tools/diag/check_guide.py`, which fails on a stale board, rather than reading the file. |
| `DOC-2a` | 406-409 | task | `docs/rules/documents.md` | PLAN, the regenerated board and GUIDE's hand-kept checkpoint and holds change in the same commit when roadmap status or requirements change; an AGENTS-only edit needs no PLAN or GUIDE churn. |
| `DOC-2b` | 406-409 | task | `docs/rules/documents.md` | Sub-steps nest one level (`C.5.1`) and addenda two (`B.2.0.1`); nothing goes deeper. |
| `DOC-3a` | 410-416 | task | `docs/rules/documents.md` | Keep GUIDE short and its board first: every phase, then the checkpoint, holds, model mapping, two prompts and operator contract. |
| `DOC-3b` | 410-416 | task | `docs/rules/documents.md` | A finished phase's PLAN heading is marked `— CLOSED <date>` over the one- or two-sentence summary the board shows, and its text moves verbatim to `docs/archive/` (maintainer decision 2026-10-05). |
| `DOC-3c` | 410-416 | duplicate | `JDG-6a` | What a step involves goes in PLAN, a completed record in HISTORY, a procedure in PROCESS, evidence in EXPERIMENTS, a derivation in `analysis/`. |
| `DOC-4a` | 417-420 | invariant | `AGENTS.md` | `docs/HISTORY.md` is history and resolves every retired numbering scheme; never take a next step from it or from `docs/archive/`. |
| `DOC-4b` | 417-420 | invariant | `AGENTS.md` | When documents disagree, source, defaults and reproducible artifacts outrank prose; fix the prose in the same change. |
| `SEQ-1a` | 424-427 | invariant | `AGENTS.md` | Work one executable leaf at a time: verify proportionately, update PLAN and GUIDE in one documentation commit, report. |
| `SEQ-1b` | 424-427 | invariant | `AGENTS.md` | In a multi-step session continue to the next eligible leaf without asking; otherwise stop at the requested scope. |
| `SEQ-1c` | 424-427 | invariant | `AGENTS.md` | Intermediate commits never mark an unfinished cluster accepted. |
| `SEQ-2a` | 428-431 | duplicate | `TOK-1a` | Read GUIDE's current and held overview and PLAN's register before selecting work. |
| `SEQ-2b` | 428-431 | invariant | `AGENTS.md` | Keep a held leaf's ID, reason, unblock condition and latest completion point visible; review holds at each handoff and resume the earliest eligible one. |
| `SEQ-2c` | 428-431 | invariant | `AGENTS.md` | Never silently skip, move or tick missing verification. |
| `SEQ-3a` | 432-434 | invariant | `AGENTS.md` | `check_guide.py` checks structure; its open-item list is not a scheduler and resolves no hold or dependency. |
| `SEQ-3b` | 432-434 | invariant | `AGENTS.md` | Always state the next executable step and any held obligation that matters. |
| `SEQ-4a` | 435-436 | invariant | `AGENTS.md` | Report confounds when found; correct contradicted current claims where they live and preserve historical measurements with explicit supersession. |
| `HND-1a` | 440-442 | invariant | `AGENTS.md` | When maintainer action is needed, give runnable commands in their own fenced block and restate them rather than referring back. |
| `HND-1b` | 440-442 | invariant | `AGENTS.md` | Routine internal checks do not become the maintainer's chores. |
| `HND-1c` | 440-442 | duplicate | `SEQ-3b` | Always name the next executable leaf. |
| `HND-2a` | 443-453 | invariant | `AGENTS.md` | When reporting the next step, use its PLAN capability class and GUIDE's mapping to recommend one Claude model family (Fable, Opus or Sonnet) with its thinking mode (`Claude: <family> — <mode>`) and a brief task-specific reason; Claude models only, by maintainer decision 2026-09-25, and the family without a version, by maintainer decision 2026-11-13: versions move too fast to store. |
| `HND-2b` | 443-453 | invariant | `AGENTS.md` | Prefer the least costly family judged sufficient and reserve deeper review for unresolved design, interaction or correctness questions; choose each mode independently at the lowest sufficient effort; never name a version, and say so when a mode name is unknown rather than inventing one. |
| `HND-2c` | 443-453 | invariant | `AGENTS.md` | A recommendation is a task judgment, not a guarantee, and changes no active setting. |
| `HND-3a` | 454-456 | task | `docs/rules/documents.md` | PLAN records capability classes, not vendor generations. |
| `HND-3b` | 454-456 | task | `docs/rules/documents.md` | Do not silently downgrade a class; if scope or uncertainty calls for escalation, say why and update PLAN and GUIDE together. |
| `HND-4a` | 457-459 | invariant | `AGENTS.md` | Report what was actually measured, and say plainly when a step was skipped or a result is partial. |
| `HND-4b` | 457-459 | invariant | `AGENTS.md` | For a multi-step session, summarise each completed leaf (ID, result, verification, commit) and name unfinished work separately. |

## Rewordings

A statement listed here arrived with the wording after `now:`; the table above keeps the old wording. A `history:` line is the incident a rule used to carry, as `docs/HISTORY.md` now records it (its record of 2026-11-25).

- `INT-1b` now: `docs/PLAN.md` says what to work on and why; `docs/PROCESS.md` holds the procedures these rules assume.
- `INT-1c` now: Each rule is stated once: here, or in the module the index below names.
- `CLS-1c` now: PLAN owns each leaf's workflow state and capability class; *Handing work back* maps classes to models.
- `JDG-6a` now: Keep the record in its owners: PLAN for what a step involves, its contracts, decisions and dependencies; source and tests for executable invariants; EXPERIMENTS and linked analyses for evidence, predictions, failures and retry triggers; PROCESS for procedures and repeatable methods; HISTORY for a completed record; `analysis/` for a derivation.
- `EXP-2c` now: A read that feeds nothing is prepared or left out, not run, and runs when a use is named; a decision is never taken without the information it needs either (maintainer decision 2026-11-14, RAR-O09).
- `EXP-2c` history: after RAR-O09's four matches were cut to the one PLAN's rule reads
- `TOK-1a` now: Orient once per session, before selecting work: this file and each module the task triggers, then PLAN's register, whose rows are the order of work and whose first open row is the current leaf, and the selected leaf's section; `python tools/diag/plan_leaf.py` prints both with their lines.
- `TOK-4d` now: Write documentation once results are ready, except registrations, new blockers and corrections to false claims.
- `MEA-1b` history: fastchess and weather-factory were reviewed at 2.5.0 on 2026-10-04 and kept, and kept again as backup only on 2026-10-14 when datagen moved to Colosseum
- `MEA-1c` now: They retire only on a maintainer decision recorded here (kept on review 2026-10-04 and again 2026-10-14).
- `MEA-9a` now: The current fingerprint is declared once, in GUIDE's checkpoint; `check_guide.py` fails when a rule file's "currently" value or PLAN's checkpoint row disagrees with it.
- `CHG-3a` now: Never push, create a tag, publish or merge to `master`; the maintainer does, on instruction.
- `CHG-3c` now: Archive tags follow *Evidence*, in `docs/rules/evidence.md`.
- `CHG-5f` now: Add the refspec to the remote once per clone instead, `git config --add remote.origin.fetch "+refs/pull/*/head:refs/remotes/origin/pr/*"`, and every fetch, pruning or not, keeps them (maintainer decision 2026-10-11).
- `CHG-5f` history: on 2026-10-11 this clone held none and the check failed
- `CHG-8b` now: Every scripted edit preserves the file's existing line endings, asserts each anchor is present exactly once and lands in executable code, and re-reads the region afterwards; a mixed-ending file or a silently unmatched anchor is a failed edit.
- `EVI-2c` now: A hash, branch or tag is at most a pointer beside that information, never its only carrier (RAR-S54, `analysis/ledger_commits_2026-09-27.md`).
- `EVI-2c` history: on 2026-09-27, 71 cited commits were on no ref
- `EVI-3b` now: The tags kept are the releases (`v*`) and the oracle's three (`oracle/*`, a vendored Stockfish tree too large for a patch), which retire by the conditions their records state.
- `GAT-8a` now: Bench and counter screens choose and explain candidates and never accept strength: only a registered SPRT accepts one (RAR-S64).
- `DOC-2a` now: PLAN, the regenerated board and GUIDE's hand-kept checkpoint and holds change in the same commit when roadmap status or requirements change; an edit to the rules alone (`AGENTS.md`, `docs/rules/`) needs no PLAN or GUIDE churn.
- `SEQ-4a` now: Report confounds immediately; correct contradicted current claims where they live and preserve historical measurements with explicit supersession.
- `HND-2a` now: When reporting the next step, use its PLAN capability class and the mapping below to recommend one Claude model family (Fable, Opus or Sonnet) with its thinking mode (`Claude: <family> — <mode>`) and a brief task-specific reason; Claude models only, by maintainer decision 2026-09-25, and the family without a version, by maintainer decision 2026-11-13: versions move too fast to store.
- `HND-2b` now: Prefer the least costly family judged sufficient and reserve deeper review for unresolved design, interaction or correctness questions; choose each mode independently at the lowest sufficient effort, and say so when a mode name is unknown rather than inventing one.

## New statements

The split and the day's decisions added these to the core; none comes from
the old rulebook.

| ID | Statement | Where |
|---|---|---|
| `NEW-1` | GUIDE is the maintainer's board: agents keep it current and never orient or take a step from it (maintainer decision 2026-11-25) | opening |
| `NEW-2` | The core holds only what binds on every task; a module binds exactly as the core does | *Rule modules* |
| `NEW-3` | Read a module in full from disk as soon as one of its lines applies; when unsure, read it | *Rule modules* |
| `NEW-4` | A module is skipped only because no trigger fired, never to save time | *Rule modules* |
| `NEW-5` | The index: six modules, each with its scope and its `Kind`, `Paths` and `Before` triggers, repeated word for word at the head of the module | *Rule modules* |
| `NEW-6` | The mapping of the six classes to model families and thinking modes, as GUIDE's *Model mapping* shows it to the maintainer | *Handing work back* |
| `NEW-7` | Before reporting: list files, commands and actions, hold them against the index, read and recheck against any module triggered and not read, name the modules read | *Before reporting work as done* |
| `NEW-8` | A new rule goes into the module its trigger belongs to; only an invariant enters the core | *Keeping these rules small* |
| `NEW-9` | The core with all that loads beside it stays within 18,000 bytes and each module within 18,000; over budget, a rule moves or is compressed first; raising a budget is a maintainer decision | *Keeping these rules small* |
| `NEW-10` | A stub imports the core and nothing else; a module is never imported | *Keeping these rules small* |
| `NEW-11` | A rule keeps at most a short citation of its incident; a maintainer decision keeps its date and substance | *Keeping these rules small* |
| `NEW-12` | Run `tools/diag/check_rules.py` after any edit to a rule file | *Keeping these rules small* |

`NEW-5` and `NEW-9` to `NEW-12` are checked by `tools/diag/check_rules.py`,
which CI runs; `NEW-6` by `tools/diag/check_guide.py`.

## Decided by the maintainer, 2026-11-25

Asked on the first draft of this split, and carried out in it:

- GUIDE is the maintainer's and agents do not read it; PLAN is the agents'
  (*Orientation* above). The first draft had shortened GUIDE's *Active
  experiment* row and put a budget on GUIDE's checkpoint and holds as the
  orientation read; both were withdrawn, and GUIDE's hand-kept text is as
  the maintainer left it, with two sentences added that say whose it is and
  where the agents' copy of the mapping is.
- Duplicates that make sense to merge are merged (*Method and totals*).
- The comment rule, `CHG-10`, moved to `docs/rules/engine.md`: it binds on
  the engine's code, and the tools' comments, which name ledger rows and
  incidents throughout, are outside it.
- Old section names are fixed where a current document or a comment cites
  them (*Old section to new home*).
- To keep room under the budget, the Rust check list (`VER-1b`) went to
  `docs/rules/engine.md` and class escalation (`HND-3`) to
  `docs/rules/documents.md`, and the core's own closing sections were
  shortened. The largest set is 15,285 bytes: 2,715 under the budget and
  8,715 under the smallest limit.

## Still for the maintainer's decision

Nothing here was deleted or reworded. Each item is as the old rulebook had
it, with a recommendation.

1. **The register budget is a new constraint on PLAN.** `check_rules.py`
   fails when PLAN's *Active workflow register* section passes 18,000 bytes
   (7,203 now), because an agent reads it whole at every orientation.
   Recommendation: keep it; when it trips, move the dated order-of-work
   decisions at the head of the section to HISTORY. Dropping it is one
   constant, `ORIENTATION_BUDGET`.
2. **`GAT-10a` cites "PLAN rule 4" for the condition on SPSA.** PLAN's rule
   4 is "Compatibility over completeness"; the curvature sweep that must
   come before a tune is under rule 8, and the tune's shape is rule 7c.
   Possibly a stale number; kept. Recommendation: say which rule is meant.
3. **`PRD-3b` and `CHG-9b` both say that nodes are not Elo.** Left apart:
   the second carries the measured example and sits with the gating rules.
4. **`MEA-5b` names `tools/diag/phase4_differential.py`**, which by the
   statement's own words reads counters that only builds before the compact
   kernel emit. Possibly obsolete; kept. Recommendation: drop the clause
   when the tool is retired.
5. **`tools/diag/feature_matrix.py`'s docstring** says AGENTS records a
   depth sweep reversed by an `--all-features` binary. No rule file does, at
   the base commit or now. Not edited: it is a claim about history, not a
   section name.
6. **Tools with no configuration here.** Windsurf documents 12,000
   characters a rule file; the core does not fit it. Claude Code advises
   under 200 lines a `CLAUDE.md`; the core is 243, and Claude Code loads it
   whole. Neither was acted on.
7. **The numbers in `GAT-3f`** (the harm read's threshold and power) are a
   derivation, kept in `docs/rules/gating.md` because a registration must
   state them. Recommendation: keep.
8. **More room, if wanted.** The next candidates to leave the core are
   `TOK-2` and `TOK-3` (working habits) and `JDG-1` (the aim). None was
   moved: each binds on most tasks.

## Trigger test

Eight past tasks, each held against the new index. *By path* is computed by
`check_rules.triggered` from the files the commits changed; *by kind or
action* is read from the index's `Kind` and `Before` lines. The rules
listed are the ones the task's own record shows it turned on.

| # | Task | By path | By kind or action | Rules the task needed | Reached |
|---|---|---|---|---|---|
| 1 | **Behaviour change.** D.6.29, a stalemate scored in quiescence (`64993ff`), registered as RAR-S139 (`ab0448f`, `22b8db8`), kept by the maintainer's decision with the SPRT unresolved (`46044d5`) | research, engine, gating, measurement, documents, evidence | research (a change to how the engine plays), gating (`GAME_GATE`) | `GAT-1b` registered before any read; `GAT-3h` repair case 3, `[-5,5]`; `PRD-1a`; `RES-5` Stockfish's form; `GAT-8b`, `GAT-9a` an unresolved stop accepts nothing by itself; `VER-8d` the fingerprint record moves with the change; `CHG-6a` changelog; `EXP-1a` the games are the maintainer's | all: core, research, gating, engine, documents |
| 2 | **Neutral repair.** D.9.1's six tablebase option and loading repairs (`68937d1` to `7d3ee78`), recorded in `1bef790` and `3828d86`; a seventh withdrawn as a false premise | research, engine, gating, evidence; documents, measurement for the record | engine (implementation) | `VER-1b` the Rust checks; `VER-8a` fingerprint exact plus targeted checks; `RES-7a` neutral repairs in their own commits; `GAT-3c` no strength gate, an activation read; `JDG-2a` active in CCRL's configuration, not in the pools; `RES-3b` and `PRD-2a` the false premise returned as `NO_CHANGE`; `EVI-1c` the fixture tables stay versioned; `CHG-6a` | all: engine, gating, research, evidence, documents, core |
| 3 | **Documentation only.** D.8's landing recorded in GUIDE and PLAN (`a8c116e`) | documents | evidence (writing a commit hash into a document) | `DOC-1`, `DOC-2a` board regenerated, one commit; `CHG-5b` the landed state cited by its squash commit; `VER-1c`, `VER-8c` no engine inputs in the diff | all: documents, evidence, core |
| 4 | **Research.** D.9's tablebase policy (`bce97cc`), corrected after a second review (`03049a2`); zero games, the policy closed `NO_CHANGE` | research, measurement, evidence, documents | research (research/diagnosis) | `TOK-1a` the leaf found in PLAN's register; `JDG-2a` (written from this task); `MEA-7a`, `MEA-7b` a foreign counter is a prior, an unmeasured size is marked so (written from its second review); `CLS-3` readiness of the repairs it spawned; `RES-5`; `PRD-2a` | all: core, research, measurement |
| 5 | **Measurement handed over.** RAR-O09, four matches prepared (`d1f8cd7`) and cut to the one a rule reads before any game (`eb2e942`) | research, gating, measurement, documents | measurement (preparing a match for the maintainer) | `EXP-1a`, `EXP-1b` prepare, verify, hand over; `EXP-2` information for a use (written from this task); `TOK-4a` no shrinking after results; `VER-7a` wires proved live; `MEA-8a` arms named and hashed; `GAT-1b`, `GAT-6b` registered first, runner named; `HND-1a` commands in a fenced block | all: core, measurement, gating |
| 6 | **Release.** 2.7.0: the no-regression read RAR-M87 (`c1b007c`), shipped after its loss by the maintainer's decision (`a3bc752`), version and notes (`29b515e`, `3c57a89`), the squash commit named (`ca7bf4a`) | documents, engine, gating, measurement, research | evidence (a commit hash into a document; a pull request) | `CHG-3a` the agent tags and publishes nothing; `CHG-4b` one squash through a PR; `CHG-6c`, `CHG-7` the section dated, notes for users; `CHG-5b`; `EVI-3b` the kept tags; `GAT-8b` no acceptance rule after the result; `HND-4a` the loss reported as measured | all: core, documents, evidence, gating |
| 7 | **Tooling and a registered read.** A fixed-depth probe read (`b9fc8a7`) and D.9.2's activation read registered (`fa1e88c`) | measurement, research, evidence | measurement (preparing a read), research (a registration) | `VER-1d` a tool's tests and a negative check; `VER-7a`, `VER-7b` the instrument proved to report; `TOK-4b`, `MEA-5a` no hand-rolled parser; `EXP-2b` the read's use named; `PRD-1a`; `CHG-1a` tool and engine in separate commits | all: core, measurement, research |
| 8 | **A rules edit.** "Ask for AGENTS read in full; a foreign counter is a prior" (`e519c1f`) | none | none | `NEW-8`, `NEW-9`, `NEW-12` where a rule goes, the budget, the check; `VER-1c` | all: core |

The test found three rules that no trigger reached, and the triggers were
changed, not the record:

- Task 2 needs `GAT-3c`, and a neutral repair is not a "playing-strength
  gate". `docs/rules/gating.md` gained the kind "any change to how the engine
  plays, a repair included, to settle what gates it" and the path `src/`.
- Tasks 5 and 6 register a match in `docs/experiments/`, which fired research
  and gating and not the measurement rules that `EXP-2` and `MEA-8a` are in.
  `docs/rules/measurement.md` gained the ledger's paths.
- Task 4 needs `MEA-7a` while writing an analysis, and task 7 a tool:
  `docs/rules/measurement.md` gained `analysis/` and all of `tools/`, and
  `docs/rules/research.md` gained `analysis/` and `src/`, so that
  `check_rules.py --triggered` names both without relying on the kind.

After those changes every needed rule above is in the core or in a module
the task triggers. Two tasks reach a module only by kind or action, which
`--triggered` cannot see: evidence in tasks 3 and 6. The closing check in
the core is what covers those.

## Not verified

- Whether any tool reads a module when its trigger fires, or runs
  `plan_leaf.py` to orient. That is behaviour, and nothing here can test
  it. The closing check and `--triggered` make a skipped module visible;
  they do not make it impossible.
- Whether Antigravity announces its cut. Its documentation says a file past
  24,000 bytes is truncated and no more.
- How Codex counts `~/.codex/AGENTS.md` against its 32 KiB. The page read
  says the limit is on the combined size; the global file here is empty.
- Windsurf's figure was reported from its documentation and not read again.
- The core's size in tokens: unmeasured (see *Load limits and the budget*).
