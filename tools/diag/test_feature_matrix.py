"""Tests for the feature-matrix audit.

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
        declared = re.findall(r"^(\w+)\s*=\s*\[", block, flags=re.MULTILINE)
        # `default` names a set of the others, so it is not a feature to check.
        return sorted(name for name in declared if name != "default")

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

    def test_the_empty_configuration_is_labelled(self):
        # Every check runs --no-default-features, so the empty subset is the
        # legacy search rather than the shipped default build.
        self.assertEqual(
            feature_matrix.describe(()), "no features (the legacy search)"
        )
        self.assertEqual(feature_matrix.describe(("tune", "diag")), "tune,diag")

    def test_every_check_starts_from_a_clean_slate(self):
        """Without --no-default-features a subset would silently include b2core."""
        source = (feature_matrix.__file__ and open(feature_matrix.__file__, encoding="utf-8").read())
        self.assertIn('"--no-default-features"', source)


if __name__ == "__main__":
    unittest.main()
