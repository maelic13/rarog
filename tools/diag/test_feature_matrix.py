"""Tests for the feature-matrix audit (PLAN 4.10.12).

The one that matters is `test_the_matrix_covers_every_declared_feature`: a
matrix that silently stops covering a feature is worse than no matrix, because
it reports success over a shrinking set. Adding a feature to Cargo.toml and
forgetting to add it here now fails the suite.
"""

import re
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import feature_matrix


class CoverageTests(unittest.TestCase):
    CARGO = Path(__file__).resolve().parents[2] / "Cargo.toml"

    def declared_features(self):
        text = self.CARGO.read_text(encoding="utf-8")
        block = text.split("[features]", 1)[1]
        # Stop at the next section header.
        block = re.split(r"^\[", block, maxsplit=1, flags=re.MULTILINE)[0]
        return sorted(re.findall(r"^(\w+)\s*=\s*\[", block, flags=re.MULTILINE))

    def test_the_matrix_covers_every_declared_feature(self):
        self.assertEqual(sorted(feature_matrix.SHIPPED_FEATURES),
                         self.declared_features())

    def test_texel_is_flagged_as_never_measurable(self):
        """AGENTS.md: `texel` bypasses the eval and pawn caches."""
        self.assertIn("texel", feature_matrix.NEVER_MEASURE)

    def test_never_measure_is_a_subset_of_the_shipped_features(self):
        self.assertTrue(
            feature_matrix.NEVER_MEASURE <= set(feature_matrix.SHIPPED_FEATURES))


class CombinationTests(unittest.TestCase):
    def test_the_empty_configuration_is_included(self):
        """The default build is a configuration and is checked first."""
        combos = feature_matrix.combinations(["a", "b"])
        self.assertEqual(combos[0], ())

    def test_every_subset_appears_exactly_once(self):
        combos = feature_matrix.combinations(["a", "b", "c"])
        self.assertEqual(len(combos), 8)
        self.assertEqual(len(set(combos)), 8)

    def test_smallest_first_so_a_failure_names_the_simplest_case(self):
        sizes = [len(c) for c in feature_matrix.combinations(["a", "b", "c"])]
        self.assertEqual(sizes, sorted(sizes))

    def test_the_full_matrix_is_two_to_the_n(self):
        self.assertEqual(
            len(feature_matrix.combinations(feature_matrix.SHIPPED_FEATURES)),
            2 ** len(feature_matrix.SHIPPED_FEATURES))

    def test_the_default_configuration_is_labelled(self):
        self.assertEqual(feature_matrix.describe(()), "default")
        self.assertEqual(feature_matrix.describe(("tune", "diag")), "tune,diag")


if __name__ == "__main__":
    unittest.main()
