"""Tests for ocb_book.py: the strata, the filters and the exclusion."""

import tempfile
import unittest
from pathlib import Path

import ocb_book as ob


class StratumTests(unittest.TestCase):
    def test_pure_needs_three_pawns(self):
        self.assertEqual(ob.stratum("4k3/p7/8/3b4/8/8/PP6/2B1K3 w - - 0 1"), "pure")
        self.assertIsNone(ob.stratum("4k3/p7/8/3b4/8/8/P7/2B1K3 w - - 0 1"))

    def test_near_is_one_matching_piece_each(self):
        self.assertEqual(ob.stratum("r3k3/p7/8/3b4/8/8/PP6/2B1K2R w - - 0 1"), "near")
        # Unmatched pieces, or two each, are neither stratum.
        self.assertIsNone(ob.stratum("n3k3/p7/8/3b4/8/8/PP6/2B1K2R w - - 0 1"))
        self.assertIsNone(ob.stratum("rn2k3/p7/8/3b4/8/8/PP6/2B1KN1R w - - 0 1"))

    def test_same_coloured_bishops_are_neither(self):
        self.assertIsNone(ob.stratum("4k3/p7/8/4b3/8/8/PP6/2B1K3 w - - 0 1"))


class BuildTests(unittest.TestCase):
    def test_filters_dedupe_and_exclusion(self):
        rows = [
            "4k3/p7/8/3b4/8/8/PP6/2B1K3 w - - 0 1;0.5;20;1",
            "4k3/p7/8/3b4/8/8/PP6/2B1K3 w - - 3 9;1;25;1",       # same EPD
            "4k3/pp6/8/3b4/8/8/P7/2B1K3 w - - 0 1;0.5;900;1",    # outside the cp band
            "4k3/pp6/8/3b4/8/8/PP6/2B1K3 b - - 0 1;0.5;-40;1",   # excluded below
            "4k3/pp6/8/3b4/8/8/PPP5/2B1K3 w - - 0 1;0;10;1",
            "r3k3/p7/8/3b4/8/8/PP6/2B1K2R w - - 0 1;0.5;0;1",
        ]
        with tempfile.TemporaryDirectory() as tmp:
            dump = Path(tmp) / "d.csv"
            dump.write_text("\n".join(rows) + "\n")
            exclude = Path(tmp) / "x.csv"
            exclude.write_text("4k3/pp6/8/3b4/8/8/PP6/2B1K3 b - - 7 30;1\n")
            book, counts = ob.build(dump, [exclude], 300, 1, 5)
            self.assertEqual(counts["unique"], {"pure": 3, "near": 1})
            self.assertEqual(counts["excluded"], 1)
            self.assertEqual(sorted(name for _k, name, _s in book), ["near", "pure"])
            with self.assertRaises(SystemExit):
                ob.build(dump, [exclude], 300, 3, 5)


if __name__ == "__main__":
    unittest.main()
