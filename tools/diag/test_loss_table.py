#!/usr/bin/env python3
"""Tests for loss_table.py: comments and clocks are read as Colosseum writes
them, the sample takes losses, spoiled draws and their twins, every move's
loss is the judge's value before it minus after it from the mover's side,
the losses telescope to the result, a resumed judge run drops a partial
game, and the cells and verdicts follow their definitions."""

from __future__ import annotations

import sys
from pathlib import Path

import chess
import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent))

import loss_table as lt  # noqa: E402

TC = "white 3000ms+30ms; black 3000ms+30ms"
SHUFFLE = ["Nf3", "Nf6", "Nc3", "Nc6", "Ng1", "Ng8", "Nb1", "Nb8", "Nf3", "Nf6", "Nc3", "Nc6"]
FOOL = ["f3", "e5", "g4", "Qh4#"]


def pgn(number, white, black, result, sans, scores, pair=1, pair_game=1, ms=40):
    """A game as Colosseum's CLI writes it."""
    text = [f'[Event "Colosseum CLI tournament"]', f'[White "{white}"]', f'[Black "{black}"]',
            f'[Result "{result}"]', f'[TimeControl "{TC}"]', '[SetUp "1"]',
            f'[FEN "{chess.STARTING_FEN}"]', f'[GameNumber "{number}"]',
            f'[PairNumber "{pair}"]', f'[PairGame "{pair_game}"]', ""]
    moves = []
    for i, (san, score) in enumerate(zip(sans, scores)):
        prefix = f"{i // 2 + 1}. " if i % 2 == 0 else ""
        moves.append(f"{prefix}{san} {{s={score} d=12 t={ms}ms h=1ms n={1000 + i}}}")
    text.append(" ".join(moves) + f" {result}")
    return "\n".join(text) + "\n\n"


def write_run(tmp_path, name, games) -> Path:
    run = tmp_path / name
    run.mkdir()
    (run / "games.pgn").write_text("".join(games), encoding="utf-8")
    return run


def test_comments_are_read_as_written():
    assert lt.parse_comment("s=-90 d=12 t=51ms h=1ms n=63287") == ("-90", 63287, 51)
    assert lt.parse_comment("s=#-3 d=9 t=0ms h=0ms n=11") == ("#-3", 11, 0)
    assert lt.parse_comment("t=7ms") == (None, None, 7)
    with pytest.raises(ValueError):
        lt.parse_comment("s=abc t=1ms")
    with pytest.raises(ValueError):
        lt.parse_comment("s=1 t=1s")
    assert lt.score_value("#2") == float("inf") and lt.score_value("#-2") == float("-inf")


def test_time_controls_and_clocks():
    assert lt.parse_time_control(TC) == {"w": (3000, 30), "b": (3000, 30)}
    assert lt.parse_time_control("180+0.03") == {"w": (180000, 30), "b": (180000, 30)}
    with pytest.raises(ValueError):
        lt.parse_time_control("40/60")
    plies = tuple(lt.Ply(m, "0", 1, ms) for m, ms in zip(["e2e4", "e7e5", "g1f3"], [100, 200, 300]))
    game = lt.Game("r", 1, 1, 1, "A", "B", "1-0", chess.STARTING_FEN,
                   lt.parse_time_control(TC), plies)
    # White: 3000, then 3000 - 100 + 30; black: 3000.
    assert lt.clocks(game) == [3000, 3000, 2930]


def read_one(tmp_path, text) -> lt.Game:
    path = tmp_path / "one.pgn"
    path.write_text(text, encoding="utf-8")
    return next(lt.scan(path, "run"))


def test_a_spoiled_draw_needs_both_engines_for_consecutive_moves(tmp_path):
    ahead = ["60", "-60"] * 6
    game = read_one(tmp_path, pgn(1, "E", "O", "1/2-1/2", SHUFFLE, ahead))
    assert lt.spoiled(game, "w", 50, 5)
    assert not lt.spoiled(game, "b", 50, 5)
    # The opponent disagrees once in the middle: no run of ten plies.
    broken = ahead[:5] + ["10"] + ahead[6:]
    game = read_one(tmp_path, pgn(1, "E", "O", "1/2-1/2", SHUFFLE, broken))
    assert not lt.spoiled(game, "w", 50, 5)
    game = read_one(tmp_path, pgn(1, "E", "O", "1-0", SHUFFLE, ahead))
    assert not lt.spoiled(game, "w", 50, 5)


def test_the_sample_takes_losses_spoiled_draws_and_their_twins(tmp_path):
    level = ["0"] * 12
    ahead = ["60", "-60"] * 6
    run = write_run(tmp_path, "pool", [
        pgn(1, "E", "O", "0-1", FOOL, ["0"] * 4, pair=1, pair_game=1),
        pgn(2, "O", "E", "1-0", FOOL, ["0"] * 4, pair=1, pair_game=2),
        pgn(3, "E", "O", "1/2-1/2", SHUFFLE, ahead, pair=2, pair_game=1),
        pgn(4, "O", "E", "1/2-1/2", SHUFFLE, level, pair=2, pair_game=2),
        pgn(5, "E", "O", "1/2-1/2", SHUFFLE, level, pair=3, pair_game=1),
        pgn(6, "O", "E", "1/2-1/2", SHUFFLE, level, pair=3, pair_game=2),
        pgn(7, "O", "P", "1-0", FOOL, ["0"] * 4, pair=4, pair_game=1),
    ])
    out = tmp_path / "sample.tsv"
    assert lt.main(["sample", "--run", str(run), "--engine", "E", "--out", str(out)]) == 0
    sample = lt.read_sample(out)
    reasons = {int(r["game"]): r["reason"] for r in sample.rows}
    assert reasons == {1: "loss", 2: "loss", 3: "spoiled", 4: "twin"}
    assert sample.engine == "E" and "pool" in sample.runs
    games = lt.sample_games(sample)
    assert [g.number for g, _ in games] == [1, 2, 3, 4]
    (run / "games.pgn").write_text("changed", encoding="utf-8")
    with pytest.raises(SystemExit):
        lt.sample_games(sample)


def fake_evaluate(fen, moves):
    """A deterministic value for the side to move that moves with the game."""
    board = chess.Board(fen)
    for move in moves:
        board.push_uci(move)
    es = 0.5 + 0.04 * (len(moves) % 5) - 0.01 * len(moves)
    return lt.Eval(round((es - 0.5) * 400), round(es * 2000) / 2000, None)


def test_every_loss_is_before_minus_after_and_the_losses_telescope(tmp_path):
    game = read_one(tmp_path, pgn(1, "O", "E", "0-1", FOOL, ["0", "0", "0", "#1"]))
    records = lt.judge_game(game, "loss", "E", fake_evaluate)
    assert [r["mover"] for r in records] == ["opponent", "engine"] * 2
    assert [r["men"] for r in records] == [32, 32, 32, 32]
    for k, r in enumerate(records):
        before = fake_evaluate(chess.STARTING_FEN, [p.uci for p in game.plies[:k]])
        assert r["es_before"] == round(before.es, 4)
        assert r["loss_es"] == pytest.approx(r["es_before"] - r["es_after"], abs=1e-9)
    # The last move mates: the rule scores it, the judge is not asked.
    assert records[-1]["es_after"] == 1.0 and records[-1]["cp_after"] == lt.MATE_CP
    net, swing = lt.identity(records)
    assert net == pytest.approx(swing, abs=1e-9)
    start = fake_evaluate(chess.STARTING_FEN, []).es
    assert swing == pytest.approx((1 - start) - 1.0, abs=1e-9)


def test_a_drawn_final_position_is_scored_by_the_rule():
    board = chess.Board("8/8/8/8/8/2k5/8/K7 w - - 0 1")
    assert lt.terminal(board) == lt.Eval(0, 0.5, None, True)
    board = chess.Board("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1")
    assert lt.terminal(board) == lt.Eval(0, 0.5, None, True)
    board = chess.Board("7k/8/6K1/8/8/8/8/5Q2 b - - 0 1")
    assert lt.terminal(board) is None
    board = chess.Board("7k/6Q1/6K1/8/8/8/8/8 b - - 0 1")
    assert lt.terminal(board) == lt.Eval(-lt.MATE_CP, 0.0, None, True)


def test_the_search_reading_takes_the_last_exact_principal_score():
    lines = ["info depth 10 multipv 1 score cp 20 wdl 100 800 100 nodes 10 pv e2e4",
             "info depth 11 multipv 2 score cp 90 wdl 900 100 0 nodes 20 pv d2d4",
             "info depth 11 multipv 1 score cp 35 lowerbound wdl 200 700 100 nodes 30 pv e2e4",
             "bestmove e2e4 ponder e7e5"]
    assert lt.parse_search(lines, True) == lt.Eval(20, 0.5, "e2e4")
    lines = ["info depth 9 score mate -3 wdl 0 0 1000", "bestmove h7h8"]
    assert lt.parse_search(lines, True) == lt.Eval(-(lt.MATE_CP - 3), 0.0, "h7h8")
    with pytest.raises(RuntimeError):
        lt.parse_search(["info depth 9 score cp 5", "bestmove a2a3"], True)
    assert lt.parse_search(["info depth 9 score cp 5", "bestmove a2a3"], False).cp == 5


def test_a_resumed_run_keeps_finished_games_and_drops_a_partial_one(tmp_path):
    header = [lt.JUDGED_MAGIC, "# sample s sha256 1", "# nodes 10 hash 1 threads 1"]
    out = tmp_path / "judged.tsv"
    assert lt.resume(out, header) == set()
    with open(out, "a", encoding="utf-8") as f:
        f.write("a\n# game pool 1 plies 1\nb\nc\n")
    assert lt.resume(out, header) == {("pool", 1)}
    assert out.read_text(encoding="utf-8").splitlines()[-2:] == ["a", "# game pool 1 plies 1"]
    with pytest.raises(SystemExit):
        lt.resume(out, header[:2] + ["# nodes 20 hash 1 threads 1"])


def record(mover, cp_before, clock_frac, men, loss_cp, loss_es, game=1, ply=0):
    return {"run": "r", "game": game, "ply": ply, "mover": mover, "name": mover,
            "opponent": "O", "reason": "loss", "cp_before": cp_before,
            "clock_frac": clock_frac, "men": men, "loss_cp": loss_cp, "loss_es": loss_es,
            "es_before": 0.5, "es_after": 0.5 - loss_es}


def test_cells_follow_their_bands_and_the_cp_loss_is_capped():
    assert lt.cell(record("engine", -50, 0.5, 9, 0, 0), "class", 50) == (0, "defending")
    assert lt.cell(record("engine", -49, 0.5, 9, 0, 0), "class", 50) == (1, "level")
    assert lt.cell(record("engine", 50, 0.5, 9, 0, 0), "class", 50) == (2, "ahead")
    assert lt.cell(record("engine", 0, 0.0999, 9, 0, 0), "clock", 50)[1] == "<10%"
    assert lt.cell(record("engine", 0, 0.10, 9, 0, 0), "clock", 50)[1] == "10-25%"
    assert lt.cell(record("engine", 0, 0.50, 9, 0, 0), "clock", 50)[1] == ">=50%"
    assert lt.cell(record("engine", 0, 0.5, 9, 0, 0), "men", 50)[1] == "<=9"
    assert lt.cell(record("engine", 0, 0.5, 10, 0, 0), "men", 50)[1] == "10-15"
    assert lt.cell(record("engine", 0, 0.5, 24, 0, 0), "men", 50)[1] == ">=24"
    rows = lt.tabulate([record("engine", -100, 0.05, 20, 900, 0.3),
                        record("engine", -100, 0.05, 20, 40, 0.1, ply=2),
                        record("engine", 0, 0.8, 20, 0, 0.0, ply=4),
                        record("opponent", 0, 0.8, 20, 60, 0.2, ply=1)], ["class", "clock"])
    first = rows[0]
    assert (first["side"], first["class"], first["clock"]) == ("engine", "defending", "<10%")
    assert first["moves"] == 2 and first["errors"] == 1
    assert first["mean_cp"] == pytest.approx((300 + 40) / 2)
    assert first["point_share"] == pytest.approx(1.0)
    assert rows[-1]["side"] == "opponent" and rows[-1]["point_share"] == pytest.approx(1.0)


def test_verdicts_name_where_an_error_goes():
    assert lt.verdict(0.1, 0.05, {1: 0.3, 4: 0.3, 16: 0.3}) == "unconfirmed"
    assert lt.verdict(0.1, 0.2, {1: 0.0, 4: 0.3, 16: 0.3}) == "not_reproduced"
    assert lt.verdict(0.1, 0.2, {1: 0.2, 4: 0.05, 16: 0.3}) == "clock_4x"
    assert lt.verdict(0.1, 0.2, {1: 0.2, 4: 0.2, 16: 0.0}) == "clock_16x"
    assert lt.verdict(0.1, 0.2, {1: 0.2, 4: 0.2, 16: 0.2}) == "persists"
