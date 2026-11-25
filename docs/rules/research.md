# Research, experiment design and readiness

A module of `AGENTS.md`, as binding as the core. Read it in full when any
line below applies; when unsure, read it.

- Scope: research, experiment design and the readiness of a playing change.
- Kind: research/diagnosis or experiment design; any change meant to alter how the engine plays, a repair included.
- Paths: `src/`, `analysis/`, `docs/experiments/`.
- Before: writing a research card, packet, prediction or experiment registration; moving a leaf to or from `READY_FOR_IMPLEMENTATION`; opening another engine's source or taking a mechanism, value, table or patch from one.

## Readiness

- `READY_FOR_IMPLEMENTATION` is a hard boundary. A substantial playing change
  crosses it only with: the measured defect or opportunity and its evidence in
  this engine, credible competing explanations, interacting mechanisms, the
  cheapest test that can kill the hypothesis, its falsifier and stop rule, and
  the condition that justifies implementation. A plausible idea or a donor
  feature is not enough.
- Research owns the causal question, competing hypotheses, interaction map,
  prospective prediction, falsifiers, the meaning of each experiment and the
  readiness decision. Prefer cheap discriminating evidence to a sophisticated
  implementation of an uncertain idea.
- Before a substantial engine change, answer in the PLAN research card or the
  experiment registration (link earlier answers): **mechanism** — what should
  improve and what evidence says it is active here; **interactions** — which
  producers, consumers and shared signals, and where it could duplicate,
  cancel or weaken an existing mechanism; **invariants** — which node, TT,
  score, history, board and protocol contracts must hold and how they are
  tested; **falsifier** — the cheapest refuting observation and the rule that
  stops further investment. Routine mechanical fixes need only the contract and
  a focused check.

## Activity and interaction

- When evaluating a defect or repair, check whether it is active in the CCRL
  rated configuration (such as tablebases, clocks, or threads) versus Rarog's
  own pools and gates, because a feature absent from local pools can still
  matter to CCRL and vice versa.
- For nontrivial playing work, check shared signals and feedback: search
  changes move evaluation populations, evaluation changes move pruning,
  ordering evidence may also prune, TT semantics can mask a candidate, and
  rule-50, repetition and promotion closure can make a local feature non-local.
  Use a bounded baseline/A/B/A+B screen when it cheaply separates interaction;
  do not require a factorial for every small change.

## Donors

- Donor engines teach mechanisms, contracts, dependencies, failure modes and
  methods. What may cross is `docs/PROCESS.md`, *The independence boundary*. Neither
  similarity nor a copied value is acceptance evidence. Stockfish is the
  primary donor; Reckless, an AGPL engine, and any other AGPL or unlicensed
  engine teach ideas only: no code, constant, table or patch of their source
  enters the tree (maintainer decision 2026-11-23).

## Predictions

- Freeze predictions before exposure. Afterwards append a calibration: which
  assumption failed, and whether the miss was in sign, magnitude, mechanism,
  interaction, confidence or instrument. Never rewrite a prediction into
  retrospective certainty.
