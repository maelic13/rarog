#!/usr/bin/env python3
"""Tests for conversion_audit.py.

No game data is committed for these. Every game is synthesised in memory from a
FEN plus a short move list, which keeps the repository free of PGN archives and
lets the tests run on a machine that has never held a tournament.
"""

from __future__ import annotations

import io
import sys
from pathlib import Path

import chess
import chess.pgn
import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent))

from conversion_audit import (  # noqa: E402
    audit,
    classify_game,
    is_lone_minor,
    material,
    non_pawn_material,
    outcome_for,
)


def make_game(fen: str, moves: list[str], white: str, black: str, result: str,
              termination: str = "normal"):
    """Build a game in memory. `moves` are UCI, played from `fen`."""
    board = chess.Board(fen)
    game = chess.pgn.Game()
    game.setup(board)
    node = game
    for uci in moves:
        move = chess.Move.from_uci(uci)
        assert move in board.legal_moves, f"{uci} illegal in {board.fen()}"
        board.push(move)
        node = node.add_variation(move)
    game.headers["White"] = white
    game.headers["Black"] = black
    game.headers["Result"] = result
    game.headers["Termination"] = termination
    return game


def shuffle_moves(fen: str, plies: int) -> list[str]:
    """A legal do-nothing sequence, long enough to satisfy the persistence window."""
    board = chess.Board(fen)
    moves = []
    for _ in range(plies):
        move = next(iter(board.legal_moves))
        moves.append(move.uci())
        board.push(move)
    return moves


# --- pure helpers -----------------------------------------------------------


def test_material_and_non_pawn_material():
    board = chess.Board()
    assert material(board, chess.WHITE) == material(board, chess.BLACK)
    # Eight pawns, two knights, two bishops, two rooks, one queen.
    assert material(board, chess.WHITE) == 8 + 6 + 6 + 10 + 9
    assert non_pawn_material(board) == 2 * (6 + 6 + 10 + 9)


@pytest.mark.parametrize(
    "fen,expected",
    [
        ("8/8/8/4k3/8/8/8/4K1B1 w - - 0 1", True),   # KB v K: dead draw
        ("8/8/8/4k3/8/8/8/4K1N1 w - - 0 1", True),   # KN v K: dead draw
        ("8/8/8/4k3/8/8/8/4KBN1 w - - 0 1", False),  # KBN v K: winnable
        ("8/8/8/4k3/8/8/8/4KBB1 w - - 0 1", False),  # KBB v K: winnable
        ("8/8/8/4k3/8/8/8/4K1R1 w - - 0 1", False),  # rook is not a minor
        ("8/8/8/4k3/8/8/4P3/4K1B1 w - - 0 1", False),  # a pawn changes everything
        ("8/8/8/3bk3/8/8/8/4K1B1 w - - 0 1", False),  # opponent is not bare
    ],
)
def test_lone_minor_exclusion_is_by_material_signature(fen, expected):
    assert is_lone_minor(chess.Board(fen), chess.WHITE) is expected


@pytest.mark.parametrize(
    "result,played_white,expected",
    [
        ("1-0", True, "W"), ("1-0", False, "L"),
        ("0-1", True, "L"), ("0-1", False, "W"),
        ("1/2-1/2", True, "D"), ("1/2-1/2", False, "D"),
        ("*", True, "?"),
    ],
)
def test_outcome_is_from_the_named_engine_point_of_view(result, played_white, expected):
    assert outcome_for(result, played_white) == expected


# --- classification ---------------------------------------------------------


ROOK_UP = "8/8/4k3/8/8/4K3/8/R7 w - - 0 1"
LONE_BISHOP = "8/8/4k3/8/8/4K3/8/2B5 w - - 0 1"


def test_a_draw_while_a_rook_up_is_a_conversion_failure():
    game = make_game(ROOK_UP, shuffle_moves(ROOK_UP, 20), "Us", "Them", "1/2-1/2")
    verdict = classify_game(game, "Us")
    assert verdict["persistent_advantage"] is True
    assert verdict["outcome"] == "D"


def test_a_lone_bishop_draw_is_excluded_by_default_and_counted_without_the_flag():
    game = make_game(LONE_BISHOP, shuffle_moves(LONE_BISHOP, 20), "Us", "Them", "1/2-1/2")
    assert classify_game(game, "Us", exclude_lone_minor=True)["persistent_advantage"] is False
    # The seed arithmetic that produced the 57/12 baseline had no exclusion.
    assert classify_game(game, "Us", exclude_lone_minor=False)["persistent_advantage"] is True


def test_an_advantage_shorter_than_the_window_does_not_count():
    short = shuffle_moves(ROOK_UP, PERSIST_SHORT := 6)
    assert PERSIST_SHORT < 12
    game = make_game(ROOK_UP, short, "Us", "Them", "1/2-1/2")
    assert classify_game(game, "Us")["persistent_advantage"] is False


def test_colour_is_taken_from_the_headers_not_assumed():
    game = make_game(ROOK_UP, shuffle_moves(ROOK_UP, 20), "Them", "Us", "1/2-1/2")
    verdict = classify_game(game, "Us")
    # "Us" is Black here and is a rook DOWN, so this is a save, not a failure.
    assert verdict["persistent_advantage"] is False
    assert verdict["persistent_deficit"] is True
    assert verdict["opponent"] == "Them"


def test_a_game_the_engine_did_not_play_is_ignored():
    game = make_game(ROOK_UP, shuffle_moves(ROOK_UP, 20), "A", "B", "1/2-1/2")
    assert classify_game(game, "Us") is None


# --- aggregation ------------------------------------------------------------


def test_audit_counts_and_filters_by_opponent():
    games = [
        make_game(ROOK_UP, shuffle_moves(ROOK_UP, 20), "Us", "Strong", "1/2-1/2", "adjudication"),
        make_game(ROOK_UP, shuffle_moves(ROOK_UP, 20), "Us", "Strong", "0-1"),
        make_game(ROOK_UP, shuffle_moves(ROOK_UP, 20), "Us", "Weak", "1/2-1/2"),
    ]
    everything = audit(iter(games), "Us", None, True)
    assert everything["counts"]["games"] == 3
    assert everything["counts"]["draw_after_persistent_advantage"] == 2
    assert everything["counts"]["loss_after_persistent_advantage"] == 1
    assert everything["thrown_away"] == 3
    assert everything["draw_terminations"] == {"adjudication": 1, "normal": 1}

    filtered = audit(iter(games), "Us", {"Strong"}, True)
    assert filtered["counts"]["games"] == 2
    assert filtered["thrown_away"] == 2


def test_summary_records_the_parameters_it_used():
    summary = audit(iter([]), "Us", None, False)
    assert summary["parameters"]["exclude_lone_minor"] is False
    assert summary["parameters"]["persist_plies"] == 12
    assert summary["parameters"]["min_advantage"] == 3


def test_round_trips_through_real_pgn_text():
    """The tool reads PGN files, so prove it survives serialisation."""
    game = make_game(ROOK_UP, shuffle_moves(ROOK_UP, 20), "Us", "Them", "1/2-1/2")
    reparsed = chess.pgn.read_game(io.StringIO(str(game)))
    assert classify_game(reparsed, "Us")["persistent_advantage"] is True


if __name__ == "__main__":
    sys.exit(pytest.main([__file__, "-v"]))
