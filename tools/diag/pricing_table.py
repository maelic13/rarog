#!/usr/bin/env python3
"""Run and read the pricing instrument (PLAN D.4.4).

A `--features pricing` build re-searches a sample of the moves each pruning
rule and reduction hid, and appends one record per probe to the file named by
`RAROG_PRICING_OUT`, with a `# search nodes ...` line per search
(`analysis/pruning_pricing_card_2026-11-04.md`). This tool drives one
`bench` with sampling on and checks the main search's node count, which must
be the fingerprint whatever the probes did, then tabulates the records.

  python tools/diag/pricing_table.py run --exe target/pricing/release/rarog.exe \\
      --out analysis/artifacts/d44-pricing/read.tsv --expect-nodes 14749043
  python tools/diag/pricing_table.py table read.tsv [--by depth|node|move]
  python tools/diag/pricing_table.py compare honest.tsv planted.tsv --rule futility
  python tools/diag/pricing_table.py run ... --confine --set CoreFpBase=-100
  python tools/diag/pricing_table.py plant honest.tsv planted.tsv --rule futility

`plant` is the registered read (the card's section 13): with the plant
confined to the measured search, the rule must fire at least 1.2 times as
often per main-search node, and the label's Mantel-Haenszel odds ratio
within depth, node-type and move-class strata decides PASS (lower bound
above 1, exit 0), FAIL (upper bound below 1.25, exit 1) or UNRESOLVED
(exit 2); a plant below the activation is INACTIVE (exit 3).

A record: rule, depth (a quiescence rule's is its quiescence ply), ply, node
type, move number, the moves the decision also removed, the probe's nodes,
the label (1: the hidden move raised alpha, or a node-level verdict was
wrong), for a raising move-level label whether its node failed low, and
whether the move was quiet (`-` for a node-level rule).
"""

from __future__ import annotations

import argparse
import collections
import math
import os
import pathlib
import subprocess
import sys
from dataclasses import dataclass

FIELDS = 10
MIN_CELL = 400
# The plant read's registered thresholds (card section 13); never tuned.
ACTIVATION = 1.2
FAIL_BELOW = 1.25
DEPTH_CAP = 12
Z95 = 1.959964


@dataclass(frozen=True)
class Record:
    rule: str
    depth: int
    ply: int
    node: str
    move: int
    tail: int
    nodes: int
    label: bool
    changed: str
    quiet: str


@dataclass
class Read:
    records: list
    main_nodes: int
    probe_nodes: int
    searches: int


def parse(lines) -> Read:
    """Every record and the summed summary lines of one or more runs."""
    records, main_nodes, probe_nodes, searches = [], 0, 0, 0
    for number, raw in enumerate(lines, 1):
        line = raw.rstrip("\r\n")
        if not line:
            continue
        if line.startswith("#"):
            words = line.split()
            # "# search nodes N probes P probe_nodes Q": SUM, one per search.
            main_nodes += int(words[3])
            probe_nodes += int(words[7])
            searches += 1
            continue
        cols = line.split("\t")
        if len(cols) != FIELDS:
            raise ValueError(f"line {number}: {len(cols)} fields, expected {FIELDS}")
        if cols[7] not in ("0", "1") or cols[8] not in ("-", "?", "0", "1"):
            raise ValueError(f"line {number}: bad label or changed field")
        if cols[9] not in ("-", "0", "1"):
            raise ValueError(f"line {number}: bad quiet field")
        records.append(Record(cols[0], int(cols[1]), int(cols[2]), cols[3], int(cols[4]),
                              int(cols[5]), int(cols[6]), cols[7] == "1", cols[8], cols[9]))
    return Read(records, main_nodes, probe_nodes, searches)


def wilson(hits: int, n: int, z: float = 1.959964) -> tuple[float, float]:
    """The 95% Wilson interval of a share."""
    if n == 0:
        return (0.0, 1.0)
    p = hits / n
    centre = p + z * z / (2 * n)
    margin = z * math.sqrt(p * (1 - p) / n + z * z / (4 * n * n))
    denominator = 1 + z * z / n
    return ((centre - margin) / denominator, (centre + margin) / denominator)


def depth_bucket(depth: int) -> str:
    for top, name in ((1, "<=1"), (2, "2"), (3, "3"), (5, "4-5"), (8, "6-8")):
        if depth <= top:
            return name
    return "9+"


def move_bucket(move: int) -> str:
    for top, name in ((3, "1-3"), (7, "4-7"), (15, "8-15")):
        if move <= top:
            return name
    return "16+"


KEYS = {
    None: lambda r: (r.rule,),
    "depth": lambda r: (r.rule, depth_bucket(r.depth)),
    "node": lambda r: (r.rule, r.node),
    "move": lambda r: (r.rule, move_bucket(r.move)),
    "quiet": lambda r: (r.rule, {"1": "quiet", "0": "noisy", "-": "node"}[r.quiet]),
}


def table(read: Read, by: str | None) -> list[dict]:
    cells = collections.defaultdict(list)
    for record in read.records:
        cells[KEYS[by](record)].append(record)
    rows = []
    for key in sorted(cells):
        group = cells[key]
        n = len(group)
        hits = sum(r.label for r in group)
        decided = [r for r in group if r.changed in ("0", "1")]
        low, high = wilson(hits, n)
        rows.append({
            "cell": " ".join(key),
            "probes": n,
            "share": hits / n,
            "low": low,
            "high": high,
            "changed": (sum(r.changed == "1" for r in decided) / len(decided)) if decided else None,
            "mean_nodes": sum(r.nodes for r in group) / n,
            "mean_tail": sum(r.tail for r in group) / n,
            "thin": n < MIN_CELL,
        })
    return rows


def print_table(read: Read, rows: list[dict]) -> None:
    print(f"searches {read.searches}  main nodes {read.main_nodes}  probes {len(read.records)}"
          f"  probe nodes {read.probe_nodes}")
    print(f"{'cell':<22}{'probes':>8}{'share':>9}{'95% interval':>19}{'changed':>9}"
          f"{'nodes':>9}{'tail':>7}")
    for row in rows:
        changed = "-" if row["changed"] is None else f"{row['changed']:.3f}"
        thin = "  thin" if row["thin"] else ""
        print(f"{row['cell']:<22}{row['probes']:>8}{row['share']:>9.4f}"
              f"   [{row['low']:.4f}, {row['high']:.4f}]{changed:>9}"
              f"{row['mean_nodes']:>9.1f}{row['mean_tail']:>7.1f}{thin}")


def stratum(record: Record) -> tuple:
    """Depth (pooled from DEPTH_CAP up), node type and move class."""
    return (min(record.depth, DEPTH_CAP), record.node, record.quiet)


def mantel_haenszel(planted: list, honest: list) -> tuple[float, float, float] | None:
    """The label's common odds ratio, planted against honest, over strata,
    with its 95% interval by Robins, Breslow and Greenland; None when a
    sum is empty and the ratio is undefined."""
    cells = collections.defaultdict(lambda: [0, 0, 0, 0])
    for arm, records in ((0, planted), (2, honest)):
        for r in records:
            cells[stratum(r)][arm + (0 if r.label else 1)] += 1
    sum_r = sum_s = sum_pr = sum_ps_qr = sum_qs = 0.0
    for a, b, c, d in cells.values():
        n = a + b + c + d
        if a + b == 0 or c + d == 0:
            continue
        r, s = a * d / n, b * c / n
        p, q = (a + d) / n, (b + c) / n
        sum_r += r
        sum_s += s
        sum_pr += p * r
        sum_ps_qr += p * s + q * r
        sum_qs += q * s
    if sum_r == 0 or sum_s == 0:
        return None
    ratio = sum_r / sum_s
    variance = (sum_pr / (2 * sum_r ** 2) + sum_ps_qr / (2 * sum_r * sum_s)
                + sum_qs / (2 * sum_s ** 2))
    spread = Z95 * math.sqrt(variance)
    return ratio, ratio * math.exp(-spread), ratio * math.exp(spread)


def plant_verdict(honest: Read, planted: Read, rule: str, quiet: str | None) -> int:
    """Print the registered plant read; returns its exit code."""
    wanted = None if quiet is None else ("1" if quiet == "quiet" else "0")
    arms = {}
    for name, read in (("honest", honest), ("planted", planted)):
        group = [r for r in read.records if r.rule == rule
                 and (wanted is None or r.quiet == wanted)]
        if not group or read.main_nodes == 0:
            raise SystemExit(f"{name}: no {rule} records or no summary lines")
        hits = sum(r.label for r in group)
        rate = len(group) / read.main_nodes
        arms[name] = group
        print(f"{name:<8} {rule}{' ' + quiet if quiet else ''}  probes {len(group)}"
              f"  per main node {rate:.6f}  share {hits / len(group):.4f}")
    activation = ((len(arms["planted"]) / planted.main_nodes)
                  / (len(arms["honest"]) / honest.main_nodes))
    print(f"activation {activation:.3f} (registered at least {ACTIVATION})")
    if activation < ACTIVATION:
        print("INACTIVE")
        return 3
    pooled = mantel_haenszel(arms["planted"], arms["honest"])
    if pooled is None:
        print("UNRESOLVED: the odds ratio is undefined")
        return 2
    ratio, low, high = pooled
    print(f"Mantel-Haenszel odds ratio {ratio:.3f}  95% [{low:.3f}, {high:.3f}]")
    if low > 1:
        print("PASS")
        return 0
    if high < FAIL_BELOW:
        print("FAIL")
        return 1
    print("UNRESOLVED")
    return 2


def run_bench(exe: str, depth: int, out: pathlib.Path, options: list[tuple[str, str]],
              shift: int, confine: bool = False) -> int:
    """One bench with sampling on; returns the main search's node count.

    Streamed like bench_counters.py: a piped `printf ... | engine` closes stdin,
    which ends the search, and `quit` sent early aborts `bench`.
    """
    if out.exists():
        raise SystemExit(f"{out} exists: the engine appends, so a rerun would mix two runs")
    env = dict(os.environ)
    env["RAROG_PRICING"] = str(shift)
    env["RAROG_PRICING_OUT"] = str(out.resolve())
    if confine:
        env["RAROG_PRICING_CONFINE"] = "1"
    else:
        env.pop("RAROG_PRICING_CONFINE", None)
    exe_path = pathlib.Path(exe).resolve()
    proc = subprocess.Popen([str(exe_path)], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                            text=True, bufsize=1, env=env, cwd=str(exe_path.parent))
    nodes = None
    try:
        if options:
            proc.stdin.write("uci\n")
            proc.stdin.flush()
            names = set()
            while True:
                line = proc.stdout.readline()
                if not line:
                    raise RuntimeError("engine closed before uciok")
                if line.startswith("option name "):
                    names.add(line.split()[2])
                if line.startswith("uciok"):
                    break
            for name, value in options:
                if name not in names:
                    raise SystemExit(f"the engine does not expose {name}: build with --features tune")
                proc.stdin.write(f"setoption name {name} value {value}\n")
        proc.stdin.write(f"bench {depth}\n")
        proc.stdin.flush()
        while True:
            line = proc.stdout.readline()
            if not line:
                raise RuntimeError("engine closed before the bench summary")
            if line.startswith("Nodes searched"):
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
    if nodes is None:
        raise RuntimeError("no node count in the bench output")
    return nodes


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = ap.add_subparsers(dest="command", required=True)
    run = sub.add_parser("run", help="one bench with sampling on")
    run.add_argument("--exe", required=True)
    run.add_argument("--out", required=True)
    run.add_argument("--depth", type=int, default=13)
    run.add_argument("--shift", type=int, default=0, help="RAROG_PRICING: 0 the card's rates")
    run.add_argument("--set", action="append", default=[], metavar="NAME=VALUE",
                     help="UCI option, repeatable (needs --features tune)")
    run.add_argument("--expect-nodes", type=int,
                     help="the fingerprint the main search must reproduce")
    run.add_argument("--confine", action="store_true",
                     help="probes search with the compiled-in parameters")
    show = sub.add_parser("table", help="tabulate records")
    show.add_argument("records", nargs="+")
    show.add_argument("--by", choices=["depth", "node", "move", "quiet"])
    cmp = sub.add_parser("compare", help="a planted read against the honest one")
    cmp.add_argument("honest")
    cmp.add_argument("planted")
    cmp.add_argument("--rule", required=True)
    cmp.add_argument("--quiet", choices=["quiet", "noisy"],
                     help="only the quiet or only the noisy moves of the rule")
    plant = sub.add_parser("plant", help="the registered plant read")
    plant.add_argument("honest")
    plant.add_argument("planted")
    plant.add_argument("--rule", required=True)
    plant.add_argument("--quiet", choices=["quiet", "noisy"])
    args = ap.parse_args(argv)

    if args.command == "run":
        options = []
        for item in args.set:
            name, sep, value = item.partition("=")
            if not sep:
                raise SystemExit(f"--set wants NAME=VALUE, got {item}")
            options.append((name, value))
        out = pathlib.Path(args.out)
        out.parent.mkdir(parents=True, exist_ok=True)
        nodes = run_bench(args.exe, args.depth, out, options, args.shift, args.confine)
        read = parse(out.read_text().splitlines())
        print(f"bench nodes {nodes}  recorded main nodes {read.main_nodes}"
              f"  probes {len(read.records)}  probe nodes {read.probe_nodes}")
        if read.main_nodes != nodes:
            print("FAIL: the summary lines do not sum to the bench count")
            return 1
        if args.expect_nodes is not None and nodes != args.expect_nodes:
            print(f"FAIL: main search {nodes}, expected {args.expect_nodes}: a probe leaked")
            return 1
        return 0
    if args.command == "table":
        lines = []
        for name in args.records:
            lines += pathlib.Path(name).read_text().splitlines()
        read = parse(lines)
        print_table(read, table(read, args.by))
        return 0
    if args.command == "plant":
        return plant_verdict(parse(pathlib.Path(args.honest).read_text().splitlines()),
                             parse(pathlib.Path(args.planted).read_text().splitlines()),
                             args.rule, args.quiet)
    honest = parse(pathlib.Path(args.honest).read_text().splitlines())
    planted = parse(pathlib.Path(args.planted).read_text().splitlines())
    rows = {}
    for name, read in (("honest", honest), ("planted", planted)):
        group = [r for r in read.records if r.rule == args.rule
                 and (args.quiet is None or r.quiet == ("1" if args.quiet == "quiet" else "0"))]
        hits = sum(r.label for r in group)
        rows[name] = (len(group), hits / len(group) if group else 0.0, wilson(hits, len(group)))
        n, share, (low, high) = rows[name]
        scope = f" {args.quiet}" if args.quiet else ""
        print(f"{name:<8} {args.rule}{scope}  probes {n}  share {share:.4f}  [{low:.4f}, {high:.4f}]")
    separated = rows["planted"][2][0] > rows["honest"][2][1]
    print("SEPARATED: the planted rule reads higher, intervals disjoint" if separated
          else "NOT SEPARATED")
    return 0 if separated else 1


if __name__ == "__main__":
    sys.exit(main())
