"""Tests for the drawn-share bias census (PLAN 4.11.4).

Two of these guard defects the census itself uncovered: a thin drawn subset
being reported as a rate, and results depending on the ORDER positions were
scored in. The second is not covered by a unit test -- it needs a real engine --
so it is asserted structurally here and was verified empirically by a
serial-versus-sharded byte-identity run, recorded in
`analysis/drawn_share_census_2026-09-05.md`.
"""

import re
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import endgame_drawn


class ThinSampleTests(unittest.TestCase):
    """Some families have almost no drawn subset; KQ-K and KR-K have none."""

    def _summary(self, n_drawn):
        return endgame_drawn.summarize(
            "X", [200] * n_drawn, sampled=1500, mates=0,
            threshold=100, nodes=60000, digest="aa" * 32)

    def test_a_thin_subset_gets_no_rate(self):
        out = self._summary(endgame_drawn.MIN_DRAWN - 1)
        self.assertIsNone(out["overclaim_rate"])
        self.assertIn("thin", out)

    def test_the_boundary_is_inclusive(self):
        out = self._summary(endgame_drawn.MIN_DRAWN)
        self.assertIsNotNone(out["overclaim_rate"])
        self.assertNotIn("thin", out)

    def test_an_empty_subset_does_not_divide(self):
        out = self._summary(0)
        self.assertIsNone(out["overclaim_rate"])
        self.assertEqual(out["drawn"], 0)

    def test_the_rate_counts_only_scores_above_the_threshold(self):
        out = endgame_drawn.summarize(
            "X", [50] * 30 + [150] * 10, sampled=100, mates=0,
            threshold=100, nodes=60000, digest="aa" * 32)
        self.assertEqual(out["overclaimed"], 10)
        self.assertAlmostEqual(out["overclaim_rate"], 0.25)

    def test_the_cohort_digest_is_carried(self):
        self.assertEqual(self._summary(40)["cohort_sha256"], "aa" * 32)


class ShardTests(unittest.TestCase):
    def test_every_position_is_scored_exactly_once(self):
        items = [(i, f"fen{i}") for i in range(31)]
        for workers in (1, 4, 31):
            with self.subTest(workers=workers):
                flat = [x for b in endgame_drawn.shard(items, workers) for x in b]
                self.assertEqual(sorted(flat), sorted(items))

    def test_empty_buckets_are_dropped(self):
        self.assertEqual(len(endgame_drawn.shard([(0, "a")], 9)), 1)


class OrderIndependenceTests(unittest.TestCase):
    """A census must be position-local, or sharding changes its numbers.

    `engine.analyse` was called with no `game=` token, so no `ucinewgame` was
    sent between positions and the transposition table carried over. KBP-KB
    then read 0.702 serially and 0.750 over six workers on the SAME positions.
    """

    SOURCE = Path(endgame_drawn.__file__).read_text(encoding="utf-8")

    def test_every_analyse_call_forces_a_new_game(self):
        calls = re.findall(r"engine\.analyse\((?:[^()]|\([^()]*\))*\)",
                           self.SOURCE, re.S)
        self.assertTrue(calls, "no engine.analyse call found; test is vacuous")
        for call in calls:
            self.assertIn("game=", call)

    def test_the_output_directory_is_created(self):
        """A completed census died on its write and lost 28,500 positions."""
        self.assertIn("mkdir(parents=True, exist_ok=True)", self.SOURCE)


if __name__ == "__main__":
    unittest.main()
