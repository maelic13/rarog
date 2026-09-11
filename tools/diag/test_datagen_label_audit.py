"""Tests for the datagen label audit (PLAN 4.10.8).

Every guard is exercised on a known-BAD input as well as a good one, per rule
15. The two that matter most are the ones the count silently depends on: a
missing table must read as UNKNOWN rather than as agreement, and a cursed win
must be excluded rather than counted as a failure.
"""

import sys
import unittest
from pathlib import Path

import chess
import chess.pgn

sys.path.insert(0, str(Path(__file__).resolve().parent))
import datagen_label_audit as audit


class StubTablebase:
    """WDL by material key, and MissingTableError for anything unlisted."""

    def __init__(self, table):
        self.table = table
        self.probes = []

    def probe_wdl(self, board):
        fen = board.board_fen()
        self.probes.append(fen)
        if fen in self.table:
            return self.table[fen]
        raise chess.syzygy.MissingTableError(fen)


def game_from(moves_san: list[str], result: str) -> chess.pgn.Game:
    board = chess.Board()
    game = chess.pgn.Game()
    game.headers["Result"] = result
    node = game
    for san in moves_san:
        move = board.parse_san(san)
        node = node.add_variation(move)
        board.push(move)
    return game


class FamilyKeyTests(unittest.TestCase):
    def test_key_is_written_strong_side_first(self):
        board = chess.Board("8/8/8/8/8/8/1KRP4/k7 w - - 0 1")
        self.assertEqual(audit.family_of(board, chess.WHITE), "KRP-K")

    def test_the_weak_side_is_listed_second(self):
        board = chess.Board("8/8/8/8/8/8/1KRP4/k1r5 w - - 0 1")
        self.assertEqual(audit.family_of(board, chess.WHITE), "KRP-KR")

    def test_the_key_flips_with_the_winner(self):
        board = chess.Board("8/8/8/8/8/8/1KRP4/k1r5 w - - 0 1")
        self.assertEqual(audit.family_of(board, chess.BLACK), "KR-KRP")


class ResultTests(unittest.TestCase):
    def test_unfinished_games_are_not_scored(self):
        self.assertIsNone(audit.result_score("*"))

    def test_the_three_real_results_map(self):
        self.assertEqual(audit.result_score("1-0"), 1.0)
        self.assertEqual(audit.result_score("0-1"), 0.0)
        self.assertEqual(audit.result_score("1/2-1/2"), 0.5)


class AuditGameTests(unittest.TestCase):
    """A KP-K position is reached after 1. e4 e5 in a stripped-down game.

    Rather than construct real games, the stub tablebase is keyed on the board
    FEN so the exact position that matters can be given a verdict.
    """

    def _position_after(self, moves_san):
        board = chess.Board()
        for san in moves_san:
            board.push(board.parse_san(san))
        return board.board_fen()

    def test_a_clean_win_that_was_won_is_not_a_defect(self):
        game = game_from(["e4", "e5"], "1-0")
        # White to move after 1.e4 e5; call it a clean win for White.
        tb = StubTablebase({self._position_after(["e4", "e5"]): 2})
        found = audit.audit_game(game, tb, max_men=32)
        self.assertIsNotNone(found)
        self.assertTrue(found["won"])

    def test_a_clean_win_that_was_drawn_is_a_defect(self):
        game = game_from(["e4", "e5"], "1/2-1/2")
        tb = StubTablebase({self._position_after(["e4", "e5"]): 2})
        found = audit.audit_game(game, tb, max_men=32)
        self.assertFalse(found["won"])

    def test_a_cursed_win_is_excluded(self):
        """WDL 1 is already drawn by the fifty-move rule (BAS-E46).

        Counting a drawn game there would inflate the defect with correct play.
        """
        game = game_from(["e4", "e5"], "1/2-1/2")
        tb = StubTablebase({self._position_after(["e4", "e5"]): 1})
        self.assertIsNone(audit.audit_game(game, tb, max_men=32))

    def test_a_draw_verdict_is_excluded(self):
        game = game_from(["e4", "e5"], "1/2-1/2")
        tb = StubTablebase({self._position_after(["e4", "e5"]): 0})
        self.assertIsNone(audit.audit_game(game, tb, max_men=32))

    def test_a_missing_table_is_unknown_not_agreement(self):
        """Probing past the installed limit converts unknown into 'agrees'."""
        game = game_from(["e4", "e5"], "1/2-1/2")
        tb = StubTablebase({})
        self.assertIsNone(audit.audit_game(game, tb, max_men=32))

    def test_the_man_limit_stops_the_probe(self):
        game = game_from(["e4", "e5"], "1/2-1/2")
        tb = StubTablebase({self._position_after(["e4", "e5"]): 2})
        self.assertIsNone(audit.audit_game(game, tb, max_men=6))
        self.assertEqual(tb.probes, [], "no probe should have been attempted")

    def test_a_black_win_is_scored_from_blacks_side(self):
        game = game_from(["e4", "e5", "Nf3"], "0-1")
        tb = StubTablebase({self._position_after(["e4", "e5", "Nf3"]): 2})
        found = audit.audit_game(game, tb, max_men=32)
        self.assertTrue(found["won"], "Black to move and winning; 0-1 is correct")

    def test_an_unfinished_game_is_skipped(self):
        game = game_from(["e4", "e5"], "*")
        tb = StubTablebase({self._position_after(["e4", "e5"]): 2})
        self.assertIsNone(audit.audit_game(game, tb, max_men=32))

    def test_only_the_first_clean_win_counts(self):
        game = game_from(["e4", "e5", "Nf3"], "1/2-1/2")
        tb = StubTablebase({
            self._position_after(["e4", "e5"]): 2,
            self._position_after(["e4", "e5", "Nf3"]): 2,
        })
        found = audit.audit_game(game, tb, max_men=32)
        self.assertEqual(found["ply"], 2)


class SummaryTests(unittest.TestCase):
    def test_both_denominators_are_reported(self):
        findings = [
            {"family": "KRP-KR", "won": False, "men": 5, "ply": 60},
            {"family": "KRP-KR", "won": True, "men": 5, "ply": 60},
            {"family": "KP-K", "won": True, "men": 3, "ply": 80},
            None, None,
        ]
        out = audit.summarize(findings, games=5)
        self.assertEqual(out["games_reaching_a_clean_win"], 3)
        self.assertEqual(out["clean_wins_not_won"], 1)
        # How badly the endings are played...
        self.assertAlmostEqual(out["share_of_clean_wins_not_won"], 1 / 3, places=4)
        # ...against how much of the CORPUS carries a wrong label.
        self.assertAlmostEqual(out["share_of_all_games_mislabelled"], 0.2, places=4)

    def test_families_are_ordered_by_volume(self):
        findings = [{"family": "KP-K", "won": True, "men": 3, "ply": 1}] * 3
        findings += [{"family": "KRP-KR", "won": False, "men": 5, "ply": 1}]
        out = audit.summarize(findings, games=4)
        self.assertEqual(list(out["families"]), ["KP-K", "KRP-KR"])
        self.assertEqual(out["families"]["KRP-KR"]["share_not_won"], 1.0)

    def test_an_empty_corpus_reports_none_rather_than_dividing(self):
        out = audit.summarize([], games=0)
        self.assertIsNone(out["share_of_clean_wins_not_won"])
        self.assertIsNone(out["share_of_all_games_mislabelled"])


class SerialAndShardedAgreeTests(unittest.TestCase):
    """Sharding must change wall time and nothing else."""

    def test_offsets_partition_the_corpus(self):
        items = list(enumerate(range(23)))
        serial = [i for i, _ in items]
        for workers in (2, 5, 23):
            with self.subTest(workers=workers):
                merged = {}
                for bucket in audit.shard(items, workers):
                    for index, value in bucket:
                        merged[index] = value
                self.assertEqual(sorted(merged), serial)


class ShardTests(unittest.TestCase):
    def test_every_game_is_audited_exactly_once(self):
        items = list(enumerate(range(37)))
        for workers in (1, 3, 8, 37):
            with self.subTest(workers=workers):
                flat = [x for b in audit.shard(items, workers) for x in b]
                self.assertEqual(sorted(flat), sorted(items))


if __name__ == "__main__":
    unittest.main()
