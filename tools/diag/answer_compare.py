#!/usr/bin/env python3
"""Compare what Rarog and the oracle ANSWER, not how often their mechanisms fire.

PLAN 4.6/4.10 diagnostic. The differential suite measures RATES -- how often
each mechanism fires per node. It cannot say whether the search returns the
right move or the right score, and the central unexplained fact about Rarog is
exactly an answer-quality problem: RAR-S53 measured it searching 2.5 plies
DEEPER than Basilisk at equal nodes and equal speed while losing 65 Elo, and
RAR-S52/S55 measured its move ordering as BETTER than the reference in every
cohort. No rate explains "deeper, better ordered, much weaker".

WHY THIS COMPARISON IS VALID. `hybrid-diag` is Stockfish's search driving
RAROG'S OWN HCE through the FFI. Evaluation is therefore held constant, and any
difference in the returned move or score at the same position and depth is
attributable to search alone. That is what makes this different from comparing
against an arbitrary stronger engine.

ONE SEARCH PER POSITION PER ENGINE. A `go depth N` emits an `info depth ...`
line per iteration, so a single search yields the whole trajectory: the score
and best move at every depth from 1 to N. Everything below is derived from
those trajectories.

WHAT IT MEASURES
  agreement      do the two engines return the same move at each depth?
  score delta    signed, so a systematic optimism or pessimism is visible
  settle depth   the last depth at which the engine changed its own best move.
                 A high settle depth means the engine keeps revising; a LOW one
                 combined with disagreement means premature conviction, which
                 is the shape `root_best_changes` 0.29x already hints at.
  revisions      how many times the root move changed across the whole search
  self-survival  does an engine's own depth-D answer survive to its own final
                 depth? Computed per engine against ITSELF, so it privileges
                 neither side -- unlike scoring both against the oracle's deep
                 answer, which would flatter the oracle by construction.
  pv split       the first ply at which the two principal variations diverge
  volatility     mean absolute score change per iteration; an unstable score is
                 a search that keeps discovering it was wrong

Usage:
  python tools/diag/answer_compare.py \\
      --rarog target/release/rarog.exe \\
      --oracle hybrid/stockfish/src/stockfish.exe \\
      --depth 14
"""

import argparse
import collections
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
SUITE = ROOT / "tools" / "diag" / "phase4_suite_v1.epd"

INFO = re.compile(
    r"^info depth (\d+).*?score (cp|mate) (-?\d+).*?\bpv ([a-h][1-8][a-h][1-8][qrbn]?(?: \S+)*)"
)


def load_suite():
    rows = []
    for line in SUITE.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        fen, _, rest = line.partition(" ; cohort ")
        rows.append((fen.strip(), rest.split(" ;")[0].strip()))
    return rows


def to_cp(kind, value):
    """Mate scores mapped to a large cp so deltas stay orderable."""
    v = int(value)
    return v if kind == "cp" else (30000 - abs(v)) * (1 if v > 0 else -1)


def is_mate(cp):
    return abs(cp) > 20000


def run_engine(exe, positions, depth, options=()):
    """One process for the whole suite. Returns {fen: {depth: (cp, pv_list)}}."""
    exe_path = pathlib.Path(exe).resolve()
    proc = subprocess.Popen(
        [str(exe_path)], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL, text=True, bufsize=1,
        cwd=str(exe_path.parent),
    )
    out = {}
    try:
        proc.stdin.write("uci\n")
        proc.stdin.flush()
        while True:
            line = proc.stdout.readline()
            if not line or line.startswith("uciok"):
                break
        # Options for the rarog side only; sent after uciok and before the
        # first ucinewgame, so every position sees them. Anchored on the
        # function body: the first attempt matched this same uciok line
        # inside the MODULE DOCSTRING, so the block was inserted as prose
        # and every --rset run silently measured the defaults.
        for opt in options:
            name, _, value = opt.partition("=")
            proc.stdin.write("setoption name %s value %s\n" % (name, value))
        proc.stdin.flush()
        for fen, _cohort in positions:
            # Streaming stdin, as the differential runner does: a piped
            # `go ... quit` aborts the search before it starts on both engines.
            # Clear the TT between positions. Without this the table carries
            # across unrelated positions and the ANSWER changes: on
            # k7/p4p2/P1q1b1p1/3p3p/3Q4/7P/5PP1/1R4K1 Rarog reports mate in 6
            # from depth 9 when searched alone, and cp 1036 when it follows
            # other positions in the same process.
            proc.stdin.write("ucinewgame\nisready\n")
            proc.stdin.flush()
            while True:
                line = proc.stdout.readline()
                if not line or line.startswith("readyok"):
                    break
            proc.stdin.write("position fen %s\ngo depth %d\n" % (fen, depth))
            proc.stdin.flush()
            traj = {}
            while True:
                line = proc.stdout.readline()
                if not line:
                    raise RuntimeError("engine closed early on %s" % fen)
                m = INFO.match(line.strip())
                if m:
                    d = int(m.group(1))
                    # Keep the LAST line at each depth: an aspiration re-search
                    # emits several, and only the final one is that depth's
                    # answer.
                    traj[d] = (to_cp(m.group(2), m.group(3)), m.group(4).split())
                elif line.startswith("bestmove"):
                    break
            out[fen] = traj
        proc.stdin.write("quit\n")
        proc.stdin.flush()
    finally:
        try:
            proc.wait(timeout=20)
        except subprocess.TimeoutExpired:
            proc.kill()
    return out


def settle_depth(traj):
    """Last depth at which this engine changed its own best move."""
    last, settle = None, 0
    for d in sorted(traj):
        mv = traj[d][1][0]
        if last is not None and mv != last:
            settle = d
        last = mv
    return settle


def revisions(traj):
    n, last = 0, None
    for d in sorted(traj):
        mv = traj[d][1][0]
        if last is not None and mv != last:
            n += 1
        last = mv
    return n


def volatility(traj):
    ds = sorted(traj)
    if len(ds) < 2:
        return 0.0
    diffs = [abs(traj[ds[i]][0] - traj[ds[i - 1]][0]) for i in range(1, len(ds))]
    return sum(diffs) / len(diffs)


def pv_split(a, b):
    """First ply at which two PVs diverge; None if one is a prefix of the other."""
    for i, (x, y) in enumerate(zip(a, b)):
        if x != y:
            return i
    return None


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--rarog", required=True)
    ap.add_argument("--oracle", required=True)
    ap.add_argument("--depth", type=int, default=14)
    ap.add_argument("--rset", action="append", default=[], metavar="NAME=VALUE",
                    help="UCI option for the rarog side (repeatable)")
    ap.add_argument("--disagree", action="store_true",
                    help="list the disagreeing positions instead of the summary")
    args = ap.parse_args()

    positions = load_suite()
    sys.stdout.write("suite %s, %d positions, depth %d, 1 thread\n"
                     % (SUITE.name, len(positions), args.depth))
    sys.stdout.write("evaluation is HELD CONSTANT: the oracle runs Rarog's HCE "
                     "through the FFI, so every\ndifference below is search.\n\n")

    r = run_engine(args.rarog, positions, args.depth, args.rset)
    o = run_engine(args.oracle, positions, args.depth)

    if args.disagree:
        # 4.6.5: the summary says HOW MANY disagree; this says WHICH, so each
        # case can be diagnosed individually.
        for fen, cohort in positions:
            if args.depth not in r.get(fen, {}) or args.depth not in o.get(fen, {}):
                continue
            a, b = r[fen][args.depth], o[fen][args.depth]
            kinds = []
            if is_mate(a[0]) != is_mate(b[0]):
                kinds.append('MATE-DISAGREE')
            elif is_mate(a[0]) and is_mate(b[0]) and a[0] != b[0]:
                kinds.append('mate-distance')
            if a[1][0] != b[1][0]:
                kinds.append('move')
            if not is_mate(a[0]) and not is_mate(b[0]) and abs(a[0] - b[0]) >= 100:
                kinds.append('dcp>=100')
            if not kinds:
                continue
            sys.stdout.write("%-14s %-22s rarog %7d %-6s | oracle %7d %-6s\n"
                             % (cohort, ','.join(kinds), a[0], a[1][0],
                                b[0], b[1][0]))
            sys.stdout.write("    %s\n" % fen)
        return 0

    # --- agreement and score delta, per depth -------------------------------
    sys.stdout.write("AGREEMENT BY DEPTH (both engines answered at that depth)\n")
    sys.stdout.write("dcp is rarog minus oracle. MEDIAN over the cp-only "
                     "population; mate/non-mate disagreements apart.\n")
    sys.stdout.write("%6s %8s %10s %10s %10s %8s %7s\n"
                     % ("depth", "n", "same move", "med dcp", "med |dcp|",
                        "cp-only", "mate!="))
    for d in range(1, args.depth + 1):
        pairs = [(r[f][d], o[f][d]) for f, _ in positions
                 if d in r.get(f, {}) and d in o.get(f, {})]
        if not pairs:
            continue
        same = sum(1 for a, b in pairs if a[1][0] == b[1][0])
        # Split the populations: a mate/non-mate disagreement is a ~30000 cp
        # delta and would swamp every ordinary one. The first run of this
        # tool reported mean |dcp| = 3014 for two engines sharing an eval,
        # which is how the contamination was caught.
        cp_pairs = [(a, b) for a, b in pairs
                    if not is_mate(a[0]) and not is_mate(b[0])]
        mate_split = sum(1 for a, b in pairs if is_mate(a[0]) != is_mate(b[0]))
        if cp_pairs:
            dcp = sorted(a[0] - b[0] for a, b in cp_pairs)
            med = dcp[len(dcp) // 2]
            mad = sorted(abs(x) for x in dcp)[len(dcp) // 2]
        else:
            med = mad = 0
        sys.stdout.write("%6d %8d %9.1f%% %10d %10d %8d %7d\n"
                         % (d, len(pairs), 100.0 * same / len(pairs),
                            med, mad, len(cp_pairs), mate_split))

    # --- per-engine trajectory shape ---------------------------------------
    sys.stdout.write("\nTRAJECTORY SHAPE (per engine, computed against ITSELF)\n")
    sys.stdout.write("%-10s %12s %12s %12s\n"
                     % ("", "settle depth", "revisions", "volatility cp"))
    for label, data in (("rarog", r), ("oracle", o)):
        fens = [f for f, _ in positions if data.get(f)]
        sys.stdout.write("%-10s %12.2f %12.2f %12.1f\n"
                         % (label,
                            sum(settle_depth(data[f]) for f in fens) / len(fens),
                            sum(revisions(data[f]) for f in fens) / len(fens),
                            sum(volatility(data[f]) for f in fens) / len(fens)))

    # --- self-survival: does a shallow answer survive to the final depth? ---
    sys.stdout.write("\nSELF-SURVIVAL: share of positions whose depth-D move equals\n")
    sys.stdout.write("that engine's OWN final-depth move. Neither engine is the\n")
    sys.stdout.write("reference, so this favours neither.\n")
    sys.stdout.write("%6s %12s %12s\n" % ("depth", "rarog", "oracle"))
    for d in range(max(1, args.depth - 8), args.depth):
        row = [d]
        for data in (r, o):
            fens = [f for f, _ in positions
                    if d in data.get(f, {}) and args.depth in data.get(f, {})]
            if not fens:
                row.append(float("nan"))
                continue
            hit = sum(1 for f in fens
                      if data[f][d][1][0] == data[f][args.depth][1][0])
            row.append(100.0 * hit / len(fens))
        sys.stdout.write("%6d %11.1f%% %11.1f%%\n" % tuple(row))

    # --- where the lines split, and by cohort ------------------------------
    splits, by_cohort = [], collections.defaultdict(lambda: [0, 0])
    for fen, cohort in positions:
        if args.depth not in r.get(fen, {}) or args.depth not in o.get(fen, {}):
            continue
        a, b = r[fen][args.depth], o[fen][args.depth]
        by_cohort[cohort][1] += 1
        if a[1][0] == b[1][0]:
            by_cohort[cohort][0] += 1
        s = pv_split(a[1], b[1])
        if s is not None:
            splits.append(s)
    if splits:
        sys.stdout.write("\nPV SPLIT PLY at depth %d: mean %.2f, median %d, "
                         "ply-0 (different move) %d of %d\n"
                         % (args.depth, sum(splits) / len(splits),
                            sorted(splits)[len(splits) // 2],
                            sum(1 for s in splits if s == 0), len(splits)))

    sys.stdout.write("\nFINAL-DEPTH AGREEMENT BY COHORT\n")
    for cohort, (same, total) in sorted(by_cohort.items()):
        sys.stdout.write("  %-18s %3d/%-3d %6.1f%%\n"
                         % (cohort, same, total, 100.0 * same / total))
    return 0


if __name__ == "__main__":
    sys.exit(main())
