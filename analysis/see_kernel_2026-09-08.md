# Incremental SEE attacker maintenance — RAR-M35 / 4.11b.11

## Decision

**Disposition, 2026-09-08: `NO_CHANGE`. The candidate was implemented in full,
measured against a contract frozen before timing, and rejected by its own
stage-1 screen. The production path is withdrawn; `src/` is byte-identical to
`8d7da2c`.**

The candidate was not merely unprofitable — it made its own target region
**slower** in all three alternating rounds. No games, no Elo claim, no adjacent
optimization, and no larger rerun.

## What was built

The exchange loop in `see_with_values` and `see_ge_impl` carried an all-colour
attacker set instead of rebuilding it each step:

- `see_attackers(target, occ)` = `attackers_to(target, occ) & occ`, built once.
- `see_recapturer` selected with `attackers & color_occ(side)`, which
  reproduces `attackers_to_color` exactly because every colour-specific term of
  `attackers_to` is a subset of that colour's occupancy.
- `see_expose` added only the ray that vacating `from` can open — diagonal for
  pawn/bishop/queen, orthogonal for rook/queen — then re-masked with `& occ`.
  A knight attacking `target` is never aligned with `target`, and a king
  recapture ends the exchange before the result is read.

The 4.11b.5 semantics were preserved throughout: the per-candidate selected-king
legality test, the `& !Bitboard::from(target)` exclusion, promotion/new-victim
values, threshold boundary parity, and the rule that an illegal candidate is
dropped only for the current selection and stays in the carried set.

## Correctness — the candidate was correct, and that was verified hard

This is a rejection on throughput, not on defects.

- `bench 13` reproduced **7,601,220 / EBF 2.474** exactly.
- A `debug_assert_eq!` in `see_recapturer` compared the carried set against a
  fresh `attackers_to_color` on **every SEE call**. It was **proven live**:
  deliberately dropping queen orthogonal exposure made
  `threshold_parity_on_deterministic_legal_walks` panic at once. With the guard
  active the whole debug suite passed **275/275**, so every SEE call in the
  fuzz, differential, WAC and endgame suites agreed with the recomputing
  baseline. Fingerprint equality was explicitly not treated as sufficient, per
  the 4.9a.4 lesson.
- All 41 external fixtures passed (`see_contract` 8/8), `see_pins` 6/6, release
  276/276, `cargo fmt --check` clean, Clippy `--all-features --all-targets`
  zero warnings.

## Stage 1 result — the registered screen rejects

Three alternating `board_v2` rounds on a host at 5.38% ambient. Registered
gate: the candidate must beat its paired baseline on `threshold SEE only` in
all three rounds **and** reach a median gain of at least **+5%**.

| Round | Baseline ops/s | Candidate ops/s | Delta |
|---|---|---|---|
| 0 | 86,703,120 | 84,175,247 | **-2.92%** |
| 1 | 86,888,185 | 77,833,849 | **-10.42%** |
| 2 | 83,505,652 | 82,932,868 | **-0.69%** |

Median **-2.92%**; zero of three rounds up. The gate fails on both conditions,
so **stage 2 was not run**, exactly as registered. Round 1 was visibly
disturbed on unrelated columns too (`make/unmake only` -4.79%, `capture
generation` +4.09%), so the honest reading of the effect size is rounds 0 and 2:
a small regression of roughly **-1% to -3%**. Discarding round 1 would not
change the verdict, and no such discard was applied.

## Calibration — the failure mode was predicted before exposure

The registration named this exact risk before any timing:

> short exchanges gain nothing, because the initial `attackers_to` computes
> **both** colours where the old first step computed one, and `see_ge_impl`
> exits early on many calls. If the call population is dominated by 0-1 step
> exchanges, this change can measure flat or slightly negative.

- **Direction: HIT.** The predicted downside mechanism is what happened.
- **Magnitude: MISS on the upside band.** Predicted +5% to +20% normalized
  throughput; measured about -1% to -3%. The prediction placed too little
  weight on its own stated counter-risk.
- **Stage 2 was correctly made conditional.** The two-stage design saved the
  entire expensive arm — roughly half an hour of measurement — on a candidate
  that could not have resolved a positive whole-search effect.

## Why it lost — and where SEE cost actually sits

The leaf's premise was that the kernel "recomputes all attackers twice per
exchange step". Reading the kernel, the two `attackers_to_color` calls per step
are **not** duplicates of each other:

1. `attackers_to_color(target, occ, side)` — the recapturer set. This is the
   only one an incremental carried set can replace.
2. `attackers_to_color(king, after, !side)` — the **selected-king legality
   test**, at a *different square*, under a *different occupancy* (`occ ^ from`),
   evaluated **per candidate**. It is the 4.11b.5 repair, it is mandatory, and
   a carried set of `target`'s attackers is structurally incapable of serving it.

So the optimization could address at most one of the two queries, while the
untouched one is evaluated once per candidate rather than once per step. Against
that it added a fixed cost at entry: `attackers_to` builds **both** colours'
piece unions where the old first step built one. With `see_ge_impl` exiting
early on a large share of its 7.55M threshold calls, that entry cost is paid
constantly and the per-step saving is rarely collected.

**Redirect for any future SEE work: the target is the per-candidate king-legality
test, not the attacker set.** That is where the untouched work concentrates.

## Retry trigger

Do not reattempt carried attacker sets on donor-engine similarity. Reckless and
Stockfish maintain attacker sets in kernels whose legality handling differs from
Rarog's post-4.11b.5 contract; neither is evidence here, and this measurement is
now direct evidence against it.

Reopen only if **both**: (a) the per-candidate king-legality test is first made
cheaper or rarer by a separately qualified change, and (b) a fresh profile shows
the recapturer-set rebuild is still a material share of SEE. Absent (a), the
arithmetic above does not change.

A fresh ETW profile is still owed and is now more valuable than before, because
it can attribute SEE's 5.3% between the recapturer set and the legality test.
It requires an elevated prompt and is a maintainer job.

## Evidence

`tools/results/see-kernel-411b11/` (ignored, local): `registration.md` frozen
before timing, `candidate.patch`, `stage1.json`, per-round board transcripts,
build and test logs. Measured artifacts are distinct — baseline
`0a0050f6...2e09dc`, candidate `f5884e78...01186b`, board arms `38400842...`
and `995db66f...`.
