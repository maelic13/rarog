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
| Reckless | `0756957` (2026-11-07) | `reckless-0756957-rarog-hce.patch` | links the `rarog` crate |

Both bridges rebuild Rarog's board from the donor's bitboards with
`Board::from_placement` and call `Evaluator::evaluate`. The network is still
loaded, so the `hcescale` command can compare it with the HCE, but the search
never evaluates it and makes none of its incremental updates.

## Scale

`hcescale <file>` prints the mean |raw network| and mean |Rarog HCE| over a
file of FENs. The constant (`SCALE_1024` in Reckless's `src/hce.rs`,
`Scale1024` in Stockfish's `evaluate.cpp`) is their ratio in 1/1024, measured on
`tools/results/b0-20260913/scale_corpus.txt`, the corpus that gave B.0's
0.406. The donor's own wrappers (Reckless's `correct_eval`, Stockfish's
`scale_evaluation`) then apply as they do to the network.

## Bridge price and time odds

A bridge costs time a native evaluation would not. Every bridge reads
`RAROG_HCE_BRIDGE_REPEAT` (default 1) and rebuilds the board that many times
per call without changing a score. `bridge_price.py` benches one fixed-depth
tree at repeat 1 and 3, refuses the read if the node counts differ, and
reports the bridge's share of the time per node and the time odds that hand
it back:

    python -I tools/oracle-hybrids/bridge_price.py <engine.exe> <out.json> 3 -- <bench command>

## Building

Reckless: clone `0756957`, copy its `networks/` file, `git apply` the patch,
point the `rarog` path dependency in `Cargo.toml` at a checkout of the Rarog
revision being measured, then with `RUSTFLAGS="-C target-cpu=native"`:
`cargo pgo build -- --no-default-features`, run the instrumented binary's
`bench`, `cargo pgo optimize build -- --no-default-features`. Syzygy is left
out because Fathom's C needs the MSVC headers and the reads play without
tablebases. The patch links `D:/code/rarog-oracle`: recreate it as a detached
worktree of the Rarog revision being measured
(`git worktree add --detach D:/code/rarog-oracle <rev>`).

Reckless's counting twin (RAR-O09): the same tree with
`reckless-0756957-rarog-hce-count.patch` applied on top, built the same
way. It counts the null moves it makes and prints
`info string nullmoves <N> nodes <M>` before `bestmove`; it is never
played, and its `bench` must report the node count of the plain hybrid.

## Reading the limits from a match

`pgn_limits.py` reads a fastchess PGN: games and terminations, each
engine's mean time a move and the ratio between the two named engines,
and, in a node-budget match (where `tools/sprt.ps1` records each move's
reported nodes as `n=`), the largest count and the moves above a budget
plus a tolerance:

    python -I tools/oracle-hybrids/pgn_limits.py <games.pgn> <A>[=<budget>] <B>[=<budget>] [--tolerance 1]

Stockfish dev: clone `def601cc`, `git apply` the patch, then in MSYS2's
MinGW64 shell `make -j profile-build ARCH=x86-64-bmi2 COMP=mingw` in `src/`
(the Makefile fetches the default network). `rarog_hce.dll` must sit beside
the executable, also while the profile bench runs.

Licences: Stockfish is GPL-3.0, Reckless AGPL-3.0. The patches are diagnostic
diffs against them and ship nowhere.
