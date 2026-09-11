# Rarog recurring procedures

Agent-facing process detail. Split out of `GUIDE.md` on 2026-08-21.
`AGENTS.md` holds the rules that stop wrong results; this holds the
step-by-step procedures those rules assume.

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
   EXPECTED value before choosing, and use the PLAN §2 sizing table.
3. **Implement** — the smallest dependency-complete cluster. Substeps may be
   compiled and diagnosed separately, but are not expected to pass standalone
   and no incomplete cluster becomes the next strength baseline.
4. **Prove correctness** — fmt, workspace tests in debug and release,
   all-feature clippy and targeted invariants. A behavior-neutral diagnostic
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

Rarog takes **ideas** from Stockfish and builds its own answer. It does not
take code, and it does not aim to resemble it. Both engines are GPLv3, so
copying would be legally permissible — this boundary is a product decision and
is deliberately stricter than the licence requires. PLAN §4 holds the full
table; the working rules are:

- What may cross: the problem a mechanism solves, that the problem exists at
  all, which mechanisms interact and in what order, which populations are
  worth measuring, and known failure modes.
- What may not cross: source code in any language or amount, line-by-line or
  structure-for-structure transcription, copied identifiers or file layout,
  and behavioral equivalence as a goal. A donor constant may cross only as an
  explicitly labelled seed under PLAN's current independence rule; it is on
  the donor's scale and must be locally fitted and gated before acceptance.
- Read, understand, close the file, then design from Rarog's own code and its
  measured evidence. If a change cannot be justified without pointing at the reference,
  it is not understood well enough to ship.
- No upstream code is copied, so Rarog is not a derivative work. `README.md`
  already states the correct posture — an independent engine, with thanks for
  the inspiration. Do not restyle that into an attribution of derived code.
- Do not merge the `hybrid` branch, copy its FFI boundary into Rarog, replace
  native Rust with C++/FFI, or read the oracle as permission for a wholesale
  unmeasured rewrite.
- Similarity is never a reason to accept anything, and a counter that diverges
  from the oracle is a question, not a defect. Closing a counter gap is not an
  outcome; winning games is.
- Rarog solving a problem differently, or deciding it does not apply here, is
  a first-class result — record it with its reason and move on.

**Adjudication is off by default as of 2026-09-01** (RAR-M16), in `sprt.ps1`
and `gauntlet.ps1`. It used to be kept for search-only candidates on the
grounds that both arms share Rarog's score scale, which is true but was never
the whole cost: RAR-M15 measured adjudication destroying **52.7% of all
endgames before they are reached**, and RAR-M16 priced playing games out at
only about **10% wall time** (97.5 games/min against 88.4). RAR-O01 versus
RAR-O02 priced the cross-evaluator confounder at about 74 Elo separately.

Adjudication is not unfair -- it is symmetric between arms -- it is **lossy**,
and the loss scales with how badly the engine converts. An engine that
converts KRP-KR at 52% disagrees with its own adjudicated verdict far more
often than one converting at 99%, which is the argument for revisiting this
default once the endgame cluster (PLAN C.5) closes rather than treating it as
permanent.

Pass `-Adjudicate` to opt back in, and justify it in the registration: wall
time genuinely binding, and a change that provably cannot touch conversion or
defensive holding. A result produced with the flag is not comparable with one
produced without it.

**This covers every instrument, SPSA and datagen included.** `setup_tools.ps1`
now strips both the resign and the draw line from weather-factory's
`cutechess.py` (marker `RAROG_ADJUDICATION_PATCH_V4`) and `spsa.ps1` refuses
to START a tune without it -- while still exempting a RESUME, because a run
that began under an older rule must finish under it rather than becoming
incomparable with itself halfway through.

`datagen.ps1` defaults to the new `datagen-v2` profile: no adjudication at
all. The case there is stronger than for a gate and is not about mislabeling
-- resign at 600/3 two-sided almost never calls a game wrong. It is **sample
depletion**: adjudication ends 52.7% of endgames before they are reached, so
an adjudicated corpus is systematically short of exactly the positions the
endgame families must be fitted on, and the phase-balanced extraction then
draws its endgame reservoir from a truncated distribution. `datagen-v1` is
retained by name and unedited, because `hce-v2` and every manifest already
written cite it and must keep meaning what they said. Pass `-Adjudicate` to
reproduce it.

Historical note, superseded: an HCE A/B used to require a registered
calibration proving adjudication safe for both
arms. Use fixed movetime or nodes only for the deterministic diagnostic suite,
never as the strength verdict.

### Toolchain and harness notes

`build_test.ps1` manifests bind every test asset to its executable hash, source
tree, compiler, build flavor and benchmark qualification. `sprt.ps1`,
`spsa.ps1` and `datagen.ps1` validate those sidecars before launch. Do not
recreate or hand-edit a sidecar to bypass a mismatch; rebuild the asset.
Successful matches additionally reject crashes, time forfeits and protocol
failures, and archive hashes of their logs/PGNs.

If a PGO build dies with "target must match host", the rustup default host has
drifted to windows-gnu, so the pinned toolchain resolves to its gnu variant
and PGO training refuses. `rust-toolchain.toml` pins the channel, not the host
triple, so it cannot catch this — check `rustup show active-toolchain` first.

`fastchess -use-affinity` with concurrency 14 is mandatory for 1T gates;
unpinned Zen 3 runs carry a hidden per-run offset of roughly ±10 nElo. It pins
one core per game and starves `Threads>1`, so drop it for multi-thread runs
and re-calibrate the null pair under that configuration. Validate any harness
change on a null pair — the same executable on both arms — before trusting a
verdict.

NPS work: validate on a self pair first (it must read about 0.00%), pool
several PGO builds per arm because two PGO builds of identical source differ
by about 0.36%, and keep compilation, profiling and unrelated load off the
match host. Roughly 2 Elo per 1% NPS at `3+0.03` — **for SMALL deltas only.**
That figure does not extrapolate: applied to the oracle's 1.80x NPS deficit it
predicts 160 Elo, where the standard ~60 Elo per doubling gives ~51. Above a
few percent, convert through doublings and say which conversion was used.

### Matched ablation (the Phase-4 measurement instrument)

How the deficit was decomposed, and the procedure for every later use.
`analysis/ablation_design.md` holds the reasoning; this is the operation.

One shared bitmask on both engines — 0 razoring, 1 futility-child, 2 nullmove,
3 probcut, 4 iir, 5 shallow-pruning, 6 extensions, 7 lmr — so the same number
ablates the same mechanism on each side. Oracle: branch `hybrid-ablate`.
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
   iterations, games, gain and estimator before launch.
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

### CPU compatibility design

There is deliberately no startup CPU guard inside specialized assets. When the
compiler is told that BMI2/AVX2/FMA are mandatory, ordinary feature-detection
macros fold those checks to true and the guard is removed. A working
in-process guard would require baseline-compiled CPUID code to execute before
specialized code, adding a separate dispatch boundary. The current design is
close to the specialized-binary model: users choose `x86-64`, `avx2`, `pext`
or `arm64`, the README states exact requirements, and release tooling
disassembles each asset to enforce the promise. If a single universal binary
becomes a product goal, 8.1 may add a Stockfish-style baseline dispatcher.

### Experiment discipline

- Begin from a clean revision and record both binary hashes.
- Register hypothesis, interactions, gate, stop rule and budget before games.
- Treat tune and non-PGO results as diagnostics unless the experiment says
  otherwise; final-PGO games decide promotion.
- Do not turn node reduction into Elo. Use diagnostics to explain a game
  result.
- Record rejected and neutral outcomes in `EXPERIMENTS.md`; never silently
  rewrite them into a later success story.
- A correctness exception must name the invariant, the tests and the
  incomplete strength evidence honestly.

## Decision rules

- One item open at a time; each candidate gates against the current accepted
  head, never against a stale baseline or another unresolved candidate.
- Categorical architecture is gated before its constants are fitted.
- A touched dormant switch must be removed, kept inert with a named owner, or
  separately gated. It is never activated opportunistically.
- Borderline results are not accumulated as hidden debt. Accept or revert.
- Commit after each finished and verified step, and keep tooling changes in
  separate commits from engine changes.
- Mirror any status or number change into **both `GUIDE.md` and `PLAN.md` in
  the same commit**. `HISTORY.md` is history and is not updated for new work.

## Common commands

```powershell
cargo fmt --check
# `-p rarog`, NEVER `--workspace`, for the engine suite: texel-tuner depends on
# rarog with `features = ["texel"]`, so a workspace test run unifies features
# and silently tests an engine with the eval and pawn caches bypassed. At the
# current head that also FAILS -- the LazyMargin regression test asserts that
# two lazy margins give different evals, and under `texel` lazy eval is off,
# so both give the same one. ci.yml has split these since it was written.
cargo test -p rarog --all-targets
cargo test -p rarog --all-targets --release
cargo test -p xtask -p texel-tuner
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo build --release
"bench" | ./target/release/rarog.exe
cargo xtask build --arch pext --pgo
cargo xtask verify-isa --arch pext
```

```powershell
# Primary SPRT [0,3] nElo — the DEFAULT bracket. Add -TC "10+0.1" for LTC.
# [3,10] is the harness default and is WRONG for a small candidate: wide bounds
# anchored high drive a true +4 to H0. Size from RAR-M10 before registering.
./tools/sprt.ps1 -EngineA <candidate.exe> -EngineB <baseline.exe> `
  -NameA candidate -NameB baseline -Elo0 0 -Elo1 3 -MaxGames 80000

# Harness calibration after any runner change — same binary on both sides
./tools/sprt.ps1 -EngineA <same.exe> -EngineB <same.exe> -NameA a -NameB b

# Test/tune binaries and the SPSA coverage audit
./tools/build_test.ps1 -Suffix <s>
./tools/audit_spsa_coverage.ps1
```
