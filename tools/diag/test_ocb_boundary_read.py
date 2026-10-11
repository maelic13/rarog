"""Tests for ocb_boundary_read.py on hand-made games."""

import tempfile
import unittest
from pathlib import Path

import ocb_boundary_read as br

# Seven men, opposite bishops (c1 and d5): KBPP against KBP. Black's Bxb3
# leaves six men, a pure opposite-bishop ending: the crossing.
CROSS = """[Event "t"]
[White "cand"]
[Black "head"]
[Result "1/2-1/2"]
[FEN "4k3/p7/8/3b4/8/1P6/P7/2B1K3 b - - 0 1"]
[SetUp "1"]

1... Bxb3 2. axb3 1/2-1/2
"""

# The same start, no crossing: a king move.
QUIET = """[Event "t"]
[White "cand"]
[Black "head"]
[Result "1-0"]
[FEN "4k3/p7/8/3b4/8/1P6/P7/2B1K3 b - - 0 1"]
[SetUp "1"]

1... Kd7 2. Kd2 1-0
"""


class BoundaryTests(unittest.TestCase):
    def test_the_first_crossing_and_its_mover(self):
        with tempfile.TemporaryDirectory() as tmp:
            pgn = Path(tmp) / "g.pgn"
            pgn.write_text(CROSS + "\n" + QUIET)
            report = br.read(pgn, ["cand", "head"])
        self.assertEqual(report["games"], 2)
        self.assertEqual(report["crossings"], 1)
        # Black (head) captured on b3: seven men to six, still opposite bishops
        # and pure, so the crossing is head's, and the game was drawn.
        self.assertEqual(report["by_mover"]["head"]["crossings"], 1)
        self.assertEqual(report["by_mover"]["head"]["draw_rate"], 1.0)
        self.assertEqual(report["by_mover"]["cand"]["crossings"], 0)


if __name__ == "__main__":
    unittest.main()
