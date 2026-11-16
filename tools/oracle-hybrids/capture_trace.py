"""D.7.3 layer A: capture the twin's decision records on layer B's positions.

One process a position, one trace file a position (parallel processes do not
share a file), `go depth 14`, Hash 64, one thread, the base configuration.
Then counts the records of each kind and the positions they came from against
the frozen floor: at least 2,000 records of each kind, 200 for a kind that
fires rarely, from at least 200 positions.

usage: capture_trace.py <engine.exe> <positions.jsonl> <out_dir> [count] [stride]
"""
import collections
import json
import os
import sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import tree_stats  # noqa: E402

exe, positions_path, out_dir = sys.argv[1:4]
count = int(sys.argv[4]) if len(sys.argv) > 4 else 250
stride = sys.argv[5] if len(sys.argv) > 5 else "512"
EXCLUDED = {int(x) for x in os.environ.get("TREE_STATS_EXCLUDE", "").split(",") if x}  # runaway searches, as layer B left them out
out = Path(out_dir)
out.mkdir(parents=True, exist_ok=True)
pos = [p for p in (json.loads(l) for l in open(positions_path)) if p["id"] not in EXCLUDED][:count]


def one(p):
    env = dict(os.environ)
    for k in ("RAROG_SPREAD_SALT", "RAROG_PLANT"):
        env.pop(k, None)
    path = out / f"trace_{p['id']:03d}.txt"
    path.unlink(missing_ok=True)
    env["RAROG_TRACE"] = str(path)
    env["RAROG_TRACE_STRIDE"] = stride
    result = tree_stats.search_one(exe, p["fen"], env)
    kinds = collections.Counter()
    if path.exists():
        with open(path) as f:
            for line in f:
                kinds[line.split(" ", 1)[0]] += 1
    return p["id"], result.get("timeout", False), kinds


with ThreadPoolExecutor(14) as ex:
    results = list(ex.map(one, pos))
total = collections.Counter()
positions_with = collections.Counter()
for _, _, kinds in results:
    total.update(kinds)
    positions_with.update(kinds.keys())
timeouts = [i for i, t, _ in results if t]
summary = {"positions": len(results), "timeouts": timeouts, "stride": int(stride),
           "kinds": {k: {"records": total[k], "positions": positions_with[k]} for k in sorted(total)}}
with open(out / "summary.json", "w", newline="\n") as f:
    json.dump(summary, f, indent=1)
short = [k for k in sorted(total) if total[k] < 2000]
for k in sorted(total):
    print(f"{k:16s} {total[k]:8d} records from {positions_with[k]:4d} positions")
print(f"{len(results)} positions, {len(timeouts)} timed out; kinds under 2,000 records: {short or 'none'}")
