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
| Branch / version | `master`; **2.3.1 PREPARED** — Windows ARM64 PGO restored, awaiting push + CI + tag |
| Accepted baseline | **p103-gate** (the whole 10.3 speed pass, ACCEPTED 2026-07-22); `bench 13` = **5,480,624**, EBF **2.420** — unchanged from p82a-nocheckext, since every 10.3 item was bench-identical. Future SPRTs gate vs `rarog-p103-gate-pext-pgo.exe`. |
| Working head | = accepted baseline (p103-gate) |
| Last strength results | **8.12(g2) index hoist: +3.76% NPS (CI 3.35…3.98) ≈ +7.5 Elo**, pooled PGO, bench-identical, accepted on NPS evidence. **8.13 SMP rework: +102.78 ± 16.38 (nElo +168.85, LOS 100%) at Threads=4** (720 games; 1-thread byte-identical, deployment unaffected). Before them, **8.4 history bundle +6.01 ± 3.57, LOS 99.95%** (15,214 games; carries 8.12's +0.98%), **10.3 speed pass +20.31 ± 7.13** (3,460 games), **8.2(a) +30.75 ± 8.83**, **8.1 +22.13**. 8.1b ❌ −6.6, 8.6 ❌ −7.78, 8.7 ❌ −7.29. |
| Current work | ▶ **2.3.1 patch release prepared.** After publishing it, the next cycle starts at **10.0 — search-accuracy decomposition**, which runs before any Phase 10 code (see below; the four-condition gauntlet reshaped its targeting). |
| Next release | **2.4.0 at 10.5.** Order: 10.0 decomposition → 10.1 → 10.4.6 re-fit → 10.2.5 capstone → 10.2 → menu → 10.4.3 → release. NNUE 2.5.0 at Phase 12 |

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
     6. NEVER renumber existing items — PLAN.md §S6 freezes item numbers
        because commits and history reference them. To insert before the
        first item use a .0 (e.g. 9.0), as 7.0 already does.
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
**D** contingent HCE deepening (13, last, may never run). Queue:
**Phase 7 ✅ → 8.1 ✅ (+22.13) → 8.1b ❌ → 8.2 ✅ (+30.75) → 8.6 ❌ (−7.78) → 8.7 ❌ (−7.29) → 8.10 ❌ (≈−5.4) → [10.3 ✅ +20.31, out of band] → 8.4 ✅ (+6.01, w/ 8.12) → 8.13 ✅ (+102.78 @4T) → 8.11 ❌ (−5.96) → 8.5 (SPSA running) = THE LAST OPEN ITEM** (8.9 capstone ⏭ moved to 10.2.5 / 2.4.0) (8.3 folded into 8.9; order ≠ numbers, which stay frozen; 8.10/8.11 added 2026-07-20 Basilisk cross-review; 8.12 speed II / 8.13 SMP added 2026-07-22 after 10.3's result)
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


### Phase 10 — Root model, aspiration/TM, speed, menu (→ 2.4.0)

- [ ] 10.0 **Search-accuracy decomposition — RUNS FIRST.** Eval measured at
      parity with Basilisk (paired Texel loss −0.0003 ± 0.0012 over 8,000 quiet
      positions, sign flipped between samples) and NPS is equal, yet Rarog is
      ~43 Elo weaker — so the deficit is search accuracy, which **survives the
      NNUE transition**. Basilisk is the measuring instrument here, not the
      target. Rarog reports 14.6 depth vs Basilisk's 12.7 at equal NPS and
      equal eval, and every Phase 8 attempt to prune harder lost (8.6, 8.7,
      8.11) — nothing in Phase 10 yet tests the opposite direction. Result
      re-aims 10.2.5 and 10.4.6.
      **2×2 MEASURED (gauntlet):** gap vs Basilisk is −55 (1T STC), −38 (1T
      LTC), −32 (4T STC), **+34 (4T LTC)**. Threads dominate (+72 at LTC) over
      time (+17 at 1T), so the **1T deficit is TC-INDEPENDENT** — which kills
      the "needs depth to pay off" story and leaves a plainly mis-tuned
      selectivity surface. Good news: a re-fit at 3+0.03 should transfer to LTC.
      ⚠ The direct head-to-head is ~40 Elo worse for Rarog than the pool rating
      at both TCs — Basilisk has a matchup-specific edge; judge general strength
      by the pool rating, which is the actual goal.
    - [ ] (a) First-move-cutoff-rate counter + the existing
          `lmr_research`/`lmr_applied` over-reduction ratio. No games.
    - [ ] (b) Fixed-nodes match vs Basilisk 1.9.1 to strip out speed and TM.
    - [ ] (c) **Over-pruning probe** — scale reductions/margins DOWN, one
          binary, one `[0,3]`.
- [ ] 10.1 Persistent `RootMove` records (bench-identical enabler; no games)
- [ ] 10.2 (a) aspiration modernization — retires the 7.0b guard, retuned;
      **(a′) revives 7.5's TM fix + `tm` re-SPSA** if it H0'd standalone.
      One `[0,3]` (+ LTC for TM)
- [x] 10.3 **[ACCEPTED +20.31 ± 7.13, nElo +33.06, LOS 100%]** Profile-guided
      speed pass — the whole stack passed its `[-3,0]` batch gate in one run
      (3,460 games, 3+0.03, H1; Ptnml [42,353,779,473,83], PairsRatio 1.41).
      New accepted head **`p103-gate`**. Both sides were bench-identical
      (5,480,624), so this gate measured **execution speed and nothing else** —
      the cleanest speed→Elo datapoint the project has.
      **COLLECTIVE SPEED: +10.35%** (CI 10.10…10.65) — 3,003,789 → 3,314,560
      NPS, pre-10.3 `c1fe620` vs 10.3 head `1d8afaa`, two independent PGO
      builds per arm, both base builds below both head builds. Target was
      ≥2.7M nps native; now ~3.31M pext-PGO ✓
      (Caveat on the speed figure: the pre-10.3 arm still carries 8.10,
      reverted inside 10.3(1), so its bench is 5,755,261 vs the head's
      5,480,624 — the node MIX differs slightly, though NPS is
      mix-normalised. The Elo gate has no such caveat.)
    - [x] **REVISED SPEED→ELO RULE: at STC ~2 Elo per 1% NPS, not 0.7.** The
          plan's "+10% speed ≈ +7 Elo LTC" predicted +7; the gate returned
          **+20.31**, ~3× that. Use **≈ +2 Elo per 1% NPS at 3+0.03** when
          grading future speed work. The old figure was an LTC estimate and is
          probably still right for LTC — deeper searches have more to lose
          from a shallower tree and less to gain per extra node — so do NOT
          transfer the STC constant to 10+0.1 without measuring it there.
          This materially raises the EV of speed work relative to the
          search-mechanism items, which have been failing (8.6 −7.78,
          8.7 −7.29).
    - [x] **NPS-MEASUREMENT PROTOCOL — REWRITTEN 2026-07-22 after the
          instrument was caught inventing results. Read the six sub-items
          below before measuring anything.**
    - [x] **The estimator must be validated on a SELF PAIR first** — the same
          `.exe` in both arms. It must read ~0.00%. An ABBA design (base,cand,
          cand,base averaged per round) and a neighbour-sandwich estimator BOTH
          read **−0.2%…−0.4% on a self pair**, and had already produced two
          confident-looking false rejections before the self pair caught them.
    - [x] **Why they fail:** bench NPS is **left-skewed** — interruptions
          create slow outliers, nothing creates fast ones. Any estimator that
          weights the arms unequally against that tail (single readings on one
          side, 2-sample averages on the other) manufactures a bias. Cancelling
          *linear* drift is not enough; the transient is convex.
    - [x] **What works:** strictly alternate the two arms, then compare
          arm-level **median** and **best-of** (both symmetric, both robust to
          the slow tail), with a bootstrap CI on the median. Reads +0.09% on
          the self pair. Script: `tools/nps_ab.ps1`.
    - [x] **Two independent PGO builds of IDENTICAL source differ by −0.36%**
          (CI −0.75…−0.06). PGO profile luck is a fixed per-binary offset, so
          **one build per arm cannot resolve a sub-1% effect** — pool 2+ builds
          per arm (`nps_multibuild.ps1`) and report per-build medians so
          non-overlap is visible.
    - [x] **Non-PGO builds are speed-reproducible** (null pair +0.17%, CI
          −0.34…0.54) and so make a cheap deterministic *screen* — but they
          badly overstate what ships: 10.3(8d) reads **+6.35% non-PGO and
          +1.18% under PGO**, because PGO already recovers most of it. Screen
          non-PGO, always confirm under PGO.
    - [x] Machine must be idle — video playback inflates the per-round SD from
          ~0.3% to ~1.4%. If a change is a strict work reduction AND
          bench-identical, keep it on the structural argument rather than
          buying rounds it cannot resolve.
    - [x] **(1) cont_history → boxed const-size tables — +1.15%.** A
          7-waypoint compiler-fixed bisect pinned the whole Phase-9 NPS loss to
          one commit (`886916b`): four `Vec` headers with runtime lengths
          defeated bounds-check elision in the hottest loops. Boxed
          `[[i16; CONT_SIZE]; 4]` restores compile-time layout, keeps 9.0a's
          table-driven source shape. Commit `8fdedc3`.
    - [x] **(2) per-node CheckInfo — +2.75%.** `gives_check` was called for
          EVERY scored quiet at EVERY node. Now per-node masks (check squares
          per piece + discovered-check blockers), per-move test = two bitboard
          tests. Promo/EP/castling fall back. Equivalence `debug_assert!`ed
          through the whole debug suite. Commit `dcdba44`.
    - [x] **(3) check-hinted `make_move` — +1.08%.** `calculate_checkers()` ran
          on every move; now the search passes the answer it already has, so
          non-checking moves store `EMPTY`. Hint asserted both directions +
          `board_differential` rebuilds `checkers` after every make/unmake.
          Commit `6b316af`.
    - [x] **(4) MovePicker single-buffer collapse — +2.04%, 9,288 → 3,136 B
          per frame (−66%).** The `Staged` variant held THREE 3,080-byte
          lists and the enum is sized to its largest variant, so every frame
          paid 9,288 B. Now ONE buffer partitioned in place: good captures /
          bad captures / quiets, each phase scanning only its own sub-slice.
          Pushes stay sequential so the `MaybeUninit` prefix invariant (and
          the KEEP-UNSAFE accessors) are untouched; captures+quiets provably
          fit 256. 12 rounds: +2.04% mean, positive in 10/12. Commit
          `17289ac`.
    - [x] **(5) pin/blocker sharing — ≲+1%, BELOW RESOLUTION, KEPT.**
          `compute_pinned` ran 2–3× per staged node: `generate_captures`
          computed it and `gen_moves` promptly recomputed it, then the quiet
          stage computed it a third time for the same position. Now computed
          once and threaded through `gen_moves_pinned` /
          `generate_quiets_pinned`, cached in `MovePicker::Staged`; a stale
          share is caught by a `debug_assert_eq!` against a fresh compute.
          20 interleaved rounds on a QUIET machine: mean +0.27%, trimmed
          +0.49%, median +0.88%, best-of +0.22%, 95% CI −0.37…+0.91 — the
          two quiet batches disagree in sign, so the effect is **not
          resolvable** above this machine's noise. Kept on the structural
          argument alone: strictly less work, bench bit-identical, no
          complexity cost. Commit `961e535`.
    - [x] **(6) qsearch make_move hint — MEASURED, REJECTED (−0.79%).**
          Implemented exactly as (3) and measured: mean −0.79%, only 2 of 6
          paired rounds positive, spread +2.0%..−3.3%. The deferral reason was
          the right one — qnodes make too few moves to amortize the
          once-per-node mask build, so the extra work outweighs the saved
          `calculate_checkers`. Reverted; do not retry without a cheaper way
          to obtain the masks.
    - [x] **(7) `has_pseudo_capture()` measure-or-keep — SPLIT, +1.5%.** The
          pre-scan (untouched since the initial commit, never measured) was
          instrumented first: it fires on **19.1%** of qsearch/ProbCut calls
          and **18.9%** of staged calls — but **78.5%** of the staged firings
          then generate quiets anyway, paying a full `compute_pinned` the
          pre-scan had just "saved". A *failing* pre-scan is a full attack pass
          over every one of our pieces; `compute_pinned` is four slider lookups
          plus a short sniper walk, i.e. strictly cheaper. So the answer is
          per-call-site, not one verdict: **kept** on the captures-only path
          (qsearch, ProbCut — nothing follows, so the skipped pin is real, and
          the 80.9% that do find a capture exit the scan early on the
          king/pawn tests), **removed** on the staged path, which now always
          computes pins and always shares them. Drops the `Option<Bitboard>`
          and its `None` branch from `MovePicker::Staged`. 20 interleaved
          rounds +1.79% (20/20 positive), 14 order-swapped control rounds
          +1.46% (14/14) — both orders agree, so ordering bias is excluded and
          the effect sits above this machine's ~1% resolution limit.
    - [x] **(8) small sweeps — +1.18% combined** (CI 0.69…1.39, two PGO builds
          per arm, both cand builds above both base builds). All four named
          candidates were tried; commit `286995b`. Detail below.
        - [x] **(8d) move-picker scan — the whole gain.** `pick_next` is the
              hottest loop in the engine and it indexed `moves[current]` AND
              `moves[best]` every iteration: two loads where one suffices, plus an
              index LLVM cannot bound-check away (`best` is in range only by
              induction through the loop). Now scans a `split_at_mut` tail by
              iterator with the running best SCORE in a local. Ties still resolve
              to the earliest entry (comparison stays strictly `>`).
              **+6.35% non-PGO (18/18 pairs), +1.18% as shipped** — PGO had
              already recovered most of it, which is the whole reason non-PGO
              screening must be confirmed under PGO.
        - [x] **(8a) last two runtime-length `Vec` tables boxed — +0.34%**
              (CI −0.11…0.62, not resolvable alone). `pawn_history` and
              `continuation_correction_history` were the shape 10.3(1) bisected
              to −2.1%. Kept on the structural argument, like (5). NB the named
              "bounds/`Option` overhead in hot accessors" was **already gone** —
              9.0 masked `Square::index()` to `& 63` and gave
              `piece_type_at_unchecked` the padded-16 table.
        - [x] **(8b) boolean attack-test helper — 0.00%, kept.**
              `is_attacked_by_with_occ` short-circuits
              `attackers_to_color(...).any()` for the two passed-pawn scans in
              eval that only need the boolean. The sites are too cold to
              register; kept because it is strictly less work and reads better.
        - [x] **(8c) insufficient-material early exit — CLOSED, NO CHANGE.**
              Sized the prize first with a probe binary that deletes the per-node
              check outright: **≤ +0.23%** (CI −0.16…0.65), and that probe also
              searched MORE nodes, so even 0.23% flatters it. Below the
              resolution floor, so no optimisation of it can pay. The existing
              code already exits on its first test (`pawns.any()`) in the common
              case, and no cheaper EXACT pre-filter exists — no piece-count bound
              works, because ANY number of same-colour bishops is insufficient
              material.
    - [x] **(9) startup-only: magics baked — 192 ms → 19 ms on the generic /
          AVX2 build** (10×, now equal to the PEXT build's 18.9 ms). The magic
          SEARCH was that build's entire startup cost. Behaviour-preserving by
          construction: `find_magic` seeds a fixed RNG and is deterministic, so
          the baked constants ARE what it computes; it verifies each baked value
          and falls back to searching if one fails, so a stale constant costs
          startup time, never correctness. `baked_magics_cover_every_square`
          asserts the fallback stays unused. Commit `1d8afaa`.
          **The old figures in this item were stale** — PEXT startup is ~19 ms
          today, not 174–199 ms, so there was never anything to win there; the
          375–429 ms generic figure is the one that was real.
    - [x] 10.3(gate) **[ACCEPTED]** — `[-3,0]` vs `p82a-rebuilt`,
          one run for the whole stack, 2026-07-22. Result in the parent item.
- [ ] **10.2.5 Unified prospective LMR depth — THE SEARCH CAPSTONE** (⏭ moved
      here from 8.9, 2026-07-25). One confidence-adjusted `lmr_depth` per move
      driving LMP, futility, SEE pruning *and* the reduction together; allows
      zero reduction for strong late moves. Absorbs 8.3 and the deferred
      8.2(b)/(c). EV +3–10, high risk.
      ⚠ **Schedule this EARLY in the 2.4.0 cycle despite the number** — item
      numbers are frozen and do not imply order; a weeks-long item with a real
      chance of rejection needs runway, not the slot before a release.
      ⚠ Entry condition already MET (8.2 ✅ + 8.4 ✅). Basilisk warning: its
      TT-PV prototype was **+51% nodes via the LMR route with no good
      operating point** — the stored bit pays only through pruning
      conservatism, so weight tt_pv toward the pruning-side consumers.
      ⚠ Guard against 8.6's failure mode: a self-play-tuned candidate that
      searched 16% more aggressively won its SPSA then lost the gate. Gate
      against the accepted head, never a sibling of the tuning run.
- [ ] 10.4 ⏭ all-skippable menu (each its own `[0,3]`, strict EV gate; incl.
      demoted 8.8 qsearch quiet checks)
- [ ] 10.6 Guarantee at least one `info` line before every `bestmove`.
      `info depth` is only emitted when an iteration COMPLETES, so a search
      whose budget expires mid-iteration reports nothing at all — legal UCI but
      unhelpful, and it made a CI test race (fixed by raising its node budget,
      not by changing the engine). Output-only, so bench-identical and needs no
      SPRT. See PLAN 10.6.
- [ ] **▶ 10.5 RELEASE 2.4.0 — YOU run the boundary gauntlet, tag & publish
      `v2.4.0`** (root / speed)

### ━━━ NNUE CUTOFF ━━━ (Phase 11 opens the NNUE line; nothing below survives on HCE)

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

- [ ] **`t` counts GAMES, not iterations.** `spsa.py` does
      `self.t += cutechess.games` (32 per iteration), and both schedule terms
      read it: `a_t = a/(t+A)^0.601`, `c_t = c/t^0.102`. The console prints
      BOTH (`iterations: N` and `games: 32N`) — quote iterations to humans,
      but do the schedule arithmetic in games or the answer is ~32× off.
- [ ] **⚠ `A` is set in the WRONG UNITS by `spsa.ps1`** (`A = Iterations/10`,
      i.e. iterations, while `t` is games). At 6000 planned iterations we set
      A=600 where the design intends ~19,200. Consequence: the gain decays far
      faster than the "A = 10% of run" rule implies — measured on 8.4 at
      iteration 1244, `a_t` was already down to **8.2%** of initial, where the
      intended damping would have left it at 51%. Not fatal (SPSA still
      converges, values are still fitted) but it means **late iterations buy
      much less than the planned budget suggests**: 1244→6000 only moves the
      gain 8.2% → 3.2%. Judge stopping on the gain curve, not on
      "% of planned iterations".
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

**8.4 SPSA (10.3 is done and accepted).** The tune binary is already built at
the accepted head `p103-gate` (sha `3240837`, clean tree, bench 5,480,624), so
this is a launch, not a build:

```powershell
./tools/spsa.ps1 -ConfigGroup histcov -EngineSuffix p84-histcov -Iterations 6000
```

1. Launch it (long job — give it the full core budget, nothing else running).
2. **Paste the final values of all 10 dims and any bound-pinned params** when
   it converges — the model never reads `state.json` itself.
3. Model then bakes the values, verifies, and hands you the 8.4 `[0,3]` gate
   command **vs `rarog-p103-gate-pext-pgo.exe`** (the new head, NOT p82a).

Afterwards the wave continues per the queue: 8.5 → 8.9, with 8.11 (fail-soft
qsearch), 8.12 (speed pass II — currently the best-paying lever at ≈2 Elo/1%
STC) and 8.13 (SMP compare-and-choose; needs the sprt.ps1 Threads
passthrough first) slotting in wherever convenient — all three are
independent of the LMR family. Then 10.4 (all-skippable menu) and the 2.4.0
boundary gauntlet at 10.5.

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
| **7** | Correctness bugs (7.6 ✅, 7.4 ✅; **7.2 in gate**; 7.5 next) | — |
| **8** | Search-mechanism wave — **must COMPLETE before 2.3.0** (8.6 ❌ → 8.7 ❌ → 8.10 ❌ → 8.4 → 8.5 → 8.9 + 8.11/8.12/8.13) | — |
| **9** | Reproducible builds, CI, shipped PGO + clean-code P1/P2 | — |
| 9.8 | Boundary gauntlet (you) — after Phase 8 completes | **▶ RELEASE 2.3.0** |
| **10** | Root model, aspiration/TM modernization, **10.2.5 the search capstone (moved from 8.9)**, speed pass ✅, ⏭ menu | — |
| 10.5 | Boundary gauntlet (you) | **▶ RELEASE 2.4.0** |
| **━ NNUE CUTOFF ━** | no standalone HCE-eval strength before here | |
| **11** | NNUE infra prep (StateInfo, accumulator scaffolding, frozen corpus) | — |
| **12** | NNUE program via `net_trainer` (contract → king buckets → scaling) | **▶ RELEASE 2.5.0** |
| **13** | HCE deepening — **only if NNUE fails/stalls** (all NNUE-subsumed eval) | — |
| **14** | Parked: SMP, platform, distributed testing | — |

**Two releases before NNUE** (2.3.0 after the search wave, 2.4.0 after
root/speed), then the **NNUE line opens at Phase 11**. Revived rejected work is
numbered: 7.2 SEE (in gate), 10.2a aspiration, 7.5 TM. 7.4c OCB moved to 13.8
(NNUE-subsumed). Nothing else from the reject pile is revived — 6.1/6.2, the
7.1 draw rework and 7.3 stay dead (lessons 1/14).













