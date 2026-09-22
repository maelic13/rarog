"""Tests for the Colosseum run recount.

The recount is what a ledger row rests on, so each test builds a small run
directory whose right answer is known and differs from the answer a plausible
mistake would give. A null pair is the case that matters: its two sides share
an engine name, and a recount that orients by name scores White's advantage as
a +19 Elo difference between two identical binaries.
"""

import json
import pathlib
import sys
import tempfile
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import colosseum_recount

NAME = "rarog-null"

# (pair, game in pair, side with White, result). Pair 1: A wins as White and
# as Black, pair score 1. Pair 2: a draw, then B wins as White, pair score 0.25.
GAMES = [
    (1, 1, "a", "1-0"),
    (1, 2, "b", "0-1"),
    (2, 1, "a", "1/2-1/2"),
    (2, 2, "b", "1-0"),
]
EXPECTED = [0, 1, 0, 0, 1]


def write_run(root, *, record_pentanomial, checkpoint_pentanomial=None, journal=True,
              names=(NAME, NAME), drop_from_journal=()):
    root = pathlib.Path(root)
    pgn, lines = [], []
    for number, (pair, pair_game, white_side, result) in enumerate(GAMES, start=1):
        white, black = (names[0], names[1]) if white_side == "a" else (names[1], names[0])
        pgn.append("\n".join([
            f'[White "{white}"]', f'[Black "{black}"]', f'[Result "{result}"]',
            '[Termination "normal"]', f'[GameNumber "{number}"]',
            f'[PairNumber "{pair}"]', f'[PairGame "{pair_game}"]', "", f"1. e4 e5 {result}", "",
        ]))
        if number not in drop_from_journal:
            lines.append(json.dumps({"game": {"number": number, "white": white_side,
                                              "black": "b" if white_side == "a" else "a",
                                              "pair_number": pair, "pair_game": pair_game}}))
    (root / "games.pgn").write_text("\n".join(pgn), encoding="utf-8")
    if journal:
        (root / "games.jsonl").write_text("\n".join(lines) + "\n", encoding="utf-8")
    (root / "run-record.json").write_text(json.dumps({
        "command": "match", "status": "completed",
        "official_sample": {"scored_games": len(GAMES), "pentanomial": record_pentanomial},
        "progress": {"fields": [{"label": "players", "value": f"{names[0]} vs. {names[1]}"}]},
    }), encoding="utf-8")
    if checkpoint_pentanomial is not None:
        (root / "checkpoint.json").write_text(json.dumps(
            {"payload": {"pentanomial": checkpoint_pentanomial}}), encoding="utf-8")
    return root


class RecountTests(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.dir = pathlib.Path(self._tmp.name)

    def tearDown(self):
        self._tmp.cleanup()

    def test_a_null_pair_is_scored_by_the_journals_sides_not_by_name(self):
        result = colosseum_recount.recount(write_run(self.dir, record_pentanomial=EXPECTED))
        self.assertEqual(result["pentanomial"], EXPECTED)
        self.assertEqual(result["oriented_by"], "journal")
        self.assertTrue(result["comparable"])
        self.assertTrue(result["agrees"])

    def test_a_null_pair_without_a_journal_is_refused_rather_than_guessed(self):
        with self.assertRaisesRegex(ValueError, "same name"):
            colosseum_recount.recount(write_run(self.dir, record_pentanomial=EXPECTED, journal=False))

    def test_a_pgn_game_missing_from_the_journal_is_refused(self):
        with self.assertRaisesRegex(ValueError, "not in the journal"):
            colosseum_recount.recount(write_run(self.dir, record_pentanomial=EXPECTED, drop_from_journal=(3,)))

    def test_distinct_names_without_a_journal_still_orient_by_name(self):
        result = colosseum_recount.recount(write_run(
            self.dir, record_pentanomial=EXPECTED, journal=False, names=("new", "base")))
        self.assertEqual(result["pentanomial"], EXPECTED)
        self.assertEqual(result["oriented_by"], "engine names")

    def test_a_match_record_at_zero_is_checked_against_the_checkpoint(self):
        result = colosseum_recount.recount(write_run(
            self.dir, record_pentanomial=[0] * 5, checkpoint_pentanomial=EXPECTED))
        self.assertEqual(result["recorded_source"], "checkpoint.json")
        self.assertTrue(result["comparable"])
        self.assertTrue(result["agrees"])

    def test_a_checkpoint_that_disagrees_is_reported(self):
        result = colosseum_recount.recount(write_run(
            self.dir, record_pentanomial=[0] * 5, checkpoint_pentanomial=[0, 0, 2, 0, 0]))
        self.assertTrue(result["comparable"])
        self.assertFalse(result["agrees"])

    def test_a_run_with_no_count_anywhere_is_not_comparable(self):
        result = colosseum_recount.recount(write_run(self.dir, record_pentanomial=[0] * 5))
        self.assertFalse(result["comparable"])


if __name__ == "__main__":
    unittest.main()
