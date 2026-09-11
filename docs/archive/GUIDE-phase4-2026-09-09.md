# Rarog development guide

## How to work with the engine agent

1. Ask **“what measured defect are we fixing?”** before asking which feature to
   add. Keep unresolved chess or architecture reasoning in `RESEARCH`.
2. Use the cheapest useful falsifier before expensive coding or games. Search
   prior negative results and do not retry one unless its recorded trigger fired.
3. Promote to `READY_FOR_IMPLEMENTATION` only when the mechanism and semantics
   are explicit, local evidence supports it, interactions and falsifiers are
   known, and acceptance/rejection is fixed.
4. Let implementation act like a colleague on ordinary code structure,
   builds, tests and cheap qualification. Do not let it silently redesign or
   broaden the experiment. A false premise returns the leaf to `RESEARCH`.
5. The agent prepares and verifies long tournaments, SPRTs, datagen, tuning,
   PGO and profiling jobs; the maintainer starts them unless explicitly agreed
   otherwise.
6. Freeze the prediction before exposure. Keep it beside the result and judge
   the postmortem against it; retrospective certainty is not prediction.
7. A clean negative result is progress. Do not chase donor features, invent an
   evidence-layer exchange rate, or increase implementation volume when the
   hypothesis is weak. Return to evidence, alternatives and a discriminating
   test.

| State | Boundary / control |
|---|---|
| `RESEARCH` | Research owner states evidence, alternatives, interactions, prediction, falsifier and stop rule. |
| `READY_FOR_IMPLEMENTATION` | Research decision is frozen; implementation may make ordinary local engineering choices. |
| `IMPLEMENTED` | Intended semantics exist, but no qualification claim is implied. |
| `LOCAL_QUALIFIED` | Cheap correctness/performance checks passed; agent prepares any expensive gate. |
| `GAME_GATE` | Registered playing gate is running or resolved under maintainer control. |
| `CLOSED` | Accepted, rejected, no-change or deferred disposition and calibration are recorded. |

### Current model mapping

PLAN records only stable capability classes. Edit this table when model
generations change; do not rewrite the roadmap. Effort is selected per model.
These are maintainer judgments, not measured rankings: the 2026-09-07
engine-choice audit records no comparative measurements for any Claude model
on this project, so do not cite this table as evidence of superiority.

| Class | Capability | GPT | Claude |
|---|---|---|---|
| `R3` | Frontier causal/architecture research | GPT-6 Astra — Extra High | Claude Fable 5.1 — High |
| `R2` | Bounded correctness-sensitive reasoning | GPT-5.6 Sol — High | Claude Opus 5 — High |
| `I2` | Difficult implementation | GPT-5.6 Sol — High | Claude Opus 5 — High |
| `I1` | Well-specified implementation | GPT-5.6 Terra — Medium | Claude Sonnet 5 — Medium |
| `M` | Mechanical/docs/provenance | GPT-5.6 Terra — Medium | Claude Sonnet 5 — Medium |
| `V` | Verification/measurement | GPT-5.6 Sol — High | Claude Sonnet 5 — High |

### Reusable research prompt

> Investigate `<PLAN leaf>` as research, not implementation. Read PLAN,
> EXPERIMENTS, linked analysis and relevant source; measured evidence outranks
> roadmap assumptions. Search negative results and retry triggers. State the
> precise question, leading and competing hypotheses, interactions/duplicated
> signals, and whether search, evaluation, tooling or instrument effects could
> explain it. Design the cheapest discriminating test first; freeze its
> prediction and confidence, falsifiers and stop rule before exposure. Avoid
> substantial engine implementation. Finish `READY_FOR_IMPLEMENTATION`,
> `MORE_RESEARCH` or `NO_CHANGE`, with the evidence for that verdict.

### Reusable implementation prompt

> Implement `<PLAN leaf>` from its registered handoff. Treat the research
> decision, semantics, invariants and experiment design as fixed. Use normal
> engineering judgment for code, focused builds/debugging/tests and cheap
> qualification. Do not broaden the mechanism, tune unrelated behavior or
> continue other roadmap work. If a research premise is false, preserve useful
> instrumentation, document the contradiction and return the leaf to
> `RESEARCH`. Prepare but do not start maintainer-owned expensive jobs. Report
> changes, interactions, validation, remaining gate and false assumptions;
> update roadmap/evidence documents under their ownership rules.

## Status board

Every phase, step and sub-step remains visible here. Rationale and design live
in `PLAN.md`; durable evidence in `EXPERIMENTS.md`; repeatable procedures in
`PROCESS.md`; finished history in `TRACKER.md`. `GUIDE.md` and `PLAN.md` change
together, and `python tools/diag/check_guide.py` must pass.

## Current checkpoint

| Item | Value |
|---|---|
| Released baseline | **2.3.2** at `f931722` on `master` |
| Last strength-qualified head | RAR-E12 + 4.9a.7, **6,901,489 nodes / EBF 2.458**. Includes the 4.9a.4 mate drive, which is bench-INVISIBLE |
| Development fingerprint | **7,601,220 / EBF 2.474**, SEE repair `fce0b44`; **cluster ACCEPTED by RAR-E15**, +12.12 +/- 10.17 Elo |
| Integration branch | `dev`; the hce-v3 refit `d1d95ab` is accepted |
| Frozen oracle | `hybrid` at `75d0d43`; never merge it into Rarog |
| Measured search deficit | **355.26 +/- 27.03 Elo** equal nodes; **250.77 +/- 13.12** equal time; speed worth **104.5 Elo** |
| Accepted Phase-4 gains | ProbCut **+15.56 +/- 10.02**; root LMR relief **+2.33 +/- 1.85**; HCE refit **+22.04 +/- 7.51**; TB-corrected labels **+6.73 +/- 3.82**; hce-v3 refit **+11.81 +/- 5.33** |
| Active experiment | none; **RAR-E15 ACCEPTED 2026-09-08 at +12.12 +/- 10.17 Elo, +18.40 nElo** |
| Instrument state | **4.10 repaired; v2 baselines/floors, budget transfer, label audit, mate-drive closure and conversion corrections recorded.** |
| Current step | **4.12.1** — adopt the 4.12 order and confirm recognizer-vs-scale classification, `RESEARCH / R3`. 4.11b.19 and section 4.11b are CLOSED |
| Next release | Conditional 2.4.0 at 4.20; NNUE follows either way |

## Next and held work

**Next: 4.12.1 — adopt the registered 4.12 order and confirm the
recognizer-vs-scale classification**, `RESEARCH / R3`. Nothing is owed to the
maintainer.

**4.11b.19 is CLOSED and section 4.11b with it.** (a) `55e228a` fixed the
cross-engine harness, (b) `021dc98` **BANKED +2.48% whole-search NPS**
[+2.29%, +2.65%], (c) closed **`NO_CHANGE`** -- two candidates cleared the
bench gate at +12.7% and the bundled pooled-PGO run then measured **−0.55%**
[−0.76%, −0.30%], so both were reverted at `39542b7` -- and (d) re-measured
all four arms on the reverted head. **The generation gap to Basilisk is now
33.2%, down from 46.2%, and capture generation is 5.8% AHEAD**; RAR-M43's
table and Elo arithmetic are superseded by
`analysis/board_comparison_411b19_2026-09-09.md`. Fingerprint stayed exactly
7,601,220 / EBF 2.474 throughout, so no game gate was owed. **The leaf's
transferable result: a board microbenchmark column is not a proxy for search
speed here — (b) removed work and +11% became +2.48%; (c) moved work around
and +12.7% became −0.55%.**

**Section 4.11b is CLOSED**, and 4.11.7–4.11.10 before it. Its playing gate
RAR-E15 was **ACCEPTED at +12.12 +/- 10.17 Elo, +18.40 nElo**, H1 at 1,950
games, so the fingerprint 7,601,220 / EBF 2.474 now has its integrated verdict
and is the accepted foundation for 4.12. RAR-M41 banked **+1.421%
[+0.953%, +1.764%]** pooled-PGO throughput; 4.11b.10, 4.11b.11, 4.11b.12,
4.11b.14, 4.11b.15 and 4.11b.19(c) closed `NO_CHANGE` on evidence; RAR-M42
verified the 4.12 order unchanged by rederivation.

| Open hold / obligation | Resume or resolve when | Must be resolved before |
|---|---|---|
| **4.12.21** Future 7-man evidence gap | Independent truth/verification becomes available, or a justified exclusion is recorded | 4.12.23 closes |
| **KRP-KB win-preserving** 0.9990 -> 0.9949 (−2.2 SE, RAR-M42) | Non-blocking now; becomes blocking if a later change pushes it past 3 SE | 4.12.6 closes (owner) |

Follow the earliest **unblocked** leaf. Keep held items unticked in their
original place; check their return conditions at every handoff. PLAN owns
dependencies and detail. The agent maintains this overview and reports the
next step; the maintainer does not need to run a queue script.

## Phase 4 — bounded pre-NNUE search and HCE

Open active-horizon leaves show `workflow state / capability class`. Dependency
holds still come from PLAN; a readiness label never lifts one.

- [x] **4.0** Evidence, baseline and oracle freeze — RAR-M12
- [x] **4.1** Instrumented oracle — `hybrid-diag` `de568b3`
- [x] **4.2** Differential observation harness — RAR-S55
- [x] **4.2a** Harness and instrument integrity sweep — Basilisk-derived
    - [x] **4.2a.1** `sprt.ps1` options-free repair and `-NoAdjudication` wire proof — `cb5ed2a`
    - [x] **4.2a.2** Exit-status sweep: no unguarded native call found; already protected
    - [x] **4.2a.3** Anomaly guard rate-limited so it discriminates instead of voiding every gate — `334c084`
    - [x] **4.2a.4** `sprt.ps1` refuses options its mode cannot honor — `3fb9f57`
- [x] **4.2b** Time-forfeit diagnosis at test concurrency — RAR-M14; fixes belong to 4.17
    - [x] **4.2b.1** Games end at 97-99% of clock; ~2% aggregate slack, ~100ms per game
- [x] **4.3** Mechanism map and order freeze
- [x] **4.4** Matched ablation plus fixed-node correction
- [x] **4.5** LMR contract study — no interior gain; RAR-S70 root gain retained
- [x] **4.6** Shallow-selectivity/rewrite continuation — no accepted gain
    - [x] **4.6.1** Quiet SEE screen: gap closure **+3.38 +/- 27.08**, stopped null
    - [x] **4.6.2** SearchCore: **-9.76 +/- 17.70** over 712 games, reverted
    - [x] **4.6.3** Broad selectivity SPSA and another rewrite declined
- [x] **4.7** Qualify HCE data, labels, instruments and current-source maturity
    - [x] **4.7.1** Archive audit: 600k independent starts; capacity 2.30M/127,778/127,778
    - [x] **4.7.2** Hash-frozen `hce-v2`: pure self-play WDL from 750k unique openings
    - [x] **4.7.3** Exact 1,218-slot instrument partition and vector/bake/rebuild smoke
    - [x] **4.7.4** Current-source Stockfish maturity map
- [x] **4.8** Refit and gate the complete existing HCE surface
    - [x] **4.8.1** Fresh 150k-game confirmation: test loss **0.12252203** vs **0.12330291**
    - [x] **4.8.2** Baked at **7,226,051 / 2.460**, NPS **-1.19%**; RAR-E06 registered
    - [x] **4.8.3** RAR-E06 **ACCEPTED**: H1 at 3,914 games, **+22.04 +/- 7.51 Elo**, +32.05 nElo
- [x] **4.8a** Post-refit redundancy removal — **closed, no gate owed**; RAR-E07
    - [x] **4.8a.1** Inventory: 5 slots zeroed, 90 of 132 sparse slots structurally unreachable
    - [x] **4.8a.2** No removal cluster exists; 3 inert 1-slot terms handed to 4.18.2
- [x] **4.9** Structural HCE clusters — **closed, none opened**; RAR-E09
    - [x] **4.9.1** Residual audit — RAR-E09: no structural residual; a label defect instead
    - [x] **4.9.2** Closed: no cohort residual licenses a cluster; retry trigger recorded
- [x] **4.9a** Endgame truth foundation — done; five results SUPERSEDED, owners named
    - [x] **4.9a.1** Truth corpus — result was invalid, REPLACED by the 4.11.1 baseline
    - [x] **4.9a.2** Endgame-start cohort book — 788 verified positions, 21 families
    - [x] **4.9a.3** Regression contract — floors REPLACED by 4.11.2; the 64 theory vetoes always stood
    - [x] **4.9a.4** SUPERSEDED -> 4.12.7 and 4.12.9 — gain stands (KBN-K **19.4% -> 96.9%**); promotion closure reaches 6 families
    - [x] **4.9a.5** RAR-E08 **ACCEPTED** +6.73 +/- 3.82 Elo — TB-corrected labels win
    - [x] **4.9a.6** `hce-v3-tb` fitted and gated — RAR-E12 **+11.81 +/- 5.33 Elo**
    - [x] **4.9a.7** SUPERSEDED -> 4.12.2 — KRPKR drawn overclaim **37.1% -> 25.8%** stands; conversion half does not
    - [x] **4.9a.8** SUPERSEDED -> 4.12.6 — KRPKB drawn-cohort null stands; conversion half does not
- [x] **4.10** Instrument integrity and tooling upgrade — 12 leaves, complete
    - [x] **4.10.1** Tablebase-truth termination; material shed becomes a diagnostic
    - [x] **4.10.2** Cohort fingerprint; refuse any comparison across position sets
    - [x] **4.10.3** Sharded workers; parallel output byte-identical to serial
    - [x] **4.10.4** Prove every guard FAILS on a known-bad input; thin-sample refusal
    - [x] **4.10.5** Measurement-layer contract and per-report layer/budget/set fields
    - [x] **4.10.6** Node budget as a run condition; 60k/200k/600k bracket runner
    - [x] **4.10.7** Held-out split, McNemar paired z, runner-up slot, spent-cohort rule
    - [x] **4.10.8** `datagen_label_audit.py` — corpus labels against tablebase truth
    - [x] **4.10.9** Gate runner refuses wrong revision, dirty tree or mismatched bench
    - [x] **4.10.10** `check_guide.py` enforces SUPERSEDED owners and compares step sets
    - [x] **4.10.11** Compile-time bound on the shipped mop-up constants, every build type
    - [x] **4.10.12** Feature-matrix build audit; `--all-features` is not the shipped config
- [x] **4.11** Re-measurement and re-derivation — corrections recorded in place
    - [x] **4.11.1** Re-run both truth arms — head **0.9300**, reference **0.9920**
    - [x] **4.11.2** Floors re-derived — **0.9300** over n=1371, 18 families, cohort-stamped
    - [x] **4.11.3** Attained reference results frozen — **1361/1372**, hard residue **8**
    - [x] **4.11.4** Drawn-share census — **KR-KN 1.000, KR-KB 0.996** at +346/+307 cp
    - [x] **4.11.5** Occurrence split by root — **3 of 40 roots give 56%** of the census
    - [x] **4.11.6** 4.12 re-ranked and REGISTERED; leaves renumbered to match
    - [x] **4.11.7** Budget transfer — net reference deficit 85/27/16 at 60k/200k/600k; RAR-M21
    - [x] **4.11.8** Label audit — raw game-result contradiction: `hce-v2` 4.39%, `hce-v3` source 8.99%; RAR-M22
    - [x] **4.11.9** Mate-drive promotion closure — 6 families; KBP-KB/KBP-KN each net −1 conversion debt; RAR-M23
    - [x] **4.11.10** Conversion claims corrected — E08 aggregate superseded; KQ-KP -3.79 pp confirmed; RAR-M24
    - [x] **4.11.11** Panic reported on stdout, where the harness keeps it
    - [x] **4.11.12** Occurrence re-measured over 36,400 rated games; 4.12 re-ranked to **v2**
- [x] **4.11b** Board correctness and HCE throughput
    - [x] **4.11b.1** Freeze the board audit and three-engine comparison
    - [x] **4.11b.2** Strengthen benchmark coverage and correctness oracles; RAR-M25
    - [x] **4.11b.3** Repair move parsing and counter boundaries; RAR-M26
    - [x] **4.11b.4** Define SEE contracts and independent fixtures; RAR-M27
    - [x] **4.11b.5** Repair SEE king legality, created pins and recapture promotions; RAR-M28
    - [x] **4.11b.6** Add neutral value injection; restore comparable SEE timing; RAR-M29
    - [x] **4.11b.7** Profile board work in HCE search; RAR-M30
    - [x] **4.11b.8** Withdraw unqualified pin candidate; retain oracle — **CLOSED**, RAR-M31
    - [x] **4.11b.9** Measure fused piece relocation — **ACCEPTED**, RAR-M33
    - [x] **4.11b.10** Research shared pin/check information — **CLOSED `NO_CHANGE`**, RAR-M34
    - [x] **4.11b.11** Optimize the corrected SEE kernel — **CLOSED `NO_CHANGE`**, RAR-M35
    - [x] **4.11b.12** Decide whether king-square caching pays — **CLOSED `NO_CHANGE`**, RAR-M37
    - [x] **4.11b.13** Define history capacity and mutation contracts — **DONE**, RAR-M38
    - [x] **4.11b.14** Decide whether a larger representation change pays — **CLOSED `NO_CHANGE`**, RAR-M39
    - [x] **4.11b.15** Review draw/null/repetition policy boundaries — **CLOSED `NO_CHANGE`**, RAR-M40
    - [x] **4.11b.16** Qualify integrated correctness and throughput — **QUALIFIED +1.421%**, RAR-M41
    - [x] **4.11b.17** Register and qualify the playing cluster — **ACCEPTED +12.12 Elo**, RAR-E15
    - [x] **4.11b.18** Refresh affected endgame evidence and close — **CLOSED**, RAR-M42
    - [x] **4.11b.19** Caller-owned move-list delivery; constant-factor screen; corrected comparison — **CLOSED / I1**, RAR-M44; (b) banked **+2.48% NPS**, (c) `NO_CHANGE` at −0.55% and reverted, (d) re-measured
- [ ] **4.12** Endgame reference functions — order registered by 4.11.6, re-derived at 4.11.12
    - [ ] **4.12.1** Adopt the order; confirm recognizer-vs-scale classification — **RESEARCH / R3**
    - [ ] **4.12.2** KRPKR [ref 13] scale — 30.7% overclaim remains after 4.9a.7 — **RESEARCH / R3**
    - [ ] **4.12.3** KXK [ref 3] verdict — largest occurrence in the set (37.8%); mechanism at 4.9a.4 — **RESEARCH / R3**
    - [ ] **4.12.4** KRKN [ref 8] scale — **100%** overclaim at +346; Rarog reaches it 1.6x the pool rate — **RESEARCH / R3**
    - [ ] **4.12.5** KRKB [ref 7] scale — **99.6%** overclaim at +307; same 1.6x over-representation — **RESEARCH / R3**
    - [ ] **4.12.6** KRPKB [ref 14] scale — **100%** overclaim at +328 after 4.9a.8's rook pawns — **RESEARCH / R3**
    - [ ] **4.12.7** KBPKB [ref 17] scale — 60.9% overclaim; mate-drive debt from 4.11.9 — **RESEARCH / R3**
    - [ ] **4.12.8** KRKP [ref 6] scale — 26.4% overclaim at +72 — **RESEARCH / R2**
    - [ ] **4.12.9** KBPKN [ref 19] scale — 50.7% overclaim; mate-drive debt from 4.11.9 — **RESEARCH / R3**
    - [ ] **4.12.10** KQKR [ref 10] verdict — deficit 23/13/3 at 60k/200k/600k; 0.63% of games — **RESEARCH / R3**
    - [ ] **4.12.11** KPK [ref 5] scale — 4.6% overclaim; present bitbase — **RESEARCH / R2**
    - [ ] **4.12.12** KPKP [ref 20] scale — 3.8% overclaim, nearly clean — **RESEARCH / R2**
    - [ ] **4.12.13** KQKP [ref 9] verdict — historical E08 -3.79 pp; current 60k deficit closes at 200k/600k — **RESEARCH / R3**
    - [ ] **4.12.14** KBNK [ref 4] verdict — corrected refit DTZ 0.7260 -> 0.6753; attribution unisolated — **RESEARCH / R3**
    - [ ] **4.12.15** KNNKP [ref 2] scale — 57.7% overclaim; holds 7 of the 8 hard residue — **RESEARCH / R3**
    - [ ] **4.12.16** KNNK [ref 1] scale — **no defect measured**; no-change closure expected — **RESEARCH / R2**
    - [ ] **4.12.17** KPsK [ref 16] ? — **MEASURE FIRST** — 4.52% of Rarog's games — **RESEARCH / R3**
    - [ ] **4.12.18** KBPsK [ref 11] ? — **MEASURE FIRST** — 2.59% of Rarog's games — **RESEARCH / R3**
    - [ ] **4.12.19** KBPPKB [ref 18] ? — **MEASURE FIRST** — 0.50% of games — **RESEARCH / R3**
    - [ ] **4.12.20** KQKRPs [ref 12] ? — **MEASURE FIRST** — 0.42% of games and 4.41% of the TREE — **RESEARCH / R3**
    - [ ] **4.12.21** KRPPKRP [ref 15] ? — **5.40% and UNVERIFIABLE**; 7-man hold — **RESEARCH / R3**
    - [ ] **4.12.22** Dependency-complete family refits and gates — **RESEARCH / R3**, families not ready
    - [ ] **4.12.23** Conversion, theory, STC/LTC and cohort closure — **READY_FOR_IMPLEMENTATION / V**, dependency-held
- [ ] **4.13** Datagen label truth and corpus contract
    - [ ] **4.13.1** Audit data/label pipeline; quantify contradictions
    - [ ] **4.13.2** Relabel and whole-game adjudication as separate registered arms
    - [ ] **4.13.3** Record that more datagen nodes is the weak fix; do not buy it
    - [ ] **4.13.4** Freeze the winning contract under a new corpus name
- [ ] **4.13a** ANALYSIS — HCE correctness, feature interactions and throughput
- [ ] **4.14** Iterated whole-surface refit cycles — at least one is owed
    - [ ] **4.14.1** Initialization control: neutral start vs accepted start, offline
    - [ ] **4.14.2** Opening supply: reuse is clean; fresh starts are a nice-to-have
    - [ ] **4.14.3** Composition screen against the `datagen-v1` archive (sizing)
    - [ ] **4.14.4** Regenerate and hash-freeze under a new corpus name
    - [ ] **4.14.5** Cycle 1: full 4.8 schedule, own frozen test, registered gate
    - [ ] **4.14.6** Repeat while a cycle accepts; stop at the first that does not
    - [ ] **4.14.7** Record cycles, refresh the residual audit and close
- [ ] **4.15** Search composition, throughput and score authority
    - [ ] **4.15.1** ANALYSIS — search/ordering/pruning interactions and throughput
    - [ ] **4.15.2** One candidate and gate, only if 4.15.1 isolates a unique defect
    - [ ] **4.15.3** Audit the SEE scale after final HCE; zero games
    - [ ] **4.15.4** Fit justified SEE values through the existing interface; gate
    - [ ] **4.15.5** Revalidate normalized SEE timing after fitting
- [ ] **4.15a** ANALYSIS — TT, caches, hashing and memory layout
- [ ] **4.15b** ANALYSIS — threads, engine/UCI lifecycle and tablebases
- [ ] **4.15c** ANALYSIS — diagnostics, harnesses and build/ISA delivery
- [ ] **4.16** Optional post-HCE search SPSA; skip without a displaced optimum
- [ ] **4.17** Time management: review, repair and gate — owns all TM work
    - [ ] **4.17.1** ANALYSIS — clock/search/root interactions; revalidate behavior
    - [ ] **4.17.2** `Move Overhead` vs forfeit rate on a null pair; size it first — RAR-M14
    - [ ] **4.17.3** `RootConfTime`'s six identifiable consumers: tune or remove — RAR-S47
    - [ ] **4.17.4** Root-instability TM from a completed snapshot only — RAR-X06, RAR-R05
    - [ ] **4.17.5** Registered gate; zero forfeits is a precondition, not the verdict
- [ ] **4.18** Search cleanup and clean checkpoint
    - [ ] **4.18.1** Audit coverage/interaction closure; dead-path inventory
    - [ ] **4.18.2** Remove unconsumed 4.6 alternatives, plus the 3 terms RAR-E07 left inert
    - [ ] **4.18.3** Re-verify tests, clippy, fingerprint, NPS and deficits
- [ ] **4.19** Final HCE/search checkpoint, attribution and maturity closure
- [ ] **4.20** STC/LTC/4T, NPS, portability, ISA and release gate
- [ ] **4.21** Universal binaries — investigate and test automatic CPU selection
    - [ ] **4.21.1** Freeze Stockfish reference and Rarog build constraints
    - [ ] **4.21.2** Compare designs; specify safe dispatch and overrides
    - [ ] **4.21.3** Build an isolated HCE prototype
    - [ ] **4.21.4** Verify compatibility, ISA and chess identity
    - [ ] **4.21.5** Measure performance, startup, size and memory
    - [ ] **4.21.6** Decide adoption; assign implementation and release gates

## Phase 5 — NNUE runway

- [ ] **5.1** Measurement corpus handoff; freeze 4.7 splits and manifests
- [ ] **5.2** Per-ply state and dirty pieces
    - [ ] **5.2.1** Freeze board baselines, including Reckless; start the cost ledger
    - [ ] **5.2.2** Define factual move deltas and callback timing
    - [ ] **5.2.3** Add the update interface and ownership
    - [ ] **5.2.4** Verify every transition independently
    - [ ] **5.2.5** Qualify HCE behavior and measure move-event costs
- [ ] **5.3** Accumulator scaffolding
    - [ ] **5.3.1** Allocate evaluator-owned per-thread/per-ply state
    - [ ] **5.3.2** Define validity, refresh and null semantics
    - [ ] **5.3.3** Verify scaffolding against full refresh
    - [ ] **5.3.4** Gate HCE behavior and measure scaffold costs
- [ ] **5.4** Trainer preflight: pin `D:/code/net_trainer`, Bullet, toolchain, GPU
- [ ] **5.5** Runway gate: fingerprint, debug/release, unwind, pilot corpus
- [ ] **5.6** Threat-map hooks, optional; reserve only to avoid a second rewrite

## Phase 6 — baseline NNUE

- [ ] **6.0** Trainer hardening: strict CLI, deterministic splits, hashes, seeds
- [ ] **6.1** Controlled data: 30-60M unique positions, by-game splits, manifests
- [ ] **6.2** Baseline networks: at least two seeds per width/bucket configuration
- [ ] **6.3** Scalar integration: `quantised.bin` contract, integer-exact conformance
- [ ] **6.4** Incremental and SIMD parity, performance and cost attribution
    - [ ] **6.4.1** Prove actual-network incremental/full-refresh parity
    - [ ] **6.4.2** Qualify SIMD, integer bounds and supported targets
    - [ ] **6.4.3** Measure board/update/inference costs; gate whole-search NPS
- [ ] **6.5** Architecture loop: output buckets, king buckets, then relation inputs
- [ ] **6.6** Gross search-scale safety; broad search fitting waits for 7.3
- [ ] **6.7** Baseline release: beat the pre-NNUE master at STC/LTC and 4T

## Phase 7 — NNUE frontier and final search fit

- [ ] **7.0** Residual and disagreement analysis by phase, material, king, cohort
- [ ] **7.1** Data frontier: scale, deduplicate, mine hard positions, refresh on policy
- [ ] **7.2** Architecture ladder, one axis at a time
- [ ] **7.3** Re-audit NNUE/search interactions; fit justified displaced constants
- [ ] **7.4** Frontier gate against 2.3.2, the Phase-4 head and target engines

## Phase 8 — scaling, platforms and product completeness

- [ ] **8.0** High-thread and NUMA: price depth diversity at 4/8/16T
- [ ] **8.1** Extend/defer dispatch from 4.21; TT/net placement and large pages
- [ ] **8.2** Product/platform: demand-led Chess960, distributed testing
- [ ] **8.3** Scaling release: topology, clock, net, ISA and user-doc gate

## Phase 9 — optional post-NNUE classical fallback

Enter only if serious NNUE work fails and the maintainer abandons NNUE.

- [ ] **9.1** King-safety semantic rework
- [ ] **9.2** Material-specific winnability and scaling
- [ ] **9.3** Passer and pawn conditionality
- [ ] **9.4** Threat and usable-activity conditionality
- [ ] **9.5** Material/phase specialization, last classical step only
