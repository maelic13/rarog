"""Tests for the two-step pooled-PGO NPS read.

Each test builds a small archived read whose right answer is known and that a
plausible mistake would get wrong: a mean where a median is due, a step-1
threshold read against the wrong bound, a disturbed cycle not noticed, a
manifest hash not checked.
"""

import hashlib
import json
import pathlib
import sys
import tempfile
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))
import nps_read

BENCH = 2_000_000.0
HEAD = 11_171_726


def record(base, cand, no_regression=False):
    """`base` and `cand`: per build, the best-of-3 reading of each cycle. The
    other two runs of a reading are made slower, so the best is the given one."""
    def arm(rows):
        return rows

    cycles = len(base[0])
    out = {"label": "test", "no_regression": no_regression,
           "arms": {"base": {"pool": "p", "builds": [f"b{i}" for i in range(len(base))]},
                    "cand": {"pool": "q", "builds": [f"c{i}" for i in range(len(cand))]}},
           "cycles": []}
    for c in range(cycles):
        readings = {"base": {}, "cand": {}}
        for name, rows in (("base", arm(base)), ("cand", arm(cand))):
            for i, row in enumerate(rows):
                best = row[c]
                readings[name][str(i)] = [{"nodes": HEAD, "ms": 1, "nps": int(v)}
                                          for v in (best * 0.97, best, best * 0.95)]
        out["cycles"].append({"cycle": c, "cpu_before": 1.0, "readings": readings})
    return out


class EstimateTests(unittest.TestCase):
    def test_delta_is_the_difference_of_means_of_per_build_medians(self):
        # One cycle of one base build is 10% off; the median over three cycles ignores it.
        base = [[BENCH, BENCH * 1.10, BENCH], [BENCH] * 3, [BENCH] * 3, [BENCH] * 3]
        cand = [[BENCH * 1.02] * 3] * 4
        est = nps_read.estimate(record(base, cand))
        self.assertAlmostEqual(est["delta_percent"], 2.0, places=6)
        self.assertEqual(est["df"], 6)

    def test_interval_follows_the_spread_of_the_per_build_medians(self):
        base = [[BENCH * (1 + d)] * 2 for d in (-0.002, 0.002, -0.001, 0.001)]
        cand = [[BENCH * 1.01 * (1 + d)] * 2 for d in (-0.002, 0.002, -0.001, 0.001)]
        est = nps_read.estimate(record(base, cand))
        # Pooled per-build sd 0.00183 of the level; se = sd * sqrt(1/4 + 1/4).
        self.assertAlmostEqual(est["se_percent"], 100 * 0.0018257 * (0.5 ** 0.5) * 1.0 / 1.0, places=2)
        self.assertAlmostEqual(est["high_percent"] - est["low_percent"], 2 * 2.447 * est["se_percent"], places=6)

    def test_a_shifted_cycle_is_flagged_in_step_one_and_absorbed_by_six(self):
        quiet = [[BENCH] * 6] * 4
        shifted = [[BENCH, BENCH * 0.97] + [BENCH] * 4] * 4
        two = record([r[:2] for r in quiet], [r[:2] for r in shifted])
        self.assertEqual(nps_read.estimate(two)["disturbed"], [("cand", 1)])
        self.assertTrue(nps_read.decide(nps_read.estimate(two), False).startswith("disturbed"))
        six = record(quiet, shifted)
        est = nps_read.estimate(six)
        self.assertEqual(est["disturbed"], [("cand", 1)])
        self.assertAlmostEqual(est["delta_percent"], 0.0, places=6)
        self.assertEqual(nps_read.decide(est, False), "closed NO_CHANGE")
        # The same cycle disturbed on both arms is one disturbed cycle, not two;
        # three distinct disturbed cycles end the read.
        both = record([[BENCH, BENCH * 0.97] + [BENCH] * 4] * 4, shifted)
        self.assertEqual(nps_read.decide(nps_read.estimate(both), False), "closed NO_CHANGE")
        three = record(quiet, [[BENCH * 0.97, BENCH * 0.97, BENCH * 0.97] + [BENCH] * 3] * 4)
        self.assertTrue(nps_read.decide(nps_read.estimate(three), False).startswith("disturbed"))

    def test_step_one_thresholds(self):
        for gain, expected in ((0.012, "accepted by step 1"), (0.0005, "closed NO_CHANGE by step 1"),
                               (0.005, "between: run step 2 (--extend, four more cycles)")):
            est = nps_read.estimate(record([[BENCH] * 2] * 4, [[BENCH * (1 + gain)] * 2] * 4))
            self.assertEqual(nps_read.decide(est, False), expected, gain)

    def test_step_two_needs_the_floor_and_a_lower_bound_above_zero(self):
        spread = (-0.004, 0.004, -0.002, 0.002)
        base = [[BENCH * (1 + d)] * 6 for d in spread]
        wide = nps_read.estimate(record(base, [[BENCH * 1.006 * (1 + d)] * 6 for d in spread]))
        self.assertTrue(wide["low_percent"] < 0 < wide["delta_percent"])
        self.assertEqual(nps_read.decide(wide, False), "closed NO_CHANGE")
        tight = nps_read.estimate(record([[BENCH] * 6] * 4, [[BENCH * 1.006] * 6] * 4))
        self.assertEqual(nps_read.decide(tight, False), "accepted")

    def test_no_regression_mirrors_the_thresholds(self):
        est = nps_read.estimate(record([[BENCH] * 2] * 4, [[BENCH * 0.9995] * 2] * 4))
        self.assertEqual(nps_read.decide(est, True), "pass: regression excluded by step 1")
        est = nps_read.estimate(record([[BENCH] * 2] * 4, [[BENCH * 0.988] * 2] * 4))
        self.assertEqual(nps_read.decide(est, True), "fail: regression by step 1")
        est = nps_read.estimate(record([[BENCH] * 2] * 4, [[BENCH * 0.995] * 2] * 4))
        self.assertTrue(nps_read.decide(est, True).startswith("between"))


class WireTests(unittest.TestCase):
    def test_a_run_at_another_fingerprint_is_refused(self):
        text = "\n".join(f"run {i}/3  nodes 11171726  time 5000ms  nps 2234345" for i in (1, 2, 3))
        self.assertEqual(len(nps_read.parse_runs(text, "x", HEAD)), 3)
        with self.assertRaises(SystemExit):
            nps_read.parse_runs(text.replace("11171726", "11171725", 1), "x", HEAD)
        with self.assertRaises(SystemExit):
            nps_read.parse_runs(text.rsplit("\n", 1)[0], "x", HEAD)
        with self.assertRaises(SystemExit):
            nps_read.parse_runs(text, "x", 7_601_220)

    @staticmethod
    def write_pool(directory, fingerprint, count, tier="pext"):
        rows = []
        for i in range(1, count + 1):
            exe = directory / f"{tier}-{i}.exe"
            exe.write_bytes(f"engine {i}".encode())
            rows.append(f"{hashlib.sha256(exe.read_bytes()).hexdigest().upper()}  {tier}-{i}.exe")
        (directory / f"manifest-{tier}.txt").write_text(
            f"fingerprint   : {fingerprint} (verified)\n\n" + "\n".join(rows) + "\n", encoding="utf-8")

    def test_a_pool_whose_bytes_differ_from_the_manifest_is_refused(self):
        with tempfile.TemporaryDirectory() as tmp:
            directory = pathlib.Path(tmp)
            self.write_pool(directory, HEAD, 2)
            self.assertEqual(nps_read.pool(directory),
                             [directory / "pext-1.exe", directory / "pext-2.exe"])
            (directory / "pext-1.exe").write_bytes(b"not the same engine")
            with self.assertRaises(SystemExit):
                nps_read.pool(directory)

    def test_a_pool_of_any_tier_is_read_and_two_manifests_are_refused(self):
        with tempfile.TemporaryDirectory() as tmp:
            directory = pathlib.Path(tmp)
            self.write_pool(directory, HEAD, 2, tier="sse42")
            self.assertEqual(nps_read.pool(directory),
                             [directory / "sse42-1.exe", directory / "sse42-2.exe"])
            self.write_pool(directory, HEAD, 2, tier="base")
            with self.assertRaises(SystemExit):
                nps_read.pool(directory)
        with tempfile.TemporaryDirectory() as tmp:
            with self.assertRaises(SystemExit):
                nps_read.pool(pathlib.Path(tmp))

    def test_each_pool_is_read_at_its_own_fingerprint(self):
        """A release baseline (2.4.0 benches 7,601,220) is read against the head."""
        with tempfile.TemporaryDirectory() as tmp:
            directory = pathlib.Path(tmp)
            self.write_pool(directory, 7_601_220, 3)
            self.assertEqual(len(nps_read.pool(directory)), 3)
            self.assertEqual(nps_read.pool_fingerprint(directory), 7_601_220)

    def test_a_one_build_pool_is_refused(self):
        with tempfile.TemporaryDirectory() as tmp:
            directory = pathlib.Path(tmp)
            self.write_pool(directory, HEAD, 1)
            with self.assertRaises(SystemExit):
                nps_read.pool(directory)

    def test_each_arm_is_read_at_its_own_fingerprint(self):
        """The wiring, not the helper: a cycle hands every base build the
        base pool's fingerprint and every candidate build the candidate's.
        Reading both arms at one fingerprint would refuse every honest base
        build (or accept a wrong one) and this would catch it."""
        seen = []

        def fake_read(exe, fingerprint, options=()):
            seen.append((str(exe), fingerprint))
            return [{"nodes": fingerprint, "ms": 1, "nps": 1}]

        with tempfile.TemporaryDirectory() as tmp:
            out = pathlib.Path(tmp)
            builds = {"base": [out / "b0", out / "b1", out / "b2"], "cand": [out / "c0", out / "c1"]}
            fingerprints = {"base": 7_601_220, "cand": HEAD}
            rec = {"cycles": []}
            original = nps_read.read
            nps_read.read = fake_read
            try:
                nps_read.run_cycles(rec, out, builds, 2, fingerprints)
            finally:
                nps_read.read = original
        self.assertEqual(len(seen), 2 * 5)
        for exe, fingerprint in seen:
            arm = "base" if pathlib.Path(exe).name.startswith("b") else "cand"
            self.assertEqual(fingerprint, fingerprints[arm], exe)
        self.assertEqual(sorted(rec["cycles"][0]["readings"]["base"]), ["0", "1", "2"])
        self.assertEqual(rec["cycles"][0]["readings"]["base"]["1"][0]["nodes"], 7_601_220)


class TreeWarningTests(unittest.TestCase):
    def test_equal_fingerprints_carry_no_warning(self):
        self.assertIsNone(nps_read.tree_warning({"base": HEAD, "cand": HEAD}))
        rec = record([[BENCH, BENCH]] * 2, [[BENCH, BENCH]] * 2)
        for arm in ("base", "cand"):
            rec["arms"][arm]["fingerprint"] = HEAD
        self.assertNotIn("WARNING", nps_read.report(rec, False))

    def test_different_trees_are_named_in_the_report(self):
        warning = nps_read.tree_warning({"base": 7_601_220, "cand": HEAD})
        self.assertIn("different trees", warning)
        self.assertIn("time-to-depth", warning)
        rec = record([[BENCH, BENCH]] * 2, [[BENCH, BENCH]] * 2)
        rec["arms"]["base"]["fingerprint"] = 7_601_220
        rec["arms"]["cand"]["fingerprint"] = HEAD
        self.assertEqual(nps_read.report(rec, False).splitlines()[-1], warning)

    def test_an_archive_without_fingerprints_is_not_warned_about(self):
        rec = record([[BENCH, BENCH]] * 2, [[BENCH, BENCH]] * 2)
        self.assertNotIn("WARNING", nps_read.report(rec, False))


class OptionArmTests(unittest.TestCase):
    """Arms that differ by UCI options on one tune build."""

    def test_options_are_set_before_the_bench(self):
        self.assertEqual(nps_read.bench_input(()), "bench 13 3\n")
        self.assertEqual(nps_read.bench_input([("LazyMargin", "2000")]),
                         "setoption name LazyMargin value 2000\nbench 13 3\n")
        self.assertEqual(nps_read.parse_option("LazyMargin=2000"), ("LazyMargin", "2000"))
        for bad in ("LazyMargin", "=2000", "LazyMargin="):
            with self.assertRaises(SystemExit):
                nps_read.parse_option(bad)

    def test_each_arm_runs_with_its_own_options_and_fingerprint(self):
        seen = []

        def fake_read(exe, fingerprint, options=()):
            seen.append((pathlib.Path(exe).name, fingerprint, list(options)))
            return [{"nodes": fingerprint, "ms": 1, "nps": 1}]

        with tempfile.TemporaryDirectory() as tmp:
            out = pathlib.Path(tmp)
            same = [out / "t0", out / "t1"]
            original = nps_read.read
            nps_read.read = fake_read
            try:
                nps_read.run_cycles({"cycles": []}, out, {"base": same, "cand": same}, 1,
                                    {"base": HEAD, "cand": 11_000_000},
                                    {"base": [], "cand": [("LazyMargin", "2000")]})
            finally:
                nps_read.read = original
        self.assertEqual(len(seen), 4)
        self.assertIn(("t0", HEAD, []), seen)
        self.assertIn(("t0", 11_000_000, [("LazyMargin", "2000")]), seen)

    def test_one_pool_needs_differing_options_and_two_pools_distinct_bytes(self):
        with tempfile.TemporaryDirectory() as tmp:
            p = pathlib.Path(tmp) / "p"
            q = pathlib.Path(tmp) / "q"
            p.mkdir()
            q.mkdir()
            WireTests.write_pool(p, HEAD, 2)
            WireTests.write_pool(q, HEAD, 2)  # the same bytes as p
            same = {"base": nps_read.pool(p), "cand": nps_read.pool(p)}
            nps_read.check_arms(p, p, same, {"base": [], "cand": [("LazyMargin", "2000")]})
            with self.assertRaises(SystemExit):
                nps_read.check_arms(p, p, same, {"base": [], "cand": []})
            twin = {"base": nps_read.pool(p), "cand": nps_read.pool(q)}
            with self.assertRaises(SystemExit):
                nps_read.check_arms(p, q, twin, {"base": [], "cand": [("LazyMargin", "2000")]})


class HostTests(unittest.TestCase):
    def test_idle_host_refuses_nothing(self):
        self.assertEqual(nps_read.idle_refusal({"busy_percent": 3.1, "busy_processes": []}), [])

    def test_load_over_the_ceiling_or_a_competing_process_refuses(self):
        self.assertEqual(len(nps_read.idle_refusal({"busy_percent": 15.1, "busy_processes": []})), 1)
        self.assertEqual(nps_read.idle_refusal({"busy_percent": 15.0, "busy_processes": []}), [])
        both = nps_read.idle_refusal({"busy_percent": 40.0, "busy_processes": ["cargo:12"]})
        self.assertEqual(len(both), 2)
        self.assertIn("cargo:12", both[0])

    def test_unreadable_cpu_refuses_only_for_processes(self):
        self.assertEqual(nps_read.idle_refusal({"busy_percent": None, "busy_processes": []}), [])
        self.assertEqual(len(nps_read.idle_refusal({"busy_percent": None, "busy_processes": ["rarog:7"]})), 1)

    def test_a_busy_host_is_refused_unless_waived_and_both_are_archived(self):
        busy = {"busy_percent": 50.0, "busy_processes": []}
        saved = nps_read.host_state
        nps_read.host_state = lambda: dict(busy)
        try:
            record = {}
            with self.assertRaises(SystemExit):
                nps_read.check_host(record, allow_busy=False)
            self.assertFalse(record["host_checks"][-1]["waived"])
            nps_read.check_host(record, allow_busy=True)
            self.assertTrue(record["host_checks"][-1]["waived"])
            self.assertEqual(record["host_checks"][-1]["busy_percent"], 50.0)
        finally:
            nps_read.host_state = saved

    def quiet_wait(self, loads, seconds, allow_busy=False, step=5.0):
        """wait_quiet over a scripted host: one load per reading, `step`
        seconds apart, the last repeating; "proc" is a quiet load with an
        engine running."""
        saved = nps_read.host_state
        calls = {"n": 0}

        def state():
            load = loads[min(calls["n"], len(loads) - 1)]
            calls["n"] += 1
            if load == "proc":
                return {"busy_percent": 2.0, "busy_processes": ["rarog:9"]}
            return {"busy_percent": load, "busy_processes": []}

        nps_read.host_state = state
        record = {}
        try:
            nps_read.wait_quiet(record, allow_busy, seconds, clock=lambda: step * calls["n"])
        finally:
            nps_read.host_state = saved
        return record, calls["n"]

    def test_the_quiet_wait_ends_once_the_host_has_stayed_quiet_long_enough(self):
        record, n = self.quiet_wait([3.0], 60)
        self.assertEqual(n, 13)  # quiet from the first reading, 60 s later at the 13th
        self.assertTrue(record["quiet_waits"][-1]["quiet"])

    def test_a_burst_or_a_competing_process_restarts_the_quiet_wait(self):
        _, quiet_throughout = self.quiet_wait([3.0], 60)
        _, burst = self.quiet_wait([3.0, 3.0, 3.0, 25.0, 3.0], 60)
        _, process = self.quiet_wait([3.0, 3.0, 3.0, "proc", 3.0], 60)
        self.assertEqual(burst, quiet_throughout + 4)
        self.assertEqual(process, quiet_throughout + 4)

    def test_the_quiet_ceiling_is_ten_percent(self):
        _, at = self.quiet_wait([10.0], 60)
        _, over = self.quiet_wait([10.1, 3.0], 60)
        self.assertEqual(over, at + 1)

    def test_a_host_never_quiet_is_refused_unless_waived_and_both_are_archived(self):
        with self.assertRaises(SystemExit):
            self.quiet_wait([30.0], 60)
        record, _ = self.quiet_wait([30.0], 60, allow_busy=True)
        self.assertFalse(record["quiet_waits"][-1]["quiet"])
        self.assertTrue(record["quiet_waits"][-1]["waived"])

    def test_the_warm_up_bench_is_short_and_follows_the_options(self):
        self.assertEqual(nps_read.bench_input([("Hash", "16")], f"bench {nps_read.WARMUP_DEPTH} 1"),
                         "setoption name Hash value 16\nbench 8 1\n")


class UnequalPoolTests(unittest.TestCase):
    def test_three_builds_against_four_use_five_degrees_of_freedom(self):
        base = [[BENCH * f] * 2 for f in (0.999, 1.000, 1.001)]
        cand = [[BENCH * 1.02 * f] * 2 for f in (0.999, 1.000, 1.001, 1.0005)]
        est = nps_read.estimate(record(base, cand))
        self.assertEqual(est["df"], 5)
        self.assertEqual(est["t"], 2.571)
        self.assertEqual(est["builds"], [3, 4])


FAKE_ENGINE = r"""
import sys
log = open(sys.argv[1], "a", encoding="utf-8")
for line in sys.stdin:
    line = line.strip()
    log.write(line + "\n")
    log.flush()
    if line == "uci":
        print("uciok", flush=True)
    elif line == "isready":
        print("readyok", flush=True)
    elif line.startswith("go"):
        print("info depth 1 nodes 20 time 0", flush=True)
        if "nodes" in line:
            print("info depth 7 seldepth 9 score cp 12 nodes 900 nps 1 time 1 pv e2e4", flush=True)
        print("bestmove e2e4", flush=True)
    elif line == "quit":
        break
"""


class GamePositionTests(unittest.TestCase):
    def frozen_set(self, directory, tamper=False):
        path = pathlib.Path(directory) / "set.jsonl"
        data = b'{"fen": "8/8/8/8/8/8/4K3/k7 w - - 0 1"}\n{"fen": "8/8/8/8/8/8/4K3/1k6 w - - 0 1"}\n'
        path.write_bytes(data)
        digest = hashlib.sha256(data).hexdigest()
        (pathlib.Path(directory) / "set.jsonl.manifest.txt").write_text(
            f"nodes       : 186000\nset_sha256  : {digest}\n", encoding="utf-8")
        if tamper:
            path.write_bytes(data + b'{"fen": "8/8/8/8/8/8/4K3/2k5 w - - 0 1"}\n')
        return path

    def test_a_set_is_read_with_its_manifests_budget_and_refused_when_its_bytes_differ(self):
        with tempfile.TemporaryDirectory() as tmp:
            workload = nps_read.game_positions(self.frozen_set(tmp))
            self.assertEqual(workload["nodes"], 186000)
            self.assertEqual(len(workload["fens"]), 2)
        with tempfile.TemporaryDirectory() as tmp:
            with self.assertRaises(SystemExit):
                nps_read.game_positions(self.frozen_set(tmp, tamper=True))

    def test_the_budget_search_is_timed_after_the_resets_and_a_warm_up_search(self):
        with tempfile.TemporaryDirectory() as tmp:
            tmp = pathlib.Path(tmp)
            (tmp / "fake.py").write_text(FAKE_ENGINE, encoding="utf-8")
            log = tmp / "log.txt"
            engine = tmp / "fake.cmd"
            engine.write_text(f'@"{sys.executable}" "{tmp / "fake.py"}" "{log}"\n', encoding="utf-8")
            row = nps_read.search_position(engine, "8/8/8/8/8/8/4K3/k7 w - - 0 1", 1234, 64, [("Foo", "3")])
            sent = log.read_text(encoding="utf-8").splitlines()
        fen = "position fen 8/8/8/8/8/8/4K3/k7 w - - 0 1"
        self.assertEqual(sent, ["uci", "setoption name Threads value 1", "setoption name Hash value 64",
                                "setoption name Foo value 3", "ucinewgame", "isready", fen, "go depth 1",
                                "ucinewgame", "isready", fen, "go nodes 1234", "quit"])
        # The timed search's own last report, not the warm-up's.
        self.assertEqual((row["best"], row["last_nodes"], row["depth"]), ("e2e4", 900, 7))
        self.assertGreater(row["seconds"], 0)

    def test_a_reading_is_the_budget_over_the_summed_time(self):
        rows = iter([{"seconds": 0.5, "best": "a", "last_nodes": 1, "depth": 9},
                     {"seconds": 1.5, "best": "b", "last_nodes": 2, "depth": 9}])
        original = nps_read.search_position
        nps_read.search_position = lambda *a, **k: next(rows)
        try:
            (run,) = nps_read.read_games(pathlib.Path("x.exe"),
                                         {"fens": ["f1", "f2"], "nodes": 1000, "hash": 64})
        finally:
            nps_read.search_position = original
        self.assertEqual(run["nps"], 1000)  # 2,000 nodes over 2 s
        self.assertEqual(run["tree"], [["a", 1], ["b", 2]])

    def test_a_reading_on_another_tree_is_refused(self):
        record = {"arms": {"base": {}, "cand": {}}}
        nps_read.check_tree(record, "base", [{"tree": [["a", 1], ["b", 2]]}], pathlib.Path("b0"))
        nps_read.check_tree(record, "base", [{"tree": [["a", 1], ["b", 2]]}], pathlib.Path("b1"))
        with self.assertRaises(SystemExit):
            nps_read.check_tree(record, "base", [{"tree": [["a", 1], ["b", 3]]}], pathlib.Path("b2"))

    def test_arms_on_different_game_trees_are_warned_and_equal_ones_are_not(self):
        record = {"arms": {"base": {"game_tree": [["a", 1]]}, "cand": {"game_tree": [["a", 1]]}}}
        self.assertIsNone(nps_read.game_tree_warning(record))
        record["arms"]["cand"]["game_tree"] = [["b", 1]]
        self.assertIn("different trees on 1 of 1", nps_read.game_tree_warning(record))


class LinuxTests(unittest.TestCase):
    """A Linux pool read in WSL: builds without `.exe`, the host measure
    through Windows' PowerShell, the Linux side's processes from `/proc`, and
    the multi-thread cell."""

    def test_a_linux_pool_without_exe_names_is_read(self):
        with tempfile.TemporaryDirectory() as tmp:
            directory = pathlib.Path(tmp)
            rows = []
            for i in (1, 2):
                exe = directory / f"pext-{i}"
                exe.write_bytes(f"linux engine {i}".encode())
                rows.append(f"{hashlib.sha256(exe.read_bytes()).hexdigest().upper()}  pext-{i}")
            (directory / "manifest-pext.txt").write_text(
                f"fingerprint   : {HEAD} (verified)\n\n" + "\n".join(rows) + "\n", encoding="utf-8")
            self.assertEqual(nps_read.pool(directory), [directory / "pext-1", directory / "pext-2"])

    def test_the_harness_shell_is_the_path_s_else_wsl_s_windows_one(self):
        saved = (nps_read.shutil.which, nps_read.WSL_PWSH, nps_read.windows_path)
        with tempfile.TemporaryDirectory() as tmp:
            fake = pathlib.Path(tmp) / "pwsh.exe"
            try:
                nps_read.shutil.which = lambda name: "/usr/bin/pwsh" if name == "pwsh" else None
                self.assertEqual(nps_read.harness_shell(), ("/usr/bin/pwsh", str(nps_read.HARNESS)))
                nps_read.shutil.which = lambda name: None
                nps_read.WSL_PWSH = fake
                self.assertIsNone(nps_read.harness_shell(), "no shell anywhere")
                fake.write_bytes(b"")
                nps_read.windows_path = lambda path: "D:\\code\\rarog\\tools\\harness_common.ps1"
                self.assertEqual(nps_read.harness_shell(),
                                 (str(fake), "D:\\code\\rarog\\tools\\harness_common.ps1"))

                def broken(path):
                    raise nps_read.subprocess.CalledProcessError(1, "wslpath")

                nps_read.windows_path = broken
                self.assertIsNone(nps_read.harness_shell(), "a path WSL cannot name")
            finally:
                nps_read.shutil.which, nps_read.WSL_PWSH, nps_read.windows_path = saved

    def test_the_linux_side_s_competing_processes_are_found_by_name(self):
        with tempfile.TemporaryDirectory() as tmp:
            proc = pathlib.Path(tmp)
            for pid, name in (("12", "cargo"), ("13", "python3"), ("14", "pext-2"), ("15", "rarog"),
                              ("16", "pextra"), ("17", "bash")):
                (proc / pid).mkdir()
                (proc / pid / "comm").write_text(name + "\n", encoding="utf-8")
            (proc / "self").mkdir()
            (proc / "18").mkdir()  # gone before its name was read
            self.assertEqual(nps_read.linux_busy_processes(proc), ["cargo:12", "pext-2:14", "rarog:15"])

    def test_a_multi_thread_cell_checks_no_fingerprint_but_still_three_runs(self):
        text = "\n".join(f"run {i}/3  nodes {n}  time 5000ms  nps 9000000" for i, n in
                         ((1, 45_000_000), (2, 44_100_000), (3, 46_300_000)))
        self.assertEqual([r["nodes"] for r in nps_read.parse_runs(text, "x", None)],
                         [45_000_000, 44_100_000, 46_300_000])
        with self.assertRaises(SystemExit):
            nps_read.parse_runs(text, "x", HEAD)
        with self.assertRaises(SystemExit):
            nps_read.parse_runs(text.rsplit("\n", 1)[0], "x", None)
        with self.assertRaises(SystemExit):
            nps_read.parse_runs(text.replace("45000000", "0", 1), "x", None)

    def test_smp_exactly_when_both_arms_run_more_than_one_thread(self):
        four = {"base": [("Threads", "4")], "cand": [("Threads", "4"), ("LargePages", "false")]}
        one = {"base": [], "cand": [("LargePages", "false")]}
        nps_read.check_threads(four, smp=True)
        nps_read.check_threads(one, smp=False)
        with self.assertRaises(SystemExit):
            nps_read.check_threads(four, smp=False)
        with self.assertRaises(SystemExit):
            nps_read.check_threads(one, smp=True)
        with self.assertRaises(SystemExit):
            nps_read.check_threads({"base": [("Threads", "4")], "cand": []}, smp=True)

    def test_a_multi_thread_cell_reports_levels_and_no_verdict(self):
        base = [[BENCH] * 2, [BENCH * 1.001] * 2]
        cand = [[BENCH * 1.03] * 2, [BENCH * 1.031] * 2]
        cell = record(base, cand)
        cell["smp"] = True
        text = nps_read.report(cell, False)
        self.assertIn("verdict: none: a multi-thread cell", text)
        self.assertNotIn("accepted", text)
        self.assertIn("accepted by step 1", nps_read.report(record(base, cand), False))


if __name__ == "__main__":
    unittest.main()
