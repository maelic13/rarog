# RAR-O09 — The donor's search against the head on Rarog's evaluation: equal work, equal move time, and clocks at `3+0.03` and `10+0.1`

Indexed under *3. Search and selectivity › Search-oracle observations* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-13, before any game and before the arms are built. Preparation is D.7.2's; the run is the maintainer's command (four fixed matches, about three hours).
- **Kind:** observation with a frozen use: PLAN D.7's decision rule reads match (1).
- **Inherits:** [RAR-O08](RAR-O08.md) match (5): `reckless-hce` **+22.50 ± 7.85** over `rarog-head` at `3+0.03` with the bridge returned as time odds. That figure bundles tree shape, kernel speed and clock use.
- **Revisions:** Rarog engine source `112eb14` (bench **8,935,848 / EBF 2.480**). Reckless `0756957` with `tools/oracle-hybrids/reckless-0756957-rarog-hce.patch` (`c546d2c4…`), linking the same Rarog source. The evaluation is frozen, so a later engine head changes the arms' names and hashes, not the question; the result names what was built.
- **Arms** (built in D.7.2 by `tools/oracle-hybrids/README.md`'s recipes; SHA-256, `id name` and bench go in the result):
  - `rarog-head`: `cargo xtask build --tier pext` (PGO) of the clean head.
  - `reckless-hce`: the patch above, `cargo pgo` with `target-cpu=native`, no Syzygy, `MoveOverhead=10`.
  - `reckless-hce-count`: the same tree with one counter added on the null move, printed after a search. Never played; it measures the node unit below. Its bench node count must equal `reckless-hce`'s, which proves the counter changes nothing.
- **Research question:** of the donor's 22.5 at equal clock, how much is the tree's shape (the same work per move), how much the kernel's speed, how much clock use, and does the surplus hold at `10+0.1`?
- **Why now:** D.7 rebuilds the kernel only if the donor's shape is better per unit of work on this evaluation, or the maintainer decides so after the reads.
- **Instrument, known today:**
  - *Node unit.* Rarog counts a position when a move or a null move reaches it (`src/search/mod.rs` `count_node`; called after the null move at `node.rs:1139`; D.6.8's contract counts the null move by rule). Reckless counts in `make_move` only (`search.rs:1475`), not on the null move (`:597`). Neither counts the root, a re-search, an exclusion search or a verification. A null-move child is a full node of work, so equal `go nodes` hands the donor its null-move children free.
  - *Node limits.* Both are hard and both finish the first iteration: Rarog stops at `nodes >= N` (`mod.rs:1907`), Reckless at `nodes > N` (`time.rs:80`), each tested at every node.
  - *Move time.* Rarog thinks for all of `go movetime T` (`src/search/time.rs:132-147`). Reckless stops at `T − 15 ms` (`time.rs:20`, `:55-56`) and ignores `MoveOverhead` there. Equal thinking time needs unequal commands.
  - *Budgets.* The head's own mean at `3+0.03` in Colosseum's summaries is 130,493 to 130,889 reported nodes and 66.5 to 66.9 ms a move (RAR-S124 and RAR-M87's head-to-head, `results.txt`).
  - *Opening counters.* Colosseum CLI 0.5.0 starts every EPD opening at `0 1` (`colosseum-engine/src/openings.rs:343-355`); the book's positions are at moves 3 to 9. Both time managers read the move number: Rarog's optimum scales with the game ply (102.9 ms on the first move against 121.7 ms under fastchess, RAR-M88), and Reckless's soft bound at `3.380+0.034` is 51 ms at move 1 against 93 ms at move 9, before its multiplier (`time.rs:35`, `:55`). The clock legs therefore differ by runner until Colosseum keeps the counters; matches (1) and (2) do not.
  - *Harness wires.* `sprt.ps1`'s per-engine node budgets and move times are proved live (D.7.1). Colosseum CLI 0.5.0's `match` resolves all four legs in dry runs; its wrapper needs D.7.1.1 for a move time, a per-side node budget and a declared foreign arm.
- **Instrument, measured in preparation, before any game** (each value is written into this record before the first game):
  1. *Null-move share `u`.* `reckless-hce-count` at `go nodes 130000`, Hash 64, a fresh process per position, on (a) the 40 bench positions and (b) 200 game positions: the position after every 20th ply of the first games of RAR-O08's match (5) PGN until 200 are collected. `u` = null moves made / nodes counted, summed over the set. The two sets must agree within 1.5 percentage points; (b) is the value. Expected 1% to 3%. Outside 0.5% to 8% the preparation stops and reports: the count is then not what this registration assumes.
  2. *Node budgets.* `rarog-head` 130,000; `reckless-hce` 130,000 / (1 + `u`), rounded to the nearest 100.
  3. *Time odds.* `tools/oracle-hybrids/bridge_price.py` on the rebuilt `reckless-hce`, three cycles, identical node counts required. Expected ×1.127 (RAR-O08); outside ×1.10 to ×1.15 the preparation stops and reports.
  4. *Move times.* `rarog-head` 70 ms. `reckless-hce` 70 × odds + 15 ms, rounded to the millisecond (94 ms at ×1.127, so it thinks 79). A 40-game smoke must read mean time a move of 70 to 74 ms and 79 to 83 ms and a ratio of 1.10 to 1.155; otherwise the donor's command moves by whole milliseconds until it does, before any game.
  5. *Clocks.* `rarog-head` `3+0.03` and `10+0.1`; `reckless-hce` the same times the odds, rounded to the millisecond (`3.380+0.034` and `11.270+0.113` at ×1.127).
- **Interacting mechanisms / confounds:**
  - *Unfitted donor.* The donor's constants are fitted to its network. A deficit at equal work is not what a fitted rebuild would reach; a surplus is a floor.
  - *The shares are differences of matches* (about ±11 Elo at 95% each) and assume Elo adds across conditions. They are descriptive.
  - *Speed is entangled with shape.* A tree with more quiescence nodes costs less per node. "Kernel speed" here means positions per second in games, whatever the cause.
  - *The bridge's price comes from bench*, not from games.
  - *Runner.* Match (3) replicates RAR-O08's match (5) only on the same runner or with the book's counters kept.
- **PRE-REGISTERED PREDICTION (frozen 2026-11-13, before any game).** Elo of `reckless-hce` over `rarog-head`, 80% bands:
  1. Equal work: **0** [−20, +20].
  2. Equal move time with odds: **+13** [−5, +31].
  3. Clocks `3+0.03` with odds: **+22** [+10, +34] with the book's counters; **+15** [0, +30] with them reset.
  4. Clocks `10+0.1` with odds: **+10** [−8, +28].
  - Speed share, (2) − (1): **+13** [−8, +34]. Clock share, (3) − (2): **+9** [−9, +27].
  - Probability that the decision rule fires on (1): 0.20. Probability (1) is at or below zero: 0.50.
  - Basis. *Clock:* the donor spent 94.9–95.1% of its budget against the head's 87.1–88.9% (RAR-O08), about 8% more thinking time, 16 Elo at PROCESS's 2 Elo per 1% if it were evenly useful; taken as +9 because it is front-loaded. *Speed:* RAR-O08's tree-shape read (unregistered; 40 positions to depth 18, wall time over 40 fresh processes) took 40.2 s for the donor's 111.6 million counted nodes and 46.4 s for the head's 151.1 million, of which 16.1% were counted twice then: about 2.8 against 2.7 million positions a second before the odds, so about 1.1 to 1.2 times the head's with them. *Shape:* what is left, about zero. This lowers the +10 [−15, +35] of `analysis/search_shape_decision_2026-11-13.md` section 7, which did not use the speed read.
  - Expected Elo for Rarog: none. These are instruments.
  - Most likely failure mode: the shares do not add across conditions, so (1) sits well away from the remainder.
- **Use of the reading, frozen:**
  - *PLAN's rule.* (1) at +10 or more for the donor with its 95% interval excluding zero opens D.7.3 to D.7.9. Below that the agent reports with a recommendation and the maintainer decides.
  - *The recommendation, fixed now.* (1) at or below zero: against a rebuild for strength; the donor's surplus is then speed and clock, and the case for D.7 rests on its second aim alone (the problem review). (1) between zero and +10: unresolved for strength; the report says so and weighs cost and the second aim.
  - *Speed share* at +10 or more with its interval excluding zero: reported as a throughput lead for the head (per-position cost against the donor's kernel). It opens nothing by itself.
  - *Clock share:* evidence for D.6.28. It opens nothing by itself.
  - *(4)* at or below zero with (3) positive: the surplus is a short-control effect; it goes to the adoption judgment.
  - *(3)* further than 16 Elo from RAR-O08's +22.5 on the same runner with the book's counters: the replication failed; the shares are then read from this experiment alone and the difference is reported.
- **Falsification criteria:** each band as stated. The node-unit correction is an instrument assumption with its own stop (`u` outside 0.5% to 8%).
- **Registered design and stop rule:** no gate. Four fixed matches in the order above, 3,000, 3,000, 3,000 and 2,000 games, no early stop, no extension, no re-run on the same binaries.
  - *Conditions:* UHO_Lichess_4852_v1 in random order, seed 20261114, Hash 64, one thread, 14 concurrent games, no adjudication, a 20 ms time margin, `MoveOverhead=10` for `reckless-hce`.
  - *Runner, one for all four, chosen and written here before the first game:* Colosseum CLI through `tools/colosseum.ps1` if D.7.1.1 has landed and the pinned CLI keeps an EPD line's move counters (request 2 of `analysis/colosseum_requests_2026-11-13.md`); otherwise fastchess through `tools/sprt.ps1`, as RAR-O08. If the maintainer chooses Colosseum on a CLI that resets the counters, (3) and (4) are read under prediction (3)'s second band and the clock share is not compared with RAR-O08's match (5); (1) and (2) are unaffected.
  - *Validity, read from each run's own files:* every game ends by a rules result; a leg with time losses or engine faults in more than 0.5% of its games is void. (1): no move reports more nodes than its side's budget plus one. (2): the two sides' mean time a move in the ratio 1.10 to 1.155. A void leg is replayed once, with seed 20261115, after its cause is fixed and recorded.
  - *Scoring:* pentanomial, 95%, Elo of `reckless-hce`: `tools/pgn_result.ps1` on the PGN (fastchess) or the wrapper's manifest (Colosseum).
  - *fastchess form* (`sprt.ps1` takes these today; `<N>`, `<T>` and the clocks come from the preparation):

    ```powershell
    ./tools/sprt.ps1 -Mode fixed -Games 3000 -NoAdjudication -Seed 20261114 -EngineA <rarog-head.exe> -EngineB <reckless-hce.exe> -NameA rarog-head -NameB reckless-hce -OptionsB MoveOverhead=10 -NodesA 130000 -NodesB <N>
    ./tools/sprt.ps1 -Mode fixed -Games 3000 -NoAdjudication -Seed 20261114 -EngineA <rarog-head.exe> -EngineB <reckless-hce.exe> -NameA rarog-head -NameB reckless-hce -OptionsB MoveOverhead=10 -MoveTimeA 0.070 -MoveTimeB <T>
    ./tools/sprt.ps1 -Mode fixed -Games 3000 -NoAdjudication -Seed 20261114 -EngineA <rarog-head.exe> -EngineB <reckless-hce.exe> -NameA rarog-head -NameB reckless-hce -OptionsB MoveOverhead=10 -TCA 3+0.03 -TCB 3.380+0.034
    ./tools/sprt.ps1 -Mode fixed -Games 2000 -NoAdjudication -Seed 20261114 -EngineA <rarog-head.exe> -EngineB <reckless-hce.exe> -NameA rarog-head -NameB reckless-hce -OptionsB MoveOverhead=10 -TCA 10+0.1 -TCB 11.270+0.113
    ```

    `-MoveTimeA` and `-MoveTimeB` are in seconds (`<T>` is 0.094 at ×1.127). The Colosseum form is written by D.7.1.1 with its dry runs.
- **Result:** not played.
- **Artifacts / commits:** this registration; the design is `analysis/search_shape_inventory_2026-11-13.md` section 6.
