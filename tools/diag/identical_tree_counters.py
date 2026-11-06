"""D.4.6's identical-tree counter read (analysis/identical_tree_counters_card_2026-11-05.md).

The four pext PGO builds of one source bench the same tree (`bench 13 1`,
node count asserted equal to the fingerprint on every run), each process
pinned to one logical CPU, in five rotated rounds on a quiet host.

  t1  unprivileged: each thread's scheduled cycles (QueryThreadCycleTime,
      which on this host counts the 3.4 GHz invariant time-stamp counter)
      beside bench NPS.
  t2  elevated: the same schedule, each bench inside an ETW kernel session
      that records TotalCycles, InstructionRetired and BranchMispredictions
      at every context switch (`xperf -pmc ... CSWITCH strict`); one ETL per
      run. --dry-run runs one round at depth 8 with no tracing, to prove the
      schedule.
  read  unprivileged: t2's ETLs to counters per node. xperf's Pmc event
      precedes its CSwitch and holds that CPU's running totals, so a thread's
      count is the sum, over its stretches on the CPU, of the totals at its
      switch out minus those at its switch in.

Writes raw.json and summary.txt (and t2's ETLs, and read's counters.json and
read.txt) under analysis/artifacts/d46-counters/<mode>/.
"""
import argparse
import ctypes
import ctypes.wintypes as wt
import hashlib
import json
import random
import re
import statistics
import subprocess
import sys
import tempfile
import time
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
POOL = ROOT / "analysis/artifacts/e33-universal-nps/pext"
OUT = ROOT / "analysis/artifacts/d46-counters"
XPERF = Path(r"C:\Program Files (x86)\Windows Kits\10\Windows Performance Toolkit\xperf.exe")
PMC = "TotalCycles,InstructionRetired,BranchMispredictions"
FINGERPRINT = 14749043
CPU = 6
ROUNDS = 5
DEPTH = 13
LOAD_LIMIT = 0.10

k32 = ctypes.WinDLL("kernel32", use_last_error=True)
k32.OpenThread.restype = wt.HANDLE
k32.OpenProcess.restype = wt.HANDLE
k32.CreateToolhelp32Snapshot.restype = wt.HANDLE


class FILETIME(ctypes.Structure):
    _fields_ = [("lo", wt.DWORD), ("hi", wt.DWORD)]


class THREADENTRY32(ctypes.Structure):
    _fields_ = [("dwSize", wt.DWORD), ("cntUsage", wt.DWORD), ("th32ThreadID", wt.DWORD),
                ("th32OwnerProcessID", wt.DWORD), ("tpBasePri", wt.LONG), ("tpDeltaPri", wt.LONG),
                ("dwFlags", wt.DWORD)]


def system_times():
    idle, kern, user = FILETIME(), FILETIME(), FILETIME()
    assert k32.GetSystemTimes(ctypes.byref(idle), ctypes.byref(kern), ctypes.byref(user))
    return tuple((f.hi << 32) | f.lo for f in (idle, kern, user))


def busy(a, b):
    idle, total = b[0] - a[0], (b[1] - a[1]) + (b[2] - a[2])  # kernel time includes idle
    return 1.0 - idle / total if total else 0.0


def wait_quiet(seconds=30, give_up=600):
    start = time.time()
    while time.time() - start < give_up:
        a = system_times()
        time.sleep(seconds)
        load = busy(a, system_times())
        if load < LOAD_LIMIT:
            return load
        print(f"host busy {load:.1%}; waiting", flush=True)
    raise SystemExit("host never quiet")


def thread_cycles(pid):
    snap = k32.CreateToolhelp32Snapshot(0x4, 0)
    entry = THREADENTRY32()
    entry.dwSize = ctypes.sizeof(THREADENTRY32)
    out = {}
    ok = k32.Thread32First(snap, ctypes.byref(entry))
    while ok:
        if entry.th32OwnerProcessID == pid:
            handle = k32.OpenThread(0x0800, False, entry.th32ThreadID)
            if handle:
                cycles = ctypes.c_ulonglong()
                if k32.QueryThreadCycleTime(wt.HANDLE(handle), ctypes.byref(cycles)):
                    out[entry.th32ThreadID] = cycles.value
                k32.CloseHandle(wt.HANDLE(handle))
        ok = k32.Thread32Next(snap, ctypes.byref(entry))
    k32.CloseHandle(wt.HANDLE(snap))
    return out


def read_until(proc, marker):
    lines = []
    while True:
        line = proc.stdout.readline()
        if not line:
            raise RuntimeError(f"engine exited waiting for {marker!r}: {lines[-5:]}")
        lines.append(line.rstrip())
        if line.startswith(marker):
            return lines


def xperf(*args, check=True):
    done = subprocess.run([str(XPERF), *args], capture_output=True, text=True)
    if check and done.returncode != 0:
        raise RuntimeError(f"xperf {' '.join(args)} failed ({done.returncode}): {done.stdout}{done.stderr}")
    return done


class Trace:
    """One ETW kernel session around one bench; the ETL is written on stop."""

    def __init__(self, etl):
        self.etl = etl

    def start(self):
        xperf("-on", "PROC_THREAD+LOADER+CSWITCH", "-pmc", PMC, "CSWITCH", "strict")

    def stop(self):
        xperf("-stop", "-d", str(self.etl))


def run(exe, depth, trace=None):
    proc = subprocess.Popen([str(exe)], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                            stderr=subprocess.DEVNULL, text=True, bufsize=1)
    try:
        handle = k32.OpenProcess(0x0200 | 0x0400, False, proc.pid)
        assert handle and k32.SetProcessAffinityMask(wt.HANDLE(handle), ctypes.c_size_t(1 << CPU)), "affinity"
        k32.CloseHandle(wt.HANDLE(handle))

        def send(command):
            proc.stdin.write(command + "\n")
            proc.stdin.flush()

        send("uci")
        read_until(proc, "uciok")
        send("isready")
        read_until(proc, "readyok")
        if trace:
            trace.start()
        before, t0 = thread_cycles(proc.pid), system_times()
        send(f"bench {depth} 1")
        lines = read_until(proc, "Nodes/second")
        after, t1 = thread_cycles(proc.pid), system_times()
        if trace:
            trace.stop()
        send("quit")
        proc.wait(timeout=30)
    finally:
        if proc.poll() is None:
            proc.kill()
    text = "\n".join(lines)
    nodes = int(re.search(r"^Nodes searched\s*:\s*(\d+)", text, re.M).group(1))
    ms = int(re.search(r"^Total time \(ms\)\s*:\s*(\d+)", text, re.M).group(1))
    deltas = {tid: after[tid] - before.get(tid, 0) for tid in after}
    search_tid = max(deltas, key=deltas.get)
    search = deltas[search_tid]
    return {"exe": exe.name, "pid": proc.pid, "search_tid": search_tid, "cpu": CPU,
            "nodes": nodes, "ms": ms, "nps": nodes * 1000 / ms,
            "search_cycles": search, "all_cycles": sum(deltas.values()),
            "threads": len(deltas), "cycles_per_node": search / nodes, "load": busy(t0, t1),
            "etl": trace.etl.name if trace else None}


def pool():
    manifest = (POOL / "manifest-pext.txt").read_text(encoding="utf-8")
    builds = []
    for digest, name in re.findall(r"^([0-9A-F]{64})\s+(\S+\.exe)$", manifest, re.M):
        exe = POOL / name
        assert hashlib.sha256(exe.read_bytes()).hexdigest().upper() == digest, f"{exe} hash"
        builds.append(exe)
    assert len(builds) == 4, f"{len(builds)} builds in the manifest"
    return builds


def summarise(runs, builds, header):
    lines = [header]
    for key in ("nps", "cycles_per_node"):
        medians, cvs = [], []
        for exe in builds:
            xs = [x[key] for x in runs if x["exe"] == exe.name]
            medians.append(statistics.median(xs))
            cvs.append(statistics.stdev(xs) / statistics.mean(xs) if len(xs) > 1 else 0.0)
            lines.append(f"  {key:16s} {exe.name}: median {medians[-1]:,.3f}"
                         f" cv {cvs[-1]:.3%} range {(max(xs) - min(xs)) / statistics.mean(xs):.3%}")
        spread = (max(medians) - min(medians)) / statistics.mean(medians)
        lines.append(f"{key}: mean within-build cv {statistics.mean(cvs):.3%};"
                     f" spread of build medians {spread:.3%}")
    return lines


def schedule(out, rounds, depth, tracing):
    builds = pool()
    out.mkdir(parents=True, exist_ok=True)
    for exe in builds:
        run(exe, 8)  # starts the antivirus scan of a fresh executable
    print(f"quiet: {wait_quiet():.1%}", flush=True)
    runs = []
    for r in range(rounds):
        order = builds[r % 4:] + builds[:r % 4]
        for exe in order:
            name = f"r{r}-{exe.stem}"
            trace = Trace(out / f"{name}.etl") if tracing else None
            res = run(exe, depth, trace)
            if res["load"] > LOAD_LIMIT:
                print(f"{exe.name}: load {res['load']:.1%}, repeated", flush=True)
                trace = Trace(out / f"{name}-repeat.etl") if tracing else None
                res = run(exe, depth, trace)
                res["repeated"] = True
            if depth == DEPTH:
                assert res["nodes"] == FINGERPRINT, f"{exe.name} benched {res['nodes']}"
            res["round"] = r
            runs.append(res)
            print(f"round {r} {exe.name} nps {res['nps']:.0f} cyc/node {res['cycles_per_node']:.2f}"
                  f" threads {res['threads']} load {res['load']:.1%}", flush=True)
    (out / "raw.json").write_text(json.dumps(runs, indent=1), encoding="utf-8")
    lines = summarise(runs, builds,
                      f"{out.name}: {len(runs)} runs, {rounds} rounds, CPU {CPU}, bench {depth},"
                      f" fingerprint {FINGERPRINT}, tracing {'on' if tracing else 'off'}")
    (out / "summary.txt").write_text("\n".join(lines) + "\n", encoding="utf-8")
    print("\n".join(lines))
    return 0


def cmd_t1(args):
    return schedule(Path(args.out or OUT / "t1"), ROUNDS, DEPTH, tracing=False)


def cmd_t2(args):
    if args.dry_run:
        return schedule(Path(args.out or OUT / "t2-dry"), 1, 8, tracing=False)
    if not ctypes.windll.shell32.IsUserAnAdmin():
        print("t2 needs an elevated shell: the ETW kernel session is refused otherwise", file=sys.stderr)
        return 2
    assert XPERF.exists(), f"xperf not found at {XPERF}"
    xperf("-stop", check=False)  # a stale kernel session from an interrupted run
    try:
        return schedule(Path(args.out or OUT / "t2"), ROUNDS, DEPTH, tracing=True)
    finally:
        xperf("-stop", check=False)


@dataclass
class ThreadCounters:
    cycles: int
    instructions: int
    misses: int
    micros: int
    stretches: int


def thread_counters(lines, tid, cpu):
    """The thread's PMC totals from an `xperf -i` dump. A switch of the thread
    without its Pmc event, on another CPU, or out of order is an error, not a
    gap to skip: the sum would silently undercount."""
    totals = [0, 0, 0]
    micros = stretches = 0
    pmc = None  # the latest Pmc event: timestamp, new thread, counters
    entered = None  # timestamp and counters at the thread's switch in
    for line in lines:
        fields = [f.strip() for f in line.split(",")]
        if fields[0] == "Pmc" and fields[1].isdigit():
            pmc = (int(fields[1]), int(fields[2]), [int(x) for x in fields[3:6]])
        elif fields[0] == "CSwitch" and fields[1].isdigit():
            assert len(fields) == 23, f"CSwitch with {len(fields)} fields: {line[:120]}"
            stamp, new, old, on = int(fields[1]), int(fields[3]), int(fields[9]), int(fields[16])
            if tid not in (new, old):
                continue
            assert on == cpu, f"thread {tid} on CPU {on} at {stamp}"
            assert pmc and pmc[0] == stamp and pmc[1] == new, f"no Pmc for the switch at {stamp}"
            if old == tid:
                assert entered, f"thread {tid} left at {stamp} without entering"
                deltas = [b - a for a, b in zip(entered[1], pmc[2])]
                assert min(deltas) >= 0, f"counters fell at {stamp}: {deltas}"
                totals = [t + d for t, d in zip(totals, deltas)]
                micros += stamp - entered[0]
                stretches += 1
                entered = None
            if new == tid:
                assert not entered, f"thread {tid} entered twice at {stamp}"
                entered = (stamp, pmc[2])
    assert not entered, f"thread {tid} still on the CPU when the trace ended"
    assert stretches, f"thread {tid} never ran"
    return ThreadCounters(*totals, micros, stretches)


def etl_counters(etl, tid, cpu):
    with tempfile.TemporaryDirectory() as folder:
        dump = Path(folder) / "dump.txt"
        done = xperf("-i", str(etl), "-o", str(dump), check=False)
        # 0x40103 is xperf's report of events it cannot decode; thread_counters
        # refuses a dump that lost any of the thread's switches.
        assert done.returncode in (0, 0x40103), f"xperf -i {etl.name}: {done.returncode:#x}"
        return thread_counters(dump.read_text(encoding="utf-8", errors="replace").split("\n"), tid, cpu)


def spread(values, labels):
    groups = {}
    for value, label in zip(values, labels):
        groups.setdefault(label, []).append(value)
    medians = [statistics.median(group) for group in groups.values()]
    return (max(medians) - min(medians)) / statistics.mean(medians)


def shuffled_spreads(values, labels, seed=46, count=20000):
    rng = random.Random(seed)
    shuffled = list(labels)
    spreads = []
    for _ in range(count):
        rng.shuffle(shuffled)
        spreads.append(spread(values, shuffled))
    return sorted(spreads)


def cmd_read(args):
    folder = Path(args.dir or OUT / "t2")
    runs = json.loads((folder / "raw.json").read_text(encoding="utf-8"))
    assert all(r["nodes"] == FINGERPRINT for r in runs), "a run off the fingerprint"
    for r in runs:
        c = etl_counters(folder / r["etl"], r["search_tid"], r["cpu"])
        assert abs(c.micros / 1000 - r["ms"]) < 0.02 * r["ms"], f"{r['etl']}: {c.micros} us against {r['ms']} ms"
        r.update(core_cycles_per_node=c.cycles / r["nodes"], instructions_per_node=c.instructions / r["nodes"],
                 misses_per_node=c.misses / r["nodes"], clock_ghz=c.cycles / c.micros / 1000,
                 ipc=c.instructions / c.cycles, stretches=c.stretches, scheduled_us=c.micros)
    (folder / "counters.json").write_text(json.dumps(runs, indent=1), encoding="utf-8")
    labels = [r["exe"] for r in runs]
    lines = [f"{folder.name}: {len(runs)} ETLs; stretches {min(r['stretches'] for r in runs)}"
             f"-{max(r['stretches'] for r in runs)}; clock {min(r['clock_ghz'] for r in runs):.3f}"
             f"-{max(r['clock_ghz'] for r in runs):.3f} GHz; IPC {min(r['ipc'] for r in runs):.3f}"
             f"-{max(r['ipc'] for r in runs):.3f}"]
    for key in ("nps", "cycles_per_node", "core_cycles_per_node", "instructions_per_node", "misses_per_node"):
        values = [r[key] for r in runs]
        cvs = []
        for build in sorted(set(labels)):
            xs = [v for v, label in zip(values, labels) if label == build]
            cvs.append(statistics.stdev(xs) / statistics.mean(xs))
            lines.append(f"  {key:22s} {build}: median {statistics.median(xs):,.3f} cv {cvs[-1]:.4%}")
        observed = spread(values, labels)
        shuffled = shuffled_spreads(values, labels)
        chance = sum(s >= observed for s in shuffled) / len(shuffled)
        lines.append(f"{key}: mean cv {statistics.mean(cvs):.4%}, largest {max(cvs):.4%};"
                     f" spread {observed:.4%}; shuffled median {statistics.median(shuffled):.4%},"
                     f" 95th {shuffled[int(0.95 * len(shuffled))]:.4%}; P(shuffled >= observed) {chance:.4f}")
    (folder / "read.txt").write_text("\n".join(lines) + "\n", encoding="utf-8")
    print("\n".join(lines))
    return 0


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest="cmd", required=True)
    p1 = sub.add_parser("t1", help="unprivileged scheduled-cycle read")
    p1.add_argument("--out")
    p1.set_defaults(func=cmd_t1)
    p2 = sub.add_parser("t2", help="elevated ETW PMC read")
    p2.add_argument("--out")
    p2.add_argument("--dry-run", action="store_true", help="one round at depth 8, no tracing")
    p2.set_defaults(func=cmd_t2)
    p3 = sub.add_parser("read", help="t2's ETLs to counters per node")
    p3.add_argument("--dir")
    p3.set_defaults(func=cmd_read)
    args = parser.parse_args(argv)
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
