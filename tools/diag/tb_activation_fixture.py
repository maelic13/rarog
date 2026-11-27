#!/usr/bin/env python3
"""Record the clock's optimum from the engine itself, for tb_activation.py's test.

A `diag` build prints `info string tm ... optimum <ms>` after each iteration
of a clock search: `compute_runtime_limits`' optimum, `budget`'s soft bound
under the hard bound and Rarog's reserve (`src/search/time.rs`). This drives
such a build over a grid of sudden-death clocks with increment and writes
what it printed, with the source extracts the values depend on, to
`tb_activation_budget_v1.json`. Regenerate it when the test reports that
`budget`, its constants or the reserve changed:

  cargo build --release --features diag --bin rarog   (CARGO_TARGET_DIR apart)
  python tools/diag/tb_activation_fixture.py --engine <diag rarog.exe>
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import tb_activation  # noqa: E402

OUT = HERE / "tb_activation_budget_v1.json"
OPTIMUM = re.compile(r"^info string tm depth \d+ .* optimum ([0-9.]+) ", re.M)
WHITE = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 {fullmove}"
BLACK = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 {fullmove}"

# (clock ms, increment ms, Move Overhead ms, game ply, threads): the read's
# `3+0.03` from the first move to a long ending, a scramble where the reserve
# binds, CCRL's Blitz control, a larger overhead, and four threads.
CASES = [
    (3000, 30, 10, 0, 1), (3000, 30, 10, 1, 1), (3000, 30, 10, 40, 1),
    (3000, 30, 10, 120, 1), (3000, 30, 10, 201, 1), (2000, 30, 10, 80, 1),
    (800, 30, 10, 150, 1), (200, 30, 10, 160, 1), (60, 30, 10, 170, 1),
    (25, 30, 10, 171, 1), (3000, 0, 10, 60, 1), (120000, 1000, 10, 40, 1),
    (3000, 30, 50, 30, 1), (3000, 30, 10, 40, 4), (100, 30, 10, 90, 4),
    # The reserve binds: `2 * overhead` at a large overhead, and the 30 ms
    # more at four threads.
    (200, 300, 50, 126, 1), (100, 300, 10, 126, 4),
]


def engine_optimum(engine: Path, time_ms: int, inc_ms: int, overhead_ms: int,
                   game_ply: int, threads: int) -> float:
    fen = (BLACK if game_ply % 2 else WHITE).format(fullmove=game_ply // 2 + 1)
    commands = (f"uci\nsetoption name Threads value {threads}\n"
                f"setoption name Move Overhead value {overhead_ms}\nisready\n"
                f"position fen {fen}\n"
                f"go wtime {time_ms} btime {time_ms} winc {inc_ms} binc {inc_ms} depth 2\n")
    proc = subprocess.Popen([str(engine)], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
    proc.stdin.write(commands)
    proc.stdin.flush()
    lines = []
    for line in proc.stdout:
        lines.append(line)
        if line.startswith("bestmove"):
            break
    proc.stdin.write("quit\n")
    proc.stdin.flush()
    proc.wait(timeout=30)
    found = set(OPTIMUM.findall("".join(lines)))
    if len(found) != 1:
        raise SystemExit(f"{engine}: expected one optimum for {time_ms}/{inc_ms} ply {game_ply}, got {found}")
    return float(found.pop())


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n", 1)[0])
    parser.add_argument("--engine", required=True, type=Path)
    parser.add_argument("--out", type=Path, default=OUT)
    args = parser.parse_args()
    name = subprocess.run([str(args.engine)], input="uci\nquit\n", capture_output=True, text=True,
                          timeout=30).stdout
    ident = re.search(r"^id name (.+)$", name, re.M)
    if not ident or "diag" not in name.splitlines()[0]:
        raise SystemExit(f"{args.engine}: not a diag build (its banner must name `diag`)")
    cases = [{"time_ms": t, "inc_ms": i, "overhead_ms": o, "game_ply": p, "threads": n,
              "optimum_ms": engine_optimum(args.engine, t, i, o, p, n)} for t, i, o, p, n in CASES]
    record = {
        "what": "compute_runtime_limits' optimum as a diag build prints it, to 0.1 ms",
        "engine": ident.group(1).strip(),
        "recipe": "cargo build --release --features diag --bin rarog; "
                  "python tools/diag/tb_activation_fixture.py --engine <exe>",
        "source": tb_activation.clock_source_extracts(),
        "constants": tb_activation.clock_constants(),
        "cases": cases,
    }
    args.out.write_bytes(json.dumps(record, indent=1).replace("\n", "\r\n").encode("utf-8") + b"\r\n")
    for case in cases:
        print(case)
    print(f"wrote {args.out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
