#!/usr/bin/env python3
"""Read the clock's soft-stop signals over B.5's replay: which factors move.

D.1.4's activation read (PLAN D.1.4 (b)). A `diag` build prints one
`info string tm` line per completed iteration of a clock search: the
iterations since the best move last changed (`k`), the best move's share of
the search's nodes, and the falling-eval, instability, stability and effort
factors. This drives such a build over B.5's replay (300 positions of the UHO
book at an even stride, a fresh game each, `3000+30` on both clocks, Hash 64,
one thread) and reports, per factor, the share of iterations at each end of
its range. A factor at one end on more than 90% of iterations is inert: it
tunes as a constant, and its coordinate leaves the fit (RAR-S47's lesson). It
also sums the clock-stop counters (`tm_mate_stop`, `tm_fail_high_stop`,
`tm_first_iteration_guard`), reads each search's spend from its last `info
depth` line, and, where a search ended on the fail-high stop (its last line a
`lowerbound`), how far past the last completed iteration's soft target the
stop fired.

Timing-dependent: run on an idle host. The ends are the seeds unless --set
names another value, which is also sent to the engine (a `tune,diag` build).

  python tools/diag/tm_activation.py --engine target/release/rarog.exe \\
      [--positions 300] [--set TmStabFloor=7000] [--json out.json]
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BOOK = ROOT / "tools" / "books" / "UHO_Lichess_4852_v1.epd"
TM_LINE = re.compile(r"^info string tm (.*)$")
DIAG_LINE = re.compile(r"^info string diag (\w+) (\d+)$")
INFO_LINE = re.compile(r"^info depth \d+ .*? score \S+ \S+( lowerbound| upperbound)? .* time (\d+) ")
COUNTERS = ("tm_mate_stop", "tm_fail_high_stop", "tm_first_iteration_guard", "root_iterations")
# The seeds, in the engine's units (ten-thousandths).
SEEDS = {
    "TmStabBase": 12_881,
    "TmStabFloor": 7_160,
    "TmEffortHigh": 9_240,
    "TmEffortLow": 7_100,
    "TmInstabBase": 11_000,
}
FALL_RANGE = (0.572, 1.708)
INERT_SHARE = 0.90
# The log prints four decimals.
TOLERANCE = 6e-5


def parse_tm_line(line: str) -> dict[str, float] | None:
    """One `info string tm` line as a dict of numbers, or None."""
    match = TM_LINE.match(line.strip())
    if not match:
        return None
    words = match.group(1).split()
    if len(words) % 2:
        raise ValueError(f"unpaired field in {line!r}")
    return {words[i]: float(words[i + 1]) for i in range(0, len(words), 2)}


def book_positions(count: int) -> list[str]:
    """B.5's replay positions: the book at an even stride, as `b5_interrupt.py` chose them."""
    fens = []
    for line in BOOK.read_text(encoding="utf-8").splitlines():
        parts = line.split()
        if len(parts) >= 4:
            fens.append(" ".join(parts[:4]) + " 0 1")
    step = max(1, len(fens) // count)
    return fens[::step][:count]


def at(value: float, end: float) -> bool:
    return abs(value - end) < TOLERANCE


def summarise(records: list[dict[str, float]], params: dict[str, int]) -> dict:
    """Per factor, the share of iterations at each end of its range."""
    n = len(records)
    if n == 0:
        raise ValueError("no iterations to summarise")
    unit = {name: value / 10_000 for name, value in params.items()}
    ends = {
        "stab": ("k = 0 (TmStabBase)", lambda r: r["k"] == 0,
                 "TmStabFloor", lambda r: at(r["stab"], unit["TmStabFloor"])),
        "effort": ("TmEffortHigh", lambda r: at(r["effort"], unit["TmEffortHigh"]),
                   "TmEffortLow", lambda r: at(r["effort"], unit["TmEffortLow"])),
        "fall": (f"{FALL_RANGE[0]}", lambda r: at(r["fall"], FALL_RANGE[0]),
                 f"{FALL_RANGE[1]}", lambda r: at(r["fall"], FALL_RANGE[1])),
        "instab": ("TmInstabBase", lambda r: at(r["instab"], unit["TmInstabBase"]),
                   "none", lambda r: False),
    }
    factors = {}
    for name, (low_name, low, high_name, high) in ends.items():
        low_share = sum(1 for r in records if low(r)) / n
        high_share = sum(1 for r in records if high(r)) / n
        factors[name] = {
            "low_end": low_name, "low_share": low_share,
            "high_end": high_name, "high_share": high_share,
            "inert": max(low_share, high_share) > INERT_SHARE,
        }
    k_hist: dict[str, int] = {}
    for r in records:
        key = str(int(r["k"])) if r["k"] < 10 else "10+"
        k_hist[key] = k_hist.get(key, 0) + 1
    shares = sorted(r["share"] for r in records)
    quantiles = {f"p{int(q * 100)}": shares[min(n - 1, round(q * (n - 1)))]
                 for q in (0.1, 0.25, 0.5, 0.75, 0.9)}
    return {"iterations": n, "factors": factors, "k_histogram": k_hist, "share_quantiles": quantiles}


def run(engine: str, fens: list[str], base: int, inc: int, sets: dict[str, int]) -> tuple[list, dict]:
    process = subprocess.Popen([engine], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, bufsize=1)

    def send(line: str) -> None:
        process.stdin.write(line + "\n")
        process.stdin.flush()

    send("uci")
    for line in process.stdout:
        if line.startswith("uciok"):
            break
    send("setoption name Hash value 64")
    send("setoption name Threads value 1")
    for name, value in sets.items():
        send(f"setoption name {name} value {value}")
    records, counters = [], {name: 0 for name in COUNTERS}
    # Per search: the spend (its last info line's time) and, for a search
    # ended by the fail-high stop, that time over the last soft target.
    spends: list[float] = []
    stop_ratios: list[float] = []
    # The last completed iteration of each search: the factors the ending
    # soft target was built from.
    last_records: list[dict[str, float]] = []
    for fen in fens:
        send("ucinewgame")
        send("position fen " + fen)
        send(f"go wtime {base} btime {base} winc {inc} binc {inc}")
        last_target = None
        last_info = None
        for line in process.stdout:
            if line.startswith("bestmove"):
                break
            record = parse_tm_line(line)
            if record is not None:
                records.append(record)
                last_target = record.get("target")
                continue
            info = INFO_LINE.match(line.strip())
            if info:
                last_info = (info.group(1) or "", float(info.group(2)))
                continue
            match = DIAG_LINE.match(line.strip())
            if match and match.group(1) in counters:
                counters[match.group(1)] += int(match.group(2))
        if not records:
            send("quit")
            raise SystemExit("no `info string tm` line after the first search: not a diag build")
        if last_info is not None:
            spends.append(last_info[1])
            if last_info[0] == " lowerbound" and last_target:
                stop_ratios.append(last_info[1] / last_target)
        last_records.append(records[-1])
    send("quit")
    process.wait(timeout=30)
    counters["searches"] = len(fens)
    counters["mean_spend_ms"] = round(sum(spends) / max(1, len(spends)), 2)
    counters["stop_ratios"] = stop_ratios
    counters["last_records"] = last_records
    return records, counters


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--engine", required=True)
    parser.add_argument("--positions", type=int, default=300)
    parser.add_argument("--base", type=int, default=3000)
    parser.add_argument("--inc", type=int, default=30)
    parser.add_argument("--set", action="append", default=[], metavar="NAME=VALUE")
    parser.add_argument("--json")
    args = parser.parse_args(argv)
    sets = {}
    for item in args.set:
        name, value = item.split("=", 1)
        sets[name] = int(value)
    params = {**SEEDS, **{k: v for k, v in sets.items() if k in SEEDS}}
    fens = book_positions(args.positions)
    records, counters = run(args.engine, fens, args.base, args.inc, sets)
    summary = summarise(records, params)
    summary.update({"positions": len(fens), "clock": f"{args.base}+{args.inc}", "counters": counters,
                    "set": sets})
    print(f"{len(fens)} positions at {args.base}+{args.inc}: {summary['iterations']} completed iterations")
    for name, f in summary["factors"].items():
        flag = "  INERT" if f["inert"] else ""
        print(f"  {name:7s} at {f['low_end']}: {f['low_share']:6.1%}   at {f['high_end']}: "
              f"{f['high_share']:6.1%}{flag}")
    print("  k: " + ", ".join(f"{k}={v}" for k, v in sorted(summary["k_histogram"].items(),
                                                            key=lambda kv: (len(kv[0]), kv[0]))))
    print("  best-move node share: " + ", ".join(f"{q}={v:.3f}" for q, v in summary["share_quantiles"].items()))
    ratios = sorted(counters.pop("stop_ratios"))
    last = counters.pop("last_records")
    mean = lambda key, rows: sum(r[key] for r in rows) / max(1, len(rows))
    factor_means = {key: (round(mean(key, records), 4), round(mean(key, last), 4))
                    for key in ("fall", "instab", "stab", "effort")}
    print("  counters: " + ", ".join(f"{k}={v}" for k, v in counters.items()))
    print("  mean factor, all iterations / last completed iteration: "
          + ", ".join(f"{k} {a:.3f}/{b:.3f}" for k, (a, b) in factor_means.items()))
    summary["factor_means"] = factor_means
    if ratios:
        n = len(ratios)
        q = lambda p: ratios[min(n - 1, int(p * n))]
        shares = ", ".join(f">={t:g}x {sum(1 for r in ratios if r >= t) / n:.1%}" for t in (1.5, 2, 3))
        print(f"  fail-high stops: {n} of {counters['searches']} searches ended on one; elapsed over the last "
              f"target p10={q(0.1):.2f} p50={q(0.5):.2f} p90={q(0.9):.2f}; {shares}")
    summary["stops"] = {"count": len(ratios), "ratios": ratios}
    if args.json:
        Path(args.json).write_text(json.dumps(summary, indent=1) + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    sys.exit(main())
