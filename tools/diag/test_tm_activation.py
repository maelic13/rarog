"""Tests for tm_activation.py's parser and summary."""

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import tm_activation  # noqa: E402


def line(k, share, fall, instab, stab, effort):
    return (f"info string tm depth 9 k {k} share {share:.4f} fall {fall:.4f} instab {instab:.4f} "
            f"stab {stab:.4f} effort {effort:.4f} optimum 120.0 target 150.0 elapsed 90.0")


class Parse(unittest.TestCase):
    def test_a_log_line_becomes_numbers(self):
        record = tm_activation.parse_tm_line(line(3, 0.42, 0.9, 1.1, 1.1561, 0.924) + "\n")
        self.assertEqual(record["k"], 3)
        self.assertAlmostEqual(record["stab"], 1.1561)
        self.assertEqual(record["elapsed"], 90.0)

    def test_other_lines_are_not_records(self):
        self.assertIsNone(tm_activation.parse_tm_line("info string diag tm_mate_stop 1"))
        self.assertIsNone(tm_activation.parse_tm_line("info depth 9 score cp 20"))


class Summary(unittest.TestCase):
    def records(self, lines):
        return [tm_activation.parse_tm_line(text) for text in lines]

    def test_a_factor_at_one_end_on_more_than_nine_in_ten_iterations_is_inert(self):
        seeds = tm_activation.SEEDS
        # Ten iterations: effort at its high end on all ten, stability at its
        # floor on two and at k = 0 on three.
        lines = [line(0, 0.3, 0.9, 1.1, 1.2881, 0.924)] * 3
        lines += [line(14, 0.5, 0.572, 1.5, 0.716, 0.924)] * 2
        lines += [line(4, 0.6, 1.0, 1.3, 1.1121, 0.924)] * 5
        summary = tm_activation.summarise(self.records(lines), seeds)
        effort = summary["factors"]["effort"]
        self.assertEqual(effort["low_share"], 1.0)
        self.assertTrue(effort["inert"])
        stab = summary["factors"]["stab"]
        self.assertAlmostEqual(stab["low_share"], 0.3)
        self.assertAlmostEqual(stab["high_share"], 0.2)
        self.assertFalse(stab["inert"])
        self.assertAlmostEqual(summary["factors"]["fall"]["low_share"], 0.2)
        self.assertAlmostEqual(summary["factors"]["instab"]["low_share"], 0.3)
        self.assertEqual(summary["k_histogram"], {"0": 3, "10+": 2, "4": 5})

    def test_a_set_value_moves_the_end(self):
        lines = [line(5, 0.5, 0.9, 1.1, 0.8, 0.924)] * 4
        seeds = tm_activation.SEEDS
        self.assertEqual(tm_activation.summarise(self.records(lines), seeds)["factors"]["stab"]["high_share"], 0)
        moved = {**seeds, "TmStabFloor": 8_000}
        self.assertEqual(tm_activation.summarise(self.records(lines), moved)["factors"]["stab"]["high_share"], 1)

    def test_nothing_to_summarise_is_an_error(self):
        with self.assertRaises(ValueError):
            tm_activation.summarise([], tm_activation.SEEDS)


class Book(unittest.TestCase):
    def test_the_replay_takes_three_hundred_distinct_positions(self):
        fens = tm_activation.book_positions(300)
        self.assertEqual(len(fens), 300)
        self.assertEqual(len(set(fens)), 300)
        self.assertTrue(all(fen.endswith(" 0 1") for fen in fens))


if __name__ == "__main__":
    unittest.main()
