"""Move-matching similarity test between UCI engines, in the manner of Don
Dailey's `sim`: every engine chooses a move in the same positions at a fixed
movetime, and each pair is scored by the share of positions where both chose
the same move. Unrelated engines agree on many moves too, so a pair is read
against the matrix's own unrelated pairs, never against a fixed number.

Each engine runs single-threaded in its own process; engines run in parallel,
one per worker. Answers are appended per engine, so an interrupted run resumes.

usage:
  python tools/diag/sim_test.py --epd <file> --positions 2000 --movetime 100
      --out <dir> --engine "rarog=D:/chess/engines/rarog/rarog.exe"
      --engine "sf11=D:/.../stockfish-11.exe|Move Overhead=10" ...
"""
import argparse
import os
import subprocess
import sys
import threading
from concurrent.futures import ThreadPoolExecutor


def load_positions(path, count):
    with open(path, encoding="utf-8") as f:
        fens = []
        for line in f:
            fields = line.split()
            if len(fields) >= 4:
                # EPD carries four FEN fields; complete the move counters.
                fens.append(" ".join(fields[:6]) if len(fields) >= 6 and fields[4].isdigit() else " ".join(fields[:4]) + " 0 1")
    if count >= len(fens):
        return fens
    stride = len(fens) / count
    return [fens[int(i * stride)] for i in range(count)]


class Uci:
    def __init__(self, path, options):
        self.p = subprocess.Popen([path], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                  stderr=subprocess.DEVNULL, text=True, bufsize=1,
                                  cwd=os.path.dirname(path) or None)
        self.send("uci")
        self.wait("uciok")
        for name, value in options:
            self.send(f"setoption name {name} value {value}")
        self.ready()

    def send(self, line):
        self.p.stdin.write(line + "\n")
        self.p.stdin.flush()

    def wait(self, prefix):
        while True:
            line = self.p.stdout.readline()
            if not line:
                raise RuntimeError("engine closed its output")
            if line.startswith(prefix):
                return line.strip()

    def ready(self):
        self.send("isready")
        self.wait("readyok")

    def best(self, fen, movetime):
        self.send("ucinewgame")
        self.ready()
        self.send(f"position fen {fen}")
        self.send(f"go movetime {movetime}")
        return self.wait("bestmove").split()[1]

    def close(self):
        try:
            self.send("quit")
            self.p.wait(timeout=5)
        except Exception:
            self.p.kill()


def run_engine(name, spec, fens, movetime, out_dir, lock):
    parts = spec.split("|")
    options = [("Threads", "1"), ("Hash", "16")]
    for extra in parts[1:]:
        k, v = extra.split("=", 1)
        options = [o for o in options if o[0] != k] + [(k, v)]
    out = os.path.join(out_dir, name + ".moves")
    done = 0
    if os.path.exists(out):
        with open(out, encoding="utf-8") as f:
            done = sum(1 for _ in f)
    if done >= len(fens):
        return name, done
    eng = Uci(parts[0], options)
    try:
        with open(out, "a", encoding="utf-8") as f:
            for i in range(done, len(fens)):
                f.write(eng.best(fens[i], movetime) + "\n")
                f.flush()
                if (i + 1) % 250 == 0:
                    with lock:
                        print(f"{name}: {i + 1}/{len(fens)}", flush=True)
    finally:
        eng.close()
    return name, len(fens)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--epd", required=True)
    ap.add_argument("--positions", type=int, default=2000)
    ap.add_argument("--movetime", type=int, default=100)
    ap.add_argument("--out", required=True)
    ap.add_argument("--jobs", type=int, default=0)
    ap.add_argument("--engine", action="append", required=True, help="name=path[|Option=value...]")
    a = ap.parse_args()

    fens = load_positions(a.epd, a.positions)
    os.makedirs(a.out, exist_ok=True)
    with open(os.path.join(a.out, "positions.fen"), "w", encoding="utf-8") as f:
        f.write("\n".join(fens) + "\n")
    engines = dict(e.split("=", 1) for e in a.engine)
    for name, spec in engines.items():
        if not os.path.isfile(spec.split("|")[0]):
            sys.exit(f"missing engine binary for {name}: {spec.split('|')[0]}")
    lock = threading.Lock()
    with ThreadPoolExecutor(max_workers=a.jobs or len(engines)) as pool:
        for name, n in pool.map(lambda kv: run_engine(kv[0], kv[1], fens, a.movetime, a.out, lock), engines.items()):
            print(f"{name}: complete ({n})", flush=True)

    moves = {}
    for name in engines:
        with open(os.path.join(a.out, name + ".moves"), encoding="utf-8") as f:
            moves[name] = [l.strip() for l in f][: len(fens)]
        if len(moves[name]) != len(fens):
            sys.exit(f"{name}: {len(moves[name])} answers for {len(fens)} positions")
    names = list(engines)
    w = max(len(n) for n in names) + 1
    lines = [" " * w + "".join(f"{n[:8]:>9}" for n in names)]
    tsv = ["engine\t" + "\t".join(names)]
    for x in names:
        row = []
        for y in names:
            same = sum(p == q for p, q in zip(moves[x], moves[y]))
            row.append(100.0 * same / len(fens))
        lines.append(f"{x:<{w}}" + "".join(f"{v:9.1f}" for v in row))
        tsv.append(x + "\t" + "\t".join(f"{v:.2f}" for v in row))
    report = "\n".join(lines)
    print(f"\nsame-move % over {len(fens)} positions at {a.movetime} ms\n" + report)
    with open(os.path.join(a.out, "matrix.tsv"), "w", encoding="utf-8") as f:
        f.write("\n".join(tsv) + "\n")
    with open(os.path.join(a.out, "matrix.txt"), "w", encoding="utf-8") as f:
        f.write(report + "\n")


if __name__ == "__main__":
    main()
