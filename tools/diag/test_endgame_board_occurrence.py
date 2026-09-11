"""Tests for endgame_board_occurrence (PLAN 4.11.10).

The predicates are the whole tool -- everything downstream is counting -- so
they are tested against positions rather than against each other. The
regression tests at the bottom guard the two mistakes this measurement has
already made once: a piece-count threshold that silently hides a family, and a
"zero" that means "not looked at" rather than "not there".
"""

import json
import sqlite3
import tempfile
import unittest
from pathlib import Path

import chess

import endgame_board_occurrence as occ


def families(fen: str) -> set[str]:
    return occ.families_at(chess.Board(fen))


class Predicates(unittest.TestCase):
    def test_exact_material_families(self):
        cases = {
            "8/8/8/4k3/8/1r6/4PK2/4R3 w - - 0 1": "KRPKR",
            "8/8/8/4k3/4p3/8/5K2/4R3 w - - 0 1": "KRKP",
            "8/8/8/8/8/5k2/5p2/4RK2 w - - 0 1": "KRKP",
            "8/8/8/3nk3/8/8/5K2/4R3 w - - 0 1": "KRKN",
            "8/8/8/3bk3/8/8/5K2/4R3 w - - 0 1": "KRKB",
            "8/8/8/3bk3/8/8/4PK2/4B3 w - - 0 1": "KBPKB",
            "8/8/8/3nk3/8/8/4PK2/4B3 w - - 0 1": "KBPKN",
            "8/8/8/3pk3/8/8/4PK2/8 w - - 0 1": "KPKP",
            "8/8/8/3pk3/8/8/5K2/3Q4 w - - 0 1": "KQKP",
            "8/8/8/3rk3/8/8/5K2/3Q4 w - - 0 1": "KQKR",
            "8/8/8/3pk3/8/8/2N1KN2/8 w - - 0 1": "KNNKP",
        }
        for fen, name in cases.items():
            with self.subTest(fen=fen):
                self.assertIn(name, families(fen))

    def test_lone_king_families_and_their_intended_overlap(self):
        # KPsK contains KPK by design, and KXK contains both: three functions,
        # three dispatch conditions, one position. The shares deliberately do
        # not sum to one. That KXK includes a pawn-only strong side is the
        # variant that reproduces RAR-M15 (see the module's KXK note) -- it is
        # pinned here because it is a choice, not an inevitability.
        one_pawn = families("8/8/8/4k3/8/8/4PK2/8 w - - 0 1")
        self.assertEqual({"KXK", "KPsK", "KPK"} & one_pawn, {"KXK", "KPsK", "KPK"})

        two_pawns = families("8/8/8/4k3/8/8/3PPK2/8 w - - 0 1")
        self.assertIn("KPsK", two_pawns)
        self.assertNotIn("KPK", two_pawns)

        self.assertIn("KBPsK", families("8/8/8/4k3/8/8/4PK2/4B3 w - - 0 1"))
        self.assertIn("KBNK", families("8/8/8/4k3/8/8/4KN2/4B3 w - - 0 1"))
        self.assertIn("KNNK", families("8/8/8/4k3/8/8/2N1KN2/8 w - - 0 1"))

    def test_kxk_excludes_what_cannot_mate(self):
        # 2.81 pp of RAR-M15's games turn on this line, so it is pinned.
        self.assertNotIn("KXK", families("8/8/8/4k3/8/8/4KN2/8 w - - 0 1"))
        self.assertNotIn("KXK", families("8/8/8/4k3/8/8/4K3/4B3 w - - 0 1"))
        self.assertNotIn("KXK", families("8/8/8/4k3/8/8/2N1KN2/8 w - - 0 1"))
        self.assertIn("KXK", families("8/8/8/4k3/8/8/5K2/4R3 w - - 0 1"))
        self.assertIn("KXK", families("8/8/8/4k3/8/8/5K2/3Q4 w - - 0 1"))
        self.assertEqual(set(), families("8/8/8/4k3/8/8/5K2/8 w - - 0 1"))

    def test_colour_is_symmetric(self):
        white_strong = families("8/8/8/4k3/8/8/4PK2/4R3 w - - 0 1")
        black_strong = families("4r3/4pk2/8/8/4K3/8/8/8 w - - 0 1")
        self.assertEqual(white_strong, black_strong)


class ThePositionsRARM15CouldNotSee(unittest.TestCase):
    """The two families RAR-M15 published as zero, from its own games.

    Both FENs were taken out of `sprt_HCERefit_vs_HCEBase_20260901_072106.pgn`
    -- the corpus RAR-M15 measured. A "zero" that a real position from the very
    same games contradicts is not a small sample; it is a blind instrument, and
    the ranking floored three families with the rule of three as though the
    only problem were sample size.
    """

    def test_krppkrp_is_seven_men_and_was_outside_the_old_six_man_cap(self):
        board = chess.Board("8/1r3p2/8/7P/8/4kPK1/1R6/8 b - - 0 66")
        self.assertEqual(chess.popcount(board.occupied), 7)
        self.assertIn("KRPPKRP", occ.families_at(board))

    def test_kqkr_is_four_men_so_the_cap_never_explained_its_zero(self):
        board = chess.Board("8/8/R6K/8/8/7k/8/5q2 w - - 0 79")
        self.assertEqual(chess.popcount(board.occupied), 4)
        self.assertIn("KQKR", occ.families_at(board))


class NoHiddenThreshold(unittest.TestCase):
    """The `<= 8 men` fast path must be a bound, never a cut-off.

    PLAN 4.11.5's occurrence result moved sharply when a "generous" threshold
    moved by one man. The guarantee here is structural: any position the fast
    path skips matches nothing anyway.
    """

    def test_every_bounded_family_fits_inside_the_fast_path(self):
        unbounded = {"KXK", "KPsK", "KBPsK"}
        for name, predicate in occ.PREDICATES.items():
            if name in unbounded:
                continue
            with self.subTest(family=name):
                # Reconstruct the widest material the predicate accepts by
                # searching short material strings, then require it to fit.
                widest = 0
                for strong in _material_strings(4):
                    for weak in _material_strings(3):
                        if predicate(strong, weak):
                            widest = max(widest, 2 + len(strong) + len(weak))
                self.assertTrue(0 < widest <= 8, f"{name} widest={widest}")

    def test_the_fast_path_skips_only_positions_that_match_nothing(self):
        # A 12-man middlegame: skipped, and correctly so.
        self.assertEqual(
            set(),
            families("4k3/pppp4/8/8/8/8/PPPP4/4K3 w - - 0 1"))
        # A lone king facing a full army: NOT skipped, despite 16 men.
        crowded = chess.Board("4k3/8/8/8/8/8/PPPPPPPP/RNBQKBNR w - - 0 1")
        self.assertGreater(chess.popcount(crowded.occupied), 8)
        self.assertIn("KXK", occ.families_at(crowded))


def _material_strings(max_len: int) -> list[str]:
    """Every material string in Q,R,B,N,P order up to `max_len` pieces."""
    out = [""]
    frontier = [""]
    for _ in range(max_len):
        nxt = []
        for base in frontier:
            for letter in "QRBNP":
                if base and "QRBNP".index(letter) < "QRBNP".index(base[-1]):
                    continue
                nxt.append(base + letter)
        out.extend(nxt)
        frontier = nxt
    return out


class Aggregation(unittest.TestCase):
    def test_shard_is_a_fixed_index_round_robin_that_loses_nothing(self):
        items = list(range(23))
        buckets = occ.shard(items, 5)
        self.assertEqual(sorted(x for b in buckets for x in b), items)
        self.assertEqual(occ.shard(items, 1), [items])

    def test_summarize_counts_games_not_positions(self):
        records = [
            {"families": ["KRKP", "KXK"], "fewest_men": 4,
             "white": "Rarog 2.3.2", "black": "Basilisk"},
            {"families": ["KXK"], "fewest_men": 3,
             "white": "Fruit", "black": "HIARCS"},
            {"families": [], "fewest_men": 20,
             "white": "Rarog 2.3.2", "black": "Fruit"},
        ]
        summary = occ.summarize(records, "Rarog")
        self.assertEqual(summary["games"], 3)
        self.assertEqual(summary["all"]["KXK"]["games"], 2)
        self.assertAlmostEqual(summary["all"]["KXK"]["share"], 2 / 3)
        self.assertEqual(summary["engine_games"], 2)
        self.assertEqual(summary["engine"]["KXK"]["games"], 1)
        self.assertAlmostEqual(summary["engine"]["KRKP"]["share"], 0.5)
        self.assertAlmostEqual(summary["reaches_6_men"], 2 / 3)
        self.assertAlmostEqual(summary["reaches_7_men"], 2 / 3)

    def test_summarize_reports_every_family_including_the_absent_ones(self):
        summary = occ.summarize(
            [{"families": [], "fewest_men": 32, "white": "a", "black": "b"}], None)
        self.assertEqual(set(summary["all"]), set(occ.PREDICATES))
        self.assertEqual(summary["all"]["KQKR"], {"games": 0, "share": 0.0})


class Calibration(unittest.TestCase):
    def test_a_matching_run_is_clean(self):
        summary = {"all": {n: {"share": v} for n, v in occ.M15.items()}}
        self.assertEqual(occ.calibrate(summary), ([], []))

    def test_a_known_difference_is_explained_not_failed(self):
        shares = dict(occ.M15)
        shares["KRPPKRP"] = 0.1009
        unexplained, explained = occ.calibrate(
            {"all": {n: {"share": v} for n, v in shares.items()}})
        self.assertEqual(unexplained, [])
        self.assertEqual(len(explained), 1)
        self.assertIn("KRPPKRP", explained[0])

    def test_an_unrecorded_difference_fails(self):
        shares = dict(occ.M15)
        shares["KRKP"] = 0.10
        unexplained, _ = occ.calibrate(
            {"all": {n: {"share": v} for n, v in shares.items()}})
        self.assertEqual(len(unexplained), 1)
        self.assertIn("KRKP", unexplained[0])

    def test_every_exception_names_a_family_that_exists(self):
        self.assertLessEqual(set(occ.CALIBRATION_EXCEPTIONS), set(occ.PREDICATES))


class Sources(unittest.TestCase):
    HEADER = ('[Event "t"]\n[White "{w}"]\n[Black "{b}"]\n'
              '[Result "{r}"]\n\n{moves} {r}\n\n')

    def _pgn(self, games) -> str:
        return "".join(self.HEADER.format(**g) for g in games)

    def test_pgn_split_returns_one_text_per_game(self):
        text = self._pgn([
            {"w": "A", "b": "B", "r": "1-0", "moves": "1. e4 e5"},
            {"w": "C", "b": "D", "r": "0-1", "moves": "1. d4 d5"},
        ])
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "g.pgn"
            path.write_text(text, encoding="utf-8")
            games = occ.games_from_pgn(path)
        self.assertEqual(len(games), 2)
        self.assertIn('[White "A"]', games[0])
        self.assertIn('[White "C"]', games[1])

    def test_sqlite_selects_by_name_or_by_id(self):
        with tempfile.TemporaryDirectory() as tmp:
            db = Path(tmp) / "c.sqlite"
            conn = sqlite3.connect(db)
            conn.execute("create table tournaments (id text, name text)")
            conn.execute("create table games (tournament_id text, pgn text)")
            conn.execute("insert into tournaments values ('tid', 'Rating')")
            conn.executemany("insert into games values (?, ?)",
                             [("tid", "pgn-a"), ("tid", None), ("other", "pgn-c")])
            conn.commit()
            conn.close()
            self.assertEqual(occ.games_from_sqlite(db, "Rating"), ["pgn-a"])
            self.assertEqual(occ.games_from_sqlite(db, "tid"), ["pgn-a"])
            self.assertEqual(occ.games_from_sqlite(db, "missing"), [])

    def test_scan_game_reads_a_setup_fen_rather_than_the_start_position(self):
        pgn = ('[Event "t"]\n[White "Rarog"]\n[Black "X"]\n[Result "1-0"]\n'
               '[SetUp "1"]\n[FEN "8/8/8/4k3/8/8/5K2/4R3 w - - 0 1"]\n\n1-0\n')
        record = occ.scan_game(pgn)
        self.assertIn("KXK", record["families"])
        self.assertEqual(record["fewest_men"], 3)
        self.assertEqual(record["white"], "Rarog")


class TheRegisteredOrderMatchesItsTrackedInputs(unittest.TestCase):
    """4.12's order must be what the stored artifacts actually produce.

    `endgame_ranking_v2.json` decides the order of twenty engine changes, and
    its corpus is a 117 MB Colosseum database in `AppData` that this repo will
    never contain. A frozen order whose inputs cannot be reconstructed is a
    promise that someone else is still storing the evidence, so the inputs are
    tracked and this test re-derives the order from them.
    """

    DIAG = Path(__file__).resolve().parent

    def _rank(self, board_path, scope):
        import endgame_ranking as rank

        def read(name):
            return json.loads((self.DIAG / name).read_text(encoding="utf-8"))

        shares, _, corpus = rank.board_occurrence(board_path, scope)
        return [r["function"] for r in rank.build(
            read("endgame_reference_results_v1.json"),
            read("endgame_drawn_census_v1.json"),
            read("endgame_tree_occurrence_v1.json"),
            shares, corpus)]

    def test_v2_reproduces_from_the_tracked_occurrence_artifact(self):
        registered = json.loads(
            (self.DIAG / "endgame_ranking_v2.json").read_text(encoding="utf-8"))
        order = self._rank(self.DIAG / "endgame_board_occurrence_v1.json", "engine")
        self.assertEqual(order, registered["order"])

    def test_v1_reproduces_too_so_the_correction_stays_visible(self):
        # Both orders must be derivable, not just the current one: v1 is what
        # 4.12 was numbered by until 4.11.12, and the difference between them
        # is the finding.
        registered = json.loads(
            (self.DIAG / "endgame_ranking_v1.json").read_text(encoding="utf-8"))
        self.assertEqual(self._rank(None, "engine"), registered["order"])
        self.assertNotEqual(registered["order"], json.loads(
            (self.DIAG / "endgame_ranking_v2.json")
            .read_text(encoding="utf-8"))["order"])

    def test_every_input_the_ranking_defaults_to_is_tracked_beside_it(self):
        # The corpora behind these are a 14 MB PGN and a 117 MB database that
        # this repo will never hold, so the DERIVED artifacts have to be here.
        for name in ("endgame_reference_results_v1.json",
                     "endgame_drawn_census_v1.json",
                     "endgame_tree_occurrence_v1.json",
                     "endgame_board_occurrence_v1.json",
                     "endgame_board_occurrence_m15_replay.json"):
            with self.subTest(artifact=name):
                self.assertTrue((self.DIAG / name).is_file(), name)

    def test_v1_still_reproduces_from_the_fallback_constants(self):
        import endgame_ranking as rank

        shares, provenance, corpus = rank.board_occurrence(None, "engine")
        self.assertEqual(shares, rank.M15_BOARD_OCCURRENCE)
        self.assertEqual(corpus, rank.DEFAULT_CORPUS_GAMES)
        self.assertIn("RAR-M15", provenance)

    def test_a_foreign_schema_is_refused_rather_than_read(self):
        import endgame_ranking as rank

        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "wrong.json"
            path.write_text(json.dumps({"schema": "something-else"}),
                            encoding="utf-8")
            with self.assertRaises(SystemExit):
                rank.board_occurrence(path, "all")


class ArtifactShape(unittest.TestCase):
    def test_the_frozen_artifact_carries_its_schema_and_source(self):
        summary = occ.summarize(
            [{"families": ["KXK"], "fewest_men": 3, "white": "a", "black": "b"}],
            None)
        summary["source"] = "somewhere.pgn"
        doc = json.loads(json.dumps(summary))
        self.assertEqual(doc["schema"], occ.SCHEMA)
        self.assertIn("source", doc)
        self.assertIn("games", doc)


if __name__ == "__main__":
    unittest.main()
