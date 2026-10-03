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
    def write_pool(directory, fingerprint, count):
        rows = []
        for i in range(1, count + 1):
            exe = directory / f"pext-{i}.exe"
            exe.write_bytes(f"engine {i}".encode())
            rows.append(f"{hashlib.sha256(exe.read_bytes()).hexdigest().upper()}  pext-{i}.exe")
        (directory / "manifest-pext.txt").write_text(
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

        def fake_read(exe, fingerprint):
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


class UnequalPoolTests(unittest.TestCase):
    def test_three_builds_against_four_use_five_degrees_of_freedom(self):
        base = [[BENCH * f] * 2 for f in (0.999, 1.000, 1.001)]
        cand = [[BENCH * 1.02 * f] * 2 for f in (0.999, 1.000, 1.001, 1.0005)]
        est = nps_read.estimate(record(base, cand))
        self.assertEqual(est["df"], 5)
        self.assertEqual(est["t"], 2.571)
        self.assertEqual(est["builds"], [3, 4])


if __name__ == "__main__":
    unittest.main()
