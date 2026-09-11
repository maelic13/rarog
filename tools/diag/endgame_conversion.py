#!/usr/bin/env python3
"""Measure deterministic bare-king conversion with one persistent TT per game."""

from __future__ import annotations

import argparse
import json
import random
import statistics
import sys
from collections import Counter
from pathlib import Path

import chess
import chess.engine


FAMILIES = {
    "KQ-K": (chess.QUEEN,),
    "KR-K": (chess.ROOK,),
    "KBB-K": (chess.BISHOP, chess.BISHOP),
    "KBN-K": (chess.BISHOP, chess.KNIGHT),
}


def random_position(rng: random.Random, pieces: tuple[int, ...]) -> chess.Board:
    for _ in range(10_000):
        squares = rng.sample(range(64), 2 + len(pieces))
        board = chess.Board(None)
        board.turn = chess.WHITE
        board.set_piece_at(squares[0], chess.Piece(chess.KING, chess.WHITE))
        for square, piece in zip(squares[1:-1], pieces):
            board.set_piece_at(square, chess.Piece(piece, chess.WHITE))
        board.set_piece_at(squares[-1], chess.Piece(chess.KING, chess.BLACK))
        if pieces == (chess.BISHOP, chess.BISHOP):
            first, second = squares[1], squares[2]
            if (chess.square_rank(first) + chess.square_file(first)) % 2 == (
                chess.square_rank(second) + chess.square_file(second)
            ) % 2:
                continue
        if board.is_valid() and not board.is_check() and any(board.legal_moves):
            return board
    raise RuntimeError(f"could not generate legal position for {pieces}")


def play_one(
    engine: chess.engine.SimpleEngine,
    board: chess.Board,
    nodes: int,
    max_plies: int,
    game_token: object,
) -> tuple[str, int]:
    initial_material = len(board.piece_map())
    for ply in range(max_plies):
        if board.is_checkmate():
            return ("mated" if board.turn == chess.BLACK else "wrong_mate", ply)
        if board.is_stalemate():
            return "stalemate", ply
        if board.is_fifty_moves() or board.can_claim_fifty_moves():
            return "fifty_move", ply
        if len(board.piece_map()) < initial_material:
            return "material_lost", ply
        result = engine.play(board, chess.engine.Limit(nodes=nodes), game=game_token)
        if result.move is None:
            return "no_move", ply
        board.push(result.move)
    return "ply_limit", max_plies


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--engine", required=True, type=Path)
    parser.add_argument("--positions", type=int, default=100)
    parser.add_argument("--nodes", type=int, default=60_000)
    parser.add_argument("--max-plies", type=int, default=100)
    parser.add_argument("--seed", type=int, default=0x5E9D18)
    parser.add_argument("--families", default=",".join(FAMILIES))
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    if min(args.positions, args.nodes, args.max_plies) <= 0:
        parser.error("positions, nodes and max-plies must be positive")
    engine_path = args.engine.resolve()
    if not engine_path.is_file():
        parser.error(f"engine not found: {engine_path}")
    families = args.families.split(",")
    unknown = [family for family in families if family not in FAMILIES]
    if unknown:
        parser.error(f"unknown families: {', '.join(unknown)}")

    report = {
        "schema": "rarog-endgame-conversion-v1",
        # LAYER 3 ONLY (analysis/endgame_measurement_layers.md). One bit per
        # game: did it mate inside the budget. This instrument sees no
        # tablebase, so it cannot report layer 1 or 2 and must not be read as
        # evidence about either -- `material_lost` here means "the bare-king
        # side has nothing left to mate with", not "the win was thrown".
        "layer": "3_conversion",
        "layer_note": (
            "bare-king families only; no tablebase, so no theory truth and no "
            "move quality. Never reports Elo."
        ),
        "engine": str(engine_path),
        "positions_per_family": args.positions,
        "nodes_per_move": args.nodes,
        "max_plies": args.max_plies,
        "seed": args.seed,
        "persistent_tt_per_game": True,
        "families": {},
    }
    engine = chess.engine.SimpleEngine.popen_uci(str(engine_path))
    try:
        available = engine.options
        options = {}
        if "Hash" in available:
            options["Hash"] = 16
        if "Threads" in available:
            options["Threads"] = 1
        if options:
            engine.configure(options)
        for family_index, family in enumerate(families):
            rng = random.Random(args.seed ^ ((family_index + 1) * 0x9E3779B1))
            outcomes: Counter[str] = Counter()
            mate_plies = []
            for index in range(args.positions):
                board = random_position(rng, FAMILIES[family])
                outcome, plies = play_one(engine, board, args.nodes, args.max_plies, object())
                outcomes[outcome] += 1
                if outcome == "mated":
                    mate_plies.append(plies)
                if (index + 1) % 10 == 0:
                    print(
                        f"{family}: {index + 1}/{args.positions} "
                        f"mated={outcomes['mated']} fifty={outcomes['fifty_move']} "
                        f"stalemate={outcomes['stalemate']} other="
                        f"{index + 1 - outcomes['mated'] - outcomes['fifty_move'] - outcomes['stalemate']}",
                        flush=True,
                    )
            report["families"][family] = {
                "outcomes": dict(outcomes),
                "conversion_rate": outcomes["mated"] / args.positions,
                "median_mate_plies": statistics.median(mate_plies) if mate_plies else None,
            }
    finally:
        engine.quit()

    rendered = json.dumps(report, indent=2) + "\n"
    if args.output:
        args.output.write_text(rendered, encoding="utf-8", newline="\n")
        print(f"Report: {args.output.resolve()}")
    else:
        print(rendered)
    return 0


if __name__ == "__main__":
    sys.exit(main())
