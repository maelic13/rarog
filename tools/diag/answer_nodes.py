#!/usr/bin/env python3
"""Compare engine answers at a FIXED NODE BUDGET, not a fixed depth.

Why this exists. `answer_compare.py` drives both engines to the same DEPTH,
which factors out the axis Rarog actually loses on: what it COSTS to reach
that depth. Rarog runs ~1.6x the oracle's quiescence per node and builds a
larger tree throughout, so at equal time the oracle simply gets more depth.
RAR-S70 measured the consequence -- 12 points of fixed-depth agreement bought
2.33 Elo, about 0.2 Elo per point -- so fixed-depth agreement ranks move
choice at equal depth and does NOT rank strength.

A fixed node budget prices both halves at once: an engine that wastes nodes
reaches a lower depth and answers worse, and both show up here.

Fixed TIME would be the wrong instrument for this pair. The oracle is
Stockfish's search calling Rarog's eval through FFI, so its NPS is depressed
by the FFI boundary; a fixed-time comparison would largely measure marshalling
overhead, and it would be machine-dependent and non-reproducible on top.

Node accounting is comparable by construction: Rarog calls `record_node()`
from `check_stop`, which runs at the top of BOTH `negamax` and `quiescence`,
so its reported `nodes` includes quiescence -- the same convention Stockfish
uses. That was verified, not assumed; per-node vs per-move denominators have
produced three false findings in this project.

Usage:
  python tools/diag/answer_nodes.py --oracle hybrid/stockfish/src/stockfish.exe \
      --nodes 300000 \
      --engine base=tools/test_engines/rarog-46base-pext-pgo.exe \
      --engine root=tools/test_engines/rarog-46root-pext-pgo.exe
"""

import argparse
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
SUITE = ROOT / "tools" / "diag" / "phase4_suite_v1.epd"
INFO = re.compile(r"^info depth (\d+).*?score (cp|mate) (-?\d+).*? pv (.+)$")


def load_suite():
    rows = []
    for line in SUITE.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        fen, _, rest = line.partition(" ; cohort ")
        rows.append((fen.strip(), rest.split(" ;")[0].strip()))
    return rows


def run(exe, positions, nodes):
    """Returns {fen: (bestmove, depth_reached, score_cp_or_None, is_mate)}."""
    p = pathlib.Path(exe).resolve()
    proc = subprocess.Popen([str(p)], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                            stderr=subprocess.DEVNULL, text=True, bufsize=1,
                            cwd=str(p.parent))
    out = {}
    try:
        proc.stdin.write("uci\n")
        proc.stdin.flush()
        while True:
            line = proc.stdout.readline()
            if not line or line.startswith("uciok"):
                break
        for fen, _cohort in positions:
            proc.stdin.write("ucinewgame\nisready\n")
            proc.stdin.flush()
            while True:
                line = proc.stdout.readline()
                if not line or line.startswith("readyok"):
                    break
            proc.stdin.write("position fen %s\ngo nodes %d\n" % (fen, nodes))
            proc.stdin.flush()
            best_depth, score, is_mate = 0, None, False
            while True:
                line = proc.stdout.readline()
                if not line:
                    raise RuntimeError("engine closed early on %s" % fen)
                m = INFO.match(line.strip())
                if m:
                    d = int(m.group(1))
                    if d >= best_depth:
                        best_depth = d
                        is_mate = m.group(2) == "mate"
                        score = int(m.group(3))
                elif line.startswith("bestmove"):
                    out[fen] = (line.split()[1], best_depth, score, is_mate)
                    break
        proc.stdin.write("quit\n")
        proc.stdin.flush()
    finally:
        try:
            proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            proc.kill()
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--oracle", required=True)
    ap.add_argument("--nodes", type=int, default=300000)
    ap.add_argument("--engine", action="append", required=True, metavar="LABEL=PATH",
                    help="repeatable; compared against the oracle at the same budget")
    args = ap.parse_args()

    positions = load_suite()
    w = sys.stdout.write
    w("suite %s, %d positions, go nodes %d\n" % (SUITE.name, len(positions), args.nodes))
    w("evaluation is HELD CONSTANT: the oracle runs Rarog's HCE through the FFI.\n\n")

    o = run(args.oracle, positions, args.nodes)
    od = sum(v[1] for v in o.values()) / len(o)

    w("%-10s %10s %12s %11s %10s\n"
      % ("engine", "agreement", "mean depth", "vs oracle", "med |dcp|"))
    w("-" * 57 + "\n")
    w("%-10s %10s %12.2f %11s %10s\n" % ("ORACLE", "--", od, "--", "--"))
    for spec in args.engine:
        label, _, path = spec.partition("=")
        r = run(path, positions, args.nodes)
        same = sum(1 for f in positions if r[f[0]][0] == o[f[0]][0])
        rd = sum(r[f[0]][1] for f in positions) / len(positions)
        dcps = sorted(abs(r[f[0]][2] - o[f[0]][2]) for f in positions
                      if not r[f[0]][3] and not o[f[0]][3])
        med = dcps[len(dcps) // 2] if dcps else float("nan")
        w("%-10s %9.1f%% %12.2f %+11.2f %10d\n"
          % (label, 100.0 * same / len(positions), rd, rd - od, med))
    return 0


if __name__ == "__main__":
    sys.exit(main())
