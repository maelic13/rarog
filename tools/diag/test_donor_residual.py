"""Tests for donor_residual.py: the parsers against real engine output, the
phase arithmetic, and the model comparison against a planted signal."""

import unittest

import numpy as np

import donor_residual as dr

SF11_BLOCK = """\
     Term    |    White    |    Black    |    Total
             |   MG    EG  |   MG    EG  |   MG    EG
 ------------+-------------+-------------+------------
    Material |  ----  ---- |  ----  ---- | -0.29  0.42
   Imbalance |  ----  ---- |  ----  ---- |  0.00  0.00
       Pawns |  0.65 -0.04 |  0.67 -0.06 | -0.02  0.01
     Knights | -0.16 -0.19 | -0.20 -0.23 |  0.03  0.04
     Bishops | -0.21 -0.56 | -0.07 -0.63 | -0.15  0.08
       Rooks | -0.24 -0.05 |  0.00  0.00 | -0.24 -0.05
      Queens |  0.00  0.00 |  0.00  0.00 |  0.00  0.00
    Mobility |  0.67  1.20 |  0.35  0.71 |  0.32  0.49
 King safety |  0.38 -0.13 |  0.36 -0.05 |  0.02 -0.08
     Threats |  0.44  0.36 |  0.63  0.51 | -0.19 -0.15
      Passed |  0.00  0.00 |  0.00  0.00 |  0.00  0.00
       Space |  0.69  0.00 |  0.46  0.00 |  0.23  0.00
  Initiative |  ----  ---- |  ----  ---- |  0.00  0.14
 ------------+-------------+-------------+------------
       Total |  ----  ---- |  ----  ---- | -0.29  0.91

Total evaluation: -0.15 (white side)
"""

ORACLE_RAROG = """\
Rarog 2.3.2 HCE through Stockfish 9587eeeb search
Final evaluation: -0.99 (white side)
"""

ORACLE_CONTROL = """\
Use Rarog HCE is disabled.

Original Stockfish HCE final evaluation: -0.24 (white side)
"""


class Parsers(unittest.TestCase):
    def test_sf11_trace_reads_every_term_and_the_total(self):
        terms, total = dr.parse_sf11_trace(SF11_BLOCK)
        self.assertEqual(total, -0.15)
        for term in dr.SF11_TERMS:
            self.assertIn(term, terms)
        self.assertEqual(terms["Material"], (-0.29, 0.42))
        self.assertEqual(terms["King safety"], (0.02, -0.08))
        self.assertEqual(terms["Initiative"], (0.0, 0.14))
        self.assertEqual(terms["Total"], (-0.29, 0.91))

    def test_sf11_in_check_has_no_total(self):
        terms, total = dr.parse_sf11_trace("Total evaluation: none (in check)\n")
        self.assertIsNone(total)
        self.assertEqual(terms, {})

    def test_oracle_totals_for_both_evaluators(self):
        self.assertEqual(dr.parse_oracle_total(ORACLE_RAROG), -0.99)
        self.assertEqual(dr.parse_oracle_total(ORACLE_CONTROL), -0.24)
        self.assertIsNone(dr.parse_oracle_total("Final evaluation: none (in check)"))

    def test_every_reported_family_term_is_a_trace_term(self):
        listed = [t for terms in dr.FAMILIES.values() for t in terms]
        self.assertEqual(sorted(listed), sorted(dr.SF11_TERMS))


class Phase(unittest.TestCase):
    def test_phase_limits(self):
        start = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        self.assertEqual(dr.sf11_phase(start), 128)
        self.assertEqual(dr.sf11_phase("8/8/4k3/8/8/4K3/4P3/8 w - - 0 1"), 0)
        # Two rooks and two minors a side: between the limits.
        mid = "r3k2r/1b6/8/8/8/8/1B6/R3K2R w - - 0 1"
        self.assertTrue(0 < dr.sf11_phase(mid) < 128)

    def test_queen_detection_reads_the_board_field_only(self):
        self.assertTrue(dr.has_queen("3qk3/8/8/8/8/8/8/4K3 w - - 0 1"))
        self.assertFalse(dr.has_queen("4k3/8/8/8/8/8/8/4K3 w KQkq - 0 1"))


class PlantedSignal(unittest.TestCase):
    def test_a_real_signal_is_reported_and_a_null_one_is_not(self):
        rng = np.random.default_rng(7)
        n = 40_000
        base = rng.normal(0.0, 150.0, n)
        real = rng.normal(0.0, 60.0, n)
        null = rng.normal(0.0, 60.0, n)
        p = 1.0 / (1.0 + np.exp(-(base + real) / 170.0))
        y = (rng.random(n) < p).astype(float)

        def error(columns):
            return (y - dr.held_out_predictions(np.column_stack(columns), y)) ** 2

        e_base = error([base])
        gain_real = e_base - error([base, real])
        gain_null = e_base - error([base, null])
        se_real = gain_real.std(ddof=1) / np.sqrt(n)
        se_null = gain_null.std(ddof=1) / np.sqrt(n)
        self.assertGreater(gain_real.mean(), 10 * se_real)
        self.assertLess(abs(gain_null.mean()), 4 * se_null)

    def test_fractional_labels_fit(self):
        rng = np.random.default_rng(11)
        x = rng.normal(0.0, 1.0, (5_000, 1))
        y = np.where(x[:, 0] > 0.5, 1.0, np.where(x[:, 0] < -0.5, 0.0, 0.5))
        w = dr.fit_logistic(x, y)
        self.assertGreater(w[1], 0.0)
        self.assertAlmostEqual(float(dr.predict(w, np.zeros((1, 1)))[0]), 0.5, delta=0.05)


if __name__ == "__main__":
    unittest.main()
