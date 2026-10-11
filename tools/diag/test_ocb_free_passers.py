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


if __name__ == "__main__":
    unittest.main()
