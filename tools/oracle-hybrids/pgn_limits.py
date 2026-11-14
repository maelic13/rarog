"""Per-engine search limits as a fastchess PGN records them: games and their
terminations, mean and median thinking time a move, and, where the PGN carries `n=`
(sprt.ps1 writes it in node-budget matches), the largest reported node count
and the moves above a budget plus a tolerance.

    python -I pgn_limits.py <games.pgn> <engine>[=<budget>[:<tol>]] <engine>[=<budget>[:<tol>]] [--tolerance 1]

Prints one line per engine and the ratios of the second named engine's mean
and median time a move to the first's; a budget turns on the node check for
that engine, which flags moves above the budget plus its tolerance (`:<tol>`,
else `--tolerance`). Exit status 1 if the
PGN has no games or a game lacks a Termination tag.
"""
import re
import statistics
import sys
from pathlib import Path

MOVE = re.compile(r"\{[^{}/]*/\d+ (\d+(?:\.\d+)?)s(?:, n=(\d+))?[^{}]*\}")


def tag(head: str, name: str) -> str | None:
    m = re.search(r'\[' + name + r' "([^"]*)"\]', head)
    return m.group(1) if m else None


def main() -> int:
    args = sys.argv[1:]
    tolerance = 1
    if "--tolerance" in args:
        i = args.index("--tolerance")
        tolerance = int(args[i + 1])
        del args[i : i + 2]
    path = Path(args[0])
    named = [a.split("=", 1)[0] for a in args[1:]]
    budgets: dict[str, int] = {}
    tolerances: dict[str, int] = {}
    for k, v in (a.split("=", 1) for a in args[1:] if "=" in a):
        budget, _, tol = v.partition(":")
        budgets[k] = int(budget)
        tolerances[k] = int(tol) if tol else tolerance
    text = path.read_text(encoding="utf-8", errors="replace")
    blocks = text.split("[Event ")[1:]
    if not blocks:
        print("no games")
        return 1
    terms: dict[str, int] = {}
    side: dict[str, dict] = {}
    order: list[str] = []
    for block in blocks:
        head, _, body = block.partition("\n\n")
        term = tag(head, "Termination")
        if term is None:
            print("a game has no Termination tag")
            return 1
        terms[term] = terms.get(term, 0) + 1
        names = {True: tag(head, "White"), False: tag(head, "Black")}
        fen = re.search(r'\[FEN "[^ ]+ ([wb]) ', head)
        white = (fen.group(1) == "w") if fen else True
        for m in MOVE.finditer(body):
            name = names[white]
            if name not in side:
                side[name] = {"moves": 0, "time": 0.0, "times": [], "nodes_seen": 0, "max_n": 0, "over": 0}
                order.append(name)
            s = side[name]
            s["moves"] += 1
            s["time"] += float(m.group(1))
            s["times"].append(float(m.group(1)))
            if m.group(2) is not None:
                n = int(m.group(2))
                s["nodes_seen"] += 1
                s["max_n"] = max(s["max_n"], n)
                if name in budgets and n > budgets[name] + tolerances[name]:
                    s["over"] += 1
            white = not white
    print(f"{path.name}: {len(blocks)} games, terminations {terms}")
    for name in order:
        s = side[name]
        line = (f"  {name:20s} moves {s['moves']:>7d}  mean {1000 * s['time'] / s['moves']:7.2f} ms"
                f"  median {1000 * statistics.median(s['times']):6.1f} ms")
        if s["nodes_seen"]:
            line += f"  n= on {s['nodes_seen']} moves, max {s['max_n']}"
            if name in budgets:
                line += f", above {budgets[name]}+{tolerances[name]}: {s['over']}"
        print(line)
    if len(named) == 2 and all(n in side for n in named):
        a, b = (side[n] for n in named)
        ratio = (b["time"] / b["moves"]) / (a["time"] / a["moves"])
        median_ratio = statistics.median(b["times"]) / statistics.median(a["times"])
        print(f"  ratio {named[1]} / {named[0]}: mean {ratio:.4f}, median {median_ratio:.4f}")
    elif named:
        print(f"  no ratio: named {named}, PGN engines {order}")
        return 1
    return 0


sys.exit(main())
