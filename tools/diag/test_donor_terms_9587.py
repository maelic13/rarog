"""Tests for donor_terms_9587.py: the row mapping on real instrumented output."""

import unittest

import donor_terms_9587 as dt
from test_king_subterms import BLOCK


class ConvertTests(unittest.TestCase):
    def test_maps_winnable_into_initiative_and_reads_the_final_total(self):
        terms = dt.convert_block(BLOCK)
        self.assertIsNotNone(terms)
        self.assertEqual(terms["Initiative"], (0.00, -0.17))
        self.assertEqual(terms["King safety"], (-0.51, -0.41))
        self.assertEqual(terms["_total"], (-0.25, -0.25))
        self.assertEqual(set(terms) - {"_total"}, set(dt.dr.SF11_TERMS))

    def test_missing_table_is_invalid(self):
        self.assertIsNone(dt.convert_block("Total evaluation: none (in check)"))
        self.assertIsNone(dt.convert_block("Final evaluation: 0.10 (white side)"))


if __name__ == "__main__":
    unittest.main()
