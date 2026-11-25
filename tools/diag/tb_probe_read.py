#!/usr/bin/env python3
"""Tablebase probe read: nodes, table reads, score class and time per root.

Drives each engine through one `go` on every position, one fresh process per
position, so no hash, history or table state carries from one root to the
next. Every engine gets `Threads 1`, `Hash 64` and, unless it is named in
`--tables-off`, the given `SyzygyPath`; the run stops if an engine with tables
does not report them loaded, or one without tables does. From the last exact
`info depth` line (an aspiration re-search's `lowerbound` or `upperbound`
line belongs to an iteration that may not finish) it records nodes, `tbhits`,
the depth reached, the score and the engine's `time`, and classes the score
as a tablebase or mate win, a loss, or neither; it also records the wall time
from `go` to `bestmove`.

Usage:
  python tools/diag/tb_probe_read.py --epd FILE --men 7,8 --per-count 16 \
      --depth 16 --syzygy DIR --out out.json NAME=ENGINE.exe [NAME=ENGINE.exe]
  python tools/diag/tb_probe_read.py --epd FILE --men 7,8 --per-count 24 \
      --go "wtime 120000 btime 120000 winc 1000 binc 1000" --syzygy DIR \
      --tables-off notb --out out.json tb=ENGINE.exe notb=ENGINE.exe

Positions are the first `--per-count` lines of the EPD with each listed number
of men, in file order, indexed from 0; `--only` or `--skip` keep or drop
indices. With `--depth`, `--max-nodes` adds a node cap (`go depth N nodes M`),
so a root whose tree explodes ends instead of hanging; it is then recorded as
not reaching the depth. With `--go`, the arguments are sent as given and every
root that answers counts as complete. The summary totals the roots that every
engine completed, compares every engine with the first one (nodes, table
reads, score classes, and per root the ratio of wall times), and lists the
roots that any engine left short.
"""

import argparse
import json
import re
import statistics
import subprocess
import sys
import time
from pathlib import Path

# The most an ordinary score prints (`MAX_ORDINARY_CP` in `src/uci/info.rs`):
# a tablebase win prints 20,000 cp less its ply, 239 plies at most.
MAX_ORDINARY_CP = 19_760


def men(fen: str) -> int:
    return sum(ch.isalpha() for ch in fen.split()[0])


def pick(epd: Path, counts: list[int], per_count: int) -> list[str]:
    chosen: dict[int, list[str]] = {n: [] for n in counts}
    for line in epd.read_text().splitlines():
        if not line.strip():
            continue
        fields = line.split()
        fen = " ".join(fields[:4]) + " 0 1"
        n = men(fen)
        if n in chosen and len(chosen[n]) < per_count:
            chosen[n].append(fen)
    for n, fens in chosen.items():
        assert len(fens) == per_count, f"only {len(fens)} positions with {n} men"
    return [fen for n in counts for fen in chosen[n]]


def indices(text: str | None) -> set[int]:
    return {int(i) for i in text.split(",")} if text else set()


def score_class(kind: str, value: int) -> str:
    if kind == "mate":
        return "win" if value > 0 else "loss"
    if value > MAX_ORDINARY_CP:
        return "win"
    if value < -MAX_ORDINARY_CP:
        return "loss"
    return "none"


def last_exact(lines: list[str]) -> str | None:
    """The last `info depth` line with a score that is not a bound."""
    exact = [
        line
        for line in lines
        if line.startswith("info depth")
        and " score " in line
        and " lowerbound" not in line
        and " upperbound" not in line
    ]
    return exact[-1] if exact else None


def parse_search(fen: str, lines: list[str], depth: int | None, wall_ms: float) -> dict:
    """One search's record from its output; `depth` None means a clock search."""
    last = last_exact(lines)
    assert last, f"no exact info line for {fen}"
    m = re.search(r" score (cp|mate) (-?\d+)", last)
    reached = int(re.search(r"^info depth (\d+)", last).group(1))
    engine_time = re.search(r" time (\d+)", last)
    return {
        "fen": fen,
        "depth": reached,
        "complete": depth is None or reached == depth,
        "nodes": int(re.search(r" nodes (\d+)", last).group(1)),
        "tbhits": int(re.search(r" tbhits (\d+)", last).group(1)),
        "score": f"{m.group(1)} {m.group(2)}",
        "class": score_class(m.group(1), int(m.group(2))),
        "time_ms": int(engine_time.group(1)) if engine_time else None,
        "wall_ms": round(wall_ms, 1),
        "bestmove": lines[-1].split()[1],
    }


def search(exe: str, fen: str, go: str, depth: int | None, syzygy: str | None) -> dict:
    proc = subprocess.Popen(
        [exe], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, bufsize=1
    )

    def send(cmd: str) -> None:
        proc.stdin.write(cmd + "\n")
        proc.stdin.flush()

    def read_until(prefix: str) -> list[str]:
        lines = []
        for line in proc.stdout:
            lines.append(line.rstrip())
            if line.startswith(prefix):
                return lines
        raise RuntimeError(f"{exe} closed before {prefix!r}")

    send("uci")
    read_until("uciok")
    send("setoption name Threads value 1")
    send("setoption name Hash value 64")
    if syzygy is not None:
        send(f"setoption name SyzygyPath value {syzygy}")
    send("isready")
    ready = read_until("readyok")
    loaded = any("tablebase files" in line and "Found" in line for line in ready)
    if loaded != (syzygy is not None):
        raise RuntimeError(f"{exe}: tables loaded {loaded}, expected {syzygy is not None}: {ready}")
    send("ucinewgame")
    send(f"position fen {fen}")
    send("isready")
    read_until("readyok")
    start = time.perf_counter()
    send(f"go {go}")
    lines = read_until("bestmove")
    wall_ms = (time.perf_counter() - start) * 1000
    send("quit")
    proc.wait(timeout=30)
    return parse_search(fen, lines, depth, wall_ms)


def summarize(roots: int, results: dict[str, list[dict]]) -> dict:
    """Totals over the roots every engine completed, each engine against the first."""
    complete = [k for k in range(roots) if all(rows[k]["complete"] for rows in results.values())]
    short = {
        name: [rows[k]["index"] for k in range(roots) if not rows[k]["complete"]]
        for name, rows in results.items()
    }
    base_name = next(iter(results))
    base = results[base_name]
    summary = {"roots": roots, "complete_in_all": len(complete), "short": short}
    base_nodes = sum(base[k]["nodes"] for k in complete)
    base_hits = sum(base[k]["tbhits"] for k in complete)
    for name, rows in results.items():
        nodes = sum(rows[k]["nodes"] for k in complete)
        tbhits = sum(rows[k]["tbhits"] for k in complete)
        entry = {
            "nodes": nodes,
            "tbhits": tbhits,
            "max_wall_ms": max(row["wall_ms"] for row in rows),
        }
        if name != base_name:
            entry["nodes_change"] = nodes / base_nodes - 1 if base_nodes else None
            entry["tbhits_change"] = tbhits / base_hits - 1 if base_hits else None
            entry["class_changes"] = [
                {"index": b["index"], "fen": b["fen"], "from": b["class"], "to": r["class"]}
                for b, r in zip(base, rows)
                if b["class"] != r["class"]
            ]
            # Each root's wall time over the first engine's.
            ratios = {
                r["index"]: r["wall_ms"] / b["wall_ms"] for b, r in zip(base, rows) if b["wall_ms"] > 0
            }
            entry["median_wall_ratio"] = statistics.median(ratios.values()) if ratios else None
            entry["roots_at_least_twice"] = [i for i, ratio in ratios.items() if ratio >= 2]
            entry["roots_at_most_half"] = [i for i, ratio in ratios.items() if ratio <= 0.5]
        summary[name] = entry
    return summary


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--epd", required=True, type=Path)
    ap.add_argument("--men", required=True)
    ap.add_argument("--per-count", required=True, type=int)
    budget = ap.add_mutually_exclusive_group(required=True)
    budget.add_argument("--depth", type=int)
    budget.add_argument("--go", help="the `go` arguments, sent as given")
    ap.add_argument("--max-nodes", type=int, default=0)
    ap.add_argument("--only")
    ap.add_argument("--skip")
    ap.add_argument("--syzygy")
    ap.add_argument("--tables-off", default="", help="engine names that run without tables")
    ap.add_argument("--out", required=True, type=Path)
    ap.add_argument("engines", nargs="+")
    args = ap.parse_args()
    if args.go is not None and args.max_nodes:
        ap.error("--max-nodes goes with --depth; put a node cap in --go")

    fens = pick(args.epd, [int(n) for n in args.men.split(",")], args.per_count)
    only, skip = indices(args.only), indices(args.skip)
    roots = [
        (i, fen) for i, fen in enumerate(fens) if (not only or i in only) and i not in skip
    ]
    assert roots, "no roots selected"
    engines = [
        (name, str(Path(exe).resolve(strict=True)))
        for name, exe in (spec.split("=", 1) for spec in args.engines)
    ]
    names = [name for name, _ in engines]
    assert len(set(names)) == len(names), "engine names repeat"
    off = {name for name in args.tables_off.split(",") if name}
    assert off <= set(names), f"--tables-off names no engine: {off - set(names)}"
    if set(names) - off and not args.syzygy:
        ap.error("--syzygy is required for an engine with tables")
    if args.go is not None:
        go, depth = args.go, None
    else:
        go = f"depth {args.depth}" + (f" nodes {args.max_nodes}" if args.max_nodes else "")
        depth = args.depth
    results = {
        name: [
            {"index": i}
            | search(exe, fen, go, depth, None if name in off else args.syzygy)
            for i, fen in roots
        ]
        for name, exe in engines
    }
    summary = summarize(len(roots), results)
    record = {"args": vars(args) | {"epd": str(args.epd), "out": str(args.out)}}
    args.out.write_text(json.dumps(record | {"results": results, "summary": summary}, indent=1))
    json.dump(summary, sys.stdout, indent=1)
    print()
    return 0


if __name__ == "__main__":
    sys.exit(main())
