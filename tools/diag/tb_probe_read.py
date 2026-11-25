#!/usr/bin/env python3
"""Fixed-depth tablebase probe read: nodes, table reads and score class per root.

Drives each engine through `go depth N` on every position, one fresh process
per position, so no hash, history or table state carries from one root to the
next. Every engine gets `Threads 1`, `Hash 64` and the given `SyzygyPath`, and
the run stops if the engine does not report the tables loaded. From the last
`info depth` line it records nodes, `tbhits`, the depth reached and the score,
and classes the score as a tablebase or mate win, a loss, or neither.

Usage:
  python tools/diag/tb_probe_read.py --epd FILE --men 7,8 --per-count 16 \
      --depth 16 --syzygy DIR --out out.json NAME=ENGINE.exe [NAME=ENGINE.exe]

Positions are the first `--per-count` lines of the EPD with each listed number
of men, in file order, indexed from 0; `--only` or `--skip` keep or drop
indices. `--max-nodes` adds a node cap (`go depth N nodes M`), so a root whose
tree explodes ends instead of hanging; it is then recorded as not reaching the
depth. The summary totals the roots that every engine searched to the depth,
compares every engine with the first one, and lists the roots that any engine
left short.
"""

import argparse
import json
import re
import subprocess
import sys
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


def search(exe: str, fen: str, depth: int, max_nodes: int, syzygy: str) -> dict:
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
    send(f"setoption name SyzygyPath value {syzygy}")
    send("isready")
    ready = read_until("readyok")
    if not any("tablebase files" in line and "Found" in line for line in ready):
        raise RuntimeError(f"{exe} did not report loading tables: {ready}")
    send("ucinewgame")
    send(f"position fen {fen}")
    send(f"go depth {depth}" + (f" nodes {max_nodes}" if max_nodes else ""))
    lines = read_until("bestmove")
    send("quit")
    proc.wait(timeout=30)
    infos = [line for line in lines if line.startswith("info depth") and " score " in line]
    assert infos, f"no info line for {fen}"
    last = infos[-1]
    m = re.search(r" score (cp|mate) (-?\d+)", last)
    reached = int(re.search(r"^info depth (\d+)", last).group(1))
    return {
        "fen": fen,
        "depth": reached,
        "complete": reached == depth,
        "nodes": int(re.search(r" nodes (\d+)", last).group(1)),
        "tbhits": int(re.search(r" tbhits (\d+)", last).group(1)),
        "score": f"{m.group(1)} {m.group(2)}",
        "class": score_class(m.group(1), int(m.group(2))),
        "bestmove": lines[-1].split()[1],
    }


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--epd", required=True, type=Path)
    ap.add_argument("--men", required=True)
    ap.add_argument("--per-count", required=True, type=int)
    ap.add_argument("--depth", required=True, type=int)
    ap.add_argument("--max-nodes", type=int, default=0)
    ap.add_argument("--only")
    ap.add_argument("--skip")
    ap.add_argument("--syzygy", required=True)
    ap.add_argument("--out", required=True, type=Path)
    ap.add_argument("engines", nargs="+")
    args = ap.parse_args()

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
    results = {
        name: [
            {"index": i} | search(exe, fen, args.depth, args.max_nodes, args.syzygy)
            for i, fen in roots
        ]
        for name, exe in engines
    }
    complete = [
        k for k in range(len(roots)) if all(rows[k]["complete"] for rows in results.values())
    ]
    short = {
        name: [rows[k]["index"] for k in range(len(roots)) if not rows[k]["complete"]]
        for name, rows in results.items()
    }
    base_name = engines[0][0]
    base = results[base_name]
    summary = {"roots": len(roots), "complete_in_all": len(complete), "short": short}
    base_nodes = sum(base[k]["nodes"] for k in complete)
    base_hits = sum(base[k]["tbhits"] for k in complete)
    for name, rows in results.items():
        nodes = sum(rows[k]["nodes"] for k in complete)
        tbhits = sum(rows[k]["tbhits"] for k in complete)
        entry = {"nodes": nodes, "tbhits": tbhits}
        if name != base_name:
            entry["nodes_change"] = nodes / base_nodes - 1 if base_nodes else None
            entry["tbhits_change"] = tbhits / base_hits - 1 if base_hits else None
            entry["class_changes"] = [
                {"index": b["index"], "fen": b["fen"], "from": b["class"], "to": r["class"]}
                for b, r in zip(base, rows)
                if b["class"] != r["class"]
            ]
        summary[name] = entry
    record = {"args": vars(args) | {"epd": str(args.epd), "out": str(args.out)}}
    args.out.write_text(json.dumps(record | {"results": results, "summary": summary}, indent=1))
    json.dump(summary, sys.stdout, indent=1)
    print()
    return 0


if __name__ == "__main__":
    sys.exit(main())
