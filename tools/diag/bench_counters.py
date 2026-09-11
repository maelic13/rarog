#!/usr/bin/env python3
"""Sum Rarog's diagnostic counters over a whole `bench` run.

This exists because `bench` dumps `info string diag <name> <value>` **once per
position** -- 40 lines per counter for `bench 13`. A parser that stores instead
of accumulating keeps only the LAST position's numbers, which look entirely
plausible and are wrong by a factor of the corpus size. That mistake reached a
commit message before it was caught, so this is the only sanctioned way to read
bench counters.

Requires a build with `--features diag`. Add `--features tune` as well if you
pass `--set`, since the parameter UCI options only exist under `tune`.

Ratios between counters are only meaningful at stride 1 (the default here):
half the core set is sampled and half is exact, deliberately. See
`analysis/phase4_counter_spec.md`.

Usage:
  python tools/diag/bench_counters.py --exe target/release/rarog.exe
  python tools/diag/bench_counters.py --depth 13 --filter probcut
  python tools/diag/bench_counters.py --set ProbCutSeeGapScale=50 \\
      --set ProbCutMoveCapBase=3 --filter probcut
"""

import argparse
import collections
import os
import pathlib
import subprocess
import sys


def run_bench(exe, depth, stride, options):
    """Drive one bench synchronously and return (nodes, summed counters).

    Streaming stdin, like the differential runner: the engine is written to and
    read from in step. A piped `printf ... | engine` closes stdin and aborts the
    search before it starts.
    """
    env = dict(os.environ)
    env["RAROG_DIAG_SAMPLE_STRIDE"] = str(stride)
    exe_path = pathlib.Path(exe).resolve()
    proc = subprocess.Popen(
        [str(exe_path)],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        text=True,
        bufsize=1,
        env=env,
        cwd=str(exe_path.parent),
    )
    totals = collections.Counter()
    dumps = collections.Counter()
    # Per-position values, in bench order, for callers that need to attribute a
    # counter to the ROOT it was measured under (PLAN 4.11.5). The sum is still
    # the only thing this tool prints: keeping the sequence does not reintroduce
    # the "read the last dump" mistake, it makes the legitimate version of that
    # question answerable.
    sequence = collections.defaultdict(list)
    nodes = 0
    try:
        if options:
            proc.stdin.write("uci\n")
            proc.stdin.flush()
            while True:
                line = proc.stdout.readline()
                if not line or line.startswith("uciok"):
                    break
            for name, value in options:
                proc.stdin.write("setoption name %s value %s\n" % (name, value))
        proc.stdin.write("bench %d\n" % depth)
        proc.stdin.flush()
        while True:
            line = proc.stdout.readline()
            if not line:
                raise RuntimeError("engine closed before the bench summary")
            line = line.rstrip()
            if line.startswith("info string diag "):
                parts = line.split()
                # ACCUMULATE. One dump per position; storing would keep only
                # the last position and silently understate the corpus.
                totals[parts[3]] += int(parts[4])
                dumps[parts[3]] += 1
                sequence[parts[3]].append(int(parts[4]))
            elif line.startswith("Nodes searched"):
                nodes = int(line.split()[-1])
            elif line.startswith("Nodes/second"):
                break
        proc.stdin.write("quit\n")
        proc.stdin.flush()
    finally:
        try:
            proc.wait(timeout=30)
        except subprocess.TimeoutExpired:
            proc.kill()
    return nodes, totals, dumps, dict(sequence)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--exe", default="target/release/rarog.exe")
    ap.add_argument("--depth", type=int, default=13)
    ap.add_argument("--stride", type=int, default=1,
                    help="RAROG_DIAG_SAMPLE_STRIDE; leave at 1 for valid ratios")
    ap.add_argument("--filter", default="",
                    help="substring; only counters whose name contains it")
    ap.add_argument("--set", action="append", default=[], metavar="NAME=VALUE",
                    help="UCI option, repeatable (needs --features tune)")
    args = ap.parse_args()

    options = []
    for item in args.set:
        name, sep, value = item.partition("=")
        if not sep:
            ap.error("--set expects NAME=VALUE, got %r" % item)
        options.append((name, value))

    nodes, totals, dumps, sequence = run_bench(
        args.exe, args.depth, args.stride, options)
    if not totals:
        sys.stdout.write(
            "no diag lines: this binary was not built with --features diag\n")
        return 1

    counts = set(dumps.values())
    sys.stdout.write("bench %d, stride %d, nodes %d\n"
                     % (args.depth, args.stride, nodes))
    sys.stdout.write("summed over %s per-position dumps\n\n"
                     % ("/".join(str(c) for c in sorted(counts))))
    for name in sorted(totals):
        if args.filter and args.filter not in name:
            continue
        sys.stdout.write("  %-32s %12d\n" % (name, totals[name]))
    return 0


if __name__ == "__main__":
    sys.exit(main())
