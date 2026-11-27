# Oracle hybrids: a donor's search on Rarog's evaluation

Diagnostic instruments, never Rarog code and never a release. Each one runs a
donor engine's search unchanged and swaps only the raw evaluation it reads for
Rarog's hand-crafted evaluation, scaled into the donor's units. Played against
Rarog's own search on the same evaluation, they measure how much a search shape
is worth on this HCE (RAR-O08).

| Hybrid | Donor | Recipe | Bridge |
|---|---|---|---|
| Stockfish classical | `9587eeeb`, the `oracle/hybrid` tag | the tag's tree | `rarog_hce.dll` (`tools/oracle-hce-ffi`) |
| Stockfish dev | `def601cc` (2026-11-10) | `stockfish-def601cc-rarog-hce.patch` | `rarog_hce.dll` |

The bridge rebuilds Rarog's board from the donor's bitboards with
`Board::from_placement` and call `Evaluator::evaluate`. The network is still
loaded, so the `hcescale` command can compare it with the HCE, but the search
never evaluates it and makes none of its incremental updates.

## Scale

`hcescale <file>` prints the mean |raw network| and mean |Rarog HCE| over a
file of FENs. The constant (`Scale1024` in Stockfish's `evaluate.cpp`) is
their ratio in 1/1024, measured on
`tools/results/b0-20260913/scale_corpus.txt`, the corpus that gave B.0's
0.406. Stockfish's own wrapper, `scale_evaluation`, then applies as it
does to the network.

## Bridge price and time odds

A bridge costs time a native evaluation would not. Every bridge reads
`RAROG_HCE_BRIDGE_REPEAT` (default 1) and rebuilds the board that many times
per call without changing a score. `bridge_price.py` benches one fixed-depth
tree at repeat 1 and 3, refuses the read if the node counts differ, and
reports the bridge's share of the time per node and the time odds that hand
it back:

    python -I tools/oracle-hybrids/bridge_price.py <engine.exe> <out.json> 3 -- <bench command>

With `--argv` before `--` the bench is the engine's arguments rather than
stdin, for Rarog's driver, whose `quit` aborts a bench read from stdin
(`tools/donor-eval`'s bridge).

## Building

Stockfish dev: clone `def601cc`, `git apply` the patch, then in MSYS2's
MinGW64 shell `make -j profile-build ARCH=x86-64-bmi2 COMP=mingw` in `src/`
(the Makefile fetches the default network). `rarog_hce.dll` must sit beside
the executable, also while the profile bench runs.

## Tree statistics

`tree_stats.py` reads the rule rates a `probes` build prints:

    python tools/oracle-hybrids/tree_stats.py positions <match.pgn> <positions.jsonl> [count] [seed]
    python tools/oracle-hybrids/tree_stats.py run <engine.exe> <positions.jsonl> <out.jsonl> [KEY=VALUE ...]
    python tools/oracle-hybrids/tree_stats.py compare <base.jsonl> <candidate.jsonl> --floor <salted.jsonl> ...

`run` stops a search past `TREE_STATS_LIMIT_S` seconds (120) and marks it.
The calibration is `analysis/d73_parity_instrument_2026-11-16.md`.

## Reading the limits from a match

`pgn_limits.py` reads a fastchess PGN: games and terminations, each
engine's mean and median time a move and their ratios between the two
named engines,
and, in a node-budget match (where `tools/sprt.ps1` records each move's
reported nodes as `n=`), the largest count and the moves above a budget
plus a tolerance (per engine after a colon, else `--tolerance`):

    python -I tools/oracle-hybrids/pgn_limits.py <games.pgn> <A>[=<budget>[:<tol>]] <B>[=<budget>[:<tol>]] [--tolerance 1]

Licence: Stockfish is GPL-3.0. The patches are diagnostic diffs against
it and ship nowhere.
