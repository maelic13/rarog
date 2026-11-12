"""Price the HCE bridge inside an oracle: time per node of the same fixed-depth
bench at two bridge repeat counts (RAROG_HCE_BRIDGE_REPEAT 1 and 3). Usage:
  python -I bridge_price.py <engine.exe> <out.json> <cycles> -- <bench command words...>
The command is sent on stdin, then quit. Every run must report the same node
count, or the tree changed and the read is void. Understands Stockfish's
`Total time (ms)` / `Nodes searched` summary and Reckless's pretty total line
(`<nodes> <seconds>s <nps> N/s`)."""
import json, os, re, subprocess, sys, time

exe, out, cycles = sys.argv[1], sys.argv[2], int(sys.argv[3])
assert sys.argv[4] == "--"
command = " ".join(sys.argv[5:])


def parse(text):
    sf_nodes = re.findall(r"Nodes searched\s*:\s*(\d+)", text)
    sf_ms = re.findall(r"Total time \(ms\)\s*:\s*(\d+)", text)
    if sf_nodes and sf_ms:
        return int(sf_nodes[-1]), float(sf_ms[-1])
    rk = re.findall(r"^\s*(\d+)\s+([\d.]+)s\s+(\d+) N/s\s*$", text, re.M)
    if rk:
        nodes, seconds, _ = rk[-1]
        return int(nodes), float(seconds) * 1000.0
    raise SystemExit("no bench summary in the output:\n" + text[-2000:])


rows = []
for cycle in range(cycles):
    for repeat in (1, 3):
        env = dict(os.environ, RAROG_HCE_BRIDGE_REPEAT=str(repeat))
        start = time.perf_counter()
        proc = subprocess.run([exe], input=f"{command}\nquit\n", capture_output=True, text=True,
                              env=env, timeout=3600, cwd=os.path.dirname(os.path.abspath(exe)))
        wall = time.perf_counter() - start
        nodes, ms = parse(proc.stdout + proc.stderr)
        rows.append({"cycle": cycle, "repeat": repeat, "exit": proc.returncode, "nodes": nodes, "ms": ms,
                     "wall_s": round(wall, 3)})
        print(rows[-1], flush=True)
assert all(r["exit"] == 0 for r in rows), "a run failed"
assert len({r["nodes"] for r in rows}) == 1, "node counts differ: the tree changed"
by = {k: sorted(r["ms"] for r in rows if r["repeat"] == k) for k in (1, 3)}
med = {k: v[len(v) // 2] for k, v in by.items()}
nodes = rows[0]["nodes"]
per_node = {k: med[k] * 1e6 / nodes for k in med}
bridge_ns = (per_node[3] - per_node[1]) / 2
summary = {"engine": exe, "command": command, "nodes": nodes, "median_ms": med, "ns_per_node": per_node,
           "bridge_ns_per_node": bridge_ns, "bridge_share_at_1": bridge_ns / per_node[1],
           "time_odds": per_node[1] / (per_node[1] - bridge_ns), "rows": rows}
json.dump(summary, open(out, "w"), indent=1)
print(json.dumps({k: v for k, v in summary.items() if k != "rows"}, indent=1))
