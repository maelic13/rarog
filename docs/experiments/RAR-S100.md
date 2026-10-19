# RAR-S100 — C.10.2: `QsCountLimit` read as a switch, two 2,000-game reads

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-19, before any game. The run is the maintainer's command (about 50 minutes).
- **Kind:** two categorical reads that fix one value for C.10's tune (PROCESS, *SPSA go/no-go*, step 3; B.6 carried `QsCountLimit` to C.10 as better read as a switch). They accept nothing on their own.
- **Why:** quiescence skips captures past this count at a node, unless the capture gives check, recaptures or promotes. On the C head it skips 39% more captures per interior node than on the B.9 head (`analysis/c10_research_2026-10-19.md`). Its consumers, the other `Qs*` coordinates, are on C.10's surface, and a discrete knob is never pinned under a tune of its consumers without a read.
- **Binary:** one tune build on both sides, `tools/test_engines/rarog-c10-tune.exe` (SHA-256 `1E5C9C32…`, the C head `ffbe3d1`, `pext-tune`, bench **10,521,141**), `-CategoricalTuneBuild`.
- **Wire** (`analysis/artifacts/c10/qscount_wire.txt`, `bench_opt.py`): the option is advertised as `default 3 min 2 max 8`, and setting 3 reproduces 10,521,141. `bench 13` moves at each read value: 2 gives 17,364,968 (1.65×) and 4 gives 12,256,555 (1.16×). An out-of-range 99 clamps to 8 (11,689,833). This is a wire check, not a forecast.
- **Reads (frozen):** Colosseum `match`, `match-fixed`, 2,000 games each, `3+0.03`, Hash 64, one thread, the UHO book in random order, no adjudication.
  - **Read 1:** `QsCountLimit=2` against the default 3, seed **20261019**, `tools/results/c10-qscount2`.
  - **Read 2:** `QsCountLimit=4` against the default 3, seed **20261020**, `tools/results/c10-qscount4`.
  - Both run from one wrapper, `analysis/artifacts/c10/reads.ps1`, which waits for an idle host before each read and shows the harness's output live. It was executed once with `-DryRun`: both reads resolved with the option on side A only, 2,000 games and the seeds above.
- **Rule (frozen, RAR-S80's):** a read of **+5 Elo or more with its 95% lower bound above −5** is a direction. A direction fixes that value for C.10's pilot and blocks on both sides of every tune game, and the bake carries it with the theta into C.10.5's SPRT. If both reads qualify, the larger read wins. With no direction, 3 stays fixed and no third value is read.
- **PRE-REGISTERED PREDICTION (freeze before exposure):**
  - 2 reads −3 ± 10 Elo; a direction 0.10; −5 or worse 0.35.
  - 4 reads 0 ± 10 Elo; a direction 0.15; −5 or worse 0.25.
  - Neither finds a direction (0.75).
  - Low confidence: the only local evidence is node counts and a firing rate, which have pointed the wrong way before on this search (RAR-S86, RAR-S89).
- **Cheapest prior falsifier:** these reads.
- **Result (played 2026-10-19 by the maintainer, `reads.ps1`; manifests `colosseum_match_c10-qscount2_20261019_124720.manifest.txt` and `colosseum_match_c10-qscount4_20261019_130821.manifest.txt`; no faults, no time losses, no abnormal games):**
  - **Read 1, 2 against 3:** +3.1 ± 8.7 Elo (95% −5.6 to +11.9), +5.4 ± 15.2 nElo; W/D/L 492/1,034/474; pentanomial [11, 257, 466, 235, 31].
  - **Read 2, 4 against 3:** +1.9 ± 9.2 Elo (95% −7.3 to +11.1), +3.2 ± 15.2 nElo; W/D/L 522/967/511; pentanomial [21, 264, 429, 255, 31].
  - Recount from `games.jsonl` by pair (`analysis/artifacts/c10/recount_s100.py`, `recount_s100.txt`): both pentanomials and both intervals agree with the harness exactly; every side-A option was the read's value and side B carried none (`resolved-config.json`).
- **Disposition:** observation. **Neither read is a direction** (both under +5 Elo), so by the frozen rule `QsCountLimit` stays at **3**, fixed on both sides of C.10's pilot and blocks, and no third value is read. C.10.2 closes `NO_CHANGE`. The two reads cannot tell 2, 3 and 4 apart at this budget; all three sit inside about ±10 Elo of each other.
- **PREDICTION CALIBRATION (appended after exposure):** neither a direction held (0.75). Read 1 missed in sign at the point estimate (predicted −3, read +3.1), inside both intervals; read 2 (predicted 0, read +1.9) held. The tree's response (1.65× the nodes at 2) did not show up as a cost in games, which is what the low confidence anticipated: node counts are not Elo.
