#!/usr/bin/env python3
"""Fixed-budget search probes over an EPD suite, for several UCI engines.

Built at PLAN B.0 (2026-09-13) and consumed by the B.2.2 screens and the B.9
checkpoint: a nominal ply is not a unit of work across engines, so tactical
quality and depth are compared at a fixed NODE budget, and canary anchors are
read off the per-depth PV of a single `go depth N`.

Modes:
  depthpv <maxdepth>  `go depth N` once per position; records the PV's first
                      move at every completed depth. With `bm` fields (WAC),
                      reports the first depth that answers correctly and the
                      depth from which the answer stays correct ("stable").
  nodes   <nodes>     `go nodes N`; records final depth, seldepth, nodes, ms,
                      and whether the bestmove is in `bm` when present.

Each engine gets Hash 64, Threads 1, and `ucinewgame` before every position.
An engine spec may carry further UCI options after `|`, as in
`core=rarog.exe|AblationMask=128`; an option the engine reports it does not
have stops the run, so a misspelt or compiled-out option cannot pass for a
measurement of the default.
Engines are driven with streaming stdin (write, read to `bestmove`, then
write again); a piped `printf ... | engine` aborts the search on every engine
this project uses. Output is one JSON file with per-position detail and a
summary, printed to stdout.

Usage:
  python tools/diag/fixed_budget_probe.py nodes 100000 src/wac.epd out.json       rarog=tools/test_engines/rarog-...exe oracle=tools/test_engines/oracle-hybrid-2.4.0/rarog-stockfish-hce-hybrid.exe
  python tools/diag/fixed_budget_probe.py depthpv 10 src/wac.epd out.json rarog=...
"""
import json
import os
import re
import subprocess
import sys
import time

import chess


def parse_epd(path):
    items = []
    for line in open(path, encoding="utf-8"):
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        fields = line.split(" ; ") if " ; " in line else [line]
        head = fields[0]
        parts = head.split()
        fen4 = " ".join(parts[:4])
        bm = []
        ident = None
        m = re.search(r"\bbm\s+([^;]+);", line)
        if m:
            bm = m.group(1).split()
        m = re.search(r'id\s+"([^"]+)"', line)
        if m:
            ident = m.group(1)
        if ident is None:
            ident = f"pos{len(items)+1}"
        fen = fen4 + " 0 1"
        board = chess.Board(fen)
        bm_uci = []
        for san in bm:
            try:
                bm_uci.append(board.parse_san(san).uci())
            except Exception:
                pass
        items.append({"id": ident, "fen": fen, "bm": bm_uci})
    return items

def parse_engine_spec(arg):
    """`label=path|Name=Value|...` -> (label, path, [(Name, Value), ...])."""
    label, rest = arg.split("=", 1)
    path, *pairs = rest.split("|")
    options = []
    for pair in pairs:
        if "=" not in pair:
            raise SystemExit(f"engine option {pair!r} in {arg!r} is not Name=Value")
        name, value = pair.split("=", 1)
        options.append((name, value))
    return label, path, options


def option_rejected(line):
    """True for an engine's report that a `setoption` named no option it has."""
    return "no such option" in line.lower()


class Engine:
    def __init__(self, path, options=()):
        path = os.path.abspath(path).replace("\\", "/")
        self.p = subprocess.Popen([path], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                  stderr=subprocess.STDOUT, text=True, bufsize=1,
                                  cwd=path.rsplit("/", 1)[0] if "/" in path else None)
        self.send("uci"); self.wait("uciok")
        self.send("setoption name Hash value 64")
        self.send("setoption name Threads value 1")
        for name, value in options:
            self.send(f"setoption name {name} value {value}")
        self.send("isready"); self.wait("readyok")
    def send(self, s):
        self.p.stdin.write(s + "\n"); self.p.stdin.flush()
    def wait(self, token):
        while True:
            line = self.p.stdout.readline()
            if not line:
                raise SystemExit("engine died")
            if option_rejected(line):
                raise SystemExit(f"engine rejected an option: {line.strip()}")
            if line.startswith(token):
                return
    def search(self, fen, go):
        self.send("ucinewgame"); self.send("isready"); self.wait("readyok")
        self.send(f"position fen {fen}")
        t0 = time.perf_counter()
        self.send(go)
        infos = []
        while True:
            line = self.p.stdout.readline()
            if not line:
                raise SystemExit("engine died")
            if line.startswith("info") and " depth " in line and " pv " in line:
                infos.append(line.strip())
            if line.startswith("bestmove"):
                best = line.split()[1]
                break
        return infos, best, time.perf_counter() - t0
    def quit(self):
        self.send("quit"); self.p.wait(timeout=10)

def info_fields(line):
    toks = line.split()
    d = {}
    for key in ("depth", "seldepth", "nodes", "time"):
        if key in toks:
            d[key] = int(toks[toks.index(key) + 1])
    if "pv" in toks:
        d["pv1"] = toks[toks.index("pv") + 1]
    if "multipv" in toks:
        d["multipv"] = int(toks[toks.index("multipv") + 1])
    return d

def last_completed(infos):
    """Fields of the last COMPLETED iteration's `info` line, or {}.

    An aspiration `lowerbound`/`upperbound` line belongs to an iteration still
    in progress when the budget ran out; reading it reports a depth the search
    never finished, which is what the engine's bound lines (printed since the
    UCI-info series) did to every node-budget depth read."""
    exact = [line for line in infos if " lowerbound " not in line and " upperbound " not in line]
    return info_fields(exact[-1]) if exact else {}


def run(mode, budget, suite, engines):
    items = parse_epd(suite)
    result = {"mode": mode, "budget": budget, "suite": suite, "engines": {}, "options": {}}
    for label, (path, options) in engines.items():
        result["options"][label] = dict(options)
        eng = Engine(path, options)
        per = {}
        for it in items:
            if mode == "depthpv":
                infos, best, secs = eng.search(it["fen"], f"go depth {budget}")
                by_depth = {}
                for line in infos:
                    f = info_fields(line)
                    if f.get("multipv", 1) != 1 or "pv1" not in f:
                        continue
                    by_depth[f["depth"]] = f["pv1"]  # last line per depth wins
                depths = sorted(by_depth)
                first = None
                stable = None
                if it["bm"]:
                    for d in depths:
                        if by_depth[d] in it["bm"]:
                            first = d
                            break
                    for d in depths:
                        if all(by_depth[e] in it["bm"] for e in depths if e >= d):
                            stable = d
                            break
                per[it["id"]] = {"by_depth": {str(d): by_depth[d] for d in depths},
                                 "bestmove": best, "bm": it["bm"], "first": first,
                                 "stable": stable, "secs": round(secs, 3)}
            else:
                infos, best, secs = eng.search(it["fen"], f"go nodes {budget}")
                last = last_completed(infos)
                per[it["id"]] = {"depth": last.get("depth"), "seldepth": last.get("seldepth"),
                                 "nodes": last.get("nodes"), "ms": last.get("time"),
                                 "bestmove": best, "secs": round(secs, 3),
                                 "bm": it["bm"], "solved": (best in it["bm"]) if it["bm"] else None}
            sys.stderr.write(f"{label} {it['id']}\n")
        eng.quit()
        result["engines"][label] = per

    summary = {}
    for label, per in result["engines"].items():
        if mode == "depthpv":
            solved_by = {}
            for d in range(1, budget + 1):
                solved_by[d] = sum(1 for v in per.values() if v["first"] is not None and v["first"] <= d)
            stable_by = {}
            for d in range(1, budget + 1):
                stable_by[d] = sum(1 for v in per.values() if v["stable"] is not None and v["stable"] <= d)
            summary[label] = {"positions": len(per), "solved_first_by_depth": solved_by,
                              "stable_by_depth": stable_by,
                              "secs": round(sum(v["secs"] for v in per.values()), 1)}
        else:
            ds = [v["depth"] for v in per.values() if v["depth"] is not None]
            summary[label] = {"positions": len(per), "mean_depth": round(sum(ds) / len(ds), 2),
                              "solved": sum(1 for v in per.values() if v.get("solved")),
                              "mean_seldepth": round(sum(v["seldepth"] for v in per.values()) / len(per), 2),
                              "total_nodes": sum(v["nodes"] for v in per.values()),
                              "total_ms": sum(v["ms"] for v in per.values())}
    result["summary"] = summary
    return result


def main(argv):
    if len(argv) < 5:
        raise SystemExit(__doc__)
    mode, budget, suite, out = argv[0], int(argv[1]), argv[2], argv[3]
    if mode not in ("depthpv", "nodes"):
        raise SystemExit(f"unknown mode {mode!r}; expected depthpv or nodes")
    engines = {}
    for arg in argv[4:]:
        label, path, options = parse_engine_spec(arg)
        engines[label] = (path, options)
    if not engines:
        raise SystemExit("no engines given (label=path ...)")
    result = run(mode, budget, suite, engines)
    json.dump(result, open(out, "w", encoding="utf-8"), indent=1)
    print(json.dumps(result["summary"], indent=1))


if __name__ == "__main__":
    main(sys.argv[1:])
