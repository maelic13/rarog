# RAR-M92 — The price of the node-rate gap: the kernel on 1.35 times the clock against Stockfish 16 HCE, 1,600 games — REGISTERED 2026-11-23, before any game; READ 2026-11-23: −43.7 ± 11.8 Elo, a gain of +58.3 ± 17.3

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-23, before any game (PLAN E.0).
  Preparation and the dry run are the agent's; the run is the maintainer's
  command, one fixed match of 1,600 games, about 25 minutes.
- **Kind:** observation with a frozen use. It accepts and rejects nothing.
- Disposition: observation — read 2026-11-23: −43.7 ± 11.8 Elo at an equal node rate, a gain of
  +58.3 ± 17.3 over the baseline; 1.67 Elo for 1% of node rate.
- **Use, named before the read:** it prices E.0's speed levers. The packet
  (`analysis/e0_remaining_strength_2026-11-23.md`) states those levers in
  percent of node rate; this read gives the Elo each percent is worth
  against the pre-NNUE meter's engine, and with it the place of the speed
  work beside the evaluation programme. The result at an equal node rate is
  also the kernel's deficit a node, the figure E.4 is judged against.
- **Binaries:**
  - Arm A, `tools/test_engines/rarog-d713kern-pext-pgo.exe`: the kernel
    RAR-S136 gated and RAR-M90 rated (`eb84cce`, `bench 13` 3,390,842 /
    EBF 2.317, SHA-256 `E2AD1268…AC99`), with its sidecar. It is `master`'s
    kernel before D.6.7's ply cap and D.6.29's stalemate rule; it is used so
    that RAR-M90's head to head is this read's baseline on the same binary
    and runner.
  - Arm B, Stockfish 16 HCE:
    `D:\chess\engines\stockfish-16-x64-bmi2-classical-pgo.exe` (SHA-256
    `041396A4…5442`, `id name Stockfish 16`), RAR-M89's file with its
    options, `Use NNUE=false` and `Move Overhead=10`. A foreign arm
    (`-ForeignB`).
- **Research question:** how much of the kernel's deficit to Stockfish 16
  HCE does an equal node rate return?
- **Measured before, zero games** (the packet, *The cost of a node*):
  Stockfish 16 HCE searches 1.32 to 1.42 times the kernel's nodes a second,
  by every read: 1.32 on 281 game positions at 70 ms a move, alone and with
  14 running; 1.36 at 150,000 nodes a move; 1.35 in RAR-M90's games once
  the kernel's 4.0% of move time after its last report is taken out (1.42
  as the runner reports it); 1.48 in RAR-M91's games as reported. Both
  engines count a node when a move is made (D.6.31). The clock ratio is
  **1.35**, RAR-M90's corrected figure, the same binary on the same runner:
  `4.050+0.041` against `3+0.03`.
- **Baseline, played:** the kernel against Stockfish 16 HCE at `3+0.03`
  each, −100.6 [−119.4, −82.3] over 600 games (RAR-M90, this binary, this
  runner) and −103.3 [−121.0, −86.0] over 800 (RAR-M91, the merged build,
  the GUI); pooled by inverse variance **−102.0 ± 12.7**. It is not played
  again.
- **Hypothesis:** the kernel's own slope holds against this opponent. Three
  speed passes of 10% to 22%, each at an identical tree, read about 145 Elo
  a doubling at `3+0.03` in self-play (the 10.3 pass, RAR-S98, RAR-S99);
  1.35 is 0.433 of a doubling, +63.
- **Competing hypotheses:** (1) *a flatter slope against a foreign engine:*
  the kernel's gain over 2.7.0 was 0.81 as large in RAR-M90's field as
  head to head, so about +50; (2) *a flatter slope at a larger step or a
  greater depth:* the three passes were steps of at most 22%; (3) *the
  node rates are not one unit of work:* the two trees differ in their share
  of quiescence nodes, which cost less, so equal reported nodes need not be
  equal work. A time-odds read prices the kernel's own rate whatever the
  unit; it needs no node budget and no correction for the kernel's driver,
  which discards the interrupted iteration (RAR-O09).
- **Interacting mechanisms:** each engine keeps its own time manager, so
  the read includes how each spends a longer clock. The kernel's manager
  was fitted to nothing at this clock (its constants are the donor's,
  `analysis/d711_fit_2026-11-20.md`, *What stays unfitted*); RAR-S136 read
  the kernel's gain over the classic level at `10+0.1` and `3+0.03`.
- **PRE-REGISTERED PREDICTION (frozen 2026-11-23, before any game):**
  - The gain over the baseline: **+55** Elo, 80% band [+35, +75]; so the
    match itself reads **−47** [−67, −27] for the kernel.
  - Probability the gain is +45 or more: 0.70; +30 or less: 0.12; +63 or
    more (the whole self-play slope): 0.30.
  - The kernel's nodes a move over Stockfish's, from `results.txt`: 0.95 to
    1.05.
  - Expected Elo for Rarog: none; an instrument.
  - Most likely failure mode: reading the gain as available. It is the
    price of the whole gap; how much of the gap can be closed is the
    profile's question (PLAN D.16.3), not this read's.
- **Use of the reading, frozen:**
  - The price of 1% of node rate is the gain divided by 35, and a
    doubling's the gain divided by 0.433. The packet's speed bands are
    written at 1.6 Elo a percent (+55); they are restated at the measured
    price.
  - Gain at **+45 or more**: the bands stand, and the speed pass on the
    kernel's search-side work (D.16) keeps the place the packet proposes,
    ahead of the board rebuild (D.12) and before the evaluation
    programme's long units.
  - Gain at **+30 or less**: the bands are halved and the speed pass moves
    behind E.4.0's reads.
  - Between: the bands scale and the order is unchanged.
  - The match's own result is recorded as the kernel's deficit to
    Stockfish 16 HCE at an equal node rate.
  - Each line is read on the point estimate; the interval is reported
    beside it. Nothing here opens, closes or builds a step: the order is
    the maintainer's.
- **Falsification criteria:** the hypothesis fails if the gain is under
  +35; competing hypothesis (1) or (2) is then the reading, not separated
  from each other here.
- **Cheapest prior falsifier:** the zero-game reads above; they could have
  shown the rate gap to be a reporting artefact or a load effect and did
  not.
- **Registered design and stop rule:** no gate. One fixed match of 1,600
  games in 800 colour-reversed pairs, no early stop, no extension, no
  re-run on these binaries. The count is sized to the use: the match reads
  to about ±12.4 Elo at 95% (RAR-M91's 800 games gave ±17.5), the gain to
  about ±17.7 with the baseline's ±12.7, and the two lines sit 18 and 25
  from the prediction.
- **Validity, read from the run's own files:** every game ends by a rules
  result, no engine fault, time losses in at most 0.5% of the games; the
  kernel's nodes a move over Stockfish's between 0.90 and 1.10, outside
  which the result is reported as a time-odds read at 1.35 and not as an
  equal node rate.
- **Conditions:** RAR-M90's. Colosseum CLI 0.5.0 (the pinned runner)
  through `tools/colosseum.ps1 -Mode match`, Hash 128, one thread, 14 games
  at a time, a 2,000 ms margin on both sides, no adjudication,
  `UHO_Lichess_4852_v1.epd` in random order, seed 2026112392. The dry run
  (2026-11-23, no game played) resolved every field:
  `tools/results/colosseum_match_rar-m92-speed-price_20261123_114509.dry-run.json`,
  its console in `analysis/artifacts/e0-2026-11-23/m92_dryrun.out`.
- **Not registered, and why:** the same pair at equal nodes a move, which
  E.0's text names beside this read. It estimates the same quantity, the
  deficit at an equal node rate, through a worse instrument: under a node
  limit the kernel's last report covers 73% of its budget (109,271 of
  150,000 nodes on 281 game positions) against 96% on the clock, and the
  correction for that is itself worth more than the difference being read.
- **The command** (from the repository root, in PowerShell, in the
  machine's own terminal, on an idle host):

  ```powershell
  ./tools/colosseum.ps1 -Mode match -EngineA tools/test_engines/rarog-d713kern-pext-pgo.exe -EngineB D:\chess\engines\stockfish-16-x64-bmi2-classical-pgo.exe -ForeignB -NameA "Rarog d713kern at 1.35 clock" -NameB "Stockfish 16 HCE" -Games 1600 -Seed 2026112392 -Hash 128 -BaseMs 4050 -IncrementMs 41 -BaseMsB 3000 -IncrementMsB 30 -MarginMs 2000 -OptionsB "Move Overhead=10","Use NNUE=false" -ExpectBench 3390842 -ExpectRevision eb84cce -ExtraArgs '--a-base-ms','4050','--a-increment-ms','41','--a-margin-ms','2000','--b-margin-ms','2000' -Dir tools/results/rar-m92-speed-price
  ```

  The wrapper passes arm B's clock itself; arm A's clock and the two
  margins go to the runner through `-ExtraArgs`, and the wrapper checks
  the resolved configuration against `-BaseMs`, `-IncrementMs` and
  `-MarginMs`. Without them it refuses the run, as its first dry run
  showed.
- **Result (played 2026-11-23 10:55 to 11:17 UTC by the maintainer, 22
  minutes; `tools/results/rar-m92-speed-price/`, manifest
  `tools/results/colosseum_match_rar-m92-speed-price_20261123_115519.manifest.txt`):**
  - *Validity.* 1,600 games in 800 complete pairs, every one to a rules
    result (978 mates, 288 repetitions, 215 insufficient material, 100
    fifty-move draws, 19 stalemates), no fault and no time loss; the host
    at 5.7% before the start. Nodes a move 173,767 for the kernel and
    182,426 for Stockfish 16 HCE, **0.953**, inside 0.90 to 1.10; 89.8 ms
    a move against 66.2, 1.36.
  - *Score* (the runner's report; the wrapper's recount of `games.pgn`
    agrees): the kernel 389-622-589, 43.75%, pentanomial
    [74, 247, 311, 141, 27], **−43.7 ± 11.8 Elo** (nElo −63.8 ± 17.0).
  - *The gain* over the baseline's −102.0 ± 12.7: **+58.3 ± 17.3**
    [+41.0, +75.6], the two reads taken as independent. The price:
    **1.67 Elo for 1% of node rate** (58.3 / 35), 135 a doubling
    [95, 175].
  - *Beside it, not Elo:* depth without mate scores 17.88 for the kernel
    on the longer clock and 19.30 for Stockfish 16 HCE; reported rates
    1.934M and 2.755M, 1.42 as in RAR-M90.
  - *Frozen use.* The gain is at +45 or more: the packet's speed bands
    stand (at 1.67 a percent they read +7 to +20, 0 to +8 and 0 to +5,
    against +6 to +20, 0 to +8 and 0 to +5 as written), and the speed
    pass keeps its place ahead of E.4.0. The kernel's deficit to
    Stockfish 16 HCE at an equal node rate is recorded as −43.7 ± 11.8.
- **PREDICTION CALIBRATION (appended after exposure):** the gain +58.3
  against +55 [+35, +75], a hit, 3.3 above the centre; the match −43.7
  against −47 [−67, −27], a hit. At +45 or more (0.70): it happened. At
  +63 or more (0.30): not at the point, and the interval reaches it.
  Nodes a move 0.953 against 0.95 to 1.05: inside, at the lower edge,
  where the kernel's 4% under-report puts an equal rate. The hypothesis
  holds: 135 Elo a doubling against this opponent, 0.93 of the self-play
  slope. Competing hypothesis (1) put it at 0.81, and the interval covers
  both, so a discount against a foreign engine is neither shown nor
  excluded; (3), the unit of work, is not touched by a time-odds read.
- **Conditional lesson:** the kernel's self-play speed slope carries to
  the meter's engine at a 35% step; against the pre-NNUE goal a speed
  change is worth about 1.7 Elo a percent at `3+0.03`.
- **Retry trigger or `closed`:** closed. A speed pass that claims a gain
  against the meter reads the meter, not this.
- **Artifacts / commits:** this registration (`6f28b4d`); the zero-game reads and the
  dry run's console in `analysis/artifacts/e0-2026-11-23/` (ignored); the run
  in `tools/results/rar-m92-speed-price/` (`games.pgn` SHA-256 `AFB770F6…69B7`,
  `results.txt` `4B83A1C0…B13D`).
