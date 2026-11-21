"""Tests for tools/wdl_fit.py: the model's identities, the fit recovering known
constants from synthetic games, the PGN reader's side and filters, and the
check of the engine's constants."""

from __future__ import annotations

import importlib.util
import pathlib
import sys
import tempfile
import unittest

import numpy as np

ROOT = pathlib.Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location("wdl_fit", ROOT / "tools" / "wdl_fit.py")
wf = importlib.util.module_from_spec(spec)
sys.modules["wdl_fit"] = wf
spec.loader.exec_module(wf)

A = [-120.0, 330.0, -300.0, 380.0]
B = [6.0, 14.0, -28.0, 70.0]


class Model(unittest.TestCase):
    def test_the_three_rates_sum_to_one_and_half_win_at_a(self):
        x = np.array([-500.0, -100.0, 0.0, 250.0, 900.0])
        win, draw, loss = wf.probabilities(x, 250.0, 60.0)
        np.testing.assert_allclose(win + draw + loss, 1.0)
        self.assertAlmostEqual(float(wf.probabilities(np.array([250.0]), 250.0, 60.0)[0][0]), 0.5)
        w0, _, l0 = wf.probabilities(np.array([0.0]), 250.0, 60.0)
        self.assertAlmostEqual(float(w0[0]), float(l0[0]))

    def test_material_is_clamped_to_the_fitted_range(self):
        self.assertEqual(float(wf.poly(A, 5)), float(wf.poly(A, 17)))
        self.assertEqual(float(wf.poly(A, 90)), float(wf.poly(A, 78)))


class Fit(unittest.TestCase):
    def test_known_constants_are_recovered_from_synthetic_games(self):
        rng = np.random.default_rng(20261121)
        rows = []
        for m in range(17, 79):
            a, b = float(wf.poly(A, m)), float(wf.poly(B, m))
            x = rng.normal(0, 300, 4000).round()
            win, draw, _ = wf.probabilities(x, a, b)
            u = rng.random(len(x))
            outcome = np.where(u < win, 1, np.where(u < win + draw, 0, -1))
            rows += [(int(s), m, int(o)) for s, o in zip(x, outcome)]
        result = wf.fit(rows, min_count=300)
        for m in (20, 40, 58, 78):
            self.assertAlmostEqual(float(wf.poly(result["a"], m)) / float(wf.poly(A, m)), 1.0, delta=0.03)
            self.assertAlmostEqual(float(wf.poly(result["b"], m)) / float(wf.poly(B, m)), 1.0, delta=0.08)
        band = next(b for b in result["calibration"] if b["printed"] == [100, 125])
        self.assertAlmostEqual(band["observed"][0], band["predicted"][0], delta=0.03)

    def test_too_few_material_counts_refuse_a_cubic(self):
        rows = [(0, 40, 0)] * 500 + [(300, 40, 1)] * 500 + [(-300, 40, -1)] * 500
        with self.assertRaises(SystemExit):
            wf.fit(rows, min_count=300)


PGN = """[Event "t"]
[White "fit"]
[Black "other"]
[Result "1-0"]
[SetUp "1"]
[FEN "4k3/8/8/8/8/8/PPPP4/RN2K3 w - - 0 1"]

1. a3 {s=120 d=9} Kd7 {s=-200 d=9} 2. b3 {s=#5 d=9} Kd6 {s=-50 d=9} 3. c3 {s=19872 d=9} Kd5 {s=0 d=9} 4. d3 {s=-15 d=9} 1-0

[Event "t"]
[White "other"]
[Black "fit"]
[Result "1/2-1/2"]

1. e4 {s=30 d=9} e5 {s=-40 d=9} 1/2-1/2

[Event "t"]
[White "a"]
[Black "b"]
[Result "0-1"]

1. d4 {s=30 d=9} d5 {s=-40 d=9} 0-1

"""


class Reader(unittest.TestCase):
    def test_only_the_named_engines_moves_with_its_side_and_the_filters(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = pathlib.Path(tmp) / "g.pgn"
            path.write_text(PGN, encoding="utf-8")
            rows, counts = wf.read_positions(path, "fit", max_abs=4000)
        # White's four moves in game 1 (one mate, one at the clamp), Black's one in game 2.
        self.assertEqual(counts["games"], 2)
        self.assertEqual(counts["moves"], 5)
        self.assertEqual((counts["mate"], counts["beyond_max_abs"]), (1, 1))
        # Game 1's start: four pawns, a rook and a knight for White, 4 + 5 + 3.
        self.assertEqual(rows[0], (120, 12, 1))
        self.assertEqual(rows[1], (-15, 12, 1))
        self.assertEqual(rows[2], (-40, 78, 0))


class RustCheck(unittest.TestCase):
    def test_the_constants_are_read_from_the_source(self):
        source = ("const WDL_A: [f64; 4] = [-1.5, 2.25, -3.0, 400.125];\n"
                  "const WDL_B: [f64; 4] = [1.0, 2.0, 3.0, 4.0];\n")
        self.assertEqual(wf.rust_constants(source), ([-1.5, 2.25, -3.0, 400.125], [1.0, 2.0, 3.0, 4.0]))
        with self.assertRaises(SystemExit):
            wf.rust_constants("const WDL_A: [f64; 4] = [1.0, 2.0, 3.0, 4.0];\n")


if __name__ == "__main__":
    unittest.main()
