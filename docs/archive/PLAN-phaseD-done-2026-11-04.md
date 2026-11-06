# PLAN: Phase D's finished steps (archived 2026-11-04)

Verbatim text moved out of `docs/PLAN.md` on 2026-11-04, after PR #13
landed as `c69761d`: PLAN keeps each finished step's heading over a short
summary, and the text below is what it held. Nothing here is a current
instruction. Citations of the form `PLAN D.x` in the ledger, HISTORY and
the analyses resolve here; the readings are in the ledger rows each step
names. D.4's full text joined on 2026-11-06, when D.4 finished.

## D.1

- **D.1 Time management: audit against the ADR-0065 checklist, soft/hard bounds with node-fraction multiplier, forfeit margin; SPRT `[0,3]` — `R2` investigation, `I2`/`V` sub-steps, DONE 2026-10-26.** **Maintainer decision 2026-10-26: closed, the residue accepted.** The stalls are solved for the vast majority and the rest is not harmful: the stop at twice the target ends every probed stall, and `d1`'s game residue is a third of 2.6.0's at `10+0.1` with no measured cost (RAR-R28 and its residue addendum). The single-window fix is held with its retry trigger, a measured cost. Audit the
  current clock (budget, overhead, `smp_reserve`; the root-confidence
  consumers are gone since B.1 and are not rebuilt) against the Reckless
  shape; implement the soft/hard bound model with the
  node-fraction and stability multiplier if B.5 has not already; forfeit
  margin sized on a null pair; one registered SPRT `[0,3]` at STC and a
  direction check at `10+0.1`. **Audit checklist**, taken from Manta's
  ADR-0065, whose integrated clock passed inside a cumulative gate where
  Rarog's own root-confidence attempt at the same idea stayed inert: distinct
  optimum and immutable maximum budgets; the maximum rooted at `go` receipt
  and never extended by root or ponder evidence; ponder credit consumed once
  at a discounted rate; the optimum adjusted only by completed exact root
  iterations, from stability, best-move change, score trend and effort
  concentration as bounded factors; an easy root defined as stable
  concentrated effort that spends *less*; no early stop below a minimum
  completed depth; helper threads contributing at most a best-move-change
  count normalised by helper count, so worker count cannot multiply wall
  time. Each item is ticked as present, absent or different in Rarog before
  the donor shape is chosen; the list is a checklist, not a design to copy.
  **Research input, measured 2026-10-01 by B.5.2**
  (`analysis/b52_research_2026-10-01.md`): in won endgames without
  tablebases, an aspiration cascade of fail-highs on a rising evaluation
  makes an iteration outlive the optimum, and the move ends only at the
  hard maximum. At `60000+600`, rec1 took 32,305 ms (maximum 32,304; the
  last `info` at 3,744 ms) and KRvK 11,251 ms (maximum 11,250, stalled at a
  downgraded mate value). Stockfish 19 took 5.6–6.4 s and about 1 s on the
  same positions. B.5.2 contains it at tablebase roots only; the general
  case is this leaf's.
    - **D.1.1 Time-management diagnosis, pulled forward: rec1's stalled re-search, rec2's search past a found mate, rec3's clock-independent stop; mechanism, frequency in games, the frozen fix and its gate — `R2`, NO_CHANGE 2026-10-06.**
      Pulled forward by maintainer decision 2026-10-06, ahead of C.2's
      baseline-refit gate and C.3's first gate (Phase C's freeze rule,
      its exception). Evidence: `analysis/basilisk_review_2026-10-06.md`, item 1.
      Without tablebases Rarog spends its hard maximum, 52–58% of the
      clock, on rec1 (a re-search stalled at `cp 1302 lowerbound`) and on
      rec2 (after finding `mate 5`, its iterations stall near depth 56) at
      both `60000+600` and `180000+2000`, where Stockfish 19 takes 9.4 and
      43.8 s on rec1 and 1.4 and 2.7 s on rec2; on rec3 it stops at 10.4 s
      and depth 21 at both clocks, so not by the clock. The maximum itself
      is the donor's and not the defect. Research first: the mechanism of
      each behaviour (is rec2 rec1's cascade or a missing stop after a
      proved mate; what ends rec3), how often each happens in game
      records, then the fix frozen with its prediction. Gate: a
      correctness repair takes AGENTS' repair case 2 (a registered
      2,000-game read with a harm rule); a change to the time formulas
      takes an SPRT `[0,3]`. The change touches only clock play (`bench
      13` unchanged); its accepted head becomes Phase C's frozen head.
      **Closed `NO_CHANGE` 2026-10-06 (RAR-R13;
      `analysis/d11_tm_diagnosis_2026-10-06.md`).** Untimed per-iteration
      traces give the three mechanisms. rec1 is a fail-high cascade at
      depth 32: nine re-searches centred on a stale `cp 534`, then a tenth
      window, [513, 1,691], that does not finish. rec2 is a proved mate
      (`mate 5` from depth 17) with no stop: there are no bound lines, so it
      is not the cascade, and depth 57 grows to the ply cap and does not
      finish. rec3 is the clock's soft stop one iteration late: both clocks'
      targets fall inside a depth-21 iteration that a six-step cascade
      stretched from 1.8 to 10.4 s. In 5,400 games of the head's search
      (RAR-M64, RAR-M65, RAR-E19), every move at ≥ 0.9 × the maximum in a
      won or mate position was in a game the head won, bar one `10+0.1`
      draw whose long move was an ordinary low-clock overrun. The upper
      bound on the effect is about 0.5 Elo, which no gate resolves. The
      repairs are either refuted territory (the aspiration loop: RAR-S89,
      RAR-S17; it also moves `bench 13`), D.1.2's bound model (a clock-only
      fail-high stop), or a time saving in won games only (a mate-proved
      stop). Nothing lands ahead of Phase C, and its frozen head is
      unchanged. The fix moves to D.1.2, which owns it, with these
      mechanisms and acceptance checks. RAR-R13's trigger pulls it forward
      again if a game record shows a cost.
    - **D.1.2 Time-management audit and bound model: the ADR-0065 checklist, soft and hard bounds with the node-fraction multiplier, the forfeit margin, the won-ending stalls (RAR-R13); SPRT `[0,3]` — `R2`, DONE 2026-10-26.**
      Closed 2026-10-26 with D.1.2.1, D.1.2.2 and D.1.2.3 done.
      The rest of D.1 as written above, after Phase C, starting from
      Phase C's head (D.1.1 changed nothing). **This step owns the fix of
      D.1.1's three behaviours** (RAR-R13,
      `analysis/d11_tm_diagnosis_2026-10-06.md`). They are real, and they
      had no measured cost in results at `3+0.03` and `10+0.1`, so they were
      not pulled ahead of Phase C, but they grow with the control.
      - **rec1:** a fail-high cascade whose late re-search never
        finishes, so the move ends only at the hard maximum.
      - **rec2:** a proved mate with no stop, whose next iteration grows to
        the ply cap and never finishes.
      - **rec3:** an iteration started under the soft target that a cascade
        stretches to about four times it.

      What the bound model must decide:

      - Whether a clock search may end inside an iteration (after a
        fail-high past the soft target, playing the fail-high move) or on a
        proved and stable mate. ADR-0065's "optimum adjusted only by
        completed exact iterations" is the item it interacts with.
      - How that interacts with B.5's retry trigger (mid-iteration stops
        above 10% reopen the discarded-best-move question).

      The aspiration loop itself is not the lever. RAR-S89 and RAR-S17
      rejected changing it, and it moves `bench 13`; the deep-iteration
      growth to the ply cap is a search property and stays out of scope.
      **Acceptance adds, beside the SPRT:**

      - a timed probe of rec1–rec3 at `60000+600` and `180000+2000`
        (`analysis/artifacts/basilisk-review-2026-10-06/tm_rec_probe.py`;
        maintainer-run, idle host) in which no move reaches the hard
        maximum;
      - `tm_games.py` (`analysis/artifacts/d11-tm-diagnosis-2026-10-06/`) on
        the gate's own games, reporting the share of won and mate moves at
        ≥ 0.9 × the maximum (0.48% and 0.24% at `3+0.03`, 1.77% and 2.59%
        at `10+0.1` on Phase C's head).

      The general overrun is input too: moves at ≥ 3 × the optimum are 4.7%
      of non-won moves at `3+0.03` and 1.8% at `10+0.1`; time past the
      optimum is 31.4% and 19.7% of all head time; and a started iteration
      always completes.

      D.1.2 is carried out in three addenda (added 2026-10-11 by maintainer
      request, so the two GUIDE holds it owns have steps that discharge
      them), gated together by one SPRT, which D.1.5 now holds.
      **Research closed 2026-10-22, `READY_FOR_IMPLEMENTATION`** (RAR-R14,
      `analysis/d12_tm_research_2026-10-22.md`; re-opened in full by the
      maintainer's request of 2026-10-22; zero games). Read from 11,200 of
      the C head's games in six runs (RAR-M73–M76) and the formulas:
      - **The floor.** The budget assumes fifty moves to go at every clock,
        so at 500 ms it still counts 1,470 ms of future increments; a
        `3+0.03` game drifts to the clock where the optimum equals the
        increment, 155 ms. 22–41% of the head's games touch 100 ms and 21%
        of its moves against RAR-M75's field are played under 200 ms.
        Stockfish 11, Dragon 1 and Komodo 13.3 never go under 200, 790 and
        650 ms; Houdini 3 and 6.03 go as low as the head. Current
        Stockfish's sub-second horizon contraction (`mtg = time_ms / 20`)
        alone lifts the drift to 400 ms; Reckless's linear bounds sit at
        510 ms.
      - **Holds and forfeits.** OS holds of 80 ms or more hit 0.11–0.15%
        of moves at one thread and 0.4% at four (the move waits for every
        helper); four time losses in 226,180 games at the 20 ms margin
        since 2026-10-10, each a 69–129 ms hold at a clock of 70–163 ms in
        a lost game. No reserve covers a hold longer than the clock; a
        policy that never holds such a clock does. Elo value here under
        0.01; the cost is robustness on another harness.
      - **Spend.** Median 0.78–0.91 of the optimum, 99th percentile 3.7–4.5;
        21–31% of all head time is past the optimum; the front-loaded shape
        (44–47% of the game's time in the first forty plies) is shared with
        Stockfish 11 and Houdini 6, while Dragon and Komodo spread it and
        leave 1.2–1.6 s unused. No zero-game instrument reads the shape's
        Elo.
      - **Stalls.** RAR-R13 reproduces on the C head (won and mate moves at
        ≥ 0.9 × the maximum 0.30–0.44% and 0.24–0.53% at `3+0.03`, 1.44% and
        3.40% at `10+0.1`), all in wins bar three draws.
      - **Donors.** Reckless 0.10.0-dev: linear soft and hard bounds in the
        main clock, a continuous node-fraction multiplier (×0.55–2.5), pv
        and eval stability discounts, no mate stop, no in-iteration stop,
        a 65% vote. Stockfish master 2026-09-30: Rarog's shape plus
        `originalTimeAdjust` (a per-control scale, 0.76 at `3+0.03`, 1.53 at
        40/15), the sub-second contraction, a time-disadvantage scale, a
        stability discount (`timeReduction`), an effort cut that fires only
        above 90% concentration, a `mate_in(3)` stop, shallower iterations
        once half the target is spent. RAR-R13's "Stockfish has no clock-mode
        mate stop" is corrected there; rec2's `mate 5` lies outside its rule.
      The decision is one clock-only cluster (D.1.3), one fit (D.1.4) and
      one gate (D.1.5); the addenda's findings:
        - **D.1.2.1 Won-ending time stalls: rec1's cascade, rec2's missing stop after a proved mate, rec3's overrun (RAR-R13) — `R2`, DONE 2026-10-26.**
          **Maintainer decision 2026-10-26: closed, the residue accepted.** The stalls are solved for the vast majority and the rest is not harmful: the stop at twice the target ends every probed stall, and `d1`'s game residue is a third of 2.6.0's at `10+0.1` with no measured cost (RAR-R28 and its residue addendum). The single-window fix is held with its retry trigger, a measured cost.
          Research closed 2026-10-22: the fix is two cluster items, a
          proved-mate stop (a clock search ends after a completed iteration
          whose root mate for the side to move has held with the same move
          and distance for three iterations, or whose score is a loss in at
          most two plies) and a root fail-high stop past the soft target
          (a window that fails high once the elapsed time is at or past the
          soft target ends the search and plays the move that failed high,
          with its lower bound; that bound exceeds the previous best's
          exact score, so no best move is discarded and B.5's mid-iteration
          trigger is not reopened). Ticks when D.1.5's timed probe
          (`tm_rec_probe.py`, both clocks, idle host) shows no rec move at
          the hard maximum, rec2 under 2 s, and `tm_profile.py` on the
          gate's games reads the won and mate shares at ≥ 0.9 × the maximum
          below 0.2%.
          **Re-owned 2026-10-23 (maintainer decision; RAR-R18).** rec1's and
          rec3's cascades are a search property, full-depth re-searches
          after a root fail-high, and are fixed in the search by **D.1.6**;
          the root fail-high stop that capped them cost about 15 Elo at
          `3+0.03` and is retired (off by default, `a0f9755`). rec2's fix stays
          the proved-mate stop (RAR-P42 saw 2.6.0 still searching past a
          proved mate at `120+1`). Ticks when D.1.6's probe shows rec1 and
          rec3 under the hard maximum with no clock stop, and D.1.5's
          closing read shows rec2 under 2 s and the won and mate shares at
          ≥ 0.9 × the maximum below 0.2%.
        - **D.1.2.2 The forfeit margin against OS holds — `R2`, DONE 2026-10-26.**
          **Ticked 2026-10-26 by RAR-R28:**
          - no candidate time loss in 5,000 games;
          - the clock-floor median 429 ms at `3+0.03`;
          - no game under 100 ms, against the 2.6.0 engine's 680 of
            2,000.
          Research closed 2026-10-22: the margin is a horizon question. The
          cluster contracts the horizon below one second of clock
          (`mtg = min(50, ⌊clock_s × TmHorizonPerSec⌋)`, seed 50, a fit
          coordinate); the engine's reserve (`2 × Move Overhead`, +30 ms
          above one thread) and the harness's 20 ms margin stay, with the
          arithmetic recorded in the packet. A deterministic test injects a
          late `go` stamp at a 150 ms clock and checks the reserve holds.
          Ticks when D.1.5's games show no candidate-arm time loss and
          `tm_profile.py` reads the candidate's clock-floor median above
          350 ms at `3+0.03` with under 3% of games touching 100 ms.
        - **D.1.2.3 The audit against the ADR-0065 checklist and the bound-model decision — `R2`, DONE 2026-10-22.**
          Ticked in the packet: present are distinct budgets, an immutable
          maximum rooted at `go` receipt, and completed-iteration
          adjustment by best-move change, score trend and effort; absent
          are a stability-over-iterations discount (both donors carry one),
          ponder credit (ponder is off in CCRL and Colosseum) and a
          first-iteration guard on the hard stop; different is the helper
          vote (D.2). The effort term has Stockfish's shape, a cut only for
          concentrated effort, which is why RAR-S47 read it inert; Reckless's
          is a continuous extender. The bound model: a clock search may end
          inside an iteration after a root fail-high past the soft target,
          and on a proved stable mate; a stability factor is added; the
          effort band's floor and endpoints become coordinates whose ranges
          reach both donors' shapes, so the fit decides the shape question.
          The SPRT moves to D.1.5.
    - **D.1.3 The clock cluster: RAR-R14's six clock-only mechanisms and the cross-move score trend — `I2`, DONE 2026-10-22.**
      From `analysis/d12_tm_research_2026-10-22.md`, *Implementation
      handoff*, in `src/search/time.rs` and the root loop of
      `src/search/mod.rs`: (1) the sub-second horizon contraction
      (`TmHorizonPerSec`, seed 50, range 20–100); (2) a stability factor
      `max(TmStabFloor, TmStabBase − TmStabSlope × k)`, `k` the completed
      iterations since the best move last changed, seeds 1.2881 / 0.0440 /
      0.716; (3) the effort band as coordinates, `TmEffortHigh` → `TmEffortLow`
      as the best move's share of the search's nodes rises from
      `TmEffortFloor` to 1.0, seeds 0.924 / 0.71 / 0.79, ranges 0.6–2.0 /
      0.4–1.0 / 0.0–0.95; (4) the proved-mate stop and (5) the root
      fail-high stop of D.1.2.1; (6) no hard stop before the first
      iteration completes. Invariants: `bench 13` unchanged (13,987,223 /
      EBF 2.565, by construction); `bestmove` is the move the last `info`
      line names and a fail-high stop's line carries `lowerbound`; the
      maximum is never raised; the DTZ root's `maximum = optimum` and the
      tablebase extension's box are unchanged; nothing per node changes.
      Tests: the first-iteration guard, the mate stop at a fixed node budget
      (fires after three stable mate iterations, not under `go infinite`),
      the fail-high stop on a constructed window, the horizon at 500 and
      1,000 ms, the injected late `go` stamp at 150 ms, SMP inertness at one
      thread. Instrumentation behind `diag`: counters for the two stops and
      the guard; a per-iteration log of `k`, the effort share and each
      factor for the activation read. Cheap qualification: the fingerprint;
      the timed probe (maintainer-run, idle host, about eight minutes).
      Non-goals: the aspiration loop (RAR-S89, RAR-S17), the thread vote
      (D.2), ponder, `nodestime`, the `movestogo` branch, the harness
      margin, Stockfish's `originalTimeAdjust` and time-disadvantage scale
      and `increaseDepth` (their triggers are in the packet).
      **Implemented 2026-10-22 on `d1`, `IMPLEMENTED`** (engine commit
      `5fec0d7`; `d829a8b` before it states the preconditions of two
      existing tests that the guard and the horizon change, and passes on
      the unchanged engine; tooling `2811a8a`). At the seeds, in
      `src/search/time.rs` (`sudden_death_horizon`, `MateWatch`,
      `soft_stop`) and both root loops of `src/search/mod.rs`. The five
      coordinates are in `SearchParams` with ranges `TmHorizonPerSec`
      20–100, `TmStabBase` 0.80–2.00, `TmStabSlope` 0–0.20, `TmStabFloor`
      0.40–1.00 (the packet set no stability ranges; these are the
      implementation's), `TmEffortFloor` 0–0.95, and `TmEffortHigh` widened
      to 0.6–2.0. What the handoff left open:
      - The two stops are the main thread's decision and end a pool's
        search directly; every thread's own soft target carries the new
        factors, and the vote is unchanged. D.1.5's 4T read covers this.
      - MultiPV takes the mate stop on line 1, not the fail-high stop, which
        would end a depth before its other lines are searched.
      - The guard holds the hard stop while the first iteration runs
        (`root_depth ≤ 1`).
      - The effort share is the best move's `RootMove.nodes` over the
        thread's nodes. The per-iteration counters went, with their one
        root-only store in `negamax`.
      - A fail-high after a fail-low can have a lower bound under the
        previous iteration's score. The move is still not a discard: the
        bound is above every move's bound from that fail-low.

      Verification on the final state, exit status read directly:
      `cargo fmt --check` 0; `cargo clippy --all-features --all-targets` 0,
      zero warnings; `cargo test --release` 0, 412 passed; `bench 13` on a
      fresh no-feature release build **13,987,223 / EBF 2.565**; the search
      tests under `diag` 0, 116 passed. `cargo test` in debug passed all
      411 on the same content before a line-ending-only change, then failed
      2 of 3 times on `first_kpk_search_in_a_fresh_process_costs_what_a_warm_one_does`
      (19 and 22 ms cold against 2 and 4 ms warm). That is a timing test of
      a `go depth 4` search the clock never reaches. The unchanged engine
      fails it at the same rate when `tt_sizing` runs just before it (1 of
      5 on each tree), and it passes 11 of 11 alone on the cluster: a
      load flake that predates the cluster. Each new test fails when its own
      mechanism is disabled (four mutations: fail-high, mate, guard,
      horizon). Instrumentation: `diag` counters `tm_mate_stop`,
      `tm_fail_high_stop` and `tm_first_iteration_guard`, and an
      `info string tm` line per completed iteration, read by
      `tools/diag/tm_activation.py` over B.5's 300 positions at `3000+30`.

      **Instrument findings for D.1.4 and D.1.5:**
      1. `TmOptScale`'s range was 0.5–2.0, so the wire check's 0.3 would have
         been clamped to 0.5 without a word. Colosseum's guard checks option
         names, not values. The lower bound is now 0.2.
      2. `tm_profile.py` profiles by PGN name, and a one-binary categorical
         match names both players alike, so it would pool the arms.
         `analysis/artifacts/d13-clock-cluster-2026-10-22/tm_wire_read.py`
         (SHA-256 `23ef9c7bcc360cda…`) relabels each game by arm from
         `games.jsonl`, checking each game's PGN hash, then runs
         `tm_profile.py` unchanged. Checked on `b32-cat-both`: 2,000 games,
         both arms 4.26 s a game.
      3. `tm_profile.py` and `tm_table.py` transcribe the fifty-move
         horizon, so they overstate the candidate's maximum on sub-second
         moves. D.1.5's "≥ 0.9 × the maximum" read must transcribe the
         contraction first. The probe's 60 s and 180 s clocks are
         unaffected.
      4. The registered fallback (mechanisms 1, 4, 5 and 6 without 2 and 3)
         cannot be set by options alone. Stability is neutral at
         `TmStabBase` = `TmStabFloor` = 1.0 with `TmStabSlope` 0, but moving
         the effort share to the whole search is code. The fallback arm
         needs the per-iteration share back, as a recipe patch.

      **`LOCAL_QUALIFIED` 2026-10-22, DONE.** The maintainer's PGO build,
      `tools/test_engines/rarog-d13cluster-pext-pgo.exe` (SHA-256
      `63d49fdcbc0824c6…`), was built from a clean tree at `da2df4e`, which
      has the same engine inputs as `5fec0d7`; it benches 13,987,223 / EBF
      2.565. The timed probe,
      `analysis/artifacts/d13-clock-cluster-2026-10-22/tm_rec_probe_d13.py`
      (SHA-256 `53a42b924714eea8…`), is the 2026-10-06 method pointed at
      that build. It ran on an idle host (3.5% CPU over 20 s) and wrote
      `tm_rec_probe_d13.txt` (SHA-256 `c33fef86716335a9…`). **Verdict PASS:**
      no rec move reached the maximum, and rec2 finished under 2 s.

      | Clock | rec1 | rec2 | rec3 |
      |---|---|---|---|
      | `60000+600` | 4,079 ms, 0.87 × optimum, d18, exact | 5 ms, d10, mate 6 | 2,061 ms, 0.47 ×, d21, lowerbound |
      | `180000+2000` | 8,585 ms, 0.57 ×, d19, lowerbound | 5 ms, d10, mate 6 | 12,821 ms, 0.90 ×, d21, lowerbound |

      On 2026-10-06 the B.9 head spent 32.3, 31.6 and 10.4 s at the short
      clock and 104.4, 102.2 and 10.4 s at the long one. Stockfish 19 spent
      1.5–45.6 s on the same three.
      - Three of the four non-mate moves ended on the fail-high stop's
        `lowerbound` line, and every one ended under its optimum.
      - The packet's "rec1 and rec3 within about 1.5 × their soft target"
        cannot be read from this output, which prints no soft target. It is
        D.1.5's to read at the fitted values.
      - rec2 stopped at depth 10 on a mate in 6 that held three iterations.
        Stockfish reports mate in 5. The stop plays a proved mate, not the
        shortest; each later move searches again from a mate at most one
        move shorter. The packet allowed this (`mate_in(3)`, the donor's
        rule, would not have stopped rec2 at all).

      This is the cheap qualification at the seeds. D.1.2.1 still ticks on
      D.1.5's probe at the fitted values and on its profile read.
      **Addendum 2026-10-22 (maintainer decision): a seventh mechanism, the
      cross-move score trend** both donors carry (RAR-R15, *Amendment*).
      Each thread keeps the root score its previous search in the game ended
      on, cleared by `ucinewgame`, and the falling eval adds `TmFallPrevSlope
      × (the previous search's score − this iteration's)` inside the same
      clamps; seed 0.0204 (Stockfish's raw slope, a foreign unit), range
      0–0.1. Zero-game screen on 1,600 wire-check games: the score moves
      18 cp at the median across a move pair, and the term lifts the factor
      off its clamp on 20% of moves. Engine `1f185cb`, clock-only;
      verification on the final state, exit status read directly:
      `cargo fmt --check` 0, `cargo clippy --all-features --all-targets` 0 at
      zero warnings, `cargo test` 0 and `cargo test --release` 0, `bench 13`
      on a fresh no-feature release build **13,987,223 / EBF 2.565**. Its
      wire check and the thirteenth coordinate are D.1.4's.
      **The fail-high stop retired 2026-10-23** (RAR-R18, maintainer
      decision): off by default (`a0f9755`, after `ec416d4` had moved it to
      twice the target); the cascades it capped are D.1.6's, in the search,
      and the spin goes when that fix lands. The other six mechanisms stay
      at their seeds for D.1.5's closing read.
    - **D.1.4 The clock fit: a wire check, an activation read, then SPSA blocks over the clock coordinates — `V`, DONE 2026-10-23.**
      Conditional (PLAN rule 4), with the zero-game sweep replaced by what
      a clock-only coordinate allows: (a) the wire check, `TmOptScale` at
      0.3 and each new coordinate at a range end in a 200-game match, each
      of which must move `tm_profile.py`'s time per game as the formula
      says (about −60% for the first); (b) the activation read from D.1.3's
      log on B.5's 300-position `3000+30` replay: a factor at one end on
      more than 90% of iterations drops its coordinate. Then one registered
      SPSA block of 60,000 games at `3+0.03`, Colosseum `spsa-tune`, over
      the twelve clock coordinates (the seven of 2.3.0 and the five new
      ones), seeded from the values above; the movement stop rule (rule 7c)
      ends it, and a second block runs only if block 1 moved a coordinate a
      full step and the rule did not fire. About ten hours a block.
      **(a) `TmOptScale` played 2026-10-22: the wire is live, but the
      registered size missed.** Maintainer-run: 200 games at `3+0.03`
      (`-CategoricalTuneBuild`, seed 20261022), on
      `tools/test_engines/rarog-d13cluster-tune.exe` (SHA-256
      `87c92774550c41d3…`, clean tree at `eff17c3`, bench 13,987,223). Arm A
      set `TmOptScale=3000`; arm B kept the seeds. The run is in
      `tools/results/d14-wire-optscale03`, with manifest
      `colosseum_match_d14-wire-optscale03_20261022_181558.manifest.txt`
      (SHA-256 `882daa78c8db131a…`): 0 faults, recount agrees. Read by
      `tm_wire_read.py`:
      - Time per game fell from 3.77 s to 2.92 s, **−22.5%**, against the
        registered "about −60%".
      - Time per move fell from 61 to 48 ms, −21%. In plies 0–19, where
        both arms' clocks are near 2.9 s, it fell from 86 to 39 ms, −55%.
      - From ply 80 arm A holds about twice the clock (2.3 s against
        1.1 s) and spends more per move than arm B. It ends draws with
        2.7 s left against 1.1 s.

      **Calibration:** the miss is in magnitude, through the mechanism. The
      budget is a share of the remaining clock, so time saved early buys
      time back later, and the expectation took the clock as fixed. The
      engine applies the value exactly (target over optimum 0.3000 of the
      seeds' at depth 1), and the harness and reader report the change move
      by move. So the instrument does what the check exists to prove. The
      plies 0–19 figure is a reading, not a pass rule written after the
      result. The coordinates still to check (each new one at a range end)
      state, before their games, whether their expectation is per game or
      per move at equal clock.

      The `−145.1 ± 31.5` Elo of arm A says only that a third of the budget
      costs a lot. On the arm at its seeds:
      - its clock floor had a median of 882 ms, p10 248;
      - no game went under 100 ms, and 16 of 200 went under 200 ms.

      That is a hint for D.1.5's floor prediction, not its read: the
      opponent was the same build at 0.3, and the sample is 200 games.

      Instrument notes:
      - `tm_profile.py` labels its 99.9th percentile as a second `p99`.
      - Its spent-over-optimum and ≥ 0.9 × maximum columns use the head's
        formula (D.1.3's finding 3).
      - The manifest's `harness_hold_ms:` reads p999 101 and max 878 ms,
        with 53 of 24,612 moves over the margin, both arms pooled. At these
        clocks no game was lost on time.

      **Registered 2026-10-22 as RAR-R15** (`docs/experiments/RAR-R15.md`),
      before any game of the remaining checks or of the block:
      - **(a) The five new coordinates' wire checks.** Each has a frozen
        formula expectation per move at equal clock, and a check is live
        past twice its SE. Two depart from "each at a range end in 200
        games", with reasons: `TmStabFloor` is checked at `TmStabSlope` 2000
        on both arms, because at the seed slope it binds on 5% of
        iterations and its check would be a null; `TmEffortFloor` plays
        800 games, because its −6% is 2.5 SE at 200. One batch runs them,
        `analysis/artifacts/d14-clock-fit-2026-10-22/run_wire_checks.ps1`.
      - **(b) The activation read, done:** 3,432 iterations, and no factor
        at one end on more than 90%. The effort factor is at `TmEffortHigh`
        on 87.5%, falling eval at its lower clamp on 66.9%, stability at
        k = 0 on 38.1% and never at its floor. All twelve coordinates stay.
        The rule's mapping from a pinned factor to "its coordinate" is
        undefined for multi-coordinate factors; it was not needed, and
        would have gone to the maintainer.
      - **(c) Block 1 of `d14block1`.** 2,000 × 30 games, seed 20261023;
        at most two blocks on rule 7c's count. The surface's ranges are
        the declared ones, except `TmOptScale` at 5000..20000. It runs
        only after all five wire checks read live.

      **Wire checks played 2026-10-22: four live, the horizon's not by its
      frozen rule** (RAR-R15, *Result of (a)*). The stability base, slope
      and floor and the effort floor moved time a move by +39%, −15%, +17%
      and −9%, each live and inside or past its band. The horizon moved the
      1000–2000 ms band −19% but the 500–1000 ms band only −2.3%, against
      "both bands ≥ 5% lower". The engine applies the coordinate exactly
      there (the optimum at 750 ms is 0.686 of the seed's, the formula's
      figure), and the clock floor rose (p10 146 → 242 ms, games under
      200 ms 35 → 11). Near 40 ms a move, time spent stops following the
      optimum. **Maintainer decision 2026-10-22: `TmHorizonPerSec` stays in
      the surface;** the miss is a calibration of the check's instrument
      assumption (a per-band read follows the optimum at every clock), not a
      pass, recorded in RAR-R15. An independent review the same day
      (RAR-R15, *Maintainer decision and review*) re-read D.1.3 from
      source, tests and artifacts and found no defect: the stops, the guard,
      the factors and the invariants are as recorded above, and the
      fail-high test fails under both mutations (the stop disabled, and the
      stop firing before the soft target). The surface stays at twelve: no
      clock constant outside the handoff becomes a coordinate, no search
      coordinate joins (an aspiration coordinate would move `bench 13` and
      D.1.5's attribution), and `TmStabFloor`, gradient-free at the seeds,
      stays with its noise walk quantified (0.41 steps SD a block) and a
      reading rule. **Block 1 is prepared, maintainer-run:** RAR-R15's
      command, dry-run through `colosseum.ps1` (12 coordinates, 2,000 × 30,
      `r-end` 0.0031, no rail warning, 15 slots, Hash 64, Threads 1,
      MultiPV 1), about 8–11 hours. The morning read is
      `python tools/spsa_block_rule.py d14block1 tools/results/d14-spsa-block1`
      and the manifest's `arm_clamps:` line. **Amended 2026-10-22
      (maintainer decision), before any game:** the cross-move score trend
      joins the cluster (D.1.3's addendum) and the surface, thirteen
      coordinates (`TmFallPrevSlope`, seed 204, range 0..1000, step 63),
      regenerated and audited; its wire check (`TmFallPrevSlope=1000` against
      the seeds, 200 games, live if plies 0–19 spend ≥ 5% more a move;
      formula ×1.36) ran before the block on the new tune build,
      `tools/test_engines/rarog-d13cluster7-tune.exe` (built clean at `2b9831f`, engine
      inputs `1f185cb`'s, bench 13,987,223), which the block's `-Engine`
      and `-ExpectRevision` name: **live, ×1.163 in plies 0–19** (about six
      SE; RAR-R15, *Result of the cross-move wire check*), 0 time losses.
      Block 1 at thirteen is dry-run and prepared, maintainer-run.
      **Block 1 played 2026-10-22/23** (60,000 games in 11 h 40 m, 0
      faults, no clamp; RAR-R15, *Result of (c)*).
      - Two of thirteen coordinates moved a full step: `TmOptScale` 10000 →
        11687 (+1.80 steps, level by iteration 1,500) and `TmStabSlope` 440
        → 302 (−1.10, still travelling). The rule says **STOP**, so theta is
        block 1's rounded centres.
      - Every move past noise is toward spending more time:
        `TmOptScale × TmEffortHigh` is ×1.233.
      - The chained launch started block 2 and stopped it after 90 games. A
        cut block never counts. Resuming it (11.6 h) is the maintainer's
        call; the agent recommends against.

      **Closed 2026-10-23 on the maintainer's decision.** Theta is block 1's
      rounded centres, and block 2 is abandoned. Another clock fit is
      registered only if D.1.5 accepts H1 at +10 Elo or more, from the gated
      head (RAR-R15, *Maintainer decision 2026-10-23*).
    - **D.1.5 The clock gate, D.1's closing read after D.1.6: the clock pieces at their seeds against the 2.6.0 engine with the cascade fix; harm reads at `3+0.03` and `10+0.1`, a four-thread read, the probe and the profile — `V`, DONE 2026-10-26.**
      **Done 2026-10-26 (RAR-R28): no harm.**
      - `d1`'s head against the 2.6.0 engine read +1.2 ± 8.9 Elo at
        `3+0.03` and −0.2 ± 7.8 at `10+0.1` (2,000 games each), and
        −8.0 ± 11.3 at four threads. 0 faults, and 3–5% less time a
        move.
      - The probe passes, with rec2 in 5 ms where 2.6.0 runs to the
        maximum.
      - D.1's clock work is kept for 2.7.0. D.1.2.2 ticks.
      - D.1.2.1 misses its won share at ≥ 0.9 × the maximum (0.24%,
        0.85%; every game won) and returns to research.

      The history follows.
      Registered as its own ledger entry before any game, with the packet's
      prediction re-frozen at the fitted values: the fitted cluster against
      the head, Colosseum `sprt-default`, `[0,3]` nElo, `3+0.03`, one
      thread, cap 20,000 pairs (about seven hours; a true +6 nElo accepts
      near 26,000 games); then 1,000 games at `10+0.1` and 1,000 at four
      threads as direction reads (about 0.6 and 0.8 hours). Acceptance adds
      D.1.2.1's probe and D.1.2.2's profile and forfeit reads. H1 bakes the
      fitted values, ticks D.1.2.1, D.1.2.2, D.1.2 and D.1, re-declares the
      fingerprint (unchanged) and writes the user-facing changelog entry.
      H0 returns the cluster to research with its estimate; the cap bakes
      nothing and records "unresolved". **Fallback, registered now:** on
      H0 or the cap, mechanisms 1, 4, 5 and 6 alone at their seeds are
      offered under AGENTS' repair case 2 (a 2,000-game read with a harm
      rule at −13 nElo), each expected near zero and bought for
      correctness; the maintainer decides whether to run it.
      **Registered 2026-10-23 as RAR-R16** (`docs/experiments/RAR-R16.md`),
      before any game:
      - **The arms.** The candidate is `rarog-d15fit-pext-pgo.exe`, block
        1's theta baked on the throwaway arm `d15-arm` (082a780, recipe
        `analysis/arm_patches/082a780-rar-r16-d15gate.patch`), version
        `2.7.0-dev+d15fit`. The head is `rarog-d15head-pext-pgo.exe` at
        `468ed7b`. Both are clean PGO builds and both bench 13,987,223.
      - **One batch runs everything,**
        `analysis/artifacts/d15-clock-gate-2026-10-23/run_d15.ps1`: the
        SPRT, the two direction reads, the probe, then the profile reads by
        `tm_profile_d15.py`. That reader is `tm_profile.py` with the
        candidate's horizon and p99.9 fixed (D.1.3's finding 3), checked
        against `tm_profile.txt`. All three matches are dry-run clean.
      - **The four-thread read** carries `--cores-per-game 4`. Without it
        the runner gave a game one physical core and no check refused it;
        `colosseum.ps1` refuses that now (`6c3a1a1`).
      - **The prediction, re-frozen:** +2 ± 4 Elo; H1 0.30, cap 0.50, H0
        0.20.

      **Before the SPRT, 2026-10-23** (RAR-R16, *Results before the
      SPRT*):
      - **The wire rule is corrected.** As registered it compared the
        candidate's time with the head's, but the fit's change is against
        the cluster's seeds; it was replaced before any SPRT game (RAR-R16,
        *Amendment*). The replacement check against a seeds PGO read
        **live, ×1.21 a move in plies 0–19**.
      - **Direction reads:** `10+0.1` −6.3 ± 11.7 Elo, four threads −10.4 ±
        11.6. They decide nothing; at four threads the clock-floor median
        rose from 151 to 353 ms and games under 100 ms fell from 39 to 0.
      - **Probe PASS.**
      - The SPRT is next, maintainer-run.

      **The SPRT is paused at 1,782 pairs (+0.2 ± 6.6 Elo, LLR −0.10,
      unresolved), and RAR-R17's decomposition has played, 2,000 games
      each:**
      - **The cluster at its seeds against the head: −14.9 ± 8.8 Elo.** It
        spends 7.4% less time a move at every stage and ends games with
        clock unused. The likeliest source is the fail-high stop cutting
        the head's overruns.
      - **The horizon at 100 against 53: −0.9 ± 8.0.** No late-game cost
        shown, and it keeps all 2,000 games above 100 ms of clock (443 at
        100 did not).
      - The fit bought back roughly what the seeds lost.
      - The research splits the mechanisms next, **the fail-high stop
        first** (a switch). The registered fallback should not run in its
        form, which keeps that stop at its seeds.
      - Maintainer requirement: any fallback shows no harm at `10+0.1` as
        well.

      **Research 2026-10-23, `MORE_RESEARCH` (RAR-R18,
      `docs/experiments/RAR-R18.md`).** Every read so far sits on the time
      rule (2 Elo per 1% of thinking time at `3+0.03`; RAR-S98, RAR-S99):
      the seeds' −14.9 against −14.8 for their −7.4% of time; the fit's
      +0.6 (the paused SPRT, 2,345 pairs) against −5.0 for −2.5%; the
      `10+0.1` and four-thread reads within one SE. Pooled, the cluster
      changes decision quality by **+1.6 ± 4.2 Elo**: it moves time. A
      replay on the switch build (`TmFailHighTarget`, engine `f3549ff`, a
      tune-only spin with play unchanged by default; bench 13 unchanged)
      shows the fail-high stop ending 29% of first-move searches at a
      median 1.18 × the target and never past 3 ×, removing about 10% of
      the spend: single fail-highs, not cascades. At twice the target it
      fires on under 1% and the spend is the no-stop level. The stability
      factor adds about 12%. Three 2,000-game categorical reads on that
      build (`stopoff`, `cascade` at 2 ×, `factorsoff` with the stop off on
      both arms) and a probe at 2 ×, frozen with the time rule as their
      null, decide the kept set: the horizon, the mate stop and the guard
      stay; the stop moves to the threshold the reads choose; the stability
      factor and the cross-move slope stay only if they decide better than
      their time. The kept set then reads as one repair (AGENTS' case 2) at
      `3+0.03` and `10+0.1` before anything lands. The research recommends
      recording RAR-R16's SPRT as unresolved, and against Reckless's shape
      or a respending budget now.
      **RAR-R18 read 2026-10-23** (2,000 games each, probe PASS): removing
      the stop **+15.5 ± 9.3** against the seeds, the stop at twice the
      target **+12.2 ± 8.8**, the factors **+6.9 ± 8.7** for keeping them.
      The stop at the target costs about 15 Elo at `3+0.03` and leaves its
      saving on the clock; at twice the target the cost is gone and the
      cascades still end (rec1 at `180000+2000` in 18.9 s against the
      head's 104 s). The time rule misread the reads: between two arms of
      one clock the whole-game time is conserved, so the rule saw
      redistributions as decisions (calibration in RAR-R18). Frozen
      dispositions: the stop to twice the target (landed on `d1` as
      `ec416d4`, its test precondition `7a344f5`; bench 13 unchanged); the
      stability factor and cross-move slope removed as insufficient; the kept set to one repair
      read at both controls. Additive over the reads the kept set is
      −9.6 ± 15.2 against the head without the factors and −2.7 ± 12.4
      with them, so their place in the repair arm is the maintainer's
      decision, recorded there.
      **Maintainer decision 2026-10-23: the correct fix, in the search.**
      The cascade is a search property (full-depth re-searches after a root
      fail-high) and is fixed there, by **D.1.6**, not capped by the clock:
      the fail-high stop is retired (off by default, `a0f9755`; the spin stays
      as D.1.6's control arm and goes with it). The other six mechanisms
      stay on `d1` at their seeds: the horizon, the mate stop and the guard
      as clock answers to clock problems; the stability factor, the effort
      band over the whole search and the cross-move trend on their lean
      (+6.9 ± 8.7, both donors), the fit's block 1 unused since it was
      fitted around the retired stop. **This leaf is now D.1's closing
      read, after D.1.6 lands:** `d1`'s head against the 2.6.0 engine plus
      the cascade fix, registered then under AGENTS' repair case 2 with no
      Elo claim: 2,000 games at `3+0.03` and 2,000 at `10+0.1`, harm rule
      −13 nElo on each; 1,000 games at four threads as a direction read;
      the probe and the profile reads that tick D.1.2.1 (rec2 under 2 s,
      the won and mate shares below 0.2%) and D.1.2.2 (no time loss, the
      clock floor). A pass lands the clock pieces in 2.7.0 as a
      correctness change; a fail lands none of D.1's clock work. RAR-R16's
      SPRT ended unresolved at 2,345 pairs (+0.6 ± 5.8 Elo, LLR −0.06):
      its arm, the fit over the stop at the target, no longer exists.
      **The clock pieces alone, registered 2026-10-24 as RAR-R24**, before
      any game:
      - **Why:** RAR-R22's addendum found that this read had never been
        run, and that `d1`'s clock reaches the hard maximum in about twice
        as many `3+0.03` games as 2.6.0's. Every D.1.6 arm carried that
        clock.
      - **The arms:** `rarog-d1head-pext-pgo.exe` (`3f4dcf4`, the `d1`
        defaults, bench 13,987,223) against `rarog-d15head-pext-pgo.exe`
        (`468ed7b`).
      - **The reads:** 2,000 games at `3+0.03` and 1,000 at `10+0.1`, harm
        rules −13 and −18 nElo, and `stall_scan2.py`'s maximum-hitting rate
        per side.
      - **Predictions:** 0 ± 14 and −3 ± 16 nElo.
      - **Command and order:** `run_r24.ps1 -Read stc|ltc|stalls`, dry-run
        clean. It informs this leaf and does not close it; the closing read
        above stays. Both arms are fixed binaries, so either order against
        RAR-R23 attributes cleanly; the maintainer ran this one first.
      **Read 2026-10-24, no harm at either control** (RAR-R24):
      - `3+0.03`: +8.0 ± 8.9 Elo (+13.7 ± 15.2 nElo).
      - `10+0.1`: +14.3 ± 11.2 Elo (+27.3 ± 21.5 nElo).
      - 0 faults and 0 time losses. d1head spends 3–4% less a move.
      - **Stalls:** d1head runs to the maximum in 38.0% of `3+0.03` games
        against 2.6.0's 20.5%, early and at balanced scores. That
        reproduces RAR-R22's addendum between PGO builds, with no measured
        cost.
      - **Disposition:** the clock pieces stay on `d1` at their seeds.
        RAR-R22's −7.4 is not the clock's. A harm read accepts no gain, so
        this leaf's closing read after D.1.6 stands.
      **The closing read, registered 2026-10-26 as RAR-R28**, before any
      game:
      - **Arms:** `d1`'s head `rarog-d1head3-pext-pgo.exe` (`6fcfac2`,
        bench 13,987,223: the clock pieces at their seeds and the stop at
        twice the target) against the 2.6.0 engine.
      - **Reads:** 2,000 games at `3+0.03` and 2,000 at `10+0.1` with −13
        nElo harm rules; 1,000 games at four threads as a direction read;
        the probe on rec1–rec3; `tm_profile_d15.py` at horizon 50.
      - **The ticks**, as D.1.2.1 and D.1.2.2 state them.
      - **Predictions:** +12 ± 14 and +20 ± 16 nElo; four threads −2 ± 15
        Elo.
      - **Command:** `run_r28.ps1 -Read probe|stc|ltc|4t|profile`, all
        three matches dry-run clean; maintainer-run.
    - **D.1.6 The root fail-high cascade in won endings: researched in the search, capped by the stop at twice the target — `R3` research, `I2`/`V` sub-steps, DONE 2026-10-26.**
      **Closed 2026-10-26.**
      - **The stalls end:** the fail-high stop at twice the soft target is
        `d1`'s default (`a79efe4`). It read no harm against off (RAR-R25:
        +4.3 ± 8.3 Elo at `3+0.03`, +4.9 ± 9.8 at `10+0.1`), and on the
        PGO build it ends every probed stall under the maximum (RAR-R27).
      - **No repair in the search or evaluation lands:** RAR-R22 H0,
        RAR-R23 harm at `10+0.1`, RAR-R27 H0 at −1.2 ± 3.0.
      - **The cause is recorded, not fixed:** an uncapped LMR correction
        term drives the won-ending tree's growth and most deeper searches
        everywhere (RAR-R26).
      - **Cleanup:** the research instruments are removed (`42b001f`),
        and the fingerprint stays 13,987,223.
      - **Retry trigger:** an LMR refit with the correction slope free.

      The history follows.
      Rarog re-searches every root fail-high at full depth, so a cascade of
      k fail-highs costs k full iterations: rec1 ran nine re-searches to the
      hard maximum (32 s at `60000+600`, 104 s at `180000+2000`; RAR-R13),
      rec3 to about four times its target. Both donors make the re-searches
      cheaper instead of stopping them: Stockfish searches each re-search at
      the root depth less the count of fail-highs so far, and stops
      deepening once half the target is spent; Reckless re-searches one ply
      shallower. D.1.3's clock stop capped the symptom and cost about 15 Elo
      at `3+0.03` at the target (RAR-R18): time spent on fail-highs is well
      spent here, which is the evidence that makes this a gate, not a
      probe. This is not the aspiration window's growth, which RAR-S89 and
      RAR-S17 refused; it is a different device, untested here.
      Interactions: the aspiration constants were fitted in C.10 around
      full-depth re-searches and may owe a refit; the clock's factors read
      iteration completion; the TT carries a shallower re-search's results
      into the next iteration. It moves `bench 13`, so the fingerprint is
      re-declared when it lands, and it changes shared search signals in
      the rated configuration, so its gate is the symmetric `[-5,5]` with
      its cap (AGENTS' repair case 3; about 12,000 games) on the 2.6.0
      engine, with no clock stop in either arm. Acceptance adds the probe
      with no clock stop: rec1 and rec3 under the hard maximum at both
      clocks. Maintainer decision 2026-10-23: the correct fix is wanted
      ahead of NNUE, at its price. If the gate refutes it, the fallback is
      the clock stop at twice the target behind its spin, read as RAR-R18
      read it.
      **Where it stands (2026-10-24).** The four-part cluster (RAR-R19,
      RAR-R20), fitted by RAR-R21 (block 1: the freeze later, the clock's
      scale up 8%, the squared divisor up 44%, nothing else moved), ends
      every probed stall (eight of eight rows) but its gate accepted H0:
      −7.4 ± 7.5 Elo at `3+0.03`, +2.1 ± 11.6 at `10+0.1` (RAR-R22). What
      is known about the cost: the evaluation half reads even at equal
      speed at every size; the in-game NPS loss is the tree's composition,
      not speed; the arm still spends 5.5% less a move after the fit; parts
      1–3 at equal time read −9.7 ± 8.9. **The level above (2026-10-24,
      RAR-R22's addendum):** d1's clock runs to the hard maximum in 41% of
      `3+0.03` games before ply 40, double the 2.6.0 head's 15–18%, so the
      gate's −7.4 sums the ungated clock pieces (D.1.5) and the cluster;
      the window term and the budget charge are global changes (32 of 40
      fresh-process bench positions move), the offset is confined to
      bare-king trees (13 of 40, all 19 pieces or fewer); a clock stop
      conditioned on the chosen move (Basilisk's) would fire on 24% of
      searches here and is rejected. **The re-scoped plan:** the confined
      cluster {`MopupKnownWin`, a known-win window floor
      `AspKnownWinWindow` active only when the centre is in the known-win
      band}. It has no clock piece and no budget charge. Its evidence was
      to be the probe and a fresh-process differential for correctness, a
      2,000-game equal-speed harm read on one tune build, and the `10+0.1`
      read (AGENTS' repair cases 1 and 2). D.1.5's harm read of the clock
      pieces alone against 2.6.0 is recommended first.
      **The floor refuted, zero games (2026-10-24, RAR-R23):**
      - The floor (`3f4dcf4`, tune-only, 0 = off) changes nothing measured:
        0 of 40 fresh-process bench positions, 0 of 2 won endings, and
        every probe node count at 1000, 2000 and 3000.
      - Its premise was a jump of about 5,800 cp as the offset enters the
        horizon. What rec1 shows instead is a 12 cp window walking a score
        that climbs from 750 to 3,314 cp, below the band, five to fifteen
        windows an iteration. Only the global window term shortens that
        walk.
      - The offset alone keeps every probe row under the maximum and
        changes 13 of 40 positions, all with 19 pieces or fewer.
      - So **the repair registered is the offset alone** (RAR-R23), with
        the floor at 0. The window walk below the band is a search
        question outside D.1.6.
      - D.1.5's harm read of the clock pieces alone is registered as
        RAR-R24.
      **Both read 2026-10-24.**
      - **RAR-R24, no harm:** d1's clock against 2.6.0 read +8.0 ± 8.9 and
        +14.3 ± 11.2 Elo, so the clock is not RAR-R22's cost.
      - **RAR-R23 fails:** the offset alone reads −4.4 ± 15.2 nElo at
        `3+0.03` and **−25.0 ± 21.5 at `10+0.1`**, below the −18 harm
        threshold, with the wire live and time a move equal. Nothing lands,
        and the leaf is back in RESEARCH.
      - **Maintainer decisions 2026-10-25:**
        - Research the cause rather than close `NO_CHANGE`.
        - The fallback stop at twice the target is open to re-evaluation if
          it does not cost strength. It is a separate read, a cap and not a
          fix; the card below is the fix.
        - That read is registered as **RAR-R25**: one tune build, the stop
          at twice the target against off, the probe, 2,000 games at
          `3+0.03` and 1,000 at `10+0.1`, harm rules −13 and −18 nElo.
          Both matches dry-ran clean. A pass makes it the default as an
          interim cap, re-read when the search fix lands.
        - **RAR-R25 read 2026-10-25: PASS.**
          - The probe ended rec1 and rec3 at `180000+2000` in 19.7 s and
            15.7 s, against 43.2 s and the hard maximum off.
          - `3+0.03` read +4.3 ± 8.3 Elo (+8.0 nElo), and `10+0.1` +4.9 ±
            9.8 (+10.7 nElo). 0 faults.
          - The landing is the default `TmFailHighTarget` 20000 in its own
            engine commit, with the changelog entry, at an unchanged
            fingerprint. It is not yet made.
          - D.1.2.1's cascade half is met by the probe; its rec2 half stays
            with D.1.5's proved-mate stop.

      The fit arm's lesson stands: a refused SPSA step and a 2-second idle
      window are preflight matters, not results.
      **Research card (2026-10-25): the won-ending tree, not the window or
      the clock.**
      - **Mechanism.** In rec1 Rarog's tree grows far faster per ply than a
        search that converts the win:
        - Rarog at 200 M nodes reaches depth 16–20 and proves no mate
          (RAR-R23's traces).
        - Basilisk's current head, with its clock stop removed, proves mate
          in 25 at depth 26 in 73 M nodes. That is 0.56 × its maximum at
          `60000+600` and 0.20 × at `180000+2000`; its 2026-10-24 probe is
          in `D:\code\basilisk\tools\results\stall-probe-20261024\`.
        - Stockfish's search over Rarog's evaluation reaches depth 34 for
          about the same nodes (RAR-R19).
        - The window walk RAR-R23 found is the symptom: the root score keeps
          rising because each deeper tree finds more, and a 12 cp window
          chases it.
      - **The measured drivers** (RAR-R19, rec1 at depth 18, 19.1 M nodes
        and selective depth 89 at the defaults):
        - with LMR's correction term at 0: 4.1 M nodes, selective depth 39;
        - with no reduced move searched deeper than its nominal depth:
          3.2 M and 40;
        - with at most 2 plies deeper: 2.7 M and 51.
        - On bench, 443,080 of 4,216,025 reduced searches (10.5%) run
          deeper than nominal, by 1.7 plies on average, outside the line
          budget. The cap at 0 halves bench and the cap at 2 takes 21% off
          it, so the deeper ceiling is a global setting. Neither has been
          played in games.
      - **A difference from Basilisk, read in both sources.**
        - Basilisk caps its LMR correction term at 2,695, about 2.6 plies,
          "the donor's largest value of the term"
          (`search_kernel_core.cpp`).
        - Rarog's term is `lmr_correction * |correction| / 1024` with no
          cap (`late_move_reduction`, `src/search/node.rs`): about 0.33 ply
          a 100 cp of correction at the default 3330.
        - So a large correction turns a reduction negative, and the reduced
          move is searched up to 4 plies deeper, past the line budget.
          Basilisk allows 2 plus 1 at a PV node.
      - **Competing hypotheses.**
        - **H1:** the uncapped correction term in LMR drives the won-ending
          explosion. A cap would be confined to nodes with a large
          correction.
        - **H2:** the deeper-search ceiling (+4 against Basilisk's +2/+3)
          drives it, and is a global change.
        - **H3:** the window walk is a cause of its own. Weakened, because
          it shrinks wherever the tree does.
        - **H4:** the evaluation's flatness. Refuted as a net fix by
          RAR-R23.
      - **Interactions.**
        - C.10 fitted the LMR terms and their correction slope around the
          uncapped term and the +4 ceiling.
        - Correction history also feeds the singular margins
          (`sing_double_corr`, `sing_triple_corr`) and the static
          evaluation.
        - The line budget's invariant is stated in source and broken by the
          deeper plies (`CoreLmrDeeperBudget` restores it).
        - The aspiration constants were fitted around the present tree.
      - **Invariants.**
        - A cap leaves every node with a correction under it unchanged,
          checked by a fresh-process differential.
        - The fingerprint moves and is re-declared on landing.
        - No clock change is part of this card.
      - **Cheapest discriminating tests** (zero games; they need an idle
        host and wait for Basilisk's runs to finish):
        1. A diag histogram of the correction term's size at LMR nodes, on
           bench and on rec1/rec3. It says whether large terms are rare in
           ordinary positions, so whether a cap is confined.
        2. A tune-only cap on the term (0 = today), and node traces of rec1
           and rec3 at 200 M nodes for a few cap values, with and without
           the deeper ceiling at 2. Does rec1 reach a proved mate, as
           Basilisk's does?
        3. A fresh differential over bench for the chosen setting.
      - **Falsifier and stop rule.**
        - If no cap value ends rec1 and rec3 under the maximum in the probe
          without the window term, H1 is false. H2 is then read the same
          way.
        - If what ends them changes more than half of the bench positions,
          it is a global search change. Its read is an equal-speed
          2,000-game harm read first, and then the `[-5,5]` gate.
        - The line stops when that read is below −13 nElo or no setting
          ends the stalls.
      - **Why it could pay for itself.** If the escape is general, as 10.5%
        of bench's reduced searches suggests, a fix that keeps lines inside
        their budget could gain where the stalls themselves cost nothing.
        Only games can say.
      - **Read 2026-10-25, zero games** (instruments `c4da889`; evidence in
        `analysis/artifacts/d16-cascade-2026-10-23/tree-growth-2026-10-25/`,
        `RESULTS.md` against the frozen `PREDICTIONS.md`):
        - **The term at reductions.** It takes a ply or more off 19.2% of
          bench's reductions and 73.5% of rec1's. It drives 56.8% and
          91.4% of their deeper searches.
        - **Cap 1024** takes rec1 from depth 20 to 26 and rec3 from 21 to
          24 inside 200 M nodes. With the term at 0 they reach 35 and 28.
          Caps of 2048 and 3072 do not help, and neither does the deeper
          ceiling at 2.
        - **But cap 1024 changes 24 of 40 bench positions**, 32-piece
          openings among them (bench 11,480,295).
        - **No configuration proves a mate in the budget.** Each ends in
          the window walk as the win resolves, which the stop at twice the
          target now caps in play.
        - **Reading:** H1 is supported as the driver, but the term drives
          ordinary positions too, so the working cap is a global search
          change. H2 is refuted. Predictions 2 and 3 held; 1 and 4 missed,
          in mechanism.
        - **By the stop rule,** the cap is read at equal speed first:
          **RAR-R26**, registered and dry-run clean.
      - **RAR-R26 read 2026-10-25: no harm.**
        - `3+0.03` −1.4 ± 9.0 Elo, `10+0.1` +1.7 ± 11.5, 0 faults. The cap
          searched 0.09 and 0.24 ply deeper at equal time.
        - **The timed probe, run before any gate:**
          - The cap alone ends rec3's single long window at `180000+2000`
            (5.9 s, against the maximum uncapped). That is the case the
            stop cannot reach.
          - But it takes rec1 into the window walk at the win's resolution
            sooner, and that walk runs to the maximum without a stop.
          - With the stop at twice the target, every row ends at or under
            its optimum: rec1 0.78 × and rec3 0.42 × at `180000+2000`,
            against 1.30 × and 1.10 × for the stop alone.
        - **So the fix is the pair:** the cap for the tree's growth, the
          stop for the walk once the win resolves.
        - **Next:** the gate, a PGO arm with the cap baked against `d1`'s
          head (which has the stop), `[-5,5]` with its cap, the probe on
          the pair. Registered before any game; maintainer-run overnight.
        - **Registered 2026-10-25 as RAR-R27:**
          - **Arm:** 7703768 on the throwaway branch `d16-gate3`
            (recipe `analysis/arm_patches/7703768-rar-r27-d16gate3.patch`,
            `rarog-d16gate3-pext-pgo.exe`, bench 11,480,295).
          - **Head:** `rarog-d1head2-pext-pgo.exe` at `6253027`, bench
            13,987,223.
          - **Reads:** sprt-repair at `3+0.03`, cap 20,000 pairs, seed
            20261111; 1,000 games at `10+0.1`; the probe on both binaries.
          - **Prediction:** −1 ± 6 Elo.
          - **Dispositions:** H1 with the probe passing lands the cap and
            the cleanup; H0 closes D.1.6 on the stop.
        - **RAR-R27 read 2026-10-26: H0**, −1.2 ± 3.0 Elo; `10+0.1` +1.7 ±
          11.4; the probe PASS on both binaries. The cap does not land.
          D.1.6 closes on the stop at twice the target after D.1.6.2's
          cleanup. Retry trigger: an LMR refit with the correction slope
          free.
        - **D.1.6.1 Research: the cascade's mechanism in Rarog's loop, the donors' devices, the cheapest prototype and its zero-game reads, the registration — `R3`, DONE 2026-10-23.**
          The research card (mechanism, interactions, invariants, falsifier)
          per AGENTS, from the diag traces of rec1 and rec3 (RAR-R13's
          method) and the activation replay (29% of first-move searches
          fail high past the target). A prototype behind a tune switch (the
          re-search's depth reduction as a coordinate, 0 = today) is allowed
          for the zero-game reads: the probe with the clock stop off, the
          bench and counter differential against the head
          (`tools/diag/bench_counters.py`, `phase4_differential.py`) and the
          fail-high rate per iteration. Ends `READY_FOR_IMPLEMENTATION` with
          the device, its seeds, the gate's registration and frozen
          prediction, or `NO_CHANGE` with the evidence.
          **Read 2026-10-23, `MORE_RESEARCH` (RAR-R19,
          `docs/experiments/RAR-R19.md`).** On the 2.6.0 engine every root
          fail-high returns exactly its beta, so a cascade's length is set
          by the window schedule alone (deltas 12, 27, 52, 93, 161, 273,
          458, 763 …; re-centring on the failing score would save at most
          one re-search) and each re-search costs more than the last: rec1's
          depth 19 runs 7 fail-highs and 75 M nodes without completing,
          rec3's depth 21 five and 18.5 M. The device is one ply off a root
          window search per fail-high so far in the iteration, the exact
          window included, floor depth 1, reset each iteration, the nominal
          depth reported and sampled (`AspFailHighReduce`, tune-only,
          default 0, engine `027a604`; bench at 0 unchanged). Reads: bench
          13 at 1 is 8,510,754 (0.61 ×; the fail-high count unchanged, the
          fail-high nodes −45%, and the exact window after fail-highs,
          searched k plies shallower, −49%); rec1's depth 19 completes in
          1 M nodes; the idle-host probe with no clock stop ends rec3 at
          0.48 × its optimum at both clocks where the 2.6.0 engine runs it to
          the hard maximum (30.3 s, 97.9 s), rec1 at 0.83 × at `180000+2000`
          (head 2.51 ×) and 2.70 × at `60000+600` (head 0.91 ×, its cascade
          completing just past the target); at `3000+30` +0.68 nominal ply
          for +6.9% first-move spend, fail-highs an iteration unchanged.
          **The same probe on the PGO arm (2d013c8, recipe `analysis/arm_patches/2d013c8-rar-r19-d16gate.patch`, 1.3 × faster) runs
          rec3 into the hard maximum at `180000+2000`:** it crosses the
          target one iteration later and depth 24's cascade costs more than
          the 90 s left. The frozen stop rule fires and the leaf stays in
          research. Why: a cascade's window count is the schedule's and
          unchanged (7, 7, 4, 4); in these endings a ply of depth saves about
          20% of a window, not the bench's 2.5 × (27 → 21 → 18 → 15 M nodes
          over four plies); and the next iteration's first window starts at
          full depth over a TT holding only the shallower result. The
          depth reduction is the search half of both donors' device; the
          other half is clock-coupled in both (Stockfish stops deepening at
          half the target and recovers depth over the next iterations).
          **Maintainer decision 2026-10-23:** complete the device, then gate
          it; the gate is not run on the partial device. **Node-indexed
          reads at 200 M nodes (RAR-R19):** with Stockfish's recovery as a
          second switch (`AspFailHighRecover`, engine commit named in
          RAR-R19) the costliest iteration is still over 150 M in both
          positions; with the window opened after three or four fail-highs
          (`AspMaxFails` 3, 4, alone and with the reduction) one open window
          costs 70–190 M. **Every root-loop device is refuted as the fix.**
          Depth for depth Rarog spends 7–57 × Stockfish 19's nodes on these
          positions: its root score rises with every ply (rec3 706 → 1149,
          rec1 799 → 2882) where Stockfish's is flat (484–573), and its
          lines reach the ply cap (seldepth 89 at depth 18 against 26). The
          cascade is the loop following a drifting score through a tree that
          grows to the cap; the cause is upstream of the root loop and is
          the re-scoped question: the static evaluation along the PV against
          the search score by depth, the extension and quiescence
          composition at stride 1, and Stockfish's classical evaluation in
          Rarog's search on these positions (RAR-O07's instrument). The gate
          (D.1.6.3) stays held; the research recommends the stop at twice
          the target (RAR-R18) as the clock's robustness piece meanwhile.
          **Maintainer decision 2026-10-23: the stop stays off; the
          research continues.** **Read 2026-10-23 (RAR-R19, zero games):**
          the oracle (Stockfish's search over Rarog's frozen evaluation)
          reaches depth 26 on rec1 in 3.1 M nodes with a flat score where
          Rarog spends 19 M on depth 18 and its score doubles, so the drift
          and the cost are the search, not the evaluation. In the cascade
          iteration a third of all reduced moves are searched deeper than
          their nominal depth (`reduced_depth`'s negative reductions, up to
          two plies above `new_depth` plus a PV node's two, outside the line
          budget; diag counters `d428c00`); zeroing the correction-history
          term alone shrinks rec1's depth 18 from 19 M to 4 M nodes,
          selective depth 89 to 39, score 1845 to 1184, while every other
          term moves the tree three to a hundred times at the ply scale.
          A tune-only cap on the deeper searches (`CoreLmrDeeperCap`, 4 =
          today, `ddbe1f2`) at 0 shrinks the
          fixed-depth trees three to six times and halves bench 13
          (6,639,253), but the explosion returns seven plies deeper when the
          search proves the win (rec3 depth 27: 100 M nodes). **Read one
          level further (RAR-R19, maintainer decision to continue):** the
          Rarog-against-oracle counter differential is 3.4 × on the
          standard suite and on these endings at depth 14 but 20.6 × at
          depth 18; the oracle to depth 34 proves rec1's win for the same
          order of nodes (100 M to score 2880 against Rarog's 154 M) in
          iterations that grow 1.3 × a ply, where Rarog's jump 26–245 × at
          the iteration that first resolves the win. A tune-only switch that
          charges deeper reduced searches to the line budget
          (`CoreLmrDeeperBudget`, `0366884`) restores the stated invariant
          (selective depth 127 → 52), reproduces the oracle's depth-18 score
          on rec1 at 7 × its nodes instead of 51 ×, and costs 30% of bench,
          but the lump remains: nine to eleven fail-high windows walking the
          schedule from 888 to 2818 in one iteration, because the root
          returns exactly beta on every fail-high (fail-hard in effect; the
          five fail-high interpolations toward beta are cleared, all at 0
          the root still returns the schedule's beta) where the oracle's
          fail-soft root and re-centring window climb the same range in one
          or two windows an iteration over eight iterations. **Read
          2026-10-23 (RAR-R19, maintainer decision to continue):** the
          fail-hardness is quiescence returning `alpha` on a fail-low (93%
          of bench's root fail-highs are exactly beta; ply-2 move-loop
          cutoffs two-thirds on the edge). Fail-soft quiescence
          (`QsFailSoft`) and re-centring (`AspRecentre`), the donors' two
          halves, together still walk eight or nine windows in the
          resolving iteration (the root's overshoot does not reach it;
          bench 1.32 ×, fail-soft alone 1.50 ×, re-centring alone 0.74 ×
          as a faster schedule). The line budget's size
          (`CoreExtensionBudgetEighths`) at half smooths rec3's score path
          (steps of 130 cp, largest iteration 86 M) but not rec1's, which
          finds its win later and still in one iteration (1274 → 7000,
          thirteen windows, over 180 M), at bench 0.50 ×. **Every
          single-device read is exhausted:** the stall is a won ending
          whose conversion is one forced line, seen all at once by the
          first iteration deep enough, and the window walking that jump in
          eight to thirteen bound-proofs. Open: a fit of the fail-high
          schedule under the bounded tree; the per-ply differential of the
          oracle's gradual score path against Rarog's step; the
          clock-coupled half. Strength candidates of their own for a
          `[-5,5]` gate: `CoreLmrDeeperBudget` (an invariant restored,
          bench 0.70 ×) and fail-soft quiescence (bench 1.50 ×). All
          switches on `d1` at their neutral defaults. **Read 2026-10-23
          (RAR-R19, maintainer decision to continue; the connected features
          against the donors, and the HCE):** the HCE values a won ending
          flat (K+R+B against K eight pawns; Stockfish 11's classical
          evaluation 55, a known win), so the search carries the win's value
          across the horizon and the score climbs only as deep lines reach
          the promotion or the mate; with Stockfish 11's own evaluation the
          oracle resolves rec1 at depth 21 for 0.7 M nodes. A known-win
          offset alone (`mopup_known_win`, an evaluation parameter, 0)
          makes rec1 worse (cascades of 7–16 windows, bench 1.29 ×) because
          the first window is 12 wide whatever the score; **with the
          donors' score-scaled first window** (`AspDeltaSquareDiv`,
          `ff14793`) **the cascade is gone:** every iteration closes in one
          or two windows and the score climbs gradually, the oracle's shape,
          at bench 0.73 ×. What remains is one window's cost at depth 20 and
          beyond in these endings (rec3's depth 21: a single exact window
          over 168 M nodes, 100 × the oracle's iteration), the tree's
          composition (less pruning per node, twice the singular attempts,
          effective depth twice nominal), a search cluster of its own. The
          candidate cluster for D.1.6.3: the known-win offset with the
          score-scaled window and the line budget, after a fit of the
          aspiration and deeper-search constants around them, gated `[-5,5]`
          with the probe; the evaluation half needs the maintainer's leave
          under the freeze. The clock-coupled half stays recommended for
          D.1.5 meanwhile. **Maintainer decision 2026-10-23: the whole
          fix, every part, now, verified.** The evaluation half is allowed;
          the fourth part is Stockfish's deepening freeze past half the
          target (`TmDepthFreeze`, `097ea5d`). The probe with no clock stop
          (RAR-R20): parts 1–3 end every probed stall under the maximum
          (rec3 at `180000+2000` 30.8 s against 98.0 s), all four end every
          move within its optimum (the PGO arm: rec3 2.5 s and 6.9 s, rec1
          2.3 s and 8.3 s, against the head's 30.3, 98.0, 4.4 and 37.5 s).
          **Done 2026-10-23, `READY_FOR_IMPLEMENTATION`:** the cluster, its
          arm 6fd195e (recipe `analysis/arm_patches/6fd195e-rar-r20-d16fix4.patch`) (bench 8,996,837), its probe, its wire check and its
          gate are RAR-R20's; the research recommends a fit of the
          aspiration and reduction constants around the cluster before the
          gate.
        - **D.1.6.2 Implementation: the cleanup, the unused D.1.6 instruments removed and the stop at twice the target kept — `I2`, DONE 2026-10-26.**
          **Done 2026-10-26 (`42b001f`):**
          - The 13 instrument commits from `027a604` to `c4da889` are
            reverted, `a79efe4` excepted: 680 lines removed, the stop
            untouched.
          - Bench is 13,987,223 / EBF 2.565 as before.
          - fmt is clean, clippy with all features and targets has 0
            warnings, and the tests pass (413 debug, 414 release, 0
            failed; the six removed were the instruments' own).
          - The `d16fix` SPSA group stays valid: `arms.txt` binds it to
            its own arm.
          **Re-scoped 2026-10-26 (RAR-R27, H0):** no repair lands. The step
          removes the D.1.6 instruments no default uses, as D.1.6.2's
          register row lists them. A coordinate whose default is today's
          behaviour returns to its constant, `TmFailHighTarget` stays at
          twice the target, and the fingerprint stays 13,987,223. It is
          verified as a behaviour-neutral change by that fingerprint plus
          the stop's test. The history follows.
          **Re-scoped 2026-10-24 to the offset alone (RAR-R23).** On its
          pass the landing on `d1` is:
          - `MopupKnownWin` 5826 as the default;
          - the fingerprint re-declared in GUIDE and the PLAN checkpoint:
            16,175,922 / EBF 2.580 on the tune build, with the landed
            production build's own bench recorded then;
          - `TmFailHighTarget` and its test deleted;
          - the unused instruments removed: `AspFailHighReduce`,
            `AspFailHighRecover`, `AspRecentre`, `QsFailSoft`,
            `CoreLmrDeeperCap`, `CoreExtensionBudgetEighths`,
            `AspDeltaSquareDiv`, `CoreLmrDeeperBudget`, `TmDepthFreeze` and
            `AspKnownWinWindow`. A coordinate whose default is today's
            behaviour returns to its constant;
          - the user-facing changelog entry: a fixed defect, the engine no
            longer runs a won ending against a bare king into its time
            limit.

          On a fail nothing lands; RAR-R23's dispositions name the two
          closes. The history follows.
          `READY_FOR_IMPLEMENTATION` 2026-10-23. The cluster under gate is
          the arm 6fd195e (recipe
          `analysis/arm_patches/6fd195e-rar-r20-d16fix4.patch`, bench
          8,996,837): the four switches of RAR-R20 baked on as the defaults
          (`mopup_known_win` 6000, `AspDeltaSquareDiv` 10193,
          `CoreLmrDeeperBudget` 1, `TmDepthFreeze` 5000), version
          `2.7.0-dev+d16fix4`. On H1 the landing on `d1` is: those four
          defaults (after the fit, if it ran, its values), the fingerprint
          re-declared in GUIDE and the PLAN checkpoint, `TmFailHighTarget`
          and its test deleted, the instruments the cluster does not use
          removed (`AspFailHighReduce`, `AspFailHighRecover`, `AspRecentre`,
          `QsFailSoft`, `CoreLmrDeeperCap`, `CoreExtensionBudgetEighths`,
          the arm 2d013c8 (recipe `analysis/arm_patches/2d013c8-rar-r19-d16gate.patch`) and its binary), the MultiPV loop given the same
          `Aspiration` form, and the user-facing changelog entry (a fixed
          defect: the engine no longer runs a won ending into its hard
          maximum). On H0 nothing lands and the fit is the next step.
        - **D.1.6.3 Gate: the repairs' reads and gates, the probe and the differential — `V`, DONE 2026-10-26.**
          **RAR-R27 read 2026-10-26: H0.**
          - The LMR correction cap at one ply against `d1`'s head, both with
            the stop: −1.2 ± 3.0 Elo (−2.2 ± 5.3 nElo) at 8,334 pairs, LLR
            −2.99; `10+0.1` +1.7 ± 11.4; 0 faults.
          - The probe passed on both binaries; on the PGO head the stop
            alone ends every row (rec1 at `180000+2000` 1.39 × its
            optimum).
          - The cap does not land, and D.1.6 closes on the stop after
            D.1.6.2's cleanup.
          **Registered 2026-10-24 as RAR-R23**, before any game:
          - **The read:** the offset alone (`MopupKnownWin=5826`) against
            the defaults on one tune build, `rarog-d16inst3-tune.exe`
            (`3f4dcf4`, bench 13,987,223), Colosseum `match
            -CategoricalTuneBuild`. 2,000 games at `3+0.03` (seed
            20261071) and 1,000 at `10+0.1` (seed 20261072).
          - **The harm rules:** −13 nElo at `3+0.03` and −18 nElo at
            `10+0.1`, AGENTS' repair case 2 with the maintainer's longer
            control.
          - **The correctness reads, done:** the probe has no row at the
            maximum; the differential confines the change to positions of
            19 pieces or fewer whose trees reach a bare king.
          - **Predictions:** −2 ± 10 nElo at `3+0.03` and 0 ± 14 at
            `10+0.1`.
          - **Command:** `run_r23.ps1 -Read stc|ltc`, dry-run clean.
          - **Read 2026-10-24, FAIL:** −2.3 ± 7.7 Elo (−4.4 nElo) at
            `3+0.03` passes; −11.1 ± 9.6 Elo (**−25.0 ± 21.5 nElo**) at
            `10+0.1` is below −18, which is harm. 0 faults. The `stc` run
            was interrupted at 79 games and resumed by the same command.
            Nothing lands.

          The earlier gate's history follows.
          The first registration (RAR-R19, the reduction arm 2d013c8 (recipe `analysis/arm_patches/2d013c8-rar-r19-d16gate.patch`),
          seed 20261040) is withdrawn: its arm was a partial device and its
          probe reached the maximum. **Registered 2026-10-23 as RAR-R20**,
          before any game: `rarog-d16fix4-pext-pgo.exe` (6fd195e, recipe `analysis/arm_patches/6fd195e-rar-r20-d16fix4.patch`, bench
          8,996,837, the four parts) against `rarog-d15head-pext-pgo.exe`
          (`468ed7b`, 13,987,223), Colosseum `sprt-repair`, seed 20261042,
          run directory `rar-r20-gate`; then `match-fixed-ltc` 1,000 games
          (seed 20261043) and the probe on both binaries;
          `run_r20.ps1 -Read wire|probe|gate|ltc`. Prediction, frozen,
          unfitted: −6 ± 8 Elo, H1 0.30, cap 0.25, H0 0.45; the research
          recommends the registered fit before the gate. **Verified
          2026-10-23 (RAR-R20):** the PGO arm ends seven of eight probed
          moves within its optimum and rec3 at `180000+2000` still reaches
          the maximum (one window at depth 24, the search-cluster half);
          the 200-game wire check reads −52.5 ± 23.9 unfitted, while at
          equal speed every part reads within noise of even and the PGO
          loss carries 5% less time a move and 5% lower in-game speed.
          **The road to the gate is RAR-R21, staged:** three 2,000-game
          equal-speed reads (the four parts, parts 1–3, the freeze alone)
          decide whether SPSA block 1 over the `d16fix` surface (fifteen
          coordinates, 60,000 games, the fit arm 217125b (recipe `analysis/arm_patches/217125b-rar-r21-d16fit.patch`)) runs at all;
          the gate is re-registered on an arm rebuilt with the fitted
          values. The maintainer's commands are `run_r21.ps1 -Read
          all4|nofreeze|freeze|fit`.
          H1 lands it (harm near −5 excluded); H0 refutes it and the
          fallback is read. The fail-high stop's spin is deleted when the
          fix lands. **Run 2026-10-24 as RAR-R22** on the arm 584c02d (recipe `analysis/arm_patches/584c02d-rar-r22-d16gate2.patch`)
          (the fifteen fitted values and the budget baked, bench
          9,013,917): the probe eight of eight within the optimum, the
          wire check faultless, the SPRT **H0 at 1,377 pairs, −7.4 ± 7.5**,
          the `10+0.1` read +2.1 ± 11.6. Nothing lands; D.1.6 returns to
          research (above), and this gate is re-registered on the
          surviving cluster.

## D.2

- **D.2 Lazy SMP quality at 4T/8T: diversity, shared TT and correction, soft-stop voting; 4T SPRT `[0,5]`; RAR-M78's deficit verdict was measured in a packed slot and is superseded (RAR-M82), the sibling-free comparison is RAR-M83 — `R2` investigation, `I2`/`V` sub-steps, DONE 2026-11-04.** 4T and
  8T scaling against 1T at equal wall time; helper diversity, TT sharing,
  shared correction histories, soft-stop voting, thread-safe counters. The
  helper depth-skip policy is designed fresh from the donor; B.1 deleted the
  inert `smp_iteration_skip` tables and they are not a seed. Competing
  hypothesis to carry into the investigation: Manta's main-authoritative lazy
  SMP (ADR-0064: worker zero alone owns the result, helpers contribute only TT
  evidence from staggered depths, no per-node shared atomics) against Rarog's
  current weighted helper voting. RAR-M46 found no 4T defect, so the question
  is which shape scales to G.1, not which repairs a deficit. Gate:
  4T SPRT `[0,5]` against the 1T-accepted head at 4T, null pair first, in a
  slot of one logical CPU per physical core (PROCESS; RAR-M82 replaced the
  earlier "no affinity"). High-thread and NUMA remain G.1.
  **Input, allocation and copy audit 2026-10-06:** `TranspositionTable`
  derives `Clone`, and a clone of the single-thread form copies the whole
  table. The one search-time clone, each helper's job in `search_parallel`,
  is safe only because `make_shared` runs first. Remove `Clone` and give
  helpers a handle only the shared form can produce, so the copy cannot be
  written; behaviour-neutral, exact fingerprint, no gate of its own.
    - **D.2.1 The premise re-scoped: no four-thread deficit against the field (RAR-M46), so self-relative scaling against modern engines becomes the instrument — `R2`, DONE 2026-09-11.**
      **PREMISE CONTRADICTED, RAR-M46, 2026-09-11 — re-scope before spending work
      here.** This leaf is written to recover strength from immature lazy SMP. At
      4T against the reference field there is no such deficit: Rarog's deficits are
      26 to 75 Elo *smaller* at 4T than at 1T against three of the four targets,
      its performance rating is +31 against its frozen 1T rating, and it beats
      Basilisk 1.10.0 at 4T (+25) having lost at 1T (−23). That does **not** show
      the SMP is good in absolute terms — it beats engines designed for two to four
      cores, which is a low bar, and says nothing about G.1's 8/16/32 threads where
      lazy SMP characteristically degrades. The cheap re-justification is a
      **self-scaling curve** — Rarog at 1T/2T/4T/8T against one fixed opponent —
      which measures Rarog's own scaling without needing a donor comparison. Until
      something like that establishes a defect, D.2 has no measured target.
      **Maintainer steer, 2026-09-11: compare against modern engines, not this
      field.** The RAR-M46 result only shows Rarog beating engines that never
      targeted SMP; the question D.2 actually asks is whether Rarog's lazy SMP is
      weak against engines that did. The clean instrument is **self-relative
      scaling**: for each engine play it against *itself* at 4T versus 1T and read
      the Elo it gains from the threads. That removes every cross-engine confound —
      strength, evaluation, NPS accounting — because each engine is its own
      control. Run it for Rarog, **Reckless** (`reckless-windows-avx2.exe`) and
      **Stockfish** (`stockfish-windows-x86-64-bmi2.exe`), all present in
      `D:/chess/engines/`, and compare the three gains. SMP value grows with time
      control, so the clock is a registration decision, not a default.
    - **D.2.2 The four-thread speed loss: measured (RAR-P46, RAR-P47, RAR-M77), located in the shared search (RAR-P48, RAR-P49), bisected with no single holder (RAR-P50) — `R2`, DONE 2026-10-30.**
      **Target measured, RAR-P46, 2026-10-27.** For the C head, RAR-M46's
      contradiction no longer holds. Its own 4T/1T bench scaling is 3.28 against
      the B.9 head's 3.80, lost already at 2T. Under the games' conditions (three
      concurrent pinned 4T processes, 100 ms searches) it alone enters a
      system-level slow state, 7–10% under B.9 instead of 23% over. **RAR-P47,
      2026-10-27:** the `LargePages` option (`2b7bca9`) turns large pages off; the
      slow state appeared only with them on (2 of 16 passes against 0 of 16), weak
      evidence for the allocator. Off costs about 1–4% NPS in these cells.
      **RAR-M77, 2026-10-28:** in games under RAR-M74's four-thread conditions,
      large pages on are +2.7% NPS and +10.3 ± 5.9 Elo over off, so the default
      stays on. The slow state does not show in games: a match keeps its engine
      processes. The same head reads 8.08 M nps against itself but read 6.47 M
      against RAR-M74's field, so that −9.2% is a question about the opponents
      sharing the slot, not about large pages. Next for the agent: the 2T loss in
      shared writes; the self-relative scaling games come after.
      **RAR-P48 registered 2026-10-29.** A static read finds nothing threads
      share that changed since B.9; the speed pass that made C's nodes 22%
      cheaper was measured at one thread only. The read separates the loss
      into hardware (independent 1T processes on the same cores), the shared
      search, a search split across the two CCDs, and large pages, at 5 s
      with one thread per physical core. Its outcome picks D.2's next step.
      **RAR-P48 read 2026-10-29: Mixed.** Pinned one thread per physical
      core, the C head's E at 4T is 0.90 against B.9's 1.07, so the loss is
      real and is not RAR-P46's placement. It is not the CCD split, and not
      large pages. The split between hardware and shared search is
      unresolved: the instrument pooled two positions, and it read NPS from
      an iteration line that ends 2.3 to 5.0 s into the search. Per
      position, both meet the shared-search rule, and B.9's threads are
      faster sharing a search than apart while C's are not. Next for the
      agent: repair the instrument (per position, one fixed window) and
      register the read that decides it.
      **RAR-P49 registered 2026-10-29.** It reads every search's nodes at
      exactly 5,000 ms, interpolated between its info lines, judges each
      position separately, and keeps B.9 and the C head with large pages
      off. Shared search or both sends D.2 to a bisect of the speed pass at
      1T and 4T; hardware or not reproduced sends it to the scaling games.
      **RAR-P49 read 2026-10-30: shared search.** At 4T the two heads lose
      exactly the same to the shared hardware (H 0.956 against 0.956), and
      C's whole loss is in the shared search (Q 0.98 against 1.06, and 0.94
      against 1.00). Next for the agent: register a bisect from B.9 to C on
      this instrument, read on Q. Its first read asks whether the speed
      pass's parent already shows the loss; within the speed pass the
      prefetch sites go first.
      **RAR-P50 registered 2026-10-30.** A long-control match first was
      considered and set aside: RAR-M76 played C against B.9's search at 4T
      at `3+0.03`, where the loss does not exist, and a match long enough to
      see it would take days. The bisect reads five revisions, B.9,
      `52cc6de`, `6ade0e6`, `138d867` and the C head, each built with rustc
      1.99.1, and names the interval that holds at least half the loss in Q.
      **RAR-P50 read 2026-10-30: Mixed.** At the tightly measured position
      the loss splits between C's evaluation rebuilds (about 0.05 in ln Q)
      and the speed-pass squash (about 0.03). At the noisier one it
      accumulates over every step. No single change holds it, and both
      holders bought large gains, so the bisect stops; a mechanism study by
      shared-table counters waits for a hypothesis that predicts which rate
      differs. The engine target RAR-P46 named stays recorded and unfixed:
      about 6–7% per thread at 4T in searches of seconds, unseen in any
      match. Next for D.2: the self-relative scaling games, the maintainer's
      run.
    - **D.2.3 The four-thread card opened: RAR-M78's self-relative comparison read deficient (later superseded), the engine review's mechanisms, six hypotheses and their tests — `R2`, DONE 2026-11-01.**
      **RAR-M78 registered 2026-10-30.** Rarog, Reckless 0.9.0 and Stockfish 18
      each play themselves at four threads against one and, on one thread, at
      `20+0.2` against `10+0.1`. A raw four-thread gain favours a weaker
      engine, which gains more from any extra search. So each gain is divided
      by the engine's own Elo per doubling of time, giving an effective speedup
      comparable across engines (maintainer decision 2026-10-30). Rarog
      deficient opens a four-thread SMP research card; otherwise D.2's
      four-thread target stays closed and scaling beyond four threads stays
      with G.1.
      **Engine review input, 2026-11-01** (`analysis/engine_review_2026-11-01.md`,
      §3.3 and §3.4; the reviewers' reports and the profile logs are in
      `analysis/artifacts/engine-review-2026-11-01/`). The static read of what
      threads share is complete: on the node path only the transposition table and
      one 64-byte counter line are shared; histories, correction tables, the pawn
      and imbalance caches, the evaluator, the stack and the root records are per
      thread, every atomic is `Relaxed`, and the batched node counter with the
      stop flag costs an estimated 0.2–0.4% at four threads and table coherence
      1–2%, none of it the 6–7% loss. Three mechanisms can make a cheaper node
      share worse with no shared-memory change, and the review ranks their reads
      by cost. Every prediction below is frozen here, before any read.
      1. **The PGO profile covers one of three kernel copies.** `negamax`,
         `quiescence`, `check_stop` and `search_root_window` are generic over the
         poll closure, so the binary holds one copy of each for the engine's
         serial poll (`Engine::search`), one for `search_parallel`'s main thread
         (`main_poll`) and one for the helpers (`helper_poll`), about 15% of the
         binary; the training run sends `bench {depth}` at one thread
         (`xtask/src/main.rs:1156-1176`) and reaches only the first. In a replica
         of the recipe the eleven multi-thread instantiations had all-zero profile
         counters and were placed in `.text$unlikely`, the untrained
         `negamax<NonPv>` at 6,680 assembly lines against 15,568 for the trained
         copy; one Threads-2 training pass made every copy hot. Every thread of a
         `Threads > 1` search, the main one included, runs an untrained copy, and
         RAR-P49's Q compares exactly that against the trained copy in its 1T and
         4×1T cells, so any PGO-sensitive speed-up (the speed pass was tuned under
         PGO) lowers Q with no shared cause: the shape of RAR-P50's loss, spread
         over unrelated commits. Read: build the head with the poll taken as `&mut
         dyn FnMut() -> SearchEvent` at `search_impl`, `search_parallel` and
         `run_worker_job` (one copy, covered by the serial profile; the poll runs
         once per 2,048 nodes), or with a Threads-2 run added to the training, one
         cure at a time; `bench 13` must stay 13,987,223 / EBF 2.565; read Q at 4T
         and the 1T cell on RAR-P50's instrument against the current PGO build.
         Prediction: Q rises by 0.03 to 0.10 in ln with the 1T cell within 1%;
         under 0.02 kills it. A confirmed cure is a behaviour-neutral change
         qualified by the 4T NPS read, no game; it improves every shipped
         multi-thread asset and the 4T arm of every read since RAR-P46.
         Independent of RAR-M78, which prices four threads in Elo.
      2. **The shared backend's cost per operation, F16** (handed to D.2 by
         `analysis/architecture_review_2026-09.md` §4.6 and absent from this text
         until now). A shared probe is up to six (8-byte load, 2-byte load,
         `fold16`, compare) and a store scans up to six slots through `load_any`
         and `unpack_entry`; the local form does the same over three plain
         entries. Fitting RAR-P50's medians to a fixed per-node cost gives 47–61
         ns, about 12% of the head's node time, and a cheaper node makes a fixed
         cost a larger share, which is what the bisect's I2 step looks like. The
         1T fingerprint never sees the shared path, and 1T and 4T play different
         replacement geometry (3-way against 6-way). Read, no code change: `bench
         3` pooled at Threads 2 against Threads 1 (below `MIN_PARALLEL_DEPTH` the
         search stays serial but `configure` has made the table shared, so only
         the backend differs). Prediction: head ratio 0.87–0.91, B.9 0.90–0.93;
         refuted at 0.97 or above. Then, in cost order: the cure of item 1; a
         `replacement` that reads only occupancy, depth and age bits
         (behaviour-neutral); skipping byte-identical stores (item 4); and F16's
         answer, one atomic backend at every thread count, an `I2` with the 1T NPS
         pool at ±0.5% and gate case 3, because the geometry moves the
         fingerprint.
      3. **Q is an NPS ratio, and NPS mixes node cost with node mix.** A shared
         table turns searched nodes into cheap probe-and-return nodes, so Q above
         one is possible with nothing wasted (B.9: 1.051 at position 0) and Q can
         fall when the evaluation gets cheaper; RAR-R07/R08 showed the opposite
         failure of the proxy (3.95× NPS, lower depth). Read: RAR-R10's paired
         time-to-depth harness, 4T against 1T, B.9 and the head, zero games, about
         30 minutes. Prediction: the ratios differ by under 5%; six percent or
         more makes the loss real waste and keeps items 2 and 4 in scope.
      4. **The rest of the card.** The two pick-time prefetches (`1aaa39e`,
         `65f490e`) and the own-cluster prefetch are the only table-side change
         since B.9 (`tt.rs` is byte-identical to `24aefb4`) and fetch lines for
         moves then pruned, measured at one thread only: counters first
         (prefetches per searched move, predicted 1.6–2.2 against B.9's 1.0), then
         one arm per site on the instrument, Q +0.00 to +0.02 each kills the
         family. Byte-identical stores keep a line Modified and a probe miss
         stores twice: a counter (predicted 8–25%), compare before storing at 10%
         or more. `tb_hits` is a `fetch_add` per tablebase probe and sits with
         `stop_state`, `nodes` and the votes on one unpadded line: a thread-local
         flush on the 128-node boundary and `repr(align(64))`, fingerprint exact,
         a 5-man four-thread read as the activation read. Six slots of 16-bit tags
         have twice the local form's false-hit rate and 64 times Reckless's: a
         shadow full-key array in the instrument build measures it before any
         wider tag. Instrument rule: a four-thread rate study uses thread-local
         counters in `ThreadData` summed at join behind a feature; a `diag`
         build's process-global RMWs perturb what they measure and its NPS is
         never read. Donor divergences recorded so they are not rediscovered as
         fixes: the vote weight `+14` with a floor (donor `+10`), decisive results
         ranked by vote rather than by a win-or-loss rule, the quorum
         `floor(N/2)+1` without withdrawal (equal to the donor's at 2–4 and 6
         threads, different at 5, 7, 8 and 16), the ponder-move preference in the
         key, the SMP bundle of RAR-R03 (root rotation, pooled root scores, window
         re-centring) accepted whole and never decomposed, and correction history
         per thread where the donor shares it; each is priced, if at all, with
         RAR-M78's instrument under this leaf or G.1. The `Clone` derive and the
         helper handle the 2026-10-06 audit asked for are D.5's.
         **Triumviratus input, 2026-11-03**
         (`analysis/triumviratus_review_2026-11-03.md` §8, TP-1, TP-3, TP-4). Its
         TT rejects a torn write with a 48-bit tag XORed with the data word,
         beside this leaf's 16-bit tag study. Its per-thread state is aligned to
         128 bytes against false sharing at make and unmake, beside the shared
         counters' `repr(align(64))`. Its SMP levers are priors for the engine
         card if RAR-M83 reads DEFICIENT, otherwise for G.1: a helper LMR bias in
         fine units (Reckless-derived, +4.91 ± 7.97 at 8 threads), staggered
         helpers with depth in the vote (+1.72 ± 4.61), a thread vote (−3.5 ±
         8.7).
      **RAR-M78 read 2026-11-01: Rarog deficient**, by the registered rule. Four
      threads are worth 0.72 of a doubling of Rarog's own time (G4 +73.7 ± 11.5 over
      G2 +101.8 ± 11.5 at `10+0.1`, effective speedup 1.65), against 1.20 (2.30) for
      Reckless 0.9.0 and 1.31 (2.48) for Stockfish 18; Rarog's D minus theirs is
      -0.48 ± 0.22 and -0.59 ± 0.23, both below zero. The runner's own statistics
      say where to look: Rarog's four-thread side reached a lower mean depth than
      its one-thread side (15.41 against 17.28) at 2.28× the nodes per second (the
      bench read 3.28), where Reckless gained 1.3 plies at 3.45× and Stockfish 2.8
      at 3.47×; and 28% of Rarog's four-thread moves were held over the 20 ms margin
      outside its own clock (p99 175 ms) against 2% at one thread and under 1% for
      the other engines. RAR-P50's recommendation against pursuing the four-thread
      loss is withdrawn. **The four-thread SMP research card is open; its reads in
      order:** (1) the PGO coverage of the multi-thread kernel copies, item 1 above,
      one build (at four threads the main thread itself runs an untrained copy,
      which a depth loss at higher NPS is consistent with); (2) time-to-depth four
      threads against one on RAR-R10's harness, item 3 above; (3) the holds: where
      the four-thread process spends the time between the clock's end and `bestmove`
      (the join of every helper, `threads.rs:363–371`, the 1 ms wait loops, the
      `info` and vote path), read from the games' `h=` comments and a timed `go
      movetime` at Threads 1 and 4, then repaired as a clock-side defect under D.1's
      rules; (4) the shared backend's cost, item 2 above; (5) the card's own
      questions from the registration, what the helpers' nodes are worth: helper
      depth, shared-table use and root diversity, with the donor divergences of item
      4 priced on this instrument. Each read is registered with its prediction
      before exposure; a repair that moves no fingerprint closes on the four-thread
      NPS and time-to-depth reads, and anything else takes the leaf's 4T SPRT
      `[0,5]`, whose premise this read restores.
      **The four-thread card, 2026-11-01** (research, zero games so far). The
      deficit is RAR-M78's: four threads are worth 0.72 doublings of Rarog's own
      time against 1.20 for Reckless and 1.31 for Stockfish, a gap of 0.48 to
      0.59 doublings. A speed-only mechanism is bounded: a gain of s in ln NPS is
      worth s / ln 2 doublings, so 7% buys 0.10. Established today, by file reads
      and builds:
      - **The kernel copies.** The head's PGO profile holds 17 kernel
        instantiations, the 11 multi-thread ones with all-zero counts, as the
        review found. `e057d84` takes the poll as a trait object: 6
        instantiations, all trained, `bench 13` exact. The table's shared-form
        branches inside the kernel stay untrained under any one-thread training.
      - **The holds are inside the clock.** `h=` is Colosseum's overhead after
        the last `info` line and lies within the charged time (RAR-R30), so
        RAR-M78's "outside its own clock" is corrected: no forfeit risk, but 9.0%
        of the four-thread time (1.2% at one thread) goes to search that is never
        reported. The code accounts for it: in a pool the soft stop is a vote, and
        the main thread past its target keeps iterating until three of four
        agree.
      - **The depth deficit is not the holds.** On moves held 20 ms or less the
        four-thread side still reports 15.25 plies against 17.30. Its main thread
        searched about a quarter of 1,392,558 nodes a move against the one-thread
        side's 622,252; at EBF 2.565 that costs 0.62 plies, not 1.87. About 1.25
        plies are unexplained by node count.

      Competing hypotheses, each with the shared signals it touches, its test and
      its falsifier:
      1. **H1, untrained kernel copies** raise every thread's node cost. Signals:
         none (code layout). Test: RAR-P52, Q at four threads. Falsifier: mean
         L(Q) under 0.02.
      2. **H2, the shared backend** (F16) costs each node, its branches cold in
         the shipped build. Signals: the table's geometry (six-way against
         three-way) and replacement. Test: RAR-P53, one binary, both backends,
         with and without PGO. Falsifier: R at 0.97 or above.
      3. **H3, the helpers slow the main thread's progress to depth.** Every
         thread searches the same depths (no skip; the old tables were inert,
         RAR-R10), same-depth helpers overwrite the main thread's entries, and the
         RAR-R03 bundle reorders and re-centres the main thread's root from the
         pool (`apply_shared_root_scores`, pooled root scores, window
         re-centring). Signals: table entries and replacement, root order,
         aspiration windows, and the best-move changes the clock reads. Test:
         RAR-R29's level X_ctl. Falsifier: X_ctl at 1.8 or above.
      4. **H4, the soft-stop vote** spends about 9% of four-thread time in a main
         iteration that never finishes. Signals: the soft target, the votes, the
         stop. Test: RAR-R30's timed read. Falsifier: NOT REPRODUCED, or JOIN
         (then a clock-side defect under D.1's rules).
      5. **H5, result selection.** The weighted vote (`+14` with a floor,
         decisive results by vote) can choose a helper's move, and the reported
         depth is then that helper's. Signals: the vote, the reported line, the
         ponder move. Test, not yet registered: the share of four-thread searches
         whose move differs from the main thread's, and their depths, counted in
         an instrument build. Falsifier: under 5% of moves differ.
      6. **H6, the table fills faster** at 2.28× the nodes. Signals: replacement.
         Test: RAR-R29's harness at Hash 1024, only if H3 survives. Falsifier: no
         change in X.

      **Interactions.** H1 and H2 multiply into per-thread NPS and share the cold
      shared branches, which a Threads-2 training run would cure together. H3 acts
      on depth per node and hides behind any NPS gain, so RAR-R29 reads levels as
      well as differences. H4 and H5 act on time and on the decision and are
      invisible to `go depth`. H2's answer, one atomic backend, would move the
      fingerprint and every table measurement H3 relies on, so it comes last.

      **The cheapest discriminating tests** are the four registered reads, about
      50 minutes of the maintainer's machine and no games: RAR-P52, RAR-R29,
      RAR-R30, RAR-P53. Then, by their outcome, H5's count and H3's ablations of
      the RAR-R03 members on RAR-R29's harness (tune-only switches), before any
      game.

      **Stop rule.** Each hypothesis stops at its falsifier. If H1 to H4 all fall
      and X_ctl is 1.8 or above, no zero-game read has found the deficit: the
      card registers H5's count and, if that falls too, closes `NO_CHANGE` with
      the deficit handed to G.1 rather than a 4T SPRT spent without a candidate.
      A repair that moves no fingerprint closes on the four-thread NPS and
      time-to-depth reads; anything else takes the leaf's 4T SPRT `[0,5]`, or a
      removal bracket for a RAR-R03 member.

      **Readiness: not `READY_FOR_IMPLEMENTATION`** for any playing change. The
      one change made, `e057d84`, is behaviour-neutral and was kept by RAR-P52.

    - **D.2.4 The card's zero-game reads: H1 kept as `e057d84`, H2 refuted, H3 and H4 hold, the games' depth deficit not reproduced, RAR-M79's host-state episode — `R2`, DONE 2026-11-02.**
      **The four reads, 2026-11-02** (zero games, the maintainer's runs):
      - **H1 holds, small.** RAR-P52: Q +0.031 in ln, in the band at its edge;
        four-thread NPS +2.4%, the one-thread cell about 0.9% slower, inside the
        registered 1%. `e057d84` stays and closes.
      - **H2 is refuted.** RAR-P53: the shared backend costs a one-thread node
        2.3% (R 0.977), and its cold PGO branches nothing (1.004). F16 leaves the
        card; cure (b), a Threads-2 PGO training run, is not registered.
      - **H3 holds, and it predates the C head.** RAR-R29: four threads reach a
        fixed depth more slowly than one, X 0.87 for the head and 0.96 for B.9,
        at about four times one thread's nodes. The main thread's own tree is no
        smaller than alone: the helpers buy it no depth. The registered
        comparison reads WASTE (d −0.099, 90% −0.235 to +0.014), about 0.06 of
        it the NPS ratio and 0.035 the nodes.
      - **H4 holds.** RAR-R30, VOTE: under the clock 25% of four-thread searches
        hold over 20 ms against 6% at one thread; the stop path is fast. A
        designed behaviour, priced only by games.
      - **The games' depth deficit is not reproduced.** At equal time X 0.87
        costs about 0.15 plies, against the games' 2.05 on unheld moves. The
        read searched depths 20 to 22 for about 2 s from a cleared table; the
        games search depths 15 to 17 in about 200 ms with the table kept.

      **Next, by RAR-R29's rule for X under 1.2:** what the helpers do to the
      main thread, each a zero-game read registered before exposure, ordered in
      that registration: RAR-R29's harness at the games' depth and time with the
      table kept across searches (it separates a short-search cost from a
      carried-table one); H3's ablations of the RAR-R03 members on the same
      harness; H5's count in an instrument build; H6 at Hash 1024, now in scope.
      H1 and H2 are closed and H4 waits for games. Still not ready for any
      playing change.

      **Registered 2026-11-02, the maintainer's, about two hours, no games:**
      - **RAR-R31,** the games replayed: twelve of RAR-M78's games searched again
        position by position with the games' clocks and the table kept, alone
        and under the games' conditions one at a time. It folds in H6 (a Hash
        1024 cell) and H5's count (searches that end on a line for a move no
        earlier line named), read from the output without an instrument build.
        Read from the games first: **in play four threads ran at 2.28 times one
        thread's NPS at every search length**, against 3.6 alone and pinned, and
        the same in every slot (H7, new: a condition of the games costs the
        four-thread side speed; Reckless keeps 3.44 under it).
      - **RAR-R32,** H3's screen: the pool's root order, the window centre, the
        helpers' rotation and the jitter offset switched off one at a time and
        together, on RAR-R29's harness.

      **Read 2026-11-02:**
      - **RAR-R31, NOT REPRODUCED.** Alone, with the games' positions, clocks
        and kept tables, four threads report the same depth as one (+0.08
        plies) at 3.78 times its NPS. On the same moves the games ran the
        four-thread side at about 60% of that speed and 1.38 plies shallower,
        the one-thread side at 91%. The slot's eight logical CPUs and three
        processes at once cost under 6%; H6 is refuted; H5 is active (+5.6
        points of closing lines) and held.
      - **RAR-R32, nothing resolves:** none of the four RAR-R03 members, nor all
        four together, holds the main thread back at about ±10%.

      **Compared with Reckless's code, 2026-11-02.** The same Lazy SMP skeleton
      (one shared table, every thread at every depth, a persistent blocking pool,
      the score-and-depth vote, a three-of-four soft stop), and six differences:
      Rarog's helper diversity (root rotation, pooled root order, window centre,
      jitter offset), which Reckless does without; per-thread correction history,
      where Reckless shares one; two table layouts by thread count against one;
      one node counter every thread adds to on the stop flag's cache line,
      against per-thread padded counters and a read-only stop line; a vote never
      withdrawn; and **large pages locked on Windows** for the table and every
      thread's big tables since `6ade0e6`, where Reckless allocates ordinarily on
      Windows and asks for transparent huge pages only on Linux. Rarog asks for
      none on Linux: D.7.

      **Read 2026-11-02 (the maintainer's run, 16:51 to 18:06):**
      - **RAR-M79, ABSENT:** in RAR-M78's own configuration (three games at
        once) the head's four threads ran at 3.44 times one thread in games,
        against RAR-M78's 2.48; `e057d84` covers 2.4% of that. Large pages off
        (3.53) and the B.9 head (3.28) carry no loss. **One game at a time**
        reads 2.69: on the same openings the four-thread side is 23% slower than
        with three games at once and the one-thread side 6% faster (an
        unregistered reading).
      - **RAR-R33, HITS WITHOUT SAVINGS:** at four threads the main thread's hit
        rate rises 5.6 points (90% +5.1 to +6.0) and its own probes do not fall
        (M 1.09, 90% 0.98 to 1.21): the helpers' entries reach it without
        cutting its tree. The next H3 read is the depth and bound of those hits
        against what the node needs.

      **H7, rewritten 2026-11-02 after RAR-M79.** RAR-M78's 2.48 was its
      sessions' host state, not a fixed condition of the games: the same
      configuration now reads 3.44. RAR-M78's threads leg is recorded as
      confounded and its effective speedup (1.65) is measured again before D.2
      reads anything from it. What stands is that the speed four threads keep in
      games depends on the host's state by up to a third, and that one game
      alone on the host, the nearest condition to a rating list that plays one
      game a machine, costs the four-thread side most. Whether Reckless loses
      the same way was never measured on one host state: its 3.44 in RAR-M78 was
      another session's. Next, each registered before exposure: one game at a
      time against three, Rarog and Reckless side by side, per-side in-game NPS;
      then Rarog's four-thread scaling measured again in the condition that read
      chooses. No engine change is in scope until then. RAR-R31's framing (40%
      lost in games, Reckless far less) stands as measured in RAR-M78's
      sessions.

    - **D.2.5 The slow state located: the audit, the night (RAR-M80, RAR-M81), Windows packing four threads on SMT siblings; restored in a sibling-free slot (RAR-M82, RAR-P54) — `R2`, DONE 2026-11-03.**
      **Registered 2026-11-02, the maintainer's, about 80 minutes:**
      **RAR-M80**, one game at a time against three, Rarog and Reckless 0.9.0
      each against itself, one thread against four at `10+0.1`, 60 games a leg
      on the same openings and host state; per-side in-game NPS, no Elo read.
      Not replicated, Rarog's own, or shared decides the next read.
      **Basilisk cross-check input, 2026-11-02** (`analysis/basilisk_crosscheck_2026-11-02.md` §H, H26–H28), for H3
      and H5 once the condition is located: the root's first move is the shared
      table's (`node.rs:650-662`, as in Reckless; Stockfish 19 takes the
      thread's own previous best), so a helper's store at its own depth, or a
      fail-low root's least-bad move (D.6.4), can put a non-PV move first
      on the main thread; RAR-R32 switched off rotation, the pool order and the
      window centre but not this. Read, zero games: at four threads, the share
      of main-thread iterations whose first root move differs from that
      thread's last completed best, split fail-low and other; if large, one
      arm with the thread's own `root_moves[0]` first on RAR-R29's instrument.
      Stockfish 19 also shares its continuation, pawn and correction histories
      across threads through atomic entries, where Rarog keeps all of them per
      thread and Reckless shares correction only: a competing mechanism under
      H3, with RAR-R06's helper-history blend (−0.52) as the prior against.
      If a coherent table reopens (TT-9), Basilisk's costs at one thread are
      the prior: a `key16 ^ fold16` tag −3.91%, one packed 64-bit word with the
      move apart −1.22% [−1.72, −0.76] (BAS-C05).
      **Audit 2026-11-02** (`analysis/d2_audit_2026-11-02.md`, a re-read of
      every record and artifact above; nothing built, no game):
      - **The depth figures that steered the card are contaminated.** Every
        mean depth quoted from the runner (1.9 plies less at four threads,
        2.05 on unheld moves, Reckless +1.3, Stockfish +2.8, B.9 +0.30)
        averages mate-score moves in, where the head's four-thread side
        reports 7 to 10 plies and its one-thread side 23 to 27 (B.9 41 to 45
        on both sides, Reckless 62, Stockfish 125): a reporting difference in
        decided positions. Without them the four-thread side is 1.05 plies
        shallower in RAR-M78's games, 0.25 in RAR-M79's `on` leg (R 3.44),
        0.15 in `off`, 0.93 in `solo`, and B.9's is 0.51 shallower, not
        deeper; Reckless's is 0.72 deeper and Stockfish's 1.6. At R 3.44 the
        0.25 is RAR-R29's X 0.87 (0.15 plies) plus the shorter own time
        (0.12): on a healthy host no in-game depth deficit is unexplained.
        RAR-R30's "the holds do not explain the depth deficit", RAR-R31's
        premise and H5's motivation rest on the contaminated figure; each
        record carries its correction.
      - **RAR-M78's threads leg is confounded, as RAR-M79 found, and the
        deficit verdict falls with it:** "Rarog deficient" is not
        established, and the card's question is open again, not answered.
        Frozen prediction for a clean rerun at R about 3.4: G4 +122 ± 12
        (0.47 doublings more than at R 2.48, by the odds leg's 101.8 Elo a
        doubling), D 1.2, X 2.3, level with Reckless; G4 at about +85 or
        below reads deficient against Reckless again and makes the lead
        below the engine's own target.
      - **The host state is the blocker, and it is unidentified.**
        Four-thread processes alone lose a third of their speed, flat for
        hours, the head's so far and never a one-thread side: RAR-M74
        (2.60), RAR-P46 (5 of 12 passes), RAR-M78 (a whole leg), RAR-M79
        `solo` (2.69), against 3.44 to 3.8 when healthy. RAR-M79's `off` leg
        ran in the healthy state, so it cannot clear large pages for the
        slow one (D.7 corrected). In the slow state the main thread loses
        about twice the depth its NPS loss predicts (RAR-M78 1.05 plies
        against 0.6 expected, `solo` 0.93 against 0.5; RAR-R31's fidelity is
        +1.33 on non-mate moves), so the loss is not a uniform slowdown of
        every thread. Candidates that predict R near 2.5: two threads a
        physical core (the slot's eight logical CPUs, a parked core) and a
        parked or down-clocked core; the host runs a custom power plan with
        its parking settings at their defaults. The instrument is passive
        and costs nothing: `typeperf` at one second over the slot's logical
        CPUs (`% Processor Time`, `Parking Status`, `% of Maximum
        Frequency`, `Processor Frequency`) and `\Thread(rarog*)\% Processor
        Time`, beside every four-thread leg from now on, so the next slow
        episode names its mechanism; and each leg's per-side in-game R
        recorded as its validity check (a four-thread Rarog block under 3.0
        is not read).
      - **What stands as the engine-side lead:** four threads reach a depth
        with four times one thread's nodes and the helpers' entries do not
        cut the main thread's tree (RAR-R29, RAR-R33), where Reckless's and
        Stockfish's four-thread sides in games reach one thread's depth at
        1.3 to 1.6 times its nodes. It predates C (B.9 0.96) and is priced
        only by the rerun.
      - **Next, in order.** (1) RAR-M80 as registered, with the sampler
        beside it (an added observation; its legs do not change). (2)
        RAR-M81, the maintainer's night: Rarog's and Reckless's threads legs
        of RAR-M78 played again in alternating blocks of 250 games (three
        games at a time, the same binaries, Hash 256, `10+0.1`, the odds
        legs reused), the sampler running, each block's per-side R its
        validity check, read as RAR-M78 with the prediction above. (3) By
        its outcome: not deficient closes D.2's four-thread target
        `NO_CHANGE` with the host-state note carried to E.2's four-thread
        arm and every rated four-thread read; deficient makes RAR-R33's next
        read (the depth and bound of the main thread's hits against what its
        node needs) the card's first engine question, with helper depth
        staggering and the shared histories (H27) as the competing
        mechanisms. No engine change is in scope before (2). Until the night
        read is in, D.2 is held and the agent works D.6.15, D.5.19
        and D.3, in the register's order.
      **Registered 2026-11-02, after the audit:** RAR-M80 amended with the
      sampler (an added observation, before any game), and **RAR-M81**, the
      decision read above: Rarog's and Reckless's threads legs of RAR-M78
      played again, 1,000 games each, the odds legs reused, the sampler
      running, a leg with any fifth under R 3.0 (Reckless 3.2) recorded as
      slow state and not read; NOT DEFICIENT 0.60, UNRESOLVED 0.25, DEFICIENT
      0.15. One night, after RAR-M80 in one command
      (`analysis/artifacts/d2-night/run.ps1`), prepared 2026-11-02: self-test,
      dry run and sampler smoke recorded in RAR-M81's *Preparation*.
      **Night read 2026-11-03** (RAR-M80, RAR-M81; the sampler's reads in
      RAR-M81's *Result*): **the slow state is Rarog's, not the host's.** On one
      quiet host Rarog's four-thread side ran at 2.62–2.66 times its one-thread
      side in every leg and every fifth for 9.5 hours, Reckless's at 3.36–3.48
      beside it. RAR-M80 NOT REPLICATED: one game at a time costs nothing
      (RAR-M79's 23% was an episode). RAR-M81 UNRESOLVED by its rule (Rarog's
      leg SLOW STATE); in that state four threads are worth 0.82 doublings of
      Rarog's time against Reckless's 1.28 and Stockfish's 1.31, and G4 follows
      R. The sampler excludes core parking, frequency and scheduling: Rarog's
      threads get Reckless's CPU time and each does a third less than Rarog's
      single thread; its load sits more often on both SMT siblings of a core.
      The audit's host-state reading is corrected. Next, zero-game and on an
      idle host (after Basilisk's tune): the slot rebuilt outside the harness,
      Rarog's four-thread search confined to one logical CPU per physical core
      against both siblings, Reckless the control. Recommended before the
      registered replay of the slow leg, which would most likely repeat it.
      **Corrected and registered 2026-11-03** (the audit, section 10): the
      sampler's per-logical-CPU busy shows where each four-thread side's work
      landed inside its slot: Rarog's four threads on both siblings of two
      physical cores in 90–96% of its turns, all night, every slot; Reckless's
      on two cores in 47–66% and three in most of the rest, one per core under
      1% for either. Windows packs four busy threads inside an
      eight-logical-CPU affinity mask for both engines, Rarog's harder; neither
      engine places its threads on Windows (Reckless binds to NUMA nodes on
      Linux only). So the slow state is the scheduler's placement inside
      Colosseum's slot, not the host's and not Rarog's code; the repair is not
      first the engine's (hard affinity in an engine fights the tester's mask;
      no donor does it), and the fair four-thread measurement is a slot of one
      logical CPU per physical core, which the pinned runner gives from an
      explicit list and which `38ef0d5` lets `colosseum.ps1` accept (game cores
      only, a core left free). Registered: **RAR-M82**, RAR-M81's threads legs
      in that slot, 200 games each, the night's openings, the sampler riding,
      an instrument rule, RESTORED 0.70 / PARTIAL 0.20 / NOT RESTORED 0.05 /
      INSTRUMENT 0.05 (`analysis/artifacts/d2-placement/run.ps1`, about an
      hour and a quarter on an idle host); **RAR-P54**, what a thread of each
      engine keeps beside its SMT sibling on one core, standalone under
      affinity masks, f_rarog 0.66 against f_reckless 0.83 predicted;
      **read 2026-11-03: REFUTED**, 0.630 and 0.647 over three cycles, the
      same loss within 0.02, so the night's ratio gap is placement alone and
      sibling sharing costs any engine here about 37% a thread: a slot of one
      logical CPU per physical core is the only fair multi-thread
      measurement, and the per-thread working-set question leaves the card.
      **RAR-M82 read 2026-11-03: RESTORED** (predicted 0.70): with one logical
      CPU per physical core Rarog's four threads run at 4.03 times one (every
      fifth 3.94 to 4.11, 1.007 a thread), Reckless's at 4.45; four-thread
      turns on four distinct cores in 94 to 98% of their seconds, no sibling
      busy. The night's Reckless control was packed too (3.46), so neither
      RAR-M78's nor RAR-M81's four-thread legs of any engine are clean
      references; RAR-M78's time-odds legs stand. Observed, not judged: on the
      night's first 200 openings Rarog's four threads over one +127.0 ± 23.7
      (the packed leg +83.9), Reckless's +121.1 ± 19.6; Rarog's non-mate depth
      +0.37 plies at four threads (the night −0.74), Reckless's +1.75; the
      four-thread holds unchanged (p99 162 ms, 28% of moves over the margin;
      Reckless 3 ms), the card's third read. Next: the self-relative
      comparison re-registered in this slot, Rarog's, Reckless's and
      Stockfish's four-thread legs at 1,000 games each (about nine hours), with
      RAR-M78's time-odds legs as G2; the engine-side placement hint a held
      leaf. The decision rule as registered: by
      RAR-M82: RESTORED supersedes RAR-M81's verdict and re-registers the
      self-relative comparison sibling-free, with the explicit placement the
      policy for every rated multi-thread read and an engine-side placement
      hint a separate leaf held on how CCRL's testers place a four-thread
      engine; PARTIAL adds the per-thread working set to the card; NOT
      RESTORED returns D.2 to its engine card.
    - **D.2.6 RAR-M83: the self-relative comparison in the sibling-free slot, the maintainer's night run — `V`, DONE 2026-11-04.**
      **RAR-M83 registered 2026-11-03**, before any game: the self-relative
      comparison in the sibling-free slot, Rarog (`e057d84`, RAR-M82's
      binary), Reckless 0.9.0 and Stockfish 18 each one thread against four,
      1,000 games at `10+0.1`, seeds 131 to 133, RAR-M78's time-odds legs as
      G2; a leg is INSTRUMENT when its four-thread turns are off four cores
      or its R is under its floor (Rarog 3.5, Reckless 4.0, Stockfish 3.8);
      DEFICIENT when Rarog's D is below both others' at 95%, NOT DEFICIENT
      otherwise; NOT DEFICIENT 0.70, DEFICIENT 0.20, INSTRUMENT 0.08, NOT
      JUDGED 0.02 (`analysis/artifacts/d2-selfrel/run.ps1`, about nine hours
      on an idle night). Stockfish's leg is in: the rule needs both
      references, and no clean Stockfish four-thread leg exists. By its
      result D.2's four-thread target opens on RAR-R33's next read or closes
      `NO_CHANGE`. **Read 2026-11-04: NOT DEFICIENT** (predicted 0.70), run
      by the maintainer from 22:51 to 07:35, the host idle, no fault or
      time loss. All three legs valid: R 4.02, 4.44 and 4.43, four-thread
      turns on four cores in 95 to 97% of their seconds, no sibling busy.
      Four threads are worth 1.37 [1.18, 1.55] doublings of Rarog's own
      time (G4 +139.0 ± 11.1, X 2.58) against Reckless's 1.49 (+128.6) and
      Stockfish's 1.54 (+134.1); Rarog minus Reckless −0.13 ± 0.27, minus
      Stockfish −0.18 ± 0.28, both upper bounds above zero. A gap above
      about 0.4 doublings is excluded, a smaller one is not. Every
      predicted level fell inside its band. Observed, not judged: non-mate
      depth +0.46 plies at four threads against +1.63 and +2.46; Rarog's
      four-thread holds as before (28% of moves over the margin, p99
      158 ms; Reckless 0.4%, Stockfish none).
    - **D.2.7 The four-thread engine card, by RAR-M83's outcome: what the helpers' entries do to the main thread, then a gate — `R2`, then `I2`/`V`, NO_CHANGE 2026-11-04.**
      **Closed `NO_CHANGE` 2026-11-04 by RAR-M83's NOT DEFICIENT;** the
      items below go to G.1 as priors, as the rule written before the read
      says. Held on D.2.6. NOT DEFICIENT closes D.2's four-thread target
      `NO_CHANGE`: the placement policy stays with E.2's four-thread arm and
      every rated four-thread read (PROCESS), and the open items below go to
      G.1 as priors. DEFICIENT opens the card on RAR-R33's next read, the
      depth and bound of the main thread's table hits against what its node
      needs (four threads reach a depth with four times one thread's nodes,
      and the helpers' entries do not cut the main thread's tree: RAR-R29,
      RAR-R33), with helper depth staggering and shared histories (H27) as
      the competing mechanisms. Its other open items are recorded in D.2.3
      to D.2.5: H5's result selection (active, +5.6 points of closing lines,
      RAR-R31); the four-thread holds (28% of moves over the 20 ms margin,
      p99 162 ms, in RAR-M82's sibling-free legs; H4, priced only by games);
      the root's first move taken from the shared table (H26); item 4 of the
      engine review input (the pick-time prefetches, byte-identical stores,
      the shared counter line, the 16-bit tag's false-hit rate, the donor
      divergences); and Triumviratus's SMP priors. A repair that moves no
      fingerprint closes on the four-thread NPS and time-to-depth reads;
      anything else takes the leaf's 4T SPRT `[0,5]`, or a removal bracket
      for a RAR-R03 member (D.2.3's stop rule). RAR-M82's rule named an
      engine-side placement hint as a held leaf; it is G.1's, which owns
      the thread-affinity policy.

## D.3

- **D.3 Engine lifecycle and protocol robustness; `src/uci/` (planned) move; score normalisation research card (`analysis/uci_info_review_2026-09-16.md` item 6); zero crashes over pool tournaments — `I1` for the eight review-confirmed defects (2026-11-01), then `R2` for the cards, DONE 2026-11-04.** UCI
  parsing and dispatch, stop/ponder/infinite semantics, new-game resets,
  malformed input, panic reporting, Syzygy probe policy and thread safety.
  **Found 2026-11-01 in D.2's checks:** in a full `cargo test` run,
  debug or release, 1 to 5 of `tests/uci_process.rs`'s 28 tests time out
  with no engine output at all (`seen: []`), a different set each run; on
  the unchanged head too (4 of 28, release). They pass 28 of 28 when the file
  runs alone. Cause unread; the start-up of a fresh process under the suite's
  load is the first candidate. A test that only fails under load is still a
  failure: D.3 owns it.
  **Done out of band, 2026-10-01:**
  - `97ea52d`: the last `info` line now always describes `bestmove`. An
    iteration stopped after a window failed high or low on an unconfirmed
    move left that move's bound line last, serially under `go nodes` (now a
    deterministic test) and at Threads 8 under a clock (the flaky
    `threaded_bestmove…` test). Output only, both fingerprints exact.
  - `2326153`: the legacy search compiles again; `d754577`'s optimism call
    needed `b2core`, and CI builds that arm.
  - `b2c98c8`: the tablebase PV extension starts under a clock only when
    at least ten move overheads remain before the hard ceiling once the
    search has ended (100 ms at the default, about twice the largest
    blocked read measured, 54 ms). RAR-S94's forfeit came from the
    extension running at 58 ms; the old skip engaged only below about
    52 ms. The donors: Reckless has no extension; Stockfish gives it the
    same half-overhead box, checked after every probe, at each iteration's
    output inside the search's own time, with no guard against one blocked
    read. Rarog keeps its one extension after the search, the form RAR-S94
    measured, and adds the start rule. A unit test holds the rule at the
    forfeited move's numbers; on two 6-man roots at `100+30` the old binary
    extends and the fixed one does not, and at `3000+30` both print the
    same line. Both fingerprints exact. RAR-S97, its correctness read
    against the old binary on the endgame cohort, played 2026-10-01 with no
    defect: every clean win converted by both, no time loss, no fault.
  - `488d34a`: a tablebase root score prints without a bound (6 of 14 such
    lines carried one at a KQvK root, `go depth 8`; none now), as the donor
    prints it. Output only, both fingerprints exact.
  - `0827ab9` (2026-10-06, from the Basilisk review, `analysis/basilisk_review_2026-10-06.md`):
    `bench` floors its per-position and per-run nps at 1 ms like the
    `info` lines (a position solved inside the first millisecond printed
    its node count); a second `const` assertion keeps the mop-up below the
    decisive band; a regression test holds that a position repeating the
    game history scores a draw before any stored result. `bench 13` exact.
  - `6433760` (2026-10-06, from an allocation and copy audit): at a
    halfmove clock of 100 in check, the search's draw test built a heap
    `Vec` to ask whether the side to move is mated; it now fills a stack
    list in a cold helper. `tests/allocation_guard.rs` counts that test
    directly, because the growth bound cannot see a path this rare (it
    fails on the old code with one allocation). `bench 13` exact on default
    and pext; pooled-PGO no-regression read passed at −0.24% (−0.55% to
    +0.07%, six cycles; `analysis/artifacts/rule50-alloc/`).
  - C.12's speed pass (2026-10-17, on `c7`; `analysis/c12_speed_pass_2026-10-17.md`),
    the search, ordering, generation and board side, each `bench 13` exact
    at 10,521,141 / EBF 2.521: the per-node search parameter sets read as
    compile-time constants outside `tune` builds (`d61d4f6`; `d2ab515` keeps
    the root, clock and quiescence set runtime); pawn and knight attack sets
    from shifts (`58547cd`); quiescence's known check status passed to the
    make (`60f8b96`); the derived keys toggled by mask (`acd767e`); a
    one-kind generation pass's move flag known statically (`09bb98c`); the
    best-move selection by conditional moves (`081041c`); the child's TT
    cluster prefetched when the move is picked (`1aaa39e`, `65f490e`);
    staged king moves decided from the node's threat set (`9f388c5`); the
    attack tables built at compile time (`b52520d`).
  - C.12's third pass (2026-10-18, on `c7`; `analysis/c12_speed_review_2026-10-18.md`),
    kept by maintainer decision below the protocol's bar, each `bench 13`
    exact at 10,521,141 / EBF 2.521: the node's threat set computed only
    where a node reads it, the reverse-futility test in two steps and
    quiescence skipping it for an empty capture list (`842b586`); the node's
    check masks shared between the move loop and quiet scoring (`c01db68`).
    Together +0.41% (+0.28% .. +0.54%) pooled-PGO NPS with eight builds an
    arm, +0.08% (−0.11% .. +0.27%) with large pages on both arms.

  **Input from B.5.2.2:** a table read the page cache misses held the PV
  extension's DTZ root probe for about 54 ms and lost RAR-S94's round 745
  on time at 58 ms of clock (the failed-games record shows the extension's
  notice on that move; the same stall in an in-search probe is inferred,
  not observed). Probe policy under a short clock is this leaf's. Slow
  probes are reads the page cache misses (151 GB of tables against 128 GB
  of RAM); concurrent engines stretch the tail without raising its rate
  (`analysis/artifacts/b52-box-replay-2026-10-01/`).
  **Left from the 2026-10-01 audit of B.5.2.1** (the short-clock skip and
  the bound suffix are repaired above):
  - The extension gates on the loaded tables' size, not on
    `SyzygyProbeLimit`, which the root ranking and the in-search probes
    honour (read from `syzygy::rank_root_moves`, not reproduced; the
    committed 3-man fixture cannot show it, so it waits for a test that
    can).
  - A read blocked for longer than the start rule's margin plus the reserve
    (120 ms at the defaults) still loses on time; `go movetime` names no
    clock, so the rule does not apply there.
  Deterministic tests; zero crashes over the pool tournaments. Owns the last
  move of the target layout: `engine.rs`, `engine_command.rs`,
  `uci_protocol.rs`, `search_options.rs` and `bench.rs`/`wac.rs` go under
  `src/uci/` (planned) in the same leaf, behaviour-neutral, exact fingerprint.
  **Research card, added 2026-09-16 (`analysis/uci_info_review_2026-09-16.md` item 6):** the
  displayed score is raw internal units (startpos depth 1 reads `cp 143`;
  a tablebase win prints `cp 31744` because `TB_WIN_SCORE` sits below
  `format_score`'s mate cutoff). Stockfish and Reckless normalise to a
  win-probability scale. Fit a win-rate model on Rarog's own games (score
  against outcome by material phase), decide the `cp` mapping and a
  distinct tablebase-win band, and only then implement; display-only,
  gated by identity, no SPRT. **Noted, not adopted (2026-09-17):** an
  auto-sized hash as RBp4wn (tissatussa, Go, CC0, `github.com/tissatussa/RBp4wn`, read 2026-09-17)
  does it (probe at the ladder's maximum, skip move 1 whose empty-table
  search over-stores by about 35%, sample entries stored on moves 2–5,
  resize once to `peak / 1.4` on a log2 ladder, never again in the game;
  fixed `Hash` bypasses it). The GUI and every rated pool own `Hash`, and
  a mid-game resize discards warm entries, so it is at most a friendlier
  default for casual play; revisit only if the release checklist asks
  for one.
  **Engine review input, 2026-11-01** (`analysis/engine_review_2026-11-01.md`
  §3.6): eight shell defects, four reproduced in live sessions against the
  debug binary, each a one-function fix that leaves the fingerprint (bench
  sends no clock and no `setoption` during a search), repair case 1 with the
  `tests/uci_process.rs` case written first. They go before the 2.7.0 cut, in
  this order:
    - **D.3.1 A zero, negative or unparsable own clock searches without limit — `I1`, DONE 2026-11-03.** A zero, negative or unparsable own clock runs an unbounded search: clocks
     parse as `usize`, a failure becomes 0 with a notice, and
     `time.rs:145–149` reads 0 as no clock, so `go wtime 0 btime 0` and `go
     wtime -50 btime 1000` search until `stop` (replayed: past 3 s with no
     limit). Parse signed, keep presence as `Option` per side, clamp a present
     clock to at least 1 ms (the donor moves at once on a zero clock;
     Stockfish clamps); a unit test on `compute_runtime_limits`.
    - **D.3.2 `setoption` during a search wedges the input thread — `I1`, DONE 2026-11-03.** `setoption` during a search wedges the stdin thread for good:
     `set_option` calls `wait_for_search_finished`, which pushes a `Ready`
     behind the running `Go` and blocks; under `go infinite` the only line
     that could end the search is one that thread never reads (replayed:
     killed at 7.5 s with `stop` and `quit` unread). The wait is not needed,
     the FIFO already orders `Configure` after the `Go`: drop it or answer
     with a notice. The donor forwards only `isready`, `stop` and `quit` while
     searching.
    - **D.3.3 End of input never ends `go infinite` or a finished ponder search — `I1`, DONE 2026-11-03.** Stdin EOF never ends a `go infinite` or a finished ponder search: EOF
     pushes an ordinary FIFO `Quit` (deliberately, so a piped `bench`
     completes) and sets no flag the search polls, so the engine searches or
     sleeps forever when stdout stays open. An `eof` flag treated as `Stop` by
     `infinite` and `ponder` searches only.
    - **D.3.4 A closed stdout panics every `println!` — `I1`, DONE 2026-11-03.** Every library `println!` panics on a closed stdout, and under `panic =
     "abort"` the engine thread dies as a crash on a normal GUI shutdown
     order: `writeln!` on a locked handle with `request_quit` on `Err`, one
     GUI-gone signal that serves item 3 too.
    - **D.3.5 A non-UTF-8 input line reads as end of input — `I1`, DONE 2026-11-03.** A non-UTF-8 input line is read as EOF (`read_line`'s `Err` becomes 0
     bytes through `unwrap_or_default`) and the engine exits silently with
     status 0: `read_until` and `from_utf8_lossy`; only `Ok(0)` is EOF.
    - **D.3.6 `stop` during `bench` or `wac` prints an ordinary-looking summary — `I1`, DONE 2026-11-03.** `stop` during `bench` or `wac` ends the suite early and prints an
     ordinary-looking summary (`Nodes searched : 110427` against 1,679,897 for
     the same `bench 9`), the line the PGO trainer waits for: print that the
     bench was interrupted and no summary on `Stop`.
    - **D.3.7 A refused `Hash` is retried on every later `setoption` — `I1`, DONE 2026-11-03.** A refused `Hash` is retried and re-announced on every later `setoption`,
     because option state lives twice (the protocol's copy and
     `SearchShared.hash_mb`) and every `Configure` resends the whole struct:
     remember the last requested size, or configure only the option that
     changed.
    - **D.3.8 The forced-move shortcut fires for a one-move `searchmoves` — `I1`, DONE 2026-11-03.** The forced-move shortcut (stop after depth 2 with one legal move) fires
     for a one-move `searchmoves` under fixed depth or time, because the
     filtered root list feeds the count: exclude a non-empty `search_moves`.
    **Done 2026-11-03:** D.3.1 to D.3.8 (`46ddf56`); each defect has its
    `uci_process` case, written to fail on the code before (a present clock
    of 1 ms at least, `setoption` without the wait, the input-closed flag,
    `uci_println!`, lossy UTF-8 input, the interrupted-suite report, the
    remembered `Hash` request, the shortcut off under `searchmoves`);
    behaviour-neutral, the fingerprint 14,749,043 / EBF 2.564 reproduced;
    debug and release suites, fmt and clippy clean.
    - **D.3.9 The score-normalisation card, the `src/uci/` (planned) move and the protocol decisions — `R2`, DONE 2026-11-04.**
      Decisions to record, not changes: a bad `position` ends the process with
      `process::exit(1)` from library code (both donors keep the previous
      position; the exit has one advantage, the message reaches the retained
      transcript); keep it until a logged game shows a GUI sending a recoverable
      bad `position`, move the `exit` to `main.rs`, and answer a pending `go` with
      `bestmove 0000` first. With the `src/uci/` (planned) move: one option table generating
      both the advertised string and the clamp, `OptionUpdate::{Applied, Rejected,
      Unknown}` so the protocol layer prints (15 `info_string!` sites leave the
      parser), one `run_suite` under `bench` and `wac`, the dead queued
      `PonderHit`, `#[ignore = "needs RAROG_SYZYGY_PATH"]` on `multipv_syzygy` so
      a skip is visible, and D.8's P12 (the option types out of the parser, a
      structured `InfoSink` record so score formatting leaves the search), which
      also removes the `search` and `search_options` cycle. `go mate N` as a
      fixed-depth search is noted, low priority. State: `READY_FOR_IMPLEMENTATION`
      for items 1–8 (`I1`); the score-normalisation card and the move stay `R2`.
      **Basilisk cross-check input, 2026-11-02** (`analysis/basilisk_crosscheck_2026-11-02.md` §H, H13–H16): for the
      score-normalisation card, `format_score` (`search/mod.rs:334-345`) maps
      the tablebase band to `cp` 19,873–20,000 but prints any ordinary score
      of 19,872 or more raw, the rule-50-downgraded mate (`TB_WIN_SCORE − 1`,
      `tt.rs:585`) among them, as `cp` above every tablebase win; Basilisk
      clamped ordinary scores to 19,871 after 111 such lines in 64 of 1,000
      games. A process-level test that panics a helper thread in a
      release-profile build (`panic = "abort"`) and requires the `info string`
      report and the exit status: Basilisk's caught a runtime keeping the
      handler per thread, and Rarog's test is in-process on the main thread
      (`crash_report.rs:167`). A mated or stalemated root under `go infinite`
      reports exactly once and gives `bestmove` only on `stop` (the existing
      case covers `go depth 5` and checks presence, not uniqueness). PROTO-6's
      decision also covers castling rights that do not match the rooks
      (`4k3/8/8/8/8/8/8/R3K3 w KQkq - 0 1`): Rarog rejects the FEN and ends
      the process, Basilisk keeps the rights the rooks support.
      **Triumviratus input, 2026-11-03**
      (`analysis/triumviratus_review_2026-11-03.md` §4 item 12, TA-2). A FEN
      halfmove clock above 255 is parsed as `u8` and refused, so the process
      ends (`board.rs:369-372`); PROTO-6's decision covers it, and a clamp
      would keep the position. For the score-normalisation card: Triumviratus
      calibrates its printed scale on self-play so that +1.00 is a 50% chance
      to win, refitted per network.
      **Done 2026-11-04** (`8656ed0` to `d0759b9`):
      - **The score-normalisation card: `NO_CHANGE`.** A printed scale with a
        stated win probability needs a model fitted to the evaluation, and
        Phase F replaces the evaluation and refits per network, as
        Triumviratus does; the fit moves to F.6, and the formatter it changes
        is `uci::info::format_score`. The defect beside it is fixed: an
        ordinary score of 19,873 or more, the rule-50-downgraded mate
        (`TB_WIN_SCORE - 1`, printed `cp 31743`) among them, read above every
        tablebase win; ordinary scores now stop at `cp 19872` (`8656ed0`).
      - **PROTO-6: the critical exit stays**, now through `main`: a `position`
        that cannot be read returns `CommandOutcome::Fatal`, input stops at
        that line and the engine shuts down as at the end of input, so a
        search already running, `go infinite` and ponder included, answers
        with its own `bestmove` before the exit with status 1. The decision's
        `bestmove 0000` was not used: the search's move is legal for the
        position the GUI asked about and needs no second shutdown path.
        Castling rights the rooks do not support and a halfmove clock over
        255 are covered by the same decision and pinned by a test
        (`09e9c8f`). Retry trigger as recorded: a logged game in which a GUI
        sends a recoverable bad `position`.
      - **Basilisk H14 and H15** (`6c8f554`): a helper thread's panic reaches
        the report with its name (the hook is the process's); a throwaway
        release build that panicked helper 1 printed `info string PANIC
        thread=rarog-search-0 ...` on stdout before exiting `0xC0000409`, so
        the shipped binary gets no crash trigger for a process test. A mated
        or stalemated root under `go infinite` reports one line and answers
        `bestmove 0000` only on `stop`, at Threads 1 and 2.
      - `multipv_syzygy`'s three tests are ignored with the tables they need
        named and fail when run without them (`7c4b5d6`; with the 3–6-man
        set all three pass); the queued `PonderHit` is gone (`9732e9b`);
        `bench` and `wac` share one `run_suite`, and `wac` gains `bench`'s
        check for a replacing command after each search (`27cd37e`).
      - **The move, D.8.13:** `cef6e97` (the option types to
        `search::options`, so the cycle is gone; `uci::position`, `uci::go`
        and `uci::options` parse and return their notices, which `uci`
        prints; one option table gives the advertised line and the range;
        `OptionUpdate` is `Applied`, `ClearHash`, `Rejected { notice }` or
        `Unknown { name }`, and a rejected value no longer queues a
        reconfiguration), `d0759b9` (the search hands its sink an
        `InfoReport`, an iteration's `InfoRecord` or `NoLegalMove`, and
        `uci::info` builds the line once).
      - `go mate N` as a fixed-depth search stays noted, low priority.
      - Verified at each engine commit: `bench 13` = 14,749,043 on the base
        and pext builds, debug and release suites, `cargo fmt --check`,
        `cargo clippy --all-targets` with all and default features at zero;
        the move's UCI output is the same before and after, timings masked,
        for a 26-command session (45 lines), a MultiPV 3 search (34) and a
        KQ-KR tablebase root at MultiPV 1 and 2 (30). Measurements in
        `analysis/artifacts/d3-9/`.
    - **D.3.10 `tests/uci_process.rs` times out under the full suite's load — `I1`, DONE 2026-11-03.**
      Found 2026-11-01 in D.2's checks (D.3's text above): under a full
      `cargo test`, debug or release, 1 to 5 of the file's 28 tests time
      out with no engine output, a different set each run, on the unchanged
      head too; read again 2026-11-02 in D.5.19's verification (1 in debug,
      4 in release; 28 of 28 with the file alone, both profiles). A test
      that fails only under load is still a failure. Cause unread; the
      start-up of a fresh process under the suite's load first. On
      2026-11-03 four full runs (D.6.15's and D.3's, debug and release, the
      file holding 28 tests and then 36) timed out none: intermittent.
      **Read and repaired 2026-11-03** (`fec05ef`; `analysis/artifacts/d3-10/`).
      In the logged failures (D.6.15's first run, 2026-11-02 22:48, 1 in
      debug and 5 in release, `analysis/artifacts/d5-item15/`) every test
      got `uciok` and then no line within its budget after its first
      command. The input thread prints `uciok` while the engine thread may
      still be building and touching its tables (`Engine::new`), and a `go`
      queued behind that has its clock running: the sessions skipped the
      `isready` UCI requires before the first search. With N fresh
      processes reaching their first `go` at once, `go` to the first line
      read 8 ms alone and 301 ms at N 36 in release (median; max 354), 38
      and 149 ms in debug at N 28. Large pages are not the multiplier: 309
      ms with them disabled in a throwaway build against 312 on, each
      build's `bench` line proving its state. `UciSession::ready()` now
      completes the handshake for all 26 sessions; no assertion or budget
      changed. **Not reproduced:** what stretched construction past 2 s on
      2026-11-01 and 2026-11-02. Neither the old file nor the new fails
      today: 27 runs each at three and six copies at once (up to 216
      processes), 8 each alternated under from-scratch release builds and
      8 each under a from-scratch `cargo test --release --no-run` (CPU 45
      to 77%): 0 timeouts, 0 `seen: []`. The repair removes the one
      load-scaling step from the timed windows and is verified only that
      far. **Retry trigger:** a `uci_process` timeout after `fec05ef`
      reopens the leaf with its log; a `seen: []` after `readyok` is a
      stall construction cannot explain. Verified: debug and release suites
      (36 of 36 in the file), `cargo fmt --check`, `cargo clippy
      --all-features --all-targets` at zero warnings; tests only, no engine
      input changed.
    - **D.3.11 `first_kpk_search_in_a_fresh_process_costs_what_a_warm_one_does` fails under CPU load — `I1`, DONE 2026-11-03.**
      Found 2026-11-03 in D.3.10's stress. The test asserts that a fresh
      process's first KPK search reports at most 15 ms more than a warm
      one; it failed in a full debug run on 2026-10-23
      (`analysis/artifacts/d16-cascade-2026-10-23/verify15.log`) and once in
      the 18 runs at six copies of the file at once (D.3.10's stress, the
      repaired file; the assertion is the same in both). It already waits
      for `readyok`, so construction is outside its window; an absolute
      15 ms margin under contention is inside it. Cause unread: the cold
      search's own first touches against the scheduler's noise first. A
      test that fails only under load is still a failure.
      **Read and repaired 2026-11-03** (`0c970d9`; `analysis/artifacts/d3-11/`):
      the scheduler's noise. The test's own sequence in 48 processes at
      once, debug: with only the cold searches started together, cold past
      warm + 15 in 11 of 192 and a warm search past another warm one in
      none, but with every search index started together (the same
      contention for each) 3 of 288 against 6 of 288, release 0 against 2:
      no cold-only cost, one delay of 15 ms or more landing on the one cold
      search. The bitbase is built before the engine reads input, as the
      test guards. A table built inside the search costs every first
      search and a delay costs one, so the test now compares the fastest
      cold search of three fresh processes with the fastest of their six
      warm ones, the 15 ms margin unchanged. 48 debug copies at once, 5
      rounds: the one-pair form failed 3 of 240 (cold 18 to 29 ms against
      1 to 7), the new form 0 of 240. With the bitbase made lazy again
      (throwaway build) it fails in both profiles: fastest cold 19 ms
      against 0 in release, 671 against 0 in debug. The release margin is
      thin, 19 against 15; debug's 671 carries the guard if the build gets
      faster. Verified: debug and release suites (36 of 36 in the file),
      `cargo fmt --check`, `cargo clippy --all-features --all-targets` at
      zero warnings; tests only, no engine input changed.

## D.4.1

    - **D.4.1 Wall-clock overhead read — `V`, DONE 2026-11-03.**
      Timed from outside on an idle host, `30593ba` (bench 14,749,043): `go
      nodes 1000` 0.55 ms against Stockfish 19's 0.93 (0.28 against 0.57 at
      four threads); `go movetime` 50, 100 and 300 overshoot by 0.56–0.68 ms
      (median) at one thread and 0.78–0.94 at four, 1.44 at most, Stockfish's
      by about 0, inside the 1 ms at which a gap would go to a repair;
      `ponderhit` to `bestmove` 0.53–1.56 ms after a finished ponder (a 1 ms
      sleep loop). In games RAR-R30 already read it at `3+0.03`: at one thread
      1.2% of the charged time is held after the last `info` line, median 1
      ms. Clean: Rarog's clock and output distorted no `3+0.03` read; the
      four-thread holds RAR-R30 found are D.2's, not the clock's.
      Triumviratus found 6.15 ms against 1.24 this way, worth +29.8 at 5+0.05
      (TT-2). Scripts and logs: `analysis/artifacts/triumviratus-checks/`.

## D.4.2

    - **D.4.2 SPSA schedule read — `V`, DONE 2026-11-03.**
      The pinned runner, Colosseum CLI 0.5.0 (`8fa63cc`, the executable
      Basilisk pins too), sets c₀ = c_end·N^0.102 and asserts that the last
      iteration reaches c_end (`colosseum-core/src/spsa.rs:120-143` at the
      pin). Basilisk's live 0.5.0 block on this host runs c 1.9999, 1.8634, …
      0.9925 at iteration 961 of 2,000, as c₀/(k+1)^0.102 predicts. Rarog's
      last tune, on 0.4.0, ran 6.0000 to 2.7634, its c_end, at iteration 1,999
      of 2,000 (`tools/results/rar-r21-fit/run.log`). Fishtest's form: no
      fitted vector was explored at a reduced perturbation, as Triumviratus's
      were for a month (TM-2). The range rule (start ± 2.25c clear of a
      formula's change of nature) is F.6's decision.

## D.4

- **D.4 Instruments from the Triumviratus review: the measurement-integrity reads, a search-path perft, pruning and reduction pricing, the loss table by cell, an identical-tree counter read — `R2`, then `I1`/`I2`/`V`, DONE 2026-11-06.**
  Added 2026-11-03 (maintainer: the instruments that help now, placed just
  before the next cluster that uses them;
  `analysis/triumviratus_review_2026-11-03.md` §13b). Rarog has their
  neighbours (pooled-PGO NPS, the `diag` counters, the ablation mask, the
  conversion audit), but nothing that prices what a pruning rule hides, splits
  a lost game's errors by cell, or reads speed below the pooled read's limit.
  None changes the tree or a gate; each is validated on a planted case before
  it is trusted, and its readings screen, never accept. First after D.3
  (maintainer decision 2026-11-03): the instruments come before the repairs
  (D.5) and the candidates (D.6) whose cards are their first users; the two
  measurement-integrity reads are done.
    - **D.4.1 Wall-clock overhead read — `V`, DONE 2026-11-03.**
      Clean, no repair: `go nodes 1000` costs 0.55 ms against Stockfish
      19's 0.93, `go movetime` overshoots by 0.56 to 0.94 ms (median),
      inside the 1 ms that would open a repair, and `ponderhit` answers in
      0.53 to 1.56 ms. Record: `docs/archive/PLAN-phaseD-done-2026-11-04.md`.
    - **D.4.2 SPSA schedule read — `V`, DONE 2026-11-03.**
      Clean: the pinned Colosseum CLI 0.5.0 runs the perturbation down to
      its c_end on the c₀/(k+1)^0.102 schedule, and Rarog's last tune
      reached its c_end at its last iteration, so no fitted vector was
      explored at a reduced perturbation. The range rule is F.6's decision.
      Record: `docs/archive/PLAN-phaseD-done-2026-11-04.md`.
    - **D.4.3 A search-path perft — `I1`, DONE 2026-11-04.**
      Perft through the staged picker (the TT move, then the noisy and quiet
      stages in the search's order), make and unmake, recomputing every key
      and derived field from scratch at each node (`check_consistency`) and
      re-checking every generated move with the TT move's legality test
      (`legal_move`), on the standard perft positions in release against their
      reference counts. `tests/board_differential.rs` does the recompute on
      random walks through `generate_legal_moves` only. Planted case: a picker
      that drops one stage's move fails it. Before D.6.1 and D.6.2, which
      change the picker's partition, and D.8.4–D.8.5, which move the board.
      Triumviratus's `tdperft` (TB-1) ran 482M nodes with 0 errors and
      validated a board rewrite.
      Done in `495adb9`: `src/search/path_perft.rs`, a test module run by
      `cargo test --release`. Each node takes a TT move from its key (none,
      a legal move as stored, the same move re-read from its UCI text with
      its flags lost, or arbitrary bits), puts it through `legal_move` as
      the search does, and drains the staged picker; each emitted move is
      re-tested by `legal_move`, checked for a repeat, made with the
      search's check hint and unmade, with `check_consistency` after both
      and the key, castling, en passant and clocks compared after the
      unmake. Release reads the six standard positions at depth 4 or 5,
      16.0M leaves in about 5 s, every count the reference and every stage
      emitting; debug reads them shallower. Five planted faults fail it:
      the picker losing its last bad noisy move (4,800,978 leaves against
      4,865,609 from the start position), the TT move left in the noisy or
      the quiet list (a repeat), a check hint that never reports check
      (the checker set desynced) and a re-read TT move played without its
      flags restored (fails the legality test). The committed test drops
      one move from each stage in turn and requires a wrong count.
    - **D.4.4 Pruning and reduction pricing — `R2`, then `I2`, DONE 2026-11-05.**
      What each pruning rule and each reduction hides (TM-9, Triumviratus's
      `prunelog` and `redlog`): a deterministic sample of the moves each rule
      prunes (razoring, reverse futility, null move, ProbCut, futility, the
      move count, SEE, and quiescence's rules) and of the reduced searches
      that fail low is searched again at the depth the rule denied, with the
      node's window. Recorded per rule, depth, node type and move number: the
      decisions, the share that would have raised alpha, and the share that
      would have changed the node's best move. The label that matters is a
      move that failed low reduced and beats alpha at full depth (1.7% of
      probed moves there); a label of beating alpha at the reduced depth spent
      nodes on moves already re-searched (−9.65). The card decides the form:
      the `diag` build may not change nodes, and a re-search writes the TT and
      the histories, so the pricing runs as its own feature or on scratch
      tables. Validation: with sampling off the bench is exact; a planted
      over-pruning rule reads higher than its honest form. Users: D.6.3,
      D.6.5, D.6.16, D.6.19 (H.2's since 2026-11-05), and F.6's fit.
      Card 2026-11-04 (`analysis/pruning_pricing_card_2026-11-04.md`):
      READY_FOR_IMPLEMENTATION, `I2`. The probe runs inside the search
      under its own feature, `pricing`, never read together with `diag`.
      While it runs, stores go to a per-thread scratch table read before
      the main one, which is never written; the histories and corrections
      are frozen; the stack and PV rows from its ply up, the node count
      and the search statistics are saved and restored; no probe nests;
      a node-level probe switches its one rule off at its own ply only.
      A pruned move is searched at `depth - 1` with `(alpha, alpha + 1)`;
      an LMR fail-low at its unreduced depth; a node-level cut is wrong
      when the node without the rule scores below beta; a raise also
      records whether the node then failed low. The sample hashes the
      decision against a per-rule rate set from probe cost alone (about
      10,000 probes a rule on `bench 13`). Validation: sampling off
      benches 14,749,043, and so does the main search with sampling on
      (the isolation proof), two runs write identical records, and
      `CoreFpBase` lowered from 150 to −100 reads a higher futility share
      than the default with disjoint 95% intervals. Predictions are frozen
      in the card. D.6.5 and D.6.19 need counters, not prices: exclusion
      searches are not priced, and a reduction the engine never makes has
      no decision to re-search. If the isolation proof fails and the leak
      is not closed, the leaf returns to `RESEARCH` with a scratch
      `Searcher` as the next form.
      Built 2026-11-04 in `7c2fec0`, read by `tools/diag/pricing_table.py`
      (`8a5fb6a`; the card's section 12). With sampling on the main search
      still benches 14,749,043 beside 317,670 probes, two runs write
      identical records, and the default build is unchanged. The futility
      plant separates (0.13% to 1.95%); the registered LMR plant does not
      (1.60% to 1.94% on quiets), nor does a reverse-futility plant run
      beyond the registration. Read by depth, both plants are higher in
      every band: a global option moves the depth mix and acts inside the
      probe's own subtree, so V4's premise holds only for rules whose
      probes contain none of their own decisions. Back to `RESEARCH`, the
      question narrowed to a plant test valid for every rule (a plant
      confined to the measured search, compared within depth bands,
      registered first). Until it passes, only futility's label is
      plant-validated and no screen reads the LMR or node-level labels.
      The plant test was registered 2026-11-05 (the card's section 13):
      one plant per rule, confined to the measured search by swapping the
      compiled-in parameters into every probe, read as a Mantel–Haenszel
      odds ratio within depth, node-type and move-class strata.
      Read 2026-11-05 (section 14; confinement `6b95225`, the read
      `ccff1a9`): the swap is neutral, isolation holds under every plant,
      and twelve labels pass (reverse futility, ProbCut, LMP, quiet and
      bad-noisy futility, history, quiet SEE and the five quiescence
      rules). Razoring fails: a lower margin's extra decisions are wrong
      less often, which is either a razor-probe defect or a plant in a
      region quiet futility already covers. The LMR label is unresolved
      (confinement changed nothing: the plant's effect is small), and the
      null-move and noisy-SEE plants were inactive. Those four labels
      stay out of every screen; D.4.4 stays in `RESEARCH` for a placebo
      read of the node-level probes and stronger plants, each a new
      registration. Registered 2026-11-05 (section 15): an identity
      probe per node-level decision tests the re-entry, probes reading
      only their scratch table test whether the planted search's table
      leaks into them, and stronger plants for the null move (entry
      margin 0), LMR (two plies) and noisy SEE (threshold 0).
      Read 2026-11-05 (section 16; `9432eb2`, `3b3767e`): every
      identity probe reproduces its decision (at most 4 of 12,737
      unreproduced), so the re-entry is right; razoring fails cold as
      it did warm, so the table does not explain it; the null-move, LMR
      and noisy-SEE plants pass. Fifteen of sixteen labels are
      plant-validated. Razoring's probe is verified but its harm is not
      shown by its own margin, so its label stays out of every screen
      until a plant harmful on its own passes, read first by the step that
      wants it (the card's retry trigger). Done.
    - **D.4.5 The loss table by cell — `V`, DONE 2026-11-05.**
      Where Rarog loses, from games already played (TT-1): every move scored
      by a judge (Stockfish 19 at fixed nodes) as the evaluation before it
      minus the evaluation after, from the mover's side, tabulated for both
      sides by position class, clock left and material with the opponent's
      moves in the same cells as the control, and the worst errors searched
      again at 4× and 16× the game's nodes to split clock errors from
      evaluation errors. Done as an instrument: `tools/diag/loss_table.py`
      (`6a48602`; the identity corrected in `158f52f`, a defect the first
      smoke found; Ctrl+C stops a run and keeps its finished games,
      `c124b2e`), its smoke repeating byte for byte across workers, a
      resume, an interrupt and two re-searches
      (`analysis/loss_table_card_2026-11-05.md`, registered `c2381c0`,
      sections 10 and 11). The registered run on the C head's 1,896 lost,
      spoiled and twin games was withdrawn before any reading (maintainer
      decision 2026-11-05): its consumer is H.3, which tunes the network
      head's clock, so the C head's games would describe an engine nobody
      will tune then. 164 games were judged before the stop, at a pace of
      5.4 hours for the run against the smoke's 3.4 (fifteen workers each
      about 1.6× slower than one alone); they stay unread in
      `analysis/artifacts/d45-loss/`. The read is H.3's first step, on the
      network head's games, registered afresh.
    - **D.4.6 An identical-tree counter read — `R2`, NO_CHANGE 2026-11-06.**
      Speed of a change that keeps the tree, by hardware counters (TM-7), on
      the four PGO builds of one source, pinned to one core. No counter beats
      the pooled builds: the thread cycle count is the time-stamp counter,
      so wall time, and ETW's core cycles vary only a sixth less than NPS (CV 0.267%
      against 0.322%). The pooled read stays the speed check; there is no
      `I1` step. Instructions retired failed the frozen rule, CV 0.051%
      against 0.05%, wholly through a split nobody predicted: a process falls
      at random into one of two states on the same tree, 0.09% apart in
      instructions and 0.40% in NPS, cause unknown, H.4's lead
      (`analysis/identical_tree_counters_card_2026-11-05.md`;
      `tools/diag/identical_tree_counters.py` stays as the instrument).

## D.5.22

    - **D.5.22 Measurement and gating methods from Basilisk — `I1`, DONE 2026-11-02.** (H38–H45; decided
      by the maintainer 2026-11-02). Adopted: a repair's registration
      states its measured exposure (AGENTS, *Gating*); a written analysis
      of the fitted constants before a bake, from
      `tools/spsa_theta_audit.py` (PROCESS, *SPSA go/no-go procedure*,
      step 9); the curvature sweep's noise floor (rule 8); `nps_read.py`
      warning when its arms search different trees; a second-step NPS
      accept below +1% on a change that moves code recorded as possibly
      layout (PROCESS, *Toolchain and harness notes*); a correct repair that fails its
      harm rule reverted by the gate with its defect left open and a
      rework registered (AGENTS, *Gating*). Declined for now, each with
      its retry trigger: the pinned bench, because the builds and not the
      cycles set the interval (retry when the disturbance rule repeats
      more than about one read in three); several pools in one read
      (retry at the first speed study with three or more arms); an
      ordered night-queue launcher (retry at the first chained run that
      started on a failed predecessor).

## D.5.24

    - **D.5.24 A `Hash` larger than the machine's physical memory is refused on every platform — `I1`, DONE 2026-11-04.**
      Found 2026-11-04 on PR #13's macOS CI: the refused-`Hash` test timed
      out with no output in both profiles. The table was bounded only by
      what the allocator accepted, and macOS grants a `Vec` reservation
      lazily, so 32 TiB succeeded and the engine spent the session filling
      it; Windows refuses the commit and Linux's overcommit heuristic the
      reservation. Neither donor bounds it: Stockfish exits the process
      when its allocation returns null, and Reckless advertises a fixed
      256 GiB maximum and does not check its allocation. `9dc8b81` put the
      test off macOS as a stopgap; `db4e9ae` repairs it: the engine reads
      the machine's physical memory once (`physical_memory`), advertises it
      in whole MiB as `Hash`'s maximum (33554432 where the platform does not
      say), and refuses a larger value as the option is parsed, with an
      `info string` naming the value and the limit; the setting is kept
      and nothing is allocated. Below the limit the allocator's own
      refusal keeps its path with the table kept, tested at search level
      with a reservation no platform can make, and the process test runs
      on macOS again. **Every `Hash` sent is checked and answered**
      (`488608c`, maintainer's rule 2026-11-04): `Configure` carries whether
      the `setoption` was `Hash`; a sent value is tried whenever it differs
      from the table in use and a refusal announced each time, while any
      other option keeps the size in use and says nothing about `Hash`.
      D.3.7's rule (retry only on a new value) left the same value sent
      again after memory was freed neither tried nor answered. A search
      runs on the table in use, the last size that was granted. The limit is total, not
      available, memory: available memory moves with the file cache and
      other processes and would refuse a table that fits. **The unsafe
      floor is 24** (maintainer decision 2026-11-04): Rust has no
      physical-memory API, so the query is FFI. Windows adds
      `GlobalMemoryStatusEx` to the existing `win32.rs` block as a `safe fn`
      (it writes only into the struct a reference guarantees), so its call
      needs no `unsafe`; Linux and macOS read `sysconf` through one new
      `unsafe extern "C"` block whose single item is a `safe fn`. A
      subprocess (`sysctl hw.memsize`) was the only unsafe-free route on
      macOS and was not taken. Verified: debug and release suites, `cargo
      fmt --check`, `cargo clippy --all-targets` with all and default
      features at zero, `bench 13` = 14,749,043 on the base and pext builds;
      the advertised maximum read 130,982 MiB here, Windows' own figure. The
      Linux and macOS paths are compiled first by CI. A container whose
      memory limit is below the host's still reports the host's memory.

## D.6.15

    - **D.6.15 The correction is added to the evaluator's draw verdicts — `I1`, DONE 2026-11-03.** (A1;
      Basilisk's B.14, accepted as BAS-S44 on 2026-11-01). The evaluator
      returns exactly 0 for the drawn KPK, the wrong-corner KBP-K,
      KK/KNK/KBK and minor against minor, and KNNK
      (`eval/endgame/mod.rs:52-58`, `148-209`); `corrected_eval_parts`
      adds the correction to every raw value (`search/correction.rs:173-188`),
      and the node-end training runs inside exclusion searches and on
      decisive scores (`search/node.rs:2087-2096`), the configuration in
      which Basilisk's shared structure keys moved its wrong-bishop draws
      off 0. A verdict is a classification with no error to correct.
      Screen: a debug probe of raw against corrected evaluation at those
      leaves after `bench 13` and after searches from Basilisk's
      wrong-bishop fixtures. If the corrected value leaves 0: the
      evaluator reports the verdict and the search adds no correction to
      it, with a test that fails on today's code; gate case 1 by a static
      differential (only nodes whose evaluation is a verdict change). If
      it never leaves 0, `NO_CHANGE` with the test kept. A cold probe is a
      false negative (H17): from cleared tables Basilisk's fixtures read 0;
      the defect appeared only after priming in one process (`ucinewgame`,
      `go depth 12` on `8/8/5K1P/8/8/8/k7/5B2 b - - 0 1` and
      `8/8/6KP/8/8/8/8/k4B2 b - - 0 1`, then the fixtures: 4 of 28 failed
      at +172), and organically on `5k2/8/6KP/8/8/8/8/5B2 b - - 0 1` (only
      Kg8 draws), +90 at depth 11 within 3,075 nodes: the exclusion search
      excludes the one drawing king move and its training lands on every
      structure table. A guard keyed on `raw == 0` moved 36–38 of 40 bench
      positions (most zeros are ordinary); keyed on the verdicts, 4–6 of
      40 searched alone, yet bench fell 28% through shared state, so the
      differential takes a fresh process per FEN. BAS-S44 read +7.4 ± 6.9
      (1,271 pairs); Basilisk also keeps a producer guard (no exclusion
      training at a raw 0), the alternative form.
      **Probe read 2026-11-02** (`analysis/draw_verdicts_2026-11-02.md`,
      §1; diag patch `analysis/d5_item15_verdict_probe.patch`): **the
      corrected value leaves 0.** In `bench 13` (fingerprint unchanged
      under the probe) 703 of 712 verdict leaves carry a correction, 434 by
      50 or more; in Basilisk's fixtures 98–99% do, by 77 to 117 on
      average, and the fixtures misread: 28 of 28 beyond ±75 from cleared
      tables (+123 to +252), 24 of 28 primed, the organic position −110 at
      depth 14 (0 at 8–13). Rarog needs no priming: the cold arm's first
      search, in a fresh process, read +123. **Repair card,
      `READY_FOR_IMPLEMENTATION` (`I1`).** *Mechanism:* a verdict is a
      classification; the search takes it as the static evaluation, 0,
      with no correction, at both static-evaluation sites (main search and
      quiescence), recognised by the verdict and never by `raw == 0`.
      *Interactions:* the table refinement of the static evaluation by a
      stored bound is search evidence and stays; the correction's training
      at a verdict node is unchanged (Basilisk's producer guard is a
      separate candidate, not this repair); a raw value read from the table
      is not re-evaluated, so the node recognises the verdict itself, only
      when the raw value is 0 and a recogniser can apply (five men or
      fewer, or a bare king; the probe's `draw_verdict_kind` is that
      test), one comparison elsewhere, and an NPS read by PROCESS's method
      only if a profile shows it. *Invariants and tests, failing today:* a
      verdict's corrected static evaluation is exactly 0 with trained
      tables; Basilisk's four fixtures read within ±75 at depths 8–14
      from cleared tables and primed (the cold `7k/8/6KP/8/8/8/8/5B2 w` at
      depth 8 reads +123 today). *Gate:* case 1 (AGENTS, *Gating*): the
      fingerprint moves, so its correctness read is a static differential
      with a fresh process per bench FEN (bench shares its tables across
      positions): every FEN whose baseline search reaches no verdict leaf
      (the probe's counters, per FEN) reproduces its node count exactly,
      and only FENs that reach one may move; the registration states its
      exposure from Colosseum's database. *Prediction, frozen 2026-11-02
      before any implementation:* no verdict leaf with a moved value in any
      arm (0.98); the cold fixtures all within ±75 (0.70), the primed all
      (0.65), the organic depth 14 (0.75); the per-FEN differential moves
      only FENs with a verdict leaf (0.85); in games about zero.
      *Falsifier and stop:* if 7 or more of the 28 cold fixtures still read
      beyond ±75 with no verdict value moved, the verdicts are not the
      fixtures' cause: the repair stays if its differential is clean (it is
      a correctness repair), and the fixtures' misread returns to
      `RESEARCH` (the training producer, H17's shared keys) as its own
      card.
      **Implemented and read 2026-11-03** (`3a16fbb`;
      `analysis/draw_verdicts_2026-11-02.md` §5): `corrected_eval_parts`
      returns 0 with no correction when the raw value is 0 and
      `is_draw_verdict` holds, a predicate that follows `scale_endgame`'s
      own order and is tested against its zeros. The fingerprint moves from
      13,987,223 / EBF 2.565 to **14,749,043 / EBF 2.564**. **Gate, case 1:**
      the per-FEN static differential (a fresh process per FEN, depth 13)
      is confined: FENs 3, 6, 37 and 40, each reaching verdict leaves,
      moved, four others that reach a few did not, and all 32 FENs without
      a verdict leaf reproduce their node counts exactly. Correctness read: in bench and in every fixture arm
      no verdict leaf keeps a moved value (703 of 712 in bench before).
      *Calibration:* no moved verdict (0.98) and a confined differential
      (0.85) held. The fixtures missed: from cleared tables 7 of 28 still
      read beyond ±75 (+83 to +103, against 28 of 28 at +123 to +252),
      after training 9 of 28 (24 before); the one-drawing-move position
      read −110 at depth 14 in the probe's process, unchanged, and within
      ±75 in a fresh searcher (predicted all within ±75 at 0.70, 0.65 and
      0.75). The miss is in coverage: the verdicts carried
      most of the misread, not all of it. **The falsifier fired** (7 of 28
      cold): by the stop rule the repair stays, its differential clean, and
      the remaining misread is D.6.23, whose acceptance test is committed
      `#[ignore]`d.
      **Measured exposure, 2026-11-03** (AGENTS, *Gating*;
      `analysis/draw_verdicts_2026-11-02.md` §6): of the 38,860 games of
      the three latest SPRT gates (`3+0.03`), 45.8% reach five men or a
      bare king on the board, where the search's leaves can be verdicts,
      and 3.5% play through a verdict before their last position (7.4%
      counting the last). Case 1 stands by its rule: the differential
      confines the change. No strength gate is registered; a
      maintainer who wants one has case 2's 2,000-game harm read.

## D.8.13

    - **D.8.13 P12: the adapter boundary, inside D.3 — `I1`, DONE 2026-11-04.** Done inside D.3.9 (`cef6e97`, `d0759b9`; the sink's method is `report(&InfoReport)`, which also carries the no-legal-move line). The adapter boundary, inside D.3: the option types to
      `search/options.rs`, the parser and its notices to `src/uci/` (planned),
      `InfoSink::info(&InfoRecord)` so `format_score` and the bound and mate
      strings leave the search and the `info` line is built once; removes the
      `search` and `search_options` cycle. Neutral; `tests/uci_process.rs`
      pins the text.

