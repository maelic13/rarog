"""Tests for ocb_free_passers.py: the free and bishop-free passer definitions
on crafted opposite-bishop positions."""

import unittest

import ocb_free_passers as fp
import ocb_scale_screen as oc


def kinds(fen):
    return fp.passer_kinds(oc.parse_board(fen))


class PasserKindTests(unittest.TestCase):
    def test_bishop_attacking_the_path(self):
        # The a4 bishop sees d7 through b5 and c6.
        self.assertEqual(kinds("7k/8/8/3P4/b7/8/8/2B1K3 w - - 0 1")[0], (1, 0, 0))

    def test_bishop_standing_on_the_path(self):
        self.assertEqual(kinds("7k/3b4/8/3P4/8/8/8/2B1K3 w - - 0 1")[0], (1, 0, 0))

    def test_a_ray_stopped_short_of_the_path_does_not_count(self):
        # The white king on b5 blocks the a4 bishop's diagonal.
        self.assertEqual(kinds("7k/8/8/1K1P4/b7/8/8/2B5 w - - 0 1")[0], (1, 1, 1))

    def test_free_passer(self):
        # The h1 bishop's diagonal ends at the pawn itself, not its path.
        self.assertEqual(kinds("7k/8/8/3P4/8/8/8/2B1K2b w - - 0 1")[0], (1, 1, 1))

    def test_king_next_to_the_path(self):
        # The c8 king touches d7 and d8; the bishop does not.
        self.assertEqual(kinds("2k5/8/8/3P4/8/8/8/2B1K2b w - - 0 1")[0], (1, 0, 1))

    def test_a_pawn_that_is_not_passed_is_not_counted(self):
        self.assertEqual(kinds("7k/8/4p3/3P4/8/8/8/2B1K2b w - - 0 1"), ((0, 0, 0), (0, 0, 0)))

    def test_black_passer_runs_down_the_board(self):
        # Black pawn e4 runs to e1; the white bishop on b1 sees c2, d3 and
        # stops at the pawn, so no path square; the white king is far away.
        self.assertEqual(kinds("7K/8/8/8/4p3/8/8/1B5k w - - 0 1")[1], (1, 1, 1))
        # From c1 the bishop attacks e3 through d2.
        self.assertEqual(kinds("7K/8/8/8/4p3/8/8/2B4k w - - 0 1")[1], (1, 0, 0))
        # With the white bishop on g3 it attacks e1 through f2.
        self.assertEqual(kinds("7K/8/8/8/4p3/6B1/8/7k w - - 0 1")[1], (1, 0, 0))


class TwoRegressorFit(unittest.TestCase):
    def test_the_fit_recovers_a_planted_rule(self):
        import numpy as np

        planted = (3, 2, 5, 7)
        cells = np.array([(p, c, f) for p in range(1, 6) for c in range(0, p + 1)
                          for f in range(0, c + 1)], dtype=float)
        target = np.clip(planted[0] + planted[1] * cells[:, 0] + planted[2] * cells[:, 1]
                         + planted[3] * cells[:, 2], 0, oc.OCB_NORMAL)
        s = np.arange(oc.OCB_NORMAL + 1, dtype=float)[None, :]
        table = (s - target[:, None]) ** 2
        grid = {"a": range(0, 10), "b": range(0, 5), "c": range(0, 8), "d": range(0, 10)}
        self.assertEqual(fp.fit_two(cells, table, grid), planted)

    def test_with_d_fixed_at_zero_it_is_the_count_form(self):
        import numpy as np

        cells = np.array([(p, c, 0) for p in range(1, 6) for c in range(0, p + 1)], dtype=float)
        target = np.clip(1 + 2 * cells[:, 0] + 9 * cells[:, 1], 0, oc.OCB_NORMAL)
        s = np.arange(oc.OCB_NORMAL + 1, dtype=float)[None, :]
        table = (s - target[:, None]) ** 2
        grid = {"a": range(0, 5), "b": range(0, 5), "c": range(0, 12), "d": [0]}
        self.assertEqual(fp.fit_two(cells, table, grid), (1, 2, 9, 0))


if __name__ == "__main__":
    unittest.main()
