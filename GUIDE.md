# Rarog Development Workflow Guide

This is the short operational view: where Rarog stands, what is running, and
what you need to do next. Detailed rationale, experiment design, history and
all phase items live in [`PLAN.md`](PLAN.md).

**This file and `PLAN.md` are the maintainer-facing pair.** `README.md`,
`CHANGELOG.md` and the GitHub release notes are user-facing and must stay free
of method, history and internal naming — see PLAN §"Documentation audiences".

## Current checkpoint

| | |
|---|---|
| Branch / version | `development` carries Phase 10; `master`/`v2.3.1` = `a5fd288`. **2.3.1 RELEASED**, tagged and pushed. |
| Accepted baseline | Post-refit search + zero-reduction LMR + frozen Texel refresh; fingerprint **6,502,902 / EBF 2.449**. Clean gate binary: `rarog-p1043-base-pext-pgo.exe`. |
| Working source | Evaluator frozen. Accepted search remains in place; Phase 10 now repairs evidence flow and cross-feature cooperation before any final search refit. |
| Last accepted strength results | Texel refresh **+11.56 ±5.19 Elo**, zero-reduction LMR **+9.13 ±5.45 nElo**, larger search refit **+15.33 ±7.34 nElo**. These are mechanism gates, not proof that their Elo adds in the current dev binary. |
| Live rating snapshot | At **5,643/36,400** games: `2.4.0-dev` 2955 vs 2.3.1 2946. Gaps from dev: Basilisk 1.9.3 +54; Rybka 4.1/4/5/6 +127/+147/+183/+207; Critter 1.6a +226; Houdini 1.5a +252. This is provisional and the dev binary includes interim unfinished aspiration constants. Houdini 2.0c and Fritz 16 still require direct tests. |
| Current work | ▶ The Rating Tournament and 5,000-iteration aspiration SPSA are both active. Let both finish unchanged; run no bench, NPS, PGO, SPRT or other games while they occupy the machine. |
| Next release | **2.4.0 at 10.12.** It requires a clean cumulative candidate whose lower confidence bound beats all Rybkas, Critter 1.6a, Houdini 2.0c and Fritz 16, plus 1T-LTC/4T transfer and release gates. NNUE remains Phase 12; Phase 10 does no further HCE work. |

## Project mandate — surface weaknesses, do not work around them silently

The goal is the **strongest possible chess engine**, not compliance with
current limitations. Any model working on Rarog must explicitly tell the user
when it sees something sub-optimal, wrong, weak, badly implemented, or blocked
by a constraint we could plausibly remove.

Before accepting the constraint, cutting scope around it, parking the issue, or
building a workaround, report four things: evidence (and whether proven or
hypothetical), expected strength/quality upside, cost/risk, and the best direct
fix with alternatives. Then the user and model decide together: fix now, test
first, defer deliberately, or reject. EV gates and SPRT still control compute
and shipping; they do **not** excuse withholding an improvement opportunity.

Tuned-off features are preserved through NNUE. A zero/off value describes the
current HCE optimum; it does not authorize deleting the implementation, UCI
option, or SPSA seat. The post-NNUE retune may reactivate it, and only then may
removal be discussed explicitly.

## Forward tracker

<!-- FORMATTING RULES for this tracker — follow them, they get broken often:
     1. ONE step per `- [ ]` bullet. Never join two steps on one line with
        "·" (e.g. "9.4 foo · 9.5 bar") — each gets its own bullet, always.
     2. Continuation lines indent 6 spaces so they align under the text after
        "- [ ] ". Sub-items indent 2 spaces then their own "- [ ]".
     3. Status boxes: `[ ]` todo · `[~]` ONLY while genuinely in flight (a
        gate running right now) · `[x]` finished — accepted, rejected,
        deferred or parked. Anything resolved is `[x]`, never `[~]`, so the
        renderer strikes it through and the eye skips it.
     4. Every item opens with its STEP NUMBER, then (for `[x]` items) a
        BRACKETED OUTCOME TAG in bold, so the reader orients by number
        first and reads the result second (user rule, 2026-07-27 — number
        BEFORE tag, never the reverse):
            - [x] 8.1 **[ACCEPTED +22.13 ± 7.28, LOS 100%]** History split ...
            - [x] 8.1b **[REJECTED −6.6]** no-aging retry — ...
            - [x] (b) **[DEFERRED → 8.9]** LMR of late evasions — ...
            - [x] 7.3 **[PARKED → Phase 14]** rule-50 TT key ...
            - [x] 7.6 **[DONE, no games]** Diagnostics counters ...
        Tags: ACCEPTED <elo> · REJECTED <elo> · DEFERRED → <item> ·
        PARKED → <phase> · DONE · FIXED. Put the Elo in the tag, detail after.
     5. Bullet order: step number, outcome tag, short title, then detail.
     6. Never renumber historical items because commits reference them. Phase
        10 is the explicit exception: the user authorized its complete rewrite
        and 10.0–10.12 supersede its old forward numbering as of 2026-08-05.
     7. Mirror any status/number change into PLAN.md in the same commit.
     8. Blank line AFTER the `###` phase heading, then NO BLANK LINES between
        bullets at all. The tracker is ONE continuous list per
        phase; a blank line splits it into separate lists and the renderer
        re-spaces everything around it (this is what made 9.5 "not show on its
        own line" once already).
     9. ONLY NUMBERED STEPS live in the tracker. A recurring procedure, a
        checklist or a reference note is NOT a step — it never gets ticked, so
        an unticked `- [ ]` for it reads as outstanding work forever. Those go
        in `## Recurring procedures`, and the owning step links to them.
    10. Wrap at ~76 columns, like every other bullet here. Do not let one
        bullet run to 100+ columns because the sentence "felt continuous".
    11. SUB-ITEM INDENT IS **4 SPACES**, not 2 — with 10-space continuations
        (aligning under the sub-item's text). This is the rule that kept
        getting broken: at 2 spaces the renderer does NOT nest them, and the
        sub-steps display as top-level siblings of the next numbered item, so
        a reader cannot tell which parent they belong to. Shape:
            - [~] **10.3 Parent item**
                  6-space continuation of the parent
              - [x] **(1) sub-item** with its own status box
                    10-space continuation of the sub-item
        Quick check after editing: every sub-item line must match `^    - \[`
        and every one of its continuation lines `^          \S`. Never a bare
        `    - ` bullet — the box is what makes the state scannable, and never
        a blank line between them (rule 8), which is also what keeps the
        continuations safe from being parsed as code blocks. -->

The model implements and locally verifies; you run only the long game jobs.
Each candidate gates against the then-current accepted head.
**Restructured 2026-07-15** (lesson 15): a fix ships either *free*
(bench-identical) or *bundled with the re-tune of the constants it feeds* —
one gate per bundle. Macro-order: **A** correctness bundles (Phase 7) → **B**
strength (8 → 9 release → 10) → **C** NNUE (11 infra prep → 12 training) →
**D** contingent HCE fallback (13, only if NNUE fails). Current queue:
**Phase 7 ✅ → Phase 8 ✅ → Phase 9 / 2.3.1 ✅ → accepted Phase-10 search
baseline ✅ → 10.0 live tournament + aspiration SPSA ▶ → 10.1 evidence
diagnostics → 10.2–10.7 search cooperation → 10.8 root confidence → 10.9
throughput → 10.10 final search refit → 10.11 cumulative matrix → 10.12
target ladder / 2.4.0.** Phase-10 forward numbering was explicitly rewritten
on 2026-08-05; older numbers remain only in the historical result record.
Full rationale per item: `PLAN.md` §S6.

### Phase 7 — Correctness repairs — ✅ COMPLETE

Aspiration hang fixed, SEE pin-awareness bundle **+1.47**, HCE semantics flat
(**−0.31**, correctness banked), TM falling-eval **+2.85 LTC**, diag counters
added. Draw-semantics rework **rejected −7.2 / −11.9** and reverted.
**Worth:** ~+4 Elo, but the point was banked correctness — it removed the class
of bugs that made later measurements untrustworthy.

### Phase 8 — Search wave — ✅ COMPLETE

History bonus/malus split **+22.13**, in-check selectivity **+30.75**, history
bundle **+6.01**, index hoist **+0.98% NPS**, SMP rework **+102.78 @4T**.
Rejected and reverted: no-aging −6.6, cutoffCnt LMR −7.78, do-deeper −7.29,
mop-up gating ≈−5.4, fail-soft qsearch −5.96, correction-history bundle (wash,
+1.43 after its guard bug cost −56).
**Worth:** ≈ +60 Elo at 1T plus +102.78 at 4T. Half the wave H0'd, as planned.

### Phase 9 — Reproducible builds, CI, shipped PGO, clean code

9.0–9.7 landed the lint wall, pinned toolchain, CI, PGO release builds, test
hardening and build manifests — **cost ≈ −2 to −3 Elo** (measured twice
independently) and bought two real bugs plus the NNUE-ready structure.
9.7.5 (SMP quality wave II) returned **net zero Elo** across twelve sub-items
but established the two facts that now steer threading work: SMP coordination
and diversification are **saturated** (the shared TT couples the threads no
matter what), while **time allocation is a live ~16 Elo lever** — which points
the next SMP investment at 10.2's root-informed TM, not at more diversification.
**9.8 boundary gauntlet ✅ COMPLETE — the cycle validated.** Four Colosseum
conditions, ~19,000 games, one anchor (Rybka 4), everything else free, **zero
time forfeits anywhere** (which discharges the LTC 4T sanity owed since 8.13).
2.3.0 over 2.2.0: **+76 ± 21** at 1T 3+0.03, **+78 ± 28** at 1T 10+0.1,
**+194 ± 24** at 4T 10+0.1. Self-play predicted ~+60 at 1T and two independent
conditions returned +76/+78 — **the gains transfer**, which is exactly what 9.8
exists to check. The 4T figure decomposes as the 1T gain plus ~+100 from 8.13's
SMP rework, so that result is externally confirmed too.

Full per-item detail for all three phases is in git history and PLAN §S5.

**1T NPS confirmation ✅ DISCHARGED 2026-07-28: +0.99% (24 cycles) and
+1.56% (40 cycles, CI +0.90…+2.13) for HEAD vs the pre-9.7.5 tree**, self-pair
validated at +0.10% first, 2 pooled PGO builds per arm, every base build below
every candidate build in both passes. Both trees bench 5,173,540, so this is
pure execution speed — **≈ +2 to +3 Elo at 1T, no regression.**


### Phase 10 — Evidence-coherent search and target ladder (→ 2.4.0)

Evaluator work is frozen. The accepted search refit, zero-reduction LMR,
`RootMove` infrastructure and Texel refresh form the clean 6,502,902 baseline.
The rewritten phase fixes how search evidence moves between features, connects
the root signals already collected, performs one final search-only refit, then
requires direct wins over the full target ladder. Full rationale, contracts and
gates are in PLAN §10.

- [~] 10.0 **Close and freeze the live experiments.** Finish the 36,400-game
      Rating Tournament and the registered 5,000-iteration aspiration SPSA
      unchanged. Archive configs, binaries, PGNs, trajectory and final theta;
      discard the interim dev vector as evidence, bake/gate the predeclared
      final estimator once, and restore a clean reproducible baseline.
- [ ] 10.1 **Diagnostic substrate and interaction map.** Add bench-neutral
      provenance/consumer, pruning-overlap, history-attribution, qsearch,
      extension-debt and root-confidence traces plus shadow predicates. Produce
      a fixed-corpus interaction report before changing behaviour.
- [ ] 10.2 **Evidence model and TT consumer contracts.** Introduce transient
      `OutcomeKind`/`NodeEvidence`/`MoveEvidence`; prototype compact TT
      provenance without growing the 10-byte entry, and define exactly which
      sources/bounds/depths each pruning or extension consumer may trust.
- [ ] 10.3 **Qsearch, ProbCut and TT hygiene.** Separate raw eval, stand pat and
      searched scores; stop depth-0/pruning estimates from posing as searched
      main-search evidence; store actual ProbCut scores with speculative
      provenance and forbid them from authorizing singular extension.
- [ ] 10.4 **NMP, IIR and singular cooperation.** Add subtree null suppression,
      node/eval/decisive-score and potential-singularity guards; keep IIR off
      PV-following nodes; constrain singularity by trustworthy evidence and
      extension debt; replace the global `tt_pv` veto with mechanism-specific
      predicates. Test isolated arms and a registered joint bundle.
- [ ] 10.5 **Correction/history attribution.** Implement true continuation
      pairs, prevent capture/speculative/aborted results from training quiet
      correction, then ablate threat, noisy/quiet, check/evasion and halfmove
      contexts. Any larger table needs residual-quality and cache/NPS evidence.
- [ ] 10.6 **Unified prospective-depth selectivity.** Give LMP, futility, SEE
      pruning, LMR and re-search one ordered `MoveEvidence`/`lmrDepth` pipeline;
      preserve the accepted reduction floor, classify quiet checks, resolve
      the stale late-evasion `!in_check` mismatch and measure pruning overlap.
- [ ] 10.7 **Qsearch as a first-class search.** After TT hygiene, test staged
      qsearch move ordering/history, check/evasion safety and coherent
      delta/futility/SEE pruning. Compare fixed-node decision quality and
      clock-time Elo; profile hot paths only after behaviour is accepted.
- [ ] 10.8 **One root-confidence model.** Make aspiration, TM, legal fallback
      and SMP timing consume the same completed-iteration variance, score-gap,
      stability, effort, fail and worker-change signals. Never publish partial
      decisive scores; confirm at 1T STC/LTC and 4T LTC with zero forfeits.
- [ ] 10.9 **Parallel and throughput pass.** Profile the accepted search at
      1/2/4/8 threads; audit TT layout/contention after provenance changes and
      apply behaviour-identical hot-path work through the interleaved NPS
      protocol. Do not reopen generic worker-diversification experiments.
- [ ] 10.10 **Final joint search refit.** With mechanisms frozen, fit coupled
      evidence/selectivity, history/correction, qsearch, NMP-family and root
      groups, including registered interaction revisits. No HCE coordinates;
      bake once and pass a clean `[0,3]` gate against the pre-fit head.
- [ ] 10.11 **Cumulative checkpoint.** Reproduce the release build, pass all
      correctness suites, beat the frozen 10.0 baseline at 1T STC/LTC, transfer
      at 4T with zero forfeits, run major-subsystem ablations and clear all
      provenance, extension, saturation, aspiration and timer telemetry gates.
- [ ] 10.12 **[RELEASE GATE] Target ladder and 2.4.0.** In a locked direct
      tournament, its Holm-adjusted 95% paired lower bound must beat Basilisk
      1.9.3, every Rybka, Critter 1.6a, Houdini 2.0c and Fritz 16; confirm 1T
      LTC/4T transfer, complete clean build/docs/version/tag/archive work, then
      release 2.4.0. Rating-list inference cannot replace a missing opponent.

### Phase 11 — NNUE infra prep (bench-identical per step; no games)

- [ ] 11.0 Clean-code P3 "structure era" (scheduled here so board surgery
      never mixes with the Phase-8/10 strength wave): `history` module
      extraction → per-ply `PlyContext` (twin of 11.2's StateInfo, the shape
      11.4 wants) → `rarog-core` workspace crate split → `TimeBudget`
      retiring the TM `f64::INFINITY` sentinels (the package's one
      SPRT-class change, `[-3,0]`) → `Move`/`Square` private fields +
      `const fn` constructors (from 9.0a(vii): 56 hot construction sites, so
      it rides 11.0's rewrite of that same code instead of churning it twice)
- [ ] 11.1 Frozen diagnostic corpus + residual harness (pullable forward into
      SPRT downtime)
- [ ] 11.2 Per-ply `StateInfo` (keys/castling/EP/rule50/checkers/pins)
- [ ] 11.3 Dirty-piece delta contract + make/unmake equivalence walks —
      **design resolved 2026-07-22: adopt Reckless's `BoardObserver` shape**
      (analysis in PLAN Phase-11 item 3). Deliberately NOT pulled into
      Phase 8: HCE gains nothing from it — 8.12(a)'s three scalars are
      cheapest updated inline in `make_move`, no trait needed — so the
      plumbing waits for its real consumer (the net) per the phase
      discipline.
- [ ] 11.4 Per-thread accumulator scaffolding (lives with the worker, not the
      copyable `Board`; HCE still runs through `Evaluator::eval()`)
- [ ] 11.5 Threat-map hooks (optional)

### Phase 12 — NNUE program via `net_trainer` (→ 2.5.0)

- [ ] 12.1 Contract bring-up (chess768→H×2→8 buckets); conformance vectors
      integer-exact; hard NPS gate before games
- [ ] 12.1a Seed-book design from 10.4.3(a2)'s measured yield matrix, BEFORE
      generating the 30–60M corpus. Solve the seed allocation instead of
      assuming a balanced book gives a balanced harvest; include an explicit
      floor of independently seeded games per phase, since positions from one
      game are correlated. At Texel scale the mismatch costs ~3.6× the games
      the smallest quota needs; at NNUE scale that is a 2–3× multiplier on the
      most expensive data step in the project. Bucket count (3 vs 5) is not the
      lever — it changes reporting, not the traversal asymmetry
- [ ] 12.2 Data through net_trainer's pipeline (datagen/extract/convert; λ on
      validation)
- [ ] 12.3 King-conditioned net (v2 king buckets) — `[0,3]` vs stage A
- [ ] 12.4 Material/PSQT path (only if residuals demand) · 12.5 relation inputs
      (threat/pawn pairs, stages D/E)
- [ ] 12.6 Data/size scaling flywheel (fresh on-policy data per stronger net)
- [ ] 12.7 Search recalibration — refit correction weights + one joint
      cp-margin re-SPSA; **shelved correctness fixes (7.2/7.3) re-enter here**
- [ ] **▶ 12.8 RELEASE 2.5.0 — acceptance gate (conformance + NPS + cohorts +
      STC/LTC SPRT + 1/2/4/8-thread) then YOU gauntlet + publish** (first net)

### Phase 13 — Contingent HCE deepening (ONLY if the NNUE program fails/stalls)

Each item = structure/fix + refit bundle, one gate (lesson 15). NNUE-subsumed,
so it may never run.

- [ ] 13.1 King-safety semantic rework (largest classical family) · 13.2
      winnability/material scaling · 13.3 passer/pawn conditionality
- [ ] 13.4 threat conditionality · 13.5 broad positional repairs · 13.6
      material/phase specialization · 13.7 lazy-margin conditioning
- [ ] 13.8 OCB material-scope (moved from 7.4c) — cheap/high-confidence, the
      natural first item here

### Phase 14 — Parked (enter on demand)

- [ ] rule-50 TT key (was 7.3) · SMP cluster (vote merge, ordering jitter,
      staggered helper depths) · large-page/NUMA TT · AVX-512/ARM64 ·
      Chess960 · OpenBench/distributed testing

## Recurring procedures

Not steps — they are never "done". The tracker links here.

### Toolchain bump

Do this every time you move rustc. NEVER between building an SPRT baseline
and its candidate.

- [ ] `rustup update stable`, then read the new version out of `rustc -vV`.
- [ ] Put that exact version in the `channel` line of
      [rust-toolchain.toml](rust-toolchain.toml).
- [ ] `cargo update` to refresh the lock to the newest compatible deps.
- [ ] `cargo clean; cargo build --release` — a CLEAN build; cached artifacts
      hide the bump.
- [ ] Verify bench reports the SAME node count as immediately before the bump.
      Record it first: it is not a frozen literal, it changes on every
      behavioural commit. This is a UB CANARY, not a formality — node count is
      deterministic integer logic, so a compiler change cannot legitimately
      move it. If it moves, we have undefined behaviour or a miscompile in one
      of the 17 remaining `unsafe` blocks. STOP and investigate.
- [ ] `cargo clippy --release --all-targets` must be zero. A new compiler can
      ship new lints, and the deny wall makes them build failures.
- [ ] Re-baseline NPS (`cargo xtask build --arch pext --native --pgo`, then the usual
      interleaved bench) and RECORD the delta. That is the point of pinning:
      compiler gains become measured, dated events instead of invisible drift
      contaminating the next gate.
- [ ] Re-test **Windows ARM64 PGO without the LLD workaround** on every
      toolchain bump. Temporarily remove `linker_flags` in
      `xtask/src/main.rs` and dispatch the build workflow. If profiles merge,
      rust-lang/rust#156675 is fixed: remove the override permanently. If
      `llvm-profdata` still reports an empty symbol name, restore it. All nine
      assets remain profile-guided either way.

### Running an SPSA (weather-factory)

Mechanics verified from the source 2026-07-23. Read this before judging any
tune "converged" — two of these were got wrong on 8.4's first night.

- [ ] **`state.t` is stored in games, but the patched schedule runs in
      ITERATIONS.** `setup_tools.ps1` changes the schedule input to
      `it = self.t / self.cutechess.games`; `a_t` and `c_t` consume `it`.
      `-ShowValues` performs the same conversion. Quote and reason in
      iterations; the stored game counter exists only for compatibility with
      old state files.
- [ ] **`A` is correctly 10% of the first-launch horizon.** For this 5,000-run,
      `A=500`, and `spsa.ps1` reads the JSON back and asserts it before launch.
      A PowerShell case-insensitivity bug briefly wrote the gain `a` into `A`,
      but the setup-only dry run caught it before this parameterization's first
      tune. The old “A is in the wrong units” warning no longer applies.
- [ ] **Convergence test that works:** parse the per-iteration parameter
      blocks out of the log, split the run in thirds, and compare each
      parameter's MEAN over the 2nd vs 3rd third, normalised to its
      `[min,max]` range. Under ~2% of range = settled; over ~5% = still
      moving. Single-iteration values are far too noisy to eyeball — SPSA
      wanders even at low gain.
- [ ] **A parameter that decays back toward its seed and stays there has been
      REJECTED by the tuner** — that is a result, not a failure to converge
      (8.4's `TtCutoffBonusPct`: spiked to 27, then sat at 1–5 for 900
      iterations).
- [ ] **Watch for bound pinning** — a value parked within ~2% of `min` or
      `max` means the RANGE was wrong, not that the value converged. Widen
      once and restart fresh (`-Resume` keeps the old ranges).
- [ ] **Stop/resume is safe and continues the schedule.** `main.py` reloads
      `state.json` (values + spsa params + `t`), so annealing resumes rather
      than restarting at full gain. A `finally:` block saves on Ctrl-C, so a
      clean stop loses NOTHING; state is also written every `save_rate` (10)
      iterations, so even a hard kill costs ≤10 iterations.
- [ ] **Resume with `-LaunchOnly`** — it skips setup entirely, so nothing can
      archive the state:
      `./tools/spsa.ps1 -ConfigGroup <group> -LaunchOnly`
      NEVER resume with the plain setup+launch form: without `-Resume` it
      archives `state.json` and silently restarts from the seeds. The launch
      path re-checks the affinity patch and throws if `cutechess.json`
      concurrency ≠ the resolved concurrency.
- [ ] **The machine is fully occupied while a tune runs** (concurrency 14 of
      16 physical cores). No NPS work, no SPRT, no bench measurement until it
      stops — see the 10.3 NPS protocol on why a busy machine invalidates
      those.

## What you run now

Both current jobs are **observational until they finish**. Do not change their
opponent set, openings, adjudication, TC, concurrency, seeds, SPSA bounds or
stopping rules, and do not bake another interim theta into source.

| Job | Do now | Completion evidence |
|---|---|---|
| Rating Tournament | Let all **36,400 games** finish unchanged. | Final standings, full PGN, engine binaries/manifests, tournament JSON/config, openings and exact UCI/TC/thread/hash settings. |
| Aspiration SPSA | Let the registered **5,000 iterations** finish unchanged. If interrupted, resume the saved state with `./tools/spsa.ps1 -ConfigGroup aspiration -LaunchOnly -Iterations 5000`. | `state.json`, seed/config, full trajectory, logs, final theta and the predeclared estimator result. |

The machine is occupied at concurrency 14 of 16 physical cores. Until both
jobs stop, run **no bench, NPS comparison, PGO build, SPRT, gauntlet, datagen or
other game workload**. Those results would be contaminated and could disturb
the registered jobs.

The current `Rarog 2.4.0-dev` is not a release or gate candidate: it combines
the accepted development head with constants sampled from an unfinished SPSA.
Its current +9 pool Elo over 2.3.1 can be the expected improvement, noise, an
interim-constant effect or a mixture. The finished tournament provides a
cumulative observation; only the post-SPSA clean A/B decides whether the
aspiration vector is accepted.

When both jobs finish, paste/export their final artifacts. The model will:

1. apply the predeclared SPSA estimator and bake it once;
2. restore/reproduce the clean **6,502,902 / EBF 2.449** baseline;
3. build a clean PGO candidate and prepare the registered aspiration gate for
   you to run;
4. update the rating-ladder baseline without attributing unproven Elo; and
5. begin 10.1 diagnostics. No new long game job is requested before that review.

### Accepted binaries retained for attribution

| binary | bench | role |
|---|--:|---|
| `rarog-p1043-base-pext-pgo.exe` | 6,502,902 | clean accepted Phase-10 baseline and next gate reference |
| `rarog-p102a-tune.exe` | 6,502,902 | active seven-coordinate aspiration tuner |
| `rarog-p1025a-zero-pext-pgo.exe` | 6,718,158 | accepted zero-reduction-LMR historical arm |
| `rarog-p1046a-theta-pext-pgo.exe` | 6,477,102 | accepted broad-search-refit historical arm |
| `rarog-p100-base-pext-pgo.exe` | 5,173,540 | pre-refit 2.3.1 comparison baseline |

## Working rhythm

```text
You   -> Ask for the next plan item or paste a completed SPSA/SPRT result.
Model -> Implements or bakes/reverts, verifies, updates PLAN + this guide,
         commits, and gives exact commands.
You   -> Run long SPSA/SPRT/gauntlet/datagen jobs and paste the result.
```

You are the only one who runs games.

## Decision rules

| Situation | Action |
|---|---|
| A weakness or plausibly removable constraint is discovered | Surface evidence/upside/cost/direct fix immediately; decide together before working around or accepting it |
| SPRT passes its registered hypothesis | Keep, record, use as the new head |
| Strength candidate is flat/fails | Revert its values/code; retain useful infrastructure |
| Correctness bundle is Elo-flat but non-regressing | Keep the semantic fix |
| SPSA value reaches a bound | Inspect; widen once and start fresh (`-Resume` keeps old ranges) |
| Is an SPSA converged? | Thirds test on the log, not eyeballed values; judge on the gain curve, not % of planned iterations — see *Running an SPSA* |
| TC-sensitive change passes STC | Confirm at `10+0.1` |
| Phase boundary | External gauntlet, then release |

Bench nodes are a behavior fingerprint, not a strength or speed metric. For
speed, compare best-of-N `bench 13 5` NPS.

## What to report

- SPRT: verdict, games, Elo/error, LOS, and time losses/timeouts.
- SPSA: final values and any bound-pinned parameters.
- Gauntlet: score/ordo table, especially versus the previous Rarog release,
  Critter and SF-capped opponents.
- Bench: total nodes, geomean EBF, and best-of-N NPS when relevant.
- Failure: the exact command and first failing/error line.

## Common commands

```powershell
# Tools
./tools/setup_tools.ps1

# If a PGO build dies with "target must match host": the rustup DEFAULT HOST
# has drifted to windows-gnu, so the pinned 1.97.1 resolves to its gnu
# variant and PGO training (which runs the instrumented binary locally)
# refuses. rust-toolchain.toml pins the CHANNEL, not the host triple, so it
# cannot catch this. Check with `rustup show active-toolchain`, then:
rustup set default-host x86_64-pc-windows-msvc

# PGO test binary / SPSA tune binary
./tools/build_test.ps1 -Suffix <s>
./tools/build_test.ps1 -Suffix <s> -Tune

# Primary SPRT [0,3]
./tools/sprt.ps1 -EngineA "tools\test_engines\A.exe" -EngineB "tools\test_engines\B.exe" `
    -NameA "cand" -NameB "head" -Elo1 3

# Non-inferiority / LTC additions
#   -Elo0 -3 -Elo1 0
#   -TC "10+0.1"

# Harness calibration after a runner change (same binary on both sides)
./tools/sprt.ps1 -EngineA "tools\test_engines\A.exe" -EngineB "tools\test_engines\A.exe" `
    -NameA "NullA" -NameB "NullB" -Mode calibrate

# SPSA (setup + launch in one command; -Resume to continue, -SetupOnly/-LaunchOnly to split)
./tools/spsa.ps1 -ConfigGroup <group> -EngineSuffix <s> -Iterations 6000

# NPS A/B (10.3 protocol). Validate on a SELF PAIR — same exe both arms —
# before trusting any verdict; it must read ~0.00%.
./tools/nps_ab.ps1 -Pairs 18 -Base "tools\test_engines\A.exe" -Cand "tools\test_engines\B.exe"

# NPS A/B pooling several PGO builds per arm (needed for any sub-1% effect,
# since two PGO builds of identical source differ by ~0.36%)
./tools/nps_multibuild.ps1 -Cycles 10 -BaseSet @("...a.exe","...b.exe") -CandSet @("...a.exe","...b.exe")

# Local verification
cargo test --release
cargo fmt --check

# WAC tactical suite (bench-like diagnostic; deterministic solved count)
.\target\release\rarog.exe    # then: wac        (or: wac <depth>)
```

SPRT defaults to physical cores minus two; the gauntlet retains its conservative
8-game concurrency. Both use an OS-derived explicit physical-core CPU list.
After any harness change, run the fixed 30k-game identical-binary
`-Mode calibrate`; PASS requires its full 95% nElo CI inside ±5.
Node-limited datagen may use 24.

## Roadmap: releases and the NNUE cutoff

Everything before the cutoff is **NNUE-durable** (correctness, search, speed,
infra — NNUE replaces only the eval function). HCE eval-*strength* work is the
one thing NNUE subsumes, so it waits in the contingent Phase 13.

| Phase | Outcome | Release / cutoff |
|---:|---|---|
| **7** | Correctness bugs (7.6 ✅, 7.4 ✅, 7.2 ✅, 7.5 ✅) — **COMPLETE** | — |
| **8** | Search-mechanism wave — **COMPLETE** (accepted and rejected arms preserved in PLAN/history) | — |
| **9** | Reproducible builds, CI, shipped PGO + clean-code P1/P2, 9.7.5 SMP II — **COMPLETE** | — |
| 9.8 | Boundary gauntlet ✅ — +76 / +78 / +194 over 2.2.0 | ✅ **RELEASED 2.3.0** (+2.3.1) |
| **10** | ▶ **CURRENT** — evidence provenance; NMP/IIR/singularity, history/selectivity/qsearch and root/SMP cooperation; final search-only refit | — |
| 10.12 | Hard direct target ladder plus cumulative 1T/LTC/4T matrix | **▶ RELEASE 2.4.0** |
| **━ NNUE CUTOFF ━** | no standalone HCE-eval strength before here | |
| **11** | NNUE infra prep (StateInfo, accumulator scaffolding, frozen corpus) | — |
| **12** | NNUE program via `net_trainer` (contract → king buckets → scaling) | **▶ RELEASE 2.5.0** |
| **13** | HCE deepening — **only if NNUE fails/stalls** (all NNUE-subsumed eval) | — |
| **14** | Parked: SMP, platform, distributed testing | — |

**Two releases before NNUE:** 2.3.0/2.3.1 ✅ after the first search wave, then
2.4.0 only after 10.12. The NNUE line opens at Phase 11. Phase 10 does not
revive HCE work; old rejected mechanisms return only when the new diagnostics
identify a concrete contract failure and a registered test.













