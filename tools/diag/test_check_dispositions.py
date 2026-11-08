"""Tests for the disposition check: the entry line, the index cell, the
exemption list, and that a disagreement or a missing side fails."""

import pathlib
import sys
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import check_dispositions as cd  # noqa: E402

HEADER = "| ID | Experiment | Disposition |\n|---|---|---|\n"


def row(name, cell):
    return f"| [{name}](experiments/{name}.md) | a title | {cell} |\n"


class EntryLine(unittest.TestCase):
    def test_the_word_after_the_field(self):
        self.assertEqual(cd.entry_disposition("x\n- Disposition: rejected, by H0\n"), "rejected")
        self.assertEqual(cd.entry_disposition("- Disposition: no-change\n"), "no-change")

    def test_prose_or_an_unknown_word_is_not_the_form(self):
        self.assertIsNone(cd.entry_disposition("- Disposition: accepted / retained / rejected\n"))
        self.assertIsNone(cd.entry_disposition("- Disposition: rejectedly\n"))
        self.assertIsNone(cd.entry_disposition("**Disposition.** rejected\n"))

    def test_two_lines_are_reported(self):
        self.assertEqual(cd.entry_disposition("- Disposition: open\n- Disposition: accepted\n"),
                         ["open", "accepted"])


class IndexCell(unittest.TestCase):
    def test_the_word_a_cell_starts_with(self):
        index = HEADER + row("RAR-S1", "rejected: H0 at 4,000 games") + row("RAR-S2", "Played and lost")
        self.assertEqual(cd.index_dispositions(index), {"RAR-S1": ["rejected"], "RAR-S2": [None]})

    def test_a_row_whose_link_names_another_file_is_not_its_row(self):
        index = HEADER + "| [RAR-S1](experiments/RAR-S2.md) | t | open |\n"
        self.assertEqual(cd.index_dispositions(index), {})


class Problems(unittest.TestCase):
    def test_a_matching_new_entry_passes(self):
        entries = {"RAR-S9": "- Disposition: accepted\n"}
        self.assertEqual(cd.problems(entries, HEADER + row("RAR-S9", "accepted"), set()), [])

    def test_a_disagreement_fails(self):
        entries = {"RAR-S9": "- Disposition: accepted\n"}
        found = cd.problems(entries, HEADER + row("RAR-S9", "rejected — H0"), set())
        self.assertEqual(found, ["RAR-S9: the entry says accepted, the index row rejected"])

    def test_a_missing_side_fails(self):
        self.assertEqual(len(cd.problems({"RAR-S9": "prose\n"}, HEADER + row("RAR-S9", "open"), set())), 1)
        self.assertEqual(len(cd.problems({"RAR-S9": "- Disposition: open\n"}, HEADER + row("RAR-S9", "Ran"), set())), 1)
        self.assertEqual(cd.problems({"RAR-S9": "- Disposition: open\n"}, HEADER, set()), ["RAR-S9: no index row"])

    def test_an_exempt_entry_passes_in_prose_and_fails_once_it_carries_the_form(self):
        self.assertEqual(cd.problems({"RAR-S1": "prose\n"}, HEADER + row("RAR-S1", "Ran"), {"RAR-S1"}), [])
        found = cd.problems({"RAR-S1": "- Disposition: open\n"}, HEADER + row("RAR-S1", "open"), {"RAR-S1"})
        self.assertIn("remove it from", found[0])

    def test_an_exempt_id_without_an_entry_fails(self):
        self.assertEqual(cd.problems({}, HEADER, {"RAR-S1"}), ["RAR-S1: exempt but has no entry file"])


class Exempt(unittest.TestCase):
    def test_the_list_is_read_strictly(self):
        self.assertEqual(cd.load_exempt("# frozen\nRAR-S1\n\nRAR-M2\n"), {"RAR-S1", "RAR-M2"})
        with self.assertRaises(ValueError):
            cd.load_exempt("RAR-S1\nRAR-S1\n")
        with self.assertRaises(ValueError):
            cd.load_exempt("RAR-S1 rejected\n")


class Repository(unittest.TestCase):
    def test_the_ledger_passes(self):
        self.assertEqual(cd.main(), 0)


if __name__ == "__main__":
    unittest.main()
