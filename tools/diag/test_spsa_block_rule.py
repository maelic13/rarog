"""Tests for spsa_block_rule.py's report of the runner's arm clamps."""

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import spsa_block_rule  # noqa: E402


class ClampLines(unittest.TestCase):
    def test_a_clamped_coordinate_is_named_with_its_counts(self):
        result = {"schema_version": 4, "tune_audit": {"warnings": [
            {"kind": "initial-differs-from-engine-default", "name": "A"},
            {"kind": "arms-clamped-at-range-end", "name": "CoreRfpCorrection", "min": 0, "max": 2048,
             "completed_iterations": 2000, "clamped_at_min": 1900, "clamped_at_max": 0},
        ]}}
        lines = spsa_block_rule.clamp_lines(result, 2000)
        self.assertEqual(lines[0], "  arm clamps:")
        self.assertEqual(len(lines), 2)
        self.assertIn("CoreRfpCorrection", lines[1])
        self.assertIn("at min on 1900, at max on 0 of 2000 iterations (range 0..2048)", lines[1])

    def test_an_audit_without_clamps_says_none(self):
        self.assertEqual(spsa_block_rule.clamp_lines({"schema_version": 4, "tune_audit": {"warnings": []}}, 10),
                         ["  arm clamps: none"])

    def test_an_audit_from_before_the_clamp_counts_is_not_read_as_none(self):
        # CLI 0.3.0 wrote schema 3 with an audit that never counted clamps.
        self.assertEqual(spsa_block_rule.clamp_lines({"schema_version": 3, "tune_audit": {"warnings": []}}, 10),
                         ["  arm clamps: not recorded (result schema 3)"])
        self.assertEqual(spsa_block_rule.clamp_lines({"schema_version": 2}, 10),
                         ["  arm clamps: not recorded (result schema 2)"])


if __name__ == "__main__":
    unittest.main()
