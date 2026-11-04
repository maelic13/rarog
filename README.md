# Rarog

<p align="center">
  <img src="logo/rarog_detailed.png" alt="Rarog logo" width="260">
</p>

Rarog is a strong UCI chess engine written in Rust. It is meant to be used from
a chess GUI or an engine-testing tool.

---

## Highlights

- **Strong modern search** — principal variation search with aspiration
  windows, null-move pruning, ProbCut, singular extensions, late move
  reductions, history and correction tables, and a quiescence search.
- **Multi-threaded** — parallel search that scales across cores, enabled with
  the standard `Threads` option; `MultiPV` for analysis.
- **Tuned evaluation** — a tapered evaluation fitted to millions of positions,
  covering king safety, mobility, threats, pawn structure and passed pawns,
  material imbalance and endgame knowledge.
- **Syzygy tablebases** — optional endgame tablebase probing, both in-search and
  at the root.
- **Careful time management** — handles increments, `movestogo` and GUI latency,
  and spends longer on critical positions.
- **Pondering** — thinks on the opponent's clock when the GUI enables it.
- **Optimized binaries** — published for Windows, Linux and macOS, on both
  x86-64 and ARM64, every one of them profile-guided-optimized; one x64
  download runs the fastest build for any Intel or AMD processor.
- **Large pages on Windows** — a few percent more speed when Windows allows
  the engine to use them; see [Large pages](#large-pages-windows).
- **Built-in benchmark** — a `bench` command for reproducible speed and
  search comparisons.

---

## Download

- [Latest release](https://github.com/maelic13/rarog/releases/latest)
- [All releases](https://github.com/maelic13/rarog/releases)

Every release provides ready-to-run executables — no installation needed. Pick
the one for your computer:

| Your computer | Download |
| --- | --- |
| Windows, Intel or AMD processor | `rarog-{version}-windows-x64.exe` |
| Linux, Intel or AMD processor | `rarog-{version}-linux-x64` |
| Windows on ARM | `rarog-{version}-windows-arm64.exe` |
| Linux on ARM | `rarog-{version}-linux-arm64` |
| Mac with Apple silicon | `rarog-{version}-macos-arm64` |

**One download for every Intel or AMD processor.** It holds builds for
several processor generations and, when it starts, runs the fastest one your
processor supports, so there is nothing to choose. Every build plays exactly
the same moves; only the speed differs.

Upgrading from 2.6 or earlier? Those releases had separate `x86-64`, `avx2`
and `pext` downloads; from 2.7 the single `x64` download replaces all three.

Rarog is 64-bit only — there is no 32-bit build.

---

## Use With A GUI

1. Download the executable for your system, or build one from source.
2. Add it as a UCI engine in your chess GUI.
3. Set `Hash` to a comfortable amount of memory and `Threads` to the number of
   cores you want to use.
4. Start a game or an analysis session.

Rarog is tested with Arena, ChessBase/Fritz, ChessOK Aquarium and Hiarcs Chess
Explorer. Any UCI-compatible GUI should work.

### Large pages (Windows)

On Windows, Rarog can keep its hash table and other large tables in 2 MB
memory pages instead of the usual 4 KB ones. The processor then spends less
time looking up memory addresses, and the engine searches a few percent more
positions per second. Nothing else changes: moves and analysis are exactly
the same.

Windows allows this only for accounts that hold the *Lock pages in memory*
right. To grant it:

1. Run `secpol.msc` (Local Security Policy; it needs administrator rights and
   is not included in Windows Home editions).
2. Open *Local Policies → User Rights Assignment → Lock pages in memory*.
3. Add your user account and confirm.
4. Sign out and sign in again; the right takes effect at the next sign-in.

Without the right, Rarog runs normally with ordinary pages. To check, run
`bench`: its first line, `Large pages`, reads `on`, `off` (the right is not
held), `partial` (Windows could not find enough contiguous memory for every
table; restarting the engine or the computer often helps) or `disabled`
(the `LargePages` option is off). Setting `LargePages` to `false` turns large
pages off without removing the right.

---

## UCI Options

| Option | Default | Description |
| --- | --- | --- |
| `Hash` | `64` | Transposition table size in MB. More memory helps longer searches. |
| `Clear Hash` | — | Empties the transposition table. |
| `Threads` | `1` | Search threads. Set to the number of cores you want to use. |
| `MultiPV` | `1` | Best lines reported per depth, `1` to `256`. For analysis; leave it at `1` for play. |
| `Ponder` | `false` | Think while the opponent moves. Enabled by the GUI. |
| `Move Overhead` | `10` | Milliseconds reserved for GUI and network delay. Raise it if you lose on time. |
| `SyzygyPath` | empty | Folders holding Syzygy tablebases. Empty disables probing. |
| `SyzygyProbeDepth` | `1` | Minimum depth at which tablebases are probed. |
| `SyzygyProbeLimit` | `7` | Largest tablebase to probe. `0` disables probing. |
| `Syzygy50MoveRule` | `true` | Whether tablebase results respect the fifty-move rule. |
| `LargePages` | `true` | Windows only. Use large pages when Windows allows them; see [Large pages](#large-pages-windows). |

`SyzygyPath` accepts several folders separated by `;` on Windows or `:`
elsewhere, and the value `<empty>` means no path at all, which is what a GUI
sends back when it echoes the advertised default. Positions resolved from
tablebases are reported through `tbhits`.

### Search output

Every `info` line carries `depth`, `seldepth`, `multipv`, `score`, `nodes`,
`nps`, `hashfull`, `tbhits`, `time` and `pv`, in that order.

- `seldepth` is the deepest ply the current iteration reached, counting the
  root as ply one, and it starts again at each depth.
- A score still being narrowed by an aspiration re-search is marked
  `lowerbound` or `upperbound`; the line that closes an iteration is exact.
- With `Threads` above one the engine picks its move by a vote across threads
  and reports that thread's line before `bestmove`, so the last line always
  describes the move played.
- A position with no legal move reports `info depth 0 score mate 0` when in
  check and `info depth 0 score cp 0` when stalemated, then `bestmove 0000`.
- With tablebases, a tablebase win or loss is reported as `score cp 20000` or
  `score cp -20000`, one less for each ply from the root, and when time allows
  the line is continued through the tables.

### Supported commands

`uci`, `isready`, `setoption`, `ucinewgame`, `position`, `go`, `stop`,
`ponderhit` and `quit`, plus `bench` and `help` for use by hand.

`go` supports `depth`, `nodes`, `movetime`, `wtime`, `btime`, `winc`, `binc`,
`movestogo`, `mate`, `searchmoves`, `ponder`, `perft` and `infinite`.

---

## Build From Source

You do not need to build Rarog to play: the downloads are already optimized.
Build it to develop or test the engine, or for a system without a download.

1. Install Rust with [rustup](https://rustup.rs). The repository pins the
   compiler version, and rustup fetches it on the first build.
2. Add the LLVM tools, which the build needs:

   ```bash
   rustup component add llvm-tools
   ```

### The fastest engine for your computer

```bash
cargo xtask build
```

This checks the processor of the computer that builds it, builds the one of
the download's four builds that runs fastest there (`pext` on Intel Haswell
and newer and AMD Zen 3 and newer), and optimizes it with a profile of the
`bench` suite (PGO). It is the same code the download runs on your processor,
in a single smaller file, and it may not start on an older processor. It
takes a while, because the engine is built twice: once to measure how it runs,
then again using what it measured.

### Other builds

One file for any Intel or AMD computer, as released (see below):

```bash
cargo xtask build --universal
```

Without the profile step, quickly:

```bash
cargo xtask build --nopgo
```

Tuned for the exact processor it is built on. It can be faster or not, so
compare with `bench`, and it may not run on any other computer:

```bash
cargo xtask build --native
```

A profile-guided build has to run the engine, so building for another
processor or system needs `--nopgo`. `cargo xtask help` lists every command
and option.

### Where the binaries go

Every build is copied to `target/dist`, named
`rarog-<version>-<system>-<processor>[-<build>][-<kind>]`. The version is the
one the engine reports: `2.7.0` for a release, `dev-<date>-<commit>` for
anything else. An Intel/AMD build names the build it holds (`base`, `sse42`,
`avx2`, `pext`, or `native`); the universal binary, which holds them all, and
ARM, which has one, name none. A profile-guided build names no kind; anything
else says what it is. For example, on a Ryzen 9 5950X:

| Build | In `target/dist` |
| --- | --- |
| `cargo xtask build` | `rarog-dev-20261104-def5fd66-windows-x64-pext.exe` |
| `cargo xtask build --nopgo` | `rarog-dev-20261104-def5fd66-windows-x64-pext-nopgo.exe` |
| `cargo xtask build --universal` | `rarog-dev-20261104-def5fd66-windows-x64.exe` |
| The 2.7.0 download | `rarog-2.7.0-windows-x64.exe` |

### The universal binary

The universal build is the release's Intel/AMD download: four profile-guided
builds of the engine in one file, and a small start-up check that runs the
fastest one the processor supports.

| Build | Chosen on |
| --- | --- |
| `pext` | Intel Haswell (2013) and newer, AMD Zen 3 (2020) and newer. The fastest. |
| `avx2` | AMD Zen 1 and Zen 2, and other processors with AVX2 whose PEXT instruction is slow |
| `sse42` | Intel Nehalem to Ivy Bridge (2008–2012), AMD Bulldozer to Steamroller |
| `base` | Any other 64-bit Intel or AMD processor. The slowest. |

The first line Rarog prints names the build it chose. To run one build on
purpose, for testing, set the environment variable `RAROG_ISA` to `base`,
`sse42`, `avx2` or `pext`; a build the processor cannot run is refused.

### Tests

In both profiles:

```bash
cargo test -p rarog
cargo test -p rarog --release
cargo test -p xtask
```

The Texel tuner is its own Cargo workspace:

```bash
cargo test --manifest-path tools/texel-tuner/Cargo.toml
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
Special thanks to Stockfish and Reckless and their developers for the
inspiration their work provides to chess engine authors and testers.
