# Rarog

<p align="center">
  <img src="logo/rarog_detailed.png" alt="Rarog logo" width="260">
</p>

Rarog is a strong UCI chess engine written in Rust. It is meant to be used from
a chess GUI or an engine-testing tool.

---

## Highlights

- **Strong modern search** — iterative deepening with principal variation
  search, aspiration windows, null-move pruning, ProbCut, singular extensions,
  late move reductions and a capture-focused quiescence search.
- **Multi-threaded** — parallel search that scales across cores, enabled with
  the standard `Threads` option.
- **Tuned evaluation** — a tapered evaluation fitted to millions of positions,
  covering king safety, mobility, threats, pawn structure and passed pawns,
  material imbalance and endgame knowledge.
- **Syzygy tablebases** — optional endgame tablebase probing, both in-search and
  at the root.
- **Careful time management** — handles increments, `movestogo` and GUI latency,
  and spends longer on critical positions.
- **Pondering** — thinks on the opponent's clock when the GUI enables it.
- **Optimized binaries** — published for Windows, Linux and macOS, on both
  x86-64 and ARM64, and every one of them profile-guided-optimized.
- **Built-in benchmark** — a `bench` command for reproducible speed and
  search comparisons.

---

## Download

- [Latest release](https://github.com/maelic13/rarog/releases/latest)
- [All releases](https://github.com/maelic13/rarog/releases)

Every release provides ready-to-run executables — no installation needed. Pick
the one matching your operating system and CPU:

| Asset suffix | CPU required | Use when |
| --- | --- | --- |
| `pext` | AVX2, BMI2, FMA (Intel 2013+, AMD 2015+) | Modern Intel, or AMD Zen 3 and newer. The fastest build. |
| `avx2` | AVX2, BMI2, FMA (Intel 2013+, AMD 2015+) | Same CPUs as `pext`, but the right choice on AMD Zen 1, Zen 2 and Excavator, where the PEXT instruction runs in microcode and is slow. |
| `x86-64` | SSE3 (Intel 2004+, AMD 2005+) | Anything older, or when the two above do not start. |
| `arm64` | ARM64 baseline | ARM64 Linux, Windows on ARM, and Apple Silicon Macs. |

### How much the choice is worth

Measured on an idle Ryzen 9 5950X, pooling four independent PGO builds per
asset so that per-build optimisation luck averages out:

| Comparison | Result |
| --- | --- |
| `pext` against `avx2` | `pext` is **2.4%** faster |
| `pext` against `x86-64` | `pext` is **6.7%** faster |

---

## Use With A GUI

1. Download the executable for your system, or build one from source.
2. Add it as a UCI engine in your chess GUI.
3. Set `Hash` to a comfortable amount of memory and `Threads` to the number of
   cores you want to use.
4. Start a game or an analysis session.

Rarog is tested with Arena, ChessBase/Fritz, ChessOK Aquarium and Hiarcs Chess
Explorer. Any UCI-compatible GUI should work.

---

## UCI Options

| Option | Default | Description |
| --- | --- | --- |
| `Hash` | `64` | Transposition table size in MB. More memory helps longer searches. |
| `Clear Hash` | — | Empties the transposition table. |
| `Threads` | `1` | Search threads. Set to the number of cores you want to use. |
| `Ponder` | `false` | Think while the opponent moves. Enabled by the GUI. |
| `Move Overhead` | `10` | Milliseconds reserved for GUI and network delay. Raise it if you lose on time. |
| `SyzygyPath` | empty | Folders holding Syzygy tablebases. Empty disables probing. |
| `SyzygyProbeDepth` | `1` | Minimum depth at which tablebases are probed. |
| `SyzygyProbeLimit` | `7` | Largest tablebase to probe. `0` disables probing. |
| `Syzygy50MoveRule` | `true` | Whether tablebase results respect the fifty-move rule. |

`SyzygyPath` accepts several folders separated by `;` on Windows or `:`
elsewhere. Positions resolved from tablebases are reported through `tbhits`.

### Supported commands

`uci`, `isready`, `ucinewgame`, `position`, `go`, `stop`, `ponderhit`, `quit`
and `bench`.

`go` supports `depth`, `nodes`, `movetime`, `wtime`, `btime`, `winc`, `binc`,
`movestogo`, `mate`, `searchmoves`, `ponder`, `perft` and `infinite`.

---

## Build From Source

Install Rust, then:

```bash
cargo build --release
```

This produces `target/release/rarog` (`rarog.exe` on Windows). It is portable
but not tuned for any particular CPU.

For a build equivalent to the published ones, use the `xtask` helper, which
writes finished binaries to `target/dist`:

```bash
cargo xtask build --arch pext --pgo
```

### Options

| Option | Meaning |
| --- | --- |
| `--arch x86-64` | Portable x86-64; requires SSE3. |
| `--arch avx2` | Requires AVX2, BMI2 and FMA. |
| `--arch pext` | Same requirements as `avx2`, plus it uses the PEXT instruction. |
| `--arch arm64` | ARM64 targets. |
| `--native` | Additionally tunes for the CPU you are building on. |
| `--pgo` | Profile-guided optimization. Used for all published builds. |

`--arch` and `--native` are independent, so they can be combined freely. A
`--native` build is faster on the machine that produced it but may not run
anywhere else, so it is marked `-native` in the filename and never published.

`--pgo` builds an instrumented engine, trains it with `bench`, then rebuilds
using the collected profile. It cannot be cross-compiled, because training has
to run the engine. On Windows ARM64, `xtask` automatically selects the linker
needed for a valid profile; no separate LLVM installation is required.

### Tests

```bash
cargo test --workspace --all-targets
```

---

## Bench

`bench` searches a fixed set of positions and reports a node count and speed.
The node count is identical on every platform, which makes it a quick way to
confirm a build is correct; the speed tells you how fast the machine is.

```text
bench
bench 13
```

It honours the current options, so a threaded benchmark is:

```text
setoption name Threads value 8
bench
```

---

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).

---

## Acknowledgements

Rarog is an independent engine, but it benefits from the open chess-engine
community's published ideas, testing practices, and protocol conventions.
Special thanks to Stockfish and its team for the inspiration their work provides
to chess engine authors and testers.
