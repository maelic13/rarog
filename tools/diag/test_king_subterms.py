"""Tests for king_subterms.py: the `ksdump` parser against real instrumented
output, the danger-index identity, and the sub-terms against the printed
`King safety` row."""

import unittest

import king_subterms as ks
import donor_residual as dr

# `eval` of 1q6/3k4/p2p3P/2p1ppr1/P1P5/1P3Qp1/4R1P1/6K1 w - - 0 18 on the
# instrumented 9587eeeb build (2026-10-06).
BLOCK = """\
     Term    |    White    |    Black    |    Total
             |   MG    EG  |   MG    EG  |   MG    EG
 ------------+-------------+-------------+------------
    Material |  ----  ---- |  ----  ---- | -0.07 -1.08
   Imbalance |  ----  ---- |  ----  ---- | -0.09 -0.09
       Pawns |  0.17 -0.29 |  0.47 -0.15 | -0.30 -0.14
     Knights |  0.00  0.00 |  0.00  0.00 |  0.00  0.00
     Bishops |  0.00  0.00 |  0.00  0.00 |  0.00  0.00
       Rooks |  0.17  0.03 |  0.00  0.00 |  0.17  0.03
      Queens |  0.00  0.00 |  0.00  0.00 |  0.00  0.00
    Mobility |  0.42  1.09 |  0.37  1.10 |  0.05 -0.01
 King safety | -1.17 -0.60 | -0.66 -0.19 | -0.51 -0.41
     Threats |  0.27  0.27 |  0.67  0.56 | -0.40 -0.29
      Passed |  1.34  1.62 | -0.04  0.17 |  1.38  1.45
       Space |  0.00  0.00 |  0.00  0.00 |  0.00  0.00
    Winnable |  ----  ---- |  ----  ---- |  0.00 -0.17
 ------------+-------------+-------------+------------
       Total |  ----  ---- |  ----  ---- |  0.23 -0.69

Final evaluation: -0.25 (white side)
ksdump v -52 w -85 -73 1 561 76 35 0 10 17 3 44 2 0 0 0 0 -10 0 0 63 0 0 0 0 9 1 b -41 -5 1 311 23 19 0 9 15 1 10 1 0 0 1 0 10 0 0 30 0 0 0 0 9 1 win 34 0 -34 47 -109 1 64 64 5 -3 1 0 0 2 0 41 -82 0 0 0
"""


class ParserTests(unittest.TestCase):
    def test_parses_both_sides_and_winnable(self):
        white, black, win = ks.parse_ksdump(BLOCK)
        self.assertEqual(white["kd"], 561)
        self.assertEqual(white["weak"], 2)
        self.assertEqual(black["kd"], 311)
        self.assertEqual(black["flank_att"], 9)
        self.assertEqual(win["complexity"], 34)
        self.assertEqual(win["sf"], 64)
        self.assertEqual(win["branch"], 5)
        self.assertEqual(win["phase"], 41)
        self.assertEqual(win["final"], -52)

    def test_no_dump_returns_none(self):
        self.assertIsNone(ks.parse_ksdump("Total evaluation: none (in check)"))

    def test_wrong_width_raises(self):
        with self.assertRaises(ValueError):
            ks.parse_ksdump("ksdump v 1 w 1 2 3")


class ArithmeticTests(unittest.TestCase):
    def test_danger_components_sum_to_the_index(self):
        white, black, _ = ks.parse_ksdump(BLOCK)
        self.assertTrue(ks.danger_identity_holds(white))
        self.assertTrue(ks.danger_identity_holds(black))
        comps = ks.danger_components(white)
        self.assertEqual(comps["attackers"], 132)
        self.assertEqual(comps["weak_ring"], 370)
        self.assertEqual(comps["flank_sq"], 37)
        self.assertEqual(comps["flank_defense"], -68)

    def test_identity_fails_on_a_corrupted_index(self):
        white, _, _ = ks.parse_ksdump(BLOCK)
        white["kd"] += 1
        self.assertFalse(ks.danger_identity_holds(white))

    def test_parts_reproduce_the_king_row(self):
        white, black, _ = ks.parse_ksdump(BLOCK)
        terms, _ = dr.parse_sf11_trace(BLOCK)
        self.assertEqual(terms["King safety"], (-0.51, -0.41))
        self.assertTrue(ks.king_row_matches(white, black, -0.51, -0.41))
        self.assertFalse(ks.king_row_matches(white, black, -0.40, -0.41))

    def test_danger_score_is_zero_at_or_below_100(self):
        mg, eg = ks.danger_score([100, 101, 561])
        self.assertEqual(list(mg), [0.0, 2.0, 76.0])
        self.assertEqual(list(eg), [0.0, 6.0, 35.0])

    def test_skipped_king_has_zero_parts(self):
        side = {f: 0 for f in ks.KING_FIELDS}
        self.assertTrue(ks.danger_identity_holds(side))
        self.assertTrue(ks.king_row_matches(side, side, 0.0, 0.0))


if __name__ == "__main__":
    unittest.main()
