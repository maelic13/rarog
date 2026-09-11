"""Run the normalized cross-engine-board-v1 SEE comparison.

The runner refuses differing value vectors, move/verdict sets, work counts,
failed preflights, dirty exits, or excessive whole-host load during timed runs. It also proves
Rarog's injection wire with a deliberately absurd rook value before timing.
"""

import argparse
import ctypes
import datetime
import hashlib
import json
import os
from pathlib import Path
import re
import statistics
import subprocess
import time

VALUES = "100/300/300/500/900/20000"
ROW = re.compile(r"^(legal moves|legal captures|make/unmake|threshold SEE|perft\(4\) startpos|two-ply simulation)\s+(\d+)\s+(\d+)\s+([\d.]+)%\s+(\d+)\s+(\d+)")
EXPECTED_WORK = [128, 10, 128, 10, 197_281, 4_597]


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def field(text, name):
    matches = re.findall(rf"^{re.escape(name)}:\s*(.+)$", text, re.MULTILINE)
    if len(matches) != 1:
        raise ValueError(f"expected one {name!r} line, received {len(matches)}")
    return matches[0].strip()


def parse_output(text, require_rows):
    if "preflight: PASS" not in text:
        raise ValueError("missing successful preflight")
    values = field(text, "see-values")
    verdicts = field(text, "see-verdicts")
    verdict_items = verdicts.split(",")
    if len(verdict_items) != 10 or len(set(verdict_items)) != 10:
        raise ValueError("SEE verdict set is not ten unique move answers")
    rows = []
    for line in text.splitlines():
        match = ROW.match(line)
        if match:
            rows.append({
                "workload": match[1], "ops_per_sec": int(match[2]),
                "mad": int(match[3]), "mad_percent": float(match[4]),
                "ops_per_iter": int(match[5]), "iterations": int(match[6]),
            })
    if require_rows and (len(rows) != 6 or [row["ops_per_iter"] for row in rows] != EXPECTED_WORK):
        raise ValueError("missing/mismatched timed rows")
    return {"values": values, "verdicts": verdicts, "rows": rows}


def get_times():
    values = [ctypes.c_ulonglong() for _ in range(3)]
    if not ctypes.WinDLL("kernel32", use_last_error=True).GetSystemTimes(
            *(ctypes.byref(value) for value in values)):
        raise ctypes.WinError(ctypes.get_last_error())
    return [value.value for value in values]


def busy(before, after):
    delta = [end - start for start, end in zip(before, after)]
    return 100 * (delta[1] + delta[2] - delta[0]) / (delta[1] + delta[2])


def run(exe, args=()):
    before = get_times()
    started = time.monotonic()
    result = subprocess.run([str(exe), *args], capture_output=True, text=True,
                            timeout=90, creationflags=0x08000000)
    return result, time.monotonic() - started, busy(before, get_times())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--rarog", type=Path, required=True)
    parser.add_argument("--basilisk", type=Path, required=True)
    parser.add_argument("--reckless", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--rounds", type=int, default=3)
    parser.add_argument("--busy-limit", type=float, default=12.0)
    parser.add_argument("--affinity-mask", type=lambda value: int(value, 0), default=4)
    args = parser.parse_args()
    if args.rounds < 1:
        parser.error("--rounds must be positive")
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    engines = {name: path.resolve() for name, path in
               [("rarog", args.rarog), ("basilisk", args.basilisk), ("reckless", args.reckless)]}
    for path in engines.values():
        if not path.is_file():
            raise FileNotFoundError(path)
    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel.GetCurrentProcess.restype = ctypes.c_void_p
    kernel.SetProcessAffinityMask.argtypes = [ctypes.c_void_p, ctypes.c_size_t]
    kernel.SetProcessAffinityMask.restype = ctypes.c_int
    if not kernel.SetProcessAffinityMask(kernel.GetCurrentProcess(), args.affinity_mask):
        raise ctypes.WinError(ctypes.get_last_error())

    manifest = {
        "date": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "sampling": f"150ms warmup + 11x150ms; {args.rounds} cyclic rounds",
        "affinity_mask": args.affinity_mask, "busy_limit_percent": args.busy_limit,
        "binaries": {name: {"path": str(path), "sha256": digest(path)} for name, path in engines.items()},
        "preflights": {}, "runs": [],
    }
    verdicts = set()
    for name, exe in engines.items():
        result, elapsed, load = run(exe, ["--preflight-only"])
        (output / f"preflight-{name}.txt").write_text(result.stdout + result.stderr, encoding="utf-8")
        if result.returncode:
            raise RuntimeError(f"{name} preflight exited {result.returncode}")
        parsed = parse_output(result.stdout, False)
        if parsed["values"] != VALUES:
            raise ValueError(f"{name} used {parsed['values']}, expected {VALUES}")
        verdicts.add(parsed["verdicts"])
        manifest["preflights"][name] = {**parsed, "elapsed": elapsed, "host_busy_percent": load}
    if len(verdicts) != 1:
        raise ValueError("engines disagree on normalized SEE verdicts")

    absurd = "100,300,300,1,900,20000"
    result, elapsed, load = run(engines["rarog"], ["--preflight-only", "--see-values", absurd])
    (output / "preflight-rarog-absurd.txt").write_text(result.stdout + result.stderr, encoding="utf-8")
    if result.returncode:
        raise RuntimeError(f"Rarog absurd-value preflight exited {result.returncode}")
    normal_probe = field((output / "preflight-rarog.txt").read_text(), "see-probe")
    absurd_probe = field(result.stdout, "see-probe")
    if (normal_probe, absurd_probe) != ("false", "true"):
        raise ValueError(f"dead Rarog value wire: normal={normal_probe}, absurd={absurd_probe}")
    manifest["wire_proof"] = {"values": absurd, "normal_probe": normal_probe,
                              "absurd_probe": absurd_probe, "exit": result.returncode,
                              "elapsed": elapsed, "host_busy_percent": load}
    (output / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")

    names = list(engines)
    for round_index in range(args.rounds):
        order = names[round_index % len(names):] + names[:round_index % len(names)]
        for name in order:
            result, elapsed, load = run(engines[name])
            raw = result.stdout + result.stderr
            (output / f"round{round_index + 1}-{name}.txt").write_text(raw, encoding="utf-8")
            if result.returncode:
                raise RuntimeError(f"{name} round {round_index + 1} exited {result.returncode}")
            parsed = parse_output(result.stdout, True)
            if parsed["values"] != VALUES or parsed["verdicts"] not in verdicts:
                raise ValueError(f"{name} changed normalized SEE contract during timing")
            entry = {"round": round_index + 1, "engine": name, "elapsed": elapsed,
                     "host_busy_percent": load, "rows": parsed["rows"]}
            manifest["runs"].append(entry)
            (output / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
            see = next(row for row in parsed["rows"] if row["workload"] == "threshold SEE")
            print(f"round {round_index + 1} {name}: SEE={see['ops_per_sec']/1e6:.3f}M busy={load:.2f}%", flush=True)
            if load > args.busy_limit:
                raise RuntimeError(f"contaminated run: {name} host busy {load:.2f}%")

    summary = {}
    for name in names:
        rows = [next(row for row in run_["rows"] if row["workload"] == "threshold SEE")
                for run_ in manifest["runs"] if run_["engine"] == name]
        rates = [row["ops_per_sec"] for row in rows]
        summary[name] = {
            "median_ops_per_sec": statistics.median(rates), "round_medians": rates,
            "round_range_percent": 100 * (max(rates) - min(rates)) / statistics.median(rates),
            "max_within_run_mad_percent": max(row["mad_percent"] for row in rows),
        }
    manifest["summary"] = summary
    (output / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
