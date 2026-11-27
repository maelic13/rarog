"""Tests for tb_activation.py's clock: the optimum a tablebase-root move is held to.

The reference values are the engine's own: a `diag` build of the clock prints
`compute_runtime_limits`' optimum, and `tb_activation_fixture.py` recorded it
over a grid of clocks (`tb_activation_budget_v1.json`). The budget's constants
are read from `src/search/time.rs`, not retyped, and the fixture records the
source it was made from, so a changed clock fails here until the fixture is
made again.
"""

import json
import pathlib
import sys
import tempfile
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import tb_activation

FIXTURE = json.loads((pathlib.Path(__file__).resolve().parent / "tb_activation_budget_v1.json")
                     .read_text(encoding="utf-8"))


class ClockTests(unittest.TestCase):
    def test_the_fixture_was_made_from_this_clock(self):
        # `budget`, its constants and the reserve as they are now: a change
        # to any of them means the engine's values must be recorded again.
        self.assertEqual(tb_activation.clock_source_extracts(), FIXTURE["source"])
        self.assertEqual(tb_activation.clock_constants(), FIXTURE["constants"])

    def test_the_optimum_is_the_engines_on_every_recorded_clock(self):
        # The engine prints to 0.1 ms, so the tool's value rounds to it.
        for case in FIXTURE["cases"]:
            with self.subTest(case=case):
                got = tb_activation.optimum_ms(case["time_ms"], case["inc_ms"], case["overhead_ms"],
                                               case["game_ply"], case["threads"])
                self.assertLessEqual(abs(got - case["optimum_ms"]), 0.05 + 1e-9, got)

    def test_the_grid_reaches_every_branch(self):
        # A grid where the soft bound always wins would pass with a wrong
        # hard bound or reserve: some case must be held by each.
        held_by = set()
        for case in FIXTURE["cases"]:
            t, i, o, p, n = (case[k] for k in ("time_ms", "inc_ms", "overhead_ms", "game_ply", "threads"))
            full = tb_activation.optimum_ms(t, i, o, p, n)
            ceiling = max(t - 2.0 * o - (30.0 if n > 1 else 0.0), 1.0)
            if abs(full - ceiling) < 1e-9 and n > 1:
                held_by.add("smp reserve")
            elif abs(full - ceiling) < 1e-9:
                held_by.add("overhead reserve")
            elif full == 1.0:
                held_by.add("floor")
            else:
                held_by.add("soft")
        self.assertEqual(held_by, {"soft", "floor", "overhead reserve", "smp reserve"})

    def test_the_share_grows_with_the_move_number(self):
        first = tb_activation.optimum_ms(3000, 30, 10, 0)
        late = tb_activation.optimum_ms(3000, 30, 10, 120)
        self.assertGreater(late, 3 * first)

    def test_a_missing_constant_is_refused(self):
        source = tb_activation.TIME_RS.read_text(encoding="utf-8")
        with tempfile.TemporaryDirectory() as tmp:
            path = pathlib.Path(tmp) / "time.rs"
            path.write_text(source.replace("const HARD_SCALE: f64", "const HARD_SCALE_X: f64"), encoding="utf-8")
            with self.assertRaises(SystemExit):
                tb_activation.clock_constants(path)
            path.write_text(source.replace("fn budget(", "fn budget_x("), encoding="utf-8")
            with self.assertRaises(SystemExit):
                tb_activation.clock_source_extracts(path)


if __name__ == "__main__":
    unittest.main()
