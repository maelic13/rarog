"""D.7.3 layer B: tree statistics of one engine configuration on fixed positions.

  tree_stats.py positions <match.pgn> <out.jsonl> [count] [seed]
      The position after every 20th ply of every game, deduplicated by FEN,
      sampled to <count> (500) with <seed> (20261116), file order kept.
  tree_stats.py run <engine.exe> <positions.jsonl> <out.jsonl> [KEY=VALUE ...]
      A fresh process a position, Hash 64, one thread, `go depth 14`; records
      the positions visited at depths 8, 11 and 14 (the engine's own count:
      since D.6.31 Rarog counts as the donor does, once per move made) and the
      twin's `info string rates` (each rule's firings and attempts). KEY=VALUE
      pairs are set in the engine's environment (RAROG_SPREAD_SALT, RAROG_PLANT).
  tree_stats.py compare <base.jsonl> <candidate.jsonl> --floor <salt1.jsonl> ... [--json out]
      The frozen read (search_shape_inventory_2026-11-13.md section 7): at each
      depth the mean log ratio of positions visited must lie within three
      standard errors of the noise floor plus 0.03; each kind's firings per
      1,000 positions within three standard errors or 10%, whichever is larger.
      The floor at a depth is the largest |mean log ratio| of a salted run
      against the base.
"""
import json
import math
import os
import random
import subprocess
import sys
import threading
import time
from concurrent.futures import ThreadPoolExecutor

DEPTHS = (8, 11, 14)
ALLOWANCE = 0.03
LIMIT_S = float(os.environ.get("TREE_STATS_LIMIT_S", "120"))


def positions(pgn_path, out_path, count=500, seed=20261116):
    import chess.pgn
    seen, found = set(), []
    with open(pgn_path, encoding="utf-8", errors="replace") as f:
        while True:
            game = chess.pgn.read_game(f)
            if game is None:
                break
            board = game.board()
            for ply, move in enumerate(game.mainline_moves(), start=1):
                board.push(move)
                if ply % 20 == 0 and not board.is_game_over():
                    fen = board.fen()
                    if fen not in seen:
                        seen.add(fen)
                        found.append(fen)
    rng = random.Random(seed)
    keep = sorted(rng.sample(range(len(found)), min(count, len(found))))
    with open(out_path, "w", newline="\n") as f:
        for i, k in enumerate(keep):
            f.write(json.dumps({"id": i, "fen": found[k]}) + "\n")
    print(f"{len(found)} candidate positions, {len(keep)} written")


def search_one(exe, fen, env):
    proc = subprocess.Popen([exe], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
                            text=True, bufsize=1, env=env)
    proc.stdin.write("uci\nsetoption name Hash value 64\nsetoption name Threads value 1\nisready\n"
                     f"position fen {fen}\ngo depth {DEPTHS[-1]}\n")
    proc.stdin.flush()
    nodes, rates = {}, {}
    timed_out = threading.Event()

    def watchdog():
        if not done.wait(LIMIT_S):
            timed_out.set()
            try:
                proc.stdin.write("stop\n")
                proc.stdin.flush()
            except OSError:
                pass

    done = threading.Event()
    threading.Thread(target=watchdog, daemon=True).start()
    for line in proc.stdout:
        parts = line.split()
        if line.startswith("info string rates"):
            for item in parts[3:]:
                kind, value = item.split("=")
                fires, attempts = value.split("/")
                rates[kind] = [int(fires), int(attempts)]
        elif line.startswith("info") and "depth" in parts and "nodes" in parts and "seldepth" in parts:
            d = int(parts[parts.index("depth") + 1])
            nodes[d] = int(parts[parts.index("nodes") + 1])
        if line.startswith("bestmove"):
            break
    done.set()
    proc.stdin.write("quit\n")
    proc.stdin.flush()
    proc.wait(timeout=30)
    if timed_out.is_set():
        return {"nodes": {str(d): nodes.get(d) for d in DEPTHS[:-1]} | {str(DEPTHS[-1]): None},
                "rates": {}, "timeout": True}
    return {"nodes": {str(d): nodes.get(d) for d in DEPTHS}, "rates": rates, "timeout": False}


def run(exe, positions_path, out_path, pairs):
    env = dict(os.environ)
    for k in ("RAROG_TRACE", "RAROG_SPREAD_SALT", "RAROG_PLANT"):
        env.pop(k, None)
    for pair in pairs:
        k, v = pair.split("=", 1)
        env[k] = v
    pos = [json.loads(l) for l in open(positions_path)]
    with ThreadPoolExecutor(14) as ex:
        rows = list(ex.map(lambda p: {"id": p["id"], **search_one(exe, p["fen"], env)}, pos))
    with open(out_path, "w", newline="\n") as f:
        for r in rows:
            f.write(json.dumps(r) + "\n")
    missing = sum(any(v is None for v in r["nodes"].values()) for r in rows)
    timeouts = [r["id"] for r in rows if r.get("timeout")]
    print(f"{len(rows)} positions, {missing} without every depth, {len(timeouts)} timed out "
          f"{timeouts[:20]}; config {pairs or ['base']}")


def load(path):
    return {r["id"]: r for r in (json.loads(l) for l in open(path))}


def log_ratio_stats(base, cand, depth):
    ids = [i for i in base if i in cand and base[i]["nodes"][depth] and cand[i]["nodes"][depth]]
    r = [math.log(cand[i]["nodes"][depth] / base[i]["nodes"][depth]) for i in ids]
    m = sum(r) / len(r)
    sd = math.sqrt(sum((x - m) ** 2 for x in r) / (len(r) - 1))
    return m, sd / math.sqrt(len(r)), len(r)


def rate_stats(base, cand, kind):
    ids = [i for i in base if i in cand and not base[i].get("timeout") and not cand[i].get("timeout")]
    fb = [base[i]["rates"].get(kind, [0, 0])[0] for i in ids]
    fc = [cand[i]["rates"].get(kind, [0, 0])[0] for i in ids]
    d = [c - b for b, c in zip(fb, fc)]
    n = len(d)
    md = sum(d) / n
    sd = math.sqrt(sum((x - md) ** 2 for x in d) / (n - 1)) if n > 1 else 0.0
    base_mean = sum(fb) / n
    return 1000 * base_mean, 1000 * (sum(fc) / n), 1000 * md, 1000 * sd / math.sqrt(n)


def compare(base_path, cand_path, floor_paths, json_out=None):
    base, cand = load(base_path), load(cand_path)
    floors = [load(p) for p in floor_paths]
    out = {"depths": {}, "rates": {}, "pass": True}
    for d in map(str, DEPTHS):
        floor = max(abs(log_ratio_stats(base, f, d)[0]) for f in floors) if floors else 0.0
        m, se, n = log_ratio_stats(base, cand, d)
        ok = abs(m) <= floor + ALLOWANCE + 3 * se
        out["depths"][d] = {"mean_log_ratio": m, "se": se, "n": n, "floor": floor, "pass": ok}
        out["pass"] &= ok
    kinds = sorted(set().union(*(r["rates"].keys() for r in base.values())))
    for k in kinds:
        bm, cm, md, se = rate_stats(base, cand, k)
        ok = abs(md) <= max(3 * se, 0.10 * abs(bm))
        out["rates"][k] = {"base_per_1000": bm, "cand_per_1000": cm, "diff": md, "se": se, "pass": ok}
        out["pass"] &= ok
    for d, v in out["depths"].items():
        print(f"depth {d}: mean log ratio {v['mean_log_ratio']:+.4f} ± {v['se']:.4f} (floor {v['floor']:.4f}) "
              f"{'pass' if v['pass'] else 'FAIL'}")
    failed = [k for k, v in out["rates"].items() if not v["pass"]]
    print(f"rates: {len(kinds) - len(failed)} of {len(kinds)} kinds pass" + (f"; failed: {', '.join(failed)}" if failed else ""))
    print("PASS" if out["pass"] else "FAIL")
    if json_out:
        with open(json_out, "w", newline="\n") as f:
            json.dump(out, f, indent=1)
    return out


if __name__ == "__main__":
    cmd = sys.argv[1]
    if cmd == "positions":
        positions(sys.argv[2], sys.argv[3], *(int(x) for x in sys.argv[4:6]))
    elif cmd == "run":
        run(sys.argv[2], sys.argv[3], sys.argv[4], sys.argv[5:])
    elif cmd == "compare":
        args = sys.argv[2:]
        floor_paths, json_out, rest = [], None, []
        i = 0
        while i < len(args):
            if args[i] == "--floor":
                i += 1
                while i < len(args) and not args[i].startswith("--"):
                    floor_paths.append(args[i])
                    i += 1
                continue
            if args[i] == "--json":
                json_out = args[i + 1]
                i += 2
                continue
            rest.append(args[i])
            i += 1
        sys.exit(0 if compare(rest[0], rest[1], floor_paths, json_out)["pass"] else 1)
