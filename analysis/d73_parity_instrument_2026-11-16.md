# D.7.3: the parity instrument, built and calibrated on the donor (2026-11-16)

The instrument the new kernel (D.7.4) is checked against before any game,
designed in `analysis/search_shape_inventory_2026-11-13.md` section 7. This
step builds the twin, the trace format, the statistics tool and the noise
floor, and proves the instrument on the donor alone. The rule tests on
Rarog's side land with D.7.4. Nothing here is Rarog code or plays a game.

## The twin

- **What it is.** The Reckless hybrid (`0756957` with
  `tools/oracle-hybrids/reckless-0756957-rarog-hce.patch`, Rarog's evaluation
  at `1f29e04`) with `tools/oracle-hybrids/reckless-0756957-rarog-hce-trace.patch`
  on top.
  - The patch applies cleanly to a fresh hybrid tree (`git apply --check`).
  - Built `cargo build --release --no-default-features`; PGO is not needed, since at a fixed depth or node count the tree does not depend on code generation.
- **Records.** With `RAROG_TRACE=<file>` the twin writes, at every n-th node entry (`RAROG_TRACE_STRIDE`, default 64), one line per decision that node takes: `<kind> fire=<0|1> <name>=<int> ...`, its inputs and its outputs as named integers.
  - 36 kinds: section 7's 35 and `q_lmp` (quiescence's move-count stop), a rule the list folded into the quiescence kinds.
  - The wrapper, the picker's scores and the history gravity run outside a node's own frame, so they sample every n-th call of their own.
  - The root window and optimism are written whole.
- **Rates.** Every rule's firings and attempts are counted at every node and printed as `info string rates <kind>=<fires>/<attempts> ...` before `bestmove`.
- **The noise floor and the plants.**
  - `RAROG_SPREAD_SALT=<k>` adds k to the node count inside the two LMR spread terms and the draw score, and nowhere else.
  - `RAROG_PLANT` selects one planted difference: `unreduced` (the full-depth search's reductions removed), `lmr_check` (no LMR at a node in check) or `q5` (quiescence stops at the fifth move, not the third).
- **Faithful and live, by `bench`'s node count.**

  | Configuration | `bench` nodes |
  |---|---|
  | nothing set | **4,158,758**, the hybrid's count (RAR-O09) |
  | tracing on | **4,158,758** |
  | salt 1 | 3,715,269 |
  | `unreduced` | 4,980,281 |
  | `lmr_check` | 4,031,674 |
  | `q5` | 3,898,590 |

  The salt moves `bench` by 10.7%, phase alone, which is why the floor is measured.

## Layer B: tree statistics

- **Tool.** `tools/oracle-hybrids/tree_stats.py` has three modes:
  - `positions` draws the positions;
  - `run` gives each position a fresh process at `go depth 14`, Hash 64, one thread, recording positions visited at depths 8, 11 and 14 and the rates;
  - `compare` applies section 7's frozen read.

  A wall-clock limit of two minutes a search sends `stop` to a runaway search and marks it.
- **Positions.** 500 positions, drawn by `tree_stats.py positions` with seed 20261116 from RAR-O08's match (5) (`tools/results/sprt_rarog-head_vs_reckless-hce_20261113_024923.pgn`, 3,000 games): the position after every 20th ply, 19,137 after deduplication. SHA-256 30ea7b93c95d7387a5d10383b860c3be97e091e5b098ac1e2ccd9a146711bd9c.
- **The unit.** Positions visited are each engine's own count. Since D.6.31 Rarog counts as the donor does, so section 7's "the twin's nodes plus its null moves" is the twin's nodes.
- **Runaway searches.** Four positions did not finish depth 14 within the limit under some configurations, though the base finished them:
  - position 268 under all four salts;
  - 206 under salts 1 and 4 and `unreduced`;
  - 446 under salts 1 and 2, `unreduced` and `lmr_check`;
  - 310 under salt 4.

  The union is left out of every comparison: 496 positions.
- **The floor.** At each depth, the largest |mean log ratio| of a salted run against the base: depth 8 **0.0210**, 11 **0.0190**, 14 **0.0423**, with a standard error of about 0.02 at 496 positions.
- **The read.**

  | Comparison | Depth 8 | Depth 11 | Depth 14 | Rates | Verdict |
  |---|---|---|---|---|---|
  | Control: salt 4, floor from salts 1–3 | −0.0045 ± 0.0176 | +0.0093 ± 0.0200 | +0.0227 ± 0.0224 | 36 of 36 | **pass**, as two equal kernels must |
  | Plant `unreduced` | +0.1961 ± 0.0248 | +0.1754 ± 0.0223 | +0.2173 ± 0.0212 | 24 of 36 | **fail**, as it must |
  | Plant `q5` | −0.0119 ± 0.0160 | +0.0210 ± 0.0175 | +0.0373 ± 0.0183 | 34 of 36 (`q_lmp`, `q_see` fail) | **fail**, as it must |
  | Plant `lmr_check` | −0.0392 ± 0.0229 | +0.0454 ± 0.0210 | +0.0836 ± 0.0220 | 36 of 36 | **passes** |

- **The instrument's stated limit.** By section 7's rule, "a plant that passes sets the instrument's stated limit".
  - A change the size of LMR switched off at in-check nodes (about +8% positions visited at depth 14) is below layer B's resolution, under the frozen 0.03 allowance and the 10% rate band.
  - Layer A catches it: every LMR record at an in-check node mismatches the donor's rule exactly.
  - The allowance and the bands are not changed after this read.

## Layer A: decision records

- **Tool.** `tools/oracle-hybrids/capture_trace.py`: one process a position, one trace file a position, so parallel processes never share a file; `go depth 14`, the base configuration.
- **Capture.** 250 of the positions (the four runaways left out), stride 512, no timeout.
  - The rarest kinds: `nmp_r` and `nmp_result` 3,418 records each, from 210 positions; `singular_result` 3,609; `research` 3,538; `optimism` 3,500; `probcut_depth` 4,655.
  - Every kind has at least 3,418 records, against the floor of 2,000, from at least 210 positions, against 200.
- **Storage.** The trace is the donor's behaviour as numbers and stays in ignored storage (maintainer decision 2026-11-13): `analysis/artifacts/d73-parity-2026-11-16/trace-a/`, 323 MB.
  - SHA-256 of the sorted per-file hashes: 73603995b71c94d7d7321900483dcb5042b99d2cffc512737279ebceb8337976.
  - The per-kind counts: `summary.json`, SHA-256 9bcf11a515747f8504d08a1efc724a3bdee844f160a74a91753717e988efa4fb.
- **D.7.4's use.** Each rule of the new kernel is a pure function of an input struct. An ignored test reads the trace named by `RAROG_DONOR_TRACE` and calls the function for each record. It passes with no mismatch in the kind's records, and a planted constant must fail its own kind and no other. The record format is the contract: the field names above are the rule's inputs and outputs.

## Independence

- The twin is a diagnostic diff against the donor and ships nowhere; Reckless is AGPL-3.0, as the hybrid patches' README states.
- The instrument compares decisions on the same inputs, not code.
- D.7.4 writes each rule from the inventory's description and Rarog's own structure, under D.7's boundary (PLAN D.7, *Independence*).

## Artifacts

`analysis/artifacts/d73-parity-2026-11-16/` (ignored):
- the twin's tree and binary (SHA-256 45c8b24c60466e149fa2e4d477d531c22873114546701b53738d39c542e4b0d9);
- `trace_patch.py`, which wrote the patch;
- the eight layer-B runs (`b_*.jsonl`) and their filtered copies (`f_*.jsonl`; the base 7e204133ed281e841e663b9dd1798bdc4168d8915ff0fd0aed9335c1d5114089);
- the comparisons (`cmp_*.json`) and the trace.

The patch's SHA-256 is 07da266fb5f1a380cf8c51984b15ba14c22080a8b63d25af9c4ef1bf71f2a910.

## Appended 2026-11-18: the extended trace (D.7.4)

The record above is unchanged. D.7.4 extended the twin and captured the trace
again (maintainer decision 6, 2026-11-17); the full read is in
`analysis/d74_compact_kernel_2026-11-18.md`.

- **Patch:** `tools/oracle-hybrids/reckless-0756957-rarog-hce-trace.patch`,
  SHA-256 `ce0217e721ba89c1e214dfc3ff11a7ce1363893e524a64d3efe2476592d973ec`
  (`e84f4f9`, `635177b`); the twin's binary
  `4fe68f824aed141be2bc3bfd52b113832b6d96523fc3424165997152de5cd577`. Its
  bench is 4,158,758 with tracing off and on, and 20 stored base rows
  reproduce exactly, so layer B's stored twin runs still describe it.
- **Added inputs:** the stored capture refutation on `nmp_gate`, whether the
  exclusion search reached a move on `singular_result`, the node type on
  `q_stand_pat`, the null search's outcome on `nmp_result`.
- **Eleven new kinds,** records (positions): `draw_score` 154,535 (250),
  `eval_bonus_gate` 92,584 (246), `malus_fade` 6,118 (239), `null_bound`
  3,418 (210), `pick_threshold` 48,371 (247), `probcut_value` 4,078 (232),
  `q_cutoff_bonus` 26,389 (241), `score_held` 3,500 (250), `spread` 121,252
  (247), `tt_cut_bonus` 11,108 (241), `tt_cut_return` 11,108 (241).
- **The capture:** `analysis/artifacts/d74-parity-2026-11-17/trace-a/`, the
  same recipe (250 positions, stride 512, the four exclusions), no timeout;
  the 36 old kinds' counts equal the ones above; 47 kinds, 3,269,262
  records, 354 MB. SHA-256 of the sorted per-file hashes
  `1480a5ec363726ee5a00fa1b04f969d87e77cc56d41e5c0e12e3b594affafb14` (this
  method reproduces 73603995… on the old trace); `summary.json`
  `63a47e89ae754ade136799313554b84039a167e4da929dace00a64b585d04e04`.
