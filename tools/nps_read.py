#!/usr/bin/env python3
"""The two-step pooled-PGO NPS read (PROCESS, *Harness notes*; RAR-P32).

Two pools of PGO builds of one source each (nps_build_pool.ps1, four builds,
manifest-pext.txt), read in interleaved cycles of `bench 13 3`. A reading is
the best of its three runs; every run is archived in the output directory as
`raw.json`, so any number here can be recomputed.

    python tools/nps_read.py --base <pool> --cand <pool> --out <dir> --label "..."
    python tools/nps_read.py --extend <dir>          # step 2: four more cycles
    python tools/nps_read.py --report <dir>          # recompute from raw.json

Step 1 is two cycles (about six minutes): at +0.9% or more the change is
accepted, at +0.1% or less it closes; between, `--extend` adds four cycles
and the +0.5% rule with the lower bound above 0 decides. `--no-regression`
mirrors the thresholds around -0.5% for a change that should cost nothing.

Estimate: the difference of the arms' means of per-build medians (median over
cycles of the best-of-3 reading). Interval: t on the per-build medians, pooled
variance, 2B-2 degrees of freedom. Disturbance: in step 1 an arm's two cycles
must agree within 1%; over six cycles a cycle more than 1% from its arm's
median cycle is disturbed, and three or more mean the read is repeated.

The host must be idle before step 1 and before `--extend`: no engine,
harness or build process and at most 15% CPU, read by the harness's own
measure (`harness_common.ps1`), the one `colosseum.ps1` refuses on.
`--allow-busy-host` waives it and the waiver is archived with the read.

Every build is checked against its manifest hash before and after the read,
every run must print its own pool's fingerprint (from that pool's manifest, so a
release baseline can be read against the head), and the pools must be distinct bytes.

An arm may instead differ by UCI options on a tune build (`--cand-option
LazyMargin=2000`, sent as `setoption` before the bench). Then one pool may serve
both arms, which cancels each build's PGO offset (the unpaired interval below is
conservative for that design), the options must differ, and an option that
changes the search names that arm's fingerprint (`--cand-fingerprint`).

Plain builds (`nps_build_pool.ps1 -NoPgo`) are deterministic, so each arm is
one binary and the spread is between cycles: the interval is t on the
per-cycle ratios of the paired readings, cycles − 1 degrees of freedom; run
it with `--cycles 10`. It is an estimate for a screen, with no verdict, and
covers run noise only: the one code layout each compiler produced is not
averaged.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import re
import shutil
import statistics
import subprocess
import sys
import time
from pathlib import Path

RUN = re.compile(r"^run (\d)/3\s+nodes (\d+)\s+time (\d+)ms\s+nps (\d+)", re.M)
MANIFEST_ROW = re.compile(r"^([0-9A-F]{64})\s+(\S+\.exe)$", re.M)
MANIFEST_FINGERPRINT = re.compile(r"^fingerprint\s*:\s*(\d+)\b", re.M)
MANIFEST_PLAIN = re.compile(r"^pgo\s*:\s*no \(deterministic", re.M)
# Two-sided 95% t quantiles by degrees of freedom; 6 is the four-build read,
# 5 a three-build pool against a four-build one.
T95 = {2: 4.303, 3: 3.182, 4: 2.776, 5: 2.571, 6: 2.447, 7: 2.365, 8: 2.306, 9: 2.262, 10: 2.228,
       12: 2.179, 14: 2.145, 16: 2.120, 20: 2.086, 30: 2.042}
HARNESS = Path(__file__).resolve().parent / "harness_common.ps1"
MAX_BUSY_PERCENT = 15.0  # colosseum.ps1's default ceiling


def pool_fingerprint(directory: Path) -> int:
    """The bench fingerprint the pool's manifest verified on every copy. Each
    arm is checked against its own, so a pool of an earlier source (a release
    baseline) can be read against the head's."""
    manifest = directory / "manifest-pext.txt"
    m = MANIFEST_FINGERPRINT.search(manifest.read_text(encoding="utf-8"))
    if not m:
        raise SystemExit(f"{manifest}: records no fingerprint")
    return int(m.group(1))


def pool(directory: Path) -> list[Path]:
    manifest = directory / "manifest-pext.txt"
    text = manifest.read_text(encoding="utf-8")
    rows = MANIFEST_ROW.findall(text)
    if not rows:
        raise SystemExit(f"{manifest}: no build rows")
    if MANIFEST_PLAIN.search(text):
        if len(rows) != 1:
            raise SystemExit(f"{manifest}: a deterministic plain arm is one binary")
    elif len(rows) < 2:
        raise SystemExit(f"{manifest}: one build gives no spread; a pool needs at least two")
    pool_fingerprint(directory)
    builds = []
    for digest, name in rows:
        exe = directory / name
        if hashlib.sha256(exe.read_bytes()).hexdigest().upper() != digest:
            raise SystemExit(f"{exe}: does not match its manifest")
        builds.append(exe)
    return builds


def parse_runs(output: str, exe: str, fingerprint: int) -> list[dict]:
    runs = [{"nodes": int(n), "ms": int(ms), "nps": int(nps)} for _, n, ms, nps in RUN.findall(output)]
    if len(runs) != 3:
        raise SystemExit(f"{exe}: expected three runs, got {len(runs)}")
    for run in runs:
        if run["nodes"] != fingerprint:
            raise SystemExit(f"{exe}: benched {run['nodes']}, not {fingerprint}: not the binary you think it is")
    return runs


def parse_option(text: str) -> tuple[str, str]:
    """`Name=Value` as given on the command line."""
    name, sep, value = text.partition("=")
    if not sep or not name.strip() or not value.strip():
        raise SystemExit(f"option {text!r}: expected NAME=VALUE")
    return name.strip(), value.strip()


def bench_input(options) -> str:
    """The engine's stdin: each option set, then the bench."""
    return "".join(f"setoption name {n} value {v}\n" for n, v in options) + "bench 13 3\n"


def read(exe: Path, fingerprint: int, options=()) -> list[dict]:
    out = subprocess.run([str(exe)], input=bench_input(options), capture_output=True, text=True,
                         timeout=900).stdout
    return parse_runs(out, str(exe), fingerprint)


def check_arms(base_dir: Path, cand_dir: Path, builds: dict, options: dict) -> None:
    """Distinct pools must be distinct bytes; one pool on both arms is a
    same-binary comparison and needs the arms' options to differ."""
    if base_dir.resolve() == cand_dir.resolve():
        if list(options["base"]) == list(options["cand"]):
            raise SystemExit("one pool on both arms needs the arms' options to differ")
        return
    digests = {hashlib.sha256(e.read_bytes()).hexdigest() for arm in builds for e in builds[arm]}
    if len(digests) != sum(len(v) for v in builds.values()):
        raise SystemExit("two pool binaries are identical")


def host_state() -> dict:
    """Host CPU and competing processes by the harness's measure. A one-shot
    `Win32_Processor.LoadPercentage` from a fresh PowerShell, which this
    replaced, reported that PowerShell's own start-up: 12-14% on a host the
    harness measure read at 3%."""
    shell = shutil.which("pwsh") or shutil.which("powershell")
    script = (f". '{HARNESS}'; [pscustomobject]@{{ busy = Get-HarnessHostBusyPercent; "
              "procs = @(Get-HarnessBusyProcess | ForEach-Object { \"$($_.ProcessName):$($_.Id)\" }) }"
              " | ConvertTo-Json -Compress")
    try:
        out = subprocess.run([shell, "-NoProfile", "-Command", script],
                             capture_output=True, text=True, timeout=60).stdout.strip()
        state = json.loads(out)
    except (OSError, TypeError, ValueError, subprocess.TimeoutExpired):
        return {"busy_percent": None, "busy_processes": []}
    busy = state.get("busy")
    return {"busy_percent": None if busy is None else round(float(busy), 1),
            "busy_processes": list(state.get("procs") or [])}


def cpu_load() -> float | None:
    return host_state()["busy_percent"]


def idle_refusal(state: dict, max_busy: float = MAX_BUSY_PERCENT) -> list[str]:
    """Why the host is not idle; empty when it is. An unreadable CPU figure
    refuses nothing on its own, as in the harness."""
    reasons = []
    if state["busy_processes"]:
        reasons.append("engine, harness or build processes are running: "
                       + ", ".join(state["busy_processes"]))
    if state["busy_percent"] is not None and state["busy_percent"] > max_busy:
        reasons.append(f"host CPU is {state['busy_percent']:.0f}%, over the {max_busy:.0f}% ceiling")
    return reasons


def check_host(record: dict, allow_busy: bool) -> None:
    """AGENTS, *Measurement*: measure only on an idle host; refuse otherwise,
    unless waived, and archive the state and any waiver with the read."""
    state = host_state()
    reasons = idle_refusal(state)
    record.setdefault("host_checks", []).append(
        {**state, "at": time.strftime("%Y-%m-%dT%H:%M:%S"), "waived": bool(reasons and allow_busy)})
    if reasons and not allow_busy:
        raise SystemExit("HOST NOT IDLE - " + "; ".join(reasons)
                         + ". Stop the other work and re-run, or pass --allow-busy-host and say why.")
    if reasons:
        print("WARNING: host not idle, check waived: " + "; ".join(reasons), flush=True)
    else:
        print(f"host idle: {state['busy_percent']}% CPU, no engine, harness or build process", flush=True)


# --- the estimator, on archived readings ------------------------------------

def best_readings(record: dict, arm: str) -> list[list[float]]:
    """Per build, the best-of-3 reading of every cycle."""
    builds = record["arms"][arm]["builds"]
    return [[max(run["nps"] for run in cycle["readings"][arm][str(i)]) for cycle in record["cycles"]]
            for i in range(len(builds))]


def estimate(record: dict) -> dict:
    base = best_readings(record, "base")
    cand = best_readings(record, "cand")
    base_medians = [statistics.median(rows) for rows in base]
    cand_medians = [statistics.median(rows) for rows in cand]
    base_level = statistics.mean(base_medians)
    cand_level = statistics.mean(cand_medians)
    delta = 100.0 * (cand_level / base_level - 1.0)
    nb, nc = len(base_medians), len(cand_medians)
    cycles = len(record["cycles"])
    if nb == 1 and nc == 1:
        # Single deterministic builds: the noise is between cycles, and each
        # cycle reads both arms, so the cycles pair.
        if cycles < 3:
            raise SystemExit("single-build arms need three or more cycles (use --cycles 10)")
        ratios = [cand[0][c] / base[0][c] - 1.0 for c in range(cycles)]
        se = 100.0 * statistics.stdev(ratios) / math.sqrt(cycles)
        df = cycles - 1
    elif nb == 1 or nc == 1:
        raise SystemExit("a single-build arm is read only against another single-build arm")
    else:
        pooled = ((nb - 1) * statistics.variance(base_medians) + (nc - 1) * statistics.variance(cand_medians)) / (nb + nc - 2)
        se = 100.0 * math.sqrt(pooled * (1 / nb + 1 / nc)) / base_level
        df = nb + nc - 2
    t = T95.get(df) or T95[max(k for k in T95 if k <= df)]
    # Disturbance: an arm's cycle mean against its median cycle.
    disturbed = []
    for arm, rows in (("base", base), ("cand", cand)):
        cycle_means = [statistics.mean(rows[b][c] for b in range(len(rows))) for c in range(cycles)]
        reference = statistics.median(cycle_means)
        for c, mean in enumerate(cycle_means):
            if abs(mean / reference - 1.0) > 0.01:
                disturbed.append((arm, c))
    if cycles == 2:
        # Two cycles have no median to lean on: they must agree.
        disturbed = [(arm, 1) for arm, rows in (("base", base), ("cand", cand))
                     if abs(statistics.mean(r[1] for r in rows) / statistics.mean(r[0] for r in rows) - 1.0) > 0.01]
    return {
        "cycles": cycles, "builds": [nb, nc], "base_level": base_level, "cand_level": cand_level,
        "base_medians": base_medians, "cand_medians": cand_medians,
        "delta_percent": delta, "se_percent": se, "t": t, "df": df,
        "low_percent": delta - t * se, "high_percent": delta + t * se,
        "disturbed": disturbed,
    }


def decide(est: dict, no_regression: bool) -> str:
    cycles, delta, low = est["cycles"], est["delta_percent"], est["low_percent"]
    disturbed_cycles = {c for _, c in est["disturbed"]}
    if (cycles == 2 and disturbed_cycles) or (cycles >= 6 and len(disturbed_cycles) >= 3):
        return "disturbed: repeat the step, do not interpret"
    if est["builds"] == [1, 1]:
        return "single-build screen: an estimate, no verdict (screens rank, reads decide)"
    if no_regression:
        # The thresholds mirrored around -0.5%.
        if cycles < 6:
            if delta >= -0.1:
                return "pass: regression excluded by step 1"
            if delta <= -0.9:
                return "fail: regression by step 1"
            return "between: run step 2 (--extend, four more cycles)"
        return "pass: no regression" if delta > -0.5 else "fail: regression"
    if cycles < 6:
        if delta >= 0.9:
            return "accepted by step 1"
        if delta <= 0.1:
            return "closed NO_CHANGE by step 1"
        return "between: run step 2 (--extend, four more cycles)"
    return "accepted" if delta >= 0.5 and low > 0 else "closed NO_CHANGE"


def report(record: dict, no_regression: bool) -> str:
    est = estimate(record)
    verdict = decide(est, no_regression)
    lines = [
        f"== {record['label']}",
        f"base {record['arms']['base']['pool']} ({est['builds'][0]} builds)  cand {record['arms']['cand']['pool']} ({est['builds'][1]} builds)",
        "options: " + "; ".join(
            f"{arm} " + (", ".join(f"{n}={v}" for n, v in record['arms'][arm].get('options', [])) or "defaults")
            + f" at {record['arms'][arm].get('fingerprint', '?')}" for arm in ("base", "cand")),
        f"cycles {est['cycles']}; CPU before each: {[c.get('cpu_before') for c in record['cycles']]}; after {record.get('cpu_after')}",
        "per-build medians (nps): base " + ", ".join(f"{m:,.0f}" for m in est["base_medians"])
        + " | cand " + ", ".join(f"{m:,.0f}" for m in est["cand_medians"]),
        f"levels: base {est['base_level']:,.0f}  cand {est['cand_level']:,.0f}",
        f"delta {est['delta_percent']:+.2f}%  95% t-interval {est['low_percent']:+.2f}% .. {est['high_percent']:+.2f}% (t {est['t']}, df {est['df']})",
        f"disturbed cycles: {est['disturbed'] or 'none'}",
        f"verdict: {verdict}",
    ]
    return "\n".join(lines)


# --- the read ----------------------------------------------------------------

def run_cycles(record: dict, out: Path, builds: dict[str, list[Path]], cycles: int,
               fingerprints: dict[str, int], options: dict | None = None) -> None:
    options = options or {}
    arms = list(builds)
    order_forward = [(arm, i) for arm in arms for i in range(len(builds[arm]))]
    for _ in range(cycles):
        number = len(record["cycles"])
        cpu = cpu_load()
        order = order_forward if number % 2 == 0 else order_forward[::-1]
        readings = {arm: {} for arm in arms}
        for arm, i in order:
            readings[arm][str(i)] = read(builds[arm][i], fingerprints[arm], options.get(arm, ()))
        record["cycles"].append({"cycle": number, "cpu_before": cpu, "readings": readings,
                                 "finished": time.strftime("%Y-%m-%dT%H:%M:%S")})
        (out / "raw.json").write_text(json.dumps(record, indent=1), encoding="utf-8")
        print(f"cycle {number + 1}: CPU {cpu}% before", flush=True)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--base", type=Path)
    parser.add_argument("--cand", type=Path)
    parser.add_argument("--out", type=Path)
    parser.add_argument("--label", default="")
    parser.add_argument("--cycles", type=int, default=2, help="step 1's cycles (default 2)")
    parser.add_argument("--extend", type=Path, help="add four cycles to this read (step 2)")
    parser.add_argument("--report", type=Path, help="recompute a read from its raw.json")
    parser.add_argument("--no-regression", action="store_true")
    parser.add_argument("--allow-busy-host", action="store_true",
                        help="read on a host that is not idle; the waiver is archived")
    parser.add_argument("--base-option", action="append", default=[], metavar="NAME=VALUE",
                        help="setoption sent to every base run (tune builds)")
    parser.add_argument("--cand-option", action="append", default=[], metavar="NAME=VALUE",
                        help="setoption sent to every candidate run (tune builds)")
    parser.add_argument("--base-fingerprint", type=int, help="the base arm's bench 13 nodes, if not the pool's")
    parser.add_argument("--cand-fingerprint", type=int, help="the candidate arm's bench 13 nodes, if not the pool's")
    args = parser.parse_args()

    if args.report:
        record = json.loads((args.report / "raw.json").read_text(encoding="utf-8"))
        print(report(record, args.no_regression or record.get("no_regression", False)))
        return 0

    if args.extend:
        out = args.extend
        record = json.loads((out / "raw.json").read_text(encoding="utf-8"))
        if len(record["cycles"]) != 2:
            raise SystemExit(f"{out}: step 2 extends a two-cycle read, this one has {len(record['cycles'])}")
        builds = {arm: pool(Path(record["arms"][arm]["pool"])) for arm in ("base", "cand")}
        fingerprints = {arm: record["arms"][arm].get("fingerprint")
                        or pool_fingerprint(Path(record["arms"][arm]["pool"])) for arm in builds}
        options = {arm: [tuple(o) for o in record["arms"][arm].get("options", [])] for arm in builds}
        for arm in builds:
            if [str(e) for e in builds[arm]] != record["arms"][arm]["builds"]:
                raise SystemExit(f"{out}: the {arm} pool changed since step 1")
        check_host(record, args.allow_busy_host)
        cycles = 4
    else:
        if not (args.base and args.cand and args.out):
            parser.error("--base, --cand and --out are required for a new read")
        out = args.out
        out.mkdir(parents=True, exist_ok=True)
        if (out / "raw.json").exists():
            raise SystemExit(f"{out}/raw.json exists: --extend it or choose another directory")
        builds = {"base": pool(args.base), "cand": pool(args.cand)}
        fingerprints = {"base": args.base_fingerprint or pool_fingerprint(args.base),
                        "cand": args.cand_fingerprint or pool_fingerprint(args.cand)}
        options = {"base": [parse_option(o) for o in args.base_option],
                   "cand": [parse_option(o) for o in args.cand_option]}
        check_arms(args.base, args.cand, builds, options)
        record = {
            "label": args.label, "started": time.strftime("%Y-%m-%dT%H:%M:%S"), "no_regression": args.no_regression,
            "arms": {arm: {"pool": str(path), "builds": [str(e) for e in builds[arm]],
                           "fingerprint": fingerprints[arm], "options": options[arm]}
                     for arm, path in (("base", args.base), ("cand", args.cand))},
            "cycles": [],
        }
        check_host(record, args.allow_busy_host)
        for arm in builds:
            for exe in builds[arm]:
                read(exe, fingerprints[arm], options[arm])  # warm-up, and the fingerprint check
        cycles = args.cycles

    run_cycles(record, out, builds, cycles, fingerprints, options)
    record["cpu_after"] = cpu_load()
    record["finished"] = time.strftime("%Y-%m-%dT%H:%M:%S")
    (out / "raw.json").write_text(json.dumps(record, indent=1), encoding="utf-8")
    for arm in builds:
        pool(Path(record["arms"][arm]["pool"]))  # unchanged bytes after the read
    text = report(record, record.get("no_regression", False))
    (out / "read.txt").write_text(text + "\n", encoding="utf-8")
    print(text)
    return 0


if __name__ == "__main__":
    sys.exit(main())
