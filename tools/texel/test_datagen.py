"""Fast unit tests for the Texel datagen sampling contracts."""

import json
import random
import sys
import tempfile
import unittest
from pathlib import Path

import chess
import chess.pgn

sys.path.insert(0, str(Path(__file__).resolve().parent))
import extract
import extract_parallel
import opening_sequence
import sample_fens


class ExtractTests(unittest.TestCase):
    def test_parallel_provenance_accepts_and_verifies_v2(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "games.pgn"
            path.write_text("test pgn\n", encoding="utf-8")
            manifest = {
                "schema": "rarog-fastchess-datagen-v2",
                "output": {
                    "path": str(path.resolve()),
                    "bytes": path.stat().st_size,
                    "sha256": extract.sha256_file(path),
                },
                "engine": {"sha256": "engine", "git_sha": "commit", "git_dirty": False},
                "book": {"sha256": "book", "seed": 42, "start": 1, "end": 1},
                "games": 1,
                "nodes_per_move": 8000,
                "adjudication": {"Name": "datagen-v1"},
            }
            path.with_suffix(".manifest.json").write_text(
                json.dumps(manifest), encoding="utf-8"
            )
            records, contract = extract_parallel.load_datagen_provenance([path])
            self.assertEqual(records[0]["games"], 1)
            self.assertEqual(contract["engine_sha256"], "engine")

    def test_allocate_is_exact(self):
        self.assertEqual(extract.allocate(3_000_003, [1, 1, 1, 1, 1]),
                         [600_001, 600_001, 600_001, 600_000, 600_000])

    def test_five_phase_boundaries(self):
        expected = {
            24: 0, 20: 0,
            19: 1, 14: 1,
            13: 2, 8: 2,
            7: 3, 3: 3,
            2: 4, 0: 4,
        }
        for phase, bucket in expected.items():
            with self.subTest(phase=phase):
                self.assertEqual(extract.phase_bucket(phase), bucket)

    def test_reservoir_never_exceeds_capacity(self):
        reservoir = extract.Reservoir(7, random.Random(4))
        for i in range(100):
            reservoir.offer((str(i), 0.5, None))
        self.assertEqual(reservoir.seen, 100)
        self.assertEqual(len(reservoir.items), 7)

    def test_static_identity_keeps_rule_fifty_clock(self):
        first = "8/8/8/8/8/8/P6p/4K2k w - - 17 42"
        second = "8/8/8/8/8/8/P6p/4K2k w - - 18 42"
        third = "8/8/8/8/8/8/P6p/4K2k w - - 17 99"
        self.assertNotEqual(extract.fen_key(first), extract.fen_key(second))
        self.assertEqual(extract.fen_key(first), extract.fen_key(third))

    def test_whole_start_determines_split(self):
        game = chess.pgn.Game()
        game.headers["Result"] = "1/2-1/2"
        game.add_variation(chess.Move.from_uci("e2e4"))
        clone = chess.pgn.Game()
        clone.headers["Result"] = "1-0"
        clone.add_variation(chess.Move.from_uci("d2d4"))
        self.assertEqual(extract.start_digest(game), extract.start_digest(clone))
        self.assertEqual(extract.split_for(game, 5, 5), extract.split_for(clone, 5, 5))

    def test_three_split_counts_are_relative_to_fixed_train(self):
        self.assertEqual(extract.split_counts(900, 5, 5),
                         {"train": 900, "validation": 50, "test": 50})

    def test_per_game_sampling_keeps_scarce_phase(self):
        game = chess.pgn.Game()
        game.headers["Result"] = "1/2-1/2"
        node = game.add_variation(chess.Move.from_uci("e2e4"))
        node.comment = "+0.10/8 0.01s"
        rows, rejected = extract.process_game(
            game, 0, 0, 1, 0, False, random.Random(1)
        )
        self.assertEqual(rejected, 0)
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0][1], 0)  # full-material opening

    def test_process_game_audit_reuses_walked_final_position(self):
        game = chess.pgn.Game()
        game.headers["Result"] = "1-0"
        node = game
        for move_uci in ("f2f3", "e7e5", "g2g4", "d8h4"):
            node = node.add_variation(chess.Move.from_uci(move_uci))
        audit = {}
        extract.process_game(game, 0, 0, 1, 0, False, random.Random(1), audit)
        self.assertEqual(audit, {"plies": 4, "natural_mate": True})

    def test_material_audit_has_exact_signature_and_classes(self):
        signature = extract_parallel.material_signature(chess.STARTING_FEN)
        self.assertEqual(signature, (8, 2, 2, 2, 1, 8, 2, 2, 2, 1))
        self.assertEqual(extract_parallel.material_class(signature), "both_queens")
        self.assertEqual(
            extract_parallel.material_class((1, 0, 0, 0, 0, 2, 0, 0, 0, 0)),
            "pawn_only",
        )

    def test_process_game_classifies_all_five_phases(self):
        cases = (
            (chess.STARTING_FEN, "e2e4", 0),
            ("rnb1kbnr/pppppppp/8/8/8/8/PPPPPPPP/RNB1KBNR w KQkq - 0 1", "e2e4", 1),
            ("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1", "a1a2", 2),
            ("4k2r/8/8/8/8/8/8/R3K3 w - - 0 1", "a1a2", 3),
            ("4k3/8/8/8/8/8/P7/4K3 w - - 0 1", "a2a3", 4),
        )
        for fen, move_uci, expected_bucket in cases:
            with self.subTest(bucket=expected_bucket):
                board = chess.Board(fen)
                game = chess.pgn.Game()
                game.setup(board)
                game.headers["Result"] = "1/2-1/2"
                game.add_variation(chess.Move.from_uci(move_uci))
                rows, _ = extract.process_game(
                    game, 0, 0, 1, 0, False, random.Random(1)
                )
                self.assertEqual(rows[0][1], expected_bucket)

    def test_parallel_worker_counts_empty_game_start(self):
        import tempfile
        game = chess.pgn.Game()
        game.headers["Result"] = "1/2-1/2"
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "empty.pgn"
            path.write_text(str(game) + "\n\n", encoding="utf-8")
            opts = {
                "validation_pct": 5.0, "test_pct": 5.0,
                "skip_start": 0, "skip_end": 6,
                "max_per_phase_per_game": 1, "max_per_game": 1,
                "quiet_filter": False, "seed": 1, "blend": 1.0,
            }
            output, stats = extract_parallel.worker((str(path), 0, path.stat().st_size, opts))
            self.assertEqual(stats["recorded_games"], 1)
            self.assertEqual(stats["skipped"], 1)
            self.assertEqual(len(output), 1)
            self.assertEqual(output[0][2], [])


class SeedSamplerTests(unittest.TestCase):
    def test_bucket_targets_are_exact(self):
        self.assertEqual(sample_fens.bucket_targets(12), [3, 3, 2, 2, 2])

    def test_phase_classifier(self):
        self.assertEqual(sample_fens.phase_bucket(chess.STARTING_FEN), 0)
        kings_and_pawn = "4k3/8/8/8/8/8/P7/4K3 w - -"
        self.assertEqual(sample_fens.phase_bucket(kings_and_pawn), 4)


class CommentScoreTests(unittest.TestCase):
    def test_both_runners_give_the_same_white_score(self):
        cases = (
            ("+1.75/6 0.010s", "s=175 d=6 t=10ms h=1ms n=7456", 175.0),
            ("-0.40/9 0.004s", "s=-40 d=9 t=4ms n=6000", -40.0),
            ("+M5/12 0.002s", "s=#5 d=12 t=2ms", extract.CP_CLAMP),
            ("-M3/12 0.002s", "s=#-3 d=12 t=2ms", -extract.CP_CLAMP),
            ("+31.00/4 0.001s", "s=3100 d=4", extract.CP_CLAMP),
        )
        for fastchess, colosseum, mover_cp in cases:
            for white_to_move in (True, False):
                expected = mover_cp if white_to_move else -mover_cp
                with self.subTest(colosseum=colosseum, white_to_move=white_to_move):
                    self.assertEqual(extract.comment_cp_white(fastchess, white_to_move), expected)
                    self.assertEqual(extract.comment_cp_white(colosseum, white_to_move), expected)

    def test_blend_probability_scales(self):
        import math
        for cp in (-2000.0, -175.0, 0.0, 40.0, 2000.0):
            with self.subTest(cp=cp):
                self.assertEqual(extract.blend_probability(cp, None), extract.sigmoid_cp(cp))
                self.assertAlmostEqual(extract.blend_probability(cp, math.log(10.0)),
                                       extract.sigmoid_cp(cp), places=12)
        # The tuner's K is flatter than the Elo scale: 200 cp is worth less.
        self.assertAlmostEqual(extract.blend_probability(200.0, 1.6166),
                               1.0 / (1.0 + math.exp(-1.6166 * 0.5)), places=12)
        self.assertLess(extract.blend_probability(200.0, 1.6166), extract.sigmoid_cp(200.0))
        self.assertEqual(extract.blend_probability(0.0, 1.6166), 0.5)

    def test_comments_without_a_score_give_none(self):
        for comment in ("", "book", "{forfeit}", "d=6 t=10ms n=7456", "hs=12 d=3"):
            with self.subTest(comment=comment):
                self.assertIsNone(extract.comment_cp_white(comment, True))

    def test_parallel_provenance_accepts_colosseum_schema(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "games.pgn"
            path.write_text("test pgn\n", encoding="utf-8")
            manifest = {
                "schema": "rarog-colosseum-datagen-v1",
                "output": {
                    "path": str(path.resolve()),
                    "bytes": path.stat().st_size,
                    "sha256": extract.sha256_file(path),
                },
                "engine": {"sha256": "engine", "git_sha": "commit", "git_dirty": False},
                "book": {"sha256": "book", "seed": 42, "start": 1, "end": 1},
                "games": 1,
                "nodes_per_move": 8000,
                "adjudication": {"Name": "datagen-v2"},
            }
            path.with_suffix(".manifest.json").write_text(
                json.dumps(manifest), encoding="utf-8"
            )
            records, contract = extract_parallel.load_datagen_provenance([path])
            self.assertEqual(records[0]["book_end"], 1)
            self.assertEqual(contract["book_seed"], 42)


class OpeningSequenceTests(unittest.TestCase):
    BOOK = {
        "k7/8/8/8/8/8/8/K7 w - -": "k7/8/8/8/8/8/8/K7 w - -",
        "k7/8/8/8/8/8/8/1K6 w - -": "k7/8/8/8/8/8/8/1K6 w - -",
        "k7/8/8/8/8/8/8/2K5 b - -": "k7/8/8/8/8/8/8/2K5 b - -",
    }

    def test_sources_merge_by_offset_and_stop_at_the_first_gap(self):
        keys = list(self.BOOK)
        pilot = {1: keys[2]}
        continuation = {2: keys[0], 3: keys[1], 5: keys[0]}
        sequence = opening_sequence.build_sequence([pilot, continuation], self.BOOK)
        self.assertEqual(sequence, [self.BOOK[keys[2]], self.BOOK[keys[0]], self.BOOK[keys[1]]])

    def test_disagreeing_sources_are_refused(self):
        keys = list(self.BOOK)
        with self.assertRaises(SystemExit):
            opening_sequence.build_sequence([{1: keys[0]}, {1: keys[1]}], self.BOOK)

    def test_an_opening_outside_the_book_is_refused(self):
        with self.assertRaises(SystemExit):
            opening_sequence.build_sequence([{1: "8/8/8/8/8/8/8/k6K w - -"}], self.BOOK)

    def test_a_pgn_fen_maps_to_its_book_line(self):
        self.assertEqual(
            opening_sequence.position_key("k7/8/8/8/8/8/8/K7 w - - 0 1"),
            "k7/8/8/8/8/8/8/K7 w - -",
        )

    def test_rounds_read_from_tag_lines(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "games.pgn"
            path.write_text(
                '[Round "2"]\n[FEN "k7/8/8/8/8/8/8/K7 w - - 0 1"]\n\n1. Kb1 *\n\n'
                '[Round "1"]\n[FEN "k7/8/8/8/8/8/8/1K6 w - - 0 1"]\n\n1. Kc1 *\n',
                encoding="utf-8",
            )
            self.assertEqual(
                opening_sequence.read_rounds(path),
                {2: "k7/8/8/8/8/8/8/K7 w - -", 1: "k7/8/8/8/8/8/8/1K6 w - -"},
            )


if __name__ == "__main__":
    unittest.main()
