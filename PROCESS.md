# Rarog recurring procedures

The recurring procedures: how a leaf is researched, registered, implemented,
gated and closed, and how the build, fit, tune and gate instruments are run.
`AGENTS.md` holds the rules that stop wrong results; `PLAN.md` holds the
roadmap. This file also owns the independence boundary with donor engines.

## Recurring procedures

### Research packet and implementation handoff

Use a packet under `analysis/` only when the decision would make PLAN
unwieldy; trivial work stays in PLAN. A packet is a live decision record, not
a second roadmap. PLAN owns its state and links it.

```markdown
# <leaf> — <research question>

## Decision needed
## Known evidence
<!-- Concrete IDs, measurements and conditions; distinguish fact from inference. -->
## Unknowns
## Hypotheses
<!-- H1 plus credible competitors; do not add decorative alternatives. -->
## Interaction map
<!-- Producer -> stored state -> consumers; duplicated signals and lifetimes. -->
## Cheapest discriminating tests
## Prospective prediction
<!-- Expected diagnostic/Elo movement if defensible, confidence, likely failure. Freeze before exposure. -->
## Falsifiers and stop conditions
## Decision
<!-- READY_FOR_IMPLEMENTATION / MORE_RESEARCH / NO_CHANGE. -->
## Implementation handoff
<!-- Fill only when READY_FOR_IMPLEMENTATION. -->
```

The handoff fixes the goal, exact intended semantics, why local evidence says
it should work here, producer/state/consumer map where relevant, files and
subsystems, interactions, invariants, instrumentation, deterministic tests,
cheap qualification, maintainer-owned expensive gate, acceptance/rejection
rule, explicit non-goals and adjacent mechanisms not to change. If any central
field is still a design choice, the leaf remains `RESEARCH`.

During research, inspect PLAN, EXPERIMENTS, the retry map, linked analyses and
relevant source. Prefer the cheapest test that separates causal explanations.
For important interactions, use a bounded baseline/A/B/A+B screen when it can
distinguish independence from masking; do not require it for every small change.
Freeze predictions in EXPERIMENTS before exposure. A later explanation is
calibration, never proof that the outcome was predicted.

### Experiment registration

Register an experiment as one row in the `EXPERIMENTS.md` section that owns
it, before any games. When the registration is longer than a row, write it in
an `analysis/` packet with the fields below and cite the packet from the row;
append the result and calibration there without rewriting the prediction.

```markdown
### RAR-<area><number> — <short name>

- Date / owner:
- Baseline SHA / candidate SHA / dirty-diff hash:
- Binary / compiler / PGO identity:
- Research question:
- Hypothesis / proposed mechanism:
- Competing hypotheses:
- Interacting mechanisms / consumers:
- **PRE-REGISTERED PREDICTION (freeze before exposure):**
  - Expected diagnostic movement:
  - Expected Elo sign/range, if defensible:
  - Probability positive/useful and confidence basis:
  - Most likely failure mode:
- Falsification criteria:
- Cheapest prior falsifier: test / result / implementation still justified?:
- Registered gate and stop rule:
- Full conditions / provenance: flags, manifests and hashes; book/hash, TC,
  threads, Hash, concurrency, affinity, adjudication, node budget and cohort:
- Result:
  - Diagnostics: nodes, EBF, NPS, depth, counters, suites (not the verdict):
  - Games/verdict: games, W-D-L, Elo/nElo and CI, LLR:
- Disposition: accepted / retained / rejected / neutral/inconclusive /
  observation / no-change / deferred:
- **PREDICTION CALIBRATION (append after exposure):**
  - Original prediction (do not rewrite):
  - Observed result; sign and magnitude reasonable?:
  - Proposed causal mechanism supported?:
  - Missed interaction or instrument failure?:
  - Confidence over/under-calibrated?:
- Postmortem: changed causal assumption / what did not change / alternatives:
- Conditional lesson:
- Retry trigger or `closed`:
- Artifacts / commits:
```

### Step lifecycle and audit handoff

Before selecting a leaf, review GUIDE's current/held overview and PLAN's
execution register. Select the earliest unblocked dependency-compatible leaf;
keep skipped work visible and return when its unblock condition holds.
After one leaf, record its result/status, commit verified work, report the
next executable leaf and relevant holds, then stop. The maintainer does not
need to run the checklist validator.

Analysis-only leaves follow PLAN section 2's subsystem audit contract. They
deliver findings, an interaction/cost map and derived numbered implementation
leaves, or a justified no-change result. They do not implement speculative
improvements during the audit. Required later work stays open under its owner.

For every behavioral step:

**Gate the fitted dependency-complete cluster, not each feature and not the
whole phase at once.** Internal substeps may be too sparse or coupled to win
before their consumers and weights move together. Conversely, postponing all
games until the end destroys attribution and lets losing structures hide.

1. **Audit** — name the problem, its Rust owner, all interacting consumers and
   the local diagnostic population. Update `PLAN.md` first if the evidence
   contradicts the planned order.
2. **Register** — add an `EXPERIMENTS.md` ID with hypothesis, baseline SHA,
   candidate scope, expected direction, gate, cap and stop rule, before games.
   Bounds default to `[0,3]` nElo; widen only for a genuinely large prior and
   justify it in the row. Removals need a bracket permitting a small loss;
   unknown-sign repairs need a symmetric one. Size from RAR-M10 at the
   EXPECTED value before choosing (`tools/spsa_convergence_model.py`).
3. **Implement** — the smallest dependency-complete cluster. Substeps may be
   compiled and diagnosed separately, but are not expected to pass standalone
   and no incomplete cluster becomes the next strength baseline.
4. **Prove correctness** — fmt, the engine suite in debug and release
   (`cargo test -p rarog`, both profiles), all-feature clippy and targeted
   invariants. A behavior-neutral diagnostic
   seam must preserve the exact accepted fingerprint when disabled.
5. **Explain** — use the frozen suite at fixed depth/nodes to compare nodes,
   qnodes, move source, cutoff index, TT use, reductions and re-searches,
   pruning, extensions and aspiration against the oracle. Counters explain a
   candidate; they cannot accept it. For extension/depth-authority changes,
   pair average depth at fixed nodes with the tactical suite at both fixed
   depth and equal node cost. Register floors and the treatment of disagreement
   before measuring. A true correctness canary may veto; disagreement between
   aggregate depth and tactical counts is otherwise inconclusive until the
   per-position and equal-cost results explain it.
6. **Fit** — hold the candidate's categorical semantics fixed while fitting;
   do not freeze unrelated HCE coordinates merely because an older stage tuned
   them. Fit the complete identifiable/covariant surface with the correct
   linear or nonlinear instrument. Targeted SPSA is residue only. Complete
   theta; do not select a checkpoint retrospectively.
7. **Gate** — prepare and verify the fitted candidate, revision-matched
   baseline, clean final-PGO recipes, manifests and registered paired UHO SPRT
   command. The maintainer starts the expensive build/game job unless
   explicitly delegated. Do not change the candidate, bounds, cap, book or
   adjudication after observing games.
8. **Close** — accept and commit only a passing result. Otherwise revert the
   behavior, keep the evidence row and restore the prior fingerprint. Ablate a
   surprising integrated result before crediting a subcomponent.
9. **Advance** — start the next item only after the preceding one is accepted,
   rejected, explicitly closed, or visibly held with a dependency rule that
   permits this independent next leaf. A hold never accepts an incomplete
   candidate or waives a gate; resume it before its recorded boundary.

A separable categorical alternative may have a preliminary SPRT, but that
never replaces the locally fitted integrated cluster SPRT. The programme
checkpoints (PLAN B.9, C.11, E.1) own the combined confirmation runs; none may
rescue an earlier losing cluster.

Two failed coherent clusters in one programme trigger a return to evidence
(PLAN rule 6), not silent closure. A programme may close early only by
explicitly conceding its target; no unknown or first-draft contract may be
presented as mature.

### The independence boundary

Donors: **Reckless** for search, threading, time management and NNUE;
**Stockfish 11** for the classical evaluation; **Stockfish 19** for NNUE and
SMP details where Reckless is silent (maintainer decision 2026-09-09; PLAN
rule 1 points here).

- May cross: architecture, mechanisms, population choices, contracts, failure
  modes and constants. A constant is a seed on the donor's scale: converted
  through the measured scale ratio, fitted locally and gated (PLAN rule 2).
- Written by us: code in Rarog's own structure. Line-for-line transcription
  only where an algorithm has one natural form or a different form provably
  loses throughput.
- Read the donor, close the file, then design from Rarog's own code and its
  measured evidence. If a change cannot be justified without pointing at the
  donor, it is not understood well enough to ship.
- `README.md`'s posture stays accurate: an independent engine, with thanks for
  the inspiration.
- Do not merge the `hybrid` oracle, copy its FFI boundary into Rarog, replace
  native Rust with C++/FFI, or read the oracle as permission for an unmeasured
  rewrite.
- Similarity is never a reason to accept anything, and a counter that diverges
  from the oracle is a question, not a defect. Games decide.
- Deciding a donor mechanism does not apply here is a first-class result;
  record it with its reason.

### Adjudication

Every instrument plays games out: the Colosseum run files (`tools/colosseum/`,
which set no draw, resign or move cap), `sprt.ps1`, `gauntlet.ps1`, SPSA on
either path (the `RAROG_ADJUDICATION_PATCH_V4` weather-factory patch is
required to start a tune there) and datagen (`datagen-v2`, or `datagen-v3`
with Syzygy truth for labels, datagen only). Adjudication ends 52.7% of
endgames before they are reached and saves about 10% wall time (RAR-M15,
RAR-M16, RAR-M17, RAR-M18). `-Adjudicate` opts back in only with a registered
reason; its results are not comparable with unadjudicated ones. `datagen-v1`
stays by name so the manifests citing it keep their meaning. Use fixed
movetime or nodes only for deterministic diagnostics.

### Harness

**Colosseum CLI is the main path** for gates, fixed matches, tunes, null pairs
and gauntlets (PLAN B.2.6, maintainer decision 2026-09-21). `tools/colosseum.ps1`
drives it from the committed run files in `tools/colosseum/`, which carry the
conditions every Rarog measurement shares; the cap, the seed and the run
directory stay on the command line, because they belong to the registration in
`EXPERIMENTS.md`. The runner is pinned by revision and SHA-256 in
`tools/colosseum/colosseum.pin.json` and staged by `setup_tools.ps1`; a binary
that is not the pinned one is refused, not substituted. The harness is
qualified in its own repository (Colosseum PLAN Phase 10, with Rarog as the
validation engine) and Rarog repeats none of that qualification.

**fastchess and weather-factory stay installed, working and documented** as the
backup and the second opinion, at least until release 2.5.0. `sprt.ps1`,
`spsa.ps1`, their books, their patches and `setup_tools.ps1`'s staging of them
are maintained, not deprecated; retirement is reviewed at that release and not
before. Both paths call one implementation of every guard
(`tools/harness_common.ps1`), so they cannot come to disagree about what a
measurable binary is.

Run the backup path as a cross-check when:

- the runner, the scheduler or the CPU topology on this host changes — the
  same trigger that owes a null pair (RAR-M03);
- a result is surprising: a sign nobody predicted, a magnitude well outside the
  registered band, or a gate that resolves far faster or slower than RAR-M10
  predicts for its bounds;
- the Colosseum version changes, which means `colosseum.pin.json` was re-pinned.

A cross-check is a fixed match or a replayed gate on the same arms, read as
"do the two instruments agree inside their intervals" and never as a second
chance at acceptance. The two agree at this host's resolution: on the B.2.2
arms over 2,000 games each, fastchess read +54.29 ± 11.12 Elo and Colosseum
+55.71 to +65.92 across three runs of the same seed, a spread as wide as the
gap between the instruments (RAR-M61); on the B.2.4a arms both reached H1 under
`[0,10]` in about the same number of games (RAR-M60).

**A registered experiment names its runner and never changes it mid-way.**
Moving an experiment in flight to the other harness voids it.

Shared conditions on both paths: `3+0.03`, Hash 64, one thread, the UHO book
in random order, no adjudication, a 20 ms margin, and fourteen concurrent
games on pinned physical cores that never include CPU 0. A tune runs fifteen,
because both perturbation arms share a slot. `tools/colosseum/README.md` says
which run file is for what.

### Toolchain and harness notes

`build_test.ps1` manifests bind every test asset to its executable hash, source
tree, compiler, build flavor and benchmark qualification. `colosseum.ps1`,
`sprt.ps1`, `spsa.ps1` and `datagen.ps1` validate those sidecars before launch.
Do not recreate or hand-edit a sidecar to bypass a mismatch; rebuild the asset.
Successful matches additionally reject crashes, time forfeits above the rate
ceiling and protocol failures, and archive hashes of their logs and PGNs.
Measure only on an idle host: `colosseum.ps1` refuses when another engine,
harness or build is running, or when the host is above 15% CPU, and
`-AllowBusyHost` records the waiver in the run's manifest.

If a PGO build dies with "target must match host", the rustup default host has
drifted to windows-gnu, so the pinned toolchain resolves to its gnu variant
and PGO training refuses. `rust-toolchain.toml` pins the channel, not the host
triple, so it cannot catch this — check `rustup show active-toolchain` first.

Games are pinned to physical cores on both paths, because unpinned Zen 3 runs
carry a hidden per-run offset of roughly ±10 nElo (RAR-M48). Colosseum does it
with `--placement auto` and one physical core of headroom; fastchess needs
`-use-affinity` with concurrency 14, and its list never contains CPU 0
(`Get-HarnessGameCpus`), because Windows services most interrupts there. The
two resolve to the same fourteen cores, checked field by field rather than by
eye (`tools/diag/colosseum_parity.py`). fastchess pins one core per game and
starves `Threads>1`, so multi-thread runs on that path drop it. After a change
to the pinned list, `setup_tools.ps1` must repatch weather-factory before an
SPSA launch there, and `spsa.ps1` refuses until it has. The 1T harness is
null-calibrated and shared with Basilisk; a new null pair (the same executable
on both arms, `-Mode calibrate`) is owed only after a runner, scheduler or
topology change, never for a symmetric adjudication toggle (RAR-M03).

NPS work: validate on a self pair first (it must read about 0.00%), pool
several PGO builds per arm because two PGO builds of identical source differ
by about 0.36%, and keep compilation, profiling and unrelated load off the
match host. Roughly 2 Elo per 1% NPS at `3+0.03` — **for SMALL deltas only.**
That figure does not extrapolate: applied to the oracle's 1.80x NPS deficit it
predicts 160 Elo, where the standard ~60 Elo per doubling gives ~51. Above a
few percent, convert through doublings and say which conversion was used.

### Matched ablation (deficit decomposition)

How the deficit was decomposed, and the procedure for every later use.
`analysis/ablation_design.md` holds the reasoning; this is the operation.

One shared bitmask on both engines — 0 razoring, 1 futility-child, 2 nullmove,
3 probcut, 4 iir, 5 shallow-pruning, 6 extensions, 7 lmr — so the same number
ablates the same mechanism on each side. Oracle: tag `oracle/hybrid-ablate`.
Rarog: `--features ablate`, which compiles every guard away in a shipped build.

0. **The harness now refuses to start when an engine does not expose an option
   being set.** fastchess only WARNS and then plays the whole match at the
   DEFAULT, which is a completed run that measures nothing. That happened twice
   here — once from a malformed option name, once from a binary that predated
   the switch. If `sprt.ps1` aborts with "does not expose", rebuild the arm;
   never work around it by dropping the option.
1. **Prove every bit live before trusting any number from it.** Nodes to a
   fixed depth must MOVE for each bit. A guard that reads x1.00 is dead — one
   did, because its anchor landed on the diagnostic `prune_shadow_*` block
   instead of the live site.
2. **Prefer matched CROSS-ENGINE runs to self-play deltas.** Play Rarog against
   the oracle at the SAME mask and read `G(0) − G(mask)` as the Elo that
   mechanism explains. One run per mechanism, one scale, no self-play
   inflation.
3. **Keep the ablated arm inside roughly 20–80%.** Outside it the Elo curve
   saturates: at 6% it runs 30.8 Elo per score point against 6.9 near parity,
   a 4.4x amplification, and a 3-point score difference reads as 105 Elo of
   nothing. Ablating four mechanisms at once collapsed both arms and produced
   exactly that.
4. **~2,000 games is enough.** These are 40–250 Elo effects; stop as soon as
   the intervals separate. Large effects are cheap, which is the whole reason
   this instrument beats gating candidates one at a time.
5. **Mechanisms under ~10 Elo are NOT measurable this way at `3+0.03`.**
   Roughly 10 time forfeits per 3,000 games is worth ~1 Elo, so for razoring
   and IID the noise equals the signal. Use fixed nodes or a longer TC.
6. **Net out NPS before comparing engines.** Rarog runs 1.80x the oracle's
   speed, worth ~51 Elo. It cancels in `G(0) − G(mask)` but not in any absolute
   statement about which search is better.

### Texel convergence procedure

**`analysis/texel_fitting_handbook.md` is the full reference** — resources, the
five-stage pipeline with commands, every tool, the settings and why they are
what they are, the corpus contract gates, and the traps. Read it before running
a fit or building a corpus. The ten rules below are the policy it implements.

Texel is cheap enough to run locally, but its static loss is not a strength
verdict:

1. Mechanically verify label domain and source before fitting. Self-play-WDL
   means exactly `0`, `0.5`, `1`; a filename, float in `[0,1]` or prose summary
   is not proof. Audit independent starts, duplicate games, phase/material and
   mate/decisive coverage, plus the exact named adjudication profile and its
   result cross-tab.
2. Keep stable whole-start hash train/validation/frozen-test splits. Retain the
   rule-50 clock in position identity, reject replay leakage, publish only
   exact per-phase quotas and hash every input/output. Never tune on the frozen
   test set.
3. Enumerate every real `EvalParams` slot and name its fitting instrument,
   algebraic gauge, invariant or measured unidentifiable disposition. Historical
   groups, PSTs and old sparse findings are not frozen. The inventory must sum
   exactly to the registry size.
4. Trace every linear term exactly and verify full evaluation reconstruction.
   Capped/bucket-selecting nonlinear terms require re-evaluation, coordinate or
   finite-difference evidence; trace activation alone does not validate their
   gradient.
5. Record the initial vector, gauge/invariant/free coordinates, activation,
   identifiability and semantic sign/bounds before selecting weights. Inspect
   post-fit covariance/compensation for materially moved families before games.
6. Smoke the complete vector→bake→source→rebuild chain with absurd changes in
   every instrument class. Check native exit codes and require source plus
   fingerprint movement; unchanged behavior after a broad fit is a failed wire.
7. Retain the complete train/validation trajectory. The validation-selected
   vector must be settled, not a transient fly-through or a semantic sign
   violation. Serialize and reload the integer candidate, then report the
   frozen test once after selection using exact full evaluation against an
   explicit saved source vector. A floating optimizer vector or prior-stage
   comparator is not the deployable model.
8. Fit and gate the complete existing linear/nonlinear HCE before adding
   features (done at RAR-E06; every C-phase family cluster repeats the fit). Run a local covariant fit after each later structural cluster.
   Apply registered
   static semantic/loss/NPS filters as refutation only. Bake the fit into
   clean PGO and SPRT the cluster; a lower loss alone accepts nothing.
9. After a structural cluster (PLAN C.3-C.7), rerun the complete instrument schedule.
   Repeat a data cycle only for a prospectively registered
   changed-data hypothesis supported by validation and the baked game verdict.
   Stop at the first no-gain/failed cycle or convergence to the same attractor.
10. Keep search parameters fixed during HCE fitting; remeasure their populations
    at PLAN C.10 rather than co-tuning evaluator and search.

The registered RAR-E06 offline run was one command from a clean worktree, and
is the template for every C-phase refit:

```powershell
pwsh -NoProfile -File tools\texel\fit_complete.ps1
```

It first publishes or hash-verifies the qualified 2,300,000 / 127,778 /
127,778 corpus. It then fixes validation-calibrated K and runs 40-epoch
nonlinear king safety on 200k positions, 200-epoch complete sparse linear Adam
(`lr=0.3`, L2-to-stage-prior `1e-7`), a second nonlinear pass and a 60-epoch
linear polish. The schedule opens the frozen test only at the end. All logs,
vectors, settings, hashes, support/cohort reports, source patch and candidate
binary are retained under `tools/results/hce-fit-<timestamp>/`; source and the
normal release binary are restored. Review those artifacts before applying a
patch or registering games. The command itself supplies no strength verdict.

### SPSA go/no-go procedure

The generic harness is retained. The current roadmap owes SPSA where a
cluster's registration names its live coordinates (PLAN B.2.3, B.6, C.9,
C.10, F.6); each is registered fresh with its surface and horizon. An
undirected broad tune stays forbidden, and HCE and search coordinates are
never mixed in one run.
Before any SPSA:

1. Name the strength-bearing mechanism and show local evidence that its
   consumers are misfit.
2. Estimate plausible Elo and opportunity cost before optimizing schedule
   details. Cancel if the plausible gain is inside the gate's dead zone.
3. Gate categorical switches separately and freeze the winner in both arms.
   Never pin a binary knob as an SPSA constant — a pinned A/B knob is an
   unmeasured assumption.
4. Select continuous coordinates from activation and interaction evidence. Do
   not target a historical coordinate count merely because it exists.
5. For a nontrivial surface, register a bounded sensitivity pilot (default
   128 iterations x 32 games unless signal/budget justifies another size).
   Pilot theta diagnoses sensitivity only: never promote it or use it as the
   full tune's seed.
6. After the pilot, re-audit the entire active interacting surface. A pilot
   coordinate returning to its seed may be inactive; an omitted high-activity
   coordinate can invalidate the proposed full tune. The full tune starts from
   accepted engine defaults.
7. Choose and register the immutable horizon from gradient quality, integer
   resolution and compute budget. `StopAfter` may stage a review without
   changing that horizon or games per iteration.
8. Run `./tools/audit_spsa_coverage.ps1` and register surface, fixed values,
   iterations, games per iteration, slots, the budget in games, gain and
   estimator before launch. On Colosseum the shape is **15 slots and 30 games
   per iteration**, so the budget is `iterations x 30` games (RAR-M62);
   `tools/spsa_config_to_colosseum.py <group> --iterations <N>` converts the
   registered surface for that horizon and `--check` refuses a file that has
   drifted from it.
9. Complete the final theta without post-hoc checkpoint selection; bake it
   into a fresh clean PGO binary and run a paired SPRT, then LTC/4T where
   appropriate.

### Opening book

SPSA and the default SPRT both use `tools/books/UHO_Lichess_4852_v1.epd`,
paired and reversed, at `3+0.03`. That alignment is the point: the optimizer
and the confirmation gate see the same opening and clock distributions. Use a
second book or LTC as an extra robustness check for a mechanism suspected of
condition sensitivity; do not create an unnecessary tuning/confirmation
mismatch.

## Decision rules

- One item open at a time; each candidate gates against the current accepted
  head, never against a stale baseline or another unresolved candidate.
- Categorical architecture is gated before its constants are fitted.
- A touched dormant switch must be removed, kept inert with a named owner, or
  separately gated. It is never activated opportunistically.
- Borderline results are not accumulated as hidden debt. Accept or revert.
- Tune and non-PGO results are diagnostics; final-PGO games decide promotion.
- A correctness exception names the invariant, the tests and the incomplete
  strength evidence.

## Common commands

```powershell
cargo fmt --check
cargo test -p rarog
cargo test -p rarog --release
cargo test -p xtask
# The texel tuner is its own workspace, so `texel` never unifies into the engine.
cargo test --manifest-path tools/texel-tuner/Cargo.toml
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo build --release
"bench" | ./target/release/rarog.exe
cargo xtask build --arch pext --pgo
cargo xtask verify-isa --arch pext
```

```powershell
# MAIN PATH — Colosseum. The gate: [0,3] nElo, the default bracket; -Bracket
# removal | repair | wide for the three registered alternatives. The cap comes
# from RAR-M10 at the EXPECTED value, before any game is played.
./tools/colosseum.ps1 -Mode sprt -EngineA <candidate.exe> -EngineB <baseline.exe> `
  -NameA candidate -NameB baseline -MaxPairs <cap> -Seed <n> `
  -ExpectRevision <sha> -Dir tools/results/<experiment>

# A measurement with an interval, which decides nothing
./tools/colosseum.ps1 -Mode match -EngineA <a.exe> -EngineB <b.exe> `
  -Games 2000 -Seed <n> -Dir tools/results/<name>

# A tune: 15 slots, 30 games per iteration, budget in games (RAR-M62)
./tools/colosseum.ps1 -Mode spsa -Engine <tune.exe> -ConfigGroup <group> `
  -Iterations <N> -TotalGames <N*30> -Seed <n> -Dir tools/results/<experiment>

# Null pair, only after a runner, scheduler or topology change (RAR-M03)
./tools/colosseum.ps1 -Mode calibrate -EngineA <same.exe> -EngineB <same.exe> `
  -Seed <n> -Dir tools/results/<name>
```

```powershell
# BACKUP PATH — fastchess and weather-factory, kept working until at least
# 2.5.0. Use it for a cross-check on the triggers in "Harness", and say in the
# registration which runner a result came from.
# [3,10] is the fastchess wrapper's default and is WRONG for a small candidate:
# wide bounds anchored high drive a true +4 to H0.
./tools/sprt.ps1 -EngineA <candidate.exe> -EngineB <baseline.exe> `
  -NameA candidate -NameB baseline -Elo0 0 -Elo1 3 -MaxGames 80000
./tools/spsa.ps1 -ConfigGroup <group> -EngineSuffix <s> -Iterations <N>
```

```powershell
# Test/tune binaries, the SPSA coverage audit, and the harness's own checks
./tools/build_test.ps1 -Suffix <s>
./tools/audit_spsa_coverage.ps1
pwsh -NoProfile -File tools/diag/test_colosseum_guards.ps1
python -m unittest discover -s tools/diag -p "test_colosseum_parity.py"
```
