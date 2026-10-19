"""Tests for neutral_vector.py: which coefficients move, and the refusal of a
vector that does not match its manifest."""
import unittest

import neutral_vector as nv

VECTOR = [
    ("mg_val", 0, 96), ("mg_val", 4, 1135), ("mg_val", 5, 0),
    ("eg_val", 1, 410), ("pst_mg", 3, 17), ("pst_mg", 7, 5),
    ("kd_constant", 0, -40), ("passed_eg", 0, 0),
]
MANIFEST = {
    ("mg_val", 0): ("free", "linear"), ("mg_val", 4): ("free", "linear"),
    ("mg_val", 5): ("fixed", "none"), ("eg_val", 1): ("free", "linear"),
    ("pst_mg", 3): ("free", "linear"), ("pst_mg", 7): ("fixed", "none"),
    ("kd_constant", 0): ("excluded", "coordinate"), ("passed_eg", 0): ("fixed", "none"),
}


class Neutral(unittest.TestCase):
    def test_free_linear_terms_reset_and_others_kept(self):
        out = dict(((f, i), v) for f, i, v in nv.neutral(VECTOR, MANIFEST))
        self.assertEqual(out[("mg_val", 0)], 100)
        self.assertEqual(out[("mg_val", 4)], 900)
        self.assertEqual(out[("eg_val", 1)], 300)
        self.assertEqual(out[("pst_mg", 3)], 0)
        self.assertEqual(out[("mg_val", 5)], 0)       # fixed, kept
        self.assertEqual(out[("pst_mg", 7)], 5)       # fixed gauge anchor, kept
        self.assertEqual(out[("kd_constant", 0)], -40)  # coordinate stage, kept

    def test_order_is_the_vectors(self):
        out = nv.neutral(VECTOR, MANIFEST)
        self.assertEqual([(f, i) for f, i, _ in out], [(f, i) for f, i, _ in VECTOR])

    def test_a_vector_that_does_not_match_its_manifest_is_refused(self):
        with self.assertRaises(ValueError):
            nv.neutral(VECTOR[:-1], MANIFEST)


if __name__ == "__main__":
    unittest.main()
