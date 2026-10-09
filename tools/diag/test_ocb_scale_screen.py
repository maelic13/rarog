"""Tests for ocb_scale_screen.py: the offline OCB rule against the engine's own
unit-test positions, the cohort split, the integer arithmetic and the fit."""

import unittest

import numpy as np

import ocb_scale_screen as oc


class RuleTests(unittest.TestCase):
    def test_the_engines_own_cases(self):
        # src/eval/endgame/mod.rs, opposite_bishop_scale_relaxed_by_passers.
        cohort, pawns, pw, pb, s = oc.classify("4k3/p7/8/3b4/8/8/P7/2B1K3 w - - 0 1")
        self.assertEqual((cohort, pawns, pw + pb, s), (oc.PURE, 2, 0, 40))
        cohort, pawns, pw, pb, s = oc.classify("4k3/7p/P7/3b4/8/8/8/2B1K3 w - - 0 1")
        self.assertEqual((cohort, pawns, pw, pb, s), (oc.PURE, 2, 1, 1, 48))

    def test_same_coloured_bishops_are_not_ocb(self):
        self.assertEqual(oc.classify("4k3/p7/8/4b3/8/8/P7/2B1K3 w - - 0 1")[0], oc.NOT_OCB)

    def test_two_bishops_on_one_side_are_not_ocb(self):
        self.assertEqual(oc.classify("4k3/p7/8/3b4/8/8/P7/2BBK3 w - - 0 1")[0], oc.NOT_OCB)

    def test_other_pieces_make_the_pieces_cohort(self):
        self.assertEqual(oc.classify("r3k3/p7/8/3b4/8/8/P7/2B1K2R w - - 0 1")[0], oc.PIECES)

    def test_pawnless_minor_against_minor_is_the_specialised_draw(self):
        self.assertEqual(oc.classify("4k3/8/8/3b4/8/8/8/2B1K3 w - - 0 1"), (oc.PAWNLESS, 0, 0, 0, 0))
        self.assertEqual(oc.classify("4k3/8/8/3b4/8/8/8/1NB1K3 w - - 0 1")[4], 32)

    def test_a_blocked_pawn_is_not_passed_but_an_advanced_one_is(self):
        # White e5 against Black d6: e5 is not passed only if a black pawn is
        # ahead on d, e or f; d6 is ahead on the adjacent file.
        _, _, pw, pb, _ = oc.classify("4k3/8/3p4/4P3/8/3b4/8/2B1K3 w - - 0 1")
        self.assertEqual((pw, pb), (0, 0))
        # White e6 past Black d5: both are passed.
        _, _, pw, pb, _ = oc.classify("4k3/8/4P3/3p4/8/3b4/8/2B1K3 w - - 0 1")
        self.assertEqual((pw, pb), (1, 1))

    def test_material_key(self):
        self.assertEqual(oc.material_key("4k3/p7/8/3b4/8/8/PP6/2B1K3 w - - 0 1"), "KBPP-KBP")


class ArithmeticTests(unittest.TestCase):
    def test_scaling_truncates_toward_zero_like_rust(self):
        self.assertEqual(oc.scaled(np.array([-101.0]), 40)[0], -84.0)
        self.assertEqual(oc.scaled(np.array([101.0]), 40)[0], 84.0)

    def test_recovered_raw_reproduces_the_dumped_score(self):
        raw = np.arange(-900, 901)
        for s0 in (32, 36, 40, 44):
            dumped = oc.scaled(raw.astype(float), s0)
            recovered = np.where(dumped == 0, 0,
                                 np.round((dumped + 0.5 * np.sign(dumped)) * 48 / s0))
            np.testing.assert_array_equal(oc.scaled(recovered, s0), dumped)

    def test_the_fit_recovers_a_planted_rule(self):
        rng = np.random.default_rng(7)
        n = 40_000
        pawns = rng.integers(1, 9, n)
        passers = rng.integers(0, 3, n)
        raw = rng.normal(0, 250, n).round()
        true_s = np.clip(20 + 2 * pawns + 6 * passers, 0, 48)
        p = oc.sigmoid(oc.scaled(raw, true_s), 1.5)
        label = (rng.random(n) < p).astype(float)
        table = oc.cell_losses(raw, label, pawns, passers, 1.5)
        a, b, c = oc.fit_rule(table, {"a": range(10, 31), "b": range(0, 5), "c": range(2, 11)})
        s_fit = np.clip(a + b * pawns + c * passers, 0, 48)
        self.assertLess(np.abs(s_fit - true_s).mean(), 3.0)


if __name__ == "__main__":
    unittest.main()
