"""Tests for the citation check: what counts as a citation, how a cited hash is
judged, and that the foreign-identifier list is read strictly."""

import pathlib
import sys
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import check_citations as cc  # noqa: E402

LIVE = "1abeb46aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
DANGLING = "bff5fbfbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"


def resolve(token):
    return {"1abeb46": LIVE, "bff5fbf": DANGLING}.get(token)


class Citations(unittest.TestCase):
    def test_backticked_hex_with_or_without_an_ellipsis(self):
        text = "landed in `1abeb46`; arm `bff5fbf…`\nplain 1abeb46 and `12345678`"
        self.assertEqual(cc.citations(text), [(1, "1abeb46", False), (1, "bff5fbf", True)])

    def test_short_or_uppercase_hex_is_not_a_citation(self):
        self.assertEqual(cc.citations("`abc123` `B2F65630` `ABCDEF0`"), [])


class Classify(unittest.TestCase):
    def test_a_reachable_commit_is_fine(self):
        self.assertIsNone(cc.classify("1abeb46", resolve, {LIVE}, {}))

    def test_a_commit_on_no_kept_ref_is_a_problem(self):
        self.assertIn("no kept ref", cc.classify("bff5fbf", resolve, {LIVE}, {}))

    def test_a_listed_foreign_identifier_is_fine_and_an_unknown_one_is_not(self):
        foreign = {"9587eeeb": "Stockfish commit"}
        self.assertIsNone(cc.classify("9587eeeb", resolve, {LIVE}, foreign))
        self.assertIn("not a commit here", cc.classify("0ddc8e5", resolve, {LIVE}, foreign))

    def test_a_truncated_digest_is_fine_but_a_truncated_dead_commit_is_not(self):
        self.assertIsNone(cc.classify("aac92114", resolve, {LIVE}, {}, truncated=True))
        self.assertIsNotNone(cc.classify("bff5fbf", resolve, {LIVE}, {}, truncated=True))
        self.assertIsNotNone(cc.classify("aac92114", resolve, {LIVE}, {}, truncated=False))

    def test_a_foreign_entry_never_excuses_a_dangling_rarog_commit(self):
        self.assertIsNotNone(cc.classify("bff5fbf", resolve, {LIVE}, {"bff5fbf": "x"}))


class Foreign(unittest.TestCase):
    def test_entries_and_comments(self):
        text = "# comment\n9587eeeb\tStockfish commit\r\n\n"
        self.assertEqual(cc.load_foreign(text), {"9587eeeb": "Stockfish commit"})

    def test_a_missing_description_or_a_duplicate_is_refused(self):
        with self.assertRaises(ValueError):
            cc.load_foreign("9587eeeb\t\n")
        with self.assertRaises(ValueError):
            cc.load_foreign("9587eeeb\ta\n9587eeeb\tb\n")

    def test_the_committed_list_parses(self):
        self.assertGreater(len(cc.load_foreign(cc.FOREIGN.read_text(encoding="utf-8"))), 0)


if __name__ == "__main__":
    unittest.main()
