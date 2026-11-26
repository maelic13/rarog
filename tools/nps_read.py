#!/usr/bin/env python3
"""The two-step pooled-PGO NPS read (PROCESS, *Harness notes*; RAR-P32).

Two pools of PGO builds of one source each (nps_build_pool.ps1, four builds,
manifest-<tier>.txt), read in interleaved cycles of `bench 13 3`. A reading is
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
Fresh executables are scanned by antivirus, which contaminates a read
(RAR-M48), so every build first runs one short `bench 8 1` (archived, not
counted) to set the scan off, and the read then waits until the host has
stayed quiet (no such process, at most 10% CPU) for `--quiet-seconds`; a
scan still running holds the wait. `--allow-busy-host` waives both checks
and the waiver is archived with the read.

Every build is checked against its manifest hash before and after the read,
every run must print its own pool's fingerprint (from that pool's manifest, so a
release baseline can be read against the head), and the pools must be distinct bytes.

An arm may instead differ by UCI options on a tune build (`--cand-option
LazyMargin=2000`, sent as `setoption` before the bench). Then one pool may serve
both arms, which cancels each build's PGO offset (the unpaired interval below is
conservative for that design), the options must differ, and an option that
changes the search names that arm's fingerprint (`--cand-fingerprint`).

Arms at different fingerprints search different trees, so their NPS is not a
speed read (PLAN rule 8: a tree-shape change reads time-to-depth). The read
still runs, and its report and console carry a warning saying so.

`--smp` reads arms that set `Threads` above one. Lazy SMP's node count varies
by run, so no run is checked against a fingerprint, the arms' trees are not
compared, and the report gives the levels and the interval with no verdict: a
multi-thread cell beside a one-thread read, never a speed rule's read.

On Linux under WSL the host measure is still the harness's: Windows' own
PowerShell runs it through interop, over the whole machine, the Linux side's
load included, and the Linux side's engine, harness and build processes are
added from `/proc`, since Windows' process list does not show them. A Linux
pool's builds have no `.exe`.

`--positions <set.jsonl>` reads game positions instead of `bench 13 3` (PLAN
D.16.1): a set frozen from a gate's games by nps_positions.py, refused unless
its hash matches its manifest. A reading is one pass over the set, each
position in a fresh process (bench shares its table across positions): one
thread, `--hash` MB (the gates' 64), `ucinewgame` and `isready` before the
clock starts, so the table and history clears have written their pages, a
`go depth 1` and a second `ucinewgame` so lazily built tables are built, then
`go nodes <budget>` (the manifest's, or `--nodes`) timed from `go` to
`bestmove`. Its rate is the positions' nodes over their summed time; a pass of
many processes takes the place of the best of three. Each reading's best moves
and last reported counts must equal its arm's first reading's, the tree check
that the bench's fingerprint makes; arms whose trees differ carry the warning.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import re
import shutil
import statistics
import subprocess
import sys
import time
from pathlib import Path

RUN = re.compile(r"^run (\d)/3\s+nodes (\d+)\s+time (\d+)ms\s+nps (\d+)", re.M)
MANIFEST_ROW = re.compile(r"^([0-9A-F]{64})\s+(\S+)$", re.M)
MANIFEST_FINGERPRINT = re.compile(r"^fingerprint\s*:\s*(\d+)\b", re.M)
# Two-sided 95% t quantiles by degrees of freedom; 6 is the four-build read,
# 5 a three-build pool against a four-build one.
T95 = {2: 4.303, 3: 3.182, 4: 2.776, 5: 2.571, 6: 2.447, 7: 2.365, 8: 2.306, 9: 2.262, 10: 2.228,
       12: 2.179, 14: 2.145, 16: 2.120, 20: 2.086, 30: 2.042}
HARNESS = Path(__file__).resolve().parent / "harness_common.ps1"
# Windows' own PowerShell as WSL reaches it.
WSL_PWSH = Path("/mnt/c/Program Files/PowerShell/7/pwsh.exe")
# Get-HarnessBusyProcess's patterns, for the Linux side of a WSL read.
LINUX_BUSY = re.compile(r"^(rarog|fastchess|colosseum-cli|cutechess|stockfish|basilisk|reckless|cargo|rustc)"
                        r"|^(pext|avx2|x86-64|base)-[0-9]")
MAX_BUSY_PERCENT = 15.0  # colosseum.ps1's default ceiling
WARMUP_DEPTH = 8
QUIET_CEILING_PERCENT = 10.0
GAME_HASH_MB = 64  # the gates' Hash (colosseum.ps1)
MANIFEST_FIELD = re.compile(r"^(\w+)\s*:\s*(.+?)\s*$", re.M)
QUIET_SECONDS = 60
MAX_QUIET_MINUTES = 30


def pool_manifest(directory: Path) -> Path:
    """The pool's one manifest, `manifest-<tier>.txt` as nps_build_pool.ps1
    names it. Two in one directory would leave the builds ambiguous."""
    manifests = sorted(directory.glob("manifest-*.txt"))
    if len(manifests) != 1:
        raise SystemExit(f"{directory}: expected one manifest-<tier>.txt, found "
                         f"{[m.name for m in manifests] or 'none'}")
    return manifests[0]


def pool_fingerprint(directory: Path) -> int:
    """The bench fingerprint the pool's manifest verified on every copy. Each
    arm is checked against its own, so a pool of an earlier source (a release
    baseline) can be read against the head's."""
    manifest = pool_manifest(directory)
    m = MANIFEST_FINGERPRINT.search(manifest.read_text(encoding="utf-8"))
    if not m:
        raise SystemExit(f"{manifest}: records no fingerprint")
    return int(m.group(1))


def pool(directory: Path) -> list[Path]:
    manifest = pool_manifest(directory)
    rows = MANIFEST_ROW.findall(manifest.read_text(encoding="utf-8"))
    if not rows:
        raise SystemExit(f"{manifest}: no build rows")
    if len(rows) < 2:
        raise SystemExit(f"{manifest}: one build gives no spread; a pool needs at least two")
    pool_fingerprint(directory)
    builds = []
    for digest, name in rows:
        exe = directory / name
        if hashlib.sha256(exe.read_bytes()).hexdigest().upper() != digest:
            raise SystemExit(f"{exe}: does not match its manifest")
        builds.append(exe)
    return builds


def parse_runs(output: str, exe: str, fingerprint: int | None) -> list[dict]:
    """The bench's three runs, each at `fingerprint`; with none (`--smp`),
    each merely searched something."""
    runs = [{"nodes": int(n), "ms": int(ms), "nps": int(nps)} for _, n, ms, nps in RUN.findall(output)]
    if len(runs) != 3:
        raise SystemExit(f"{exe}: expected three runs, got {len(runs)}")
    for run in runs:
        if fingerprint is None:
            if run["nodes"] <= 0:
                raise SystemExit(f"{exe}: a run searched no nodes")
        elif run["nodes"] != fingerprint:
            raise SystemExit(f"{exe}: benched {run['nodes']}, not {fingerprint}: not the binary you think it is")
    return runs


def parse_option(text: str) -> tuple[str, str]:
    """`Name=Value` as given on the command line."""
    name, sep, value = text.partition("=")
    if not sep or not name.strip() or not value.strip():
        raise SystemExit(f"option {text!r}: expected NAME=VALUE")
    return name.strip(), value.strip()


def bench_input(options, bench: str = "bench 13 3") -> str:
    """The engine's stdin: each option set, then the bench."""
    return "".join(f"setoption name {n} value {v}\n" for n, v in options) + bench + "\n"


def read(exe: Path, fingerprint: int | None, options=()) -> list[dict]:
    out = subprocess.run([str(exe)], input=bench_input(options), capture_output=True, text=True,
                         timeout=900).stdout
    return parse_runs(out, str(exe), fingerprint)


def game_positions(path: Path) -> dict:
    """A frozen set and its manifest; refused when the set's bytes are not the
    manifest's."""
    manifest = path.with_name(path.name + ".manifest.txt")
    if not manifest.exists():
        raise SystemExit(f"{path}: no manifest {manifest.name}")
    fields = dict(MANIFEST_FIELD.findall(manifest.read_text(encoding="utf-8")))
    data = path.read_bytes()
    if hashlib.sha256(data).hexdigest() != fields.get("set_sha256"):
        raise SystemExit(f"{path}: does not match its manifest's set_sha256")
    fens = [json.loads(line)["fen"] for line in data.decode("utf-8").splitlines() if line.strip()]
    if not fens:
        raise SystemExit(f"{path}: no positions")
    return {"path": str(path), "sha256": fields["set_sha256"], "nodes": int(fields["nodes"]), "fens": fens}


def search_position(exe: Path, fen: str, nodes: int, hash_mb: int, options=()) -> dict:
    """One position in a fresh process: the time from `go` to `bestmove`, the
    best move and the last reported count and depth."""
    proc = subprocess.Popen([str(exe)], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                            stderr=subprocess.DEVNULL, text=True, bufsize=1)

    def send(line: str) -> None:
        proc.stdin.write(line + "\n")
        proc.stdin.flush()

    def until(token: str) -> list[str]:
        lines = []
        while True:
            line = proc.stdout.readline()
            if not line:
                raise SystemExit(f"{exe}: exited before {token!r}")
            lines.append(line)
            if line.startswith(token):
                return lines

    try:
        send("uci")
        until("uciok")
        for name, value in [("Threads", "1"), ("Hash", str(hash_mb)), *options]:
            send(f"setoption name {name} value {value}")
        for go in ("depth 1", f"nodes {nodes}"):
            send("ucinewgame")
            send("isready")
            until("readyok")
            send(f"position fen {fen}")
            started = time.perf_counter()
            send(f"go {go}")
            lines = until("bestmove")
            elapsed = time.perf_counter() - started
        send("quit")
        proc.wait(timeout=30)
    finally:
        if proc.poll() is None:
            proc.kill()
    last_nodes, last_depth = 0, 0
    for line in lines:
        tokens = line.split()
        if tokens[:1] == ["info"] and "nodes" in tokens:
            last_nodes = int(tokens[tokens.index("nodes") + 1])
            if "depth" in tokens:
                last_depth = int(tokens[tokens.index("depth") + 1])
    return {"seconds": elapsed, "best": lines[-1].split()[1], "last_nodes": last_nodes, "depth": last_depth}


def read_games(exe: Path, workload: dict, options=()) -> list[dict]:
    """One pass over the set: a reading of one run, its rate the budget's nodes
    over the summed search time, with the tree it searched."""
    rows = [search_position(exe, fen, workload["nodes"], workload["hash"], options) for fen in workload["fens"]]
    seconds = sum(r["seconds"] for r in rows)
    nodes = workload["nodes"] * len(rows)
    return [{"nodes": nodes, "ms": round(1000 * seconds, 3), "nps": int(nodes / seconds),
             "tree": [[r["best"], r["last_nodes"]] for r in rows], "depths": [r["depth"] for r in rows]}]


def check_tree(record: dict, arm: str, runs: list[dict], exe: Path) -> None:
    """A game-position reading must search its arm's tree: the first reading
    sets it, every later one, any build of the arm, must equal it."""
    tree = runs[0]["tree"]
    known = record["arms"][arm].get("game_tree")
    if known is None:
        record["arms"][arm]["game_tree"] = tree
    elif tree != known:
        moved = sum(1 for a, b in zip(tree, known) if a != b)
        raise SystemExit(f"{exe}: searched another tree on {moved} of {len(tree)} positions: "
                         "not the binary you think it is, or the search is not deterministic")


def warm_up(exe: Path, options=()) -> dict:
    """One short bench, so a fresh executable's antivirus scan starts before
    the quiet wait rather than inside a cycle. Its node count is not the
    fingerprint; every counted reading checks that."""
    bench = f"bench {WARMUP_DEPTH} 1"
    out = subprocess.run([str(exe)], input=bench_input(options, bench), capture_output=True, text=True,
                         timeout=300).stdout
    # One run prints only the summary, not a `run 1/1` line.
    runs = re.findall(r"^Nodes searched\s*:\s*(\d+).*?^Total time \(ms\)\s*:\s*(\d+).*?^Nodes/second\s*:\s*(\d+)",
                      out, re.M | re.S)
    if len(runs) != 1:
        raise SystemExit(f"{exe}: `{bench}` printed {len(runs)} summaries, expected one")
    nodes, ms, nps = runs[0]
    return {"exe": str(exe), "depth": WARMUP_DEPTH, "nodes": int(nodes), "ms": int(ms), "nps": int(nps)}


def check_threads(options: dict, smp: bool) -> None:
    """`--smp` exactly when the arms set `Threads` above one: a one-thread
    read keeps its fingerprint check, and a multi-thread one cannot pass it."""
    threads = {arm: max([int(v) for n, v in options[arm] if n.lower() == "threads"] or [1])
               for arm in options}
    multi = {arm for arm, count in threads.items() if count > 1}
    if smp and multi != set(options):
        raise SystemExit("--smp is for arms that both set Threads above 1")
    if not smp and multi:
        raise SystemExit("an arm sets Threads above 1: lazy SMP's node counts vary, read it with --smp")


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


def windows_path(path: Path) -> str:
    """`path` as Windows names it, from WSL."""
    return subprocess.run(["wslpath", "-w", str(path)], capture_output=True, text=True,
                          timeout=30, check=True).stdout.strip()


def harness_shell() -> tuple[str, str] | None:
    """The PowerShell that runs the harness's measure, and the harness's path
    as that shell names it: one on the path, else under WSL Windows' own
    through interop; None where there is neither."""
    shell = shutil.which("pwsh") or shutil.which("powershell")
    if shell:
        return shell, str(HARNESS)
    if WSL_PWSH.exists():
        try:
            return str(WSL_PWSH), windows_path(HARNESS)
        except (OSError, subprocess.SubprocessError):
            return None
    return None


def linux_busy_processes(proc: Path = Path("/proc")) -> list[str]:
    """The Linux side's engine, harness and build processes, `name:pid`, by
    the harness's patterns; this process is never one of them."""
    found = []
    for entry in proc.iterdir():
        if not entry.name.isdigit() or int(entry.name) == os.getpid():
            continue
        try:
            name = (entry / "comm").read_text(encoding="utf-8").strip().lower()
        except OSError:
            continue
        if LINUX_BUSY.match(name):
            found.append(f"{name}:{entry.name}")
    return sorted(found)


def host_state() -> dict:
    """Host CPU and competing processes by the harness's measure. A one-shot
    `Win32_Processor.LoadPercentage` from a fresh PowerShell, which this
    replaced, reported that PowerShell's own start-up: 12-14% on a host the
    harness measure read at 3%."""
    linux = linux_busy_processes() if sys.platform == "linux" else []
    found = harness_shell()
    if found is None:
        return {"busy_percent": None, "busy_processes": linux}
    shell, harness = found
    script = (f". '{harness}'; [pscustomobject]@{{ busy = Get-HarnessHostBusyPercent; "
              "procs = @(Get-HarnessBusyProcess | ForEach-Object { \"$($_.ProcessName):$($_.Id)\" }) }"
              " | ConvertTo-Json -Compress")
    try:
        out = subprocess.run([shell, "-NoProfile", "-Command", script],
                             capture_output=True, text=True, timeout=60).stdout.strip()
        state = json.loads(out)
    except (OSError, TypeError, ValueError, subprocess.TimeoutExpired):
        return {"busy_percent": None, "busy_processes": linux}
    busy = state.get("busy")
    return {"busy_percent": None if busy is None else round(float(busy), 1),
            "busy_processes": list(state.get("procs") or []) + linux}


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
    """docs/rules/measurement.md: measure only on an idle host; refuse otherwise,
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


def wait_quiet(record: dict, allow_busy: bool, seconds: float, clock=time.monotonic) -> None:
    """Wait until the host has stayed quiet for `seconds`: no engine, harness or
    build process and at most QUIET_CEILING_PERCENT CPU at every reading. A
    single start-of-read check passes in a lull of an antivirus scan; this
    holds the read until the scan is over. Gives up after MAX_QUIET_MINUTES,
    refusing unless waived, and archives the wait."""
    print(f"waiting for {seconds:.0f} s of host load at most {QUIET_CEILING_PERCENT:.0f}% ...", flush=True)
    started = clock()
    quiet_since = None
    readings = []
    while True:
        state = host_state()
        readings.append(state["busy_percent"])
        quiet = not state["busy_processes"] and (
            state["busy_percent"] is None or state["busy_percent"] <= QUIET_CEILING_PERCENT)
        now = clock()
        if quiet:
            quiet_since = now if quiet_since is None else quiet_since
            if now - quiet_since >= seconds:
                break
        else:
            quiet_since = None
        if now - started > 60 * MAX_QUIET_MINUTES:
            reasons = idle_refusal(state, QUIET_CEILING_PERCENT) or ["host load never stayed down"]
            record.setdefault("quiet_waits", []).append(
                {"seconds": round(now - started, 1), "busy_percent": readings, "quiet": False, "waived": allow_busy})
            if not allow_busy:
                raise SystemExit(f"HOST NOT QUIET - {'; '.join(reasons)} for {MAX_QUIET_MINUTES} min; a read needs "
                                 f"{seconds:.0f} s at most {QUIET_CEILING_PERCENT:.0f}%. Stop the other work and re-run.")
            print("WARNING: host never quiet, wait waived: " + "; ".join(reasons), flush=True)
            return
    record.setdefault("quiet_waits", []).append(
        {"seconds": round(clock() - started, 1), "busy_percent": readings, "quiet": True, "waived": False})
    print(f"host quiet after {clock() - started:.0f} s", flush=True)


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
    pooled = ((nb - 1) * statistics.variance(base_medians) + (nc - 1) * statistics.variance(cand_medians)) / (nb + nc - 2)
    se = 100.0 * math.sqrt(pooled * (1 / nb + 1 / nc)) / base_level
    df = nb + nc - 2
    t = T95.get(df) or T95[max(k for k in T95 if k <= df)]
    cycles = len(record["cycles"])
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


def game_tree_warning(record: dict) -> str | None:
    """The warning a game-position read carries when its arms' trees differ."""
    base, cand = (record["arms"][arm].get("game_tree") for arm in ("base", "cand"))
    if base is None or cand is None or base == cand:
        return None
    moved = sum(1 for a, b in zip(base, cand) if a != b)
    return (f"WARNING: the arms search different trees on {moved} of {len(base)} game positions; "
            "their NPS is not a speed read: read time-to-depth (PLAN rule 8)")


def tree_warning(fingerprints: dict) -> str | None:
    """The warning a read carries when its arms search different trees."""
    base, cand = fingerprints.get("base"), fingerprints.get("cand")
    if base is None or cand is None or base == cand:
        return None
    return (f"WARNING: the arms search different trees (bench 13 {base:,} against {cand:,}); "
            "their NPS is not a speed read: read time-to-depth (PLAN rule 8)")


def report(record: dict, no_regression: bool) -> str:
    est = estimate(record)
    verdict = ("none: a multi-thread cell (--smp), its node counts vary by run; no speed rule reads it"
               if record.get("smp") else decide(est, no_regression))
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
    workload = record.get("workload")
    if workload:
        lines.insert(1, f"workload: {len(workload['fens'])} game positions ({workload['path']}, "
                        f"{workload['sha256'][:12]}), go nodes {workload['nodes']}, Hash {workload['hash']}, "
                        "a fresh process a position")
    for warning in (tree_warning({arm: record["arms"][arm].get("fingerprint") for arm in ("base", "cand")}),
                    game_tree_warning(record)):
        if warning:
            lines.append(warning)
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
        workload = record.get("workload")
        for arm, i in order:
            if workload:
                runs = read_games(builds[arm][i], workload, options.get(arm, ()))
                check_tree(record, arm, runs, builds[arm][i])
            else:
                runs = read(builds[arm][i], fingerprints[arm], options.get(arm, ()))
            readings[arm][str(i)] = runs
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
    parser.add_argument("--quiet-seconds", type=float, default=QUIET_SECONDS,
                        help=f"quiet host time before the cycles (default {QUIET_SECONDS})")
    parser.add_argument("--base-option", action="append", default=[], metavar="NAME=VALUE",
                        help="setoption sent to every base run (tune builds)")
    parser.add_argument("--cand-option", action="append", default=[], metavar="NAME=VALUE",
                        help="setoption sent to every candidate run (tune builds)")
    parser.add_argument("--base-fingerprint", type=int, help="the base arm's bench 13 nodes, if not the pool's")
    parser.add_argument("--cand-fingerprint", type=int, help="the candidate arm's bench 13 nodes, if not the pool's")
    parser.add_argument("--positions", type=Path,
                        help="read this frozen game-position set instead of bench 13 3 (nps_positions.py)")
    parser.add_argument("--nodes", type=int, help="the game-position budget, if not the set manifest's")
    parser.add_argument("--hash", type=int, default=GAME_HASH_MB, help="Hash for game positions (default 64)")
    parser.add_argument("--smp", action="store_true",
                        help="arms at Threads above 1: no fingerprint check and no verdict")
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
        fingerprints = {arm: None if record.get("smp") else record["arms"][arm].get("fingerprint")
                        or pool_fingerprint(Path(record["arms"][arm]["pool"])) for arm in builds}
        options = {arm: [tuple(o) for o in record["arms"][arm].get("options", [])] for arm in builds}
        for arm in builds:
            if [str(e) for e in builds[arm]] != record["arms"][arm]["builds"]:
                raise SystemExit(f"{out}: the {arm} pool changed since step 1")
        if record.get("workload"):
            current = game_positions(Path(record["workload"]["path"]))
            if current["sha256"] != record["workload"]["sha256"]:
                raise SystemExit(f"{out}: the position set changed since step 1")
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
        options = {"base": [parse_option(o) for o in args.base_option],
                   "cand": [parse_option(o) for o in args.cand_option]}
        check_threads(options, args.smp)
        if args.smp and (args.positions or args.base_fingerprint or args.cand_fingerprint):
            parser.error("--smp reads bench 13 3 with no fingerprint")
        fingerprints = ({"base": None, "cand": None} if args.smp else
                        {"base": args.base_fingerprint or pool_fingerprint(args.base),
                         "cand": args.cand_fingerprint or pool_fingerprint(args.cand)})
        check_arms(args.base, args.cand, builds, options)
        warning = tree_warning(fingerprints)
        if warning:
            print(warning, flush=True)
        if args.nodes is not None and not args.positions:
            parser.error("--nodes is for a game-position read (--positions)")
        record = {
            "label": args.label, "started": time.strftime("%Y-%m-%dT%H:%M:%S"), "no_regression": args.no_regression,
            "smp": args.smp,
            "arms": {arm: {"pool": str(path), "builds": [str(e) for e in builds[arm]],
                           "fingerprint": fingerprints[arm], "options": options[arm]}
                     for arm, path in (("base", args.base), ("cand", args.cand))},
            "cycles": [],
        }
        if args.positions:
            workload = game_positions(args.positions)
            workload["nodes"] = args.nodes or workload["nodes"]
            workload["hash"] = args.hash
            record["workload"] = workload
        check_host(record, args.allow_busy_host)
        record["warmup"] = [warm_up(exe, options[arm]) for arm in builds for exe in builds[arm]]
        cycles = args.cycles

    wait_quiet(record, args.allow_busy_host, args.quiet_seconds)
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
