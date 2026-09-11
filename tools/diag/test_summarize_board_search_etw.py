#!/usr/bin/env python3
"""Focused tests for summarize_board_search_etw.py."""

from __future__ import annotations

import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).with_name("summarize_board_search_etw.py")
SPEC = importlib.util.spec_from_file_location("summarize_board_search_etw", SCRIPT)
assert SPEC and SPEC.loader
MODULE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = MODULE
SPEC.loader.exec_module(MODULE)


class ParserTests(unittest.TestCase):
    def test_extracts_process_total_and_exclusive_rvas(self) -> None:
        report = """<html><body>
<h2>Processes and Root functions</h2><table><tr><th>process</th><th>pid</th>
<th>exclusive hits</th></tr><tr><td>rarog.exe</td><td>42</td><td>100</td></tr></table>
<h2>Functions by Exclusive Hits</h2><table>
<tr><th>function</th><th>hits</th><th>percent</th><th>a</th><th>b</th><th>address</th></tr>
<tr><td>rarog.exe!***unknown***</td><td>70</td><td>70%</td><td>x</td><td>x</td><td>0x123</td></tr>
<tr><td>kernel.dll!***unknown***</td><td>30</td><td>30%</td><td>x</td><td>x</td><td>0x456</td></tr>
</table></body></html>"""
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "profile.txt"
            path.write_text(report, encoding="utf-8")
            total, hits = MODULE.parse_report(path, "rarog.exe")
        self.assertEqual(total, 100)
        self.assertEqual(hits, [MODULE.ExclusiveHit(rva=0x123, hits=70)])


class ClassificationTests(unittest.TestCase):
    def test_inline_consumer_wins_over_shared_helper(self) -> None:
        functions = [
            "rarog::board::AttackTables::bishop",
            "rarog::board::see_recapturer",
        ]
        self.assertEqual(MODULE.classify(functions), "see")

    def test_generation_wins_over_check_helper(self) -> None:
        functions = [
            "rarog::board::Board::is_attacked_with_occ",
            "rarog::board::movegen::generate_king_moves",
        ]
        self.assertEqual(MODULE.classify(functions), "generation_and_legality")

    def test_make_wins_over_checker_update(self) -> None:
        functions = [
            "rarog::board::Board::calculate_checkers",
            "rarog::board::Board::make_move_inner",
        ]
        self.assertEqual(MODULE.classify(functions), "make_unmake")

    def test_mechanisms_are_intentionally_overlapping(self) -> None:
        functions = [
            "rarog::board::Board::king_sq",
            "rarog::board::movegen::compute_pinned",
        ]
        self.assertEqual(
            MODULE.mechanisms(functions), ["compute_pinned", "king_square_lookup"]
        )


if __name__ == "__main__":
    unittest.main()
