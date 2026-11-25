# Roadmap and user documents

A module of `AGENTS.md`, as binding as the core. Read it in full when any
line below applies; when unsure, read it.

- Scope: the roadmap documents, the owners of the record, the changelog and the README.
- Kind: documentation/provenance; recording or closing any step.
- Paths: `GUIDE.md`, `docs/PLAN.md`, `docs/HISTORY.md`, `docs/PROCESS.md`, `docs/archive/`, `CHANGELOG.md`, `README.md`.
- Before: marking a step `DONE`, `CLOSED` or `NO_CHANGE`; a leaf's documentation commit; landing a change a user of the engine would notice; writing release notes.

## Documents

- GUIDE's board is generated from `docs/PLAN.md` by
  `python tools/diag/guide_board.py`; never edit it by hand. A PLAN step
  head reads ``- **<ID> <title> — `<class>`….**``: the title is what the
  board shows, the first backticked class is the step's class, and
  `DONE <date>` (or CLOSED, NO_CHANGE) in the field ticks it. Mark a step
  DONE only when finished and verified, in the commit that finishes it,
  and a parent when its last sub-step is; then regenerate. Run
  `python tools/diag/check_guide.py`, which fails on a stale board, rather
  than reading the file.
- PLAN, the regenerated board and GUIDE's hand-kept checkpoint and holds
  change in the same commit when roadmap status or requirements change; an
  edit to the rules alone (`AGENTS.md`, `docs/rules/`) needs no PLAN or GUIDE
  churn. Sub-steps nest one level
  (`C.5.1`) and addenda two (`B.2.0.1`); nothing goes deeper.
- Keep GUIDE short and its board first: every phase, then the checkpoint,
  holds, model mapping, two prompts and operator contract. A finished
  phase's PLAN heading is marked `— CLOSED <date>` over the one- or
  two-sentence summary the board shows, and its text moves verbatim to
  `docs/archive/` (maintainer decision 2026-10-05).
- Keep the record in its owners: PLAN for what a step involves, its
  contracts, decisions and dependencies; source and tests for executable
  invariants; EXPERIMENTS and linked analyses for evidence, predictions,
  failures and retry triggers; PROCESS for procedures and repeatable
  methods; HISTORY for a completed record; `analysis/` for a derivation.
  Extend a missing contract there, not in a parallel
  summary.
- PLAN records capability classes, not vendor generations. Do not silently
  downgrade a class; if scope or uncertainty calls for escalation, say why and
  update PLAN and GUIDE together.

## Changelog, README and release notes

- `CHANGELOG.md`'s `[Unreleased]` grows as the work lands: a change a user
  of the engine would notice (a gate-accepted strength change; a UCI option,
  output or default; a fixed defect; a removal; a build or asset change)
  gets its entry, written for users, in the commit that records it.
  Experiments, refutations and internal refactors get none. A release dates
  the section and never reconstructs it.
- The README, the changelog and the release notes cut from it are for users,
  not for the record: short, simple and to the point. An entry says what
  changed for the user in a sentence or two, not how it was done or measured:
  no game counts, error bars, node counts, parameter counts or internal
  names. A number appears only for a strength gain (rounded, once, in the
  release's opening line) or when the user acts on it, such as an option's
  range. The evidence lives in EXPERIMENTS and the ledger, never here.
