"""Unit tests for the parsing and summary halves of tb_probe_read.py.

The engine-driving half is exercised by the reads recorded in the D.9
record; these tests pin what a root's record is read from and what the
summary compares.
"""

import pathlib
import sys
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))

import tb_probe_read as read  # noqa: E402

FEN = "8/6P1/8/3b3P/5B2/3k1pK1/2p5/8 b - - 0 1"


def info(depth, score, nodes, tbhits, ms, bound=""):
    return (
        f"info depth {depth} seldepth {depth + 9} multipv 1 score {score}{bound} "
        f"nodes {nodes} nps 1 hashfull 1 tbhits {tbhits} time {ms} pv c2c1q"
    )


def row(index, nodes, tbhits, wall_ms, klass="none", complete=True):
    return {
        "index": index,
        "fen": FEN,
        "complete": complete,
        "nodes": nodes,
        "tbhits": tbhits,
        "class": klass,
        "wall_ms": wall_ms,
    }


class ParseSearch(unittest.TestCase):
    def test_a_bound_line_of_an_unfinished_iteration_is_not_its_depth(self):
        lines = [
            info(19, "cp 1173", 1_687_242, 68_165, 563),
            info(20, "cp 1500", 12_000_000, 90_000, 4_000, " lowerbound"),
            "bestmove e5e4 ponder c6b5",
        ]
        rec = read.parse_search(FEN, lines, 20, 9_000.0)
        self.assertEqual(rec["depth"], 19)
        self.assertFalse(rec["complete"])
        self.assertEqual(rec["nodes"], 1_687_242)
        self.assertEqual(rec["time_ms"], 563)
        self.assertEqual(rec["wall_ms"], 9_000.0)
        self.assertEqual(rec["bestmove"], "e5e4")

    def test_an_exact_line_after_a_bound_line_completes_the_depth(self):
        lines = [
            info(20, "cp 1500", 12_000_000, 90_000, 4_000, " upperbound"),
            info(20, "cp 1450", 13_000_000, 95_000, 4_400),
            "bestmove e5e4",
        ]
        rec = read.parse_search(FEN, lines, 20, 4_500.0)
        self.assertTrue(rec["complete"])
        self.assertEqual(rec["score"], "cp 1450")

    def test_a_clock_search_is_complete_at_any_depth_and_classes_its_score(self):
        lines = [info(23, "cp 19987", 900_000, 4_000, 812), "bestmove c2c1q"]
        rec = read.parse_search(FEN, lines, None, 830.0)
        self.assertTrue(rec["complete"])
        self.assertEqual(rec["depth"], 23)
        self.assertEqual(rec["class"], "win")
        self.assertEqual(read.score_class("cp", -19_761), "loss")
        self.assertEqual(read.score_class("mate", -3), "loss")
        self.assertEqual(read.score_class("cp", 19_760), "none")


class Summarize(unittest.TestCase):
    def test_wall_ratios_and_totals_are_against_the_first_engine(self):
        results = {
            "tb": [row(26, 900, 50, 4_000.0, "win"), row(30, 800, 40, 1_000.0)],
            "notb": [row(26, 1_000, 0, 1_000.0), row(30, 1_200, 0, 1_500.0)],
        }
        summary = read.summarize(2, results)
        self.assertEqual(summary["complete_in_all"], 2)
        self.assertEqual(summary["tb"]["nodes"], 1_700)
        self.assertEqual(summary["tb"]["max_wall_ms"], 4_000.0)
        notb = summary["notb"]
        self.assertAlmostEqual(notb["nodes_change"], 2_200 / 1_700 - 1)
        self.assertEqual(notb["tbhits_change"], -1)
        self.assertEqual(notb["roots_at_most_half"], [26])
        self.assertEqual(notb["roots_at_least_twice"], [])
        self.assertAlmostEqual(notb["median_wall_ratio"], (0.25 + 1.5) / 2)
        self.assertEqual([c["index"] for c in notb["class_changes"]], [26])

    def test_a_short_root_leaves_the_totals_and_is_listed(self):
        results = {
            "before": [row(11, 1_903_586, 10, 1.0), row(26, 500, 5, 1.0)],
            "after": [row(11, 1_917_508, 9, 1.0, complete=False), row(26, 400, 2, 1.0)],
        }
        summary = read.summarize(2, results)
        self.assertEqual(summary["complete_in_all"], 1)
        self.assertEqual(summary["short"], {"before": [], "after": [11]})
        self.assertEqual(summary["before"]["nodes"], 500)
        self.assertAlmostEqual(summary["after"]["nodes_change"], -0.2)


if __name__ == "__main__":
    unittest.main()
